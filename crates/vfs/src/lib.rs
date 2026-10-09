//! Virtual file system over the user's own game install.
//!
//! - **Mounting for browsing.** Top-level archives come from the folder the
//!   user picked ([`Vfs::mount_install`]). Archives nested inside them are
//!   found by content: an entry whose bytes start with a MIX header that is
//!   exactly consistent with the entry's size is mounted as an archive
//!   (names are not stored in archives, so content is the only reliable
//!   signal for browsing).
//! - **The game's search order** (S-0005) is built separately by
//!   [`Vfs::mount_startup`] and [`Vfs::mount_game`]: archives are mounted by
//!   name, in the original's order, each resolved through the loose files
//!   and the archives mounted before it. Only archives in that order take
//!   part in [`Vfs::resolve`]; the others stay browsable. See [`game`].
//! - **Reads are lazy**: mounting reads only headers, and file contents are
//!   read by range from the picked files when asked for.
//! - **Loose files** in the picked folder are honoured as S-0005 says for
//!   the archives of the search order; loose *data* files win over archive
//!   members only when the mod layer is on ([`game::GameOptions::mod_layer`],
//!   PLAN.md 4.3: off by default). Otherwise a loose file that would have won
//!   is reported, never read.
//!
//! Nothing here ever ships or caches game data in this repository; see
//! CONTRIBUTING.md, "No EA bytes, anywhere". Errors carry offsets, sizes
//! and archive names, never contents.

use std::io;
use std::rc::Rc;

use glasnost_fmt::mix::{MixEntry, MixIndex, MixKeys, MixLayout, id_hash, may_be_mix};

pub mod game;
pub mod names;
pub mod source;
pub mod theater;

pub use game::{GameOptions, GameQuirks, Resolved, SearchRank, StartupReport};
pub use names::Names;
pub use source::{ByteSource, MemSource, ReadFuture};

/// Index of a picked file in a [`Vfs`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceId(pub u32);

/// Index of a mounted archive in a [`Vfs`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArchiveId(pub u32);

/// Bytes of one picked file: `len` bytes at `offset`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Location {
    /// The picked file holding the bytes.
    pub source: SourceId,
    /// Offset within it.
    pub offset: u64,
    /// Length.
    pub len: u64,
}

/// A mounted archive.
#[derive(Clone, Debug)]
pub struct Archive {
    /// Its own name: the file name for top-level archives, else a known name
    /// or the hex ID.
    pub name: String,
    /// Path for display: parent path, `/`, own name.
    pub path: String,
    /// Containing archive and the entry ID there, if nested.
    pub parent: Option<(ArchiveId, u32)>,
    /// Nesting depth (top level is 0).
    pub depth: u32,
    /// Where the archive's bytes are.
    pub location: Location,
    /// Its index.
    pub index: MixIndex,
}

impl Archive {
    /// Whether the header was encrypted.
    pub fn encrypted(&self) -> bool {
        matches!(
            self.index.layout,
            MixLayout::Flagged {
                encrypted: true,
                ..
            }
        )
    }
}

/// A file found by lookup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileRef {
    /// Archive holding it.
    pub archive: ArchiveId,
    /// Its index entry.
    pub entry: MixEntry,
}

/// A file in the picked folder that is not mounted as an archive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LooseFile {
    /// Name relative to the picked folder.
    pub name: String,
    /// Size in bytes.
    pub len: u64,
    /// Its bytes (read only when the mod layer allows it).
    pub source: SourceId,
}

/// One file of the folder the user picked.
pub struct PickedFile {
    /// Path relative to the picked folder, with `/` separators.
    pub path: String,
    /// Its bytes.
    pub source: Rc<dyn ByteSource>,
}

/// Why an archive could not be mounted.
#[derive(Debug)]
pub enum MountError {
    /// Reading failed.
    Io(io::Error),
    /// The header is malformed or inconsistent with the archive's size.
    Format(glasnost_fmt::Error),
    /// A configured limit was reached (depth or archive count).
    Limit(&'static str),
}

impl std::fmt::Display for MountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "read failed: {e}"),
            Self::Format(e) => write!(f, "{e}"),
            Self::Limit(what) => write!(f, "limit reached: {what}"),
        }
    }
}

impl std::error::Error for MountError {}

impl From<io::Error> for MountError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<glasnost_fmt::Error> for MountError {
    fn from(e: glasnost_fmt::Error) -> Self {
        Self::Format(e)
    }
}

/// Mounting options.
#[derive(Clone, Debug)]
pub struct MountOptions {
    /// Look for archives inside archives.
    pub scan_nested: bool,
    /// Deepest nesting followed (top level is 0).
    pub max_depth: u32,
    /// Most archives mounted in total.
    pub max_archives: usize,
    /// File extensions (lower-case) of top-level files mounted as archives.
    pub archive_extensions: Vec<String>,
}

impl Default for MountOptions {
    fn default() -> Self {
        Self {
            scan_nested: true,
            // Engineering bounds of our own against hostile archives.
            max_depth: 4,
            max_archives: 4096,
            // MIX containers: archives, and the two map-package extensions,
            // which are MIX containers too.
            archive_extensions: vec!["mix".into(), "mmx".into(), "yro".into()],
        }
    }
}

/// Outcome of [`Vfs::mount_install`].
#[derive(Debug, Default)]
pub struct MountReport {
    /// Top-level archives mounted.
    pub top_level: Vec<ArchiveId>,
    /// Nested archives mounted.
    pub nested: Vec<ArchiveId>,
    /// Top-level archives that failed, with the reason.
    pub failed: Vec<(String, MountError)>,
    /// Nested candidates (entries that start like a MIX header) that turned
    /// out not to be consistent archives.
    pub nested_rejected: usize,
    /// Top-level files whose names differ only in letter case (S-0005 Edge
    /// cases: an import error; the second name is ignored).
    pub case_clashes: Vec<(String, String)>,
}

/// The mounted install.
pub struct Vfs {
    keys: MixKeys,
    options: MountOptions,
    sources: Vec<(String, Rc<dyn ByteSource>)>,
    archives: Vec<Archive>,
    names: Names,
    loose: Vec<LooseFile>,
    /// Top-level files of the picked folder by upper-cased name.
    top: std::collections::BTreeMap<Vec<u8>, game::TopFile>,
    /// The game's search order, rank 1 first (S-0005).
    search: Vec<SearchRank>,
    game: game::GameState,
}

impl Default for Vfs {
    fn default() -> Self {
        Self::new(MixKeys::default(), MountOptions::default())
    }
}

/// Bytes read first when looking for an archive's header; enough for most
/// plain headers, and [`MixIndex::parse`] asks for more when needed.
const FIRST_HEADER_READ: u64 = 4096;
/// Bytes read from each entry to decide whether it may be an archive.
const PROBE_LEN: u64 = 16;

impl Vfs {
    /// An empty file system.
    pub fn new(keys: MixKeys, options: MountOptions) -> Self {
        Self {
            keys,
            options,
            sources: Vec::new(),
            archives: Vec::new(),
            names: Names::default(),
            loose: Vec::new(),
            top: std::collections::BTreeMap::new(),
            search: Vec::new(),
            game: game::GameState::default(),
        }
    }

    /// Known names.
    pub fn names(&self) -> &Names {
        &self.names
    }

    /// Adds a known name; renames a nested archive that has this ID.
    pub fn learn_name(&mut self, name: &str) -> u32 {
        let id = self.names.insert(name);
        let known = self.names.get(id).map(str::to_owned);
        if let Some(known) = known {
            for i in 0..self.archives.len() {
                if self.archives[i].parent.is_some_and(|(_, e)| e == id) {
                    self.archives[i].name.clone_from(&known);
                }
            }
            self.refresh_paths();
        }
        id
    }

    /// Whether any mounted archive holds this ID.
    pub fn contains_id(&self, id: u32) -> bool {
        self.archives.iter().any(|a| a.index.find(id).is_some())
    }

    /// All mounted archives in mount order.
    pub fn archives(&self) -> &[Archive] {
        &self.archives
    }

    /// One archive.
    pub fn archive(&self, id: ArchiveId) -> Option<&Archive> {
        self.archives.get(id.0 as usize)
    }

    /// The game's search order (S-0005 Table 1), rank 1 first. Empty until
    /// [`Vfs::mount_startup`] runs.
    pub fn search_order(&self) -> &[SearchRank] {
        &self.search
    }

    /// Loose files of the picked folder (not mounted as archives).
    pub fn loose_files(&self) -> &[LooseFile] {
        &self.loose
    }

    /// A picked file, for reads that must not borrow the file system (an
    /// async task reading a [`Location`]).
    pub fn source(&self, id: SourceId) -> Option<Rc<dyn ByteSource>> {
        self.sources.get(id.0 as usize).map(|(_, s)| s.clone())
    }

    /// Name of a picked file.
    pub fn source_name(&self, id: SourceId) -> Option<&str> {
        self.sources.get(id.0 as usize).map(|(n, _)| n.as_str())
    }

    /// Finds an archive member by name in the game's search order (S-0005
    /// claims 6 to 8); loose files are not considered (see
    /// [`Vfs::resolve`]).
    pub fn find(&self, name: &str) -> Option<FileRef> {
        if name.len() > game::LONGEST_NAME {
            return None;
        }
        self.find_id(id_hash(name))
    }

    /// Finds an archive member by ID in the game's search order.
    pub fn find_id(&self, id: u32) -> Option<FileRef> {
        self.search.iter().find_map(|rank| {
            let entry = game::index_search(
                &self.archives[rank.archive.0 as usize].index,
                id,
                &self.game.options.quirks,
            )?;
            Some(FileRef {
                archive: rank.archive,
                entry,
            })
        })
    }

    /// Every mounted archive that lists `id`: first the archives of the
    /// search order in rank order, then the others in mount order. The
    /// first one of the search order is the copy the game uses.
    pub fn find_all(&self, id: u32) -> Vec<FileRef> {
        let ranked = self.search.iter().map(|r| r.archive);
        let rest = (0..self.archives.len())
            .map(|i| ArchiveId(i as u32))
            .filter(|a| !self.search.iter().any(|r| r.archive == *a));
        ranked
            .chain(rest)
            .filter_map(|a| {
                let entry = *self.archive(a)?.index.find(id)?;
                Some(FileRef { archive: a, entry })
            })
            .collect()
    }

    /// Where a file's bytes are. Fails if the entry lies outside its
    /// archive's body (reported here, at read time, not at mount).
    pub fn locate(&self, file: &FileRef) -> Result<Location, MountError> {
        let archive = self
            .archive(file.archive)
            .ok_or(MountError::Limit("unknown archive"))?;
        let range = archive.index.range(&file.entry)?;
        if range.end > archive.location.len {
            return Err(MountError::Format(glasnost_fmt::Error::Invalid(
                "entry extends past its archive",
            )));
        }
        Ok(Location {
            source: archive.location.source,
            offset: archive.location.offset + range.start,
            len: range.end - range.start,
        })
    }

    /// Reads a whole file.
    pub async fn read(&self, file: &FileRef) -> Result<Vec<u8>, MountError> {
        let loc = self.locate(file)?;
        self.read_location(loc, loc.len).await
    }

    /// Reads up to `max` bytes from the start of a file.
    pub async fn read_prefix(&self, file: &FileRef, max: u64) -> Result<Vec<u8>, MountError> {
        let loc = self.locate(file)?;
        self.read_location(loc, loc.len.min(max)).await
    }

    async fn read_location(&self, loc: Location, len: u64) -> Result<Vec<u8>, MountError> {
        let (_, source) = self
            .sources
            .get(loc.source.0 as usize)
            .ok_or(MountError::Limit("unknown source"))?;
        let len = usize::try_from(len).map_err(|_| MountError::Limit("read too large"))?;
        Ok(source.read_at(loc.offset, len).await?)
    }

    /// Mounts the top-level archives of a picked folder (files directly in
    /// it whose extension is in [`MountOptions::archive_extensions`]), in
    /// case-insensitive name order, and their nested archives, for browsing.
    /// Every other file is recorded as loose and read only when the mod
    /// layer allows it. The game's search order is built afterwards by
    /// [`Vfs::mount_startup`].
    pub async fn mount_install(&mut self, files: Vec<PickedFile>) -> MountReport {
        let mut report = MountReport::default();
        let mut top: Vec<PickedFile> = Vec::new();
        for f in files {
            let is_top = !f.path.contains('/');
            let ext = f
                .path
                .rsplit_once('.')
                .map(|(_, e)| e.to_ascii_lowercase())
                .unwrap_or_default();
            if is_top && self.options.archive_extensions.contains(&ext) {
                top.push(f);
            } else {
                let source = self.push_source(&f.path, f.source.clone());
                if is_top {
                    self.note_top(&f.path, source, f.source.len(), None, &mut report);
                }
                self.loose.push(LooseFile {
                    name: f.path.clone(),
                    len: f.source.len(),
                    source,
                });
            }
        }
        top.sort_by_key(|f| f.path.to_ascii_lowercase());
        for f in top {
            let len = f.source.len();
            match self.mount_top(&f.path, f.source.clone()).await {
                Ok((id, nested)) => {
                    let source = self.archives[id.0 as usize].location.source;
                    self.note_top(&f.path, source, len, Some(id), &mut report);
                    report.top_level.push(id);
                    report.nested.extend(nested.mounted);
                    report.nested_rejected += nested.rejected;
                }
                Err(e) => {
                    let source = self.push_source(&f.path, f.source);
                    self.note_top(&f.path, source, len, None, &mut report);
                    report.failed.push((f.path, e));
                }
            }
        }
        report
    }

    fn push_source(&mut self, name: &str, source: Rc<dyn ByteSource>) -> SourceId {
        let id = SourceId(u32::try_from(self.sources.len()).unwrap_or(u32::MAX));
        self.sources.push((name.to_owned(), source));
        id
    }

    /// Records a top-level file for the loose-file test (S-0005 claims 1
    /// and 2), and a clash when two names differ only in letter case.
    fn note_top(
        &mut self,
        name: &str,
        source: SourceId,
        len: u64,
        archive: Option<ArchiveId>,
        report: &mut MountReport,
    ) {
        let key = name.to_ascii_uppercase().into_bytes();
        let file = game::TopFile {
            name: name.to_owned(),
            source,
            len,
            archive,
        };
        if let Some(previous) = self.top.get(&key) {
            // spec: S-0005 Edge cases (loose names that differ only in letter case: an error)
            report
                .case_clashes
                .push((previous.name.clone(), name.to_owned()));
            return;
        }
        self.top.insert(key, file);
    }

    /// Mounts one top-level archive and (if enabled) its nested archives.
    pub async fn mount_top(
        &mut self,
        name: &str,
        source: Rc<dyn ByteSource>,
    ) -> Result<(ArchiveId, NestedReport), MountError> {
        if self.archives.len() >= self.options.max_archives {
            return Err(MountError::Limit("too many archives"));
        }
        let len = source.len();
        let index = read_index(source.as_ref(), 0, len, &self.keys).await?;
        index.check_consistent(len)?;
        let source_id = self.push_source(name, source);
        let id = self.push_archive(Archive {
            name: name.to_owned(),
            path: name.to_owned(),
            parent: None,
            depth: 0,
            location: Location {
                source: source_id,
                offset: 0,
                len,
            },
            index,
        });
        let nested = if self.options.scan_nested {
            self.scan_nested(id).await
        } else {
            NestedReport::default()
        };
        Ok((id, nested))
    }

    fn push_archive(&mut self, archive: Archive) -> ArchiveId {
        let id = ArchiveId(u32::try_from(self.archives.len()).unwrap_or(u32::MAX));
        self.archives.push(archive);
        id
    }

    /// Depth-first search for archives nested in `root`.
    async fn scan_nested(&mut self, root: ArchiveId) -> NestedReport {
        let mut report = NestedReport::default();
        let mut stack = vec![root];
        while let Some(parent) = stack.pop() {
            let Some(archive) = self.archive(parent) else {
                continue;
            };
            if archive.depth >= self.options.max_depth {
                continue;
            }
            let (depth, parent_path, source) = (
                archive.depth + 1,
                archive.path.clone(),
                archive.location.source,
            );
            let entries = archive.index.entries.clone();
            let mut children = Vec::new();
            for entry in entries {
                if self.archives.len() + children.len() >= self.options.max_archives {
                    break;
                }
                let file = FileRef {
                    archive: parent,
                    entry,
                };
                let Ok(loc) = self.locate(&file) else {
                    continue;
                };
                let Ok(probe) = self.read_location(loc, loc.len.min(PROBE_LEN)).await else {
                    continue;
                };
                if !may_be_mix(&probe, loc.len) {
                    continue;
                }
                let Some((_, src)) = self.sources.get(source.0 as usize) else {
                    continue;
                };
                let src = src.clone();
                let parsed = read_index(src.as_ref(), loc.offset, loc.len, &self.keys)
                    .await
                    .and_then(|ix| {
                        ix.check_consistent(loc.len)?;
                        Ok(ix)
                    });
                match parsed {
                    Ok(index) => {
                        let name = self
                            .names
                            .get(entry.id)
                            .map_or_else(|| format!("{:08X}", entry.id), str::to_owned);
                        children.push(Archive {
                            path: format!("{parent_path}/{name}"),
                            name,
                            parent: Some((parent, entry.id)),
                            depth,
                            location: loc,
                            index,
                        });
                    }
                    Err(_) => report.rejected += 1,
                }
            }
            // Push in reverse so the stack visits children in file order.
            let mut ids: Vec<ArchiveId> =
                children.into_iter().map(|c| self.push_archive(c)).collect();
            report.mounted.extend(&ids);
            ids.reverse();
            stack.extend(ids);
        }
        report
    }

    fn refresh_paths(&mut self) {
        for i in 0..self.archives.len() {
            let path = match self.archives[i].parent {
                Some((p, _)) => format!(
                    "{}/{}",
                    self.archives
                        .get(p.0 as usize)
                        .map_or("?", |a| a.path.as_str()),
                    self.archives[i].name
                ),
                None => self.archives[i].name.clone(),
            };
            self.archives[i].path = path;
        }
    }
}

/// Nested archives found under one top-level archive.
#[derive(Debug, Default)]
pub struct NestedReport {
    /// Archives mounted.
    pub mounted: Vec<ArchiveId>,
    /// Candidates rejected as inconsistent.
    pub rejected: usize,
}

/// Reads an archive index at `base` in `source`, fetching more of the
/// header as [`MixIndex::parse`] asks for it.
pub async fn read_index(
    source: &dyn ByteSource,
    base: u64,
    len: u64,
    keys: &MixKeys,
) -> Result<MixIndex, MountError> {
    let mut want = len.min(FIRST_HEADER_READ);
    // Each round at least grows the read; the header is bounded, so a few
    // rounds always suffice.
    for _ in 0..8 {
        let n = usize::try_from(want).map_err(|_| MountError::Limit("header too large"))?;
        let prefix = source.read_at(base, n).await?;
        match MixIndex::parse(&prefix, keys) {
            Ok(index) => return Ok(index),
            Err(glasnost_fmt::Error::Truncated { needed }) if needed > want && needed <= len => {
                want = needed;
            }
            Err(e @ glasnost_fmt::Error::Truncated { .. }) => {
                return Err(MountError::Format(if want >= len {
                    glasnost_fmt::Error::Invalid("MIX header longer than the archive")
                } else {
                    e
                }));
            }
            Err(e) => return Err(e.into()),
        }
    }
    Err(MountError::Limit("header read did not converge"))
}

#[cfg(test)]
mod tests;
