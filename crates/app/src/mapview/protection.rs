//! The protection appearance of curtain-protected and force-shielded
//! objects (S-0513 claim 29 and F9; S-0102 claim 34 and F9).
//!
//! The simulation keeps the brightness envelope's phases and takes its draw
//! (`glasnost_sim::superweapons::protect`); this module turns them into a
//! drawing's brightness and colour mask. The mask is the imported
//! colour-adjustment triple a selector names, packed once in our 5-6-5
//! layout; which drawing takes which mask is S-0102 claim 34's table.

use glasnost_convert::Import;
use glasnost_ini::value;
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
}

impl Look {
    /// The look of `o` in tick `t`, the tick whose updates the picture
    /// shows; `None` when it is not protected then.
    pub fn of(o: &Object, t: i32) -> Option<Self> {
        let p = o.sw.protection.as_ref()?;
        // spec: S-0513 claim 29 and F7 (expiry immediately removes the appearance)
        let (kind, factor) = protect::appearance(p, t)?;
        Some(Self { kind, factor })
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

/// The note for a selector outside the colour-adjustment table (Q-1527).
pub fn selector_note(key: &str, selector: i64) -> String {
    format!(
        "[AudioVisual] {key}={selector} names no position of the 16-entry colour-adjustment table: its protection drawings take no colour mask (S-0102 F9, K = 0; interim rule, Q-1527)"
    )
}

impl Masks {
    /// The masks the import's selectors name, with a note for each selector
    /// outside the table.
    pub fn read(import: &Import) -> (Self, Vec<String>) {
        let mut notes = Vec::new();
        let mut mask = |key: &str| {
            // spec: S-0102 Import mapping, S-0513 Import mapping (IronCurtainColor and
            // ForceShieldColor: zero-based integer positions; absent: 0)
            let selector = import
                .outcome
                .globals
                .get(&b"AudioVisual"[..])
                .and_then(|s| s.get(key.as_bytes()))
                .and_then(|k| k.value.as_deref())
                .and_then(value::integer)
                .map_or(0, i64::from);
            let entry = usize::try_from(selector)
                .ok()
                .and_then(|i| import.outcome.colour_add.get(i));
            match entry {
                // spec: S-0102 claim 34, F9 (the selected RGB byte triple, packed once in the surface's layout; our frames are 5-6-5)
                Some(e) => protection_mask(e.0, PixelLayout::Rgb565),
                None => {
                    // Q-1527 interim rule: a selector outside the table gives K = 0.
                    notes.push(selector_note(key, selector));
                    0
                }
            }
        };
        let curtain = mask("IronCurtainColor");
        let shield = mask("ForceShieldColor");
        (Self { curtain, shield }, notes)
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
                ..
            }) if vehicle && voxel => self.curtain,
            // spec: S-0102 claim 34 (curtain sprite vehicle and aircraft voxel body: no mask)
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
                ..
            }) => self.shield,
            // spec: S-0102 claim 34 (curtain-protected building body and voxel turret/barrel: no mask)
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glasnost_sim::superweapons::Tint;

    fn look(kind: ProtectKind) -> Option<Look> {
        Some(Look { kind, factor: None })
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
        let at = |t| protect::appearance(&p, t).map(|(kind, factor)| Look { kind, factor });
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
        };
        assert_eq!(ordinary.brightness(2400), 2400, "not newly capped");
    }
}
