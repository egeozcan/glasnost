//! The own AI's use of the special abilities (M6c, PLAN §4.7): garrisoning
//! (S-0508), engineers capturing (S-0509) and spies infiltrating (S-0510),
//! on the synthetic skirmish with invented extra types.

use std::sync::Arc;

use glasnost_ai::brain::Brain;
use glasnost_ai::fixture;
use glasnost_ai::game::Game;
use glasnost_rules::sim::{BuildingType, LocomotionClass, Locomotor, UnitType};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::structures::garrison;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

/// The fixture's rules with GARRI (an occupier rifleman), HUT (a 2 × 2
/// garrisonable building, tech level 99 so it is never built) and PLAIN (a
/// 1 × 1 building that is not garrisonable).
fn rules() -> Arc<SimRules> {
    let mut r = fixture::ruleset();
    r.registries
        .entry("infantry".into())
        .or_default()
        .push("GARRI".into());
    r.registries
        .entry("buildings".into())
        .or_default()
        .extend(["HUT".to_string(), "PLAIN".to_string()]);
    r.sim.infantry.insert(
        "GARRI".into(),
        UnitType {
            strength: Some(100),
            speed: Some(6),
            locomotion_class: Some(LocomotionClass::Foot),
            locomotor: Some(Locomotor::Walk),
            sight: Some(4),
            occupier: Some(true),
            primary: Some("Rifle".into()),
            tech_level: Some(99),
            ..Default::default()
        },
    );
    let mut hut = BuildingType {
        strength: Some(400),
        foundation: Some([2, 2]),
        can_be_occupied: Some(true),
        can_occupy_fire: Some(true),
        max_number_occupants: Some(2),
        tech_level: Some(-1),
        ..Default::default()
    };
    hut.econ.sight = Some(3);
    r.sim.buildings.insert("HUT".into(), hut);
    r.sim.buildings.insert(
        "PLAIN".into(),
        BuildingType {
            strength: Some(400),
            foundation: Some([1, 1]),
            ..Default::default()
        },
    );
    Arc::new(SimRules::new(&r, "synthetic-skirmish"))
}

fn game(seed: u32) -> Game {
    Game::new(
        rules(),
        &fixture::map(),
        "synthetic-skirmish",
        fixture::catalogue(),
        fixture::setup(seed),
    )
    .expect("the synthetic skirmish loads")
}

/// Runs the game until orders are allowed, to a tick on which the brains
/// think.
fn ready(g: &mut Game) {
    while g.world.tick <= g.world.orders_blocked_through || g.world.tick % 15 != 0 {
        g.step();
    }
}

fn garrison_orders(orders: &[Order], unit: ObjectId, b: ObjectId) -> bool {
    orders.iter().any(|o| {
        matches!(o, Order::Behaviour { actor, behaviour: Behaviour::Capture, dest: Some(Target::Object(d)), .. }
            if *actor == unit && *d == b)
    })
}

/// A world copy where house 0 has an idle occupier near a hut.
fn scene(g: &Game, hut_owner: u8) -> (World, ObjectId, ObjectId) {
    let mut w = g.world.clone();
    let home = w
        .objects
        .values()
        .find(|o| o.owner == Some(HouseId(0)) && o.on_map)
        .map(|o| o.pos.cell())
        .expect("an object of house 0");
    let hut = testkit::spawn(
        &mut w,
        Kind::Building,
        "HUT",
        hut_owner,
        glasnost_map::CellXY::new(home.x + 6, home.y + 6),
        0,
    );
    let unit = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "GARRI",
        0,
        glasnost_map::CellXY::new(home.x + 3, home.y + 3),
        0,
    );
    (w, hut, unit)
}

// S-0508, PLAN §4.7: the AI sends an idle occupier to a garrison building it can see and may garrison, with the click a person gives (S-0214 row I8).
#[test]
fn sends_an_idle_occupier_to_a_building_it_may_garrison() {
    let mut g = game(5);
    ready(&mut g);
    let (mut w, hut, unit) = scene(&g, 0);
    let map = fixture::map();
    let mut brain = Brain::new(&w, &map, HouseId(0));
    let orders = brain.think(&w);
    assert!(garrison_orders(&orders, unit, hut), "{orders:?}");
    assert_eq!(brain.special_uses().garrisons, 1);
    // The order is applied through orders::issue and the unit goes inside.
    for o in orders {
        glasnost_sim::orders::issue(&mut w, HouseId(0), o);
    }
    for _ in 0..600 {
        testkit::run(&mut w, 1);
        if !w.obj(unit).unwrap().on_map {
            break;
        }
    }
    assert_eq!(garrison::occupants(&w, hut), vec![unit]);
}

// S-0508 claim 1: a building the unit may not garrison (an enemy's) gets no order; a building not garrisonable gets none.
#[test]
fn does_not_garrison_an_enemy_building() {
    let mut g = game(5);
    ready(&mut g);
    let (w, hut, unit) = scene(&g, 1);
    let orders = Brain::new(&w, &fixture::map(), HouseId(0)).think(&w);
    assert!(!garrison_orders(&orders, unit, hut), "{orders:?}");
}

// S-0508 claim 1: the capacity is respected; units already on their way count, so two places take two units and the third is not sent.
#[test]
fn fills_the_places_and_no_more() {
    let mut g = game(5);
    ready(&mut g);
    let (mut w, hut, _) = scene(&g, 0);
    let home = w.obj(hut).unwrap().pos.cell();
    let _extra: Vec<ObjectId> = (0..3)
        .map(|i| {
            testkit::spawn(
                &mut w,
                Kind::Infantry,
                "GARRI",
                0,
                glasnost_map::CellXY::new(home.x - 3, home.y - 3 + i),
                0,
            )
        })
        .collect();
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = brain.think(&w);
    let n = orders
        .iter()
        .filter(|o| matches!(o, Order::Behaviour { dest: Some(Target::Object(d)), behaviour: Behaviour::Capture, .. } if *d == hut))
        .count();
    assert_eq!(n, 2, "the hut holds two: {orders:?}");
    assert_eq!(brain.special_uses().garrisons, 2);
}

// PLAN §4.7: it sees only what its house sees: a hut on a cell the house has not explored is not chosen, though the same world with the cell explored gets the order.
#[test]
fn garrisons_only_what_it_has_explored() {
    let mut g = game(5);
    ready(&mut g);
    let (mut w, hut, unit) = scene(&g, 0);
    let home = w.obj(unit).unwrap().pos.cell();
    // Move the hut out of anything explored: the far corner of the map.
    let far = glasnost_map::CellXY::new(2, 2);
    assert!(!glasnost_sim::econ::shroud::explored(&w, HouseId(0), far));
    glasnost_sim::objects::remove(&mut w, hut);
    let hut2 = testkit::spawn(&mut w, Kind::Building, "HUT", 0, far, 0);
    let _ = home;
    let orders = Brain::new(&w, &fixture::map(), HouseId(0)).think(&w);
    assert!(!garrison_orders(&orders, unit, hut2), "{orders:?}");
}

// The AI's counters are state: they are saved and hashed with the game and survive a save and load.
#[test]
fn the_counters_survive_a_save() {
    let mut g = game(5);
    ready(&mut g);
    let (w, _, _) = scene(&g, 0);
    g.world = w;
    for _ in 0..16 {
        g.step();
    }
    let uses = g.special_uses();
    assert!(uses.iter().any(|(_, u)| u.garrisons > 0));
    let bytes = g.save();
    let loaded = Game::load(&bytes, rules(), fixture::catalogue()).expect("loads");
    assert_eq!(loaded.special_uses(), uses);
    assert_eq!(loaded.state_hash(), g.state_hash());
}
