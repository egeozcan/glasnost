//! IEEE 754 binary32 and binary64 arithmetic computed with integers only.
//!
//! S-0205 (number domains) fixes how the original computes in floating
//! point: every arithmetic result is formed with a 53-bit significand
//! (claim 4) and rounded toward zero (claim 5), and a narrowing to single
//! precision rounds toward zero too. With `quirks.chop_rounding` off, results
//! round to nearest, ties to even. A conversion to an integer truncates and
//! wraps (claim 7).
//!
//! The simulation must give bit-identical results on every platform, so no
//! host floating-point unit is involved: values are kept as their bit
//! patterns ([`F64`], [`F32`]) and every operation works on `u64`/`u128`.
//! Results are exact IEEE 754 results under the chosen [`Rounding`], gradual
//! underflow included. NaN results are the canonical quiet NaN.
//!
//! The numeric literals in this file describe the IEEE 754 formats (field
//! widths, exponent bias), not game behaviour.

use serde::{Deserialize, Serialize};

/// How an arithmetic result or a narrowing is rounded (S-0205 claim 5,
/// `quirks.chop_rounding`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rounding {
    /// Round toward zero (RZ): the original's program-wide mode.
    #[default]
    TowardZero,
    /// Round to nearest, ties to even (RN).
    Nearest,
}

/// A double (binary64) held as its bit pattern.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct F64(pub u64);

/// A single (binary32) held as its bit pattern.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct F32(pub u32);

/// A binary floating-point format.
#[derive(Clone, Copy, Debug)]
struct Fmt {
    /// Significand bits, hidden bit included.
    p: u32,
    /// Exponent of the smallest normal value.
    emin: i32,
    /// Exponent of the largest finite value.
    emax: i32,
}

// nocite: IEEE 754 binary64 layout
const DOUBLE: Fmt = Fmt {
    p: 53,
    emin: -1022,
    emax: 1023,
};

// nocite: IEEE 754 binary32 layout
const SINGLE: Fmt = Fmt {
    p: 24,
    emin: -126,
    emax: 127,
};

impl Fmt {
    fn frac_bits(self) -> u32 {
        self.p - 1
    }

    fn exp_mask(self) -> u64 {
        // nocite: IEEE 754 layout (all-ones exponent field)
        let ebits = if self.p == DOUBLE.p { 11 } else { 8 };
        (1u64 << ebits) - 1
    }

    fn sign_bit(self) -> u64 {
        // nocite: IEEE 754 layout (sign is the top bit)
        let total = if self.p == DOUBLE.p { 64 } else { 32 };
        1u64 << (total - 1)
    }

    fn quiet_nan(self) -> u64 {
        (self.exp_mask() << self.frac_bits()) | (1u64 << (self.frac_bits() - 1))
    }

    fn infinity(self, neg: bool) -> u64 {
        let m = self.exp_mask() << self.frac_bits();
        if neg { m | self.sign_bit() } else { m }
    }

    fn zero(self, neg: bool) -> u64 {
        if neg { self.sign_bit() } else { 0 }
    }

    fn max_finite(self, neg: bool) -> u64 {
        let m = ((self.exp_mask() - 1) << self.frac_bits()) | ((1u64 << self.frac_bits()) - 1);
        if neg { m | self.sign_bit() } else { m }
    }
}

/// A decoded value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Val {
    Zero(bool),
    /// `(-1)^neg × m × 2^e`, with `m` nonzero (normalised or subnormal).
    Finite {
        neg: bool,
        m: u64,
        e: i32,
    },
    Inf(bool),
    NaN,
}

fn decode(f: Fmt, bits: u64) -> Val {
    let neg = bits & f.sign_bit() != 0;
    let fb = f.frac_bits();
    let frac = bits & ((1u64 << fb) - 1);
    let ef = (bits >> fb) & f.exp_mask();
    let bias = f.emax;
    if ef == f.exp_mask() {
        return if frac == 0 { Val::Inf(neg) } else { Val::NaN };
    }
    if ef == 0 {
        if frac == 0 {
            return Val::Zero(neg);
        }
        return Val::Finite {
            neg,
            m: frac,
            e: f.emin - fb as i32,
        };
    }
    Val::Finite {
        neg,
        m: frac | (1u64 << fb),
        e: ef as i32 - bias - fb as i32,
    }
}

/// Rounds `(-1)^neg × m × 2^e` to format `f`. When `m`'s lowest bit is a
/// *jam* bit (set because nonzero bits were shifted out), the caller must
/// leave at least two bits below the precision kept, which every operation
/// here does by working with a wide `m`.
fn round_pack(f: Fmt, neg: bool, m: u128, e: i32, mode: Rounding) -> u64 {
    if m == 0 {
        return f.zero(neg);
    }
    let bits = 128 - m.leading_zeros() as i32;
    let p = f.p as i32;
    let mut shift = bits - p;
    // Exponent of the lowest kept bit.
    let mut e_lo = e + shift;
    let e_lo_min = f.emin - (p - 1);
    if e_lo < e_lo_min {
        shift += e_lo_min - e_lo;
        e_lo = e_lo_min;
    }
    let mut q: u128;
    if shift <= 0 {
        q = m << (-shift) as u32;
    } else if shift >= 128 {
        // Everything is below the kept precision.
        q = match mode {
            Rounding::TowardZero => 0,
            Rounding::Nearest => 0,
        };
    } else {
        let s = shift as u32;
        q = m >> s;
        let rem = m & ((1u128 << s) - 1);
        if mode == Rounding::Nearest {
            let half = 1u128 << (s - 1);
            if rem > half || (rem == half && q & 1 == 1) {
                q += 1;
            }
        }
    }
    if q >> f.p != 0 {
        q >>= 1;
        e_lo += 1;
    }
    // Leading bit's exponent.
    let top = e_lo + p - 1;
    if q == 0 {
        return f.zero(neg);
    }
    if q >> (f.p - 1) != 0 && top > f.emax {
        return match mode {
            Rounding::TowardZero => f.max_finite(neg),
            Rounding::Nearest => f.infinity(neg),
        };
    }
    let fb = f.frac_bits();
    let q = q as u64;
    let magnitude = if q >> fb == 0 {
        // Subnormal (e_lo is at its minimum).
        q
    } else {
        let biased = (top + f.emax) as u64;
        (biased << fb) | (q & ((1u64 << fb) - 1))
    };
    if neg {
        magnitude | f.sign_bit()
    } else {
        magnitude
    }
}

/// Shifts `m` right by `s` with a jam bit.
fn shr_jam(m: u128, s: u32) -> u128 {
    if s == 0 {
        m
    } else if s >= 128 {
        u128::from(m != 0)
    } else {
        let lost = m & ((1u128 << s) - 1);
        (m >> s) | u128::from(lost != 0)
    }
}

/// Extra bits kept below the operands in additions and divisions.
// nocite: working precision of this implementation, not game behaviour
const GUARD: u32 = 70;

fn add_impl(f: Fmt, a: u64, b: u64, mode: Rounding) -> u64 {
    let (va, vb) = (decode(f, a), decode(f, b));
    match (va, vb) {
        (Val::NaN, _) | (_, Val::NaN) => f.quiet_nan(),
        (Val::Inf(x), Val::Inf(y)) => {
            if x == y {
                f.infinity(x)
            } else {
                f.quiet_nan()
            }
        }
        (Val::Inf(x), _) | (_, Val::Inf(x)) => f.infinity(x),
        (Val::Zero(x), Val::Zero(y)) => f.zero(x && y),
        (Val::Zero(_), _) => b,
        (_, Val::Zero(_)) => a,
        (
            Val::Finite {
                neg: na,
                m: ma,
                e: ea,
            },
            Val::Finite {
                neg: nb,
                m: mb,
                e: eb,
            },
        ) => {
            // Order by magnitude exponent of the leading bit.
            let lead = |m: u64, e: i32| e + 64 - m.leading_zeros() as i32;
            let (big, small) = if (lead(ma, ea), ma << ma.leading_zeros())
                >= (lead(mb, eb), mb << mb.leading_zeros())
            {
                ((na, ma, ea), (nb, mb, eb))
            } else {
                ((nb, mb, eb), (na, ma, ea))
            };
            let (nbig, mbig, ebig) = big;
            let (nsmall, msmall, esmall) = small;
            let base = ebig - GUARD as i32;
            let abig = u128::from(mbig) << GUARD;
            let d = esmall - base;
            let asmall = if d >= 0 {
                let d = d as u32;
                if d >= 64 {
                    // Cannot happen for finite inputs of one format with the
                    // big operand chosen above; keep exact anyway.
                    u128::from(msmall) << d.min(GUARD + 53)
                } else {
                    u128::from(msmall) << d
                }
            } else {
                shr_jam(u128::from(msmall), (-d) as u32)
            };
            if nbig == nsmall {
                round_pack(f, nbig, abig + asmall, base, mode)
            } else if abig >= asmall {
                let diff = abig - asmall;
                if diff == 0 {
                    return f.zero(false);
                }
                round_pack(f, nbig, diff, base, mode)
            } else {
                round_pack(f, nsmall, asmall - abig, base, mode)
            }
        }
    }
}

fn mul_impl(f: Fmt, a: u64, b: u64, mode: Rounding) -> u64 {
    let (va, vb) = (decode(f, a), decode(f, b));
    match (va, vb) {
        (Val::NaN, _) | (_, Val::NaN) => f.quiet_nan(),
        (Val::Inf(x), Val::Zero(_)) | (Val::Zero(_), Val::Inf(x)) => {
            let _ = x;
            f.quiet_nan()
        }
        (Val::Inf(x), Val::Inf(y)) => f.infinity(x != y),
        (Val::Inf(x), Val::Finite { neg, .. }) | (Val::Finite { neg, .. }, Val::Inf(x)) => {
            f.infinity(x != neg)
        }
        (Val::Zero(x), Val::Zero(y)) => f.zero(x != y),
        (Val::Zero(x), Val::Finite { neg, .. }) | (Val::Finite { neg, .. }, Val::Zero(x)) => {
            f.zero(x != neg)
        }
        (
            Val::Finite {
                neg: na,
                m: ma,
                e: ea,
            },
            Val::Finite {
                neg: nb,
                m: mb,
                e: eb,
            },
        ) => {
            // The exact product; widen so that a jam-free result keeps two
            // bits below the precision even for subnormal operands.
            let m = u128::from(ma) * u128::from(mb);
            round_pack(f, na != nb, m << 2, ea + eb - 2, mode)
        }
    }
}

fn div_impl(f: Fmt, a: u64, b: u64, mode: Rounding) -> u64 {
    let (va, vb) = (decode(f, a), decode(f, b));
    match (va, vb) {
        (Val::NaN, _) | (_, Val::NaN) => f.quiet_nan(),
        (Val::Inf(_), Val::Inf(_)) | (Val::Zero(_), Val::Zero(_)) => f.quiet_nan(),
        (Val::Inf(x), Val::Zero(y)) => f.infinity(x != y),
        (Val::Inf(x), Val::Finite { neg, .. }) => f.infinity(x != neg),
        (Val::Zero(x), Val::Inf(y)) => f.zero(x != y),
        (Val::Finite { neg, .. }, Val::Inf(y)) => f.zero(neg != y),
        (Val::Zero(x), Val::Finite { neg, .. }) => f.zero(x != neg),
        (Val::Finite { neg, .. }, Val::Zero(y)) => f.infinity(neg != y),
        (
            Val::Finite {
                neg: na,
                m: ma,
                e: ea,
            },
            Val::Finite {
                neg: nb,
                m: mb,
                e: eb,
            },
        ) => {
            // Normalise both significands to 64 bits.
            let sa = ma.leading_zeros();
            let sb = mb.leading_zeros();
            let na_m = u128::from(ma << sa);
            let nb_m = u128::from(mb << sb);
            let num = na_m << 63;
            let q = num / nb_m;
            let r = num % nb_m;
            let q = (q << 1) | u128::from(r != 0);
            let e = (ea - sa as i32) - (eb - sb as i32) - 63 - 1;
            round_pack(f, na != nb, q, e, mode)
        }
    }
}

impl F64 {
    /// Positive zero.
    pub const ZERO: F64 = F64(0);

    /// Whether the value is a NaN.
    pub fn is_nan(self) -> bool {
        decode(DOUBLE, self.0) == Val::NaN
    }

    /// Whether the sign bit is set.
    pub fn is_sign_negative(self) -> bool {
        self.0 & DOUBLE.sign_bit() != 0
    }

    /// The exact value of an integer, rounded when its magnitude needs more
    /// than 53 bits.
    pub fn from_i64(v: i64, mode: Rounding) -> F64 {
        let neg = v < 0;
        let a = v.unsigned_abs();
        if a == 0 {
            return F64(0);
        }
        if a < 1 << 53 {
            // Exact: the leading bit sets the exponent, the rest is the fraction.
            let lead = 63 - a.leading_zeros();
            let frac = (a << (52 - lead)) & ((1 << 52) - 1);
            let bits = ((u64::from(lead) + 1023) << 52) | frac;
            return F64(if neg { bits | 1 << 63 } else { bits });
        }
        F64(round_pack(
            DOUBLE,
            neg,
            u128::from(v.unsigned_abs()) << 2,
            -2,
            mode,
        ))
    }

    /// `m × 2^e` exactly, rounded to a double.
    pub fn from_parts(neg: bool, m: u128, e: i32, mode: Rounding) -> F64 {
        let lead = 128 - m.leading_zeros();
        if m == 0 {
            return F64(DOUBLE.zero(neg));
        }
        // Make room for two rounding bits when `m` is narrow.
        if lead < 126 {
            F64(round_pack(DOUBLE, neg, m << 2, e - 2, mode))
        } else {
            F64(round_pack(DOUBLE, neg, m, e, mode))
        }
    }

    /// `self + other`.
    pub fn add(self, other: F64, mode: Rounding) -> F64 {
        F64(add_impl(DOUBLE, self.0, other.0, mode))
    }

    /// `self − other`.
    pub fn sub(self, other: F64, mode: Rounding) -> F64 {
        F64(add_impl(DOUBLE, self.0, other.negated().0, mode))
    }

    /// `self × other`.
    pub fn mul(self, other: F64, mode: Rounding) -> F64 {
        F64(mul_impl(DOUBLE, self.0, other.0, mode))
    }

    /// `self ÷ other`.
    pub fn div(self, other: F64, mode: Rounding) -> F64 {
        F64(div_impl(DOUBLE, self.0, other.0, mode))
    }

    /// The negation (sign flip, exact).
    pub fn negated(self) -> F64 {
        F64(self.0 ^ DOUBLE.sign_bit())
    }

    /// The magnitude (sign cleared, exact).
    pub fn abs(self) -> F64 {
        F64(self.0 & !DOUBLE.sign_bit())
    }

    /// The narrowing to single precision (S-0205 claim 5).
    pub fn to_f32(self, mode: Rounding) -> F32 {
        match decode(DOUBLE, self.0) {
            Val::NaN => F32(SINGLE.quiet_nan() as u32),
            Val::Inf(n) => F32(SINGLE.infinity(n) as u32),
            Val::Zero(n) => F32(SINGLE.zero(n) as u32),
            Val::Finite { neg, m, e } => {
                F32(round_pack(SINGLE, neg, u128::from(m) << 2, e - 2, mode) as u32)
            }
        }
    }

    /// Truncation toward zero to a whole number, as an `i128` (exact for
    /// every finite double below 2^127 in magnitude); `None` for NaN,
    /// infinities and larger values.
    pub fn trunc_i128(self) -> Option<i128> {
        match decode(DOUBLE, self.0) {
            Val::Zero(_) => Some(0),
            Val::Finite { neg, m, e } => {
                let mag: i128 = if e >= 0 {
                    // nocite: i128 range (m < 2^53, so m × 2^74 < 2^127)
                    if e > 74 {
                        return None;
                    }
                    i128::from(m) << e
                } else if e <= -64 {
                    0
                } else {
                    i128::from(m >> (-e) as u32)
                };
                Some(if neg { -mag } else { mag })
            }
            _ => None,
        }
    }

    /// The conversion to an integer of S-0205 claim 7, common form: the
    /// value truncated toward zero and wrapped modulo 2^32 when its
    /// magnitude is below 2^63; 0 otherwise (infinities and NaN included).
    pub fn to_i32_common(self) -> i32 {
        // spec: S-0205 claim 7 (zero at 2^63 and above; truncate and wrap below)
        let limit: i128 = 1 << 63;
        match self.trunc_i128() {
            Some(t) if t.abs() < limit && !self.at_least_2_63() => t as i32,
            _ => 0,
        }
    }

    /// The conversion of S-0205 claim 7, 32-bit form: the truncated value in
    /// range, −2147483648 otherwise.
    pub fn to_i32_32bit(self) -> i32 {
        match self.trunc_i128() {
            // spec: S-0205 claim 7 (32-bit form)
            Some(t) if (i128::from(i32::MIN)..=i128::from(i32::MAX)).contains(&t) => t as i32,
            _ => i32::MIN,
        }
    }

    fn at_least_2_63(self) -> bool {
        match decode(DOUBLE, self.0) {
            // spec: S-0205 claim 7 (the zero threshold is 2^63)
            Val::Finite { m, e, .. } => e + 64 - m.leading_zeros() as i32 > 63,
            Val::Inf(_) | Val::NaN => true,
            Val::Zero(_) => false,
        }
    }

    /// Exact numeric comparison (S-0205 claim 4); `None` when either is NaN.
    pub fn cmp_num(self, other: F64) -> Option<std::cmp::Ordering> {
        if self.is_nan() || other.is_nan() {
            return None;
        }
        Some(order_key(DOUBLE, self.0).cmp(&order_key(DOUBLE, other.0)))
    }

    /// `self < other`, exact.
    pub fn lt(self, other: F64) -> bool {
        self.cmp_num(other) == Some(std::cmp::Ordering::Less)
    }

    /// `self ≤ other`, exact.
    pub fn le(self, other: F64) -> bool {
        matches!(
            self.cmp_num(other),
            Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)
        )
    }

    /// `self > other`, exact.
    pub fn gt(self, other: F64) -> bool {
        other.lt(self)
    }

    /// `self ≥ other`, exact.
    pub fn ge(self, other: F64) -> bool {
        other.le(self)
    }

    /// Numeric equality, exact (+0 equals −0; NaN equals nothing).
    pub fn eq_num(self, other: F64) -> bool {
        self.cmp_num(other) == Some(std::cmp::Ordering::Equal)
    }

    /// The smaller of two values (the first when equal).
    pub fn min(self, other: F64) -> F64 {
        if other.lt(self) { other } else { self }
    }

    /// The larger of two values (the first when equal).
    pub fn max(self, other: F64) -> F64 {
        if other.gt(self) { other } else { self }
    }

    /// The exact value as `(neg, m, e)` with `value = (-1)^neg × m × 2^e`;
    /// `None` for zero, infinities and NaN.
    pub fn parts(self) -> Option<(bool, u64, i32)> {
        match decode(DOUBLE, self.0) {
            Val::Finite { neg, m, e } => Some((neg, m, e)),
            _ => None,
        }
    }
}

impl F32 {
    /// Positive zero.
    pub const ZERO: F32 = F32(0);

    /// The exact widening to double precision (S-0205 claim 4).
    pub fn to_f64(self) -> F64 {
        let exp = (self.0 >> 23) & 0xff;
        if (1..0xff).contains(&exp) {
            // A normal single: rebias the exponent, widen the fraction.
            let sign = u64::from(self.0 >> 31) << 63;
            let frac = u64::from(self.0 & 0x7f_ffff) << 29;
            return F64(sign | ((u64::from(exp) + 1023 - 127) << 52) | frac);
        }
        match decode(SINGLE, u64::from(self.0)) {
            Val::NaN => F64(DOUBLE.quiet_nan()),
            Val::Inf(n) => F64(DOUBLE.infinity(n)),
            Val::Zero(n) => F64(DOUBLE.zero(n)),
            Val::Finite { neg, m, e } => F64(round_pack(
                DOUBLE,
                neg,
                u128::from(m) << 2,
                e - 2,
                Rounding::TowardZero,
            )),
        }
    }

    /// Whether the value is a NaN.
    pub fn is_nan(self) -> bool {
        decode(SINGLE, u64::from(self.0)) == Val::NaN
    }
}

/// A key that orders non-NaN bit patterns numerically.
fn order_key(f: Fmt, bits: u64) -> i128 {
    let mag = i128::from(bits & !f.sign_bit());
    if bits & f.sign_bit() != 0 { -mag } else { mag }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RZ: Rounding = Rounding::TowardZero;
    const RN: Rounding = Rounding::Nearest;

    // Host floats are allowed in tests: they serve as an RN oracle.
    #[allow(clippy::disallowed_types, clippy::float_arithmetic)]
    fn h(x: f64) -> F64 {
        F64(x.to_bits())
    }

    #[allow(clippy::disallowed_types)]
    fn back(x: F64) -> f64 {
        f64::from_bits(x.0)
    }

    /// A small deterministic generator for test inputs.
    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0
        }
    }

    #[allow(clippy::disallowed_types, clippy::float_arithmetic, clippy::float_cmp)]
    #[test]
    fn nearest_matches_host_on_random_bits() {
        let mut g = Lcg(12345);
        for i in 0..200_000 {
            let mut a = g.next();
            let mut b = g.next();
            if i % 3 == 0 {
                // Keep exponents close so that cancellation is exercised.
                b = (a & 0xFFF0_0000_0000_0000) | (b & 0x000F_FFFF_FFFF_FFFF);
            }
            if i % 7 == 0 {
                a &= 0x800F_FFFF_FFFF_FFFF; // subnormal
            }
            let (x, y) = (f64::from_bits(a), f64::from_bits(b));
            let check = |got: F64, want: f64| {
                if want.is_nan() {
                    assert!(got.is_nan());
                } else {
                    assert_eq!(got.0, want.to_bits(), "{x:e} {y:e}");
                }
            };
            check(F64(a).add(F64(b), RN), x + y);
            check(F64(a).sub(F64(b), RN), x - y);
            check(F64(a).mul(F64(b), RN), x * y);
            check(F64(a).div(F64(b), RN), x / y);
            let n = F64(a).to_f32(RN);
            let want = x as f32;
            if want.is_nan() {
                assert!(n.is_nan());
            } else {
                assert_eq!(n.0, want.to_bits());
            }
        }
    }

    /// Toward zero: the result never exceeds the exact value in magnitude
    /// and is within one ulp of the nearest result.
    #[allow(clippy::disallowed_types, clippy::float_arithmetic)]
    #[test]
    fn toward_zero_is_nearest_or_one_below() {
        let mut g = Lcg(999);
        for _ in 0..100_000 {
            let a = (g.next() >> 2) | 0x3000_0000_0000_0000; // moderate range
            let b = (g.next() >> 2) | 0x3000_0000_0000_0000;
            for (rz, rn) in [
                (F64(a).add(F64(b), RZ), F64(a).add(F64(b), RN)),
                (F64(a).mul(F64(b), RZ), F64(a).mul(F64(b), RN)),
                (F64(a).div(F64(b), RZ), F64(a).div(F64(b), RN)),
            ] {
                let d = rn.0 as i128 - rz.0 as i128;
                assert!(d == 0 || d == 1, "positive values: RZ is RN or one below");
            }
        }
    }

    // spec: S-0205 claim 5 (rounding toward zero vs to nearest)
    #[test]
    fn third_toward_zero_and_nearest() {
        let one = F64::from_i64(1, RZ);
        let three = F64::from_i64(3, RZ);
        let rz = one.div(three, RZ);
        let rn = one.div(three, RN);
        assert_eq!(rz.0, 0x3FD5_5555_5555_5555);
        assert_eq!(rn.0, 0x3FD5_5555_5555_5555);
        let ten = F64::from_i64(10, RZ);
        assert_eq!(one.div(ten, RZ).0, 0x3FB9_9999_9999_9999);
        assert_eq!(one.div(ten, RN).0, 0x3FB9_9999_9999_999A);
    }

    // spec: S-0205 claim 7 (the conversion examples)
    #[allow(clippy::disallowed_types)]
    #[test]
    fn conversion_examples() {
        let c = |x: i64| F64::from_i64(x, RZ).to_i32_common();
        assert_eq!(c(2147483648), -2147483648);
        assert_eq!(c(3000000000), -1294967296);
        assert_eq!(c(-3000000000), 1294967296);
        assert_eq!(c(4294967301), 5);
        assert_eq!(h(2.9).to_i32_common(), 2);
        assert_eq!(h(-2.9).to_i32_common(), -2);
        assert_eq!(h(9.3e18).to_i32_common(), 0);
        assert_eq!(h(f64::INFINITY).to_i32_common(), 0);
        assert_eq!(h(f64::NAN).to_i32_common(), 0);
        assert_eq!(h(3e9).to_i32_32bit(), i32::MIN);
        assert_eq!(h(-2.5).to_i32_32bit(), -2);
    }

    // spec: S-0205 Formulas (adding one half: 0.49999999999999994 gives 0 under RZ, 1 under RN)
    #[allow(clippy::disallowed_types)]
    #[test]
    fn adding_one_half() {
        let x = h(0.49999999999999994);
        let half = h(0.5);
        assert_eq!(x.add(half, RZ).to_i32_common(), 0);
        assert_eq!(x.add(half, RN).to_i32_common(), 1);
        assert_eq!(h(-1.2).add(half, RZ).to_i32_common(), 0);
        assert_eq!(h(-1.7).add(half, RZ).to_i32_common(), -1);
    }

    // spec: S-0205 Formulas (adding one half: an out-of-range rounded sum takes claim 7's common form, wrap and zero result)
    #[allow(clippy::disallowed_types)]
    #[test]
    fn adding_one_half_out_of_range() {
        let half = h(0.5);
        // 2^32 + 5 wraps to 5 (claim 7 wrap from 2^31 up).
        assert_eq!(h(4294967301.0).add(half, RZ).to_i32_common(), 5);
        assert_eq!(h(4294967301.0).add(half, RN).to_i32_common(), 5);
        // 2^63 and above give zero.
        assert_eq!(h(1e19).add(half, RZ).to_i32_common(), 0);
        assert_eq!(h(f64::INFINITY).add(half, RZ).to_i32_common(), 0);
    }

    #[test]
    fn overflow_saturates_toward_zero() {
        let max = F64(0x7FEF_FFFF_FFFF_FFFF);
        assert_eq!(max.add(max, RZ).0, max.0);
        assert_eq!(max.add(max, RN).0, 0x7FF0_0000_0000_0000);
    }

    #[allow(clippy::disallowed_types)]
    #[test]
    fn widen_and_compare() {
        let s = F32(0.3f32.to_bits());
        assert_eq!(back(s.to_f64()), f64::from(0.3f32));
        assert!(h(1.0).lt(h(2.0)));
        assert!(h(-0.0).eq_num(h(0.0)));
        assert!(!h(f64::NAN).le(h(1.0)));
    }

    // Exact conversions: widening every kind of single, and integers on both
    // sides of 2^53, agree with the host.
    #[test]
    #[allow(clippy::disallowed_types, clippy::cast_precision_loss)]
    fn exact_conversions_match_host() {
        let mut x: u64 = 0x9e37_79b9_7f4a_7c15;
        for _ in 0..200_000 {
            x = x
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let b = (x >> 32) as u32;
            let host = f64::from(f32::from_bits(b));
            if !host.is_nan() {
                assert_eq!(F32(b).to_f64().0, host.to_bits(), "{b:#x}");
            }
            let v = x as i64 >> (x % 40);
            assert_eq!(
                F64::from_i64(v, Rounding::Nearest).0,
                (v as f64).to_bits(),
                "{v}"
            );
        }
        for v in [
            0i64,
            1,
            -1,
            (1 << 53) - 1,
            1 << 53,
            -(1 << 53),
            i64::MIN,
            i64::MAX,
        ] {
            assert_eq!(
                F64::from_i64(v, Rounding::Nearest).0,
                (v as f64).to_bits(),
                "{v}"
            );
        }
    }
}
