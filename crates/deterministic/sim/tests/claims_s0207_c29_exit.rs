//! Tests of S-0207 claims 29 and 31 (r103, Q-3374) and S-0417 claim 5: a
//! driving vehicle that holds an order-level destination when the factory
//! exit track installs *E*'s point reads *E*'s point while it is present;
//! a new assignment installs its own. On an empty synthetic map with
//! invented types: every name and number here is made up, no game data.
//! House Alpha (0) of the testkit's sandbox game.

use glasnost_map::CellXY;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{ObjectId, Pos, Target, World};
use glasnost_sim::{drive, econ, path, testkit};

const SIZE: i32 = 30;

fn game() -> World {
    let mut w = testkit::game_with(&testkit::combat_ruleset(), SIZE, 7);
    testkit::run(&mut w, 2);
    w
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

/// A driver at (0, 0) given cell D = (-8, 5) as its destination, then put
/// on the exit track towards E = (1, 0), the order-level destination kept:
/// the driver, D and E.
fn exit_scene() -> (World, ObjectId, CellXY, CellXY) {
    let mut w = game();
    let v = testkit::spawn(&mut w, Kind::Vehicle, "CRUSHER", 0, at(0, 0), 0);
    testkit::run(&mut w, 2);
    let d = at(-8, 5);
    econ::assign_destination(&mut w, v, Some(Target::Cell(d)));
    let e = at(1, 0);
    drive::begin_exit_track(&mut w, v, e);
    assert_eq!(
        w.obj(v).unwrap().dest,
        Some(Target::Cell(d)),
        "the exit track keeps the order-level destination"
    );
    // spec: S-0417 claim 5 (the exit-track instruction supplies the locomotor destination point at E's horizontal centre with height 0)
    assert_eq!(
        drive::goal(&w, v),
        Some(Pos::new(256 * e.x + 128, 256 * e.y + 128, 0)),
        "E's point installed"
    );
    (w, v, d, e)
}

// spec: S-0207 claim 31 (r103, Q-3374: a destination held from before the exit-track instruction is not asked for its point while the exit point is present, claim 29)
#[test]
fn claim_31_a_drivers_readings_on_the_exit_track_use_the_exit_point() {
    let (w, v, d, e) = exit_scene();
    assert_ne!(d, e);
    assert_eq!(path::search_goal(&w, v), Some(e), "the search looks for E");
    assert_eq!(
        drive::measured_point(&w, v).map(|p| p.cell()),
        Some(e),
        "braking and the give-up test measure to E's point"
    );
}

// spec: S-0207 claim 31 (r103: a destination assigned during the exit track installs its own point, claim 25) and claim 29 (the readings use the point it installed)
#[test]
fn claim_31_a_new_assignment_installs_its_point() {
    let (mut w, v, _, _) = exit_scene();
    let f = at(6, -4);
    econ::assign_destination(&mut w, v, Some(Target::Cell(f)));
    let p = drive::goal(&w, v).expect("F's point installed");
    assert_eq!(p.cell(), f);
    assert_eq!(drive::measured_point(&w, v), Some(p));
    assert_eq!(path::search_goal(&w, v), Some(f));
}
