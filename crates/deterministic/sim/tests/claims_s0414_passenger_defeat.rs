//! S-0414 claims 5, 15 and 21 and S-0605 claim 15 (#383): a passenger that
//! its own house's defeat hits inside a transport or an absorbing building
//! leaves its carrier at once and dies once. Synthetic rules: the test kit's
//! combat ruleset plus BIO (an infantry absorber), NH (WALKER marked not
//! human, so its death asks for a die sequence), VAN (an unarmed two-seat
//! BUGGY) and CTRL (a BUGGY with a takeover weapon). Every name and number
//! is invented.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Decimal, WarheadType, WeaponType};
use glasnost_sim::econ::{stats, victory};
use glasnost_sim::mind;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::absorb;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{testkit, tick, transport};

const SIZE: i32 = 24;

fn dec(s: &str) -> Decimal {
    Decimal(s.into())
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let mut bio = BuildingType {
        strength: Some(1000),
        foundation: Some([2, 2]),
        infantry_absorb: Some(true),
        passengers: Some(3),
        size_limit: Some(dec("1")),
        ..Default::default()
    };
    bio.econ.power = Some(10);
    bio.econ.extra_power = Some(25);
    register(&mut r, "buildings", "BIO");
    r.sim.buildings.insert("BIO".into(), bio);
    let mut nh = r.sim.infantry["WALKER"].clone();
    nh.not_human = Some(true);
    register(&mut r, "infantry", "NH");
    r.sim.infantry.insert("NH".into(), nh);
    let mut van = r.sim.vehicles["BUGGY"].clone();
    van.primary = None;
    van.passengers = Some(2);
    van.size_limit = Some(dec("2"));
    register(&mut r, "vehicles", "VAN");
    r.sim.vehicles.insert("VAN".into(), van);
    r.sim.warheads.insert(
        "MIND".into(),
        WarheadType {
            verses: Some(vec![dec("1"); 11]),
            mind_control: Some(true),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "Ray".into(),
        WeaponType {
            damage: Some(3),
            rof: Some(30),
            range: Some(dec("10")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("MIND".into()),
            ..Default::default()
        },
    );
    let mut ctrl = r.sim.vehicles["BUGGY"].clone();
    ctrl.primary = Some("Ray".into());
    register(&mut r, "vehicles", "CTRL");
    r.sim.vehicles.insert("CTRL".into(), ctrl);
    r
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, at(dx, dy), 0)
}

/// House 1's infantry of type `name`, taken by house 0's controller, sent
/// into house 0's absorber (it goes in as house 1's again, S-0605 claim
/// 7); then house 1 is defeated.
fn absorbed_then_defeated(name: &str) -> (World, ObjectId, ObjectId) {
    let mut w = testkit::game_with(&rules(), SIZE, 7);
    testkit::run(&mut w, 2);
    let b = put(&mut w, Kind::Building, "BIO", 0, 0, 0);
    testkit::run(&mut w, 3);
    let ctrl = put(&mut w, Kind::Vehicle, "CTRL", 0, -5, 0);
    let foe = put(&mut w, Kind::Infantry, name, 1, 4, 0);
    assert!(mind::take(&mut w, ctrl, foe));
    orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: foe,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: None,
            dest: Some(Target::Object(b)),
        },
    );
    for _ in 0..600 {
        tick::step(&mut w);
        if absorb::absorbed_in(&w, foe) == Some(b) {
            break;
        }
    }
    assert_eq!(absorb::absorbed_in(&w, foe), Some(b), "went in");
    assert_eq!(w.obj(foe).unwrap().owner, Some(HouseId(1)));
    assert_eq!(absorb::count(&w, b), 1);
    victory::defeat(&mut w, HouseId(1));
    (w, b, foe)
}

// spec: S-0605 claim 15 (control: a passenger that leaves the game inside drops the building's count at once)
#[test]
fn control_human_passenger_is_removed_by_its_house_defeat() {
    let (w, b, foe) = absorbed_then_defeated("WALKER");
    assert!(w.obj(foe).is_none());
    assert_eq!(absorb::count(&w, b), 0);
}

// spec: S-0605 claim 15 and scenario A11 (the passenger is deleted and the count drops when defeat removes it)
#[test]
fn not_human_passenger_is_removed_by_its_house_defeat() {
    let (w, b, foe) = absorbed_then_defeated("NH");
    assert!(
        w.obj(foe).is_none(),
        "the passenger leaves the game at its defeat hit"
    );
    assert_eq!(absorb::count(&w, b), 0, "the count drops at once");
}

// spec: S-0414 claim 15 and scenario V11 part b (a passenger hit at its own place before its carrier receives no second death) and claim 21 (one loss per destruction)
#[test]
fn passenger_created_before_its_transport_dies_once() {
    let mut w = testkit::game_with(&rules(), SIZE, 7);
    testkit::run(&mut w, 2);
    let p = put(&mut w, Kind::Infantry, "NH", 0, 3, 0);
    let t = put(&mut w, Kind::Vehicle, "VAN", 0, 0, 0);
    testkit::run(&mut w, 3);
    transport::board(&mut w, p, t);
    assert_eq!(transport::riding(&w, p), Some(t));
    let before = stats::of(&w, HouseId(0)).losses;
    let mine = w
        .objects
        .values()
        .filter(|o| o.owner == Some(HouseId(0)))
        .count();
    assert_eq!(mine, 2, "the house owns the passenger and the transport only");
    w.effects.clear();
    victory::defeat(&mut w, HouseId(0));
    assert!(w.obj(p).is_none() && w.obj(t).is_none());
    assert_eq!(
        stats::of(&w, HouseId(0)).losses - before,
        2,
        "two objects destroyed, two losses"
    );
    let deaths = w
        .effects
        .iter()
        .filter(|e| matches!(e, glasnost_sim::combat::effects::Effect::Killed { id, .. } if *id == p))
        .count();
    assert_eq!(deaths, 1, "the passenger dies once");
}
