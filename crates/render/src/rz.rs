//! Round-toward-zero floating-point arithmetic (S-0105 Formulas F0).
//!
//! S-0105 F0 states that every floating-point operation of voxel rendering
//! runs with 53-bit (double) intermediates **rounded toward zero**, and that
//! every single-precision store truncates toward zero. Rust's arithmetic
//! rounds to nearest, so these helpers recover the round-toward-zero result
//! exactly from the nearest one with error-free transformations:
//!
//! - addition and subtraction: TwoSum gives the exact rounding error;
//! - multiplication: a fused multiply-add gives the exact residual
//!   `a·b − p`;
//! - division: a fused multiply-add gives the exact remainder `a − q·b`;
//! - double → single: the nearest single is compared with the double.
//!
//! When the nearest result lies farther from zero than the exact value, the
//! result steps one unit in the last place toward zero. A finite exact value
//! whose nearest result overflows to infinity gives the largest finite value
//! of its sign, as round-toward-zero does. Residuals are exact as long as no
//! intermediate underflows into the subnormal range, which voxel rendering
//! never comes near (its values are pixel coordinates and unit vectors).
//!
//! Only rendering uses floating point; the simulation never does.

/// One unit in the last place toward zero (`x` finite and non-zero).
fn toward_zero_f64(x: f64) -> f64 {
    if x == 0.0 || !x.is_finite() {
        return x;
    }
    // Decrementing the bit pattern lowers the magnitude for either sign.
    f64::from_bits(x.to_bits() - 1)
}

/// One unit in the last place toward zero (`x` finite and non-zero).
fn toward_zero_f32(x: f32) -> f32 {
    if x == 0.0 || !x.is_finite() {
        return x;
    }
    f32::from_bits(x.to_bits() - 1)
}

/// The nearest result `r` of an operation whose exact value is `r + err`
/// (`err` exact), corrected to round toward zero.
fn correct(r: f64, err: f64) -> f64 {
    if r.is_infinite() {
        return r;
    }
    // The exact value is closer to zero than `r` when the error points the
    // other way from `r`.
    if err != 0.0 && (err < 0.0) != (r < 0.0) {
        toward_zero_f64(r)
    } else {
        r
    }
}

/// An overflow to infinity from finite operands: round-toward-zero gives the
/// largest finite value instead.
fn clamp_overflow(r: f64, finite_inputs: bool) -> f64 {
    if r.is_infinite() && finite_inputs {
        f64::MAX.copysign(r)
    } else {
        r
    }
}

/// `a + b` at 53-bit precision, rounded toward zero (S-0105 F0).
pub fn add_rz(a: f64, b: f64) -> f64 {
    let s = a + b;
    if !s.is_finite() {
        return clamp_overflow(s, a.is_finite() && b.is_finite());
    }
    // TwoSum (Knuth): s + e == a + b exactly.
    let bb = s - a;
    let e = (a - (s - bb)) + (b - bb);
    correct(s, e)
}

/// `a − b` at 53-bit precision, rounded toward zero (S-0105 F0).
pub fn sub_rz(a: f64, b: f64) -> f64 {
    add_rz(a, -b)
}

/// `a × b` at 53-bit precision, rounded toward zero (S-0105 F0).
pub fn mul_rz(a: f64, b: f64) -> f64 {
    let p = a * b;
    if !p.is_finite() {
        return clamp_overflow(p, a.is_finite() && b.is_finite());
    }
    // a·b − p, exact (fused multiply-add rounds once, and the residual of a
    // product is representable).
    let e = a.mul_add(b, -p);
    correct(p, e)
}

/// `a ÷ b` at 53-bit precision, rounded toward zero (S-0105 F0).
pub fn div_rz(a: f64, b: f64) -> f64 {
    let q = a / b;
    if !q.is_finite() {
        return clamp_overflow(q, a.is_finite() && b.is_finite() && b != 0.0);
    }
    if q == 0.0 {
        return q;
    }
    // r = a − q·b exactly; the exact quotient is q + r / b.
    let r = (-q).mul_add(b, a);
    let err_sign_negative = (r < 0.0) != (b < 0.0);
    if r != 0.0 && err_sign_negative != (q < 0.0) {
        toward_zero_f64(q)
    } else {
        q
    }
}

/// A double stored in single precision, truncated toward zero (S-0105 F0).
pub fn to_f32_rz(x: f64) -> f32 {
    let f = x as f32;
    if f.is_infinite() && x.is_finite() {
        return f32::MAX.copysign(f);
    }
    if f.is_finite() && f64::from(f).abs() > x.abs() {
        toward_zero_f32(f)
    } else {
        f
    }
}

/// `a × b` of two singles, stored in single precision truncated toward zero
/// (the product itself is exact at 53 bits).
pub fn mul_f32_rz(a: f32, b: f32) -> f32 {
    to_f32_rz(mul_rz(f64::from(a), f64::from(b)))
}

/// A dot product of single-precision vectors at 53-bit precision rounded
/// toward zero, summed left to right: ((a₀b₀ + a₁b₁) + a₂b₂). Not stored.
pub fn dot3_rz(a: [f32; 3], b: [f32; 3]) -> f64 {
    let p = |i: usize| mul_rz(f64::from(a[i]), f64::from(b[i]));
    add_rz(add_rz(p(0), p(1)), p(2))
}

/// Float to integer conversion: truncation toward zero (S-0105 F0), with
/// NaN as 0 and out-of-range values saturated (hostile model data only).
pub fn trunc_i64(x: f64) -> i64 {
    x as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    /// spec: S-0105 Formulas F0 (sums round toward zero, not to nearest)
    #[test]
    fn add_truncates_where_nearest_rounds_up() {
        // 1 + 0.75 ulp: nearest gives 1 + ulp, toward zero gives 1.
        let ulp = f64::EPSILON;
        assert_eq!(1.0 + 0.75 * ulp, 1.0 + ulp);
        assert_eq!(add_rz(1.0, 0.75 * ulp), 1.0);
        // Negative: −1 − 0.75 ulp truncates to −1.
        assert_eq!(add_rz(-1.0, -0.75 * ulp), -1.0);
        // Exact sums stay.
        assert_eq!(add_rz(1.5, 2.25), 3.75);
        // 1 − 0.25 ulp(1): the exact value lies just below 1, between
        // 1 − ulp/2 and 1; toward zero gives 1 − ulp/2.
        let below = 1.0 - ulp / 2.0;
        assert_eq!(add_rz(1.0, -0.25 * ulp), below);
        assert_eq!(sub_rz(1.0, 0.25 * ulp), below);
        // Halfway cases: nearest-even rounds 1 + ulp/2 down already.
        assert_eq!(add_rz(1.0 + ulp, ulp / 2.0), 1.0 + ulp);
    }

    /// spec: S-0105 Formulas F0
    #[test]
    fn mul_and_div_truncate() {
        // (1 + ulp)² = 1 + 2ulp + ulp²: nearest 1 + 2ulp, exact just above,
        // so both agree; (1 − ulp/2)·(1 + ulp) = 1 + ulp/2 − ulp²/2 lies just
        // below 1 + ulp/2, so nearest gives 1 and toward zero also 1.
        let ulp = f64::EPSILON;
        assert_eq!(mul_rz(1.0 + ulp, 1.0 + ulp), 1.0 + 2.0 * ulp);
        // 3 × (1/3 rounded): known product just below 1.
        let third = 1.0 / 3.0;
        assert_eq!(third * 3.0, 1.0);
        assert!(mul_rz(third, 3.0) < 1.0);
        assert_eq!(mul_rz(third, 3.0), 1.0 - ulp / 2.0);
        assert_eq!(mul_rz(-third, 3.0), -(1.0 - ulp / 2.0));
        // 1/10: nearest rounds up (0.1000…0055), toward zero down.
        let near = 1.0 / 10.0;
        let rz = div_rz(1.0, 10.0);
        assert!(rz < near);
        assert_eq!(rz, f64::from_bits(near.to_bits() - 1));
        assert_eq!(div_rz(-1.0, 10.0), -rz);
        assert_eq!(div_rz(1.0, -10.0), -rz);
        // 1/3: nearest rounds down already.
        assert_eq!(div_rz(1.0, 3.0), 1.0 / 3.0);
        assert_eq!(div_rz(6.0, 4.0), 1.5);
    }

    /// spec: S-0105 Formulas F0 (single-precision stores truncate)
    #[test]
    fn single_stores_truncate() {
        // 0.1 as f32 nearest is above 0.1; truncation steps below.
        let near = 0.1f64 as f32;
        assert!(f64::from(near) > 0.1);
        let t = to_f32_rz(0.1);
        assert_eq!(t, f32::from_bits(near.to_bits() - 1));
        assert_eq!(to_f32_rz(-0.1), -t);
        // Exactly representable values stay.
        assert_eq!(to_f32_rz(0.375), 0.375);
        // 0.7 as f32 nearest is below already.
        assert!(f64::from(0.7f64 as f32) < 0.7);
        assert_eq!(to_f32_rz(0.7), 0.7f64 as f32);
        // Overflow truncates to the largest finite single.
        assert_eq!(to_f32_rz(1e300), f32::MAX);
        assert_eq!(to_f32_rz(-1e300), -f32::MAX);
        assert!(to_f32_rz(f64::NAN).is_nan());
    }

    /// spec: S-0105 Formulas F0
    #[test]
    fn overflow_and_specials() {
        assert_eq!(add_rz(f64::MAX, f64::MAX), f64::MAX);
        assert_eq!(mul_rz(-f64::MAX, 2.0), -f64::MAX);
        assert_eq!(add_rz(f64::INFINITY, 1.0), f64::INFINITY);
        assert!(div_rz(1.0, 0.0).is_infinite());
        assert!(add_rz(f64::NAN, 1.0).is_nan());
        assert_eq!(trunc_i64(f64::NAN), 0);
        assert_eq!(trunc_i64(-2.9), -2);
    }

    /// spec: S-0105 Formulas F0 (random cross-check against exact rationals)
    #[test]
    fn matches_exact_reference_on_small_dyadics() {
        // For dyadic operands the exact sum is representable in i128 units;
        // compare the sign of (exact − result) with the expected direction.
        let mut s = 0x1234_5678_9ABC_DEF0u64;
        let mut next = || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s
        };
        for _ in 0..20000 {
            let a = (next() >> 11) as i64 - (1 << 52);
            let shift = (next() % 60) as i32;
            let b = (next() >> 11) as i64 - (1 << 52);
            // a + b / 2^shift, exactly, in units of 2^-shift.
            let af = a as f64;
            let bf = b as f64 / f64::powi(2.0, shift);
            let r = add_rz(af, bf);
            let exact = (i128::from(a) << shift) + i128::from(b);
            let r_units = (r * f64::powi(2.0, shift)) as i128;
            // |r| ≤ |exact| and the next value away from zero overshoots.
            assert!(r_units.abs() <= exact.abs(), "{a} {b} {shift}");
            if r_units == exact {
                continue;
            }
            // Inexact: the next value away from zero must overshoot.
            let away = if r >= 0.0 { r.next_up() } else { r.next_down() };
            let away_units = away * f64::powi(2.0, shift);
            assert_eq!(away_units.fract(), 0.0);
            assert!((away_units as i128).abs() > exact.abs(), "{a} {b} {shift}");
        }
    }
}
