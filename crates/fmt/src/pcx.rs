//! PCX images (ZSoft format): 8-bit indexed with a 256-colour palette at the
//! end of the file, or 24-bit as three 8-bit planes.
//!
//! 128-byte header: `u8 manufacturer (10), u8 version, u8 encoding (1:
//! RLE), u8 bits per pixel, u16 xmin, ymin, xmax, ymax, u16 dpi × 2,
//! 48-byte EGA palette, u8 reserved, u8 planes, u16 bytes per plane line,
//! u16 palette info, u16 screen size × 2, 54 bytes filler`. RLE: a byte
//! with both top bits set repeats the next byte `byte & 63` times; any other
//! byte is one literal. The 256-colour palette follows a 12 byte marker at
//! the end of the file. This is the public ZSoft layout (also in XCC's
//! `cc_structures.h`).

use crate::bytes::{PutLe, Reader, len_u16, slice};
use crate::error::{Error, Result};
use crate::fixture::FixtureWriter;
use crate::image::{IndexedImage, PALETTE_LEN, Palette, Rgb8};
use crate::limits::MAX_IMAGE_PIXELS;

/// Header length.
pub const HEADER_LEN: usize = 128;
/// Marker before the trailing 256-colour palette.
const PALETTE_MARKER: u8 = 12;

/// A decoded PCX image.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pcx {
    /// 8-bit indexed, with its palette.
    Indexed {
        /// Pixels.
        image: IndexedImage,
        /// The file's palette.
        palette: Box<Palette>,
    },
    /// 24-bit colour.
    Rgb {
        /// Width.
        width: u32,
        /// Height.
        height: u32,
        /// `width × height` RGB triples, row-major.
        pixels: Vec<u8>,
    },
}

impl Pcx {
    /// Width in pixels.
    pub fn width(&self) -> u32 {
        match self {
            Self::Indexed { image, .. } => image.width(),
            Self::Rgb { width, .. } => *width,
        }
    }

    /// Height in pixels.
    pub fn height(&self) -> u32 {
        match self {
            Self::Indexed { image, .. } => image.height(),
            Self::Rgb { height, .. } => *height,
        }
    }

    /// Decodes a PCX file.
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        let [manufacturer, _version, encoding, bpp] = r.array()?;
        if manufacturer != 10 {
            return Err(Error::Invalid("PCX manufacturer byte is not 10"));
        }
        if encoding > 1 {
            return Err(Error::Unsupported("PCX encoding"));
        }
        let (xmin, ymin, xmax, ymax) = (r.u16()?, r.u16()?, r.u16()?, r.u16()?);
        if xmax < xmin || ymax < ymin {
            return Err(Error::Invalid("PCX bounds"));
        }
        let mut h = Reader::at(data, 65);
        let planes = h.u8()?;
        let bytes_per_line = usize::from(h.u16()?);
        slice(data, 0, HEADER_LEN)?;
        let width = usize::from(xmax - xmin) + 1;
        let height = usize::from(ymax - ymin) + 1;
        if width * height > MAX_IMAGE_PIXELS {
            return Err(Error::TooLarge("PCX image"));
        }
        if bpp != 8 || !(planes == 1 || planes == 3) {
            return Err(Error::Unsupported(
                "PCX depth other than 8-bit × 1 or 3 planes",
            ));
        }
        if bytes_per_line < width {
            return Err(Error::Invalid("PCX line shorter than the image"));
        }
        let line = bytes_per_line * usize::from(planes);
        let total = line
            .checked_mul(height)
            .filter(|&t| t <= MAX_IMAGE_PIXELS * 3)
            .ok_or(Error::TooLarge("PCX image"))?;
        let body = data.get(HEADER_LEN..).unwrap_or_default();
        let raw = if encoding == 1 {
            rle_decode(body, total)?
        } else {
            slice(body, 0, total)?.to_vec()
        };
        if planes == 1 {
            let mut px = Vec::with_capacity(width * height);
            for row in raw.chunks_exact(line) {
                px.extend_from_slice(&row[..width]);
            }
            let image = IndexedImage::new(width as u32, height as u32, px)
                .map_err(|_| Error::Invalid("PCX size"))?;
            let tail_start = data
                .len()
                .checked_sub(PALETTE_LEN * 3 + 1)
                .ok_or(Error::Invalid("PCX file has no 256-colour palette"))?;
            let tail = slice(data, tail_start, PALETTE_LEN * 3 + 1)?;
            if tail[0] != PALETTE_MARKER || tail_start < HEADER_LEN {
                return Err(Error::Invalid("PCX file has no 256-colour palette"));
            }
            let mut palette = Palette::default();
            for (dst, c) in palette.colors.iter_mut().zip(tail[1..].as_chunks::<3>().0) {
                *dst = Rgb8::new(c[0], c[1], c[2]);
            }
            Ok(Self::Indexed {
                image,
                palette: Box::new(palette),
            })
        } else {
            let mut px = Vec::with_capacity(width * height * 3);
            for row in raw.chunks_exact(line) {
                let (r, rest) = row.split_at(bytes_per_line);
                let (g, b) = rest.split_at(bytes_per_line);
                for x in 0..width {
                    px.extend_from_slice(&[r[x], g[x], b[x]]);
                }
            }
            Ok(Self::Rgb {
                width: width as u32,
                height: height as u32,
                pixels: px,
            })
        }
    }

    /// Writes an RLE-encoded PCX (lines padded to an even length).
    pub fn write(&self) -> Vec<u8> {
        let (w, h, planes) = match self {
            Self::Indexed { image, .. } => (image.width() as usize, image.height() as usize, 1u8),
            Self::Rgb { width, height, .. } => (*width as usize, *height as usize, 3),
        };
        let bpl = w + (w % 2);
        let mut out = vec![10, 5, 1, 8];
        for v in [
            0,
            0,
            len_u16(w.saturating_sub(1)),
            len_u16(h.saturating_sub(1)),
            72,
            72,
        ] {
            out.put_u16(v);
        }
        out.resize(65, 0);
        out.push(planes);
        out.put_u16(len_u16(bpl));
        out.put_u16(1);
        out.resize(HEADER_LEN, 0);
        let mut raw = Vec::with_capacity(bpl * usize::from(planes) * h);
        for y in 0..h {
            match self {
                Self::Indexed { image, .. } => {
                    raw.extend_from_slice(&image.pixels()[y * w..(y + 1) * w]);
                    raw.resize(raw.len() + bpl - w, 0);
                }
                Self::Rgb { pixels, .. } => {
                    for c in 0..3 {
                        raw.extend((0..w).map(|x| pixels[(y * w + x) * 3 + c]));
                        raw.resize(raw.len() + bpl - w, 0);
                    }
                }
            }
        }
        for line in raw.chunks(bpl.max(1)) {
            rle_encode(line, &mut out);
        }
        if let Self::Indexed { palette, .. } = self {
            out.push(PALETTE_MARKER);
            for c in &palette.colors {
                out.extend_from_slice(&[c.r, c.g, c.b]);
            }
        }
        out
    }
}

fn rle_decode(body: &[u8], total: usize) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(total);
    let mut it = body.iter();
    while out.len() < total {
        let &b = it.next().ok_or(Error::Truncated {
            needed: (HEADER_LEN + body.len() + 1) as u64,
        })?;
        if b >= 0xC0 {
            let &v = it.next().ok_or(Error::Truncated {
                needed: (HEADER_LEN + body.len() + 1) as u64,
            })?;
            let n = usize::from(b & 0x3F).min(total - out.len());
            out.extend(std::iter::repeat_n(v, n));
        } else {
            out.push(b);
        }
    }
    Ok(out)
}

fn rle_encode(line: &[u8], out: &mut Vec<u8>) {
    let mut i = 0;
    while i < line.len() {
        let v = line[i];
        let run = line[i..].iter().take(63).take_while(|&&b| b == v).count();
        if run > 1 || v >= 0xC0 {
            out.extend_from_slice(&[0xC0 | run as u8, v]);
        } else {
            out.push(v);
        }
        i += run;
    }
}

/// Synthetic PCX fixtures: an indexed checkerboard and an RGB gradient.
struct SyntheticPcx {
    rgb: bool,
}

impl SyntheticPcx {
    fn image(&self) -> Pcx {
        if self.rgb {
            let (w, h) = (21u32, 9u32);
            let pixels = (0..h)
                .flat_map(|y| (0..w).flat_map(move |x| [(x * 12) as u8, (y * 28) as u8, 200]))
                .collect();
            Pcx::Rgb {
                width: w,
                height: h,
                pixels,
            }
        } else {
            Pcx::Indexed {
                image: crate::synth::checkerboard(33, 17, 4),
                palette: Box::new(crate::synth::ramp_palette(2)),
            }
        }
    }
}

impl FixtureWriter for SyntheticPcx {
    fn format(&self) -> &'static str {
        "pcx"
    }
    fn name(&self) -> String {
        if self.rgb {
            "gradient-rgb"
        } else {
            "checker-indexed"
        }
        .into()
    }
    fn extension(&self) -> &'static str {
        "pcx"
    }
    fn write(&self, out: &mut Vec<u8>) {
        out.extend(self.image().write());
    }
}

/// Fixture writers for this module.
pub fn fixture_writers() -> Vec<Box<dyn FixtureWriter>> {
    vec![
        Box::new(SyntheticPcx { rgb: false }),
        Box::new(SyntheticPcx { rgb: true }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        for rgb in [false, true] {
            let img = SyntheticPcx { rgb }.image();
            let bytes = img.write();
            assert_eq!(Pcx::parse(&bytes), Ok(img));
        }
    }

    #[test]
    fn rle_by_hand() {
        // 0xC3 0x07: three 7s; 0x05 literal; 0xC1 0xC8: one 0xC8.
        assert_eq!(
            rle_decode(&[0xC3, 7, 5, 0xC1, 0xC8], 5),
            Ok(vec![7, 7, 7, 5, 0xC8])
        );
        assert!(rle_decode(&[0xC3], 3).is_err());
    }

    #[test]
    fn truncation_never_panics() {
        let bytes = SyntheticPcx { rgb: false }.image().write();
        for cut in 0..bytes.len() {
            let _ = Pcx::parse(&bytes[..cut]);
        }
    }
}
