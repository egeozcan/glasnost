//! TMP terrain templates, TS/RA2 isometric variant ("TMP-TS").
//!
//! A template is a grid of `cells_x × cells_y` cells; each cell holds an
//! isometric tile or nothing. File header: `i32 cells_x, i32 cells_y,
//! i32 tile width, i32 tile height`, then one `u32` offset per cell (0:
//! empty). Each tile starts with a 52-byte header:
//!
//! | field | type |
//! |---|---|
//! | x, y (tile position in template pixels) | i32 × 2 |
//! | extra-image, depth, extra-depth offsets (from the tile start) | i32 × 3 |
//! | extra-image x, y, width, height | i32 × 4 |
//! | flags: 1 extra image, 2 depth data, 4 damaged variant | u32 |
//! | height level, terrain type, ramp type | u8 × 3 |
//! | radar colours, left and right RGB | u8 × 6 |
//! | padding | u8 × 3 |
//!
//! The header is followed by the diamond's pixels (`width × height / 2`
//! bytes, row by row; rows grow by 4 pixels per row to the middle, then
//! shrink), the depth values for the same pixels, and optionally an extra
//! rectangular image with its own depth values (cliff faces and other
//! parts that rise above the diamond).
//!
//! Layout from XCC's `cc_structures.h` and `tmp_ts_file.h` and OpenRA's
//! `TmpTSLoader.cs` (GPL-3.0-or-later; XCC Copyright (C) 2000 Olaf van der
//! Spek, OpenRA Copyright (c) The OpenRA Developers and Contributors). We
//! locate the depth and extra data by the header offsets, as XCC does.

use crate::bytes::{PutLe, Reader, len_u32, slice};
use crate::error::{Error, Result};
use crate::fixture::FixtureWriter;
use crate::image::IndexedImage;
use crate::limits::MAX_IMAGE_PIXELS;

/// Length of the file header.
pub const FILE_HEADER_LEN: usize = 16;
/// Length of a tile header.
pub const TILE_HEADER_LEN: usize = 52;

/// Flag: the tile has an extra image.
pub const FLAG_EXTRA: u32 = 1;
/// Flag: the tile has depth data.
pub const FLAG_DEPTH: u32 = 2;
/// Flag: the tile has a damaged variant.
pub const FLAG_DAMAGED: u32 = 4;

/// One tile header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TmpTile {
    /// Offset of the tile in the file.
    pub offset: u32,
    /// Tile position in template pixels.
    pub x: i32,
    /// Tile position in template pixels.
    pub y: i32,
    /// Extra image offset from the tile start.
    pub extra_offset: i32,
    /// Depth data offset from the tile start.
    pub depth_offset: i32,
    /// Extra depth offset from the tile start.
    pub extra_depth_offset: i32,
    /// Extra image position in template pixels.
    pub extra_x: i32,
    /// Extra image position in template pixels.
    pub extra_y: i32,
    /// Extra image width.
    pub extra_width: i32,
    /// Extra image height.
    pub extra_height: i32,
    /// Flags (see `FLAG_*`).
    pub flags: u32,
    /// Height level.
    pub height: u8,
    /// Terrain type, raw.
    pub terrain_type: u8,
    /// Ramp type, raw.
    pub ramp_type: u8,
    /// Radar colour of the left half.
    pub radar_left: [u8; 3],
    /// Radar colour of the right half.
    pub radar_right: [u8; 3],
}

impl TmpTile {
    /// Whether the tile carries an extra image.
    pub fn has_extra(&self) -> bool {
        self.flags & FLAG_EXTRA != 0
    }
}

/// A parsed template; tiles are decoded on demand.
#[derive(Clone, Debug)]
pub struct Tmp<'a> {
    /// Cells across.
    pub cells_x: u32,
    /// Cells down.
    pub cells_y: u32,
    /// Diamond width in pixels.
    pub tile_width: u32,
    /// Diamond height in pixels.
    pub tile_height: u32,
    /// One entry per cell, row-major; `None` for empty cells.
    pub tiles: Vec<Option<TmpTile>>,
    data: &'a [u8],
}

/// An image placed in template pixel coordinates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placed {
    /// Left edge.
    pub x: i32,
    /// Top edge.
    pub y: i32,
    /// Palette indices.
    pub image: IndexedImage,
    /// Depth values for the same pixels, if present.
    pub depth: Option<IndexedImage>,
}

/// A decoded tile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedTile {
    /// The diamond, `tile_width × tile_height`, index 0 outside it.
    pub diamond: Placed,
    /// The extra image, if any.
    pub extra: Option<Placed>,
}

/// Width of each diamond row, top to bottom (rows grow by 4 to the middle,
/// then shrink, as OpenRA's loader lays them out). Fails for sizes whose
/// rows do not add up to `width × height / 2` within the width.
pub fn diamond_rows(width: u32, height: u32) -> Result<Vec<u32>> {
    // Engineering bound of our own, far above any real tile, checked before
    // allocating; the row rule makes the width about twice the height.
    const MAX_SIDE: u32 = 4096;
    if height > MAX_SIDE || width > 2 * MAX_SIDE + 8 {
        return Err(Error::Unsupported("TMP tile size"));
    }
    let mut rows = Vec::with_capacity(height as usize);
    let mut w: i64 = 4;
    let half = i64::from(height / 2);
    for j in 0..i64::from(height) {
        rows.push(u32::try_from(w).map_err(|_| Error::Unsupported("TMP tile size"))?);
        w += if j < half - 1 { 4 } else { -4 };
    }
    let total: u64 = rows.iter().map(|&r| u64::from(r)).sum();
    if total != u64::from(width) * u64::from(height) / 2 || rows.iter().any(|&r| r > width) {
        return Err(Error::Unsupported("TMP tile size"));
    }
    Ok(rows)
}

impl<'a> Tmp<'a> {
    /// Parses the headers.
    pub fn parse(data: &'a [u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        let dim = |v: i32| u32::try_from(v).map_err(|_| Error::Invalid("negative TMP size"));
        let cells_x = dim(r.i32()?)?;
        let cells_y = dim(r.i32()?)?;
        let tile_width = dim(r.i32()?)?;
        let tile_height = dim(r.i32()?)?;
        if tile_width == 0 || tile_height == 0 {
            return Err(Error::Invalid("TMP tile size is zero"));
        }
        // In u64: two u32 sizes overflow a 32-bit usize (wasm32). After this
        // check `tile_width * tile_height` fits in u32 and usize.
        if u64::from(tile_width) * u64::from(tile_height) > MAX_IMAGE_PIXELS as u64 {
            return Err(Error::TooLarge("TMP tile"));
        }
        diamond_rows(tile_width, tile_height)?;
        let cells = (cells_x as usize)
            .checked_mul(cells_y as usize)
            .ok_or(Error::TooLarge("TMP cell count"))?;
        if r.remaining() / 4 < cells {
            return Err(Error::Truncated {
                needed: (FILE_HEADER_LEN as u64).saturating_add(4 * cells as u64),
            });
        }
        let mut offsets = Vec::with_capacity(cells);
        for _ in 0..cells {
            offsets.push(r.u32()?);
        }
        let mut tiles = Vec::with_capacity(cells);
        for offset in offsets {
            if offset == 0 {
                tiles.push(None);
                continue;
            }
            let mut t = Reader::at(data, offset as usize);
            tiles.push(Some(TmpTile {
                offset,
                x: t.i32()?,
                y: t.i32()?,
                extra_offset: t.i32()?,
                depth_offset: t.i32()?,
                extra_depth_offset: t.i32()?,
                extra_x: t.i32()?,
                extra_y: t.i32()?,
                extra_width: t.i32()?,
                extra_height: t.i32()?,
                flags: t.u32()?,
                height: t.u8()?,
                terrain_type: t.u8()?,
                ramp_type: t.u8()?,
                radar_left: t.array()?,
                radar_right: t.array()?,
            }));
            t.skip(3)?;
        }
        Ok(Self {
            cells_x,
            cells_y,
            tile_width,
            tile_height,
            tiles,
            data,
        })
    }

    /// Decodes the tile of cell `i`.
    pub fn decode_tile(&self, i: usize) -> Result<DecodedTile> {
        let tile = self
            .tiles
            .get(i)
            .copied()
            .flatten()
            .ok_or(Error::Invalid("TMP cell is empty or out of range"))?;
        let rows = diamond_rows(self.tile_width, self.tile_height)?;
        let n = (self.tile_width * self.tile_height / 2) as usize;
        let base = tile.offset as usize;
        let at = |rel: i32, len: usize| -> Result<&[u8]> {
            let rel = usize::try_from(rel).map_err(|_| Error::Invalid("negative TMP offset"))?;
            slice(self.data, base.saturating_add(rel), len)
        };
        let diamond_bytes = at(TILE_HEADER_LEN as i32, n)?;
        let expand = |bytes: &[u8]| {
            let (w, h) = (self.tile_width as usize, self.tile_height as usize);
            let mut px = vec![0u8; w * h];
            let mut src = bytes.iter();
            for (y, &rw) in rows.iter().enumerate() {
                let rw = rw as usize;
                let x0 = (w - rw) / 2;
                for x in 0..rw {
                    px[y * w + x0 + x] = src.next().copied().unwrap_or(0);
                }
            }
            IndexedImage::new(self.tile_width, self.tile_height, px)
                .map_err(|_| Error::Invalid("TMP tile size"))
        };
        let depth = if tile.flags & FLAG_DEPTH != 0 {
            Some(expand(at(tile.depth_offset, n)?)?)
        } else {
            None
        };
        let diamond = Placed {
            x: tile.x,
            y: tile.y,
            image: expand(diamond_bytes)?,
            depth,
        };
        let extra = if tile.has_extra() {
            let w =
                u32::try_from(tile.extra_width).map_err(|_| Error::Invalid("TMP extra size"))?;
            let h =
                u32::try_from(tile.extra_height).map_err(|_| Error::Invalid("TMP extra size"))?;
            let len = (w as usize)
                .checked_mul(h as usize)
                .filter(|&l| l <= MAX_IMAGE_PIXELS)
                .ok_or(Error::TooLarge("TMP extra image"))?;
            let image = IndexedImage::new(w, h, at(tile.extra_offset, len)?.to_vec())
                .map_err(|_| Error::Invalid("TMP extra size"))?;
            let depth = if tile.flags & FLAG_DEPTH != 0 {
                Some(
                    IndexedImage::new(w, h, at(tile.extra_depth_offset, len)?.to_vec())
                        .map_err(|_| Error::Invalid("TMP extra size"))?,
                )
            } else {
                None
            };
            Some(Placed {
                x: tile.extra_x,
                y: tile.extra_y,
                image,
                depth,
            })
        } else {
            None
        };
        Ok(DecodedTile { diamond, extra })
    }

    /// Draws every tile and extra image of the template into one image
    /// (painter's order: cells in order, each diamond then its extra; index
    /// 0 is transparent). Returns the image and the template coordinates of
    /// its top-left pixel.
    pub fn render(&self) -> Result<(IndexedImage, (i32, i32))> {
        let mut parts = Vec::new();
        for (i, t) in self.tiles.iter().enumerate() {
            if t.is_some() {
                let d = self.decode_tile(i)?;
                parts.push(d.diamond);
                parts.extend(d.extra);
            }
        }
        compose(&parts)
    }
}

/// Composes placed images (later ones drawn over earlier ones, index 0
/// transparent) into their bounding box.
pub fn compose(parts: &[Placed]) -> Result<(IndexedImage, (i32, i32))> {
    let bounds = parts
        .iter()
        .map(|p| {
            let x1 = i64::from(p.x) + i64::from(p.image.width());
            let y1 = i64::from(p.y) + i64::from(p.image.height());
            (i64::from(p.x), i64::from(p.y), x1, y1)
        })
        .reduce(|a, b| (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3)));
    let Some((x0, y0, x1, y1)) = bounds else {
        return Ok((IndexedImage::empty(), (0, 0)));
    };
    let (w, h) = ((x1 - x0) as usize, (y1 - y0) as usize);
    if w.checked_mul(h).is_none_or(|n| n > MAX_IMAGE_PIXELS) {
        return Err(Error::TooLarge("TMP template image"));
    }
    let mut px = vec![0u8; w * h];
    for p in parts {
        let (ox, oy) = (
            (i64::from(p.x) - x0) as usize,
            (i64::from(p.y) - y0) as usize,
        );
        let pw = p.image.width() as usize;
        for (row, src) in p.image.pixels().chunks_exact(pw.max(1)).enumerate() {
            for (col, &v) in src.iter().enumerate() {
                if v != 0 {
                    px[(oy + row) * w + ox + col] = v;
                }
            }
        }
    }
    let image = IndexedImage::new(w as u32, h as u32, px)
        .map_err(|_| Error::Invalid("TMP template image"))?;
    // In range: bounds come from i32 coordinates.
    Ok((image, (x0 as i32, y0 as i32)))
}

/// One tile for [`TmpWriter`].
#[derive(Clone, Debug)]
pub struct WriteTile {
    /// Header fields; offsets and flags are filled in by the writer.
    pub header: TmpTile,
    /// Diamond pixels in row order (`width × height / 2` bytes).
    pub diamond: Vec<u8>,
    /// Depth values for the diamond.
    pub depth: Option<Vec<u8>>,
    /// Extra image and its depth (size from the header's extra fields).
    pub extra: Option<(Vec<u8>, Option<Vec<u8>>)>,
}

/// Writes synthetic TMP-TS templates.
#[derive(Clone, Debug)]
pub struct TmpWriter {
    /// Cells across.
    pub cells_x: u32,
    /// Cells down.
    pub cells_y: u32,
    /// Diamond width.
    pub tile_width: u32,
    /// Diamond height.
    pub tile_height: u32,
    /// One per cell.
    pub tiles: Vec<Option<WriteTile>>,
}

impl TmpWriter {
    /// Serialises the template.
    pub fn write(&self) -> Vec<u8> {
        let cells = self.tiles.len();
        let mut out = Vec::new();
        for v in [
            self.cells_x,
            self.cells_y,
            self.tile_width,
            self.tile_height,
        ] {
            out.put_u32(v);
        }
        let table = out.len();
        out.resize(table + 4 * cells, 0);
        for (i, t) in self.tiles.iter().enumerate() {
            let Some(t) = t else { continue };
            let start = out.len();
            out[table + 4 * i..table + 4 * i + 4].copy_from_slice(&len_u32(start).to_le_bytes());
            let mut h = t.header;
            let mut body = Vec::new();
            body.extend_from_slice(&t.diamond);
            h.flags &= FLAG_DAMAGED;
            if let Some(d) = &t.depth {
                h.flags |= FLAG_DEPTH;
                h.depth_offset = (TILE_HEADER_LEN + body.len()) as i32;
                body.extend_from_slice(d);
            }
            if let Some((img, depth)) = &t.extra {
                h.flags |= FLAG_EXTRA;
                h.extra_offset = (TILE_HEADER_LEN + body.len()) as i32;
                body.extend_from_slice(img);
                if let Some(d) = depth {
                    h.extra_depth_offset = (TILE_HEADER_LEN + body.len()) as i32;
                    body.extend_from_slice(d);
                }
            }
            for v in [
                h.x,
                h.y,
                h.extra_offset,
                h.depth_offset,
                h.extra_depth_offset,
                h.extra_x,
                h.extra_y,
                h.extra_width,
                h.extra_height,
            ] {
                out.put_i32(v);
            }
            out.put_u32(h.flags);
            out.extend_from_slice(&[h.height, h.terrain_type, h.ramp_type]);
            out.extend_from_slice(&h.radar_left);
            out.extend_from_slice(&h.radar_right);
            out.extend_from_slice(&[0, 0, 0]);
            out.extend_from_slice(&body);
        }
        out
    }
}

/// A synthetic 2 × 2 template of invented tiles: one empty cell, one tile
/// with an extra image rising above it.
struct SyntheticTmp {
    tile_width: u32,
    tile_height: u32,
}

impl SyntheticTmp {
    fn writer(&self) -> TmpWriter {
        let (w, h) = (self.tile_width, self.tile_height);
        let n = (w * h / 2) as usize;
        let header = |cx: i32, cy: i32| TmpTile {
            offset: 0,
            x: (cx - cy) * w as i32 / 2,
            y: (cx + cy) * h as i32 / 2,
            extra_offset: 0,
            depth_offset: 0,
            extra_depth_offset: 0,
            extra_x: 0,
            extra_y: 0,
            extra_width: 0,
            extra_height: 0,
            flags: 0,
            height: (cx + cy) as u8,
            terrain_type: 1,
            ramp_type: 0,
            radar_left: [40, 90, 40],
            radar_right: [50, 100, 50],
        };
        let diamond = |ramp: u8| {
            (0..n)
                .map(|i| ramp * 16 + (i % 16) as u8)
                .collect::<Vec<_>>()
        };
        let depth = |_: u8| (0..n).map(|i| (i % 32) as u8).collect::<Vec<_>>();
        let mut raised = header(1, 0);
        raised.extra_x = raised.x + 4;
        raised.extra_y = raised.y - 10;
        raised.extra_width = 12;
        raised.extra_height = 14;
        let extra: Vec<u8> = (0..12 * 14)
            .map(|i| if i % 5 == 0 { 0 } else { 0x90 + (i % 7) as u8 })
            .collect();
        TmpWriter {
            cells_x: 2,
            cells_y: 2,
            tile_width: w,
            tile_height: h,
            tiles: vec![
                Some(WriteTile {
                    header: header(0, 0),
                    diamond: diamond(3),
                    depth: Some(depth(0)),
                    extra: None,
                }),
                Some(WriteTile {
                    header: raised,
                    diamond: diamond(5),
                    depth: Some(depth(1)),
                    extra: Some((extra.clone(), Some(vec![7; extra.len()]))),
                }),
                None,
                Some(WriteTile {
                    header: header(1, 1),
                    diamond: diamond(9),
                    depth: None,
                    extra: None,
                }),
            ],
        }
    }
}

impl FixtureWriter for SyntheticTmp {
    fn format(&self) -> &'static str {
        "tmp"
    }
    fn name(&self) -> String {
        format!("grid2x2-{}x{}", self.tile_width, self.tile_height)
    }
    fn extension(&self) -> &'static str {
        "tmp"
    }
    fn write(&self, out: &mut Vec<u8>) {
        out.extend(self.writer().write());
    }
}

/// Fixture writers for this module (two invented diamond sizes).
pub fn fixture_writers() -> Vec<Box<dyn FixtureWriter>> {
    vec![
        Box::new(SyntheticTmp {
            tile_width: 24,
            tile_height: 12,
        }),
        Box::new(SyntheticTmp {
            tile_width: 40,
            tile_height: 20,
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diamond_rows_add_up() {
        let rows = diamond_rows(24, 12).unwrap();
        assert_eq!(rows, [4, 8, 12, 16, 20, 24, 20, 16, 12, 8, 4, 0]);
        assert!(diamond_rows(24, 13).is_err());
        assert!(diamond_rows(10, 12).is_err());
    }

    #[test]
    fn writer_output_parses_back() {
        let w = SyntheticTmp {
            tile_width: 24,
            tile_height: 12,
        }
        .writer();
        let bytes = w.write();
        let tmp = Tmp::parse(&bytes).unwrap();
        assert_eq!((tmp.cells_x, tmp.cells_y), (2, 2));
        assert!(tmp.tiles[2].is_none());
        let t1 = tmp.decode_tile(1).unwrap();
        let extra = t1.extra.as_ref().unwrap();
        assert_eq!((extra.image.width(), extra.image.height()), (12, 14));
        assert_eq!(
            extra.image.pixels(),
            w.tiles[1].as_ref().unwrap().extra.as_ref().unwrap().0
        );
        assert!(tmp.decode_tile(3).unwrap().diamond.depth.is_none());
        // Top row of the diamond holds the first 4 indices, centred.
        assert_eq!(t1.diamond.image.get(10, 0), Some(5 * 16));
        assert_eq!(t1.diamond.image.get(9, 0), Some(0));
        let (img, origin) = tmp.render().unwrap();
        assert!(img.width() >= 24 && img.height() >= 12);
        assert!(origin.1 < 0, "extra image rises above the template");
        assert!(tmp.decode_tile(2).is_err());
    }

    #[test]
    fn truncation_never_panics() {
        let bytes = SyntheticTmp {
            tile_width: 24,
            tile_height: 12,
        }
        .writer()
        .write();
        for cut in 0..bytes.len() {
            if let Ok(tmp) = Tmp::parse(&bytes[..cut]) {
                for i in 0..tmp.tiles.len() {
                    let _ = tmp.decode_tile(i);
                }
                let _ = tmp.render();
            }
        }
    }
}
