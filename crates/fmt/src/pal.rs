//! PAL palettes: 256 colours of three 6-bit components (VGA DAC layout),
//! 768 bytes, red, green, blue per entry.
//!
//! Expanding the 6-bit components to 8-bit display values is a rendering
//! fidelity question (spec request Q-0003), so the decoded form keeps the
//! raw components and [`Palette6::to_palette`] takes the expansion rule as
//! an argument.

use crate::bytes::Reader;
use crate::error::{Error, Result};
use crate::fixture::FixtureWriter;
use crate::image::{PALETTE_LEN, Palette, Rgb8};

/// Size of a PAL file.
pub const PAL_LEN: usize = PALETTE_LEN * 3;

/// A palette with raw 6-bit components (each 0..=63).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Palette6 {
    /// Components per entry, red, green, blue.
    pub colors: [[u8; 3]; PALETTE_LEN],
}

/// How 6-bit components become 8-bit values. Provisional pending Q-0003.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Expansion {
    /// `c << 2`: 63 becomes 252.
    #[default]
    Shift,
    /// `(c << 2) | (c >> 4)`: 63 becomes 255, 0 stays 0.
    Replicate,
}

impl Expansion {
    /// Expands one component.
    pub fn expand(self, c: u8) -> u8 {
        let c = c & 0x3F;
        match self {
            Self::Shift => c << 2,
            Self::Replicate => (c << 2) | (c >> 4),
        }
    }
}

impl Palette6 {
    /// Parses a PAL file. Files longer than 768 bytes are rejected, and so
    /// are components above 63.
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() > PAL_LEN {
            return Err(Error::Invalid("PAL file longer than 768 bytes"));
        }
        let mut r = Reader::new(data);
        let mut colors = [[0u8; 3]; PALETTE_LEN];
        for c in &mut colors {
            *c = r.array()?;
            if c.iter().any(|&v| v > 63) {
                return Err(Error::Invalid("PAL component above 63"));
            }
        }
        Ok(Self { colors })
    }

    /// Serialises the palette.
    pub fn write(&self) -> Vec<u8> {
        self.colors.iter().flatten().map(|&c| c & 0x3F).collect()
    }

    /// The palette with 8-bit components.
    pub fn to_palette(&self, expansion: Expansion) -> Palette {
        let mut pal = Palette::default();
        for (dst, src) in pal.colors.iter_mut().zip(self.colors.iter()) {
            let [r, g, b] = src.map(|c| expansion.expand(c));
            *dst = Rgb8::new(r, g, b);
        }
        pal
    }

    /// Reduces an 8-bit palette to 6-bit components (drops the low bits),
    /// for writing fixtures.
    pub fn from_palette(pal: &Palette) -> Self {
        let mut colors = [[0u8; 3]; PALETTE_LEN];
        for (dst, c) in colors.iter_mut().zip(pal.colors.iter()) {
            *dst = [c.r >> 2, c.g >> 2, c.b >> 2];
        }
        Self { colors }
    }
}

/// Synthetic PAL fixture: [`crate::synth::ramp_palette`] reduced to 6 bits.
struct RampPal {
    tint: usize,
}

impl FixtureWriter for RampPal {
    fn format(&self) -> &'static str {
        "pal"
    }
    fn name(&self) -> String {
        format!("ramps-tint{}", self.tint)
    }
    fn extension(&self) -> &'static str {
        "pal"
    }
    fn write(&self, out: &mut Vec<u8>) {
        out.extend(Palette6::from_palette(&crate::synth::ramp_palette(self.tint)).write());
    }
}

/// Fixture writers for this module.
pub fn fixture_writers() -> Vec<Box<dyn FixtureWriter>> {
    vec![Box::new(RampPal { tint: 0 }), Box::new(RampPal { tint: 7 })]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_expansion() {
        let pal = Palette6::from_palette(&crate::synth::ramp_palette(3));
        let bytes = pal.write();
        assert_eq!(bytes.len(), PAL_LEN);
        assert_eq!(Palette6::parse(&bytes), Ok(pal));
        assert_eq!(Expansion::Shift.expand(63), 252);
        assert_eq!(Expansion::Replicate.expand(63), 255);
        assert_eq!(Expansion::Replicate.expand(0), 0);
        assert_eq!(Expansion::Replicate.expand(32), 130);
    }

    #[test]
    fn rejects_bad_files() {
        assert!(matches!(
            Palette6::parse(&[0; 10]),
            Err(Error::Truncated { .. })
        ));
        assert!(Palette6::parse(&[0; PAL_LEN + 1]).is_err());
        let mut bytes = vec![0; PAL_LEN];
        bytes[5] = 64;
        assert!(Palette6::parse(&bytes).is_err());
    }
}
