//! House-colour remap rows and CPU palette resolution.
//!
//! **Provisional.** Which palette indices take the owner's colour, and how
//! the owner's colour becomes a ramp of shades, are game behaviour asked in
//! spec request Q-0001. Until it is answered nothing here has a default
//! range: callers pass a [`RemapRange`] the user chose, and the ramp rule
//! ([`shade_ramp`]) is a plain linear ramp of our own with a selectable
//! order.

use glasnost_fmt::image::{IndexedImage, PALETTE_LEN, Palette, Rgb8};

/// Palette indices replaced by the house colour: `start .. start + len`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemapRange {
    /// First index.
    pub start: u8,
    /// Number of indices (0 disables the remap).
    pub len: u8,
}

/// Order of shades within a ramp.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RampOrder {
    /// The first remapped index gets the brightest shade.
    #[default]
    BrightestFirst,
    /// The first remapped index gets the darkest shade.
    DarkestFirst,
}

/// `len` shades of `colour`, from full brightness down to `1/len` of it
/// (linear, integer), in the given order.
pub fn shade_ramp(colour: Rgb8, len: usize, order: RampOrder) -> Vec<Rgb8> {
    let n = len.max(1) as u32;
    let mut ramp: Vec<Rgb8> = (0..len as u32)
        .map(|k| {
            let s = |c: u8| (u32::from(c) * (n - k) / n) as u8;
            Rgb8::new(s(colour.r), s(colour.g), s(colour.b))
        })
        .collect();
    if order == RampOrder::DarkestFirst {
        ramp.reverse();
    }
    ramp
}

/// `base` with the indices of `range` replaced by `ramp` (cycled if the
/// ramp is shorter; indices past 255 are ignored).
pub fn remap_row(base: &Palette, range: RemapRange, ramp: &[Rgb8]) -> Palette {
    let mut row = base.clone();
    if ramp.is_empty() {
        return row;
    }
    for k in 0..usize::from(range.len) {
        let i = usize::from(range.start) + k;
        if i >= PALETTE_LEN {
            break;
        }
        row.colors[i] = ramp[k % ramp.len()];
    }
    row
}

/// Resolves an indexed image to RGBA bytes through `palette`. With
/// `transparent_zero`, index 0 becomes fully transparent (the sprite and
/// tile formats' convention for "no pixel").
pub fn resolve_rgba(image: &IndexedImage, palette: &Palette, transparent_zero: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(image.pixels().len() * 4);
    for &i in image.pixels() {
        let c = palette.colors[usize::from(i)];
        let a = if transparent_zero && i == 0 { 0 } else { 255 };
        out.extend_from_slice(&[c.r, c.g, c.b, a]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ramp_and_row() {
        let ramp = shade_ramp(Rgb8::new(200, 100, 40), 4, RampOrder::BrightestFirst);
        assert_eq!(ramp[0], Rgb8::new(200, 100, 40));
        assert_eq!(ramp[3], Rgb8::new(50, 25, 10));
        let rev = shade_ramp(Rgb8::new(200, 100, 40), 4, RampOrder::DarkestFirst);
        assert_eq!(rev[0], ramp[3]);
        let base = Palette::default();
        let row = remap_row(&base, RemapRange { start: 254, len: 4 }, &ramp);
        assert_eq!(row.colors[254], ramp[0]);
        assert_eq!(row.colors[255], ramp[1]);
        assert_eq!(row.colors[253], Rgb8::default());
        let off = remap_row(&base, RemapRange { start: 3, len: 0 }, &ramp);
        assert_eq!(off, base);
    }

    #[test]
    fn resolve_respects_transparency() {
        let mut pal = Palette::default();
        pal.colors[1] = Rgb8::new(1, 2, 3);
        let img = IndexedImage::new(2, 1, vec![0, 1]).unwrap();
        assert_eq!(resolve_rgba(&img, &pal, true), [0, 0, 0, 0, 1, 2, 3, 255]);
        assert_eq!(resolve_rgba(&img, &pal, false)[3], 255);
    }
}
