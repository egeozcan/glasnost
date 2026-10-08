//! Format-neutral decoded images: palette indices plus palettes.
//!
//! Sprite, tile and font parsers (M1) decode into [`IndexedImage`]; palette
//! parsers decode into [`Palette`]. The renderer uploads both unchanged and
//! resolves colours on the GPU, so house-colour remapping and palette effects
//! never touch the decoded pixels.

/// An 8-bit RGB colour as stored in a decoded palette.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Rgb8 {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
}

impl Rgb8 {
    /// Builds a colour from its channels.
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// Number of entries in a palette.
pub const PALETTE_LEN: usize = 256;

/// A full 256-entry palette.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Palette {
    /// Colour for each palette index.
    pub colors: [Rgb8; PALETTE_LEN],
}

impl Palette {
    /// The palette as tightly packed RGBA bytes (alpha 255), ready for upload
    /// as one row of a palette texture.
    pub fn to_rgba8(&self) -> [u8; PALETTE_LEN * 4] {
        let mut out = [0u8; PALETTE_LEN * 4];
        for (dst, c) in out
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(self.colors.iter())
        {
            *dst = [c.r, c.g, c.b, 255];
        }
        out
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            colors: [Rgb8::default(); PALETTE_LEN],
        }
    }
}

/// A rectangular image of palette indices, row-major, top row first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexedImage {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

/// Error returned when pixel data does not match the stated dimensions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SizeMismatch {
    /// `width * height`.
    pub expected: usize,
    /// Length of the pixel buffer that was supplied.
    pub actual: usize,
}

impl std::fmt::Display for SizeMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "pixel buffer holds {} indices, dimensions need {}",
            self.actual, self.expected
        )
    }
}

impl std::error::Error for SizeMismatch {}

impl IndexedImage {
    /// Wraps a pixel buffer; fails if its length is not `width * height`.
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self, SizeMismatch> {
        let expected = width as usize * height as usize;
        if pixels.len() != expected {
            return Err(SizeMismatch {
                expected,
                actual: pixels.len(),
            });
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    /// Width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// All indices, row-major.
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// The index at `(x, y)`, or `None` outside the image.
    pub fn get(&self, x: u32, y: u32) -> Option<u8> {
        if x < self.width && y < self.height {
            Some(self.pixels[y as usize * self.width as usize + x as usize])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_wrong_pixel_count() {
        let err = IndexedImage::new(3, 2, vec![0; 5]).unwrap_err();
        assert_eq!(
            err,
            SizeMismatch {
                expected: 6,
                actual: 5
            }
        );
    }

    #[test]
    fn get_is_row_major() {
        let img = IndexedImage::new(2, 2, vec![1, 2, 3, 4]).unwrap();
        assert_eq!(img.get(1, 0), Some(2));
        assert_eq!(img.get(0, 1), Some(3));
        assert_eq!(img.get(2, 0), None);
    }

    #[test]
    fn rgba_rows_are_opaque() {
        let mut pal = Palette::default();
        pal.colors[1] = Rgb8::new(9, 8, 7);
        let rgba = pal.to_rgba8();
        assert_eq!(&rgba[4..8], &[9, 8, 7, 255]);
        assert_eq!(rgba[3], 255);
    }
}
