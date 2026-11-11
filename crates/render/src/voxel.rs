//! Voxel rendering (S-0105): CPU draws of a voxel model part (body, turret
//! or barrel) or of a voxel shadow in a given pose, the depth-buffered
//! mode of fresh draws, and the image cache's keys.
//!
//! What is here:
//!
//! - 32-step and 4-step facings (Terms), table trigonometry and the table
//!   square root (F1), the view transform (claim 1, F2), the turret
//!   displacement unit (F3);
//! - part poses: vehicle body, turret and barrel ([`vehicle_parts`],
//!   claims 3, 4, 6, 7, 8, 10, 11), building turret and barrel
//!   ([`building_turret_parts`], claims 5, 9, 10, 11) and a building barrel
//!   without a turret ([`building_barrel_only`], claim 37), all on flat
//!   ground;
//! - the barrel elevation after placement ([`barrel_elevation_after`],
//!   claim 6, F9) and the HVA frames each part uses (claim 16);
//! - lighting: the light in the model frame and the shade of each normal
//!   index (claims 17, 18, 20, F4), and the VPL lookup (claim 19);
//! - rasterisation of one part into a fresh, fully transparent image with
//!   unwrapped signed coordinates ([`draw_model`], claims 13–15, 21–25, 34,
//!   35, 36, F5, F6, F8) and of a shadow footprint ([`draw_shadow`],
//!   claims 29–31, F10), and a vehicle shadow's own-image exclusion
//!   ([`exclude_own_image`], claim 39);
//! - the depth-buffered mode, on only during a fresh depth-option part draw
//!   ([`VoxelCanvas`], claims 24, 32, 33, 34, S2);
//! - the image cache's keys and reuse ([`ImageKey`], [`ImageCache`],
//!   claims 26, 31, 37);
//! - the recoil slide of a building turret or barrel as a pure state
//!   machine ([`RecoilCycle`], claim 27, F7, S1).
//!
//! Every floating-point step follows S-0105 F0 through [`crate::rz`]:
//! 53-bit intermediates rounded toward zero, single-precision stores
//! truncated toward zero, float-to-integer conversions truncated. Where the
//! spec fixes no evaluation order, sums run left to right, as each function
//! documents.
//!
//! **Not modelled.**
//!
//! - stale shades of normal indices outside the part's table (claim 20):
//!   such indices get shade 0, the value at program start;
//! - recoil placement in a pose (claims 10, 27): [`RecoilCycle`] gives the
//!   slide, but the shift (*−λ_turret* for the turret and
//!   *−(λ_turret + λ_barrel)* for the barrel, along the turret's forward
//!   axis, not tilted by elevation) is not applied to the poses here;
//! - ramp tilt and locomotor shifts (claims 7, 25, 30): left to the planned
//!   voxel tilt spec; poses here are on flat ground;
//! - alternative and multiple models (claim 12): the caller picks the
//!   model (with the alternative-body option and no alternative body, the
//!   main body, and the turret drawn as a turret).
//!
//! S-0105's lead-confirmed presentation departures (wrap, carry, partial
//! clears, depth-mode leak, reduced frame keys, barrel keys without
//! elevation, pivot mixup, recoil hold and refire, cached shadow cutouts)
//! are fixed here with no switch. So are claim 22's bounded images
//! (Q-0470): a part or shadow image keeps at most the centred
//! [`IMAGE_LIMIT`] window of its copy rectangle, and an image whose
//! placement is not finite is not drawn.

use std::collections::BTreeMap;
use std::f64::consts::PI;

use glasnost_fmt::hva::{self, Hva};
use glasnost_fmt::vpl::Vpl;
use glasnost_fmt::vxl::{Limb, Vxl};

use crate::rz::{add_rz, div_rz, dot3_rz, mul_f32_rz, mul_rz, sub_rz, to_f32_rz, trunc_i64};

// ---------------------------------------------------------------------------
// Quirk switches
// ---------------------------------------------------------------------------

/// Quirk switches of S-0105 implemented here. Each defaults to the
/// original's behaviour (S-0105 Edge cases and quirks).
///
/// Switches of S-0105 not here: `voxel_single_pixel` and
/// `voxel_no_intersection_depth` (fixed behaviour not implemented), and
/// the switches of claims 28 and 37 that concern recoil placement
/// (`vehicle_no_recoil`, `barrel_only_no_recoil`). The bugs S-0105 lists
/// as lead-confirmed presentation departures have no switch: their
/// corrected rule is the only behaviour here. `voxel_stale_normal_shades` is here only for its fixed
/// value. The round-toward-zero switch belongs to the planned number
/// domains spec, not to S-0105.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VoxelQuirks {
    /// `quirks.table_trig_negative_bias` (F1): negative angles whose
    /// fractional table step is at least one half land two steps short.
    /// Off: nearest table step, halves up, wrapping at a whole turn.
    pub table_trig_negative_bias: bool,
    /// `quirks.hva_scale_first_section` (claim 14): HVA translations are
    /// scaled by the first section's scale. Off: each section's own.
    pub hva_scale_first_section: bool,
    /// `quirks.voxel_light_ignores_hva` (claim 18): the light is brought
    /// into the model frame without the section's HVA rotation. Off: with
    /// it, per section.
    pub voxel_light_ignores_hva: bool,
    /// `quirks.voxel_eye_in_model_frame` (claim 18): the eye vector is
    /// (0, 0, 1) in the model frame. Off: *Rᵀ (0, 0, 1)*.
    pub voxel_eye_in_model_frame: bool,
    /// `quirks.voxel_light_first_section_mode` (claim 18): one shade table
    /// from the first section's normal mode. Off: per-section tables.
    pub voxel_light_first_section_mode: bool,
    /// `quirks.voxel_stale_normal_shades` (claim 20): normal indices beyond
    /// the table and below 253 show a stale shade (here always shade 0, the
    /// program-start value; stale state is not modelled). Off: shade 16.
    pub voxel_stale_normal_shades: bool,
    /// `quirks.voxel_section_sort_ties` (claim 23, F8): sections with equal
    /// nearest depth need not keep file order; F8's record-low relation
    /// gives the overlap priority of every sequence. Off: stable order.
    pub voxel_section_sort_ties: bool,
    /// `quirks.voxel_depth_double_width` (claim 24): depth-buffered voxels
    /// are two pixels wide. Off: one pixel.
    pub voxel_depth_double_width: bool,
    /// `quirks.voxel_mode0_misread` (claim 35): normal mode 0 reads one
    /// value per voxel. Off: colour and normal per voxel, colour unlit.
    pub voxel_mode0_misread: bool,
    /// `quirks.voxel_topdown_column_origin` (claim 36): top-down columns are
    /// positioned from the grid's top and every column stops at the first
    /// pair boundary congruent to *Nz* modulo 256 (modulo 2³² in the
    /// depth-buffered mode). Off: every voxel at the
    /// cell its runs give, cells 0 … *Nz* − 1 only.
    pub voxel_topdown_column_origin: bool,
    /// `quirks.shadow_double_width` (claim 30, F10): each shadow column
    /// marks two pixels. Off: one.
    pub shadow_double_width: bool,
    /// `quirks.shadow_frame_zero` (claim 29): shadows use HVA frame 0. Off:
    /// the body's current frame.
    pub shadow_frame_zero: bool,
    /// `quirks.building_turret_shift_eighths` (claim 9): a building turret
    /// moves by trunc(δ / 8) model units. Off: δ × ρ as for vehicles (F3).
    pub building_turret_shift_eighths: bool,
    /// `quirks.barrel_elevation_quantised` (claim 6): the barrel pitch
    /// shows only 32-step values. Off: continuous pitch from the raw
    /// elevation.
    pub barrel_elevation_quantised: bool,
    /// `quirks.barrel_frame_zero` (claim 16): barrels drawn with a voxel
    /// turret use HVA frame 0. Off: the voxel turret's frame. A building
    /// barrel drawn without a turret is not affected (claim 37), and the
    /// barrel of a sprite-body vehicle stays in frame 0 in both states
    /// (claim 12, *Sprite body*).
    pub barrel_frame_zero: bool,
    /// `quirks.vpl_unloaded_rows_blank` (claim 38): rows the lighting table
    /// file does not supply read as zero and a missing or truncated file
    /// does not stop the game. Off: the importer rejects such a file.
    pub vpl_unloaded_rows_blank: bool,
}

impl Default for VoxelQuirks {
    fn default() -> Self {
        // spec: S-0105 Edge cases and quirks (every switch defaults to the original)
        Self {
            table_trig_negative_bias: true,
            hva_scale_first_section: true,
            voxel_light_ignores_hva: true,
            voxel_eye_in_model_frame: true,
            voxel_light_first_section_mode: true,
            voxel_stale_normal_shades: true,
            voxel_section_sort_ties: true,
            voxel_depth_double_width: true,
            voxel_mode0_misread: true,
            voxel_topdown_column_origin: true,
            shadow_double_width: true,
            shadow_frame_zero: true,
            building_turret_shift_eighths: true,
            barrel_elevation_quantised: true,
            barrel_frame_zero: true,
            vpl_unloaded_rows_blank: true,
        }
    }
}

impl VoxelQuirks {
    /// The switches by name.
    pub fn named(&self) -> [(&'static str, bool); 16] {
        [
            ("table_trig_negative_bias", self.table_trig_negative_bias),
            ("hva_scale_first_section", self.hva_scale_first_section),
            ("voxel_light_ignores_hva", self.voxel_light_ignores_hva),
            ("voxel_eye_in_model_frame", self.voxel_eye_in_model_frame),
            (
                "voxel_light_first_section_mode",
                self.voxel_light_first_section_mode,
            ),
            ("voxel_stale_normal_shades", self.voxel_stale_normal_shades),
            ("voxel_section_sort_ties", self.voxel_section_sort_ties),
            ("voxel_depth_double_width", self.voxel_depth_double_width),
            ("voxel_mode0_misread", self.voxel_mode0_misread),
            (
                "voxel_topdown_column_origin",
                self.voxel_topdown_column_origin,
            ),
            ("shadow_double_width", self.shadow_double_width),
            ("shadow_frame_zero", self.shadow_frame_zero),
            (
                "building_turret_shift_eighths",
                self.building_turret_shift_eighths,
            ),
            (
                "barrel_elevation_quantised",
                self.barrel_elevation_quantised,
            ),
            ("barrel_frame_zero", self.barrel_frame_zero),
            ("vpl_unloaded_rows_blank", self.vpl_unloaded_rows_blank),
        ]
    }
}

// ---------------------------------------------------------------------------
// Facings (Terms)
// ---------------------------------------------------------------------------

/// The 32-step facing of a raw facing: ⌊(⌊r / 1024⌋ + 1) / 2⌋ mod 32
/// (S-0105 Terms).
#[allow(clippy::manual_div_ceil)] // written as the spec's formula
pub fn facing32(raw: u16) -> u8 {
    // spec: S-0105 Terms (32-step facing)
    (((raw / 1024 + 1) / 2) % 32) as u8
}

/// The 4-step facing of a raw facing: ⌊(⌊r / 8192⌋ + 1) / 2⌋ mod 4
/// (S-0105 Terms).
#[allow(clippy::manual_div_ceil)] // written as the spec's formula
pub fn facing4(raw: u16) -> u8 {
    // spec: S-0105 Terms (4-step facing)
    (((raw / 8192 + 1) / 2) % 4) as u8
}

/// The 32-step facing at which a part is drawn unrotated: east (S-0105
/// claim 2, Constants).
pub const UNROTATED_FACING32: i32 = 8; // spec: S-0105 claim 2, Constants (unrotated body facing)

/// The facing step, π/16 radians (S-0105 Constants).
const FACING_STEP: f64 = PI / 16.0; // spec: S-0105 Constants (facing step)

/// The angle (radians, double) of `steps` facing steps: `steps × π/16`,
/// formed at 53-bit precision rounded toward zero (F0).
fn facing_angle(steps: i32) -> f64 {
    mul_rz(FACING_STEP, f64::from(steps))
}

/// Body rotation angle *θ_body = −(π/16)·(f₃₂ − 8)* (S-0105 claim 3); also
/// the building turret angle of claim 5.
pub fn body_angle(raw_facing: u16) -> f64 {
    // spec: S-0105 claim 3, claim 5
    facing_angle(-(i32::from(facing32(raw_facing)) - UNROTATED_FACING32))
}

/// Vehicle turret angle relative to the body, *−(π/16)·(t₃₂ − b₃₂)*
/// (S-0105 claim 4).
pub fn turret_relative_angle(turret_facing: u16, body_facing: u16) -> f64 {
    // spec: S-0105 claim 4
    facing_angle(-(i32::from(facing32(turret_facing)) - i32::from(facing32(body_facing))))
}

/// Barrel pitch *φ = +(π/16)·(e₃₂ − 8)* (S-0105 claim 6). With
/// `quirks.barrel_elevation_quantised` off, the pitch follows the raw
/// elevation: *π · (e − 16384) / 32768*.
pub fn barrel_pitch(elevation_raw: u16, quirks: &VoxelQuirks) -> f64 {
    if quirks.barrel_elevation_quantised {
        // spec: S-0105 claim 6
        facing_angle(i32::from(facing32(elevation_raw)) - UNROTATED_FACING32)
    } else {
        // spec: S-0105 Edge cases (barrel_elevation_quantised off: continuous pitch)
        mul_rz(
            PI / 32768.0,
            f64::from(i32::from(elevation_raw) - i32::from(ELEVATION_LEVEL)),
        )
    }
}

// ---------------------------------------------------------------------------
// Table trigonometry and square root (F1)
// ---------------------------------------------------------------------------

/// Half table steps per radian, 16384 / 2π in single precision (S-0105 F1).
pub const HALF_STEPS_PER_RADIAN: f64 = 2607.594482421875; // spec: S-0105 Formulas F1

/// Sine table steps per turn (S-0105 Constants).
pub const TABLE_STEPS: i64 = 8192; // spec: S-0105 Constants (sine table)

/// *τ(4096)*, where the exact sine is 0 (S-0105 F1).
#[allow(clippy::excessive_precision)] // the spec's decimals, which identify the single exactly
const TAU_4096: f32 = 1.2246063538223773e-16; // spec: S-0105 Formulas F1 (τ(4096))
/// *τ(8192)*, where the exact sine is 0 (S-0105 F1).
#[allow(clippy::excessive_precision)] // the spec's decimals, which identify the single exactly
const TAU_8192: f32 = -2.4492127076447545e-16; // spec: S-0105 Formulas F1 (τ(8192))

/// The sine table *τ(n)*, 0 ≤ n ≤ 10239: the exact *sin(2πn / 8192)*
/// truncated toward zero to single precision, except τ(4096) and τ(8192)
/// (S-0105 F1). F1 notes that truncating a double-precision sine gives the
/// same table; the sine is taken on the first quadrant for accuracy.
pub fn tau(n: u32) -> f32 {
    match n {
        4096 => TAU_4096,
        8192 => TAU_8192,
        _ => {
            let k = n % 8192;
            let (m, negative) = match k {
                0..=2048 => (k, false),
                2049..=4096 => (4096 - k, false),
                4097..=6144 => (k - 4096, true),
                _ => (8192 - k, true),
            };
            let s = match m {
                0 => 0.0,
                2048 => 1.0,
                _ => (f64::from(m) * (PI / 4096.0)).sin(),
            };
            // spec: S-0105 Formulas F1 (truncated toward zero to single precision)
            let t = to_f32_rz(s);
            if negative { -t } else { t }
        }
    }
}

/// The table step *σ* (0 … 8191) of an angle `x` in radians (S-0105 F1):
/// *I = trunc(x × 2607.594482421875)*, *σ = min((trunc(I/2) mod 8192) +
/// (|I| mod 2), 8191)*.
pub fn table_step(x: f64, quirks: &VoxelQuirks) -> u32 {
    if quirks.table_trig_negative_bias {
        // spec: S-0105 Formulas F1
        let i = trunc_i64(mul_rz(x, HALF_STEPS_PER_RADIAN));
        let s = (i / 2).rem_euclid(TABLE_STEPS) + (i % 2).abs();
        s.min(TABLE_STEPS - 1) as u32
    } else {
        // spec: S-0105 Edge cases (table_trig_negative_bias off: exact rounding)
        let steps = x * (TABLE_STEPS as f64) / (2.0 * PI);
        ((steps + 0.5).floor() as i64).rem_euclid(TABLE_STEPS) as u32
    }
}

/// Table sine and cosine of `x` (S-0105 F1): *τ(σ)*, *τ(σ + 2048)*.
pub fn table_sin_cos(x: f64, quirks: &VoxelQuirks) -> (f32, f32) {
    let s = table_step(x, quirks);
    // spec: S-0105 Formulas F1 (sin x = τ(σ), cos x = τ(σ + 2048))
    (tau(s), tau(s + 2048))
}

/// The table square root *√ₜ(x)* (S-0105 F1): *x* stored in single
/// precision truncated toward zero, its significand cut to 14 binary
/// digits, the exact root of that truncated to single precision. Never
/// larger than the exact root. NaN and infinity pass through.
pub fn table_sqrt(x: f64) -> f32 {
    let xh = to_f32_rz(x).abs();
    if xh == 0.0 {
        return 0.0;
    }
    if !xh.is_finite() {
        return xh;
    }
    // |x̂| = m × 4^ζ with 1 ≤ m < 4.
    let v = f64::from(xh);
    let e2 = ((v.to_bits() >> 52) & 0x7FF) as i32 - 1023;
    let zeta = e2.div_euclid(2);
    let m = v * 2f64.powi(-2 * zeta);
    // spec: S-0105 Formulas F1 (m′: 14 significant binary digits, truncated)
    let unit = if m < 2.0 { 8192.0 } else { 4096.0 };
    let m_cut = (m * unit).floor() / unit;
    // The exact root of m′ truncated to single precision: the nearest
    // double root, truncated, then one step down if its square (exact for
    // a single) still exceeds m′.
    let mut r = to_f32_rz(m_cut.sqrt());
    if f64::from(r) * f64::from(r) > m_cut {
        r = f32::from_bits(r.to_bits() - 1);
    }
    // spec: S-0105 Formulas F1 (2^ζ × √m′, exact scaling)
    (f64::from(r) * 2f64.powi(zeta)) as f32
}

// ---------------------------------------------------------------------------
// Transforms (F2, F3)
// ---------------------------------------------------------------------------

/// A 3 × 4 transform, row-major, single precision: rotation in columns
/// 0–2, translation in column 3.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat34(pub [[f32; 4]; 3]);

impl Mat34 {
    /// Whether every entry is finite (S-0105 claim 22: a non-finite
    /// placement is not drawn).
    pub fn is_finite(&self) -> bool {
        self.0.iter().flatten().all(|v| v.is_finite())
    }

    /// The identity.
    pub const IDENTITY: Self = Self([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
    ]);

    /// An HVA transform as stored.
    pub fn from_hva(m: &hva::Matrix) -> Self {
        Self(m.map(|row| row.map(|v| v.get())))
    }

    /// `self · rhs` (the new step acts in the current local frame, F2).
    /// Each entry is ((a₀b₀ + a₁b₁) + a₂b₂) [+ a₃], at 53-bit precision
    /// rounded toward zero, stored truncated to single precision (F0).
    pub fn then(&self, rhs: &Self) -> Self {
        let (a, b) = (&self.0, &rhs.0);
        let mut out = [[0.0f32; 4]; 3];
        for (i, row) in out.iter_mut().enumerate() {
            for (j, v) in row.iter_mut().enumerate() {
                let p = |k: usize| mul_rz(f64::from(a[i][k]), f64::from(b[k][j]));
                let mut s = add_rz(add_rz(p(0), p(1)), p(2));
                if j == 3 {
                    s = add_rz(s, f64::from(a[i][3]));
                }
                *v = to_f32_rz(s);
            }
        }
        Self(out)
    }

    /// `self · Rx(x)` with table trigonometry (F1, F2).
    pub fn rotate_x(&self, x: f64, quirks: &VoxelQuirks) -> Self {
        let (s, c) = table_sin_cos(x, quirks);
        // spec: S-0105 Formulas F2 (right-handed rotations)
        self.then(&Self([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, c, -s, 0.0],
            [0.0, s, c, 0.0],
        ]))
    }

    /// `self · Ry(x)`: rows (cos, 0, sin), (0, 1, 0), (−sin, 0, cos) (F2).
    pub fn rotate_y(&self, x: f64, quirks: &VoxelQuirks) -> Self {
        let (s, c) = table_sin_cos(x, quirks);
        // spec: S-0105 Formulas F2 (Ry rows)
        self.then(&Self([
            [c, 0.0, s, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [-s, 0.0, c, 0.0],
        ]))
    }

    /// `self · Rz(x)`; Rz maps +x toward +y for positive x (F2).
    pub fn rotate_z(&self, x: f64, quirks: &VoxelQuirks) -> Self {
        let (s, c) = table_sin_cos(x, quirks);
        // spec: S-0105 Formulas F2 (Rz maps +x toward +y)
        self.then(&Self([
            [c, -s, 0.0, 0.0],
            [s, c, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
        ]))
    }

    /// `self · Tr(v)`, a translation in the current local frame (F2).
    pub fn translate(&self, v: [f32; 3]) -> Self {
        self.then(&Self([
            [1.0, 0.0, 0.0, v[0]],
            [0.0, 1.0, 0.0, v[1]],
            [0.0, 0.0, 1.0, v[2]],
        ]))
    }

    /// The translation column.
    pub fn translation(&self) -> [f32; 3] {
        [self.0[0][3], self.0[1][3], self.0[2][3]]
    }

    /// The transformed point, each component ((r₀p₀ + r₁p₁) + r₂p₂) + t at
    /// 53-bit precision rounded toward zero, stored truncated (F0).
    pub fn apply(&self, p: [f32; 3]) -> [f32; 3] {
        self.0.map(|row| {
            let s = dot3_rz([row[0], row[1], row[2]], p);
            to_f32_rz(add_rz(s, f64::from(row[3])))
        })
    }

    /// *Rᵀ v* for the rotation part *R*, stored truncated (F0).
    pub fn transpose_apply(&self, v: [f32; 3]) -> [f32; 3] {
        let m = &self.0;
        [0, 1, 2].map(|j| to_f32_rz(dot3_rz([m[0][j], m[1][j], m[2][j]], v)))
    }
}

/// A debris piece's orientation (S-0317 claim 22, F10): the right-handed
/// turn by `angle` radians about the direction of `axis`, in the model's
/// frame. `None` when the axis is zero and has no direction (S-0317 F5:
/// our engine then draws the piece without spin). The matrix is formed in
/// double precision and stored in single precision; S-0317 F10 fixes only
/// the turn, to the precision of captured frames (design).
pub fn piece_rotation(axis: [i32; 3], angle: f64) -> Option<Mat34> {
    let a = axis.map(f64::from);
    let len = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
    if len == 0.0 {
        return None;
    }
    // spec: S-0317 F5 (the axis normalised to unit length)
    let k = a.map(|c| c / len);
    let (s, c) = angle.sin_cos();
    let t = 1.0 - c;
    // Rodrigues' rotation formula: c I + s [k]× + (1 − c) k kᵀ.
    let r = |i: usize, j: usize| -> f32 {
        let cross = match (i, j) {
            (0, 1) => -k[2],
            (0, 2) => k[1],
            (1, 0) => k[2],
            (1, 2) => -k[0],
            (2, 0) => -k[1],
            (2, 1) => k[0],
            _ => 0.0,
        };
        let id = if i == j { c } else { 0.0 };
        (id + s * cross + t * k[i] * k[j]) as f32
    };
    Some(Mat34([
        [r(0, 0), r(0, 1), r(0, 2), 0.0],
        [r(1, 0), r(1, 1), r(1, 2), 0.0],
        [r(2, 0), r(2, 1), r(2, 2), 0.0],
    ]))
}

/// The view tilt, −60° about view x, in radians (S-0105 claim 1, F2).
fn view_tilt() -> f64 {
    // spec: S-0105 claim 1, Constants (view tilt 60 degrees)
    -div_rz(PI, 3.0)
}

/// The view turn, −45° about view z, in radians (S-0105 claim 1, F2).
fn view_turn() -> f64 {
    // spec: S-0105 claim 1, Constants (view turn 45 degrees)
    -(PI / 4.0)
}

/// The view transform *V = Rx(−60°) · Rz(−45°)* with table trigonometry
/// (S-0105 claim 1, F2).
pub fn view(quirks: &VoxelQuirks) -> Mat34 {
    // spec: S-0105 claim 1, Formulas F2
    Mat34::IDENTITY
        .rotate_x(view_tilt(), quirks)
        .rotate_z(view_turn(), quirks)
}

/// Leptons per model unit at which the turret displacement unit is fixed:
/// the table square root of 2 × 256² (S-0105 F3).
const RHO_LEPTONS_SQUARED: f64 = 2.0 * 256.0 * 256.0; // spec: S-0105 Formulas F3
/// Model units in the turret displacement unit's numerator (S-0105 F3).
const RHO_MODEL_UNITS: f64 = 60.0; // spec: S-0105 Formulas F3

/// Pixels per lepton *ρ = 60 / √ₜ(2 × 256²)*, at double precision (S-0105
/// F3).
pub fn pixels_per_lepton() -> f64 {
    // spec: S-0105 Formulas F3
    div_rz(RHO_MODEL_UNITS, f64::from(table_sqrt(RHO_LEPTONS_SQUARED)))
}

/// Vehicle turret shift *d = δ × ρ*, stored truncated (S-0105 claim 8, F3).
pub fn vehicle_turret_shift(offset_leptons: i32) -> f32 {
    // spec: S-0105 claim 8, Formulas F3
    to_f32_rz(mul_rz(f64::from(offset_leptons), pixels_per_lepton()))
}

/// Building turret shift *d_b = trunc(δ / 8)* model units (S-0105 claim
/// 9), or *δ × ρ* with `quirks.building_turret_shift_eighths` off.
pub fn building_turret_shift(offset_leptons: i32, quirks: &VoxelQuirks) -> f32 {
    if quirks.building_turret_shift_eighths {
        // spec: S-0105 claim 9, Constants (building turret shift divisor 8)
        (offset_leptons / 8) as f32
    } else {
        // spec: S-0105 Edge cases (building_turret_shift_eighths off)
        vehicle_turret_shift(offset_leptons)
    }
}

// ---------------------------------------------------------------------------
// Part poses (claims 3–11, 37)
// ---------------------------------------------------------------------------

/// Draw transforms (view included) of a vehicle's parts on flat ground.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehiclePose {
    /// *V · D* (claim 7).
    pub body: Mat34,
    /// *V · T* (claim 8).
    pub turret: Mat34,
    /// *V · B* (claim 10).
    pub barrel: Mat34,
    /// *V · S*, the shadow transform before the shadow section's HVA
    /// transform (claim 30; on flat ground the body's 32-step rotation).
    pub shadow: Mat34,
    /// Whether the turret is drawn before the barrel (claim 11).
    pub turret_first: bool,
}

/// Poses a vehicle's body, turret and barrel on flat ground (S-0105 claims
/// 3, 4, 6, 7, 8, 10, 11):
///
/// - *D = Rz(θ_body)* (claims 3, 7; the ramp tilt of the planned voxel tilt
///   spec is not applied),
/// - *T = D · X(d) · Rz(θ_rel)* with *d = δ × ρ* (claims 4, 8, F3),
/// - *B = T · Ry(φ)*, pitched about the turret's origin (claims 6, 10),
///
/// each drawn with *V* in front. `elevation_raw` is the barrel-elevation
/// facing (16384 = level; see [`barrel_elevation_after`]).
pub fn vehicle_parts(
    body_facing: u16,
    turret_facing: u16,
    turret_offset_leptons: i32,
    elevation_raw: u16,
    quirks: &VoxelQuirks,
) -> VehiclePose {
    let v = view(quirks);
    let [d, t, b] = vehicle_local_transforms(
        body_facing,
        turret_facing,
        turret_offset_leptons,
        elevation_raw,
        quirks,
    );
    let body = v.then(&d);
    VehiclePose {
        body,
        turret: v.then(&t),
        barrel: v.then(&b),
        shadow: body,
        turret_first: turret_drawn_first(turret_facing),
    }
}

/// The vehicle part transforms *[D, T, B]* before the view (S-0105 claims
/// 7, 8, 10); [`vehicle_parts`] puts *V* in front of each.
pub fn vehicle_local_transforms(
    body_facing: u16,
    turret_facing: u16,
    turret_offset_leptons: i32,
    elevation_raw: u16,
    quirks: &VoxelQuirks,
) -> [Mat34; 3] {
    // spec: S-0105 claims 3, 7 (flat ground: D is the body rotation)
    let d = Mat34::IDENTITY.rotate_z(body_angle(body_facing), quirks);
    // spec: S-0105 claim 8 (T = D · X(d) · Rz(θ_rel))
    let t = d
        .translate([vehicle_turret_shift(turret_offset_leptons), 0.0, 0.0])
        .rotate_z(turret_relative_angle(turret_facing, body_facing), quirks);
    let b = barrel_from_turret(&t, elevation_raw, quirks);
    [d, t, b]
}

/// The rocking of a body with the driving or ship locomotor (S-0106 claim
/// 9; S-0504 Terms *Pitch and roll*, F5): its pitch *p* and roll *r* in
/// radians, single precision, and its body model's size values *W* and
/// *H*, model units. A positive pitch lowers the bow (S-0105's *Ry*).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rocking {
    /// The pitch *p*, about the body's own lateral (*y*) axis.
    pub pitch: f32,
    /// The roll *r*, about the body's own forward (*x*) axis.
    pub roll: f32,
    /// *W = Ny × 0.5 + 0.5*.
    pub width: f32,
    /// *H = Nx × 0.5*.
    pub length: f32,
}

/// The least pitch or roll magnitude, radians, at which a body rocks
/// (S-0106 Terms, *Rocking*) and so is not settled (S-0105 claim 26).
pub const ROCKING_THRESHOLD: f32 = 0.005; // spec: S-0106 Terms (rocking: a pitch or roll of 0.005 radians or more) and S-0105 claim 26

impl Rocking {
    /// The rocking with pitch `pitch` and roll `roll` of a body whose main
    /// model's first section has grid size `size` *(Nx, Ny, Nz)* (S-0504 F5).
    pub fn new(pitch: f32, roll: f32, size: [u8; 3]) -> Self {
        // spec: S-0504 F5 (W = Ny × 0.5 + 0.5 and H = Nx × 0.5, each formed in double precision and held in single precision)
        let half = 0.5; // spec: S-0504 Constants (rocking size factors: 0.5 per voxel, plus 0.5 on the width)
        let width = to_f32_rz(add_rz(mul_rz(f64::from(size[1]), half), half));
        let length = to_f32_rz(mul_rz(f64::from(size[0]), half));
        Self {
            pitch,
            roll,
            width,
            length,
        }
    }

    /// Whether the body rocks: its pitch or roll is 0.005 radians or more,
    /// compared in floating point (S-0106 Terms; S-0105 claim 26).
    pub fn rocks(&self) -> bool {
        self.pitch.abs() >= ROCKING_THRESHOLD || self.roll.abs() >= ROCKING_THRESHOLD
    }

    /// The lift and the forward and sideways shifts of F5, whole model
    /// units: *(lift, sₓ, sᵧ)* (S-0504 F5).
    pub fn shifts(&self, quirks: &VoxelQuirks) -> (i64, i64, i64) {
        let (sin_r, cos_r) = table_sin_cos(f64::from(self.roll), quirks);
        let (sin_p, cos_p) = table_sin_cos(f64::from(self.pitch), quirks);
        let (w, h) = (f64::from(self.width), f64::from(self.length));
        // spec: S-0504 F5 (lift = trunc(|sin r| × W + |sin p| × H), double precision under S-0205 claims 4 and 5, table sine of S-0105 F1)
        let lift = trunc_i64(add_rz(
            mul_rz(f64::from(sin_r.abs()), w),
            mul_rz(f64::from(sin_p.abs()), h),
        ));
        // spec: S-0504 F5 (sₓ = trunc(H − trunc(H × cos p)), negated when p < 0)
        let sx = trunc_i64(sub_rz(h, trunc_i64(mul_rz(h, f64::from(cos_p))) as f64));
        let sx = if self.pitch < 0.0 { -sx } else { sx };
        // spec: S-0504 F5 (sᵧ = trunc(W − trunc(W × cos r)), negated when r > 0)
        let sy = trunc_i64(sub_rz(w, trunc_i64(mul_rz(w, f64::from(cos_r))) as f64));
        let sy = if self.roll > 0.0 { -sy } else { sy };
        (lift, sx, sy)
    }

    /// The rocking body transform *D* of S-0504 F5, flat ground (no slope
    /// tilt *T*: slope tilt is not drawn, S-0106): *Z(lift) · Rz(θ_body) ·
    /// X(sₓ) · Y(sᵧ) · Rx(r) · Ry(p)*.
    pub fn body(&self, body_facing: u16, quirks: &VoxelQuirks) -> Mat34 {
        let (lift, sx, sy) = self.shifts(quirks);
        // spec: S-0504 F5 (D = Z(lift) · T · Rz(θ_body) · X(sₓ) · Y(sᵧ) · Rx(r) · Ry(p); the lift along the map's vertical, the shifts along the body's forward and left axes after its facing rotation, roll outermost)
        Mat34::IDENTITY
            .translate([0.0, 0.0, lift as f32])
            .rotate_z(body_angle(body_facing), quirks)
            .translate([sx as f32, sy as f32, 0.0])
            .rotate_x(f64::from(self.roll), quirks)
            .rotate_y(f64::from(self.pitch), quirks)
    }
}

/// [`vehicle_parts`] for a rocking body of the driving or ship locomotor
/// (S-0106 claim 9, S-0504 claim 16, F5): *D* is [`Rocking::body`], and *T*
/// and *B* are built from that *D* as in claims 8 and 10, so the turret and
/// barrel lean with the body. The shadow transform is the body's (a sinking
/// vehicle draws no shadow, S-0504 claim 16; the caller leaves it out).
pub fn vehicle_parts_rocking(
    body_facing: u16,
    turret_facing: u16,
    turret_offset_leptons: i32,
    elevation_raw: u16,
    rocking: &Rocking,
    quirks: &VoxelQuirks,
) -> VehiclePose {
    // spec: S-0105 claim 7 and S-0504 F5 (the rocking body transform)
    let d = rocking.body(body_facing, quirks);
    let body = view(quirks).then(&d);
    let pose = VehiclePose {
        body,
        turret: body,
        barrel: body,
        shadow: body,
        turret_first: false,
    };
    turret_on_body(
        pose,
        &d,
        body_facing,
        turret_facing,
        turret_offset_leptons,
        elevation_raw,
        quirks,
    )
}

/// `pose` with its turret and barrel placed on the body transform `d`
/// (before the view) as in claims 8 and 10: *T = D · X(d) · Rz(θ_rel)*,
/// *B = T · Ry(φ)*, so they lean with the body. `body_facing` is the facing
/// the body is drawn with, which the relative turret angle is taken
/// against (claim 4). The body and shadow transforms are kept.
pub fn turret_on_body(
    pose: VehiclePose,
    d: &Mat34,
    body_facing: u16,
    turret_facing: u16,
    turret_offset_leptons: i32,
    elevation_raw: u16,
    quirks: &VoxelQuirks,
) -> VehiclePose {
    let v = view(quirks);
    // spec: S-0105 claim 8 (T = D · X(d) · Rz(θ_rel))
    let t = d
        .translate([vehicle_turret_shift(turret_offset_leptons), 0.0, 0.0])
        .rotate_z(turret_relative_angle(turret_facing, body_facing), quirks);
    let b = barrel_from_turret(&t, elevation_raw, quirks);
    VehiclePose {
        turret: v.then(&t),
        barrel: v.then(&b),
        // spec: S-0105 claim 11 (turret first when its 4-step facing is 1 or 2)
        turret_first: turret_drawn_first(turret_facing),
        ..pose
    }
}

/// The barrel shift of a sprite-body vehicle, *d = trunc(δ / 8)* model
/// units along the body's forward axis (S-0105 claim 12, *Barrel pose*:
/// claim 9's unit, with no switch).
pub fn sprite_body_barrel_shift(offset_leptons: i32) -> f32 {
    // spec: S-0105 claim 12 (Barrel pose: d = trunc(δ / 8), integer division truncated toward zero)
    (offset_leptons / 8) as f32
}

/// The draw transform *V · B* of the single voxel barrel of a vehicle drawn
/// as a sprite (S-0105 claim 12, *Barrel pose*): claim 8's transform with
/// [`sprite_body_barrel_shift`] for its displacement, *T = D · X(d) ·
/// Rz(θ_rel)* (claims 3, 4, 7), then *B = T · Ry(φ)*, pitched by claim 6's
/// 32-step elevation about claim 10's axis through the shifted point (the
/// lead-confirmed pivot departure, extended to this barrel).
pub fn sprite_body_barrel_pose(
    body_facing: u16,
    turret_facing: u16,
    turret_offset_leptons: i32,
    elevation_raw: u16,
    quirks: &VoxelQuirks,
) -> Mat34 {
    // spec: S-0105 claims 3, 7 (flat ground: D is the body rotation)
    let d = Mat34::IDENTITY.rotate_z(body_angle(body_facing), quirks);
    // spec: S-0105 claim 12 (Barrel pose: claim 8's shift first, then claim 4's relative turn about the shifted point)
    let t = d
        .translate([sprite_body_barrel_shift(turret_offset_leptons), 0.0, 0.0])
        .rotate_z(turret_relative_angle(turret_facing, body_facing), quirks);
    // spec: S-0105 claim 12 (Barrel pose: claim 6's elevation about claim 10's axis)
    view(quirks).then(&barrel_from_turret(&t, elevation_raw, quirks))
}

/// *B = T · Ry(φ)*: the barrel pitches about the lateral axis through the
/// turret's origin (S-0105 claim 10).
fn barrel_from_turret(t: &Mat34, elevation_raw: u16, quirks: &VoxelQuirks) -> Mat34 {
    // spec: S-0105 claim 10 (B = T · Ry(φ); lead-confirmed pivot departure)
    t.rotate_y(barrel_pitch(elevation_raw, quirks), quirks)
}

// ---------------------------------------------------------------------------
// Aircraft body attitude (claims 41, 42)
// ---------------------------------------------------------------------------

/// Radians per quarter turn as a double (S-0105 claim 41: *c*).
#[allow(clippy::approx_constant)] // the spec's decimal, claim 41
const QUARTER_TURN: f64 = 1.5707963267948966; // spec: S-0105 claim 41 (c = 1.5707963267948966 radians per quarter turn)

/// The reuse class of a rocket body's pitch (S-0105 claim 41), and for a
/// pitch outside the three eligible classes the table step of its rotation.
///
/// Only the first three classes are reused in the original; any other pitch
/// is drawn fresh. A fresh draw is a function of the pitch's table step
/// alone (F1), so our image cache keeps it under that step: the pixels are
/// those of a fresh draw, the retention only bounds the memory of the
/// scene's images.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RocketPitch {
    /// Zero pitch.
    Level,
    /// A nonzero pitch equal to *pᵢ*.
    Initial,
    /// A nonzero pitch, not *pᵢ*, equal to *p𝒇*.
    Final,
    /// Any other pitch: the table step *σ* of −*p* (F1).
    Fresh(u32),
}

/// The reuse class of the rocket's current single-precision pitch `p` (S-0105
/// claim 41), given the selected set's single-precision initial and final
/// pitch fractions in quarter turns.
pub fn rocket_pitch_class(p: f32, initial: f32, final_: f32, quirks: &VoxelQuirks) -> RocketPitch {
    // spec: S-0105 claim 41 (pᵢ = RZ₅₃(Pᵢ × c), compared with the current single widened exactly, no tolerance, no narrowing of the product)
    let p_initial = mul_rz(f64::from(initial), QUARTER_TURN);
    // spec: S-0105 claim 41 (p𝒇 = RZ₂₄(RZ₅₃(P𝒇 × c)))
    let p_final = to_f32_rz(mul_rz(f64::from(final_), QUARTER_TURN));
    classify_rocket_pitch(p, p_initial, p_final, quirks)
}

/// [`rocket_pitch_class`] with the two endpoint pitches already formed.
fn classify_rocket_pitch(
    p: f32,
    p_initial: f64,
    p_final: f32,
    quirks: &VoxelQuirks,
) -> RocketPitch {
    // spec: S-0105 claim 41 (zero pitch is eligible for the level class)
    if p == 0.0 {
        return RocketPitch::Level;
    }
    if f64::from(p) == p_initial {
        // spec: S-0105 claim 41 (coincident initial and final values favour the initial class)
        RocketPitch::Initial
    } else if p == p_final {
        RocketPitch::Final
    } else {
        // spec: S-0105 claim 41 (all other pitches require fresh draws; F1's table step bounds the retention)
        RocketPitch::Fresh(table_step(-f64::from(p), quirks))
    }
}

/// A rocket body's local draw transform *D = Rz(θ_body) · Ry(−p)* (S-0105
/// claim 41), before the view and the section HVA transforms.
pub fn rocket_body_local(body_facing: u16, pitch: f32, quirks: &VoxelQuirks) -> Mat34 {
    // spec: S-0105 claim 41 (Rz(θ_body) with the 32-step body heading, then Ry(−p): the current single-precision pitch negated and used directly, no rounding to pitch facings; positive p raises the nose)
    Mat34::IDENTITY
        .rotate_z(body_angle(body_facing), quirks)
        .rotate_y(-f64::from(pitch), quirks)
}

/// The pose of a rocket missile's body and its shadow (S-0105 claim 41):
/// the body *V · Rz(θ_body) · Ry(−p)*, the shadow transform *V · Rz(θ_body)*
/// on flat ground whatever the pitch. `turret` and `barrel` are the body's
/// transform; [`turret_on_body`] places them on [`rocket_body_local`].
pub fn rocket_body_pose(body_facing: u16, pitch: f32, quirks: &VoxelQuirks) -> VehiclePose {
    let v = view(quirks);
    let body = v.then(&rocket_body_local(body_facing, pitch, quirks));
    // spec: S-0105 claim 41 (shadow: S = R_s · Rz(θ_body), flat ground here, never the pitch)
    let shadow = v.then(&Mat34::IDENTITY.rotate_z(body_angle(body_facing), quirks));
    VehiclePose {
        body,
        turret: body,
        barrel: body,
        shadow,
        turret_first: false,
    }
}

/// The bank state of a flying body (S-0105 claims 26, 42).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Bank {
    /// No bank.
    None,
    /// An unfinished clockwise aiming-facing turn.
    Clockwise,
    /// An unfinished counterclockwise aiming-facing turn.
    Counterclockwise,
}

/// What a flying body is drawn tilted by (S-0105 claim 42).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlightAttitude {
    /// The drawing pitch *α* in radians (positive lowers the nose); 0 when
    /// none applies.
    pub alpha: f64,
    /// The bank *β* in radians; 0 when none applies.
    pub beta: f64,
    /// Pitch is present in the image key: the height and speed tests pass
    /// and the configured angle is nonzero.
    pub pitched: bool,
    /// The bank state of the image key; none when the configured angle is 0.
    pub bank: Bank,
}

impl FlightAttitude {
    /// No pitch and no bank: the attitude of claim 42 when its tests fail.
    pub const LEVEL: Self = Self {
        alpha: 0.0,
        beta: 0.0,
        pitched: false,
        bank: Bank::None,
    };
}

/// The pitch and bank of a non-crashing flying body (S-0105 claim 42).
///
/// `height` is the signed height above the current surface in leptons,
/// `speed` the current speed fraction, `turn` the sign of the unfinished
/// aiming-facing turn (positive clockwise, 0 none), `pitch_speed` the
/// threshold, `pitch_angle` and `bank_angle` the configured drawing angles
/// in radians.
pub fn flight_attitude(
    height: i32,
    speed: f64,
    turn: i32,
    pitch_speed: f64,
    pitch_angle: f64,
    bank_angle: f64,
) -> FlightAttitude {
    // spec: S-0105 claim 42 (at strictly positive height and a speed fraction strictly above the threshold, compared in double precision; otherwise both angles are zero, take-off and landing included)
    let flying = height > 0 && speed > pitch_speed;
    // spec: S-0105 claim 42 (the configured angles are narrowed to single precision toward zero for the rotations)
    let narrow = |x: f64| f64::from(to_f32_rz(x));
    let alpha = if flying { narrow(pitch_angle) } else { 0.0 };
    let (beta, bank) = match turn.signum() {
        // spec: S-0105 claim 42 (the imported bank angle during an unfinished clockwise turn, its negative during a counterclockwise one; a discrete state, and it needs the speed condition)
        1 if flying => (narrow(bank_angle), Bank::Clockwise),
        -1 if flying => (-narrow(bank_angle), Bank::Counterclockwise),
        _ => (0.0, Bank::None),
    };
    FlightAttitude {
        alpha,
        beta,
        // spec: S-0105 claim 26 (pitch is present in the key only when the tests pass and the configured angle is nonzero; a zero bank angle has the no-bank state)
        pitched: flying && pitch_angle != 0.0,
        bank: if bank_angle == 0.0 { Bank::None } else { bank },
    }
}

/// A flying body's local draw transform *D = Rz(θ_flight) · Ry(α) · Rx(β)*
/// (S-0105 claim 42), before the view and the section HVA transforms.
pub fn flying_body_local(heading: u16, a: &FlightAttitude, quirks: &VoxelQuirks) -> Mat34 {
    // spec: S-0105 claim 42 (D = Rz(θ_flight) · Ry(α) · Rx(β) with the 32-step flight drawing heading; no slope tilt)
    Mat34::IDENTITY
        .rotate_z(body_angle(heading), quirks)
        .rotate_y(a.alpha, quirks)
        .rotate_x(a.beta, quirks)
}

/// The pose of a flying body and its shadow (S-0105 claim 42): the body
/// *V · D*, the shadow *V · Rz(θ_flight)* on flat ground, never the pitch or
/// bank. `heading` is the flight drawing heading: the aiming facing.
/// `turret` and `barrel` are the body's transform; [`turret_on_body`]
/// places them on [`flying_body_local`].
pub fn flying_body_pose(heading: u16, a: &FlightAttitude, quirks: &VoxelQuirks) -> VehiclePose {
    let v = view(quirks);
    let body = v.then(&flying_body_local(heading, a, quirks));
    // spec: S-0105 claim 42 (the shadow uses the slope transform and the 32-step flight drawing heading, never the pitch or bank angles)
    let shadow = v.then(&Mat34::IDENTITY.rotate_z(body_angle(heading), quirks));
    VehiclePose {
        body,
        turret: body,
        barrel: body,
        shadow,
        turret_first: false,
    }
}

/// The drawn pitch *ψ′* of a crashing flying body at strictly positive
/// signed height (S-0113 claim 20): the tumble pitch `pitch` plus the
/// imported drawing-pitch angle `pitch_angle` when S-0105 claim 42's speed
/// test passes (`speed` strictly above `pitch_speed`, compared in double
/// precision), else the tumble pitch alone. The angle is not narrowed to
/// single precision first: the sum is formed at 53-bit precision and
/// stored in single precision, truncated toward zero.
pub fn crash_pitch(pitch: f32, speed: f64, pitch_speed: f64, pitch_angle: f64) -> f32 {
    // spec: S-0113 claim 20 (ψ′ = ψ + α when S-0105 claim 42's speed test passes, the current speed fraction strictly greater than the imported threshold; otherwise ψ′ = ψ)
    if speed > pitch_speed {
        // spec: S-0113 claim 20 and S-0105 F0 (the single pitch and the double angle added at 53-bit precision, the sum stored in single precision toward zero; the angle not first narrowed)
        to_f32_rz(add_rz(f64::from(pitch), pitch_angle))
    } else {
        pitch
    }
}

/// A crashing flying body's local draw transform *D_crash = Rz(θ_flight) ·
/// Rx(φ) · Ry(ψ′)* (S-0113 claim 20), before the view and the section HVA
/// transforms: `roll` is the tumble roll *φ*, `pitch` the drawn pitch *ψ′*
/// of [`crash_pitch`]. No slope tilt, lift or shift, and never the bank.
pub fn crashing_body_local(heading: u16, roll: f32, pitch: f32, quirks: &VoxelQuirks) -> Mat34 {
    // spec: S-0113 claim 20 (D_crash = Rz(θ_flight) · Rx(φ) · Ry(ψ′): pitched about its own lateral axis first, then rolled about its own forward axis, then turned to its heading; no slope tilt, lift or shift; the bank never enters; the rotations of S-0105 F1 and F0)
    Mat34::IDENTITY
        .rotate_z(body_angle(heading), quirks)
        .rotate_x(f64::from(roll), quirks)
        .rotate_y(f64::from(pitch), quirks)
}

/// The pose of a crashing flying body at strictly positive signed height
/// and its shadow (S-0113 claim 20): the body *V · D_crash* on the flight
/// drawing heading, the shadow *V · Rz(θ_flight)* on flat ground, as for
/// level flight. At zero or negative height the caller draws the heading
/// alone ([`flying_body_pose`] with [`FlightAttitude::LEVEL`]). `turret`
/// and `barrel` are the body's transform; [`turret_on_body`] places them on
/// [`crashing_body_local`].
pub fn crashing_body_pose(
    heading: u16,
    roll: f32,
    pitch: f32,
    quirks: &VoxelQuirks,
) -> VehiclePose {
    let v = view(quirks);
    // spec: S-0113 claim 20 (drawn with V · D_crash before the HVA, as S-0105 claim 42's transform is)
    let body = v.then(&crashing_body_local(heading, roll, pitch, quirks));
    // spec: S-0113 claim 20 and S-0105 claim 42 (the shadow follows only the heading and the slope shape, never φ, ψ, α or β)
    let shadow = v.then(&Mat34::IDENTITY.rotate_z(body_angle(heading), quirks));
    VehiclePose {
        body,
        turret: body,
        barrel: body,
        shadow,
        turret_first: false,
    }
}

/// Draw transforms (view included) of a building's voxel turret and barrel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BuildingPose {
    /// *V · T* (claim 9).
    pub turret: Mat34,
    /// *V · B* (claim 10).
    pub barrel: Mat34,
    /// Whether the turret is drawn before the barrel (claim 11).
    pub turret_first: bool,
}

/// Poses a building's voxel turret and barrel (S-0105 claims 5, 9, 10,
/// 11): *T = Rz(θ) · X(d_b)*, *B = T · Ry(φ)*. Both images
/// are placed around the turret anchor (the building's draw point moved by
/// its turret-animation x and y displacements, Terms), which the caller
/// adds to the draw point.
pub fn building_turret_parts(
    facing: u16,
    turret_offset_leptons: i32,
    elevation_raw: u16,
    quirks: &VoxelQuirks,
) -> BuildingPose {
    let v = view(quirks);
    let [t, b] = building_local_transforms(facing, turret_offset_leptons, elevation_raw, quirks);
    BuildingPose {
        turret: v.then(&t),
        barrel: v.then(&b),
        turret_first: turret_drawn_first(facing),
    }
}

/// The building turret and barrel transforms *[T, B]* before the view
/// (S-0105 claims 9, 10); [`building_turret_parts`] puts *V* in front.
pub fn building_local_transforms(
    facing: u16,
    turret_offset_leptons: i32,
    elevation_raw: u16,
    quirks: &VoxelQuirks,
) -> [Mat34; 2] {
    // spec: S-0105 claims 5, 9 (T = Rz(θ) · X(d_b))
    let t = Mat34::IDENTITY
        .rotate_z(body_angle(facing), quirks)
        .translate([
            building_turret_shift(turret_offset_leptons, quirks),
            0.0,
            0.0,
        ]);
    let b = barrel_from_turret(&t, elevation_raw, quirks);
    [t, b]
}

/// The draw transform *V · Rz(θ) · Ry(φ)* of a building's barrel drawn
/// without a turret (S-0105 claim 37): no turret shift, pitched about its
/// own origin, placed around the turret anchor.
pub fn building_barrel_only(facing: u16, elevation_raw: u16, quirks: &VoxelQuirks) -> Mat34 {
    // spec: S-0105 claim 37 (pose)
    view(quirks).then(
        &Mat34::IDENTITY
            .rotate_z(body_angle(facing), quirks)
            .rotate_y(barrel_pitch(elevation_raw, quirks), quirks),
    )
}

/// Whether the turret is drawn before the barrel: when the turret's 4-step
/// facing is 1 or 2; otherwise the barrel goes first (S-0105 claim 11). The
/// body is always drawn before both.
pub fn turret_drawn_first(turret_facing: u16) -> bool {
    // spec: S-0105 claim 11
    matches!(facing4(turret_facing), 1 | 2)
}

// ---------------------------------------------------------------------------
// Barrel elevation turning (claim 6, F9)
// ---------------------------------------------------------------------------

/// The level barrel elevation, raw (S-0105 claim 6).
pub const ELEVATION_LEVEL: u16 = 16384; // spec: S-0105 claim 6
/// Turn rate of the barrel elevation after placement (S-0105 claim 6).
pub const ELEVATION_TURN_RATE: i32 = 3; // spec: S-0105 claim 6, Constants (barrel elevation turn rate)
/// Fire angle of a type without the key (S-0105 Import mapping).
pub const DEFAULT_FIRE_ANGLE: i32 = 8; // spec: S-0105 claim 6, Import mapping (FireAngle default)
/// Highest effective turn rate (S-0105 Terms).
pub const MAX_TURN_RATE: i32 = 127; // spec: S-0105 Terms (turn rate), Formulas F9
/// Facing units per tick per unit of turn rate (S-0105 Terms, F9).
const TURN_UNITS_PER_RATE: i32 = 256; // spec: S-0105 Terms (turn rate), Formulas F9
/// Facing units per fire-angle unit (S-0105 claim 6).
const FIRE_ANGLE_UNIT: i32 = 256; // spec: S-0105 claim 6, Import mapping (1/256 turn)

/// A facing `ticks` whole ticks after it began turning from `start` toward
/// `target` at turn rate `rate` (S-0105 Terms, F9): *Δ* the shorter signed
/// difference, *Θ = ⌊|Δ| / (256 ω)⌋*, *β = trunc(Δ / Θ)*, facing *(ψ₁ −
/// β(Θ − ε)) mod 65536* before *Θ* and *ψ₁* from then on.
pub fn turn_facing(start: u16, target: u16, rate: i32, ticks: u32) -> u16 {
    // spec: S-0105 Formulas F9 (a turn rate of 0 or less turns at once)
    if rate <= 0 {
        return target;
    }
    // spec: S-0105 Terms (values above 127 count as 127)
    let rate = rate.min(MAX_TURN_RATE);
    // spec: S-0105 Formulas F9 (signed remainder, a half-turn is −32768)
    let delta = i32::from(target.wrapping_sub(start) as i16);
    let theta = delta.abs() / (TURN_UNITS_PER_RATE * rate);
    if theta == 0 || i64::from(ticks) >= i64::from(theta) {
        return target;
    }
    let beta = delta / theta;
    let left = theta - ticks as i32;
    (i32::from(target) - beta * left).rem_euclid(65536) as u16
}

/// The elevation a placed vehicle turns toward: *(16384 − 256 × A) mod
/// 65536*, *A* the fire angle modulo 256 (S-0105 claim 6).
pub fn elevation_target(fire_angle: i32) -> u16 {
    // spec: S-0105 claim 6
    let a = fire_angle.rem_euclid(256);
    (i32::from(ELEVATION_LEVEL) - FIRE_ANGLE_UNIT * a).rem_euclid(65536) as u16
}

/// The raw barrel elevation of a vehicle or other non-building object
/// `ticks` ticks after it was placed (S-0105 claim 6, F9).
pub fn barrel_elevation_after(fire_angle: i32, ticks: u32) -> u16 {
    // spec: S-0105 claim 6 (starts level, turns at rate 3)
    turn_facing(
        ELEVATION_LEVEL,
        elevation_target(fire_angle),
        ELEVATION_TURN_RATE,
        ticks,
    )
}

// ---------------------------------------------------------------------------
// HVA frames (claim 16)
// ---------------------------------------------------------------------------

/// A vehicle body's HVA frame *ν mod F_body* (S-0105 claim 16); 0 without
/// frames.
pub fn body_frame(walk_counter: u32, body_frames: u32) -> u32 {
    // spec: S-0105 claim 16
    walk_counter.checked_rem(body_frames).unwrap_or(0)
}

/// A vehicle turret's HVA frame (S-0105 claim 16): the body frame itself
/// when it is not 0, otherwise *u mod F_tur*, or 0 without a turret HVA.
/// The draw then takes it modulo the turret's frame count (claim 13).
pub fn vehicle_turret_frame(body_frame: u32, anim_counter: u32, turret_frames: Option<u32>) -> u32 {
    if body_frame != 0 {
        // spec: S-0105 claim 16 (f_b itself, not reduced)
        body_frame
    } else {
        // spec: S-0105 claim 16
        turret_frames
            .and_then(|f| anim_counter.checked_rem(f))
            .unwrap_or(0)
    }
}

/// A building turret's HVA frame *u mod F_tur* (S-0105 claim 16).
pub fn building_turret_frame(anim_counter: u32, turret_frames: u32) -> u32 {
    // spec: S-0105 claim 16
    anim_counter.checked_rem(turret_frames).unwrap_or(0)
}

/// The HVA frame of a barrel drawn with a voxel turret: 0 (S-0105 claim
/// 16), or the turret's frame with `quirks.barrel_frame_zero` off. Not for
/// the barrel of a sprite-body vehicle, always frame 0 (claim 12).
pub fn barrel_frame_with_turret(turret_frame: u32, quirks: &VoxelQuirks) -> u32 {
    if quirks.barrel_frame_zero {
        // spec: S-0105 claim 16 (quirks.barrel_frame_zero)
        0
    } else {
        turret_frame
    }
}

/// The HVA frame *u mod F_bar* of a building's barrel drawn without a turret
/// (S-0105 claim 37).
pub fn barrel_only_frame(anim_counter: u32, barrel_frames: u32) -> u32 {
    // spec: S-0105 claim 37 (frame)
    anim_counter.checked_rem(barrel_frames).unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Lighting (claims 17–20, F4)
// ---------------------------------------------------------------------------

/// Light direction in view space (S-0105 claim 17, Constants).
#[allow(clippy::excessive_precision)] // the spec's decimals, which identify each single exactly
pub const LIGHT_VIEW: [f32; 3] = [-0.49999943, -0.70710599, 0.49999943]; // spec: S-0105 claim 17
/// Eye vector in the model frame (S-0105 claim 18).
pub const EYE_MODEL: [f32; 3] = [0.0, 0.0, 1.0]; // spec: S-0105 claim 18, Constants (eye vector)
/// Specular sharpness *κ* (S-0105 claim 18, F4).
pub const SPECULAR_SHARPNESS: f64 = 3.0; // spec: S-0105 claim 18, Formulas F4
/// Shade indices per unit of *a + g* (S-0105 F4).
pub const SHADE_LEVELS: f64 = 16.0; // spec: S-0105 Formulas F4, Constants (shade levels)
/// Shade of normal indices 253–255 (S-0105 claim 20).
pub const FORCED_SHADE: u8 = 16; // spec: S-0105 claim 20, Constants
/// First normal index that always gets [`FORCED_SHADE`] (S-0105 claim 20).
pub const FORCED_SHADE_FROM: u8 = 253; // spec: S-0105 claim 20
/// Shade of an out-of-table index that no draw has shaded since program
/// start (S-0105 claim 20); stateless draws use it for every such index.
pub const PROGRAM_START_SHADE: u8 = 0; // spec: S-0105 claim 20

/// The normal direction table of normal mode `mode` (S-0105 Constants);
/// empty for mode 0 (no table) and for modes the spec does not define
/// (above 4; not covered by S-0105).
pub fn normal_table(mode: u8) -> &'static [[f32; 3]] {
    match mode {
        1 => &NORMALS_MODE1,
        2 => &NORMALS_MODE2,
        3 => &NORMALS_MODE3,
        4 => &NORMALS_MODE4,
        _ => &[],
    }
}

/// The light brought into a part's model frame, *L′ = Rᵀ L* (S-0105 claims
/// 17, 18), with *R* the rotation part of `draw`.
pub fn light_in_model_frame(draw: &Mat34) -> [f32; 3] {
    // spec: S-0105 claim 18
    draw.transpose_apply(LIGHT_VIEW)
}

/// The vectors F4 needs from a draw transform: *L′* and *Ĥ*.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadeFrame {
    /// *L′*.
    pub light: [f32; 3],
    /// *Ĥ = (L′ + E) / |L′ + E|*, unnormalised when the length is 0.
    pub half: [f32; 3],
}

impl ShadeFrame {
    /// *L′* and *Ĥ* for rotation `rot` (S-0105 claim 18, F4). The sum and
    /// *Ĥ* are stored in single precision; the length uses the table square
    /// root of the 53-bit sum of squares (summed x, y, z).
    pub fn new(rot: &Mat34, quirks: &VoxelQuirks) -> Self {
        let light = light_in_model_frame(rot);
        let eye = if quirks.voxel_eye_in_model_frame {
            // spec: S-0105 claim 18 (quirks.voxel_eye_in_model_frame)
            EYE_MODEL
        } else {
            // spec: S-0105 Edge cases (voxel_eye_in_model_frame off: Rᵀ (0, 0, 1))
            rot.transpose_apply(EYE_MODEL)
        };
        // spec: S-0105 Formulas F4
        let h = [0, 1, 2].map(|i| to_f32_rz(add_rz(f64::from(light[i]), f64::from(eye[i]))));
        let len = table_sqrt(dot3_rz(h, h));
        let half = if len == 0.0 {
            h
        } else {
            h.map(|c| to_f32_rz(div_rz(f64::from(c), f64::from(len))))
        };
        Self { light, half }
    }

    /// The shade index of normal `n` (S-0105 F4): *a = max(0, n · L′)* at
    /// 53-bit precision, *η = n · Ĥ* (kept at 53-bit precision),
    /// *g = max(0, η / ((κ − η·κ) + η))* stored in single precision before
    /// the clamp, *q = trunc(16 × (a + g)) mod 256*.
    pub fn shade(&self, n: [f32; 3]) -> u8 {
        // spec: S-0105 Formulas F4
        let a = dot3_rz(n, self.light).max(0.0);
        let eta = dot3_rz(n, self.half);
        let k = SPECULAR_SHARPNESS;
        let den = add_rz(sub_rz(k, mul_rz(eta, k)), eta);
        let g = f64::from(to_f32_rz(div_rz(eta, den))).max(0.0);
        let q = trunc_i64(mul_rz(SHADE_LEVELS, add_rz(a, g)));
        q.rem_euclid(256) as u8
    }
}

/// The shade index this draw gives each normal index 0–255 (S-0105 claims
/// 18, 20): F4 for indices the table of `mode` covers, 16 for 253–255, and
/// for the rest the stale shade, which a stateless draw cannot know: 0, the
/// program-start value (16 with `quirks.voxel_stale_normal_shades` off).
pub fn shade_table(rot: &Mat34, mode: u8, quirks: &VoxelQuirks) -> [u8; 256] {
    let frame = ShadeFrame::new(rot, quirks);
    let table = normal_table(mode);
    let mut out = [0u8; 256];
    for (i, q) in out.iter_mut().enumerate() {
        *q = if let Some(&n) = table.get(i) {
            frame.shade(n)
        } else if i >= usize::from(FORCED_SHADE_FROM) || !quirks.voxel_stale_normal_shades {
            // spec: S-0105 claim 20 (253–255 → 16; switch off: every
            // out-of-table index → 16)
            FORCED_SHADE
        } else {
            // spec: S-0105 claim 20 (stale shade; not modelled, program-start value)
            PROGRAM_START_SHADE
        };
    }
    out
}

/// The shade index a voxel projectile's draw gives each normal index 0–255
/// (S-0306 claim 30, F16): one diffuse term of the fixed light brought into
/// the model frame by the rotation of `draw` (the projectile's transform of
/// [`projectile_body`], view included, no motion-file transform), with no
/// specular term and no eye vector: 0 when *d = n · L′* is negative, else
/// trunc(16 *d*). `mode` is the normal mode of the model's first section.
/// Indices 253–255 get 16; the other indices the table does not cover keep
/// an earlier draw's shade, which a stateless draw gives as the
/// program-start value (16 with `quirks.voxel_stale_normal_shades` off).
pub fn projectile_shade_table(draw: &Mat34, mode: u8, quirks: &VoxelQuirks) -> [u8; 256] {
    // spec: S-0306 F16 (L′ = Rₘᵀ L in single precision)
    let light = light_in_model_frame(draw);
    let table = normal_table(mode);
    let mut out = [0u8; 256];
    for (i, q) in out.iter_mut().enumerate() {
        *q = if i >= usize::from(FORCED_SHADE_FROM) {
            // spec: S-0306 claim 30 (normal indices 253 to 255 get shade 16)
            FORCED_SHADE
        } else if let Some(&n) = table.get(i) {
            // spec: S-0306 F16 (d at 53-bit precision; 0 if d < 0, otherwise trunc(16 × d))
            let d = dot3_rz(n, light);
            if d < 0.0 {
                0
            } else {
                trunc_i64(mul_rz(SHADE_LEVELS, d)).clamp(0, i64::from(FORCED_SHADE)) as u8
            }
        } else if !quirks.voxel_stale_normal_shades {
            // spec: S-0105 claim 20 (switch off: every out-of-table index → 16)
            FORCED_SHADE
        } else {
            // spec: S-0306 claim 30 and S-0105 claim 20 (stale shade; not modelled, program-start value)
            PROGRAM_START_SHADE
        };
    }
    out
}

/// A voxel projectile's body draw transform (S-0306 claim 30): the view of
/// S-0105 claim 1, then Rz(−(π/16)(ψ − 8)) · Ry(+(π/16)(e − 8)), with `psi`
/// the 32-step facing of its velocity's horizontal direction and `e` the
/// 32-step facing of its pitch (S-0306 F14). The motion-file transforms of
/// the sections follow in [`draw_model_shaded`].
pub fn projectile_body(psi: u8, e: u8, quirks: &VoxelQuirks) -> Mat34 {
    view(quirks).then(&projectile_body_local(psi, e, quirks))
}

/// [`projectile_body`] without the view: Rz(−(π/16)(ψ − 8)) · Ry(+(π/16)(e − 8)).
pub fn projectile_body_local(psi: u8, e: u8, quirks: &VoxelQuirks) -> Mat34 {
    // spec: S-0306 claim 30 (heading as S-0105 claim 3, pitch with S-0105 claim 6's sign: e below 8 raises the nose)
    Mat34::IDENTITY
        .rotate_z(facing_angle(-(i32::from(psi) - UNROTATED_FACING32)), quirks)
        .rotate_y(facing_angle(i32::from(e) - UNROTATED_FACING32), quirks)
}

/// The palette index drawn for voxel colour `colour` at shade `shade`:
/// *VPL(q, χ)* (S-0105 claim 19). Without a table, or when the table has
/// no row `shade` (neither case is covered by S-0105), the colour index
/// itself.
pub fn lit_colour(vpl: Option<&Vpl>, shade: u8, colour: u8) -> u8 {
    // spec: S-0105 claim 19
    vpl.map_or(colour, |v| v.get(shade, colour))
}
/// Normal mode 1: 16 directions (S-0105 Constants, normal direction tables).
#[allow(clippy::excessive_precision)] // the spec's decimals, which identify each single exactly
pub const NORMALS_MODE1: [[f32; 3]; 16] = [
    // spec: S-0105 Constants (normal direction tables)
    [0.549462974, -0.000182999996, -0.835518003],
    [0.000144000005, 0.549404025, -0.835556984],
    [-0.549404025, -6.80000012e-05, -0.835556984],
    [0.000106, -0.549462974, -0.835518003],
    [0.949007988, 0.000315999991, -0.315250009],
    [-0.000186000005, 0.948997021, -0.315284014],
    [-0.948997021, 0.000318000006, -0.315284014],
    [-0.000446999999, -0.949007988, -0.315250009],
    [0.95084399, -0.000279, 0.309671015],
    [0.000201999996, 0.950847983, 0.309657007],
    [-0.950847983, -7.00000019e-05, 0.309657007],
    [0.000146999999, -0.95084399, 0.309671015],
    [0.552370012, -1.10000001e-05, 0.833598971],
    [1.99999995e-05, 0.552380025, 0.833591998],
    [-0.552380025, 5.70000011e-05, 0.833593011],
    [-6.60000005e-05, -0.552370012, 0.833598971],
];

/// Normal mode 2: 36 directions (S-0105 Constants, normal direction tables).
#[allow(clippy::excessive_precision)] // the spec's decimals, which identify each single exactly
pub const NORMALS_MODE2: [[f32; 3]; 36] = [
    // spec: S-0105 Constants (normal direction tables)
    [0.671213984, 0.198492005, -0.714194],
    [0.269643009, 0.584393978, -0.765359998],
    [-0.040546, 0.096988, -0.994458973],
    [-0.572427988, -0.0919139981, -0.814786971],
    [-0.171400994, -0.572709978, -0.80163902],
    [0.362556994, -0.30299899, -0.881331027],
    [0.810347021, -0.348971993, -0.470697999],
    [0.103961997, 0.938672006, -0.328767002],
    [-0.324046999, 0.587669015, -0.741375983],
    [-0.800864995, 0.340460986, -0.492646992],
    [-0.665498018, -0.590147018, -0.45698899],
    [0.314767003, -0.803002, -0.506072998],
    [0.972629011, 0.151076004, -0.176550001],
    [0.680290997, 0.68423599, -0.262726992],
    [-0.520079017, 0.827777028, -0.210483],
    [-0.961643994, -0.179001004, -0.207846999],
    [-0.262713999, -0.937451005, -0.228401005],
    [0.219706997, -0.971301019, 0.0911249965],
    [0.923807979, -0.229975, 0.306086987],
    [-0.0824889988, 0.970659971, 0.225866005],
    [-0.591798007, 0.69678998, 0.405288994],
    [-0.925296009, 0.36660099, 0.0971110016],
    [-0.705051005, -0.687775016, 0.172828004],
    [0.7324, -0.680366993, -0.0263049994],
    [0.855162024, 0.374581993, 0.358310997],
    [0.47300601, 0.836480021, 0.276704997],
    [-0.0976170003, 0.654111981, 0.750072002],
    [-0.904124022, -0.153724998, 0.398658007],
    [-0.211916, -0.858089983, 0.467732012],
    [0.500226974, -0.674408019, 0.543090999],
    [0.584538996, -0.110248998, 0.803840995],
    [0.437373012, 0.454643995, 0.775888979],
    [-0.0424409993, 0.0833180025, 0.995618999],
    [-0.596251011, 0.220131993, 0.772028029],
    [-0.506455004, -0.396977007, 0.765448987],
    [0.0705690011, -0.478473991, 0.875262022],
];

/// Normal mode 3: 64 directions (S-0105 Constants, normal direction tables).
#[allow(clippy::excessive_precision)] // the spec's decimals, which identify each single exactly
pub const NORMALS_MODE3: [[f32; 3]; 64] = [
    // spec: S-0105 Constants (normal direction tables)
    [0.456510991, -0.0739680007, -0.886637986],
    [0.507694006, 0.385116994, -0.770669997],
    [0.0954319984, 0.226664007, -0.969286025],
    [-0.35876599, 0.543187976, -0.759100974],
    [-0.361276001, 0.132994995, -0.922926009],
    [-0.483117014, -0.324066013, -0.813374996],
    [-0.018073, -0.197558999, -0.980123997],
    [0.321099997, -0.501477003, -0.803377986],
    [0.799490988, 0.0696159974, -0.596629977],
    [0.390971005, 0.771305978, -0.502224028],
    [0.0807820037, 0.614489973, -0.784777999],
    [-0.732749999, 0.411431015, -0.542034984],
    [-0.73525399, 0.00910199992, -0.677730978],
    [-0.802493989, -0.394908011, -0.447270989],
    [-0.134130001, -0.589155018, -0.796809018],
    [0.719552994, -0.376226991, -0.583693027],
    [0.966875017, 0.173593, -0.187132001],
    [0.760830998, 0.519105971, -0.38944301],
    [-0.114642002, 0.875518978, -0.469386011],
    [-0.532366991, 0.768859029, -0.354176998],
    [-0.962266982, 0.0249770004, -0.270958006],
    [-0.467386991, -0.721985996, -0.510182023],
    [0.0584499985, -0.85235399, -0.519689023],
    [0.498232991, -0.743740022, -0.445663005],
    [0.939150989, -0.270244986, -0.212044001],
    [0.583931983, 0.809441984, -0.061857],
    [0.183797002, 0.973227978, -0.138007],
    [-0.884355009, 0.452219009, -0.115822002],
    [-0.943177998, -0.332067013, 0.0121379998],
    [-0.698440015, -0.706566989, -0.113771997],
    [-0.228411004, -0.954706013, -0.190694004],
    [0.731563985, -0.675861001, -0.0895880014],
    [0.969250977, 0.0468039997, 0.241582006],
    [0.855647027, 0.503476977, 0.119915999],
    [-0.251152992, 0.967947006, -8.09999983e-05],
    [-0.647795022, 0.756748974, 0.0877119973],
    [-0.969164014, 0.145193994, 0.199100003],
    [-0.414793015, -0.888966978, 0.194125995],
    [0.250775009, -0.961178005, -0.115108997],
    [0.478628993, -0.842593014, 0.246883005],
    [0.890043974, -0.396142006, 0.225594997],
    [0.524051011, 0.762359977, 0.379707009],
    [0.119620003, 0.945482016, 0.30291],
    [-0.760850012, 0.490074992, 0.425361991],
    [-0.869785011, -0.202150002, 0.450121999],
    [-0.709466994, -0.602424026, 0.36570701],
    [0.0193089992, -0.958871007, 0.283185989],
    [0.626112998, -0.564677, 0.537701011],
    [0.769942999, -0.126662999, 0.625415027],
    [0.764190972, 0.350701988, 0.541314006],
    [-0.00187799998, 0.741366982, 0.671097994],
    [-0.370880008, 0.818368018, 0.439000994],
    [-0.71390897, 0.128652006, 0.688318014],
    [-0.295165002, -0.738663971, 0.606014013],
    [0.186195001, -0.738368988, 0.648184001],
    [0.387522995, -0.358783007, 0.84917599],
    [0.481022, 0.124845996, 0.86777401],
    [0.391808003, 0.545055985, 0.741216004],
    [-0.00353599992, 0.365597993, 0.930765986],
    [-0.420498013, 0.484961003, 0.766807973],
    [-0.354903013, 0.0194700006, 0.934700012],
    [-0.547837019, -0.359207988, 0.755542994],
    [-0.106661998, -0.445115, 0.889097989],
    [0.0867960006, -0.0593070015, 0.994458973],
];

/// Normal mode 4: 245 directions (S-0105 Constants, normal direction tables).
#[allow(clippy::excessive_precision)] // the spec's decimals, which identify each single exactly
pub const NORMALS_MODE4: [[f32; 3]; 245] = [
    // spec: S-0105 Constants (normal direction tables)
    [0.526578009, -0.359620988, -0.770317018],
    [0.150481999, 0.435983986, 0.887283981],
    [0.414195001, 0.738255024, -0.532374024],
    [0.0751520023, 0.916248977, -0.393498003],
    [-0.316148996, 0.930736005, -0.183792993],
    [-0.773819029, 0.623333991, -0.112510003],
    [-0.900842011, 0.428537011, -0.0695680007],
    [-0.998942018, -0.0109710004, 0.0446650013],
    [-0.979761004, -0.157670006, -0.123323999],
    [-0.911274016, -0.362370998, -0.19562],
    [-0.624068975, -0.720941007, -0.301301003],
    [-0.310173005, -0.809345007, -0.498751998],
    [0.146613002, -0.815819025, -0.559414029],
    [-0.716516018, -0.694356024, -0.0668879971],
    [0.503971994, -0.114202, -0.856136978],
    [0.455491006, 0.87262702, -0.176210999],
    [-0.00500999996, -0.114372998, -0.993425012],
    [-0.104675002, -0.327701002, -0.938965023],
    [0.56041199, 0.752588987, -0.345755994],
    [-0.0605759993, 0.821627975, -0.566796005],
    [-0.302341014, 0.797007024, -0.522846997],
    [-0.671543002, 0.670740008, -0.314862996],
    [-0.778401017, -0.128356993, 0.614504993],
    [-0.924049973, 0.278382003, -0.261985004],
    [-0.699773014, -0.550490975, -0.455278009],
    [-0.568247974, -0.517189026, -0.640007973],
    [0.0540979989, -0.93286401, -0.356142998],
    [0.758382022, 0.572893023, -0.310887992],
    [0.00362000009, 0.305025995, -0.952337027],
    [-0.0608499981, -0.986886024, -0.149510995],
    [0.635230005, 0.0454780012, -0.770982981],
    [0.521704972, 0.241309002, -0.818287015],
    [0.269403994, 0.635424972, -0.723640978],
    [0.0456760004, 0.67275399, -0.738454998],
    [-0.180510998, 0.674656987, -0.715718985],
    [-0.397130996, 0.636640012, -0.661041975],
    [-0.55200398, 0.472514987, -0.687038004],
    [-0.772170007, 0.0830899999, -0.629960001],
    [-0.669818997, -0.119533002, -0.732840002],
    [-0.540454984, -0.318444014, -0.77878201],
    [-0.386135012, -0.522789001, -0.75999397],
    [-0.261465997, -0.688566983, -0.676394999],
    [-0.0194119997, -0.696102977, -0.717679977],
    [0.303568989, -0.481844008, -0.821992993],
    [0.681939006, -0.195129007, -0.704900026],
    [-0.244889006, -0.116562001, -0.96251899],
    [0.800759017, -0.0229790006, -0.598546028],
    [-0.370274991, 0.0955839977, -0.923991024],
    [-0.330671012, -0.326577991, -0.885439992],
    [-0.163220003, -0.52757901, -0.83367902],
    [0.126389995, -0.313145995, -0.941257],
    [0.349548012, -0.272226006, -0.896498024],
    [0.239917994, -0.0858250037, -0.966992021],
    [0.390845001, 0.0815370008, -0.91683799],
    [0.255266994, 0.268696994, -0.928785026],
    [0.146245003, 0.480437994, -0.864749014],
    [-0.326016009, 0.478455991, -0.815348983],
    [-0.469682008, -0.112519003, -0.875635982],
    [0.81844002, -0.258520007, -0.51315099],
    [-0.474317998, 0.292237997, -0.830433011],
    [0.778943002, 0.395841986, -0.486371011],
    [0.624094009, 0.39377299, -0.674870014],
    [0.740885973, 0.203833997, -0.639953017],
    [0.48021701, 0.565768003, -0.670297027],
    [0.380930007, 0.424535006, -0.821377993],
    [-0.0934220031, 0.501124024, -0.860318005],
    [-0.236485004, 0.29619801, -0.925387025],
    [-0.131531, 0.0939590037, -0.98684901],
    [-0.823562026, 0.295776993, -0.484005988],
    [0.611065984, -0.624303997, -0.486663997],
    [0.0694959983, -0.520330012, -0.851132989],
    [0.226521999, -0.664879024, -0.711775005],
    [0.471307993, -0.568903983, -0.67395699],
    [0.388424993, -0.742623985, -0.545560002],
    [0.783675015, -0.480729014, -0.393384993],
    [0.962393999, 0.135675997, -0.235349],
    [0.876607001, 0.172033995, -0.449405998],
    [0.63340503, 0.589793026, -0.500940979],
    [0.182275996, 0.800657988, -0.570720971],
    [0.177002996, 0.76413399, 0.620297015],
    [-0.544016004, 0.675514996, -0.497720987],
    [-0.67929697, 0.286466986, -0.675642014],
    [-0.59039098, 0.0913690031, -0.801928997],
    [-0.824360013, -0.133123994, -0.550189018],
    [-0.715794027, -0.334542006, -0.612960994],
    [0.174285993, -0.892484009, 0.416049004],
    [-0.0825280026, -0.837122977, -0.540753007],
    [0.283331007, -0.880873978, -0.379189014],
    [0.675134003, -0.42662701, -0.601817012],
    [0.843720019, -0.512335002, -0.160155997],
    [0.977303982, -0.098555997, -0.187519997],
    [0.846294999, 0.522671998, -0.102946997],
    [0.677141011, 0.72132498, -0.145501003],
    [0.320964992, 0.870891988, -0.372193992],
    [-0.178977996, 0.911532998, -0.370236009],
    [-0.447169006, 0.826700985, -0.341473997],
    [-0.703203022, 0.496327996, -0.509081006],
    [-0.977181017, 0.0635629967, -0.202674001],
    [-0.878170013, -0.412937999, 0.241455004],
    [-0.835830986, -0.358550012, -0.415728003],
    [-0.499173999, -0.693432987, -0.519591987],
    [-0.188788995, -0.923753023, -0.333225012],
    [0.192254007, -0.969361007, -0.152896002],
    [0.515940011, -0.783906996, -0.345391989],
    [0.905924976, -0.300951988, -0.297870994],
    [0.991111994, -0.127746001, 0.0371069983],
    [0.995135009, 0.0984240025, -0.00438300008],
    [0.760123014, 0.64627701, 0.0673670024],
    [0.205220997, 0.959580004, -0.192590997],
    [-0.042750001, 0.97951299, -0.196790993],
    [-0.438017011, 0.898926973, 0.0084920004],
    [-0.821994007, 0.480785012, -0.305238992],
    [-0.899917006, 0.0817100033, -0.428337008],
    [-0.92661202, -0.144618005, -0.347095996],
    [-0.793659985, -0.557792008, -0.242838994],
    [-0.431349993, -0.847778976, -0.308557987],
    [-0.00549199991, -0.964999974, 0.262192994],
    [0.58790499, -0.804026008, -0.088940002],
    [0.699492991, -0.667685986, -0.254765004],
    [0.889303029, 0.359795004, -0.282290995],
    [0.780972004, 0.197036996, 0.59267199],
    [0.520120978, 0.506695986, 0.687556982],
    [0.403894991, 0.693961024, 0.596059978],
    [-0.154982999, 0.899236023, 0.409090012],
    [-0.657338023, 0.537168026, 0.528542995],
    [-0.746195018, 0.334091008, 0.575827003],
    [-0.624952018, -0.0491439998, 0.779115021],
    [0.318141013, -0.254714996, 0.913185],
    [-0.555896997, 0.405294001, 0.725751996],
    [-0.794434011, 0.0994059965, 0.599160016],
    [-0.640361011, -0.689463019, 0.338494986],
    [-0.126712993, -0.734094977, 0.66711998],
    [0.105457, -0.780816972, 0.615795016],
    [0.407992989, -0.480915993, 0.776054978],
    [0.695136011, -0.545120001, 0.468647003],
    [0.973191023, -0.00648899982, 0.229908004],
    [0.94689399, 0.317508996, -0.050799001],
    [0.563583016, 0.825612009, 0.027183],
    [0.325773001, 0.945423007, 0.00694900006],
    [-0.171820998, 0.985096991, -0.00781499967],
    [-0.670440972, 0.739938974, 0.0547689982],
    [-0.822981, 0.554961979, 0.121321999],
    [-0.96619302, 0.117857002, 0.229306996],
    [-0.953769028, -0.29470399, 0.0589450002],
    [-0.864386976, -0.502727985, -0.0100149997],
    [-0.530609012, -0.842006028, -0.0973659977],
    [-0.162617996, -0.98407501, 0.0717720017],
    [0.0814469978, -0.996011019, 0.0364390016],
    [0.745984018, -0.665962994, 0.000761999981],
    [0.942057014, -0.329268992, -0.0641060024],
    [0.939701974, -0.281089991, 0.194802999],
    [0.771214008, 0.550670028, 0.319362998],
    [0.641348004, 0.730690002, 0.234020993],
    [0.080682002, 0.996690989, 0.00987899955],
    [-0.0467250012, 0.976643026, 0.209725007],
    [-0.531076014, 0.821000993, 0.209562004],
    [-0.695815027, 0.655990005, 0.29243499],
    [-0.976122022, 0.216709003, -0.0149130002],
    [-0.961660981, -0.144128993, 0.233313993],
    [-0.772083998, -0.613646984, 0.165298998],
    [-0.449600011, -0.836059988, 0.314426005],
    [-0.392699987, -0.914615989, 0.0962470025],
    [0.390588999, -0.919470012, 0.0448900014],
    [0.582529008, -0.799197972, 0.148127005],
    [0.866430998, -0.489811987, 0.096864],
    [0.904586971, 0.111497998, 0.411449999],
    [0.953536987, 0.232329994, 0.191806003],
    [0.497310996, 0.770802975, 0.398176998],
    [0.194066003, 0.956319988, 0.218611002],
    [0.422876, 0.882275999, 0.206797004],
    [-0.373796999, 0.849565983, 0.372173995],
    [-0.534497023, 0.714022994, 0.452199996],
    [-0.881826997, 0.237159997, 0.407597989],
    [-0.904947996, -0.0140690003, 0.425289005],
    [-0.751827002, -0.512817025, 0.414458007],
    [-0.501015007, -0.697916985, 0.511758029],
    [-0.235190004, -0.92592299, 0.295554996],
    [0.228983, -0.953939974, 0.193819001],
    [0.734025002, -0.634898007, 0.241062],
    [0.913752973, -0.0632530004, -0.401315987],
    [0.905735016, -0.161486998, 0.391874999],
    [0.858929992, 0.342445999, 0.380748987],
    [0.624486029, 0.607581019, 0.490776986],
    [0.289263994, 0.857478976, 0.425507993],
    [0.0699680001, 0.902168989, 0.425671011],
    [-0.28617999, 0.940699995, 0.182164997],
    [-0.574012995, 0.805118978, -0.149308994],
    [0.111258, 0.099717997, -0.988776028],
    [-0.30539301, -0.944227993, -0.123159997],
    [-0.60116601, -0.789575994, 0.123163],
    [-0.290645003, -0.812139988, 0.50591898],
    [-0.0649200007, -0.877162993, 0.475784987],
    [0.408300996, -0.862215996, 0.299789011],
    [0.566097021, -0.72556603, 0.391263992],
    [0.839363992, -0.427386999, 0.335869014],
    [0.818899989, -0.0413050018, 0.572448015],
    [0.719784021, 0.414997011, 0.556496978],
    [0.881744027, 0.450269997, 0.140659004],
    [0.401823014, -0.898220003, -0.178151995],
    [-0.0540199988, 0.791343987, 0.60898],
    [-0.293774009, 0.763993979, 0.574464977],
    [-0.450798005, 0.610346973, 0.651350975],
    [-0.638221025, 0.186693996, 0.746873021],
    [-0.872870028, -0.257126987, 0.414707989],
    [-0.587257028, -0.521709979, 0.618827999],
    [-0.353657991, -0.641973972, 0.680290997],
    [0.0416489989, -0.611272991, 0.790323019],
    [0.348342001, -0.779182971, 0.521086991],
    [0.499166995, -0.622440994, 0.602825999],
    [0.790018976, -0.303831011, 0.532500029],
    [0.660117984, 0.0607330017, 0.74870199],
    [0.604920983, 0.294160992, 0.739960015],
    [0.385697007, 0.379346013, 0.841032028],
    [0.239693001, 0.207875997, 0.948332012],
    [0.012623, 0.258531988, 0.965919971],
    [-0.100556999, 0.457147002, 0.883687973],
    [0.0469669998, 0.628588021, 0.776319027],
    [-0.430391014, -0.445405006, 0.785097003],
    [-0.434291005, -0.196227998, 0.879139006],
    [-0.256637007, -0.336867005, 0.905902028],
    [-0.131372005, -0.158910006, 0.978514016],
    [0.102379002, -0.208766997, 0.972591996],
    [0.195686996, -0.450129002, 0.87125802],
    [0.627318978, -0.423148006, 0.653770983],
    [0.687439024, -0.171582997, 0.70568198],
    [0.275920004, -0.0212549996, 0.960946023],
    [0.459367007, 0.157465994, 0.874177992],
    [0.285394996, 0.583184004, 0.760555983],
    [-0.812174022, 0.460303009, 0.358460993],
    [-0.189068004, 0.641223013, 0.743698001],
    [-0.338874996, 0.476480007, 0.811251998],
    [-0.920993984, 0.347185999, 0.176726997],
    [0.0406389982, 0.0244650003, 0.998874009],
    [-0.739131987, -0.35374701, 0.573189974],
    [-0.603511989, -0.286615014, 0.74405998],
    [-0.188676, -0.547059, 0.815554023],
    [-0.0260450002, -0.397819996, 0.917093992],
    [0.26789701, -0.649040997, 0.71202302],
    [0.518245995, -0.284891009, 0.806385994],
    [0.493450999, -0.0665329993, 0.867224991],
    [-0.328188002, 0.140250996, 0.934143007],
    [-0.328188002, 0.140250996, 0.934143007],
    [-0.328188002, 0.140250996, 0.934143007],
    [-0.328188002, 0.140250996, 0.934143007],
    [-0.328188002, 0.140250996, 0.934143007],
];

// ---------------------------------------------------------------------------
// Model sections (claims 13–15)
// ---------------------------------------------------------------------------

/// One section of a voxel model, as the rasteriser reads it: geometry plus
/// each column's column data (S-0105 Terms: *Column data*).
///
/// Claims 21, 30, 35 and 36 depend on the column data exactly as the file
/// stores it, and on whether a column is marked empty or present without
/// voxels. [`Section::from_limb`] cannot recover either from a parsed
/// [`Limb`] (see its docs); build a `Section` directly from the file's
/// column data where that matters.
#[derive(Clone, Debug, PartialEq)]
pub struct Section {
    /// Grid size *(Nx, Ny, Nz)*.
    pub size: [u8; 3],
    /// *Bmin* (model units).
    pub min: [f32; 3],
    /// *Bmax* (model units).
    pub max: [f32; 3],
    /// Scale factor (claim 14).
    pub scale: f32,
    /// Normal-table mode (claims 18, 35).
    pub normal_mode: u8,
    /// Column data by column index *j · Nx + i*; `None` for a column the
    /// file marks as empty.
    pub columns: Vec<Option<Vec<u8>>>,
}

impl Section {
    /// A section from a parsed limb, with each column's data exactly as the
    /// file stores it (`Limb::columns`), so columns marked empty and
    /// columns present without voxels stay apart (claim 30) and normal mode
    /// 0 reads the file's own values (claim 35). The VXL file's own
    /// per-section transform is not used (claim 13).
    pub fn from_limb(limb: &Limb) -> Self {
        let [sx, sy, sz] = limb.size;
        let (nx, ny) = (usize::from(sx), usize::from(sy));
        let columns = if limb.columns.len() == nx * ny {
            limb.columns.clone()
        } else {
            // A limb built without its stored columns: re-encode its voxels.
            let mut cells: Vec<Vec<(u8, u8, u8)>> = vec![Vec::new(); nx * ny];
            for v in &limb.voxels {
                if v.x < sx && v.y < sy && v.z < sz {
                    cells[usize::from(v.y) * nx + usize::from(v.x)].push((v.z, v.color, v.normal));
                }
            }
            cells
                .into_iter()
                .map(|mut c| {
                    if c.is_empty() {
                        return None;
                    }
                    c.sort_unstable_by_key(|v| v.0);
                    c.dedup_by_key(|v| v.0);
                    Some(encode_column(&c, sz))
                })
                .collect()
        };
        Self {
            size: limb.size,
            min: limb.min.map(|v| v.get()),
            max: limb.max.map(|v| v.get()),
            scale: limb.scale.get(),
            normal_mode: limb.normals_mode,
            columns,
        }
    }

    /// The box corner `idx` (bit 0: max x, bit 1: max y, bit 2: max z).
    fn corner(&self, idx: usize) -> [f32; 3] {
        [0, 1, 2].map(|a| {
            if idx & (1 << a) != 0 {
                self.max[a]
            } else {
                self.min[a]
            }
        })
    }
}

/// Standard column data for voxels `(z, colour, normal)` in ascending `z`:
/// pairs of an empty run and a voxel run, each voxel run written as count,
/// colour and normal per voxel, count again; a closing empty run up to
/// `nz` when the voxels stop short of it (S-0105 Terms: *Column data*).
pub fn encode_column(voxels: &[(u8, u8, u8)], nz: u8) -> Vec<u8> {
    let mut out = Vec::new();
    let mut cur = 0u32;
    let mut i = 0;
    while i < voxels.len() {
        let z0 = u32::from(voxels[i].0);
        let mut n = 1;
        while i + n < voxels.len() && n < 255 && u32::from(voxels[i + n].0) == z0 + n as u32 {
            n += 1;
        }
        out.push(z0.saturating_sub(cur).min(255) as u8);
        out.push(n as u8);
        for v in &voxels[i..i + n] {
            out.extend_from_slice(&[v.1, v.2]);
        }
        out.push(n as u8);
        cur = z0 + n as u32;
        i += n;
    }
    if cur < u32::from(nz) {
        out.extend_from_slice(&[(u32::from(nz) - cur) as u8, 0, 0]);
    }
    out
}

/// A voxel model ready to draw: its sections and its HVA (S-0105 claims
/// 13, 14).
#[derive(Clone, Debug, PartialEq)]
pub struct VoxelModel {
    /// Sections in file order.
    pub sections: Vec<Section>,
    hva: Option<Hva>,
    hva_scale_first_section: bool,
}

impl VoxelModel {
    /// A model from a parsed VXL (lossy, see [`Section::from_limb`]) and its
    /// HVA, if any.
    pub fn new(vxl: &Vxl, hva: Option<&Hva>, quirks: &VoxelQuirks) -> Self {
        Self::from_sections(
            vxl.limbs.iter().map(Section::from_limb).collect(),
            hva,
            quirks,
        )
    }

    /// A model from sections and an HVA.
    pub fn from_sections(sections: Vec<Section>, hva: Option<&Hva>, quirks: &VoxelQuirks) -> Self {
        Self {
            sections,
            hva: hva.cloned(),
            hva_scale_first_section: quirks.hva_scale_first_section,
        }
    }

    /// The HVA's frame count *F* (0 without an HVA).
    pub fn frame_count(&self) -> usize {
        self.hva.as_ref().map_or(0, Hva::frames)
    }

    /// *H(f mod F, i)* with its translation multiplied by the first
    /// section's scale factor, each product stored truncated (S-0105
    /// claims 13, 14). Without an HVA, or for a section the HVA does not
    /// have, the identity (neither case is covered by S-0105).
    pub fn section_transform(&self, frame: u32, section: usize) -> Mat34 {
        let Some(hva) = &self.hva else {
            return Mat34::IDENTITY;
        };
        let frames = hva.frames();
        if frames == 0 {
            return Mat34::IDENTITY;
        }
        // spec: S-0105 claim 13 (frame f mod F)
        let f = frame as usize % frames;
        let Some(m) = hva.matrix(f, section) else {
            return Mat34::IDENTITY;
        };
        let scale_of = if self.hva_scale_first_section {
            0
        } else {
            section
        };
        // spec: S-0105 claim 14 (quirks.hva_scale_first_section)
        let scale = self.sections.get(scale_of).map_or(1.0, |s| s.scale);
        let mut out = Mat34::from_hva(m);
        for row in &mut out.0 {
            // spec: S-0105 claim 14 (product truncated to single precision)
            row[3] = mul_f32_rz(row[3], scale);
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Column reading (claims 21, 35, 36)
// ---------------------------------------------------------------------------

/// One step of a column as the rasteriser reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    /// A run of empty cells: one *e·Δ_z* addend (F5).
    Empty(u32),
    /// A voxel: a lit colour with its normal index, or an unlit value.
    Voxel { value: u8, normal: Option<u8> },
}

/// The steps of one column, in drawing order (S-0105 claims 21, 35, 36).
///
/// In normal mode 0 this grouping is claim 35's colour-height relation:
/// bottom-up a span's empty count precedes its colours, top-down it follows
/// them and affects only later spans, for every span with no first-span
/// exception (r10, Q-0216).
///
/// With `quirks.voxel_topdown_column_origin` on, a column is read in groups
/// from its first value (bottom-up) or its last value backward (top-down):
/// bottom-up *e, n,* the *n* voxels, one unused value; top-down *n,* the
/// *n* voxels, one unused value, *e*. The column ends at the first group
/// boundary, its start included, where the cells counted so far are
/// congruent to *Nz* modulo 256 (modulo 2³² in the depth-buffered mode),
/// or where its own column data ends (our engine's choice for the part the
/// spec leaves unspecified). A voxel whose values are cut off by the end of
/// the column data is not drawn.
///
/// A voxel holds two values (colour, normal) except in normal mode 0 with
/// `quirks.voxel_mode0_misread` on, where it holds one value drawn unlit
/// (claim 35). Top-down, a voxel's values are taken in file order from
/// just before the current reading position.
fn column_steps(
    data: &[u8],
    nz: u8,
    top_down: bool,
    mode: u8,
    depth_buffered: bool,
    quirks: &VoxelQuirks,
) -> Vec<Step> {
    // spec: S-0105 claim 35 (mode 0: one value per voxel, unlit)
    let per = if mode == 0 && quirks.voxel_mode0_misread {
        1
    } else {
        2
    };
    let voxel = |v: &[u8]| {
        if mode == 0 {
            // spec: S-0105 claim 35 (unlit, value as it is)
            Step::Voxel {
                value: v[0],
                normal: None,
            }
        } else {
            Step::Voxel {
                value: v[0],
                normal: v.get(1).copied(),
            }
        }
    };
    if !quirks.voxel_topdown_column_origin {
        return fixed_column_steps(data, nz, top_down, per, voxel);
    }
    // spec: S-0105 claims 35, 36, Edge cases (quirks.voxel_topdown_column_origin: the
    // boundary is taken modulo 256, or modulo 4294967296 in the depth-buffered mode)
    let mask = if depth_buffered { u32::MAX } else { 0xFF };
    let at_boundary = |count: u32| count & mask == u32::from(nz);
    let mut steps = Vec::new();
    let mut count = 0u32;
    if !top_down {
        let mut pos = 0usize;
        loop {
            // spec: S-0105 claim 36 (ends at a pair boundary ≡ Nz)
            if at_boundary(count) {
                break;
            }
            let (Some(&e), Some(&n)) = (data.get(pos), data.get(pos + 1)) else {
                break;
            };
            pos += 2;
            steps.push(Step::Empty(u32::from(e)));
            for _ in 0..n {
                let Some(v) = data.get(pos..pos + per) else {
                    return steps;
                };
                steps.push(voxel(v));
                pos += per;
            }
            // the span's closing count (VXL layout, glasnost_fmt::vxl) is skipped; in mode 0
            // the relation's ignored value b (claim 35: e, n, a₀ … aₙ₋₁, b)
            pos += 1;
            count = count.wrapping_add(u32::from(e)).wrapping_add(u32::from(n));
        }
    } else {
        let mut end = data.len();
        loop {
            // spec: S-0105 claim 36 (ends at a pair boundary ≡ Nz)
            if at_boundary(count) {
                break;
            }
            let Some(at) = end.checked_sub(1) else {
                break;
            };
            let n = data[at];
            end = at;
            for _ in 0..n {
                let Some(start) = end.checked_sub(per) else {
                    return steps;
                };
                steps.push(voxel(&data[start..end]));
                end = start;
            }
            // the span's opening count, then the empty count (VXL layout, glasnost_fmt::vxl); in
            // mode 0 claim 35's (e, b, aₙ₋₁ … a₀, n)
            // spec: S-0105 claim 35 (top-down d = Cᵣ₋₁ + l: the empty count follows the colours, every span, Q-0216)
            let Some(at) = end.checked_sub(2) else {
                break;
            };
            let e = data[at];
            end = at;
            steps.push(Step::Empty(u32::from(e)));
            count = count.wrapping_add(u32::from(n)).wrapping_add(u32::from(e));
        }
    }
    steps
}

/// Column steps with `quirks.voxel_topdown_column_origin` off (S-0105 Edge
/// cases): the column's own runs read bottom-up to the end of its data,
/// each voxel at the cell its runs give, cells 0 … *Nz* − 1 only, then
/// drawn in the section's direction with the gaps as empty runs. A top-down
/// column of one value per voxel (normal mode 0, claim 35) is read from its
/// end with the grouping of [`column_steps`] and anchored to the decoded
/// column's total height.
fn fixed_column_steps(
    data: &[u8],
    nz: u8,
    top_down: bool,
    per: usize,
    voxel: impl Fn(&[u8]) -> Step,
) -> Vec<Step> {
    let mut cells = Vec::new();
    if top_down && per == 1 {
        // spec: S-0105 claim 35 (switch off: positions anchored to the decoded column's total
        // height; only cells 0 … Nz − 1 contribute)
        // Only whole groups are decoded: a group cut off by the start of the
        // column data has no empty count to anchor it (our engine's choice).
        let (mut end, mut d) = (data.len(), 0u32);
        let mut placed = Vec::new();
        while let Some(at) = end.checked_sub(1) {
            let n = usize::from(data[at]);
            let Some(e_at) = at.checked_sub(n + 2) else {
                break;
            };
            for l in 0..n {
                placed.push((d, voxel(&data[at - 1 - l..at - l])));
                d += 1;
            }
            d += u32::from(data[e_at]);
            end = e_at;
        }
        let total = d;
        for (d, v) in placed.into_iter().rev() {
            let cell = total - 1 - d;
            if cell < u32::from(nz) {
                cells.push((cell, v));
            }
        }
    }
    let (mut pos, mut cell) = (0usize, 0u32);
    while !(top_down && per == 1)
        && let (Some(&e), Some(&n)) = (data.get(pos), data.get(pos + 1))
    {
        pos += 2;
        cell += u32::from(e);
        for _ in 0..n {
            let Some(v) = data.get(pos..pos + per) else {
                break;
            };
            if cell < u32::from(nz) {
                cells.push((cell, voxel(v)));
            }
            pos += per;
            cell += 1;
        }
        pos += 1;
    }
    if top_down {
        cells.reverse();
    }
    let mut steps = Vec::new();
    let mut cur = 0u32;
    for (cell, v) in cells {
        let n_z = if top_down {
            u32::from(nz) - 1 - cell
        } else {
            cell
        };
        if n_z > cur {
            steps.push(Step::Empty(n_z - cur));
        }
        steps.push(v);
        cur = n_z + 1;
    }
    steps
}

// ---------------------------------------------------------------------------
// Rasterisation (claims 21–25, F5, F6, F8)
// ---------------------------------------------------------------------------

/// Voxel-image coordinate origin shift, pixels (S-0105 F5, F6).
const IMAGE_ORIGIN: f64 = 128.0; // spec: S-0105 Constants (voxel-image coordinate origin shift), Formulas F5, F6
/// Copy margin on every side, pixels (S-0105 claim 25).
const COPY_MARGIN: i64 = 4; // spec: S-0105 claim 25, Constants (copy margin)
/// Sub-pixel units per pixel (S-0105 F5).
const SUBPIXEL: f64 = 256.0; // spec: S-0105 Formulas F5, Constants (sub-pixel precision)
/// Largest retained side of a part or shadow image, pixels (S-0105 claim
/// 22, F6): a larger copy rectangle keeps only a centred window of this
/// size. No copy rectangle that fits is affected.
pub const IMAGE_LIMIT: i64 = 4096; // spec: S-0105 claim 22, Constants (largest retained image side)
/// The empty depth sentinel, logically −∞: below every numeric depth
/// (S-0105 claim 24).
const EMPTY_DEPTH: i64 = i64::MIN; // spec: S-0105 claims 24, 34 (empty sentinel below all numeric depths)

/// Box corners in the farthest-corner tie order of S-0105 claim 21: the
/// bottom face (max x, max y), (max x, min y), (min x, min y), (min x,
/// max y), then the same on the top face. Bit 0: max x, bit 1: max y, bit
/// 2: max z.
const FAR_CORNER_ORDER: [usize; 8] = [3, 1, 0, 2, 7, 5, 4, 6]; // spec: S-0105 claim 21

/// A finished part or shadow image: its copy rectangle (F6; claim 22, no
/// wrap), and where it lands. Only a copy rectangle larger than
/// [`IMAGE_LIMIT`] is cut to its retained window (claim 22, F6).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartImage {
    /// Width of `pixels`.
    pub width: u32,
    /// Height of `pixels`.
    pub height: u32,
    /// Palette indices, row-major; 0 is transparent (claim 25). Shadow
    /// images hold [`SHADOW_MARK`] where marked.
    pub pixels: Vec<u8>,
    /// Screen position of `pixels`' top-left corner relative to the draw
    /// point (claim 25, F6): *(trunc(c_x) − ⌊w/2⌋, trunc(c_y) − ⌊h/2⌋)*,
    /// moved by any cut to the retained window (claim 22, F6).
    pub offset: [i64; 2],
    /// The copy rectangle's size *w × h* (F6), before any cut.
    pub copy_size: [i64; 2],
}

impl PartImage {
    /// Whether the copy rectangle has a side greater than [`IMAGE_LIMIT`]
    /// (S-0105 claim 22, Import mapping: the importer warns once per such
    /// model).
    pub fn exceeds_limit(&self) -> bool {
        self.copy_size.iter().any(|&s| s > IMAGE_LIMIT)
    }

    /// The pixel at `(x, y)` of the image, if inside.
    pub fn get(&self, x: u32, y: u32) -> Option<u8> {
        (x < self.width && y < self.height).then(|| self.pixels[(y * self.width + x) as usize])
    }

    /// The pixel landing at screen position `(x, y)` relative to the draw
    /// point, if the image covers it.
    pub fn at_screen(&self, x: i64, y: i64) -> Option<u8> {
        let (ix, iy) = (x - self.offset[0], y - self.offset[1]);
        let w = i64::from(self.width);
        let h = i64::from(self.height);
        ((0..w).contains(&ix) && (0..h).contains(&iy)).then(|| self.pixels[(iy * w + ix) as usize])
    }

    /// The non-transparent pixels as screen positions relative to the draw
    /// point, with their values, row by row.
    pub fn opaque(&self) -> Vec<([i64; 2], u8)> {
        let w = self.width as usize;
        self.pixels
            .iter()
            .enumerate()
            .filter(|(_, v)| **v != 0)
            .map(|(i, &v)| {
                (
                    [
                        self.offset[0] + (i % w) as i64,
                        self.offset[1] + (i / w) as i64,
                    ],
                    v,
                )
            })
            .collect()
    }
}

/// Axis-aligned bounds of view-space points (y flipped).
#[derive(Clone, Copy, Debug)]
struct Bounds {
    min: [f32; 3],
    max: [f32; 3],
}

impl Bounds {
    fn of<'a>(points: impl IntoIterator<Item = &'a [f32; 3]>) -> Option<Self> {
        let mut it = points.into_iter();
        let first = *it.next()?;
        let mut b = Self {
            min: first,
            max: first,
        };
        for p in it {
            for ((lo, hi), &v) in b.min.iter_mut().zip(b.max.iter_mut()).zip(p) {
                *lo = lo.min(v);
                *hi = hi.max(v);
            }
        }
        Some(b)
    }

    /// The centre *c*: *(min + max) / 2* at 53-bit precision, stored
    /// truncated (F0; the spec does not fix how the centre is formed).
    fn centre(&self) -> [f32; 3] {
        [0, 1, 2].map(|a| {
            to_f32_rz(div_rz(
                add_rz(f64::from(self.min[a]), f64::from(self.max[a])),
                2.0,
            ))
        })
    }
}

/// `trunc(256 × ((p + 128) − c))` (F5, F10), a signed unwrapped position in
/// 1/256 pixel.
fn origin_units(p: f32, c: f32) -> i64 {
    // spec: S-0105 Formulas F5 (o = trunc(256 × (P₀ + 128 − c)), unwrapped)
    let v = mul_rz(
        SUBPIXEL,
        sub_rz(add_rz(f64::from(p), IMAGE_ORIGIN), f64::from(c)),
    );
    trunc_i64(v)
}

/// `trunc(256 × (to − from) / n)` (F5, F10), unreduced; 0 when `n` is 0.
fn step_units(to: f32, from: f32, n: u8) -> i64 {
    if n == 0 {
        return 0;
    }
    // spec: S-0105 Formulas F5 (Δ = trunc(256 × (P_ξ − P₀) / N_ξ))
    trunc_i64(div_rz(
        mul_rz(SUBPIXEL, sub_rz(f64::from(to), f64::from(from))),
        f64::from(n),
    ))
}

/// `o + Σ nᵢ · Δᵢ` in exact signed integer arithmetic (F5, F10), saturating
/// only where a malformed model leaves the 64-bit range.
fn position(o: i64, terms: &[(i64, i64)]) -> i64 {
    // spec: S-0105 Formulas F5 (exact signed integer arithmetic, no wrap, no carry)
    terms
        .iter()
        .fold(o, |acc, &(n, d)| acc.saturating_add(n.saturating_mul(d)))
}

/// The image pixel or depth level of a position in 1/256 pixel:
/// *⌊p / 256⌋*, toward negative infinity (F5, F10).
fn pixel_of(p: i64) -> i64 {
    // spec: S-0105 Formulas F5 (floors toward negative infinity, negative coordinates included)
    p.div_euclid(256)
}

/// The sections of a part from lowest to highest overlap priority (claim
/// 23, F8): `depths[s]` is the nearest view-space depth of section `s`
/// (file order); the result lists sections in drawing order. F8's
/// relation: the lowest priority is the last strict record low of the
/// sequence, and the rest take their priorities from the residual sequence
/// (the first position omitted, each later record low replaced by its
/// predecessor among the record lows). Each pass of the loop below picks
/// that lowest section and leaves the residual in the remaining places.
/// Distinct depths ascend, and a part whose sections all share one depth
/// keeps file order. With `quirks.voxel_section_sort_ties` off, a stable
/// ascending sort.
pub fn section_order(depths: &[f32], quirks: &VoxelQuirks) -> Vec<usize> {
    let mut pi: Vec<usize> = (0..depths.len()).collect();
    if !quirks.voxel_section_sort_ties {
        // spec: S-0105 Edge cases (voxel_section_sort_ties off: stable order); F8 (exact
        // comparisons: equal values, −0 and +0 included, tie)
        // (adding +0 turns −0 into +0, keeping a total order for any value)
        pi.sort_by(|&a, &b| (depths[a] + 0.0).total_cmp(&(depths[b] + 0.0)));
        return pi;
    }
    // spec: S-0105 F8 (question 7's relation: the last strict record low has the lowest
    // priority; the residual sequence, first position omitted and each later record low
    // replaced by its predecessor in R, orders the rest; exact single-precision comparisons)
    for t in 0..pi.len() {
        let mut chain = vec![t];
        let mut min = depths[pi[t]];
        for (i, &s) in pi.iter().enumerate().skip(t + 1) {
            if depths[s] < min {
                chain.push(i);
                min = depths[s];
            }
        }
        let last = pi[chain[chain.len() - 1]];
        for m in (1..chain.len()).rev() {
            pi[chain[m]] = pi[chain[m - 1]];
        }
        pi[t] = last;
    }
    pi
}

/// A copy rectangle in logical image pixels (F6): corner *(x₀, y₀)* and
/// size *w × h*.
type CopyRect = [i64; 4];

/// The copy rectangle of F6 for the overall bounding box `bounds`.
fn copy_rect(bounds: &Bounds) -> CopyRect {
    // spec: S-0105 Formulas F6 (w₀ = trunc(x_max − x_min), h₀ = trunc(y_max − y_min))
    let w0 = trunc_i64(sub_rz(f64::from(bounds.max[0]), f64::from(bounds.min[0])));
    let h0 = trunc_i64(sub_rz(f64::from(bounds.max[1]), f64::from(bounds.min[1])));
    // spec: S-0105 claim 25, Formulas F6 (w = w₀ + 8, h = h₀ + 8)
    let w = w0.saturating_add(2 * COPY_MARGIN);
    let h = h0.saturating_add(2 * COPY_MARGIN);
    // spec: S-0105 Formulas F6 (corner (128 − ⌊w₀/2⌋ − 4, 128 − ⌊h₀/2⌋ − 4))
    let origin = IMAGE_ORIGIN as i64;
    let x0 = origin
        .saturating_sub(w0.div_euclid(2))
        .saturating_sub(COPY_MARGIN);
    let y0 = origin
        .saturating_sub(h0.div_euclid(2))
        .saturating_sub(COPY_MARGIN);
    [x0, y0, w, h]
}

/// One fresh image being drawn (S-0105 claims 22, 24, 34). Logical image
/// coordinates are signed and unbounded (claim 22, F5); the colour of the
/// copy rectangle (F6), the only part that reaches the screen, is stored,
/// cut to its retained window (claim 22, F6). Every coordinate
/// starts transparent and, in depth mode, with the empty depth (claim 34).
///
/// In depth mode the comparison depth is kept for every coordinate whose
/// depth can still change a stored pixel: rows of the stored region, left
/// of its right edge. A right-hand write reaches one column to the right
/// (claim 24), so depths left of the region can decide what a later voxel
/// writes into its first column.
#[derive(Clone, Debug)]
struct Raster {
    /// The F6 copy rectangle, before any cut.
    copy: CopyRect,
    /// The stored region: columns `x0 .. x1`, rows `y0 .. y1`.
    x0: i64,
    y0: i64,
    x1: i64,
    y1: i64,
    /// Colour of the stored region, row-major; 0 is transparent.
    colour: Vec<u8>,
    /// Comparison depths written in depth mode, by `(y, x)`; a missing entry
    /// is empty (claim 24).
    depth: BTreeMap<(i64, i64), i64>,
}

impl Raster {
    /// A fresh image for the overall bounding box `bounds` (claim 34: fully
    /// transparent, every depth empty).
    fn new(bounds: &Bounds) -> Self {
        let copy = copy_rect(bounds);
        let [x0, y0, w, h] = copy;
        // spec: S-0105 claim 22, Formulas F6 (retained window: u = min(w, 4096), v = min(h, 4096),
        // corner (a + ⌊(w − u)/2⌋, b + ⌊(h − v)/2⌋); an odd extra pixel is discarded on the right
        // or bottom)
        let (u, v) = (w.min(IMAGE_LIMIT), h.min(IMAGE_LIMIT));
        let sx0 = x0.saturating_add((w - u).div_euclid(2));
        let sy0 = y0.saturating_add((h - v).div_euclid(2));
        let (sx1, sy1) = (sx0.saturating_add(u), sy0.saturating_add(v));
        let len = ((sx1 - sx0) * (sy1 - sy0)) as usize;
        Self {
            copy,
            x0: sx0,
            y0: sy0,
            x1: sx1,
            y1: sy1,
            // spec: S-0105 claim 34 (every fresh draw starts completely transparent)
            colour: vec![0; len],
            // spec: S-0105 claims 24, 34 (every coordinate starts empty)
            depth: BTreeMap::new(),
        }
    }

    /// The colour index of a stored pixel.
    fn index(&self, x: i64, y: i64) -> Option<usize> {
        ((self.x0..self.x1).contains(&x) && (self.y0..self.y1).contains(&y))
            .then(|| ((y - self.y0) * (self.x1 - self.x0) + (x - self.x0)) as usize)
    }

    /// Writes `value` at `(x, y)`; pixels outside the stored region never
    /// reach the screen (claims 22, 25).
    fn put(&mut self, x: i64, y: i64, value: u8) {
        if let Some(i) = self.index(x, y) {
            self.colour[i] = value;
        }
    }

    /// Whether the depth at `(x, y)` can matter to a stored pixel.
    fn depth_kept(&self, x: i64, y: i64) -> bool {
        (self.y0..self.y1).contains(&y) && x < self.x1
    }

    /// The comparison depth at `(x, y)`, [`EMPTY_DEPTH`] if none was written.
    fn depth_at(&self, x: i64, y: i64) -> i64 {
        self.depth.get(&(y, x)).copied().unwrap_or(EMPTY_DEPTH)
    }

    /// A depth-mode write of `value` and `depth` at `(x, y)`.
    fn put_depth(&mut self, x: i64, y: i64, value: u8, depth: i64) {
        self.put(x, y, value);
        if self.depth_kept(x, y) {
            self.depth.insert((y, x), depth);
        }
    }

    /// The finished image: the stored region of the copy rectangle, placed
    /// around the draw point by F6.
    fn finish(self, centre: [f32; 3]) -> PartImage {
        let [x0, y0, w, h] = self.copy;
        // spec: S-0105 claim 25, Formulas F6 (lands at (trunc(c_x) − ⌊w/2⌋, trunc(c_y) − ⌊h/2⌋))
        let ox = trunc_i64(f64::from(centre[0])).saturating_sub(w.div_euclid(2));
        let oy = trunc_i64(f64::from(centre[1])).saturating_sub(h.div_euclid(2));
        PartImage {
            width: (self.x1 - self.x0) as u32,
            height: (self.y1 - self.y0) as u32,
            pixels: self.colour,
            offset: [
                ox.saturating_add(self.x0.saturating_sub(x0)),
                oy.saturating_add(self.y0.saturating_sub(y0)),
            ],
            copy_size: [w, h],
        }
    }
}

/// Draws one part as a fresh image and returns it (S-0105 claims 13–15,
/// 18–25, 34, 35, 36, F4–F6, F8).
///
/// - `frame`: the part's HVA frame (claim 16), taken modulo the frame
///   count (claim 13).
/// - `draw`: the part's draw transform, view included (from
///   [`vehicle_parts`], [`building_turret_parts`], [`building_barrel_only`]
///   or [`view`]).
/// - `vpl`: the voxel lighting table; without it lit voxels write their
///   raw colour index.
/// - `depth_buffered`: whether this draw is made in the depth-buffered mode
///   (claim 24): exactly when its type has the depth option (claim 32).
///
/// Every draw starts fully transparent with every depth empty (claim 34),
/// so no earlier draw affects it.
///
/// Returns `None` for a model without sections, and for a part whose
/// placement is not finite (claim 22: not drawn).
pub fn draw_model(
    model: &VoxelModel,
    frame: u32,
    draw: &Mat34,
    vpl: Option<&Vpl>,
    depth_buffered: bool,
    quirks: &VoxelQuirks,
) -> Option<PartImage> {
    draw_with(model, frame, draw, vpl, depth_buffered, quirks, None)
}

/// [`draw_model`] with one fixed shade table for every section instead of
/// S-0105 claim 18's: a voxel projectile's table of
/// [`projectile_shade_table`] (S-0306 claim 30), whose rasterising is
/// otherwise S-0105's (claims 19, 21 to 23).
pub fn draw_model_shaded(
    model: &VoxelModel,
    frame: u32,
    draw: &Mat34,
    vpl: Option<&Vpl>,
    shades: &[u8; 256],
    quirks: &VoxelQuirks,
) -> Option<PartImage> {
    draw_with(model, frame, draw, vpl, false, quirks, Some(shades))
}

fn draw_with(
    model: &VoxelModel,
    frame: u32,
    draw: &Mat34,
    vpl: Option<&Vpl>,
    depth_buffered: bool,
    quirks: &VoxelQuirks,
    fixed: Option<&[u8; 256]>,
) -> Option<PartImage> {
    let sections = &model.sections;
    // spec: S-0105 claim 13 (each section drawn with M · H(f, i))
    let transforms: Vec<Mat34> = (0..sections.len())
        .map(|i| draw.then(&model.section_transform(frame, i)))
        .collect();
    // View-space box corners, y flipped (claim 25, F5).
    let corners: Vec<[[f32; 3]; 8]> = sections
        .iter()
        .zip(&transforms)
        .map(|(s, g)| {
            std::array::from_fn(|k| {
                let p = g.apply(s.corner(k));
                [p[0], -p[1], p[2]]
            })
        })
        .collect();
    // spec: S-0105 claim 22 (a part whose placement inputs, section bounds, transforms or derived
    // projection, are not finite is not drawn)
    if !transforms.iter().all(Mat34::is_finite)
        || !corners.iter().flatten().flatten().all(|v| v.is_finite())
    {
        return None;
    }
    let bounds = Bounds::of(corners.iter().flatten())?;
    let centre = bounds.centre();
    if !centre.iter().all(|v| v.is_finite()) {
        return None;
    }

    // Shade tables (claim 18).
    let shared = quirks.voxel_light_first_section_mode && quirks.voxel_light_ignores_hva;
    let first_mode = sections[0].normal_mode;
    let table_for = |i: usize| {
        // spec: S-0105 claim 18 (quirks.voxel_light_ignores_hva,
        // quirks.voxel_light_first_section_mode)
        let rot = if quirks.voxel_light_ignores_hva {
            draw
        } else {
            &transforms[i]
        };
        let mode = if quirks.voxel_light_first_section_mode {
            first_mode
        } else {
            sections[i].normal_mode
        };
        shade_table(rot, mode, quirks)
    };
    let shared = shared || fixed.is_some();
    let shades: Vec<[u8; 256]> = if let Some(t) = fixed {
        vec![*t]
    } else if shared {
        vec![table_for(0)]
    } else {
        (0..sections.len()).map(table_for).collect()
    };

    // spec: S-0105 claim 23 (nearest depth: largest z of the box)
    let near: Vec<f32> = corners
        .iter()
        .map(|c| c.iter().fold(f32::NEG_INFINITY, |m, p| m.max(p[2])))
        .collect();
    let mut order = section_order(&near, quirks);
    if depth_buffered {
        // spec: S-0105 claim 24, F8 (the depth-mode contributing order is exactly the reverse
        // of the painter priority)
        order.reverse();
    }
    let mut img = Raster::new(&bounds);
    for s in order {
        let table = &shades[if shared { 0 } else { s }];
        raster_section(
            &sections[s],
            &corners[s],
            centre,
            table,
            vpl,
            depth_buffered,
            quirks,
            &mut img,
        );
    }
    Some(img.finish(centre))
}

/// [`draw_model`] from a parsed VXL and HVA (lossy for claims 30 and 35,
/// see [`Section::from_limb`]).
pub fn draw_part(
    vxl: &Vxl,
    hva: Option<&Hva>,
    frame: u32,
    draw: &Mat34,
    vpl: Option<&Vpl>,
    depth_buffered: bool,
    quirks: &VoxelQuirks,
) -> Option<PartImage> {
    draw_model(
        &VoxelModel::new(vxl, hva, quirks),
        frame,
        draw,
        vpl,
        depth_buffered,
        quirks,
    )
}

/// Rasterises one section (claims 15, 21, 22, 24, F5).
#[allow(clippy::too_many_arguments)]
fn raster_section(
    sec: &Section,
    corners: &[[f32; 3]; 8],
    centre: [f32; 3],
    shades: &[u8; 256],
    vpl: Option<&Vpl>,
    depth_buffered: bool,
    quirks: &VoxelQuirks,
    img: &mut Raster,
) {
    let [sx, sy, sz] = sec.size;
    let (nx, ny) = (usize::from(sx), usize::from(sy));
    // spec: S-0105 claim 21 (farthest corner: smallest z, ties in list order)
    let mut far = FAR_CORNER_ORDER[0];
    for &k in &FAR_CORNER_ORDER[1..] {
        if corners[k][2] < corners[far][2] {
            far = k;
        }
    }
    let p0 = corners[far];
    // spec: S-0105 Formulas F5 (o and the per-axis steps Δ_x, Δ_y, Δ_z)
    let o: [i64; 3] = [0, 1, 2].map(|a| origin_units(p0[a], centre[a]));
    let step = |axis_bit: usize, n: u8| -> [i64; 3] {
        let p = corners[far ^ axis_bit];
        [0, 1, 2].map(|a| step_units(p[a], p0[a], n))
    };
    let (dx, dy, dz) = (step(1, sx), step(2, sy), step(4, sz));
    // spec: S-0105 claim 21 (bottom-up when the farthest corner is on the
    // bottom face, top-down when it is on the top face)
    let top_down = far & 4 != 0;
    // spec: S-0105 claim 21 (ascending (n_y, n_x, n_z))
    for n_y in 0..ny {
        let j = if far & 2 != 0 { ny - 1 - n_y } else { n_y };
        for n_x in 0..nx {
            let i = if far & 1 != 0 { nx - 1 - n_x } else { n_x };
            let Some(data) = sec.columns.get(j * nx + i).and_then(Option::as_deref) else {
                continue;
            };
            let mut n_z: i64 = 0;
            for st in column_steps(data, sz, top_down, sec.normal_mode, depth_buffered, quirks) {
                match st {
                    Step::Empty(e) => {
                        // spec: S-0105 Formulas F5 (empty cells advance n_z)
                        n_z = n_z.saturating_add(i64::from(e));
                    }
                    Step::Voxel { value, normal } => {
                        // spec: S-0105 Formulas F5 (p = o + n_x Δ_x + n_y Δ_y + n_z Δ_z per
                        // component, exact, no wrap, no carry into y)
                        let p: [i64; 3] = [0, 1, 2].map(|a| {
                            position(
                                o[a],
                                &[(n_y as i64, dy[a]), (n_x as i64, dx[a]), (n_z, dz[a])],
                            )
                        });
                        // spec: S-0105 claim 22, Formulas F5 (pixel (⌊px / 256⌋, ⌊py / 256⌋))
                        let (x, y) = (pixel_of(p[0]), pixel_of(p[1]));
                        let colour = match normal {
                            // spec: S-0105 claims 18, 19
                            Some(n) => lit_colour(vpl, shades[usize::from(n)], value),
                            None => value,
                        };
                        if depth_buffered {
                            // spec: S-0105 claim 24 (comparison depth ⌊pz / 256⌋, unwrapped)
                            let depth = pixel_of(p[2]);
                            // spec: S-0105 claim 24 (passes an empty coordinate or a strictly
                            // smaller stored depth; only the left pixel is tested)
                            if depth > img.depth_at(x, y) {
                                img.put_depth(x, y, colour, depth);
                                // spec: S-0105 claim 24 (quirks.voxel_depth_double_width;
                                // the right-hand pixel is (x + 1, y), never another row)
                                if quirks.voxel_depth_double_width {
                                    img.put_depth(x.saturating_add(1), y, colour, depth);
                                }
                            }
                        } else {
                            // spec: S-0105 claims 21, 22 (one pixel; later voxels overwrite)
                            img.put(x, y, colour);
                        }
                        // spec: S-0105 Formulas F5 (Δ_z per voxel drawn before)
                        n_z = n_z.saturating_add(1);
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Shadows (claims 29–31, F10)
// ---------------------------------------------------------------------------

/// The shadow's screen shift, *−6 × L_x* stored truncated (S-0105 claim 31,
/// F10).
#[allow(clippy::excessive_precision)] // the spec's decimals, which identify the single exactly
pub const SHADOW_SHIFT: f32 = 2.9999964; // spec: S-0105 claim 31, Formulas F10, Constants (shadow shift)
/// Shadow scale of aircraft-like types (S-0105 claim 31).
pub const AIRCRAFT_SHADOW_SCALE: f64 = 0.5; // spec: S-0105 claim 31, Constants
/// The one value every marked shadow pixel takes (S-0105 F10 says only
/// that it is one value; 1 is our choice).
pub const SHADOW_MARK: u8 = 1;

/// Draws the shadow footprint of section `shadow_section` as a fresh image
/// (S-0105 claims 29–31, 34, F10): every column the file lists as present
/// marks two pixels at its lower corner on the section's bottom face,
/// shifted 3 pixels right, half size for aircraft-like types. `shadow_draw`
/// is *V · S* (for a vehicle on flat ground, [`VehiclePose::shadow`]); the
/// shadow section's HVA transform of frame 0 is applied here (`body_frame`
/// is used only with `quirks.shadow_frame_zero` off). The image lands
/// around the shadow draw point. It is the unmasked footprint; claim 39's
/// own-image exclusion is [`exclude_own_image`]. Cloak state and the
/// no-shadow option (claim 31) are the caller's to check. A shadow is never
/// drawn in the depth-buffered mode (claim 32). Returns `None` if the
/// section does not exist or the shadow's placement is not finite (claim
/// 22: not drawn).
pub fn draw_shadow(
    model: &VoxelModel,
    shadow_section: usize,
    shadow_draw: &Mat34,
    body_frame: u32,
    aircraft_like: bool,
    quirks: &VoxelQuirks,
) -> Option<PartImage> {
    let sec = model.sections.get(shadow_section)?;
    // spec: S-0105 claim 29 (quirks.shadow_frame_zero)
    let frame = if quirks.shadow_frame_zero {
        0
    } else {
        body_frame
    };
    // spec: S-0105 claim 30, Formulas F10 (G = V · S · H(0, shadow section))
    let g = shadow_draw.then(&model.section_transform(frame, shadow_section));
    // spec: S-0105 claim 31, Formulas F10 (μ = 1/2 for aircraft-like types)
    let mu = if aircraft_like {
        AIRCRAFT_SHADOW_SCALE
    } else {
        1.0
    };
    // spec: S-0105 Formulas F10 (the corners of the section's bottom face)
    footprint(sec, &g, mu, sec.min[2], quirks)
}

/// A debris piece's shadow footprint (S-0317 claim 27, F11): S-0105 F10
/// for `section` with the corners of its bottom face at height 0 turned by
/// the piece's rotation `rotation` alone (no view, no section or tilt
/// transform), a scale of 1 and S-0105 claim 31's shift. Returns `None` if
/// the placement is not finite (S-0105 claim 22: not drawn).
pub fn draw_piece_shadow(
    section: &Section,
    rotation: &Mat34,
    quirks: &VoxelQuirks,
) -> Option<PartImage> {
    // spec: S-0317 F11 (K̄ = (Kₓ, K_y, 0); G K replaced by R K̄; μ = 1)
    footprint(section, rotation, 1.0, 0.0, quirks)
}

/// S-0105 F10's footprint of `sec` under `g`, scale `mu`, with the
/// bottom-face corners at model height `z`.
fn footprint(sec: &Section, g: &Mat34, mu: f64, z: f32, quirks: &VoxelQuirks) -> Option<PartImage> {
    let q = |xmax: bool, ymax: bool| -> [f32; 3] {
        let k = [
            if xmax { sec.max[0] } else { sec.min[0] },
            if ymax { sec.max[1] } else { sec.min[1] },
            z,
        ];
        let gk = g.apply(k);
        // spec: S-0105 Formulas F10 (Q = (μ·x + 2.9999964, −μ·y))
        [
            to_f32_rz(add_rz(
                mul_rz(mu, f64::from(gk[0])),
                f64::from(SHADOW_SHIFT),
            )),
            to_f32_rz(-mul_rz(mu, f64::from(gk[1]))),
            0.0,
        ]
    };
    let (q00, q10, q01, q11) = (
        q(false, false),
        q(true, false),
        q(false, true),
        q(true, true),
    );
    // spec: S-0105 claim 22 (a shadow whose placement inputs are not finite is not drawn)
    if !g.is_finite() || ![q00, q10, q01, q11].iter().flatten().all(|v| v.is_finite()) {
        return None;
    }
    let bounds = Bounds::of([&q00, &q10, &q01, &q11])?;
    let centre = bounds.centre();
    if !centre.iter().all(|v| v.is_finite()) {
        return None;
    }
    let [sx, sy, _] = sec.size;
    // spec: S-0105 Formulas F10 (o, Δ_i, Δ_j)
    let o = [0, 1].map(|a| origin_units(q00[a], centre[a]));
    let di = [0, 1].map(|a| step_units(q10[a], q00[a], sx));
    let dj = [0, 1].map(|a| step_units(q01[a], q00[a], sy));
    // spec: S-0105 claim 34 (a fresh shadow starts completely transparent)
    let mut img = Raster::new(&bounds);
    let (nx, ny) = (usize::from(sx), usize::from(sy));
    for j in 0..ny {
        for i in 0..nx {
            // spec: S-0105 claim 30 (present columns only, voxels or not)
            if sec.columns.get(j * nx + i).is_none_or(Option::is_none) {
                continue;
            }
            // spec: S-0105 Formulas F10 (p = o + iΔ_i + jΔ_j per component, exact, no wrap
            // or carry; pixel ⌊p / 256⌋)
            let [x, y] =
                [0, 1].map(|a| pixel_of(position(o[a], &[(i as i64, di[a]), (j as i64, dj[a])])));
            img.put(x, y, SHADOW_MARK);
            // spec: S-0105 claim 30, Formulas F10 (quirks.shadow_double_width; the second
            // pixel is (x + 1, y), never another row)
            if quirks.shadow_double_width {
                img.put(x.saturating_add(1), y, SHADOW_MARK);
            }
        }
    }
    Some(img.finish(centre))
}

/// Excludes a vehicle's own image from its shadow (S-0105 claim 39): every
/// shadow pixel at a screen position (relative to the shared draw point)
/// where any of `parts`, the completed body, turret and barrel images, is
/// non-transparent becomes transparent. Each of a column's two pixels is
/// tested on its own; the scene's depth test plays no part. Applied on
/// every appearance to the unmasked footprint, fresh or reused.
pub fn exclude_own_image(shadow: &mut PartImage, parts: &[&PartImage]) {
    let w = shadow.width as usize;
    for (i, px) in shadow.pixels.iter_mut().enumerate() {
        if *px == 0 {
            continue;
        }
        let x = shadow.offset[0] + (i % w) as i64;
        let y = shadow.offset[1] + (i / w) as i64;
        // spec: S-0105 claim 39 (a non-transparent part pixel at the same screen position excludes the shadow pixel)
        if parts
            .iter()
            .any(|p| p.at_screen(x, y).is_some_and(|v| v != 0))
        {
            *px = 0;
        }
    }
}

// ---------------------------------------------------------------------------
// Fresh draws and the depth-buffered mode (claims 24, 32, 33, 34, S2)
// ---------------------------------------------------------------------------

/// The place every fresh voxel draw goes through (S-0105 claims 24, 32,
/// 33, 34, state table S2).
///
/// Every draw through a canvas is a *fresh* draw: the caller decides,
/// through the image cache (claim 26, [`ImageCache`]), whether a draw is
/// needed at all. Each fresh draw starts completely transparent (claim 34).
/// The depth-buffered mode is on exactly during a fresh part draw of a type
/// with the depth option, which starts with every depth empty; it is off
/// between draws, drawn frames and games, and shadows never turn it on
/// (claims 32, 33, S2).
///
/// Claim 34 lets non-depth draws preserve the retained depths, but the
/// only reads of depth happen inside a depth-mode draw, which first empties
/// every depth. Retained depths are therefore never observable, and the
/// canvas keeps no state between draws: a draw's result never depends on
/// earlier draws. The canvas stays a value so that it can still be carried
/// from one game to the next.
#[derive(Clone, Debug, Default)]
pub struct VoxelCanvas {
    _private: (),
}

impl VoxelCanvas {
    /// A canvas as at program start, the mode off (claim 34, S2).
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the depth-buffered mode is on. Between draws it is always
    /// off (claim 32, S2).
    pub fn depth_mode(&self) -> bool {
        // spec: S-0105 claim 32, S2 (off between draws, drawn frames and games)
        false
    }

    /// A fresh draw of a part (body, turret or barrel) of a type with the
    /// depth option `depth_option` (S-0105 claims 13–25, 32, 34, S2); the
    /// other arguments as for [`draw_model`]. Returns `None` for a model
    /// without sections.
    pub fn draw_part(
        &self,
        model: &VoxelModel,
        frame: u32,
        draw: &Mat34,
        vpl: Option<&Vpl>,
        depth_option: bool,
        quirks: &VoxelQuirks,
    ) -> Option<PartImage> {
        // spec: S-0105 claims 32, 33, S2 (the mode is on only during a fresh
        // depth-option part draw, selected by the part's own type option)
        draw_model(model, frame, draw, vpl, depth_option, quirks)
    }

    /// A fresh draw of a shadow (S-0105 claims 29–32, 34, S2); the
    /// arguments as for [`draw_shadow`]. A shadow never turns the
    /// depth-buffered mode on, whatever its type's depth option (claim 32).
    /// Returns `None` if the section does not exist.
    pub fn draw_shadow(
        &self,
        model: &VoxelModel,
        shadow_section: usize,
        shadow_draw: &Mat34,
        body_frame: u32,
        aircraft_like: bool,
        quirks: &VoxelQuirks,
    ) -> Option<PartImage> {
        // spec: S-0105 claim 32, S2 (shadows never enable depth mode)
        draw_shadow(
            model,
            shadow_section,
            shadow_draw,
            body_frame,
            aircraft_like,
            quirks,
        )
    }
}

// ---------------------------------------------------------------------------
// Image reuse (claims 26, 31, 37)
// ---------------------------------------------------------------------------

/// The ramp state in an image key: which settled tilt the body has
/// (S-0105 claim 26; the tilts are S-0106's). Bodies are drawn on flat
/// ground only, so [`RAMP_FLAT`] is the only value used yet.
pub type RampState = u8;
/// The ramp state of flat ground (our encoding).
pub const RAMP_FLAT: RampState = 0;

/// The key of a turret image, which is also the base of a barrel image's
/// key (S-0105 claims 26, 37). Frames are complete applicable frame
/// numbers, never reduced (lead-confirmed departure).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TurretKey {
    /// A vehicle turret (claim 26).
    Vehicle {
        /// The ramp state.
        ramp: RampState,
        /// The 32-step body facing; `None` when the turret displacement is 0.
        body: Option<u8>,
        /// The 32-step turret facing.
        turret: u8,
        /// The selected turret index of a multi-turret type.
        index: Option<u8>,
        /// The alternative-model state, a component of its own.
        alt: bool,
        /// The complete applicable turret frame.
        frame: u32,
    },
    /// A building turret, or a building's barrel drawn without a turret
    /// (claims 26, 37).
    Building {
        /// The 32-step facing.
        facing: u8,
        /// The complete applicable frame.
        frame: u32,
    },
}

/// The key of a part or shadow image in its type's image cache (S-0105
/// claims 26, 31, 37). Barrel images are kept apart from turret images.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImageKey {
    /// A vehicle body (claim 26).
    Body {
        /// The ramp state.
        ramp: RampState,
        /// The 32-step body facing.
        body: u8,
        /// The alternative-model state.
        alt: bool,
        /// The complete applicable HVA frame.
        frame: u32,
    },
    /// A rocket missile's body (claims 26, 41): the 32-step body heading, the
    /// complete HVA frame and the pitch class, instead of a ramp state.
    RocketBody {
        /// The 32-step body heading.
        body: u8,
        /// The complete applicable HVA frame.
        frame: u32,
        /// The pitch class.
        pitch: RocketPitch,
    },
    /// A flying-locomotor body (claims 26, 42): the 32-step flight drawing
    /// heading, the complete HVA frame, whether speed-dependent pitch is
    /// present and the bank state, as independent components.
    FlyingBody {
        /// The 32-step flight drawing heading.
        heading: u8,
        /// The complete applicable HVA frame.
        frame: u32,
        /// Pitch is present.
        pitched: bool,
        /// The bank state.
        bank: Bank,
    },
    /// A turret (claim 26).
    Turret(TurretKey),
    /// A barrel (claims 26, 37).
    Barrel {
        /// The turret's key (claim 37's key for a barrel without a turret).
        turret: TurretKey,
        /// The current 32-step elevation *e₃₂*.
        elevation: u8,
    },
    /// A shadow (claim 31).
    Shadow {
        /// The ramp state.
        ramp: RampState,
        /// The 32-step body facing.
        body: u8,
    },
    /// A voxel projectile's body (S-0306 claim 30): its 32-step drawn
    /// facing ψ and pitch facing *e*. Ours: the original draws it afresh in
    /// every drawn frame, and the image depends on nothing else.
    Projectile {
        /// The 32-step drawn facing ψ.
        heading: u8,
        /// The 32-step pitch facing *e*.
        pitch: u8,
    },
}

/// A vehicle body's image key: ramp state, 32-step body facing,
/// alternative-model state and complete HVA frame (S-0105 claim 26).
pub fn body_key(ramp: RampState, body_facing: u16, alt: bool, frame: u32) -> ImageKey {
    // spec: S-0105 claim 26 (body key: ramp, 32-step facing, alternative state, complete frame)
    ImageKey::Body {
        ramp,
        body: facing32(body_facing),
        alt,
        frame,
    }
}

/// A rocket body's image key (S-0105 claims 26, 41): 32-step body heading,
/// complete frame and pitch class.
pub fn rocket_body_key(body_facing: u16, frame: u32, pitch: RocketPitch) -> ImageKey {
    // spec: S-0105 claim 26 (rocket body key: the 32-step body heading, complete HVA frame and pitch reuse class of claim 41, no ramp state)
    ImageKey::RocketBody {
        body: facing32(body_facing),
        frame,
        pitch,
    }
}

/// A flying body's image key (S-0105 claims 26, 42): 32-step flight
/// drawing heading, complete frame, pitch presence and bank state.
pub fn flying_body_key(heading: u16, frame: u32, a: &FlightAttitude) -> ImageKey {
    // spec: S-0105 claim 26 (flying body key: the 32-step flight drawing heading, complete applicable HVA frame, the presence of speed-dependent pitch and the bank state as independent components; no ramp state)
    ImageKey::FlyingBody {
        heading: facing32(heading),
        frame,
        pitched: a.pitched,
        bank: a.bank,
    }
}

/// A vehicle turret's key (S-0105 claim 26): ramp state, 32-step body
/// facing (left out when `turret_offset`, the turret displacement, is 0),
/// 32-step turret facing, selected turret index of a multi-turret type,
/// the alternative-model state `alt` and the complete turret frame `frame`
/// (claim 16's turret frame) as separate components.
pub fn vehicle_turret_key(
    ramp: RampState,
    body_facing: u16,
    turret_facing: u16,
    turret_offset: i32,
    index: Option<u8>,
    alt: bool,
    frame: u32,
) -> TurretKey {
    // spec: S-0105 claim 26 (vehicle turret key; state and complete frame apart)
    TurretKey::Vehicle {
        ramp,
        // spec: S-0105 claim 26 (at turret displacement 0 the body facing is left out)
        body: (turret_offset != 0).then(|| facing32(body_facing)),
        turret: facing32(turret_facing),
        index,
        alt,
        frame,
    }
}

/// A building turret's key: its 32-step facing and its complete frame
/// (S-0105 claim 26). No elevation.
pub fn building_turret_key(turret_facing: u16, frame: u32) -> TurretKey {
    // spec: S-0105 claim 26 (building turret key: 32-step facing, complete frame)
    TurretKey::Building {
        facing: facing32(turret_facing),
        frame,
    }
}

/// A barrel's key: the turret's key plus the current 32-step elevation
/// (S-0105 claims 6, 26).
pub fn barrel_key(turret: TurretKey, elevation_raw: u16) -> ImageKey {
    // spec: S-0105 claims 6, 26 (barrel key: turret key plus current 32-step elevation)
    ImageKey::Barrel {
        turret,
        elevation: facing32(elevation_raw),
    }
}

/// The key of a building's barrel drawn without a turret: the 32-step
/// facing of claim 5, its complete frame and its current 32-step
/// elevation, kept with the barrel images (S-0105 claim 37).
pub fn barrel_only_key(facing: u16, frame: u32, elevation_raw: u16) -> ImageKey {
    // spec: S-0105 claim 37 (image reuse: facing, complete frame, current elevation)
    barrel_key(building_turret_key(facing, frame), elevation_raw)
}

/// A shadow's key: ramp state and 32-step body facing (S-0105 claim 31).
pub fn shadow_key(ramp: RampState, body_facing: u16) -> ImageKey {
    // spec: S-0105 claim 31 (shadow key: ramp, 32-step body facing)
    ImageKey::Shadow {
        ramp,
        body: facing32(body_facing),
    }
}

/// One type's image cache (S-0105 claims 26, 31): finished images by key,
/// shared by every object of the type.
#[derive(Clone, Debug)]
pub struct ImageCache<T> {
    images: BTreeMap<ImageKey, T>,
}

impl<T> Default for ImageCache<T> {
    fn default() -> Self {
        Self {
            images: BTreeMap::new(),
        }
    }
}

impl<T: Clone> ImageCache<T> {
    /// The image kept for `key`, if any.
    pub fn get(&self, key: &ImageKey) -> Option<&T> {
        self.images.get(key)
    }

    /// Keeps `image` for `key`.
    pub fn insert(&mut self, key: ImageKey, image: T) {
        self.images.insert(key, image);
    }

    /// The image for `key`: the kept one if there is one, else `make`'s
    /// (a fresh draw), which is kept. With `disabled` (the type's
    /// cache-disable option, a running recoil cycle, an unsettled tilt) the
    /// image is made fresh and neither looked up nor stored (claim 26).
    pub fn reuse_or(&mut self, key: ImageKey, disabled: bool, make: impl FnOnce() -> T) -> T {
        if disabled {
            // spec: S-0105 claim 26 (made fresh, neither looked up nor stored)
            return make();
        }
        if let Some(img) = self.images.get(&key) {
            return img.clone();
        }
        let img = make();
        self.images.insert(key, img.clone());
        img
    }
}

// ---------------------------------------------------------------------------
// Recoil (claim 27, F7, S1)
// ---------------------------------------------------------------------------

/// Imported recoil values of one part (S-0105 claim 27, Import mapping).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecoilParams {
    /// Travel distance *ℓ*, model units.
    pub travel: i32,
    /// Compress duration *t_c*, ticks.
    pub compress: i32,
    /// Hold duration *t_h*, ticks.
    pub hold: i32,
    /// Recover duration *t_r*, ticks.
    pub recover: i32,
}

impl Default for RecoilParams {
    fn default() -> Self {
        // spec: S-0105 claim 27, Constants (turret recoil defaults)
        Self {
            travel: 2,
            compress: 1,
            hold: 1,
            recover: 1,
        }
    }
}

/// Phase of a recoil cycle with the ticks left in it (S-0105 S1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecoilPhase {
    Idle,
    Compress(i32),
    Hold(i32),
    Recover(i32),
}

/// A building turret's or barrel's recoil cycle (S-0105 claim 27, F7, S1):
/// call [`RecoilCycle::fire`] on each shot and [`RecoilCycle::tick`] once
/// per tick of the cycle. Where the slide enters the pose is not applied by
/// this module (claim 27: the turret moves by *−λ_turret* and the barrel by
/// *−(λ_turret + λ_barrel)* along the turret's forward axis).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RecoilCycle {
    params: RecoilParams,
    phase: RecoilPhase,
    slide: f32,
    compress_step: f32,
}

impl RecoilCycle {
    /// A cycle at rest.
    pub fn new(params: RecoilParams) -> Self {
        Self {
            params,
            phase: RecoilPhase::Idle,
            slide: 0.0,
            compress_step: 0.0,
        }
    }

    /// The recoil slide *λ*, model units.
    pub fn slide(&self) -> f32 {
        self.slide
    }

    /// Whether a cycle is running.
    pub fn is_running(&self) -> bool {
        self.phase != RecoilPhase::Idle
    }

    fn compress_ticks(&self) -> i32 {
        // spec: S-0105 Formulas F7 (t_c′ = max(1, t_c))
        self.params.compress.max(1)
    }

    fn recover_ticks(&self) -> i32 {
        // spec: S-0105 Formulas F7 (t_r′ = max(1, t_r))
        self.params.recover.max(1)
    }

    fn hold_ticks(&self) -> i32 {
        // spec: S-0105 claim 27, Formulas F7 (t_h′ = max(0, t_h): every positive hold holds that many ticks)
        self.params.hold.max(0)
    }

    /// `v` clamped to the closed interval between 0 and the travel *ℓ*
    /// (F7).
    fn clamp(&self, v: f32) -> f32 {
        let travel = self.params.travel as f32;
        // spec: S-0105 Formulas F7 (clamped to the closed interval between 0 and ℓ)
        v.clamp(travel.min(0.0), travel.max(0.0))
    }

    /// A shot: restarts the cycle from the current slide (S-0105 claim 27,
    /// F7, S1). A travel of 0 starts no cycle.
    pub fn fire(&mut self) {
        if self.params.travel == 0 {
            // spec: S-0105 Formulas F7 (zero travel starts no cycle)
            return;
        }
        let travel = f64::from(self.params.travel);
        let tc = f64::from(self.compress_ticks());
        // spec: S-0105 Formulas F7 (s_c = (ℓ − λ_r) / t_c′, 53-bit, stored truncated)
        self.compress_step = to_f32_rz(div_rz(sub_rz(travel, f64::from(self.slide)), tc));
        self.phase = RecoilPhase::Compress(self.compress_ticks());
    }

    /// Advances one tick and returns the slide for that tick (S-0105 F7,
    /// S1).
    pub fn tick(&mut self) -> f32 {
        match self.phase {
            RecoilPhase::Idle => {}
            RecoilPhase::Compress(k) => {
                self.slide = if k > 1 {
                    // spec: S-0105 Formulas F7 (λ + s_c, stored truncated, clamped)
                    self.clamp(to_f32_rz(add_rz(
                        f64::from(self.slide),
                        f64::from(self.compress_step),
                    )))
                } else {
                    // spec: S-0105 Formulas F7 (the last compress tick reaches exactly ℓ)
                    self.params.travel as f32
                };
                self.phase = if k > 1 {
                    RecoilPhase::Compress(k - 1)
                } else if self.hold_ticks() > 0 {
                    // spec: S-0105 S1 (compressing, t_h ≥ 1: holding from the next tick)
                    RecoilPhase::Hold(self.hold_ticks())
                } else {
                    // spec: S-0105 S1 (compressing, t_h ≤ 0: recovering with no hold)
                    RecoilPhase::Recover(self.recover_ticks())
                };
            }
            RecoilPhase::Hold(k) => {
                // spec: S-0105 S1 (holding: λ unchanged for t_h′ ticks)
                self.phase = if k > 1 {
                    RecoilPhase::Hold(k - 1)
                } else {
                    RecoilPhase::Recover(self.recover_ticks())
                };
            }
            RecoilPhase::Recover(k) => {
                if k > 1 {
                    // spec: S-0105 Formulas F7 (s_r = −ℓ / t_r′, 53-bit, stored truncated)
                    let s_r = to_f32_rz(div_rz(
                        -f64::from(self.params.travel),
                        f64::from(self.recover_ticks()),
                    ));
                    // spec: S-0105 Formulas F7 (λ + s_r, stored truncated, clamped)
                    self.slide =
                        self.clamp(to_f32_rz(add_rz(f64::from(self.slide), f64::from(s_r))));
                    self.phase = RecoilPhase::Recover(k - 1);
                } else {
                    // spec: S-0105 Formulas F7, S1 (the last recover tick sets λ to exactly 0)
                    self.slide = 0.0;
                    self.phase = RecoilPhase::Idle;
                }
            }
        }
        self.slide
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::approx_constant)] // spec: S-0105 claim 41 and Import mapping: the angles are the spec's decimals
    use super::*;
    use glasnost_fmt::bytes::F32Bits;
    use glasnost_fmt::vxl::{VxlWriter, synthetic_model};

    fn q() -> VoxelQuirks {
        VoxelQuirks::default()
    }

    /// Raw facing of a 32-step facing.
    fn raw32(f: i32) -> u16 {
        (f.rem_euclid(32) * 2048) as u16
    }

    fn steps_of(x_steps: f64) -> u32 {
        table_step(x_steps * 2.0 * PI / 8192.0, &q())
    }

    /// A section of explicit column data.
    fn section(
        size: [u8; 3],
        min: [f32; 3],
        max: [f32; 3],
        mode: u8,
        columns: Vec<Option<Vec<u8>>>,
    ) -> Section {
        Section {
            size,
            min,
            max,
            scale: 1.0,
            normal_mode: mode,
            columns,
        }
    }

    /// A filled section: every cell a voxel of `colour`, normal `normal`.
    fn filled(
        size: [u8; 3],
        min: [f32; 3],
        max: [f32; 3],
        mode: u8,
        colour: u8,
        normal: u8,
    ) -> Section {
        let col: Vec<(u8, u8, u8)> = (0..size[2]).map(|z| (z, colour, normal)).collect();
        let n = usize::from(size[0]) * usize::from(size[1]);
        section(
            size,
            min,
            max,
            mode,
            vec![Some(encode_column(&col, size[2])); n],
        )
    }

    fn model(sections: Vec<Section>) -> VoxelModel {
        VoxelModel::from_sections(sections, None, &q())
    }

    fn mat(rows: [[f32; 4]; 3]) -> Mat34 {
        Mat34(rows)
    }

    fn hva_of(frames: &[Vec<Mat34>]) -> Hva {
        let limbs = frames[0].len();
        Hva {
            name: [0; 16],
            limb_names: vec![[0; 16]; limbs],
            matrices: frames
                .iter()
                .flatten()
                .map(|m| m.0.map(|r| r.map(F32Bits::new)))
                .collect(),
        }
    }

    /// A table whose every row maps each colour `χ ≠ 0` to `100 + q`.
    fn marker_vpl() -> Vpl {
        let mut t = glasnost_fmt::vpl::synthetic_table(32);
        for (qi, row) in t.rows.iter_mut().enumerate() {
            for v in row.iter_mut().skip(1) {
                *v = 100 + qi as u8;
            }
        }
        t
    }

    // --- Terms ---

    /// spec: S-0105 Terms (32-step and 4-step facings)
    #[test]
    fn coarse_facings() {
        assert_eq!(facing32(0), 0);
        assert_eq!(facing32(1023), 0);
        assert_eq!(facing32(1024), 1);
        assert_eq!(facing32(16384), 8);
        assert_eq!(facing32(65535), 0);
        assert_eq!(facing32(64511), 31);
        assert_eq!(facing4(8191), 0);
        assert_eq!(facing4(8192), 1);
        assert_eq!(facing4(57344), 0);
        assert_eq!(facing4(32768), 2);
    }

    // --- F1 ---

    /// spec: S-0105 Formulas F1 (τ: truncated exact sines, two exceptions)
    #[test]
    fn sine_table() {
        assert_eq!(tau(0), 0.0);
        assert_eq!(tau(2048), 1.0);
        assert_eq!(tau(6144), -1.0);
        assert_eq!(tau(10240 - 1), tau(2047));
        assert_eq!(f64::from(tau(4096)), 1.2246063538223773e-16);
        assert_eq!(f64::from(tau(8192)), -2.4492127076447545e-16);
        for n in 1..4096u32 {
            if n == 2048 {
                continue;
            }
            let exact = (f64::from(n) * PI / 4096.0).sin();
            let t = f64::from(tau(n));
            assert!(t <= exact && t > 0.0, "{n}");
            let up = f64::from(f32::from_bits(tau(n).to_bits() + 1));
            assert!(up > exact, "{n}");
            assert_eq!(tau(n + 4096), -tau(n));
            assert_eq!(tau(n + 8192), tau(n));
        }
    }

    /// spec: S-0105 Formulas F1 (rounding of steps, quirks.table_trig_negative_bias)
    #[test]
    fn table_steps_and_negative_bias() {
        // Non-negative: nearest step, halves up.
        assert_eq!(steps_of(5.3), 5);
        assert_eq!(steps_of(5.6), 6);
        // A whole turn short by less than half a step gives 8191, not 0.
        assert_eq!(steps_of(8191.7), 8191);
        // Negative −(m + f): f < 1/2 → −m.
        assert_eq!(steps_of(-5.3), 8192 - 5);
        // f ≥ 1/2, m ≥ 2 → −(m − 1), two short of the nearest −(m + 1).
        assert_eq!(steps_of(-2.6), 8192 - 1);
        assert_eq!(steps_of(-7.6), 8192 - 6);
        // At f = 1/2 exactly the single-precision factor (just below
        // 16384 / 2π) leaves f just under 1/2: nearest.
        assert_eq!(steps_of(-7.5), 8192 - 7);
        // f ≥ 1/2, m = 1 → −1; m = 0 → +1.
        assert_eq!(steps_of(-1.6), 8192 - 1);
        assert_eq!(steps_of(-0.6), 1);
        // Switch off: nearest.
        let fixed = VoxelQuirks {
            table_trig_negative_bias: false,
            ..q()
        };
        assert_eq!(table_step(-7.6 * 2.0 * PI / 8192.0, &fixed), 8192 - 8);
    }

    /// spec: S-0105 Formulas F1 (facings 0–8 exact, 9–31 two steps short),
    /// Edge cases (table_trig_negative_bias: −89.912° and −44.912°)
    #[test]
    fn facing_rotations_land_two_steps_short() {
        for f in 0..32 {
            let s = i64::from(table_step(body_angle(raw32(f)), &q()));
            let nominal = (-(i64::from(f) - 8) * 256).rem_euclid(8192);
            if f <= 8 {
                assert_eq!(s, nominal, "facing {f}");
            } else {
                assert_eq!(s, (nominal + 2).rem_euclid(8192), "facing {f}");
            }
        }
        // South (16): −2046 steps = −89.912°; south-east (12): −1022 = −44.912°.
        assert_eq!(table_step(body_angle(raw32(16)), &q()), 8192 - 2046);
        assert_eq!(table_step(body_angle(raw32(12)), &q()), 8192 - 1022);
    }

    /// spec: S-0105 Formulas F1 (table square root), F3
    #[test]
    fn table_square_root() {
        assert_eq!(table_sqrt(0.0), 0.0);
        assert_eq!(table_sqrt(4.0), 2.0);
        assert_eq!(table_sqrt(1.0), 1.0);
        #[allow(clippy::excessive_precision)] // the spec's exact decimal (F3)
        let root: f64 = 362.038665771484375;
        assert_eq!(f64::from(table_sqrt(2.0 * 256.0 * 256.0)), root);
        let mut x = 0.37f64;
        for _ in 0..5000 {
            x = (x * 1.75 + 0.123).fract() * 1000.0 + 1e-3;
            let r = f64::from(table_sqrt(x));
            let exact = f64::from(to_f32_rz(x)).sqrt();
            assert!(r <= exact, "{x}");
            assert!((exact - r) / exact < 2f64.powi(-13), "{x}");
        }
    }

    /// spec: S-0105 Formulas F3 (ρ), claim 8
    #[test]
    fn pixels_per_lepton_value() {
        let rho = pixels_per_lepton();
        // The spec's decimal, read to nearest; ours is formed toward zero.
        let spec = 0.16572815467691362f64;
        assert!(rho == spec || rho == f64::from_bits(spec.to_bits() - 1));
        assert_eq!(rho, spec);
        let d = vehicle_turret_shift(20);
        assert_eq!(d, to_f32_rz(20.0 * spec));
        assert!(f64::from(d) <= 20.0 * spec);
    }

    // --- F2 and the view ---

    /// spec: S-0105 claim 1, Formulas F2 (view matrix as tabulated)
    #[test]
    fn view_matrix_as_tabulated() {
        #[allow(clippy::excessive_precision)]
        let expect: [[f32; 3]; 3] = [
            [0.708190620, 0.706021249, 0.0],
            [-0.353166908, 0.354252070, 0.865897536],
            [0.611342013, -0.613220513, 0.500221372],
        ];
        let v = view(&q());
        for (row, want) in v.0.iter().zip(&expect) {
            assert_eq!(&row[..3], want);
            assert_eq!(row[3], 0.0);
        }
        // Under nearest rounding the (z, x) entry would be 0.611342072.
        let (s60, _) = table_sin_cos(view_tilt(), &q());
        let (s45, _) = table_sin_cos(view_turn(), &q());
        #[allow(clippy::excessive_precision)]
        let nearest: f32 = 0.611342072;
        assert_eq!((f64::from(s60) * f64::from(s45)) as f32, nearest);
        // Effective angles 59.985° and 44.912°.
        assert_eq!(table_step(view_tilt(), &q()), 8192 - 1365);
        assert_eq!(table_step(view_turn(), &q()), 8192 - 1022);
    }

    /// spec: S-0105 claim 2 (east is unrotated: +x screen right-and-down,
    /// +y screen right-and-up, +z up)
    #[test]
    fn unrotated_part_axes() {
        let pose = vehicle_parts(16384, 16384, 0, ELEVATION_LEVEL, &q());
        assert_eq!(pose.body, view(&q()));
        let x = pose.body.apply([1.0, 0.0, 0.0]);
        assert!(x[0] > 0.0 && x[1] < 0.0, "{x:?}");
        let y = pose.body.apply([0.0, 1.0, 0.0]);
        assert!(y[0] > 0.0 && y[1] > 0.0, "{y:?}");
        let z = pose.body.apply([0.0, 0.0, 1.0]);
        assert!(z[0] == 0.0 && z[1] > 0.0 && z[2] > 0.0, "{z:?}");
    }

    /// spec: S-0105 claim 3 (32 body facings; north turns +x to where east
    /// has +y)
    #[test]
    fn body_facings() {
        let at = |raw: u16| vehicle_local_transforms(raw, raw, 0, ELEVATION_LEVEL, &q())[0];
        // Facing 0 (north): model +x points map north = model +y of east.
        let north = at(0).apply([1.0, 0.0, 0.0]);
        assert!((north[0]).abs() < 1e-6 && (north[1] - 1.0).abs() < 1e-6);
        // Raw facings within one 32-step bucket give identical transforms.
        assert_eq!(at(2048 * 5 - 1024), at(2048 * 5 + 1023));
        assert_ne!(at(2048 * 5 - 1025), at(2048 * 5));
        let distinct: std::collections::BTreeSet<_> = (0..=u16::MAX)
            .step_by(512)
            .map(|r| format!("{:?}", at(r)))
            .collect();
        assert_eq!(distinct.len(), 32);
    }

    /// spec: S-0105 claims 4, 8 (turret relative to the body, origin d
    /// along the body's forward axis)
    #[test]
    fn vehicle_turret_pose() {
        // Equal 32-step facings: the turret's rotation is the body's.
        let [d, t, _] = vehicle_local_transforms(raw32(13), raw32(13), 0, ELEVATION_LEVEL, &q());
        assert_eq!(d, t);
        // Turret displacement δ: origin at δ × ρ along the body's forward.
        let [d, t, _] = vehicle_local_transforms(raw32(8), raw32(3), 100, ELEVATION_LEVEL, &q());
        let shift = vehicle_turret_shift(100);
        assert_eq!(t.translation(), [shift, 0.0, 0.0]);
        assert_eq!(d.translation(), [0.0; 3]);
        // The turret's forward follows its own facing, up to the 0.088°
        // shortfall of a negative rotation (F1).
        let [_, t3, _] = vehicle_local_transforms(raw32(0), raw32(3), 0, ELEVATION_LEVEL, &q());
        let [b3, _, _] = vehicle_local_transforms(raw32(3), raw32(3), 0, ELEVATION_LEVEL, &q());
        let (a, b) = (t3.apply([1.0, 0.0, 0.0]), b3.apply([1.0, 0.0, 0.0]));
        assert!((a[0] - b[0]).abs() < 2e-3 && (a[1] - b[1]).abs() < 2e-3);
        assert_ne!(a, b);
    }

    /// spec: S-0105 claim 6 (e₃₂ < 8 raises the muzzle; 32-step pitch)
    #[test]
    fn barrel_pitch_sense() {
        let raised = Mat34::IDENTITY.rotate_y(barrel_pitch(14336, &q()), &q());
        assert!(raised.apply([1.0, 0.0, 0.0])[2] > 0.0);
        assert_eq!(barrel_pitch(16384, &q()), 0.0);
        // Within one 32-step bucket the pitch does not change.
        assert_eq!(barrel_pitch(15360, &q()), 0.0);
        let cont = VoxelQuirks {
            barrel_elevation_quantised: false,
            ..q()
        };
        assert!(barrel_pitch(15360, &cont) < 0.0);
    }

    /// spec: S-0105 claims 7, 8, 10 and S-0504 claim 16 (a level rocking
    /// body is the plain one; a rocking one leans its turret and barrel
    /// with it: T = D · X(d) · Rz(θ_rel), B = T · Ry(φ) from F5's D)
    #[test]
    fn rocking_parts_ride_the_body() {
        for offset in [0, 120, -80] {
            let plain = vehicle_parts(raw32(5), raw32(9), offset, 12288, &q());
            let level = Rocking::new(0.0, 0.0, [20, 12, 9]);
            assert!(!level.rocks());
            assert_eq!(
                level.shifts(&q()),
                (0, 0, 0),
                "F5: a level body gets no lift or shift"
            );
            let same = vehicle_parts_rocking(raw32(5), raw32(9), offset, 12288, &level, &q());
            assert_eq!(plain, same);
            let a = Rocking::new(0.3, -0.1, [20, 12, 9]);
            assert!(a.rocks());
            let posed = vehicle_parts_rocking(raw32(5), raw32(9), offset, 12288, &a, &q());
            let d = a.body(raw32(5), &q());
            let t = d
                .translate([vehicle_turret_shift(offset), 0.0, 0.0])
                .rotate_z(turret_relative_angle(raw32(9), raw32(5)), &q());
            let v = view(&q());
            assert_eq!(posed.body, v.then(&d));
            assert_eq!(posed.shadow, posed.body);
            assert_eq!(posed.turret, v.then(&t));
            assert_eq!(
                posed.barrel,
                v.then(&t.rotate_y(barrel_pitch(12288, &q()), &q()))
            );
            assert_ne!(posed.body, plain.body);
        }
    }

    /// spec: S-0504 F5 (W = Ny × 0.5 + 0.5, H = Nx × 0.5; the lift and the
    /// shifts in whole model units, sₓ negated for p < 0 and sᵧ for r > 0;
    /// at the full lean the end that goes down keeps the level body's edge
    /// and the other end rises: a positive pitch lowers the bow)
    #[test]
    fn f5_lift_and_shifts() {
        let full = 0.785_398_2_f32; // S-0504 F4's L
        let r = Rocking::new(full, 0.0, [20, 12, 9]);
        assert_eq!((r.width, r.length), (6.5, 10.0));
        // |sin p| × H = 7.07…, so the lift is 7; sₓ = 10 − trunc(7.07…) = 3.
        assert_eq!(r.shifts(&q()), (7, 3, 0));
        assert_eq!(
            Rocking::new(-full, 0.0, [20, 12, 9]).shifts(&q()),
            (7, -3, 0)
        );
        // A roll: lift adds |sin r| × W; sᵧ = trunc(6.5 − trunc(6.5 cos r)), negated for r > 0.
        let rolled = Rocking::new(0.0, 0.5, [20, 12, 9]);
        assert_eq!(rolled.shifts(&q()), (3, 0, -1));
        assert_eq!(Rocking::new(0.0, -0.5, [20, 12, 9]).shifts(&q()), (3, 0, 1));
        // The outline at the full lean, facing map east (θ_body = 0).
        let east = 16384;
        assert_eq!(body_angle(east), 0.0);
        let d = r.body(east, &q());
        let bow = d.apply([10.0, 0.0, 0.0]);
        let stern = d.apply([-10.0, 0.0, 0.0]);
        assert!(
            (bow[0] - 10.0).abs() < 1.0 && bow[2].abs() < 1.0,
            "bow edge kept: {bow:?}"
        );
        assert!(stern[2] > 13.0, "the stern rises: {stern:?}");
        let down = Rocking::new(-full, 0.0, [20, 12, 9]).body(east, &q());
        let stern = down.apply([-10.0, 0.0, 0.0]);
        assert!(
            (stern[0] + 10.0).abs() < 1.0 && stern[2].abs() < 1.0,
            "stern edge kept: {stern:?}"
        );
        assert!(down.apply([10.0, 0.0, 0.0])[2] > 13.0, "the bow rises");
    }

    /// spec: S-0105 claim 10 (B = T · Ry(φ): the barrel pitches about the
    /// lateral axis through the turret origin, also at a non-zero
    /// displacement; lead-confirmed pivot departure)
    #[test]
    fn barrel_pivot() {
        for offset in [0, 120, -80] {
            let [_, t, b] = vehicle_local_transforms(raw32(8), raw32(11), offset, 12288, &q());
            assert_eq!(b, t.rotate_y(barrel_pitch(12288, &q()), &q()));
            // The turret origin stays put under the pitch.
            assert_eq!(b.apply([0.0; 3]), t.apply([0.0; 3]));
            assert_eq!(b.translation(), t.translation());
            // A point on the barrel's forward axis is pitched.
            assert!((b.apply([10.0, 0.0, 0.0])[2] - t.apply([10.0, 0.0, 0.0])[2]).abs() > 1.0);
        }
        // Buildings: the same pivot through the shifted turret origin.
        let [t, b] = building_local_transforms(raw32(5), 40, 12288, &q());
        assert_eq!(b, t.rotate_y(barrel_pitch(12288, &q()), &q()));
        assert_eq!(b.translation(), t.translation());
        assert_ne!(t.translation(), [0.0; 3]);
    }

    /// spec: S-0105 claim 12 (Barrel pose: a sprite-body vehicle's barrel
    /// sits trunc(δ / 8) model units ahead of the body's origin, 0, 25, −12
    /// and −1 for MS-33 part k's pose runs, never δ × ρ nor rounded down;
    /// turned by the relative turret angle and pitched about claim 10's
    /// axis through the shifted point)
    #[test]
    fn sprite_body_barrel_pose_shifts_in_eighths() {
        for (offset, d) in [(0, 0.0), (200, 25.0), (-100, -12.0), (-12, -1.0)] {
            assert_eq!(sprite_body_barrel_shift(offset), d, "δ = {offset}");
        }
        assert_ne!(sprite_body_barrel_shift(200), vehicle_turret_shift(200));
        for offset in [0, 200, -100, -12] {
            for (body, turret, elevation) in [(5, 9, 12288), (0, 0, ELEVATION_LEVEL), (8, 3, 14336)]
            {
                let pose =
                    sprite_body_barrel_pose(raw32(body), raw32(turret), offset, elevation, &q());
                let d = Mat34::IDENTITY.rotate_z(body_angle(raw32(body)), &q());
                let t = d
                    .translate([sprite_body_barrel_shift(offset), 0.0, 0.0])
                    .rotate_z(turret_relative_angle(raw32(turret), raw32(body)), &q());
                let b = t.rotate_y(barrel_pitch(elevation, &q()), &q());
                assert_eq!(pose, view(&q()).then(&b));
                // The pivot is the shifted point, on the body's forward axis.
                assert_eq!(b.translation(), t.translation());
                let ahead = d.apply([sprite_body_barrel_shift(offset), 0.0, 0.0]);
                let o = t.translation();
                assert!((o[0] - ahead[0]).abs() < 1e-4 && (o[1] - ahead[1]).abs() < 1e-4);
            }
        }
        // Equal facings, level: the barrel turns with the body as a voxel vehicle's barrel does,
        // only the shift differs.
        let plain = vehicle_parts(raw32(4), raw32(4), 0, ELEVATION_LEVEL, &q());
        assert_eq!(
            sprite_body_barrel_pose(raw32(4), raw32(4), 0, ELEVATION_LEVEL, &q()),
            plain.barrel
        );
    }

    /// spec: S-0105 claims 5, 9 (building turret: absolute facing, shift
    /// trunc(δ / 8) along its own forward axis)
    #[test]
    fn building_turret_pose() {
        assert_eq!(building_turret_shift(20, &q()), 2.0);
        assert_eq!(building_turret_shift(-12, &q()), -1.0);
        let rho = VoxelQuirks {
            building_turret_shift_eighths: false,
            ..q()
        };
        assert_eq!(building_turret_shift(20, &rho), vehicle_turret_shift(20));
        let [t, _] = building_local_transforms(16384, 20, ELEVATION_LEVEL, &q());
        assert_eq!(t.translation(), [2.0, 0.0, 0.0]);
        // Facing north (0): the shift points along model +y of the map.
        let [t, _] = building_local_transforms(0, 20, ELEVATION_LEVEL, &q());
        let tr = t.translation();
        assert!(tr[0].abs() < 1e-5 && (tr[1] - 2.0).abs() < 1e-5);
        let pose = building_turret_parts(0, 20, ELEVATION_LEVEL, &q());
        assert_eq!(pose.turret, view(&q()).then(&t));
    }

    /// spec: S-0105 claim 37 (barrel without a turret: V · Rz(θ) · Ry(φ))
    #[test]
    fn barrel_only_pose() {
        let m = building_barrel_only(raw32(20), 12288, &q());
        let expect = view(&q()).then(
            &Mat34::IDENTITY
                .rotate_z(body_angle(raw32(20)), &q())
                .rotate_y(barrel_pitch(12288, &q()), &q()),
        );
        assert_eq!(m, expect);
        assert_eq!(m.translation(), [0.0; 3]);
    }

    /// spec: S-0105 claim 11 (turret first at 4-step facings 1 and 2)
    #[test]
    fn turret_and_barrel_order() {
        for f in 5..=19 {
            assert!(turret_drawn_first(raw32(f)), "{f}");
        }
        for f in (21..=31).chain(0..=3) {
            assert!(!turret_drawn_first(raw32(f)), "{f}");
        }
        let p = vehicle_parts(0, raw32(10), 0, ELEVATION_LEVEL, &q());
        assert!(p.turret_first);
    }

    // --- Aircraft body attitude (claims 41, 42) ---

    /// spec: S-0105 claim 41 (D = Rz(θ_body) · Ry(−p): the single pitch negated and
    /// used directly; positive p raises the nose in every heading; zero pitch is
    /// level; no rounding to 32 pitch facings)
    #[test]
    fn rocket_pitch_raises_the_nose_and_is_not_quantised() {
        let nose = |facing: u16, p: f32| rocket_body_local(facing, p, &q()).apply([1.0, 0.0, 0.0]);
        // Facing 8 is unrotated (east): the nose is +x.
        assert_eq!(nose(raw32(8), 0.0), [1.0, 0.0, 0.0]);
        for f in 0..32 {
            let level = nose(raw32(f), 0.0);
            let up = nose(raw32(f), 0.5);
            assert!(up[2] > 0.4, "heading {f}: nose up");
            assert!((up[0].hypot(up[1]) - 0.8776).abs() < 0.01, "heading {f}");
            assert_eq!(level[2], 0.0);
            assert!(nose(raw32(f), -0.5)[2] < -0.4, "negative p lowers the nose");
        }
        // Not 32 pitch facings: distinct pitches 0.001 apart in the table give
        // distinct poses once a table step apart (0.00077 rad).
        let (a, b) = (nose(raw32(8), 0.5), nose(raw32(8), 0.5015));
        assert_ne!(a, b);
        // The matrix is the product of the heading and the table rotation of −p.
        assert_eq!(
            rocket_body_local(raw32(11), 0.3, &q()),
            Mat34::IDENTITY
                .rotate_z(body_angle(raw32(11)), &q())
                .rotate_y(-f64::from(0.3f32), &q())
        );
    }

    /// spec: S-0105 claim 41 (shadow: V · Rz(θ_body) whatever the pitch; the body
    /// is V · D)
    #[test]
    fn rocket_shadow_is_not_pitched() {
        let level = rocket_body_pose(raw32(5), 0.0, &q());
        let up = rocket_body_pose(raw32(5), 0.7, &q());
        assert_eq!(level.shadow, up.shadow);
        assert_eq!(
            level.shadow,
            vehicle_parts(raw32(5), 0, 0, ELEVATION_LEVEL, &q()).body
        );
        assert_ne!(level.body, up.body);
        assert_eq!(
            level.body, level.shadow,
            "level: the body is the heading alone"
        );
    }

    /// spec: S-0105 claim 41 (reuse classes: zero is level; nonzero equality with
    /// pᵢ = RZ₅₃(Pᵢ × c), compared with the single widened exactly and without
    /// narrowing the product, is initial; otherwise equality with
    /// p𝒇 = RZ₂₄(RZ₅₃(P𝒇 × c)) is final; every other pitch is fresh; coincident
    /// values favour initial except zero)
    #[test]
    fn rocket_pitch_classes() {
        let quirks = q();
        let (pi, pf) = (0.25f32, 0.75f32);
        // The initial comparison does not narrow: a single-precision product that
        // differs from the double one is not initial.
        let p_i = mul_rz(f64::from(pi), QUARTER_TURN);
        let narrowed = to_f32_rz(p_i);
        assert_ne!(
            f64::from(narrowed),
            p_i,
            "the double product is not a single"
        );
        assert_eq!(
            rocket_pitch_class(narrowed, pi, pf, &quirks),
            RocketPitch::Fresh(table_step(-f64::from(narrowed), &quirks))
        );
        // The final value is the narrowed one.
        let p_f = to_f32_rz(mul_rz(f64::from(pf), QUARTER_TURN));
        assert_eq!(rocket_pitch_class(p_f, pi, pf, &quirks), RocketPitch::Final);
        // A pitch equal to the double product widened is initial (here the product
        // is formed by hand, as a stock fraction's rarely is a single).
        let p1 = 0.7853982f32;
        assert_eq!(
            classify_rocket_pitch(p1, f64::from(p1), 9.0, &quirks),
            RocketPitch::Initial
        );
        // Coincident initial and final values: initial.
        assert_eq!(
            classify_rocket_pitch(p1, f64::from(p1), p1, &quirks),
            RocketPitch::Initial
        );
        // Zero is level, even with an initial of 0 or a final of 0 (coincident values).
        assert_eq!(
            classify_rocket_pitch(0.0, 0.0, 0.0, &quirks),
            RocketPitch::Level
        );
        assert_eq!(
            rocket_pitch_class(-0.0, pi, pf, &quirks),
            RocketPitch::Level
        );
        // Any other pitch is fresh, keyed by its table step.
        assert!(matches!(
            rocket_pitch_class(0.123, pi, pf, &quirks),
            RocketPitch::Fresh(_)
        ));
        assert_ne!(
            rocket_pitch_class(0.1, pi, pf, &quirks),
            rocket_pitch_class(0.2, pi, pf, &quirks)
        );
    }

    /// spec: S-0105 claims 26, 41 (the rocket body's key has the heading, the
    /// frame and the pitch class)
    #[test]
    fn rocket_body_keys() {
        let k = |f, frame, c| rocket_body_key(f, frame, c);
        assert_ne!(k(0, 0, RocketPitch::Level), k(0, 0, RocketPitch::Initial));
        assert_ne!(k(0, 0, RocketPitch::Level), k(2048, 0, RocketPitch::Level));
        assert_ne!(k(0, 0, RocketPitch::Level), k(0, 1, RocketPitch::Level));
        // Within one 32-step heading bucket the key is unchanged.
        assert_eq!(k(0, 0, RocketPitch::Level), k(500, 0, RocketPitch::Level));
        assert_ne!(
            k(0, 0, RocketPitch::Level),
            body_key(RAMP_FLAT, 0, false, 0),
            "not a vehicle body key"
        );
    }

    /// spec: S-0105 claim 42 (pitch and bank need strictly positive height and a speed
    /// fraction strictly above the threshold; bank is a discrete state of an unfinished
    /// turn; a zero configured angle has the no-pitch or no-bank state)
    #[test]
    fn flight_attitude_tests() {
        let (pa, ba) = (0.3490658503988659, 0.5235987755982988);
        let at = |h, s, turn| flight_attitude(h, s, turn, 0.25, pa, ba);
        // Level at nonpositive height or speed at or below the threshold.
        for (h, s) in [(0, 1.0), (-5, 1.0), (100, 0.25), (100, 0.0)] {
            let a = at(h, s, 1);
            assert_eq!(
                (a.alpha, a.beta, a.pitched, a.bank),
                (0.0, 0.0, false, Bank::None)
            );
        }
        // Just above the threshold: the configured pitch.
        let a = at(1, 0.2500001, 0);
        assert_eq!(a.alpha, f64::from(to_f32_rz(pa)));
        assert!(a.pitched && a.bank == Bank::None && a.beta == 0.0);
        // Bank by the turn direction, a discrete state.
        let cw = at(100, 1.0, 1);
        let ccw = at(100, 1.0, -1);
        assert_eq!(
            (cw.bank, ccw.bank),
            (Bank::Clockwise, Bank::Counterclockwise)
        );
        assert_eq!(cw.beta, f64::from(to_f32_rz(ba)));
        assert_eq!(ccw.beta, -cw.beta);
        assert_eq!(
            at(100, 1.0, 40).beta,
            cw.beta,
            "not proportional to anything"
        );
        // Zero configured pitch: no pitch state but the bank remains.
        let z = flight_attitude(100, 1.0, 1, 0.25, 0.0, ba);
        assert!(!z.pitched && z.alpha == 0.0 && z.bank == Bank::Clockwise);
        // Zero configured bank: no bank state, the pitch remains.
        let z = flight_attitude(100, 1.0, 1, 0.25, pa, 0.0);
        assert!(z.pitched && z.bank == Bank::None && z.beta == 0.0);
    }

    /// spec: S-0105 claims 8 and 10 (T = D · X(d) · Rz(θ_rel), B = T · Ry(φ) on the body's D):
    /// on the flat-ground D the turret and barrel are vehicle_parts' own; on a flying body's D
    /// they lean with it, and the body and shadow stay as given (#82)
    #[test]
    fn turret_on_body_builds_on_the_given_d() {
        let (body, turret, offset, elevation) = (raw32(5), raw32(9), 40, 14336);
        let flat = vehicle_parts(body, turret, offset, elevation, &q());
        let d = Mat34::IDENTITY.rotate_z(body_angle(body), &q());
        let placed = turret_on_body(
            flying_body_pose(body, &FlightAttitude::LEVEL, &q()),
            &d,
            body,
            turret,
            offset,
            elevation,
            &q(),
        );
        assert_eq!(placed.turret, flat.turret);
        assert_eq!(placed.barrel, flat.barrel);
        assert_eq!(placed.turret_first, flat.turret_first);
        let a = flight_attitude(100, 1.0, 1, 0.25, 0.3490658503988659, 0.5235987755982988);
        let banked = flying_body_pose(raw32(9), &a, &q());
        let on = turret_on_body(
            banked,
            &flying_body_local(raw32(9), &a, &q()),
            raw32(9),
            turret,
            offset,
            elevation,
            &q(),
        );
        assert_eq!((on.body, on.shadow), (banked.body, banked.shadow));
        assert_ne!(
            on.turret, banked.body,
            "the turret is placed, not the body's"
        );
        // Facing the drawn heading, the turret turns with the body alone: its frame is D · X(d).
        let t = flying_body_local(raw32(9), &a, &q())
            .translate([vehicle_turret_shift(offset), 0.0, 0.0])
            .rotate_z(0.0, &q());
        assert_eq!(on.turret, view(&q()).then(&t));
    }

    /// spec: S-0105 claim 42 (D = Rz(θ_flight) · Ry(α) · Rx(β); positive α lowers the
    /// nose, a positive bank raises the +y side; the shadow is the heading alone)
    #[test]
    fn flying_pose() {
        let none = FlightAttitude {
            alpha: 0.0,
            beta: 0.0,
            pitched: false,
            bank: Bank::None,
        };
        let a = flight_attitude(100, 1.0, 1, 0.25, 0.3490658503988659, 0.5235987755982988);
        let d = flying_body_local(raw32(8), &a, &q());
        assert!(
            d.apply([1.0, 0.0, 0.0])[2] < -0.3,
            "positive α lowers the nose"
        );
        assert!(
            d.apply([0.0, 1.0, 0.0])[2] > 0.3,
            "a positive bank raises +y"
        );
        assert_eq!(
            d,
            Mat34::IDENTITY
                .rotate_z(body_angle(raw32(8)), &q())
                .rotate_y(a.alpha, &q())
                .rotate_x(a.beta, &q())
        );
        let level = flying_body_pose(raw32(5), &none, &q());
        let tilted = flying_body_pose(raw32(5), &a, &q());
        assert_eq!(level.shadow, tilted.shadow, "never the pitch or bank");
        assert_eq!(level.body, level.shadow);
        assert_ne!(level.body, tilted.body);
        // The flight drawing heading is the one given, not a body facing.
        assert_ne!(
            flying_body_pose(raw32(5), &none, &q()).body,
            flying_body_pose(raw32(6), &none, &q()).body
        );
        assert_eq!(none, FlightAttitude::LEVEL);
    }

    /// spec: S-0113 claim 20 (D_crash = Rz(θ_flight) · Rx(φ) · Ry(ψ′) drawn with V before the
    /// HVA; no lift or shift; the shadow the heading alone, as level flight's)
    #[test]
    fn crashing_pose() {
        let (roll, pitch) = (1.3f32, 0.4f32);
        let p = crashing_body_pose(raw32(5), roll, pitch, &q());
        let v = view(&q());
        let explicit = Mat34::IDENTITY
            .rotate_z(body_angle(raw32(5)), &q())
            .rotate_x(f64::from(roll), &q())
            .rotate_y(f64::from(pitch), &q());
        assert_eq!(p.body, v.then(&explicit));
        assert_eq!(p.turret, p.body);
        assert_eq!(p.barrel, p.body);
        assert_eq!(
            p.shadow,
            flying_body_pose(raw32(5), &FlightAttitude::LEVEL, &q()).shadow
        );
        assert_ne!(p.body, p.shadow, "the body rolls, the shadow does not");
        // Not S-0504 F5: its lift and shifts move a body this rolled and pitched.
        let r = Rocking::new(pitch, roll, [20, 12, 8]);
        assert_ne!(r.shifts(&q()), (0, 0, 0));
        assert_ne!(p.body, v.then(&r.body(raw32(5), &q())), "no lift or shift");
        // Not the reverse order Rz · Ry(ψ′) · Rx(φ).
        let reverse = Mat34::IDENTITY
            .rotate_z(body_angle(raw32(5)), &q())
            .rotate_y(f64::from(pitch), &q())
            .rotate_x(f64::from(roll), &q());
        assert_ne!(p.body, v.then(&reverse), "roll outermost");
        // The heading turns the body.
        assert_ne!(p.body, crashing_body_pose(raw32(6), roll, pitch, &q()).body);
        // Tiny tumble angles still give D_crash, not the settled level body.
        let tiny = crashing_body_pose(raw32(5), 0.001, 0.001, &q());
        assert_eq!(
            tiny.body,
            v.then(&crashing_body_local(raw32(5), 0.001, 0.001, &q()))
        );
    }

    /// spec: S-0113 claim 20 (ψ′ = ψ + α only when the speed fraction is strictly above the
    /// threshold; the double α is added at 53-bit precision and the sum stored toward zero, the
    /// angle not first narrowed to single precision)
    #[test]
    fn crash_pitch_adds_the_drawing_pitch_on_the_speed_test() {
        let (psi, alpha, threshold) = (0.3f32, 0.35f64, 0.25f64);
        assert_eq!(crash_pitch(psi, 0.25, threshold, alpha), psi, "not above");
        assert_eq!(crash_pitch(psi, 0.2, threshold, alpha), psi);
        let sum = crash_pitch(psi, 0.26, threshold, alpha);
        assert_eq!(sum, to_f32_rz(add_rz(f64::from(psi), alpha)));
        assert!((f64::from(sum) - 0.65).abs() < 1e-6);
        // The precision: α narrowed first gives another single for these values (a sum
        // smaller than α, so its last place is finer than α's single).
        let (psi, alpha) = (-0.3f32, 0.35f64);
        let narrowed_first = to_f32_rz(add_rz(f64::from(psi), f64::from(to_f32_rz(alpha))));
        let wide = crash_pitch(psi, 1.0, 0.0, alpha);
        assert_eq!(wide, to_f32_rz(add_rz(f64::from(psi), alpha)));
        assert_ne!(wide, narrowed_first, "the angle is not narrowed first");
    }

    /// spec: S-0105 claims 26, 42 (the flying body's key: the 32-step flight drawing
    /// heading, the frame, pitch presence and bank state as independent components)
    #[test]
    fn flying_body_keys() {
        let a = |h, s, t| flight_attitude(h, s, t, 0.25, 0.35, 0.52);
        let key = |h, frame, at: &FlightAttitude| flying_body_key(h, frame, at);
        let (level, pitched, cw, ccw) =
            (a(0, 1.0, 0), a(10, 1.0, 0), a(10, 1.0, 1), a(10, 1.0, -1));
        let ks = [
            key(0, 0, &level),
            key(0, 0, &pitched),
            key(0, 0, &cw),
            key(0, 0, &ccw),
            key(2048, 0, &level),
            key(0, 1, &level),
        ];
        let distinct: std::collections::BTreeSet<_> = ks.iter().collect();
        assert_eq!(distinct.len(), ks.len());
        assert_eq!(key(0, 0, &level), key(500, 0, &level));
        // The flying shadow's key is the heading alone (claim 42): the vehicle shadow key.
        assert_eq!(
            shadow_key(RAMP_FLAT, raw32(3)),
            shadow_key(RAMP_FLAT, raw32(3) + 100)
        );
    }

    // --- F9 ---

    /// spec: S-0105 Formulas F9 (example: 16384 → 11264 at rate 3), Terms
    #[test]
    fn turning_example() {
        let got: Vec<u16> = (0..=7).map(|e| turn_facing(16384, 11264, 3, e)).collect();
        assert_eq!(
            got,
            [16382, 15529, 14676, 13823, 12970, 12117, 11264, 11264]
        );
        // A turn shorter than 256 ω completes at once; rate 0 too.
        assert_eq!(turn_facing(100, 800, 3, 0), 800);
        assert_eq!(turn_facing(100, 30000, 0, 0), 30000);
        // Rates above 127 count as 127; half-turns go the negative way.
        assert_eq!(turn_facing(0, 32768, 500, 0), turn_facing(0, 32768, 127, 0));
        assert_eq!(turn_facing(0, 32768, 1, 127), 32768u16.wrapping_add(256));
    }

    /// spec: S-0105 claim 6 (elevation after placement: A = 8 and A = 20)
    #[test]
    fn barrel_elevation_timing() {
        let e32 = |a, t| facing32(barrel_elevation_after(a, t));
        let default: Vec<u8> = (0..=4).map(|t| e32(DEFAULT_FIRE_ANGLE, t)).collect();
        assert_eq!(default, [8, 8, 7, 7, 7]);
        let twenty: Vec<u8> = (0..=7).map(|t| e32(20, t)).collect();
        assert_eq!(twenty, [8, 8, 7, 7, 6, 6, 6, 6]);
        assert_eq!(elevation_target(8 + 256), elevation_target(8));
        assert_eq!(barrel_elevation_after(20, 0), 16382);
    }

    // --- claim 16 ---

    /// spec: S-0105 claim 16 (frames of body, turret, barrel), claim 37
    #[test]
    fn part_frames() {
        assert_eq!(body_frame(41, 40), 1);
        assert_eq!(body_frame(5, 0), 0);
        // f_b ≠ 0: the turret takes f_b itself, not reduced.
        assert_eq!(vehicle_turret_frame(7, 2, Some(3)), 7);
        assert_eq!(vehicle_turret_frame(0, 5, Some(3)), 2);
        assert_eq!(vehicle_turret_frame(0, 5, None), 0);
        assert_eq!(building_turret_frame(7, 3), 1);
        assert_eq!(barrel_frame_with_turret(9, &q()), 0);
        let off = VoxelQuirks {
            barrel_frame_zero: false,
            ..q()
        };
        assert_eq!(barrel_frame_with_turret(9, &off), 9);
        assert_eq!(barrel_only_frame(4, 3), 1);
    }

    // --- Lighting ---

    /// spec: S-0105 claim 17 (light in the model frame of an unrotated part)
    #[test]
    fn light_in_model_frame_of_unrotated_part() {
        let l = light_in_model_frame(&view(&q()));
        let expect = [0.2013, -0.9101, -0.3622];
        for a in 0..3 {
            assert!((l[a] - expect[a]).abs() < 1e-4, "{l:?}");
        }
    }

    /// spec: S-0105 claim 17 (about 4 straight up, about 20 along L′,
    /// highest about 26 toward map south 20° up)
    #[test]
    fn shades_of_reference_directions() {
        let frame = ShadeFrame::new(&view(&q()), &q());
        let up = frame.shade([0.0, 0.0, 1.0]);
        assert!((3..=5).contains(&up), "{up}");
        let along = frame.shade(frame.light);
        assert!((19..=21).contains(&along), "{along}");
        let (s, c) = (20f32.to_radians().sin(), 20f32.to_radians().cos());
        let peak = frame.shade([0.0, -c, s]);
        assert!((25..=27).contains(&peak), "{peak}");
    }

    /// spec: S-0105 claim 18, Formulas F4 (MS-15 and MS-17 shades)
    #[test]
    fn shades_of_table_normals() {
        let v = view(&q());
        let east = vehicle_parts(raw32(8), raw32(8), 0, ELEVATION_LEVEL, &q()).body;
        let south = vehicle_parts(raw32(16), raw32(16), 0, ELEVATION_LEVEL, &q()).body;
        assert_eq!(shade_table(&v, 2, &q())[16], 19);
        assert_eq!(shade_table(&v, 4, &q())[16], 7);
        assert_eq!(shade_table(&east, 4, &q())[85], 26);
        assert_eq!(shade_table(&south, 4, &q())[85], 1);
        // MS-15: the HVA turn of 90° about z; with the light following it
        // (switch off), normal 85 gets 1.
        let turn = mat([
            [0.0, -1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
        ]);
        assert_eq!(shade_table(&v.then(&turn), 4, &q())[85], 1);
    }

    /// spec: S-0105 Constants (normal direction tables)
    #[test]
    fn normal_tables() {
        assert_eq!(normal_table(0).len(), 0);
        assert_eq!(normal_table(1).len(), 16);
        assert_eq!(normal_table(2).len(), 36);
        assert_eq!(normal_table(3).len(), 64);
        assert_eq!(normal_table(4).len(), 245);
        assert_eq!(normal_table(5).len(), 0);
        #[allow(clippy::excessive_precision)]
        {
            assert_eq!(
                NORMALS_MODE1[0],
                [0.549462974, -0.000182999996, -0.835518003]
            );
            assert_eq!(NORMALS_MODE4[85], [0.174285993, -0.892484009, 0.416049004]);
            assert_eq!(NORMALS_MODE4[244], [-0.328188002, 0.140250996, 0.934143007]);
        }
        for t in [1, 2, 3, 4] {
            for n in normal_table(t) {
                let len = n.iter().map(|c| c * c).sum::<f32>();
                assert!((len - 1.0).abs() < 1e-3);
            }
        }
    }

    /// spec: S-0105 claim 20 (253–255 → 16; other out-of-table indices:
    /// the stale shade, 0 here; switch off → 16)
    #[test]
    fn out_of_table_shades() {
        let v = view(&q());
        let t = shade_table(&v, 1, &q());
        assert_eq!(&t[253..], &[16, 16, 16]);
        assert!(t[16..253].iter().all(|&s| s == PROGRAM_START_SHADE));
        let fixed = VoxelQuirks {
            voxel_stale_normal_shades: false,
            ..q()
        };
        assert!(shade_table(&v, 1, &fixed)[16..].iter().all(|&s| s == 16));
        // Mode 0 has no table: every index below 253 is out of it.
        assert!(shade_table(&v, 0, &q())[..253].iter().all(|&s| s == 0));
    }

    /// spec: S-0306 claim 30, F16 (one diffuse term of the fixed light, scaled to 16 and
    /// truncated, no specular term; indices 253 to 255 get 16; the other out-of-table indices
    /// keep the stale shade)
    #[test]
    fn projectile_shades_are_diffuse_only() {
        let m = projectile_body(8, 8, &q());
        let t = projectile_shade_table(&m, 4, &q());
        let l = light_in_model_frame(&m);
        for (i, n) in normal_table(4).iter().enumerate() {
            let d: f64 = (0..3).map(|k| f64::from(n[k]) * f64::from(l[k])).sum();
            let want = if d < 0.0 { 0 } else { (16.0 * d) as u8 };
            assert!(t[i].abs_diff(want) <= 1, "{i}: {} against {want}", t[i]);
        }
        assert!(t.iter().all(|&s| s <= 16));
        assert_eq!(&t[253..], &[16, 16, 16]);
        assert!(t[245..253].iter().all(|&s| s == PROGRAM_START_SHADE));
        // The normal nearest the light gets 15 or 16; one facing away gets 0.
        let best = normal_table(4)
            .iter()
            .enumerate()
            .max_by(|a, b| dot3_rz(*a.1, l).total_cmp(&dot3_rz(*b.1, l)))
            .map(|(i, _)| i)
            .unwrap();
        assert!(t[best] >= 15);
        // S-0105's F4 table adds a specular term: a different table.
        assert_ne!(t, shade_table(&m, 4, &q()));
        let off = VoxelQuirks {
            voxel_stale_normal_shades: false,
            ..q()
        };
        assert!(
            projectile_shade_table(&m, 4, &off)[245..]
                .iter()
                .all(|&s| s == 16)
        );
    }

    /// spec: S-0306 claim 30 (the heading is S-0105 claim 3's body turn; e below 8 raises the
    /// nose: e = 0 straight up, 8 level, 16 straight down)
    #[test]
    fn projectile_body_orientation() {
        for f in 0..32 {
            assert_eq!(
                projectile_body(f as u8, 8, &q()),
                view(&q()).rotate_z(body_angle(raw32(f)), &q()),
                "{f}"
            );
        }
        let nose = |e: u8| projectile_body_local(8, e, &q()).apply([1.0, 0.0, 0.0]);
        assert!(nose(0)[2] > 0.99);
        assert!(nose(16)[2] < -0.99);
        assert!(nose(8)[0] > 0.99);
        assert!(nose(4)[2] > 0.5);
    }

    /// spec: S-0105 claim 18 (quirks.voxel_eye_in_model_frame changes the
    /// highlights)
    #[test]
    fn eye_vector_switch() {
        let v = view(&q());
        let off = VoxelQuirks {
            voxel_eye_in_model_frame: false,
            ..q()
        };
        assert_ne!(shade_table(&v, 4, &q()), shade_table(&v, 4, &off));
    }

    /// spec: S-0105 claims 18, 19 (MS-15: one table from the first
    /// section's mode, light without the HVA; pixels VPL(q, χ))
    #[test]
    fn first_section_mode_and_light_without_hva() {
        let vpl = marker_vpl();
        let v = view(&q());
        let two = |m1, m2| {
            model(vec![
                filled([2, 2, 2], [0.0; 3], [2.0; 3], m1, 9, 16),
                filled([2, 2, 2], [10.0, 0.0, 0.0], [12.0, 2.0, 2.0], m2, 9, 16),
            ])
        };
        let colours = |img: PartImage| {
            img.opaque()
                .into_iter()
                .map(|(_, c)| c)
                .collect::<std::collections::BTreeSet<_>>()
        };
        let first = draw_model(&two(2, 4), 0, &v, Some(&vpl), false, &q()).unwrap();
        assert_eq!(colours(first), [100 + 19].into());
        let second = draw_model(&two(4, 2), 0, &v, Some(&vpl), false, &q()).unwrap();
        assert_eq!(colours(second), [100 + 7].into());
        let per = VoxelQuirks {
            voxel_light_first_section_mode: false,
            ..q()
        };
        let mixed = draw_model(&two(2, 4), 0, &v, Some(&vpl), false, &per).unwrap();
        assert_eq!(colours(mixed), [100 + 7, 100 + 19].into());

        let turn = mat([
            [0.0, -1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
        ]);
        let turned = VoxelModel::from_sections(
            vec![filled([2, 2, 2], [0.0; 3], [2.0; 3], 4, 9, 85)],
            Some(&hva_of(&[vec![turn]])),
            &q(),
        );
        let img = draw_model(&turned, 0, &v, Some(&vpl), false, &q()).unwrap();
        assert_eq!(colours(img), [100 + 26].into());
        let follow = VoxelQuirks {
            voxel_light_ignores_hva: false,
            ..q()
        };
        let img = draw_model(&turned, 0, &v, Some(&vpl), false, &follow).unwrap();
        assert_eq!(colours(img), [100 + 1].into());
        // Without a table, the raw colour index.
        let img = draw_model(&turned, 0, &v, None, false, &q()).unwrap();
        assert_eq!(colours(img), [9].into());
    }

    // --- Sections and HVA ---

    /// spec: S-0105 claim 13 (frame f mod F; the VXL's own transform unused)
    #[test]
    fn hva_frames_and_file_transform() {
        let shift = |x: f32| {
            mat([
                [1.0, 0.0, 0.0, x],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
            ])
        };
        let hva = hva_of(&[vec![shift(0.0)], vec![shift(5.0)], vec![shift(9.0)]]);
        let m = VoxelModel::from_sections(
            vec![filled([1, 1, 1], [0.0; 3], [1.0; 3], 1, 3, 0)],
            Some(&hva),
            &q(),
        );
        assert_eq!(m.frame_count(), 3);
        assert_eq!(m.section_transform(4, 0), m.section_transform(1, 0));
        assert_eq!(m.section_transform(4, 0).translation(), [5.0, 0.0, 0.0]);
        assert_eq!(m.section_transform(0, 7), Mat34::IDENTITY);

        let mut vxl = synthetic_model();
        let a = draw_part(&vxl, None, 0, &view(&q()), None, false, &q()).unwrap();
        vxl.limbs[0].transform[0][3] = F32Bits::new(20.0);
        vxl.limbs[0].transform[0][0] = F32Bits::new(0.0);
        let b = draw_part(&vxl, None, 0, &view(&q()), None, false, &q()).unwrap();
        assert_eq!(a, b);
    }

    /// spec: S-0105 claim 14 (HVA translations × first section's scale,
    /// truncated; quirks.hva_scale_first_section)
    #[test]
    fn hva_translation_scale() {
        let shift = mat([
            [1.0, 0.0, 0.0, 10.0],
            [0.0, 1.0, 0.0, 0.1],
            [0.0, 0.0, 1.0, 0.0],
        ]);
        let mut s1 = filled([1, 1, 1], [0.0; 3], [1.0; 3], 1, 3, 0);
        let mut s2 = s1.clone();
        s1.scale = 1.0;
        s2.scale = 2.0;
        let hva = hva_of(&[vec![Mat34::IDENTITY, shift]]);
        let m = VoxelModel::from_sections(vec![s1.clone(), s2.clone()], Some(&hva), &q());
        assert_eq!(m.section_transform(0, 1).translation()[0], 10.0);
        let own = VoxelQuirks {
            hva_scale_first_section: false,
            ..q()
        };
        let m2 = VoxelModel::from_sections(vec![s1.clone(), s2], Some(&hva), &own);
        assert_eq!(m2.section_transform(0, 1).translation()[0], 20.0);
        // The product is truncated: 0.1 × (1/12) toward zero.
        s1.scale = 1.0 / 12.0;
        let m3 = VoxelModel::from_sections(vec![s1.clone(), s1], Some(&hva), &q());
        let got = m3.section_transform(0, 1).translation()[1];
        assert_eq!(got, to_f32_rz(f64::from(0.1f32) * f64::from(1.0f32 / 12.0)));
    }

    /// spec: S-0105 claims 15, 22 (MS-13: a 10 × 10 × 1 plate stretched to
    /// 30 × 30 × 3 shows exactly 100 pixels with gaps)
    #[test]
    fn stretched_plate_one_pixel_per_voxel() {
        let m = model(vec![filled(
            [10, 10, 1],
            [0.0; 3],
            [30.0, 30.0, 3.0],
            1,
            5,
            0,
        )]);
        let img = draw_model(&m, 0, &view(&q()), None, false, &q()).unwrap();
        assert_eq!(img.opaque().len(), 100);
    }

    /// spec: S-0105 claim 15 (voxel at the corner on the farthest corner's
    /// side), claim 25, Formulas F6 (copy rectangle and placement)
    #[test]
    fn voxel_corner_and_copy_rectangle() {
        // Identity transform: view = model, screen y = −model y. The
        // farthest corner ties on the bottom face: (max x, max y) first.
        let s = section(
            [2, 1, 1],
            [0.0; 3],
            [10.0, 6.0, 1.0],
            1,
            vec![Some(encode_column(&[(0, 7, 0)], 1)), None],
        );
        let img = draw_model(&model(vec![s]), 0, &Mat34::IDENTITY, None, false, &q()).unwrap();
        // w₀ = 10, h₀ = 6: copy rectangle 18 × 14, c = (5, −3).
        assert_eq!(img.copy_size, [18, 14]);
        assert_eq!((img.width, img.height), (18, 14));
        assert_eq!(img.offset, [5 - 9, -3 - 7]);
        // Voxel i = 0 spans x 0 … 5 and is drawn at its max-x, max-y
        // corner: (5, −6) on screen.
        assert_eq!(img.opaque(), vec![([5, -6], 7)]);
    }

    // --- Rasterisation order ---

    /// spec: S-0105 claim 21 (bottom-up when the farthest corner is on the
    /// bottom face, top-down when on the top face; later voxels overwrite)
    #[test]
    fn column_direction() {
        // Both voxels of a 1 × 1 × 2 column land on one pixel when model z
        // does not reach the screen.
        let col = encode_column(&[(0, 10, 0), (1, 11, 0)], 2);
        let s = section([1, 1, 2], [0.0; 3], [1.0; 3], 1, vec![Some(col)]);
        let flat_up = mat([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
        ]);
        let img = draw_model(&model(vec![s.clone()]), 0, &flat_up, None, false, &q()).unwrap();
        assert_eq!(img.opaque().iter().map(|p| p.1).collect::<Vec<_>>(), [11]);
        let flat_down = mat([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, -1.0, 0.0],
        ]);
        let img = draw_model(&model(vec![s]), 0, &flat_down, None, false, &q()).unwrap();
        assert_eq!(img.opaque().iter().map(|p| p.1).collect::<Vec<_>>(), [10]);
    }

    /// spec: S-0105 claim 21 (rows of columns in ascending n_y from the
    /// farthest corner)
    #[test]
    fn row_order() {
        let c = |colour| Some(encode_column(&[(0, colour, 0)], 1));
        let s = section([1, 2, 1], [0.0; 3], [1.0; 3], 1, vec![c(20), c(21)]);
        // Model y goes only into view depth.
        let y_far_min = mat([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
        ]);
        let img = draw_model(&model(vec![s.clone()]), 0, &y_far_min, None, false, &q()).unwrap();
        assert_eq!(img.opaque().iter().map(|p| p.1).collect::<Vec<_>>(), [21]);
        let y_far_max = mat([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0, 0.0],
        ]);
        let img = draw_model(&model(vec![s]), 0, &y_far_max, None, false, &q()).unwrap();
        assert_eq!(img.opaque().iter().map(|p| p.1).collect::<Vec<_>>(), [20]);
    }

    /// spec: S-0105 claim 23, Formulas F8 (A, B, C at depths 2, 2, 1 draw
    /// C, B, A; quirks.voxel_section_sort_ties)
    #[test]
    fn section_order_ties() {
        assert_eq!(section_order(&[2.0, 2.0, 1.0], &q()), [2, 1, 0]);
        assert_eq!(section_order(&[3.0, 1.0, 2.0], &q()), [1, 2, 0]);
        assert_eq!(section_order(&[1.0, 2.0, 3.0], &q()), [0, 1, 2]);
        let stable = VoxelQuirks {
            voxel_section_sort_ties: false,
            ..q()
        };
        assert_eq!(section_order(&[2.0, 2.0, 1.0], &stable), [2, 0, 1]);
        // File order A, B at equal depths draws A, B in both states (claim 23).
        assert_eq!(section_order(&[2.0, 2.0], &q()), [0, 1]);
        assert_eq!(section_order(&[2.0, 2.0], &stable), [0, 1]);
        // F8: exact comparisons, so −0 and +0 tie.
        assert_eq!(section_order(&[0.0, -0.0], &stable), [0, 1]);
        // Distinct depths always ascend (F8).
        let d = [5.0, 1.0, 4.0, 1.0, 3.0, 9.0, 2.0, 2.0];
        let o = section_order(&d, &q());
        assert!(o.windows(2).all(|w| d[w[0]] <= d[w[1]]));
    }

    /// F8's question-7 relation written from its words: `seq` holds
    /// (section, nearest depth) in file order; the result is the overlap
    /// priority, lowest first.
    fn f8_priority(seq: &[(usize, f32)]) -> Vec<usize> {
        let Some(&(_, d0)) = seq.first() else {
            return Vec::new();
        };
        // The strict record lows: the first position and every later one
        // strictly below all before it.
        let mut r = vec![0];
        let mut min = d0;
        for (i, &(_, d)) in seq.iter().enumerate().skip(1) {
            if d < min {
                r.push(i);
                min = d;
            }
        }
        let lowest = seq[r[r.len() - 1]].0;
        let residual: Vec<(usize, f32)> = (1..seq.len())
            .map(|i| match r.iter().position(|&x| x == i) {
                Some(k) => seq[r[k - 1]],
                None => seq[i],
            })
            .collect();
        let mut out = vec![lowest];
        out.extend(f8_priority(&residual));
        out
    }

    /// The pair winner: the section later in the priority.
    fn pair_winner(order: &[usize], a: usize, b: usize) -> usize {
        let pa = order.iter().position(|&s| s == a).unwrap();
        let pb = order.iter().position(|&s| s == b).unwrap();
        if pa > pb { a } else { b }
    }

    /// Checks `pairs`, written as in S-0105's tables ("AB:B AC:A …"),
    /// against the drawing order.
    fn check_pairs(order: &[usize], pairs: &str) {
        let n = |c: u8| usize::from(c - b'A');
        for p in pairs.split_whitespace() {
            let b = p.as_bytes();
            assert_eq!(
                pair_winner(order, n(b[0]), n(b[1])),
                n(b[3]),
                "{p} in {order:?}"
            );
        }
    }

    /// spec: S-0105 claims 23 and 24, Formulas F8 and MS-25 (the expected pair winners of
    /// all fourteen rows and the common winners B,A,C,D,B,C,A,C,B,C,A,A,A,D: the later
    /// section of each pair in the priority, the last at the common pixel, in both modes)
    #[test]
    fn section_order_ms25_pair_winners() {
        let rows: [(&[f32], &str); 14] = [
            (&[2.0, 2.0], "AB:B"),
            (&[2.0, 2.0, 1.0], "AB:A AC:A BC:B"),
            (&[2.0, 2.0, 2.0], "AB:B AC:C BC:C"),
            (&[2.0; 4], "AB:B AC:C AD:D BC:C BD:D CD:D"),
            (&[2.0, 2.0, 1.0, 1.0], "AB:B AC:A AD:A BC:B BD:B CD:D"),
            (&[1.0, 2.0, 2.0], "AB:B AC:C BC:C"),
            (&[2.0, 1.0, 1.0], "AB:A AC:A BC:C"),
            (&[2.0, 1.0, 2.0], "AB:A AC:C BC:C"),
            (&[1.0, 2.0, 1.0], "AB:B AC:C BC:B"),
            (&[1.0, 1.0, 2.0], "AB:B AC:C BC:C"),
            (&[2.0, 2.0, 2.0, 1.0], "AB:A AC:A AD:A BC:C BD:B CD:C"),
            (&[2.0, 1.0, 1.0, 1.0], "AB:A AC:A AD:A BC:C BD:D CD:D"),
            (&[2.0, 1.0, 2.0, 1.0], "AB:A AC:A AD:A BC:C BD:D CD:C"),
            (&[2.0, 1.0, 1.0, 2.0], "AB:A AC:A AD:D BC:C BD:D CD:D"),
        ];
        let common = "BACDBCACBCAAAD".as_bytes();
        for (i, (depths, pairs)) in rows.into_iter().enumerate() {
            let o = section_order(depths, &q());
            check_pairs(&o, pairs);
            assert_eq!(
                o[o.len() - 1],
                usize::from(common[i] - b'A'),
                "row {}",
                i + 1
            );
        }
        // Five sections at one depth: the later file section wins every pair,
        // E the common pixel.
        let o = section_order(&[4.0; 5], &q());
        assert_eq!(o, [0, 1, 2, 3, 4]);
        // A = B alone gives B; A = B > C gives A at the A/B probe, whatever
        // C draws there: the order reads depths only, never pixels.
        assert_eq!(pair_winner(&section_order(&[2.0, 2.0], &q()), 0, 1), 1);
        assert_eq!(pair_winner(&section_order(&[2.0, 2.0, 1.0], &q()), 0, 1), 0);
    }

    /// spec: S-0105 claims 23 and 24, Formulas F8 and MS-28 (the four larger mixed-depth rows:
    /// their priorities, every pair winner, the common winners C,B,A,B, unchanged under changed
    /// gaps; stable file order would differ at row 2 B/D, row 3 A/E and row 4 B/E)
    #[test]
    fn section_order_ms28_rows() {
        let rows: [(&[f32], &[usize], &str); 4] = [
            // A = C > B = E > D: D, B, E, A, C
            (
                &[3.0, 2.0, 3.0, 1.0, 2.0],
                &[3, 1, 4, 0, 2],
                "AB:A AC:C AD:A AE:A BC:C BD:B BE:E CD:C CE:C DE:E",
            ),
            // B = D > A = E > C: C, A, E, D, B
            (
                &[2.0, 3.0, 1.0, 3.0, 2.0],
                &[2, 0, 4, 3, 1],
                "AB:B AC:A AD:D AE:E BC:B BD:B BE:B CD:D CE:E DE:D",
            ),
            // A = E > C = F > B = D: B, D, C, F, E, A
            (
                &[3.0, 1.0, 2.0, 1.0, 3.0, 2.0],
                &[1, 3, 2, 5, 4, 0],
                "AB:A AC:A AD:A AE:A AF:A BC:C BD:D BE:E BF:F CD:C CE:E CF:F DE:E DF:F EF:E",
            ),
            // B = E > A = C > D = F: D, F, A, C, E, B
            (
                &[2.0, 3.0, 2.0, 1.0, 3.0, 1.0],
                &[3, 5, 0, 2, 4, 1],
                "AB:B AC:C AD:A AE:E AF:A BC:B BD:B BE:B BF:B CD:C CE:E CF:C DE:E DF:F EF:E",
            ),
        ];
        let stable = VoxelQuirks {
            voxel_section_sort_ties: false,
            ..q()
        };
        for (i, (depths, priority, pairs)) in rows.into_iter().enumerate() {
            let o = section_order(depths, &q());
            assert_eq!(o, priority, "row {}", i + 1);
            check_pairs(&o, pairs);
            // Changed unequal gaps keep every winner.
            let spread: Vec<f32> = depths.iter().map(|d| d * d * 13.0 + 0.25).collect();
            assert_eq!(section_order(&spread, &q()), priority, "row {} gaps", i + 1);
            // The stable rule is not this relation.
            let (a, b) = [(1, 3), (0, 4), (1, 4)][i.max(1) - 1];
            if i > 0 {
                assert_ne!(
                    pair_winner(&section_order(depths, &stable), a, b),
                    pair_winner(&o, a, b),
                    "row {}",
                    i + 1
                );
            }
        }
        let common: Vec<usize> = rows.iter().map(|r| r.1[r.1.len() - 1]).collect();
        assert_eq!(common, [2, 1, 0, 1], "C, B, A, B");
    }

    /// spec: S-0105 Formulas F8 (question 7: the record-low relation defines a unique priority
    /// for every finite sequence, depending only on exact comparisons: every pattern of up to
    /// six sections with up to six depth levels, and near-equal distinct depths)
    #[test]
    fn section_order_is_f8_relation() {
        for n in 0..=6usize {
            let total = n.pow(n as u32).max(1);
            for code in 0..total {
                let mut c = code;
                let depths: Vec<f32> = (0..n)
                    .map(|_| {
                        let v = c % n.max(1);
                        c /= n.max(1);
                        v as f32
                    })
                    .collect();
                let seq: Vec<(usize, f32)> = depths.iter().copied().enumerate().collect();
                assert_eq!(
                    section_order(&depths, &q()),
                    f8_priority(&seq),
                    "{depths:?}"
                );
            }
        }
        // Exact comparison, no tolerance: one unit in the last place apart.
        let a = 1.0f32;
        let b = f32::from_bits(a.to_bits() + 1);
        assert_eq!(section_order(&[b, a], &q()), [1, 0]);
        assert_eq!(section_order(&[a, a], &q()), [0, 1]);
    }

    /// spec: S-0105 claims 23 and 24, Formulas F8 (every row of the overlap-priority table,
    /// r10's eight added rows included: painter priority, and the depth-mode contributing order
    /// as its reverse; the result depends only on the exact relations, not the gaps; an
    /// all-equal part of any size in file order)
    #[test]
    fn section_order_f8_table() {
        let rows: [(&[f32], &[usize], &[usize]); 14] = [
            (&[2.0, 2.0], &[0, 1], &[1, 0]),
            (&[2.0, 2.0, 1.0], &[2, 1, 0], &[0, 1, 2]),
            (&[2.0, 2.0, 2.0], &[0, 1, 2], &[2, 1, 0]),
            (&[2.0, 2.0, 2.0, 2.0], &[0, 1, 2, 3], &[3, 2, 1, 0]),
            (&[2.0, 2.0, 1.0, 1.0], &[2, 3, 0, 1], &[1, 0, 3, 2]),
            (&[1.0, 2.0, 2.0], &[0, 1, 2], &[2, 1, 0]),
            // r10 (Q-0215 questions 1 to 6)
            (&[2.0, 1.0, 1.0], &[1, 2, 0], &[0, 2, 1]),
            (&[2.0, 1.0, 2.0], &[1, 0, 2], &[2, 0, 1]),
            (&[1.0, 2.0, 1.0], &[0, 2, 1], &[1, 2, 0]),
            (&[1.0, 1.0, 2.0], &[0, 1, 2], &[2, 1, 0]),
            (&[2.0, 2.0, 2.0, 1.0], &[3, 1, 2, 0], &[0, 2, 1, 3]),
            (&[2.0, 1.0, 1.0, 1.0], &[1, 2, 3, 0], &[0, 3, 2, 1]),
            (&[2.0, 1.0, 2.0, 1.0], &[1, 3, 2, 0], &[0, 2, 3, 1]),
            (&[2.0, 1.0, 1.0, 2.0], &[1, 2, 0, 3], &[3, 0, 2, 1]),
        ];
        for (depths, painter, depth_mode) in rows {
            let mut o = section_order(depths, &q());
            assert_eq!(o, painter, "{depths:?}");
            o.reverse();
            assert_eq!(o, depth_mode, "{depths:?}");
            // r10: only the exact relations count, not the gaps between depths.
            let spread: Vec<f32> = depths.iter().map(|d| d * d * 40.0 - 7.5).collect();
            assert_eq!(section_order(&spread, &q()), painter, "{spread:?}");
        }
        for n in 1..9 {
            let all: Vec<usize> = (0..n).collect();
            assert_eq!(section_order(&vec![7.0; n], &q()), all);
        }
    }

    /// spec: S-0105 claim 21 (r11: within a section, of the voxels on one
    /// pixel the greatest (n_y, n_x, n_z) supplies it without depth
    /// buffering; with it, equal comparison depths favour the smallest
    /// tuple under claim 24's strict test)
    #[test]
    fn within_section_pixel_priority() {
        // One column of two cells; model z feeds neither screen position
        // nor depth, so both cells land on one pixel at one depth.
        let col = encode_column(&[(0, 50, 0), (1, 51, 0)], 2);
        let s = section([1, 1, 2], [0.0; 3], [1.0, 1.0, 2.0], 1, vec![Some(col)]);
        let m = model(vec![s]);
        let flat = mat([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
        ]);
        let painter = draw_model(&m, 0, &flat, None, false, &q()).unwrap();
        assert_eq!(
            painter.opaque().iter().map(|p| p.1).collect::<Vec<_>>(),
            [51]
        );
        let depth = draw_model(&m, 0, &flat, None, true, &q()).unwrap();
        // Two-pixel output in depth mode; both show the smaller tuple.
        assert!(!depth.opaque().is_empty());
        assert!(depth.opaque().iter().all(|p| p.1 == 50));
    }

    /// spec: S-0105 claim 23 (the section with the farthest nearest point
    /// first; later sections overwrite, no depth test)
    #[test]
    fn later_section_overwrites() {
        let near = filled([1, 1, 1], [0.0, 0.0, 5.0], [1.0, 1.0, 6.0], 1, 30, 0);
        let far = filled([1, 1, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 1, 31, 0);
        // Model z goes only into view depth, so both land on one pixel.
        let m = model(vec![near, far]);
        let flat = mat([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
        ]);
        let img = draw_model(&m, 0, &flat, None, false, &q()).unwrap();
        assert_eq!(img.opaque().iter().map(|p| p.1).collect::<Vec<_>>(), [30]);
    }

    /// spec: S-0105 claim 24, Formulas F5 (independent coordinates: a
    /// negative x step never carries into y, in either mode;
    /// lead-confirmed carry departure)
    #[test]
    fn no_x_carry_into_y() {
        // y bounds 0 … 2/256 put the row at 255/256 of a pixel.
        let c = |colour| Some(encode_column(&[(0, colour, 0)], 1));
        let s = section(
            [2, 1, 1],
            [0.0; 3],
            [2.0, 2.0 / 256.0, 1.0],
            1,
            vec![c(40), c(41)],
        );
        let m = model(vec![s]);
        let mut got = draw_model(&m, 0, &Mat34::IDENTITY, None, false, &q())
            .unwrap()
            .opaque();
        got.sort();
        // Voxel i = 0 (one negative x step from the farthest corner) stays
        // in the row of voxel i = 1 (no step).
        assert_eq!(got, vec![([1, -1], 40), ([2, -1], 41)]);
        let depth = draw_model(&m, 0, &Mat34::IDENTITY, None, true, &q()).unwrap();
        assert!(depth.opaque().iter().all(|p| p.0[1] == -1));
    }

    // --- Depth-buffered mode ---

    /// spec: S-0105 claim 24 (two pixels per voxel; reversed order; equal
    /// depth: the first drawn stays — MS-12 equal depths show only Y)
    #[test]
    fn depth_buffered_mode() {
        let single = model(vec![filled([1, 1, 1], [0.0; 3], [1.0; 3], 1, 50, 0)]);
        let v = view(&q());
        assert_eq!(
            draw_model(&single, 0, &v, None, false, &q())
                .unwrap()
                .opaque()
                .len(),
            1
        );
        let img = draw_model(&single, 0, &v, None, true, &q()).unwrap();
        let px = img.opaque();
        assert_eq!(px.len(), 2);
        assert_eq!(px[1].0, [px[0].0[0] + 1, px[0].0[1]]);
        let one = VoxelQuirks {
            voxel_depth_double_width: false,
            ..q()
        };
        assert_eq!(
            draw_model(&single, 0, &v, None, true, &one)
                .unwrap()
                .opaque()
                .len(),
            1
        );

        let x = filled([2, 2, 2], [0.0; 3], [8.0; 3], 1, 60, 0);
        let mut y = x.clone();
        for c in y.columns.iter_mut().flatten() {
            *c = encode_column(&[(0, 61, 0), (1, 61, 0)], 2);
        }
        let twins = model(vec![x, y]);
        let img = draw_model(&twins, 0, &v, None, true, &q()).unwrap();
        assert!(img.opaque().iter().all(|p| p.1 == 61));
        // Under claim 23's order X is drawn first; keeping the first drawn
        // would show X, keeping the last drawn (reversed order) would show X.
        // Without the mode, Y (drawn second) overwrites.
        let img = draw_model(&twins, 0, &v, None, false, &q()).unwrap();
        assert!(img.opaque().iter().all(|p| p.1 == 61));
        let mut reversed_last = twins.clone();
        reversed_last.sections.swap(0, 1);
        let img = draw_model(&reversed_last, 0, &v, None, true, &q()).unwrap();
        assert!(img.opaque().iter().all(|p| p.1 == 60));
    }

    /// spec: S-0105 claim 24 (only the left pixel's depth is tested: the
    /// right-hand pixel is overwritten even if it held a greater depth)
    #[test]
    fn depth_right_pixel_overwritten() {
        // Identity view: near section A (z 10 … 11) at x = 2, far section B
        // (z 0 … 1) at x = 1. Nearest first: A is drawn, then B, whose
        // left pixel is free and whose right pixel is A's.
        let a = filled([1, 1, 1], [1.0, 0.0, 10.0], [2.0, 1.0, 11.0], 1, 80, 0);
        let b = filled([1, 1, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 1, 81, 0);
        let img = draw_model(&model(vec![a, b]), 0, &Mat34::IDENTITY, None, true, &q()).unwrap();
        let mut got: Vec<(i64, u8)> = img.opaque().into_iter().map(|(p, c)| (p[0], c)).collect();
        got.sort();
        assert_eq!(got, vec![(1, 81), (2, 81), (3, 80)]);
    }

    /// spec: S-0105 claims 22, 24, Formulas F5 (unwrapped signed depths:
    /// depth 0 and negative depths pass empty coordinates; a part wider
    /// than 256 pixels keeps every voxel; lead-confirmed depth departure)
    #[test]
    fn depth_zero_and_negative_depths_drawn() {
        // Model z maps to view x and view z; cell k sits at x = z = 2k,
        // c = 150, so cell 11 has depth 0, cells 0 … 10 negative depths.
        let col: Vec<(u8, u8, u8)> = (0..150).map(|z| (z, 70, 0)).collect();
        let s = section(
            [1, 1, 150],
            [0.0; 3],
            [1.0, 1.0, 300.0],
            1,
            vec![Some(encode_column(&col, 150))],
        );
        let m = model(vec![s]);
        let tilt = mat([
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
        ]);
        let img = draw_model(&m, 0, &tilt, None, true, &q()).unwrap();
        assert_eq!(img.copy_size[0], 308);
        let mut xs: Vec<i64> = img.opaque().iter().map(|p| p.0[0]).collect();
        xs.sort();
        // Screen x = 2k for cell k and its right-hand pixel 2k + 1: every
        // cell, from depth −22 to 276, with nothing wrapped.
        assert_eq!(xs, (0..300).collect::<Vec<_>>());
    }

    /// spec: S-0105 claim 24 (depths are compared unwrapped: depth 300
    /// stays in front of depth 44 in either drawing order)
    #[test]
    fn depth_order_never_wraps() {
        let near = filled([1, 1, 1], [0.0, 0.0, 300.0], [1.0, 1.0, 301.0], 1, 90, 0);
        let far = filled([1, 1, 1], [0.0, 0.0, 44.0], [1.0, 1.0, 45.0], 1, 91, 0);
        for m in [
            model(vec![near.clone(), far.clone()]),
            model(vec![far.clone(), near.clone()]),
        ] {
            let img = draw_model(&m, 0, &Mat34::IDENTITY, None, true, &q()).unwrap();
            let values: Vec<u8> = img.opaque().iter().map(|p| p.1).collect();
            assert_eq!(values, [90, 90]);
        }
    }

    /// spec: S-0105 claim 24 (the right-hand pixel of a depth-mode voxel is
    /// (x + 1, y) at every column, including 255, never the next row)
    #[test]
    fn depth_right_pixel_never_wraps_to_the_next_row() {
        // A row of 300 one-voxel columns: image columns well past 255.
        let n = 150u8;
        let col = Some(encode_column(&[(0, 77, 0)], 1));
        let s = section(
            [n, 1, 1],
            [0.0; 3],
            [300.0, 1.0, 1.0],
            1,
            vec![col; usize::from(n)],
        );
        let img = draw_model(&model(vec![s]), 0, &Mat34::IDENTITY, None, true, &q()).unwrap();
        let px = img.opaque();
        let row = px[0].0[1];
        assert!(px.iter().all(|p| p.0[1] == row), "{px:?}");
        let mut xs: Vec<i64> = px.iter().map(|p| p.0[0]).collect();
        xs.sort();
        xs.dedup();
        // Voxel i at 2i: pixels 2i and 2i + 1, 300 in one row.
        assert_eq!(xs.len(), 300);
        assert_eq!(xs[299] - xs[0], 299);
    }

    // --- Normal mode 0 and malformed columns ---

    /// Pixels of a single-column model drawn with model z up the screen,
    /// listed bottom to top: (screen y, value).
    fn column_pixels(s: Section, top_down: bool, quirks: &VoxelQuirks) -> Vec<(i64, u8)> {
        let z = if top_down { -1.0 } else { 1.0 };
        let up = mat([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, z, 0.0],
        ]);
        let img = draw_model(&model(vec![s]), 0, &up, Some(&marker_vpl()), false, quirks).unwrap();
        let mut v: Vec<(i64, u8)> = img.opaque().into_iter().map(|(p, c)| (p[1], c)).collect();
        v.sort_by_key(|p| std::cmp::Reverse(p.0));
        v
    }

    /// spec: S-0105 claim 35 (MS-19 (a): mode 0 unlit, one value per voxel)
    #[test]
    fn mode0_one_value_per_voxel() {
        let (x, yv) = (90, 9);
        let data = vec![0, 2, x, yv, x, yv, 2];
        let s0 = section(
            [1, 1, 2],
            [0.0; 3],
            [1.0, 1.0, 2.0],
            0,
            vec![Some(data.clone())],
        );
        let got: Vec<u8> = column_pixels(s0.clone(), false, &q())
            .iter()
            .map(|p| p.1)
            .collect();
        assert_eq!(got, [x, yv]);
        // Mode 1 twin: both voxels lit versions of X (marker rows ≥ 100).
        let mut s1 = s0.clone();
        s1.normal_mode = 1;
        let got: Vec<u8> = column_pixels(s1, false, &q()).iter().map(|p| p.1).collect();
        assert_eq!(got.len(), 2);
        assert!(got.iter().all(|&c| c >= 100));
        // Switch off: colour and normal per voxel, colour unlit.
        let fixed = VoxelQuirks {
            voxel_mode0_misread: false,
            ..q()
        };
        let got: Vec<u8> = column_pixels(s0, false, &fixed)
            .iter()
            .map(|p| p.1)
            .collect();
        assert_eq!(got, [x, x]);
    }

    /// spec: S-0105 claim 35 (MS-19 (b): bottom-up and top-down groups; top-down Y at cell
    /// e + 1 and X at cell e, r10)
    #[test]
    fn mode0_groups_both_directions() {
        let (x, yv, z) = (90, 9, 7);
        let data = vec![40, 2, 40, z, x, yv, 2];
        let s = section([1, 1, 42], [0.0; 3], [1.0, 1.0, 42.0], 0, vec![Some(data)]);
        // Bottom-up: cells 40 and 41 show 40 and Z.
        let up = column_pixels(s.clone(), false, &q());
        assert_eq!(up.iter().map(|p| p.1).collect::<Vec<_>>(), [40, z]);
        assert_eq!(up[0].0 - up[1].0, 1);
        // Top-down: cell 41 shows Y and cell 40 shows X.
        let down = column_pixels(s, true, &q());
        assert_eq!(down.iter().map(|p| p.1).collect::<Vec<_>>(), [x, yv]);
    }

    /// A mode-0 column's steps as (grid cell, value): counted up from the
    /// bottom face bottom-up, down from cell Nz − 1 top-down.
    fn mode0_cells(
        data: &[u8],
        nz: u8,
        top_down: bool,
        depth: bool,
        q: &VoxelQuirks,
    ) -> Vec<(i64, u8)> {
        let mut d = 0i64;
        let mut out = Vec::new();
        for st in column_steps(data, nz, top_down, 0, depth, q) {
            match st {
                Step::Empty(e) => d += i64::from(e),
                Step::Voxel { value, .. } => {
                    out.push((if top_down { i64::from(nz) - 1 - d } else { d }, value));
                    d += 1;
                }
            }
        }
        out
    }

    /// spec: S-0105 claim 35 (MS-26's general mode-0 columns, invented values: spans, the
    /// prefix boundary and the bounded-data stop in both drawing directions; r10, Q-0216: the
    /// two-span top-down row at cells 4, 3, 2, 1 and the three-span row with no first-span
    /// exception)
    #[test]
    fn mode0_general_columns() {
        let q = q();
        let up = |d: &[u8], nz| mode0_cells(d, nz, false, false, &q);
        let down = |d: &[u8], nz| mode0_cells(d, nz, true, false, &q);
        assert_eq!(
            up(&[0, 2, 10, 11, 90, 1, 2, 20, 21, 91], 5),
            [(0, 10), (1, 11), (3, 20), (4, 21)]
        );
        // MS-26's top-down two-span row (Q-0216's discriminator).
        assert_eq!(
            down(&[1, 91, 21, 20, 2, 0, 90, 11, 10, 2], 5),
            [(4, 10), (3, 11), (2, 20), (1, 21)]
        );
        // The three-span row with nonzero empty counts.
        assert_eq!(
            down(&[1, 92, 30, 1, 2, 91, 21, 20, 2, 1, 90, 10, 1], 8),
            [(7, 10), (5, 20), (4, 21), (1, 30)]
        );
        assert_eq!(up(&[0, 2, 10, 11, 90, 0, 1, 20, 91], 2), [(0, 10), (1, 11)]);
        assert_eq!(
            down(&[0, 91, 20, 1, 0, 90, 11, 10, 2], 2),
            [(1, 10), (0, 11)]
        );
        assert_eq!(up(&[0, 3, 10, 11, 12, 90], 2), [(0, 10), (1, 11), (2, 12)]);
        assert_eq!(
            down(&[0, 90, 12, 11, 10, 3], 2),
            [(1, 10), (0, 11), (-1, 12)]
        );
        assert_eq!(up(&[0, 1, 10, 90], 3), [(0, 10)]);
        assert_eq!(down(&[0, 90, 10, 1], 3), [(2, 10)]);
    }

    /// spec: S-0105 claims 35, 36 and Edge cases (quirks.voxel_topdown_column_origin: the
    /// boundary is ≡ Nz modulo 256, modulo 4294967296 in the depth-buffered mode; MS-26's
    /// 260-colour column in a four-cell grid with a span appended)
    #[test]
    fn prefix_boundary_modulus() {
        let mut data = vec![0, 255];
        data.extend(std::iter::repeat_n(10, 255));
        data.extend([90, 0, 5]);
        data.extend(std::iter::repeat_n(11, 5));
        data.extend([90, 0, 1, 77, 90]);
        let q = q();
        let normal = mode0_cells(&data, 4, false, false, &q);
        assert_eq!(normal.len(), 260);
        assert!(normal.iter().all(|c| c.1 != 77));
        let depth = mode0_cells(&data, 4, false, true, &q);
        assert_eq!(depth.len(), 261);
        assert_eq!(depth[260], (260, 77));
        // The modulus is the same for every normal mode: two-value voxels here.
        let mut pairs = vec![0, 255];
        pairs.extend(std::iter::repeat_n([10, 0], 255).flatten());
        pairs.extend([255, 0, 5]);
        pairs.extend(std::iter::repeat_n([11, 0], 5).flatten());
        pairs.extend([5, 0, 1, 77, 0, 1]);
        let voxels = |depth| {
            column_steps(&pairs, 4, false, 1, depth, &q)
                .iter()
                .filter(|s| matches!(s, Step::Voxel { .. }))
                .count()
        };
        assert_eq!((voxels(false), voxels(true)), (260, 261));
    }

    /// spec: S-0105 claim 35 (with quirks.voxel_topdown_column_origin off a top-down mode-0
    /// column is anchored to its decoded total height: MS-19 (b), a well-formed column, is
    /// drawn as with the switch on)
    #[test]
    fn mode0_top_down_fixed_origin() {
        let (x, yv, z) = (90, 9, 7);
        let data = [40, 2, 40, z, x, yv, 2];
        let fixed = VoxelQuirks {
            voxel_topdown_column_origin: false,
            ..q()
        };
        let on = mode0_cells(&data, 42, true, false, &q());
        assert_eq!(on, [(41, yv), (40, x)]);
        assert_eq!(mode0_cells(&data, 42, true, false, &fixed), on);
        // Bottom-up both states give 40 and Z.
        assert_eq!(
            mode0_cells(&data, 42, false, false, &fixed),
            [(40, 40), (41, z)]
        );
    }

    /// spec: S-0105 claim 36 (MS-20: a column ends at the first pair
    /// boundary at Nz; quirks.voxel_topdown_column_origin)
    #[test]
    fn column_ends_at_pair_boundary() {
        let (x, yv) = (21, 22);
        let data = vec![2, 2, x, 0, x, 0, 2, 0, 2, yv, 0, yv, 0, 2];
        let s = section([1, 1, 4], [0.0; 3], [1.0, 1.0, 4.0], 1, vec![Some(data)]);
        let got = column_pixels(s, false, &q());
        assert_eq!(got.len(), 2);
        assert!(got.iter().all(|p| p.1 == 100 + shade_of_mode1_normal0()));
        // Cells 2 and 3: two cells above the bottom of the 4-cell grid.
        let bottom = column_pixels(
            section(
                [1, 1, 4],
                [0.0; 3],
                [1.0, 1.0, 4.0],
                1,
                vec![Some(encode_column(&[(0, x, 0)], 4))],
            ),
            false,
            &q(),
        );
        assert_eq!(bottom[0].0 - got[0].0, 2);
    }

    fn shade_of_mode1_normal0() -> u8 {
        let up = mat([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
        ]);
        shade_table(&up, 1, &q())[0]
    }

    /// spec: S-0105 claim 36 (a short column drawn top-down sits γ cells
    /// too high; bottom-up it is placed correctly; switch off: both correct)
    #[test]
    fn short_column_top_down_shift() {
        // One voxel at cell 0 in a 4-cell column; runs cover 1 cell (γ = 3).
        let data = vec![0, 1, 33, 0, 1];
        let s = section([1, 1, 4], [0.0; 3], [1.0, 1.0, 4.0], 1, vec![Some(data)]);
        let up = column_pixels(s.clone(), false, &q());
        let down = column_pixels(s.clone(), true, &q());
        assert_eq!((up.len(), down.len()), (1, 1));
        // Top-down the voxel is drawn at the top cell's corner (screen y
        // 3 higher: cell 3 upper corner at 4 vs cell 0 lower corner at 0;
        // the copy rectangle is the same in both poses).
        let fixed = VoxelQuirks {
            voxel_topdown_column_origin: false,
            ..q()
        };
        let down_fixed = column_pixels(s.clone(), true, &fixed);
        let up_fixed = column_pixels(s, false, &fixed);
        assert_eq!(up_fixed, up);
        assert_eq!(down_fixed[0].0 - down[0].0, 3);
    }

    /// spec: S-0105 claims 21, 35, 36 (column data ending early: our engine
    /// stops at the end of the column's own data)
    #[test]
    fn truncated_column_data_stops() {
        let s = section(
            [1, 1, 8],
            [0.0; 3],
            [1.0, 1.0, 8.0],
            1,
            vec![Some(vec![0, 3, 5, 0, 6])],
        );
        assert_eq!(column_pixels(s.clone(), false, &q()).len(), 1);
        assert_eq!(column_pixels(s, true, &q()).len(), 1);
    }

    // --- Shadows ---

    /// spec: S-0105 claim 39 (each shadow pixel is excluded on its own where any completed part
    /// image is non-transparent at the same screen position)
    #[test]
    fn shadow_excludes_own_image() {
        let part = |w: u32, offset: [i64; 2], px: &[u8]| PartImage {
            width: w,
            height: 1,
            pixels: px.to_vec(),
            offset,
            copy_size: [i64::from(w), 1],
        };
        let m = SHADOW_MARK;
        // Shadow at screen x = 10 … 15, row 4.
        let mut shadow = part(6, [10, 4], &[m, m, m, m, m, 0]);
        // A body covering x = 11 (its x = 12 is transparent), a turret covering x = 14 … 15.
        let body = part(2, [11, 4], &[7, 0]);
        let turret = part(2, [14, 4], &[3, 3]);
        // A barrel on another row excludes nothing.
        let barrel = part(6, [10, 5], &[5; 6]);
        exclude_own_image(&mut shadow, &[&body, &turret, &barrel]);
        assert_eq!(shadow.pixels, [m, 0, m, m, 0, 0]);
    }

    /// spec: S-0105 claim 31, Formulas F10 (shift −6 × L_x stored truncated)
    #[test]
    fn shadow_shift_constant() {
        assert_eq!(
            to_f32_rz(mul_rz(-6.0, f64::from(LIGHT_VIEW[0]))),
            SHADOW_SHIFT
        );
    }

    /// spec: S-0105 claim 30, Formulas F10 (present columns, with or
    /// without voxels, mark two pixels; marked-empty columns nothing)
    #[test]
    fn shadow_footprint() {
        let with = Some(encode_column(&[(0, 5, 0)], 1));
        let without = Some(vec![1, 0, 0]);
        let s = section(
            [2, 2, 1],
            [0.0; 3],
            [2.0, 2.0, 1.0],
            1,
            vec![with, without, None, Some(vec![])],
        );
        let m = model(vec![s]);
        let img = draw_shadow(&m, 0, &Mat34::IDENTITY, 0, false, &q()).unwrap();
        // By F10 with truncating stores: Q₀₀ = (2.9999964, −0), Q₁₀ =
        // (4.99999619, −0) (2 + 2.9999964 truncated), Q₀₁ = (2.9999964, −2).
        // w₀ = trunc(1.99999976) = 1, h₀ = 2: copy rectangle 9 × 10, corner
        // (124, 123), c ≈ (3.9999963, −1), so screen = image + (−125, −129).
        assert_eq!(img.copy_size, [9, 10]);
        // o = (32512, 33024) → pixel (127, 129); Δ_i = (trunc(255.99997),
        // 0) = (255, 0), so column (1, 0) lands on the same pixel as (0, 0);
        // Δ_j = (0, −256): column (1, 1) one row up.
        let mut got: Vec<[i64; 2]> = img.opaque().into_iter().map(|p| p.0).collect();
        got.sort();
        assert_eq!(got, vec![[2, -1], [2, 0], [3, -1], [3, 0]]);
        assert!(img.opaque().iter().all(|p| p.1 == SHADOW_MARK));
        let single = VoxelQuirks {
            shadow_double_width: false,
            ..q()
        };
        let one = draw_shadow(&m, 0, &Mat34::IDENTITY, 0, false, &single).unwrap();
        assert_eq!(one.opaque().len(), 2);
        // A column marked empty does not mark: without (1, 1) row −1 is gone.
        let mut no_11 = m.clone();
        no_11.sections[0].columns[3] = None;
        let img = draw_shadow(&no_11, 0, &Mat34::IDENTITY, 0, false, &q()).unwrap();
        assert!(img.opaque().iter().all(|p| p.0[1] == 0));
        // The present column without voxels marks: without (0, 0) and
        // (1, 1), column (1, 0) alone still marks two pixels.
        let mut only_10 = no_11;
        only_10.sections[0].columns[0] = None;
        let img = draw_shadow(&only_10, 0, &Mat34::IDENTITY, 0, false, &q()).unwrap();
        assert_eq!(img.opaque().len(), 2);
        assert!(draw_shadow(&m, 1, &Mat34::IDENTITY, 0, false, &q()).is_none());
    }

    /// spec: S-0105 claim 31, Formulas F10 (aircraft-like: half size about
    /// the draw point, the shift not halved)
    #[test]
    fn aircraft_shadow_half_size() {
        let cols = vec![Some(encode_column(&[(0, 5, 0)], 1)); 16];
        let s = section([4, 4, 1], [0.0; 3], [8.0, 8.0, 1.0], 1, cols);
        let m = model(vec![s]);
        let full = draw_shadow(&m, 0, &Mat34::IDENTITY, 0, false, &q()).unwrap();
        let half = draw_shadow(&m, 0, &Mat34::IDENTITY, 0, true, &q()).unwrap();
        let span = |img: &PartImage| {
            let xs: Vec<i64> = img.opaque().iter().map(|p| p.0[0]).collect();
            (*xs.iter().min().unwrap(), *xs.iter().max().unwrap())
        };
        // Full: Q x = x + 2.9999964 truncated, Δ_i = trunc(511.99997) =
        // 511, columns at image x 124 + ⌊511 i / 256⌋ = 124, 125, 127, 129,
        // screen = image − 122: 2, 3, 5, 7 (+1 for the second pixel).
        assert_eq!(span(&full), (2, 8));
        // Half: Q x = x / 2 + 2.9999964, Δ_i = 255, image x 126, 126, 127,
        // 128, screen = image − 124: 2, 2, 3, 4 (+1). The shift is not
        // halved: both start at the same x.
        assert_eq!(span(&half), (2, 5));
        assert_eq!(full.copy_size, [15, 16]);
        assert_eq!(half.copy_size, [11, 12]);
    }

    /// spec: S-0105 claim 29 (shadow in HVA frame 0; quirks.shadow_frame_zero)
    #[test]
    fn shadow_frame() {
        let shift = |x: f32| {
            mat([
                [1.0, 0.0, 0.0, x],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
            ])
        };
        let s = filled([1, 1, 1], [0.0; 3], [1.0; 3], 1, 5, 0);
        let m = VoxelModel::from_sections(
            vec![s],
            Some(&hva_of(&[vec![shift(0.0)], vec![shift(7.0)]])),
            &q(),
        );
        let a = draw_shadow(&m, 0, &Mat34::IDENTITY, 0, false, &q()).unwrap();
        let b = draw_shadow(&m, 0, &Mat34::IDENTITY, 1, false, &q()).unwrap();
        assert_eq!(a, b);
        let cur = VoxelQuirks {
            shadow_frame_zero: false,
            ..q()
        };
        let c = draw_shadow(&m, 0, &Mat34::IDENTITY, 1, false, &cur).unwrap();
        assert_eq!(c.opaque()[0].0[0], a.opaque()[0].0[0] + 7);
    }

    // --- Recoil ---

    fn run(params: RecoilParams, shots: &[usize], ticks: usize) -> Vec<f32> {
        let mut c = RecoilCycle::new(params);
        (0..ticks)
            .map(|t| {
                if shots.contains(&t) {
                    c.fire();
                }
                c.tick()
            })
            .collect()
    }

    fn p(travel: i32, compress: i32, hold: i32, recover: i32) -> RecoilParams {
        RecoilParams {
            travel,
            compress,
            hold,
            recover,
        }
    }

    /// spec: S-0105 claim 27, Formulas F7, S1 (worked examples; every
    /// positive hold holds that many ticks; lead-confirmed hold departure)
    #[test]
    fn recoil_examples() {
        assert_eq!(
            run(p(3, 2, 3, 4), &[0], 10),
            [1.5, 3.0, 3.0, 3.0, 3.0, 2.25, 1.5, 0.75, 0.0, 0.0]
        );
        assert_eq!(
            run(p(3, 2, 1, 4), &[0], 7),
            [1.5, 3.0, 3.0, 2.25, 1.5, 0.75, 0.0]
        );
        assert_eq!(
            run(p(3, 2, 2, 4), &[0], 8),
            [1.5, 3.0, 3.0, 3.0, 2.25, 1.5, 0.75, 0.0]
        );
        // Absent-key defaults: 2, 2, 0.
        assert_eq!(run(RecoilParams::default(), &[0], 4), [2.0, 2.0, 0.0, 0.0]);
        // A hold of 0 or less holds no tick.
        assert_eq!(
            run(p(3, 2, 0, 4), &[0], 6),
            [1.5, 3.0, 2.25, 1.5, 0.75, 0.0]
        );
        assert_eq!(
            run(p(3, 2, -5, 4), &[0], 6),
            [1.5, 3.0, 2.25, 1.5, 0.75, 0.0]
        );
        // Negative travel slides forward, same magnitudes.
        assert_eq!(
            run(p(-3, 2, 3, 4), &[0], 9),
            [-1.5, -3.0, -3.0, -3.0, -3.0, -2.25, -1.5, -0.75, 0.0]
        );
        // Travel 0 starts no cycle.
        assert_eq!(run(p(0, 2, 3, 4), &[0], 3), [0.0; 3]);
    }

    /// spec: S-0105 claim 27, Formulas F7, S1 (a shot during a cycle
    /// restarts from the current slide toward the travel, never adding
    /// another travel; lead-confirmed refire departure)
    #[test]
    fn recoil_refire_is_bounded() {
        // Refire at travel 3 (during the hold): compress ticks stay at 3,
        // then the new hold and recovery follow.
        assert_eq!(
            run(p(3, 2, 3, 4), &[0, 3], 12),
            [1.5, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 2.25, 1.5, 0.75, 0.0]
        );
        // Travel 8, compress 8, hold 2, recover 2, refire at slide 3.
        let mut want: Vec<f32> = vec![1.0, 2.0, 3.0];
        want.extend([3.625, 4.25, 4.875, 5.5, 6.125, 6.75, 7.375, 8.0]);
        want.extend([8.0, 8.0, 4.0, 0.0]);
        assert_eq!(run(p(8, 8, 2, 2), &[0, 3], 15), want);
        // Repeated shots never exceed the travel, either sign.
        for travel in [5, -5] {
            let shots: Vec<usize> = (0..40).step_by(2).collect();
            let got = run(p(travel, 3, 1, 3), &shots, 50);
            assert!(got.iter().all(|v| v.abs() <= 5.0), "{got:?}");
            assert_eq!(got[49], 0.0);
        }
    }

    /// spec: S-0105 Formulas F7 (steps stored truncated; the last compress
    /// tick reaches exactly ℓ: ℓ = 1, t_c = 3)
    #[test]
    fn recoil_step_truncated() {
        let got = run(p(1, 3, 0, 3), &[0], 6);
        let step = to_f32_rz(1.0 / 3.0);
        assert_eq!(got[0], step);
        assert!(f64::from(got[0]) < 1.0 / 3.0);
        assert_ne!(got[0], (1.0f64 / 3.0) as f32);
        assert_eq!(got[1], to_f32_rz(f64::from(step) + f64::from(step)));
        assert!(got[1] < 2.0 / 3.0);
        // The third compress tick is exactly the travel.
        assert_eq!(got[2], 1.0);
        // Recovery: −1/3 truncated toward zero per tick, then exactly 0.
        assert_eq!(got[3], to_f32_rz(1.0 - f64::from(step)));
        assert_eq!(got[5], 0.0);
    }

    // --- Whole draws ---

    /// spec: S-0105 claims 7, 8, 10, 13, 25 (a synthetic model drawn as body,
    /// turret and barrel with its shadow)
    #[test]
    fn draws_a_synthetic_vehicle() {
        let vxl = Vxl::parse(
            &VxlWriter {
                model: &synthetic_model(),
            }
            .write(),
        )
        .unwrap();
        let hva = glasnost_fmt::hva::synthetic_animation();
        let vpl = glasnost_fmt::vpl::synthetic_table(32);
        let m = VoxelModel::new(&vxl, Some(&hva), &q());
        let pose = vehicle_parts(raw32(5), raw32(12), 40, barrel_elevation_after(20, 6), &q());
        for draw in [pose.body, pose.turret, pose.barrel] {
            for depth in [false, true] {
                let img = draw_model(&m, 3, &draw, Some(&vpl), depth, &q()).unwrap();
                assert_eq!(img.pixels.len(), (img.width * img.height) as usize);
                assert!(!img.opaque().is_empty());
            }
        }
        let shadow = draw_shadow(&m, 0, &pose.shadow, 0, false, &q()).unwrap();
        assert!(!shadow.opaque().is_empty());
    }

    /// Hostile sections and transforms never panic.
    #[test]
    fn hostile_sections_never_panic() {
        let mut rng = glasnost_fmt::synth::SynthRng::new(7);
        for round in 0..300 {
            let mut b = [0u8; 64];
            rng.fill(&mut b);
            let size = [b[0] % 5, b[1] % 5, b[2]];
            let n = usize::from(size[0]) * usize::from(size[1]);
            let columns = (0..n)
                .map(|k| {
                    (b[3 + k % 8] % 3 != 0)
                        .then(|| b[10 + k % 20..10 + k % 20 + usize::from(b[40] % 24)].to_vec())
                })
                .collect();
            let weird = [f32::NAN, f32::INFINITY, -1e30, 0.0, 3.5, -7.25][round % 6];
            let s = section(
                size,
                [weird, 0.0, -1.0],
                [b[50] as f32, weird, b[51] as f32],
                b[52] % 6,
                columns,
            );
            let draw = mat([
                [weird, 1.0, 0.0, 1e9],
                [0.0, weird, 1.0, 0.0],
                [1.0, 0.0, weird, -1e9],
            ]);
            let m = model(vec![s.clone(), s]);
            for depth in [false, true] {
                let _ = draw_model(&m, round as u32, &draw, None, depth, &q());
                let _ = draw_model(&m, 0, &view(&q()), None, depth, &q());
            }
            let _ = draw_shadow(&m, 1, &draw, 0, round % 2 == 0, &q());
        }
        assert!(draw_model(&model(vec![]), 0, &Mat34::IDENTITY, None, false, &q()).is_none());
    }

    // --- Image reuse (claims 26, 31, 37) and fresh draws (claims 22, 32, 34) ---

    /// A one-voxel model whose HVA frame *f* moves it *f* units along x, for
    /// `frames` frames.
    fn walking_marker(frames: usize) -> VoxelModel {
        let shift = |x: f32| {
            mat([
                [1.0, 0.0, 0.0, x],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
            ])
        };
        let all: Vec<Vec<Mat34>> = (0..frames).map(|f| vec![shift(f as f32)]).collect();
        VoxelModel::from_sections(
            vec![filled([1, 1, 1], [0.0; 3], [1.0; 3], 1, 40, 0)],
            Some(&hva_of(&all)),
            &q(),
        )
    }

    /// spec: S-0105 claim 26 (complete applicable frames in body, vehicle
    /// turret and building turret keys; adjacent frames never collide; the
    /// cache-disable option draws fresh; MS-3b's frame 33 shows frame 33;
    /// lead-confirmed frame-key departure)
    #[test]
    fn frame_keys_hold_complete_frames() {
        let body = |f| body_key(RAMP_FLAT, 0, false, f);
        assert_ne!(body(1), body(33));
        assert_ne!(body(1), body(2));
        assert_eq!(body(33), body(33));
        let turret = |f| vehicle_turret_key(RAMP_FLAT, 0, 0, 0, None, false, f);
        assert_ne!(turret(5), turret(261));
        assert_ne!(turret(5), turret(6));
        assert_ne!(building_turret_key(0, 5), building_turret_key(0, 261));
        assert_ne!(building_turret_key(0, 255), building_turret_key(0, 256));

        let m = walking_marker(40);
        let draw = |f: u32| draw_model(&m, f, &Mat34::IDENTITY, None, false, &q()).unwrap();
        assert_ne!(draw(1), draw(33), "the frames look different");
        for disabled in [false, true] {
            let mut cache = ImageCache::default();
            cache.reuse_or(body(1), disabled, || draw(1));
            let got = cache.reuse_or(body(33), disabled, || draw(33));
            assert_eq!(got, draw(33));
        }
    }

    /// spec: S-0105 claims 12, 26 (the alternative-model state and the
    /// complete turret frame are separate key components: distinct states
    /// and adjacent frames never collide; lead-confirmed departure)
    #[test]
    fn alternative_state_is_its_own_key_component() {
        let key = |alt, f| vehicle_turret_key(RAMP_FLAT, 0, 0, 0, None, alt, f);
        let keys = [
            key(true, 2),
            key(true, 3),
            key(false, 2),
            key(false, 3),
            key(true, 258),
            key(false, 258),
        ];
        for (i, a) in keys.iter().enumerate() {
            for b in &keys[i + 1..] {
                assert_ne!(a, b);
            }
        }
        assert_ne!(
            body_key(RAMP_FLAT, 0, true, 3),
            body_key(RAMP_FLAT, 0, false, 3)
        );
    }

    /// spec: S-0105 claim 26 (vehicle turret key: the body facing is left
    /// out when the turret displacement is 0; the selected turret index is
    /// part of it), claim 31 (shadow key: ramp and 32-step body facing)
    #[test]
    fn turret_and_shadow_keys() {
        let key = |body: i32, turret: i32, offset| {
            vehicle_turret_key(
                RAMP_FLAT,
                raw32(body),
                raw32(turret),
                offset,
                None,
                false,
                0,
            )
        };
        assert_eq!(key(0, 4, 0), key(9, 4, 0));
        assert_ne!(key(0, 4, 20), key(9, 4, 20));
        assert_ne!(key(0, 4, 0), key(0, 5, 0));
        // Raw facings within one 32-step facing share a key.
        assert_eq!(key(3, 4, 20), {
            vehicle_turret_key(
                RAMP_FLAT,
                raw32(3) + 500,
                raw32(4) - 500,
                20,
                None,
                false,
                0,
            )
        });
        let indexed = |i| vehicle_turret_key(RAMP_FLAT, 0, 0, 0, i, false, 0);
        assert_ne!(indexed(Some(0)), indexed(Some(1)));
        assert_eq!(
            shadow_key(RAMP_FLAT, raw32(7)),
            shadow_key(RAMP_FLAT, raw32(7) + 900)
        );
        assert_ne!(
            shadow_key(RAMP_FLAT, raw32(7)),
            shadow_key(RAMP_FLAT, raw32(8))
        );
        // Barrel images are kept apart from turret images.
        assert_ne!(
            ImageKey::Turret(indexed(None)),
            barrel_key(indexed(None), ELEVATION_LEVEL)
        );
    }

    /// spec: S-0105 claims 6, 26 (the barrel key holds the current 32-step
    /// elevation: a barrel cached level shows the raised pitch once the
    /// elevation rises; the turret key has no elevation; lead-confirmed
    /// departure)
    #[test]
    fn barrel_key_holds_the_elevation() {
        let barrel = model(vec![filled([8, 1, 1], [0.0; 3], [8.0, 1.0, 1.0], 1, 30, 0)]);
        let level = ELEVATION_LEVEL;
        let raised = barrel_elevation_after(DEFAULT_FIRE_ANGLE, 2);
        assert_ne!(facing32(level), facing32(raised));
        let draw = |e: u16| {
            let pose = vehicle_parts(0, 0, 20, e, &q());
            draw_model(&barrel, 0, &pose.barrel, None, false, &q()).unwrap()
        };
        assert_ne!(draw(level), draw(raised), "the pitch shows");
        let turret = vehicle_turret_key(RAMP_FLAT, 0, 0, 20, None, false, 0);
        assert_ne!(barrel_key(turret, level), barrel_key(turret, raised));
        // Raw elevations within one 32-step value share a key.
        assert_eq!(barrel_key(turret, level), barrel_key(turret, level - 500));
        let mut cache = ImageCache::default();
        cache.reuse_or(barrel_key(turret, level), false, || draw(level));
        let got = cache.reuse_or(barrel_key(turret, raised), false, || draw(raised));
        assert_eq!(got, draw(raised));
        // Back at level, the level image is reused.
        let back = cache.reuse_or(barrel_key(turret, level), false, || draw(raised));
        assert_eq!(back, draw(level));
    }

    /// spec: S-0105 claim 37 (a barrel without a turret: kept with the barrel
    /// images under its 32-step facing, complete frame and current 32-step
    /// elevation; lead-confirmed departure)
    #[test]
    fn barrel_only_key() {
        let raised = barrel_elevation_after(DEFAULT_FIRE_ANGLE, 2);
        let key = |facing: i32, frame, e| super::barrel_only_key(raw32(facing), frame, e);
        assert_ne!(key(4, 3, ELEVATION_LEVEL), key(4, 259, ELEVATION_LEVEL));
        assert_ne!(key(4, 3, ELEVATION_LEVEL), key(4, 3, raised));
        assert_ne!(key(4, 3, ELEVATION_LEVEL), key(5, 3, ELEVATION_LEVEL));
        assert_ne!(key(4, 3, ELEVATION_LEVEL), key(4, 4, ELEVATION_LEVEL));
        assert_eq!(key(4, 3, ELEVATION_LEVEL), key(4, 3, ELEVATION_LEVEL - 500));
        assert!(matches!(
            key(4, 3, ELEVATION_LEVEL),
            ImageKey::Barrel { .. }
        ));
    }

    /// The probe of the history tests: one flat layer of isolated voxels,
    /// 3 pixels apart, over 60 × 60 pixels.
    fn probe() -> VoxelModel {
        model(vec![filled(
            [20, 20, 1],
            [0.0; 3],
            [60.0, 60.0, 1.0],
            1,
            90,
            0,
        )])
    }

    /// spec: S-0105 claims 24, 34, S2 (every fresh depth-mode draw starts
    /// with empty depth everywhere: no draw in between, with or without
    /// the mode, suppresses a fresh depth-option part, MS-18;
    /// lead-confirmed clearing departure)
    #[test]
    fn depth_cleared_for_every_depth_draw() {
        let probe = probe();
        let spoiler = model(vec![filled([1, 1, 1], [0.0; 3], [1.0; 3], 1, 91, 0)]);
        let id = Mat34::IDENTITY;
        for spoiler_depth in [false, true] {
            let c = VoxelCanvas::new();
            let first = c.draw_part(&probe, 0, &id, None, true, &q()).unwrap();
            assert_eq!(
                first.opaque().len(),
                2 * 20 * 20,
                "every voxel, two pixels each"
            );
            c.draw_part(&spoiler, 0, &id, None, spoiler_depth, &q())
                .unwrap();
            c.draw_shadow(&probe, 0, &id, 0, false, &q()).unwrap();
            let second = c.draw_part(&probe, 0, &id, None, true, &q()).unwrap();
            assert_eq!(second, first);
        }
    }

    /// spec: S-0105 claims 22, 34 (every fresh draw starts completely
    /// transparent: an earlier, larger image never contaminates a later
    /// one; lead-confirmed clearing departure)
    #[test]
    fn fresh_draws_start_transparent() {
        let single = model(vec![filled([1, 1, 1], [0.0; 3], [1.0; 3], 1, 50, 0)]);
        // A wide model spanning more than 256 pixels, then a small one.
        let wide = model(vec![filled(
            [150, 1, 1],
            [0.0; 3],
            [300.0, 1.0, 1.0],
            1,
            60,
            0,
        )]);
        let id = Mat34::IDENTITY;
        let v = view(&q());
        let alone = VoxelCanvas::new()
            .draw_part(&single, 0, &v, None, false, &q())
            .unwrap();
        let c = VoxelCanvas::new();
        for depth in [false, true] {
            c.draw_part(&wide, 0, &id, None, depth, &q()).unwrap();
            c.draw_shadow(&wide, 0, &id, 0, false, &q()).unwrap();
            let after = c.draw_part(&single, 0, &v, None, false, &q()).unwrap();
            assert_eq!(after, alone);
            assert_eq!(after.opaque().len(), 1);
            assert!(after.opaque().iter().all(|p| p.1 == 50));
        }
    }

    /// spec: S-0105 claims 22, 25, Formulas F6 (no wrap: a part wider than
    /// 256 pixels shows every voxel at its own place, and a voxel beyond
    /// the copy rectangle never reappears inside it; lead-confirmed wrap
    /// departure)
    #[test]
    fn wide_part_never_wraps() {
        let id = Mat34::IDENTITY;
        let wide = model(vec![filled(
            [150, 1, 1],
            [0.0; 3],
            [300.0, 1.0, 1.0],
            1,
            60,
            0,
        )]);
        let img = draw_model(&wide, 0, &id, None, false, &q()).unwrap();
        assert_eq!(img.copy_size, [308, 9]);
        assert_eq!([img.width, img.height], [308, 9]);
        let mut xs: Vec<i64> = img.opaque().iter().map(|p| p.0[0]).collect();
        xs.sort();
        // Counted from the farthest corner (max x, max y; claim 21), the
        // voxel n_x steps away lands at screen x = 300 − 2 n_x: 150 distinct
        // pixels in one row, 2 … 300.
        assert_eq!(xs, (1..=150).map(|i| 2 * i).collect::<Vec<_>>());
        // A malformed column (claim 36) whose second voxel lies 256 cells,
        // 256 pixels, beyond its first: it falls outside the copy rectangle
        // and is not drawn, rather than wrapping onto the first one's pixel.
        let data = vec![0, 1, 62, 0, 1, 255, 0, 0, 0, 1, 63, 0, 1, 1, 0, 0];
        let s = section([1, 1, 2], [0.0; 3], [1.0, 1.0, 2.0], 1, vec![Some(data)]);
        let along_x = mat([
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
        ]);
        let img = draw_model(&model(vec![s]), 0, &along_x, None, false, &q()).unwrap();
        assert_eq!(img.copy_size, [10, 9]);
        let px = img.opaque();
        assert_eq!(px.len(), 1, "{px:?}");
        assert_eq!(px[0].1, 62);
    }

    /// spec: S-0105 claim 22, Formulas F6 (Q-0470: a copy rectangle larger than 4096 pixels on a
    /// side keeps the centred 4096-pixel window; the copy size stays unclipped and the retained
    /// pixels keep their screen positions)
    #[test]
    fn oversized_copy_rectangle_is_cut_to_the_window() {
        let huge = model(vec![filled(
            [3, 1, 1],
            [0.0; 3],
            [12000.0, 1.0, 1.0],
            1,
            64,
            0,
        )]);
        for depth in [false, true] {
            let img = draw_model(&huge, 0, &Mat34::IDENTITY, None, depth, &q()).unwrap();
            assert_eq!(img.copy_size, [12008, 9]);
            assert!(img.exceeds_limit());
            assert_eq!([img.width, img.height], [IMAGE_LIMIT as u32, 9]);
            // The copy rectangle lands at screen x 6000 − 6004 = −4; the window keeps
            // ⌊(12008 − 4096)/2⌋ = 3956 columns less on the left: screen x 3952 … 8047. The
            // voxels lie at screen x 4000, 8000 and 12000; the last is cut.
            assert_eq!(img.offset[0], 3952);
            let xs: Vec<i64> = img.opaque().iter().map(|p| p.0[0]).collect();
            assert!(xs.contains(&4000) && xs.contains(&8000), "{xs:?}");
            assert!(xs.iter().all(|&x| x < 8048), "{xs:?}");
        }
    }

    /// spec: S-0105 claim 22, Formulas F6 (retained window u = min(w, 4096), corner shifted by
    /// ⌊(w − u)/2⌋: sides of 4096 or less unchanged, an odd extra pixel discarded on the right or
    /// bottom, a million-pixel side kept to 4096 without moving the retained pixels), F10 (the
    /// same for a shadow)
    #[test]
    fn claim_22_retained_window() {
        // Model x from 0 to X: c = X/2, w = X + 8, the copy rectangle lands at
        // trunc(X/2) − ⌊w/2⌋ = −4 for these X.
        for (x, copy, offset) in [
            (4088.0, 4096, -4),
            (4089.0, 4097, -4),
            (4090.0, 4098, -3),
            (4091.0, 4099, -3),
            (1_000_000.0, 1_000_008, -4 + (1_000_008 - 4096) / 2),
        ] {
            let m = model(vec![filled([1, 1, 1], [0.0; 3], [x, 1.0, 1.0], 1, 64, 0)]);
            for depth in [false, true] {
                let img = draw_model(&m, 0, &Mat34::IDENTITY, None, depth, &q()).unwrap();
                assert_eq!(img.copy_size, [copy, 9], "X = {x}");
                assert_eq!(img.exceeds_limit(), copy > IMAGE_LIMIT);
                assert_eq!([img.width, img.height], [copy.min(IMAGE_LIMIT) as u32, 9]);
                assert_eq!(img.offset[0], offset, "X = {x}");
            }
        }
        // Vertically too (rows are −y).
        let tall = model(vec![filled(
            [1, 1, 1],
            [0.0; 3],
            [1.0, 9000.0, 1.0],
            1,
            64,
            0,
        )]);
        let img = draw_model(&tall, 0, &Mat34::IDENTITY, None, false, &q()).unwrap();
        assert_eq!(img.copy_size, [9, 9008]);
        assert_eq!([img.width, img.height], [9, IMAGE_LIMIT as u32]);
        // Lands at trunc(−4500) − 4504 = −9004, shifted by ⌊(9008 − 4096)/2⌋ = 2456.
        assert_eq!(img.offset[1], -9004 + 2456);
        // A shadow footprint follows the same window.
        let plate = model(vec![filled(
            [2, 2, 1],
            [0.0; 3],
            [9000.0, 2.0, 1.0],
            1,
            51,
            0,
        )]);
        let s = draw_shadow(&plate, 0, &Mat34::IDENTITY, 0, false, &q()).unwrap();
        assert!(s.exceeds_limit());
        assert_eq!(s.width, IMAGE_LIMIT as u32);
        let [w, _] = s.copy_size;
        let landed = (9000 + 2 * 3) / 2 - w / 2;
        assert!(
            (s.offset[0] - (landed + (w - IMAGE_LIMIT) / 2)).abs() <= 1,
            "{s:?}"
        );
    }

    /// spec: S-0105 claim 22 (non-finite placement: a part or shadow whose section bounds or
    /// transforms are not finite is not drawn; other finite parts are unaffected)
    #[test]
    fn claim_22_non_finite_placement_is_not_drawn() {
        let good = model(vec![filled([1, 1, 1], [0.0; 3], [1.0; 3], 1, 50, 0)]);
        let alone = draw_model(&good, 0, &Mat34::IDENTITY, None, false, &q()).unwrap();
        for bad in [f32::INFINITY, f32::NEG_INFINITY, f32::NAN] {
            let m = model(vec![filled([1, 1, 1], [0.0; 3], [bad, 1.0, 1.0], 1, 50, 0)]);
            for depth in [false, true] {
                assert!(draw_model(&m, 0, &Mat34::IDENTITY, None, depth, &q()).is_none());
            }
            assert!(draw_shadow(&m, 0, &Mat34::IDENTITY, 0, false, &q()).is_none());
            let t = mat([
                [1.0, 0.0, 0.0, bad],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
            ]);
            assert!(draw_model(&good, 0, &t, None, false, &q()).is_none());
            assert!(draw_shadow(&good, 0, &t, 0, false, &q()).is_none());
            // The finite part still draws, unchanged.
            assert_eq!(
                draw_model(&good, 0, &Mat34::IDENTITY, None, false, &q()).unwrap(),
                alone
            );
        }
    }

    /// spec: S-0105 claims 32, 33, S2 (the depth-buffered mode is on only
    /// during a fresh depth-option part draw: shadows never enable it, it
    /// is off between draws, and a part's depth look follows only its own
    /// type's option, fresh or reused; Q-0056, lead-confirmed departure)
    #[test]
    fn depth_mode_belongs_to_depth_option_parts() {
        let single = model(vec![filled([1, 1, 1], [0.0; 3], [1.0; 3], 1, 50, 0)]);
        let plate = model(vec![filled([2, 2, 1], [0.0; 3], [2.0, 2.0, 1.0], 1, 51, 0)]);
        let v = view(&q());
        let c = VoxelCanvas::new();
        let width = |img: PartImage| img.opaque().len();
        assert!(!c.depth_mode());
        assert_eq!(
            width(c.draw_part(&single, 0, &v, None, false, &q()).unwrap()),
            1
        );
        // A shadow (of any type) leaves the mode off and is drawn without
        // it: one mark pair per present column, as without the mode.
        let shadow = c
            .draw_shadow(&plate, 0, &Mat34::IDENTITY, 0, false, &q())
            .unwrap();
        assert_eq!(
            shadow,
            draw_shadow(&plate, 0, &Mat34::IDENTITY, 0, false, &q()).unwrap()
        );
        assert!(!c.depth_mode());
        assert_eq!(
            width(c.draw_part(&single, 0, &v, None, false, &q()).unwrap()),
            1,
            "no depth look after a shadow"
        );
        // A depth-option part has the depth look, and the mode is off again
        // afterwards.
        assert_eq!(
            width(c.draw_part(&single, 0, &v, None, true, &q()).unwrap()),
            2
        );
        assert!(!c.depth_mode());
        assert_eq!(
            width(c.draw_part(&single, 0, &v, None, false, &q()).unwrap()),
            1
        );
        // Reused images keep their own type's look.
        let mut cache = ImageCache::default();
        let key = body_key(RAMP_FLAT, 0, false, 0);
        let plain = cache.reuse_or(key, false, || {
            c.draw_part(&single, 0, &v, None, false, &q()).unwrap()
        });
        c.draw_part(&single, 0, &v, None, true, &q()).unwrap();
        let reused = cache.reuse_or(key, false, || {
            c.draw_part(&single, 0, &v, None, true, &q()).unwrap()
        });
        assert_eq!(reused, plain);
        assert!(!c.depth_mode());
    }

    /// spec: S-0105 claims 19, 21 (a later voxel whose lighting-table entry is
    /// 0 is still written: it erases the earlier voxel's pixel), claim 24
    /// (in the depth-buffered mode it is tested and stores its depth)
    #[test]
    fn zero_entry_voxel_erases() {
        let id = Mat34::IDENTITY;
        // Two sections on the same pixel; the later one (drawn last by claim 23, being nearer)
        // has colour 0, whose entry is 0 in every row of the table.
        let vpl = marker_vpl();
        let far = filled([1, 1, 1], [0.0, 0.0, 0.0], [1.0; 3], 1, 9, 0);
        let near = filled([1, 1, 1], [0.0, 0.0, 5.0], [1.0, 1.0, 6.0], 1, 0, 0);
        let m = model(vec![far.clone(), near]);
        let img = draw_model(&m, 0, &id, Some(&vpl), false, &q()).unwrap();
        assert!(
            img.opaque().is_empty(),
            "the 0 voxel erased the earlier one"
        );
        let alone = draw_model(&model(vec![far]), 0, &id, Some(&vpl), false, &q()).unwrap();
        assert_eq!(alone.opaque().len(), 1);
        // Depth-buffered: nearest first, so the 0 voxel comes first, passes, stores its depth,
        // and the farther voxel then fails the test.
        let img = draw_model(&m, 0, &id, Some(&vpl), true, &q()).unwrap();
        assert!(img.opaque().is_empty());
    }
}
