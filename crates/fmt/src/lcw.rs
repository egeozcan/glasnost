//! LCW ("Format80") decompression, plus an encoder for synthetic fixtures.
//!
//! LCW compresses sprite frames and the map overlay packs
//! ([`crate::mappack`]). A stream is a sequence of commands ending with
//! `0x80`:
//!
//! | Command | Meaning |
//! |---|---|
//! | `0x80` | end of stream |
//! | `10cccccc` (`0x81..=0xBF`) | copy `c` literal bytes from the input |
//! | `0cccpppp pppppppp` | copy `c + 3` bytes from `p` (12 bits) bytes back |
//! | `11cccccc` + u16 `p`, `c < 0x3E` | copy `c + 3` bytes from output position `p` |
//! | `0xFE` + u16 `n` + byte `v` | write `n` copies of `v` |
//! | `0xFF` + u16 `n` + u16 `p` | copy `n` bytes from output position `p` |
//!
//! Copies run byte by byte, so a source shortly before the destination
//! repeats a pattern. A copy whose source is at or after the current output
//! position, or a relative distance of 0 or beyond the start, is an error.
//!
//! **Positions.** In the usual form ([`Positions::Absolute`]) the 16-bit
//! `p` of the `11cccccc` and `0xFF` commands is an offset from the start of
//! the output. A variant ([`Positions::Relative`], OpenRA's `reverse` flag)
//! stores it as a distance back from the current output position instead.
//! The 12-bit form is relative in both.
//!
//! The command set follows OpenRA's
//! `OpenRA.Mods.Cnc/FileFormats/LCWCompression.cs` (Copyright (c) The OpenRA
//! Developers and Contributors, GPL-3.0-or-later), XCC's `misc/shp_decode.cpp`
//! (Copyright (C) 2001 Olaf van der Spek, GPL-3.0-or-later) and Vladan Bato's
//! public format notes. The decoder and the greedy encoder are our own code.

use crate::bytes::{PutLe, Reader};
use crate::error::{Error, Result};
use crate::fixture::FixtureWriter;
use crate::limits::MAX_DECOMPRESSED;
use crate::lz::{MatchFinder, copy_back, copy_from, push_fill, push_literals, sample_data};

/// The end-of-stream command.
pub const END: u8 = 0x80;

/// How the 16-bit positions of the long copy commands are stored.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Positions {
    /// Offsets from the start of the output (the common form).
    #[default]
    Absolute,
    /// Distances back from the current output position.
    Relative,
}

/// Command classes, reported to the decoder's trace in tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Op {
    Literals,
    Near,
    Copy,
    Fill,
    LongCopy,
    End,
}

/// Decompresses a whole LCW stream with absolute positions into at most
/// `max_out` bytes (and at most [`MAX_DECOMPRESSED`]).
///
/// The stream must end with [`END`] and nothing may follow it. Errors:
/// [`Error::Truncated`] if the input ends early, [`Error::Invalid`] for a bad
/// copy source or data after the end, [`Error::TooLarge`] if the output
/// would exceed the limit.
pub fn decompress(input: &[u8], max_out: usize) -> Result<Vec<u8>> {
    decompress_with(input, Positions::Absolute, max_out)
}

/// [`decompress`] with a choice of [`Positions`].
pub fn decompress_with(input: &[u8], positions: Positions, max_out: usize) -> Result<Vec<u8>> {
    let (out, used) = decompress_prefix(input, positions, max_out)?;
    if used != input.len() {
        return Err(Error::Invalid("LCW: data after the end command"));
    }
    Ok(out)
}

/// Like [`decompress_with`], but stops at the end command and returns the
/// output together with the number of input bytes consumed (end command
/// included), for streams embedded in a larger buffer.
pub fn decompress_prefix(
    input: &[u8],
    positions: Positions,
    max_out: usize,
) -> Result<(Vec<u8>, usize)> {
    decode(input, positions, max_out, &mut |_| {})
}

fn decode(
    input: &[u8],
    positions: Positions,
    max_out: usize,
    trace: &mut impl FnMut(Op),
) -> Result<(Vec<u8>, usize)> {
    let cap = max_out.min(MAX_DECOMPRESSED);
    let mut r = Reader::new(input);
    let mut out = Vec::new();
    loop {
        let c = r.u8()?;
        if c & 0x80 == 0 {
            trace(Op::Near);
            let len = usize::from(c >> 4) + 3;
            let dist = usize::from(c & 0x0F) << 8 | usize::from(r.u8()?);
            copy_back(&mut out, dist, len, cap)?;
        } else if c & 0x40 == 0 {
            let n = usize::from(c & 0x3F);
            if n == 0 {
                trace(Op::End);
                return Ok((out, r.pos()));
            }
            trace(Op::Literals);
            push_literals(&mut out, r.bytes(n)?, cap)?;
        } else if c == 0xFE {
            trace(Op::Fill);
            let n = usize::from(r.u16()?);
            let v = r.u8()?;
            push_fill(&mut out, v, n, cap)?;
        } else {
            let len = if c == 0xFF {
                trace(Op::LongCopy);
                usize::from(r.u16()?)
            } else {
                trace(Op::Copy);
                usize::from(c & 0x3F) + 3
            };
            let p = usize::from(r.u16()?);
            match positions {
                Positions::Absolute => copy_from(&mut out, p, len, cap)?,
                Positions::Relative => copy_back(&mut out, p, len, cap)?,
            }
        }
    }
}

/// Longest copy or fill a single command holds.
const MAX_COUNT: usize = 0xFFFF;
/// Farthest distance of the 12-bit relative copy.
const NEAR_MAX_DIST: usize = 0x0FFF;
/// Longest relative copy.
const NEAR_MAX_LEN: usize = 10;
/// Longest `11cccccc` copy (`c = 0x3D`).
const COPY_MAX_LEN: usize = 0x3D + 3;
/// Most literals one command holds.
const LITERALS_MAX: usize = 0x3F;

/// Compresses `input` with absolute positions; see [`compress_with`].
pub fn compress(input: &[u8]) -> Vec<u8> {
    compress_with(input, Positions::Absolute)
}

/// Compresses `input` into an LCW stream.
///
/// A greedy encoder of our own: runs of three or more equal bytes become
/// fills; other repeats become 12-bit relative copies when short and near,
/// otherwise `11cccccc` or `0xFF` copies with 16-bit positions stored as
/// `positions` says; everything else is literals. Output is deterministic.
pub fn compress_with(input: &[u8], positions: Positions) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len() / 2 + 16);
    let mut mf = MatchFinder::new(input.len(), 32);
    let (mut lit, mut pos) = (0, 0);
    // Whether a 16-bit position can reach `dist` back from `pos`.
    let wide_ok = |pos: usize, dist: usize| match positions {
        Positions::Absolute => pos - dist <= MAX_COUNT,
        Positions::Relative => dist <= MAX_COUNT,
    };
    while pos < input.len() {
        let run = input[pos..]
            .iter()
            .take(MAX_COUNT)
            .take_while(|&&b| b == input[pos])
            .count();
        let max_dist = match positions {
            Positions::Absolute => pos,
            Positions::Relative => MAX_COUNT,
        };
        let found = mf.find(input, pos, max_dist, MAX_COUNT, |d| {
            d <= NEAR_MAX_DIST || wide_ok(pos, d)
        });
        let step = match found {
            _ if run >= 3 && run >= found.map_or(0, |m| m.1) => {
                put_literals(&mut out, &input[lit..pos]);
                out.push(0xFE);
                out.put_u16(run as u16);
                out.push(input[pos]);
                run
            }
            Some((dist, len)) => {
                put_literals(&mut out, &input[lit..pos]);
                put_copy(&mut out, pos, dist, len, positions, wide_ok(pos, dist))
            }
            None => {
                mf.insert(input, pos);
                pos += 1;
                continue;
            }
        };
        for p in pos..pos + step {
            mf.insert(input, p);
        }
        pos += step;
        lit = pos;
    }
    put_literals(&mut out, &input[lit..]);
    out.push(END);
    out
}

fn put_literals(out: &mut Vec<u8>, lits: &[u8]) {
    for chunk in lits.chunks(LITERALS_MAX) {
        out.push(0x80 | chunk.len() as u8);
        out.extend_from_slice(chunk);
    }
}

/// Emits a copy of up to `len` bytes from `dist` back; returns the length
/// actually encoded.
fn put_copy(
    out: &mut Vec<u8>,
    pos: usize,
    dist: usize,
    len: usize,
    positions: Positions,
    wide_ok: bool,
) -> usize {
    if dist <= NEAR_MAX_DIST && (len <= NEAR_MAX_LEN || !wide_ok) {
        let len = len.min(NEAR_MAX_LEN);
        out.push(((len - 3) << 4 | dist >> 8) as u8);
        out.push(dist as u8);
        return len;
    }
    let p = match positions {
        Positions::Absolute => pos - dist,
        Positions::Relative => dist,
    } as u16;
    if len <= COPY_MAX_LEN {
        out.push(0xC0 | (len - 3) as u8);
    } else {
        out.push(0xFF);
        out.put_u16(len as u16);
    }
    out.put_u16(p);
    len
}

/// A compressed synthetic stream.
struct LcwFixture {
    name: &'static str,
    positions: Positions,
    data: fn() -> Vec<u8>,
}

impl FixtureWriter for LcwFixture {
    fn format(&self) -> &'static str {
        "lcw"
    }
    fn name(&self) -> String {
        self.name.to_owned()
    }
    fn extension(&self) -> &'static str {
        "bin"
    }
    fn write(&self, out: &mut Vec<u8>) {
        out.extend(compress_with(&(self.data)(), self.positions));
    }
}

/// Concentric bands on a 96 × 96 grid: long fills and row repeats.
fn bands() -> Vec<u8> {
    let mut v = Vec::with_capacity(96 * 96);
    for y in 0..96i32 {
        for x in 0..96i32 {
            let (dx, dy) = (x - 48, y - 48);
            v.push(((dx * dx + dy * dy) / 97) as u8);
        }
    }
    v
}

/// Fixture writers for this module: LCW streams of invented content.
pub fn fixture_writers() -> Vec<Box<dyn FixtureWriter>> {
    vec![
        Box::new(LcwFixture {
            name: "sample-absolute",
            positions: Positions::Absolute,
            data: || sample_data(21, 9000),
        }),
        Box::new(LcwFixture {
            name: "sample-relative",
            positions: Positions::Relative,
            data: || sample_data(22, 9000),
        }),
        Box::new(LcwFixture {
            name: "bands",
            positions: Positions::Absolute,
            data: bands,
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synth::SynthRng;
    use std::collections::BTreeSet;

    fn ops(input: &[u8], positions: Positions) -> BTreeSet<Op> {
        let mut seen = BTreeSet::new();
        decode(input, positions, usize::MAX, &mut |op| {
            seen.insert(op);
        })
        .unwrap();
        seen
    }

    fn assert_prefixes_fail(stream: &[u8], positions: Positions) {
        for n in 0..stream.len() {
            assert!(
                decompress_with(&stream[..n], positions, usize::MAX).is_err(),
                "prefix {n}"
            );
        }
    }

    /// The output both hand-made streams decode to.
    fn expected() -> Vec<u8> {
        // Literals, relative copies, a short absolute copy and a fill.
        let mut v = b"abcabcabbbbbbbbbbbbcabzzzzz".to_vec();
        // The 300-byte long copy from position 20 repeats "abzzzzz".
        for _ in 0..42 {
            v.extend_from_slice(b"abzzzzz");
        }
        v.extend_from_slice(b"abzzzz");
        // The 64-byte copy from position 0.
        v.extend_from_slice(b"abcabcabbbbbbbbbbbbcabzzzzz");
        for _ in 0..5 {
            v.extend_from_slice(b"abzzzzz");
        }
        v.extend_from_slice(b"ab");
        v
    }

    #[test]
    fn hand_made_absolute_stream() {
        let s = [
            0x83, b'a', b'b', b'c', // 3 literals -> "abc"
            0x20, 0x03, // relative: len 2+3 = 5 from 3 back (overlaps) -> "abcab"
            0x70, 0x01, // relative: len 7+3 = 10 from 1 back -> ten 'b'
            0xC1, 0x01, 0x00, // len 1+3 = 4 from position 1 -> "bcab"
            0xFE, 0x05, 0x00, b'z', // fill 5 'z'
            0xFF, 0x2C, 0x01, 0x14, 0x00, // 300 bytes from position 20 (7 back)
            0xFD, 0x00, 0x00, // len 0x3D+3 = 64 from position 0
            0x80,
        ];
        let want = expected();
        assert_eq!(want.len(), 327 + 64);
        assert_eq!(decompress(&s, want.len()).unwrap(), want);
        assert_eq!(
            ops(&s, Positions::Absolute),
            BTreeSet::from([
                Op::Literals,
                Op::Near,
                Op::Copy,
                Op::Fill,
                Op::LongCopy,
                Op::End
            ])
        );
        assert_prefixes_fail(&s, Positions::Absolute);
        assert_eq!(
            decompress(&s, want.len() - 1).unwrap_err().kind(),
            "too-large"
        );
    }

    #[test]
    fn hand_made_relative_stream() {
        // Same output; the 16-bit positions are distances back instead.
        let s = [
            0x83, b'a', b'b', b'c', 0x20, 0x03, 0x70, 0x01, //
            0xC1, 0x11, 0x00, // at 18, 17 back -> position 1
            0xFE, 0x05, 0x00, b'z', //
            0xFF, 0x2C, 0x01, 0x07, 0x00, // at 27, 7 back -> position 20
            0xFD, 0x47, 0x01, // at 327, 327 back -> position 0
            0x80,
        ];
        assert_eq!(
            decompress_with(&s, Positions::Relative, 1000).unwrap(),
            expected()
        );
        assert_prefixes_fail(&s, Positions::Relative);
    }

    #[test]
    fn empty_and_zero_counts() {
        assert_eq!(decompress(&[0x80], 0).unwrap(), b"");
        // Zero-length fill and long copy are no-ops.
        assert_eq!(
            decompress(&[0x81, 7, 0xFE, 0, 0, 9, 0xFF, 0, 0, 0, 0, 0x80], 10).unwrap(),
            [7]
        );
    }

    #[test]
    fn rejects_bad_streams() {
        let invalid = |s: &[u8], p| decompress_with(s, p, 100).unwrap_err().kind();
        assert_eq!(decompress(&[], 10), Err(Error::Truncated { needed: 1 }));
        assert_eq!(
            decompress(&[0x82, 1, 2], 10),
            Err(Error::Truncated { needed: 4 })
        );
        // Relative distance 0, and beyond the start.
        assert_eq!(
            invalid(&[0x83, 1, 2, 3, 0x00, 0x00, 0x80], Positions::Absolute),
            "invalid"
        );
        assert_eq!(
            invalid(&[0x83, 1, 2, 3, 0x00, 0x04, 0x80], Positions::Absolute),
            "invalid"
        );
        // Absolute source at the current position.
        assert_eq!(
            invalid(&[0x81, 1, 0xC0, 0x01, 0x00, 0x80], Positions::Absolute),
            "invalid"
        );
        assert_eq!(
            invalid(
                &[0x81, 1, 0xFF, 1, 0, 0x05, 0x00, 0x80],
                Positions::Absolute
            ),
            "invalid"
        );
        // Relative variant: distance 0 and beyond the start.
        assert_eq!(
            invalid(&[0x81, 1, 0xC0, 0x00, 0x00, 0x80], Positions::Relative),
            "invalid"
        );
        assert_eq!(
            invalid(&[0x81, 1, 0xC0, 0x02, 0x00, 0x80], Positions::Relative),
            "invalid"
        );
        // Data after the end command.
        assert_eq!(
            decompress(&[0x80, 0x80], 10),
            Err(Error::Invalid("LCW: data after the end command"))
        );
        assert_eq!(
            decompress_prefix(&[0x80, 0x80], Positions::Absolute, 10),
            Ok((vec![], 1))
        );
        // Fills past the cap.
        let bomb: Vec<u8> = [0xFE, 0xFF, 0xFF, 0].repeat(1000);
        assert_eq!(decompress(&bomb, 1 << 20).unwrap_err().kind(), "too-large");
    }

    fn inputs() -> Vec<Vec<u8>> {
        let mut v = vec![
            vec![],
            vec![1],
            vec![1, 2],
            vec![5; 3],
            b"abababababababab".to_vec(),
            vec![0; 200_000],
            bands(),
        ];
        let mut noise = vec![0u8; 3000];
        SynthRng::new(4).fill(&mut noise);
        v.push(noise.clone());
        // Repeats beyond 4095 and beyond 65535 bytes back.
        for dist in [4095, 4096, 70_000] {
            let mut d = vec![0u8; dist];
            SynthRng::new(dist as u64).fill(&mut d);
            d.extend_from_within(..500);
            v.push(d);
        }
        for seed in 0..6 {
            v.push(sample_data(seed, 2000 + 9000 * seed as usize));
        }
        v
    }

    #[test]
    fn round_trips_both_variants() {
        for positions in [Positions::Absolute, Positions::Relative] {
            for data in inputs() {
                let c = compress_with(&data, positions);
                assert_eq!(
                    decompress_with(&c, positions, data.len()).unwrap(),
                    data,
                    "{positions:?} len {}",
                    data.len()
                );
            }
        }
    }

    #[test]
    fn encoder_uses_every_command() {
        for positions in [Positions::Absolute, Positions::Relative] {
            let mut seen = BTreeSet::new();
            for data in inputs() {
                decode(
                    &compress_with(&data, positions),
                    positions,
                    usize::MAX,
                    &mut |op| {
                        seen.insert(op);
                    },
                )
                .unwrap();
            }
            assert_eq!(seen.len(), 6, "{positions:?}: {seen:?}");
        }
    }

    #[test]
    fn encoded_prefixes_fail() {
        assert_prefixes_fail(&compress(&sample_data(8, 1500)), Positions::Absolute);
    }

    #[test]
    fn mutated_streams_never_panic() {
        let mut rng = SynthRng::new(78);
        let streams = [
            (compress(&sample_data(12, 4000)), Positions::Absolute),
            (
                compress_with(&sample_data(13, 4000), Positions::Relative),
                Positions::Relative,
            ),
            (compress(&bands()), Positions::Absolute),
        ];
        for i in 0..3000 {
            let (s, positions) = &streams[i % streams.len()];
            let mut s = s.clone();
            for _ in 0..1 + rng.next_u64() % 4 {
                let at = (rng.next_u64() % s.len() as u64) as usize;
                s[at] = rng.next_u64() as u8;
            }
            if rng.next_u64().is_multiple_of(4) {
                s.truncate((rng.next_u64() % s.len() as u64) as usize);
            }
            let max = 20_000;
            if let Ok(out) = decompress_with(&s, *positions, max) {
                assert!(out.len() <= max);
            }
        }
    }

    #[test]
    fn fixtures_decode() {
        for (w, positions) in fixture_writers().iter().zip([
            Positions::Absolute,
            Positions::Relative,
            Positions::Absolute,
        ]) {
            let mut out = Vec::new();
            w.write(&mut out);
            assert!(out.len() < 16 * 1024, "{}", w.name());
            assert!(decompress_with(&out, positions, 1 << 16).is_ok());
        }
    }
}
