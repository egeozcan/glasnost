//! The music's files in a mounted install (S-0703 claim 8, S-0005): the
//! music database, the settings file's music keys, and each
//! track entry's file, found once when the install is read and read
//! through the picked file when a pick starts the track.

use std::future::Future;
use std::io;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

use glasnost_ini::Document;
use glasnost_vfs::Vfs;
use glasnost_vfs::game::Resolved;
use glasnost_vfs::source::ByteSource;

use crate::music::{Load, MusicDatabase, MusicSettings, TrackSource, playable};

/// The bytes of a track's file read first to judge whether it plays: enough
/// for the header of any ordinary sound file (a longer header is read
/// whole).
const HEAD_LEN: u64 = 64 * 1024; // nocite: our own read size (any ordinary header fits; a longer one is read whole)

/// The music database's file name.
pub const MUSIC_DATABASE: &str = "thememd.ini"; // spec: S-0005 Import mapping (thememd.ini, the music database)
/// The settings file's name.
pub const SETTINGS_FILE: &str = "ra2md.ini"; // spec: S-0005 Import mapping (ra2md.ini, the settings file, loose only)

/// Where a track's bytes are: a picked file and a byte range in it.
#[derive(Clone)]
pub struct TrackLocation {
    /// The file name looked for (claim 1a).
    pub name: String,
    /// The picked file holding the bytes.
    pub source: Rc<dyn ByteSource>,
    /// Offset within it.
    pub offset: u64,
    /// Length.
    pub len: u64,
}

type Read = Pin<Box<dyn Future<Output = io::Result<Vec<u8>>>>>;

/// The track files of an install, by entry: a [`TrackSource`] over the
/// mounted install. Natively a read finishes at once; in the browser it
/// finishes a few frames later ([`Load::Pending`]).
#[derive(Default)]
pub struct VfsTracks {
    files: Rc<[Option<TrackLocation>]>,
    pending: Option<Read>,
}

impl Clone for VfsTracks {
    /// The same files, with no read under way.
    fn clone(&self) -> Self {
        Self {
            files: self.files.clone(),
            pending: None,
        }
    }
}

impl VfsTracks {
    /// Finds each entry's file in `vfs` (claim 8: presence is decided
    /// here, once, and does not change afterwards).
    pub fn find(vfs: &Vfs, db: &MusicDatabase) -> Self {
        let files = db
            .entries
            .iter()
            .map(|t| {
                // spec: S-0703 claim 1a and claim 8 (the file is looked for among the mounted archives and loose files, through the game's search order, S-0005)
                let name = t.file.as_deref()?;
                let r = vfs.resolve(name)?;
                let (source, offset, len) = match r {
                    Resolved::Loose { source, len, .. } => (source, 0, len),
                    Resolved::Member(f) => {
                        let loc = vfs.locate(&f).ok()?;
                        (loc.source, loc.offset, loc.len)
                    }
                };
                Some(TrackLocation {
                    name: name.to_owned(),
                    source: vfs.source(source)?,
                    offset,
                    len,
                })
            })
            .collect();
        Self {
            files,
            pending: None,
        }
    }

    /// Where entry `k`'s file is, when present.
    pub fn location(&self, k: usize) -> Option<TrackLocation> {
        self.files.get(k).cloned().flatten()
    }

    /// Which entries have their file present (claim 8).
    pub fn present(&self) -> Vec<bool> {
        self.files.iter().map(Option::is_some).collect()
    }

    /// The importer warnings for present files that do not parse as a
    /// playable sound (S-0703 claim 8a), naming the entry. Such a file
    /// still counts as present.
    pub async fn warnings(&self, db: &MusicDatabase) -> Vec<String> {
        let mut out = Vec::new();
        for (k, f) in self.files.iter().enumerate() {
            let Some(f) = f else {
                continue;
            };
            let Ok(len) = usize::try_from(f.len) else {
                continue;
            };
            let head = f.len.min(HEAD_LEN) as usize;
            let mut ok = match f.source.read_at(f.offset, head).await {
                Ok(b) => playable(&b, len).is_some(),
                Err(_) => false,
            };
            if !ok && head < len {
                ok = match f.source.read_at(f.offset, len).await {
                    Ok(b) => playable(&b, len).is_some(),
                    Err(_) => false,
                };
            }
            if !ok {
                let id = db.entries.get(k).map_or("?", |t| t.id.as_str());
                // spec: S-0703 claim 8a (our engine: the importer warning for a found sound file that does not parse, naming the entry; the file still counts as present)
                out.push(format!(
                    "music track {id}: sound file {} does not parse as a playable sound; its starts fail",
                    f.name
                ));
            }
        }
        out
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

impl TrackSource for VfsTracks {
    fn open(&mut self, k: usize) -> Load {
        let Some(f) = self.files.get(k).cloned().flatten() else {
            self.pending = None;
            return Load::Failed;
        };
        let Ok(n) = usize::try_from(f.len) else {
            self.pending = None;
            return Load::Failed;
        };
        self.pending = Some(Box::pin(async move { f.source.read_at(f.offset, n).await }));
        self.step()
    }

    fn poll(&mut self) -> Load {
        self.step()
    }
}

/// The music of an install: its database, the shuffle and repeat
/// settings, and its track files.
#[derive(Clone)]
pub struct InstallMusic {
    /// The track list.
    pub db: MusicDatabase,
    /// The settings file's shuffle and repeat settings.
    pub settings: MusicSettings,
    /// The track files.
    pub files: VfsTracks,
    /// The importer warnings for present files that do not play (S-0703
    /// claim 8a).
    pub warnings: Vec<String>,
}

impl InstallMusic {
    /// Reads the music of a mounted install: `None` when the music
    /// database does not resolve.
    pub async fn read(vfs: &Vfs) -> Option<Self> {
        // spec: S-0005 claim 17 and Import mapping (the music database, resolved once through the game's search order)
        let r = vfs.resolve(MUSIC_DATABASE)?;
        let bytes = vfs.read_resolved(&r).await.ok()?;
        let db = MusicDatabase::from_ini(&Document::parse(
            &bytes,
            Default::default(),
            &mut Vec::new(),
        ));
        // spec: S-0005 claim 5 (the settings file is read only as a loose file; a missing one means defaults)
        let settings = match vfs.loose_only(SETTINGS_FILE) {
            Some(r) => vfs
                .read_resolved(&r)
                .await
                .ok()
                .map(|b| Document::parse(&b, Default::default(), &mut Vec::new())),
            None => None,
        };
        let settings = MusicSettings::from_ini(settings.as_ref());
        let files = VfsTracks::find(vfs, &db);
        let warnings = files.warnings(&db).await;
        Some(Self {
            db,
            settings,
            files,
            warnings,
        })
    }

    /// A player for one game, reading the picked files.
    pub fn player(&self) -> crate::music::Music {
        self.player_with(Box::new(self.files.clone()))
    }

    /// A player for one game, reading through `source` (the web's music
    /// share).
    pub fn player_with(&self, source: Box<dyn TrackSource>) -> crate::music::Music {
        crate::music::Music::new(self.db.clone(), self.files.present(), self.settings, source)
    }

    /// How many entries have their file present.
    pub fn present_count(&self) -> usize {
        self.files.present().iter().filter(|p| **p).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glasnost_fmt::pcm::Pcm;
    use glasnost_vfs::PickedFile;
    use glasnost_vfs::source::MemSource;

    fn picked(path: &str, bytes: Vec<u8>) -> PickedFile {
        PickedFile {
            path: path.into(),
            source: Rc::new(MemSource(bytes)),
        }
    }

    fn tone() -> Vec<u8> {
        Pcm::synthetic_tone(8000, 1, 400, 40, 8000, 1).to_wav()
    }

    // spec: S-0703 claim 8 and S-0005 claim 5 (presence decided when the install is read; the settings file read as a loose file)
    #[test]
    fn install_music_finds_loose_files() {
        let db = b"[Themes]\n0=A\n1=B\n[A]\nSound=$ta\n[B]\nSound=tb\n".to_vec();
        let mut vfs = Vfs::new(Default::default(), glasnost_vfs::MountOptions::default());
        let mut opts = *vfs.game_options();
        opts.mod_layer = true;
        pollster::block_on(vfs.mount_install(vec![
            picked("thememd.ini", db),
            picked("TA.wav", tone()),
            picked("ra2md.ini", b"[Audio]\nIsScoreShuffle=yes\n".to_vec()),
        ]));
        pollster::block_on(vfs.mount_startup(opts));
        let m = pollster::block_on(InstallMusic::read(&vfs)).expect("database");
        assert_eq!(m.files.present(), [true, false]);
        assert!(m.settings.shuffle && !m.settings.repeat);
        let mut files = m.files.clone();
        assert!(matches!(files.open(0), Load::Ready(b) if *b == *tone()));
        assert!(matches!(files.open(1), Load::Failed));
        // With the mod layer off (PLAN.md 4.3) a loose music database is
        // not used, while the settings file is still read.
        let mut vfs = Vfs::default();
        pollster::block_on(vfs.mount_install(vec![
            picked("thememd.ini", b"[Themes]\n0=A\n".to_vec()),
            picked("RA2MD.INI", b"[Audio]\n".to_vec()),
        ]));
        pollster::block_on(vfs.mount_startup(Default::default()));
        assert!(pollster::block_on(InstallMusic::read(&vfs)).is_none());
        assert!(vfs.loose_only(SETTINGS_FILE).is_some());
    }

    // spec: S-0703 claim 1a (MU1 part m's engine fixture: a name already ending in .wav or .WAV gets the extension again, matched ignoring case; the file with the extension once is not found)
    #[test]
    fn a_name_ending_in_the_extension_gets_it_again() {
        for (name, twice) in [
            ("tone4.wav", false),
            ("tone4.wav", true),
            ("tone4.WAV", true),
        ] {
            let db = format!("[Themes]\n0=T4\n[T4]\nSound={name}\n").into_bytes();
            let mut vfs = Vfs::new(Default::default(), glasnost_vfs::MountOptions::default());
            let mut opts = *vfs.game_options();
            opts.mod_layer = true;
            let mut files = vec![picked("thememd.ini", db), picked("tone4.wav", tone())];
            if twice {
                files.push(picked("tone4.wav.wav", tone()));
            }
            pollster::block_on(vfs.mount_install(files));
            pollster::block_on(vfs.mount_startup(opts));
            let m = pollster::block_on(InstallMusic::read(&vfs)).expect("database");
            assert_eq!(m.files.present(), [twice], "{name}, twice: {twice}");
        }
    }

    // spec: S-0703 claim 8a and MU10 part a's engine fixture (the importer warning names the entry whose present file does not parse; the file still counts as present)
    #[test]
    fn a_present_file_that_does_not_parse_gives_a_warning() {
        let db = b"[Themes]\n0=T0\n1=T1\n2=T2\n[T0]\nSound=t0\n[T1]\nSound=t1\n[T2]\nSound=t2\n"
            .to_vec();
        let mut vfs = Vfs::new(Default::default(), glasnost_vfs::MountOptions::default());
        let mut opts = *vfs.game_options();
        opts.mod_layer = true;
        // A header and no sample data after it.
        let mut empty = tone();
        let data = empty
            .windows(4)
            .position(|w| w == b"data")
            .expect("data chunk");
        empty.truncate(data + 8);
        empty[data + 4..data + 8].copy_from_slice(&0u32.to_le_bytes());
        pollster::block_on(vfs.mount_install(vec![
            picked("thememd.ini", db),
            picked("t0.wav", tone()),
            picked("t1.wav", empty),
            picked(
                "t2.wav",
                b"not a sound file, only a few words of text".to_vec(),
            ),
        ]));
        pollster::block_on(vfs.mount_startup(opts));
        let m = pollster::block_on(InstallMusic::read(&vfs)).expect("database");
        assert_eq!(m.files.present(), [true, true, true]);
        assert_eq!(m.warnings.len(), 2, "{:?}", m.warnings);
        assert!(m.warnings[0].contains("T1"), "{:?}", m.warnings);
        assert!(m.warnings[1].contains("T2"), "{:?}", m.warnings);
    }
}
