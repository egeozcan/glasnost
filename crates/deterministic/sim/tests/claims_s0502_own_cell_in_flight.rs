//! Claim test for S-0502 claim 23 and S-0213 claim 21 (#190): a jumpjet
//! infantry unit in flight ignores a destination assignment whose cell is
//! its current cell, also after it has flown on out of that cell. Synthetic
//! rules (`jump_common`: JJ is a jumpjet infantry type).

mod jump_common;

use glasnost_sim::jump::Phase;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::Target;
use jump_common::*;

// spec: S-0502 claim 23 (outside an ion storm a jumpjet-type infantry unit ignores a nonempty destination assignment whose cell equals its current cell) and S-0213 claim 21 (the filter suppresses the assignment); Q-3043 (interim: the order-level destination keeps the filtered one, and the locomotor does not take it later)
#[test]
fn a_filtered_own_cell_assignment_stays_ignored_after_the_unit_leaves_the_cell() {
    let mut w = game();
    let u = put(&mut w, "JJ", Kind::Infantry, 0, 0, 0);
    let far = at(8, 0);
    go(&mut w, u, far);
    // Fly until cruising and two cells or more from the start.
    assert!(until(&mut w, 400, |w| {
        phase(w, u) == Phase::Cruising && pos(w, u).cell().x >= at(2, 0).x
    }));
    let over = pos(&w, u).cell();
    assert_ne!(over, far);
    let before = w.obj(u).unwrap().jump.as_ref().unwrap().clone();
    let taken = w.rng.taken();
    // The own-cell order: filtered at the assignment.
    go(&mut w, u, over);
    let now = w.obj(u).unwrap().jump.as_ref().unwrap().clone();
    assert_eq!(now.order, before.order, "the locomotor takes nothing at the assignment");
    assert_eq!(now.dest, before.dest, "the movement point is kept");
    assert_eq!(w.rng.taken(), taken, "no draw at the assignment");
    assert_eq!(now.order, Some(Target::Cell(far)));
    // Fly on until the unit is over another cell, then a few ticks more.
    assert!(until(&mut w, 200, |w| pos(w, u).cell() != over));
    for _ in 0..10 {
        step(&mut w);
    }
    let later = w.obj(u).unwrap().jump.as_ref().unwrap().clone();
    assert_eq!(
        later.order,
        Some(Target::Cell(far)),
        "the ignored assignment stays ignored"
    );
    assert_eq!(later.dest, before.dest, "the movement point is kept");
}
