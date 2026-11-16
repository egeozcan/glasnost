//! S-0301 claim 4 (S20's seam of #371): every clear of a firer's own
//! target sets its burst counter to 0, not only the clears that reset it
//! by hand; a change from one target to another keeps it. Synthetic rules:
//! the test kit's combat rules (CRUSHER, BUGGY). No game data.

use glasnost_map::CellXY;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{ObjectId, Target, World};
use glasnost_sim::{objects, spawn, testkit};

const SIZE: i32 = 24;

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

/// A game with a CRUSHER of house 0 at (0, 0) whose target is a BUGGY of
/// house 1 at (4, 0), in mid-burst (burst counter 2). Returns the world,
/// the tank and the buggy.
fn mid_burst() -> (World, ObjectId, ObjectId) {
    let mut w = testkit::game_with(&testkit::combat_ruleset(), SIZE, 7);
    testkit::run(&mut w, 2);
    let t = testkit::spawn(&mut w, Kind::Vehicle, "CRUSHER", 0, at(0, 0), 0);
    let b = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, at(4, 0), 0);
    let o = w.obj_mut(t).unwrap();
    o.target = Some(Target::Object(b));
    o.combat.burst = 2;
    (w, t, b)
}

// spec: S-0301 claim 4 (when the firer's target is cleared, its burst counter is set to 0) and table, row "the firer's target is cleared" (burst counter 0)
#[test]
fn clearing_the_own_target_resets_the_burst_counter() {
    let (mut w, t, _) = mid_burst();
    spawn::clear_own_target(w.obj_mut(t).unwrap());
    assert_eq!(w.obj(t).unwrap().target, None);
    assert_eq!(w.obj(t).unwrap().combat.burst, 0);
}

// spec: S-0301 claim 4 (an explicit clear by an order is a clear too)
#[test]
fn an_explicit_clear_resets_the_burst_counter() {
    let (mut w, t, _) = mid_burst();
    spawn::explicit_clear_own_target(w.obj_mut(t).unwrap());
    assert_eq!(w.obj(t).unwrap().combat.burst, 0);
}

// spec: S-0301 claim 4 (a target redirected to none, S-0309 claim 24, leaves the firer with no target: a clear)
#[test]
fn a_redirect_to_none_resets_the_burst_counter() {
    let (mut w, t, _) = mid_burst();
    spawn::redirected_to_none(w.obj_mut(t).unwrap());
    assert_eq!(w.obj(t).unwrap().combat.burst, 0);
}

// spec: S-0301 claim 4 (a change from one target to another keeps the burst counter)
#[test]
fn a_change_to_another_target_keeps_the_burst_counter() {
    let (mut w, t, _) = mid_burst();
    let other = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, at(4, 3), 0);
    spawn::set_own_target(w.obj_mut(t).unwrap(), Some(Target::Object(other)));
    assert_eq!(w.obj(t).unwrap().combat.burst, 2);
}

// spec: S-0301 claim 4 (our engine's taking of an object inside clears its target: one of the clears that did not reset the counter by hand, `objects::take_inside`)
#[test]
fn going_inside_resets_the_burst_counter() {
    let (mut w, t, _) = mid_burst();
    objects::take_inside(&mut w, t);
    assert_eq!(w.obj(t).unwrap().target, None);
    assert_eq!(w.obj(t).unwrap().combat.burst, 0);
}

// spec: S-0301 claim 4 with S-0506's Slot table (a non-missile slot object that flies back to its spawner loses its target: the one direct clear outside the helpers, `spawn::aircraft_return`)
#[test]
fn a_slot_object_returning_to_its_spawner_resets_the_burst_counter() {
    let (mut w, t, b) = mid_burst();
    spawn::aircraft_return(&mut w, t, b);
    assert_eq!(w.obj(t).unwrap().target, None);
    assert_eq!(w.obj(t).unwrap().combat.burst, 0);
}
