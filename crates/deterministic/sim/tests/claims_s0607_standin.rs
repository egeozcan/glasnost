//! Claim tests for magnetic lift with a stand-in locomotor other than the
//! jumpjet's (S-0607 claims 22 to 24, `quirks.lift_standin_stays_seized`),
//! and the current-locomotor rule of S-0215 claim 11 (revision 68).
//! Synthetic data only.

#[path = "lift_common/mod.rs"]
mod lift_common;

use glasnost_map::CellXY;
use glasnost_rules::sim::Locomotor as RawLocomotor;
use glasnost_sim::combat::check::{self, Answer};
use glasnost_sim::lift::{self, StandIn};
use glasnost_sim::rules::{Kind, Locomotor};
use glasnost_sim::save;
use glasnost_sim::testkit;
use glasnost_sim::world::{Layer, ObjectId, Target, World};
use lift_common::*;

/// A ruleset tweak: the seizing warhead names `loco` (None: no key), the
/// victims carry a gun, CHRONO is a teleporting victim, and the stand-in
/// quirk is `stays`.
fn rules(
    loco: Option<&'static str>,
    stays: bool,
) -> impl FnOnce(&mut glasnost_rules::schema::Ruleset) {
    move |r| {
        r.sim.warheads.get_mut("MAG").unwrap().locomotor = loco.map(str::to_owned);
        r.sim.vehicles.get_mut("VICT").unwrap().primary = Some("Gun".into());
        vehicle(r, "CHRONO", |t| t.locomotor = Some(RawLocomotor::Teleport));
        r.globals.quirks.lift_standin_stays_seized = stays;
    }
}

/// A world with LIFTER (house 0) at (0, 0) and a victim of `name` (house
/// 1) at (6, 0), hit by the beam.
fn scene(loco: Option<&'static str>, stays: bool, name: &str) -> (World, ObjectId, ObjectId) {
    let mut w = world(rules(loco, stays));
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, name, 1, at(6, 0));
    beam(&mut w, l, v);
    (w, l, v)
}

/// The cell distance between two objects (the larger axis).
fn cells_apart(w: &World, a: ObjectId, b: ObjectId) -> i32 {
    let (p, q) = (w.obj(a).unwrap().pos.cell(), w.obj(b).unwrap().pos.cell());
    (p.x - q.x).abs().max((p.y - q.y).abs())
}

/// The sum of the vehicle-present counts of every cell layer near the
/// middle of the map.
fn vehicle_marks(w: &World) -> u32 {
    let mut n = 0;
    for dx in -12..=12 {
        for dy in -12..=12 {
            for l in [Layer::Ground, Layer::Deck] {
                n += w
                    .cells
                    .occ(at(dx, dy), l)
                    .map_or(0, |o| u32::from(o.vehicles));
            }
        }
    }
    n
}

/// Ends the grip by ordering the lifter away (S-0607 claim 12 row 2).
fn let_go(w: &mut World, l: ObjectId) {
    glasnost_sim::econ::assign_destination(w, l, Some(Target::Cell(at(0, 1))));
}

/// Gives the unit move orders for `ticks` ticks; whether it ever left the
/// seized state.
fn ever_restored(w: &mut World, v: ObjectId, ticks: i32) -> bool {
    for _ in 0..ticks / 20 {
        let c = w.obj(v).unwrap().pos.cell();
        order_move(
            w,
            v,
            CellXY {
                x: c.x + 2,
                y: c.y + 2,
            },
        );
        if run_until(w, 20, |w| !lift::seized(w, v)) {
            return true;
        }
    }
    false
}

// spec: S-0607 claim 22 and Import mapping: a locomotor that can stand in temporarily (driving, walking, ship, teleporting; an absent key means teleporting) seizes the target with that stand-in, not the jumpjet; claim 23: its order-level destination is the lifter itself, an object destination
#[test]
fn claim_22_stand_in_seizes() {
    for (loco, k) in [
        (Some("drive"), StandIn::Drive),
        (Some("walk"), StandIn::Walk),
        (Some("ship"), StandIn::Ship),
        (Some("teleport"), StandIn::Teleport),
        (None, StandIn::Teleport),
    ] {
        let (w, l, v) = scene(loco, true, "VICT");
        assert!(lift::seized(&w, v), "{loco:?}: seized");
        assert_eq!(lift::stand_in(&w, v), Some(k), "{loco:?}");
        assert!(lift::grips(&w, l, v), "{loco:?}: gripped");
        assert!(
            w.obj(v).unwrap().jump.is_none(),
            "{loco:?}: no jumpjet locomotor"
        );
        assert_eq!(
            w.obj(v).unwrap().dest,
            Some(Target::Object(l)),
            "{loco:?}: the lifter is its destination"
        );
        assert_eq!(health(&w, v), 100, "{loco:?}: no damage");
    }
}

// spec: S-0607 claim 22 (revision 4: mechanical walking cannot stand in temporarily): the target stops and is put in guard but is not seized
#[test]
fn claim_22_mechanical_walking_cannot_stand_in() {
    let (w, l, v) = scene(Some("mech"), true, "VICT");
    assert!(!lift::seized(&w, v));
    assert!(!lift::grips(&w, l, v));
    assert_eq!(w.obj(v).unwrap().dest, None, "it stops");
}

// spec: S-0607 claim 23: with the driving stand-in the unit drives towards the lifter under S-0207's rules (here a teleporting type, which would otherwise jump), is never lifted, gets no arrival drop, and is set down, not dropped, when its grip ends
#[test]
fn claim_23_drive_stand_in_drives_to_the_lifter() {
    let (mut w, l, v) = scene(Some("drive"), true, "CHRONO");
    assert_eq!(lift::current_locomotor(&w, v), Some(Locomotor::Drive));
    let start = cells_apart(&w, l, v);
    let mut max_h = 0;
    let mut max_step = 0;
    let mut last = w.obj(v).unwrap().pos;
    let ended = run_until(&mut w, 600, |w| {
        let p = w.obj(v).unwrap().pos;
        max_step = max_step.max((p.x - last.x).abs().max((p.y - last.y).abs()));
        last = p;
        max_h = max_h.max(height(w, v));
        assert!(!lift::dropped(w, v), "never dropped");
        !lift::grips(w, l, v)
    });
    assert!(cells_apart(&w, l, v) < start, "it came nearer");
    assert!(max_step < 256, "it drove, never jumped a cell at once");
    assert_eq!(max_h, 0, "never lifted");
    assert!(ended, "the drive gives up beside the occupied lifter cell");
    assert!(lift::seized(&w, v), "the end of the grip does not restore");
    assert_eq!(w.obj(v).unwrap().dest, None, "set down: no destination");
    assert_eq!(health(&w, v), 100, "no impact damage");
}

// spec: S-0607 claim 23: with the teleporting stand-in the unit jumps (S-0207 claim 20) to a landing point found from the cell of the lifter's point, here a drive type, and is never lifted
#[test]
fn claim_23_teleport_stand_in_jumps_to_the_lifter() {
    let (mut w, l, v) = scene(Some("teleport"), true, "VICT");
    assert_eq!(lift::current_locomotor(&w, v), Some(Locomotor::Teleport));
    assert!(
        run_until(&mut w, 10, |w| cells_apart(w, l, v) <= 1),
        "it jumped next to the lifter"
    );
    assert_eq!(height(&w, v), 0, "never lifted");
    assert!(!lift::dropped(&w, v));
    // Revision 7: S-0207 claim 20's jump removes the destination, so claim
    // 12's no-destination row ends the grip and the unit is set down.
    assert!(!lift::grips(&w, l, v), "the jump ends the grip");
    assert!(lift::seized(&w, v));
}

// spec: S-0607 claim 23 (walking, design): our engine keeps the unit where it is; it neither moves nor turns while seized, and the grip lasts
#[test]
fn claim_23_walk_stand_in_stays() {
    let (mut w, l, v) = scene(Some("walk"), true, "VICT");
    let p = w.obj(v).unwrap().pos;
    let f = w.obj(v).unwrap().body.value(w.tick);
    run(&mut w, 60);
    assert_eq!(w.obj(v).unwrap().pos, p);
    assert_eq!(w.obj(v).unwrap().body.value(w.tick), f);
    assert!(lift::grips(&w, l, v));
}

// spec: S-0607 claims 23 and 24 (drop-pod, design): the stand-in ends at once at the seize; the unit stays where it is with its own locomotor, the grip ends with it set down, and it stays seized (the lifter's target: `claim_12_drop_pod_end_clears_the_lifter_target` in claims_s0607_r7.rs)
#[test]
fn claim_23_drop_pod_ends_at_once() {
    let mut w = world(rules(Some("drop_pod"), true));
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
    let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
    let marks = vehicle_marks(&w);
    let p = w.obj(v).unwrap().pos;
    beam(&mut w, l, v);
    assert!(lift::seized(&w, v), "still seized");
    assert_eq!(lift::stand_in(&w, v), Some(StandIn::DropPod));
    assert!(!lift::grips(&w, l, v), "the grip has ended");
    assert_eq!(w.obj(v).unwrap().pos, p, "it stays where it is");
    assert_eq!(w.obj(v).unwrap().dest, None, "set down");
    assert_eq!(
        lift::current_locomotor(&w, v),
        Some(Locomotor::Drive),
        "its own"
    );
    assert!(w.obj(v).unwrap().drive.is_some(), "its own driving state");
    assert_eq!(vehicle_marks(&w), marks, "its own mark is back");
}

// spec: S-0607 claim 23 (claims 9 and 10 apply unchanged): a gripped stand-in unit ignores new destinations and cannot fire
#[test]
fn claim_23_claims_9_and_10_apply() {
    let (mut w, l, v) = scene(Some("walk"), true, "VICT");
    order_move(&mut w, v, at(6, 6));
    run(&mut w, 5);
    assert_eq!(w.obj(v).unwrap().dest, Some(Target::Object(l)));
    assert_eq!(
        check::check(&w, v, Some(Target::Object(l)), 0, false),
        Answer::Illegal
    );
}

// spec: S-0607 claims 12 and 23: a gripped stand-in unit given no destination by any rule: the grip ends at once, the unit is set down and the lifter's target is not cleared
#[test]
fn claim_23_no_destination_ends_the_grip() {
    let (mut w, l, v) = scene(Some("walk"), true, "VICT");
    glasnost_sim::econ::assign_destination(&mut w, v, None);
    assert!(!lift::grips(&w, l, v), "at once");
    assert!(lift::seized(&w, v));
    assert!(!lift::dropped(&w, v), "set down, not dropped");
    assert_eq!(w.obj(l).unwrap().target, Some(Target::Object(v)));
}

/// The seized scene with its grip ended by the lifter's order.
fn let_go_scene(loco: &'static str, stays: bool) -> (World, ObjectId, ObjectId) {
    let (mut w, l, v) = scene(Some(loco), stays, "VICT");
    run(&mut w, 30);
    let_go(&mut w, l);
    assert!(!lift::grips(&w, l, v), "{loco}: the grip ends");
    (w, l, v)
}

// spec: S-0607 claim 24 and quirks.lift_standin_stays_seized (on): with the driving, walking, ship or teleporting stand-in the unit is never restored: it ignores every order, never fires and cannot be seized again
#[test]
fn quirk_lift_standin_stays_seized_on() {
    for loco in ["drive", "walk", "ship", "teleport"] {
        let (mut w, _, v) = let_go_scene(loco, true);
        assert!(!ever_restored(&mut w, v, 200), "{loco}: never restored");
        assert_ne!(
            w.obj(v).unwrap().dest.map(|t| matches!(t, Target::Cell(_))),
            Some(true),
            "{loco}: no order accepted"
        );
        assert_eq!(
            check::check(&w, v, Some(Target::Cell(at(3, 3))), 0, false),
            Answer::Illegal,
            "{loco}: it never fires"
        );
        let l2 = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, -6));
        run(&mut w, 25);
        beam(&mut w, l2, v);
        assert!(!lift::grips(&w, l2, v), "{loco}: not seized again");
    }
}

// spec: S-0607 claim 24 and quirks.lift_standin_stays_seized (on): with the drop-pod stand-in the unit has its own locomotor back but stays seized: orders are ignored indefinitely
#[test]
fn quirk_lift_standin_stays_seized_on_drop_pod() {
    let (mut w, _, v) = scene(Some("drop_pod"), true, "VICT");
    assert!(!ever_restored(&mut w, v, 200));
    assert_eq!(
        check::check(&w, v, Some(Target::Cell(at(3, 3))), 0, false),
        Answer::Illegal
    );
}

// spec: S-0607 claim 24 and quirks.lift_standin_stays_seized (off): claim 20 applies to every stand-in with its own moving test, at any restoration test (revision 14: the end of each of its turns, with no order given); restoration clears seized and the unit then moves on orders
#[test]
fn quirk_lift_standin_stays_seized_off() {
    for loco in ["drive", "walk", "ship", "teleport"] {
        let (mut w, l, v) = let_go_scene(loco, false);
        // The lifter leaves, so it cannot seize the unit again once restored.
        glasnost_sim::objects::remove(&mut w, l);
        assert!(
            run_until(&mut w, 200, |w| !lift::seized(w, v)),
            "{loco}: restored with no order"
        );
        assert_eq!(lift::stand_in(&w, v), None);
        assert_eq!(
            lift::current_locomotor(&w, v),
            Some(Locomotor::Drive),
            "{loco}: its own"
        );
        assert!(w.obj(v).unwrap().lift.own.is_none());
        let c = w.obj(v).unwrap().pos.cell();
        let to = CellXY { x: c.x + 3, y: c.y };
        order_move(&mut w, v, to);
        assert!(
            run_until(&mut w, 300, |w| w.obj(v).unwrap().pos.cell() == to),
            "{loco}: it moves again"
        );
    }
    let (mut w, l, v) = scene(Some("drop_pod"), false, "VICT");
    glasnost_sim::objects::remove(&mut w, l);
    assert!(ever_restored(&mut w, v, 200), "drop_pod: restored");
}

// spec: S-0209 claim 1 and S-0607 claims 5, 23 and 24 (revision 5, occupancy): the seize gives up the unit's mark and installing the stand-in marks no cell; a driving stand-in marks cells as its track enters them, a teleporting one by its own landing reservation (S-0207 claim 20); a unit that moved under the driving stand-in, or jumped under the teleporting one, and is then removed leaves no vehicle-present mark behind
#[test]
fn claim_23_stand_in_marks_are_released() {
    for loco in ["drive", "teleport", "walk"] {
        let mut w = world(rules(Some(loco), true));
        let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(0, 0));
        let base = vehicle_marks(&w);
        let v = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 0));
        assert_eq!(vehicle_marks(&w), base + 1);
        let c = w.obj(v).unwrap().pos.cell();
        beam(&mut w, l, v);
        assert_eq!(
            w.cells.occ(c, Layer::Ground).map_or(0, |o| o.vehicles),
            0,
            "{loco}: its own cell holds no mark after the seize"
        );
        let mut marked = vehicle_marks(&w) > base;
        for _ in 0..60 {
            run(&mut w, 1);
            marked |= vehicle_marks(&w) > base;
        }
        assert_eq!(
            marked,
            loco != "walk",
            "{loco}: only the stand-in's own movement marks a cell (the driving track, the teleporting landing reservation)"
        );
        glasnost_sim::objects::remove(&mut w, v);
        run(&mut w, 1);
        assert_eq!(vehicle_marks(&w), base, "{loco}: none left");
    }
}

// spec: S-0607 claim 23 (revision 7, occupancy): a unit that never moves under its stand-in stands with no vehicle-present mark, but it stays listed in its cell and the listed occupants decide first: while it has the lifter as its destination an allied vehicle meets moving blocker (S-0209 claim 5 row VP22), so the missing mark does not let it in
#[test]
fn claim_23_unmoved_stand_in_holds_no_mark() {
    let (mut w, l, v) = scene(Some("walk"), true, "VICT");
    run(&mut w, 5);
    assert!(lift::grips(&w, l, v));
    let c = w.obj(v).unwrap().pos.cell();
    assert_eq!(
        w.cells.occ(c, Layer::Ground).map_or(0, |o| o.vehicles),
        0,
        "no vehicle-present mark"
    );
    let other = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(6, 2));
    let reference = glasnost_sim::occupancy::reference_level(&w, other);
    assert_eq!(
        glasnost_sim::occupancy::vehicle_outcome(&w, other, c, reference),
        glasnost_sim::occupancy::Outcome::MovingBlocker,
        "the listed seized unit still decides the entry"
    );
}

// spec: S-0215 claim 11 (revision 68): readiness asks the vehicle's current locomotor; while a temporary replacement is installed its report applies: the jumpjet while seized with it, the stand-in otherwise, the own locomotor once a drop-pod stand-in has ended
#[test]
fn s0215_claim_11_current_locomotor() {
    let (w, _, v) = scene(Some("jumpjet"), true, "VICT");
    assert_eq!(lift::current_locomotor(&w, v), Some(Locomotor::Jump));
    let (w, _, v) = scene(Some("ship"), true, "VICT");
    assert_eq!(lift::current_locomotor(&w, v), Some(Locomotor::Ship));
    let (w, _, v) = scene(Some("teleport"), true, "VICT");
    assert_eq!(lift::current_locomotor(&w, v), Some(Locomotor::Teleport));
    let (w, _, v) = scene(Some("drop_pod"), true, "CHRONO");
    assert_eq!(lift::current_locomotor(&w, v), Some(Locomotor::Teleport));
}

// spec: S-0607 claims 22 to 24 (the stand-in, the kept own movement state and the seized state survive a save/load round trip with the same state hash)
#[test]
fn stand_in_save_round_trip() {
    let (mut w, _, v) = scene(Some("drive"), true, "CHRONO");
    run(&mut w, 10);
    assert!(w.obj(v).unwrap().lift.own.is_some());
    let h = save::state_hash(&w);
    let l = save::load(&save::save(&w), w.rules_arc(), testkit::catalogue()).unwrap();
    assert_eq!(save::state_hash(&l), h);
    assert_eq!(l.obj(v).unwrap().lift, w.obj(v).unwrap().lift);
}
