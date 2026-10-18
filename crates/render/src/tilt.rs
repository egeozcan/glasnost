//! Voxel body tilt during a slope-shape transition (S-0106 claims 6 and 7,
//! F3, table C3, State and ordering): the tilt state of a unit with a
//! tilting locomotor as a pure state machine, and the interpolated tilt
//! orientation and its matrix for the drawn frames of a transition.
//!
//! What is here: the tilt orientations *q_s* by C3's construction rule with
//! claim 4's corrected angles (and C3's original rows for oracle
//! comparisons), the spherical
//! interpolation of F3 with its table arc cosine, the quaternion's matrix,
//! and [`TiltState`] with S-0106's continuous restart (claim 6, a
//! lead-confirmed departure with no switch).
//!
//! **Not modelled.** The settled tilt transforms *R_s* (C4, claim 8), which
//! locomotors keep a tilt state and when they observe a cell (claim 6's
//! ground-layer movement ticks), rocking, and the drawing of tilted bodies:
//! poses in [`crate::voxel`] stay on flat ground. Every floating-point step
//! follows S-0105 F0 through [`crate::rz`].

use std::f64::consts::FRAC_PI_2;

use crate::rz::{add_rz, div_rz, mul_rz, sub_rz, to_f32_rz};
use crate::voxel::{Mat34, VoxelQuirks, table_sin_cos, table_sqrt};

/// A tilt orientation quaternion *(x, y, z, w)*, single precision (S-0106
/// Terms).
pub type Quat = [f32; 4];

/// Length of a tilt transition, ticks (S-0106 claims 6, 7).
pub const TRANSITION_TICKS: u64 = 3; // spec: S-0106 claims 6, 7, Constants (transition length)

/// Linear-interpolation threshold of F3 (S-0106 F3, Constants).
const LINEAR_THRESHOLD: f64 = 1e-5; // spec: S-0106 F3, Constants (interpolation linear threshold)

/// Arc-sine table resolution of F3's table arc cosine: entries per unit
/// of argument (S-0106 F3).
const ACOS_STEPS: f64 = 2048.0; // spec: S-0106 F3 (S(m) for m/2048 − 1)

/// Last entry of F3's arc-sine table (S-0106 F3: m = 0 … 4096).
const ACOS_LAST: i64 = 4096; // spec: S-0106 F3 (4097 fixed values)

/// The original tilt orientation of slope shape `shape`: table C3's
/// numeric rows (S-0106), kept as oracle references only. Shapes 0 and
/// 17 … 20, and any shape outside 0 … 20, have the identity.
pub fn original_orientation(shape: u8) -> Quat {
    // spec: S-0106 Constants table C3 (original tilt orientations, oracle references only)
    const E: f32 = 0.253_382; // shapes 1 … 4
    const W1: f32 = 0.967_366_3; // shapes 1 … 4 and 13 … 16
    const C: f32 = 0.131_031_9; // shapes 5 … 12
    const W2: f32 = 0.982_681_7; // shapes 5 … 12
    const S: f32 = 0.179_173_6; // shapes 13 … 16
    match shape {
        1 => [0.0, -E, 0.0, W1],
        2 => [-E, 0.0, 0.0, W1],
        3 => [0.0, E, 0.0, W1],
        4 => [E, 0.0, 0.0, W1],
        5 | 9 => [-C, -C, 0.0, W2],
        6 | 10 => [-C, C, 0.0, W2],
        7 | 11 => [C, C, 0.0, W2],
        8 | 12 => [C, -C, 0.0, W2],
        13 => [-S, -S, 0.0, W1],
        14 => [-S, S, 0.0, W1],
        15 => [S, S, 0.0, W1],
        16 => [S, -S, 0.0, W1],
        _ => [0.0, 0.0, 0.0, 1.0],
    }
}

/// The tilt angle of the original for shapes 1 … 4 and 13 … 16, *τ_A*
/// (S-0106 F2), for oracle comparisons.
#[allow(clippy::excessive_precision)] // the digits as S-0106 F2 gives them
pub const TAU_A: f64 = 0.511_634_767_055_511_47; // spec: S-0106 F2 (τ_A = A₂₃)

/// The tilt angle of the original for shapes 5 … 12, *τ_B* (S-0106 F2), for
/// oracle comparisons.
#[allow(clippy::excessive_precision)] // the digits as S-0106 F2 gives them
pub const TAU_B: f64 = 0.372_388_541_698_455_81; // spec: S-0106 F2 (τ_B = A₁₆)

/// C3's construction rule (S-0106): the rotation about the horizontal axis
/// (cos *α_s*, sin *α_s*, 0) of F2 by the shape's angle, `edge` for shapes
/// 1 … 4, `corner` for 5 … 12 and `steep` for 13 … 16. The axis is
/// normalised with the table square root, the half angle's sine and cosine
/// are table trigonometry (S-0105 F1), single-precision stores truncated
/// toward zero (S-0105 F0).
pub fn construct(shape: u8, (edge, corner, steep): (f64, f64, f64), quirks: &VoxelQuirks) -> Quat {
    use std::f64::consts::PI;
    // spec: S-0106 F2 (α_s and τ_s by shape; identity for 0 and 17 … 20)
    let (alpha, tau) = match shape {
        1 => (3.0 * PI / 2.0, edge),
        2 => (PI, edge),
        3 => (PI / 2.0, edge),
        4 => (0.0, edge),
        5 | 9 => (5.0 * PI / 4.0, corner),
        6 | 10 => (3.0 * PI / 4.0, corner),
        7 | 11 => (PI / 4.0, corner),
        8 | 12 => (7.0 * PI / 4.0, corner),
        13 => (5.0 * PI / 4.0, steep),
        14 => (3.0 * PI / 4.0, steep),
        15 => (PI / 4.0, steep),
        16 => (7.0 * PI / 4.0, steep),
        _ => return [0.0, 0.0, 0.0, 1.0],
    };
    // spec: S-0106 C3 construction rule (axis (cos α, sin α, 0) by table trigonometry, normalised with the table square root)
    let (sa, ca) = table_sin_cos(alpha, quirks);
    let (ax, ay) = (f64::from(ca), f64::from(sa));
    let len = f64::from(table_sqrt(add_rz(mul_rz(ax, ax), mul_rz(ay, ay))));
    // spec: S-0106 C3 construction rule ((x, y, z) = sin(τ/2) × the normalised axis, w = cos(τ/2), table trigonometry)
    let (sh, ch) = table_sin_cos(div_rz(tau, 2.0), quirks);
    let k = f64::from(sh);
    [
        to_f32_rz(mul_rz(k, div_rz(ax, len))),
        to_f32_rz(mul_rz(k, div_rz(ay, len))),
        0.0,
        ch,
    ]
}

/// The corrected tilt angles of S-0106 claim 4 and F2: arctan(1/√6) for
/// shapes 1 … 4, π/6 for shapes 5 … 16, in double precision.
pub fn corrected_angles() -> (f64, f64, f64) {
    // spec: S-0106 claim 4 and F2 (corrected angles: arctan(1/√6) for edge slopes, π/6 for corner, valley and steep slopes)
    let edge = (1.0 / 6.0_f64.sqrt()).atan();
    let other = std::f64::consts::PI / 6.0;
    (edge, other, other)
}

/// The tilt orientation *q_s* our engine draws for slope shape `shape`:
/// C3's construction rule with claim 4's corrected angles (S-0106 F3).
pub fn tilt_orientation(shape: u8, quirks: &VoxelQuirks) -> Quat {
    // spec: S-0106 F3 and C3 (corrected orientations: the construction rule with the corrected angles, no switch)
    construct(shape, corrected_angles(), quirks)
}

/// F3's table arc cosine: *π/2 − S(⌊2048(x + 1)⌋)*, where *S(m)* is the
/// arc sine of *m/2048 − 1* truncated to single precision (S-0106 F3).
/// The index is clamped to the table (F3 holds only for *x + 1 > 10⁻⁵*).
fn acos_table(x: f64) -> f64 {
    // spec: S-0106 F3 (m = ⌊2048(x + 1)⌋, 0 … 4096)
    let m = mul_rz(ACOS_STEPS, add_rz(x, 1.0))
        .floor()
        .clamp(0.0, ACOS_LAST as f64);
    // spec: S-0106 F3 (S(m): the arcsine of m/2048 − 1, truncated to single precision)
    let s = to_f32_rz(sub_rz(m / ACOS_STEPS, 1.0).asin());
    // spec: S-0106 F3 (π/2 in double precision)
    sub_rz(FRAC_PI_2, f64::from(s))
}

/// F3's interpolated orientation from `q0` to `q1` at fraction `p`, given
/// as `j / 3` by its numerator `j` (S-0106 F3). Not renormalised.
///
/// The weights *k₀, k₁* are kept at 53-bit precision; F3 stores only the
/// dot product and the result in single precision.
pub fn interpolate(q0: Quat, q1: Quat, j: u64, quirks: &VoxelQuirks) -> Quat {
    // spec: S-0106 F3 (t = p stored in single precision)
    let t = f64::from(to_f32_rz(div_rz(j as f64, TRANSITION_TICKS as f64)));
    // spec: S-0106 F3 (δ = q₀ · q₁, stored in single precision)
    let delta = f64::from(to_f32_rz((0..4).fold(0.0, |acc, i| {
        add_rz(acc, mul_rz(f64::from(q0[i]), f64::from(q1[i])))
    })));
    let (k0, k1) = if sub_rz(1.0, delta) <= LINEAR_THRESHOLD {
        // spec: S-0106 F3 ((k₀, k₁) = (1 − t, t) when 1 − δ ≤ 10⁻⁵)
        (sub_rz(1.0, t), t)
    } else {
        // spec: S-0106 F3 (θ = acos_t(δ); k₀ = sin((1 − t)θ) / sin θ, k₁ = sin(tθ) / sin θ, table sines)
        let theta = acos_table(delta);
        let sin = |x: f64| f64::from(table_sin_cos(x, quirks).0);
        let s = sin(theta);
        (
            div_rz(sin(mul_rz(sub_rz(1.0, t), theta)), s),
            div_rz(sin(mul_rz(t, theta)), s),
        )
    };
    // spec: S-0106 F3 (q = k₀ q₀ + k₁ q₁, each component stored in single precision)
    std::array::from_fn(|i| {
        to_f32_rz(add_rz(
            mul_rz(k0, f64::from(q0[i])),
            mul_rz(k1, f64::from(q1[i])),
        ))
    })
}

/// The matrix of the unit-quaternion formula for `q` as it is (S-0106 F3),
/// with no translation. `q` is not renormalised, so the matrix need not be
/// exactly a rotation.
pub fn quat_matrix(q: Quat) -> Mat34 {
    let [x, y, z, w] = q.map(f64::from);
    let m = |a: f64, b: f64| mul_rz(a, b);
    let two = |v: f64| to_f32_rz(mul_rz(2.0, v));
    let one_minus = |a: f64, b: f64| to_f32_rz(sub_rz(1.0, mul_rz(2.0, add_rz(a, b))));
    // spec: S-0106 F3 (the matrix table of the unit-quaternion formula)
    Mat34([
        [
            one_minus(m(y, y), m(z, z)),
            two(sub_rz(m(x, y), m(w, z))),
            two(add_rz(m(x, z), m(w, y))),
            0.0,
        ],
        [
            two(add_rz(m(x, y), m(w, z))),
            one_minus(m(x, x), m(z, z)),
            two(sub_rz(m(y, z), m(w, x))),
            0.0,
        ],
        [
            two(sub_rz(m(x, z), m(w, y))),
            two(add_rz(m(y, z), m(w, x))),
            one_minus(m(x, x), m(y, y)),
            0.0,
        ],
    ])
}

/// The tilt state of a unit with a tilting locomotor (S-0106 Terms,
/// claims 6 and 7, State and ordering): current and previous shape, the
/// start tick of the last transition and its start orientation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TiltState {
    current: u8,
    previous: u8,
    /// The tick *t₀* of the last transition; `None` before any.
    start_tick: Option<u64>,
    /// The orientation the last transition starts from.
    start: Quat,
}

impl TiltState {
    /// A unit placed on a cell of shape `shape`: current and previous shape
    /// both that shape, no transition running (S-0106 claim 6).
    pub fn placed(shape: u8, quirks: &VoxelQuirks) -> Self {
        // spec: S-0106 claim 6, State and ordering (placed: s_c = s_p = σ, settled)
        Self {
            current: shape,
            previous: shape,
            start_tick: None,
            start: tilt_orientation(shape, quirks),
        }
    }

    /// The current shape *s_c*.
    pub fn current(&self) -> u8 {
        self.current
    }

    /// The previous shape *s_p*.
    pub fn previous(&self) -> u8 {
        self.previous
    }

    /// The step *j* of the transition running at `tick`, if one is.
    fn step(&self, tick: u64) -> Option<u64> {
        // spec: S-0106 claims 6, 7 (a transition runs at t₀, t₀ + 1, t₀ + 2)
        let j = tick.checked_sub(self.start_tick?)?;
        (j < TRANSITION_TICKS).then_some(j)
    }

    /// Whether a transition is running at `tick`.
    pub fn in_transition(&self, tick: u64) -> bool {
        self.step(tick).is_some()
    }

    /// A ground-layer movement tick `tick` on which the unit's centre is in
    /// a cell of shape `shape` (S-0106 claim 6, State and ordering). A
    /// different shape starts a three-tick transition at this tick. One
    /// that interrupts a running transition starts from that transition's
    /// orientation at this tick (F3), so the tilt stays continuous;
    /// otherwise it starts from the old current shape's orientation.
    pub fn observe(&mut self, tick: u64, shape: u8, quirks: &VoxelQuirks) {
        if shape == self.current {
            return;
        }
        // spec: S-0106 claim 6, State and ordering (a restart runs from the interrupted
        // orientation at the tick of the change, evaluated by F3; lead-confirmed departure)
        let start = self
            .orientation(tick, quirks)
            .unwrap_or_else(|| tilt_orientation(self.current, quirks));
        // spec: S-0106 claim 6 (s_p is the former s_c, s_c the cell's shape, t₀ this tick)
        self.previous = self.current;
        self.current = shape;
        self.start_tick = Some(tick);
        self.start = start;
    }

    /// The interpolated tilt orientation drawn at `tick` (S-0106 claim 7,
    /// F3): `Some` while a transition runs, `None` once the body is settled
    /// on its current shape (claim 8's table transform applies).
    pub fn orientation(&self, tick: u64, quirks: &VoxelQuirks) -> Option<Quat> {
        let j = self.step(tick)?;
        // spec: S-0106 claim 7, F3 (from the start orientation to q_{s_c} at p = j/3)
        Some(interpolate(
            self.start,
            tilt_orientation(self.current, quirks),
            j,
            quirks,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q() -> VoxelQuirks {
        VoxelQuirks::default()
    }

    /// Tilt of the up axis, degrees: the arc cosine of the row z, column z
    /// entry (S-0106 F3, sample values).
    fn up_tilt(q: Quat) -> f64 {
        f64::from(quat_matrix(q).0[2][2]).acos().to_degrees()
    }

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 2e-3, "{a} vs {b}");
    }

    /// spec: S-0106 claims 6, 7, F3, State and ordering (a placed unit is
    /// settled; flat to shape 1 draws j = 0, 1, 2 of the way to the
    /// corrected edge orientation, then settles)
    #[test]
    fn transition_from_flat_to_an_edge_slope() {
        let mut s = TiltState::placed(0, &q());
        assert_eq!((s.current(), s.previous()), (0, 0));
        assert!(s.orientation(5, &q()).is_none());
        s.observe(10, 0, &q());
        assert!(!s.in_transition(10), "the same shape starts nothing");
        s.observe(10, 1, &q());
        assert_eq!((s.current(), s.previous()), (1, 0));
        let tilts: Vec<f64> = (10..13)
            .map(|t| up_tilt(s.orientation(t, &q()).unwrap()))
            .collect();
        let end = up_tilt(tilt_orientation(1, &q()));
        close(tilts[0], 0.0);
        assert!((tilts[1] - end / 3.0).abs() < 0.1, "{tilts:?} {end}");
        assert!((tilts[2] - 2.0 * end / 3.0).abs() < 0.1, "{tilts:?} {end}");
        // From t₀ + 3 the body is settled.
        assert!(s.orientation(13, &q()).is_none());
    }

    /// spec: S-0106 claim 7, F3 (oracle samples with C3's original rows: an
    /// edge slope's q drawn as 29.3555°; F3's sample values)
    #[test]
    fn interpolation_samples() {
        let o = original_orientation;
        close(up_tilt(o(1)), 29.3555);
        // Shape 1 to shape 4: 22.0749° at both fractions.
        for j in [1, 2] {
            close(up_tilt(interpolate(o(1), o(4), j, &q())), 22.0749);
            // Shape 5 to shape 7: 7.1054° at both fractions.
            close(up_tilt(interpolate(o(5), o(7), j, &q())), 7.1054);
        }
        // Equal orientations interpolate linearly to themselves.
        assert_eq!(interpolate(o(5), o(9), 1, &q()), o(5));
    }

    /// spec: S-0106 claim 6, claim 7, State and ordering (a shape change
    /// during a transition restarts a three-tick transition from the
    /// interrupted orientation at that tick: the tilt stays continuous, the
    /// previous shape records the old current shape; lead-confirmed
    /// departure with no switch)
    #[test]
    fn restart_is_continuous() {
        let mut s = TiltState::placed(0, &q());
        s.observe(10, 1, &q());
        let interrupted = s.orientation(11, &q()).unwrap();
        s.observe(11, 4, &q());
        assert_eq!((s.current(), s.previous()), (4, 1));
        // Drawn at the tick of the change: the interrupted orientation, not
        // shape 1's (the original's jump).
        assert_eq!(s.orientation(11, &q()), Some(interrupted));
        assert_ne!(interrupted, tilt_orientation(1, &q()));
        assert!(up_tilt(s.orientation(11, &q()).unwrap()) > 1.0);
        // Then on toward shape 4 over three ticks from the change.
        let next = s.orientation(12, &q()).unwrap();
        assert_eq!(
            next,
            interpolate(interrupted, tilt_orientation(4, &q()), 1, &q())
        );
        assert!(s.in_transition(13));
        assert!(s.orientation(14, &q()).is_none());
        // A change once settled starts from the current shape's orientation.
        s.observe(20, 0, &q());
        assert_eq!(s.orientation(20, &q()), Some(tilt_orientation(4, &q())));
    }

    /// spec: S-0106 F3 (the matrix of the identity orientation is the
    /// identity)
    #[test]
    fn identity_matrix() {
        assert_eq!(quat_matrix(tilt_orientation(0, &q())), Mat34::IDENTITY);
    }

    /// spec: S-0106 C3 (the construction rule with the original angles τ_A
    /// and τ_B reproduces C3's numeric rows)
    #[test]
    fn construction_rule_reproduces_c3() {
        for shape in 0..=20 {
            let c = construct(shape, (TAU_A, TAU_B, TAU_A), &q());
            let o = original_orientation(shape);
            for i in 0..4 {
                assert!((c[i] - o[i]).abs() < 2e-6, "shape {shape}: {c:?} vs {o:?}");
            }
        }
    }

    /// spec: S-0106 claim 4, F2 and C3 (corrected orientations: arctan(1/√6)
    /// ≈ 22.21° for edge slopes, π/6 = 30° for corner, valley and steep
    /// slopes, to the table trigonometry's resolution; flat and saddle
    /// shapes the identity)
    #[test]
    fn corrected_orientations() {
        for shape in 1..=4 {
            let d = up_tilt(tilt_orientation(shape, &q()));
            assert!((d - 22.2077).abs() < 0.1, "shape {shape}: {d}");
        }
        for shape in 5..=16 {
            let d = up_tilt(tilt_orientation(shape, &q()));
            assert!((d - 30.0).abs() < 0.1, "shape {shape}: {d}");
        }
        for shape in [0, 17, 18, 19, 20] {
            assert_eq!(tilt_orientation(shape, &q()), [0.0, 0.0, 0.0, 1.0]);
        }
    }
}
