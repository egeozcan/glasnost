//! A flat cell's floor height below level 0 (#289).

use glasnost_map::Cell;

fn at(level: i8) -> i32 {
    Cell {
        level,
        ..Cell::BLANK
    }
    .floor_height()
}

// spec: S-0009 claim 9 and Formulas (104ℓ for ℓ ≥ 0, 104ℓ + 1 below; level −1 at −103, level −2 at −207), S-0106 Edge cases (quirks.negative_level_floor_raised, default on)
#[test]
fn floor_height_below_level_zero_is_raised() {
    assert_eq!(at(0), 0);
    assert_eq!(at(1), 104);
    assert_eq!(at(-1), -103);
    assert_eq!(at(-2), -207);
}
