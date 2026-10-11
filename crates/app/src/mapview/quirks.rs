//! Quirk switches of the rendering specs that the map viewer's scene
//! builder and live drawing implement (S-0102 draw order, S-0108 building
//! frames), each defaulting to the original's behaviour, with the small
//! rules they switch.
//!
//! The switches of the render crate live with the code they govern
//! (`render::projection::ProjectionQuirks`, `render::compose::ComposeQuirks`,
//! `render::light::LightQuirks`, `render::voxel::VoxelQuirks` and
//! `RecoilQuirks`, `render::overlay::OverlayQuirks`,
//! `render::palette::PaletteQuirks`). [`NOT_IMPLEMENTED`] lists the named
//! switches of the rendering specs that have no field, with the reason;
//! the test at the end of this file checks that every switch the rendering
//! specs name is one or the other.

use glasnost_render::projection::{
    LEVEL_LEPTONS, ProjectionQuirks, TERRAIN_LIFT, height_lift_with,
};

/// Switches of S-0102 and S-0108 implemented in `mapview`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewQuirks {
    /// `quirks.ground_layer_single_pass_sort` (S-0102 claim 6): the ground
    /// layer gets one pass of adjacent swaps per tick. Off: it is fully
    /// sorted by key every tick (stable).
    pub ground_layer_single_pass_sort: bool,
    /// `quirks.cliff_fudge_far_cliff_weaker` (S-0102 claim 14): a high
    /// cell two steps in front gives cliff factor 1 and overrides a high
    /// cell one step in front, which alone gives 2. Off: factor 2 whenever
    /// either cell is high.
    pub cliff_fudge_far_cliff_weaker: bool,
    /// `quirks.overlay_lift_exact_levels` (S-0102 claim 13, Edge cases):
    /// overlay depth lifts by 15 per level while object depth uses
    /// −L(z). Off: "use the same lift for both"; the switch is the
    /// overlay's, so we read it as the overlay taking the objects' lift
    /// L(104 × level) (S-0101 F2).
    pub overlay_lift_exact_levels: bool,
    /// `quirks.occupiable_hurt_rule` (S-0108 claim 2): for an occupiable
    /// type, "hurt" is badly damaged at tech level 0 or below and damaged
    /// above. Off: always damaged.
    pub occupiable_hurt_rule: bool,
    /// `quirks.building_frame_clamp_to_shadow` (S-0108 claim 3): the drawn
    /// body frame is min(f, ⌊N/2⌋), the first shadow frame. Off:
    /// min(f, ⌊N/2⌋ − 1), the last body frame.
    pub building_frame_clamp_to_shadow: bool,
}

impl Default for ViewQuirks {
    fn default() -> Self {
        // spec: S-0102 Edge cases and quirks, S-0108 Edge cases and quirks (every switch defaults on)
        Self {
            ground_layer_single_pass_sort: true,
            cliff_fudge_far_cliff_weaker: true,
            overlay_lift_exact_levels: true,
            occupiable_hurt_rule: true,
            building_frame_clamp_to_shadow: true,
        }
    }
}

impl ViewQuirks {
    /// The switches by name.
    pub fn named(&self) -> [(&'static str, bool); 5] {
        [
            (
                "ground_layer_single_pass_sort",
                self.ground_layer_single_pass_sort,
            ),
            (
                "cliff_fudge_far_cliff_weaker",
                self.cliff_fudge_far_cliff_weaker,
            ),
            ("overlay_lift_exact_levels", self.overlay_lift_exact_levels),
            ("occupiable_hurt_rule", self.occupiable_hurt_rule),
            (
                "building_frame_clamp_to_shadow",
                self.building_frame_clamp_to_shadow,
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
/// (S-0102 claim 13: 15 per level). `p` gives the objects' lift L(z),
/// which the overlay takes with the switch off.
pub fn overlay_depth_lift(level: i32, q: &ViewQuirks, p: &ProjectionQuirks) -> i32 {
    if q.overlay_lift_exact_levels {
        // spec: S-0102 claim 13 (overlay biases lift by 15 per level)
        TERRAIN_LIFT * level
    } else {
        // spec: S-0102 Edge cases (quirks.overlay_lift_exact_levels off: the objects' lift L(z), S-0101 F2)
        height_lift_with(LEVEL_LEPTONS * level, p)
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

/// The body frame drawn for body frame `f` of a sprite with `n` image
/// frames (S-0108 claim 3).
pub fn drawn_body_frame(f: usize, n: usize, q: &ViewQuirks) -> usize {
    if q.building_frame_clamp_to_shadow {
        // spec: S-0108 claim 3 (drawn frame min(f, ⌊N/2⌋))
        f.min(n / 2)
    } else {
        // spec: S-0108 Edge cases (quirks.building_frame_clamp_to_shadow off: clamp to ⌊N/2⌋ − 1)
        f.min((n / 2).saturating_sub(1))
    }
}

/// Named switches of the rendering specs (`spec/rendering/S-01*.md`) that
/// have no field in a quirk struct, with the reason: the behaviour they
/// switch is not implemented yet, or another part of the engine owns them.
/// Mirrored in `docs/development.md`.
pub const NOT_IMPLEMENTED: &[(&str, &str)] = &[
    // S-0101
    (
        "screen_band_tests_position_only",
        "S-0101 claim 6: our renderer does no on-screen culling (no screen band or ground box)",
    ),
    (
        "screen_to_ground_not_exact_inverse",
        "S-0101 claim 9: no F6 lepton position is used; the cell under a pixel is F7's pixel formula, the same in both settings",
    ),
    // S-0102
    (
        "unsorted_upper_layers",
        "S-0102 claim 7: every mobile object is drawn in the ground layer; the upper layers are not modelled",
    ),
    (
        "layer_change_on_events_only",
        "S-0102 claim 4: render layer membership is not modelled (the moments of claim 4 may be simulation state)",
    ),
    (
        "remembered_animation_in_cell_pass",
        "S-0102 claim 16: no fog, so no remembered animations",
    ),
    (
        "slot_values_replace_animation_adjustments",
        "S-0102 claim 17: building slot animations are not drawn",
    ),
    (
        "sprite_vehicle_eight_facings_only",
        "S-0102 claim 22: the on state is drawn; the off state's facing part for counts 2 and 4 is asked in Q-0050",
    ),
    // S-0103
    (
        "deploy_animation_neutral_brightness",
        "S-0103 claim 8: deploy and undeploy animations are not drawn",
    ),
    (
        "parachute_brightness_fixed",
        "S-0103 claim 8: parachutes are not drawn",
    ),
    // S-0104
    (
        "zero_rate_freezes_ambient",
        "S-0104 claim 4: the ambient fade's timing is not modelled (render::light::ambient_step takes one step)",
    ),
    (
        "dominator_rate_thousandths",
        "S-0104 claim 4: no dominator effect and no fade timing",
    ),
    (
        "split_effect_priority",
        "S-0104 claim 2: the lighting effects in force (nuke, dominator, ion storm, chrono flash) are not modelled",
    ),
    (
        "chrono_flash_no_reevaluation",
        "S-0104 claims 2 and 5: no chrono flash and no lighting re-evaluations",
    ),
    (
        "dominator_height_terms_linger",
        "S-0104 claim 16: no dominator effect",
    ),
    (
        "trigger_tint_retints_all_tables",
        "S-0104 claim 17: trigger actions do not change the lighting",
    ),
    (
        "infantry_height_light_limited",
        "S-0104 claim 13: the on state is drawn; the off state at a height of 208 leptons in a shadowed cell is asked in Q-0050",
    ),
    // S-0105
    (
        "turret_recoil_leaves_barrel",
        "S-0105 claim 27: recoil placement in a pose is not applied (no firing yet)",
    ),
    (
        "vehicle_no_recoil",
        "S-0105 claim 28: recoil placement is not applied; vehicles never recoil, as with the switch on",
    ),
    (
        "barrel_only_no_recoil",
        "S-0105 claim 37: recoil placement is not applied; a barrel without a turret never recoils, as with the switch on",
    ),
    (
        "alt_body_turret_model",
        "S-0105 claim 12: the alternative body and spawned-aircraft docking are not drawn; the main body always is",
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
        "voxel_buffer_wrap",
        "S-0105 claim 22: the on state is drawn; which off state the switch selects is a question for the lead in S-0105",
    ),
    (
        "barrel_pivot_frame_mixup",
        "S-0105 claim 10: the on state is drawn; which off state the switch selects is a question for the lead in S-0105",
    ),
    (
        "depth_mode_leak",
        "S-0105 claim 32: the on state is drawn (render::voxel::VoxelCanvas); which off state the switch selects is asked in Q-0056",
    ),
    (
        "no_turret_shadow",
        "S-0105 claim 29: turrets and barrels cast no shadow, as with the switch on; the off state's shadow is not specified",
    ),
    // S-0106
    (
        "slope_tilt_angles",
        "S-0106 claim 4: voxel bodies are not tilted on slopes",
    ),
    (
        "tilter_flag_ignored",
        "S-0106 claim 10: voxel bodies are not tilted on slopes",
    ),
    (
        "tilt_transition_restart_jump",
        "S-0106 claim 6: no tilt state (it may belong to the simulation's locomotors)",
    ),
    // S-0108
    (
        "unbuildable_occupied_damaged_frame",
        "S-0108 claim 2: no building is occupied in our engine yet",
    ),
    (
        "full_state_unmaxed",
        "S-0108 claim 2: buildings are drawn in their idle state only",
    ),
    (
        "gate_double_reverse",
        "S-0108 claim 4: buildings are not sold",
    ),
    (
        "building_wrap_normalisation",
        "S-0108 claim 6: building body ranges are not played",
    ),
    (
        "turret_building_no_body_wrap",
        "S-0108 claim 6: building body ranges are not played",
    ),
    (
        "turret_start_clears_damaged",
        "S-0108 claims 11 and 12: building slot animations are not drawn",
    ),
    (
        "enable_unfreezes_unpowered",
        "S-0108 claim 13: building slot animations are not drawn",
    ),
    (
        "damage_swap_drops_occupied",
        "S-0108 claims 11 and 12: building slot animations are not drawn",
    ),
    (
        "roof_unload_no_fallback",
        "S-0108 claim 22: units do not unload from buildings",
    ),
    (
        "theater_refresh_buildup_range",
        "S-0108 claim 5: no theater refresh",
    ),
    // S-0109
    (
        "anim_damage_starts_at_one",
        "S-0109: animation playback is not implemented",
    ),
    (
        "anim_erasure_late_slots_opaque",
        "S-0109: animation playback is not implemented",
    ),
    (
        "anim_fast_rate_freezes",
        "S-0109: animation playback is not implemented",
    ),
    (
        "anim_followup_keeps_step",
        "S-0109: animation playback is not implemented",
    ),
    (
        "anim_followup_start_doubled",
        "S-0109: animation playback is not implemented",
    ),
    (
        "anim_hide_at_level_15",
        "S-0109: animation playback is not implemented",
    ),
    (
        "anim_loop_count_wrap",
        "S-0109: animation playback is not implemented",
    ),
    (
        "anim_loop_end_from_sprite",
        "S-0109: animation playback is not implemented",
    ),
    (
        "anim_low_detail_shows_cloaked",
        "S-0109: animation playback is not implemented",
    ),
    (
        "anim_pingpong_lower_turn",
        "S-0109: animation playback is not implemented",
    ),
    (
        "anim_random_rate_collapse",
        "S-0109: animation playback is not implemented",
    ),
    (
        "anim_reverse_raw_loop_end",
        "S-0109: animation playback is not implemented",
    ),
    (
        "anim_translucency_levels",
        "S-0109: animation playback is not implemented",
    ),
    // S-0110
    (
        "shroud_on_flat_grid",
        "S-0110: shroud and fog are not drawn",
    ),
    (
        "shroud_point_cell_wrap",
        "S-0110: shroud and fog are not drawn",
    ),
    // S-0111
    (
        "building_resource_pips_hidden",
        "S-0111 claim 11: the pip scale is not drawn (health bars, unit brackets and insignia are; they have no switch)",
    ),
    (
        "passenger_overflow_hides_pips",
        "S-0111 claims 11 and 12: the pip scale is not drawn",
    ),
    (
        "pip_colour_default_shift",
        "S-0111 Edge cases: the passenger and occupant pip colours are not imported; the pip scale is not drawn",
    ),
];

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    fn off() -> ViewQuirks {
        ViewQuirks {
            ground_layer_single_pass_sort: false,
            cliff_fudge_far_cliff_weaker: false,
            overlay_lift_exact_levels: false,
            occupiable_hurt_rule: false,
            building_frame_clamp_to_shadow: false,
        }
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

    /// spec: S-0102 claim 13, Edge cases (quirks.overlay_lift_exact_levels), S-0101 F2
    #[test]
    fn overlay_lift_switch() {
        let on = ViewQuirks::default();
        let off = off();
        let p = ProjectionQuirks::default();
        // Up to level 19 the two lifts agree; from level 20 they differ.
        for level in 0..20 {
            assert_eq!(
                overlay_depth_lift(level, &on, &p),
                overlay_depth_lift(level, &off, &p)
            );
        }
        assert_eq!(overlay_depth_lift(20, &on, &p), 300);
        assert_eq!(overlay_depth_lift(20, &off, &p), 299);
        // Off, with the exact object lift (S-0101 quirks.height_lift_float_scale off).
        let exact = ProjectionQuirks {
            height_lift_float_scale: false,
            ..p
        };
        assert_eq!(overlay_depth_lift(20, &off, &exact), 300);
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

    /// spec: S-0108 claim 3, Edge cases (quirks.building_frame_clamp_to_shadow)
    #[test]
    fn body_frame_clamp_switch() {
        let on = ViewQuirks::default();
        // A sprite of 4 frames: body frames 0 and 1, shadows 2 and 3.
        assert_eq!(drawn_body_frame(1, 4, &on), 1);
        assert_eq!(drawn_body_frame(3, 4, &on), 2);
        assert_eq!(drawn_body_frame(3, 4, &off()), 1);
        assert_eq!(drawn_body_frame(0, 1, &off()), 0);
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
        use glasnost_render::voxel::{RecoilQuirks, VoxelQuirks};

        // The switches of the renderer and the map viewer.
        let ours: Vec<(&str, bool)> = [
            ViewQuirks::default().named().to_vec(),
            ProjectionQuirks::default().named().to_vec(),
            ComposeQuirks::default().named().to_vec(),
            LightQuirks::default().named().to_vec(),
            VoxelQuirks::default().named().to_vec(),
            RecoilQuirks::default().named().to_vec(),
            OverlayQuirks::default().named().to_vec(),
            PaletteQuirks::default().named().to_vec(),
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
        let sim = glasnost_rules::quirks::Quirks::default();
        have.extend(sim.named().iter().map(|(n, _)| *n));
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
                if !have.contains(name.as_str()) && !NOT_IMPLEMENTED.iter().any(|(n, _)| *n == name)
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
