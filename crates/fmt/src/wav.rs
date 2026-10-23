//! RIFF/WAVE files: plain PCM (8-bit unsigned or 16-bit signed) and IMA
//! ADPCM (format code 0x11, the block layout of [`crate::adpcm`]), read
//! whole ([`crate::pcm::Pcm::from_wav`]) or one chunk at a time
//! ([`WavInfo::decode_chunk`]) for long files such as music.
//!
//! The container follows the published RIFF/WAVE layout and Microsoft's
//! IMA ADPCM WAVE format notes (a `fmt ` chunk with the block alignment,
//! 4 bits per sample, an optional `fact` chunk with the true frame count);
//! the block layout is the one XCC's `ima_adpcm_wav_decode.cpp` reads
//! (Copyright (C) 2000 Olaf van der Spek, GPL-3.0-or-later), see
//! [`crate::adpcm::ima_block_decode`]. None of this describes game
//! behaviour.

use crate::adpcm::{IMA_BLOCK_HEADER_LEN, ima_block_decode, ima_block_encode};
use crate::bytes::{PutLe, Reader};
use crate::error::{Error, Result};
use crate::fixture::FixtureWriter;
use crate::limits::MAX_DECOMPRESSED;
use crate::pcm::Pcm;

/// WAVE format code of plain PCM.
const FORMAT_PCM: u16 = 1;
/// WAVE format code of IMA (DVI) ADPCM.
const FORMAT_IMA: u16 = 0x11;
/// Frames in one chunk of [`WavInfo::decode_chunk`] for plain PCM.
const PCM_CHUNK_FRAMES: usize = 4096;

/// How a WAVE file stores its samples.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WavCoding {
    /// Plain PCM, 8-bit unsigned.
    U8,
    /// Plain PCM, 16-bit signed little-endian.
    I16,
    /// IMA ADPCM in blocks of `block_align` bytes.
    Ima {
        /// Bytes per block (the `fmt ` chunk's block alignment).
        block_align: usize,
    },
}

/// What a WAVE file's header says: enough to count its frames and decode
/// it a chunk at a time without decoding the rest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WavInfo {
    /// The sample coding.
    pub coding: WavCoding,
    /// Frames per second (positive).
    pub sample_rate: u32,
    /// 1 (mono) or 2 (stereo).
    pub channels: u16,
    /// Offset of the `data` chunk's body in the file.
    pub data_offset: usize,
    /// Length the `data` chunk claims (the file may hold less).
    pub data_len: usize,
    /// The `fact` chunk's frame count, when one comes before the data
    /// (IMA ADPCM only: it caps the frames the blocks decode to).
    pub fact_frames: Option<u64>,
}

impl WavInfo {
    /// Reads the header of a WAVE file from its first bytes, which must
    /// reach the `data` chunk's header (not its body). Chunks other than
    /// `fmt `, `fact` and `data` are skipped (each padded to an even length,
    /// as RIFF requires). Formats other than plain PCM and IMA ADPCM are
    /// [`Error::Unsupported`].
    pub fn parse(head: &[u8]) -> Result<Self> {
        let mut r = Reader::new(head);
        if r.array::<4>()? != *b"RIFF" {
            return Err(Error::Invalid("not a RIFF file"));
        }
        r.skip(4)?;
        if r.array::<4>()? != *b"WAVE" {
            return Err(Error::Invalid("not a WAVE file"));
        }
        let mut format: Option<(WavCoding, u16, u32)> = None;
        let mut fact = None;
        while r.remaining() >= 8 {
            let id = r.array::<4>()?;
            let len = r.u32()? as usize;
            if &id == b"data" {
                let (coding, channels, sample_rate) =
                    format.ok_or(Error::Invalid("WAVE data before its format"))?;
                return Ok(Self {
                    coding,
                    sample_rate,
                    channels,
                    data_offset: r.pos(),
                    data_len: len,
                    fact_frames: fact,
                });
            }
            let avail = len.min(r.remaining());
            let body = r.bytes(avail)?;
            match &id {
                b"fmt " => format = Some(parse_format(body)?),
                b"fact" if body.len() >= 4 => {
                    let mut f = Reader::new(body);
                    fact = Some(u64::from(f.u32()?));
                }
                _ => {}
            }
            if len % 2 == 1 && r.remaining() > 0 {
                r.skip(1)?;
            }
        }
        Err(Error::Invalid("WAVE file without a data chunk"))
    }

    /// Bytes of each IMA block's channel headers.
    fn header_len(&self) -> usize {
        IMA_BLOCK_HEADER_LEN * usize::from(self.channels)
    }

    /// The data bytes a file of `file_len` bytes holds that decode: the
    /// claimed length cut to the file, then to whole frames (PCM) or with a
    /// last block too short for its headers dropped (IMA ADPCM).
    pub fn data_present(&self, file_len: usize) -> usize {
        let held = self.data_len.min(file_len.saturating_sub(self.data_offset));
        match self.coding {
            WavCoding::U8 => held - held % usize::from(self.channels),
            WavCoding::I16 => held - held % (2 * usize::from(self.channels)),
            WavCoding::Ima { block_align } => {
                let rest = held % block_align;
                if rest != 0 && rest < self.header_len() {
                    held - rest
                } else {
                    held
                }
            }
        }
    }

    /// Frames one IMA block of `len` bytes (at least its headers) decodes
    /// to, as [`ima_block_decode`] reads it.
    fn ima_block_frames(&self, len: usize) -> usize {
        let data = len - self.header_len();
        if self.channels == 1 {
            1 + data * 2
        } else {
            1 + data / 8 * 8
        }
    }

    /// The decoded frames of a file of `file_len` bytes: what
    /// [`Pcm::from_wav`] returns, counted without decoding.
    pub fn frames(&self, file_len: usize) -> u64 {
        let present = self.data_present(file_len);
        match self.coding {
            WavCoding::U8 => (present / usize::from(self.channels)) as u64,
            WavCoding::I16 => (present / (2 * usize::from(self.channels))) as u64,
            WavCoding::Ima { block_align } => {
                let full = present / block_align;
                let rest = present % block_align;
                let mut n = full as u64 * self.ima_block_frames(block_align) as u64;
                if rest != 0 {
                    n += self.ima_block_frames(rest) as u64;
                }
                match self.fact_frames {
                    Some(f) => n.min(f),
                    None => n,
                }
            }
        }
    }

    /// The number of chunks [`WavInfo::decode_chunk`] reads a file of
    /// `file_len` bytes in: IMA blocks, or runs of 4096 PCM frames.
    pub fn chunks(&self, file_len: usize) -> usize {
        let present = self.data_present(file_len);
        match self.coding {
            WavCoding::Ima { block_align } => {
                let blocks = present.div_ceil(block_align);
                // The fact count can end the audio before the last block.
                match self.fact_frames {
                    Some(f) => {
                        let per = self.ima_block_frames(block_align) as u64;
                        blocks.min(usize::try_from(f.div_ceil(per)).unwrap_or(usize::MAX))
                    }
                    None => blocks,
                }
            }
            _ => self.frames(file_len).div_ceil(PCM_CHUNK_FRAMES as u64) as usize,
        }
    }

    /// Appends chunk `k`'s interleaved samples to `out`. `file` is the whole
    /// file (or at least every byte through that chunk). Chunk boundaries
    /// fall on whole frames, so the chunks in order give exactly what
    /// [`Pcm::from_wav`] returns.
    pub fn decode_chunk(&self, file: &[u8], k: usize, out: &mut Vec<i16>) -> Result<()> {
        let present = self.data_present(file.len());
        let data = file
            .get(self.data_offset..self.data_offset + present)
            .ok_or(Error::Invalid("WAVE data outside the file"))?;
        let ch = usize::from(self.channels);
        match self.coding {
            WavCoding::U8 | WavCoding::I16 => {
                let width = if self.coding == WavCoding::U8 { 1 } else { 2 };
                let frame = width * ch;
                let start = k.saturating_mul(PCM_CHUNK_FRAMES * frame);
                let end = start
                    .saturating_add(PCM_CHUNK_FRAMES * frame)
                    .min(data.len());
                let body = data.get(start..end).unwrap_or(&[]);
                if width == 1 {
                    out.extend(body.iter().map(|&b| (i16::from(b) - 128) << 8));
                } else {
                    out.extend(
                        body.as_chunks::<2>()
                            .0
                            .iter()
                            .map(|c| i16::from_le_bytes(*c)),
                    );
                }
            }
            WavCoding::Ima { block_align } => {
                let start = k.saturating_mul(block_align);
                if start >= data.len() {
                    return Ok(());
                }
                let block = &data[start..(start + block_align).min(data.len())];
                let before = out.len();
                out.extend(ima_block_decode(block, self.channels, block_align)?);
                if let Some(f) = self.fact_frames {
                    let first = k as u64 * self.ima_block_frames(block_align) as u64;
                    let keep = f.saturating_sub(first);
                    let have = ((out.len() - before) / ch) as u64;
                    if keep < have {
                        out.truncate(before + keep as usize * ch);
                    }
                }
            }
        }
        Ok(())
    }

    /// Decodes the whole file.
    pub fn decode(&self, file: &[u8]) -> Result<Pcm> {
        let frames = self.frames(file.len());
        let samples = frames.saturating_mul(u64::from(self.channels));
        if samples.saturating_mul(2) > MAX_DECOMPRESSED as u64 {
            return Err(Error::TooLarge("WAVE data"));
        }
        let mut out = Vec::with_capacity(samples as usize);
        for k in 0..self.chunks(file.len()) {
            self.decode_chunk(file, k, &mut out)?;
        }
        Ok(Pcm {
            sample_rate: self.sample_rate,
            channels: self.channels,
            samples: out,
        })
    }
}

/// The coding, channel count and rate of a `fmt ` chunk body.
fn parse_format(body: &[u8]) -> Result<(WavCoding, u16, u32)> {
    let mut f = Reader::new(body);
    let code = f.u16()?;
    let channels = f.u16()?;
    let rate = f.u32()?;
    f.skip(4)?;
    let block_align = usize::from(f.u16()?);
    let bits = f.u16()?;
    if code != FORMAT_PCM && code != FORMAT_IMA {
        return Err(Error::Unsupported("compressed WAVE format"));
    }
    if !(1..=2).contains(&channels) || rate == 0 {
        return Err(Error::Invalid("WAVE channel count or sample rate"));
    }
    let coding = if code == FORMAT_PCM {
        match bits {
            8 => WavCoding::U8,
            16 => WavCoding::I16,
            _ => return Err(Error::Unsupported("WAVE sample width")),
        }
    } else {
        if bits != 4 {
            return Err(Error::Unsupported("IMA ADPCM WAVE sample width"));
        }
        if block_align < IMA_BLOCK_HEADER_LEN * usize::from(channels) {
            return Err(Error::Invalid("IMA ADPCM block smaller than its headers"));
        }
        WavCoding::Ima { block_align }
    };
    Ok((coding, channels, rate))
}

/// Writes `pcm` as an IMA ADPCM WAVE file with blocks of `block_align`
/// bytes and a `fact` chunk holding the input's frame count (for synthetic
/// fixtures and tests). Returns the file and the samples a decoder gives
/// back (the encoder's reconstruction, cut to the input's frames).
///
/// # Panics
/// As [`ima_block_encode`] does for the channel count and block length.
pub fn ima_wav(pcm: &Pcm, block_align: u16) -> (Vec<u8>, Vec<i16>) {
    let ch = usize::from(pcm.channels);
    let whole = &pcm.samples[..pcm.frames() * ch];
    let (body, mut recon) = ima_block_encode(whole, pcm.channels, usize::from(block_align));
    recon.truncate(whole.len());
    let header = IMA_BLOCK_HEADER_LEN * ch;
    let per_block = if ch == 1 {
        1 + (usize::from(block_align) - header) * 2
    } else {
        1 + (usize::from(block_align) - header) / 8 * 8
    };
    let sat = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    let mut out = Vec::new();
    out.extend_from_slice(b"RIFF");
    out.put_u32(sat(4 + 8 + 20 + 8 + 4 + 8 + body.len() + body.len() % 2));
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.put_u32(20);
    out.put_u16(FORMAT_IMA);
    out.put_u16(pcm.channels);
    out.put_u32(pcm.sample_rate);
    let byte_rate = u64::from(pcm.sample_rate) * u64::from(block_align) / (per_block as u64).max(1);
    out.put_u32(u32::try_from(byte_rate).unwrap_or(u32::MAX));
    out.put_u16(block_align);
    out.put_u16(4);
    out.put_u16(2);
    out.put_u16(u16::try_from(per_block).unwrap_or(u16::MAX));
    out.extend_from_slice(b"fact");
    out.put_u32(4);
    out.put_u32(sat(pcm.frames()));
    out.extend_from_slice(b"data");
    out.put_u32(sat(body.len()));
    out.extend_from_slice(&body);
    if body.len() % 2 == 1 {
        out.push(0);
    }
    (out, recon)
}

/// The synthetic WAVE fixtures.
#[derive(Clone, Copy, Debug)]
enum Kind {
    Pcm16Stereo,
    ImaMono,
    ImaStereo,
}

struct WavFixture(Kind);

impl WavFixture {
    fn bytes(&self) -> Vec<u8> {
        match self.0 {
            Kind::Pcm16Stereo => Pcm::synthetic_tone(11025, 2, 700, 50, 9000, 11).to_wav(),
            Kind::ImaMono => ima_wav(&Pcm::synthetic_tone(22050, 1, 1500, 60, 12000, 12), 256).0,
            Kind::ImaStereo => ima_wav(&Pcm::synthetic_tone(22050, 2, 1300, 44, 10000, 13), 512).0,
        }
    }
}

impl FixtureWriter for WavFixture {
    fn format(&self) -> &'static str {
        "wav"
    }
    fn name(&self) -> String {
        match self.0 {
            Kind::Pcm16Stereo => "tone-pcm16-stereo",
            Kind::ImaMono => "tone-ima-mono",
            Kind::ImaStereo => "tone-ima-stereo",
        }
        .into()
    }
    fn extension(&self) -> &'static str {
        "wav"
    }
    fn write(&self, out: &mut Vec<u8>) {
        out.extend(self.bytes());
    }
}

/// Fixture writers for synthetic WAVE files.
pub fn fixture_writers() -> Vec<Box<dyn FixtureWriter>> {
    [Kind::Pcm16Stereo, Kind::ImaMono, Kind::ImaStereo]
        .into_iter()
        .map(|k| Box::new(WavFixture(k)) as Box<dyn FixtureWriter>)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ima_wav_round_trips_through_the_reader() {
        for (channels, block) in [(1u16, 256u16), (1, 36), (2, 512), (2, 16)] {
            for frames in [1usize, 2, 9, 100, 1000, 1017] {
                let pcm = Pcm::synthetic_tone(22050, channels, frames, 40, 9000, 7);
                let (file, recon) = ima_wav(&pcm, block);
                let info = WavInfo::parse(&file).unwrap();
                assert_eq!(
                    info.coding,
                    WavCoding::Ima {
                        block_align: usize::from(block)
                    }
                );
                assert_eq!(info.frames(file.len()), frames as u64);
                let got = Pcm::from_wav(&file).unwrap();
                assert_eq!(got.sample_rate, 22050);
                assert_eq!(got.channels, channels);
                assert_eq!(
                    got.samples, recon,
                    "{channels} ch, block {block}, {frames} frames"
                );
                // The first frame is stored exactly in its block header.
                assert_eq!(
                    got.samples[..usize::from(channels)],
                    pcm.samples[..usize::from(channels)]
                );
            }
        }
    }

    #[test]
    fn chunks_in_order_give_the_whole_decode() {
        let files = [
            Pcm::synthetic_tone(8000, 2, 9000, 40, 9000, 1).to_wav(),
            ima_wav(&Pcm::synthetic_tone(8000, 1, 3001, 40, 9000, 2), 64).0,
            ima_wav(&Pcm::synthetic_tone(8000, 2, 3001, 40, 9000, 3), 72).0,
        ];
        for file in files {
            let info = WavInfo::parse(&file).unwrap();
            let whole = Pcm::from_wav(&file).unwrap();
            let mut joined = Vec::new();
            for k in 0..info.chunks(file.len()) {
                info.decode_chunk(&file, k, &mut joined).unwrap();
            }
            assert_eq!(joined, whole.samples);
            assert_eq!(info.frames(file.len()), whole.frames() as u64);
            // Chunks past the end add nothing.
            let n = joined.len();
            info.decode_chunk(&file, info.chunks(file.len()) + 3, &mut joined)
                .unwrap();
            assert_eq!(joined.len(), n);
        }
    }

    #[test]
    fn header_alone_counts_the_frames() {
        let (file, _) = ima_wav(&Pcm::synthetic_tone(22050, 2, 5000, 40, 9000, 4), 512);
        let info = WavInfo::parse(&file).unwrap();
        let head = &file[..info.data_offset];
        let from_head = WavInfo::parse(head).unwrap();
        assert_eq!(from_head, info);
        assert_eq!(from_head.frames(file.len()), 5000);
    }

    #[test]
    fn truncated_ima_data_counts_what_is_there() {
        let pcm = Pcm::synthetic_tone(11025, 1, 2000, 40, 9000, 5);
        let (file, _) = ima_wav(&pcm, 256);
        let mut info = WavInfo::parse(&file).unwrap();
        // Without the fact count, a cut inside a block keeps its part ...
        info.fact_frames = None;
        let cut = info.data_offset + 256 + 10;
        assert_eq!(info.frames(cut), (1 + 252 * 2 + 1 + 6 * 2) as u64);
        // ... and a last piece shorter than its header is dropped.
        let cut = info.data_offset + 256 + 3;
        assert_eq!(info.frames(cut), (1 + 252 * 2) as u64);
        // The reader decodes exactly those frames (the file's fact count is
        // larger than what the cut file holds, so it does not cap them).
        let mut file2 = file[..cut].to_vec();
        let got = Pcm::from_wav(&file2).unwrap();
        assert_eq!(got.frames(), 1 + 252 * 2);
        file2.truncate(info.data_offset);
        assert_eq!(Pcm::from_wav(&file2).unwrap().frames(), 0);
    }

    #[test]
    fn hostile_ima_headers_are_errors() {
        let (mut file, _) = ima_wav(&Pcm::synthetic_tone(11025, 2, 100, 40, 9000, 6), 512);
        // Block alignment below the two channel headers.
        file[32] = 7;
        file[33] = 0;
        assert!(matches!(Pcm::from_wav(&file), Err(Error::Invalid(_))));
        let (mut file, _) = ima_wav(&Pcm::synthetic_tone(11025, 1, 100, 40, 9000, 6), 64);
        // Bits per sample other than 4.
        file[34] = 8;
        assert!(matches!(Pcm::from_wav(&file), Err(Error::Unsupported(_))));
        // Another compressed format.
        file[20] = 2;
        assert!(matches!(Pcm::from_wav(&file), Err(Error::Unsupported(_))));
    }

    #[test]
    fn fixtures_decode() {
        for w in fixture_writers() {
            let mut out = Vec::new();
            w.write(&mut out);
            let pcm = Pcm::from_wav(&out).unwrap();
            assert!(pcm.frames() > 0, "{}", w.name());
        }
    }
}
