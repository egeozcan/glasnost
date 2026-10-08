//! ADPCM codecs used by the audio formats.
//!
//! - **IMA ADPCM** (the published IMA/DVI algorithm): a 4-bit code per
//!   sample, low nibble of each byte first. [`ima_decode`] reads a plain
//!   nibble stream (AUD compression type 99); [`ima_block_decode`] reads the
//!   block layout of IDX/BAG banks, where every block restarts the predictor
//!   from a per-channel header.
//! - **Westwood ADPCM** ("WS SND1", AUD compression type 1): command bytes
//!   producing unsigned 8-bit samples ([`westwood_decode`]).
//!
//! Each decoder has an encoder companion for synthetic fixtures only.
//!
//! Ported from XCC's `aud_decode.cpp` and `ima_adpcm_wav_decode.cpp`
//! (Copyright (C) 2000 Olaf van der Spek) and OpenRA's
//! `WestwoodCompressedReader.cs` and `ImaAdpcmReader.cs` (Copyright (c) The
//! OpenRA Developers and Contributors), both GPL-3.0-or-later, rewritten to
//! bound every read and write.

use crate::bytes::Reader;
use crate::error::{Error, Result};
use crate::limits::MAX_DECOMPRESSED;

/// The 89-entry IMA ADPCM step table.
pub const IMA_STEPS: [i32; 89] = [
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60, 66,
    73, 80, 88, 97, 107, 118, 130, 143, 157, 173, 190, 209, 230, 253, 279, 307, 337, 371, 408, 449,
    494, 544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066, 2272,
    2499, 2749, 3024, 3327, 3660, 4026, 4428, 4871, 5358, 5894, 6484, 7132, 7845, 8630, 9493,
    10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350, 22385, 24623, 27086, 29794, 32767,
];

/// IMA ADPCM step-index adjustment, indexed by the code's magnitude bits.
pub const IMA_INDEX_ADJUST: [i32; 8] = [-1, -1, -1, -1, 2, 4, 6, 8];

/// Largest valid IMA ADPCM step index.
pub const IMA_MAX_INDEX: u8 = 88;

/// Length of one channel's header in [`ima_block_decode`]'s layout.
pub const IMA_BLOCK_HEADER_LEN: usize = 4;

/// IMA ADPCM predictor state: the last sample and the step index.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ImaState {
    sample: i16,
    index: u8,
}

impl ImaState {
    /// A state with this predicted sample and step index (at most
    /// [`IMA_MAX_INDEX`]).
    pub fn new(sample: i16, index: u8) -> Result<Self> {
        if index > IMA_MAX_INDEX {
            return Err(Error::Invalid("IMA ADPCM step index above 88"));
        }
        Ok(Self { sample, index })
    }

    /// The predicted (last decoded) sample.
    pub fn sample(&self) -> i16 {
        self.sample
    }

    /// The step index.
    pub fn index(&self) -> u8 {
        self.index
    }

    /// Decodes one 4-bit code (the high nibble of `code` is ignored).
    ///
    /// The delta is built by shifts and adds, as in the published algorithm
    /// and XCC's `aud_decode.cpp`: `step >> 3`, plus `step` for bit 2,
    /// `step >> 1` for bit 1 and `step >> 2` for bit 0; bit 3 is the sign.
    /// OpenRA's `ImaAdpcmReader.cs` computes `step * code / 4 + step / 8`
    /// instead, which rounds differently for some steps; we follow the
    /// shift-and-add form.
    pub fn decode(&mut self, code: u8) -> i16 {
        let step = IMA_STEPS[usize::from(self.index.min(IMA_MAX_INDEX))];
        let mut delta = step >> 3;
        if code & 4 != 0 {
            delta += step;
        }
        if code & 2 != 0 {
            delta += step >> 1;
        }
        if code & 1 != 0 {
            delta += step >> 2;
        }
        let prev = i32::from(self.sample);
        let next = if code & 8 != 0 {
            prev - delta
        } else {
            prev + delta
        };
        self.sample = next.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
        let index = i32::from(self.index) + IMA_INDEX_ADJUST[usize::from(code & 7)];
        self.index = index.clamp(0, i32::from(IMA_MAX_INDEX)) as u8;
        self.sample
    }

    /// Encodes one sample: picks the code whose decoded value approaches
    /// `target`, then advances the state exactly as [`ImaState::decode`]
    /// would. Returns the 4-bit code.
    pub fn encode(&mut self, target: i16) -> u8 {
        let mut diff = i32::from(target) - i32::from(self.sample);
        let mut code = 0u8;
        if diff < 0 {
            code = 8;
            diff = -diff;
        }
        let mut step = IMA_STEPS[usize::from(self.index.min(IMA_MAX_INDEX))];
        for bit in [4u8, 2, 1] {
            if diff >= step {
                code |= bit;
                diff -= step;
            }
            step >>= 1;
        }
        self.decode(code);
        code
    }
}

/// Decodes `count` samples from a nibble stream (low nibble of each byte
/// first), appending them to `out` and advancing `state`.
pub fn ima_decode(
    data: &[u8],
    count: usize,
    state: &mut ImaState,
    out: &mut Vec<i16>,
) -> Result<()> {
    let bytes = count.div_ceil(2);
    let data = data.get(..bytes).ok_or(Error::Truncated {
        needed: bytes as u64,
    })?;
    out.reserve(count);
    for (i, &b) in data.iter().enumerate() {
        out.push(state.decode(b & 0x0F));
        if 2 * i + 1 < count {
            out.push(state.decode(b >> 4));
        }
    }
    Ok(())
}

/// Encodes `samples` as a nibble stream (an odd count leaves the last high
/// nibble zero). Returns the bytes and the samples a decoder reproduces.
pub fn ima_encode(samples: &[i16], state: &mut ImaState) -> (Vec<u8>, Vec<i16>) {
    let mut bytes = Vec::with_capacity(samples.len().div_ceil(2));
    let mut recon = Vec::with_capacity(samples.len());
    for pair in samples.chunks(2) {
        let lo = state.encode(pair[0]);
        recon.push(state.sample);
        let hi = match pair.get(1) {
            Some(&s) => {
                let c = state.encode(s);
                recon.push(state.sample);
                c
            }
            None => 0,
        };
        bytes.push(lo | hi << 4);
    }
    (bytes, recon)
}

fn read_block_header(r: &mut Reader<'_>) -> Result<ImaState> {
    let sample = r.i16()?;
    let index = r.u8()?;
    r.skip(1)?;
    ImaState::new(sample, index)
}

/// Decodes IMA ADPCM in the block layout of IDX/BAG banks (the layout of
/// XCC's `ima_adpcm_wav_decode.cpp`).
///
/// `data` is split into blocks of `block_len` bytes; the last may be
/// shorter. Each block starts with one 4-byte header per channel (`i16`
/// sample, `u8` step index, one reserved byte); the header sample is itself
/// the block's first output frame and seeds the predictor. In mono the rest
/// of the block is a nibble stream. In stereo it alternates 4 bytes (8
/// samples) of the left channel and 4 of the right; trailing bytes that do
/// not form a whole 8-byte group are skipped. (XCC drops them the same
/// way in the last block; in earlier blocks it would lose its place, so
/// valid files keep each stereo block's data a multiple of 8 bytes.)
///
/// Returns interleaved samples.
pub fn ima_block_decode(data: &[u8], channels: u16, block_len: usize) -> Result<Vec<i16>> {
    if !(1..=2).contains(&channels) {
        return Err(Error::Unsupported(
            "IMA ADPCM channel count other than 1 or 2",
        ));
    }
    let ch = usize::from(channels);
    let header_len = IMA_BLOCK_HEADER_LEN * ch;
    if block_len < header_len {
        return Err(Error::Invalid("IMA ADPCM block smaller than its headers"));
    }
    // At most two samples per byte plus one header sample per channel and
    // block; check before allocating.
    let blocks = data.len().div_ceil(block_len);
    let max_samples = data
        .len()
        .saturating_mul(2)
        .saturating_add(blocks.saturating_mul(ch));
    if max_samples.saturating_mul(2) > MAX_DECOMPRESSED {
        return Err(Error::TooLarge("IMA ADPCM sample"));
    }
    let mut out = Vec::with_capacity(max_samples);
    for block in data.chunks(block_len) {
        if block.len() < header_len {
            return Err(Error::Invalid("IMA ADPCM block shorter than its headers"));
        }
        let mut r = Reader::new(block);
        if ch == 1 {
            let mut st = read_block_header(&mut r)?;
            out.push(st.sample);
            let rest = r.bytes(r.remaining())?;
            ima_decode(rest, rest.len() * 2, &mut st, &mut out)?;
        } else {
            let mut left = read_block_header(&mut r)?;
            let mut right = read_block_header(&mut r)?;
            out.push(left.sample);
            out.push(right.sample);
            let rest = r.bytes(r.remaining())?;
            for group in rest.as_chunks::<8>().0 {
                let (l, rr) = group.split_at(4);
                for (&a, &b) in l.iter().zip(rr) {
                    for shift in [0, 4] {
                        out.push(left.decode(a >> shift & 0x0F));
                        out.push(right.decode(b >> shift & 0x0F));
                    }
                }
            }
        }
    }
    Ok(out)
}

/// Encodes interleaved `samples` in the block layout read by
/// [`ima_block_decode`]. Returns the bytes and the interleaved samples a
/// decoder reproduces.
///
/// The layout cannot hold every length: a mono block holds an odd number
/// of frames, a stereo block one plus a multiple of 8. The input is padded
/// by repeating its last frame up to the next length that fits, so the
/// decoded audio may be a few frames longer than the input.
///
/// # Panics
/// If `channels` is not 1 or 2, `samples` is not whole frames, or
/// `block_len` cannot hold the headers plus at least one byte (mono) or
/// one whole 8-byte group (stereo, which also needs `block_len - 8` to be a
/// multiple of 8).
pub fn ima_block_encode(samples: &[i16], channels: u16, block_len: usize) -> (Vec<u8>, Vec<i16>) {
    let ch = usize::from(channels);
    assert!(ch == 1 || ch == 2, "IMA blocks hold 1 or 2 channels");
    assert!(
        samples.len().is_multiple_of(ch),
        "samples must be whole frames"
    );
    let header_len = IMA_BLOCK_HEADER_LEN * ch;
    let data_len = block_len.saturating_sub(header_len);
    if ch == 1 {
        assert!(data_len >= 1, "block too small");
    } else {
        assert!(
            data_len >= 8 && data_len.is_multiple_of(8),
            "stereo block data must be whole 8-byte groups"
        );
    }
    // Frames in a full block.
    let per_block = 1 + data_len * 2 / ch;
    let mut index = 0u8;
    let mut bytes = Vec::new();
    let mut recon = Vec::with_capacity(samples.len());
    let frames: Vec<&[i16]> = samples.chunks(ch).collect();
    for block in frames.chunks(per_block) {
        let mut block = block.to_vec();
        // Pad to a length this layout represents (see above).
        let step = if ch == 1 { 2 } else { 8 };
        while !(block.len() - 1).is_multiple_of(step) {
            block.push(block[block.len() - 1]);
        }
        let mut states = Vec::with_capacity(ch);
        for &s in block[0] {
            bytes.extend_from_slice(&s.to_le_bytes());
            bytes.push(index);
            bytes.push(0);
            recon.push(s);
            states.push(ImaState { sample: s, index });
        }
        if ch == 1 {
            let rest: Vec<i16> = block[1..].iter().map(|f| f[0]).collect();
            let (b, r) = ima_encode(&rest, &mut states[0]);
            bytes.extend(b);
            recon.extend(r);
        } else {
            for group in block[1..].chunks(8) {
                let mut decoded = [[0i16; 8]; 2];
                for c in 0..2 {
                    let chan: Vec<i16> = group.iter().map(|f| f[c]).collect();
                    let (b, r) = ima_encode(&chan, &mut states[c]);
                    bytes.extend(b);
                    decoded[c].copy_from_slice(&r);
                }
                for (&l, &r) in decoded[0].iter().zip(&decoded[1]) {
                    recon.push(l);
                    recon.push(r);
                }
            }
        }
        // The step index carries on into the next block's headers.
        index = states[ch - 1].index;
    }
    (bytes, recon)
}

/// Deltas of the 2-bit Westwood ADPCM codes.
const WS_STEP2: [i32; 4] = [-2, -1, 0, 1];
/// Deltas of the 4-bit Westwood ADPCM codes.
const WS_STEP4: [i32; 16] = [-9, -8, -6, -5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 6, 8];
/// Most output bytes one input byte can produce (a repeat command).
const WS_MAX_EXPANSION: usize = 64;
/// Predictor value at the start of every chunk.
const WS_START: i32 = 0x80;

fn clip8(v: i32) -> i32 {
    v.clamp(0, 255)
}

/// Decodes one Westwood ADPCM chunk of `out_len` unsigned 8-bit samples.
///
/// If `input` is exactly `out_len` bytes the chunk is stored raw. Otherwise
/// it is a sequence of commands; the top two bits of each command byte pick
/// the kind and the low six bits hold a count `n`:
///
/// - 0: `n + 1` bytes follow, each holding four 2-bit deltas (low bits
///   first);
/// - 1: `n + 1` bytes follow, each holding two 4-bit deltas (low nibble
///   first);
/// - 2 with bit 5 of `n` set: the low five bits are a signed delta added
///   to the predictor without clamping (the output byte is the low 8 bits
///   of the result, and the next 2- or 4-bit delta clamps it), as in both
///   XCC and OpenRA;
/// - 2 otherwise: `n + 1` raw sample bytes follow; the last becomes the
///   predictor;
/// - 3: the predictor's sample repeated `n + 1` times.
///
/// The predictor starts at 128 in every chunk. 2- and 4-bit deltas clamp
/// it to 0..=255.
///
/// Errors (rather than stopping early) if the commands would write past
/// `out_len`, need more input than there is, or end before filling
/// `out_len`.
pub fn westwood_decode(input: &[u8], out_len: usize) -> Result<Vec<u8>> {
    if out_len > MAX_DECOMPRESSED {
        return Err(Error::TooLarge("Westwood ADPCM chunk"));
    }
    if input.len() == out_len {
        return Ok(input.to_vec());
    }
    if out_len > input.len().saturating_mul(WS_MAX_EXPANSION) {
        return Err(Error::Invalid(
            "Westwood ADPCM output larger than its input can encode",
        ));
    }
    let mut out = Vec::with_capacity(out_len);
    let room = |out: &Vec<u8>, n: usize| {
        if out_len - out.len() < n {
            Err(Error::Invalid(
                "Westwood ADPCM chunk overruns its output size",
            ))
        } else {
            Ok(())
        }
    };
    let mut sample = WS_START;
    let mut r = Reader::new(input);
    while !r.is_empty() {
        let cmd = r.u8()?;
        let count = usize::from(cmd & 0x3F);
        match cmd >> 6 {
            0 => {
                let codes = r.bytes(count + 1)?;
                room(&out, codes.len() * 4)?;
                for &c in codes {
                    for shift in [0, 2, 4, 6] {
                        sample =
                            clip8(sample.saturating_add(WS_STEP2[usize::from(c >> shift & 3)]));
                        out.push(sample as u8);
                    }
                }
            }
            1 => {
                let codes = r.bytes(count + 1)?;
                room(&out, codes.len() * 2)?;
                for &c in codes {
                    for shift in [0, 4] {
                        sample =
                            clip8(sample.saturating_add(WS_STEP4[usize::from(c >> shift & 0xF)]));
                        out.push(sample as u8);
                    }
                }
            }
            2 if count & 0x20 != 0 => {
                room(&out, 1)?;
                // Sign-extend the low five bits.
                let delta = ((count as i32 & 0x1F) ^ 0x10) - 0x10;
                sample = sample.saturating_add(delta);
                out.push(sample as u8);
            }
            2 => {
                let raw = r.bytes(count + 1)?;
                room(&out, raw.len())?;
                out.extend_from_slice(raw);
                sample = raw.last().map_or(sample, |&b| i32::from(b));
            }
            _ => {
                room(&out, count + 1)?;
                out.extend(std::iter::repeat_n(sample as u8, count + 1));
            }
        }
    }
    if out.len() != out_len {
        return Err(Error::Invalid(
            "Westwood ADPCM chunk ends before its output size",
        ));
    }
    Ok(out)
}

/// Leading samples of `samples` reachable from `prev` one delta from
/// `table` at a time (without clamping), at most `cap`.
fn reachable(samples: &[u8], prev: i32, table: &[i32], cap: usize) -> usize {
    let mut p = prev;
    samples
        .iter()
        .take(cap)
        .take_while(|&&s| {
            let ok = table.contains(&(i32::from(s) - p));
            p = i32::from(s);
            ok
        })
        .count()
}

/// Encodes one chunk of unsigned 8-bit samples losslessly in the command
/// format read by [`westwood_decode`] (for fixtures). Falls back to a raw
/// chunk (the samples themselves) when commands would not be shorter,
/// which also keeps a command stream from being mistaken for a raw chunk.
pub fn westwood_encode(samples: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut prev = WS_START;
    let mut i = 0;
    while i < samples.len() {
        let rest = &samples[i..];
        let repeat = rest
            .iter()
            .take(64)
            .take_while(|&&s| i32::from(s) == prev)
            .count();
        let two = reachable(rest, prev, &WS_STEP2, 256) / 4 * 4;
        let four = reachable(rest, prev, &WS_STEP4, 128) / 2 * 2;
        let jump = i32::from(rest[0]) - prev;
        if repeat >= 2 {
            out.push(0xC0 | (repeat - 1) as u8);
            i += repeat;
        } else if two >= 8 {
            out.push((two / 4 - 1) as u8);
            for q in rest[..two].chunks(4) {
                let mut byte = 0u8;
                for (k, &s) in q.iter().enumerate() {
                    let d = i32::from(s) - prev;
                    let code = WS_STEP2.iter().position(|&x| x == d).unwrap_or(0);
                    byte |= (code as u8) << (2 * k);
                    prev = i32::from(s);
                }
                out.push(byte);
            }
            i += two;
        } else if four >= 4 {
            out.push(0x40 | (four / 2 - 1) as u8);
            for q in rest[..four].chunks(2) {
                let mut byte = 0u8;
                for (k, &s) in q.iter().enumerate() {
                    let d = i32::from(s) - prev;
                    let code = WS_STEP4.iter().position(|&x| x == d).unwrap_or(0);
                    byte |= (code as u8) << (4 * k);
                    prev = i32::from(s);
                }
                out.push(byte);
            }
            i += four;
        } else if (-16..=15).contains(&jump) {
            out.push(0xA0 | (jump & 0x1F) as u8);
            prev = i32::from(rest[0]);
            i += 1;
        } else {
            // A raw run, up to the next sample a short delta can reach.
            let mut n = 1;
            while n < rest.len().min(32)
                && !(-16..=15).contains(&(i32::from(rest[n]) - i32::from(rest[n - 1])))
            {
                n += 1;
            }
            out.push(0x80 | (n - 1) as u8);
            out.extend_from_slice(&rest[..n]);
            prev = i32::from(rest[n - 1]);
            i += n;
        }
    }
    if out.len() >= samples.len() {
        return samples.to_vec();
    }
    out
}

/// A synthetic unsigned 8-bit signal of our own, built from segments that
/// suit each Westwood command: silence, a slow ramp, a stepped wave, jumps
/// and seeded noise. For fixtures and tests.
pub fn synthetic_u8_signal(len: usize, seed: u64) -> Vec<u8> {
    let mut rng = crate::synth::SynthRng::new(seed);
    let mut out = Vec::with_capacity(len);
    let mut v: i32 = 128;
    for i in 0..len {
        match (i / 64) % 5 {
            0 => {}
            1 => v += if (i / 16) % 2 == 0 { 1 } else { -1 },
            2 => v += if (i / 8) % 2 == 0 { 5 } else { -5 },
            3 => v += if i % 2 == 0 { 12 } else { -12 },
            _ => v = (rng.next_u64() % 256) as i32,
        }
        v = v.clamp(0, 255);
        out.push(v as u8);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synth::SynthRng;

    fn st(sample: i16, index: u8) -> ImaState {
        ImaState::new(sample, index).unwrap()
    }

    #[test]
    fn ima_hand_vectors() {
        // From sample 0, index 0 (step 7). Bytes are read low nibble first.
        // 0x10 -> codes 0, 1; 0x42 -> 2, 4; 0x7F -> 15, 7.
        // code 0: delta 7>>3 = 0 -> 0; index 0-1 clamps to 0.
        // code 1: delta 0 + 7>>2 = 1 -> 1; index stays 0.
        // code 2: delta 0 + 7>>1 = 3 -> 4; index 0.
        // code 4: delta 0 + 7 = 7 -> 11; index 0+2 = 2 (step 9).
        // code 15: delta 9>>3 + 9 + 9>>1 + 9>>2 = 1+9+4+2 = 16, negative
        //   -> -5; index 2+8 = 10 (step 19).
        // code 7: delta 19>>3 + 19 + 19>>1 + 19>>2 = 2+19+9+4 = 34 -> 29;
        //   index 18. (OpenRA's 19*7/4 + 19/8 would give 35.)
        let mut s = ImaState::default();
        let mut out = Vec::new();
        ima_decode(&[0x10, 0x42, 0x7F], 6, &mut s, &mut out).unwrap();
        assert_eq!(out, [0, 1, 4, 11, -5, 29]);
        assert_eq!(s.index(), 18);
        assert_eq!(s.sample(), 29);
    }

    #[test]
    fn ima_rounding_differs_from_multiply_form() {
        // Index 0, step 7. Code 7: 7>>3 + 7 + 7>>1 + 7>>2 = 0+7+3+1 = 11
        // (multiply form: 7*7/4 + 7/8 = 12). Code 3: 0+3+1 = 4 (multiply
        // form: 21/4 + 0 = 5).
        let mut s = ImaState::default();
        assert_eq!(s.decode(7), 11);
        let mut s = ImaState::default();
        assert_eq!(s.decode(3), 4);
    }

    #[test]
    fn ima_clamps_sample_and_index() {
        // Step 32767: delta 4095 + 32767 + 16383 + 8191 = 61436, past i16.
        let mut s = st(0, 88);
        assert_eq!(s.decode(7), i16::MAX);
        assert_eq!(s.index(), 88);
        let mut s = st(0, 88);
        assert_eq!(s.decode(15), i16::MIN);
        // Only the low nibble counts.
        let mut s = st(100, 0);
        assert_eq!(s.decode(0xF0), 100);
        assert!(ImaState::new(0, 89).is_err());
    }

    #[test]
    fn ima_odd_count_stops_mid_byte_and_truncation_errors() {
        let mut s = ImaState::default();
        let mut out = Vec::new();
        ima_decode(&[0x10, 0x42], 3, &mut s, &mut out).unwrap();
        assert_eq!(out, [0, 1, 4]);
        assert_eq!(
            ima_decode(&[0x10], 3, &mut s, &mut out),
            Err(Error::Truncated { needed: 2 })
        );
    }

    #[test]
    fn ima_encoder_matches_decoder_and_tracks_signal() {
        let pcm = crate::pcm::Pcm::synthetic_tone(22050, 1, 2001, 80, 12000, 5);
        let mut enc = ImaState::default();
        let (bytes, recon) = ima_encode(&pcm.samples, &mut enc);
        let mut dec = ImaState::default();
        let mut out = Vec::new();
        ima_decode(&bytes, pcm.samples.len(), &mut dec, &mut out).unwrap();
        assert_eq!(out, recon);
        assert_eq!(dec, enc);
        // After the step size adapts, the error stays small.
        for (a, b) in pcm.samples.iter().zip(&out).skip(100) {
            assert!((i32::from(*a) - i32::from(*b)).abs() < 2000, "{a} vs {b}");
        }
    }

    fn header(sample: i16, index: u8) -> [u8; 4] {
        let s = sample.to_le_bytes();
        [s[0], s[1], index, 0]
    }

    #[test]
    fn block_layout_hand_vectors() {
        // Mono, block length 6. Block 1: header (sample 100, index 0), then
        // 0x10 0x00 = codes 0, 1, 0, 0 at step 7: +0 -> 100, +1 -> 101,
        // +0 -> 101, +0 -> 101 (the index stays clamped at 0).
        // Block 2 (the last, 5 bytes): header (sample -7, index 2), then
        // 0x04 = codes 4, 0: step 9 gives 9>>3 + 9 = 10 -> 3 (index 4,
        // step 11); then 11>>3 = 1 -> 4.
        let mut data = Vec::new();
        data.extend(header(100, 0));
        data.extend([0x10, 0x00]);
        data.extend(header(-7, 2));
        data.push(0x04);
        let out = ima_block_decode(&data, 1, 6).unwrap();
        assert_eq!(out, [100, 100, 101, 101, 101, -7, 3, 4]);
    }

    #[test]
    fn block_layout_stereo_interleaves_groups() {
        // Left header (0, index 0), right header (1000, index 0), then one
        // group: 4 bytes of left codes all 0x11 (code 1: +1 each time,
        // index stays 0), 4 bytes of right codes all 0x88 (code 8: delta
        // 7>>3 = 0, so the sample stays 1000). Two trailing bytes are
        // skipped.
        let mut data = Vec::new();
        data.extend(header(0, 0));
        data.extend(header(1000, 0));
        data.extend([0x11; 4]);
        data.extend([0x88; 4]);
        data.extend([0x77; 2]);
        let out = ima_block_decode(&data, 2, data.len()).unwrap();
        let mut want = vec![0, 1000];
        for i in 1..=8 {
            want.extend([i, 1000]);
        }
        assert_eq!(out, want);
    }

    #[test]
    fn block_layout_rejects_bad_parameters() {
        let data = [0u8, 0, 89, 0, 0x11];
        assert_eq!(
            ima_block_decode(&data, 1, 5),
            Err(Error::Invalid("IMA ADPCM step index above 88"))
        );
        assert!(ima_block_decode(&data, 1, 0).is_err());
        assert!(ima_block_decode(&data, 2, 7).is_err());
        assert!(ima_block_decode(&data, 3, 64).is_err());
        // A trailing block shorter than its header.
        let data = [0u8, 0, 0, 0, 0x11, 0, 0];
        assert!(ima_block_decode(&data, 1, 5).is_err());
        assert_eq!(ima_block_decode(&[], 2, 16), Ok(Vec::new()));
    }

    #[test]
    fn block_round_trips_mono_and_stereo() {
        for (channels, block_len, frames) in [
            (1u16, 40usize, 300usize),
            (1, 5, 10),
            (2, 48, 211),
            (2, 16, 9),
        ] {
            let pcm = crate::pcm::Pcm::synthetic_tone(11025, channels, frames, 50, 9000, 2);
            let (bytes, recon) = ima_block_encode(&pcm.samples, channels, block_len);
            let out = ima_block_decode(&bytes, channels, block_len).unwrap();
            assert_eq!(out, recon);
            assert!(out.len() >= pcm.samples.len());
            assert_eq!(&out[..channels as usize], &pcm.samples[..channels as usize]);
        }
    }

    #[test]
    fn westwood_hand_vectors_cover_every_command() {
        // Predictor starts at 128.
        // 0x00 0xE4: 2-bit, one byte; codes (low first) 0,1,2,3 = deltas
        //   -2,-1,0,+1 -> 126, 125, 125, 126.
        // 0x40 0xF0: 4-bit, one byte; nibble 0 = -9 -> 117, nibble 15 = +8
        //   -> 125.
        // 0xBF: 5-bit delta 0x1F = -1 -> 124. 0xAF: 0x0F = +15 -> 139.
        //   0xB0: 0x10 = -16 -> 123.
        // 0x81 0x05 0xFA: raw run of two -> 5, 250; predictor 250.
        // 0xC2: repeat three times -> 250, 250, 250.
        // 0x40 0xFF: 4-bit +8 twice, clamped -> 255, 255.
        let input = [
            0x00, 0xE4, 0x40, 0xF0, 0xBF, 0xAF, 0xB0, 0x81, 0x05, 0xFA, 0xC2, 0x40, 0xFF,
        ];
        let want = [
            126, 125, 125, 126, 117, 125, 124, 139, 123, 5, 250, 250, 250, 250, 255, 255,
        ];
        assert_eq!(westwood_decode(&input, want.len()).unwrap(), want);
    }

    #[test]
    fn westwood_five_bit_delta_is_not_clamped() {
        // 0x80 0xFA: raw 250. 0xAF: +15 -> predictor 265, output byte
        // 265 - 256 = 9. 0xC0: repeat once -> 9. 0x40 0x88: 4-bit delta 0
        // twice clamps 265 -> 255, 255.
        let input = [0x80, 0xFA, 0xAF, 0xC0, 0x40, 0x88];
        assert_eq!(westwood_decode(&input, 5).unwrap(), [250, 9, 9, 255, 255]);
        // And below zero: raw 5, -16 -> -11 (byte 245), 2-bit 0 -> 0.
        // 0x00 0xAA: codes 2,2,2,2 = delta 0 -> clamp to 0 four times.
        let input = [0x80, 0x05, 0xB0, 0x00, 0xAA];
        assert_eq!(westwood_decode(&input, 6).unwrap(), [5, 245, 0, 0, 0, 0]);
    }

    #[test]
    fn westwood_raw_chunk_and_errors() {
        assert_eq!(westwood_decode(&[0xC3, 1, 2], 3).unwrap(), [0xC3, 1, 2]);
        // Repeat of 4 into room for 2.
        assert_eq!(
            westwood_decode(&[0xC3], 2),
            Err(Error::Invalid(
                "Westwood ADPCM chunk overruns its output size"
            ))
        );
        // Raw run of two with one byte present.
        assert_eq!(
            westwood_decode(&[0x81, 0x05], 3),
            Err(Error::Truncated { needed: 3 })
        );
        // Ends short.
        assert!(westwood_decode(&[0xC0], 3).is_err());
        // More output than the input could ever encode.
        assert!(westwood_decode(&[0xFF], 65).is_err());
        assert_eq!(westwood_decode(&[0xFF], 64).unwrap(), [128; 64]);
    }

    /// The command kinds in an encoded stream (0..=3, with 4 for 5-bit
    /// deltas), walking commands without decoding.
    fn command_kinds(stream: &[u8]) -> [bool; 5] {
        let mut seen = [false; 5];
        let mut i = 0;
        while i < stream.len() {
            let cmd = stream[i];
            let n = usize::from(cmd & 0x3F) + 1;
            i += 1;
            match cmd >> 6 {
                0 | 1 => {
                    seen[usize::from(cmd >> 6)] = true;
                    i += n;
                }
                2 if cmd & 0x20 != 0 => seen[4] = true,
                2 => {
                    seen[2] = true;
                    i += n;
                }
                _ => seen[3] = true,
            }
        }
        seen
    }

    #[test]
    fn westwood_encoder_round_trips_and_uses_every_command() {
        let signal = synthetic_u8_signal(1024, 9);
        let enc = westwood_encode(&signal);
        assert!(enc.len() < signal.len());
        assert_eq!(command_kinds(&enc), [true; 5]);
        assert_eq!(westwood_decode(&enc, signal.len()).unwrap(), signal);
        // Noise does not compress: falls back to a raw chunk.
        let mut noise = vec![0u8; 100];
        SynthRng::new(4).fill(&mut noise);
        assert_eq!(westwood_encode(&noise), noise);
        assert_eq!(westwood_encode(&[]), Vec::<u8>::new());
    }

    #[test]
    fn westwood_decoder_survives_random_streams() {
        let mut rng = SynthRng::new(77);
        for _ in 0..3000 {
            let mut input = vec![0u8; (rng.next_u64() % 40) as usize];
            rng.fill(&mut input);
            let out_len = (rng.next_u64() % 300) as usize;
            if let Ok(out) = westwood_decode(&input, out_len) {
                assert_eq!(out.len(), out_len);
            }
            let _ = ima_block_decode(
                &input,
                1 + (rng.next_u64() % 2) as u16,
                (rng.next_u64() % 20) as usize,
            );
        }
    }
}
