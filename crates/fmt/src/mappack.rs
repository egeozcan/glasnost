//! Map packs: the base64, block-compressed binary sections of map files.
//!
//! Several map sections (`IsoMapPack5`, `OverlayPack`, `OverlayDataPack`)
//! hold binary data as base64 text split over numbered keys. Decoding is:
//!
//! 1. the caller extracts the section's values in key order (this crate
//!    does not parse INI) and [`decode_base64`] joins and decodes them;
//! 2. [`unpack`] splits the bytes into blocks — each a little-endian
//!    `u16` compressed size, a `u16` uncompressed size, then the compressed
//!    bytes — and decompresses each with the section's [`PackCodec`];
//! 3. for `IsoMapPack5`, [`iso_tiles`] reads the result as [`IsoTile`]
//!    records.
//!
//! [`pack`], [`encode_base64_lines`] and [`write_iso_tiles`] are the
//! matching writers, used for synthetic fixtures.
//!
//! Layouts follow OpenRA's `OpenRA.Mods.Cnc/UtilityCommands/ImportGen2MapCommand.cs`
//! (Copyright (c) The OpenRA Developers and Contributors) and XCC's
//! `misc/map_ts_encoder.cpp`, `misc/shp_decode.cpp` and `misc/cc_structures.h`
//! (Copyright (C) 2001 Olaf van der Spek), both GPL-3.0-or-later.

use base64::Engine;
use base64::engine::general_purpose::STANDARD_PAD_INDIFFERENT;

use crate::bytes::{PutLe, Reader, len_u16};
use crate::error::{Error, Result};
use crate::fixture::FixtureWriter;
use crate::limits::MAX_DECOMPRESSED;
use crate::{lcw, lzo};

/// Largest uncompressed block [`pack`] writes (XCC's encoder uses this
/// size; it keeps every compressed block within the 16-bit size field).
pub const MAX_BLOCK: usize = 0x2000;

/// Length of one [`IsoTile`] record.
pub const ISO_TILE_LEN: usize = 11;

/// Block compression of a map pack.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackCodec {
    /// LZO1X ([`crate::lzo`]), used by `IsoMapPack5`.
    Lzo,
    /// LCW with absolute positions ([`crate::lcw`]), used by `OverlayPack`
    /// and `OverlayDataPack`.
    Lcw,
}

/// Joins `values` and decodes them as standard base64.
///
/// Leading and trailing whitespace of each value is ignored. Padding (`=`)
/// at the end is optional: it is accepted when present and not required
/// when absent. Any other non-alphabet character is [`Error::Invalid`].
pub fn decode_base64<'a>(values: impl IntoIterator<Item = &'a str>) -> Result<Vec<u8>> {
    let joined: String = values.into_iter().map(str::trim).collect();
    STANDARD_PAD_INDIFFERENT
        .decode(joined)
        .map_err(|_| Error::Invalid("map pack: bad base64"))
}

/// Encodes `bytes` as padded standard base64 split into lines of `width`
/// characters (the last may be shorter). A `width` of 0 gives one line.
/// Empty input gives no lines.
pub fn encode_base64_lines(bytes: &[u8], width: usize) -> Vec<String> {
    let text = STANDARD_PAD_INDIFFERENT.encode(bytes);
    if width == 0 {
        return if text.is_empty() {
            Vec::new()
        } else {
            vec![text]
        };
    }
    // Base64 output is ASCII, so byte chunks are whole characters.
    text.as_bytes()
        .chunks(width)
        .map(|c| String::from_utf8_lossy(c).into_owned())
        .collect()
}

/// Decompresses a block-framed map pack into at most `max_out` bytes (and
/// at most [`MAX_DECOMPRESSED`]).
///
/// Blocks run to the end of `data`. Each must decompress to exactly its
/// stated uncompressed size, with its compressed stream ending exactly at
/// the block's end; otherwise the pack is [`Error::Invalid`]. A block whose
/// sizes are both zero is accepted as empty (OpenRA's importer notes that
/// some packs end with such a header). A header or block cut short by the
/// end of `data` is [`Error::Truncated`]; going over the limit is
/// [`Error::TooLarge`].
pub fn unpack(data: &[u8], codec: PackCodec, max_out: usize) -> Result<Vec<u8>> {
    let cap = max_out.min(MAX_DECOMPRESSED);
    let mut r = Reader::new(data);
    let mut out = Vec::new();
    while !r.is_empty() {
        let packed = usize::from(r.u16()?);
        let size = usize::from(r.u16()?);
        let block = r.bytes(packed)?;
        if out.len().saturating_add(size) > cap {
            return Err(Error::TooLarge("map pack exceeds the limit"));
        }
        if packed == 0 && size == 0 {
            continue;
        }
        let decoded = match codec {
            PackCodec::Lzo => lzo::decompress(block, size),
            PackCodec::Lcw => lcw::decompress(block, size),
        }
        .map_err(|e| match e {
            Error::Truncated { .. } => Error::Invalid("map pack: block stream cut short"),
            Error::TooLarge(_) => Error::Invalid("map pack: block larger than its stated size"),
            e => e,
        })?;
        if decoded.len() != size {
            return Err(Error::Invalid(
                "map pack: block smaller than its stated size",
            ));
        }
        out.extend_from_slice(&decoded);
    }
    Ok(out)
}

/// The value of a base-64 digit character (S-0012 claim 2): `A`–`Z` 0–25,
/// `a`–`z` 26–51, `0`–`9` 52–61, `+` 62, `/` 63.
pub fn base64_digit(c: u8) -> Option<u8> {
    // spec: S-0012 claim 2 (the alphabet, in increasing digit value; no `-` and `_` alphabet)
    match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

/// Packed bytes decoded from a map block's joined text (S-0012 claim 2).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextBytes {
    /// The bytes of the complete four-character groups.
    pub bytes: Vec<u8>,
    /// Index (from 0) of the first nonstandard complete group, if any.
    pub first_nonstandard: Option<usize>,
    /// Number of nonstandard complete groups.
    pub nonstandard: usize,
    /// Characters of a final group shorter than four, discarded.
    pub short_tail: usize,
}

/// Decodes joined base-64 text group by group, as the original does
/// (S-0012 claim 2 and Formulas F1): each complete group of four
/// *characters* collects digits until its end or an `=`, skipping other
/// characters outside the alphabet, and emits one byte for zero to two
/// digits, two for three and three for four. A final group shorter than
/// four characters is discarded (our engine's rule).
pub fn decode_text_groups(text: &[u8]) -> TextBytes {
    let mut out = TextBytes::default();
    let groups = text.chunks_exact(4);
    out.short_tail = groups.remainder().len();
    let count = text.len() / 4;
    for (gi, group) in groups.enumerate() {
        let mut digits = [0u8; 4];
        let mut n = 0usize;
        for &c in group {
            if c == b'=' {
                // spec: S-0012 claim 2 (`=` ends that group's interpretation)
                break;
            }
            // spec: S-0012 claim 2 (characters outside the alphabet other than `=` are skipped)
            if let Some(d) = base64_digit(c) {
                digits[n] = d;
                n += 1;
            }
        }
        // spec: S-0012 Formulas F1 (V = 262144·d₀ + 4096·d₁ + 64·d₂ + d₃, unfilled places 0)
        let v = (u32::from(digits[0]) << 18)
            | (u32::from(digits[1]) << 12)
            | (u32::from(digits[2]) << 6)
            | u32::from(digits[3]);
        let b = [(v >> 16) as u8, (v >> 8) as u8, v as u8];
        // spec: S-0012 claim 2 (one byte for zero, one or two digits; two for three; three for four)
        let emit = match n {
            0..=2 => 1,
            3 => 2,
            _ => 3,
        };
        out.bytes.extend_from_slice(&b[..emit]);
        // spec: S-0012 claim 2 (a standard group: four digits, or a final group of two digits and
        // `==` or three digits and `=`)
        let all_digits = group.iter().all(|&c| base64_digit(c).is_some());
        let last = gi + 1 == count;
        let padded = |k: usize| {
            group[..k].iter().all(|&c| base64_digit(c).is_some())
                && group[k..].iter().all(|&c| c == b'=')
        };
        let standard = all_digits || (last && (padded(2) || padded(3)));
        if !standard {
            out.nonstandard += 1;
            out.first_nonstandard.get_or_insert(gi);
        }
    }
    out
}

/// Why a map block's chunk was not used (S-0012 claims 9 and 10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChunkProblem {
    /// Index of the chunk, from 0.
    pub chunk: usize,
    /// What was wrong.
    pub what: &'static str,
}

/// Largest expanded chunk our engine supports (S-0012 claim 10).
pub const MAX_CHUNK: usize = 8192; // spec: S-0012 claim 10 (supported chunks have 1 … 8192 expanded bytes)

/// Expands a map block's packed bytes chunk by chunk (S-0012 claims 3 to
/// 5, 9 and 10). Returns the bytes of the complete, valid chunks before the
/// first problem, and that problem: the block stops at an incomplete or
/// rejected chunk, keeping the earlier chunks' bytes.
pub fn unpack_prefix(data: &[u8], codec: PackCodec) -> (Vec<u8>, Option<ChunkProblem>) {
    let mut out = Vec::new();
    let mut pos = 0usize;
    let mut chunk = 0usize;
    let fail = |chunk, what| Some(ChunkProblem { chunk, what });
    while pos < data.len() {
        // spec: S-0012 claim 3 (C at positions 0 and 1, U at 2 and 3, unsigned 16-bit little-endian)
        let Some(h) = data.get(pos..pos + 4) else {
            // spec: S-0012 claim 9 (a chunk header of fewer than four bytes: none of that chunk)
            return (out, fail(chunk, "short chunk header"));
        };
        let packed = usize::from(u16::from_le_bytes([h[0], h[1]]));
        let size = usize::from(u16::from_le_bytes([h[2], h[3]]));
        // spec: S-0012 claim 3 (the next chunk starts 4 + C bytes after this chunk's start)
        let Some(block) = data.get(pos + 4..pos + 4 + packed) else {
            // spec: S-0012 claim 9 (fewer than C compressed bytes: none of that chunk)
            return (out, fail(chunk, "short compressed content"));
        };
        // spec: S-0012 claim 10 (supported chunks: 1 … 8192 expanded bytes, at least one compressed byte)
        if packed == 0 || size == 0 || size > MAX_CHUNK {
            return (out, fail(chunk, "unsupported chunk size"));
        }
        if out.len().saturating_add(size) > MAX_DECOMPRESSED {
            return (out, fail(chunk, "over our decoding limit"));
        }
        // spec: S-0012 claim 3 (each chunk starts with empty copy history)
        let decoded = match codec {
            PackCodec::Lzo => lzo::decompress(block, size),
            PackCodec::Lcw => lcw::decompress(block, size),
        };
        // spec: S-0012 claim 10 (missing or early end marker, unavailable input or copy source,
        // or an expansion length other than U: the chunk is rejected)
        let decoded = match decoded {
            Ok(d) if d.len() == size => d,
            Ok(_) => return (out, fail(chunk, "expansion shorter than its stated length")),
            Err(Error::Truncated { .. }) => {
                return (
                    out,
                    fail(
                        chunk,
                        "compressed content without an end marker or cut short",
                    ),
                );
            }
            Err(Error::TooLarge(_)) => {
                return (out, fail(chunk, "expansion longer than its stated length"));
            }
            Err(Error::Invalid(m)) => return (out, fail(chunk, m)),
            Err(_) => return (out, fail(chunk, "invalid compressed content")),
        };
        out.extend_from_slice(&decoded);
        pos += 4 + packed;
        chunk += 1;
    }
    (out, None)
}

/// Compresses `data` into the block framing [`unpack`] reads, splitting it
/// into blocks of `block_size` bytes (clamped to `1..=`[`MAX_BLOCK`]).
/// Empty input gives no blocks.
pub fn pack(data: &[u8], codec: PackCodec, block_size: usize) -> Vec<u8> {
    let mut out = Vec::new();
    for chunk in data.chunks(block_size.clamp(1, MAX_BLOCK)) {
        let c = match codec {
            PackCodec::Lzo => lzo::compress(chunk),
            PackCodec::Lcw => lcw::compress(chunk),
        };
        out.put_u16(len_u16(c.len()));
        out.put_u16(len_u16(chunk.len()));
        out.extend_from_slice(&c);
    }
    out
}

/// One `IsoMapPack5` record: the terrain of one cell.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct IsoTile {
    /// Cell x coordinate.
    pub x: u16,
    /// Cell y coordinate.
    pub y: u16,
    /// Tile number, the four little-endian bytes at offset 4 read as one
    /// signed integer. XCC declares this as a signed 16-bit tile followed by
    /// two zero bytes, and OpenRA reads an unsigned 16-bit tile and skips
    /// the next two bytes; reading all four keeps every value exact and
    /// re-encodable. Callers wanting either 16-bit view take `tile as i16`
    /// or `tile as u16`.
    pub tile: i32,
    /// Sub-tile index within the tile.
    pub sub_tile: u8,
    /// Height level.
    pub level: u8,
    /// The record's last byte, kept raw (XCC names it as a zero byte).
    pub extra: u8,
}

/// Reads `IsoMapPack5` records from unpacked data.
///
/// A trailing remainder shorter than one record is ignored: OpenRA sizes
/// its buffer for four bytes beyond the last record, so such tails are
/// expected rather than malformed. The function never fails today; it
/// returns [`Result`] so stricter checks can be added without an API change.
pub fn iso_tiles(data: &[u8]) -> Result<Vec<IsoTile>> {
    let mut r = Reader::new(data);
    let mut out = Vec::with_capacity(data.len() / ISO_TILE_LEN);
    while r.remaining() >= ISO_TILE_LEN {
        out.push(IsoTile {
            x: r.u16()?,
            y: r.u16()?,
            tile: r.i32()?,
            sub_tile: r.u8()?,
            level: r.u8()?,
            extra: r.u8()?,
        });
    }
    Ok(out)
}

/// Appends `tiles` as `IsoMapPack5` records.
pub fn write_iso_tiles(tiles: &[IsoTile], out: &mut Vec<u8>) {
    for t in tiles {
        out.put_u16(t.x);
        out.put_u16(t.y);
        out.put_i32(t.tile);
        out.extend_from_slice(&[t.sub_tile, t.level, t.extra]);
    }
}

/// An invented `side × side` diamond of tiles with patterned numbers and a
/// smooth height field; a few cells use tile −1.
fn synthetic_tiles(side: u16) -> Vec<IsoTile> {
    let mut v = Vec::new();
    for y in 0..side {
        for x in 0..side {
            let (xi, yi) = (i32::from(x), i32::from(y));
            v.push(IsoTile {
                x: x + side,
                y: y + 1,
                tile: if (xi + 2 * yi) % 23 == 0 {
                    -1
                } else {
                    (xi * 3 + yi * 5) % 40
                },
                sub_tile: ((x ^ y) % 4) as u8,
                level: (((xi - 12).abs() + (yi - 9).abs()) / 4).min(14) as u8,
                extra: 0,
            });
        }
    }
    v
}

/// Invented overlay bytes: mostly 0xFF ("none") with scattered values.
fn synthetic_overlay(side: usize) -> Vec<u8> {
    (0..side * side)
        .map(|i| {
            let (x, y) = (i % side, i / side);
            if (x * x + 3 * y) % 17 == 0 {
                (x % 5) as u8
            } else {
                0xFF
            }
        })
        .collect()
}

/// How a [`MapPackFixture`] stores its pack.
#[derive(Clone, Copy)]
enum Form {
    /// Raw block framing.
    Binary,
    /// Base64 lines of 70 characters, one per text line.
    Base64,
}

struct MapPackFixture {
    name: &'static str,
    codec: PackCodec,
    form: Form,
    data: fn() -> Vec<u8>,
}

impl FixtureWriter for MapPackFixture {
    fn format(&self) -> &'static str {
        "mappack"
    }
    fn name(&self) -> String {
        self.name.to_owned()
    }
    fn extension(&self) -> &'static str {
        match self.form {
            Form::Binary => "bin",
            Form::Base64 => "txt",
        }
    }
    fn write(&self, out: &mut Vec<u8>) {
        let packed = pack(&(self.data)(), self.codec, MAX_BLOCK);
        match self.form {
            Form::Binary => out.extend(packed),
            Form::Base64 => {
                for line in encode_base64_lines(&packed, 70) {
                    out.extend_from_slice(line.as_bytes());
                    out.push(b'\n');
                }
            }
        }
    }
}

fn iso_grid() -> Vec<u8> {
    let mut v = Vec::new();
    write_iso_tiles(&synthetic_tiles(28), &mut v);
    v
}

/// Fixture writers for this module: block-framed packs of invented tiles
/// and overlay bytes, raw and as base64 lines.
pub fn fixture_writers() -> Vec<Box<dyn FixtureWriter>> {
    vec![
        Box::new(MapPackFixture {
            name: "iso-grid",
            codec: PackCodec::Lzo,
            form: Form::Binary,
            data: iso_grid,
        }),
        Box::new(MapPackFixture {
            name: "iso-grid",
            codec: PackCodec::Lzo,
            form: Form::Base64,
            data: iso_grid,
        }),
        Box::new(MapPackFixture {
            name: "overlay",
            codec: PackCodec::Lcw,
            form: Form::Binary,
            data: || synthetic_overlay(100),
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{Codec, check_round_trip};
    use crate::lz::sample_data;
    use crate::synth::SynthRng;

    struct IsoRecords;

    impl Codec for IsoRecords {
        type Value = Vec<IsoTile>;
        type Error = Error;
        fn decode(bytes: &[u8]) -> Result<Vec<IsoTile>> {
            iso_tiles(bytes)
        }
        fn encode(value: &Vec<IsoTile>) -> Vec<u8> {
            let mut out = Vec::new();
            write_iso_tiles(value, &mut out);
            out
        }
    }

    #[test]
    fn base64_lines_round_trip() {
        for len in [0, 1, 2, 3, 52, 53, 54, 1000] {
            let mut data = vec![0u8; len];
            SynthRng::new(len as u64).fill(&mut data);
            let lines = encode_base64_lines(&data, 70);
            assert!(lines.iter().all(|l| l.len() <= 70));
            assert_eq!(
                decode_base64(lines.iter().map(String::as_str)).unwrap(),
                data
            );
        }
        assert_eq!(encode_base64_lines(b"abcd", 0), ["YWJjZA=="]);
        assert!(encode_base64_lines(b"", 0).is_empty());
    }

    #[test]
    fn base64_padding_is_optional_and_junk_is_rejected() {
        assert_eq!(decode_base64(["YWJj", "ZA=="]).unwrap(), b"abcd");
        assert_eq!(decode_base64(["YWJj", "ZA"]).unwrap(), b"abcd");
        assert_eq!(decode_base64([" YWJj\r", "ZA\t"]).unwrap(), b"abcd");
        assert_eq!(decode_base64(["YW*j"]).unwrap_err().kind(), "invalid");
        assert_eq!(decode_base64(["YWJjZ"]).unwrap_err().kind(), "invalid");
        assert_eq!(decode_base64(std::iter::empty()).unwrap(), b"");
    }

    #[test]
    fn packs_round_trip() {
        for codec in [PackCodec::Lzo, PackCodec::Lcw] {
            for (len, block) in [
                (0, MAX_BLOCK),
                (1, MAX_BLOCK),
                (20_000, MAX_BLOCK),
                (5000, 999),
            ] {
                let data = sample_data(len as u64, len);
                let packed = pack(&data, codec, block);
                assert_eq!(
                    unpack(&packed, codec, len).unwrap(),
                    data,
                    "{codec:?} {len}"
                );
                let lines = encode_base64_lines(&packed, 70);
                let bytes = decode_base64(lines.iter().map(String::as_str)).unwrap();
                assert_eq!(unpack(&bytes, codec, len).unwrap(), data);
            }
        }
        // Oversized and zero block sizes are clamped.
        let data = sample_data(5, 20_000);
        assert_eq!(
            pack(&data, PackCodec::Lzo, 1 << 20),
            pack(&data, PackCodec::Lzo, MAX_BLOCK)
        );
        assert_eq!(
            unpack(&pack(b"xyz", PackCodec::Lcw, 0), PackCodec::Lcw, 3).unwrap(),
            b"xyz"
        );
    }

    #[test]
    fn block_framing_is_checked() {
        let data = sample_data(3, 300);
        let mut packed = pack(&data, PackCodec::Lzo, MAX_BLOCK);
        // A trailing empty block is accepted.
        packed.extend_from_slice(&[0, 0, 0, 0]);
        assert_eq!(unpack(&packed, PackCodec::Lzo, 300).unwrap(), data);
        // A partial header is truncation.
        packed.push(0);
        assert_eq!(
            unpack(&packed, PackCodec::Lzo, 300).unwrap_err().kind(),
            "truncated"
        );
        packed.truncate(packed.len() - 5);

        // Stated size too large or too small.
        let mut wrong = packed.clone();
        wrong[2] = wrong[2].wrapping_add(1);
        assert_eq!(
            unpack(&wrong, PackCodec::Lzo, 1000),
            Err(Error::Invalid(
                "map pack: block smaller than its stated size"
            ))
        );
        wrong[2] = wrong[2].wrapping_sub(2);
        assert_eq!(
            unpack(&wrong, PackCodec::Lzo, 1000),
            Err(Error::Invalid(
                "map pack: block larger than its stated size"
            ))
        );
        // Compressed size one short of the stream.
        let set_packed = |v: &mut Vec<u8>, delta: i32| {
            let n = i32::from(u16::from_le_bytes([v[0], v[1]])) + delta;
            v[..2].copy_from_slice(&u16::try_from(n).unwrap().to_le_bytes());
        };
        let mut short = packed.clone();
        set_packed(&mut short, -1);
        short.pop();
        assert_eq!(
            unpack(&short, PackCodec::Lzo, 1000),
            Err(Error::Invalid("map pack: block stream cut short"))
        );
        // Compressed size beyond the data.
        let mut long = packed.clone();
        set_packed(&mut long, 1);
        assert_eq!(
            unpack(&long, PackCodec::Lzo, 1000).unwrap_err().kind(),
            "truncated"
        );
        // Over the caller's limit.
        assert_eq!(
            unpack(&packed, PackCodec::Lzo, 299).unwrap_err().kind(),
            "too-large"
        );
        // Every prefix that cuts a block fails.
        for n in 1..packed.len() {
            assert!(unpack(&packed[..n], PackCodec::Lzo, 1000).is_err(), "{n}");
        }
    }

    #[test]
    fn s0012_claim_2_text_groups() {
        // spec: S-0012 claim 2 (standard groups, and the padded final group)
        let t = decode_text_groups(b"YWJjZA==");
        assert_eq!(
            (t.bytes.as_slice(), t.nonstandard, t.short_tail),
            (&b"abcd"[..], 0, 0)
        );
        assert_eq!(decode_text_groups(b"YWJjZGU=").bytes, b"abcde");
        // a group crossing a line boundary is joined text (claim 1): no separator
        assert_eq!(decode_text_groups(b"YWJjZGVm").bytes, b"abcdef");
        // characters outside the alphabet are skipped inside their group
        let t = decode_text_groups(b"Y*Jj");
        // digits Y, J, j: three digits, two bytes; nonstandard
        assert_eq!(
            t.bytes,
            [
                ((24u32 << 18 | 9 << 12 | 35 << 6) >> 16) as u8,
                ((24u32 << 18 | 9 << 12 | 35 << 6) >> 8) as u8
            ]
        );
        assert_eq!((t.nonstandard, t.first_nonstandard), (1, Some(0)));
        // `=` ends only its group; interpretation resumes with the next group
        let t = decode_text_groups(b"YQ==YWJj");
        assert_eq!(t.bytes, b"aabc");
        assert_eq!(
            t.first_nonstandard,
            Some(0),
            "a padded group that is not final"
        );
        // zero digits emit one byte 0; two digits emit ⌊(64·d₀ + d₁) / 16⌋
        assert_eq!(decode_text_groups(b"====").bytes, [0]);
        assert_eq!(
            decode_text_groups(b"YW=x").bytes,
            [((64 * 24 + 22) / 16) as u8]
        );
        // no `-` and `_` alphabet
        assert_eq!(decode_text_groups(b"-_AB").bytes, [0]);
        // a short final group is discarded; earlier groups stay
        let t = decode_text_groups(b"YWJjZA");
        assert_eq!((t.bytes.as_slice(), t.short_tail), (&b"abc"[..], 2));
    }

    #[test]
    fn s0012_claims_3_9_10_chunk_prefix() {
        let data = sample_data(3, 20_000);
        let packed = pack(&data, PackCodec::Lzo, MAX_BLOCK);
        // spec: S-0012 claim 3 (chunks concatenate; a shorter last chunk is normal)
        assert_eq!(unpack_prefix(&packed, PackCodec::Lzo), (data.clone(), None));
        let first = 4 + usize::from(u16::from_le_bytes([packed[0], packed[1]]));
        // spec: S-0012 claim 9 (a short header or short content keeps the earlier chunks)
        for cut in [first + 3, first + 10] {
            let (out, p) = unpack_prefix(&packed[..cut], PackCodec::Lzo);
            assert_eq!(out, &data[..MAX_BLOCK]);
            assert_eq!(p.map(|p| p.chunk), Some(1));
        }
        // spec: S-0012 claim 10 (an expansion length other than U rejects the chunk)
        let mut wrong = packed.clone();
        wrong[first + 2] ^= 1;
        let (out, p) = unpack_prefix(&wrong, PackCodec::Lzo);
        assert_eq!((out.len(), p.map(|p| p.chunk)), (MAX_BLOCK, Some(1)));
        // spec: S-0012 claim 10 (an end marker before the end of the compressed content)
        let mut early = pack(b"xyz", PackCodec::Lcw, MAX_BLOCK);
        early.push(0x80);
        early[0] += 1;
        assert_eq!(
            unpack_prefix(&early, PackCodec::Lcw).1.map(|p| p.chunk),
            Some(0)
        );
        // spec: S-0012 claim 10 (empty chunks and chunks above 8192 bytes are unsupported)
        let (out, p) = unpack_prefix(&[0, 0, 0, 0], PackCodec::Lcw);
        assert_eq!(
            (out.len(), p.map(|p| p.what)),
            (0, Some("unsupported chunk size"))
        );
        let big = [1, 0, 0x01, 0x20, 0x80];
        assert_eq!(
            unpack_prefix(&big, PackCodec::Lcw).1.map(|p| p.what),
            Some("unsupported chunk size")
        );
        // spec: S-0012 claim 10 (a copy before the chunk's first byte rejects it: no shared history)
        let two = [
            &pack(b"abc", PackCodec::Lcw, 3)[..],
            &[3, 0, 3, 0, 0x00, 0x03, 0x80],
        ]
        .concat();
        let (out, p) = unpack_prefix(&two, PackCodec::Lcw);
        assert_eq!((out.as_slice(), p.map(|p| p.chunk)), (&b"abc"[..], Some(1)));
    }

    #[test]
    fn s0012_table_l_zero_length_runs() {
        // spec: S-0012 Formulas, table L (a zero-length run or absolute copy needs no source byte)
        let s = [0xFE, 0, 0, 7, 0xFF, 0, 0, 5, 0, 0x81, b'q', 0x80];
        assert_eq!(lcw::decompress(&s, 10).unwrap(), b"q");
    }

    #[test]
    fn iso_tiles_round_trip() {
        let tiles = synthetic_tiles(9);
        check_round_trip::<IsoRecords>(&tiles).unwrap();
        let mut bytes = Vec::new();
        write_iso_tiles(&tiles, &mut bytes);
        assert_eq!(bytes.len(), tiles.len() * ISO_TILE_LEN);
        // A short tail is ignored.
        bytes.extend_from_slice(&[0xAA; 4]);
        assert_eq!(iso_tiles(&bytes).unwrap(), tiles);
        assert_eq!(iso_tiles(&bytes[..10]).unwrap(), vec![]);
    }

    #[test]
    fn iso_tile_layout() {
        let bytes = [
            0x34, 0x12, // x = 0x1234
            0x02, 0x00, // y = 2
            0xFF, 0xFF, 0x00, 0x00, // tile: -1 as i16, then two zero bytes
            0x03, // sub-tile
            0x04, // level
            0x05, // extra
        ];
        let t = iso_tiles(&bytes).unwrap();
        assert_eq!(
            t,
            [IsoTile {
                x: 0x1234,
                y: 2,
                tile: 0xFFFF,
                sub_tile: 3,
                level: 4,
                extra: 5
            }]
        );
        assert_eq!(t[0].tile as i16, -1);
        let mut back = Vec::new();
        write_iso_tiles(&t, &mut back);
        assert_eq!(back, bytes);
    }

    #[test]
    fn mutated_packs_never_panic() {
        let mut rng = SynthRng::new(79);
        let streams = [
            pack(&iso_grid(), PackCodec::Lzo, MAX_BLOCK),
            pack(&synthetic_overlay(80), PackCodec::Lcw, 2000),
        ];
        let codecs = [PackCodec::Lzo, PackCodec::Lcw];
        for i in 0..2000 {
            let mut s = streams[i % 2].clone();
            for _ in 0..1 + rng.next_u64() % 4 {
                let at = (rng.next_u64() % s.len() as u64) as usize;
                s[at] = rng.next_u64() as u8;
            }
            let max = 1 << 15;
            if let Ok(out) = unpack(&s, codecs[i % 2], max) {
                assert!(out.len() <= max);
                let _ = iso_tiles(&out);
            }
        }
    }

    #[test]
    fn fixtures_decode() {
        for w in fixture_writers() {
            let mut out = Vec::new();
            w.write(&mut out);
            assert!(out.len() < 16 * 1024, "{}", w.name());
        }
        let mut text = Vec::new();
        fixture_writers()[1].write(&mut text);
        let text = String::from_utf8(text).unwrap();
        let packed = decode_base64(text.lines()).unwrap();
        let tiles = iso_tiles(&unpack(&packed, PackCodec::Lzo, 1 << 16).unwrap()).unwrap();
        assert_eq!(tiles, synthetic_tiles(28));
    }
}
