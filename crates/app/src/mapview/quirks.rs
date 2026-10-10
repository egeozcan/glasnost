//! Quirk switches of the rendering specs that the map viewer's scene
//! builder and live drawing implement (S-0102 draw order, S-0108 building
//! frames), each defaulting to the original's behaviour, with the small
//! rules they switch.
//!
//! The switches of the render crate live with the code they govern
//! (`render::projection::ProjectionQuirks`, `render::compose::ComposeQuirks`,
//! `render::light::LightQuirks`, `render::voxel::VoxelQuirks` and
//! `RecoilQuirks`, `render::overlay::OverlayQuirks`,
//! `render::palette::PaletteQuirks`).

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
/// (S-0102 claim 13: 15 per level).
pub fn overlay_depth_lift(level: i32, q: &ViewQuirks) -> i32 {
    if q.overlay_lift_exact_levels {
        // spec: S-0102 claim 13 (overlay biases lift by 15 per level)
        TERRAIN_LIFT * level
    } else {
        // spec: S-0102 Edge cases (quirks.overlay_lift_exact_levels off: the objects' lift L(z), S-0101 F2)
        height_lift_with(LEVEL_LEPTONS * level, &ProjectionQuirks::default())
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

#[cfg(test)]
mod tests {
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
        // Up to level 19 the two lifts agree; from level 20 they differ.
        for level in 0..20 {
            assert_eq!(
                overlay_depth_lift(level, &on),
                overlay_depth_lift(level, &off)
            );
        }
        assert_eq!(overlay_depth_lift(20, &on), 300);
        assert_eq!(overlay_depth_lift(20, &off), 299);
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
}
