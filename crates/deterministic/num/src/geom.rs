//! Distances, directions and facings (S-0204).

use num_bigint::BigUint;

use crate::bigmath;
use crate::soft::{F32, F64, Rounding};
use crate::tables::ATAN;

/// A facing: 0 … 65535 facing units, 0 = map north, clockwise (glossary G1).
pub type Facing = u16;

/// Integer square root (floor).
// nocite: integer square root (Newton iteration), not game behaviour
pub fn isqrt(n: u128) -> u128 {
    if n < 2 {
        return n;
    }
    let mut x = 1u128 << (128 - n.leading_zeros()).div_ceil(2);
    loop {
        let y = (x + n / x) / 2;
        if y >= x {
            return x;
        }
        x = y;
    }
}

/// The table square root r(n) of S-0204 F1 as `(R, z)` with
/// r(n) = R × 2^(z − 23), 2^23 ≤ R < 2^24; `None` for n = 0.
///
/// `mode` is the rounding of the conversion of `n` to single precision
/// (S-0204 F1, `quirks.chop_rounding` of S-0205): toward zero it changes
/// nothing; to nearest it rounds `n` to 24 significant bits first.
// spec: S-0204 F1 (table square root: 14 significant digits of n / 4^z, root to 24 digits)
pub fn table_root_parts(n: u64, mode: Rounding) -> Option<(u32, u32)> {
    if n == 0 {
        return None;
    }
    // spec: S-0204 F1 (n is converted to single precision: 24 significant bits; to nearest, n may round up to 2^64)
    let n = single_of_integer(n, mode);
    let len = 128 - n.leading_zeros();
    let z = (len - 1) / 2;
    // spec: S-0204 F1 (n / 4^z truncated toward zero to 14 significant binary digits)
    let r = if len > 14 {
        let top = n >> (len - 14);
        isqrt(top << (len - 14 + 46 - 2 * z))
    } else {
        // spec: S-0204 F1 (root truncated to 24 significant digits: 2^46 scale for 2^23 ≤ R < 2^24)
        isqrt(n << (46 - 2 * z))
    };
    Some((r as u32, z))
}

/// `n` (non-negative integer) as the integer value of the single nearest to
/// it by `mode`; 128 bits wide, since rounding to nearest carries the top of
/// the 64-bit range to 2^64 (#295).
// spec: S-0205 claim 4 (single precision keeps 24 significant bits)
fn single_of_integer(n: u64, mode: Rounding) -> u128 {
    let n = u128::from(n);
    let len = 128 - n.leading_zeros();
    // spec: S-0205 claim 4 (single precision has a 24-bit significand)
    if len <= 24 {
        return n;
    }
    let sh = len - 24;
    let t = n >> sh;
    let rem = n & ((1u128 << sh) - 1);
    let t = match mode {
        Rounding::TowardZero => t,
        Rounding::Nearest => {
            let half = 1u128 << (sh - 1);
            if rem > half || (rem == half && t & 1 == 1) {
                t + 1
            } else {
                t
            }
        }
    };
    t << sh
}

/// r(n) as a single-precision value (S-0204 F1).
pub fn table_root(n: u64, mode: Rounding) -> F32 {
    match table_root_parts(n, mode) {
        None => F32::ZERO,
        Some((r, z)) => {
            // spec: S-0204 F1 (r = R × 2^(z − 23))
            F64::from_parts(false, u128::from(r), z as i32 - 23, Rounding::TowardZero)
                .to_f32(Rounding::TowardZero)
        }
    }
}

/// The table square root of a real argument (S-0105 F1, S-0306 Formulas):
/// |x| narrowed to single precision by `mode`, then the root of its
/// significand truncated to 14 significant binary digits, truncated to 24.
/// The result is a single, returned widened to a double. With `table` false
/// (`quirks.table_root_distance` off) the exact root truncated to single
/// precision.
// spec: S-0105 F1 (√ₜ(x) = trunc₂₄(2^ζ × √m′), m′ the 14-digit truncation of m)
pub fn table_root_real(x: F64, table: bool, mode: Rounding) -> F64 {
    let s = x.abs().to_f32(mode);
    let bits = s.0;
    // nocite: binary32 field layout (exponent and fraction masks)
    let exp = (bits >> 23) & 0xff;
    // nocite: binary32 field layout
    let frac = bits & 0x7f_ffff;
    // nocite: binary32 field layout (all-ones exponent: infinity or NaN)
    if exp == 0xff {
        return s.to_f64();
    }
    if exp == 0 && frac == 0 {
        return F64::ZERO;
    }
    // The single as M × 2^k with M normalised to 24 significant bits.
    // nocite: binary32 field layout (hidden bit and exponent bias)
    let (mut m, mut k): (u64, i32) = if exp == 0 {
        (u64::from(frac), -149)
    } else {
        (u64::from(frac | 0x80_0000), exp as i32 - 150)
    };
    while m < (1 << 23) {
        m <<= 1;
        k -= 1;
    }
    // ζ = ⌊log₄ x⌋: x lies in [2^(23 + k), 2^(24 + k)).
    let zeta = (23 + k).div_euclid(2);
    // m′ × 2^46 as an integer: m′ = (M >> 10) × 2^(k − 2ζ + 10), k − 2ζ is −23 or −22.
    let e = k - 2 * zeta + 10 + 46;
    let radicand = if table {
        // spec: S-0105 F1 (m truncated toward zero to 14 significant binary digits)
        u128::from(m >> 10) << e
    } else {
        u128::from(m) << (e - 10)
    };
    // spec: S-0105 F1 (the exact root, truncated to 24 significant digits)
    let r = isqrt(radicand);
    F64::from_parts(false, r, zeta - 23, Rounding::TowardZero)
        .to_f32(Rounding::TowardZero)
        .to_f64()
}

/// ⌊r(n)⌋, the distance term of S-0204 claim 5; with `table` false
/// (`quirks.table_root_distance` off) ⌊√n⌋.
pub fn root_floor(n: u64, table: bool, mode: Rounding) -> u64 {
    if !table {
        return isqrt(u128::from(n)) as u64;
    }
    match table_root_parts(n, mode) {
        None => 0,
        Some((r, z)) => {
            // spec: S-0204 F1 (⌊R × 2^(z − 23)⌋)
            if z <= 23 {
                u64::from(r >> (23 - z))
            } else {
                u64::from(r) << (z - 23)
            }
        }
    }
}

/// The signed facing difference δ(a, b) of S-0204 F5: −32768 … 32767.
pub fn delta(a: Facing, b: Facing) -> i32 {
    // spec: S-0204 F5 (δ = ((b − a + 32768) mod 65536) − 32768)
    i32::from(b.wrapping_sub(a).wrapping_add(32768)) - 32768
}

/// The n-step facing of S-0204 F4 for n = 2^b (1 ≤ b ≤ 8).
pub fn n_step(f: Facing, n: u32) -> u32 {
    // spec: S-0204 F4 (⌊f × n / 65536 + ½⌋ mod n)
    ((u32::from(f) * n + 32768) / 65536) % n
}

/// The 8-direction of a facing (S-0204 F4 with n = 8).
pub fn eight_direction(f: Facing) -> u8 {
    // spec: S-0204 claim 9 (8-directions; F4 with n = 8)
    n_step(f, 8) as u8
}

/// The 50 table indices at which the original's facing is one unit nearer
/// to map east or west than the surrogate gives (S-0204 F3).
// spec: S-0204 F3 (the 50 table exceptions)
pub const ATAN_EXCEPTIONS: [u16; 50] = [
    441, 486, 789, 810, 842, 993, 1024, 1115, 1148, 1183, 1203, 1405, 1419, 1443, 1504, 1542, 1582,
    1618, 1810, 1907, 2015, 2044, 2094, 2136, 2202, 2225, 2400, 2513, 2605, 2621, 2670, 2704, 2721,
    2848, 3030, 3096, 3312, 3364, 3418, 3502, 3531, 3683, 3715, 3748, 3781, 3849, 3884, 3956, 3993,
    4069,
];

/// The direction constants of S-0204 F3.
struct DirConst {
    h: F64,
    c1: F64,
    pi_d: F64,
    c2: F64,
}

fn dir_const() -> DirConst {
    let rz = Rounding::TowardZero;
    DirConst {
        // spec: S-0204 F3 (h = 13176795 / 2^23, single-precision π/2)
        h: F64::from_parts(false, 13176795, -23, rz),
        // spec: S-0204 F3 (c₁ = 884279719003555 / 2^49, double-precision π/2)
        c1: F64::from_parts(false, 884279719003555, -49, rz),
        // spec: S-0204 F3 (π_d = 884279719003555 / 2^48)
        pi_d: F64::from_parts(false, 884279719003555, -48, rz),
        // spec: S-0204 F3 (c₂ = −5733986146512111 / 2^39)
        c2: F64::from_parts(true, 5733986146512111, -39, rz),
    }
}

/// The direction of point Q from point P (S-0204 claim 8, F3), from the
/// map-axis differences dx = x_Q − x_P and dy = y_Q − y_P in leptons. With
/// `table` false (`quirks.table_direction` off) the exact angle, rounded to
/// the nearest facing with halves up.
// spec: S-0204 F3 (direction; facings modulo 65536; table of 4097 entries)
pub fn direction(dx: i64, dy: i64, table: bool) -> Facing {
    // spec: S-0204 F3 (Y = y_P − y_Q grows towards map north)
    let x = dx;
    let y = -dy;
    if !table {
        return exact_direction(x, y);
    }
    let rz = Rounding::TowardZero;
    let k = dir_const();
    let ax = u128::from(x.unsigned_abs());
    let ay = u128::from(y.unsigned_abs());
    let (theta, index) = if x == 0 {
        let t = if y > 0 {
            k.h
        } else if y < 0 {
            k.h.negated()
        } else {
            F64::ZERO
        };
        (t, None)
    } else {
        // spec: S-0204 F3 (i = ⌊2^27 |Y| / (3276705 |X|)⌋)
        let i = (ay << 27) / (3276705 * ax);
        // spec: S-0204 F3 (A = T(i) for i ≤ 4096, h beyond)
        let a = if i <= 4096 {
            F32(ATAN[i as usize]).to_f64()
        } else {
            k.h
        };
        let base = if x > 0 { a } else { k.pi_d.sub(a, rz) };
        let t = if y < 0 { base.negated() } else { base };
        (t, Some(i))
    };
    // spec: S-0204 F3 (φ = trunc(trunc53(trunc53(θ − c₁) × c₂)) mod 65536)
    let v = theta.sub(k.c1, rz).mul(k.c2, rz);
    let t = v.trunc_i128().unwrap_or(0);
    let mut phi = t.rem_euclid(65536) as i32;
    if let Some(i) = index
        && i <= 4096
        && ATAN_EXCEPTIONS.contains(&(i as u16))
    {
        // spec: S-0204 F3 (one facing unit nearer to map east, or map west when X < 0)
        let axis = if x > 0 { 16383 } else { 49153 };
        if phi < axis {
            phi += 1;
        } else if phi > axis {
            phi -= 1;
        }
    }
    phi as Facing
}

/// The table angle θ of S-0204 F3 for real arguments X and Y (Y growing
/// towards map north, or upwards for a pitch), both narrowed to single
/// precision by `mode` first (S-0306 Formulas), with the table index used
/// (`None` when X is 0).
// spec: S-0204 F3 (the table index, 4097 entries; an index beyond the table gives h)
pub fn table_angle_real(x: F64, y: F64, mode: Rounding) -> (F64, Option<u32>) {
    let k = dir_const();
    let x = x.to_f32(mode).to_f64();
    let y = y.to_f32(mode).to_f64();
    let zero = F64::ZERO;
    if x.eq_num(zero) {
        // spec: S-0204 F3 (X = 0: h, 0 or −h by the sign of Y)
        let t = if y.gt(zero) {
            k.h
        } else if y.lt(zero) {
            k.h.negated()
        } else {
            zero
        };
        return (t, None);
    }
    // spec: S-0204 F3 (σ = 3276705 / 2^27; the quotient Y / X and its quotient by σ are doubles, truncated to the index)
    let sigma = F64::from_parts(false, 3276705, -27, Rounding::TowardZero);
    let q = y.abs().div(x.abs(), mode).div(sigma, mode);
    let i = q.trunc_i128().unwrap_or(i128::MAX).clamp(0, 1 << 20) as u32;
    // spec: S-0204 F3 (A = T(i) for i ≤ 4096, h beyond)
    let a = if i <= 4096 {
        F32(ATAN[i as usize]).to_f64()
    } else {
        k.h
    };
    let base = if x.gt(zero) { a } else { k.pi_d.sub(a, mode) };
    let t = if y.lt(zero) { base.negated() } else { base };
    (t, Some(i))
}

/// The facing value of an angle by S-0204 F3, trunc(trunc₅₃(trunc₅₃(θ − c₁)
/// × c₂)), before the reduction modulo 65536, with the table exceptions
/// applied for the index (towards 16383 for X > 0, 49153 for X < 0).
// spec: S-0204 F3 (facings modulo 65536; the table exceptions up to index 4096)
pub fn facing_of_angle(theta: F64, index: Option<u32>, x_positive: bool, mode: Rounding) -> i64 {
    let k = dir_const();
    // spec: S-0204 F3 (φ = trunc(trunc₅₃(trunc₅₃(θ − c₁) × c₂)))
    let v = theta.sub(k.c1, mode).mul(k.c2, mode);
    let mut phi = v.trunc_i128().unwrap_or(0) as i64;
    if let Some(i) = index
        && i <= 4096
        && ATAN_EXCEPTIONS.contains(&(i as u16))
    {
        // spec: S-0204 F3 (one facing unit nearer to map east, or map west when X < 0)
        let axis = if x_positive { 16383 } else { 49153 };
        let p = phi.rem_euclid(65536);
        if p < axis {
            phi += 1;
        } else if p > axis {
            phi -= 1;
        }
    }
    phi
}

/// A pitch facing from a signed table angle (S-0204 claim 12, F6): the
/// surrogate conversion of F3, trunc(trunc₅₃(trunc₅₃(α − c₁) × c₂)), plus
/// the correction *e*: at one of F3's exception indices, +1 for a positive
/// angle and −1 for a negative one (both nearer to level, 16383), 0 for a
/// zero angle; 0 at every other index, with no index or one beyond 4096.
/// The result is not reduced modulo 65536 (the caller narrows it).
// spec: S-0204 F6 (the correction follows the sign of the angle supplied, not of the table argument)
pub fn pitch_of_angle(alpha: F64, index: Option<u32>, mode: Rounding) -> i64 {
    let k = dir_const();
    // spec: S-0204 F6 (p_s = trunc(trunc₅₃(trunc₅₃(α_s − c₁) × c₂)))
    let ps = alpha
        .sub(k.c1, mode)
        .mul(k.c2, mode)
        .trunc_i128()
        .unwrap_or(0) as i64;
    let exception = index.is_some_and(|i| i <= 4096 && ATAN_EXCEPTIONS.contains(&(i as u16)));
    // spec: S-0204 F6 (e = 1 for a positive angle, −1 for a negative one, 0 for zero, at an exception index)
    let e = if !exception {
        0
    } else if alpha.gt(F64::ZERO) {
        1
    } else if alpha.lt(F64::ZERO) {
        -1
    } else {
        0
    };
    ps + e
}

/// The direction facing of a real displacement (X, Y up), S-0204 F3.
// spec: S-0204 F3 (the facing modulo 65536)
pub fn direction_real(x: F64, y: F64, mode: Rounding) -> Facing {
    let (t, i) = table_angle_real(x, y, mode);
    facing_of_angle(t, i, x.gt(F64::ZERO), mode).rem_euclid(65536) as Facing
}

/// [`direction_real`] with `quirks.table_direction` at `table`: off, the
/// exact direction of S-0204 Edge cases for X and Y narrowed to single
/// precision by `mode` (S-0306 Formulas), with the Off form's rules for
/// arguments that are not finite or both zero (S-0204 Edge cases, revision
/// 14): not a number counts as zero; if an argument is then infinite, each
/// infinite one counts as ±1 by its sign and each finite one as 0; two
/// zero arguments, of either sign, give 16384.
// spec: S-0204 Edge cases (quirks.table_direction off: the exact angle), S-0306 Formulas (both arguments narrowed to single precision)
pub fn direction_real_b(x: F64, y: F64, table: bool, mode: Rounding) -> Facing {
    if table {
        return direction_real(x, y, mode);
    }
    let (px, py) = (off_part(x.to_f32(mode)), off_part(y.to_f32(mode)));
    // spec: S-0204 Edge cases (revision 14, Q-3103: an argument that is not a number counts as zero)
    let zero = (false, 0, 0);
    let finite = |p: OffPart| match p {
        OffPart::Finite(n, m, k) => (n, m, k),
        OffPart::Infinite(_) | OffPart::NotANumber => zero,
    };
    let any_infinite = matches!(px, OffPart::Infinite(_)) || matches!(py, OffPart::Infinite(_));
    let unit = |p: OffPart| match p {
        // spec: S-0204 Edge cases (revision 14: if an argument is then infinite, each infinite argument counts as +1 or −1 by its sign and each finite one as 0)
        OffPart::Infinite(n) => (n, 1, 0),
        OffPart::Finite(..) | OffPart::NotANumber => zero,
    };
    let ((nx, mx, kx), (ny, my, ky)) = if any_infinite {
        (unit(px), unit(py))
    } else {
        (finite(px), finite(py))
    };
    if mx == 0 && my == 0 {
        // spec: S-0204 Edge cases (revision 14: if both arguments are then zero, of either sign, the direction is 16384, map east on the exact scale) and S-0306 Formulas (revision 90)
        return 16384;
    }
    angle_of_parts(nx && mx != 0, (mx, kx), ny && my != 0, (my, ky))
}

/// A single-precision argument of the Off form of the direction (S-0204
/// Edge cases): finite as (negative, M, k) with value ±M × 2^k, infinite
/// with its sign, or not a number.
#[derive(Clone, Copy)]
enum OffPart {
    Finite(bool, u64, i32),
    Infinite(bool),
    NotANumber,
}

// nocite: binary32 field layout (an all-ones exponent field: an infinity with a zero fraction, not a number otherwise)
fn off_part(s: F32) -> OffPart {
    match single_parts(s) {
        Some((n, m, k)) => OffPart::Finite(n, m, k),
        None if s.0 & 0x7f_ffff == 0 => OffPart::Infinite(s.0 >> 31 == 1),
        None => OffPart::NotANumber,
    }
}

/// A finite single as (negative, M, k) with value ±M × 2^k; `None` for an
/// infinity or NaN.
// nocite: binary32 field layout (sign, exponent and fraction fields, bias and hidden bit)
fn single_parts(s: F32) -> Option<(bool, u64, i32)> {
    let bits = s.0;
    let neg = bits >> 31 == 1;
    let exp = (bits >> 23) & 0xff;
    let frac = bits & 0x7f_ffff;
    match exp {
        0xff => None,
        0 => Some((neg, u64::from(frac), -149)),
        _ => Some((neg, u64::from(frac | 0x80_0000), exp as i32 - 150)),
    }
}

/// The constants c₁ (π/2) and c₂ (−32767/π) of S-0204 F3, as doubles.
pub fn direction_constants() -> (F64, F64) {
    let k = dir_const();
    (k.c1, k.c2)
}

/// The exact direction (S-0204 Edge cases, `quirks.table_direction` off):
/// round(arctan2(X, Y) × 65536 / (2π)) mod 65536, halves up, with
/// arctan2(X, Y) the angle clockwise from map north.
// spec: S-0204 Edge cases (exact direction with `quirks.table_direction` off)
fn exact_direction(x: i64, y: i64) -> Facing {
    if x == 0 && y == 0 {
        // spec: S-0204 Edge cases (revision 14, Q-3103: two zero arguments give 16384, map east on the exact scale, wherever a rule takes a direction by F3 with quirks.table_direction off)
        return 16384;
    }
    angle_of_parts(x < 0, (x.unsigned_abs(), 0), y < 0, (y.unsigned_abs(), 0))
}

/// The exact direction for X = ±mx × 2^kx and Y = ±my × 2^ky (`ax` = (mx,
/// kx), `ay` = (my, ky), signs by `neg_x` and `neg_y`, false for a zero),
/// not both zero: [`fast_angle`], or within its margin of a half the
/// big-integer [`exact_angle`] (#393).
// spec: S-0204 Edge cases (exact direction with `quirks.table_direction` off)
fn angle_of_parts(neg_x: bool, ax: (u64, i32), neg_y: bool, ay: (u64, i32)) -> Facing {
    fast_angle(neg_x, ax, neg_y, ay).unwrap_or_else(|| {
        // Both magnitudes on the smaller of the two binary exponents.
        let k = ax.1.min(ay.1);
        let bx = BigUint::from(ax.0) << (ax.1 - k) as u32;
        let by = BigUint::from(ay.0) << (ay.1 - k) as u32;
        exact_angle(neg_x, &bx, neg_y, &by)
    })
}

/// The exact direction of [`exact_direction`] for X = ±`ax` and Y = ±`ay`
/// (signs by `neg_x` and `neg_y`, false for a zero), not both zero.
// spec: S-0204 Edge cases (exact direction with `quirks.table_direction` off)
fn exact_angle(neg_x: bool, ax: &BigUint, neg_y: bool, ay: &BigUint) -> Facing {
    let s = BigUint::from(1u32) << bigmath::FRAC;
    let pi = bigmath::pi();
    // β = arctan(|X| / |Y|), the angle from the Y axis.
    let beta = if *ay == BigUint::from(0u32) {
        &pi >> 1u32
    } else {
        bigmath::atan(&((ax * &s) / ay))
    };
    let alpha = match (!neg_x, !neg_y) {
        (true, true) => beta,
        (true, false) => &pi - beta,
        (false, false) => &pi + beta,
        (false, true) => (&pi << 1u32) - beta,
    };
    // spec: S-0204 Edge cases (× 65536 / (2π), halves up, mod 65536)
    let t = (alpha * 32768u32 * &s) / &pi;
    let rounded = (t + (&s >> 1u32)) >> bigmath::FRAC;
    let r = u64::try_from(rounded % 65536u32).unwrap_or(0);
    r as Facing
}

/// arctan(i/64) for i = 0 to 64, each the nearest multiple of 2^-64 (Q64),
/// for the fast path of the exact direction ([`fast_angle`], #393); the
/// test `fast_angle_constants_match_bigmath` checks every entry against
/// [`bigmath::atan`].
// nocite: engineering table of our own for the exact direction's fast path (#393), checked against bigmath in a test
#[rustfmt::skip]
const ATAN_64THS: [u64; 65] = [
    0x0000000000000000, 0x03ffeaab776e5357, 0x07ff556eea5d892a, 0x0bfdc0c2186d14fd,
    0x0ffaaddb967ef4e3, 0x13f59f0e7c559d6b, 0x17ee182602f10e8c, 0x1be39ebe6f07c37e,
    0x1fd5ba9aac2f6dc6, 0x23c3f5f6086e4dc9, 0x27adddd18cc4d8b1, 0x2b93023c7d84d3bf,
    0x2f72f6979cb6044d, 0x334d51d2d90c4c3a, 0x3721aea524c14409, 0x3aefabbe40ae6ce3,
    0x3eb6ebf25901bac5, 0x4277165f618d8963, 0x462fd68c2fc5e098, 0x49e0dc815fbd16f9,
    0x4d89dcdc1faf2f35, 0x512a90db0abc26a3, 0x54c2b6654735276d, 0x5852100c273f8659,
    0x5bd86507937bc23a, 0x5f55812d8ecfdd6a, 0x62c934e5286c95b7, 0x6633551535ac619e,
    0x6993bb0f308ff2db, 0x6cea44769971b1ae, 0x7036d3253b27be34, 0x73794d0cb04d425d,
    0x76b19c1586ed3da3, 0x79dfadfc5d68d10e, 0x7d03742d50505f2e, 0x801ce39e0d205c9a,
    0x832bf4a6d9867e2a, 0x8630a2dada1ed066, 0x892aecdfde9547b5, 0x8c1ad445f3e09b8c,
    0x8f005d5ef7f59f9b, 0x91db8f1664f350e2, 0x94ac72c9847186f6, 0x97731420365e538c,
    0x9a2f80e671bdda20, 0x9ce1c8e6a0b8cdba, 0x9f89fdc4f4b7a1ed, 0xa22832dbcadaae09,
    0xa4bc7d1934f70924, 0xa746f2ddb7602294, 0xa9c7abdc4830f5c9, 0xac3ec0fb997dd6a2,
    0xaeac4c38b4d8c080, 0xb110688aebdc6f6a, 0xb36b31c91f043691, 0xb5bcc49059ecc4b0,
    0xb8053e2bc2319e74, 0xba44bc7dd470782f, 0xbc7b5deae98af281, 0xbea94144fd049aac,
    0xc0ce85b8ac526641, 0xc2eb4abb661628b6, 0xc4ffaffabf8fbd55, 0xc70bd54ce602ee14,
    0xc90fdaa22168c235,
];

/// 32768/π, the facing units per radian, as the nearest multiple of 2^-50
/// (#393); the test `fast_angle_constants_match_bigmath` checks it against
/// [`bigmath::pi`].
// nocite: engineering constant of our own for the exact direction's fast path (#393), checked against bigmath in a test
const FACINGS_PER_RADIAN_Q50: u128 = 0xa2f9_836e_4e44_152a;

/// A non-zero magnitude m × 2^k with m shifted so that its top bit is bit 63.
// nocite: normalisation of our own fixed-point representation (#393)
fn normalised(m: u64, k: i32) -> (u64, i32) {
    let z = m.leading_zeros();
    (m << z, k - z as i32)
}

/// arctan(q) for 0 ≤ q ≤ 1 in Q64 (q ≤ 2^64), within a few units of 2^-64:
/// the table entry at the nearest 64th c, plus arctan(r) for r = (q − c) /
/// (1 + qc) by its series to the eleventh power (|r| ≤ 1/128, so the first
/// term left out is below 2^-91).
// nocite: Q64 fixed point, the 64ths table and the series length are our own engineering (#393)
fn atan_q64(q: u128) -> u128 {
    if q >= 1 << 64 {
        return u128::from(ATAN_64THS[64]);
    }
    let i = ((q + (1 << 57)) >> 58) as usize;
    let c = (i as u128) << 58;
    let (d, below) = if q >= c {
        (q - c, false)
    } else {
        (c - q, true)
    };
    let den = (1u128 << 64) + ((q * c) >> 64);
    let r = (d << 64) / den;
    let r2 = (r * r) >> 64;
    let mut t = r;
    let mut sum = r;
    // The odd powers 3 to 11: r^5 and r^9 add, the others subtract.
    for n in [3u128, 5, 7, 9, 11] {
        t = (t * r2) >> 64;
        if n % 4 == 1 {
            sum += t / n;
        } else {
            sum -= t / n;
        }
    }
    let base = u128::from(ATAN_64THS[i]);
    if below { base - sum } else { base + sum }
}

/// The exact direction of [`exact_angle`] in 128-bit fixed point for
/// magnitudes given as (m, k) for m × 2^k, not both zero; `None` when the
/// angle in facing units lies within 2^-40 of a half, where the caller falls
/// back to [`exact_angle`]. Elsewhere the result is the same: the error of
/// the facing value here is below 2^-44, so both round the same real number
/// on the same side of the half.
// spec: S-0204 Edge cases (exact direction with `quirks.table_direction` off: round(arctan2(X, Y) × 65536 / (2π)), halves up, mod 65536)
// nocite: Q64, Q50 and Q114 fixed point and the 2^-40 margin are our own engineering (#393)
fn fast_angle(neg_x: bool, ax: (u64, i32), neg_y: bool, ay: (u64, i32)) -> Option<Facing> {
    // b: the angle from the Y axis, arctan(|X| / |Y|), in facing units, rounded.
    let b = if ay.0 == 0 {
        16384
    } else if ax.0 == 0 {
        0
    } else {
        let (mx, ex) = normalised(ax.0, ax.1);
        let (my, ey) = normalised(ay.0, ay.1);
        let x_larger = (ex, mx) > (ey, my);
        let ((ms, es), (ml, el)) = if x_larger {
            ((my, ey), (mx, ex))
        } else {
            ((mx, ex), (my, ey))
        };
        // q = small / large in Q64, at most 1.
        let shift = i64::from(el) - i64::from(es);
        let q = if shift >= 128 {
            0
        } else {
            ((u128::from(ms) << 64) / u128::from(ml)) >> shift
        };
        // u = arctan(q) × 32768 / π in Q114.
        let u = atan_q64(q) * FACINGS_PER_RADIAN_Q50;
        let half = 1u128 << 113;
        let frac = u & ((1u128 << 114) - 1);
        if frac.abs_diff(half) < 1u128 << 74 {
            return None;
        }
        let r = ((u + half) >> 114) as u32;
        if x_larger { 16384 - r } else { r }
    };
    // spec: S-0204 Edge cases (the angle clockwise from map north, mod 65536)
    let f = match (neg_x, neg_y) {
        (false, false) => b,
        (false, true) => 32768 - b,
        (true, true) => 32768 + b,
        (true, false) => 65536 - b,
    };
    Some((f % 65536) as Facing)
}

#[cfg(test)]
mod tests {
    #[test]
    fn real_root_matches_integer_root_below_two_to_the_24() {
        for n in [1u64, 2, 3, 4, 7, 99, 255, 256, 65535, 100_000, 16_777_215] {
            let a = table_root_real(
                F64::from_i64(n as i64, Rounding::TowardZero),
                true,
                Rounding::TowardZero,
            );
            assert_eq!(a, table_root(n, Rounding::TowardZero).to_f64(), "{n}");
        }
        assert_eq!(
            table_root_real(F64::ZERO, true, Rounding::TowardZero),
            F64::ZERO
        );
    }

    use super::*;

    const RZ: Rounding = Rounding::TowardZero;

    // The fast path's table and constant are the nearest Q64 and Q50 values
    // of bigmath's arctan(i/64) and 32768/π (#393).
    #[test]
    fn fast_angle_constants_match_bigmath() {
        let f = bigmath::FRAC;
        for (i, &want) in ATAN_64THS.iter().enumerate() {
            let a = bigmath::atan(&((BigUint::from(i) << f) / 64u32));
            let q64: BigUint = (a + (BigUint::from(1u32) << (f - 65))) >> (f - 64);
            assert_eq!(q64, BigUint::from(want), "arctan({i}/64)");
        }
        let k2 = (BigUint::from(32768u32) << (f + 51)) / bigmath::pi();
        let k: BigUint = (k2 + 1u32) >> 1u32;
        assert_eq!(k, BigUint::from(FACINGS_PER_RADIAN_Q50));
    }

    // spec: S-0204 Edge cases (exact direction with `quirks.table_direction` off)
    // Displacements whose facing lies within 2^-60 of a half go to the
    // big-integer routine; plain ones do not (#393).
    #[test]
    fn fast_angle_falls_back_near_a_half() {
        // nocite: synthetic magnitudes near a rounding half, not game data
        let near: [(u64, u64); 4] = [
            (22915627, 159345772067),
            (95240465374, 95249596877),
            (175176187187, 6443712252),
            (757313546896, 648151647047),
        ];
        for (a, b) in near {
            assert_eq!(fast_angle(false, (a, 0), false, (b, 0)), None, "{a} {b}");
            assert_eq!(fast_angle(false, (b, 0), false, (a, 0)), None, "{b} {a}");
            assert_eq!(
                angle_of_parts(false, (a, 0), false, (b, 0)),
                exact_angle(false, &BigUint::from(a), false, &BigUint::from(b))
            );
        }
        assert_eq!(fast_angle(false, (1, 0), false, (1, 0)), Some(8192));
        assert_eq!(fast_angle(false, (1, 0), false, (0, 0)), Some(16384));
        assert_eq!(fast_angle(true, (0, 0), true, (1, 0)), Some(32768));
        assert_eq!(fast_angle(true, (1, 0), false, (0, 0)), Some(49152));
    }

    // spec: S-0204 claim 5 (s = 0 below 16641; r(16641) = 128; shortfall points)
    #[test]
    fn table_root_claim_5() {
        for n in 0..16641u64 {
            assert_eq!(root_floor(n, true, RZ), isqrt(u128::from(n)) as u64, "{n}");
        }
        assert_eq!(root_floor(16641, true, RZ), 128);
        assert_eq!(isqrt(16641), 129);
        // s first reaches 2 at 536987929 and 3 at 2147673649.
        let s = |n: u64| isqrt(u128::from(n)) as u64 - root_floor(n, true, RZ);
        assert_eq!(s(536987929), 2);
        assert!((536_000_000..536987929).step_by(997).all(|n| s(n) <= 1));
        assert_eq!(s(2147673649), 3);
        // Cell centres: r(65536 m) = 256 r(m) exactly.
        for m in 1..2000u64 {
            let (a, b) = (table_root(65536 * m, RZ), table_root(m, RZ));
            assert_eq!(
                F64::from_i64(256, RZ).mul(b.to_f64(), RZ).0,
                a.to_f64().0,
                "{m}"
            );
        }
        // r(n) never exceeds ⌊√n⌋ and is monotone.
        let mut prev = 0;
        for n in (0..10_000_000u64).step_by(9973) {
            let r = root_floor(n, true, RZ);
            assert!(r <= isqrt(u128::from(n)) as u64);
            assert!(r >= prev);
            prev = r;
        }
    }

    // spec: S-0204 claim 5 (the m counts for cells up to 511 apart)
    #[test]
    fn table_root_cell_counts() {
        let mut ms = std::collections::BTreeSet::new();
        for dx in 0..=511u64 {
            for dy in 0..=511u64 {
                if dx + dy > 0 {
                    ms.insert(dx * dx + dy * dy);
                }
            }
        }
        assert_eq!(ms.len(), 82489);
        let (mut exact, mut one) = (0, 0);
        for &m in &ms {
            let s = isqrt(u128::from(65536 * m)) as u64 - root_floor(65536 * m, true, RZ);
            match s {
                0 => exact += 1,
                1 => one += 1,
                _ => {}
            }
        }
        assert_eq!(exact, 23995);
        assert_eq!(one, 22923);
    }

    // spec: S-0204 F3 (test values)
    #[test]
    fn direction_test_values() {
        let cases: [(i64, i64, u16); 17] = [
            (0, -1, 0),
            (1, 0, 16383),
            (0, 1, 32767),
            (-1, 0, 49153),
            (-1000, 1, 49150),
            (-1, 200, 32766),
            (1, -1, 8315),
            (1, 1, 24451),
            (-1, 1, 41082),
            (-1, -1, 57221),
            (2560, -256, 15368),
            (256, -2560, 1041),
            (300, -200, 10304),
            (-700, 1300, 37922),
            (1, -100, 104),
            (1, -200, 0),
            (9, -97, 966),
        ];
        for (dx, dy, want) in cases {
            assert_eq!(direction(dx, dy, true), want, "({dx}, {dy})");
        }
    }

    // spec: S-0204 claim 8 (b) (nearest directions beside each axis and never-taken facings)
    #[test]
    fn direction_quantisation() {
        let mut taken = std::collections::BTreeSet::new();
        for dx in -300i64..=300 {
            for dy in -300i64..=300 {
                taken.insert(direction(dx, dy, true));
            }
        }
        for f in 1..=103u16 {
            assert!(!taken.contains(&f));
        }
        for f in 16129..=16382u16 {
            assert!(!taken.contains(&f));
        }
        assert!(taken.contains(&104));
        assert!(taken.contains(&65432));
        assert!(taken.contains(&16128));
        assert!(taken.contains(&16638));
        // C1 run g and run f expectations.
        assert_eq!(direction(256, -6400, true), 417);
        assert_eq!(direction(-2560, 256, true), 48135);
    }

    // spec: S-0204 Edge cases (`quirks.table_direction` off: exact angle, halves up)
    #[test]
    fn exact_direction_values() {
        assert_eq!(direction(1, 0, false), 16384);
        assert_eq!(direction(0, 1, false), 32768);
        assert_eq!(direction(-1, 0, false), 49152);
        assert_eq!(direction(1, -1, false), 8192);
        assert_eq!(direction(-1, -1, false), 57344);
        assert_eq!(direction(2560, -256, false), 15344);
        assert_eq!(direction(-2560, 256, false), 48112);
        assert_eq!(direction(256, -6400, false), 417);
    }

    // spec: S-0204 F4, F5 (n-step facings and the signed difference)
    #[test]
    fn facings() {
        assert_eq!(delta(0, 32768), -32768);
        assert_eq!(delta(0, 49153), -16383);
        assert_eq!(delta(49153, 16383), 32766);
        assert_eq!(n_step(4095, 8), 0);
        assert_eq!(n_step(4096, 8), 1);
        assert_eq!(n_step(65535, 8), 0);
        assert_eq!(n_step(1024, 32), 1);
        assert_eq!(n_step(127, 256), 0);
        assert_eq!(n_step(128, 256), 1);
    }

    // spec: S-0204 claim 12 and F6 (index 1024: 417 for a positive angle, 32349 for a negative one; e = 0 off the exception list)
    #[test]
    fn pitch_from_signed_table_angle_claim_12() {
        let a = F32(ATAN[1024]).to_f64();
        assert_eq!(pitch_of_angle(a, Some(1024), RZ).rem_euclid(65536), 417);
        assert_eq!(
            pitch_of_angle(a.negated(), Some(1024), RZ).rem_euclid(65536),
            32349
        );
        assert_eq!(pitch_of_angle(a, None, RZ).rem_euclid(65536), 416);
        assert_eq!(
            pitch_of_angle(a.negated(), None, RZ).rem_euclid(65536),
            32350
        );
        assert_eq!(
            pitch_of_angle(F64::ZERO, Some(1024), RZ),
            pitch_of_angle(F64::ZERO, None, RZ)
        );
    }
}
