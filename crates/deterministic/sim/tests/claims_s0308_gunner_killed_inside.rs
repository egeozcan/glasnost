//! S-0308 claim 6 with S-0312 revision 88 claim 34 (a) (S24's seam of
//! #383): a gunner vehicle whose last passenger is killed inside. Synthetic
//! rules: the test kit's combat rules plus GUN (an unarmed BUGGY of a gunner
//! type with 2 seats), SEAT (a WALKER whose mode index is 3) and warhead DK0
//! (infantry-death kind 0, versus 1). Every name and number is invented; no
//! game data.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, WarheadType};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::weapon;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{ObjectId, World};
use glasnost_sim::{testkit, transport};

const SIZE: i32 = 24;
/// SEAT's mode index.
const MODE: usize = 3;

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    register(&mut r, "warheads", "DK0");
    r.sim.warheads.insert(
        "DK0".into(),
        WarheadType {
            verses: Some(vec![Decimal("1".into()); 11]),
            inf_death: Some(0),
            ..Default::default()
        },
    );
    let mut gun = r.sim.vehicles["BUGGY"].clone();
    gun.primary = None;
    gun.passengers = Some(2);
    gun.size_limit = Some(Decimal("2".into()));
    gun.gunner = Some(true);
    register(&mut r, "vehicles", "GUN");
    r.sim.vehicles.insert("GUN".into(), gun);
    let mut seat = r.sim.infantry["WALKER"].clone();
    seat.ifv_mode = Some(MODE as i32);
    register(&mut r, "infantry", "SEAT");
    r.sim.infantry.insert("SEAT".into(), seat);
    r
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

/// A game with `n` SEATs of house 0 boarded into a GUN at (0, 0), the first
/// one first. Returns the world, the passengers in boarding order and the
/// vehicle.
fn boarded(n: i32) -> (World, Vec<ObjectId>, ObjectId) {
    let mut w = testkit::game_with(&rules(), SIZE, 7);
    testkit::run(&mut w, 2);
    let t = testkit::spawn(&mut w, Kind::Vehicle, "GUN", 0, at(0, 0), 0);
    let ps: Vec<ObjectId> = (0..n)
        .map(|i| testkit::spawn(&mut w, Kind::Infantry, "SEAT", 0, at(3, i), 0))
        .collect();
    testkit::run(&mut w, 2);
    for &p in &ps {
        transport::board(&mut w, p, t);
    }
    assert_eq!(weapon::weapon_number(&w, t), MODE, "set-up: the hand-over");
    (w, ps, t)
}

fn kill(w: &mut World, id: ObjectId) {
    let h = w
        .rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == "DK0")
        .unwrap() as u32;
    damage::apply(
        w,
        id,
        Hit {
            amount: 10_000,
            warhead: Some(h),
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    );
}

// spec: S-0308 claim 6 (when its last passenger leaves, a gunner vehicle's weapon number becomes 0) and S-0312 claim 34 (a) (a passenger a killing hit reaches inside leaves the transport at the hit)
#[test]
fn a_gunner_vehicle_whose_last_passenger_is_killed_inside_goes_back_to_weapon_number_0() {
    let (mut w, ps, t) = boarded(1);
    kill(&mut w, ps[0]);
    assert_eq!(transport::count(&w, t), 0, "set-up: the seat is free");
    assert_eq!(weapon::weapon_number(&w, t), 0);
}

// spec: S-0308 claim 6 (a passenger leaving while at least one remains changes nothing)
#[test]
fn a_gunner_vehicle_keeps_its_weapon_number_while_a_passenger_remains() {
    let (mut w, ps, t) = boarded(2);
    kill(&mut w, ps[0]);
    assert_eq!(transport::count(&w, t), 1, "set-up: one left");
    assert_eq!(weapon::weapon_number(&w, t), MODE);
}
