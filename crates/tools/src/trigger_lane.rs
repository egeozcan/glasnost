//! The local lane's trigger inventory (M5b): which map trigger event and
//! action codes the install's maps use, to scope the multiplayer trigger
//! subset of S-0011. The report holds codes and counts only (no map names,
//! no values, no paths). Never in CI.
//!
//! Events and actions are split as S-0011's Import mapping says: items at
//! commas, empty items skipped; an event is three items plus a fourth when
//! its parameter kind is 2, an action eight items (the last action may lack
//! its eighth).

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use glasnost_ini::{Document, Quirks};

/// Per-code counts: (entries using the code, maps using it).
type Codes = BTreeMap<i64, (usize, usize)>;

/// The items of a value: split at commas, empty items skipped
/// (S-0011 Import mapping).
fn items(v: &[u8]) -> Vec<&[u8]> {
    v.split(|&b| b == b',')
        .map(|s| s.trim_ascii())
        .filter(|s| !s.is_empty())
        .collect()
}

fn int(s: &[u8]) -> Option<i64> {
    std::str::from_utf8(s).ok()?.trim().parse().ok()
}

/// The event codes of one `[Events]` value.
fn event_codes(v: &[u8]) -> Vec<i64> {
    let it = items(v);
    let Some(n) = it.first().and_then(|s| int(s)) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut i = 1;
    for _ in 0..n.max(0) {
        let (Some(code), Some(kind)) = (
            it.get(i).and_then(|s| int(s)),
            it.get(i + 1).and_then(|s| int(s)),
        ) else {
            break;
        };
        out.push(code);
        i += if kind == 2 { 4 } else { 3 };
    }
    out
}

/// The action codes of one `[Actions]` value.
fn action_codes(v: &[u8]) -> Vec<i64> {
    let it = items(v);
    let Some(n) = it.first().and_then(|s| int(s)) else {
        return Vec::new();
    };
    (0..n.max(0) as usize)
        .map_while(|k| it.get(1 + 8 * k).and_then(|s| int(s)))
        .collect()
}

/// Counts of one map.
#[derive(Default)]
struct MapCount {
    triggers: usize,
    tags: usize,
    teams: usize,
    scripts: usize,
    events: BTreeMap<i64, usize>,
    actions: BTreeMap<i64, usize>,
}

fn count(bytes: &[u8]) -> MapCount {
    let mut warnings = Vec::new();
    let doc = Document::parse(bytes, Quirks::default(), &mut warnings);
    let len = |s: &[u8]| doc.section(s).map_or(0, |s| s.entries().len());
    let mut m = MapCount {
        triggers: len(b"Triggers"),
        tags: len(b"Tags"),
        teams: len(b"TeamTypes"),
        scripts: len(b"ScriptTypes"),
        ..Default::default()
    };
    if let Some(s) = doc.section(b"Events") {
        for e in s.entries() {
            for c in event_codes(&e.value) {
                *m.events.entry(c).or_default() += 1;
            }
        }
    }
    if let Some(s) = doc.section(b"Actions") {
        for e in s.entries() {
            for c in action_codes(&e.value) {
                *m.actions.entry(c).or_default() += 1;
            }
        }
    }
    m
}

fn add(all: &mut Codes, one: &BTreeMap<i64, usize>) {
    for (&c, &n) in one {
        let e = all.entry(c).or_default();
        e.0 += n;
        e.1 += 1;
    }
}

fn codes_line(label: &str, codes: &BTreeMap<i64, usize>) -> String {
    let list: Vec<String> = codes.iter().map(|(c, n)| format!("{c}×{n}")).collect();
    format!("  {label}: {}", list.join(" "))
}

/// Counts the trigger codes of every map of the install; `detail` lists
/// the map indices whose own counts the report shows.
pub fn run(dir: &Path, detail: &[usize]) -> std::io::Result<String> {
    pollster::block_on(async {
        let vfs = crate::local_lane::mount(dir).await?;
        let listings = glasnost_convert::find::find_maps(&vfs).await;
        let mut events = Codes::new();
        let mut actions = Codes::new();
        let mut with_triggers = 0usize;
        let mut with_teams = 0usize;
        let mut details = String::new();
        let mut read = 0usize;
        for (index, listing) in listings.iter().enumerate() {
            let Ok(bytes) = vfs.read(&listing.file).await else {
                continue;
            };
            read += 1;
            let m = count(&bytes);
            if m.triggers > 0 {
                with_triggers += 1;
            }
            if m.teams > 0 || m.scripts > 0 {
                with_teams += 1;
            }
            add(&mut events, &m.events);
            add(&mut actions, &m.actions);
            if detail.contains(&index) {
                let _ = writeln!(
                    details,
                    "map index {index}: {} triggers, {} tags, {} team definitions, {} script definitions",
                    m.triggers, m.tags, m.teams, m.scripts
                );
                let _ = writeln!(details, "{}", codes_line("event codes", &m.events));
                let _ = writeln!(details, "{}", codes_line("action codes", &m.actions));
            }
        }
        let mut out = String::new();
        let _ = writeln!(
            out,
            "glasnost-headless triggers: {read} maps read, {with_triggers} with triggers, {with_teams} with team or script definitions"
        );
        let fmt = |codes: &Codes| {
            codes
                .iter()
                .map(|(c, (n, maps))| format!("{c}: {n} in {maps} maps"))
                .collect::<Vec<_>>()
                .join("; ")
        };
        let _ = writeln!(out, "event codes: {}", fmt(&events));
        let _ = writeln!(out, "action codes: {}", fmt(&actions));
        out.push_str(&details);
        Ok(out)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: S-0011 (Import mapping: items at commas, empty items skipped,
    // a fourth event item for kind 2, eight items per action)
    #[test]
    fn splits_events_and_actions() {
        assert_eq!(event_codes(b"2,13,0,30,60,2,1,TYPE"), vec![13, 60]);
        assert_eq!(event_codes(b"2,,13,0,30,8,0,0"), vec![13, 8]);
        assert_eq!(
            action_codes(b"2,14,0,3,0,0,0,0,A,53,2,T,0,0,0,0"),
            vec![14, 53]
        );
    }
}
