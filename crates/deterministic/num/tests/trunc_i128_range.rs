//! F64::trunc_i128 is exact for every finite double below 2^127 in
//! magnitude, as its doc says (#297).

use glasnost_num::trig::acos_t;
use glasnost_num::{F64, Rounding};

const RZ: Rounding = Rounding::TowardZero;

// nocite: our own conversion helper's documented range (#297), not game behaviour
#[test]
fn exact_below_2_pow_127() {
    let m: u128 = (1 << 53) - 1;
    let v = F64::from_parts(false, m, 70, RZ);
    assert_eq!(v.trunc_i128(), Some((m as i128) << 70));
    let v = F64::from_parts(false, 1, 123, RZ);
    assert_eq!(v.trunc_i128(), Some(1i128 << 123));
    let v = F64::from_parts(true, m, 74, RZ);
    assert_eq!(v.trunc_i128(), Some(-((m as i128) << 74)));
    let v = F64::from_parts(false, 1, 127, RZ);
    assert_eq!(v.trunc_i128(), None);
}

// spec: S-0306 F8 (k = trunc(2048 (x + 1)); an index outside the table clamped, a value that is not a number gives index 0)
#[test]
fn acos_t_clamps_a_finite_argument_below_2_pow_127_to_the_top() {
    let at_one = acos_t(F64::from_i64(1, RZ), RZ);
    // (2^111 + 1) × 2048 = 2^122 and (2^113 + 1) × 2048 = 2^124: both finite.
    assert_eq!(acos_t(F64::from_parts(false, 1, 111, RZ), RZ), at_one);
    assert_eq!(acos_t(F64::from_parts(false, 1, 113, RZ), RZ), at_one);
}
