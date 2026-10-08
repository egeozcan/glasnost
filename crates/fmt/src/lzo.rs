//! LZO1X decompression, plus a simple encoder for synthetic fixtures.
//!
//! Map files store their terrain pack (`IsoMapPack5`, see
//! [`crate::mappack`]) as blocks of LZO1X data. An LZO1X stream is a
//! sequence of instructions, each a literal run or a back-reference into the
//! output written so far, ending with a marker. How an opcode byte below 16
//! is read depends on how many literals the previous instruction copied, so
//! the decoder tracks that count as its *state* (0, 1–3, or "4 or more"):
//!
//! | Opcode | Meaning |
//! |---|---|
//! | first byte 18..=255 | copy `byte − 17` literals; state = that count (4+ if ≥ 4) |
//! | `0000LLLL`, state 0 | copy `L + 3` literals; `L = 0` adds `15 + 255·zeros + next nonzero byte`; state 4 |
//! | `0000DDSS H`, state 1–3 | copy 2 bytes from distance `1 + D + 4H` |
//! | `0000DDSS H`, state 4 | copy 3 bytes from distance `2049 + D + 4H` |
//! | `01LDDDSS H` / `1LLDDDSS H` (M2) | copy `(op >> 5) + 1` (3–8) bytes from distance `1 + D + 8H` |
//! | `001LLLLL` + LE16 (M3) | copy `L + 2` bytes (L = 0: `33 + 255·zeros + nonzero`) from distance `1 + (LE16 >> 2)` |
//! | `0001HLLL` + LE16 (M4) | copy `L + 2` bytes (L = 0: `9 + 255·zeros + nonzero`) from distance `16384 + 16384·H + (LE16 >> 2)`; distance exactly 16384 ends the stream |
//!
//! After every back-reference, the two low bits `S` of the instruction's
//! last-but-one byte (the opcode for the one-byte-operand forms, the low
//! byte of the LE16 for M3/M4) give 0–3 literals that follow immediately;
//! the state becomes `S`. The canonical end marker is `0x11 0x00 0x00`.
//!
//! The decoder is a safe state-machine rewrite: every input read goes through
//! [`Reader`], every back-reference is checked against the output written so
//! far, copies run byte by byte where source and destination overlap,
//! length-extension loops consume one input byte per step, and the output is
//! capped by the caller's limit and [`MAX_DECOMPRESSED`].
//!
//! Sources: the stream format as described in the Linux kernel's
//! `Documentation/staging/lzo.rst`, and the decoder in OpenRA's
//! `OpenRA.Mods.Cnc/FileFormats/LZOCompression.cs` (Copyright (c) The OpenRA
//! Developers and Contributors, GPL-3.0-or-later), which is a C# port by
//! Frank Razenberg of minilzo 2.06 from the LZO real-time data compression
//! library, Copyright (C) 1996-2011 Markus Franz Xaver Johannes Oberhumer,
//! All Rights Reserved, distributed under the GNU General Public License
//! version 2 or (at your option) any later version
//! (<https://www.oberhumer.com/opensource/lzo/>). The encoder is our own
//! greedy design and makes no attempt to reproduce LZO's output.

use crate::bytes::Reader;
use crate::error::{Error, Result};
use crate::fixture::FixtureWriter;
use crate::limits::MAX_DECOMPRESSED;
use crate::lz::{MatchFinder, copy_back, push_literals, sample_data};

/// The canonical end-of-stream marker: an M4 instruction with distance 16384.
pub const END_MARKER: [u8; 3] = [0x11, 0x00, 0x00];

/// Instruction classes, reported to the decoder's trace in tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Op {
    FirstLiterals,
    LongLiteral,
    LongLiteralExt,
    Near2,
    Far3,
    M2Short,
    M2Long,
    M3,
    M3Ext,
    M4,
    M4Ext,
    TrailingLiterals,
    End,
}

/// Decompresses a whole LZO1X stream into at most `max_out` bytes (and at
/// most [`MAX_DECOMPRESSED`]).
///
/// The stream must end with the end-of-stream marker and nothing may follow
/// it. Errors: [`Error::Truncated`] if the input ends early,
/// [`Error::Invalid`] for a back-reference before the start of the output or
/// data after the marker, [`Error::TooLarge`] if the output would exceed the
/// limit.
pub fn decompress(input: &[u8], max_out: usize) -> Result<Vec<u8>> {
    let (out, used) = decompress_prefix(input, max_out)?;
    if used != input.len() {
        return Err(Error::Invalid("LZO: data after the end-of-stream marker"));
    }
    Ok(out)
}

/// Like [`decompress`], but stops at the end-of-stream marker and returns
/// the output together with the number of input bytes consumed (marker
/// included), for streams embedded in a larger buffer.
pub fn decompress_prefix(input: &[u8], max_out: usize) -> Result<(Vec<u8>, usize)> {
    decode(input, max_out, &mut |_| {})
}

/// Reads a length extension: `base + 255 × (zero bytes) + (first nonzero
/// byte)`. Each step consumes a byte, so the loop ends with the input.
fn extension(r: &mut Reader<'_>, base: usize) -> Result<usize> {
    const OVERFLOW: Error = Error::TooLarge("LZO length extension");
    let mut n = base;
    loop {
        match r.u8()? {
            0 => n = n.checked_add(255).ok_or(OVERFLOW)?,
            b => return n.checked_add(usize::from(b)).ok_or(OVERFLOW),
        }
    }
}

fn decode(input: &[u8], max_out: usize, trace: &mut impl FnMut(Op)) -> Result<(Vec<u8>, usize)> {
    let cap = max_out.min(MAX_DECOMPRESSED);
    let mut r = Reader::new(input);
    let mut out = Vec::new();
    // Literals copied by the previous instruction: 0, 1..=3, or 4 for "four
    // or more".
    let mut state: u8 = 0;

    if let Some(&first) = input.first()
        && first > 17
    {
        r.skip(1)?;
        let n = first - 17;
        push_literals(&mut out, r.bytes(usize::from(n))?, cap)?;
        state = n.min(4);
        trace(Op::FirstLiterals);
    }

    loop {
        let t = r.u8()?;
        let (dist, len, s);
        if t < 16 {
            match state {
                0 => {
                    let n = if t == 0 {
                        trace(Op::LongLiteralExt);
                        extension(&mut r, 15)?
                    } else {
                        trace(Op::LongLiteral);
                        usize::from(t)
                    };
                    let n = n.checked_add(3).ok_or(Error::TooLarge("LZO literal run"))?;
                    push_literals(&mut out, r.bytes(n)?, cap)?;
                    state = 4;
                    continue;
                }
                4 => {
                    trace(Op::Far3);
                    let h = usize::from(r.u8()?);
                    dist = 2049 + usize::from((t >> 2) & 3) + (h << 2);
                    len = 3;
                }
                _ => {
                    trace(Op::Near2);
                    let h = usize::from(r.u8()?);
                    dist = 1 + usize::from((t >> 2) & 3) + (h << 2);
                    len = 2;
                }
            }
            s = t & 3;
        } else if t >= 64 {
            trace(if t >= 128 { Op::M2Long } else { Op::M2Short });
            let h = usize::from(r.u8()?);
            dist = 1 + usize::from((t >> 2) & 7) + (h << 3);
            len = usize::from(t >> 5) + 1;
            s = t & 3;
        } else if t >= 32 {
            let l = match t & 31 {
                0 => {
                    trace(Op::M3Ext);
                    extension(&mut r, 31)?
                }
                l => {
                    trace(Op::M3);
                    usize::from(l)
                }
            };
            len = l
                .checked_add(2)
                .ok_or(Error::TooLarge("LZO match length"))?;
            let d = r.u16()?;
            dist = 1 + usize::from(d >> 2);
            s = (d & 3) as u8;
        } else {
            let l = match t & 7 {
                0 => extension(&mut r, 7)?,
                l => usize::from(l),
            };
            len = l
                .checked_add(2)
                .ok_or(Error::TooLarge("LZO match length"))?;
            let d = r.u16()?;
            let offset = (usize::from(t & 8) << 11) + usize::from(d >> 2);
            if offset == 0 {
                trace(Op::End);
                return Ok((out, r.pos()));
            }
            trace(if t & 7 == 0 { Op::M4Ext } else { Op::M4 });
            dist = 16384 + offset;
            s = (d & 3) as u8;
        }
        copy_back(&mut out, dist, len, cap)?;
        if s > 0 {
            trace(Op::TrailingLiterals);
            push_literals(&mut out, r.bytes(usize::from(s))?, cap)?;
        }
        state = s;
    }
}

/// Longest match the encoder emits (any length is encodable; this only
/// bounds the match search).
const MAX_MATCH: usize = 4096;
/// Farthest distance an M2 instruction reaches.
const M2_MAX_DIST: usize = 2048;
/// Farthest distance an M3 instruction reaches.
const M3_MAX_DIST: usize = 16384;
/// Farthest distance an M4 instruction reaches.
const M4_MAX_DIST: usize = 49151;

/// Compresses `input` into a valid LZO1X stream.
///
/// A greedy hash-chain encoder of our own: it emits the initial
/// `17 + n` literal form, long literal runs, M2/M3/M4 matches with trailing
/// literals in their `S` bits, and the end marker. It never emits the two
/// short `0000DDSS` match forms. Output is deterministic.
pub fn compress(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len() / 2 + 16);
    let mut mf = MatchFinder::new(input.len(), 32);
    // Index of the byte holding the last match's `S` bits.
    let mut s_pos = None;
    let (mut lit, mut pos) = (0, 0);
    while pos < input.len() {
        let found = mf.find(input, pos, M4_MAX_DIST, MAX_MATCH, |_| true);
        mf.insert(input, pos);
        if let Some((dist, len)) = found {
            put_literals(&mut out, &input[lit..pos], &mut s_pos);
            put_match(&mut out, dist, len, &mut s_pos);
            for p in pos + 1..pos + len {
                mf.insert(input, p);
            }
            pos += len;
            lit = pos;
        } else {
            pos += 1;
        }
    }
    put_literals(&mut out, &input[lit..], &mut s_pos);
    out.extend_from_slice(&END_MARKER);
    out
}

/// Appends `base | n` if `n` fits the opcode's length field (`max`),
/// otherwise `base` followed by the zero-byte length extension of `n − max`.
fn put_length(out: &mut Vec<u8>, base: u8, max: usize, n: usize) {
    if n <= max {
        out.push(base | n as u8);
    } else {
        out.push(base);
        let mut rem = n - max;
        while rem > 255 {
            out.push(0);
            rem -= 255;
        }
        out.push(rem as u8);
    }
}

fn put_literals(out: &mut Vec<u8>, lits: &[u8], s_pos: &mut Option<usize>) {
    let n = lits.len();
    if n == 0 {
        return;
    }
    match *s_pos {
        // First instruction: the `17 + n` form.
        None if out.is_empty() && n <= 238 => out.push(17 + n as u8),
        // 1-3 literals after a match ride in its S bits.
        Some(i) if n <= 3 => out[i] |= n as u8,
        // A long literal run, valid in state 0 (after S = 0 or at the start).
        _ => put_length(out, 0, 15, n - 3),
    }
    out.extend_from_slice(lits);
    *s_pos = None;
}

fn put_match(out: &mut Vec<u8>, dist: usize, len: usize, s_pos: &mut Option<usize>) {
    if dist <= M2_MAX_DIST && len <= 8 {
        let d = dist - 1;
        out.push(((len - 1) << 5 | (d & 7) << 2) as u8);
        *s_pos = Some(out.len() - 1);
        out.push((d >> 3) as u8);
    } else if dist <= M3_MAX_DIST {
        let d = dist - 1;
        put_length(out, 0x20, 31, len - 2);
        *s_pos = Some(out.len());
        out.push(((d & 63) << 2) as u8);
        out.push((d >> 6) as u8);
    } else {
        let d = dist - 16384;
        put_length(out, 0x10 | ((d >> 14) << 3) as u8, 7, len - 2);
        let d = d & 0x3FFF;
        *s_pos = Some(out.len());
        out.push(((d & 63) << 2) as u8);
        out.push((d >> 6) as u8);
    }
}

/// A compressed synthetic stream.
struct LzoFixture {
    name: &'static str,
    data: fn() -> Vec<u8>,
}

impl FixtureWriter for LzoFixture {
    fn format(&self) -> &'static str {
        "lzo"
    }
    fn name(&self) -> String {
        self.name.to_owned()
    }
    fn extension(&self) -> &'static str {
        "bin"
    }
    fn write(&self, out: &mut Vec<u8>) {
        out.extend(compress(&(self.data)()));
    }
}

/// Integer ramps of varying period: highly compressible, long matches.
fn ramps() -> Vec<u8> {
    (0..12000u32)
        .map(|i| (i % (7 + i / 1000)) as u8 ^ (i / 3000) as u8)
        .collect()
}

/// Fixture writers for this module: LZO streams of invented content.
pub fn fixture_writers() -> Vec<Box<dyn FixtureWriter>> {
    vec![
        Box::new(LzoFixture {
            name: "sample-1",
            data: || sample_data(1, 12000),
        }),
        Box::new(LzoFixture {
            name: "sample-2",
            data: || sample_data(2, 6000),
        }),
        Box::new(LzoFixture {
            name: "ramps",
            data: ramps,
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synth::SynthRng;
    use std::collections::BTreeSet;

    fn ops(input: &[u8]) -> BTreeSet<Op> {
        let mut seen = BTreeSet::new();
        decode(input, usize::MAX, &mut |op| {
            seen.insert(op);
        })
        .unwrap();
        seen
    }

    /// Every proper prefix fails, without panicking.
    fn assert_prefixes_fail(stream: &[u8]) {
        for n in 0..stream.len() {
            assert!(decompress(&stream[..n], usize::MAX).is_err(), "prefix {n}");
        }
    }

    /// The synthetic pattern used as literal content in [`far_stream`].
    fn pattern(n: usize) -> Vec<u8> {
        (0..n).map(|i| (i * 7 + i / 13) as u8).collect()
    }

    #[test]
    fn first_byte_with_four_literals_then_end() {
        // 0x15: first-byte form, 21 - 17 = 4 literals. Then the end marker.
        let s = [0x15, b'a', b'b', b'c', b'd', 0x11, 0x00, 0x00];
        assert_eq!(decompress(&s, 100).unwrap(), b"abcd");
        assert_eq!(ops(&s), BTreeSet::from([Op::FirstLiterals, Op::End]));
        assert_prefixes_fail(&s);
    }

    #[test]
    fn short_first_run_and_near_matches() {
        let s = [
            0x12, b'x', // first byte 18: 1 literal, state 1 -> "x"
            0x02, 0x00, // state 1, 0000DDSS: D=0 H=0 -> 2 bytes from distance 1 -> "xxx"; S=2
            b'y', b'z', // the 2 S literals -> "xxxyz"
            0x0F, 0x00, // state 2: D=3 H=0 -> distance 4, copies "xx"; S=3
            b'1', b'2', b'3', // -> "xxxyzxx123"
            0x05, 0x01, // state 3: D=1 H=1 -> distance 1+1+4=6, copies "zx"; S=1
            b'q', // -> "xxxyzxx123zxq"; state 1
            0x00, 0x00, // state 1: D=0 H=0 -> distance 1, copies "qq"; S=0
            0x11, 0x00, 0x00,
        ];
        assert_eq!(decompress(&s, 100).unwrap(), b"xxxyzxx123zxqqq");
        assert_eq!(
            ops(&s),
            BTreeSet::from([Op::FirstLiterals, Op::Near2, Op::TrailingLiterals, Op::End])
        );
        assert_prefixes_fail(&s);
    }

    #[test]
    fn literal_runs_m2_m3_and_overlaps() {
        let mut s = vec![0x16]; // first byte 22: 5 literals, state 4
        s.extend_from_slice(b"hello");
        s.extend_from_slice(&[
            0x4C, 0x00, // M2 short class: len (0x4C>>5)+1 = 3, D=3 -> distance 4: "ell"; S=0
            0x02, // state 0 long literal without extension: 2 + 3 = 5 literals
        ]);
        s.extend_from_slice(b"world");
        s.extend_from_slice(&[
            0xE1, 0x00, // M2 long class: len 8, distance 1 (overlapping): "dddddddd"; S=1
            b'!', // S literal
            0x23, 0x56, 0x00, // M3: len 3+2 = 5; LE16 86 -> distance 22 (start): "hello"; S=2
            b'A', b'B', // S literals
            // M3 with extension: L=0, one zero byte, then 1 -> len 2+31+255+1 = 289;
            // LE16 4 -> distance 2, overlapping "AB" repeat; S=0
            0x20, 0x00, 0x01, 0x04, 0x00, 0x11, 0x00, 0x00,
        ]);
        let mut want = b"helloellworlddddddddd!helloAB".to_vec();
        for _ in 0..144 {
            want.extend_from_slice(b"AB");
        }
        want.push(b'A');
        assert_eq!(want.len(), 29 + 289);
        assert_eq!(decompress(&s, 1000).unwrap(), want);
        assert_eq!(
            ops(&s),
            BTreeSet::from([
                Op::FirstLiterals,
                Op::LongLiteral,
                Op::M2Short,
                Op::M2Long,
                Op::M3,
                Op::M3Ext,
                Op::TrailingLiterals,
                Op::End,
            ])
        );
        assert_prefixes_fail(&s);
        assert_eq!(
            decompress(&s, 317),
            Err(Error::TooLarge("decompressed output exceeds the limit"))
        );
    }

    /// A stream exercising the long-literal extension, the 3-byte far match
    /// and M4 in both distance halves, with and without length extension.
    fn far_stream() -> (Vec<u8>, Vec<u8>) {
        let p = pattern(2100);
        let mut s = vec![0x00]; // first byte 0: state 0 long literal with extension
        s.extend_from_slice(&[0; 8]); // 8 zero bytes: +8*255
        s.push(0x2A); // +42: 15 + 2040 + 42 = 2097, + 3 = 2100 literals
        s.extend_from_slice(&p);
        // State 4, 0000DDSS: D=1 H=0 -> 3 bytes from distance 2049+1 = 2050,
        // i.e. from position 50; S=3.
        s.extend_from_slice(&[0x07, 0x00]);
        s.extend_from_slice(b"xyz");
        // M3 with extension: 62 zero bytes, then 157 -> len 33+15810+157 =
        // 16000 from distance 1 (repeats 'z'); S=0. Output 18106.
        s.push(0x20);
        s.extend_from_slice(&[0; 62]);
        s.extend_from_slice(&[157, 0x00, 0x00]);
        // M4 H=0 L=5 -> len 7; LE16 1616<<2 -> distance 18000, position 106.
        s.extend_from_slice(&[0x15, 0x40, 0x19]);
        // M4 H=0 with extension byte 5 -> len 14; LE16 16<<2|2 -> distance
        // 16400, position 1713; S=2.
        s.extend_from_slice(&[0x10, 0x05, 0x42, 0x00]);
        s.extend_from_slice(b"QR");
        // M3 with extension: 58 zero bytes, then 177 -> len 15000, distance 1.
        // Output 33129.
        s.push(0x20);
        s.extend_from_slice(&[0; 58]);
        s.extend_from_slice(&[177, 0x00, 0x00]);
        // M4 H=1 L=1 -> len 3; LE16 61<<2|1 -> distance 32768+61 = 32829,
        // position 300; S=1.
        s.extend_from_slice(&[0x19, 0xF5, 0x00]);
        s.push(b'!');
        // M4 H=1 with extension (0, 1) -> len 2+7+255+1 = 265; LE16 365<<2 ->
        // distance 32768+365 = 33133, position 0.
        s.extend_from_slice(&[0x18, 0x00, 0x01, 0xB4, 0x05]);
        s.extend_from_slice(&END_MARKER);

        let mut want = p.clone();
        want.extend_from_slice(&p[50..53]);
        want.extend_from_slice(b"xyz");
        want.extend(std::iter::repeat_n(b'z', 16000));
        want.extend_from_slice(&p[106..113]);
        want.extend_from_slice(&p[1713..1727]);
        want.extend_from_slice(b"QR");
        want.extend(std::iter::repeat_n(b'R', 15000));
        want.extend_from_slice(&p[300..303]);
        want.push(b'!');
        want.extend_from_slice(&p[0..265]);
        (s, want)
    }

    #[test]
    fn far_matches_and_m4() {
        let (s, want) = far_stream();
        assert_eq!(want.len(), 33398);
        assert_eq!(decompress(&s, want.len()).unwrap(), want);
        assert_eq!(
            ops(&s),
            BTreeSet::from([
                Op::LongLiteralExt,
                Op::Far3,
                Op::M3Ext,
                Op::M4,
                Op::M4Ext,
                Op::TrailingLiterals,
                Op::End,
            ])
        );
        assert_eq!(
            decompress(&s, want.len() - 1).unwrap_err().kind(),
            "too-large"
        );
        assert_prefixes_fail(&s);
    }

    #[test]
    fn long_literal_in_state_zero_after_match() {
        // "abcd" via first byte, M2 len 3 distance 4 ("abc", S=0), then a long
        // literal 0x01 (1 + 3 = 4 literals) in state 0, then the end marker.
        let s = [
            0x15, b'a', b'b', b'c', b'd', 0x4C, 0x00, 0x01, b'w', b'x', b'y', b'z', 0x11, 0x00,
            0x00,
        ];
        assert_eq!(decompress(&s, 64).unwrap(), b"abcdabcwxyz");
    }

    #[test]
    fn end_marker_alone_is_empty() {
        assert_eq!(decompress(&END_MARKER, 0).unwrap(), b"");
        assert_eq!(compress(b""), END_MARKER);
        // An end marker with a length extension is still an end marker.
        assert_eq!(decompress(&[0x10, 0x00, 0x07, 0x00, 0x00], 0).unwrap(), b"");
    }

    #[test]
    fn rejects_bad_streams() {
        assert_eq!(decompress(&[], 10), Err(Error::Truncated { needed: 1 }));
        // Missing end marker.
        assert_eq!(
            decompress(&[0x13, b'a', b'b'], 10),
            Err(Error::Truncated { needed: 4 })
        );
        // 2-byte match from distance 2 with only one byte written.
        assert_eq!(
            decompress(&[0x12, b'x', 0x05, 0x00, 0x11, 0x00, 0x00], 10)
                .unwrap_err()
                .kind(),
            "invalid"
        );
        // After 4 first-byte literals, 0000DDSS is the 2049+ distance form.
        assert_eq!(
            decompress(&[0x15, 1, 2, 3, 4, 0x00, 0x00, 0x11, 0x00, 0x00], 10)
                .unwrap_err()
                .kind(),
            "invalid"
        );
        // M3 reaching before the start.
        assert_eq!(
            decompress(&[0x15, 1, 2, 3, 4, 0x21, 0x10, 0x00, 0x11, 0x00, 0x00], 10)
                .unwrap_err()
                .kind(),
            "invalid"
        );
        // Data after the end marker.
        assert_eq!(
            decompress(&[0x11, 0x00, 0x00, 0x00], 10),
            Err(Error::Invalid("LZO: data after the end-of-stream marker"))
        );
        assert_eq!(
            decompress_prefix(&[0x11, 0x00, 0x00, 0x00], 10),
            Ok((vec![], 3))
        );
        // Output beyond the cap.
        assert_eq!(
            decompress(&[0x15, 1, 2, 3, 4, 0x11, 0x00, 0x00], 3)
                .unwrap_err()
                .kind(),
            "too-large"
        );
    }

    #[test]
    fn long_zero_extension_runs_terminate() {
        let mut s = vec![0x00];
        s.extend(std::iter::repeat_n(0u8, 200_000));
        assert_eq!(decompress(&s, 1 << 20).unwrap_err().kind(), "truncated");
        let mut s = vec![0x15, 1, 2, 3, 4, 0x20];
        s.extend(std::iter::repeat_n(0u8, 200_000));
        assert_eq!(decompress(&s, 1 << 20).unwrap_err().kind(), "truncated");
    }

    fn inputs() -> Vec<Vec<u8>> {
        let mut v = vec![
            vec![],
            vec![7],
            vec![1, 2],
            vec![1, 2, 3],
            b"abcabcabc".to_vec(),
            vec![0; 1000],
            vec![0xAA; 70_000],
            ramps(),
        ];
        let mut noise = vec![0u8; 5000];
        SynthRng::new(9).fill(&mut noise);
        v.push(noise.clone());
        // A literal run longer than the first-byte form allows, then matches.
        let mut lead = noise[..600].to_vec();
        lead.extend_from_slice(&noise[..600]);
        v.push(lead);
        // Repeats at distances around each class boundary.
        for dist in [2048, 2049, 3072, 16384, 16385, 32768, 32769, 49151, 49152] {
            let mut d = vec![0u8; dist + 300];
            SynthRng::new(dist as u64).fill(&mut d[..dist]);
            for i in dist..d.len() {
                d[i] = d[i - dist];
            }
            v.push(d);
        }
        for seed in 0..6 {
            v.push(sample_data(seed, 3000 + 7000 * seed as usize));
        }
        v
    }

    #[test]
    fn round_trips() {
        for data in inputs() {
            let c = compress(&data);
            assert_eq!(
                decompress(&c, data.len()).unwrap(),
                data,
                "len {}",
                data.len()
            );
        }
    }

    #[test]
    fn encoder_uses_every_class_it_claims() {
        let mut seen = BTreeSet::new();
        for data in inputs() {
            decode(&compress(&data), usize::MAX, &mut |op| {
                seen.insert(op);
            })
            .unwrap();
        }
        for op in [
            Op::FirstLiterals,
            Op::LongLiteral,
            Op::LongLiteralExt,
            Op::M2Short,
            Op::M2Long,
            Op::M3,
            Op::M3Ext,
            Op::M4,
            Op::M4Ext,
            Op::TrailingLiterals,
            Op::End,
        ] {
            assert!(seen.contains(&op), "{op:?}");
        }
    }

    #[test]
    fn encoded_prefixes_fail() {
        assert_prefixes_fail(&compress(&sample_data(5, 1500)));
    }

    #[test]
    fn mutated_streams_never_panic() {
        let mut rng = SynthRng::new(77);
        let streams = [
            compress(&sample_data(11, 4000)),
            compress(&ramps()[..3000]),
            far_stream().0,
        ];
        for i in 0..3000 {
            let mut s = streams[i % streams.len()].clone();
            for _ in 0..1 + rng.next_u64() % 4 {
                let at = (rng.next_u64() % s.len() as u64) as usize;
                s[at] = rng.next_u64() as u8;
            }
            if rng.next_u64().is_multiple_of(4) {
                s.truncate((rng.next_u64() % s.len() as u64) as usize);
            }
            let max = 40_000;
            if let Ok(out) = decompress(&s, max) {
                assert!(out.len() <= max);
            }
        }
    }

    #[test]
    fn fixtures_decode() {
        for w in fixture_writers() {
            let mut out = Vec::new();
            w.write(&mut out);
            assert!(out.len() < 16 * 1024, "{}", w.name());
            assert!(decompress(&out, 1 << 16).is_ok());
        }
    }
}
