//! S-0518 claim 12: a charger that goes inside something leaves play and so
//! leaves every charger list at once (#340). Synthetic data: the
//! `linked_common` fixture (TOWER2 a building of a type that is not
//! overpowerable, CHARGER an infantry unit with an electric-charge
//! secondary, VICT a vehicle that is not open-topped).

#[path = "linked_common/mod.rs"]
mod linked_common;

use glasnost_sim::econ::power;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};
use linked_common::*;

fn sleeper(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    let id = spawn(w, kind, name, house, at(dx, dy));
    let tick = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    if let Some(o) = w.obj_mut(id) {
        o.beh.force(Behaviour::Sleep, tick, stale);
    }
    id
}

fn zap(w: &mut World, unit: ObjectId, building: ObjectId) {
    assert!(glasnost_sim::combat::fire::shot(
        w,
        unit,
        1,
        Target::Object(building)
    ));
    while !w.projectiles.is_empty() {
        testkit::run(w, 1);
    }
}

/// A building of a type that is not overpowerable with two chargers on its
/// list, and the two chargers.
fn charged() -> (World, ObjectId, Vec<ObjectId>) {
    let mut w = world(|_| {});
    let b = sleeper(&mut w, Kind::Building, "TOWER2", 0, 0, 0);
    let us: Vec<ObjectId> = (0..2)
        .map(|i| sleeper(&mut w, Kind::Infantry, "CHARGER", 0, 1 + i, 2))
        .collect();
    for &u in &us {
        zap(&mut w, u, b);
    }
    assert_eq!(power::chargers(&w, b), 2);
    (w, b, us)
}

// spec: S-0518 claim 12 and S-0203 claim 3 (a charger that boards a transport that is not open-topped leaves play, so it leaves the charger list at once)
#[test]
fn issue_340_a_charger_that_boards_a_closed_transport_leaves_the_list() {
    let (mut w, b, us) = charged();
    let car = sleeper(&mut w, Kind::Vehicle, "VICT", 0, 6, 6);
    assert!(!glasnost_sim::transport::open_topped(&w, car));
    for &u in &us {
        glasnost_sim::transport::board(&mut w, u, car);
        assert!(
            !w.update.contains(u),
            "out of the update order: out of play"
        );
        assert!(!w.obj(u).unwrap().on_map);
    }
    let at_once = power::chargers(&w, b);
    testkit::run(&mut w, 10);
    let later = power::chargers(&w, b);
    assert_eq!((at_once, later), (0, 0), "(at once, ten ticks later)");
}

// spec: S-0518 claim 12 (a charger leaves the list when it leaves play; going inside something is leaving play, S-0203 Terms)
#[test]
fn issue_340_a_charger_taken_inside_leaves_the_list() {
    let (mut w, b, us) = charged();
    for &u in &us {
        glasnost_sim::objects::take_inside(&mut w, u);
    }
    assert_eq!(power::chargers(&w, b), 0);
}
