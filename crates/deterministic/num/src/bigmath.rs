//! Exact-enough reference arithmetic on big integers: π, arctangent and sine
//! in fixed point with [`FRAC`] fraction bits.
//!
//! The original reads roots, sines and angles from tables (S-0205 claim 6).
//! The tables of S-0204 (F3, arctangent surrogate) and S-0105 (F1, sine) are
//! defined as exact functions truncated to single precision. This module
//! computes those exact functions to far more precision than the truncation
//! needs, so that the committed tables can be checked against their
//! definitions, and gives the exact angle that `quirks.table_direction` off
//! asks for (S-0204 Edge cases). No floating point is involved.

use num_bigint::{BigInt, BigUint, Sign};

use crate::soft::{F32, F64, Rounding};

/// Fraction bits of the fixed-point values.
// nocite: working precision of this reference arithmetic, not game behaviour
pub const FRAC: u32 = 256;

fn one() -> BigUint {
    BigUint::from(1u32) << FRAC
}

/// arctan(x) for a fixed-point x = `x` / 2^FRAC with 0 ≤ x ≤ 1, by Euler's
/// series Σ 2^2n (n!)² / (2n+1)! · x^(2n+1) / (1 + x²)^(n+1).
fn atan_small(x: &BigUint) -> BigUint {
    let s = one();
    let y = (x * x) >> FRAC;
    let d = &s + &y;
    let mut term = (x * &s) / &d;
    let mut sum = term.clone();
    let mut n: u32 = 0;
    loop {
        // term_{n+1} = term_n × (2n+2)/(2n+3) × y/d
        let num = &term * (2 * n + 2) * &y;
        let den = &d * (2 * n + 3);
        term = num / den;
        if term == BigUint::from(0u32) {
            break;
        }
        sum += &term;
        n += 1;
    }
    sum
}

/// arctan of a non-negative fixed-point value.
pub fn atan(x: &BigUint) -> BigUint {
    let s = one();
    if x <= &s {
        atan_small(x)
    } else {
        // arctan(x) = π/2 − arctan(1/x)
        let inv = (&s * &s) / x;
        (pi() >> 1u32) - atan_small(&inv)
    }
}

/// π in fixed point (Machin's formula).
pub fn pi() -> BigUint {
    let s = one();
    let a = atan_small(&(&s / 5u32));
    let b = atan_small(&(&s / 239u32));
    (a * 16u32) - (b * 4u32)
}

/// sin(θ) for 0 ≤ θ ≤ π/2 in fixed point (Taylor series).
pub fn sin(theta: &BigUint) -> BigUint {
    let s = one();
    let t = BigInt::from(theta.clone());
    let t2 = (&t * &t) >> FRAC;
    let mut term = t.clone();
    let mut sum = t;
    let mut k: u32 = 1;
    loop {
        // term_{k+1} = −term_k × θ² / ((2k)(2k+1))
        term = -(&term * &t2 / BigInt::from(s.clone())) / BigInt::from((2 * k) * (2 * k + 1));
        if term.sign() == Sign::NoSign {
            break;
        }
        sum += &term;
        k += 1;
    }
    sum.to_biguint().unwrap_or_default()
}

/// The integer square root of a big integer (floor).
pub fn isqrt_big(n: &BigUint) -> BigUint {
    n.sqrt()
}

/// A positive fixed-point value truncated toward zero to single precision.
pub fn trunc_single(v: &BigUint) -> F32 {
    if v == &BigUint::from(0u32) {
        return F32::ZERO;
    }
    let bits = v.bits();
    // Keep 120 bits; truncation toward zero composes.
    let (m, e) = if bits > 120 {
        let sh = bits - 120;
        ((v >> sh), sh as i32 - FRAC as i32)
    } else {
        (v.clone(), -(FRAC as i32))
    };
    let m = u128::try_from(m).unwrap_or(0);
    F64::from_parts(false, m, e, Rounding::TowardZero).to_f32(Rounding::TowardZero)
}

/// The sign bit of a double.
// nocite: binary64 layout
const SIGN64: u64 = 1 << 63;

/// Bits of a fixed-point magnitude kept before the final rounding.
// nocite: working precision of this reference arithmetic, not game behaviour
const KEEP: u64 = 120;

/// A double as a signed fixed-point value with [`FRAC`] fraction bits
/// (truncated toward zero below 2^−FRAC); `None` for infinities and NaN.
fn fixed_of(x: F64) -> Option<BigInt> {
    if x.0 & !SIGN64 == 0 {
        return Some(BigInt::from(0u32));
    }
    let (neg, m, e) = x.parts()?;
    let m = BigUint::from(m);
    let shift = e + FRAC as i32;
    let v = if shift >= 0 {
        m << shift.unsigned_abs()
    } else {
        m >> shift.unsigned_abs()
    };
    let v = BigInt::from(v);
    Some(if neg { -v } else { v })
}

/// A signed fixed-point value rounded to a double in `mode`.
///
/// The magnitude is first cut to its leading [`KEEP`] bits, toward zero,
/// which composes with round-toward-zero; with round-to-nearest the result
/// can differ only for a value within 2^−119 relative of a tie.
fn double_of(v: &BigInt, mode: Rounding) -> F64 {
    let neg = v.sign() == Sign::Minus;
    let a = v.magnitude();
    let bits = a.bits();
    if bits == 0 {
        return F64::ZERO;
    }
    let (m, e) = if bits > KEEP {
        let sh = bits - KEEP;
        (a >> sh, sh as i32 - FRAC as i32)
    } else {
        (a.clone(), -(FRAC as i32))
    };
    F64::from_parts(neg, u128::try_from(m).unwrap_or(0), e, mode)
}

/// sin of a fixed-point angle *r* in 0 … 2π, by the quarter-wave symmetries
/// of the sine (`p` is π).
fn sin_reduced(r: &BigUint, p: &BigUint) -> BigInt {
    let half = p >> 1u32;
    let three_halves = p + &half;
    let two_pi = p << 1u32;
    if r <= &half {
        BigInt::from(sin(r))
    } else if r <= p {
        BigInt::from(sin(&(p - r)))
    } else if r <= &three_halves {
        -BigInt::from(sin(&(r - p)))
    } else {
        -BigInt::from(sin(&(&two_pi - r.min(&two_pi))))
    }
}

/// The exact sine and cosine of a double, each rounded once to a double
/// in `mode` (with round-toward-zero: RZ₅₃(sin *x*) and RZ₅₃(cos *x*)).
/// The harvesting-sprite anchor of S-0402 F8 uses them, on the literal
/// reading of F8 (plain sin and cos, rounded toward zero). Whether F8
/// means table sines instead, as S-0205 claim 6 and sibling formulas do,
/// is open in Q-0580. `None` for infinities
/// and NaN. The sine of a zero keeps the zero's sign.
///
/// The working precision ([`FRAC`] bits) is far beyond a double's, so the
/// result is the correctly rounded value unless the exact one lies within
/// about 2^−250 of a double boundary.
pub fn sin_cos(x: F64, mode: Rounding) -> Option<(F64, F64)> {
    let v = fixed_of(x)?;
    let p = pi();
    let two_pi = &p << 1u32;
    // Reduce to 0 … 2π.
    let mut r = &v % BigInt::from(two_pi.clone());
    if r.sign() == Sign::Minus {
        r += BigInt::from(two_pi.clone());
    }
    let r = r.magnitude().clone();
    let s = sin_reduced(&r, &p);
    // cos x = sin(x + π/2).
    let mut rc = &r + (&p >> 1u32);
    if rc >= two_pi {
        rc -= &two_pi;
    }
    let c = sin_reduced(&rc, &p);
    let s = if s.sign() == Sign::NoSign {
        F64(x.0 & SIGN64)
    } else {
        double_of(&s, mode)
    };
    Some((s, double_of(&c, mode)))
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: S-0402 F8 (trigonometric results rounded toward zero: sin and cos of a double, rounded once)
    #[test]
    fn sin_cos_rounded_once() {
        let rz = Rounding::TowardZero;
        let rn = Rounding::Nearest;
        // π_d / 2 lies just below π/2: its sine rounds toward zero to the double below 1
        // (1 − 2^−53) and to nearest to 1; its cosine is a tiny positive value.
        let half_pi = F64::from_parts(false, 884279719003555, -49, rz);
        let (s, c) = sin_cos(half_pi, rz).unwrap();
        assert_eq!(s, F64(0x3fef_ffff_ffff_ffff));
        assert!(c.gt(F64::ZERO) && c.lt(F64::from_parts(false, 1, -53, rz)));
        let (s, _) = sin_cos(half_pi, rn).unwrap();
        assert_eq!(s, F64::from_i64(1, rz));
        // Zeros: the sine keeps the sign, the cosine is 1.
        assert_eq!(
            sin_cos(F64::ZERO, rz),
            Some((F64::ZERO, F64::from_i64(1, rz)))
        );
        let neg_zero = F64(SIGN64);
        assert_eq!(
            sin_cos(neg_zero, rz),
            Some((neg_zero, F64::from_i64(1, rz)))
        );
        // Odd and even symmetry, and a negative angle reduced into 0 … 2π.
        let x = F64::from_parts(false, 3, -2, rz); // 0.75
        let (s1, c1) = sin_cos(x, rz).unwrap();
        let (s2, c2) = sin_cos(x.negated(), rz).unwrap();
        assert_eq!((s2, c2), (s1.negated(), c1));
        // sin 0.75 = 0.68163876002333416673…, cos 0.75 = 0.73168886887382088631…
        let close = |v: F64, num: u128, den_pow: i32| {
            let lo = F64::from_parts(false, num, -den_pow, rz);
            let hi = F64::from_parts(false, num + 1, -den_pow, rz);
            v.ge(lo) && v.lt(hi)
        };
        // 0.68163876002333416673 × 2^40 = 749 469 742 588.47…; 0.73168886887382088631 × 2^40 = 804 500 419 241.03…
        assert!(close(s1, 749_469_742_588, 40), "{s1:?}");
        assert!(close(c1, 804_500_419_241, 40), "{c1:?}");
        // Infinities and NaN have none.
        assert_eq!(sin_cos(F64(0x7ff0_0000_0000_0000), rz), None);
    }

    #[test]
    fn pi_digits() {
        // π × 2^FRAC truncated, checked against its known leading bits.
        let p = pi() >> (FRAC - 56);
        assert_eq!(p, BigUint::from(0x0324_3F6A_8885_A308u64));
    }
}
