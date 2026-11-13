//! Quirk switches of the rendering specs that the map viewer's scene
//! builder and live drawing implement (S-0102 draw order, S-0108 building
//! frames and slot animations), each defaulting to the original's behaviour, with the small
//! rules they switch.
//!
//! The switches of the render crate live with the code they govern
//! (`render::projection::ProjectionQuirks`, `render::compose::ComposeQuirks`,
//! `render::light::LightQuirks`, `render::voxel::VoxelQuirks`,
//! `render::overlay::OverlayQuirks`,
//! `render::palette::PaletteQuirks`); the file lookup's switch that S-0111
//! claim 19a names for the cursor palette lives with S-0005's other
//! switches (`vfs::GameQuirks`). [`NOT_IMPLEMENTED`] lists the named
//! switches of the rendering specs that have no field, with the reason;
//! the test at the end of this file checks that every switch the rendering
//! specs name is one or the other.

use glasnost_num::geom::n_step;
use glasnost_render::projection::TERRAIN_LIFT;

/// Switches of S-0102 and S-0108 implemented in `mapview`.
///
/// The ground layer's full sort in every drawn frame (S-0102 claim 6) and
/// the building body clamp to the last body frame (S-0108 claim 3) are
/// rules without switches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewQuirks {
    /// `quirks.cliff_fudge_far_cliff_weaker` (S-0102 claim 14): a high
    /// cell two steps in front gives cliff factor 1 and overrides a high
    /// cell one step in front, which alone gives 2. Off: factor 2 whenever
    /// either cell is high.
    pub cliff_fudge_far_cliff_weaker: bool,
    /// `quirks.occupiable_hurt_rule` (S-0108 claim 2): for an occupiable
    /// type, "hurt" is badly damaged at tech level 0 or below and damaged
    /// above. Off: always damaged.
    pub occupiable_hurt_rule: bool,
    /// `quirks.sprite_vehicle_eight_facings_only` (S-0102 claim 22): a
    /// vehicle drawn as a sprite turns its image only with a facing count
    /// of exactly 8. Off: with a count that is a power of two from 2 to 256,
    /// the count-step facing of the body facing turned an eighth of a turn
    /// clockwise.
    pub sprite_vehicle_eight_facings_only: bool,
    /// `quirks.roof_unload_no_fallback` (S-0108 claim 22): a jumpjet or
    /// balloon-hover unit unloading from a building without roof-deploying
    /// or under-roof-door sprites keeps the body and draws no under-door
    /// image. Off: the ordinary deploying and under-door sprites stand in.
    pub roof_unload_no_fallback: bool,
    /// `quirks.unsorted_upper_layers` (S-0102 claim 7): the underground,
    /// surface, air and top layers keep their entering order. Off: they
    /// are sorted like the ground layer.
    pub unsorted_upper_layers: bool,
    /// `quirks.layer_change_on_events_only` (S-0102 claim 4): a mobile
    /// object's layer is evaluated only at claim 4's moments, and the
    /// checks of jumpjet movement and falling compare the layers before and
    /// after the object's own step, not the list it is in. Off: the checks
    /// give way to a sweep every tick that moves an object whose evaluated
    /// layer differs from the list it is in; claim 4's re-files happen in
    /// both states. A drawing setting (S-0102 r43, Q-2467): its off position
    /// moves objects only between the lists the drawing reads, and the
    /// simulation's list readers take the on position's membership in both
    /// positions, so it never reaches the simulation (S-0306 r75 claim 17).
    pub layer_change_on_events_only: bool,
    /// `quirks.enable_unfreezes_unpowered` (S-0108 claims 13, 26 and 28):
    /// re-enabling a disabled building (a switch-on, also in a house short
    /// of power and of a building the map placed switched off; the end of a
    /// pulse; an ownership change of a building whose type needs an
    /// engineer, also while it is switched off) unfreezes its
    /// *freeze-unpowered* slot animations even when it is not in operation.
    /// An ownership change of any other building re-enables nothing. Off:
    /// only when it is in operation right after the re-enabling.
    pub enable_unfreezes_unpowered: bool,
    /// `quirks.building_resource_pips_hidden` (S-0111 claim 11 and Edge
    /// cases): a building whose pip-scale kind is resource shows no pip
    /// scale at all, its control-group number included. Off: it shows its
    /// group number as usual (the resource row of claim 14 is not drawn
    /// yet).
    pub building_resource_pips_hidden: bool,
}

impl Default for ViewQuirks {
    fn default() -> Self {
        // spec: S-0102 Edge cases and quirks, S-0108 Edge cases and quirks (every switch defaults on)
        Self {
            cliff_fudge_far_cliff_weaker: true,
            occupiable_hurt_rule: true,
            sprite_vehicle_eight_facings_only: true,
            roof_unload_no_fallback: true,
            unsorted_upper_layers: true,
            layer_change_on_events_only: true,
            enable_unfreezes_unpowered: true,
            // spec: S-0111 Edge cases (quirks.building_resource_pips_hidden, default on)
            building_resource_pips_hidden: true,
        }
    }
}

impl ViewQuirks {
    /// The switches by name.
    pub fn named(&self) -> [(&'static str, bool); 8] {
        [
            (
                "cliff_fudge_far_cliff_weaker",
                self.cliff_fudge_far_cliff_weaker,
            ),
            ("occupiable_hurt_rule", self.occupiable_hurt_rule),
            (
                "sprite_vehicle_eight_facings_only",
                self.sprite_vehicle_eight_facings_only,
            ),
            ("roof_unload_no_fallback", self.roof_unload_no_fallback),
            ("unsorted_upper_layers", self.unsorted_upper_layers),
            (
                "layer_change_on_events_only",
                self.layer_change_on_events_only,
            ),
            (
                "enable_unfreezes_unpowered",
                self.enable_unfreezes_unpowered,
            ),
            (
                "building_resource_pips_hidden",
                self.building_resource_pips_hidden,
            ),
        ]
    }
}

/// The cliff factor of S-0102 claim 14 from whether the cell one step and
/// the cell two steps in front are high.
pub fn cliff_factor(high_one: bool, high_two: bool, q: &ViewQuirks) -> i32 {
    if !q.cliff_fudge_far_cliff_weaker {
        // spec: S-0102 Edge cases (quirks.cliff_fudge_far_cliff_weaker off: factor 2 whenever either cell qualifies)
        return if high_one || high_two { 2 } else { 0 };
    }
    // spec: S-0102 claim 14 (cliff factor: two-step high 1, else one-step high 2, else 0)
    if high_two {
        1
    } else if high_one {
        2
    } else {
        0
    }
}

/// The lift an overlay's depth bias takes for `level` height levels
/// (S-0102 claim 13: exactly 15 per level, which equals the objects' lift
/// L(104 × level) at every whole level; there is no switch).
pub fn overlay_depth_lift(level: i32) -> i32 {
    // spec: S-0102 claims 11 and 13 (overlays and their shadows use exactly 15 pixels per level)
    TERRAIN_LIFT * level
}

/// The facing part *d* of a vehicle drawn as a sprite (S-0102 claim 22)
/// whose type has `facings` facings, for body facing `body`. Disguise is
/// not modelled (a disguised vehicle would give 0 in both states).
pub fn sprite_vehicle_facing_part(facings: i32, body: u16, q: &ViewQuirks) -> i32 {
    if q.sprite_vehicle_eight_facings_only {
        // spec: S-0102 claim 22 (d = (f₈ + 1) mod 8 with 8 facings, else 0)
        return if facings == 8 {
            (n_step(body, 8) as i32 + 1) % 8
        } else {
            0
        };
    }
    // spec: S-0102 claim 29, Edge cases (quirks.sprite_vehicle_eight_facings_only off: a power of
    // two n from 2 to 256 gives d = ⌊((f + 8192) mod 65536)·n/65536 + ½⌋ mod n, the n-step facing
    // (S-0204 F4) of the body facing turned clockwise by 8192 facing units; any other count gives 0)
    match u32::try_from(facings) {
        Ok(n) if (2..=256).contains(&n) && n.is_power_of_two() => {
            n_step(body.wrapping_add(8192), n) as i32
        }
        _ => 0,
    }
}

/// Whether an occupiable building counts as hurt (S-0108 claim 2).
pub fn occupiable_hurt(tech_level: i32, damaged: bool, badly: bool, q: &ViewQuirks) -> bool {
    if !q.occupiable_hurt_rule {
        // spec: S-0108 Edge cases (quirks.occupiable_hurt_rule off: always the damaged threshold)
        return damaged;
    }
    // spec: S-0108 claim 2 (hurt: badly damaged at tech level 0 or below, damaged above)
    if tech_level <= 0 { badly } else { damaged }
}

/// The image-frame index of an occupiable building outside construction
/// (S-0108 claim 2): 2·[occupied] + [hurt], so an occupied, hurt building
/// uses frame 3, a civilian building (tech level −1) included.
pub fn occupiable_frame(
    tech_level: i32,
    damaged: bool,
    badly: bool,
    occupied: bool,
    q: &ViewQuirks,
) -> i32 {
    // spec: S-0108 claim 2 (2·[occupied] + [hurt])
    2 * i32::from(occupied) + i32::from(occupiable_hurt(tech_level, damaged, badly, q))
}

/// The body frame drawn for body frame `f` of a sprite with `n` image
/// frames (S-0108 claim 3). The result may be negative; such an index
/// draws nothing (claim 3, F2). A sprite with no body frame (`n` < 2)
/// always gives a negative index, so neither its body nor its shadow
/// (the body frame plus ⌊N/2⌋) is drawn.
pub fn drawn_body_frame(f: i32, n: i32) -> i32 {
    // spec: S-0108 claim 3 (drawn frame min(f, ⌊N/2⌋ − 1), the last body frame, a departure; with no
    // body frames nothing is drawn; a negative index is not raised to 0)
    f.min(n / 2 - 1)
}

/// Named switches of the rendering specs (`spec/rendering/S-01*.md`) that
/// have no field in a quirk struct, with the reason: the behaviour they
/// switch is not implemented yet, or another part of the engine owns them.
/// Mirrored in `docs/development.md`.
pub const NOT_IMPLEMENTED: &[(&str, &str)] = &[
    // S-0101
    (
        "screen_to_ground_not_exact_inverse",
        "S-0101 claim 9: no F6 lepton position is used; the cell under a pixel is F7's pixel formula, the same in both settings",
    ),
    // S-0102
    (
        "slot_values_replace_animation_adjustments",
        "S-0102 claim 17: the on state is drawn; the off state (the slot's values added to the type's) is not implemented",
    ),
    // S-0103
    (
        "parachute_brightness_fixed",
        "S-0103 claim 8: the on state is drawn (a parachute keeps the light of its creation tick); the off state is not implemented",
    ),
    (
        "named_palette_true_owner",
        "S-0103 claim 16: needs S-0516 claim 37 disguise drawing; named palettes are drawn, but no object is drawn as its disguise yet, so neither state can show",
    ),
    // S-0104
    (
        "split_effect_priority",
        "S-0104 claim 2: objects' height terms take the height-term order, as with the switch on, but every cell is relit with the one global choice (the tint/target order); the off state is not implemented",
    ),
    (
        "chrono_flash_no_reevaluation",
        "S-0104 claims 2 and 5: nothing starts a chrono flash",
    ),
    (
        "dominator_height_terms_linger",
        "S-0104 claim 16: every change of the ambient or of the global lighting choice relights every cell in full with the chosen set; the lingering height terms are not modelled",
    ),
    (
        "trigger_tint_retints_all_tables",
        "S-0104 claim 17: no trigger action changes the map tint (the set-base-ambient action, claim 14, changes only the ambient)",
    ),
    // S-0105
    (
        "vehicle_no_recoil",
        "S-0105 claim 28: recoil placement is not applied; vehicles never recoil, as with the switch on",
    ),
    (
        "barrel_only_no_recoil",
        "S-0105 claim 37: recoil placement is not applied; a barrel without a turret never recoils, as with the switch on",
    ),
    (
        "voxel_single_pixel",
        "S-0105 claim 22: the on state is drawn; the off state (no gaps in magnified sections) is not implemented",
    ),
    (
        "voxel_no_intersection_depth",
        "S-0105 claim 23: the on state is drawn; the off state (a depth test between sections) is not implemented",
    ),
    (
        "no_turret_shadow",
        "S-0105 claim 29: turrets and barrels cast no shadow, as with the switch on; the off state's shadow is not specified",
    ),
    // S-0106
    (
        "tilter_flag_ignored",
        "S-0106 claim 10: voxel bodies are not tilted on slopes",
    ),
    // S-0108
    (
        "full_state_unmaxed",
        "S-0108 claim 2: buildings are drawn in their idle state only",
    ),
    (
        "turret_building_no_body_wrap",
        "S-0108 claim 6: building body ranges are not played",
    ),
    (
        "turret_start_clears_damaged",
        "S-0108 claims 11 and 12: building slot animations are played, but the turret slot is not",
    ),
    (
        "damage_swap_drops_occupied",
        "S-0108 claims 11 and 12: the on state is played (a damage swap never picks the occupied slot); the off state is not implemented",
    ),
    (
        "theater_refresh_buildup_range",
        "S-0108 claim 5: no theater refresh",
    ),
    // S-0109
    (
        "anim_hide_at_level_15",
        "S-0109 claim 20: building slot animations are played but drawn at translucency level t = 0, as every other animation; their level is not modelled",
    ),
    (
        "anim_low_detail_shows_cloaked",
        "S-0109 claim 20: building slot animations are played but drawn at translucency level t = 0, as every other animation; their level is not modelled",
    ),
    (
        "anim_translucency_levels",
        "S-0109 claim 20: building slot animations are played but drawn at translucency level t = 0, as every other animation; their level is not modelled",
    ),
];

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use glasnost_render::projection::{LEVEL_LEPTONS, height_lift};

    use super::*;

    fn off() -> ViewQuirks {
        ViewQuirks {
            cliff_fudge_far_cliff_weaker: false,
            occupiable_hurt_rule: false,
            sprite_vehicle_eight_facings_only: false,
            roof_unload_no_fallback: false,
            unsorted_upper_layers: false,
            layer_change_on_events_only: false,
            enable_unfreezes_unpowered: false,
            building_resource_pips_hidden: false,
        }
    }

    /// spec: S-0102 claims 22 and 29, Edge cases (quirks.sprite_vehicle_eight_facings_only; D37)
    #[test]
    fn sprite_vehicle_facing_switch() {
        let on = ViewQuirks::default();
        let off = off();
        const NORTH: u16 = 0;
        const EAST: u16 = 16384;
        // On: only a count of 8 turns the image; map north is part 1, east part 3.
        assert_eq!(sprite_vehicle_facing_part(8, NORTH, &on), 1);
        assert_eq!(sprite_vehicle_facing_part(8, EAST, &on), 3);
        assert_eq!(sprite_vehicle_facing_part(4, EAST, &on), 0);
        assert_eq!(sprite_vehicle_facing_part(16, EAST, &on), 0);
        // Off, n = 8: the on state's formula for every facing.
        for f in (0..=u16::MAX).step_by(97) {
            assert_eq!(
                sprite_vehicle_facing_part(8, f, &off),
                sprite_vehicle_facing_part(8, f, &on)
            );
        }
        // Off, n = 4: a vehicle facing map north shows part 1 (the half goes clockwise).
        assert_eq!(sprite_vehicle_facing_part(4, NORTH, &off), 1);
        assert_eq!(sprite_vehicle_facing_part(4, EAST, &off), 2);
        // Off, n = 2: 0 + 8192 rounds to step 0, east + 8192 to step 1.
        assert_eq!(sprite_vehicle_facing_part(2, NORTH, &off), 0);
        assert_eq!(sprite_vehicle_facing_part(2, EAST, &off), 1);
        // Off, n = 16: (f₁₆ + 2) mod 16.
        assert_eq!(sprite_vehicle_facing_part(16, EAST, &off), 6);
        assert_eq!(sprite_vehicle_facing_part(16, 65535, &off), 2);
        // Off, the boundary checks of D37 (S-0102 claim 29).
        for (n, f, d) in [
            (2, 8191, 0),
            (2, 8192, 1),
            (2, 40959, 1),
            (2, 40960, 0),
            (4, 65535, 0),
            (4, 0, 1),
            (4, 16383, 1),
            (4, 16384, 2),
        ] {
            assert_eq!(sprite_vehicle_facing_part(n, f, &off), d, "n {n}, f {f}");
        }
        // Off, a count that is not a power of two from 2 to 256: 0.
        assert_eq!(sprite_vehicle_facing_part(3, EAST, &off), 0);
        assert_eq!(sprite_vehicle_facing_part(1, EAST, &off), 0);
        assert_eq!(sprite_vehicle_facing_part(512, EAST, &off), 0);
    }

    /// spec: S-0102 claim 14, Edge cases (quirks.cliff_fudge_far_cliff_weaker)
    #[test]
    fn cliff_factor_switch() {
        let on = ViewQuirks::default();
        assert_eq!(cliff_factor(false, false, &on), 0);
        assert_eq!(cliff_factor(true, false, &on), 2);
        assert_eq!(cliff_factor(false, true, &on), 1);
        assert_eq!(cliff_factor(true, true, &on), 1);
        let off = off();
        assert_eq!(cliff_factor(false, false, &off), 0);
        assert_eq!(cliff_factor(true, false, &off), 2);
        assert_eq!(cliff_factor(false, true, &off), 2);
        assert_eq!(cliff_factor(true, true, &off), 2);
    }

    /// spec: S-0102 claims 11 and 13 (overlays lift exactly 15 per level, equal to L(104·ℓ) at
    /// every whole level), S-0102 Terms (L(z) = ⌊15·z/104 + ½⌋ for signed z: L(400) = 58),
    /// S-0101 claim 3, F2
    #[test]
    fn overlay_lift_is_fifteen_per_level() {
        assert_eq!(overlay_depth_lift(20), 300);
        for level in -128..=127 {
            assert_eq!(overlay_depth_lift(level), 15 * level, "level {level}");
            assert_eq!(
                overlay_depth_lift(level),
                height_lift(LEVEL_LEPTONS * level),
                "level {level}"
            );
        }
        assert_eq!(height_lift(400), 58);
        // Signed z: the floor, not truncation toward zero (−75/104 + ½ < 0).
        assert_eq!(height_lift(-5), -1);
        assert_eq!(height_lift(-3), 0);
    }

    /// spec: S-0108 claim 2, Edge cases (quirks.occupiable_hurt_rule)
    #[test]
    fn occupiable_hurt_switch() {
        let on = ViewQuirks::default();
        // Damaged but not badly damaged.
        assert!(!occupiable_hurt(0, true, false, &on));
        assert!(occupiable_hurt(1, true, false, &on));
        assert!(occupiable_hurt(0, true, false, &off()));
        assert!(occupiable_hurt(-1, true, false, &off()));
        assert!(!occupiable_hurt(0, false, false, &off()));
    }

    /// spec: S-0108 claim 2 (an occupied building uses frame 2, an occupied, hurt one frame 3, a civilian building included)
    #[test]
    fn occupied_frames() {
        let on = ViewQuirks::default();
        assert_eq!(occupiable_frame(-1, false, false, false, &on), 0);
        assert_eq!(occupiable_frame(-1, true, true, false, &on), 1);
        assert_eq!(occupiable_frame(-1, false, false, true, &on), 2);
        assert_eq!(occupiable_frame(-1, true, true, true, &on), 3);
        assert_eq!(occupiable_frame(5, true, false, true, &on), 3);
    }

    /// spec: S-0108 claim 3 (drawn frame min(f, ⌊N/2⌋ − 1): the last body frame, with its shadow)
    #[test]
    fn body_frame_clamp() {
        // A sprite of 4 frames: body frames 0 and 1, shadows 2 and 3.
        assert_eq!(drawn_body_frame(1, 4), 1);
        assert_eq!(drawn_body_frame(2, 4), 1);
        assert_eq!(drawn_body_frame(3, 4), 1);
        // Its shadow is the clamped frame plus ⌊N/2⌋: 2⌊N/2⌋ − 1.
        assert_eq!(drawn_body_frame(9, 4) + 4 / 2, 3);
        // An odd count: 5 frames, ⌊5/2⌋ = 2 body frames; the shadow of the
        // clamped frame is frame 3 = 2⌊N/2⌋ − 1.
        assert_eq!(drawn_body_frame(4, 5), 1);
        assert_eq!(drawn_body_frame(4, 5) + 5 / 2, 3);
    }

    /// spec: S-0108 claim 3 (with no body frames, no body or shadow is drawn)
    #[test]
    fn no_body_frames_draw_nothing() {
        for n in [0, 1] {
            for f in [0, 1, 5] {
                let body = drawn_body_frame(f, n);
                assert!(body < 0, "n {n}, f {f}");
                // The shadow index (body + ⌊N/2⌋) is negative too.
                assert!(body + n / 2 < 0, "n {n}, f {f}");
            }
        }
    }

    /// spec: S-0108 claim 3, F2 (a negative index is kept, not raised to image frame 0)
    #[test]
    fn negative_body_frame_is_kept() {
        assert_eq!(drawn_body_frame(-1, 4), -1);
        assert_eq!(drawn_body_frame(-3, 4), -3);
    }

    /// Every `quirks.name` that a rendering spec names is a field of a quirk
    /// struct (render, mapview, the importer's or the simulation's) or is
    /// listed in [`NOT_IMPLEMENTED`] with its reason.
    #[test]
    fn every_rendering_spec_quirk_has_a_switch() {
        use glasnost_convert::art::ArtQuirks;
        use glasnost_convert::layers::LayerQuirks;
        use glasnost_render::compose::ComposeQuirks;
        use glasnost_render::light::LightQuirks;
        use glasnost_render::overlay::OverlayQuirks;
        use glasnost_render::palette::PaletteQuirks;
        use glasnost_render::projection::ProjectionQuirks;
        use glasnost_render::shroud::ShroudQuirks;
        use glasnost_render::voxel::VoxelQuirks;

        // The switches of the renderer and the map viewer.
        let ours: Vec<(&str, bool)> = [
            ViewQuirks::default().named().to_vec(),
            ProjectionQuirks::default().named().to_vec(),
            ComposeQuirks::default().named().to_vec(),
            LightQuirks::default().named().to_vec(),
            VoxelQuirks::default().named().to_vec(),
            OverlayQuirks::default().named().to_vec(),
            PaletteQuirks::default().named().to_vec(),
            ShroudQuirks::default().named().to_vec(),
            crate::mapview::anims::AnimQuirks::default()
                .named()
                .to_vec(),
        ]
        .concat();
        assert!(
            ours.iter().all(|(_, v)| *v),
            "every default is the original"
        );
        // Their owners elsewhere: the importer's and the simulation's.
        let mut have: BTreeSet<&str> = ours.iter().map(|(n, _)| *n).collect();
        have.extend(LayerQuirks::default().named().map(|(n, _)| n));
        have.extend(ArtQuirks::default().named().map(|(n, _)| n));
        have.extend(
            glasnost_convert::sim::SimQuirks::default()
                .named()
                .map(|(n, _)| n),
        );
        let sim = glasnost_rules::quirks::Quirks::default();
        have.extend(sim.named().iter().map(|(n, _)| *n));
        // S-0005's switches (the file lookup's), which S-0111 claim 19a names for the cursor
        // palette; every default is the original.
        let files = glasnost_vfs::GameQuirks::default().named();
        assert!(
            files.iter().all(|(_, v)| *v),
            "every default is the original"
        );
        have.extend(files.map(|(n, _)| n));
        // A listed name must not also be a field.
        for (n, _) in NOT_IMPLEMENTED {
            assert!(!have.contains(n), "{n} is a field and also listed");
        }

        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../spec/rendering");
        let mut files: Vec<_> = std::fs::read_dir(root)
            .expect("spec/rendering")
            .map(|e| e.expect("entry").path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("S-01") && n.ends_with(".md"))
            })
            .collect();
        files.sort();
        assert!(files.len() >= 12, "found {} rendering specs", files.len());
        // Switches a spec names only to withdraw them.
        const WITHDRAWN: &[&str] = &[
            "pip_colour_default_shift", // spec: S-0111 Cross-spec consistency (the former switch is withdrawn; pip-value retention is mandatory)
        ];
        let mut missing = BTreeSet::new();
        let mut seen = BTreeSet::new();
        for f in files {
            let text = std::fs::read_to_string(&f).expect("read spec");
            let mut rest = text.as_str();
            while let Some(i) = rest.find("quirks.") {
                rest = &rest[i + 7..];
                let name: String = rest
                    .chars()
                    .take_while(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '_')
                    .collect();
                if name.is_empty() {
                    continue;
                }
                seen.insert(name.clone());
                if !have.contains(name.as_str())
                    && !NOT_IMPLEMENTED.iter().any(|(n, _)| *n == name)
                    && !WITHDRAWN.contains(&name.as_str())
                {
                    missing.insert(format!("{} in {}", name, f.display()));
                }
            }
        }
        assert!(missing.is_empty(), "quirks without a switch: {missing:#?}");
        // The list holds no stale names.
        for (n, _) in NOT_IMPLEMENTED {
            assert!(
                seen.contains(*n),
                "{n} is listed but no rendering spec names it"
            );
        }
    }

    /// The table of unimplemented switches in docs/development.md lists
    /// exactly the names of `NOT_IMPLEMENTED`.
    #[test]
    fn docs_table_matches_not_implemented() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/development.md");
        let text = std::fs::read_to_string(path).expect("docs/development.md");
        let start = text
            .find("**Named switches of specs whose behaviour is not implemented yet.**")
            .expect("the table's heading");
        let documented: BTreeSet<&str> = text[start..]
            .lines()
            .skip_while(|l| !l.starts_with('|'))
            .take_while(|l| l.starts_with('|'))
            .filter_map(|l| l.strip_prefix("| `")?.split('`').next())
            .collect();
        let listed: BTreeSet<&str> = NOT_IMPLEMENTED.iter().map(|(n, _)| *n).collect();
        assert_eq!(documented, listed);
    }
}
