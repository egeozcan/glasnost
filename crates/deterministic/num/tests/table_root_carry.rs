//! The table square root at the top of the 64-bit range when the input is
//! rounded to nearest (#295).

use glasnost_num::Rounding;
use glasnost_num::geom::{root_floor, table_root, table_root_parts};

// spec: S-0204 F1 (r(n) for n rounded to single precision by the mode of S-0205; n′ = 2^64 gives R = 2^23, z = 32)
#[test]
fn nearest_rounding_carries_to_two_to_the_64() {
    // 2^64 − 2^39: the smallest n whose 24-bit rounding to nearest is 2^64.
    let at = (1u64 << 39).wrapping_neg();
    // One below rounds down to (2^24 − 1) × 2^40.
    let (r, z) = table_root_parts(at - 1, Rounding::Nearest).expect("n > 0");
    assert!((1 << 23..1 << 24).contains(&r), "R = {r}");
    assert_eq!(z, 31);
    // Toward zero never carries.
    let (r, z) = table_root_parts(u64::MAX, Rounding::TowardZero).expect("n > 0");
    assert!((1 << 23..1 << 24).contains(&r), "R = {r}");
    assert_eq!(z, 31);
    for n in [at, u64::MAX] {
        assert_eq!(
            table_root_parts(n, Rounding::Nearest),
            Some((1 << 23, 32)),
            "n = {n}"
        );
        assert_eq!(root_floor(n, true, Rounding::Nearest), 1 << 32, "n = {n}");
        assert_eq!(
            table_root(n, Rounding::Nearest).to_f64(),
            glasnost_num::F64::from_parts(false, 1, 32, Rounding::TowardZero),
            "n = {n}"
        );
    }
}
