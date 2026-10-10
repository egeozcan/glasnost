//! Integer rules of S-0205: width, wrapping and division.

/// The integer quotient truncated toward zero (S-0205 claim 2), with the
/// departure of claim 3: a divisor of 0 gives 0, and −2147483648 ÷ −1 gives
/// −2147483648.
pub fn div(a: i32, b: i32) -> i32 {
    // spec: S-0205 claim 3 (departure: a ÷ 0 = 0; the overflowing quotient wraps)
    if b == 0 { 0 } else { a.wrapping_div(b) }
}

/// The remainder with the sign of the dividend (S-0205 claim 2), with the
/// departure of claim 3: a rem 0 = a, and −2147483648 rem −1 = 0.
pub fn rem(a: i32, b: i32) -> i32 {
    // spec: S-0205 claim 3 (departure: a rem 0 = a)
    if b == 0 { a } else { a.wrapping_rem(b) }
}

/// Halving toward −∞ (S-0205 claim 2, written ⌊a / 2⌋).
pub fn half_floor(a: i32) -> i32 {
    a >> 1
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: S-0205 claims 2 and 3 (truncating division, remainder sign, departures)
    #[test]
    fn division() {
        assert_eq!(div(-7, 2), -3);
        assert_eq!(rem(-7, 2), -1);
        assert_eq!(div(5, 0), 0);
        assert_eq!(rem(5, 0), 5);
        assert_eq!(div(i32::MIN, -1), i32::MIN);
        assert_eq!(rem(i32::MIN, -1), 0);
        assert_eq!(half_floor(-7), -4);
        assert_eq!(half_floor(7), 3);
    }
}
