//! The direction facing of a real displacement with
//! `quirks.table_direction` off: the exact angle (#294).

use glasnost_num::geom::{direction, direction_real_b};
use glasnost_num::{F64, Rounding};

const RZ: Rounding = Rounding::TowardZero;

fn num(n: i64) -> F64 {
    F64::from_i64(n, RZ)
}

// spec: S-0204 Edge cases (quirks.table_direction off: φ = round(arctan2(X, Y) × 65536 / (2π)) mod 65536; axes 0, 16384, 32768, 49152; diagonals 8192, 24576, 40960, 57344), S-0306 Formulas (F11 and F14 directions are S-0204 F3's, under its switch)
#[test]
fn off_gives_the_exact_axes_and_diagonals() {
    let cases = [
        ((0, 1), 0),
        ((1, 1), 8192),
        ((1, 0), 16384),
        ((1, -1), 24576),
        ((0, -1), 32768),
        ((-1, -1), 40960),
        ((-1, 0), 49152),
        ((-1, 1), 57344),
    ];
    for ((x, y), want) in cases {
        assert_eq!(
            direction_real_b(num(x), num(y), false, RZ),
            want,
            "X = {x}, Y = {y}"
        );
        assert_eq!(
            direction_real_b(num(7 * x), num(7 * y), false, Rounding::Nearest),
            want,
            "X = {}, Y = {}",
            7 * x,
            7 * y
        );
    }
}

// spec: S-0204 Edge cases (the exact direction), S-0306 Formulas (both arguments narrowed to single precision)
#[test]
fn off_matches_the_integer_exact_direction() {
    for (dx, dy) in [(3, 5), (-200, 77), (1000, -1), (-12345, -6789), (256, 1)] {
        // Y grows towards map north: Y = −dy.
        assert_eq!(
            direction_real_b(num(dx), num(-dy), false, RZ),
            direction(dx, dy, false),
            "dx = {dx}, dy = {dy}"
        );
    }
}

// spec: S-0204 claim 8 (the table diagonal is about 123 units off) and F3 (default on: the table path)
#[test]
fn on_keeps_the_table_path() {
    assert_eq!(direction_real_b(num(1), num(1), true, RZ), 8315);
    assert_eq!(direction_real_b(num(1), num(0), true, RZ), 16383);
}

// spec: S-0306 Formulas (revision 90: the direction of a zero displacement, both arguments zero of either sign after the narrowing to single precision, is 16383 with quirks.table_direction on and 16384 with it off)
#[test]
fn zero_displacement_faces_map_east() {
    let zero = F64::ZERO;
    let minus_zero = F64::ZERO.negated();
    // The smallest positive double: zero once narrowed to single precision.
    let tiny = F64(1);
    let cases = [
        (zero, zero),
        (minus_zero, zero),
        (zero, minus_zero),
        (minus_zero, minus_zero),
        (tiny, tiny.negated()),
        (tiny.negated(), zero),
    ];
    for (x, y) in cases {
        for mode in [RZ, Rounding::Nearest] {
            assert_eq!(
                direction_real_b(x, y, false, mode),
                16384,
                "off: X = {x:?}, Y = {y:?}"
            );
            assert_eq!(
                direction_real_b(x, y, true, mode),
                16383,
                "on: X = {x:?}, Y = {y:?}"
            );
        }
    }
}
