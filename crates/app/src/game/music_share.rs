//! The web's music share (PLAN.md §5, Stage B): copies of the music
//! tracks read from the picked install, kept between visits in the
//! browser's origin-private file system (OPFS). Our own design; nothing
//! here changes what music plays (S-0703), only where its bytes come from.
//!
//! A track is read through the share: its key is the file name, its length
//! and a hash of its first bytes in the picked file. When the manifest
//! names a stored copy with that key and the copy reads back whole, the
//! copy is used; otherwise the picked file is read and a copy is written,
//! the manifest last. Storage that is refused, full or evicted (a browser
//! may clear the origin's storage, Safari after seven days without a
//! visit) only means the picked file is read: every failure falls back to
//! it with a log note.
//!
//! The store is a trait so the read-through logic is tested natively with
//! an in-memory store; the OPFS store is wasm-only.

use std::cell::RefCell;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

use glasnost_audio::music::{Load, TrackSource};
use glasnost_audio::music_files::{TrackLocation, VfsTracks};
use glasnost_fmt::fixture::fnv1a64;

/// Bytes hashed from the start of a track for its key.
const PROBE: u64 = 64 * 1024; // nocite: our own storage design (enough to tell copies apart, cheap to read)
/// The manifest's file name in the share.
pub const MANIFEST: &str = "manifest.txt";
/// The manifest's first line.
const HEADER: &str = "glasnost-music-share 1"; // nocite: our own file format

/// A boxed future that need not be `Send` (the browser's are not).
pub type LocalFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

/// Where the share keeps its files.
pub trait ShareStore {
    /// A stored file's bytes, `Ok(None)` when there is none.
    fn read<'a>(&'a self, file: &'a str) -> LocalFuture<'a, Result<Option<Vec<u8>>, String>>;
    /// Writes a file whole, replacing any.
    fn write<'a>(&'a self, file: &'a str, bytes: &'a [u8]) -> LocalFuture<'a, Result<(), String>>;
}

/// What identifies a track's bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShareKey {
    /// The track's file name, lower-cased.
    pub name: String,
    /// Its length.
    pub len: u64,
    /// The hash of its first [`PROBE`] bytes.
    pub probe: u64,
}

impl ShareKey {
    /// The stored copy's file name.
    fn stored(&self) -> String {
        let text = format!("{}\n{}\n{}", self.name, self.len, self.probe);
        format!("t{:016x}.bin", fnv1a64(text.as_bytes()))
    }
}

/// The share's list of stored copies.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Manifest {
    entries: Vec<(ShareKey, String)>,
}

impl Manifest {
    /// Reads a manifest; a wrong first line or a malformed line drops
    /// what it cannot read (the copies are then read again).
    pub fn parse(text: &str) -> Self {
        let mut lines = text.lines();
        if lines.next() != Some(HEADER) {
            return Self::default();
        }
        let entries = lines
            .filter_map(|l| {
                let mut w = l.splitn(4, ' ');
                let stored = w.next()?.to_owned();
                let len = w.next()?.parse().ok()?;
                let probe = u64::from_str_radix(w.next()?, 16).ok()?;
                let name = w.next()?.to_owned();
                Some((ShareKey { name, len, probe }, stored))
            })
            .collect();
        Self { entries }
    }

    /// The manifest as text.
    pub fn to_text(&self) -> String {
        let mut s = format!("{HEADER}\n");
        for (k, stored) in &self.entries {
            s += &format!("{stored} {} {:016x} {}\n", k.len, k.probe, k.name);
        }
        s
    }

    /// The stored copy of `key`.
    pub fn find(&self, key: &ShareKey) -> Option<&str> {
        self.entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, s)| s.as_str())
    }

    /// Records the stored copy of `key`, replacing any copy of the same
    /// name.
    pub fn put(&mut self, key: ShareKey, stored: String) {
        self.entries.retain(|(k, _)| k.name != key.name);
        self.entries.push((key, stored));
    }

    /// Forgets the copy of `key` (it did not read back).
    pub fn forget(&mut self, key: &ShareKey) {
        self.entries.retain(|(k, _)| k != key);
    }

    /// The number of copies.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether there is none.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// A share over a store, with its manifest read once.
pub struct Share<S> {
    store: S,
    manifest: RefCell<Option<Manifest>>,
    /// What happened, for the log and the tests.
    notes: RefCell<Vec<String>>,
}

impl<S: ShareStore> Share<S> {
    /// A share over `store`.
    pub fn new(store: S) -> Self {
        Self {
            store,
            manifest: RefCell::new(None),
            notes: RefCell::new(Vec::new()),
        }
    }

    /// The store.
    pub fn store(&self) -> &S {
        &self.store
    }

    /// Takes the notes so far.
    pub fn take_notes(&self) -> Vec<String> {
        std::mem::take(&mut self.notes.borrow_mut())
    }

    fn note(&self, n: String) {
        log::info!("music share: {n}");
        self.notes.borrow_mut().push(n);
    }

    async fn manifest(&self) -> Manifest {
        if let Some(m) = self.manifest.borrow().as_ref() {
            return m.clone();
        }
        let m = match self.store.read(MANIFEST).await {
            Ok(Some(b)) => Manifest::parse(&String::from_utf8_lossy(&b)),
            Ok(None) => Manifest::default(),
            Err(e) => {
                self.note(format!(
                    "not available ({e}); tracks are read from the picked files"
                ));
                Manifest::default()
            }
        };
        *self.manifest.borrow_mut() = Some(m.clone());
        m
    }

    fn set_manifest(&self, m: Manifest) {
        *self.manifest.borrow_mut() = Some(m);
    }

    /// Reads a track through the share.
    pub async fn read(&self, t: &TrackLocation) -> io::Result<Vec<u8>> {
        let n = usize::try_from(t.len).map_err(|_| io::Error::other("track too large"))?;
        let head = t
            .source
            .read_at(t.offset, usize::try_from(t.len.min(PROBE)).unwrap_or(0))
            .await?;
        let key = ShareKey {
            name: t.name.to_ascii_lowercase(),
            len: t.len,
            probe: fnv1a64(&head),
        };
        let mut manifest = self.manifest().await;
        if let Some(stored) = manifest.find(&key).map(str::to_owned) {
            match self.store.read(&stored).await {
                Ok(Some(b)) if b.len() == n && fnv1a64(&b[..head.len()]) == key.probe => {
                    return Ok(b);
                }
                Ok(_) => self.note(format!("the copy of {} is gone or changed", key.name)),
                Err(e) => self.note(format!("the copy of {} did not read ({e})", key.name)),
            }
            manifest.forget(&key);
            self.set_manifest(manifest.clone());
        }
        let bytes = t.source.read_at(t.offset, n).await?;
        let stored = key.stored();
        match self.store.write(&stored, &bytes).await {
            Ok(()) => {
                manifest.put(key, stored);
                // The manifest is written last, so it never names a copy
                // that was not written whole.
                if let Err(e) = self
                    .store
                    .write(MANIFEST, manifest.to_text().as_bytes())
                    .await
                {
                    self.note(format!("the manifest was not written ({e})"));
                }
                self.set_manifest(manifest);
            }
            Err(e) => self.note(format!("no copy of {} kept ({e})", t.name)),
        }
        Ok(bytes)
    }
}

type Read = LocalFuture<'static, io::Result<Vec<u8>>>;

/// The track files read through a share.
pub struct SharedTracks<S> {
    files: VfsTracks,
    share: Rc<Share<S>>,
    pending: Option<Read>,
}

impl<S: ShareStore + 'static> SharedTracks<S> {
    /// `files` read through `share`.
    pub fn new(files: VfsTracks, share: Rc<Share<S>>) -> Self {
        Self {
            files,
            share,
            pending: None,
        }
    }

    fn step(&mut self) -> Load {
        let Some(read) = self.pending.as_mut() else {
            return Load::Pending;
        };
        match read.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
            Poll::Pending => Load::Pending,
            Poll::Ready(r) => {
                self.pending = None;
                // spec: S-0703 claim 8 (a file that can no longer be read: the start fails)
                r.map_or(Load::Failed, |b| Load::Ready(Arc::from(b)))
            }
        }
    }
}

impl<S: ShareStore + 'static> TrackSource for SharedTracks<S> {
    fn open(&mut self, k: usize) -> Load {
        let Some(t) = self.files.location(k) else {
            self.pending = None;
            return Load::Failed;
        };
        let share = self.share.clone();
        self.pending = Some(Box::pin(async move { share.read(&t).await }));
        self.step()
    }

    fn poll(&mut self) -> Load {
        self.step()
    }
}

/// The share in the browser's origin-private file system.
#[cfg(target_arch = "wasm32")]
pub mod opfs {
    use super::{LocalFuture, ShareStore};
    use std::cell::RefCell;
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{FileSystemDirectoryHandle, FileSystemFileHandle};

    /// The share's folder in the origin's file system.
    const FOLDER: &str = "glasnost-music";

    fn js(e: JsValue) -> String {
        e.as_string().unwrap_or_else(|| format!("{e:?}"))
    }

    /// The OPFS store, opened on first use.
    #[derive(Default)]
    pub struct Opfs {
        dir: RefCell<Option<FileSystemDirectoryHandle>>,
        refused: RefCell<Option<String>>,
    }

    impl Opfs {
        async fn dir(&self) -> Result<FileSystemDirectoryHandle, String> {
            if let Some(d) = self.dir.borrow().as_ref() {
                return Ok(d.clone());
            }
            if let Some(e) = self.refused.borrow().as_ref() {
                return Err(e.clone());
            }
            let r = Self::open().await;
            match &r {
                Ok(d) => *self.dir.borrow_mut() = Some(d.clone()),
                Err(e) => *self.refused.borrow_mut() = Some(e.clone()),
            }
            r
        }

        async fn open() -> Result<FileSystemDirectoryHandle, String> {
            let storage = web_sys::window().ok_or("no window")?.navigator().storage();
            // Ask the browser to keep the share; a refusal leaves it
            // evictable, which the read-through handles.
            if let Ok(p) = storage.persist() {
                match JsFuture::from(p).await {
                    Ok(v) if v.as_bool() == Some(true) => {}
                    _ => log::info!("music share: storage is not persistent; it may be evicted"),
                }
            }
            let root: FileSystemDirectoryHandle = JsFuture::from(storage.get_directory())
                .await
                .map_err(js)?
                .unchecked_into();
            let opts = web_sys::FileSystemGetDirectoryOptions::new();
            opts.set_create(true);
            Ok(
                JsFuture::from(root.get_directory_handle_with_options(FOLDER, &opts))
                    .await
                    .map_err(js)?
                    .unchecked_into(),
            )
        }
    }

    impl ShareStore for Opfs {
        fn read<'a>(&'a self, file: &'a str) -> LocalFuture<'a, Result<Option<Vec<u8>>, String>> {
            Box::pin(async move {
                let dir = self.dir().await?;
                let Ok(h) = JsFuture::from(dir.get_file_handle(file)).await else {
                    return Ok(None);
                };
                let h: FileSystemFileHandle = h.unchecked_into();
                let f: web_sys::File = JsFuture::from(h.get_file())
                    .await
                    .map_err(js)?
                    .unchecked_into();
                let buf = JsFuture::from(f.array_buffer()).await.map_err(js)?;
                Ok(Some(js_sys::Uint8Array::new(&buf).to_vec()))
            })
        }

        fn write<'a>(
            &'a self,
            file: &'a str,
            bytes: &'a [u8],
        ) -> LocalFuture<'a, Result<(), String>> {
            Box::pin(async move {
                let dir = self.dir().await?;
                let opts = web_sys::FileSystemGetFileOptions::new();
                opts.set_create(true);
                let h: FileSystemFileHandle =
                    JsFuture::from(dir.get_file_handle_with_options(file, &opts))
                        .await
                        .map_err(js)?
                        .unchecked_into();
                let w: web_sys::FileSystemWritableFileStream = JsFuture::from(h.create_writable())
                    .await
                    .map_err(js)?
                    .unchecked_into();
                let wrote = match w.write_with_u8_array(bytes) {
                    Ok(p) => JsFuture::from(p).await.map(|_| ()).map_err(js),
                    Err(e) => Err(js(e)),
                };
                if let Err(e) = wrote {
                    // A full store: drop what was written.
                    let _ = JsFuture::from(w.abort()).await;
                    return Err(e);
                }
                JsFuture::from(w.close()).await.map_err(js)?;
                Ok(())
            })
        }
    }

    thread_local! {
        static SHARE: std::rc::Rc<super::Share<Opfs>> = std::rc::Rc::new(super::Share::new(Opfs::default()));
    }

    /// The page's share.
    pub fn share() -> std::rc::Rc<super::Share<Opfs>> {
        SHARE.with(std::rc::Rc::clone)
    }
}

/// The track source of a game: on the web the picked files read through
/// the music share, natively the picked files.
pub fn track_source(files: &VfsTracks) -> Box<dyn TrackSource> {
    #[cfg(target_arch = "wasm32")]
    {
        Box::new(SharedTracks::new(files.clone(), opfs::share()))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Box::new(files.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glasnost_vfs::source::MemSource;
    use std::collections::BTreeMap;

    /// A store in memory that can refuse, fill up or lose its files.
    #[derive(Default)]
    struct Mem {
        files: RefCell<BTreeMap<String, Vec<u8>>>,
        refuse: bool,
        room: Option<usize>,
    }

    impl ShareStore for Mem {
        fn read<'a>(&'a self, file: &'a str) -> LocalFuture<'a, Result<Option<Vec<u8>>, String>> {
            let r = if self.refuse {
                Err("refused".to_owned())
            } else {
                Ok(self.files.borrow().get(file).cloned())
            };
            Box::pin(std::future::ready(r))
        }
        fn write<'a>(
            &'a self,
            file: &'a str,
            bytes: &'a [u8],
        ) -> LocalFuture<'a, Result<(), String>> {
            let used: usize = self.files.borrow().values().map(Vec::len).sum();
            let r = if self.refuse {
                Err("refused".to_owned())
            } else if self.room.is_some_and(|r| used + bytes.len() > r) {
                Err("quota".to_owned())
            } else {
                self.files
                    .borrow_mut()
                    .insert(file.to_owned(), bytes.to_vec());
                Ok(())
            };
            Box::pin(std::future::ready(r))
        }
    }

    fn track(name: &str, bytes: &[u8]) -> TrackLocation {
        TrackLocation {
            name: name.into(),
            source: Rc::new(MemSource(bytes.to_vec())),
            offset: 0,
            len: bytes.len() as u64,
        }
    }

    fn read(share: &Share<Mem>, t: &TrackLocation) -> Vec<u8> {
        pollster::block_on(share.read(t)).unwrap()
    }

    #[test]
    fn manifest_round_trips_and_drops_what_it_cannot_read() {
        let mut m = Manifest::default();
        let k = ShareKey {
            name: "a b.wav".into(),
            len: 9,
            probe: 0xabc,
        };
        m.put(k.clone(), k.stored());
        assert_eq!(Manifest::parse(&m.to_text()), m);
        assert_eq!(
            Manifest::parse(&m.to_text()).find(&k),
            Some(k.stored().as_str())
        );
        assert!(Manifest::parse("other 1\nx 1 2 n\n").is_empty());
        assert_eq!(Manifest::parse(&format!("{HEADER}\nx y z n\n")).len(), 0);
    }

    #[test]
    fn a_track_is_copied_once_and_read_from_the_copy() {
        let share = Share::new(Mem::default());
        let t = track("T1.wav", &[7u8; 5000]);
        assert_eq!(read(&share, &t), vec![7u8; 5000]);
        assert_eq!(share.store().files.borrow().len(), 2);
        // A later visit: a new share over the same files reads the copy.
        let files = std::mem::take(&mut *share.store().files.borrow_mut());
        let later = Share::new(Mem {
            files: RefCell::new(files),
            ..Mem::default()
        });
        let m = pollster::block_on(later.manifest());
        assert_eq!(m.len(), 1);
        assert_eq!(read(&later, &t), vec![7u8; 5000]);
        assert!(later.take_notes().is_empty());
    }

    #[test]
    fn a_changed_file_is_read_again() {
        let share = Share::new(Mem::default());
        read(&share, &track("t1.wav", &[1u8; 300]));
        let other = track("t1.wav", &[2u8; 300]);
        assert_eq!(read(&share, &other), vec![2u8; 300]);
        let m = pollster::block_on(share.manifest());
        assert_eq!(m.len(), 1, "one copy per name");
    }

    #[test]
    fn refused_full_or_evicted_storage_falls_back_to_the_picked_file() {
        let t = track("t1.wav", &[3u8; 1000]);
        let refused = Share::new(Mem {
            refuse: true,
            ..Mem::default()
        });
        assert_eq!(read(&refused, &t), vec![3u8; 1000]);
        assert!(!refused.take_notes().is_empty());
        let full = Share::new(Mem {
            room: Some(10),
            ..Mem::default()
        });
        assert_eq!(read(&full, &t), vec![3u8; 1000]);
        assert!(full.take_notes().iter().any(|n| n.contains("quota")));
        assert!(
            full.store().files.borrow().is_empty(),
            "no manifest names a copy not written"
        );
        // Evicted: the manifest names a copy that is gone.
        let share = Share::new(Mem::default());
        read(&share, &t);
        let copy = ShareKey {
            name: "t1.wav".into(),
            len: 1000,
            probe: fnv1a64(&[3u8; 1000]),
        }
        .stored();
        share.store().files.borrow_mut().remove(&copy);
        assert_eq!(read(&share, &t), vec![3u8; 1000]);
        assert!(share.take_notes().iter().any(|n| n.contains("gone")));
        assert!(
            share.store().files.borrow().contains_key(&copy),
            "copied again"
        );
    }

    #[test]
    fn shared_tracks_feed_the_player() {
        let tone = glasnost_fmt::pcm::Pcm::synthetic_tone(8000, 1, 400, 40, 8000, 1).to_wav();
        let mut vfs = glasnost_vfs::Vfs::default();
        pollster::block_on(vfs.mount_install(vec![glasnost_vfs::PickedFile {
            path: "ta.wav".into(),
            source: Rc::new(MemSource(tone.clone())),
        }]));
        let mut opts = *vfs.game_options();
        opts.mod_layer = true;
        pollster::block_on(vfs.mount_startup(opts));
        let db = glasnost_audio::music::MusicDatabase::from_ini(&glasnost_ini::Document::parse(
            b"[Themes]\n0=A\n[A]\nSound=ta\n",
            Default::default(),
            &mut Vec::new(),
        ));
        let files = VfsTracks::find(&vfs, &db);
        let share = Rc::new(Share::new(Mem::default()));
        let mut src = SharedTracks::new(files, share.clone());
        assert!(matches!(src.open(0), Load::Ready(b) if *b == *tone));
        assert!(matches!(src.open(0), Load::Ready(b) if *b == *tone));
        assert_eq!(share.store().files.borrow().len(), 2);
        assert!(matches!(src.open(1), Load::Failed));
    }
}
