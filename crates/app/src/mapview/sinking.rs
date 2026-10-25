//! The list of a sinking vessel: a drawing rule of our own design, the
//! interim of Q-1871 (the lead's play note 20 of 2026-10-25, "the ships do
//! not sink with an angle").
//!
//! No approved spec gives a sinking vessel's body a tilt: S-0504 claim 4
//! lowers it by its descent and S-0105 builds the body transform from its
//! facing alone (slopes and rocking aside). Until Q-1871 is answered, the
//! view draws a vessel that the simulation marks sinking (S-0504 Terms) with
//! a list of our own:
//!
//! - **Axis and direction.** It pitches about the axis across its heading,
//!   the bow or the stern going under, with a smaller roll to one side about
//!   the axis along its heading. Which end and which side follow from the
//!   object's id by a fixed bit mix, with no random draw, so every client
//!   and every replay draws the same list.
//! - **Angle and timing.** The list grows in [`STEPS`] equal steps with the
//!   depth below the surface, from level at the surface to the full
//!   [`FULL_PITCH_DEGREES`] of pitch and [`FULL_ROLL_DEGREES`] of roll at a
//!   depth of [`FULL_DEPTH`] leptons, and holds there until the vessel
//!   leaves play (S-0504 claim 4: it leaves below a depth of 400).
//! - **Drawing only.** It reads the sinking flag and the height that the
//!   simulation already keeps; it adds no state and changes no rule. The
//!   tilt rides in the body transform, so the turret and barrel follow it
//!   and the shadow is made from the same pose.
//!
//! [`list`] is the one switch point: the spec's answer replaces its body.

use glasnost_render::voxel::{Mat34, VoxelQuirks};

/// The number of equal steps from level to the full list (design: Q-1871
/// interim). Each step is its own set of images, so the steps are few.
pub const STEPS: u8 = 12; // design: Q-1871 interim
/// The depth below the surface, in leptons, at which the list is full
/// (design: Q-1871 interim): 48 of S-0504 claim 4's 5-lepton descents, a
/// little over half the way down to its removal below 400.
pub const FULL_DEPTH: i32 = 240; // design: Q-1871 interim
/// The full pitch about the axis across the heading, degrees (design:
/// Q-1871 interim).
pub const FULL_PITCH_DEGREES: i32 = 24; // design: Q-1871 interim
/// The full roll about the axis along the heading, degrees (design: Q-1871
/// interim).
pub const FULL_ROLL_DEGREES: i32 = 8; // design: Q-1871 interim
/// The multiplier of the id's bit mix (design: Q-1871 interim; the 32-bit
/// golden-ratio constant of Fibonacci hashing, so that neighbouring ids do
/// not all list alike).
const MIX: u32 = 0x9E37_79B9; // design: Q-1871 interim

/// How a sinking vessel's body is listed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct List {
    /// How far the list has gone: 0 (level) to [`STEPS`] (full).
    pub step: u8,
    /// The bow goes under (else the stern).
    pub bow_down: bool,
    /// The starboard side (the right, looking forward) goes under (else
    /// the port side).
    pub starboard_down: bool,
}

/// The list of a vessel the simulation marks sinking (design: Q-1871
/// interim), from its height above the surface in leptons (negative below
/// it; S-0504 claim 4's height) and its object id.
pub fn list(height: i32, id: u32) -> List {
    // design: Q-1871 interim (the depth below the surface; level above it, full from FULL_DEPTH on)
    let depth = (-height).clamp(0, FULL_DEPTH);
    let step = depth * i32::from(STEPS) / FULL_DEPTH;
    // design: Q-1871 interim (the end and the side from the id's two top mixed bits)
    let mixed = id.wrapping_mul(MIX);
    List {
        step: step as u8,
        bow_down: mixed & (1 << 31) != 0,
        starboard_down: mixed & (1 << 30) != 0,
    }
}

impl List {
    /// Whether it is drawn level (step 0).
    pub fn level(self) -> bool {
        self.step == 0
    }

    /// The pitch in radians, positive lowering the bow (as S-0105 claim
    /// 42's drawing pitch).
    pub fn pitch(self) -> f64 {
        let a = angle(FULL_PITCH_DEGREES, self.step);
        if self.bow_down { a } else { -a }
    }

    /// The roll in radians about the forward axis, positive lowering the
    /// starboard side.
    pub fn roll(self) -> f64 {
        let a = angle(FULL_ROLL_DEGREES, self.step);
        if self.starboard_down { a } else { -a }
    }

    /// The body-frame rotation *Ry(pitch) · Rx(roll)* that follows the
    /// body's heading in its transform: *D = Rz(θ_body) · Ry · Rx*, the
    /// order of S-0105 claim 42's flying body.
    pub fn rotation(self, quirks: &VoxelQuirks) -> Mat34 {
        Mat34::IDENTITY
            .rotate_y(self.pitch(), quirks)
            .rotate_x(self.roll(), quirks)
    }
}

/// `step` of [`STEPS`] of `full` degrees, in radians.
fn angle(full: i32, step: u8) -> f64 {
    f64::from(full) * f64::from(step) / f64::from(STEPS) * std::f64::consts::PI / 180.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn degrees(r: f64) -> f64 {
        r * 180.0 / std::f64::consts::PI
    }

    // design: Q-1871 interim (level at the surface, growing with the depth, full from FULL_DEPTH to the removal depth)
    #[test]
    fn the_list_grows_with_the_depth_and_holds_at_full() {
        let id = 7;
        // At the start of the descent: level.
        assert!(list(0, id).level());
        assert_eq!(list(0, id).pitch(), 0.0);
        assert_eq!(list(0, id).roll(), 0.0);
        // Above the surface (a vessel sinking from a height): level.
        assert!(list(120, id).level());
        // The first step after FULL_DEPTH / STEPS leptons.
        assert!(list(-19, id).level());
        assert_eq!(list(-20, id).step, 1);
        // Partway: half the full angle at half FULL_DEPTH.
        let half = list(-FULL_DEPTH / 2, id);
        assert_eq!(half.step, STEPS / 2);
        assert!((degrees(half.pitch().abs()) - 12.0).abs() < 1e-9);
        assert!((degrees(half.roll().abs()) - 4.0).abs() < 1e-9);
        // Full at FULL_DEPTH, and still full at the end of the descent.
        for h in [-FULL_DEPTH, -300, -400, -404] {
            let l = list(h, id);
            assert_eq!(l.step, STEPS);
            assert!((degrees(l.pitch().abs()) - 24.0).abs() < 1e-9);
            assert!((degrees(l.roll().abs()) - 8.0).abs() < 1e-9);
        }
        // Monotonic over the descent.
        let steps: Vec<u8> = (0..=81).map(|k| list(-5 * k, id).step).collect();
        assert!(steps.windows(2).all(|p| p[0] <= p[1]), "{steps:?}");
    }

    // design: Q-1871 interim (the end and side depend on the id alone: the same at every depth, every call)
    #[test]
    fn the_direction_is_stable_for_one_id_and_both_ways_occur() {
        for id in 0..64 {
            let first = list(-5, id);
            for h in (-404..=0).step_by(5) {
                let l = list(h, id);
                assert_eq!(
                    (l.bow_down, l.starboard_down),
                    (first.bow_down, first.starboard_down),
                    "id {id} at height {h}"
                );
            }
            // The sign of the angles follows the direction.
            let full = list(-400, id);
            assert_eq!(full.pitch() > 0.0, full.bow_down);
            assert_eq!(full.roll() > 0.0, full.starboard_down);
        }
        let ways: std::collections::BTreeSet<(bool, bool)> = (0..64)
            .map(|id| {
                let l = list(-400, id);
                (l.bow_down, l.starboard_down)
            })
            .collect();
        assert_eq!(ways.len(), 4, "all four ways occur among 64 ids");
    }

    // design: Q-1871 interim (a positive pitch lowers the bow: the forward axis points below the horizon)
    #[test]
    fn a_bow_down_list_lowers_the_forward_axis() {
        let q = VoxelQuirks::default();
        let mut l = list(-400, 0);
        l.bow_down = true;
        l.starboard_down = true;
        let r = l.rotation(&q);
        let forward = r.apply([1.0, 0.0, 0.0]);
        assert!(forward[2] < -0.3, "{forward:?}");
        // The port side (+y, as Rz maps +x toward +y) rises.
        let port = r.apply([0.0, 1.0, 0.0]);
        assert!(port[2] > 0.1, "{port:?}");
        let mut stern = l;
        stern.bow_down = false;
        assert!(stern.rotation(&q).apply([1.0, 0.0, 0.0])[2] > 0.3);
    }
}
