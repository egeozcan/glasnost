//! Which copy of a data file the game uses (S-0005).
//!
//! The original mounts its archives by name, one after another; each new
//! archive goes to the end of the search order and is itself resolved
//! through the loose files and the archives mounted before it (S-0005
//! claims 3 and 9 to 14a, Table 1). A request is answered by a loose file of
//! that name if there is one, else by the first archive in that order whose
//! index search finds the name's member id (claims 1, 6 to 8).
//!
//! [`Vfs::mount_startup`] builds the startup part of Table 1 (rows 1 to
//! 15a); [`Vfs::mount_game`] adds the per-game archives (rows 16 to 18) and
//! [`Vfs::finish_loading`] releases the loading archives. Where the
//! original depends on the host's file listing order, this engine sorts
//! names (S-0005 Formulas, *sorted name order*, a departure).

use crate::theater::Theater;
use crate::{ArchiveId, FileRef, MountError, SourceId, Vfs, read_index};
use glasnost_fmt::mix::{MixEntry, MixIndex};

/// Longest requested name in bytes (S-0005 Constants); longer names are
/// not found (claim 6, our engine's result).
pub const LONGEST_NAME: usize = 259; // spec: S-0005 Constants (longest name)

/// Highest expansion archive number (S-0005 Constants: 0 to 99).
const LAST_EXPANSION: u32 = 99; // spec: S-0005 Constants (expansion archive numbers)

/// Quirk switches of S-0005 implemented here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameQuirks {
    /// `quirks.mix_signed_binary_search` (claim 8): members are found by a
    /// bisection over the stored index comparing signed ids; off, every
    /// listed member is found whatever the index order.
    pub mix_signed_binary_search: bool,
    /// `quirks.art_cache_never_invalidated` (S-0005 claim 22, S-0111
    /// claim 19a): a kept file keeps its first copy for the whole session;
    /// off, it is resolved and read afresh at the start of every game. In
    /// this engine it governs only the cursor palette's lifetime, the one
    /// kept file the game client keeps across games: every other file is
    /// read at each game's start, as with the switch off.
    pub art_cache_never_invalidated: bool,
}

impl Default for GameQuirks {
    fn default() -> Self {
        Self {
            // spec: S-0005 Edge cases (signed index order: default is the original's bisection)
            mix_signed_binary_search: true,
            // spec: S-0005 Edge cases (session-long kept art: default on, the original), S-0111 claim 19a (the cursor palette follows it)
            art_cache_never_invalidated: true,
        }
    }
}

impl GameQuirks {
    /// The switches by name.
    pub fn named(&self) -> [(&'static str, bool); 2] {
        [
            ("mix_signed_binary_search", self.mix_signed_binary_search),
            (
                "art_cache_never_invalidated",
                self.art_cache_never_invalidated,
            ),
        ]
    }
}

/// How the search order is built.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameOptions {
    /// Loose *data* files (anything that is not an archive of Table 1) win
    /// over archive members, as in the original (S-0005 claim 1). PLAN.md
    /// 4.3 keeps this off by default: loose files are an opt-in mod layer,
    /// and [`Vfs::shadowing_loose`] reports the ones that would win.
    pub mod_layer: bool,
    /// The sound system starts, so a sound archive is mounted (claim 10).
    pub sound: bool,
    /// Quirk switches.
    pub quirks: GameQuirks,
}

impl Default for GameOptions {
    fn default() -> Self {
        Self {
            mod_layer: false,
            sound: true,
            quirks: GameQuirks::default(),
        }
    }
}

/// A top-level file of the picked folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TopFile {
    pub name: String,
    pub source: SourceId,
    pub len: u64,
    pub archive: Option<ArchiveId>,
}

/// The rows of S-0005 Table 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Row {
    /// Row 1: the expansion's language archive.
    ExpansionLanguage,
    /// Row 2: the base language archive.
    BaseLanguage,
    /// Row 3: numbered expansion archives.
    Expansion,
    /// Rows 4 and 5: the main archives.
    Main,
    /// Row 6: cache archives.
    Cache,
    /// Row 7: local archives.
    Local,
    /// Row 8: neutral archives.
    Neutral,
    /// Row 9: the sound archive.
    Sound,
    /// Row 10: extra-cache and extra-local archives.
    Extra,
    /// Row 11: the remaining startup archives.
    Startup,
    /// Row 12: map archives.
    Maps,
    /// Row 13: the multiplayer map archive.
    Multi,
    /// Row 14: the theme archive.
    Theme,
    /// Row 15: the movie archive.
    Movie,
    /// Row 15a: map packages.
    MapPackage,
    /// Row 16: loading-screen archives (while a game loads).
    Loading,
    /// Row 17: the theater group.
    TheaterGroup,
    /// Row 18: the side group.
    SideGroup,
}

/// Where a mounted archive of the search order came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    /// A loose file in the game directory.
    Loose,
    /// A member of an archive mounted earlier.
    Member(ArchiveId),
}

/// One archive of the search order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchRank {
    /// The mounted archive.
    pub archive: ArchiveId,
    /// The name it was requested under.
    pub name: String,
    /// Its row of Table 1.
    pub row: Row,
    /// Where it was found.
    pub origin: Origin,
}

/// One requested archive name and what became of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    /// Requested name.
    pub name: String,
    /// Row of Table 1.
    pub row: Row,
    /// The rank it got, or `None` when it could not be resolved (skipped
    /// silently by the original, claim 16).
    pub mounted: Option<SearchRank>,
    /// It resolved, but the copy could not be read as an archive.
    pub error: Option<String>,
}

/// What [`Vfs::mount_startup`] did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StartupReport {
    /// Every name requested, in order.
    pub steps: Vec<Step>,
    /// Required archives that could not be resolved: the game refuses to
    /// start (S-0005 claim 16). The two movie archives count as one.
    pub missing_required: Vec<String>,
}

impl StartupReport {
    /// Whether the original would start with these archives (claim 16,
    /// archives only; the required databases are checked by the importer).
    pub fn starts(&self) -> bool {
        self.missing_required.is_empty()
    }
}

/// What [`Vfs::mount_game`] did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GameReport {
    /// Every name requested, in order.
    pub steps: Vec<Step>,
    /// The side archive number used, if the country has a side (claim 13a).
    pub side_number: Option<u32>,
    /// The game never reaches play: the side group's `sidecnn.mix` could
    /// not be resolved, or the country has no side (claims 13a and 16).
    pub never_reaches_play: bool,
}

/// Per-game mounts (S-0005 claim 13).
#[derive(Clone, Debug, Default)]
pub(crate) struct GameState {
    pub options: GameOptions,
    pub theater: Option<Theater>,
}

/// A resolved request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Resolved {
    /// A loose file of the game directory (row 0 of Table 1).
    Loose {
        /// Its name as stored.
        name: String,
        /// Its bytes.
        source: SourceId,
        /// Its length.
        len: u64,
    },
    /// An archive member.
    Member(FileRef),
}

/// The bisection of S-0005 Formulas (*Index search*) over the index in
/// stored order, comparing signed ids; with the switch off, the first
/// entry with the id in stored order.
pub fn index_search(index: &MixIndex, id: u32, quirks: &GameQuirks) -> Option<MixEntry> {
    if !quirks.mix_signed_binary_search {
        return index.find(id).copied();
    }
    // spec: S-0005 claim 8, Formulas (index search by signed id)
    let t = id as i32;
    let entries = &index.entries;
    let (mut a, mut c) = (0usize, entries.len());
    while c > 0 {
        let h = c / 2;
        let e = entries[a + h];
        let s = e.id as i32;
        if s == t {
            return Some(e);
        }
        if s > t {
            c = h;
        } else {
            a += h + 1;
            c -= h + 1;
        }
    }
    None
}

/// Side archive number for side index `s` (S-0005 claim 13a, Formulas).
pub fn side_archive_number(s: u32) -> u32 {
    // spec: S-0005 claim 13a, Formulas (nn(s) = 2 if s = 2, else s + 1)
    if s == 2 { 2 } else { s + 1 }
}

/// The side group's archive names for side index `s` (S-0005 claim 13).
pub fn side_group(s: u32) -> [String; 3] {
    let nn = side_archive_number(s);
    // spec: S-0005 claim 13 (sidecnnmd.mix, sidecnn.mix, sidencnn.mix; nn at least two digits)
    [
        format!("sidec{nn:02}md.mix"),
        format!("sidec{nn:02}.mix"),
        format!("sidenc{nn:02}.mix"),
    ]
}

/// What one step of the startup sequence asks for.
enum Want {
    /// A name found only as a loose file.
    Loose(&'static str),
    /// A name resolved through loose files and mounted archives.
    Name(&'static str),
    /// The first name, or the second only if the first cannot be resolved.
    Either(&'static str, &'static str),
}

/// Archives the game refuses to start without (S-0005 claim 16).
// spec: S-0005 claim 16 (required archives; the movie archive counts once, either name)
const REQUIRED: [&str; 6] = [
    "conqmd.mix",
    "conquer.mix",
    "cameomd.mix",
    "cameo.mix",
    "mapsmd03.mix",
    "multimd.mix",
];

/// Matches `name` (upper-cased) against `PREFIX*SUFFIX` patterns.
fn matches_pattern(upper: &[u8], prefix: &[u8], suffix: &[u8]) -> bool {
    upper.len() >= prefix.len() + suffix.len()
        && upper.starts_with(prefix)
        && upper.ends_with(suffix)
}

impl Vfs {
    /// The quirk switches and options of the search order.
    pub fn game_options(&self) -> &GameOptions {
        &self.game.options
    }

    /// Builds the startup search order of S-0005 Table 1 (rows 1 to 15a),
    /// replacing any earlier one.
    pub async fn mount_startup(&mut self, options: GameOptions) -> StartupReport {
        self.game = GameState {
            options,
            theater: None,
        };
        self.search.clear();
        let mut report = StartupReport::default();

        // spec: S-0005 claim 9 (langmd.mix first, loose only; then language.mix)
        self.step(
            Row::ExpansionLanguage,
            Want::Loose("langmd.mix"),
            &mut report.steps,
        )
        .await;
        self.step(
            Row::BaseLanguage,
            Want::Name("language.mix"),
            &mut report.steps,
        )
        .await;
        // spec: S-0005 claim 9, Formulas (expansion archives, higher number first, loose only)
        for n in (0..=LAST_EXPANSION).rev() {
            let name = format!("expandmd{n:02}.mix");
            if self.top.contains_key(name.to_ascii_uppercase().as_bytes()) {
                self.mount_named(&name, Row::Expansion, true, &mut report.steps)
                    .await;
            }
        }
        // spec: S-0005 claim 10 (main, cache, local and neutral archives, in this order)
        let middle: [(Row, &str); 8] = [
            (Row::Main, "ra2md.mix"),
            (Row::Main, "ra2.mix"),
            (Row::Cache, "cachemd.mix"),
            (Row::Cache, "cache.mix"),
            (Row::Local, "localmd.mix"),
            (Row::Local, "local.mix"),
            (Row::Neutral, "ntrlmd.mix"),
            (Row::Neutral, "neutral.mix"),
        ];
        for (row, name) in middle {
            self.step(row, Want::Name(name), &mut report.steps).await;
        }
        // spec: S-0005 claim 10 (one sound archive, only when the sound system starts)
        if options.sound {
            self.step(
                Row::Sound,
                Want::Either("audiomd.mix", "audio.mix"),
                &mut report.steps,
            )
            .await;
        }
        // spec: S-0005 claim 11 (ecache*.mix, then elocal*.mix; loose only; sorted name order)
        for prefix in [&b"ECACHE"[..], b"ELOCAL"] {
            let names: Vec<String> = self
                .top
                .iter()
                .filter(|(k, _)| matches_pattern(k, prefix, b".MIX"))
                .map(|(_, f)| f.name.clone())
                .collect();
            for name in names {
                self.mount_named(&name, Row::Extra, true, &mut report.steps)
                    .await;
            }
        }
        // spec: S-0005 claim 12 (the remaining startup archives, in this order)
        let rest: [(Row, Want); 13] = [
            (Row::Startup, Want::Name("conqmd.mix")),
            (Row::Startup, Want::Name("genermd.mix")),
            (Row::Startup, Want::Name("generic.mix")),
            (Row::Startup, Want::Name("isogenmd.mix")),
            (Row::Startup, Want::Name("isogen.mix")),
            (Row::Startup, Want::Name("conquer.mix")),
            (Row::Startup, Want::Name("cameomd.mix")),
            (Row::Startup, Want::Name("cameo.mix")),
            (Row::Maps, Want::Name("mapsmd03.mix")),
            (Row::Maps, Want::Name("maps03.mix")),
            (Row::Multi, Want::Name("multimd.mix")),
            (Row::Theme, Want::Either("thememd.mix", "theme.mix")),
            (Row::Movie, Want::Either("movmd03.mix", "movies03.mix")),
        ];
        for (row, want) in rest {
            self.step(row, want, &mut report.steps).await;
        }
        // spec: S-0005 claim 14a (map packages: *.yro except missions.yro, loose, sorted)
        let packages: Vec<String> = self
            .top
            .iter()
            .filter(|(k, _)| matches_pattern(k, b"", b".YRO") && k.as_slice() != b"MISSIONS.YRO")
            .map(|(_, f)| f.name.clone())
            .collect();
        for name in packages {
            self.mount_named(&name, Row::MapPackage, true, &mut report.steps)
                .await;
        }

        // spec: S-0005 claim 16 (required archives)
        let mounted = |n: &str| {
            report
                .steps
                .iter()
                .any(|s| s.mounted.is_some() && s.name.eq_ignore_ascii_case(n))
        };
        for name in REQUIRED {
            if !mounted(name) {
                report.missing_required.push(name.to_owned());
            }
        }
        if !mounted("movmd03.mix") && !mounted("movies03.mix") {
            report
                .missing_required
                .push("movmd03.mix or movies03.mix".to_owned());
        }
        report
    }

    /// Mounts the per-game archives of a game starting in `theater` for a
    /// local player whose country has side index `side` (`None`: no side;
    /// S-0005 claims 13, 13a and 14). The loading archives stay in the
    /// search order until [`Vfs::finish_loading`].
    pub async fn mount_game(&mut self, theater: Theater, side: Option<u32>) -> GameReport {
        let mut report = GameReport::default();
        self.release(Row::Loading);
        // spec: S-0005 claim 13 (loadmd.mix then load.mix, appended while the game loads)
        for name in ["loadmd.mix", "load.mix"] {
            self.mount_named(name, Row::Loading, false, &mut report.steps)
                .await;
        }
        // spec: S-0005 claim 14 (theater group: mounted again only on a theater change)
        if self.game.theater != Some(theater) {
            self.release(Row::TheaterGroup);
            for name in theater.archive_group() {
                self.mount_named(&name, Row::TheaterGroup, false, &mut report.steps)
                    .await;
            }
            self.game.theater = Some(theater);
        }
        // spec: S-0005 claim 13 (the side group is released and mounted again at every game start)
        self.release(Row::SideGroup);
        match side {
            Some(s) => {
                report.side_number = Some(side_archive_number(s));
                let [md, base, n] = side_group(s);
                for name in [&md, &base, &n] {
                    self.mount_named(name, Row::SideGroup, false, &mut report.steps)
                        .await;
                }
                // spec: S-0005 claim 16 (a game whose sidecnn.mix cannot be resolved never reaches play)
                report.never_reaches_play = !report
                    .steps
                    .iter()
                    .any(|st| st.name == base && st.mounted.is_some());
            }
            // spec: S-0005 claim 13a (a country with no side: no side group, never reaches play)
            None => report.never_reaches_play = true,
        }
        report
    }

    /// The game has finished loading: the loading archives are released
    /// (S-0005 claim 13).
    pub fn finish_loading(&mut self) {
        self.release(Row::Loading);
    }

    fn release(&mut self, row: Row) {
        self.search.retain(|r| r.row != row);
    }

    async fn step(&mut self, row: Row, want: Want, steps: &mut Vec<Step>) {
        match want {
            Want::Loose(name) => {
                self.mount_named(name, row, true, steps).await;
            }
            Want::Name(name) => {
                self.mount_named(name, row, false, steps).await;
            }
            Want::Either(first, second) => {
                if !self.mount_named(first, row, false, steps).await {
                    self.mount_named(second, row, false, steps).await;
                }
            }
        }
    }

    /// Resolves `name` as an archive and appends it to the search order
    /// (S-0005 claims 1, 3, 5 and 7). Returns whether it was mounted.
    async fn mount_named(
        &mut self,
        name: &str,
        row: Row,
        loose_only: bool,
        steps: &mut Vec<Step>,
    ) -> bool {
        let mut step = Step {
            name: name.to_owned(),
            row,
            mounted: None,
            error: None,
        };
        match self.archive_for(name, loose_only).await {
            Ok(Some((archive, origin))) => {
                self.learn_name(name);
                let rank = SearchRank {
                    archive,
                    name: name.to_owned(),
                    row,
                    origin,
                };
                self.search.push(rank.clone());
                step.mounted = Some(rank);
            }
            Ok(None) => {}
            Err(e) => step.error = Some(e.to_string()),
        }
        let ok = step.mounted.is_some();
        steps.push(step);
        ok
    }

    /// The archive a name resolves to, mounting a nested one if needed.
    async fn archive_for(
        &mut self,
        name: &str,
        loose_only: bool,
    ) -> Result<Option<(ArchiveId, Origin)>, MountError> {
        if name.len() > LONGEST_NAME {
            return Ok(None);
        }
        // spec: S-0005 claims 1 and 2 (a loose file wins; names match ignoring case)
        if let Some(top) = self.top.get(name.to_ascii_uppercase().as_bytes()) {
            return Ok(top.archive.map(|a| (a, Origin::Loose)));
        }
        // spec: S-0005 claim 5 (some archives are only ever taken as loose files)
        if loose_only {
            return Ok(None);
        }
        // spec: S-0005 claims 3 and 7 (resolved through the archives mounted so far)
        let Some(file) = self.find(name) else {
            return Ok(None);
        };
        let existing = self
            .archives
            .iter()
            .position(|a| a.parent == Some((file.archive, file.entry.id)));
        if let Some(i) = existing {
            return Ok(Some((ArchiveId(i as u32), Origin::Member(file.archive))));
        }
        let loc = self.locate(&file)?;
        let src = self
            .source(loc.source)
            .ok_or(MountError::Limit("unknown source"))?;
        let index = read_index(src.as_ref(), loc.offset, loc.len, &self.keys).await?;
        index.check_consistent(loc.len)?;
        let parent = &self.archives[file.archive.0 as usize];
        let archive = crate::Archive {
            name: name.to_ascii_lowercase(),
            path: format!("{}/{}", parent.path, name.to_ascii_lowercase()),
            parent: Some((file.archive, file.entry.id)),
            depth: parent.depth + 1,
            location: loc,
            index,
        };
        let id = self.push_archive(archive);
        Ok(Some((id, Origin::Member(file.archive))))
    }

    /// [`Vfs::resolve`] for a name given as bytes.
    pub fn resolve_bytes(&self, name: &[u8]) -> Option<Resolved> {
        if name.len() > LONGEST_NAME {
            return None;
        }
        if self.game.options.mod_layer
            && let Some(top) = self.top.get(&name.to_ascii_uppercase())
        {
            return Some(Resolved::Loose {
                name: top.name.clone(),
                source: top.source,
                len: top.len,
            });
        }
        self.find_id(glasnost_fmt::mix::id_hash_bytes(name))
            .map(Resolved::Member)
    }

    /// The copy of a data file the game uses (S-0005 claim 1 and Table 1):
    /// a loose file when the mod layer is on and one exists, else the first
    /// archive of the search order that finds the member.
    pub fn resolve(&self, name: &str) -> Option<Resolved> {
        if name.len() > LONGEST_NAME {
            return None;
        }
        if self.game.options.mod_layer
            && let Some(f) = self.loose_top(name)
        {
            return Some(f);
        }
        self.find(name).map(Resolved::Member)
    }

    /// [`Vfs::resolve`] as it stood while only the two language archives
    /// were mounted (S-0005 Table 1, rows 1 and 2): the loose file under the
    /// mod layer, else the first of those archives that finds the member. A
    /// member that only a later archive holds is not found.
    pub fn resolve_language_only(&self, name: &str) -> Option<Resolved> {
        if name.len() > LONGEST_NAME {
            return None;
        }
        if self.game.options.mod_layer
            && let Some(f) = self.loose_top(name)
        {
            return Some(f);
        }
        let id = glasnost_fmt::mix::id_hash(name);
        // spec: S-0005 claim 9 (the two language archives are mounted first, so they are the leading ranks of the search order)
        self.search
            .iter()
            .take_while(|r| matches!(r.row, Row::ExpansionLanguage | Row::BaseLanguage))
            .find_map(|rank| {
                let entry = index_search(
                    &self.archives[rank.archive.0 as usize].index,
                    id,
                    &self.game.options.quirks,
                )?;
                Some(Resolved::Member(FileRef {
                    archive: rank.archive,
                    entry,
                }))
            })
    }

    /// A loose file that the original would use for `name` but this import
    /// does not, because the mod layer is off (PLAN.md 4.3: such files are
    /// flagged).
    pub fn shadowing_loose(&self, name: &str) -> Option<Resolved> {
        if self.game.options.mod_layer {
            return None;
        }
        self.loose_top(name)
    }

    /// A file that is only ever read as a loose file of the game directory,
    /// whatever the mod layer (S-0005 claim 5: the settings file); `None`
    /// when there is none, never an archived copy.
    pub fn loose_only(&self, name: &str) -> Option<Resolved> {
        // spec: S-0005 claim 5 (the settings file is read only as a loose file; a missing one means defaults, never an archived copy)
        self.loose_top(name)
    }

    fn loose_top(&self, name: &str) -> Option<Resolved> {
        let top = self.top.get(name.to_ascii_uppercase().as_bytes())?;
        Some(Resolved::Loose {
            name: top.name.clone(),
            source: top.source,
            len: top.len,
        })
    }

    /// Reads a resolved file.
    pub async fn read_resolved(&self, r: &Resolved) -> Result<Vec<u8>, MountError> {
        match r {
            Resolved::Member(f) => self.read(f).await,
            Resolved::Loose { source, len, .. } => {
                let src = self
                    .source(*source)
                    .ok_or(MountError::Limit("unknown source"))?;
                let n = usize::try_from(*len).map_err(|_| MountError::Limit("read too large"))?;
                Ok(src.read_at(0, n).await?)
            }
        }
    }

    /// A short description of where a resolved file came from: the loose
    /// file's name, or the archive path.
    pub fn describe(&self, r: &Resolved) -> String {
        match r {
            Resolved::Loose { name, .. } => format!("loose file {name}"),
            Resolved::Member(f) => self
                .archive(f.archive)
                .map_or_else(|| "?".to_owned(), |a| a.path.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glasnost_fmt::mix::{IndexOrder, MixWriter, WriteLayout, id_hash};

    fn index(ids: &[u32], order: IndexOrder) -> MixIndex {
        let bytes = MixWriter {
            layout: WriteLayout::Plain,
            files: ids.iter().map(|&i| (i, vec![1])).collect(),
        }
        .write_ordered(order);
        MixIndex::parse(&bytes, &Default::default()).unwrap()
    }

    /// spec: S-0005 claim 8 (signed index search) and its switch
    #[test]
    fn signed_bisection() {
        let ids = [5u32, 0x8000_0001, 7, 0xFFFF_FFF0, 0x7FFF_FFFF, 0x8000_0000];
        let q = GameQuirks::default();
        let signed = index(&ids, IndexOrder::SignedAscending);
        for id in ids {
            assert!(index_search(&signed, id, &q).is_some(), "{id:#x}");
        }
        // Unsigned order with ids on both sides of 2^31: at least one of the
        // largest below and the smallest at or above 2^31 is missed.
        let unsigned = index(&ids, IndexOrder::UnsignedAscending);
        let below = index_search(&unsigned, 0x7FFF_FFFF, &q).is_some();
        let above = index_search(&unsigned, 0x8000_0000, &q).is_some();
        assert!(!(below && above));
        let fixed = GameQuirks {
            mix_signed_binary_search: false,
            ..GameQuirks::default()
        };
        for id in ids {
            assert!(index_search(&unsigned, id, &fixed).is_some());
        }
        assert!(index_search(&signed, id_hash("absent"), &q).is_none());
    }

    /// spec: S-0005 claim 13a, Formulas
    #[test]
    fn side_numbers() {
        assert_eq!(side_archive_number(0), 1);
        assert_eq!(side_archive_number(1), 2);
        assert_eq!(side_archive_number(2), 2);
        assert_eq!(side_archive_number(3), 4);
        assert_eq!(
            side_group(0),
            ["sidec01md.mix", "sidec01.mix", "sidenc01.mix"]
        );
        assert_eq!(side_group(9)[1], "sidec10.mix");
    }
}
