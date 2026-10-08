//! Virtual file system over the user's own game install.
//!
//! - **Mounting.** Top-level archives come from the folder the user picked
//!   ([`Vfs::mount_install`]). Archives nested inside them are found by
//!   content: an entry whose bytes start with a MIX header that is exactly
//!   consistent with the entry's size is mounted as an archive (names are
//!   not stored in archives, so content is the only reliable signal).
//! - **Lookup** by name hashes the name ([`glasnost_fmt::mix::id_hash`]) and
//!   asks the archives in [`Precedence`] order. The default order is a
//!   placeholder pending spec request Q-0002 (see [`precedence`]).
//! - **Reads are lazy**: mounting reads only headers, and file contents are
//!   read by range from the picked files when asked for.
//! - **Loose files** in the picked folder are recorded by name only and
//!   never opened unless the caller enables the mod layer (PLAN.md 4.3:
//!   loose files are off by default).
//!
//! Nothing here ever ships or caches game data in this repository; see
//! CONTRIBUTING.md, "No EA bytes, anywhere". Errors carry offsets, sizes
//! and archive names, never contents.

use std::io;
use std::rc::Rc;

use glasnost_fmt::mix::{MixEntry, MixIndex, MixKeys, MixLayout, id_hash};

pub mod names;
pub mod precedence;
pub mod source;

pub use names::Names;
pub use precedence::{ArchiveSummary, NamedPrecedence, PlaceholderPrecedence, Precedence};
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
}

/// The mounted install.
pub struct Vfs {
    keys: MixKeys,
    options: MountOptions,
    sources: Vec<(String, Rc<dyn ByteSource>)>,
    archives: Vec<Archive>,
    order: Vec<ArchiveId>,
    precedence: Box<dyn Precedence>,
    names: Names,
    loose: Vec<LooseFile>,
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
            order: Vec::new(),
            precedence: Box::new(PlaceholderPrecedence),
            names: Names::default(),
            loose: Vec::new(),
        }
    }

    /// Replaces the lookup order.
    pub fn set_precedence(&mut self, precedence: Box<dyn Precedence>) {
        self.precedence = precedence;
        self.reorder();
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

    /// Archives in lookup order, highest precedence first.
    pub fn lookup_order(&self) -> &[ArchiveId] {
        &self.order
    }

    /// Loose files of the picked folder (never opened by the file system).
    pub fn loose_files(&self) -> &[LooseFile] {
        &self.loose
    }

    /// Name of a picked file.
    pub fn source_name(&self, id: SourceId) -> Option<&str> {
        self.sources.get(id.0 as usize).map(|(n, _)| n.as_str())
    }

    /// Finds a file by name, honouring precedence.
    pub fn find(&self, name: &str) -> Option<FileRef> {
        self.find_id(id_hash(name))
    }

    /// Finds a file by ID, honouring precedence.
    pub fn find_id(&self, id: u32) -> Option<FileRef> {
        self.find_all(id).into_iter().next()
    }

    /// Every copy of a file, highest precedence first.
    pub fn find_all(&self, id: u32) -> Vec<FileRef> {
        self.order
            .iter()
            .filter_map(|&a| {
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
    /// case-insensitive name order, and their nested archives. Every other
    /// file is recorded as loose and never read.
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
                self.loose.push(LooseFile {
                    name: f.path.clone(),
                    len: f.source.len(),
                });
            }
        }
        top.sort_by_key(|f| f.path.to_ascii_lowercase());
        for f in top {
            match self.mount_top(&f.path, f.source).await {
                Ok((id, nested)) => {
                    report.top_level.push(id);
                    report.nested.extend(nested.mounted);
                    report.nested_rejected += nested.rejected;
                }
                Err(e) => report.failed.push((f.path, e)),
            }
        }
        report
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
        let source_id = SourceId(u32::try_from(self.sources.len()).unwrap_or(u32::MAX));
        let len = source.len();
        let index = read_index(source.as_ref(), 0, len, &self.keys).await?;
        index.check_consistent(len)?;
        self.sources.push((name.to_owned(), source));
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
        self.reorder();
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

    fn summaries(&self) -> Vec<ArchiveSummary> {
        // Mount order with nested archives depth-first after their parent.
        let mut out = Vec::with_capacity(self.archives.len());
        let mut stack: Vec<ArchiveId> = (0..self.archives.len())
            .rev()
            .map(|i| ArchiveId(i as u32))
            .filter(|&id| self.archive(id).is_some_and(|a| a.parent.is_none()))
            .collect();
        while let Some(id) = stack.pop() {
            let Some(a) = self.archive(id) else { continue };
            out.push(ArchiveSummary {
                id,
                name: a.name.clone(),
                parent: a.parent.map(|(p, _)| p),
                depth: a.depth,
            });
            let children = (0..self.archives.len())
                .rev()
                .map(|i| ArchiveId(i as u32))
                .filter(|&c| {
                    self.archive(c)
                        .is_some_and(|x| x.parent.map(|(p, _)| p) == Some(id))
                });
            stack.extend(children);
        }
        out
    }

    fn reorder(&mut self) {
        let summaries = self.summaries();
        let mut order = self.precedence.order(&summaries);
        // Guard against a precedence that drops or repeats archives.
        let mut seen = vec![false; self.archives.len()];
        order.retain(|id| {
            seen.get_mut(id.0 as usize)
                .is_some_and(|s| !std::mem::replace(s, true))
        });
        order.extend(
            summaries
                .iter()
                .map(|s| s.id)
                .filter(|id| !seen[id.0 as usize]),
        );
        self.order = order;
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

/// Cheap pre-check on the first bytes of an entry of `len` bytes: could it
/// be a MIX archive? Plain headers must already imply exactly `len`;
/// encrypted ones need the full parse.
pub fn may_be_mix(probe: &[u8], len: u64) -> bool {
    let u16_at = |i: usize| {
        probe
            .get(i..i + 2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
    };
    let u32_at = |i: usize| {
        probe
            .get(i..i + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    };
    let implied =
        |count: u16, body: u32, header: u64| header + 12 * u64::from(count) + u64::from(body);
    match (u16_at(0), u32_at(0)) {
        (Some(count), _) if count != 0 => {
            u32_at(2).is_some_and(|body| implied(count, body, 6) == len)
        }
        (_, Some(flags)) => {
            use glasnost_fmt::mix::{CHECKSUM_LEN, FLAG_CHECKSUM, FLAG_ENCRYPTED};
            if flags & !(FLAG_CHECKSUM | FLAG_ENCRYPTED) != 0 {
                return false;
            }
            if flags & FLAG_ENCRYPTED != 0 {
                // Smallest encrypted header: flags, key source, one block.
                return len >= 92;
            }
            let digest = if flags & FLAG_CHECKSUM != 0 {
                CHECKSUM_LEN
            } else {
                0
            };
            match (u16_at(4), u32_at(6)) {
                (Some(count), Some(body)) => implied(count, body, 10) + digest == len,
                _ => false,
            }
        }
        _ => false,
    }
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
