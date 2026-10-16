//! Claim tests of S-0207 revision 32 claim 27 (answers Q-1090): the
//! destination-refresh state, the one-application movement-point
//! suppression and the teleport-capable dock fallback, with S-0502 claim 27
//! and S-0402 claim 10 row 2. Every name and number is invented; no game
//! data.

mod jump_common;

use glasnost_map::CellXY;
use glasnost_sim::econ::assign_destination;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};
use jump_common::{SIZE, at, step};

fn game() -> World {
    let mut w = testkit::game_with(&jump_common::factory::ruleset(), SIZE, 7);
    testkit::run(&mut w, 4);
    w
}

fn spawn(w: &mut World, kind: Kind, name: &str, c: CellXY) -> ObjectId {
    testkit::spawn(w, kind, name, 0, c, 0)
}

/// A qualified scene: a driving vehicle with the teleport-capable flag in
/// contact with a dock-unload refinery, and a free cell to offer.
fn dock_scene() -> (World, ObjectId, CellXY) {
    let mut w = game();
    let refinery = spawn(&mut w, Kind::Building, "REFN", at(-6, 0));
    let u = spawn(&mut w, Kind::Vehicle, "TELEDRIVE", at(0, 0));
    w.obj_mut(u).unwrap().contact = Some(refinery);
    let offer = at(3, 0);
    (w, u, offer)
}

fn flags(w: &World, u: ObjectId) -> (bool, bool) {
    let o = w.obj(u).unwrap();
    (o.econ.refresh, o.econ.point_suppression)
}

fn offer(w: &mut World, u: ObjectId, c: CellXY) {
    assign_destination(w, u, Some(Target::Cell(c)));
}

// spec: S-0207 claim 27 (successful placement enables refresh only during initial idle-behaviour setup and clears it when that setup finishes; no suppression)
#[test]
fn placement_leaves_no_pending_state() {
    let mut w = game();
    for name in ["TELEDRIVE", "JV", "TELE"] {
        let u = spawn(&mut w, Kind::Vehicle, name, at(0, 0));
        assert_eq!(flags(&w, u), (false, false), "{name}");
        assert!(w.obj(u).unwrap().dest.is_none(), "{name}");
    }
}

// spec: S-0207 claim 27 (the qualified dock fallback: movement stops, current behaviour is cleared, enter is queued without an immediate start, and both pending states become pending; the assignment's own nonempty point is skipped, which consumes the suppression, while refresh remains after that very assignment)
#[test]
fn the_dock_fallback_skips_its_own_point_and_leaves_refresh() {
    let (mut w, u, c) = dock_scene();
    w.obj_mut(u).unwrap().beh.current = Some(Behaviour::Guard);
    offer(&mut w, u, c);
    let o = w.obj(u).unwrap();
    assert_eq!(
        o.dest,
        Some(Target::Cell(c)),
        "the order-level destination is installed"
    );
    assert_eq!(
        o.drive.as_ref().unwrap().goal,
        None,
        "its movement point is skipped"
    );
    assert_eq!(o.drive.as_ref().unwrap().for_dest, Some(Target::Cell(c)));
    assert_eq!(o.beh.current, None);
    assert_eq!(o.beh.queued, Some(Behaviour::Enter));
    assert_eq!(
        flags(&w, u),
        (true, false),
        "refresh stays, suppression is consumed"
    );
}

// spec: S-0207 claim 27 (an unchanged assignment is ignored before the dock test without refresh, and reaches it again with refresh pending: there is no once-per-entry limit; each fallback skips its own point and leaves refresh)
#[test]
fn a_repeated_offer_reaches_the_test_again_through_refresh() {
    let (mut w, u, c) = dock_scene();
    offer(&mut w, u, c);
    assert_eq!(flags(&w, u), (true, false));
    // The same offer again: refresh makes it effective and it qualifies again.
    w.obj_mut(u).unwrap().beh.queued = None;
    offer(&mut w, u, c);
    let o = w.obj(u).unwrap();
    assert_eq!(o.beh.queued, Some(Behaviour::Enter), "tested again");
    assert_eq!(
        o.drive.as_ref().unwrap().goal,
        None,
        "its point is skipped again"
    );
    assert_eq!(flags(&w, u), (true, false));
    // With refresh gone, the unchanged offer is ignored before the test.
    w.obj_mut(u).unwrap().econ.refresh = false;
    w.obj_mut(u).unwrap().beh.queued = None;
    offer(&mut w, u, c);
    let o = w.obj(u).unwrap();
    assert_eq!(
        o.beh.queued, None,
        "an unchanged offer without refresh is ignored"
    );
    assert_eq!(flags(&w, u), (false, false));
}

// spec: S-0207 claim 27 and S-0402 claim 10 row 2 (acquiring contact alone, or a change of occupancy, triggers no test: only an effective assignment does)
#[test]
fn contact_alone_does_not_trigger_the_test() {
    let mut w = game();
    let refinery = spawn(&mut w, Kind::Building, "REFN", at(-6, 0));
    let u = spawn(&mut w, Kind::Vehicle, "TELEDRIVE", at(0, 0));
    let c = at(3, 0);
    offer(&mut w, u, c);
    assert_eq!(flags(&w, u), (false, false));
    w.obj_mut(u).unwrap().contact = Some(refinery);
    for _ in 0..5 {
        step(&mut w);
    }
    assert_eq!(flags(&w, u), (false, false));
    // The same cell again, unchanged and unrefreshed: still ignored.
    offer(&mut w, u, c);
    assert_eq!(flags(&w, u), (false, false));
}

// spec: S-0207 claim 27 (every predicate is needed: the teleport-capable type flag, a contact that is a dock-unload building, a cell destination holding no ground-layer vehicle of any house, the vehicle itself included, and a current locomotor that is not teleporting; failure of any adds no stop, restoration or pending state)
#[test]
fn every_predicate_is_needed() {
    // No type flag.
    let mut w = game();
    let refinery = spawn(&mut w, Kind::Building, "REFN", at(-6, 0));
    let u = spawn(&mut w, Kind::Vehicle, "JV", at(0, 0));
    w.obj_mut(u).unwrap().contact = Some(refinery);
    w.obj_mut(u).unwrap().beh.current = Some(Behaviour::Guard);
    offer(&mut w, u, at(3, 0));
    assert_eq!(flags(&w, u), (false, false));
    assert_eq!(w.obj(u).unwrap().beh.current, Some(Behaviour::Guard));
    // A contact that does not permit dock unloading.
    let (mut w, u, c) = dock_scene();
    let plain = spawn(&mut w, Kind::Building, "PLAIN", at(-6, 6));
    w.obj_mut(u).unwrap().contact = Some(plain);
    offer(&mut w, u, c);
    assert_eq!(flags(&w, u), (false, false));
    // No contact.
    let (mut w, u, c) = dock_scene();
    w.obj_mut(u).unwrap().contact = None;
    offer(&mut w, u, c);
    assert_eq!(flags(&w, u), (false, false));
    // A building destination is not the required cell destination.
    let (mut w, u, _) = dock_scene();
    let b = spawn(&mut w, Kind::Building, "LOW", at(4, 4));
    assign_destination(&mut w, u, Some(Target::Object(b)));
    assert_eq!(flags(&w, u), (false, false));
    // A ground-layer vehicle in the offered cell, of any house.
    let (mut w, u, c) = dock_scene();
    testkit::spawn(&mut w, Kind::Vehicle, "JV", 1, c, 0);
    offer(&mut w, u, c);
    assert_eq!(flags(&w, u), (false, false));
    // The vehicle itself already standing there.
    let (mut w, u, _) = dock_scene();
    offer(&mut w, u, at(0, 0));
    assert_eq!(flags(&w, u), (false, false));
    // A teleporting current locomotor.
    let mut w = game();
    let refinery = spawn(&mut w, Kind::Building, "REFN", at(-6, 0));
    let u = spawn(&mut w, Kind::Vehicle, "TELE", at(0, 0));
    w.obj_mut(u).unwrap().contact = Some(refinery);
    offer(&mut w, u, at(3, 0));
    assert_eq!(flags(&w, u), (false, false));
}

// spec: S-0207 claim 27 (suppression skips the next nonempty point, equally for driving and for a jumpjet movement point; an empty clear stops locomotion but does not consume it)
#[test]
fn an_empty_clear_does_not_consume_the_suppression() {
    for name in ["TELEDRIVE", "JV"] {
        let mut w = game();
        let u = spawn(&mut w, Kind::Vehicle, name, at(0, 0));
        w.obj_mut(u).unwrap().econ.point_suppression = true;
        // A first, nonempty destination: the order changes, the point is skipped.
        offer(&mut w, u, at(4, 0));
        assert!(!w.obj(u).unwrap().econ.point_suppression, "{name}");
        let o = w.obj(u).unwrap();
        assert_eq!(o.dest, Some(Target::Cell(at(4, 0))), "{name}");
        match name {
            "JV" => {
                let j = o.jump.as_ref().unwrap();
                assert_eq!(j.dest, None, "{name}: no movement point");
                assert!(!j.moving, "{name}");
            }
            _ => assert_eq!(o.drive.as_ref().unwrap().goal, None, "{name}"),
        }
        // The next nonempty assignment applies its point.
        offer(&mut w, u, at(5, 0));
        let o = w.obj(u).unwrap();
        match name {
            "JV" => assert!(o.jump.as_ref().unwrap().dest.is_some(), "{name}"),
            _ => assert!(o.drive.as_ref().unwrap().goal.is_some(), "{name}"),
        }
        // An empty clear does not consume a pending suppression.
        w.obj_mut(u).unwrap().econ.point_suppression = true;
        assign_destination(&mut w, u, None);
        assert!(w.obj(u).unwrap().econ.point_suppression, "{name}");
    }
}

// spec: S-0207 claim 27 and S-0502 claim 27 (pending refresh makes an unchanged assignment of a jumpjet vehicle effective: the movement-point adjustment repeats with its draw; the assignment consumes refresh; without it the repeat is ignored)
#[test]
fn refresh_makes_an_unchanged_jumpjet_assignment_effective() {
    let mut w = game();
    let u = spawn(&mut w, Kind::Vehicle, "JV", at(0, 0));
    offer(&mut w, u, at(6, 0));
    let t0 = w.rng.taken();
    offer(&mut w, u, at(6, 0));
    assert_eq!(
        w.rng.taken(),
        t0,
        "an unchanged vehicle assignment is ignored"
    );
    w.obj_mut(u).unwrap().econ.refresh = true;
    let point = w.obj(u).unwrap().jump.as_ref().unwrap().dest;
    offer(&mut w, u, at(6, 0));
    assert_eq!(
        flags(&w, u),
        (false, false),
        "the assignment consumed refresh"
    );
    assert_eq!(w.obj(u).unwrap().jump.as_ref().unwrap().dest, point);
}

// spec: S-0207 claim 27 table row 1 (temporary driving with a suspended predecessor, no movement report, no restoration lock and not seized: the predecessor is restored, with neither pending state enabled; the offered destination then goes to the restored locomotor)
// spec: S-0207 claim 27 table rows 2 and 3 (a suspended predecessor whose restoration is not permitted, here by a retained locomotor destination point: the fallback)
#[test]
fn temporary_driving_restores_or_falls_back() {
    for report in [false, true] {
        let mut w = game();
        let refinery = spawn(&mut w, Kind::Building, "REFN", at(-6, 0));
        let u = spawn(&mut w, Kind::Vehicle, "TELE", at(0, 0));
        glasnost_sim::econ::release::install_temporary_drive(&mut w, u);
        w.obj_mut(u).unwrap().contact = Some(refinery);
        if report {
            w.obj_mut(u).unwrap().drive.as_mut().unwrap().goal =
                Some(glasnost_sim::world::Pos::centre(at(1, 0), 0));
        }
        assert!(glasnost_sim::econ::release::temporary_drive(&w, u));
        offer(&mut w, u, at(3, 0));
        if report {
            assert!(
                glasnost_sim::econ::release::temporary_drive(&w, u),
                "no restoration with a report"
            );
            assert_eq!(w.obj(u).unwrap().beh.queued, Some(Behaviour::Enter));
            assert_eq!(flags(&w, u), (true, false));
        } else {
            assert!(
                !glasnost_sim::econ::release::temporary_drive(&w, u),
                "restored"
            );
            assert_eq!(flags(&w, u), (false, false));
            assert_eq!(w.obj(u).unwrap().beh.queued, None);
        }
    }
}
