//! HVA animation files: one 3 × 4 transform per limb per frame for a VXL
//! model.
//!
//! Layout: 16-byte name, `u32 frame count`, `u32 limb count`, 16-byte limb
//! names, then `frames × limbs` row-major 3 × 4 `f32` matrices, frame by
//! frame. From XCC's `cc_structures.h` and OpenRA's `HvaReader.cs`
//! (GPL-3.0-or-later; XCC Copyright (C) 2000 Olaf van der Spek, OpenRA
//! Copyright (c) The OpenRA Developers and Contributors). Floats are kept
//! as raw bits.

use crate::bytes::{F32Bits, PutLe, Reader, len_u32};
use crate::error::{Error, Result};
use crate::fixture::FixtureWriter;

/// One 3 × 4 transform, row-major.
pub type Matrix = [[F32Bits; 4]; 3];

/// A parsed HVA file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hva {
    /// File name field (16 bytes as stored).
    pub name: [u8; 16],
    /// Limb names (16 bytes each as stored).
    pub limb_names: Vec<[u8; 16]>,
    /// `frames × limbs` matrices, frame by frame.
    pub matrices: Vec<Matrix>,
}

impl Hva {
    /// Parses an HVA file.
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        let name = r.array()?;
        let frames = r.u32()? as usize;
        let limbs = r.u32()? as usize;
        let need = limbs
            .checked_mul(16)
            .and_then(|n| frames.checked_mul(limbs)?.checked_mul(48)?.checked_add(n))
            .ok_or(Error::Invalid("HVA counts overflow"))?;
        if r.remaining() < need {
            return Err(Error::Truncated {
                needed: (24 + need) as u64,
            });
        }
        let mut limb_names = Vec::with_capacity(limbs);
        for _ in 0..limbs {
            limb_names.push(r.array()?);
        }
        let mut matrices = Vec::with_capacity(frames * limbs);
        for _ in 0..frames * limbs {
            let mut m = Matrix::default();
            for row in &mut m {
                for v in row.iter_mut() {
                    *v = r.f32_bits()?;
                }
            }
            matrices.push(m);
        }
        Ok(Self {
            name,
            limb_names,
            matrices,
        })
    }

    /// Number of frames.
    pub fn frames(&self) -> usize {
        self.matrices
            .len()
            .checked_div(self.limb_names.len())
            .unwrap_or(0)
    }

    /// Transform of `limb` in `frame`.
    pub fn matrix(&self, frame: usize, limb: usize) -> Option<&Matrix> {
        if limb >= self.limb_names.len() {
            return None;
        }
        self.matrices
            .get(frame.checked_mul(self.limb_names.len())? + limb)
    }

    /// Serialises the file.
    pub fn write(&self) -> Vec<u8> {
        let mut out = self.name.to_vec();
        out.put_u32(len_u32(self.frames()));
        out.put_u32(len_u32(self.limb_names.len()));
        for n in &self.limb_names {
            out.extend_from_slice(n);
        }
        for v in self.matrices.iter().flatten().flatten() {
            out.put_u32(v.0);
        }
        out
    }
}

/// A synthetic two-limb, four-frame animation (invented translations).
pub fn synthetic_animation() -> Hva {
    let pad = |s: &[u8]| {
        let mut a = [0u8; 16];
        a[..s.len()].copy_from_slice(s);
        a
    };
    let f = F32Bits::new;
    let mut matrices = Vec::new();
    for frame in 0..4u8 {
        for limb in 0..2u8 {
            let t = f32::from(frame) * if limb == 0 { 0.5 } else { -0.25 };
            matrices.push([
                [f(1.0), f(0.0), f(0.0), f(t)],
                [f(0.0), f(1.0), f(0.0), f(0.0)],
                [f(0.0), f(0.0), f(1.0), f(f32::from(limb))],
            ]);
        }
    }
    Hva {
        name: pad(b"box-and-orb"),
        limb_names: vec![pad(b"body"), pad(b"orb")],
        matrices,
    }
}

struct SyntheticHva;

impl FixtureWriter for SyntheticHva {
    fn format(&self) -> &'static str {
        "hva"
    }
    fn name(&self) -> String {
        "box-and-orb".into()
    }
    fn extension(&self) -> &'static str {
        "hva"
    }
    fn write(&self, out: &mut Vec<u8>) {
        out.extend(synthetic_animation().write());
    }
}

/// Fixture writers for this module.
pub fn fixture_writers() -> Vec<Box<dyn FixtureWriter>> {
    vec![Box::new(SyntheticHva)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let h = synthetic_animation();
        let bytes = h.write();
        let back = Hva::parse(&bytes).unwrap();
        assert_eq!(back, h);
        assert_eq!(back.frames(), 4);
        assert_eq!(back.matrix(3, 0).unwrap()[0][3].get(), 1.5);
        assert!(back.matrix(0, 2).is_none());
    }

    #[test]
    fn truncation_and_huge_counts() {
        let bytes = synthetic_animation().write();
        for cut in 0..bytes.len() {
            assert!(Hva::parse(&bytes[..cut]).is_err());
        }
        let mut huge = bytes.clone();
        huge[16..20].copy_from_slice(&u32::MAX.to_le_bytes());
        huge[20..24].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(Hva::parse(&huge).is_err());
    }
}
