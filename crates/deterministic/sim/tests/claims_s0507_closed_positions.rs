//! S-0507 claim 15 with S-0312 revision 88 claim 34 (b) (S24's seam of
//! #383): only an open-topped transport moves its passengers with itself.
//! Synthetic rules: the test kit's combat rules plus VAN (an unarmed BUGGY
//! with 2 seats) and OPEN (VAN, open-topped). Every name and number is
//! invented; no game data.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::Decimal;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{ObjectId, Pos, World};
use glasnost_sim::{testkit, transport};

const SIZE: i32 = 24;

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let mut van = r.sim.vehicles["BUGGY"].clone();
    van.primary = None;
    van.passengers = Some(2);
    van.size_limit = Some(Decimal("2".into()));
    let mut open = van.clone();
    open.open_topped = Some(true);
    register(&mut r, "vehicles", "VAN");
    r.sim.vehicles.insert("VAN".into(), van);
    register(&mut r, "vehicles", "OPEN");
    r.sim.vehicles.insert("OPEN".into(), open);
    r
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

/// A game with a WALKER of house 0 at (3, 0) boarded into `carrier` at
/// (0, 0). Returns the world, the passenger and the carrier.
fn boarded(carrier: &str) -> (World, ObjectId, ObjectId) {
    let mut w = testkit::game_with(&rules(), SIZE, 7);
    testkit::run(&mut w, 2);
    let p = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, at(3, 0), 0);
    let t = testkit::spawn(&mut w, Kind::Vehicle, carrier, 0, at(0, 0), 0);
    testkit::run(&mut w, 2);
    transport::board(&mut w, p, t);
    assert_eq!(transport::riding(&w, p), Some(t), "set-up: boarded");
    (w, p, t)
}

// spec: S-0312 claim 34 (b) (a transport that is not open-topped does not move its passengers with itself: S-0507 claim 15 gives that only for open-topped transports)
#[test]
fn a_closed_transport_does_not_move_its_passengers() {
    let (mut w, p, t) = boarded("VAN");
    let before = w.obj(p).unwrap().pos;
    w.obj_mut(t).unwrap().pos = Pos::centre(at(-5, 2), 0);
    transport::sync(&mut w, t);
    assert_eq!(w.obj(p).unwrap().pos, before, "the passenger stays put");
}

// spec: S-0507 claim 15, Position (whenever an open-topped transport moves to a new position, its passengers' positions become the transport's position at once)
#[test]
fn an_open_topped_transport_moves_its_passengers() {
    let (mut w, p, t) = boarded("OPEN");
    let here = Pos::centre(at(-5, 2), 0);
    w.obj_mut(t).unwrap().pos = here;
    transport::sync(&mut w, t);
    assert_eq!(w.obj(p).unwrap().pos, here);
}
