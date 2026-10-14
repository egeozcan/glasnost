//! The sample bank: samples by name, from the player's files.
//!
//! Samples come from indexed sample banks (IDX/BAG, [`glasnost_fmt::idx`]),
//! AUD files ([`glasnost_fmt::aud`]) and plain PCM WAVE files
//! ([`glasnost_fmt::pcm::Pcm::from_wav`]). They are decoded when first
//! played and then kept.
//!
//! The samples of sound entries come from one indexed bank, found by name
//! through the game's file lookup (S-0701 claim 34, [`discover`]). Which copy
//! wins when the index lists a name twice is not specified (Q-0263): the
//! first copy wins, our interim rule.

use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;

use glasnost_fmt::aud::Aud;
use glasnost_fmt::idx::{Idx, IdxEntry};
use glasnost_fmt::pcm::Pcm;
use glasnost_vfs::Vfs;

use crate::mixer::Clip;

/// Where a sample's bytes are.
#[derive(Clone, Debug)]
enum Source {
    /// An entry of an indexed bank, with the bank's data file.
    Idx { entry: IdxEntry, bag: Rc<[u8]> },
    /// An AUD file.
    Aud(Rc<[u8]>),
    /// Already decoded.
    Clip(Arc<Clip>),
}

/// Samples by name. Names compare with ASCII case folding (S-0701 claim 34:
/// samples are looked up ignoring case).
#[derive(Clone, Debug, Default)]
pub struct SoundBank {
    sources: BTreeMap<String, Source>,
    cache: BTreeMap<String, Option<Arc<Clip>>>,
    misses: u64,
    failures: u64,
}

/// The key a name is stored under.
fn key(name: &str) -> String {
    name.to_ascii_uppercase()
}

impl SoundBank {
    /// An empty bank.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds every sample of an indexed bank; `bag` is its data file. A
    /// name the bank already holds keeps its first copy (Q-0263 interim).
    /// Returns how many samples were added.
    pub fn add_idx(&mut self, idx: &Idx, bag: Rc<[u8]>) -> usize {
        let mut added = 0;
        for e in &idx.entries {
            if let std::collections::btree_map::Entry::Vacant(v) = self.sources.entry(key(&e.name))
            {
                v.insert(Source::Idx {
                    entry: e.clone(),
                    bag: bag.clone(),
                });
                added += 1;
            }
        }
        added
    }

    /// Adds a sound file by name: an AUD file, or a plain PCM WAVE file.
    /// A name the bank already holds keeps its first copy. Returns whether
    /// the bytes were recognised and added.
    pub fn add_file(&mut self, name: &str, bytes: Rc<[u8]>) -> bool {
        let k = key(name);
        if self.sources.contains_key(&k) {
            return false;
        }
        let source = if bytes.starts_with(b"RIFF") {
            match Pcm::from_wav(&bytes).ok().and_then(Clip::from_pcm) {
                Some(c) => Source::Clip(Arc::new(c)),
                None => return false,
            }
        } else if Aud::parse(&bytes).is_ok() {
            Source::Aud(bytes)
        } else {
            return false;
        };
        self.sources.insert(k, source);
        true
    }

    /// Adds decoded PCM by name (first copy wins). Returns whether it was
    /// playable and added.
    pub fn add_pcm(&mut self, name: &str, pcm: Pcm) -> bool {
        let k = key(name);
        if self.sources.contains_key(&k) {
            return false;
        }
        match Clip::from_pcm(pcm) {
            Some(c) => {
                self.sources.insert(k, Source::Clip(Arc::new(c)));
                true
            }
            None => false,
        }
    }

    /// Samples held.
    pub fn len(&self) -> usize {
        self.sources.len()
    }

    /// Whether the bank is empty.
    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }

    /// Whether the bank holds a sample of that name.
    pub fn contains(&self, name: &str) -> bool {
        self.sources.contains_key(&key(name))
    }

    /// Sample names, in key order.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.sources.keys().map(String::as_str)
    }

    /// Lookups of names the bank does not hold (for diagnostics: how much
    /// of the rules data's sound names resolve under the interim rules).
    pub fn misses(&self) -> u64 {
        self.misses
    }

    /// Samples that failed to decode.
    pub fn failures(&self) -> u64 {
        self.failures
    }

    /// The decoded sample of that name, decoding it on first use.
    pub fn clip(&mut self, name: &str) -> Option<Arc<Clip>> {
        let k = key(name);
        if let Some(c) = self.cache.get(&k) {
            return c.clone();
        }
        let Some(source) = self.sources.get(&k) else {
            self.misses += 1;
            return None;
        };
        let decoded = match source {
            Source::Idx { entry, bag } => {
                // An entry's offset is into its own bank's data file.
                entry
                    .decode(bag)
                    .ok()
                    .and_then(Clip::from_pcm)
                    .map(Arc::new)
            }
            Source::Aud(bytes) => Aud::parse(bytes)
                .and_then(|a| a.decode())
                .ok()
                .and_then(Clip::from_pcm)
                .map(Arc::new),
            Source::Clip(c) => Some(c.clone()),
        };
        if decoded.is_none() {
            self.failures += 1;
        }
        self.cache.insert(k, decoded.clone());
        decoded
    }
}

/// What [`discover`] found.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiscoverReport {
    /// Index files found (0 or 1).
    pub banks: usize,
    /// Index files whose data file was missing or unreadable, or that did
    /// not parse.
    pub unpaired: usize,
    /// Samples added.
    pub samples: usize,
}

/// The sample bank's index file (S-0701 Import mapping, claim 34).
pub const INDEX_FILE: &str = "audio.idx"; // spec: S-0701 Import mapping (`AUDIO.IDX`, matched ignoring case)
/// The sample bank's data file (S-0701 Import mapping, claim 34).
pub const DATA_FILE: &str = "audio.bag"; // spec: S-0701 Import mapping (`AUDIO.BAG`, matched ignoring case)

/// Builds the sample bank: the index file and the data file, each found
/// through the game's file lookup (S-0005), so either may be a member of a
/// mounted archive or a loose file (S-0701 claim 34).
pub async fn discover(vfs: &Vfs) -> (SoundBank, DiscoverReport) {
    let mut bank = SoundBank::new();
    let mut report = DiscoverReport::default();
    // spec: S-0701 claim 34 (one bank: its index file and data file through the ordinary file lookup of S-0005)
    let Some(idx_ref) = vfs.resolve(INDEX_FILE) else {
        return (bank, report);
    };
    report.banks = 1;
    let idx = match vfs.read_resolved(&idx_ref).await {
        Ok(bytes) => Idx::parse(&bytes).ok(),
        Err(_) => None,
    };
    let bag = match vfs.resolve(DATA_FILE) {
        Some(r) => vfs.read_resolved(&r).await.ok(),
        None => None,
    };
    match (idx, bag) {
        (Some(idx), Some(bag)) => report.samples = bank.add_idx(&idx, Rc::from(bag)),
        _ => report.unpaired = 1,
    }
    (bank, report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use glasnost_fmt::aud::{AudCompression, AudWriter};
    use glasnost_fmt::idx::{IdxEncoding, IdxSound, IdxWriter};

    fn tone(rate: u32, channels: u16, seed: u64) -> Pcm {
        Pcm::synthetic_tone(rate, channels, 600, 40, 6000, seed)
    }

    /// An indexed bank of our own with invented sample names.
    fn bank_bytes(names: &[&str], seed: u64) -> (Vec<u8>, Vec<u8>) {
        IdxWriter {
            sounds: names
                .iter()
                .enumerate()
                .map(|(i, n)| IdxSound {
                    name: (*n).into(),
                    pcm: tone(22050, 1, seed + i as u64),
                    encoding: if i % 2 == 0 {
                        IdxEncoding::RawPcm
                    } else {
                        IdxEncoding::Ima
                    },
                })
                .collect(),
        }
        .write()
    }

    #[test]
    fn idx_samples_resolve_case_insensitively_and_decode_once() {
        let (idx, bag) = bank_bytes(&["zap01", "Hum02"], 1);
        let mut b = SoundBank::new();
        assert_eq!(b.add_idx(&Idx::parse(&idx).unwrap(), Rc::from(bag)), 2);
        let raw = b.clip("ZAP01").unwrap();
        assert_eq!(raw.samples, tone(22050, 1, 1).samples);
        let ima = b.clip("hum02").unwrap();
        assert!(ima.frames() >= 600, "{}", ima.frames());
        assert!(Arc::ptr_eq(&ima, &b.clip("HUM02").unwrap()));
        assert!(b.clip("nothing").is_none());
        assert_eq!(b.misses(), 1);
    }

    #[test]
    fn the_first_copy_of_a_name_wins() {
        let (i1, b1) = bank_bytes(&["dup"], 10);
        let (i2, b2) = bank_bytes(&["DUP", "other"], 20);
        let mut b = SoundBank::new();
        b.add_idx(&Idx::parse(&i1).unwrap(), Rc::from(b1));
        assert_eq!(b.add_idx(&Idx::parse(&i2).unwrap(), Rc::from(b2)), 1);
        assert_eq!(b.clip("dup").unwrap().samples, tone(22050, 1, 10).samples);
        assert_eq!(b.len(), 2);
    }

    #[test]
    fn aud_and_wav_files_are_added_by_name() {
        let pcm = tone(11025, 1, 3);
        let aud = AudWriter {
            compression: AudCompression::Ima,
            pcm: pcm.clone(),
            chunk_frames: 256,
        }
        .write();
        let mut b = SoundBank::new();
        assert!(b.add_file("beep.aud", Rc::from(aud)));
        assert!(b.add_file("boop.wav", Rc::from(pcm.to_wav())));
        assert!(!b.add_file("junk", Rc::from(&b"not sound"[..])));
        assert!(b.clip("BEEP.AUD").unwrap().frames() >= 600);
        assert_eq!(b.clip("boop.wav").unwrap().samples, pcm.samples);
        assert!(b.add_pcm("made", tone(8000, 1, 4)));
        assert!(b.contains("MADE"));
    }

    // spec: S-0701 claim 34 (the bank's two files by name through the file lookup, loose files included; other banks are not read)
    #[test]
    fn discover_reads_the_named_bank_files() {
        use glasnost_fmt::mix::{HeaderCipher, MixKeys};
        use glasnost_fmt::mix_key::RsaKeyPair;
        use glasnost_vfs::source::MemSource;
        use glasnost_vfs::{GameOptions, MountOptions, PickedFile};

        let (i1, b1) = bank_bytes(&["alpha", "shared"], 1);
        let (i2, b2) = bank_bytes(&["beta"], 9);
        let picked = |path: &str, bytes: Vec<u8>| PickedFile {
            path: path.into(),
            source: Rc::new(MemSource(bytes)),
        };
        let files = vec![
            picked("AUDIO.IDX", i1),
            picked("Audio.Bag", b1),
            picked("other.idx", i2),
            picked("other.bag", b2),
        ];
        let keys = MixKeys {
            rsa: RsaKeyPair::synthetic().public().clone(),
            cipher: HeaderCipher::BigEndianWords,
        };
        let mut vfs = Vfs::new(keys, MountOptions::default());
        pollster::block_on(vfs.mount_install(files));
        let on = GameOptions {
            mod_layer: true,
            ..GameOptions::default()
        };
        pollster::block_on(vfs.mount_startup(on));
        let (mut bank, report) = pollster::block_on(discover(&vfs));
        assert_eq!(
            report,
            DiscoverReport {
                banks: 1,
                unpaired: 0,
                samples: 2
            }
        );
        assert!(bank.clip("SHARED").is_some());
        assert!(bank.clip("beta").is_none());
    }

    #[test]
    fn no_index_file_gives_an_empty_bank() {
        use glasnost_fmt::mix::{HeaderCipher, MixKeys};
        use glasnost_fmt::mix_key::RsaKeyPair;
        use glasnost_vfs::MountOptions;

        let keys = MixKeys {
            rsa: RsaKeyPair::synthetic().public().clone(),
            cipher: HeaderCipher::BigEndianWords,
        };
        let vfs = Vfs::new(keys, MountOptions::default());
        let (bank, report) = pollster::block_on(discover(&vfs));
        assert_eq!(report, DiscoverReport::default());
        assert!(bank.is_empty());
    }
}
