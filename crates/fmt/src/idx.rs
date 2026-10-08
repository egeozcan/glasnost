//! IDX/BAG sound banks.
//!
//! The `.idx` file indexes sounds stored back to back in a `.bag` file. It
//! starts with the magic `GABA`, an `i32` version ([`VERSION`]) and an `i32`
//! entry count, then 36-byte entries: a 16-byte NUL-padded name, then `u32`
//! offset into the BAG, size, sample rate, flags and block length.
//!
//! Flags: [`FLAG_STEREO`]; [`FLAG_RAW_PCM`] for raw little-endian 16-bit
//! PCM; otherwise [`FLAG_IMA`] for IMA ADPCM in the block layout of
//! [`adpcm::ima_block_decode`], with the entry's block length. Bit 2
//! ([`FLAG_BIT2`]) is set on every entry XCC writes; its meaning is not
//! documented and it is ignored.
//!
//! Only version 2 is known (XCC and OpenRA both require it); other versions
//! are [`Error::Unsupported`]. Layout from XCC's `cc_structures.h`,
//! `audio_idx_file.cpp` and `xse.cpp` (Copyright (C) 2000 Olaf van der
//! Spek) and OpenRA's `IdxReader.cs` and `IdxEntry.cs` (Copyright (c) The
//! OpenRA Developers and Contributors), both GPL-3.0-or-later.

use crate::adpcm;
use crate::bytes::{PutLe, Reader, len_u32, slice64, wide};
use crate::error::{Error, Result};
use crate::fixture::FixtureWriter;
use crate::limits::MAX_DECOMPRESSED;
use crate::pcm::Pcm;

/// Magic at the start of an IDX file.
pub const MAGIC: [u8; 4] = *b"GABA";
/// The only known version.
pub const VERSION: i32 = 2;
/// Length of the header.
pub const HEADER_LEN: usize = 12;
/// Length of one entry.
pub const ENTRY_LEN: usize = 36;
/// Length of the name field.
pub const NAME_LEN: usize = 16;
/// Flag: interleaved stereo.
pub const FLAG_STEREO: u32 = 1;
/// Flag: raw 16-bit PCM (takes precedence over [`FLAG_IMA`], as in XCC).
pub const FLAG_RAW_PCM: u32 = 2;
/// Flag set on every entry XCC writes; meaning undocumented, ignored.
pub const FLAG_BIT2: u32 = 4;
/// Flag: IMA ADPCM blocks.
pub const FLAG_IMA: u32 = 8;
/// Block length per channel that [`IdxWriter`] uses for IMA entries (as
/// XCC's sound editor does).
pub const WRITER_BLOCK_PER_CHANNEL: u32 = 512;

/// How an entry's data is stored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdxEncoding {
    /// Raw little-endian 16-bit PCM.
    RawPcm,
    /// IMA ADPCM blocks.
    Ima,
}

/// One sound in the bank.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdxEntry {
    /// Name, up to the first NUL (bytes that are not UTF-8 are replaced).
    pub name: String,
    /// Offset of the data in the BAG.
    pub offset: u32,
    /// Size of the data in bytes.
    pub size: u32,
    /// Frames per second.
    pub sample_rate: u32,
    /// Raw flags.
    pub flags: u32,
    /// IMA block length in bytes (0 for raw entries).
    pub chunk_size: u32,
}

impl IdxEntry {
    /// Channel count.
    pub fn channels(&self) -> u16 {
        if self.flags & FLAG_STEREO != 0 { 2 } else { 1 }
    }

    /// How the data is stored.
    pub fn encoding(&self) -> Result<IdxEncoding> {
        if self.flags & FLAG_RAW_PCM != 0 {
            Ok(IdxEncoding::RawPcm)
        } else if self.flags & FLAG_IMA != 0 {
            Ok(IdxEncoding::Ima)
        } else {
            Err(Error::Unsupported("IDX entry encoding"))
        }
    }

    /// Decodes this entry from the whole BAG file.
    ///
    /// Raw entries keep only whole frames (a trailing partial frame is
    /// dropped, as XCC's extractor does).
    pub fn decode(&self, bag: &[u8]) -> Result<Pcm> {
        let encoding = self.encoding()?;
        if self.size as usize > MAX_DECOMPRESSED {
            return Err(Error::TooLarge("IDX entry"));
        }
        let data = slice64(bag, u64::from(self.offset), u64::from(self.size))?;
        let channels = self.channels();
        let samples = match encoding {
            IdxEncoding::RawPcm => {
                let frame = 2 * usize::from(channels);
                let whole = data.len() / frame * frame;
                data[..whole]
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|b| i16::from_le_bytes(*b))
                    .collect()
            }
            IdxEncoding::Ima => {
                let block = usize::try_from(self.chunk_size)
                    .map_err(|_| Error::Invalid("IDX block length"))?;
                adpcm::ima_block_decode(data, channels, block)?
            }
        };
        Ok(Pcm {
            sample_rate: self.sample_rate,
            channels,
            samples,
        })
    }
}

/// A parsed IDX file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Idx {
    /// Entries in file order.
    pub entries: Vec<IdxEntry>,
}

impl Idx {
    /// Parses an IDX file. Bytes after the last entry are ignored.
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        if r.array::<4>()? != MAGIC {
            return Err(Error::Invalid("IDX magic"));
        }
        if r.i32()? != VERSION {
            return Err(Error::Unsupported("IDX version"));
        }
        let count = usize::try_from(r.i32()?).map_err(|_| Error::Invalid("IDX entry count"))?;
        // Check the whole table is present before allocating for it.
        let need =
            (HEADER_LEN as u64).saturating_add((count as u64).saturating_mul(ENTRY_LEN as u64));
        if wide(data.len()) < need {
            return Err(Error::Truncated { needed: need });
        }
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            let raw = r.bytes(NAME_LEN)?;
            let end = raw.iter().position(|&b| b == 0).unwrap_or(NAME_LEN);
            entries.push(IdxEntry {
                name: String::from_utf8_lossy(&raw[..end]).into_owned(),
                offset: r.u32()?,
                size: r.u32()?,
                sample_rate: r.u32()?,
                flags: r.u32()?,
                chunk_size: r.u32()?,
            });
        }
        Ok(Self { entries })
    }

    /// The first entry with this name (exact match).
    pub fn find(&self, name: &str) -> Option<&IdxEntry> {
        self.entries.iter().find(|e| e.name == name)
    }
}

/// One sound for [`IdxWriter`].
#[derive(Clone, Debug)]
pub struct IdxSound {
    /// Name, at most 15 bytes.
    pub name: String,
    /// Audio (mono or stereo).
    pub pcm: Pcm,
    /// Storage. IMA uses [`WRITER_BLOCK_PER_CHANNEL`] bytes per channel per
    /// block and pads the audio as [`adpcm::ima_block_encode`] describes.
    pub encoding: IdxEncoding,
}

/// Writes synthetic IDX/BAG pairs.
#[derive(Clone, Debug, Default)]
pub struct IdxWriter {
    /// Sounds, stored in this order.
    pub sounds: Vec<IdxSound>,
}

impl IdxWriter {
    /// Serialises the bank as `(idx, bag)`.
    ///
    /// # Panics
    /// If a name is 16 bytes or longer or contains NUL, a sound has other
    /// than 1 or 2 channels, or the BAG exceeds 4 GiB.
    pub fn write(&self) -> (Vec<u8>, Vec<u8>) {
        let mut idx = Vec::with_capacity(HEADER_LEN + ENTRY_LEN * self.sounds.len());
        idx.extend_from_slice(&MAGIC);
        idx.put_i32(VERSION);
        idx.put_u32(len_u32(self.sounds.len()));
        let mut bag = Vec::new();
        for s in &self.sounds {
            let name = s.name.as_bytes();
            assert!(name.len() < NAME_LEN && !name.contains(&0), "bad IDX name");
            let ch = s.pcm.channels;
            assert!(ch == 1 || ch == 2, "IDX sounds are mono or stereo");
            let stereo = if ch == 2 { FLAG_STEREO } else { 0 };
            let (data, flags, chunk_size) = match s.encoding {
                IdxEncoding::RawPcm => {
                    let data: Vec<u8> =
                        s.pcm.samples.iter().flat_map(|v| v.to_le_bytes()).collect();
                    (data, FLAG_BIT2 | FLAG_RAW_PCM | stereo, 0)
                }
                IdxEncoding::Ima => {
                    let block = WRITER_BLOCK_PER_CHANNEL * u32::from(ch);
                    let (data, _) = adpcm::ima_block_encode(&s.pcm.samples, ch, block as usize);
                    (data, FLAG_BIT2 | FLAG_IMA | stereo, block)
                }
            };
            let mut field = [0u8; NAME_LEN];
            field[..name.len()].copy_from_slice(name);
            idx.extend_from_slice(&field);
            idx.put_u32(len_u32(bag.len()));
            idx.put_u32(len_u32(data.len()));
            idx.put_u32(s.pcm.sample_rate);
            idx.put_u32(flags);
            idx.put_u32(chunk_size);
            bag.extend_from_slice(&data);
        }
        (idx, bag)
    }
}

/// The synthetic bank behind the IDX fixtures: raw and IMA, mono and
/// stereo.
fn fixture_bank() -> IdxWriter {
    let sound = |name: &str, channels, frames, period, encoding| IdxSound {
        name: name.into(),
        pcm: Pcm::synthetic_tone(22050, channels, frames, period, 9000, u64::from(period)),
        encoding,
    };
    IdxWriter {
        sounds: vec![
            sound("tone-raw", 1, 800, 40, IdxEncoding::RawPcm),
            sound("tone-raw-st", 2, 400, 52, IdxEncoding::RawPcm),
            sound("tone-ima", 1, 2500, 64, IdxEncoding::Ima),
            sound("tone-ima-st", 2, 1200, 90, IdxEncoding::Ima),
        ],
    }
}

/// One half of the fixture bank.
struct IdxFixture {
    bag: bool,
}

impl FixtureWriter for IdxFixture {
    fn format(&self) -> &'static str {
        "idx"
    }
    fn name(&self) -> String {
        "bank".into()
    }
    fn extension(&self) -> &'static str {
        if self.bag { "bag" } else { "idx" }
    }
    fn write(&self, out: &mut Vec<u8>) {
        let (idx, bag) = fixture_bank().write();
        out.extend(if self.bag { bag } else { idx });
    }
}

/// Fixture writers for a synthetic IDX/BAG bank (`idx/bank.idx` and
/// `idx/bank.bag`).
pub fn fixture_writers() -> Vec<Box<dyn FixtureWriter>> {
    vec![
        Box::new(IdxFixture { bag: false }),
        Box::new(IdxFixture { bag: true }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synth::SynthRng;

    #[test]
    fn round_trip_raw_and_ima() {
        let bank = fixture_bank();
        let (idx_bytes, bag) = bank.write();
        assert!(idx_bytes.len() < 16 * 1024 && bag.len() < 16 * 1024);
        let idx = Idx::parse(&idx_bytes).unwrap();
        assert_eq!(idx.entries.len(), bank.sounds.len());
        for (e, s) in idx.entries.iter().zip(&bank.sounds) {
            assert_eq!(e.name, s.name);
            assert_eq!(e.flags & FLAG_BIT2, FLAG_BIT2);
            let pcm = e.decode(&bag).unwrap();
            assert_eq!(pcm.channels, s.pcm.channels);
            assert_eq!(pcm.sample_rate, 22050);
            match s.encoding {
                IdxEncoding::RawPcm => assert_eq!(pcm, s.pcm),
                IdxEncoding::Ima => {
                    assert_eq!(e.chunk_size, 512 * u32::from(s.pcm.channels));
                    let block = e.chunk_size as usize;
                    let (_, recon) = adpcm::ima_block_encode(&s.pcm.samples, s.pcm.channels, block);
                    assert_eq!(pcm.samples, recon);
                    // Spans several blocks, and stays close to the input.
                    assert!(e.size as usize > block);
                    for (a, b) in s.pcm.samples.iter().zip(&pcm.samples).skip(200) {
                        assert!((i32::from(*a) - i32::from(*b)).abs() < 3000);
                    }
                }
            }
        }
        assert_eq!(idx.find("tone-ima").unwrap().offset, 1600 + 1600);
        assert!(idx.find("missing").is_none());
    }

    #[test]
    fn hand_built_entries() {
        let mut f = Vec::new();
        f.extend_from_slice(b"GABA");
        f.put_i32(2);
        f.put_i32(2);
        // A 16-byte name with no NUL, raw stereo, 6 bytes (one frame and a
        // half: the half is dropped).
        f.extend_from_slice(b"sixteen-bytes-ab");
        for v in [0, 6, 8000, FLAG_RAW_PCM | FLAG_STEREO, 0] {
            f.put_u32(v);
        }
        // Mono IMA, block length 5: header (sample 3, index 0) then 0x10
        // = codes 0, 1 -> 3 + 0 = 3, 3 + 1 = 4.
        f.extend_from_slice(b"ima\0\0\0\0\0\0\0\0\0\0\0\0\0");
        for v in [6, 5, 8000, FLAG_IMA, 5] {
            f.put_u32(v);
        }
        let idx = Idx::parse(&f).unwrap();
        assert_eq!(idx.entries[0].name, "sixteen-bytes-ab");
        let bag = [1, 0, 0xFF, 0xFF, 9, 9, 3, 0, 0, 0, 0x10];
        assert_eq!(idx.entries[0].decode(&bag).unwrap().samples, [1, -1]);
        assert_eq!(idx.entries[1].decode(&bag).unwrap().samples, [3, 3, 4]);
        // Past the end of the BAG.
        assert_eq!(
            idx.entries[1].decode(&bag[..10]),
            Err(Error::Truncated { needed: 11 })
        );
    }

    #[test]
    fn rejects_bad_headers_and_entries() {
        let (mut f, bag) = fixture_bank().write();
        f[4] = 1;
        assert_eq!(Idx::parse(&f), Err(Error::Unsupported("IDX version")));
        f[4] = 2;
        f[11] = 0x80; // negative count
        assert_eq!(Idx::parse(&f), Err(Error::Invalid("IDX entry count")));
        f[11] = 0x7F; // huge count: rejected before allocating
        assert!(matches!(Idx::parse(&f), Err(Error::Truncated { .. })));
        f[11] = 0;
        f[0] = b'X';
        assert_eq!(Idx::parse(&f), Err(Error::Invalid("IDX magic")));
        f[0] = b'G';
        let mut idx = Idx::parse(&f).unwrap();
        let e = &mut idx.entries[2];
        e.flags = FLAG_BIT2;
        assert_eq!(
            e.decode(&bag),
            Err(Error::Unsupported("IDX entry encoding"))
        );
        e.flags = FLAG_IMA;
        e.chunk_size = 0;
        assert!(e.decode(&bag).is_err());
        e.offset = u32::MAX;
        e.chunk_size = 512;
        assert!(matches!(e.decode(&bag), Err(Error::Truncated { .. })));
    }

    #[test]
    fn every_truncation_is_an_error() {
        let (idx_bytes, bag) = fixture_bank().write();
        for cut in 0..idx_bytes.len() {
            assert!(Idx::parse(&idx_bytes[..cut]).is_err(), "cut {cut}");
        }
        let idx = Idx::parse(&idx_bytes).unwrap();
        let last = idx.entries.last().unwrap();
        let end = (last.offset + last.size) as usize;
        assert_eq!(end, bag.len());
        for cut in (0..bag.len()).step_by(7) {
            assert!(idx.entries.iter().any(|e| e.decode(&bag[..cut]).is_err()));
        }
    }

    #[test]
    fn mutations_never_panic() {
        let (idx_bytes, bag) = fixture_bank().write();
        let mut rng = SynthRng::new(0x1D8);
        let mut decoded = 0;
        for i in 0..3000 {
            let mut f = idx_bytes.clone();
            let mut b = bag.clone();
            for _ in 0..1 + rng.next_u64() % 3 {
                let at = (rng.next_u64() % f.len() as u64) as usize;
                f[at] = rng.next_u64() as u8;
            }
            if i % 2 == 0 {
                let at = (rng.next_u64() % b.len() as u64) as usize;
                b[at] ^= 1 << (rng.next_u64() % 8);
            }
            if let Ok(idx) = Idx::parse(&f) {
                for e in &idx.entries {
                    if let Ok(pcm) = e.decode(&b) {
                        let _ = pcm.to_wav();
                        decoded += 1;
                    }
                }
            }
        }
        assert!(decoded > 100, "{decoded}");
    }
}
