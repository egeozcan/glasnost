//! The sample bank: samples by name, from the player's files.
//!
//! Samples come from indexed sample banks (IDX/BAG, [`glasnost_fmt::idx`]),
//! AUD files ([`glasnost_fmt::aud`]) and plain PCM WAVE files
//! ([`glasnost_fmt::pcm::Pcm::from_wav`]). They are decoded when first
//! played and then kept.
//!
//! Which files hold the samples, and which copy wins when a name is in
//! several, is not specified (Q-0263). [`discover`] finds sample banks by
//! content and the first bank to hold a name wins: our interim rule.

use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;

use glasnost_fmt::aud::Aud;
use glasnost_fmt::idx::{Idx, IdxEntry};
use glasnost_fmt::pcm::Pcm;
use glasnost_vfs::{FileRef, Vfs};

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

/// Samples by name. Names compare with ASCII case folding (interim, as
/// sample names of an index file are matched; Q-0260).
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
    /// Index files found by content.
    pub banks: usize,
    /// Index files whose data file could not be identified or read.
    pub unpaired: usize,
    /// Samples added.
    pub samples: usize,
}

/// Whether an entry's size fits an index file: a 12-byte header and whole
/// 36-byte entries (the layout of [`glasnost_fmt::idx`]).
fn idx_shaped(size: u32) -> bool {
    let size = size as usize;
    size >= glasnost_fmt::idx::HEADER_LEN
        && (size - glasnost_fmt::idx::HEADER_LEN).is_multiple_of(glasnost_fmt::idx::ENTRY_LEN)
}

/// Builds a bank from the mounted archives (interim rule, Q-0263): every
/// archive entry that parses as an index file, in archive mount order, with
/// the entry of the same archive whose size equals the end of its furthest
/// sample as its data file (as the asset viewer pairs them). The first bank
/// to hold a name wins. Loose files are not read.
pub async fn discover(vfs: &Vfs) -> (SoundBank, DiscoverReport) {
    let mut bank = SoundBank::new();
    let mut report = DiscoverReport::default();
    for (ai, archive) in vfs.archives().iter().enumerate() {
        for entry in &archive.index.entries {
            if !idx_shaped(entry.size) {
                continue;
            }
            let file = FileRef {
                archive: glasnost_vfs::ArchiveId(ai as u32),
                entry: *entry,
            };
            let Ok(head) = vfs.read_prefix(&file, 4).await else {
                continue;
            };
            if head.as_slice() != glasnost_fmt::idx::MAGIC {
                continue;
            }
            let Ok(bytes) = vfs.read(&file).await else {
                continue;
            };
            let Ok(idx) = Idx::parse(&bytes) else {
                continue;
            };
            report.banks += 1;
            let end = idx
                .entries
                .iter()
                .map(|e| u64::from(e.offset) + u64::from(e.size))
                .max()
                .unwrap_or(0);
            let mut matches = archive
                .index
                .entries
                .iter()
                .filter(|e| e.id != entry.id && u64::from(e.size) == end);
            let (Some(bag_entry), None) = (matches.next(), matches.next()) else {
                report.unpaired += 1;
                continue;
            };
            let bag_file = FileRef {
                archive: file.archive,
                entry: *bag_entry,
            };
            match vfs.read(&bag_file).await {
                Ok(bag) => report.samples += bank.add_idx(&idx, Rc::from(bag)),
                Err(_) => report.unpaired += 1,
            }
        }
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

    /// Q-0263 interim: banks found by content, paired by size, first
    /// archive's copy of a name wins.
    #[test]
    fn discover_finds_banks_by_content_in_mount_order() {
        use glasnost_fmt::mix::{HeaderCipher, MixKeys, MixWriter, WriteLayout, id_hash};
        use glasnost_fmt::mix_key::RsaKeyPair;
        use glasnost_vfs::source::MemSource;
        use glasnost_vfs::{MountOptions, PickedFile};

        let mix = |files: Vec<(&str, Vec<u8>)>| {
            MixWriter {
                layout: WriteLayout::Classic,
                files: files.into_iter().map(|(n, d)| (id_hash(n), d)).collect(),
            }
            .write()
        };
        let (i1, b1) = bank_bytes(&["alpha", "shared"], 1);
        let first = Idx::parse(&i1).unwrap().entries[1].decode(&b1).unwrap();
        let (i2, b2) = bank_bytes(&["shared", "beta"], 9);
        let picked = |path: &str, bytes: Vec<u8>| PickedFile {
            path: path.into(),
            source: Rc::new(MemSource(bytes)),
        };
        let files = vec![
            picked(
                "one.mix",
                mix(vec![
                    ("x1.idx", i1),
                    ("x1.bag", b1),
                    ("note.txt", b"hi".to_vec()),
                ]),
            ),
            picked("two.mix", mix(vec![("x2.idx", i2), ("x2.bag", b2)])),
        ];
        let keys = MixKeys {
            rsa: RsaKeyPair::synthetic().public().clone(),
            cipher: HeaderCipher::BigEndianWords,
        };
        let mut vfs = Vfs::new(keys, MountOptions::default());
        pollster::block_on(vfs.mount_install(files));
        let (mut bank, report) = pollster::block_on(discover(&vfs));
        assert_eq!(
            report,
            DiscoverReport {
                banks: 2,
                unpaired: 0,
                samples: 3
            }
        );
        assert_eq!(bank.clip("shared").unwrap().samples, first.samples);
        assert!(bank.clip("beta").is_some());
    }

    #[test]
    fn index_shape_filter() {
        assert!(idx_shaped(12));
        assert!(idx_shaped(12 + 36 * 3));
        assert!(!idx_shaped(11));
        assert!(!idx_shaped(12 + 35));
    }
}
