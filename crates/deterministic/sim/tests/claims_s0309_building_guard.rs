//! S-0309 claim 8, F1 and F3 on synthetic data: a building's searches read
//! its type's guard range (sim issues S12, issue 247).

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{ArmourClass, Decimal, UnitType};
use glasnost_sim::combat::acquire::{self, SearchContext, SearchKind};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, World};

const SIZE: i32 = 30;

/// The combat test rules with the unarmed vehicle T0 and the guard range of
/// the synthetic TOWER (weapon range 7.25 cells) set to `guard_cells`.
fn rules(guard_cells: &str) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let mut t = UnitType {
        strength: Some(100),
        speed: Some(5),
        rot: Some(5),
        locomotion_class: Some(glasnost_rules::sim::LocomotionClass::Wheel),
        locomotor: Some(glasnost_rules::sim::Locomotor::Drive),
        ..Default::default()
    };
    t.combat.armor = Some(ArmourClass::Light);
    r.sim.vehicles.insert("T0".into(), t);
    r.registries
        .get_mut("vehicles")
        .expect("vehicles registry")
        .push("T0".into());
    r.sim.buildings.get_mut("TOWER").unwrap().econ.guard_range = Some(Decimal(guard_cells.into()));
    r
}

fn search(w: &mut World, a: ObjectId) -> Option<ObjectId> {
    acquire::search(w, a, SearchKind::Range, SearchContext::PerTick)
}

// spec: S-0309 claim 8, F1 and F3 (range search: R = g when g ≠ 0; a candidate passes when δ ≤ R) and Import mapping (GuardRange of a type section, a building's included)
#[test]
fn claim_8_building_guard_range_smaller_than_weapon_range() {
    let mut w = testkit::game_with(&rules("3"), SIZE, 11);
    let tower = testkit::spawn(
        &mut w,
        Kind::Building,
        "TOWER",
        0,
        testkit::mid(SIZE, 0, 0),
        0,
    );
    // 5 cells away: beyond R = 768 leptons, within the weapon's reach.
    testkit::spawn(&mut w, Kind::Vehicle, "T0", 1, testkit::mid(SIZE, 5, 0), 0);
    assert_eq!(
        search(&mut w, tower),
        None,
        "R = 768 leptons: an object 5 cells away is no candidate"
    );
}

// spec: S-0309 claim 8, F1 and F3; scenario A8 part h (a building with a guard range beyond its weapon range picks a target it cannot reach)
#[test]
fn claim_8_building_guard_range_larger_than_weapon_range() {
    let mut w = testkit::game_with(&rules("12"), SIZE, 11);
    let tower = testkit::spawn(
        &mut w,
        Kind::Building,
        "TOWER",
        0,
        testkit::mid(SIZE, 0, 0),
        0,
    );
    // 10 cells away: within R = 3072 leptons, beyond the weapon's reach.
    let b = testkit::spawn(&mut w, Kind::Vehicle, "T0", 1, testkit::mid(SIZE, 10, 0), 0);
    assert_eq!(
        search(&mut w, tower),
        Some(b),
        "R = 3072 leptons: an object 10 cells away is a candidate"
    );
}

// spec: S-0309 claim 8 and F1 (control: a vehicle with the same guard range at the same place picks the same object)
#[test]
fn claim_8_control_vehicle_guard_range_larger_than_weapon_range() {
    let mut r = rules("12");
    r.sim.vehicles.get_mut("CART").unwrap().guard_range = Some(Decimal("12".into()));
    let mut w = testkit::game_with(&r, SIZE, 11);
    let cart = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "CART",
        0,
        testkit::mid(SIZE, 0, 0),
        0,
    );
    let b = testkit::spawn(&mut w, Kind::Vehicle, "T0", 1, testkit::mid(SIZE, 10, 0), 0);
    assert_eq!(search(&mut w, cart), Some(b));
}
