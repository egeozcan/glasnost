//! Claim tests of S-0207 claim 29 (r92, Q-3353): the point a driving
//! vehicle measures to is the locomotor destination point installed at
//! the last assignment (claim 25), not the point its destination would
//! give it at the time of the reading; the completion row of claim 25 asks
//! the destination again. On an empty synthetic map with invented types:
//! every name and number here is made up, no game data. House Alpha (0) of
//! the testkit's sandbox game.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::BuildingType;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::bunker;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};
use glasnost_sim::{drive, econ, path, testkit};

const SIZE: i32 = 30;

/// The combat ruleset with PIT, a 2 × 2 vehicle bunker of our own.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("PIT".into());
    r.sim.buildings.insert(
        "PIT".into(),
        BuildingType {
            strength: Some(1000),
            foundation: Some([2, 2]),
            number_impassable_rows: Some(0),
            bunker: Some(true),
            ..Default::default()
        },
    );
    r
}

fn game() -> World {
    let mut w = testkit::game_with(&rules(), SIZE, 7);
    testkit::run(&mut w, 2);
    w
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn put(w: &mut World, kind: Kind, name: &str, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, 0, at(dx, dy), 0)
}

fn xy(p: Pos) -> (i32, i32) {
    (p.x, p.y)
}

/// A bunker with its position cell at (0, 0) and a driver west of it with
/// no bunker contact, given the bunker as its destination: the bunker, the
/// driver and the quadrant point installed at the assignment.
fn bunker_scene() -> (World, ObjectId, ObjectId, Pos) {
    let mut w = game();
    let b = put(&mut w, Kind::Building, "PIT", 0, 0);
    testkit::run(&mut w, 3);
    let v = put(&mut w, Kind::Vehicle, "CRUSHER", -8, 1);
    testkit::run(&mut w, 2);
    assert_eq!(w.obj(v).unwrap().contact, None, "no contact");
    econ::assign_destination(&mut w, v, Some(Target::Object(b)));
    let q = drive::goal(&w, v).expect("a point installed at the assignment");
    assert_eq!(q, bunker::docking_point(&w, b, v), "the quadrant point then");
    (w, b, v, q)
}

/// Puts the driver at the centre of the cell (`dx`, `dy`) east of the
/// bunker, as if it had driven round it, with no new assignment.
fn move_round(w: &mut World, v: ObjectId, dx: i32, dy: i32) {
    let c = at(dx, dy);
    let z = w.obj(v).unwrap().pos.z;
    w.obj_mut(v).unwrap().pos = Pos::new(256 * c.x + 128, 256 * c.y + 128, z);
}

// spec: S-0207 claim 29 (r92, Q-3353: every path search of S-0212 claims 20 to 23 looks for the cell of the point installed at the last assignment; for a bunker destination the docking cell of S-0606 claim 4's quadrant at the assignment, while the vehicle drives round the bunker) and S-0208 claim 27
#[test]
fn claim_29_a_drivers_searches_keep_the_bunker_point_of_the_assignment() {
    let (mut w, b, v, q) = bunker_scene();
    move_round(&mut w, v, 9, 1);
    let now = bunker::docking_point(&w, b, v);
    assert_ne!(now.cell(), q.cell(), "the quadrant point from here differs");
    assert_eq!(drive::goal(&w, v), Some(q), "the installed point stays");
    assert_eq!(
        path::search_goal(&w, v),
        Some(q.cell()),
        "the searches look for the installed point's cell"
    );
}

// spec: S-0207 claim 29 (r92, Q-3353: the 512-lepton start of S-0212 claim 20's recomputation, the braking test of claim 10 and the give-up test of claim 15 measure from the vehicle to the installed point, the vehicle's move destination, S-0207 Terms)
#[test]
fn claim_29_braking_and_the_give_up_test_measure_to_the_installed_point() {
    let (mut w, b, v, q) = bunker_scene();
    move_round(&mut w, v, 9, 1);
    assert_ne!(xy(bunker::docking_point(&w, b, v)), xy(q));
    assert_eq!(drive::measured_point(&w, v), Some(q));
}

// spec: S-0207 claim 29 (r92, Q-3353: only a new assignment supplies a new point) and claim 25 (an accepted instruction with a point replaces the old one)
#[test]
fn claim_29_a_new_assignment_takes_the_point_again() {
    let (mut w, b, v, _) = bunker_scene();
    move_round(&mut w, v, 9, 1);
    econ::assign_destination(&mut w, v, Some(Target::Object(b)));
    let now = bunker::docking_point(&w, b, v);
    assert_eq!(drive::goal(&w, v), Some(now));
    assert_eq!(path::search_goal(&w, v), Some(now.cell()));
    assert_eq!(drive::measured_point(&w, v), Some(now));
}

/// A driver given a move onto a friendly vehicle `u` that then drives
/// away: the driver, `u` and the point installed at the assignment.
fn chase_scene() -> (World, ObjectId, ObjectId, Pos) {
    let mut w = game();
    let u = put(&mut w, Kind::Vehicle, "BUGGY", 0, 0);
    let v = put(&mut w, Kind::Vehicle, "BUGGY", -6, 0);
    while orders::blocked(&w) {
        testkit::run(&mut w, 1);
    }
    let order = |w: &mut World, actor: ObjectId, dest: Target| {
        assert!(
            orders::issue(
                w,
                HouseId(0),
                Order::Behaviour {
                    actor,
                    behaviour: Behaviour::Move,
                    attack_move: false,
                    target: None,
                    dest: Some(dest),
                },
            ),
            "order refused"
        );
        orders::apply_pending(w);
    };
    order(&mut w, v, Target::Object(u));
    testkit::run(&mut w, 1);
    let p = drive::goal(&w, v).expect("installed at the assignment");
    assert_eq!(p.cell(), at(0, 0), "the unit's position then");
    order(&mut w, u, Target::Cell(at(0, 8)));
    (w, v, u, p)
}

// spec: S-0207 claim 29 (r92, Q-3353: the searches look for the installed point's cell, which does not follow the destination) and claim 25 (the completion row asks the destination for its point when the track ends: the vehicle is not in the cell of the point the unit supplies then, so the installed point stays)
#[test]
fn claim_29_a_driver_goes_to_the_point_of_the_assignment_and_keeps_it() {
    let (mut w, v, u, p) = chase_scene();
    testkit::run(&mut w, 400);
    let o = w.obj(v).unwrap();
    assert_ne!(w.obj(u).unwrap().pos.cell(), at(0, 0), "the unit has left");
    assert_eq!(o.pos.cell(), p.cell(), "the driver ends where the unit was");
    assert_eq!(o.dest, Some(Target::Object(u)), "it keeps the unit");
    assert_eq!(drive::goal(&w, v), Some(p), "and the installed point");
}
