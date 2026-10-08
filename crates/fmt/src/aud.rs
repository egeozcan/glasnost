//! AUD sound files.
//!
//! A 12-byte little-endian header (`u16` sample rate, `u32` compressed size
//! counting everything after the header, `u32` decoded size in bytes of 8-
//! or 16-bit PCM, `u8` flags, `u8` compression type), then chunks. Each
//! chunk has an 8-byte header (`u16` compressed size, `u16` decoded size,
//! `u32` [`CHUNK_ID`]) followed by its data.
//!
//! - Compression 1 ([`AudCompression::Westwood`]): each chunk is decoded on
//!   its own by [`adpcm::westwood_decode`] into unsigned 8-bit samples.
//! - Compression 99 ([`AudCompression::Ima`]): an IMA ADPCM nibble stream
//!   whose predictor starts at sample 0, index 0 and carries across chunks.
//!   Each chunk decodes `decoded size / 2` samples, so the last byte of a
//!   chunk may hold only one. (XCC uses each chunk's decoded size; OpenRA
//!   instead decodes whole bytes until the file's total decoded size is
//!   reached. Both agree on files whose chunks hold even sample counts
//!   except the last.) The last chunk may state one sample more than its
//!   nibbles hold; real files do this, and the missing sample repeats the
//!   one before it.
//!
//! Supported combinations: Westwood mono or stereo 8-bit (stereo bytes are
//! taken as interleaved), IMA mono 16-bit. Others are
//! [`Error::Unsupported`]. Layout from XCC's `cc_structures.h`,
//! `aud_file.cpp` and `aud_file_write.cpp` (Copyright (C) 2000 Olaf van der
//! Spek) and OpenRA's `AudReader.cs` (Copyright (c) The OpenRA Developers
//! and Contributors), both GPL-3.0-or-later.

use crate::adpcm::{self, ImaState};
use crate::bytes::{PutLe, Reader, len_u16, len_u32, slice};
use crate::error::{Error, Result};
use crate::fixture::FixtureWriter;
use crate::limits::MAX_DECOMPRESSED;
use crate::pcm::Pcm;

/// Length of the file header.
pub const HEADER_LEN: usize = 12;
/// Length of a chunk header.
pub const CHUNK_HEADER_LEN: usize = 8;
/// Tag that every chunk header carries.
pub const CHUNK_ID: u32 = 0x0000_DEAF;
/// Flag: interleaved stereo.
pub const FLAG_STEREO: u8 = 1;
/// Flag: 16-bit samples (otherwise 8-bit).
pub const FLAG_16BIT: u8 = 2;

/// How the sample data is compressed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudCompression {
    /// Westwood ADPCM (type 1), 8-bit output.
    Westwood,
    /// IMA ADPCM (type 99), 16-bit output.
    Ima,
}

impl AudCompression {
    /// The type byte stored in the header.
    pub fn code(self) -> u8 {
        match self {
            Self::Westwood => 1,
            Self::Ima => 99,
        }
    }

    fn from_code(code: u8) -> Result<Self> {
        match code {
            1 => Ok(Self::Westwood),
            99 => Ok(Self::Ima),
            _ => Err(Error::Unsupported("AUD compression type")),
        }
    }
}

/// The file header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AudHeader {
    /// Frames per second.
    pub sample_rate: u16,
    /// Bytes after the header (chunk headers included).
    pub compressed_size: u32,
    /// Bytes of decoded 8- or 16-bit PCM.
    pub output_size: u32,
    /// Raw flags byte ([`FLAG_STEREO`], [`FLAG_16BIT`]).
    pub flags: u8,
    /// Compression type.
    pub compression: AudCompression,
}

impl AudHeader {
    /// Reads and checks the header at the start of `data`.
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        let sample_rate = r.u16()?;
        let compressed_size = r.u32()?;
        let output_size = r.u32()?;
        let flags = r.u8()?;
        let compression = AudCompression::from_code(r.u8()?)?;
        let h = Self {
            sample_rate,
            compressed_size,
            output_size,
            flags,
            compression,
        };
        if sample_rate == 0 {
            return Err(Error::Invalid("AUD sample rate is zero"));
        }
        match (compression, h.sixteen_bit(), h.stereo()) {
            (AudCompression::Westwood, false, _) | (AudCompression::Ima, true, false) => {}
            (AudCompression::Westwood, true, _) => {
                return Err(Error::Unsupported("16-bit Westwood ADPCM AUD"));
            }
            (AudCompression::Ima, false, _) => {
                return Err(Error::Unsupported("8-bit IMA ADPCM AUD"));
            }
            (AudCompression::Ima, true, true) => {
                return Err(Error::Unsupported("stereo IMA ADPCM AUD"));
            }
        }
        Ok(h)
    }

    /// Whether the audio is stereo.
    pub fn stereo(&self) -> bool {
        self.flags & FLAG_STEREO != 0
    }

    /// Whether decoded samples are 16-bit.
    pub fn sixteen_bit(&self) -> bool {
        self.flags & FLAG_16BIT != 0
    }

    /// Channel count.
    pub fn channels(&self) -> u16 {
        if self.stereo() { 2 } else { 1 }
    }
}

/// One chunk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AudChunk<'a> {
    /// Decoded size in bytes.
    pub output_size: u16,
    /// Compressed data.
    pub data: &'a [u8],
}

/// A parsed AUD file, borrowing its chunk data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Aud<'a> {
    /// The header.
    pub header: AudHeader,
    /// Chunks in file order.
    pub chunks: Vec<AudChunk<'a>>,
}

impl<'a> Aud<'a> {
    /// Parses the header and chunk list.
    ///
    /// The chunks must exactly fill the header's compressed size, their
    /// decoded sizes must add up to the header's decoded size, and each
    /// chunk's decoded size must be one its data can produce. Bytes after
    /// the compressed data are ignored.
    pub fn parse(data: &'a [u8]) -> Result<Self> {
        let header = AudHeader::parse(data)?;
        let body = slice(data, HEADER_LEN, header.compressed_size as usize)?;
        let ima = header.compression == AudCompression::Ima;
        if ima && !header.output_size.is_multiple_of(2) {
            return Err(Error::Invalid("AUD 16-bit output size is odd"));
        }
        let mut r = Reader::new(body);
        let mut chunks = Vec::new();
        let mut total: u64 = 0;
        let past = |_| Error::Invalid("AUD chunk extends past the compressed data");
        while !r.is_empty() {
            let last_fits = |input: u32, output: u32| {
                ima && output.is_multiple_of(2) && output == input * 4 + 2
            };
            let size_in = r.u16().map_err(past)?;
            let output_size = r.u16().map_err(past)?;
            if r.u32().map_err(past)? != CHUNK_ID {
                return Err(Error::Invalid("AUD chunk header tag"));
            }
            let chunk = r.bytes(usize::from(size_in)).map_err(past)?;
            let (input, output) = (u32::from(size_in), u32::from(output_size));
            let fits = if ima {
                output.is_multiple_of(2) && output <= input * 4
                    || (r.is_empty() && last_fits(input, output))
            } else {
                input == output || output <= input * 64
            };
            if !fits {
                return Err(Error::Invalid(
                    "AUD chunk output size too large for its data",
                ));
            }
            total += u64::from(output_size);
            chunks.push(AudChunk {
                output_size,
                data: chunk,
            });
        }
        if total != u64::from(header.output_size) {
            return Err(Error::Invalid(
                "AUD chunk output sizes do not add up to the header's",
            ));
        }
        Ok(Self { header, chunks })
    }

    /// Decodes to 16-bit PCM. Westwood's unsigned 8-bit samples `s` become
    /// `(s - 128) * 256`.
    pub fn decode(&self) -> Result<Pcm> {
        let total: usize = self
            .chunks
            .iter()
            .map(|c| usize::from(c.output_size))
            .fold(0, usize::saturating_add);
        if total > MAX_DECOMPRESSED {
            return Err(Error::TooLarge("AUD output"));
        }
        let mut samples = Vec::with_capacity(total);
        match self.header.compression {
            AudCompression::Westwood => {
                for c in &self.chunks {
                    // A short command stream is bad data, not a short file:
                    // `Truncated` would point inside the chunk.
                    let bytes = adpcm::westwood_decode(c.data, usize::from(c.output_size))
                        .map_err(|e| match e {
                            Error::Truncated { .. } => {
                                Error::Invalid("AUD Westwood chunk commands run past its data")
                            }
                            other => other,
                        })?;
                    samples.extend(bytes.iter().map(|&s| (i16::from(s) - 128) << 8));
                }
            }
            AudCompression::Ima => {
                let mut state = ImaState::default();
                for c in &self.chunks {
                    let n = usize::from(c.output_size) / 2;
                    // Only the last chunk may state one sample more than
                    // its nibbles hold (checked in `parse`).
                    let held = n.min(c.data.len() * 2);
                    adpcm::ima_decode(c.data, held, &mut state, &mut samples)
                        .map_err(|_| Error::Invalid("AUD IMA chunk too short"))?;
                    if held < n {
                        samples.push(state.sample());
                    }
                }
            }
        }
        Ok(Pcm {
            sample_rate: u32::from(self.header.sample_rate),
            channels: self.header.channels(),
            samples,
        })
    }
}

/// Writes synthetic AUD files.
#[derive(Clone, Debug)]
pub struct AudWriter {
    /// Compression to use.
    pub compression: AudCompression,
    /// The audio. Westwood quantises each sample to 8 bits (`s / 256 +
    /// 128`, rounding down), so samples that are multiples of 256 survive
    /// exactly.
    pub pcm: Pcm,
    /// Frames per chunk (XCC writes 1024 for IMA).
    pub chunk_frames: usize,
}

impl AudWriter {
    /// Serialises the file.
    ///
    /// # Panics
    /// If the sample rate exceeds 65535, the combination is unsupported
    /// (IMA needs mono; channels must be 1 or 2), `chunk_frames` is zero or
    /// odd for IMA, or a chunk exceeds the 16-bit size fields.
    pub fn write(&self) -> Vec<u8> {
        let ch = usize::from(self.pcm.channels);
        assert!(ch == 1 || ch == 2, "AUD holds 1 or 2 channels");
        assert!(self.chunk_frames > 0, "chunk_frames must be positive");
        let ima = self.compression == AudCompression::Ima;
        assert!(!ima || ch == 1, "IMA AUD is mono");
        assert!(
            !ima || self.chunk_frames.is_multiple_of(2),
            "IMA chunks hold whole bytes"
        );
        let rate = len_u16(self.pcm.sample_rate as usize);
        let mut body = Vec::new();
        let mut output_size = 0usize;
        let mut state = ImaState::default();
        for chunk in self.pcm.samples.chunks(self.chunk_frames * ch) {
            let (data, out_len) = if ima {
                (adpcm::ima_encode(chunk, &mut state).0, chunk.len() * 2)
            } else {
                let bytes: Vec<u8> = chunk.iter().map(|&s| ((s >> 8) + 128) as u8).collect();
                (adpcm::westwood_encode(&bytes), bytes.len())
            };
            body.put_u16(len_u16(data.len()));
            body.put_u16(len_u16(out_len));
            body.put_u32(CHUNK_ID);
            body.extend_from_slice(&data);
            output_size += out_len;
        }
        let mut out = Vec::with_capacity(HEADER_LEN + body.len());
        out.put_u16(rate);
        out.put_u32(len_u32(body.len()));
        out.put_u32(len_u32(output_size));
        let stereo = if ch == 2 { FLAG_STEREO } else { 0 };
        out.push(stereo | if ima { FLAG_16BIT } else { 0 });
        out.push(self.compression.code());
        out.extend_from_slice(&body);
        out
    }
}

/// Synthetic AUD fixture contents.
#[derive(Clone, Copy, Debug)]
enum Kind {
    ToneIma,
    MixedWestwood,
    StereoWestwood,
}

struct AudFixture(Kind);

impl AudFixture {
    fn writer(&self) -> AudWriter {
        let widen = |b: Vec<u8>| b.into_iter().map(|s| (i16::from(s) - 128) << 8).collect();
        match self.0 {
            Kind::ToneIma => AudWriter {
                compression: AudCompression::Ima,
                pcm: Pcm::synthetic_tone(22050, 1, 3000, 64, 10000, 1),
                chunk_frames: 1024,
            },
            Kind::MixedWestwood => AudWriter {
                compression: AudCompression::Westwood,
                pcm: Pcm {
                    sample_rate: 22050,
                    channels: 1,
                    samples: widen(adpcm::synthetic_u8_signal(4000, 2)),
                },
                chunk_frames: 1024,
            },
            Kind::StereoWestwood => AudWriter {
                compression: AudCompression::Westwood,
                pcm: Pcm {
                    sample_rate: 11025,
                    channels: 2,
                    samples: widen(adpcm::synthetic_u8_signal(3000, 3)),
                },
                chunk_frames: 600,
            },
        }
    }
}

impl FixtureWriter for AudFixture {
    fn format(&self) -> &'static str {
        "aud"
    }
    fn name(&self) -> String {
        match self.0 {
            Kind::ToneIma => "tone-ima",
            Kind::MixedWestwood => "mixed-westwood",
            Kind::StereoWestwood => "stereo-westwood",
        }
        .into()
    }
    fn extension(&self) -> &'static str {
        "aud"
    }
    fn write(&self, out: &mut Vec<u8>) {
        out.extend(self.writer().write());
    }
}

/// Fixture writers for synthetic AUD files.
pub fn fixture_writers() -> Vec<Box<dyn FixtureWriter>> {
    [Kind::ToneIma, Kind::MixedWestwood, Kind::StereoWestwood]
        .into_iter()
        .map(|k| Box::new(AudFixture(k)) as Box<dyn FixtureWriter>)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synth::SynthRng;

    fn fixture_bytes() -> Vec<Vec<u8>> {
        fixture_writers()
            .iter()
            .map(|w| {
                let mut out = Vec::new();
                w.write(&mut out);
                out
            })
            .collect()
    }

    #[test]
    fn westwood_round_trip_is_exact() {
        for kind in [Kind::MixedWestwood, Kind::StereoWestwood] {
            let w = AudFixture(kind).writer();
            let bytes = w.write();
            let aud = Aud::parse(&bytes).unwrap();
            assert!(aud.chunks.len() > 1);
            assert_eq!(aud.header.compression, AudCompression::Westwood);
            assert_eq!(aud.decode().unwrap(), w.pcm);
        }
    }

    #[test]
    fn ima_round_trip_matches_encoder() {
        let w = AudFixture(Kind::ToneIma).writer();
        let bytes = w.write();
        let aud = Aud::parse(&bytes).unwrap();
        assert_eq!(aud.chunks.len(), 3);
        assert_eq!(aud.header.output_size, 6000);
        let pcm = aud.decode().unwrap();
        let (_, recon) = adpcm::ima_encode(&w.pcm.samples, &mut ImaState::default());
        assert_eq!(pcm.samples, recon);
        assert_eq!(pcm.sample_rate, 22050);
        assert_eq!(pcm.channels, 1);
    }

    #[test]
    fn ima_odd_last_chunk_uses_half_a_byte() {
        let w = AudWriter {
            compression: AudCompression::Ima,
            pcm: Pcm::synthetic_tone(8000, 1, 7, 8, 3000, 1),
            chunk_frames: 4,
        };
        let bytes = w.write();
        let aud = Aud::parse(&bytes).unwrap();
        assert_eq!(aud.chunks[1].data.len(), 2);
        assert_eq!(aud.chunks[1].output_size, 6);
        assert_eq!(aud.decode().unwrap().samples.len(), 7);
    }

    #[test]
    fn hand_built_file() {
        // Westwood, 8000 Hz mono. One chunk: 0x41 0x88 0x88 = two 4-bit
        // bytes of zero deltas -> four samples of 128 -> PCM 0. Second chunk
        // stored raw (2 in, 2 out): 0x00, 0xFF -> PCM -32768, 32512.
        let mut f = Vec::new();
        f.put_u16(8000);
        f.put_u32(8 + 3 + 8 + 2);
        f.put_u32(6);
        f.extend([0, 1]);
        f.extend([3, 0, 4, 0, 0xAF, 0xDE, 0, 0, 0x41, 0x88, 0x88]);
        f.extend([2, 0, 2, 0, 0xAF, 0xDE, 0, 0, 0x00, 0xFF]);
        let pcm = Aud::parse(&f).unwrap().decode().unwrap();
        assert_eq!(pcm.samples, [0, 0, 0, 0, -32768, 32512]);
    }

    #[test]
    fn short_command_stream_is_invalid_not_truncated() {
        // One chunk, 2 bytes in, 3 out: a raw run of two with one byte
        // present. The file itself is complete.
        let mut f = Vec::new();
        f.put_u16(8000);
        f.put_u32(8 + 2);
        f.put_u32(3);
        f.extend([0, 1]);
        f.extend([2, 0, 3, 0, 0xAF, 0xDE, 0, 0, 0x81, 0x05]);
        assert_eq!(
            Aud::parse(&f).unwrap().decode(),
            Err(Error::Invalid(
                "AUD Westwood chunk commands run past its data"
            ))
        );
    }

    #[test]
    fn odd_stereo_output_exports_whole_frames() {
        // Stereo Westwood with 3 output bytes: one and a half frames.
        let mut f = Vec::new();
        f.put_u16(8000);
        f.put_u32(8 + 3);
        f.put_u32(3);
        f.extend([FLAG_STEREO, 1]);
        f.extend([3, 0, 3, 0, 0xAF, 0xDE, 0, 0, 0x80, 0x80, 0x80]);
        let pcm = Aud::parse(&f).unwrap().decode().unwrap();
        assert_eq!(pcm.samples.len(), 3);
        assert_eq!(pcm.frames(), 1);
        assert_eq!(pcm.to_wav().len(), crate::pcm::WAV_HEADER_LEN + 4);
    }

    #[test]
    fn rejects_bad_headers() {
        let mut f = AudFixture(Kind::ToneIma).writer().write();
        f[11] = 7;
        assert_eq!(
            Aud::parse(&f),
            Err(Error::Unsupported("AUD compression type"))
        );
        f[11] = 99;
        f[10] = 3;
        assert!(matches!(Aud::parse(&f), Err(Error::Unsupported(_))));
        f[10] = 0;
        assert!(matches!(Aud::parse(&f), Err(Error::Unsupported(_))));
        f[10] = 2;
        f[16] = 0; // chunk tag
        assert_eq!(Aud::parse(&f), Err(Error::Invalid("AUD chunk header tag")));
        let mut f = AudFixture(Kind::ToneIma).writer().write();
        f[6] ^= 2; // output size
        assert!(Aud::parse(&f).is_err());
    }

    #[test]
    fn every_truncation_is_an_error() {
        for bytes in fixture_bytes() {
            for cut in 0..bytes.len() {
                assert!(Aud::parse(&bytes[..cut]).is_err(), "cut {cut}");
            }
        }
    }

    #[test]
    fn mutations_never_panic() {
        let fixtures = fixture_bytes();
        let mut rng = SynthRng::new(0xA0D);
        let mut decoded = 0;
        for i in 0..3000 {
            let mut f = fixtures[i % fixtures.len()].clone();
            for _ in 0..1 + rng.next_u64() % 4 {
                let at = (rng.next_u64() % f.len() as u64) as usize;
                f[at] ^= 1 << (rng.next_u64() % 8);
            }
            if i % 7 == 0 {
                let at = (rng.next_u64() % 40) as usize;
                f[at] = rng.next_u64() as u8;
            }
            if let Ok(aud) = Aud::parse(&f)
                && let Ok(pcm) = aud.decode()
            {
                let _ = pcm.to_wav();
                decoded += 1;
            }
        }
        // Some mutations land in sample data and still decode.
        assert!(decoded > 100, "{decoded}");
    }

    #[test]
    fn fixtures_are_small() {
        for f in fixture_bytes() {
            assert!(f.len() < 16 * 1024, "{}", f.len());
        }
    }

    fn ima_file(chunks: &[(&[u8], u16)]) -> Vec<u8> {
        let body: usize = chunks.iter().map(|(d, _)| 8 + d.len()).sum();
        let out: u32 = chunks.iter().map(|&(_, o)| u32::from(o)).sum();
        let mut f = Vec::new();
        f.put_u16(22050);
        f.put_u32(len_u32(body));
        f.put_u32(out);
        f.extend_from_slice(&[FLAG_16BIT, 99]);
        for (d, o) in chunks {
            f.put_u16(len_u16(d.len()));
            f.put_u16(*o);
            f.put_u32(CHUNK_ID);
            f.extend_from_slice(d);
        }
        f
    }

    #[test]
    fn last_ima_chunk_may_state_one_extra_sample() {
        // Chunk 1: 2 bytes, 4 samples. Chunk 2: 1 byte holds 2 samples but
        // states 3; the third repeats the second.
        let file = ima_file(&[(&[0x11, 0x22], 8), (&[0x33], 6)]);
        let pcm = Aud::parse(&file).unwrap().decode().unwrap();
        assert_eq!(pcm.samples.len(), 7);
        assert_eq!(pcm.samples[6], pcm.samples[5]);
        // The same overstatement on a chunk that is not the last is an error.
        let file = ima_file(&[(&[0x33], 6), (&[0x11, 0x22], 8)]);
        assert!(Aud::parse(&file).is_err());
        // Two samples too many is an error even on the last chunk.
        let file = ima_file(&[(&[0x11, 0x22], 8), (&[0x33], 8)]);
        assert!(Aud::parse(&file).is_err());
    }
}
