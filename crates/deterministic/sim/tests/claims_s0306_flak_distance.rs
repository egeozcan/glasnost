//! S-0306 F17 (revision 93, answers Q-3129), sim issues S21 (#317): the
//! flak distance *D*_f of claims 4 and 9. Synthetic data only. Ledger:
//! `docs/m8-claims/sim-issues.md`, *Batch S21*.

use glasnost_sim::combat::projectile::flak_distance;
use glasnost_sim::testkit;
use glasnost_sim::world::{Pos, World};

fn world(table_root: bool) -> World {
    let mut r = testkit::combat_ruleset();
    r.globals.quirks.table_root_distance = table_root;
    testkit::game_with(&r, 40, 7)
}

// spec: S-0306 F17 (the example: differences (-6582, 7963, 206) give n_f = 106774529 and D_f = 10333)
#[test]
fn issue_317_the_spec_example() {
    let w = world(true);
    let x = Pos::new(10_000, 2_000, 300);
    let y = Pos::new(10_000 - 6582, 2_000 + 7963, 300 + 206);
    assert_eq!(flak_distance(&w, x, y), 10333);
}

// spec: S-0306 F17 (each difference is a 32-bit integer difference that wraps modulo 2^32 outside the signed range)
#[test]
fn issue_317_the_differences_wrap() {
    let w = world(true);
    let x = Pos::new(i32::MIN, 0, 0);
    let y = Pos::new(i32::MAX, 0, 0);
    assert_eq!(
        flak_distance(&w, x, y),
        1,
        "Δx = i32::MAX − i32::MIN wraps to −1"
    );
    let x = Pos::new(0, i32::MAX, 0);
    let y = Pos::new(0, i32::MIN, 0);
    assert_eq!(flak_distance(&w, x, y), 1, "Δy wraps to +1");
}

// spec: S-0306 F17 (u = RZ24(Δx): a horizontal difference is narrowed to single precision before it is squared; with quirks.table_root_distance off the root is the exact one, so the narrowing shows: 2^24 + 1 narrows to 2^24)
#[test]
fn issue_317_a_horizontal_difference_is_narrowed_to_single() {
    let w = world(false);
    let x = Pos::new(0, 0, 0);
    let y = Pos::new((1 << 24) + 1, 0, 0);
    assert_eq!(flak_distance(&w, x, y), 1 << 24);
    // The vertical difference enters double precision exactly.
    let y = Pos::new(0, 0, (1 << 24) + 1);
    assert_eq!(flak_distance(&w, x, y), (1 << 24) + 1);
}
