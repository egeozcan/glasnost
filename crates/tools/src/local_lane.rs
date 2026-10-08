//! The local-only lane: parses the user's real install and reports counts.
//!
//! Opt-in: it runs only when `GLASNOST_GAME_DIR` names an install folder,
//! never in CI, and nothing it reads or produces is committed. It opens
//! only the MIX containers at the top of that folder (by extension); every
//! other file there is counted by name and never opened. The report holds
//! counts and error kinds only: no names from inside archives, no bytes, no
//! paths.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;
use std::rc::Rc;

use glasnost_vfs::source::FileSource;
use glasnost_vfs::{MountOptions, PickedFile, Vfs};

/// Environment variable naming the install folder.
pub const GAME_DIR_VAR: &str = "GLASNOST_GAME_DIR";

/// Counts of successes and failures by error kind.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tally {
    /// Items that parsed or decoded.
    pub ok: u64,
    /// Failures by error kind (see `glasnost_fmt::Error::kind`).
    pub failed: BTreeMap<String, u64>,
}

impl Tally {
    /// Records a success.
    pub fn pass(&mut self) {
        self.ok += 1;
    }

    /// Records a failure of the given kind.
    pub fn fail(&mut self, kind: &str) {
        *self.failed.entry(kind.to_owned()).or_default() += 1;
    }

    /// Total attempts.
    pub fn total(&self) -> u64 {
        self.ok + self.failed.values().sum::<u64>()
    }

    /// Whether every attempt succeeded.
    pub fn all_ok(&self) -> bool {
        self.failed.is_empty()
    }

    fn describe(&self) -> String {
        let pct = if self.total() == 0 {
            "n/a".to_owned()
        } else {
            // Integer per-mille, so the report needs no floats.
            let pm = self.ok * 1000 / self.total();
            format!("{}.{}%", pm / 10, pm % 10)
        };
        let mut s = format!("{}/{} ({pct})", self.ok, self.total());
        for (kind, n) in &self.failed {
            let _ = write!(s, ", {kind}: {n}");
        }
        s
    }
}

/// The lane's findings.
#[derive(Clone, Debug, Default)]
pub struct Report {
    /// Ordered (label, tally) lines.
    pub tallies: Vec<(String, Tally)>,
    /// Ordered (label, count) lines for plain counts.
    pub counts: Vec<(String, u64)>,
}

impl Report {
    fn tally(&mut self, label: &str) -> &mut Tally {
        if let Some(i) = self.tallies.iter().position(|(l, _)| l == label) {
            &mut self.tallies[i].1
        } else {
            self.tallies.push((label.to_owned(), Tally::default()));
            &mut self.tallies.last_mut().expect("just pushed").1
        }
    }

    fn count(&mut self, label: &str, n: u64) {
        self.counts.push((label.to_owned(), n));
    }

    /// The tally with this label, if present.
    pub fn get(&self, label: &str) -> Option<&Tally> {
        self.tallies
            .iter()
            .find(|(l, _)| l == label)
            .map(|(_, t)| t)
    }
}

impl std::fmt::Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "local lane report (counts only)")?;
        for (label, n) in &self.counts {
            writeln!(f, "  {label}: {n}")?;
        }
        for (label, t) in &self.tallies {
            writeln!(f, "  {label}: {}", t.describe())?;
        }
        Ok(())
    }
}

/// Labels of the M1 headline numbers.
pub mod labels {
    /// Encrypted MIX headers decrypted to a consistent index.
    pub const MIX_DECRYPTED: &str = "MIX headers decrypted (top-level and nested)";
    /// Top-level archives mounted.
    pub const MIX_TOP: &str = "top-level MIX containers mounted";
}

/// Runs the lane over the install at `dir`.
pub fn run(dir: &Path) -> std::io::Result<Report> {
    pollster::block_on(run_async(dir))
}

async fn run_async(dir: &Path) -> std::io::Result<Report> {
    let mut report = Report::default();
    let options = MountOptions::default();
    let mut picked = Vec::new();
    let mut loose = 0u64;
    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let name = e.file_name().to_string_lossy().into_owned();
        let ext = name
            .rsplit_once('.')
            .map(|(_, x)| x.to_ascii_lowercase())
            .unwrap_or_default();
        if !options.archive_extensions.contains(&ext) {
            // Never opened: loose files may be user mods or unrelated tools.
            loose += 1;
            continue;
        }
        picked.push(PickedFile {
            path: name,
            source: Rc::new(FileSource::open(&e.path())?),
        });
    }
    report.count("loose files skipped (not opened)", loose);

    let mut vfs = Vfs::new(Default::default(), options);
    let mounted = vfs.mount_install(picked).await;
    {
        let t = report.tally(labels::MIX_TOP);
        for _ in &mounted.top_level {
            t.pass();
        }
        for (_, e) in &mounted.failed {
            t.fail(&mount_error_kind(e));
        }
    }
    {
        // Every encrypted header that reached a consistent index, against
        // every container that announced encryption: top-level archives
        // whose flags say so, plus nested candidates that did and failed.
        let mut t = Tally::default();
        for a in vfs.archives() {
            if a.encrypted() {
                t.pass();
            }
        }
        for (_, e) in &mounted.failed {
            t.fail(&mount_error_kind(e));
        }
        if mounted.nested_rejected > 0 {
            report.count(
                "nested MIX candidates rejected (inconsistent header)",
                mounted.nested_rejected as u64,
            );
        }
        report.tallies.push((labels::MIX_DECRYPTED.to_owned(), t));
    }
    report.count("nested MIX archives mounted", mounted.nested.len() as u64);
    report.count(
        "MIX entries in all mounted archives",
        vfs.archives()
            .iter()
            .map(|a| a.index.entries.len() as u64)
            .sum(),
    );
    Ok(report)
}

fn mount_error_kind(e: &glasnost_vfs::MountError) -> String {
    match e {
        glasnost_vfs::MountError::Io(_) => "io".into(),
        glasnost_vfs::MountError::Format(f) => f.kind().into(),
        glasnost_vfs::MountError::Limit(_) => "limit".into(),
    }
}
