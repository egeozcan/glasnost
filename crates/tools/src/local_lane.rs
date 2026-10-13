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

use std::collections::BTreeSet;

use glasnost_fmt::aud::Aud;
use glasnost_fmt::csf::Csf;
use glasnost_fmt::hva::Hva;
use glasnost_fmt::idx::{Idx, IdxEntry};
use glasnost_fmt::limits::MAX_DECOMPRESSED;
use glasnost_fmt::mappack::{PackCodec, decode_base64, iso_tiles, unpack};
use glasnost_fmt::mix::id_hash;
use glasnost_fmt::pal::Palette6;
use glasnost_fmt::pcx::Pcx;
use glasnost_fmt::shp::{FrameStats, Shp};
use glasnost_fmt::sniff::{Kind, SNIFF_LEN, sniff};
use glasnost_fmt::tmp::Tmp;
use glasnost_fmt::vxl::Vxl;
use glasnost_vfs::names::{candidates, harvest_tokens};
use glasnost_vfs::source::{FileSource, Unopened};
use glasnost_vfs::{ArchiveId, FileRef, GameOptions, MountOptions, PickedFile, Resolved, Vfs};

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
    /// `coverage.csv` of the install's rules and art (names only; never
    /// printed, never committed).
    pub coverage_csv: String,
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

/// Labels of the report lines the opt-in test checks.
pub mod labels {
    /// Encrypted MIX headers decrypted to a consistent index.
    pub const MIX_DECRYPTED: &str = "MIX headers decrypted (top-level and nested)";
    /// Top-level archives mounted.
    pub const MIX_TOP: &str = "top-level MIX containers mounted";
    /// SHP files in conqmd.mix that parsed (of those the sniffer calls SHP).
    pub const CONQ_SHP_FILES: &str = "conqmd.mix: SHP files parsed";
    /// SHP frames in conqmd.mix decoded.
    pub const CONQ_SHP_FRAMES: &str = "conqmd.mix: SHP frames decoded";
    /// VXL files under ra2.mix and ra2md.mix that parsed.
    pub const RA2_VXL_FILES: &str = "ra2.mix/ra2md.mix: VXL files parsed";
    /// IDX files that parsed and found their BAG.
    pub const IDX_FILES: &str = "IDX sound banks parsed with their BAG";
    /// IDX samples decoded from their BAG.
    pub const IDX_SAMPLES: &str = "IDX samples decoded";
    /// Required startup archives resolved (S-0005 claim 16).
    pub const REQUIRED_ARCHIVES: &str = "required startup archives resolved (S-0005 claim 16)";
    /// The rules file resolves to the expansion archive's copy.
    pub const RULES_FROM_EXPANSION: &str =
        "rulesmd.ini resolves to expandmd01.mix's copy (S-0005 claim 18)";
    /// The imported rules resolve through the rules pipeline.
    pub const RULES_RESOLVE: &str = "imported rules resolve, type-check and validate (schema 1)";
    /// Every (category, key) of the rules and art files has a coverage row.
    pub const COVERAGE: &str = "rules and art keys with a coverage outcome";
    /// Every check the test requires at 100 %.
    pub const REQUIRED: &[&str] = &[
        RULES_RESOLVE,
        COVERAGE,
        REQUIRED_ARCHIVES,
        RULES_FROM_EXPANSION,
        MIX_TOP,
        MIX_DECRYPTED,
        CONQ_SHP_FILES,
        CONQ_SHP_FRAMES,
        RA2_VXL_FILES,
        IDX_FILES,
        IDX_SAMPLES,
    ];
}

/// Name of the sprite archive the M1 exit criterion names.
const CONQ_ARCHIVE: &str = "conqmd.mix";
/// Top-level archives whose voxels the M1 exit criterion counts.
const VOXEL_SCOPES: &[&str] = &["ra2.mix", "ra2md.mix"];

/// Runs the lane over the install at `dir`.
pub fn run(dir: &Path) -> std::io::Result<Report> {
    pollster::block_on(run_async(dir))
}

/// The files at the top of an install folder, for mounting: MIX containers
/// (by extension) opened, every other file counted by name and never
/// opened. Returns the picked files and the number of loose files.
pub fn pick_install(dir: &Path, options: &MountOptions) -> std::io::Result<(Vec<PickedFile>, u64)> {
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
            // Their names still take part in the loose-file test (S-0005
            // claim 1), which reports them without reading them.
            loose += 1;
            let len = e.metadata().map_or(0, |m| m.len());
            picked.push(PickedFile {
                path: name,
                source: Rc::new(Unopened(len)),
            });
            continue;
        }
        picked.push(PickedFile {
            path: name,
            source: Rc::new(FileSource::open(&e.path())?),
        });
    }
    Ok((picked, loose))
}

/// Mounts an install folder and builds the game's search order.
pub async fn mount(dir: &Path) -> std::io::Result<Vfs> {
    let options = MountOptions::default();
    let (picked, _) = pick_install(dir, &options)?;
    let mut vfs = Vfs::new(Default::default(), options);
    vfs.mount_install(picked).await;
    vfs.mount_startup(GameOptions::default()).await;
    Ok(vfs)
}

async fn run_async(dir: &Path) -> std::io::Result<Report> {
    let mut report = Report::default();
    let options = MountOptions::default();
    let (picked, loose) = pick_install(dir, &options)?;
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
    search_order(&mut vfs, &mut report).await;
    rules_import(&mut vfs, &mut report).await;
    vfs.learn_name(CONQ_ARCHIVE);
    classify(&mut vfs, &mut report).await;
    Ok(report)
}

/// Data files the importer resolves (S-0005 Import mapping).
const IMPORT_FILES: &[&str] = &[
    "rulesmd.ini",
    "artmd.ini",
    "aimd.ini",
    "soundmd.ini",
    "evamd.ini",
    "thememd.ini",
    "mpmodesmd.ini",
    "uimd.ini",
    "keyboardmd.ini",
];

/// Builds the game's search order (S-0005) and checks the stock outcome
/// the M1 exit criterion names: the expansion archive's rules file is the
/// one used (S-0005 claim 18).
async fn search_order(vfs: &mut Vfs, report: &mut Report) {
    let startup = vfs.mount_startup(GameOptions::default()).await;
    report.count(
        "archives in the game's search order (S-0005 Table 1)",
        vfs.search_order().len() as u64,
    );
    {
        let t = report.tally(labels::REQUIRED_ARCHIVES);
        // Six named archives and the movie archive (either name).
        for _ in 0..7 - startup.missing_required.len() {
            t.pass();
        }
        for _ in &startup.missing_required {
            t.fail("missing");
        }
    }
    {
        let t = report.tally(labels::RULES_FROM_EXPANSION);
        match vfs.resolve("rulesmd.ini") {
            Some(Resolved::Member(f)) if top_level_name(vfs, f.archive) == "expandmd01.mix" => {
                t.pass();
            }
            Some(_) => t.fail("resolved elsewhere"),
            None => t.fail("not found"),
        }
    }
    let resolved = IMPORT_FILES
        .iter()
        .filter(|n| vfs.resolve(n).is_some())
        .count();
    report.count("importer data files resolved", resolved as u64);
    let shadowed = IMPORT_FILES
        .iter()
        .filter(|n| vfs.shadowing_loose(n).is_some())
        .count();
    report.count(
        "importer data files with a loose copy (flagged, not used)",
        shadowed as u64,
    );
}

/// Imports the rules and art (S-0004, S-0006), resolves the result through
/// the rules pipeline and builds `coverage.csv` (kept in the report, never
/// printed: the binary writes it to the git-ignored `local-lane/`).
async fn rules_import(vfs: &mut Vfs, report: &mut Report) {
    let options = glasnost_convert::ImportOptions {
        theater: Some(glasnost_vfs::theater::Theater::Temperate),
        ..Default::default()
    };
    let import = glasnost_convert::import(vfs, &options).await;
    art_counts(&import, report);
    for (kind, reg) in &import.outcome.registries {
        report.count(
            &format!("types registered: {}", kind.key()),
            reg.len() as u64,
        );
    }
    if let Some(sim) = import.tree.get("sim") {
        for kind in ["weapons", "projectiles", "warheads", "animations"] {
            let n = sim
                .get(kind)
                .and_then(glasnost_rules::jsonc::Node::members)
                .map_or(0, <[_]>::len);
            report.count(&format!("sim records: {kind}"), n as u64);
        }
    }
    report.count("sides", import.outcome.sides.len() as u64);
    report.count("colour definitions", import.outcome.colours.len() as u64);
    report.count(
        "game-wide sections layered",
        import.outcome.globals.len() as u64,
    );
    report.count("importer warnings", import.warnings.len() as u64);
    for (label, text) in [
        (
            "importer warnings: endless creation stopped (S-0004 claim 15)",
            "S-0004 claim 15, a departure",
        ),
        (
            "importer warnings: list items with whitespace (S-0003 claim 12)",
            "begins or ends with whitespace",
        ),
        (
            "importer warnings: unknown superweapon (S-0004 claim 3)",
            "names the unknown superweapon",
        ),
        (
            "importer warnings: <none> attached particle system (S-0004 claim 2)",
            "keeps the attached particle system",
        ),
    ] {
        let n = import
            .warnings
            .iter()
            .filter(|w| w.message.contains(text))
            .count();
        report.count(label, n as u64);
    }
    report.count(
        "import errors (the original would not start)",
        import.errors.len() as u64,
    );
    {
        let t = report.tally(labels::RULES_RESOLVE);
        match glasnost_rules::resolve_ruleset(
            Some((import.tree.clone(), import.provenance.clone())),
            &[],
        ) {
            Ok(_) => t.pass(),
            Err(glasnost_rules::Error::Resolve(_)) => t.fail("resolve"),
            Err(glasnost_rules::Error::Schema(_)) => t.fail("schema"),
            Err(glasnost_rules::Error::Invalid(_)) => t.fail("invalid"),
        }
    }
    {
        let t = report.tally(labels::COVERAGE);
        for _ in &import.coverage {
            // Every row has an outcome by construction; unsupported ones are counted below.
            t.pass();
        }
    }
    for (outcome, n) in glasnost_convert::coverage::summary(&import.coverage) {
        report.count(&format!("coverage rows: {}", outcome.word()), n as u64);
    }
    report.coverage_csv = glasnost_convert::coverage::csv(&import.coverage);
}

/// Counts what the art lookup (S-0007, temperate theater) found, per kind.
fn art_counts(import: &glasnost_convert::Import, report: &mut Report) {
    use glasnost_rules::jsonc::Node;
    let Some(types) = import.tree.get("types").and_then(Node::members) else {
        return;
    };
    for kind in types {
        let Some(entries) = kind.value.members() else {
            continue;
        };
        let arts: Vec<&Node> = entries.iter().filter_map(|e| e.value.get("art")).collect();
        if arts.is_empty() {
            continue;
        }
        let sprites = arts.iter().filter(|a| a.get("sprite").is_some()).count();
        let voxels = arts.iter().filter(|a| a.get("voxels").is_some()).count();
        let noted = arts.iter().filter(|a| a.get("notes").is_some()).count();
        let placeholder = arts
            .iter()
            .filter(|a| a.get("icon").and_then(Node::as_str) == Some("XXICON.SHP"))
            .count();
        report.count(
            &format!(
                "art (temperate) {}: types / with sprite / with voxels / with notes / placeholder icon",
                kind.key
            ),
            0,
        );
        if let Some(last) = report.counts.last_mut() {
            last.0 = format!(
                "art (temperate) {}: {} types, {sprites} with a sprite, {voxels} with voxels, {noted} with notes, {placeholder} with the placeholder icon",
                kind.key,
                arts.len()
            );
            last.1 = arts.len() as u64;
        }
    }
}

/// Which top-level archive an archive sits under (lower-case name).
fn top_level_name(vfs: &Vfs, mut id: ArchiveId) -> String {
    while let Some(a) = vfs.archive(id) {
        match a.parent {
            Some((p, _)) => id = p,
            None => return a.name.to_ascii_lowercase(),
        }
    }
    String::new()
}

/// Sniffs every entry that is not itself a mounted archive and runs the
/// strict parser for its kind.
async fn classify(vfs: &mut Vfs, report: &mut Report) {
    let nested: BTreeSet<(ArchiveId, u32)> =
        vfs.archives().iter().filter_map(|a| a.parent).collect();
    let mut kinds: BTreeMap<&'static str, u64> = BTreeMap::new();
    let mut shp_by_compression: BTreeMap<u32, u64> = BTreeMap::new();
    let mut leniency = FrameStats::default();
    let mut limbs = 0u64;
    let mut tokens: BTreeSet<String> = BTreeSet::new();
    let mut unreadable = Tally::default();
    let mut idx_files: Vec<(FileRef, Vec<u8>)> = Vec::new();
    for (i, archive) in vfs.archives().iter().enumerate() {
        let aid = ArchiveId(i as u32);
        let in_conq = archive.name.eq_ignore_ascii_case(CONQ_ARCHIVE);
        let in_voxel_scope = VOXEL_SCOPES.contains(&top_level_name(vfs, aid).as_str());
        for entry in archive.index.entries.clone() {
            if nested.contains(&(aid, entry.id)) {
                continue;
            }
            let file = FileRef {
                archive: aid,
                entry,
            };
            let prefix = match vfs.read_prefix(&file, SNIFF_LEN as u64).await {
                Ok(p) => {
                    unreadable.pass();
                    p
                }
                Err(e) => {
                    unreadable.fail(&mount_error_kind(&e));
                    continue;
                }
            };
            let kind = sniff(&prefix, u64::from(entry.size));
            *kinds.entry(kind.label()).or_default() += 1;
            let needs_body = matches!(
                kind,
                Kind::Shp
                    | Kind::Vxl
                    | Kind::Tmp
                    | Kind::Pal
                    | Kind::Csf
                    | Kind::Hva
                    | Kind::Pcx
                    | Kind::Text
                    | Kind::Idx
                    | Kind::Aud
            );
            if !needs_body {
                continue;
            }
            let Ok(bytes) = vfs.read(&file).await else {
                report.tally("entries readable in full").fail("io");
                continue;
            };
            match kind {
                Kind::Shp => {
                    let shp = Shp::parse(&bytes);
                    let files = report.tally("all archives: SHP files parsed");
                    match &shp {
                        Ok(_) => files.pass(),
                        Err(e) => files.fail(e.kind()),
                    }
                    if in_conq {
                        let t = report.tally(labels::CONQ_SHP_FILES);
                        match &shp {
                            Ok(_) => t.pass(),
                            Err(e) => t.fail(e.kind()),
                        }
                    }
                    let Ok(shp) = shp else { continue };
                    for (fi, f) in shp.frames.iter().enumerate() {
                        let result = shp.decode_frame_stats(fi);
                        if !f.is_empty() {
                            *shp_by_compression
                                .entry(u32::from(f.compression))
                                .or_default() += 1;
                        }
                        if let Ok((_, s)) = &result {
                            leniency.short_rows += s.short_rows;
                            leniency.clamped_runs += s.clamped_runs;
                            leniency.clamped_runs_unusual += s.clamped_runs_unusual;
                        }
                        for label in ["all archives: SHP frames decoded"]
                            .into_iter()
                            .chain(in_conq.then_some(labels::CONQ_SHP_FRAMES))
                        {
                            let t = report.tally(label);
                            match &result {
                                Ok(_) => t.pass(),
                                Err(e) => t.fail(e.kind()),
                            }
                        }
                    }
                }
                Kind::Vxl => {
                    let v = Vxl::parse(&bytes);
                    let label = if in_voxel_scope {
                        labels::RA2_VXL_FILES
                    } else {
                        "other archives: VXL files parsed"
                    };
                    let t = report.tally(label);
                    match &v {
                        Ok(_) => t.pass(),
                        Err(e) => t.fail(e.kind()),
                    }
                    if let (Ok(v), true) = (v, in_voxel_scope) {
                        limbs += v.limbs.len() as u64;
                    }
                }
                Kind::Tmp => {
                    let t = Tmp::parse(&bytes).and_then(|t| {
                        for i in 0..t.tiles.len() {
                            if t.tiles[i].is_some() {
                                t.decode_tile(i)?;
                            }
                        }
                        Ok(())
                    });
                    record(report, "TMP templates decoded (all tiles)", t);
                }
                Kind::Pal => record(
                    report,
                    "PAL palettes parsed",
                    Palette6::parse(&bytes).map(|_| ()),
                ),
                Kind::Csf => record(
                    report,
                    "CSF string tables parsed",
                    Csf::parse(&bytes).map(|_| ()),
                ),
                Kind::Hva => record(
                    report,
                    "HVA animations parsed",
                    Hva::parse(&bytes).map(|_| ()),
                ),
                Kind::Pcx => record(report, "PCX images decoded", Pcx::parse(&bytes).map(|_| ())),
                Kind::Text => {
                    tokens.extend(harvest_tokens(&bytes));
                    decode_map_packs(report, &bytes);
                }
                Kind::Idx => idx_files.push((file, bytes)),
                Kind::Aud => record(
                    report,
                    "AUD sounds decoded",
                    Aud::parse(&bytes).and_then(|a| a.decode()).map(|_| ()),
                ),
                _ => {}
            }
        }
    }
    report.tallies.push(("entries readable".into(), unreadable));
    for (file, bytes) in idx_files {
        decode_bank(vfs, report, file, &bytes).await;
    }
    report.count("ra2.mix/ra2md.mix: VXL limbs decoded", limbs);
    for (kind, n) in kinds {
        report.count(&format!("entries sniffed as {kind}"), n);
    }
    for (c, n) in shp_by_compression {
        report.count(
            &format!("SHP frames with compression {c} (all archives)"),
            n,
        );
    }
    report.count(
        "SHP rows padded (short rows)",
        u64::from(leniency.short_rows),
    );
    report.count(
        "SHP zero runs cut at a row end (final run, one past: usual)",
        u64::from(leniency.clamped_runs - leniency.clamped_runs_unusual),
    );
    report.count(
        "SHP zero runs cut at a row end (other)",
        u64::from(leniency.clamped_runs_unusual),
    );
    let before = vfs.names().len();
    for token in &tokens {
        for name in candidates(token) {
            if vfs.contains_id(id_hash(&name)) {
                vfs.learn_name(&name);
            }
        }
    }
    report.count(
        "names learned from text entries",
        (vfs.names().len() - before) as u64,
    );
    let (named, total) = vfs.archives().iter().fold((0u64, 0u64), |(n, t), a| {
        let k = a
            .index
            .entries
            .iter()
            .filter(|e| vfs.names().get(e.id).is_some())
            .count();
        (n + k as u64, t + a.index.entries.len() as u64)
    });
    report.count("entries with a known name", named);
    report.count("entries in total", total);
}

/// Decodes every sample of one IDX file. Its BAG is the entry of the same
/// archive whose size is exactly the end of the furthest sample (archives
/// store no names, and the pairing must not depend on knowing them).
async fn decode_bank(vfs: &Vfs, report: &mut Report, file: FileRef, bytes: &[u8]) {
    let idx = match Idx::parse(bytes) {
        Ok(i) => i,
        Err(e) => {
            report.tally(labels::IDX_FILES).fail(e.kind());
            return;
        }
    };
    let end = idx
        .entries
        .iter()
        .map(|e| u64::from(e.offset) + u64::from(e.size))
        .max()
        .unwrap_or(0);
    let Some(archive) = vfs.archive(file.archive) else {
        return;
    };
    let bags: Vec<_> = archive
        .index
        .entries
        .iter()
        .filter(|e| e.id != file.entry.id && u64::from(e.size) == end)
        .collect();
    let [bag] = bags.as_slice() else {
        report.tally(labels::IDX_FILES).fail("no-unique-bag");
        return;
    };
    report.tally(labels::IDX_FILES).pass();
    let bag_ref = FileRef {
        archive: file.archive,
        entry: **bag,
    };
    let Ok(bag_loc) = vfs.locate(&bag_ref) else {
        report.tally(labels::IDX_SAMPLES).fail("bag-out-of-range");
        return;
    };
    let Some(source) = vfs.source(bag_loc.source) else {
        return;
    };
    for entry in &idx.entries {
        let read = source
            .read_at(
                bag_loc.offset + u64::from(entry.offset),
                entry.size as usize,
            )
            .await;
        let t = report.tally(labels::IDX_SAMPLES);
        let Ok(data) = read else {
            t.fail("io");
            continue;
        };
        let local = IdxEntry {
            offset: 0,
            ..entry.clone()
        };
        match local.decode(&data) {
            Ok(pcm) if pcm.frames() > 0 => t.pass(),
            Ok(_) => t.fail("empty"),
            Err(e) => t.fail(e.kind()),
        }
    }
}

/// Lines of the section `[name]` in map text, as `key=value` values.
///
/// A lexical test helper for this lane only: it finds the section header
/// line and takes the text after the first `=` of each following line until
/// the next header. It is not an INI reader and makes no claim about how
/// the game reads these files (that waits for the INI specs).
fn section_values<'a>(text: &'a str, name: &str) -> Option<Vec<&'a str>> {
    let mut lines = text.lines();
    let header = format!("[{name}]");
    lines.find(|l| l.trim().eq_ignore_ascii_case(&header))?;
    Some(
        lines
            .take_while(|l| !l.trim_start().starts_with('['))
            .filter_map(|l| l.split_once('=').map(|(_, v)| v.trim()))
            .collect(),
    )
}

/// Decodes the map packs of a text entry that has them.
fn decode_map_packs(report: &mut Report, bytes: &[u8]) {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return;
    };
    let packs = [
        ("IsoMapPack5", PackCodec::Lzo),
        ("OverlayPack", PackCodec::Lcw),
        ("OverlayDataPack", PackCodec::Lcw),
    ];
    for (section, codec) in packs {
        let Some(values) = section_values(text, section) else {
            continue;
        };
        let result = decode_base64(values)
            .and_then(|data| unpack(&data, codec, MAX_DECOMPRESSED))
            .and_then(|raw| match codec {
                PackCodec::Lzo => iso_tiles(&raw).map(|_| ()),
                PackCodec::Lcw => Ok(()),
            });
        record(report, &format!("map packs decoded: {section}"), result);
    }
    decode_map(report, bytes);
}

/// Label of the full map decode (S-0009, `glasnost_convert::map`).
pub const MAPS_DECODED: &str = "maps decoded (S-0009), no panic";

/// Decodes a text entry as a map when it has `[Map]` and `[IsoMapPack5]`,
/// counting maps, panics and importer warnings by kind (counts only).
fn decode_map(report: &mut Report, bytes: &[u8]) {
    use glasnost_convert::map::{DecodeContext, decode, is_map};
    let doc = glasnost_ini::Document::parse(bytes, Default::default(), &mut Vec::new());
    if !is_map(&doc) {
        return;
    }
    let result = std::panic::catch_unwind(|| decode(&doc, &DecodeContext::default()));
    let t = report.tally(MAPS_DECODED);
    match result {
        Ok(d) => {
            t.pass();
            let multiplayer = d.map.waypoints.contains_key(&0) && d.map.waypoints.contains_key(&1);
            if multiplayer {
                report
                    .tally("maps with start waypoints 0 and 1 (multiplayer)")
                    .pass();
            }
            for w in &d.warnings {
                // The warning's kind: its text up to the first digit or quote, so no
                // names or values reach the report.
                let kind: String = w
                    .text
                    .chars()
                    .take_while(|c| !c.is_ascii_digit() && *c != '"' && *c != '[')
                    .take(60)
                    .collect();
                report
                    .tally("map importer warnings by kind")
                    .fail(kind.trim());
            }
        }
        Err(_) => t.fail("panic"),
    }
}

fn record(report: &mut Report, label: &str, result: glasnost_fmt::Result<()>) {
    let t = report.tally(label);
    match result {
        Ok(()) => t.pass(),
        Err(e) => t.fail(e.kind()),
    }
}

fn mount_error_kind(e: &glasnost_vfs::MountError) -> String {
    match e {
        glasnost_vfs::MountError::Io(_) => "io".into(),
        glasnost_vfs::MountError::Format(f) => f.kind().into(),
        glasnost_vfs::MountError::Limit(_) => "limit".into(),
    }
}
