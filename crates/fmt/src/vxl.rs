//! VXL voxel models: one or more limbs (sections), each a sparse grid of
//! voxels with a palette index and a normal index.
//!
//! Layout (little-endian):
//!
//! - header (802 bytes): `"Voxel Animation\0"`, `u32 palette count`,
//!   `u32 limb count`, `u32 tailer count`, `u32 body size`, two raw bytes,
//!   a 768-byte RGB palette;
//! - limb headers (28 bytes each): 16-byte name, `u32 number`, two raw
//!   `u32` fields;
//! - body (`body size` bytes);
//! - limb tailers (92 bytes each): span-start table offset, span-end table
//!   offset, span-data offset (`u32`, from the body start), `f32 scale`,
//!   a 3 × 4 `f32` transform, `f32` bounds (min x y z, max x y z),
//!   `u8 size x, y, z`, `u8 normals mode`.
//!
//! Per limb the body holds two tables of `size x × size y` `i32` offsets
//! (start and inclusive end of each column's data, −1 for an empty column;
//! column index `y × size x + x`) and the column data: repeated spans of
//! `u8 skip, u8 count, count × (u8 colour, u8 normal), u8 count`, until the
//! column reaches `size z`.
//!
//! Layout from XCC's `cc_structures.h` and `vxl_file.cpp` and OpenRA's
//! `VxlReader.cs` (GPL-3.0-or-later; XCC Copyright (C) 2000 Olaf van der
//! Spek, OpenRA Copyright (c) The OpenRA Developers and Contributors).
//! Floats are kept as raw bits ([`F32Bits`]).

use crate::bytes::{F32Bits, PutLe, Reader, len_u32, slice};
use crate::error::{Error, Result};
use crate::fixture::FixtureWriter;
use crate::image::{PALETTE_LEN, Palette, Rgb8};
use crate::limits::MAX_VOXEL_CELLS;

/// File magic (16 bytes with the terminating NUL).
pub const MAGIC: &[u8; 16] = b"Voxel Animation\0";
/// Header length including the palette.
pub const HEADER_LEN: usize = 802;
/// Limb header length.
pub const LIMB_HEADER_LEN: usize = 28;
/// Limb tailer length.
pub const LIMB_TAILER_LEN: usize = 92;

/// One voxel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Voxel {
    /// Column position.
    pub x: u8,
    /// Column position.
    pub y: u8,
    /// Height within the column.
    pub z: u8,
    /// Palette index.
    pub color: u8,
    /// Normal index (meaning depends on the limb's normals mode).
    pub normal: u8,
}

/// One limb.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Limb {
    /// Name, trimmed at the first NUL.
    pub name: Vec<u8>,
    /// Number field of the header.
    pub number: u32,
    /// Raw header fields after the number.
    pub header_raw: [u32; 2],
    /// Scale.
    pub scale: F32Bits,
    /// 3 × 4 transform, row-major.
    pub transform: [[F32Bits; 4]; 3],
    /// Minimum bounds (x, y, z).
    pub min: [F32Bits; 3],
    /// Maximum bounds (x, y, z).
    pub max: [F32Bits; 3],
    /// Grid size (x, y, z).
    pub size: [u8; 3],
    /// Normals mode, raw.
    pub normals_mode: u8,
    /// Voxels, by column (y, then x) and then height.
    pub voxels: Vec<Voxel>,
    /// Each column's data exactly as stored, by column (y, then x): `None`
    /// for a column the file marks empty, `Some` (possibly holding no voxel)
    /// otherwise. Renderers that must tell those two apart, or read the
    /// runs as stored, use this rather than [`Limb::voxels`].
    pub columns: Vec<Option<Vec<u8>>>,
}

/// A parsed VXL file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vxl {
    /// Palette count field.
    pub palette_count: u32,
    /// Raw bytes after the body size.
    pub header_raw: [u8; 2],
    /// Embedded palette (8-bit components as stored).
    pub palette: Palette,
    /// Limbs.
    pub limbs: Vec<Limb>,
}

impl Vxl {
    /// Parses a VXL file, decoding every limb's voxels.
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        if &r.array::<16>()? != MAGIC {
            return Err(Error::Invalid("VXL magic"));
        }
        let palette_count = r.u32()?;
        let limb_count = r.u32()? as usize;
        let tailer_count = r.u32()? as usize;
        let body_size = r.u32()? as usize;
        let header_raw = r.array()?;
        let mut palette = Palette::default();
        for c in palette.colors.iter_mut().take(PALETTE_LEN) {
            let [red, g, b] = r.array()?;
            *c = Rgb8::new(red, g, b);
        }
        if tailer_count != limb_count {
            return Err(Error::Invalid("VXL limb and tailer counts differ"));
        }
        let body_start = HEADER_LEN
            .checked_add(
                limb_count
                    .checked_mul(LIMB_HEADER_LEN)
                    .ok_or(Error::Invalid("VXL"))?,
            )
            .ok_or(Error::Invalid("VXL limb count"))?;
        let tailers_start = body_start
            .checked_add(body_size)
            .ok_or(Error::Invalid("VXL body size"))?;
        let tailers = slice(
            data,
            tailers_start,
            limb_count.saturating_mul(LIMB_TAILER_LEN),
        )?;
        let body = slice(data, body_start, body_size)?;
        let mut limbs = Vec::with_capacity(limb_count);
        for i in 0..limb_count {
            let mut h = Reader::at(data, HEADER_LEN + i * LIMB_HEADER_LEN);
            let name_raw = h.array::<16>()?;
            let name_len = name_raw.iter().position(|&b| b == 0).unwrap_or(16);
            let number = h.u32()?;
            let header_raw = [h.u32()?, h.u32()?];
            let mut t = Reader::at(tailers, i * LIMB_TAILER_LEN);
            let start_ofs = t.u32()? as usize;
            let end_ofs = t.u32()? as usize;
            let data_ofs = t.u32()? as usize;
            let scale = t.f32_bits()?;
            let mut transform = [[F32Bits::default(); 4]; 3];
            for row in &mut transform {
                for v in row.iter_mut() {
                    *v = t.f32_bits()?;
                }
            }
            let min = [t.f32_bits()?, t.f32_bits()?, t.f32_bits()?];
            let max = [t.f32_bits()?, t.f32_bits()?, t.f32_bits()?];
            let size: [u8; 3] = t.array()?;
            let normals_mode = t.u8()?;
            let voxels = decode_columns(body, start_ofs, end_ofs, data_ofs, size)?;
            let columns = raw_columns(body, start_ofs, end_ofs, data_ofs, size)?;
            limbs.push(Limb {
                name: name_raw[..name_len].to_vec(),
                number,
                header_raw,
                scale,
                transform,
                min,
                max,
                size,
                normals_mode,
                voxels,
                columns,
            });
        }
        Ok(Self {
            palette_count,
            header_raw,
            palette,
            limbs,
        })
    }
}

/// Each column's stored data (`None`: marked empty), in column order.
fn raw_columns(
    body: &[u8],
    start_ofs: usize,
    end_ofs: usize,
    data_ofs: usize,
    size: [u8; 3],
) -> Result<Vec<Option<Vec<u8>>>> {
    let [sx, sy, _] = size.map(usize::from);
    let columns = sx * sy;
    let starts = slice(body, start_ofs, columns * 4)?;
    let ends = slice(body, end_ofs, columns * 4)?;
    let span_data = body.get(data_ofs..).ok_or(Error::Truncated {
        needed: data_ofs as u64,
    })?;
    let i32_at = |t: &[u8], i: usize| {
        let b = &t[i * 4..i * 4 + 4];
        i32::from_le_bytes([b[0], b[1], b[2], b[3]])
    };
    let mut out = Vec::with_capacity(columns);
    for col in 0..columns {
        let (start, end) = (i32_at(starts, col), i32_at(ends, col));
        if start == -1 {
            out.push(None);
            continue;
        }
        let start = usize::try_from(start).map_err(|_| Error::Invalid("VXL column offset"))?;
        let end = usize::try_from(end).map_err(|_| Error::Invalid("VXL column end"))?;
        if end < start {
            return Err(Error::Invalid("VXL column ends before it starts"));
        }
        out.push(Some(slice(span_data, start, end - start + 1)?.to_vec()));
    }
    Ok(out)
}

/// The column data [`VxlWriter`] stores for `voxels` in a grid of `size`
/// (spans as XCC writes them: one voxel per span, and a closing empty span
/// when the column stops short of the top), in column order.
pub fn column_bytes(voxels: &[Voxel], size: [u8; 3]) -> Vec<Option<Vec<u8>>> {
    let [sx, sy, sz] = size.map(usize::from);
    let mut sorted = voxels.to_vec();
    sorted.sort_by_key(|v| (v.y, v.x, v.z));
    (0..sx * sy)
        .map(|col| {
            let (x, y) = ((col % sx) as u8, (col / sx) as u8);
            let in_col: Vec<&Voxel> = sorted.iter().filter(|v| v.x == x && v.y == y).collect();
            if in_col.is_empty() {
                return None;
            }
            let mut out = Vec::new();
            let mut z = 0usize;
            for v in in_col {
                let vz = usize::from(v.z);
                out.extend_from_slice(&[(vz - z) as u8, 1, v.color, v.normal, 1]);
                z = vz + 1;
            }
            if z != sz {
                out.extend_from_slice(&[(sz - z) as u8, 0, 0]);
            }
            Some(out)
        })
        .collect()
}

fn decode_columns(
    body: &[u8],
    start_ofs: usize,
    end_ofs: usize,
    data_ofs: usize,
    size: [u8; 3],
) -> Result<Vec<Voxel>> {
    let [sx, sy, sz] = size.map(usize::from);
    let columns = sx * sy;
    if columns * sz > MAX_VOXEL_CELLS {
        return Err(Error::TooLarge("VXL limb grid"));
    }
    let starts = slice(body, start_ofs, columns * 4)?;
    let ends = slice(body, end_ofs, columns * 4)?;
    let span_data = body.get(data_ofs..).ok_or(Error::Truncated {
        needed: data_ofs as u64,
    })?;
    let i32_at = |t: &[u8], i: usize| {
        let b = &t[i * 4..i * 4 + 4];
        i32::from_le_bytes([b[0], b[1], b[2], b[3]])
    };
    let mut voxels = Vec::new();
    for col in 0..columns {
        let (start, end) = (i32_at(starts, col), i32_at(ends, col));
        if start == -1 {
            continue;
        }
        let start = usize::try_from(start).map_err(|_| Error::Invalid("VXL column offset"))?;
        let end = usize::try_from(end).map_err(|_| Error::Invalid("VXL column end"))?;
        if end < start {
            return Err(Error::Invalid("VXL column ends before it starts"));
        }
        let column = slice(span_data, start, end - start + 1)?;
        let (x, y) = ((col % sx) as u8, (col / sx) as u8);
        let mut r = Reader::new(column);
        let mut z = 0usize;
        while z < sz {
            let skip = usize::from(r.u8()?);
            let count = r.u8()?;
            z += skip;
            for _ in 0..count {
                if z >= sz {
                    return Err(Error::Invalid("VXL voxel above the limb's height"));
                }
                let [color, normal] = r.array()?;
                voxels.push(Voxel {
                    x,
                    y,
                    z: z as u8,
                    color,
                    normal,
                });
                z += 1;
            }
            if r.u8()? != count {
                return Err(Error::Invalid("VXL span counts differ"));
            }
        }
    }
    Ok(voxels)
}

/// Writes synthetic VXL files.
#[derive(Clone, Debug)]
pub struct VxlWriter<'a> {
    /// The model; each limb's voxels must lie inside its size, at most one
    /// per position.
    pub model: &'a Vxl,
}

impl VxlWriter<'_> {
    /// Serialises the model (spans as XCC writes them: one voxel per span,
    /// and a closing empty span when the column stops short of the top).
    pub fn write(&self) -> Vec<u8> {
        let m = self.model;
        let mut body = Vec::new();
        let mut tailers = Vec::new();
        for limb in &m.limbs {
            let [sx, sy, _] = limb.size.map(usize::from);
            let start_ofs = body.len();
            body.resize(start_ofs + sx * sy * 8, 0);
            let data_ofs = body.len();
            for (col, data) in column_bytes(&limb.voxels, limb.size)
                .into_iter()
                .enumerate()
            {
                let (s, e) = match data {
                    None => (-1i32, -1i32),
                    Some(data) => {
                        let begin = body.len();
                        body.extend_from_slice(&data);
                        (
                            (begin - data_ofs) as i32,
                            (body.len() - data_ofs - 1) as i32,
                        )
                    }
                };
                body[start_ofs + col * 4..start_ofs + col * 4 + 4]
                    .copy_from_slice(&s.to_le_bytes());
                let e_at = start_ofs + sx * sy * 4 + col * 4;
                body[e_at..e_at + 4].copy_from_slice(&e.to_le_bytes());
            }
            tailers.put_u32(len_u32(start_ofs));
            tailers.put_u32(len_u32(start_ofs + sx * sy * 4));
            tailers.put_u32(len_u32(data_ofs));
            tailers.put_u32(limb.scale.0);
            for v in limb
                .transform
                .iter()
                .flatten()
                .chain(&limb.min)
                .chain(&limb.max)
            {
                tailers.put_u32(v.0);
            }
            tailers.extend_from_slice(&limb.size);
            tailers.push(limb.normals_mode);
        }
        let mut out = MAGIC.to_vec();
        out.put_u32(m.palette_count);
        out.put_u32(len_u32(m.limbs.len()));
        out.put_u32(len_u32(m.limbs.len()));
        out.put_u32(len_u32(body.len()));
        out.extend_from_slice(&m.header_raw);
        for c in &m.palette.colors {
            out.extend_from_slice(&[c.r, c.g, c.b]);
        }
        for limb in &m.limbs {
            let mut name = [0u8; 16];
            let n = limb.name.len().min(15);
            name[..n].copy_from_slice(&limb.name[..n]);
            out.extend_from_slice(&name);
            out.put_u32(limb.number);
            out.put_u32(limb.header_raw[0]);
            out.put_u32(limb.header_raw[1]);
        }
        out.extend_from_slice(&body);
        out.extend_from_slice(&tailers);
        out
    }
}

/// A synthetic model: a hollow box limb and a small sphere limb, invented.
pub fn synthetic_model() -> Vxl {
    let mut limbs = Vec::new();
    for (i, (name, size)) in [(&b"body"[..], [10u8, 8, 6]), (&b"orb"[..], [7, 7, 7])]
        .into_iter()
        .enumerate()
    {
        let mut voxels = Vec::new();
        let [sx, sy, sz] = size;
        for y in 0..sy {
            for x in 0..sx {
                for z in 0..sz {
                    let keep = if i == 0 {
                        // Box shell, open at the top.
                        x == 0 || y == 0 || x == sx - 1 || y == sy - 1 || z == 0
                    } else {
                        let c = |v: u8, s: u8| 2 * i32::from(v) - i32::from(s) + 1;
                        c(x, sx).pow(2) + c(y, sy).pow(2) + c(z, sz).pow(2) <= 49
                    };
                    if keep {
                        voxels.push(Voxel {
                            x,
                            y,
                            z,
                            color: 16 * (i as u8 + 2) + z,
                            normal: (x + y + z) % 36,
                        });
                    }
                }
            }
        }
        let f = F32Bits::new;
        limbs.push(Limb {
            name: name.to_vec(),
            number: i as u32,
            header_raw: [1, 0],
            scale: f(1.0 / 12.0),
            transform: [
                [f(1.0), f(0.0), f(0.0), f(0.0)],
                [f(0.0), f(1.0), f(0.0), f(0.0)],
                [f(0.0), f(0.0), f(1.0), f(0.0)],
            ],
            min: [f(-5.0), f(-4.0), f(0.0)],
            max: [f(5.0), f(4.0), f(6.0)],
            size,
            normals_mode: 4,
            columns: column_bytes(&voxels, size),
            voxels,
        });
    }
    Vxl {
        palette_count: 1,
        header_raw: [0x10, 0x1F],
        palette: crate::synth::ramp_palette(4),
        limbs,
    }
}

struct SyntheticVxl;

impl FixtureWriter for SyntheticVxl {
    fn format(&self) -> &'static str {
        "vxl"
    }
    fn name(&self) -> String {
        "box-and-orb".into()
    }
    fn extension(&self) -> &'static str {
        "vxl"
    }
    fn write(&self, out: &mut Vec<u8>) {
        out.extend(
            VxlWriter {
                model: &synthetic_model(),
            }
            .write(),
        );
    }
}

/// Fixture writers for this module.
pub fn fixture_writers() -> Vec<Box<dyn FixtureWriter>> {
    vec![Box::new(SyntheticVxl)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let model = synthetic_model();
        let bytes = VxlWriter { model: &model }.write();
        let back = Vxl::parse(&bytes).unwrap();
        let mut expected = model.clone();
        for l in &mut expected.limbs {
            l.voxels.sort_by_key(|v| (v.y, v.x, v.z));
        }
        assert_eq!(back, expected);
        assert_eq!(VxlWriter { model: &back }.write(), bytes);
    }

    #[test]
    fn spans_decode_by_hand() {
        // One 1 × 1 × 5 column: skip 1, two voxels (colour 9 normal 3,
        // colour 8 normal 2), then skip 2 with no voxels: z = 1, 2 filled.
        let mut body = Vec::new();
        body.extend_from_slice(&0i32.to_le_bytes()); // start
        body.extend_from_slice(&9i32.to_le_bytes()); // inclusive end
        body.extend_from_slice(&[1, 2, 9, 3, 8, 2, 2, 2, 0, 0]);
        let v = decode_columns(&body, 0, 4, 8, [1, 1, 5]).unwrap();
        assert_eq!(
            v.iter().map(|v| (v.z, v.color)).collect::<Vec<_>>(),
            [(1, 9), (2, 8)]
        );
        // Mismatched trailing count (data byte 6 of the column).
        body[8 + 6] = 1;
        assert!(decode_columns(&body, 0, 4, 8, [1, 1, 5]).is_err());
    }

    #[test]
    fn truncation_never_panics() {
        let bytes = VxlWriter {
            model: &synthetic_model(),
        }
        .write();
        for cut in 0..bytes.len() {
            assert!(Vxl::parse(&bytes[..cut]).is_err());
        }
    }
}
