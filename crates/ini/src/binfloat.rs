//! Binary floating-point values computed with exact integer arithmetic.
//!
//! S-0003 fixes how decimal text becomes a single or double (claims 6 and
//! 13) and how a percentage is multiplied by the double nearest to 0.01 and
//! rounded toward zero (claims 7 and 17). The importer reproduces those
//! results bit for bit without using floating-point types: a value is kept
//! as its IEEE 754 bit pattern, and every conversion is done on big
//! integers. Nothing here depends on the host's floating-point unit.

use num_bigint::BigUint;

/// A binary floating-point format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Format {
    /// 24-bit significand (S-0003 Terms, *single*).
    Single,
    /// 53-bit significand (S-0003 Terms, *double*).
    Double,
}

impl Format {
    /// Significand bits *P*, hidden bit included.
    pub fn precision(self) -> u32 {
        // nocite: IEEE 754 binary32 and binary64 layouts, not game behaviour
        match self {
            Self::Single => 24,
            Self::Double => 53,
        }
    }

    /// Exponent of the smallest normal value (2^emin).
    fn emin(self) -> i64 {
        // nocite: IEEE 754 layout
        match self {
            Self::Single => -126,
            Self::Double => -1022,
        }
    }

    /// Exponent of the largest finite value's leading bit.
    fn emax(self) -> i64 {
        // nocite: IEEE 754 layout
        match self {
            Self::Single => 127,
            Self::Double => 1023,
        }
    }

    fn total_bits(self) -> u32 {
        // nocite: IEEE 754 layout
        match self {
            Self::Single => 32,
            Self::Double => 64,
        }
    }

    fn exp_mask(self) -> u64 {
        // nocite: IEEE 754 layout (all-ones exponent field)
        match self {
            Self::Single => 255,
            Self::Double => 2047,
        }
    }
}

/// An IEEE 754 value held as its bit pattern.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Binary {
    /// Its format.
    pub format: Format,
    /// Bit pattern in the low 32 (single) or 64 (double) bits.
    pub bits: u64,
}

/// A finite or infinite value, decoded: `(-1)^neg × m × 2^e`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Decoded {
    Finite { neg: bool, m: u64, e: i64 },
    Infinite { neg: bool },
    NaN,
}

impl Binary {
    /// Positive or negative zero.
    pub fn zero(format: Format, neg: bool) -> Self {
        Self::encode_sign(format, neg, 0)
    }

    /// One (1.0).
    pub fn one(format: Format) -> Self {
        Self::round_toward_zero(format, false, 1, 0)
    }

    /// Positive or negative infinity.
    pub fn infinity(format: Format, neg: bool) -> Self {
        let p = u64::from(format.precision());
        Self::encode_sign(format, neg, format.exp_mask() << (p - 1))
    }

    fn sign_bit(format: Format) -> u64 {
        1 << (format.total_bits() - 1)
    }

    fn encode_sign(format: Format, neg: bool, magnitude: u64) -> Self {
        let bits = if neg {
            magnitude | Self::sign_bit(format)
        } else {
            magnitude
        };
        Self { format, bits }
    }

    /// Whether the sign bit is set.
    pub fn is_negative(&self) -> bool {
        self.bits & Self::sign_bit(self.format) != 0
    }

    fn decode(&self) -> Decoded {
        let p = u64::from(self.format.precision());
        let neg = self.is_negative();
        let frac = self.bits & ((1 << (p - 1)) - 1);
        let exp_field = (self.bits >> (p - 1)) & self.format.exp_mask();
        let e_shift = self.format.emin() - (p as i64 - 1);
        if exp_field == self.format.exp_mask() {
            return if frac == 0 {
                Decoded::Infinite { neg }
            } else {
                Decoded::NaN
            };
        }
        if exp_field == 0 {
            // Zero or subnormal: frac × 2^(emin - (P - 1)).
            return Decoded::Finite {
                neg,
                m: frac,
                e: e_shift,
            };
        }
        Decoded::Finite {
            neg,
            m: frac | (1 << (p - 1)),
            e: exp_field as i64 - 1 + e_shift,
        }
    }

    /// `(-1)^neg × m × 2^e` rounded toward zero to `format` (RZ of S-0003
    /// Formulas): excess bits are dropped, values below the smallest
    /// normal become subnormal by truncation, and values above the largest
    /// finite value become the largest finite value.
    pub fn round_toward_zero(format: Format, neg: bool, m: u128, e: i64) -> Self {
        if m == 0 {
            return Self::zero(format, neg);
        }
        let p = i64::from(format.precision());
        let bitlen = i64::from(128 - m.leading_zeros());
        // Leading bit's exponent.
        let top = e + bitlen - 1;
        let sub_unit = format.emin() - (p - 1);
        if top < format.emin() {
            // Subnormal: truncate to a multiple of 2^sub_unit.
            let shift = sub_unit - e;
            let t = shr_u128(m, shift);
            return Self::encode_sign(format, neg, t as u64);
        }
        if top > format.emax() {
            let max = ((format.exp_mask() - 1) << (p - 1)) | ((1u64 << (p - 1)) - 1);
            return Self::encode_sign(format, neg, max);
        }
        let shift = bitlen - p;
        let t = shr_u128(m, shift) as u64;
        let biased = (top - format.emin() + 1) as u64;
        let frac = t & ((1u64 << (p - 1)) - 1);
        Self::encode_sign(format, neg, (biased << (p - 1)) | frac)
    }

    /// The product of `self` and `other`, rounded toward zero to `out`
    /// (exact product, one rounding). Infinity times a non-zero finite
    /// value is infinity.
    pub fn mul_rz(&self, other: &Binary, out: Format) -> Self {
        match (self.decode(), other.decode()) {
            (Decoded::NaN, _) | (_, Decoded::NaN) => Self {
                format: out,
                bits: Self::infinity(out, false).bits | 1,
            },
            (Decoded::Infinite { neg: a }, Decoded::Infinite { neg: b }) => {
                Self::infinity(out, a != b)
            }
            (Decoded::Infinite { neg: a }, Decoded::Finite { neg: b, m, .. })
            | (Decoded::Finite { neg: b, m, .. }, Decoded::Infinite { neg: a }) => {
                if m == 0 {
                    Self {
                        format: out,
                        bits: Self::infinity(out, false).bits | 1,
                    }
                } else {
                    Self::infinity(out, a != b)
                }
            }
            (
                Decoded::Finite {
                    neg: na,
                    m: ma,
                    e: ea,
                },
                Decoded::Finite {
                    neg: nb,
                    m: mb,
                    e: eb,
                },
            ) => Self::round_toward_zero(out, na != nb, u128::from(ma) * u128::from(mb), ea + eb),
        }
    }

    /// `self` converted to `out`, rounded toward zero.
    pub fn convert_rz(&self, out: Format) -> Self {
        match self.decode() {
            Decoded::Finite { neg, m, e } => Self::round_toward_zero(out, neg, u128::from(m), e),
            Decoded::Infinite { neg } => Self::infinity(out, neg),
            Decoded::NaN => Self {
                format: out,
                bits: Self::infinity(out, false).bits | 1,
            },
        }
    }

    /// The exact value as decimal text: `-` for a negative sign (also of
    /// zero), the integer part, and a fraction without trailing zeros.
    /// Infinities are `inf` and `-inf`; NaN is `nan`.
    pub fn to_exact_decimal(&self) -> String {
        let (neg, m, e) = match self.decode() {
            Decoded::Finite { neg, m, e } => (neg, m, e),
            Decoded::Infinite { neg } => return if neg { "-inf".into() } else { "inf".into() },
            Decoded::NaN => return "nan".into(),
        };
        let sign = if neg { "-" } else { "" };
        if e >= 0 {
            return format!("{sign}{}", BigUint::from(m) << (e as u64));
        }
        // m × 2^e = m × 5^k / 10^k with k = -e.
        let k = (-e) as usize;
        let digits = (BigUint::from(m) * BigUint::from(5u32).pow(k as u32)).to_string();
        let digits = format!("{digits:0>width$}", width = k + 1);
        let (int, frac) = digits.split_at(digits.len() - k);
        let frac = frac.trim_end_matches('0');
        if frac.is_empty() {
            format!("{sign}{int}")
        } else {
            format!("{sign}{int}.{frac}")
        }
    }
}

fn shr_u128(m: u128, shift: i64) -> u128 {
    if shift <= 0 {
        // Callers keep the result within range: a left shift only happens
        // for values that fit the target precision.
        m << ((-shift).min(127) as u32)
    } else if shift >= 128 {
        0
    } else {
        m >> shift as u32
    }
}

/// `c` of S-0003: the double nearest to 1/100 (round to nearest, ties to
/// even), computed exactly.
pub fn percent_factor() -> Binary {
    // spec: S-0003 Constants (percent_factor = RN53(1/100))
    let hundred = 100u128;
    // 1/100 = t × 2^-59 with 2^52 <= t < 2^53.
    let e = -59i64;
    let num = 1u128 << 59;
    let mut t = num / hundred;
    let rem = num % hundred;
    if 2 * rem > hundred || (2 * rem == hundred && t & 1 == 1) {
        t += 1;
    }
    Binary::round_toward_zero(Format::Double, false, t, e)
}

/// The digits and exponent of a decimal number as written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecimalText {
    /// A `-` sign was read.
    pub neg: bool,
    /// All digits, integer and fraction parts together (ASCII `0`-`9`).
    pub digits: Vec<u8>,
    /// Power of ten applied to `digits` read as an integer.
    pub exp10: i64,
}

/// Outcome of converting decimal text to binary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Conversion {
    /// The value.
    pub value: Binary,
    /// The importer warning test of S-0003 Formulas fired: the value lies
    /// in one of claim 6's three uncovered cases (or close to one).
    pub uncovered: bool,
}

/// Magnitudes far outside both formats' ranges are settled without big
/// integers: `10^LIMIT` overflows a double and `10^-LIMIT` is below its
/// smallest subnormal.
const DECIMAL_EXPONENT_LIMIT: i64 = 400; // nocite: engineering bound beyond both formats' ranges

/// Converts decimal text by the rule of S-0003 Formulas (claim 6), at the
/// precision of `format` (claim 13 uses the double format).
pub fn decimal_to_binary(text: &DecimalText, format: Format) -> Conversion {
    let first = text.digits.iter().position(|&d| d != b'0');
    let Some(first) = first else {
        // x = 0: a zero of the prefix's sign.
        return Conversion {
            value: Binary::zero(format, text.neg),
            uncovered: false,
        };
    };
    let digits = &text.digits[first..];
    let magnitude = digits.len() as i64 + text.exp10;
    if magnitude > DECIMAL_EXPONENT_LIMIT {
        return Conversion {
            value: Binary::infinity(format, text.neg),
            uncovered: false,
        };
    }
    if magnitude < -DECIMAL_EXPONENT_LIMIT {
        // Below the smallest subnormal: truncation gives zero (claim 6, case 1).
        return Conversion {
            value: Binary::zero(format, text.neg),
            uncovered: true,
        };
    }
    let n = BigUint::parse_bytes(digits, 10).unwrap_or_default();
    let ten = BigUint::from(10u32);
    let (num, den) = if text.exp10 >= 0 {
        (n * ten.pow(text.exp10 as u32), BigUint::from(1u32))
    } else {
        (n, ten.pow((-text.exp10) as u32))
    };
    let p = i64::from(format.precision());
    // spec: S-0003 Formulas (W: x is a whole number and x < 2^64)
    let whole = (&num % &den) == BigUint::from(0u32);
    let w = whole && num < (&den << 64u32);

    // Find e with 2^(P-1) <= x / 2^e < 2^P.
    let mut e = num.bits() as i64 - den.bits() as i64 - (p - 1);
    let (mut t, mut rem, mut d) = scaled_quotient(&num, &den, e);
    let lo = BigUint::from(1u32) << ((p - 1) as u32);
    let hi = BigUint::from(1u32) << (p as u32);
    while t >= hi {
        e += 1;
        (t, rem, d) = scaled_quotient(&num, &den, e);
    }
    while t < lo {
        e -= 1;
        (t, rem, d) = scaled_quotient(&num, &den, e);
    }

    // spec: S-0003 Formulas (importer warning test)
    let below_normal = e + (p - 1) < format.emin();
    let margin = BigUint::from(1u32) << ((60 - p) as u32);
    let two_rem = &rem << 1u32;
    let four_rem = &rem << 2u32;
    let three_d = &d * 3u32;
    let near_half = abs_diff(&two_rem, &d) * &margin <= (&d << 1u32);
    let near_three_quarters = abs_diff(&four_rem, &three_d) * &margin <= (&d << 2u32);
    let uncovered = below_normal || (!w && (near_half || near_three_quarters));

    if below_normal {
        // Our engine: round toward zero to a multiple of the smallest subnormal.
        let sub_unit = format.emin() - (p - 1);
        let (q, _, _) = scaled_quotient(&num, &den, sub_unit);
        let q = u128::try_from(q).unwrap_or(0);
        return Conversion {
            value: Binary::round_toward_zero(format, text.neg, q, sub_unit),
            uncovered,
        };
    }
    // spec: S-0003 Formulas (f = t if r <= 1/2, or r = 3/4 and W; else t + 1)
    let r_at_most_half = two_rem <= d;
    let r_three_quarters = four_rem == three_d;
    let keep = r_at_most_half || (r_three_quarters && w);
    let mut t = u128::try_from(t).unwrap_or(0);
    if !keep {
        t += 1;
    }
    let top = e + bitlen(t) - 1;
    let value = if top > format.emax() {
        Binary::infinity(format, text.neg)
    } else {
        Binary::round_toward_zero(format, text.neg, t, e)
    };
    Conversion { value, uncovered }
}

fn bitlen(t: u128) -> i64 {
    i64::from(128 - t.leading_zeros())
}

fn abs_diff(a: &BigUint, b: &BigUint) -> BigUint {
    if a >= b { a - b } else { b - a }
}

/// floor(num / den / 2^e), the remainder, and the divisor the remainder
/// is relative to.
fn scaled_quotient(num: &BigUint, den: &BigUint, e: i64) -> (BigUint, BigUint, BigUint) {
    if e >= 0 {
        let d = den << (e as u64);
        (num / &d, num % &d, d)
    } else {
        let n = num << ((-e) as u64);
        (&n / den, &n % den, den.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dec(s: &str) -> DecimalText {
        let (neg, s) = match s.strip_prefix('-') {
            Some(r) => (true, r),
            None => (false, s),
        };
        let (mant, exp) = match s.split_once('e') {
            Some((m, x)) => (m, x.parse::<i64>().unwrap()),
            None => (s, 0),
        };
        let (int, frac) = mant.split_once('.').unwrap_or((mant, ""));
        DecimalText {
            neg,
            digits: format!("{int}{frac}").into_bytes(),
            exp10: exp - frac.len() as i64,
        }
    }

    fn single(s: &str) -> String {
        decimal_to_binary(&dec(s), Format::Single)
            .value
            .to_exact_decimal()
    }

    /// spec: S-0003 claim 6 (examples)
    #[test]
    fn claim_six_examples() {
        assert_eq!(single("0.1"), "0.100000001490116119384765625");
        assert_eq!(single("16777217"), "16777216");
        assert_eq!(single("16777219"), "16777218");
        assert_eq!(single("67108870"), "67108864");
        assert_eq!(single("67108870.0"), "67108864");
        assert_eq!(single("6710887e1"), "67108864");
        assert_eq!(single("74115990"), "74115984");
        assert_eq!(single("791e7"), "7909999616");
        assert_eq!(single("-0"), "-0");
        assert_eq!(single("1e39"), "inf");
        assert_eq!(single("-1e39"), "-inf");
        // 2^63 + 2^39 + 1: discarded part just over one half, so rounded up.
        let x = (1u128 << 63) + (1 << 39) + 1;
        assert_eq!(
            single(&x.to_string()),
            ((1u128 << 63) + (1 << 40)).to_string()
        );
    }

    /// spec: S-0003 Formulas (the importer warning test)
    #[test]
    fn warning_test() {
        let w = |s: &str| decimal_to_binary(&dec(s), Format::Single).uncovered;
        assert!(!w("0.1"));
        assert!(!w("0.35"));
        // Whole numbers below 2^64 never warn.
        assert!(!w("16777217"));
        assert!(!w("67108870"));
        // A non-whole value whose discarded part is exactly one half.
        assert!(w("8388608.5"));
        // Below the smallest normal single.
        assert!(w("1e-40"));
        assert_eq!(
            single("1e-40"),
            Binary::round_toward_zero(Format::Single, false, 71362, -149).to_exact_decimal()
        );
    }

    /// spec: S-0003 claim 6, our engine's rule for a three-quarter remainder
    /// of a value that is not a whole number below 2^64
    #[test]
    fn three_quarters_not_whole() {
        // 8388608.75: t = 8388608, r = 3/4, not whole: rounds up.
        assert_eq!(single("8388608.75"), "8388609");
        // Whole and below 2^64, r = 3/4: dropped (67108870 = t*4 + 6? see claim 6).
        assert_eq!(single("33554435"), "33554432");
    }

    /// spec: S-0003 claim 7 (percentages, RZ53 of y × c)
    #[test]
    fn percentages() {
        let c = percent_factor();
        assert_eq!(
            c.to_exact_decimal(),
            "0.01000000000000000020816681711721685132943093776702880859375"
        );
        let pct = |s: &str| {
            let y = decimal_to_binary(&dec(s), Format::Single).value;
            y.mul_rz(&c, Format::Double)
        };
        assert_eq!(pct("50").to_exact_decimal(), "0.5");
        assert_eq!(pct("100").to_exact_decimal(), "1");
        // 10%: one unit below 0.1 in double.
        let ten = pct("10");
        assert_eq!(ten.bits, 0x3FB9_9999_9999_9999);
        // Narrowed to single, toward zero: the single just below 0.1.
        assert_eq!(ten.convert_rz(Format::Single).bits, 0x3DCC_CCCC);
        assert_eq!(
            decimal_to_binary(&dec("0.1"), Format::Single).value.bits,
            0x3DCC_CCCD
        );
    }

    /// spec: S-0003 claim 13 (double precision, 0.125 exact)
    #[test]
    fn double_precision() {
        let d = decimal_to_binary(&dec("0.125"), Format::Double).value;
        assert_eq!(d.to_exact_decimal(), "0.125");
        assert_eq!(
            decimal_to_binary(&dec("0.1"), Format::Double).value.bits,
            0x3FB9_9999_9999_999A
        );
    }

    #[test]
    fn rz_edges() {
        let one = Binary::one(Format::Single);
        assert_eq!(one.bits, 0x3F80_0000);
        assert_eq!(Binary::one(Format::Double).to_exact_decimal(), "1");
        let tiny = Binary::round_toward_zero(Format::Single, false, 1, -149);
        assert_eq!(tiny.bits, 1);
        let big = Binary::round_toward_zero(Format::Single, false, 1, 200);
        assert_eq!(big.bits, 0x7F7F_FFFF);
    }
}
