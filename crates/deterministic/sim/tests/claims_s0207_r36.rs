//! Claim tests of S-0207 revisions 35 and 36 (with Q-1130 answered): the
//! Q-1130 table of claim 27 (pending point suppression meets a teleporting
//! destination), claim 20 (a)'s single exception and claim 20 (h)'s empty
//! assignment while a jump is pending; with S-0215 revision 54 claim 2,
//! S-0514 revision 7 claims 16 and 17, S-0214 revision 38 (Terms) and
//! S-0502 revision 11 claim 27. The fixture is the jumpjet factory ruleset
//! (TELEQ teleports: it lacks the teleport-capable flag, which would make
//! it drive under S-0207 claim 28). Every name and number is invented; no game data.

mod jump_common;

use glasnost_map::CellXY;
use glasnost_sim::econ::assign_destination;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::superweapons::warp::{Member, Stage};
use glasnost_sim::teleport::TeleportState;
use glasnost_sim::temporal::Lock;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, Layer, ObjectId, Target, World};
use jump_common::{SIZE, at, step};

fn game() -> World {
    let mut w = testkit::game_with(&jump_common::factory::ruleset(), SIZE, 7);
    testkit::run(&mut w, 4);
    w
}

fn tele(w: &mut World, c: CellXY) -> ObjectId {
    testkit::spawn(w, Kind::Vehicle, "TELEQ", 0, c, 0)
}

fn offer(w: &mut World, u: ObjectId, c: CellXY) {
    assign_destination(w, u, Some(Target::Cell(c)));
}

fn landing(w: &World, u: ObjectId) -> Option<CellXY> {
    w.obj(u)
        .unwrap()
        .teleport
        .as_ref()
        .and_then(|t| t.landing)
        .map(|l| l.cell())
}

fn marked(w: &World, c: CellXY) -> bool {
    w.cells
        .occ(c, Layer::Ground)
        .is_some_and(|o| o.vehicle_present())
}

fn suppress(w: &mut World, u: ObjectId) {
    w.obj_mut(u).unwrap().econ.point_suppression = true;
}

fn suppressed(w: &World, u: ObjectId) -> bool {
    w.obj(u).unwrap().econ.point_suppression
}

/// Puts `u` under an erasing lock led by a fresh vehicle (S-0514).
fn erase(w: &mut World, u: ObjectId) {
    let leader = testkit::spawn(w, Kind::Vehicle, "TELEQ", 1, at(-8, -8), 0);
    w.temporal.locks.insert(
        u,
        Lock {
            leader,
            joiners: Vec::new(),
            amount: 1_000_000,
            was_off: false,
            suspended: false,
        },
    );
}

// spec: S-0207 claim 27 (Q-1130 table, second row: suppression pending before the assignment, a teleporting vehicle that is not disabled: the locomotor is not offered the point, no landing point is fixed, the suppression is consumed and the nonempty order-level destination stays)
// spec: S-0207 claim 27 (Edge cases: the destination stays and no jump follows until a later effective assignment; a later equal offer is ignored by the unchanged-destination rejection)
// spec: S-0502 claim 27 (point suppression also covers a teleporting locomotor's landing point)
#[test]
fn q1130_row_2_suppression_skips_the_landing_point() {
    let mut w = game();
    let u = tele(&mut w, at(0, 0));
    suppress(&mut w, u);
    let c = at(4, 0);
    offer(&mut w, u, c);
    assert_eq!(w.obj(u).unwrap().dest, Some(Target::Cell(c)));
    assert_eq!(landing(&w, u), None, "no landing point fixed");
    assert!(!marked(&w, c), "no reservation at the destination");
    assert!(!suppressed(&w, u), "consumed");
    for _ in 0..5 {
        step(&mut w);
    }
    let o = w.obj(u).unwrap();
    assert_eq!(o.pos.cell(), at(0, 0), "no jump");
    assert_eq!(o.dest, Some(Target::Cell(c)), "the destination stays");
    // A later equal offer is ignored; a different one is effective and fixes its point.
    offer(&mut w, u, c);
    assert_eq!(landing(&w, u), None, "the equal offer is ignored");
    offer(&mut w, u, at(5, 0));
    assert_eq!(landing(&w, u), Some(at(5, 0)));
}

// spec: S-0207 claim 27 (Q-1130 table, second row: in any state, including phasing in or held, pending suppression keeps the destination without a landing point and is consumed; no claim 20 a refusal runs)
// spec: S-0207 claim 20 (a) (a pending suppression is the only exception to the drop: under one, whatever the object's state, the destination is kept and no landing point is fixed) and (f) (while phasing in every new destination is dropped, except one that a pending suppression keeps)
// spec: S-0214 Terms (Being erased, phasing in: a destination given while phasing in is dropped, except under a pending point suppression, which keeps it as the order-level destination, fixes no landing point and consumes the suppression)
#[test]
fn q1130_row_2_phasing_in_or_held_keeps_the_destination() {
    for held in [false, true] {
        let mut w = game();
        let u = tele(&mut w, at(0, 0));
        let t = w.tick;
        let o = w.obj_mut(u).unwrap();
        if held {
            o.parasite.get_or_insert_with(Default::default).hold_until = Some(t + 50);
        } else {
            o.teleport = Some(TeleportState {
                phasing: Some((t, 50)),
                ..Default::default()
            });
        }
        suppress(&mut w, u);
        let c = at(4, 0);
        offer(&mut w, u, c);
        let o = w.obj(u).unwrap();
        assert_eq!(o.dest, Some(Target::Cell(c)), "held {held}: kept");
        assert_eq!(landing(&w, u), None, "held {held}");
        assert!(!suppressed(&w, u), "held {held}: consumed");
        step(&mut w);
        assert_eq!(w.obj(u).unwrap().pos.cell(), at(0, 0), "held {held}");
        assert_eq!(w.obj(u).unwrap().dest, Some(Target::Cell(c)), "held {held}");
    }
}

// spec: S-0207 claim 27 (Q-1130 table, third row: disabled, held, being erased or phasing in with no suppression pending: the destination is dropped, the order-level destination becomes empty and nothing is consumed)
// spec: S-0514 claim 16 (an erased target's destination is cleared: being erased is one of claim 20 a's disabling states)
#[test]
fn q1130_row_3_dropped_without_suppression() {
    for state in ["phasing", "held", "erased"] {
        let mut w = game();
        let u = tele(&mut w, at(0, 0));
        let t = w.tick;
        match state {
            "phasing" => {
                w.obj_mut(u).unwrap().teleport = Some(TeleportState {
                    phasing: Some((t, 50)),
                    ..Default::default()
                })
            }
            "held" => {
                w.obj_mut(u)
                    .unwrap()
                    .parasite
                    .get_or_insert_with(Default::default)
                    .hold_until = Some(t + 50)
            }
            _ => erase(&mut w, u),
        }
        offer(&mut w, u, at(4, 0));
        assert_eq!(w.obj(u).unwrap().dest, None, "{state}: dropped");
        assert_eq!(landing(&w, u), None, "{state}");
        assert!(!marked(&w, at(4, 0)), "{state}");
    }
}

// spec: S-0207 claim 27 (Q-1130 table, second row: no earlier landing point, pending jump or reservation changes; D32: an unchanged earlier jump)
#[test]
fn q1130_row_2_an_earlier_pending_jump_is_unchanged() {
    let mut w = game();
    let u = tele(&mut w, at(0, 0));
    let a = at(3, 0);
    offer(&mut w, u, a);
    assert_eq!(landing(&w, u), Some(a));
    suppress(&mut w, u);
    let b = at(-4, 2);
    offer(&mut w, u, b);
    assert_eq!(landing(&w, u), Some(a), "the earlier landing point stays");
    assert!(marked(&w, a), "its reservation stays");
    assert!(!marked(&w, b));
    assert_eq!(w.obj(u).unwrap().dest, Some(Target::Cell(b)));
    assert!(!suppressed(&w, u));
    step(&mut w);
    assert_eq!(w.obj(u).unwrap().pos.cell(), a, "the earlier jump is made");
}

// spec: S-0207 claim 20 (h) (an empty assignment while a jump is pending cancels the pending jump and the moving state; the landing point stays remembered and its vehicle-present mark does not move; the next accepted destination clears it from the remembered point)
// spec: S-0207 State and ordering (teleporting, L fixed, jump pending, empty destination assigned; L fixed, no pending jump: no jump at the next movement update)
#[test]
fn claim_20_h_an_empty_assignment_cancels_the_pending_jump() {
    let mut w = game();
    let u = tele(&mut w, at(0, 0));
    let a = at(3, 0);
    offer(&mut w, u, a);
    assert!(marked(&w, a));
    assert_eq!(landing(&w, u), Some(a), "a jump is pending (moving)");
    assign_destination(&mut w, u, None);
    assert_eq!(landing(&w, u), None, "no pending jump, not moving");
    assert!(marked(&w, a), "the reservation stays at L's cell");
    for _ in 0..3 {
        step(&mut w);
    }
    assert_eq!(w.obj(u).unwrap().pos.cell(), at(0, 0), "no jump");
    assert!(marked(&w, a), "still reserved");
    // The next accepted destination clears the reservation from the remembered point.
    let b = at(-3, 3);
    offer(&mut w, u, b);
    assert!(!marked(&w, a), "cleared from the remembered point");
    assert!(marked(&w, b));
    step(&mut w);
    assert_eq!(w.obj(u).unwrap().pos.cell(), b);
}

// spec: S-0215 claim 2 (an infantry unit or vehicle being erased drops its target and destination on each such tick, except that a vehicle under a pending point suppression keeps its destination; the target is dropped in every case)
// spec: S-0514 claim 17 (being erased clears target and destination, except that a pending point suppression keeps the destination) and claim 16 (under a pending suppression the destination is kept, and the target still cannot jump)
#[test]
fn being_erased_under_pending_suppression_keeps_the_destination() {
    let mut w = game();
    let u = tele(&mut w, at(0, 0));
    let other = testkit::spawn(&mut w, Kind::Vehicle, "TELEQ", 1, at(6, 6), 0);
    let c = at(4, 0);
    let o = w.obj_mut(u).unwrap();
    o.dest = Some(Target::Cell(c));
    o.target = Some(Target::Object(other));
    o.econ.point_suppression = true;
    erase(&mut w, u);
    step(&mut w);
    let o = w.obj(u).unwrap();
    assert_eq!(o.target, None, "the target is dropped");
    assert_eq!(o.dest, Some(Target::Cell(c)), "the destination is kept");
    assert!(o.econ.point_suppression, "reading it does not consume it");
    assert_eq!(o.pos.cell(), at(0, 0), "no jump while erased");
    // An assignment consumes the suppression; the next erasure tick drops the destination.
    let d = at(5, 1);
    offer(&mut w, u, d);
    assert_eq!(w.obj(u).unwrap().dest, Some(Target::Cell(d)));
    assert!(!suppressed(&w, u));
    step(&mut w);
    assert_eq!(
        w.obj(u).unwrap().dest,
        None,
        "dropped once nothing is pending"
    );
    assert_eq!(w.obj(u).unwrap().pos.cell(), at(0, 0));
}

// spec: S-0215 claim 2 (a vehicle in the leaving stage of a chrono move under a pending point suppression keeps its destination; the target is dropped in every case; without one the destination is dropped)
#[test]
fn leaving_stage_under_pending_suppression_keeps_the_destination() {
    for pending in [false, true] {
        let mut w = game();
        let u = tele(&mut w, at(0, 0));
        let other = testkit::spawn(&mut w, Kind::Vehicle, "TELEQ", 1, at(6, 6), 0);
        let c = at(4, 0);
        let pos = w.obj(u).unwrap().pos;
        let o = w.obj_mut(u).unwrap();
        o.dest = Some(Target::Cell(c));
        o.target = Some(Target::Object(other));
        o.econ.point_suppression = pending;
        o.beh.current = Some(Behaviour::Guard);
        o.sw.warp = Some(Member {
            stage: Stage::Pending,
            house: HouseId(0),
            landing: pos,
            redirected: false,
            fell_back: false,
            refused: false,
            entry_ok: false,
        });
        step(&mut w);
        let o = w.obj(u).unwrap();
        assert_eq!(o.target, None, "pending {pending}: the target is dropped");
        let want = pending.then_some(Target::Cell(c));
        assert_eq!(o.dest, want, "pending {pending}");
    }
}
