//! The own AI and gunner vehicles (S-0308 claims 5 and 6; PLAN §4.7): an
//! infantry unit whose mode gives an empty gunner vehicle a healing weapon
//! is sent into it, one for every `PER_HEALER` armed vehicles.

use std::sync::Arc;

use glasnost_ai::brain::Brain;
use glasnost_ai::gunner::PER_HEALER;
use glasnost_ai::{fixture, game::Game};
use glasnost_rules::schema::Ruleset;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{combat::weapon, transport};

/// The fixture's rules with GUNR a switching gunner transport (slot 0 its
/// gun, slot 1 a healing weapon) and MEDI an unarmed infantry type of mode
/// `mode`.
fn ruleset(mode: i32) -> Ruleset {
    let mut r = fixture::ruleset();
    let mut heal = r.sim.weapons["Gun"].clone();
    heal.damage = Some(-50);
    r.sim.weapons.insert("Heal".into(), heal);
    let g = r.sim.vehicles.get_mut("GUNR").expect("GUNR");
    g.passengers = Some(1);
    g.size_limit = Some(glasnost_rules::sim::Decimal("2".into()));
    g.turret_count = Some(2);
    g.weapon_count = Some(2);
    g.combat.weapon_slots = Some(vec!["Gun".into(), "Heal".into()]);
    g.gunner = Some(true);
    let mut medi = r.sim.infantry["WALKER"].clone();
    medi.primary = None;
    medi.secondary = None;
    medi.ifv_mode = Some(mode);
    r.sim.infantry.insert("MEDI".into(), medi);
    r.registries
        .get_mut("infantry")
        .expect("infantry registry")
        .push("MEDI".into());
    r
}

/// House 0 of a skirmish under `rules` once its construction yard stands,
/// with `guns` GUNR and one MEDI spawned at home.
fn scene(rules: &Ruleset, guns: usize) -> (World, ObjectId, Vec<ObjectId>) {
    let map = fixture::map();
    let mut g = Game::new(
        Arc::new(SimRules::new(rules, "synthetic-gunner")),
        &map,
        "synthetic-gunner",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the skirmish loads");
    let yard = |w: &World| {
        w.objects.values().any(|o| {
            o.owner == Some(HouseId(0))
                && o.kind == Kind::Building
                && w.rules().buildings[o.type_index as usize].name == "YARD"
        })
    };
    while g.world.tick < 600 && !yard(&g.world) {
        g.step();
    }
    assert!(yard(&g.world), "the yard stands");
    let mut w = g.world.clone();
    let x = -fixture::START_DX;
    let guns: Vec<ObjectId> = (0..guns)
        .map(|k| {
            testkit::spawn(
                &mut w,
                Kind::Vehicle,
                "GUNR",
                0,
                fixture::at(x - 4 + 2 * k as i32, -5),
                0,
            )
        })
        .collect();
    let medi = testkit::spawn(&mut w, Kind::Infantry, "MEDI", 0, fixture::at(x + 4, 4), 0);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    (w, medi, guns)
}

fn think(w: &World) -> (Vec<Order>, u32) {
    let mut b = Brain::with_plan(w, &fixture::map(), HouseId(0), fixture::plan());
    let orders = b.think(w);
    (orders, b.special_uses().gunner_loads)
}

fn sent_into(orders: &[Order], unit: ObjectId) -> Option<ObjectId> {
    orders.iter().find_map(|o| match o {
        Order::Behaviour {
            actor,
            behaviour: Behaviour::Enter,
            target: Some(Target::Object(v)),
            ..
        } if *actor == unit => Some(*v),
        Order::Behaviour {
            actor,
            behaviour: Behaviour::Enter,
            dest: Some(Target::Object(v)),
            ..
        } if *actor == unit => Some(*v),
        _ => None,
    })
}

// S-0308 claims 5 and 6, PLAN §4.7: with PER_HEALER armed vehicles and an infantry unit whose mode gives the empty gunner vehicle a healing weapon, the unit is sent in; once aboard, the vehicle's weapon number is the mode.
#[test]
fn loads_a_healer_into_an_empty_gunner_vehicle() {
    let (mut w, medi, guns) = scene(&ruleset(1), PER_HEALER);
    let (orders, loads) = think(&w);
    let v = sent_into(&orders, medi).expect("MEDI is sent into a gunner vehicle");
    assert!(guns.contains(&v));
    assert_eq!(loads, 1);
    for o in orders
        .into_iter()
        .filter(|o| matches!(o, Order::Behaviour { actor, .. } if *actor == medi))
    {
        orders::issue(&mut w, HouseId(0), o);
    }
    for _ in 0..600 {
        testkit::run(&mut w, 1);
        if transport::count(&w, v) == 1 {
            break;
        }
    }
    assert_eq!(transport::count(&w, v), 1, "MEDI is aboard");
    assert_eq!(weapon::weapon_number(&w, v), 1);
}

// PLAN §4.7: a person keeps one healer for every PER_HEALER armed vehicles; with fewer, nobody is sent.
#[test]
fn no_healer_for_a_small_army() {
    let (w, medi, _) = scene(&ruleset(1), PER_HEALER - 1);
    let (orders, loads) = think(&w);
    assert_eq!(sent_into(&orders, medi), None);
    assert_eq!(loads, 0);
}

// S-0308 claim 6, PLAN §4.7: an infantry unit whose mode gives the vehicle its gun, not a healing weapon, is not sent.
#[test]
fn no_load_without_a_healing_mode() {
    let (w, medi, _) = scene(&ruleset(0), PER_HEALER);
    let (orders, loads) = think(&w);
    assert_eq!(sent_into(&orders, medi), None);
    assert_eq!(loads, 0);
}

// S-0308 claims 5 and 6, S-0509, PLAN §4.7: when the healer is an engineer and a capturable enemy building is in sight, the engineer still goes into the gunner vehicle (a person who has picked it for the vehicle does not send it to capture); the M7b lane found every such engineer sent to capture.
#[test]
fn keeps_an_engineer_healer_from_capturing() {
    let mut r = ruleset(1);
    r.sim
        .infantry
        .get_mut("MEDI")
        .expect("MEDI")
        .engineer = Some(true);
    r.registries
        .get_mut("buildings")
        .expect("buildings registry")
        .push("CAPT".into());
    r.sim.buildings.insert(
        "CAPT".into(),
        glasnost_rules::sim::BuildingType {
            strength: Some(400),
            foundation: Some([2, 2]),
            capturable: Some(true),
            tech_level: Some(99),
            ..Default::default()
        },
    );
    let (mut w, medi, guns) = scene(&r, PER_HEALER);
    let at = fixture::at(-fixture::START_DX + 7, 7);
    testkit::spawn(&mut w, Kind::Building, "CAPT", 1, at, 0);
    let p = glasnost_sim::world::Pos::centre(at, 0);
    glasnost_sim::econ::shroud::area_reveal(&mut w, HouseId(0), p, 4);
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    let mut b = Brain::with_plan(&w, &fixture::map(), HouseId(0), fixture::plan());
    let orders = b.think(&w);
    let v = sent_into(&orders, medi).expect("the engineer is sent into a gunner vehicle");
    assert!(guns.contains(&v));
    assert_eq!(b.special_uses().gunner_loads, 1);
    assert_eq!(b.special_uses().engineers, 0);
}
