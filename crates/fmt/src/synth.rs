//! Deterministic generators of invented content for fixtures, tests and demos.
//!
//! Everything here is our own work: procedural patterns and colour ramps that
//! do not imitate any game asset. Output depends only on the arguments, so a
//! fixture regenerated on any machine is byte-identical.

use crate::image::{IndexedImage, PALETTE_LEN, Palette, Rgb8};

/// Small seeded generator (SplitMix64) for synthetic test content.
///
/// This is **not** the simulation RNG; it only varies fixture bytes.
#[derive(Clone, Debug)]
pub struct SynthRng {
    state: u64,
}

impl SynthRng {
    /// Creates a generator from a seed.
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Fills `out` with random bytes.
    pub fn fill(&mut self, out: &mut [u8]) {
        for chunk in out.chunks_mut(8) {
            let bytes = self.next_u64().to_le_bytes();
            chunk.copy_from_slice(&bytes[..chunk.len()]);
        }
    }
}

/// Number of colour ramps in [`ramp_palette`].
pub const RAMPS: usize = 16;
/// Shades per ramp in [`ramp_palette`].
pub const SHADES: usize = PALETTE_LEN / RAMPS;

/// Hand-picked ramp end colours for [`ramp_palette`] (an arbitrary, invented
/// set chosen to be easy to tell apart on screen).
const RAMP_COLOURS: [Rgb8; RAMPS] = [
    Rgb8::new(232, 92, 72),
    Rgb8::new(244, 162, 64),
    Rgb8::new(236, 216, 88),
    Rgb8::new(150, 210, 84),
    Rgb8::new(72, 186, 120),
    Rgb8::new(60, 196, 196),
    Rgb8::new(70, 150, 230),
    Rgb8::new(98, 104, 222),
    Rgb8::new(150, 96, 220),
    Rgb8::new(206, 92, 196),
    Rgb8::new(230, 110, 150),
    Rgb8::new(176, 128, 96),
    Rgb8::new(132, 150, 140),
    Rgb8::new(190, 190, 176),
    Rgb8::new(110, 120, 150),
    Rgb8::new(226, 226, 232),
];

/// A palette of [`RAMPS`] ramps × [`SHADES`] shades: index `r * SHADES + s`
/// is ramp `r` at brightness `s` (0 darkest). `tint` rotates which colour each
/// ramp uses, giving a visibly different row for remap tests.
pub fn ramp_palette(tint: usize) -> Palette {
    let mut pal = Palette::default();
    for ramp in 0..RAMPS {
        let base = RAMP_COLOURS[(ramp + tint) % RAMPS];
        for shade in 0..SHADES {
            // Brightness from 1/4 up to full, linear in integer steps.
            let num = (SHADES + 3 * shade) as u32;
            let den = (4 * SHADES - 3) as u32;
            let scale = |c: u8| ((u32::from(c) * num) / den).min(255) as u8;
            pal.colors[ramp * SHADES + shade] =
                Rgb8::new(scale(base.r), scale(base.g), scale(base.b));
        }
    }
    pal
}

/// A checkerboard of `cell`-pixel squares. Each square uses its own ramp from
/// [`ramp_palette`], and shades run diagonally across the square so palette
/// cycling is visible.
///
/// # Panics
/// If `cell` is zero.
pub fn checkerboard(width: u32, height: u32, cell: u32) -> IndexedImage {
    assert!(cell > 0, "cell size must be positive");
    let mut pixels = Vec::with_capacity(width as usize * height as usize);
    for y in 0..height {
        for x in 0..width {
            let (cx, cy) = (x / cell, y / cell);
            let ramp = if (cx + cy) % 2 == 0 {
                (cx + 2 * cy) as usize % RAMPS
            } else {
                RAMPS - 1 - (cx + cy) as usize % (RAMPS / 2)
            };
            let (fx, fy) = (x % cell, y % cell);
            let shade = ((fx + fy) * SHADES as u32 / (2 * cell)) as usize;
            pixels.push((ramp * SHADES + shade) as u8);
        }
    }
    IndexedImage::new(width, height, pixels).expect("dimensions match by construction")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rng_is_reproducible() {
        let mut a = SynthRng::new(7);
        let mut b = SynthRng::new(7);
        let (mut x, mut y) = ([0u8; 13], [0u8; 13]);
        a.fill(&mut x);
        b.fill(&mut y);
        assert_eq!(x, y);
        assert_ne!(SynthRng::new(8).next_u64(), SynthRng::new(7).next_u64());
    }

    #[test]
    fn ramps_get_brighter() {
        let pal = ramp_palette(0);
        for ramp in 0..RAMPS {
            let first = pal.colors[ramp * SHADES];
            let last = pal.colors[ramp * SHADES + SHADES - 1];
            assert!(
                u32::from(first.r) + u32::from(first.g) + u32::from(first.b)
                    < u32::from(last.r) + u32::from(last.g) + u32::from(last.b)
            );
        }
    }

    #[test]
    fn tint_changes_the_palette() {
        assert_ne!(ramp_palette(0), ramp_palette(3));
    }

    #[test]
    fn checkerboard_neighbours_differ() {
        let img = checkerboard(32, 32, 8);
        let ramp = |x, y| img.get(x, y).unwrap() as usize / SHADES;
        assert_ne!(ramp(0, 0), ramp(8, 0));
        assert_ne!(ramp(0, 0), ramp(0, 8));
        assert_eq!(img.get(0, 0), checkerboard(32, 32, 8).get(0, 0));
    }
}
