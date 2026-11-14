//! Decimal text to binary values, exactly as S-0003 reads decimals.
//!
//! Our rules format writes every decimal of the user's data as a quoted
//! string (PLAN.md 4.6): `-?digits[.digits][e[+-]digits][%]`. The value it
//! stands for is the one S-0003 gives the same text: a single-precision value
//! by the conversion rule of S-0003 Formulas (claim 6), and with a `%` sign
//! that single times the double nearest to 1/100, rounded as S-0003 claim 7
//! and S-0205 claim 5 say. The importer (`glasnost-ini`) computes the same
//! bits, which a cross-check test of the importer confirms.
//!
//! The literals in the conversion are IEEE 754 layout values and the
//! conversion rule's own fractions (one half, three quarters), cited below.

use num_bigint::BigUint;

use crate::soft::{F32, F64, Rounding};

/// The precision a field keeps (S-0003 claims 6 and 7).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Field {
    /// A decimal field: a single, or with `%` the double product.
    #[default]
    Decimal,
    /// A single-precision field: a `%` product is narrowed (S-0003 claim 7).
    Single,
}

/// Why decimal text was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecimalError(pub String);

impl std::fmt::Display for DecimalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a decimal: {:?}", self.0)
    }
}

impl std::error::Error for DecimalError {}

/// The digits and power of ten of a decimal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecimalText {
    /// A `-` sign was read.
    pub neg: bool,
    /// All digits, integer and fraction parts together (ASCII).
    pub digits: Vec<u8>,
    /// Power of ten applied to `digits` read as an integer.
    pub exp10: i64,
    /// A `%` sign follows.
    pub percent: bool,
}

/// Parses our format's decimal text (whole string, no spaces).
// nocite: lexical grammar of our own rules format (PLAN.md 4.6); digit arithmetic, not game behaviour
pub fn parse_text(s: &str) -> Result<DecimalText, DecimalError> {
    let err = || DecimalError(s.to_owned());
    let b = s.as_bytes();
    let (neg, mut i) = match b.first() {
        Some(b'-') => (true, 1),
        _ => (false, 0),
    };
    let mut digits = Vec::new();
    let int_start = i;
    while i < b.len() && b[i].is_ascii_digit() {
        digits.push(b[i]);
        i += 1;
    }
    let mut frac_len = 0i64;
    if b.get(i) == Some(&b'.') {
        i += 1;
        while i < b.len() && b[i].is_ascii_digit() {
            digits.push(b[i]);
            frac_len += 1;
            i += 1;
        }
    }
    if digits.is_empty() || i == int_start && frac_len == 0 {
        return Err(err());
    }
    let mut exp10 = 0i64;
    if matches!(b.get(i), Some(b'e' | b'E')) {
        i += 1;
        let eneg = match b.get(i) {
            Some(b'-') => {
                i += 1;
                true
            }
            Some(b'+') => {
                i += 1;
                false
            }
            _ => false,
        };
        let start = i;
        let mut e = 0i64;
        while i < b.len() && b[i].is_ascii_digit() {
            // nocite: saturation bound of our own parser, far beyond both formats
            e = (e * 10 + i64::from(b[i] - b'0')).min(1_000_000);
            i += 1;
        }
        if i == start {
            return Err(err());
        }
        exp10 = if eneg { -e } else { e };
    }
    let percent = b.get(i) == Some(&b'%');
    if percent {
        i += 1;
    }
    if i != b.len() {
        return Err(err());
    }
    Ok(DecimalText {
        neg,
        digits,
        exp10: exp10 - frac_len,
        percent,
    })
}

/// The double nearest to 1/100 (S-0003 Constants, the percent factor).
// spec: S-0003 Constants (percent factor: the double nearest to 1/100)
pub fn percent_factor() -> F64 {
    // spec: S-0003 Constants (percent factor = RN53(1/100))
    let hundred = 100u128;
    // nocite: 1/100 = t × 2^-59 with 2^52 <= t < 2^53 (binary64 layout)
    let e = -59i32;
    let num = 1u128 << 59;
    let mut t = num / hundred;
    let rem = num % hundred;
    if 2 * rem > hundred || (2 * rem == hundred && t & 1 == 1) {
        t += 1;
    }
    F64::from_parts(false, t, e, Rounding::TowardZero)
}

/// The value of decimal text: a single by S-0003 claim 6, widened exactly;
/// with `%`, the product with the percent factor rounded by `mode` (S-0003
/// claim 7, S-0205 claim 5), narrowed again for a single-precision field.
pub fn value(text: &DecimalText, field: Field, mode: Rounding) -> F64 {
    let single = to_single(text);
    if !text.percent {
        return single.to_f64();
    }
    let product = single.to_f64().mul(percent_factor(), mode);
    match field {
        Field::Decimal => product,
        Field::Single => product.to_f32(mode).to_f64(),
    }
}

/// Parses and converts in one step.
pub fn parse(s: &str, field: Field, mode: Rounding) -> Result<F64, DecimalError> {
    Ok(value(&parse_text(s)?, field, mode))
}

/// Magnitudes far outside the single range are settled without big
/// integers.
// nocite: engineering bound beyond both formats' ranges
const DECIMAL_EXPONENT_LIMIT: i64 = 400;

/// Decimal text to a single by the rule of S-0003 Formulas (claim 6).
// spec: S-0003 Formulas (claim 6 conversion: binary32 layout, W test, f = t or t + 1)
pub fn to_single(text: &DecimalText) -> F32 {
    // nocite: binary32 layout (precision, exponent range)
    let (p, emin, emax): (i64, i64, i64) = (24, -126, 127);
    let signed = |m: u128, e: i64| -> F32 {
        F64::from_parts(text.neg, m, e as i32, Rounding::TowardZero).to_f32(Rounding::TowardZero)
    };
    let zero = F32(if text.neg { 1 << 31 } else { 0 });
    let first = text.digits.iter().position(|&d| d != b'0');
    let Some(first) = first else {
        return zero;
    };
    let digits = &text.digits[first..];
    let magnitude = digits.len() as i64 + text.exp10;
    // nocite: binary32 infinity bit pattern
    let inf = F32(if text.neg { 0xFF80_0000 } else { 0x7F80_0000 });
    if magnitude > DECIMAL_EXPONENT_LIMIT {
        return inf;
    }
    if magnitude < -DECIMAL_EXPONENT_LIMIT {
        return zero;
    }
    let n = BigUint::parse_bytes(digits, 10).unwrap_or_default();
    let ten = BigUint::from(10u32);
    let (num, den) = if text.exp10 >= 0 {
        (n * ten.pow(text.exp10 as u32), BigUint::from(1u32))
    } else {
        (n, ten.pow((-text.exp10) as u32))
    };
    // spec: S-0003 Formulas (W: x is a whole number and x < 2^64)
    let whole = (&num % &den) == BigUint::from(0u32);
    let w = whole && num < (&den << 64u32);
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
    if e + (p - 1) < emin {
        // Our engine: round toward zero to a multiple of the smallest subnormal.
        let sub_unit = emin - (p - 1);
        let (q, _, _) = scaled_quotient(&num, &den, sub_unit);
        let q = u128::try_from(q).unwrap_or(0);
        return signed(q, sub_unit);
    }
    // spec: S-0003 Formulas (f = t if r <= 1/2, or r = 3/4 and W; else t + 1)
    let two_rem = &rem << 1u32;
    let four_rem = &rem << 2u32;
    let three_d = &d * 3u32;
    let keep = two_rem <= d || (four_rem == three_d && w);
    let mut t = u128::try_from(t).unwrap_or(0);
    if !keep {
        t += 1;
    }
    let top = e + i64::from(128 - t.leading_zeros()) - 1;
    if top > emax {
        return inf;
    }
    signed(t, e)
}

/// floor(num / den / 2^e), the remainder, and the divisor the remainder is
/// relative to.
// nocite: exact rational arithmetic helper
fn scaled_quotient(num: &BigUint, den: &BigUint, e: i64) -> (BigUint, BigUint, BigUint) {
    if e >= 0 {
        let d = den << (e as u64);
        (num / &d, num % &d, d)
    } else {
        let n = num << ((-e) as u64);
        (&n / den, &n % den, den.clone())
    }
}

/// The double nearest to the value of decimal text (ties to even), for
/// engine constants that a spec gives as "the double nearest to" a decimal.
/// Text that is not a decimal gives 0; a `%` mark is not applied (rules
/// data that must not carry one is checked with [`exact_double`], #293).
// nocite: exact rational rounding helper (binary64 precision and guard bits)
pub fn nearest_double(text: &str) -> F64 {
    parse_text(text).map_or(F64::ZERO, |t| nearest_of(&t))
}

/// The double nearest to the value of plain decimal text (ties to even); an
/// error for text that is not a decimal or that carries `%`, which this
/// reading has no factor for (#293: a versus item of our rules format is the
/// exact decimal of its double).
// nocite: exact rational rounding helper (binary64 precision and guard bits)
pub fn exact_double(text: &str) -> Result<F64, DecimalError> {
    let t = parse_text(text)?;
    if t.percent {
        return Err(DecimalError(text.to_owned()));
    }
    Ok(nearest_of(&t))
}

/// RN₅₃ of the value of parsed decimal text (sign kept; `%` not applied).
// nocite: exact rational rounding helper (binary64 precision and guard bits)
fn nearest_of(t: &DecimalText) -> F64 {
    let n = BigUint::parse_bytes(&t.digits, 10).unwrap_or_default();
    let ten = BigUint::from(10u32);
    let (num, den) = if t.exp10 >= 0 {
        (n * ten.pow(t.exp10 as u32), BigUint::from(1u32))
    } else {
        (n, ten.pow((-t.exp10) as u32))
    };
    if num == BigUint::from(0u32) {
        return F64::ZERO;
    }
    let shift: i64 = 120 - (num.bits() as i64 - den.bits() as i64);
    let (q, r) = if shift >= 0 {
        let n2 = &num << (shift as u64);
        (&n2 / &den, &n2 % &den)
    } else {
        let d2 = &den << ((-shift) as u64);
        (&num / &d2, &num % &d2)
    };
    let q = u128::try_from(q).unwrap_or(0) | u128::from(r != BigUint::from(0u32));
    F64::from_parts(t.neg, q, -(shift as i32), Rounding::Nearest)
}

/// The exact decimal expansion of a double (for messages and tests).
// nocite: exact decimal printing for messages and tests
pub fn exact_decimal(x: F64) -> String {
    let Some((neg, m, e)) = x.parts() else {
        return if x.is_sign_negative() {
            "-0".into()
        } else {
            "0".into()
        };
    };
    let sign = if neg { "-" } else { "" };
    if e >= 0 {
        return format!("{sign}{}", BigUint::from(m) << (e as u64));
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    const RZ: Rounding = Rounding::TowardZero;
    const RN: Rounding = Rounding::Nearest;

    fn single(s: &str) -> String {
        exact_decimal(parse(s, Field::Decimal, RZ).unwrap())
    }

    // spec: S-0003 claim 6 (examples)
    #[test]
    fn claim_six_examples() {
        assert_eq!(single("0.1"), "0.100000001490116119384765625");
        assert_eq!(single("16777217"), "16777216");
        assert_eq!(single("16777219"), "16777218");
        assert_eq!(single("67108870"), "67108864");
        assert_eq!(single("6710887e1"), "67108864");
        assert_eq!(single("791e7"), "7909999616");
        assert_eq!(single("8388608.75"), "8388609");
        assert_eq!(single("33554435"), "33554432");
    }

    // spec: S-0205 claim 9 (0.06 is the single just below 0.06; 0.3 the single just above)
    #[test]
    fn short_decimals() {
        assert_eq!(single("0.06"), "0.0599999986588954925537109375");
        assert_eq!(single("0.3"), "0.300000011920928955078125");
    }

    // spec: S-0205 claim 9 (`30%` × 900 gives 269 under RZ and 270 under RN; `0.3` gives 270)
    #[test]
    fn percent_products_n7() {
        let ticks = |s: &str, mode: Rounding| {
            let v = parse(s, Field::Decimal, mode).unwrap();
            v.mul(F64::from_i64(900, mode), mode).to_i32_common()
        };
        assert_eq!(ticks("30%", RZ), 269);
        assert_eq!(ticks("30%", RN), 270);
        assert_eq!(ticks("0.3", RZ), 270);
        assert_eq!(ticks("0.3", RN), 270);
        assert_eq!(ticks("0.5009765625", RZ), 450);
    }

    // nocite: our own rules format (#293): the exact-double reading has no `%` factor
    #[test]
    fn exact_double_refuses_a_percent() {
        assert_eq!(exact_decimal(exact_double("0.5").unwrap()), "0.5");
        assert_eq!(exact_decimal(exact_double("-0.25").unwrap()), "-0.25");
        assert!(exact_double("50%").is_err());
        assert!(exact_double("x").is_err());
        assert_eq!(nearest_double("50%"), nearest_double("50"));
        assert_eq!(nearest_double("x"), F64::ZERO);
    }

    #[test]
    fn grammar() {
        assert!(parse_text("1.").is_ok());
        assert!(parse_text(".5").is_ok());
        assert!(parse_text("-2.5e-3%").unwrap().percent);
        assert!(parse_text("").is_err());
        assert!(parse_text(".").is_err());
        assert!(parse_text("1x").is_err());
        assert!(parse_text("1e").is_err());
    }
}
