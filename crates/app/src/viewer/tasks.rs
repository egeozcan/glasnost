//! Asynchronous work for the viewer: mounting, indexing and file reads.
//!
//! Tasks never borrow the file system; they own reference-counted handles
//! to the picked files and the [`Location`]s to read, so the same futures
//! run natively (finished at once with `pollster`) and in the browser
//! (spawned on the page's event loop).

use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

use glasnost_fmt::sniff::{Kind, SNIFF_LEN, sniff};
use glasnost_vfs::names::harvest_tokens;
use glasnost_vfs::theater::Theater;
use glasnost_vfs::{ArchiveId, ByteSource, FileRef, GameOptions, Location, PickedFile, Vfs};

/// A boxed task.
pub type Task = Pin<Box<dyn Future<Output = TaskResult>>>;

/// Something that runs tasks and delivers their results to the viewer.
pub trait Spawner {
    /// Starts `task`.
    fn spawn(&self, task: Task);
}

/// Why a file was read.
#[derive(Clone, Debug)]
pub enum Purpose {
    /// To show it.
    Preview(FileRef),
    /// To use it as the palette.
    Palette(FileRef),
    /// One sample of a sound bank, to play it.
    Sample(glasnost_fmt::idx::IdxEntry),
}

/// The result of a rules import (the file system travels with it).
pub struct Imported {
    /// The file system, with the game's per-game archives mounted.
    pub vfs: Vfs,
    /// The import.
    pub import: glasnost_convert::Import,
    /// The import resolved through the rules pipeline.
    pub resolved: Result<glasnost_rules::Resolved, String>,
    /// Palette files by upper-case name (S-0103 claim 2); `None` when one
    /// cannot be resolved or read.
    pub palettes: BTreeMap<String, Option<glasnost_fmt::pal::Palette6>>,
    /// The theater the art was looked up for.
    pub theater: Theater,
}

/// A finished task.
pub enum TaskResult {
    /// The install was mounted.
    Mounted(Result<Box<(Vfs, String)>, String>),
    /// The rules were imported.
    Imported(Box<Imported>),
    /// Every entry was classified.
    Indexed {
        /// Kind of each entry, by (archive, entry ID).
        kinds: BTreeMap<(ArchiveId, u32), Kind>,
        /// Name-like tokens from text entries.
        tokens: BTreeSet<String>,
    },
    /// The install's maps were found.
    MapsFound(Box<(Vfs, Vec<crate::mapview::find::Listing>)>),
    /// A map was loaded for viewing.
    MapLoaded(Box<(Vfs, crate::mapview::load::Loaded)>),
    /// A file was read.
    Loaded {
        /// What it was read for.
        purpose: Purpose,
        /// Its bytes, or why reading failed.
        bytes: Result<Vec<u8>, String>,
    },
    /// The result of a task started for mount number `mount` ([`for_mount`]).
    ForMount {
        /// The mount the task was started for.
        mount: u64,
        /// What the task gave.
        result: Box<TaskResult>,
    },
}

/// `task`, its result marked as belonging to mount number `mount`, so that a
/// result that comes back after a later mount can be told apart (#89).
pub fn for_mount(mount: u64, task: Task) -> Task {
    Box::pin(async move {
        TaskResult::ForMount {
            mount,
            result: Box::new(task.await),
        }
    })
}

/// Mounts the picked files.
pub fn mount(files: Vec<PickedFile>) -> Task {
    Box::pin(async move {
        let mut vfs = Vfs::default();
        let report = vfs.mount_install(files).await;
        // The game's search order (S-0005), loose data files off (PLAN.md 4.3).
        let startup = vfs.mount_startup(GameOptions::default()).await;
        let mut summary = format!(
            "{} archives mounted ({} nested), {} failed, {} loose files not opened; \
             {} in the game's search order",
            report.top_level.len() + report.nested.len(),
            report.nested.len(),
            report.failed.len(),
            vfs.loose_files().len(),
            vfs.search_order().len(),
        );
        if !startup.starts() {
            summary += &format!(
                "; the game would not start without {}",
                startup.missing_required.join(", ")
            );
        }
        for (a, b) in &report.case_clashes {
            summary += &format!("; {a} and {b} differ only in letter case (import error)");
        }
        if report.top_level.is_empty() {
            TaskResult::Mounted(Err(format!("no archives could be mounted: {summary}")))
        } else {
            TaskResult::Mounted(Ok(Box::new((vfs, summary))))
        }
    })
}

/// Imports the rules and art of a mounted install for one theater
/// (base rules, an empty game-mode layer, no map), resolves the result and
/// reads the palette files S-0103 names.
pub fn import(mut vfs: Vfs, theater: Theater) -> Task {
    Box::pin(async move {
        let options = glasnost_convert::ImportOptions {
            theater: Some(theater),
            ..Default::default()
        };
        let import = glasnost_convert::import(&mut vfs, &options).await;
        let mut palettes = BTreeMap::new();
        let names = [
            Theater::Snow.unit_palette(),
            glasnost_render::palette::ANIM_PALETTE_FILE.to_owned(),
            theater.unit_palette(),
            theater.tile_palette(),
            theater.theater_palette(),
        ];
        for name in names {
            let pal = match vfs.resolve(&name) {
                Some(r) => vfs
                    .read_resolved(&r)
                    .await
                    .ok()
                    .and_then(|b| glasnost_fmt::pal::Palette6::parse(&b).ok()),
                None => None,
            };
            palettes.insert(name.to_ascii_uppercase(), pal);
        }
        let resolved = glasnost_rules::resolve_ruleset(
            Some((import.tree.clone(), import.provenance.clone())),
            &[],
        )
        .map_err(|e| e.to_string());
        TaskResult::Imported(Box::new(Imported {
            vfs,
            import,
            resolved,
            palettes,
            theater,
        }))
    })
}

/// Finds the maps of the mounted install.
pub fn find_maps(vfs: Vfs) -> Task {
    Box::pin(async move {
        let maps = crate::mapview::find::find_maps(&vfs).await;
        TaskResult::MapsFound(Box::new((vfs, maps)))
    })
}

/// Loads a map for viewing, continuing from the voxel buffer `canvas` the
/// program's earlier games left (S-0105 claim 34, S2).
pub fn load_map(
    mut vfs: Vfs,
    file: FileRef,
    origin: String,
    options: crate::mapview::build::Options,
    canvas: glasnost_render::voxel::VoxelCanvas,
) -> Task {
    Box::pin(async move {
        let bytes = vfs.read(&file).await.unwrap_or_default();
        let loaded =
            crate::mapview::load::load(&mut vfs, bytes, origin, &options, true, canvas).await;
        TaskResult::MapLoaded(Box::new((vfs, loaded)))
    })
}

/// Largest text entry read for name harvesting.
const MAX_TEXT: u64 = 4 << 20;

/// One entry to classify.
pub struct IndexJob {
    /// The entry.
    pub file: FileRef,
    /// Where its bytes are.
    pub location: Location,
    /// The picked file holding them.
    pub source: Rc<dyn ByteSource>,
}

/// Index jobs for every entry that is not itself a mounted archive.
pub fn index_jobs(vfs: &Vfs) -> Vec<IndexJob> {
    let nested: BTreeSet<(ArchiveId, u32)> =
        vfs.archives().iter().filter_map(|a| a.parent).collect();
    let mut jobs = Vec::new();
    for (i, archive) in vfs.archives().iter().enumerate() {
        let aid = ArchiveId(i as u32);
        for &entry in &archive.index.entries {
            if nested.contains(&(aid, entry.id)) {
                continue;
            }
            let file = FileRef {
                archive: aid,
                entry,
            };
            let (Ok(location), Some(source)) =
                (vfs.locate(&file), vfs.source(archive.location.source))
            else {
                continue;
            };
            jobs.push(IndexJob {
                file,
                location,
                source,
            });
        }
    }
    jobs
}

/// Classifies every entry and harvests names from text entries.
pub fn index(jobs: Vec<IndexJob>) -> Task {
    Box::pin(async move {
        let mut kinds = BTreeMap::new();
        let mut tokens = BTreeSet::new();
        for job in jobs {
            let n = job.location.len.min(SNIFF_LEN as u64) as usize;
            let Ok(prefix) = job.source.read_at(job.location.offset, n).await else {
                continue;
            };
            let kind = sniff(&prefix, job.location.len);
            kinds.insert((job.file.archive, job.file.entry.id), kind);
            if kind == Kind::Text
                && job.location.len <= MAX_TEXT
                && let Ok(text) = job
                    .source
                    .read_at(job.location.offset, job.location.len as usize)
                    .await
            {
                tokens.extend(harvest_tokens(&text));
            }
        }
        TaskResult::Indexed { kinds, tokens }
    })
}

/// Reads `len` bytes at `location` (the whole file when `len` is `None`).
pub fn read(
    purpose: Purpose,
    source: Rc<dyn ByteSource>,
    location: Location,
    len: Option<u64>,
) -> Task {
    Box::pin(async move {
        let len = len.unwrap_or(location.len).min(location.len);
        let bytes = match usize::try_from(len) {
            Ok(n) => source
                .read_at(location.offset, n)
                .await
                .map_err(|e| format!("read failed ({:?})", e.kind())),
            Err(_) => Err("file too large".to_owned()),
        };
        TaskResult::Loaded { purpose, bytes }
    })
}

/// The picked files of a native folder: top-level files only, archives
/// opened for reading, everything else listed by size and never opened.
#[cfg(not(target_arch = "wasm32"))]
pub fn native_files(dir: &std::path::Path) -> Result<Vec<PickedFile>, String> {
    use glasnost_vfs::MountOptions;
    use glasnost_vfs::source::{FileSource, Unopened};
    let archive_ext = MountOptions::default().archive_extensions;
    let entries =
        std::fs::read_dir(dir).map_err(|e| format!("cannot list the folder ({:?})", e.kind()))?;
    let mut files = Vec::new();
    for e in entries.flatten() {
        let Ok(meta) = e.metadata() else { continue };
        if !meta.is_file() {
            continue;
        }
        let name = e.file_name().to_string_lossy().into_owned();
        let ext = name
            .rsplit_once('.')
            .map(|(_, x)| x.to_ascii_lowercase())
            .unwrap_or_default();
        // The settings file is opened too: it is read only as a loose file
        // (S-0005 claim 5), for the music's shuffle and repeat (S-0703).
        let settings = name.eq_ignore_ascii_case(glasnost_audio::music_files::SETTINGS_FILE);
        let source: Rc<dyn ByteSource> = if archive_ext.contains(&ext) || settings {
            match FileSource::open(&e.path()) {
                Ok(f) => Rc::new(f),
                Err(_) => continue,
            }
        } else {
            Rc::new(Unopened(meta.len()))
        };
        files.push(PickedFile { path: name, source });
    }
    Ok(files)
}
