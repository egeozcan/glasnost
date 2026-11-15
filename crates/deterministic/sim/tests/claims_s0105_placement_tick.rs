//! Claim tests for S-0105 r50 claim 6 (*When the turn starts*, Q-2982),
//! sim issues S15 (#81, the simulation half): each object that is not a
//! building keeps the tick of its last placement on the map, saved with the
//! game. A placement sets it, a re-placement sets it again, the end of a
//! drop-pod stand-in at the seize sets it (design), and the jumpjet lift
//! keeps it. Synthetic data of `lift_common`; every name and number is
//! invented. Ledger: `docs/m8-claims/sim-issues.md`, *Batch S15*.

#[path = "lift_common/mod.rs"]
mod lift_common;

use glasnost_sim::lift::{self, StandIn};
use glasnost_sim::objects;
use glasnost_sim::rules::Kind;
use glasnost_sim::save;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, World};
use lift_common::*;

fn placed(w: &World, id: ObjectId) -> i32 {
    w.obj(id).unwrap().placed_tick
}

// spec: S-0105 claim 6 (When the turn starts: a non-building object's turn begins each time it is placed on the map, at the tick number current then)
#[test]
fn issue_81_a_placement_records_its_tick() {
    let mut w = world(|_| {});
    run(&mut w, 9);
    let t = w.tick;
    assert!(t > 0, "set-up: a tick after the first");
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 0, at(0, 0));
    let i = spawn(&mut w, Kind::Infantry, "WALKER", 0, at(3, 0));
    assert_eq!(placed(&w, v), t, "the vehicle");
    assert_eq!(placed(&w, i), t, "the infantry unit");
    run(&mut w, 5);
    assert_eq!(placed(&w, v), t, "nothing else sets it");
}

// spec: S-0105 claim 6 (When the turn starts: any vehicle that leaves a building or transport it was inside is placed again and turns again)
#[test]
fn issue_81_a_placement_again_records_the_new_tick() {
    let mut w = world(|_| {});
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 0, at(0, 0));
    let first = placed(&w, v);
    objects::take_inside(&mut w, v);
    assert!(!w.obj(v).unwrap().on_map, "set-up: off the map");
    run(&mut w, 7);
    let t = w.tick;
    assert_ne!(t, first);
    let p = glasnost_sim::world::Pos::centre(at(2, 2), 0);
    objects::place(&mut w, v, p, 0);
    assert_eq!(placed(&w, v), t, "placed again: the tick then");
}

// spec: S-0105 claim 6 (When the turn starts: a vehicle seized by a magnetic lift stays on the map while it is pulled, held and set down, so none of these restarts the turn)
#[test]
fn issue_81_the_jumpjet_lift_keeps_it() {
    let mut w = world(|_| {});
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
    let t = placed(&w, v);
    run(&mut w, 3);
    beam(&mut w, l, v);
    assert!(lift::seized(&w, v), "set-up: seized");
    run(&mut w, 30);
    assert_eq!(placed(&w, v), t, "the lift keeps it");
}

// spec: S-0105 claim 6 (When the turn starts: our engine ends a drop-pod stand-in at once at the seize, leaving the unit on the map, and that end sets the elevation level and starts its turn at the tick number current then, design); S-0607 claim 23
#[test]
fn issue_81_the_end_of_a_drop_pod_stand_in_records_its_tick() {
    let mut w = world(|r| {
        r.sim.warheads.get_mut("MAG").unwrap().locomotor = Some("drop_pod".into());
    });
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
    let t = placed(&w, v);
    run(&mut w, 3);
    beam(&mut w, l, v);
    assert_eq!(lift::stand_in(&w, v), Some(StandIn::DropPod), "set-up");
    let seized_at = placed(&w, v);
    assert_ne!(seized_at, t, "the stand-in's end restarts it");
    assert!(seized_at > t && seized_at <= w.tick, "at the seize's tick");
}

// spec: S-0105 claim 6 (Saved games: a saved game keeps each object's turn and the tick number)
#[test]
fn issue_81_a_saved_game_keeps_it() {
    let mut w = world(|_| {});
    run(&mut w, 11);
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 0, at(0, 0));
    let t = placed(&w, v);
    let h = save::state_hash(&w);
    let l = save::load(&save::save(&w), w.rules_arc(), testkit::catalogue()).unwrap();
    assert_eq!(save::state_hash(&l), h);
    assert_eq!(placed(&l, v), t);
}
