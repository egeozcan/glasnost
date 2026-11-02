//! The "no stock numbers" guard (PLAN.md 3.5, CONTRIBUTING.md "No EA bytes,
//! anywhere").
//!
//! Stock values cannot be recognised without knowing them, and they must
//! never be known to this repository. So the guard enforces the rules that
//! keep them out structurally, over every file git tracks or would add
//! (`git ls-files --cached --others --exclude-standard`):
//!
//! 1. Files in the game's formats (by extension, including `.ini`) may only
//!    live in fixture directories. In `tests/fixtures/synthetic/` they must
//!    be byte-identical to what our fixture writers generate; in any other
//!    fixture directory they must be text carrying the `glasnost-synthetic:`
//!    marker.
//! 2. Everything under `data/synthetic/` starts with the CC0 SPDX line and
//!    the `glasnost-synthetic:` marker.
//! 3. Nothing under `measurements/local/` is tracked, and measurement
//!    records with `value-visibility: local` show no numbers in the value and
//!    spread columns of their Result table. Labels there, such as `run 1`,
//!    `Procedure step 4` or `E1`, are not numbers.
//! 4. No real-install or home-directory paths in text files, and no
//!    `LOCAL.md` or `.claude/settings.local.json`.
//! 5. No file larger than 1 MiB.
//!
//! Comparing synthetic data against the user's own imported values needs the
//! converter's (field, value) output; that local-only check arrives with the
//! converter in M1 (see docs/development.md).

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use glasnost_fmt::fixture::{self, MANIFEST_FILE};

use crate::workspace;

/// Extensions of the game's file formats (lowercase).
pub const GAME_FORMAT_EXTENSIONS: &[&str] = &[
    "mix", "shp", "vxl", "hva", "pal", "idx", "bag", "aud", "csf", "bik", "map", "mpr", "yrm",
    "exe", "dll", "ini", "tmp", "pcx", "fnt", "wav",
];

/// Where committed binary fixtures from our writers live.
pub const CANONICAL_FIXTURE_DIR: &str = "tests/fixtures/synthetic/";

/// Marker that every synthetic data or text fixture file carries.
pub const SYNTHETIC_MARKER: &str = "glasnost-synthetic:";

const MAX_FILE_BYTES: u64 = 1 << 20;

/// This file names the path patterns it looks for, so it is not scanned.
const SELF: &str = "xtask/src/stock_guard.rs";

/// Case-insensitive substrings that indicate a real install or a personal
/// machine path.
fn path_patterns() -> Vec<String> {
    let home = ["/", "home", "/"].concat();
    vec![
        ["steamapps", "/common"].concat(),
        ["steamapps", "\\common"].concat(),
        ["program", " files"].concat(),
        ["origin", " games"].concat(),
        ["ea", " games"].concat(),
        ["westwood", "/"].concat(),
        ["westwood", "\\"].concat(),
        ["c:\\", "users\\"].concat(),
        ["/", "users/"].concat(),
        ["/var", &home].concat(),
        home,
    ]
}

fn tracked_files(root: &Path) -> Result<Vec<String>, String> {
    let out = Command::new("git")
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ])
        .current_dir(root)
        .output()
        .map_err(|e| format!("git ls-files: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git ls-files failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let mut files: Vec<String> = out
        .stdout
        .split(|&b| b == 0)
        .filter(|p| !p.is_empty())
        .map(|p| String::from_utf8_lossy(p).into_owned())
        .filter(|p| root.join(p).is_file())
        .collect();
    files.sort();
    files.dedup();
    Ok(files)
}

fn extension(path: &str) -> String {
    Path::new(path)
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default()
}

fn is_fixture_path(path: &str) -> bool {
    path.starts_with("tests/fixtures/")
        || path.contains("/tests/fixtures/")
        || path.contains("fixtures/synthetic/")
}

fn is_text(bytes: &[u8]) -> bool {
    !bytes[..bytes.len().min(8192)].contains(&0) && std::str::from_utf8(bytes).is_ok()
}

fn has_synthetic_marker(text: &str) -> bool {
    text.lines().take(5).any(|l| l.contains(SYNTHETIC_MARKER))
}

/// Checks one file's contents against rules 1, 2, 4 and 5. `generated` maps
/// canonical fixture paths to the bytes our writers produce.
pub fn check_file(path: &str, bytes: &[u8], generated: &BTreeMap<String, Vec<u8>>) -> Vec<String> {
    let mut out = Vec::new();
    let ext = extension(path);
    let text = is_text(bytes).then(|| String::from_utf8_lossy(bytes));

    if bytes.len() as u64 > MAX_FILE_BYTES {
        out.push(format!(
            "{path}: larger than 1 MiB; large files are not committed"
        ));
    }
    if path == "LOCAL.md" || path.ends_with(".claude/settings.local.json") {
        out.push(format!("{path}: machine-local file must stay untracked"));
    }
    if path.starts_with("measurements/local/") {
        out.push(format!(
            "{path}: local measurement values are never committed"
        ));
    }

    if let Some(rel) = path.strip_prefix(CANONICAL_FIXTURE_DIR) {
        let is_doc = rel == MANIFEST_FILE || rel == "README.md";
        if !is_doc && generated.get(rel).is_none_or(|g| g.as_slice() != bytes) {
            out.push(format!(
                "{path}: not reproduced byte for byte by our fixture writers; regenerate with \
                 `cargo run -p glasnost-tools --bin glasnost-fixtures`"
            ));
        }
    } else if is_fixture_path(path) {
        if !text.as_deref().is_some_and(has_synthetic_marker) {
            out.push(format!(
                "{path}: fixture files outside {CANONICAL_FIXTURE_DIR} must be text with a \
                 `{SYNTHETIC_MARKER}` line in their first five lines; binary fixtures come from \
                 our writers"
            ));
        }
    } else if GAME_FORMAT_EXTENSIONS.contains(&ext.as_str()) {
        out.push(format!(
            "{path}: game-format file (.{ext}) outside a fixture directory"
        ));
    }

    if path.starts_with("data/synthetic/") {
        let ok = text.as_deref().is_some_and(|t| {
            let mut lines = t.lines();
            lines
                .next()
                .is_some_and(|l| l.ends_with("SPDX-License-Identifier: CC0-1.0"))
                && lines.next().is_some_and(|l| l.contains(SYNTHETIC_MARKER))
        });
        if !ok {
            out.push(format!(
                "{path}: synthetic data must start with the `SPDX-License-Identifier: CC0-1.0` \
                 line and then a `{SYNTHETIC_MARKER}` line"
            ));
        }
    }

    if path != SELF
        && let Some(t) = &text
    {
        let lower = t.to_lowercase();
        for pattern in path_patterns() {
            // Absolute paths must start a path, not continue a URL
            // (`https://example.org/home/` is fine).
            let starts_path = |at: usize| {
                !pattern.starts_with('/')
                    || !lower[..at]
                        .chars()
                        .next_back()
                        .is_some_and(|c| c.is_alphanumeric() || "./-_".contains(c))
            };
            if let Some((at, _)) = lower
                .match_indices(&pattern)
                .find(|(at, _)| starts_path(*at))
            {
                let line = lower[..at].matches('\n').count() + 1;
                out.push(format!(
                    "{path}:{line}: looks like a real install or home-directory path ({pattern:?}); \
                     use `~` or a placeholder, and keep machine details in LOCAL.md"
                ));
            }
        }
    }
    out
}

/// Rule 3 for one measurement record.
pub fn check_measurement(path: &str, text: &str) -> Vec<String> {
    let local = text.lines().take_while(|l| !l.starts_with("# ")).any(|l| {
        l.strip_prefix("value-visibility:")
            .is_some_and(|v| v.split('#').next().unwrap_or("").trim() == "local")
    });
    if !local {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut in_result = false;
    let mut columns: Vec<usize> = Vec::new();
    let mut seen_separator = false;
    for (n, line) in text.lines().enumerate() {
        if line.starts_with("## ") {
            in_result = line.trim() == "## Result";
            columns.clear();
            seen_separator = false;
            continue;
        }
        if !in_result {
            continue;
        }
        if !line.trim_start().starts_with('|') {
            // A line outside a table ends it: the next table's own header row
            // names its columns.
            columns.clear();
            seen_separator = false;
            continue;
        }
        let cells: Vec<&str> = line
            .trim()
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        if columns.is_empty() && !seen_separator {
            columns = cells
                .iter()
                .enumerate()
                .filter(|(_, c)| {
                    let c = c.to_lowercase();
                    c.contains("value") || c.contains("spread")
                })
                .map(|(i, _)| i)
                .collect();
            continue;
        }
        if cells
            .iter()
            .all(|c| c.chars().all(|ch| ch == '-' || ch == ':'))
        {
            seen_separator = true;
            continue;
        }
        for &i in &columns {
            if cells
                .get(i)
                .is_some_and(|c| without_labels(c).chars().any(|ch| ch.is_ascii_digit()))
            {
                out.push(format!(
                    "{path}:{}: value-visibility is local, but the Result table shows a number; \
                     put the value in measurements/local/ instead",
                    n + 1
                ));
            }
        }
    }
    out
}

/// Words whose following number names a run, step or seed of the record's
/// own procedure, not a measured value. The plural forms may take a list
/// ("attempts 1 and 3", "launches 1, 2, 3").
const LABEL_WORDS: &[&str] = &["run", "step", "attempt", "seed", "launch"];
const LABEL_WORDS_PLURAL: &[&str] = &["runs", "steps", "attempts", "seeds", "launches"];

/// `cell` with its labels blanked out, so that only numbers that may be
/// values remain. Labels are names that carry a digit (`E1`, `P3`,
/// `special-1`, `S-0202`) and the numbers right after a label word
/// (`run 1`, `Procedure step 4`, `attempts 1 and 3`). A number written in
/// groups (`seed 39 557 213`) is a value, not a label.
fn without_labels(cell: &str) -> String {
    // Tokens: runs of ASCII letters and digits, with inner hyphens.
    let mut tokens: Vec<(usize, usize)> = Vec::new();
    let mut start = None;
    for (i, ch) in cell.char_indices().chain([(cell.len(), ' ')]) {
        let part = ch.is_ascii_alphanumeric() || (ch == '-' && start.is_some());
        match (start, part) {
            (None, true) => start = Some(i),
            (Some(s), false) => {
                let end = s + cell[s..i].trim_end_matches('-').len();
                tokens.push((s, end));
                start = None;
            }
            _ => {}
        }
    }
    let text = |t: (usize, usize)| &cell[t.0..t.1];
    let gap = |a: (usize, usize), b: (usize, usize)| cell[a.1..b.0].trim();
    let is_number = |t: (usize, usize)| {
        let w = text(t);
        w.starts_with(|c: char| c.is_ascii_digit())
            && w.chars().all(|c| c.is_ascii_digit() || c == '-')
    };

    let mut blank = vec![false; tokens.len()];
    for (k, &t) in tokens.iter().enumerate() {
        let w = text(t);
        if w.starts_with(|c: char| c.is_ascii_alphabetic()) && w.chars().any(|c| c.is_ascii_digit())
        {
            blank[k] = true;
            continue;
        }
        let lower = w.to_ascii_lowercase();
        let plural = LABEL_WORDS_PLURAL.contains(&lower.as_str());
        if !plural && !LABEL_WORDS.contains(&lower.as_str()) {
            continue;
        }
        let mut last = k;
        let mut taken = Vec::new();
        while let Some(&t) = tokens.get(last + 1) {
            let mut num = last + 1;
            if !taken.is_empty() {
                // A list goes on after a comma, a dash, "and" or "to".
                let sep = gap(tokens[last], t);
                if !plural {
                    break;
                }
                if sep.is_empty() && matches!(text(t), "and" | "to") {
                    num += 1;
                } else if !matches!(sep, "," | "–") {
                    break;
                }
            }
            let Some(&n) = tokens.get(num) else { break };
            let space_only = taken.is_empty() || num > last + 1;
            if !is_number(n) || (space_only && !gap(tokens[num - 1], n).is_empty()) {
                break;
            }
            taken.push(num);
            last = num;
        }
        let grouped = tokens
            .get(last + 1)
            .is_some_and(|&n| !taken.is_empty() && is_number(n) && gap(tokens[last], n).is_empty());
        if !grouped {
            for j in taken {
                blank[j] = true;
            }
        }
    }

    let mut out = cell.to_owned();
    for (k, &(s, e)) in tokens.iter().enumerate() {
        if blank[k] {
            out.replace_range(s..e, &" ".repeat(e - s));
        }
    }
    out
}

/// `cargo xtask stock-guard`.
pub fn run() -> Result<Vec<String>, String> {
    let root = workspace::root();
    let generated: BTreeMap<String, Vec<u8>> = fixture::generate(&fixture::registry())
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|f| (f.path, f.bytes))
        .collect();
    let files = tracked_files(&root)?;
    let mut out = Vec::new();
    for path in &files {
        let bytes = std::fs::read(root.join(path)).map_err(|e| format!("{path}: {e}"))?;
        out.extend(check_file(path, &bytes, &generated));
        if path.starts_with("measurements/") && extension(path) == "md" {
            out.extend(check_measurement(path, &String::from_utf8_lossy(&bytes)));
        }
    }

    // The canonical fixture directory, if present, is complete and its
    // manifest matches the writers.
    let manifest_path = format!("{CANONICAL_FIXTURE_DIR}{MANIFEST_FILE}");
    let fixture_files: Vec<_> = files
        .iter()
        .filter(|p| p.starts_with(CANONICAL_FIXTURE_DIR))
        .collect();
    if !fixture_files.is_empty() {
        let fixtures = fixture::generate(&fixture::registry()).map_err(|e| e.to_string())?;
        match std::fs::read_to_string(root.join(&manifest_path)) {
            Ok(m) if m == fixture::manifest(&fixtures) => {}
            Ok(_) => out.push(format!(
                "{manifest_path}: out of date; regenerate the fixtures"
            )),
            Err(_) => out.push(format!("{manifest_path}: missing; regenerate the fixtures")),
        }
        for f in &fixtures {
            let p = format!("{CANONICAL_FIXTURE_DIR}{}", f.path);
            if !files.contains(&p) {
                out.push(format!("{p}: generated by our writers but not committed"));
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(path: &str, bytes: &[u8]) -> Vec<String> {
        check_file(
            path,
            bytes,
            &BTreeMap::from([("pal/a.pal".to_owned(), vec![1, 2, 3])]),
        )
    }

    #[test]
    fn game_formats_only_in_fixture_dirs() {
        assert_eq!(check("assets/units.shp", b"\x00\x01").len(), 1);
        assert_eq!(check("crates/x/rules.INI", b"[A]\nB=1\n").len(), 1);
        // Canonical fixtures must match the writers exactly.
        assert!(check("tests/fixtures/synthetic/pal/a.pal", &[1, 2, 3]).is_empty());
        assert_eq!(
            check("tests/fixtures/synthetic/pal/a.pal", &[1, 2, 4]).len(),
            1
        );
        assert_eq!(check("tests/fixtures/synthetic/pal/b.pal", &[1]).len(), 1);
        // Other fixture dirs: marked text only.
        assert!(
            check(
                "crates/ini/tests/fixtures/dupes.ini",
                b"; glasnost-synthetic: ours\n[A]\n"
            )
            .is_empty()
        );
        assert_eq!(
            check("crates/ini/tests/fixtures/dupes.ini", b"[A]\nB=1\n").len(),
            1
        );
        assert_eq!(
            check(
                "crates/fmt/tests/fixtures/x.shp",
                b"\x00glasnost-synthetic:"
            )
            .len(),
            1
        );
    }

    #[test]
    fn synthetic_data_needs_header() {
        let good = b"// SPDX-License-Identifier: CC0-1.0\n// glasnost-synthetic: invented\n{}\n";
        assert!(check("data/synthetic/ruleset/a.jsonc", good).is_empty());
        assert_eq!(check("data/synthetic/ruleset/a.jsonc", b"{}\n").len(), 1);
    }

    #[test]
    fn install_and_home_paths_are_rejected() {
        let steam = ["D:\\Steam\\steamapps", "\\common\\game\n"].concat();
        assert_eq!(check("docs/x.md", steam.as_bytes()).len(), 1);
        let home = ["see ", "/home", "/someone/glasnost\n"].concat();
        assert_eq!(check("docs/x.md", home.as_bytes()).len(), 1);
        assert!(check("docs/x.md", b"keep it in ~/glasnost-scrub\n").is_empty());
        assert!(check("docs/x.md", b"https://example.org/home/page\n").is_empty());
    }

    #[test]
    fn local_files_and_big_files_are_rejected() {
        assert_eq!(check("LOCAL.md", b"notes").len(), 1);
        assert_eq!(check("measurements/local/M-0001.json", b"{}").len(), 1);
        assert_eq!(
            check("docs/big.txt", &vec![b'a'; (MAX_FILE_BYTES + 1) as usize]).len(),
            1
        );
    }

    #[test]
    fn local_measurements_hide_values() {
        let record = |row: &str| {
            format!(
                "---\nid: M-0001\nvalue-visibility: local # kept local\n---\n\n# M-0001: x\n\n\
                 ## Result\n\n| Quantity | Unit | n | Mean / value | Spread | Tolerance for tests |\n\
                 |---|---|---|---|---|---|\n{row}\n\n## Notes\n| 5 | 6 |\n"
            )
        };
        assert!(
            check_measurement(
                "m.md",
                &record("| reload | ticks | 20 | local | local | ±1 tick |")
            )
            .is_empty()
        );
        assert_eq!(
            check_measurement("m.md", &record("| reload | ticks | 20 | 45 | 0 | ±1 |")).len(),
            2
        );
        let committed = record("| rate | ticks/s | 20 | 15 | 0 | 0 |")
            .replace("local # kept local", "committed");
        assert!(check_measurement("m.md", &committed).is_empty());
    }

    #[test]
    fn labels_in_value_cells_are_not_numbers() {
        let record = |value: &str| {
            format!(
                "---\nid: M-0001\nvalue-visibility: local\n---\n\n# M-0001: x\n\n\
                 ## Result\n\n| Quantity | Value |\n|---|---|\n| q | {value} |\n"
            )
        };
        for label in [
            "equal to the control in run 1 (lists local)",
            "as in Procedure step 4",
            "in A5, B3 and B5 (counts local)",
            "*special-1* starts; *active-1* stops",
            "see S-0202, attempts 1 and 3, launches 1–2 and 2–3, Runs 4, 5",
            "local in seed 0",
        ] {
            assert!(
                check_measurement("m.md", &record(label)).is_empty(),
                "{label}"
            );
        }
        for value in [
            "45",
            "run 1: 45",
            "56 in seed 0, 52 in seed 1",
            "seed 39 557 213",
            "launch 1: seed 39 556 602",
            "every 2-shot burst",
            "from 0 to 2 (Procedure step 4)",
        ] {
            assert_eq!(
                check_measurement("m.md", &record(value)).len(),
                1,
                "{value}"
            );
        }
    }

    #[test]
    fn each_result_table_is_read_with_its_own_header() {
        let text = "---\nid: M-0001\nvalue-visibility: local\n---\n\n# M-0001: x\n\n\
                    ## Result\n\n| Seed | Left |\n|---|---|\n| 0 | 3 of 9 |\n\n\
                    | Quantity | Value | Tolerance for tests |\n|---|---|---|\n\
                    | reload | local | range only |\n\n\
                    | Quantity | Seed 0 | Seed 1 |\n|---|---|---|\n| hits | 15 | 18 |\n";
        assert!(check_measurement("m.md", text).is_empty());
        let leaked = text.replace("| reload | local |", "| reload | 45 |");
        assert_eq!(check_measurement("m.md", &leaked).len(), 1);
    }
}
