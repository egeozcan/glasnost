//! The protection appearance of curtain-protected and force-shielded
//! objects (S-0513 claim 29 and F9; S-0102 claim 34 and F9).
//!
//! The simulation keeps the brightness envelope's phases and takes its draw
//! (`glasnost_sim::superweapons::protect`); this module turns them into a
//! drawing's brightness and colour mask. The mask is the imported
//! colour-adjustment triple a selector names, packed once in our 5-6-5
//! layout; which drawing takes which mask is S-0102 claim 34's table. The
//! same table gives the laser-target mask of an air-strike-marked building
//! (S-0519 claim 26) and the berserk mask (S-0610 claim 7), and a
//! building's brightness combines its charge flash, protection and mark by
//! S-0519 F7 ([`BuildingLook`]).

use glasnost_convert::Import;
use glasnost_convert::sim::{
    BERSERK_SELECTOR, LASER_TARGET_SELECTOR, PROTECTION_SELECTORS, protection_selector,
};
use glasnost_render::compose::{PixelLayout, protection_mask};
use glasnost_sim::mark::Adjust;
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

/// A building's look in the shown tick: its protection look and what
/// S-0519 F7 combines with it, the charge flash count and the air-strike
/// mark.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BuildingLook {
    /// Its protection look (S-0513 claim 29), shroud test included.
    pub protection: Option<Look>,
    /// Its flash count and mark in the shown tick (S-0519 claims 25 and
    /// 26); the protection factor in it is replaced by `protection`'s.
    pub adjust: Adjust,
}

impl BuildingLook {
    /// The brightness *b*₃ of ordinary brightness `b` ‰ (S-0519 F7),
    /// which then selects the shade row (S-0104 claim 15).
    pub fn brightness(&self, b: i32) -> i32 {
        // spec: S-0519 F7 (b₁ the flashed value of F6, b₂ = P(b₁) by S-0513 F9, then the mark factor)
        Adjust {
            protection: self.protection.map(|l| l.factor),
            ..self.adjust
        }
        .brightness(b)
    }

    /// Whether the building is marked by an air strike (S-0519 claim 26).
    pub fn marked(&self) -> bool {
        self.adjust.mark.is_some()
    }

    /// Whether anything of this look changes the building's drawing: it
    /// is protected, marked, or has flash ticks to come.
    pub fn active(&self) -> bool {
        self.protection.is_some() || self.marked() || self.adjust.flash > 0
    }
}

/// What an opaque building slot animation drawn over one cell takes its
/// colour mask from (S-0513 claim 29b, slot animations; S-0102 claim 34,
/// animation rule): the look of the building occupying that cell, the cell
/// that holds the animation's own position, and whether that cell is
/// shrouded for the local player.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SlotCell {
    /// The occupying building's look. Its centre-cell shroud answer plays
    /// no part here (S-0513 claim 29b: the building's centre cell plays no
    /// part for its slot animations).
    pub look: BuildingLook,
    /// The animation's cell is shrouded for the local player by S-0110
    /// claim 8's point test at its centre, at its ground height.
    pub shrouded: bool,
}

/// The two protection colour masks, packed for our 5-6-5 frames (S-0102
/// F9).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Masks {
    /// The curtain selector's mask.
    pub curtain: u16,
    /// The force-shield selector's mask.
    pub shield: u16,
    /// The laser-target selector's mask (S-0519 claim 26).
    pub laser: u16,
    /// The berserk selector's mask (S-0610 claim 7).
    pub berserk: u16,
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
    // spec: S-0102 F9, S-0513 claim 29a, S-0519 claim 26 (no selected adjustment, or a protection or laser-target selector below 0 or above 15: K = 0, on 5-6-5 and 5-5-5 alike)
    triple.map_or(0, |rgb| protection_mask(rgb, layout))
}

impl Masks {
    /// The masks the import's selectors name. A selector outside the table
    /// gives *K* = 0; the importer warns about it (S-0513 claim 29a, S-0519
    /// claim 26, S-0610 Edge cases).
    pub fn read(import: &Import) -> Self {
        let mask = |key: &str| {
            // spec: S-0513 Import mapping (IronCurtainColor and ForceShieldColor: zero-based
            // integer positions held after every layer; absent: 0); S-0519 Import mapping
            // (LaserTargetColor, a position in the table, default 0) and S-0610 Import mapping
            // (BerserkColor, zero-based, default 0; absent in a later layer retains it)
            let selector = protection_selector(&import.outcome, key).map_or(0, |(v, _, _)| v);
            // spec: S-0102 claim 34, F9 (the selected RGB byte triple, packed once in the surface's layout; our frames are 5-6-5)
            mask_of(selected(import, selector), PixelLayout::Rgb565)
        };
        let [curtain, shield] = PROTECTION_SELECTORS.map(mask);
        // spec: S-0519 claim 26 (the laser-target colour is read from the rules at drawing time; a held selector below 0 or above 15 after every layer: no selected adjustment, K = 0, a departure)
        let laser = mask(LASER_TARGET_SELECTOR);
        // spec: S-0610 claim 7 (the berserk selector's triple becomes the berserk mask by S-0102 F9); Edge cases (outside 0 to 15: no mask)
        let berserk = mask(BERSERK_SELECTOR);
        Self {
            curtain,
            shield,
            laser,
            berserk,
        }
    }

    /// The berserk mask of a mobile drawing (S-0610 claim 7): a voxel
    /// vehicle's body and voxel turret and barrel, and an infantry sprite
    /// body unless its cell's centre is shrouded for the viewer
    /// (`shrouded`); none for a sprite vehicle and an aircraft.
    pub fn berserk_of(&self, berserk: bool, kind: MobileKind, shrouded: bool) -> u16 {
        match kind {
            // spec: S-0610 claim 7 (vehicle drawn as voxels: body and voxel turret/barrel)
            MobileKind::VoxelVehicle if berserk => self.berserk,
            // spec: S-0610 claim 7 (infantry sprite body: yes, unless the centre of its cell is shrouded for the viewing player)
            MobileKind::Infantry if berserk && !shrouded => self.berserk,
            // spec: S-0610 claim 7 (sprite vehicle, including the voxel part on a sprite body, which S-0105 claim 12 limits to the barrel, and aircraft: none)
            _ => 0,
        }
    }

    /// The colour mask of a building's body sprite, its voxel turret or
    /// barrel, or an opaque slot animation over its cells, under `look`
    /// (S-0102 claim 34 and its animation rule): the force-shield mask as
    /// [`Masks::building`] gives it, united with the laser-target mask while
    /// the building is marked (S-0519 claim 26).
    pub fn building_look(&self, look: BuildingLook) -> u16 {
        // spec: S-0519 claim 26 (marked: the laser-target mask, by bitwise union with the force-shield mask when both apply; curtain protection and the charge flash add none)
        let laser = if look.marked() { self.laser } else { 0 };
        self.building(look.protection) | laser
    }

    /// The colour mask of an opaque building slot animation over the cell
    /// `cell` describes (S-0513 claim 29b, slot animations), `None` when no
    /// protected, marked or flashing building occupies that cell.
    pub fn slot_animation(&self, cell: Option<SlotCell>) -> u16 {
        match cell {
            // spec: S-0513 claim 29b (slot animations: with no building in the animation's cell, no mask)
            None => 0,
            // spec: S-0513 claim 29b and S-0102 claim 34 (r34: the cell holding the animation's own position shrouded for the local player: no colour mask at all, neither the force-shield nor the laser-target mask; brightness unchanged); S-0519 claim 26
            Some(SlotCell { shrouded: true, .. }) => 0,
            // spec: S-0513 claim 29b (the animation's cell not shrouded: S-0102 claim 34's masks of the occupying building unchanged, the force-shield mask, the laser-target mask and their union; the building's centre cell plays no part)
            Some(SlotCell { look, .. }) => self.building_look(BuildingLook {
                protection: look.protection.map(|l| l.shrouded(false)),
                ..look
            }),
        }
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

/// What a mobile drawing is, for the berserk mask (S-0610 claim 7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MobileKind {
    /// A vehicle drawn as voxels.
    VoxelVehicle,
    /// A vehicle drawn as a sprite.
    SpriteVehicle,
    /// An infantry unit.
    Infantry,
    /// An aircraft.
    Aircraft,
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
            ..Masks::default()
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

    /// spec: S-0519 claim 26 (selector outside the table, T27 part e's engine fixture: a held
    /// laser-target selector of 16 or −1 gives K = 0 on 5-6-5 and 5-5-5 alike, so a marked
    /// building's body, voxel parts and building animations take no laser-target colour; a
    /// force-shielded marked building keeps the force-shield mask unchanged; one importer warning
    /// per run naming the key and its value; a position from 0 to 15 behaves as usual, one holding
    /// (0, 0, 0) also giving K = 0), Edge cases, S-0102 F9
    #[test]
    fn claim26_laser_selector_outside_the_table() {
        use glasnost_convert::{ImportOptions, convert, inputs::InputFile, inputs::Inputs};
        // Positions 0 and 15 hold distinct non-zero triples, so a wrap or a clamp would show.
        let rules = |audio: &str| {
            let text = format!(
                "[ColorAdd]\nA=9,9,9\nB=7,19,3\nC=1,0,1\nD=0,0,0\nE=5,5,5\nF=6,6,6\nG=7,7,7\nH=8,8,8\n\
                 I=9,9,9\nJ=10,10,10\nK=11,11,11\nL=12,12,12\nM=13,13,13\nN=14,14,14\nO=15,15,15\nP=16,16,16\n\
                 [AudioVisual]\nForceShieldColor=2\n{audio}"
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
        let shield = protection_mask([1, 0, 1], PixelLayout::Rgb565);
        let marked = BuildingLook {
            protection: None,
            adjust: Adjust {
                mark: Some(Some(512)),
                ..Adjust::default()
            },
        };
        let shielded = BuildingLook {
            protection: look(ProtectKind::ForceShield),
            ..marked
        };
        let valid = Masks::read(&rules("LaserTargetColor=1\n"));
        assert_eq!(
            valid.laser,
            protection_mask([7, 19, 3], PixelLayout::Rgb565)
        );
        assert_eq!(valid.building_look(shielded), valid.laser | shield);
        for bad in [16, -1] {
            let import = rules(&format!("LaserTargetColor={bad}\n"));
            let m = Masks::read(&import);
            assert_eq!(m.laser, 0, "{bad}: K = 0");
            for layout in [PixelLayout::Rgb565, PixelLayout::Rgb555] {
                assert_eq!(
                    mask_of(selected(&import, bad), layout),
                    0,
                    "{bad} {layout:?}"
                );
            }
            assert_eq!(m.building_look(marked), 0, "{bad}: marked body, no colour");
            let slot = SlotCell {
                look: marked,
                shrouded: false,
            };
            assert_eq!(
                m.slot_animation(Some(slot)),
                0,
                "{bad}: animation, no colour"
            );
            assert_eq!(
                m.building_look(shielded),
                shield,
                "{bad}: the force-shield mask kept"
            );
            // The brightness is the mark's own, whatever the selector.
            assert_eq!(marked.brightness(1000), 2000);
            let key = format!("LaserTargetColor={bad}");
            assert_eq!(
                import
                    .warnings
                    .iter()
                    .filter(|w| w.message.contains(&key))
                    .count(),
                1,
                "{key}: one warning"
            );
        }
        // Position 3 holds the empty triple: K = 0 as well, with no warning.
        let import = rules("LaserTargetColor=3\n");
        assert_eq!(Masks::read(&import).laser, 0);
        assert!(
            !import
                .warnings
                .iter()
                .any(|w| w.message.contains("LaserTargetColor"))
        );
    }

    /// spec: S-0102 claim 34 (r34, the shrouded-cell exception for slot animations) and S-0513 claim 29b (slot animations: the animation's own cell decides; not shrouded,
    /// S-0102 claim 34's masks of the occupying building, the force-shield mask, the laser-target
    /// mask and their union, whatever the state of that building's centre cell; shrouded, no colour
    /// mask at all; no building in the cell, no mask), S-0519 claim 26
    #[test]
    fn claim29b_slot_animation_masks_follow_their_own_cell() {
        let m = Masks {
            shield: 0x0841,
            laser: 0x1234,
            ..Masks::default()
        };
        let mark = Adjust {
            mark: Some(Some(256)),
            ..Adjust::default()
        };
        let cell = |protection: Option<Look>, adjust, shrouded| {
            Some(SlotCell {
                look: BuildingLook { protection, adjust },
                shrouded,
            })
        };
        let shield = look(ProtectKind::ForceShield);
        let centre_shrouded = shield.map(|l| l.shrouded(true));
        for centre in [shield, centre_shrouded] {
            // i1: the animation's cell visible, the centre cell either way.
            assert_eq!(
                m.slot_animation(cell(centre, Adjust::default(), false)),
                0x0841
            );
            // i5: marked too, the union.
            assert_eq!(m.slot_animation(cell(centre, mark, false)), 0x0841 | 0x1234);
            // i2 and i5: the animation's cell shrouded, nothing.
            assert_eq!(m.slot_animation(cell(centre, Adjust::default(), true)), 0);
            assert_eq!(m.slot_animation(cell(centre, mark, true)), 0);
        }
        // Only marked: the laser-target mask, removed under shroud.
        assert_eq!(m.slot_animation(cell(None, mark, false)), 0x1234);
        assert_eq!(m.slot_animation(cell(None, mark, true)), 0);
        // The curtain supplies none.
        let curtain = look(ProtectKind::Curtain);
        assert_eq!(m.slot_animation(cell(curtain, Adjust::default(), false)), 0);
        assert_eq!(m.slot_animation(None), 0);
        // The body keeps the centre-cell rule: shrouded centre, no force-shield mask.
        assert_eq!(
            m.building_look(BuildingLook {
                protection: centre_shrouded,
                adjust: Adjust::default()
            }),
            0
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

    /// spec: S-0519 claim 26 and Import mapping (LaserTargetColor: a table position, default 0),
    /// S-0610 claim 7 and Import mapping (BerserkColor: zero-based, default 0), Edge cases (a
    /// berserk selector outside 0 to 15: the importer warns and no mask), S-0102 F9 (no selected
    /// adjustment: K = 0), S-0519 claim 26 (a laser-target selector outside the table: no mask,
    /// the importer warns)
    #[test]
    fn laser_and_berserk_selectors() {
        use glasnost_convert::{ImportOptions, convert, inputs::InputFile, inputs::Inputs};
        let import = |audio: &str| {
            let text = format!("[ColorAdd]\nA=9,9,9\nB=7,19,3\nC=1,0,1\n[AudioVisual]\n{audio}");
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
        let k = |rgb| protection_mask(rgb, PixelLayout::Rgb565);
        let m = Masks::read(&import(""));
        assert_eq!(
            (m.laser, m.berserk),
            (k([9, 9, 9]), k([9, 9, 9])),
            "absent: 0"
        );
        let m = Masks::read(&import("LaserTargetColor=1\nBerserkColor=2\n"));
        assert_eq!((m.laser, m.berserk), (k([7, 19, 3]), k([1, 0, 1])));
        let bad = import("LaserTargetColor=16\nBerserkColor=-1\n");
        let m = Masks::read(&bad);
        assert_eq!((m.laser, m.berserk), (0, 0));
        for key in ["LaserTargetColor=16", "BerserkColor=-1"] {
            assert_eq!(
                bad.warnings
                    .iter()
                    .filter(|w| w.message.contains(key))
                    .count(),
                1,
                "{key}"
            );
        }
    }
}
