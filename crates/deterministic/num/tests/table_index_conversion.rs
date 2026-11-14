//! The table index of a large angle: I is a conversion to an integer, so
//! it wraps modulo 2^32 and is 0 from 2^63 (#296).

use glasnost_num::trig::table_index;
use glasnost_num::{F64, Rounding};

// spec: S-0105 F1 (the factor 2607.594482421875, exactly 10680707 / 4096)
fn factor() -> F64 {
    F64::from_parts(false, 10680707, -12, Rounding::TowardZero)
}

// spec: S-0105 F1 (σ from I), with I converted by S-0205 claim 7 (common form)
fn sigma_from_claim_7(x: F64, mode: Rounding) -> u32 {
    let i = i64::from(x.mul(factor(), mode).to_i32_common());
    let half = i / 2;
    let s = half.rem_euclid(8192) + (i.abs() % 2);
    s.min(8191) as u32
}

// spec: S-0105 F1, S-0205 claim 7 (trunc wrapped modulo 2^32 below 2^63)
#[test]
fn odd_i_at_2_pow_31_plus_1_wraps() {
    let rz = Rounding::TowardZero;
    let want: i64 = (1 << 31) + 1;
    // An angle whose product with the factor is a little above 2^31 + 1.
    let x = F64::from_i64(want, rz)
        .div(factor(), Rounding::Nearest)
        .add(F64::from_parts(false, 1, -12, rz), rz);
    let i = x.mul(factor(), rz).trunc_i128();
    assert_eq!(
        i,
        Some(i128::from(want)),
        "the product truncates to 2^31 + 1"
    );
    assert_eq!(x.mul(factor(), rz).to_i32_common(), -2147483647);
    assert_eq!(sigma_from_claim_7(x, rz), 2);
    assert_eq!(table_index(x, rz), 2);
}

// spec: S-0105 F1, S-0205 claim 7 (magnitudes at or above 2^63 convert to 0)
#[test]
fn i_at_or_above_2_pow_63_is_zero() {
    let rz = Rounding::TowardZero;
    let x = F64::from_parts(false, (1u128 << 52) + 1, 11, rz).div(factor(), Rounding::Nearest);
    let i = x.mul(factor(), rz).trunc_i128().expect("finite");
    assert!(i >= 1i128 << 63);
    assert_eq!(x.mul(factor(), rz).to_i32_common(), 0);
    assert_eq!(table_index(x, rz), 0, "I = {i}");
}
