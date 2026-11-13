//! Claim tests for S-0502 claim 23 (r24, Q-3043) and S-0213 claim 21: the
//! own-cell assignment that a jumpjet-type infantry unit ignores has no
//! effect at all. Its order-level destination keeps its preceding value,
//! empty or not, its movement point, commitment and flight state are
//! unchanged, it takes no draw and requests no get-up, while the order's
//! other effects stand (scenario J32). Synthetic rules (`jump_common`: JJ is
//! a jumpjet infantry type); every name and number is invented.

mod jump_common;

use glasnost_rules::sim::SequenceEntry;
use glasnost_sim::infantry::Seq;
use glasnost_sim::jump::Phase;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use jump_common::*;

/// Issues a behaviour order of house 0 to `id` and applies it.
fn order(w: &mut World, id: ObjectId, b: Behaviour, c: glasnost_map::CellXY) {
    let (target, dest) = if b == Behaviour::AreaGuard {
        (Some(Target::Cell(c)), None)
    } else {
        (None, Some(Target::Cell(c)))
    };
    assert!(orders::issue(
        w,
        HouseId(0),
        Order::Behaviour {
            actor: id,
            behaviour: b,
            attack_move: false,
            target,
            dest,
        },
    ));
    step(w);
}

/// A JJ ordered to a far cell, cruising two cells or more from the start.
fn cruising() -> (World, ObjectId, glasnost_map::CellXY) {
    let mut w = game();
    let u = put(&mut w, "JJ", Kind::Infantry, 0, 0, 0);
    let far = at(8, 0);
    go(&mut w, u, far);
    assert!(until(&mut w, 400, |w| {
        phase(w, u) == Phase::Cruising && pos(w, u).cell().x >= at(2, 0).x
    }));
    (w, u, far)
}

// spec: S-0502 claim 23 (r24, Q-3043: the ignored assignment has no effect at all: the order-level destination keeps its preceding value, its movement point, commitment, flight state and landing reservation are unchanged, and it takes no draw; it does not end a retry delay or restart a blockage delay) and S-0213 claim 21 (r106)
#[test]
fn j32_a_an_own_cell_assignment_in_flight_changes_nothing() {
    let (mut w, u, far) = cruising();
    let over = pos(&w, u).cell();
    assert_ne!(over, far);
    let before = w.obj(u).unwrap().clone();
    let taken = w.rng.taken();
    go(&mut w, u, over);
    assert_eq!(
        w.obj(u).unwrap(),
        &before,
        "the ignored assignment changes nothing on the unit"
    );
    assert_eq!(w.rng.taken(), taken, "no draw");
    assert_eq!(w.obj(u).unwrap().dest, Some(Target::Cell(far)));
}

// spec: S-0502 claim 23 (r24, Q-3043: an own-cell move order in flight leaves the order-level destination the far cell, and the unit flies on to it); S-0213 claim 21 (r106: the order's other effects stand)
#[test]
fn j32_a_an_own_cell_move_order_in_flight_flies_on_to_the_far_cell() {
    let (mut w, u, far) = cruising();
    let over = pos(&w, u).cell();
    let point = w.obj(u).unwrap().jump.as_ref().unwrap().dest;
    order(&mut w, u, Behaviour::Move, over);
    let o = w.obj(u).unwrap();
    assert_eq!(o.dest, Some(Target::Cell(far)), "the destination stays");
    assert_eq!(o.jump.as_ref().unwrap().dest, point, "the point stays");
    assert_eq!(o.jump.as_ref().unwrap().order, Some(Target::Cell(far)));
    assert!(
        until(&mut w, 600, |w| pos(w, u).cell() == far),
        "it flies on to the far cell"
    );
}

// spec: S-0502 claim 23 (r24, Q-3043: an area-guard order on the cell it is over: the same unchanged values and no draw, while the guard spot changes); S-0213 claim 21 (r106: the guard-spot effect is not prevented)
#[test]
fn j32_b_an_own_cell_area_guard_order_sets_only_the_guard_spot() {
    let (mut w, u, far) = cruising();
    // The order is applied in the next tick; take the cell then.
    let over = pos(&w, u).cell();
    let point = w.obj(u).unwrap().jump.as_ref().unwrap().dest;
    order(&mut w, u, Behaviour::AreaGuard, over);
    let o = w.obj(u).unwrap();
    assert_eq!(o.guard_spot, Some(Target::Cell(over)), "the guard spot");
    assert_eq!(o.dest, Some(Target::Cell(far)), "the destination stays");
    assert_eq!(o.jump.as_ref().unwrap().dest, point, "the point stays");
}

/// A prone, crawling JJ of the human house 0, grounded with no destination.
fn prone_jumper() -> (World, ObjectId) {
    let mut r = ruleset();
    let jj = r.sim.infantry.get_mut("JJ").unwrap();
    jj.crawls = Some(true);
    jj.fraidycat = Some(false);
    jj.cyborg = Some(false);
    // Image frames for the kinds involved (S-0107 claim 6 row 1).
    for k in ["stand", "walk", "go_prone", "get_up", "prone", "crawl"] {
        jj.sequences.insert(
            k.into(),
            SequenceEntry {
                start: 0,
                count: 4,
                multiplier: 0,
                direction: None,
            },
        );
    }
    let mut w = testkit::game_with(&r, SIZE, 7);
    testkit::run(&mut w, 4);
    let u = put(&mut w, "JJ", Kind::Infantry, 0, 0, 0);
    w.obj_mut(u).unwrap().infantry.as_mut().unwrap().prone = true;
    w.obj_mut(u).unwrap().infantry.as_mut().unwrap().seq = Some(Seq::Prone);
    assert!(w.houses[0].human);
    (w, u)
}

// spec: S-0502 claim 23 (r24, Q-3043: the ignored assignment is not an assignment made for any other rule: it requests no get-up, S-0107 claim 15); S-0213 claim 21 (r106: an ordinary own-cell order does not promise take-off, an assignment or get-up)
#[test]
fn j32_c_own_cell_orders_to_a_prone_unit_request_no_get_up() {
    let (mut w, u) = prone_jumper();
    let here = pos(&w, u).cell();
    for _ in 0..2 {
        order(&mut w, u, Behaviour::Move, here);
        let o = w.obj(u).unwrap();
        assert_eq!(o.dest, None, "the order-level destination stays empty");
        let s = o.infantry.as_ref().unwrap();
        assert!(s.prone, "still prone");
        assert_ne!(s.seq, Some(Seq::GetUp), "no get-up");
        for _ in 0..3 {
            step(&mut w);
        }
    }
    assert_eq!(phase(&w, u), Phase::Grounded, "it does not ascend");
    assert_eq!(height(&w, u), 0);
}
