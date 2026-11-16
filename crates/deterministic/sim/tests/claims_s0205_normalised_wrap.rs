//! The game-speed normalisation at extreme base delays (#153): S-0107 F3's
//! product 8b is an integer result, and S-0205 claim 1 says an integer
//! result outside the signed 32-bit range wraps; the quotient is truncated
//! toward zero (S-0205 claim 2). A debug build must give the wrapped result,
//! as a release build does, not panic. Every number is our own.

use glasnost_sim::infantry::normalised;

/// spec: S-0107 F3 (N(b, s) = ⌊8b / (s + 1)⌋ for b ≥ 5); S-0205 claims 1
/// and 2 (8b wraps; the quotient is truncated toward zero)
#[test]
fn the_product_of_a_huge_base_delay_wraps() {
    // 8 × 2²⁹ = 2³² wraps to 0.
    assert_eq!(normalised(1 << 29, 0), 0);
    // 8 × (2²⁸ + 1) = 2³¹ + 8 wraps to −2147483640.
    assert_eq!(normalised((1 << 28) + 1, 0), -2_147_483_640);
    // −2147483640 / 7 = −306783377.1…, truncated toward zero.
    assert_eq!(normalised((1 << 28) + 1, 6), -306_783_377);
    // 8 × 2147483647 wraps to −8; −8 / 8 = −1.
    assert_eq!(normalised(i32::MAX, 7), -1);
}

/// spec: S-0107 F3 (the largest base delay whose product fits: no wrap)
#[test]
fn a_base_delay_whose_product_fits_is_exact() {
    let b = i32::MAX / 8;
    assert_eq!(i64::from(normalised(b, 0)), 8 * i64::from(b));
    assert_eq!(i64::from(normalised(b, 3)), 8 * i64::from(b) / 4);
    // spec: S-0107 F3 (the in-range values are as before)
    assert_eq!(normalised(5, 0), 40);
    assert_eq!(normalised(900, 7), 900);
}
