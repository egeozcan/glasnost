//! Claim tests of S-0207 r103 claims 30 and 31 (Q-3373, Q-3374): a failed
//! fresh search of a driving vehicle clears its locomotor destination point
//! and, within 1 cell, keeps its order-level destination; a locomotion
//! update with no point installs the destination's point again; a point in
//! the vehicle's own cell is searched for and cleared; an infantry
//! destination is asked again at a track's end. On synthetic maps with
//! invented types (the `common` scene's TESTCAR, the testkit's CRUSHER,
//! BUGGY and WALKER): every name and number here is made up, no game data.

mod common;

use common::{Scene, WHEEL, cell, objects_of, open_orders, order_move, unit_type};
use glasnost_map::CellXY;
use glasnost_rules::sim::Decimal;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{drive, econ, testkit, tick};

const CAR: &str = "TESTCAR";

fn step(w: &mut World) {
    tick::step(w);
}

/// A TESTCAR of Alpha at cell(0, 0) and an enemy TESTCAR at cell(1, 0),
/// which blocks that cell for the unarmed car (S-0212 claim 15); R = 45.
fn blocked_scene() -> (World, ObjectId) {
    let mut s = Scene::new();
    s.vehicle(CAR, unit_type(WHEEL, 40, 6));
    s.ruleset.sim.general.path_delay = Some(Decimal("0.05".into()));
    s.place_unit(1, CAR, cell(1, 0), 64);
    s.place_unit(0, CAR, cell(0, 0), 64);
    let mut w = s.build();
    open_orders(&mut w);
    let a = objects_of(&w, Kind::Vehicle)
        .into_iter()
        .find(|&v| w.obj(v).unwrap().pos.cell() == cell(0, 0))
        .unwrap();
    assert_eq!(w.rules().general.retry_delay, 45);
    (w, a)
}

/// Steps until `a`'s retry delay runs past the current tick, which a failed
/// fresh search starts; the tick of that failure.
fn until_failure(w: &mut World, a: ObjectId, max: i32) -> i32 {
    for _ in 0..max {
        step(w);
        let o = w.obj(a).unwrap();
        if o.path.retry_at >= w.tick {
            return w.tick - 1;
        }
    }
    panic!("no failed search within {max} ticks");
}

// spec: S-0207 claim 31 (r103, Q-3373: a failed fresh search clears the locomotor destination point in that update and R starts; within 1 cell, S-0212 claim 23 row 3, the vehicle keeps its order-level destination and R keeps running) and claim 30 (the next locomotion update installs the destination's point again and begins no track; it searches once R has passed, about every R ticks, for as long as it holds the destination)
#[test]
fn claim_31_a_failure_within_one_cell_keeps_the_destination_and_searches_every_r() {
    let (mut w, a) = blocked_scene();
    let r = w.rules().general.retry_delay;
    order_move(&mut w, a, cell(1, 0));
    let f = until_failure(&mut w, a, 10);
    let o = w.obj(a).unwrap();
    assert_eq!(o.path.retry_at, f + r, "R starts at the failure");
    assert_eq!(o.dest, Some(Target::Cell(cell(1, 0))), "the destination stays");
    assert_eq!(drive::goal(&w, a), None, "the point is cleared");
    // spec: S-0207 claim 30 (r103: no point at a locomotion update, the destination held: its point is installed and no track begins)
    step(&mut w);
    let o = w.obj(a).unwrap();
    let p = drive::goal(&w, a).expect("the point installed again");
    assert_eq!(p.cell(), cell(1, 0));
    assert_eq!(o.pos.cell(), cell(0, 0));
    assert!(o.drive.as_ref().unwrap().track.is_none(), "no track begins");
    assert_eq!(o.path.retry_at, f + r, "the install does not end R");
    // spec: S-0207 claim 31 (r103: the search again once R has passed)
    let first = f + r;
    common::run_until(&mut w, 3 * r, |w| w.obj(a).unwrap().path.retry_at != first);
    assert_eq!(w.obj(a).unwrap().path.retry_at, first + r, "searched at R");
    common::run(&mut w, 4 * r);
    let o = w.obj(a).unwrap();
    assert_eq!(o.dest, Some(Target::Cell(cell(1, 0))), "still held");
    assert_eq!(o.beh.current, Some(Behaviour::Move), "move does not end");
    assert_eq!(o.pos.cell(), cell(0, 0), "it stands where it is");
}

fn combat_game() -> World {
    let mut w = testkit::game_with(&testkit::combat_ruleset(), 30, 7);
    testkit::run(&mut w, 2);
    w
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(30, dx, dy)
}

fn order(w: &mut World, actor: ObjectId, behaviour: Behaviour, dest: Target) {
    while orders::blocked(w) {
        testkit::run(w, 1);
    }
    assert!(
        orders::issue(
            w,
            HouseId(0),
            Order::Behaviour {
                actor,
                behaviour,
                attack_move: false,
                target: None,
                dest: Some(dest),
            },
        ),
        "order refused"
    );
    orders::apply_pending(w);
}

fn en_route(w: &World, v: ObjectId) -> bool {
    let o = w.obj(v).unwrap();
    o.drive.as_ref().unwrap().track.is_some() || !o.path.steps.is_empty()
}

// spec: S-0207 claim 31 (r103: a point in the vehicle's own cell: with no track and no stored steps it searches for that cell at once, the update in which a track ends there included; the search finds no path, S-0212 claim 12, and the point is cleared, the destination kept) and claim 30 (Consequences: a unit named by the destination that has moved supplies its new point at the next update, so the vehicle drives after it)
#[test]
fn claim_31_a_track_ending_in_the_points_cell_clears_it_and_the_driver_follows() {
    let mut w = combat_game();
    let u = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, at(0, 0), 0);
    let v = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, at(-6, 0), 0);
    order(&mut w, v, Behaviour::Move, Target::Object(u));
    testkit::run(&mut w, 1);
    let p = drive::goal(&w, v).expect("installed at the assignment");
    assert_eq!(p.cell(), at(0, 0));
    order(&mut w, u, Behaviour::Move, Target::Cell(at(0, 8)));
    let mut cleared_at = None;
    for _ in 0..600 {
        let before = en_route(&w, v);
        step(&mut w);
        let o = w.obj(v).unwrap();
        if before && !en_route(&w, v) && o.pos.cell() == p.cell() {
            cleared_at = Some(w.tick - 1);
            assert_eq!(drive::goal(&w, v), None, "cleared in the update the track ends");
            assert_eq!(o.dest, Some(Target::Object(u)), "the unit kept");
            assert!(o.path.retry_at >= w.tick, "R runs");
            break;
        }
    }
    assert!(cleared_at.is_some(), "the driver never ended a track at p");
    step(&mut w);
    let now = w.obj(u).unwrap().pos;
    assert_eq!(drive::goal(&w, v), Some(now), "the unit's new point installed");
    testkit::run(&mut w, 600);
    let (vc, uc) = (w.obj(v).unwrap().pos.cell(), w.obj(u).unwrap().pos.cell());
    assert!(
        (vc.x - uc.x).abs().max((vc.y - uc.y).abs()) <= 2,
        "the driver went after the unit: {vc:?} and {uc:?}"
    );
}

/// Steps until `v` has been en route while its installed point differs
/// from `p`; whether it was.
fn point_changes_en_route(w: &mut World, v: ObjectId, p: glasnost_sim::world::Pos) -> bool {
    for _ in 0..400 {
        step(w);
        if en_route(w, v) && drive::goal(w, v).is_some_and(|g| g != p) {
            return true;
        }
        if !en_route(w, v) && w.obj(v).unwrap().pos.cell() == p.cell() {
            return false;
        }
    }
    false
}

// spec: S-0207 claim 30 (r103: when a track ends and the vehicle still has a point or stored steps, an infantry destination is asked for its point, which is installed when it differs)
#[test]
fn claim_30_an_infantry_destination_is_asked_again_at_a_tracks_end() {
    let mut w = combat_game();
    let i = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, at(0, 0), 0);
    let v = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, at(-10, 0), 0);
    order(&mut w, v, Behaviour::Move, Target::Object(i));
    testkit::run(&mut w, 1);
    let p = drive::goal(&w, v).expect("installed at the assignment");
    order(&mut w, i, Behaviour::Move, Target::Cell(at(0, 5)));
    assert!(
        point_changes_en_route(&mut w, v, p),
        "the walker's new point is installed while the driver is on its way"
    );
}

// spec: S-0207 claim 30 (r103: a destination of any other kind is not asked at a track's end) and claim 29 (the readings use the installed point)
#[test]
fn claim_30_a_vehicle_destination_is_not_asked_at_a_tracks_end() {
    let mut w = combat_game();
    let u = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, at(0, 0), 0);
    let v = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, at(-10, 0), 0);
    order(&mut w, v, Behaviour::Move, Target::Object(u));
    testkit::run(&mut w, 1);
    let p = drive::goal(&w, v).expect("installed at the assignment");
    order(&mut w, u, Behaviour::Move, Target::Cell(at(0, 5)));
    assert!(
        !point_changes_en_route(&mut w, v, p),
        "the installed point stays while the driver is on its way"
    );
}

/// A CRUSHER at (0, 0) holding cell D = (-8, 5) as its order-level
/// destination with its point cleared, in behaviour `b`.
fn pointless(b: Behaviour) -> (World, ObjectId, CellXY) {
    let mut w = combat_game();
    let v = testkit::spawn(&mut w, Kind::Vehicle, "CRUSHER", 0, at(0, 0), 0);
    testkit::run(&mut w, 2);
    let d = at(-8, 5);
    econ::assign_destination(&mut w, v, Some(Target::Cell(d)));
    drive::stop_point(&mut w, v);
    let o = w.obj_mut(v).unwrap();
    o.beh.current = Some(b);
    o.beh.queued = None;
    assert_eq!(drive::goal(&w, v), None);
    (w, v, d)
}

// spec: S-0207 claim 30 (r103: no point at a locomotion update with an order-level destination held: the point it supplies then is installed, no track begins in that update, and from the next update the vehicle searches for it and begins a track)
#[test]
fn claim_30_an_update_with_no_point_installs_the_destinations_point() {
    let (mut w, v, d) = pointless(Behaviour::Move);
    let here = w.obj(v).unwrap().pos;
    step(&mut w);
    let p = drive::goal(&w, v).expect("installed");
    assert_eq!(p.cell(), d);
    assert_eq!(w.obj(v).unwrap().pos, here, "no track in that update");
    // The body turns to the path first, then the track begins.
    common::run(&mut w, 120);
    assert_ne!(w.obj(v).unwrap().pos.cell(), here.cell(), "then it drives");
}

// spec: S-0207 claim 30 (r103: no install while the current behaviour is guard; the vehicle keeps its destination with no point, as claim 26 leaves it)
#[test]
fn claim_30_a_guard_vehicle_installs_nothing() {
    let (mut w, v, d) = pointless(Behaviour::Guard);
    let here = w.obj(v).unwrap().pos;
    step(&mut w);
    assert_eq!(drive::goal(&w, v), None, "nothing installed");
    assert_eq!(w.obj(v).unwrap().dest, Some(Target::Cell(d)), "kept");
    common::run(&mut w, 30);
    assert_eq!(w.obj(v).unwrap().pos, here, "no search, no track");
}
