//! A flat cell's floor height below level 0 (#289).

use glasnost_map::Cell;

fn at(level: i8, raised: bool) -> i32 {
    Cell {
        level,
        ..Cell::BLANK
    }
    .floor_height(raised)
}

// spec: S-0009 claim 9 and Formulas (104ℓ for ℓ ≥ 0, 104ℓ + 1 below; level −1 at −103, level −2 at −207), S-0106 Edge cases (quirks.negative_level_floor_raised, default on)
#[test]
fn floor_height_below_level_zero_is_raised() {
    assert_eq!(at(0, true), 0);
    assert_eq!(at(1, true), 104);
    assert_eq!(at(-1, true), -103);
    assert_eq!(at(-2, true), -207);
}

// spec: S-0009 Formulas, S-0106 Edge cases (quirks.negative_level_floor_raised off: 104ℓ at every level)
#[test]
fn floor_height_with_the_switch_off() {
    assert_eq!(at(1, false), 104);
    assert_eq!(at(-1, false), -104);
    assert_eq!(at(-2, false), -208);
}
