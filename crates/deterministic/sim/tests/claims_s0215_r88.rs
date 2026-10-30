//! S-0215 revision 88 claim 37 part c (M8 step A3 intake 52a; answers
//! Q-2113 with S-0607 revision 18): the old exception for a target that is
//! the object holding the actor with a magnetic beam is gone. A seized unit
//! whose target is its own lifter takes the approach step as for any other
//! target: part d's keep-or-drop table applies to it, and a destination the
//! step gives it has no effect (S-0607 claim 9). The gripping lifter's own
//! exception stays (`claims_s0607_r18.rs`). Synthetic data only (the lift
//! fixture of `lift_common`); no stock value. Ledger:
//! `docs/m8-claims/intake52a-S-0210-S-0215-S-0216.md`.
//!
//! The step is called directly (`approach::step`, as the hunt and area
//! guard processing call it, S-0215 claims 26 and 27): the hunt's own
//! map-wide search would otherwise change the unarmed victim's target
//! first. The tick that follows is the real one.

#[path = "lift_common/mod.rs"]
mod lift_common;

use glasnost_sim::lift;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{ObjectId, Target, World};
use lift_common::*;

/// A lifter of house 0 at (0, 0) grips VICT of house 1 at (6, 0), whose
/// type cannot approach its target (S-0215 claim 37 (d)'s *may approach*
/// false) and whose owner is human-controlled or not as `human` says; the
/// victim then hunts with its lifter as its target.
fn hunting_its_lifter(human: bool) -> (World, ObjectId, ObjectId) {
    let mut w = world(|r| {
        r.sim.vehicles.get_mut("VICT").unwrap().can_approach_target = Some(false);
    });
    w.houses[1].human = human;
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
    beam(&mut w, l, v);
    assert!(lift::grips(&w, l, v), "the lifter grips the victim");
    let t = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    let o = w.obj_mut(v).unwrap();
    o.target = Some(Target::Object(l));
    o.beh.force(Behaviour::Hunt, t, stale);
    (w, l, v)
}

// spec: S-0215 claim 37 (c) (revision 88: a seized unit whose target is its lifter takes the step as for any target) and (d) (hunt, may not approach, human-controlled owner: the target is dropped, target and destination cleared at once) with S-0607 claim 9 (a rule that gives a gripped unit no destination ends its grip without a drop; still seized) and claim 12 (that row leaves the lifter's target as it was)
// Interim rule under request Q-2151 (does a cleared destination end the grip here?): as built, yes.
// Flip this test when Q-2151 is answered otherwise.
#[test]
fn a_seized_unit_hunting_its_lifter_drops_it_and_the_grip_ends() {
    let (mut w, l, v) = hunting_its_lifter(true);
    assert!(w.obj(v).unwrap().dest.is_some(), "the grip's destination");
    glasnost_sim::approach::step(&mut w, v);
    let o = w.obj(v).unwrap();
    assert_eq!(o.target, None, "part d dropped the target");
    assert_eq!(o.dest, None, "and cleared the destination");
    run(&mut w, 1);
    assert!(!lift::grips(&w, l, v), "the grip ended");
    assert!(lift::seized(&w, v), "still seized");
    assert_eq!(
        w.obj(l).unwrap().target,
        Some(Target::Object(v)),
        "the lifter's target is not cleared"
    );
}

// spec: S-0215 claim 37 (c) (revision 88) and (d) (hunt with a computer-controlled owner keeps the target), (e) (the grip's destination, the lifter's cell, lies farther from the lifter than m × r′ of the unarmed unit: cleared) and (f) with S-0307 claim 17 (the search drops the target and gives a nearby cell); S-0607 claim 9 (that destination has no effect: the grip's destination comes back and the grip holds)
#[test]
fn a_computer_seized_unit_hunting_its_lifter_is_given_a_cell_that_has_no_effect() {
    let (mut w, l, v) = hunting_its_lifter(false);
    let dest = w.obj(v).unwrap().dest;
    assert!(dest.is_some(), "the grip's destination");
    glasnost_sim::approach::step(&mut w, v);
    let o = w.obj(v).unwrap();
    assert_eq!(o.target, None, "the search dropped the target");
    assert!(
        matches!(o.dest, Some(Target::Cell(_))) && o.dest != dest,
        "the step gave a nearby cell"
    );
    run(&mut w, 1);
    assert!(lift::grips(&w, l, v), "the grip holds");
    assert_eq!(w.obj(v).unwrap().dest, dest, "the grip's destination");
}
