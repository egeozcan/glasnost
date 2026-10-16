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

/// The fixture's rules with GARRI (an occupier rifleman), ENGI (an
/// engineer the barracks builds), HUT (a 2 × 2 garrisonable building, tech
/// level 99 so it is never built), CAPT (a 2 × 2 capturable building) and
/// PLAIN (a 1 × 1 building that is not garrisonable).
fn rules() -> Arc<SimRules> {
    let mut r = fixture::ruleset();
    r.registries
        .entry("infantry".into())
        .or_default()
        .push("GARRI".into());
    r.registries
        .entry("infantry".into())
        .or_default()
        .push("ENGI".into());
    r.registries.entry("buildings".into()).or_default().extend([
        "HUT".to_string(),
        "PLAIN".to_string(),
        "CAPT".to_string(),
    ]);
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
    r.sim.infantry.insert(
        "ENGI".into(),
        UnitType {
            strength: Some(100),
            speed: Some(6),
            locomotion_class: Some(LocomotionClass::Foot),
            locomotor: Some(Locomotor::Walk),
            sight: Some(4),
            engineer: Some(true),
            tech_level: Some(1),
            cost: Some(100),
            owner: Some(vec!["Alphaland".into(), "Betaland".into()]),
            ..Default::default()
        },
    );
    r.sim.buildings.insert(
        "CAPT".into(),
        BuildingType {
            strength: Some(400),
            foundation: Some([2, 2]),
            capturable: Some(true),
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

/// Reveals the area around `c` to house `h`.
fn reveal(w: &mut World, h: u8, c: glasnost_map::CellXY) {
    let p = glasnost_sim::world::Pos::centre(c, 0);
    glasnost_sim::econ::shroud::area_reveal(w, HouseId(h), p, 4);
}

/// A world copy with an idle engineer of house 0 and a capturable enemy
/// building of house 1 near it; also returns the objects.
fn engineer_scene(g: &Game) -> (World, ObjectId, ObjectId) {
    let mut w = g.world.clone();
    let home = w
        .objects
        .values()
        .find(|o| o.owner == Some(HouseId(0)) && o.on_map)
        .map(|o| o.pos.cell())
        .expect("an object of house 0");
    let target = testkit::spawn(
        &mut w,
        Kind::Building,
        "CAPT",
        1,
        glasnost_map::CellXY::new(home.x + 7, home.y + 7),
        0,
    );
    let unit = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "ENGI",
        0,
        glasnost_map::CellXY::new(home.x + 3, home.y + 3),
        0,
    );
    // A reveal of the area (as a map trigger gives) shows the building to the house; the next think tick follows.
    reveal(&mut w, 0, glasnost_map::CellXY::new(home.x + 7, home.y + 7));
    testkit::run(&mut w, 2);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    (w, target, unit)
}

// S-0509, PLAN §4.7: an idle engineer is sent to capture an enemy building it can see, with the click a person gives (S-0214 row I7), and it captures it.
#[test]
fn sends_an_idle_engineer_to_capture_a_seen_enemy_building() {
    let mut g = game(5);
    ready(&mut g);
    let (mut w, target, unit) = engineer_scene(&g);
    assert!(glasnost_sim::econ::shroud::explored(
        &w,
        HouseId(0),
        w.obj(target).unwrap().pos.cell()
    ));
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = brain.think(&w);
    assert!(garrison_orders(&orders, unit, target), "{orders:?}");
    assert_eq!(brain.special_uses().engineers, 1);
    for o in orders {
        glasnost_sim::orders::issue(&mut w, HouseId(0), o);
    }
    for _ in 0..800 {
        testkit::run(&mut w, 1);
        if w.obj(unit).is_none() {
            break;
        }
    }
    assert_eq!(w.obj(target).unwrap().owner, Some(HouseId(0)), "captured");
}

// S-0509, PLAN §4.7: two engineers are not sent at one building, and an engineer is not sent at a building on a cell the house has not explored.
#[test]
fn one_engineer_per_building_and_only_what_it_has_explored() {
    let mut g = game(5);
    ready(&mut g);
    let (mut w, target, unit) = engineer_scene(&g);
    let home = w.obj(unit).unwrap().pos.cell();
    let second = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "ENGI",
        0,
        glasnost_map::CellXY::new(home.x, home.y + 1),
        0,
    );
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = brain.think(&w);
    let sent = [unit, second]
        .iter()
        .filter(|&&e| garrison_orders(&orders, e, target))
        .count();
    assert_eq!(sent, 1, "{orders:?}");
    // Move the target to the far corner, unexplored.
    let far = glasnost_map::CellXY::new(2, 2);
    assert!(!glasnost_sim::econ::shroud::explored(&w, HouseId(0), far));
    glasnost_sim::objects::remove(&mut w, target);
    let far_target = testkit::spawn(&mut w, Kind::Building, "CAPT", 1, far, 0);
    let orders = Brain::new(&w, &fixture::map(), HouseId(0)).think(&w);
    assert!(!garrison_orders(&orders, unit, far_target), "{orders:?}");
    assert!(!garrison_orders(&orders, second, far_target));
}

// S-0509, PLAN §4.7: with no engineer, a barracks and a seen capturable enemy building, the house orders an engineer from its sidebar once, and does not order a second while one is in its line.
#[test]
fn orders_an_engineer_when_it_has_none() {
    let mut g = game(5);
    ready(&mut g);
    let mut w = g.world.clone();
    let home = w
        .objects
        .values()
        .find(|o| o.owner == Some(HouseId(0)) && o.on_map)
        .map(|o| o.pos.cell())
        .unwrap();
    testkit::spawn(
        &mut w,
        Kind::Building,
        "PILL",
        0,
        glasnost_map::CellXY::new(home.x + 5, home.y),
        0,
    );
    testkit::spawn(
        &mut w,
        Kind::Building,
        "CAPT",
        1,
        glasnost_map::CellXY::new(home.x + 8, home.y + 8),
        0,
    );
    reveal(&mut w, 0, glasnost_map::CellXY::new(home.x + 8, home.y + 8));
    w.houses[0].credits = 5000;
    testkit::run(&mut w, 20);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    let engi = w.rules().index_of(Kind::Infantry, "ENGI").unwrap();
    let want = |o: &Order| matches!(o, Order::Build { what } if what.kind == Kind::Infantry && what.index == engi);
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = brain.think(&w);
    assert!(orders.iter().any(want), "{orders:?}");
    for o in orders {
        glasnost_sim::orders::issue(&mut w, HouseId(0), o);
    }
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    let again = brain.think(&w);
    assert!(
        !again.iter().any(want),
        "one in the line already: {again:?}"
    );
}
