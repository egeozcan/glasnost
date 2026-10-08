//! SHP sprites, TS/RA2 variant ("SHP-TS"): a canvas size and a list of
//! frames, each a rectangle placed on the canvas.
//!
//! File header: `u16 0, u16 canvas width, u16 canvas height, u16 frames`.
//! Each frame header (24 bytes): `u16 x, u16 y, u16 width, u16 height,
//! u8 compression, 3 raw bytes, u32 colour, u32 reserved, u32 data offset`
//! (offset 0: the frame has no pixels). Only the low byte of the
//! compression word is the mode, as OpenRA reads it; real files carry
//! other values in the next three bytes, and other modes on empty frames.
//! Compression:
//!
//! - 0 and 1: `width × height` raw indices;
//! - 2: per row, a `u16` row length (counting itself) then raw indices;
//! - 3: per row, a `u16` row length (counting itself) then zero-run
//!   encoded indices: a non-zero byte is one pixel, `0, n` is `n`
//!   transparent pixels. A row's last run may reach past the row end; the
//!   excess is dropped (real files end rows one pixel past).
//!
//! Layout from XCC's `cc_structures.h` and `shp_decode.cpp` (decode3) and
//! OpenRA's `ShpTSLoader.cs` (GPL-3.0-or-later; XCC Copyright (C) 2000 Olaf
//! van der Spek, OpenRA Copyright (c) The OpenRA Developers and
//! Contributors). The two disagree on compression 2: XCC decodes it like 3,
//! OpenRA reads one length prefix for the whole frame; we read a prefix per
//! row, as for 3, and the local lane checks that against real files.

use crate::bytes::{PutLe, Reader, len_u16, len_u32};
use crate::error::{Error, Result};
use crate::fixture::FixtureWriter;
use crate::image::IndexedImage;
use crate::limits::MAX_IMAGE_PIXELS;

/// Length of the file header.
pub const HEADER_LEN: usize = 8;
/// Length of one frame header.
pub const FRAME_HEADER_LEN: usize = 24;

/// One frame header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShpFrame {
    /// Left edge on the canvas.
    pub x: u16,
    /// Top edge on the canvas.
    pub y: u16,
    /// Width of the frame's rectangle.
    pub width: u16,
    /// Height of the frame's rectangle.
    pub height: u16,
    /// Compression mode (0–3 for frames with pixels; see the module docs).
    pub compression: u8,
    /// The three raw bytes after the compression mode.
    pub compression_extra: [u8; 3],
    /// Colour field, kept raw.
    pub color: u32,
    /// Reserved field, kept raw.
    pub reserved: u32,
    /// Offset of the pixel data in the file; 0 for an empty frame.
    pub offset: u32,
}

impl ShpFrame {
    /// Whether the frame has no pixels.
    pub fn is_empty(&self) -> bool {
        self.offset == 0 || self.width == 0 || self.height == 0
    }
}

/// A parsed SHP-TS file; frames are decoded on demand.
#[derive(Clone, Debug)]
pub struct Shp<'a> {
    /// Canvas width.
    pub width: u16,
    /// Canvas height.
    pub height: u16,
    /// Frame headers.
    pub frames: Vec<ShpFrame>,
    data: &'a [u8],
}

impl<'a> Shp<'a> {
    /// Parses the headers. Pixel data is checked when a frame is decoded.
    pub fn parse(data: &'a [u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        if r.u16()? != 0 {
            return Err(Error::Invalid("SHP-TS header does not start with zero"));
        }
        let width = r.u16()?;
        let height = r.u16()?;
        let count = usize::from(r.u16()?);
        if r.remaining() < count * FRAME_HEADER_LEN {
            return Err(Error::Truncated {
                needed: (HEADER_LEN + count * FRAME_HEADER_LEN) as u64,
            });
        }
        let mut frames = Vec::with_capacity(count);
        for _ in 0..count {
            let f = ShpFrame {
                x: r.u16()?,
                y: r.u16()?,
                width: r.u16()?,
                height: r.u16()?,
                compression: r.u8()?,
                compression_extra: r.array()?,
                color: r.u32()?,
                reserved: r.u32()?,
                offset: r.u32()?,
            };
            if !f.is_empty() && f.compression > 3 {
                return Err(Error::Unsupported("SHP-TS compression above 3"));
            }
            frames.push(f);
        }
        Ok(Self {
            width,
            height,
            frames,
            data,
        })
    }

    /// Decodes frame `i` into an image of the frame's own rectangle
    /// (`width × height`); empty frames give a 0 × 0 image.
    pub fn decode_frame(&self, i: usize) -> Result<IndexedImage> {
        self.decode_frame_stats(i).map(|(img, _)| img)
    }

    /// Like [`Self::decode_frame`], also reporting how lenient the decoder
    /// had to be.
    pub fn decode_frame_stats(&self, i: usize) -> Result<(IndexedImage, FrameStats)> {
        let f = self
            .frames
            .get(i)
            .ok_or(Error::Invalid("SHP frame index out of range"))?;
        let mut stats = FrameStats::default();
        let img = decode_frame(self.data, f, &mut stats)?;
        Ok((img, stats))
    }

    /// Decodes frame `i` placed on the canvas (pixels outside it clipped).
    pub fn decode_on_canvas(&self, i: usize) -> Result<IndexedImage> {
        let (cw, ch) = (usize::from(self.width), usize::from(self.height));
        if cw * ch > MAX_IMAGE_PIXELS {
            return Err(Error::TooLarge("SHP canvas"));
        }
        let mut canvas = vec![0u8; cw * ch];
        let f = self
            .frames
            .get(i)
            .ok_or(Error::Invalid("SHP frame index out of range"))?;
        let img = decode_frame(self.data, f, &mut FrameStats::default())?;
        let (fx, fy) = (usize::from(f.x), usize::from(f.y));
        let fw = img.width() as usize;
        for (row, src) in img.pixels().chunks_exact(fw.max(1)).enumerate() {
            let y = fy + row;
            if y >= ch || fx >= cw {
                continue;
            }
            let n = fw.min(cw - fx);
            canvas[y * cw + fx..y * cw + fx + n].copy_from_slice(&src[..n]);
        }
        IndexedImage::new(u32::from(self.width), u32::from(self.height), canvas)
            .map_err(|_| Error::Invalid("SHP canvas size"))
    }
}

/// Leniency counters from decoding one frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameStats {
    /// Rows that ended before the frame width (padded with index 0).
    pub short_rows: u32,
    /// Zero runs cut at the row end.
    pub clamped_runs: u32,
    /// Of those, runs that were not the row's last command or reached more
    /// than one pixel past the end.
    pub clamped_runs_unusual: u32,
}

fn decode_frame(data: &[u8], f: &ShpFrame, stats: &mut FrameStats) -> Result<IndexedImage> {
    if f.is_empty() {
        return Ok(IndexedImage::empty());
    }
    let (w, h) = (usize::from(f.width), usize::from(f.height));
    if w * h > MAX_IMAGE_PIXELS {
        return Err(Error::TooLarge("SHP frame"));
    }
    let start = usize::try_from(f.offset).map_err(|_| Error::Invalid("SHP offset"))?;
    let mut r = Reader::at(data, start);
    let mut pixels = Vec::with_capacity(w * h);
    match f.compression {
        0 | 1 => pixels.extend_from_slice(r.bytes(w * h)?),
        2 | 3 => {
            for _ in 0..h {
                let len = usize::from(r.u16()?)
                    .checked_sub(2)
                    .ok_or(Error::Invalid("SHP row length below 2"))?;
                let row = r.bytes(len)?;
                let before = pixels.len();
                if f.compression == 2 {
                    if row.len() > w {
                        return Err(Error::Invalid("SHP row longer than the frame"));
                    }
                    pixels.extend_from_slice(row);
                } else {
                    let (cut, unusual) = decode_zero_runs(row, w, &mut pixels)?;
                    stats.clamped_runs += cut;
                    stats.clamped_runs_unusual += unusual;
                }
                // Short rows end in transparent pixels.
                if pixels.len() < before + w {
                    stats.short_rows += 1;
                }
                pixels.resize(before + w, 0);
            }
        }
        _ => return Err(Error::Unsupported("SHP-TS compression above 3")),
    }
    IndexedImage::new(u32::from(f.width), u32::from(f.height), pixels)
        .map_err(|_| Error::Invalid("SHP frame size"))
}

/// Appends one zero-run encoded row of at most `width` pixels. Zero runs
/// that pass the row end are cut at the row end (as XCC's decoder does);
/// opaque pixels past it are an error. Returns the number of cut runs and
/// how many of them were unusual (not last, or more than one past the end).
fn decode_zero_runs(row: &[u8], width: usize, out: &mut Vec<u8>) -> Result<(u32, u32)> {
    let (mut clamped, mut unusual) = (0, 0);
    let mut x = 0usize;
    let mut i = 0usize;
    while let Some(&v) = row.get(i) {
        i += 1;
        if v != 0 {
            if x >= width {
                return Err(Error::Invalid("SHP row longer than the frame"));
            }
            out.push(v);
            x += 1;
        } else {
            let n = usize::from(*row.get(i).ok_or(Error::Invalid("SHP zero run cut off"))?);
            i += 1;
            if n > width - x {
                clamped += 1;
                if n > width - x + 1 || i < row.len() {
                    unusual += 1;
                }
            }
            let n = n.min(width - x);
            out.extend(std::iter::repeat_n(0, n));
            x += n;
        }
    }
    Ok((clamped, unusual))
}

/// Encodes one row with zero runs (runs of at most 255).
fn encode_zero_runs(row: &[u8], out: &mut Vec<u8>) {
    let mut i = 0;
    while i < row.len() {
        if row[i] == 0 {
            let run = row[i..].iter().take(255).take_while(|&&b| b == 0).count();
            out.extend_from_slice(&[0, run as u8]);
            i += run;
        } else {
            out.push(row[i]);
            i += 1;
        }
    }
}

/// One frame for [`ShpWriter`].
#[derive(Clone, Debug)]
pub struct WriteFrame {
    /// Placement on the canvas.
    pub x: u16,
    /// Placement on the canvas.
    pub y: u16,
    /// Pixels; a 0 × 0 image writes an empty frame.
    pub image: IndexedImage,
    /// Compression (1, 2 or 3).
    pub compression: u8,
}

/// Writes synthetic SHP-TS files.
#[derive(Clone, Debug)]
pub struct ShpWriter {
    /// Canvas width.
    pub width: u16,
    /// Canvas height.
    pub height: u16,
    /// Frames in order.
    pub frames: Vec<WriteFrame>,
}

impl ShpWriter {
    /// Serialises the file.
    ///
    /// # Panics
    /// If a frame is larger than 65535 pixels on a side or the file exceeds
    /// 4 GiB (writer inputs are our own).
    pub fn write(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.put_u16(0);
        out.put_u16(self.width);
        out.put_u16(self.height);
        out.put_u16(len_u16(self.frames.len()));
        let mut data = Vec::new();
        let data_start = HEADER_LEN + self.frames.len() * FRAME_HEADER_LEN;
        for f in &self.frames {
            let (w, h) = (f.image.width() as usize, f.image.height() as usize);
            let empty = w == 0 || h == 0;
            let offset = if empty {
                0
            } else {
                len_u32(data_start + data.len())
            };
            if !empty {
                for row in f.image.pixels().chunks_exact(w) {
                    match f.compression {
                        2 => {
                            data.put_u16(len_u16(row.len() + 2));
                            data.extend_from_slice(row);
                        }
                        3 => {
                            let mut enc = Vec::new();
                            encode_zero_runs(row, &mut enc);
                            data.put_u16(len_u16(enc.len() + 2));
                            data.extend_from_slice(&enc);
                        }
                        _ => data.extend_from_slice(row),
                    }
                }
            }
            out.put_u16(f.x);
            out.put_u16(f.y);
            out.put_u16(len_u16(w));
            out.put_u16(len_u16(h));
            out.extend_from_slice(&[f.compression, 0, 0, 0]);
            out.put_u32(0);
            out.put_u32(0);
            out.put_u32(offset);
        }
        out.extend_from_slice(&data);
        out
    }
}

/// Whether a file of `len` bytes starting with `prefix` looks like an
/// SHP-TS file (a permissive check for classification; [`Shp::parse`] is
/// the strict one).
pub fn looks_like_shp(prefix: &[u8], len: u64) -> bool {
    let mut r = Reader::new(prefix);
    match (r.u16(), r.u16(), r.u16(), r.u16()) {
        (Ok(0), Ok(w), Ok(h), Ok(n)) => {
            w > 0
                && h > 0
                && n > 0
                && len >= (HEADER_LEN + usize::from(n) * FRAME_HEADER_LEN) as u64
        }
        _ => false,
    }
}

/// A synthetic sprite: a few frames of invented shapes, one per
/// compression mode, plus an empty frame.
struct SyntheticShp {
    seed: u64,
}

impl SyntheticShp {
    fn writer(&self) -> ShpWriter {
        let mut rng = crate::synth::SynthRng::new(self.seed);
        let mut frames = Vec::new();
        for (i, compression) in [1u8, 2, 3, 3].into_iter().enumerate() {
            let w = 8 + (rng.next_u64() % 24) as u32;
            let h = 6 + (rng.next_u64() % 20) as u32;
            // A disc with a ring, transparent outside: exercises zero runs.
            let mut pixels = Vec::with_capacity((w * h) as usize);
            for y in 0..h {
                for x in 0..w {
                    let (dx, dy) = (2 * x as i64 - w as i64 + 1, 2 * y as i64 - h as i64 + 1);
                    let d = (dx * dx + dy * dy) as u64;
                    let r2 = u64::from(w.min(h)).pow(2);
                    pixels.push(if d > r2 {
                        0
                    } else {
                        ((d * 15 / r2.max(1)) as u8) + 16 * (i as u8 + 1)
                    });
                }
            }
            frames.push(WriteFrame {
                x: (rng.next_u64() % 8) as u16,
                y: (rng.next_u64() % 8) as u16,
                image: IndexedImage::new(w, h, pixels).unwrap_or_else(|_| IndexedImage::empty()),
                compression,
            });
        }
        frames.push(WriteFrame {
            x: 0,
            y: 0,
            image: IndexedImage::empty(),
            compression: 0,
        });
        ShpWriter {
            width: 40,
            height: 34,
            frames,
        }
    }
}

impl FixtureWriter for SyntheticShp {
    fn format(&self) -> &'static str {
        "shp"
    }
    fn name(&self) -> String {
        format!("discs-seed{}", self.seed)
    }
    fn extension(&self) -> &'static str {
        "shp"
    }
    fn write(&self, out: &mut Vec<u8>) {
        out.extend(self.writer().write());
    }
}

/// Fixture writers for this module.
pub fn fixture_writers() -> Vec<Box<dyn FixtureWriter>> {
    vec![
        Box::new(SyntheticShp { seed: 1 }),
        Box::new(SyntheticShp { seed: 2 }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writer_output_parses_back() {
        let w = SyntheticShp { seed: 5 }.writer();
        let bytes = w.write();
        assert!(looks_like_shp(&bytes[..16], bytes.len() as u64));
        let shp = Shp::parse(&bytes).unwrap();
        assert_eq!(shp.frames.len(), w.frames.len());
        for (i, f) in w.frames.iter().enumerate() {
            assert_eq!(shp.decode_frame(i).unwrap(), f.image, "frame {i}");
            assert_eq!(shp.frames[i].compression, f.compression);
        }
        assert!(shp.frames.last().unwrap().is_empty());
        let canvas = shp.decode_on_canvas(0).unwrap();
        let f = &shp.frames[0];
        let src = shp.decode_frame(0).unwrap();
        assert_eq!(canvas.get(u32::from(f.x), u32::from(f.y)), src.get(0, 0));
    }

    #[test]
    fn zero_runs_decode_by_hand() {
        // Row of width 6: pixel 5, a run of two zeros, pixel 7, then a run
        // of 9 zeros that is cut at the row end (two remain).
        let mut out = Vec::new();
        assert_eq!(
            decode_zero_runs(&[5, 0, 2, 7, 0, 9], 6, &mut out),
            Ok((1, 1))
        );
        assert_eq!(out, [5, 0, 0, 7, 0, 0]);
        // Final run one past the end: the usual case in real files.
        let mut out = Vec::new();
        assert_eq!(decode_zero_runs(&[5, 0, 6], 6, &mut out), Ok((1, 0)));
        assert_eq!(out, [5, 0, 0, 0, 0, 0]);
        assert!(decode_zero_runs(&[1, 2, 3], 2, &mut Vec::new()).is_err());
        assert!(decode_zero_runs(&[1, 0], 4, &mut Vec::new()).is_err());
    }

    #[test]
    fn truncation_never_panics() {
        let bytes = SyntheticShp { seed: 9 }.writer().write();
        for cut in 0..bytes.len() {
            if let Ok(shp) = Shp::parse(&bytes[..cut]) {
                for i in 0..shp.frames.len() {
                    let _ = shp.decode_frame(i);
                    let _ = shp.decode_on_canvas(i);
                }
            }
        }
    }

    #[test]
    fn rejects_bad_headers() {
        assert!(Shp::parse(&[1, 0, 0, 0, 0, 0, 0, 0]).is_err());
        let mut bytes = SyntheticShp { seed: 3 }.writer().write();
        bytes[HEADER_LEN + 8] = 9; // first frame's compression
        assert!(Shp::parse(&bytes).is_err());
    }
}
