//! A spawn count or slave count from the rules is bounded by our engine
//! (#344): one rules key cannot size an allocation and a creation loop
//! without bound. Invented types on the testkit's combat ruleset: aircraft
//! DRN (plain), infantry SLV (plain), vehicles BIGS (spawns DRN with a huge
//! count) and BIGV (enslaves SLV with a huge count). No game data.

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
    let s = &mut r.sim;
    s.aircraft.insert(
        "DRN".into(),
        UnitType {
            strength: Some(50),
            speed: Some(10),
            ..Default::default()
        },
    );
    s.infantry.insert("SLV".into(), walker);
    let mut bigs = cart.clone();
    bigs.combat.spawns = Some("DRN".into());
    bigs.combat.spawns_number = Some(HUGE);
    s.vehicles.insert("BIGS".into(), bigs);
    let mut bigv = cart;
    bigv.econ.enslaves = Some("SLV".into());
    bigv.econ.slaves_number = Some(HUGE);
    s.vehicles.insert("BIGV".into(), bigv);
    r
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
