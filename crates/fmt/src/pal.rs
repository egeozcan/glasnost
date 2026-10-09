//! PAL palettes: 256 colours of three components (VGA DAC layout), 768
//! bytes, red, green, blue per entry.
//!
//! Components are meant to be 6-bit values (0 to 63), but the file stores
//! whole bytes and the parser keeps them as stored. How components become
//! displayed colours is game behaviour (S-0103 claim 1) and lives in
//! `glasnost_render::palette`.

use crate::bytes::Reader;
use crate::error::{Error, Result};
use crate::fixture::FixtureWriter;
use crate::image::{PALETTE_LEN, Palette};

/// Size of a PAL file.
pub const PAL_LEN: usize = PALETTE_LEN * 3;

/// A palette file's raw components, as stored (each meant to be 0..=63).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Palette6 {
    /// Components per entry, red, green, blue.
    pub colors: [[u8; 3]; PALETTE_LEN],
}

impl Palette6 {
    /// Parses a PAL file: exactly 768 bytes. Components above 63 are kept.
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() > PAL_LEN {
            return Err(Error::Invalid("PAL file longer than 768 bytes"));
        }
        let mut r = Reader::new(data);
        let mut colors = [[0u8; 3]; PALETTE_LEN];
        for c in &mut colors {
            *c = r.array()?;
        }
        Ok(Self { colors })
    }

    /// Whether every component is a 6-bit value.
    pub fn is_six_bit(&self) -> bool {
        self.colors.iter().flatten().all(|&c| c <= 63)
    }

    /// Serialises the palette.
    pub fn write(&self) -> Vec<u8> {
        self.colors.iter().flatten().copied().collect()
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
    fn round_trip() {
        let pal = Palette6::from_palette(&crate::synth::ramp_palette(3));
        let bytes = pal.write();
        assert_eq!(bytes.len(), PAL_LEN);
        assert!(pal.is_six_bit());
        assert_eq!(Palette6::parse(&bytes), Ok(pal));
    }

    #[test]
    fn rejects_bad_lengths() {
        assert!(matches!(
            Palette6::parse(&[0; 10]),
            Err(Error::Truncated { .. })
        ));
        assert!(Palette6::parse(&[0; PAL_LEN + 1]).is_err());
        // Components above 63 are kept as stored.
        let mut bytes = vec![0; PAL_LEN];
        bytes[5] = 64;
        let p = Palette6::parse(&bytes).unwrap();
        assert_eq!(p.colors[1][2], 64);
        assert!(!p.is_six_bit());
    }
}
