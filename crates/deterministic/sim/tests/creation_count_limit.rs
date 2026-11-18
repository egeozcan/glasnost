//! A spawn count or slave count from the rules is bounded by our engine
//! (#344): one rules key cannot size an allocation and a creation loop
//! without bound. Invented types on the testkit's combat ruleset: aircraft
//! DRN (plain), NDRN (spawns 100 DRN) and SDRN (enslaves 100 SLV), infantry
//! SLV (plain), vehicles BIGS (spawns DRN with a huge count), BIGV
//! (enslaves SLV with a huge count), NEST (spawns 100 NDRN) and NESTV
//! (spawns 100 SDRN). No game data.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::UnitType;
use glasnost_sim::objects;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, World};

/// Far above any count game data uses (small enough that an unbounded
/// engine still finishes the creation).
const HUGE: i32 = 5000;

/// Our engine's limit (`spawn::MAX_CREATED_COUNT`, #344).
const LIMIT: usize = 100;

fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let push = |r: &mut Ruleset, reg: &str, n: &str| {
        r.registries.entry(reg.into()).or_default().push(n.into());
    };
    push(&mut r, "aircraft", "DRN");
    push(&mut r, "infantry", "SLV");
    push(&mut r, "vehicles", "BIGS");
    push(&mut r, "vehicles", "BIGV");
    let walker = r.sim.infantry["WALKER"].clone();
    let cart = r.sim.vehicles["CART"].clone();
    push(&mut r, "aircraft", "NDRN");
    push(&mut r, "aircraft", "SDRN");
    push(&mut r, "vehicles", "NEST");
    push(&mut r, "vehicles", "NESTV");
    let s = &mut r.sim;
    let drone = UnitType {
        strength: Some(50),
        speed: Some(10),
        ..Default::default()
    };
    s.aircraft.insert("DRN".into(), drone.clone());
    // A slot type that is itself a spawner of LIMIT drones.
    let mut ndrn = drone.clone();
    ndrn.combat.spawns = Some("DRN".into());
    ndrn.combat.spawns_number = Some(LIMIT as i32);
    s.aircraft.insert("NDRN".into(), ndrn);
    // A slot type that is a slave master of LIMIT slaves.
    let mut sdrn = drone;
    sdrn.econ.enslaves = Some("SLV".into());
    sdrn.econ.slaves_number = Some(LIMIT as i32);
    s.aircraft.insert("SDRN".into(), sdrn);
    s.infantry.insert("SLV".into(), walker);
    let mut bigs = cart.clone();
    bigs.combat.spawns = Some("DRN".into());
    bigs.combat.spawns_number = Some(HUGE);
    s.vehicles.insert("BIGS".into(), bigs);
    let mut bigv = cart.clone();
    bigv.econ.enslaves = Some("SLV".into());
    bigv.econ.slaves_number = Some(HUGE);
    s.vehicles.insert("BIGV".into(), bigv);
    let mut nest = cart.clone();
    nest.combat.spawns = Some("NDRN".into());
    nest.combat.spawns_number = Some(LIMIT as i32);
    s.vehicles.insert("NEST".into(), nest);
    let mut nestv = cart;
    nestv.combat.spawns = Some("SDRN".into());
    nestv.combat.spawns_number = Some(LIMIT as i32);
    s.vehicles.insert("NESTV".into(), nestv);
    r
}

/// Our engine's limit on one creation set (`objects::MAX_CREATION_SET`,
/// #344).
const SET_LIMIT: usize = 1000;

/// The slot objects of a spawner, each slot's object in slot order.
fn slot_objects(w: &World, id: ObjectId) -> Vec<Option<ObjectId>> {
    w.obj(id)
        .and_then(|o| o.spawn.as_ref())
        .map(|c| c.slots.iter().map(|s| s.object).collect())
        .unwrap_or_default()
}

// design: #344 (a creation set holds at most MAX_CREATION_SET members, the root included; once it is full a spawner keeps only the slots it filled; the importer warns for a type whose full creation set is above the limit)
#[test]
fn issue_344_nested_spawners_do_not_multiply() {
    let mut w = testkit::game_with(&rules(), 30, 7);
    let warnings = w.rules().missiles.warnings.clone();
    assert!(
        warnings.iter().any(|x| x.contains("creation set")),
        "{warnings:?}"
    );
    let (id, made) = create(&mut w, "NEST");
    // 1 + 100 × (1 + 100) = 10101 objects without the set limit.
    assert_eq!(made, SET_LIMIT, "the creation set stops at the limit");
    let slots = slot_objects(&w, id);
    // The root, nine full slot objects of 101 each (910), then the tenth
    // slot object (911) and 89 of its drones (1000).
    assert_eq!(slots.len(), 10, "the root keeps only the slots it filled");
    assert!(slots.iter().all(Option::is_some), "no empty slot");
    let last = slots[9].unwrap();
    let inner = slot_objects(&w, last);
    assert_eq!(inner.len(), 89, "the last slot object stops at the limit");
    assert!(inner.iter().all(Option::is_some), "no empty inner slot");
}

// design: #344 (once the creation set is full a slave master keeps only the slaves it made)
#[test]
fn issue_344_slave_masters_in_slots_do_not_multiply() {
    let mut w = testkit::game_with(&rules(), 30, 7);
    let (id, made) = create(&mut w, "NESTV");
    assert_eq!(made, SET_LIMIT, "the creation set stops at the limit");
    let slots = slot_objects(&w, id);
    assert_eq!(slots.len(), 10, "the root keeps only the slots it filled");
    assert!(slots.iter().all(Option::is_some), "no empty slot");
    let last = slots[9].unwrap();
    let slaves = glasnost_sim::econ::slaves::slaves_of(&w, last).len();
    assert_eq!(slaves, 89, "the last master stops at the limit");
}

fn create(w: &mut World, name: &str) -> (ObjectId, usize) {
    let ti = w.rules().index_of(Kind::Vehicle, name).unwrap();
    let before = w.objects.len();
    let id = objects::create(w, Kind::Vehicle, ti, Some(HouseId(0)));
    (id, w.objects.len() - before)
}

// design: #344 (a spawn count above our engine's limit creates the limit's number of slots and slot objects; the importer warns)
#[test]
fn issue_344_a_huge_spawn_count_is_bounded() {
    let mut w = testkit::game_with(&rules(), 30, 7);
    let warnings = w.rules().missiles.warnings.clone();
    assert!(
        warnings.iter().any(|x| x.contains("spawn count of 5000")),
        "{warnings:?}"
    );
    let (id, made) = create(&mut w, "BIGS");
    let slots = w.obj(id).unwrap().spawn.as_ref().unwrap().slots.len();
    assert_eq!(slots, LIMIT);
    assert_eq!(made, 1 + slots, "the spawner and one object per slot");
}

// design: #344 (a slave count above our engine's limit creates the limit's number of slaves; the importer warns)
#[test]
fn issue_344_a_huge_slave_count_is_bounded() {
    let mut w = testkit::game_with(&rules(), 30, 7);
    let warnings = w.rules().missiles.warnings.clone();
    assert!(
        warnings.iter().any(|x| x.contains("slave count of 5000")),
        "{warnings:?}"
    );
    let (id, made) = create(&mut w, "BIGV");
    let slaves = glasnost_sim::econ::slaves::slaves_of(&w, id).len();
    assert_eq!(slaves, LIMIT);
    assert_eq!(made, 1 + slaves, "the master and its slaves");
}
