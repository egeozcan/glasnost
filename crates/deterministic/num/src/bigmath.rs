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

/// round(v × sin α) and round(v × cos α) for the angle α = 2π f / 65536 of
/// a facing `f`, rounding half away from zero (S-0502 Edge cases,
/// `quirks.jumpjet_step_truncation` off: the exact functions, evaluated far
/// beyond double precision, then the integer rounding of the product).
pub fn rounded_sin_cos(f: u16, v: i32) -> (i32, i32) {
    // spec: S-0502 Edge cases (α = 2π F / 65536; a quarter turn is 16384 facing units)
    let quarter = u32::from(f) / 16384;
    let r = u32::from(f) % 16384;
    let half_pi = pi() >> 1u32;
    // spec: S-0502 Edge cases (θ = 2π r / 65536 = π r / 32768 within the quarter turn)
    let theta = (pi() * BigUint::from(r)) >> 15u32;
    let s = BigInt::from(sin(&theta));
    let c = BigInt::from(sin(&(half_pi - theta)));
    // The quadrant: (sin, cos) = (s, c), (c, −s), (−s, −c), (−c, s).
    let (a, b) = match quarter {
        0 => (s, c),
        1 => (c, -s),
        2 => (-s, -c),
        _ => (-c, s),
    };
    let round = |x: BigInt| -> i32 {
        let m = x * BigInt::from(v);
        let half = BigInt::from(1u32) << (FRAC - 1);
        let negative = m.sign() == Sign::Minus;
        let mag = if negative { -m } else { m };
        let q = i32::try_from((mag + half) >> FRAC).unwrap_or(0);
        if negative { -q } else { q }
    };
    (round(a), round(b))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pi_digits() {
        // π × 2^FRAC truncated, checked against its known leading bits.
        let p = pi() >> (FRAC - 56);
        assert_eq!(p, BigUint::from(0x0324_3F6A_8885_A308u64));
    }
}
