//! The protection appearance of curtain-protected and force-shielded
//! objects (S-0513 claim 29 and F9; S-0102 claim 34 and F9).
//!
//! The simulation keeps the brightness envelope's phases and takes its draw
//! (`glasnost_sim::superweapons::protect`); this module turns them into a
//! drawing's brightness and colour mask. The mask is the imported
//! colour-adjustment triple a selector names, packed once in our 5-6-5
//! layout; which drawing takes which mask is S-0102 claim 34's table.

use glasnost_convert::Import;
use glasnost_convert::sim::{PROTECTION_SELECTORS, protection_selector};
use glasnost_render::compose::{PixelLayout, protection_mask};
use glasnost_sim::superweapons::{ProtectKind, protect};
use glasnost_sim::world::Object;

/// What protection does to an object's drawing in the shown tick: its kind
/// and the envelope's phase factor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Look {
    /// The protection kind in force.
    pub kind: ProtectKind,
    /// The phase factor *m*; `None` before the first protected turn and in
    /// Ordinary.
    pub factor: Option<i32>,
    /// The object's centre cell is shrouded for the local player: its
    /// drawings keep the envelope and take no mask (S-0513 claim 29b).
    pub shrouded: bool,
}

impl Look {
    /// The look of `o` in tick `t`, the tick whose updates the picture
    /// shows; `None` when it is not protected then.
    pub fn of(o: &Object, t: i32) -> Option<Self> {
        let p = o.sw.protection.as_ref()?;
        // spec: S-0513 claim 29 and F7 (expiry immediately removes the appearance)
        let (kind, factor) = protect::appearance(p, t)?;
        Some(Self {
            kind,
            factor,
            shrouded: false,
        })
    }

    /// The same look with the centre-cell shroud test's answer `shrouded`
    /// (S-0513 claim 29b).
    pub fn shrouded(self, shrouded: bool) -> Self {
        Self { shrouded, ..self }
    }

    /// The protection-adjusted brightness of ordinary brightness `b` ‰
    /// (S-0513 F9), which then selects the shade row (S-0104 claim 15).
    pub fn brightness(&self, b: i32) -> i32 {
        protect::brightness(b, self.factor)
    }
}

/// The adjusted brightness of `b` ‰ under an optional look.
pub fn brightness(look: Option<Look>, b: i32) -> i32 {
    look.map_or(b, |l| l.brightness(b))
}

/// The two protection colour masks, packed for our 5-6-5 frames (S-0102
/// F9).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Masks {
    /// The curtain selector's mask.
    pub curtain: u16,
    /// The force-shield selector's mask.
    pub shield: u16,
}

/// The colour-adjustment triple a selector names, or `None` when it names
/// no position of the table (S-0513 claim 29a: a selector below 0 or above
/// 15 is used as held, not reduced modulo 16 or clamped).
pub fn selected(import: &Import, selector: i32) -> Option<[u8; 3]> {
    usize::try_from(selector)
        .ok()
        .and_then(|i| import.outcome.colour_add.get(i))
        .map(|e| e.0)
}

/// The mask *K* of an optional selected triple in `layout` (S-0102 F9):
/// with no selected adjustment *K* = 0.
pub fn mask_of(triple: Option<[u8; 3]>, layout: PixelLayout) -> u16 {
    // spec: S-0102 F9, S-0513 claim 29a (no selected adjustment, or a selector below 0 or above 15: K = 0, on 5-6-5 and 5-5-5 alike)
    triple.map_or(0, |rgb| protection_mask(rgb, layout))
}

impl Masks {
    /// The masks the import's selectors name. A selector outside the table
    /// gives *K* = 0; the importer warns about it (S-0513 claim 29a).
    pub fn read(import: &Import) -> Self {
        let mask = |key: &str| {
            // spec: S-0513 Import mapping (IronCurtainColor and ForceShieldColor: zero-based
            // integer positions held after every layer; absent: 0)
            let selector = protection_selector(&import.outcome, key).map_or(0, |(v, _, _)| v);
            // spec: S-0102 claim 34, F9 (the selected RGB byte triple, packed once in the surface's layout; our frames are 5-6-5)
            mask_of(selected(import, selector), PixelLayout::Rgb565)
        };
        let [curtain, shield] = PROTECTION_SELECTORS.map(mask);
        Self { curtain, shield }
    }

    /// The mask of a vehicle or aircraft drawing (S-0102 claim 34): only a
    /// curtain-protected vehicle drawn as voxels, its body and voxel
    /// turret and barrel, takes the curtain selector's mask; a vehicle
    /// drawn as a sprite and an aircraft's voxel body take none.
    pub fn mobile(&self, look: Option<Look>, vehicle: bool, voxel: bool) -> u16 {
        match look {
            // spec: S-0102 claim 34 (curtain-protected vehicle drawn as voxels: curtain selector)
            Some(Look {
                kind: ProtectKind::Curtain,
                shrouded: false,
                ..
            }) if vehicle && voxel => self.curtain,
            // spec: S-0102 claim 34 (curtain sprite vehicle and aircraft voxel body: no mask);
            // S-0513 claim 29b (centre cell shrouded for the local player: no mask)
            _ => 0,
        }
    }

    /// The mask of a building's body sprite or its voxel turret or barrel
    /// (S-0102 claim 34): the force-shield selector's mask while it is
    /// force-shielded, none under the curtain.
    pub fn building(&self, look: Option<Look>) -> u16 {
        match look {
            // spec: S-0102 claim 34 (force-shielded building body and voxel turret/barrel: force-shield selector)
            Some(Look {
                kind: ProtectKind::ForceShield,
                shrouded: false,
                ..
            }) => self.shield,
            // spec: S-0102 claim 34 (curtain-protected building body and voxel turret/barrel: no mask);
            // S-0513 claim 29b (centre cell shrouded for the local player: no mask)
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glasnost_sim::superweapons::Tint;

    fn look(kind: ProtectKind) -> Option<Look> {
        Some(Look {
            kind,
            factor: None,
            shrouded: false,
        })
    }

    /// spec: S-0102 claim 34 (the table: the curtain mask only on a curtain-protected vehicle
    /// drawn as voxels; none on a sprite vehicle or an aircraft voxel body; a building body or
    /// voxel turret takes the force-shield mask under the force shield and none under the
    /// curtain; no protection, no mask)
    #[test]
    fn claim34_table_rows_select_their_mask() {
        let m = Masks {
            curtain: 0x1234,
            shield: 0x0841,
        };
        let (c, f) = (look(ProtectKind::Curtain), look(ProtectKind::ForceShield));
        assert_eq!(m.mobile(c, true, true), 0x1234, "voxel vehicle");
        assert_eq!(m.mobile(c, true, false), 0, "sprite vehicle");
        assert_eq!(m.mobile(c, false, true), 0, "aircraft voxel body");
        assert_eq!(m.mobile(None, true, true), 0);
        assert_eq!(m.building(f), 0x0841, "force-shielded building");
        assert_eq!(m.building(c), 0, "curtain-protected building");
        assert_eq!(m.building(None), 0);
    }

    /// spec: S-0513 claim 29a (a selector below 0 or above 15 names no table position: K = 0 on
    /// 5-6-5 and 5-5-5 surfaces alike; not reduced modulo 16, so 16 is not position 0 and −1 not
    /// position 15; a position whose triple is still the empty (0, 0, 0) also gives K = 0), S-0102
    /// F9
    #[test]
    fn claim29a_selector_outside_the_table() {
        use glasnost_convert::{ImportOptions, convert, inputs::InputFile, inputs::Inputs};
        let rules = |audio: &str| {
            let text = format!(
                "[ColorAdd]\nA=9,9,9\nB=7,19,3\nC=1,0,1\nD=0,0,0\nE=5,5,5\nF=6,6,6\nG=7,7,7\nH=8,8,8\n\
                 I=9,9,9\nJ=10,10,10\nK=11,11,11\nL=12,12,12\nM=13,13,13\nN=14,14,14\nO=15,15,15\nP=16,16,16\n\
                 [AudioVisual]\n{audio}"
            );
            convert(
                &Inputs {
                    base: Some(InputFile {
                        name: "rules.ini".into(),
                        origin: "test".into(),
                        bytes: text.into_bytes(),
                    }),
                    ..Default::default()
                },
                &ImportOptions::default(),
            )
        };
        for layout in [PixelLayout::Rgb565, PixelLayout::Rgb555] {
            let import = rules("");
            for bad in [-1, 16, 99, i32::MIN] {
                assert_eq!(selected(&import, bad), None, "selector {bad}");
                assert_eq!(mask_of(selected(&import, bad), layout), 0, "{layout:?}");
            }
            assert_eq!(selected(&import, 0), Some([9, 9, 9]));
            assert_eq!(selected(&import, 15), Some([16, 16, 16]));
            // Position 3 holds the empty triple: K = 0 too.
            assert_eq!(mask_of(selected(&import, 3), layout), 0);
            assert_ne!(mask_of(selected(&import, 1), layout), 0);
        }
        // The masks of out-of-range selectors are 0, and the other kind is unaffected.
        let import = rules("IronCurtainColor=16\nForceShieldColor=2\n");
        let m = Masks::read(&import);
        assert_eq!(m.curtain, 0);
        assert_eq!(m.shield, protection_mask([1, 0, 1], PixelLayout::Rgb565));
        let import = rules("IronCurtainColor=1\nForceShieldColor=-1\n");
        let m = Masks::read(&import);
        assert_eq!(m.curtain, protection_mask([7, 19, 3], PixelLayout::Rgb565));
        assert_eq!(m.shield, 0);
        assert_eq!(
            import
                .warnings
                .iter()
                .filter(|w| w.message.contains("ForceShieldColor=-1"))
                .count(),
            1,
            "the importer warns once"
        );
    }

    /// spec: S-0513 claim 29 and F9 (every row of S-0102 claim 34's table takes the envelope's
    /// brightness; unchanged before the first turn and in Ordinary); F7 (expiry removes it)
    #[test]
    fn claim29_look_follows_the_shown_tick() {
        use glasnost_sim::superweapons::Protection;
        let p = Protection {
            start: 100,
            duration: 50,
            kind: ProtectKind::Curtain,
            tint: Tint::Rise(100),
        };
        let at = |t| {
            protect::appearance(&p, t).map(|(kind, factor)| Look {
                kind,
                factor,
                shrouded: false,
            })
        };
        assert_eq!(
            brightness(at(100), 1000),
            1000,
            "factor 256 at the rise's entry"
        );
        assert_eq!(
            brightness(at(106), 1000),
            2000,
            "factor 512 when the rise ends"
        );
        assert_eq!(brightness(at(150), 1000), 1000, "expired: no appearance");
        assert_eq!(at(150), None);
        let ordinary = Look {
            kind: ProtectKind::ForceShield,
            factor: None,
            shrouded: false,
        };
        assert_eq!(ordinary.brightness(2400), 2400, "not newly capped");
    }
}
