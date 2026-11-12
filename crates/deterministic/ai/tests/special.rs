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
    r.registries
        .entry("infantry".into())
        .or_default()
        .push("SPYU".into());
    r.registries.entry("buildings".into()).or_default().extend([
        "HUT".to_string(),
        "PLAIN".to_string(),
        "CAPT".to_string(),
        "SPYB".to_string(),
        "DERR".to_string(),
        "LAB".to_string(),
    ]);
    r.registries
        .entry("countries".into())
        .or_default()
        .push("Neutral".to_string());
    r.sim.countries.insert(
        "Neutral".into(),
        glasnost_rules::sim::Country {
            multiplay_passive: Some(true),
            ..Default::default()
        },
    );
    // S-0515: a tech building that pays (DERR) and one that only serves (LAB).
    for (n, cash) in [("DERR", true), ("LAB", false)] {
        r.sim.buildings.insert(
            n.into(),
            BuildingType {
                strength: Some(400),
                foundation: Some([1, 1]),
                capturable: Some(true),
                needs_engineer: Some(true),
                tech_level: Some(99),
                produce_cash_startup: cash.then_some(300),
                produce_cash_amount: cash.then_some(25),
                produce_cash_delay: cash.then_some(21),
                ..Default::default()
            },
        );
    }
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
    r.sim.infantry.insert(
        "SPYU".into(),
        UnitType {
            strength: Some(100),
            speed: Some(6),
            locomotion_class: Some(LocomotionClass::Foot),
            locomotor: Some(Locomotor::Walk),
            sight: Some(4),
            agent: Some(true),
            infiltrate: Some(true),
            tech_level: Some(1),
            cost: Some(100),
            owner: Some(vec!["Alphaland".into(), "Betaland".into()]),
            ..Default::default()
        },
    );
    let mut spyb = BuildingType {
        strength: Some(400),
        foundation: Some([2, 2]),
        spyable: Some(true),
        tech_level: Some(99),
        ..Default::default()
    };
    spyb.econ.power = Some(100);
    r.sim.buildings.insert("SPYB".into(), spyb);
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

// S-0510, PLAN §4.7: an idle spy is sent into a seen enemy building whose infiltration has an effect and that it may enter (S-0214 row I24), and the infiltration happens: the enemy house suffers a blackout.
#[test]
fn sends_an_idle_spy_into_a_seen_enemy_building() {
    let mut g = game(5);
    ready(&mut g);
    let mut w = g.world.clone();
    let home = w
        .objects
        .values()
        .find(|o| o.owner == Some(HouseId(0)) && o.on_map)
        .map(|o| o.pos.cell())
        .unwrap();
    let target = testkit::spawn(
        &mut w,
        Kind::Building,
        "SPYB",
        1,
        glasnost_map::CellXY::new(home.x + 7, home.y + 7),
        0,
    );
    let unit = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "SPYU",
        0,
        glasnost_map::CellXY::new(home.x + 5, home.y + 5),
        0,
    );
    reveal(&mut w, 0, glasnost_map::CellXY::new(home.x + 7, home.y + 7));
    testkit::run(&mut w, 2);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = brain.think(&w);
    assert!(garrison_orders(&orders, unit, target), "{orders:?}");
    assert_eq!(brain.special_uses().spies, 1);
    for o in orders {
        glasnost_sim::orders::issue(&mut w, HouseId(0), o);
    }
    for _ in 0..800 {
        testkit::run(&mut w, 1);
        if w.obj(unit).is_none() {
            break;
        }
    }
    assert!(w.obj(unit).is_none(), "the spy was used up");
    assert_eq!(
        w.obj(target).unwrap().owner,
        Some(HouseId(1)),
        "not captured"
    );
}

// S-0510, PLAN §4.7: with no spy, a barracks and a seen enemy building worth infiltrating, the house orders a spy once; and nothing is ordered for a building on a cell it has not explored.
#[test]
fn orders_a_spy_when_it_has_none() {
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
        "SPYB",
        1,
        glasnost_map::CellXY::new(home.x + 8, home.y + 8),
        0,
    );
    w.houses[0].credits = 5000;
    let spy = w.rules().index_of(Kind::Infantry, "SPYU").unwrap();
    let want = |o: &Order| matches!(o, Order::Build { what } if what.kind == Kind::Infantry && what.index == spy);
    testkit::run(&mut w, 20);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    // Unexplored: no spy is ordered.
    let none = Brain::new(&w, &fixture::map(), HouseId(0)).think(&w);
    assert!(!none.iter().any(want), "{none:?}");
    reveal(&mut w, 0, glasnost_map::CellXY::new(home.x + 8, home.y + 8));
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = brain.think(&w);
    assert!(orders.iter().any(want), "{orders:?}");
}

/// The game with a passive Neutral house (house 2) next to the two players.
fn game_with_neutral(seed: u32) -> Game {
    let mut setup = fixture::setup(seed);
    setup.houses.push(glasnost_sim::world::HouseSetup {
        name: "Neutral".into(),
        country: "Neutral".into(),
        human: false,
        start: None,
        role: glasnost_sim::world::HouseRole::Neutral,
        ai: false,
    });
    Game::new(
        rules(),
        &fixture::map(),
        "synthetic-skirmish",
        fixture::catalogue(),
        setup,
    )
    .expect("the synthetic skirmish loads")
}

/// A world where house 0 has an idle engineer and a service-only tech
/// building (LAB, nearer) and a paying one (DERR, farther) of the Neutral
/// house in sight.
fn tech_scene(g: &Game) -> (World, ObjectId, ObjectId, ObjectId) {
    let mut w = g.world.clone();
    let home = w
        .objects
        .values()
        .find(|o| o.owner == Some(HouseId(0)) && o.on_map)
        .map(|o| o.pos.cell())
        .expect("an object of house 0");
    let neutral = w
        .houses
        .iter()
        .position(|h| h.role == glasnost_sim::world::HouseRole::Neutral)
        .expect("a neutral house") as u8;
    let lab = testkit::spawn(
        &mut w,
        Kind::Building,
        "LAB",
        neutral,
        glasnost_map::CellXY::new(home.x + 5, home.y + 5),
        0,
    );
    let derr = testkit::spawn(
        &mut w,
        Kind::Building,
        "DERR",
        neutral,
        glasnost_map::CellXY::new(home.x + 8, home.y + 8),
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
    reveal(&mut w, 0, glasnost_map::CellXY::new(home.x + 7, home.y + 7));
    testkit::run(&mut w, 2);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    (w, lab, derr, unit)
}

// S-0515, PLAN §4.7: an idle engineer is sent to the tech building that pays (its acquisition bonus and income) before a nearer one that only serves, and the capture pays the bonus.
#[test]
fn sends_an_engineer_to_a_paying_tech_building_first() {
    let mut g = game_with_neutral(5);
    ready(&mut g);
    let (mut w, lab, derr, unit) = tech_scene(&g);
    for b in [lab, derr] {
        assert!(glasnost_sim::econ::shroud::explored(
            &w,
            HouseId(0),
            w.obj(b).unwrap().pos.cell()
        ));
    }
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = brain.think(&w);
    assert!(garrison_orders(&orders, unit, derr), "{orders:?}");
    assert!(!garrison_orders(&orders, unit, lab));
    let uses = brain.special_uses();
    assert_eq!((uses.engineers, uses.tech), (1, 1));
    let before = w.houses[0].credits;
    for o in orders {
        glasnost_sim::orders::issue(&mut w, HouseId(0), o);
    }
    for _ in 0..800 {
        testkit::run(&mut w, 1);
        if w.obj(unit).is_none() {
            break;
        }
    }
    assert_eq!(w.obj(derr).unwrap().owner, Some(HouseId(0)), "captured");
    assert!(w.houses[0].credits >= before + 300, "the acquisition bonus");
}

// S-0515, PLAN §4.7: it takes the building that only serves when nothing pays, and sees only explored cells.
#[test]
fn takes_a_serving_tech_building_and_only_an_explored_one() {
    let mut g = game_with_neutral(5);
    ready(&mut g);
    let (mut w, lab, derr, unit) = tech_scene(&g);
    glasnost_sim::objects::remove(&mut w, derr);
    let orders = Brain::new(&w, &fixture::map(), HouseId(0)).think(&w);
    assert!(garrison_orders(&orders, unit, lab), "{orders:?}");
    // On an unexplored cell nothing is ordered.
    let far = glasnost_map::CellXY::new(2, 2);
    assert!(!glasnost_sim::econ::shroud::explored(&w, HouseId(0), far));
    glasnost_sim::objects::remove(&mut w, lab);
    let far_derr = testkit::spawn(&mut w, Kind::Building, "DERR", 2, far, 0);
    let orders = Brain::new(&w, &fixture::map(), HouseId(0)).think(&w);
    assert!(!garrison_orders(&orders, unit, far_derr), "{orders:?}");
}

// S-0515, PLAN §4.7: with no engineer, a barracks and a seen tech building of the passive house, the house orders an engineer once.
#[test]
fn orders_an_engineer_for_a_tech_building() {
    let mut g = game_with_neutral(5);
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
        "DERR",
        2,
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
    let orders = Brain::new(&w, &fixture::map(), HouseId(0)).think(&w);
    assert!(orders.iter().any(want), "{orders:?}");
}

/// Runs the world on to the next think tick after `ticks` more ticks.
fn run_to_think(w: &mut World, ticks: i32) {
    testkit::run(w, ticks);
    while w.tick % 15 != 0 {
        testkit::run(w, 1);
    }
}

// PLAN §4.7 (memory): an engineer that does not take its order (it stands on the same cell and idle at the next think) is not sent at the same building again every think; the first think orders it once, the next thinks do not, and after a long while it tries again.
#[test]
fn does_not_reorder_an_engineer_that_did_not_take_the_order() {
    let mut g = game(5);
    ready(&mut g);
    let (mut w, target, unit) = engineer_scene(&g);
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = brain.think(&w);
    assert!(garrison_orders(&orders, unit, target), "{orders:?}");
    // The orders are not issued: the engineer stays where it is.
    run_to_think(&mut w, 1);
    let again = brain.think(&w);
    assert!(!garrison_orders(&again, unit, target), "{again:?}");
    assert_eq!(brain.special_uses().engineers, 1);
    assert_eq!(brain.given_up(), 1);
    for _ in 0..5 {
        run_to_think(&mut w, 1);
        let more = brain.think(&w);
        assert!(!garrison_orders(&more, unit, target), "{more:?}");
    }
    assert_eq!(brain.special_uses().engineers, 1);
    // Long after, the world may have changed: it tries once more.
    run_to_think(&mut w, 5000);
    let later = brain.think(&w);
    assert!(garrison_orders(&later, unit, target), "{later:?}");
    assert_eq!(brain.special_uses().engineers, 2);
}

// design: own AI (#19) and S-0213 claim 2 (an order issued while the house's pending list is full is dropped): an engineer's order that the full list dropped is not remembered as one the engineer did not take; it is given again at the next think.
#[test]
fn an_order_dropped_by_the_pending_limit_is_not_given_up() {
    let mut g = game(5);
    ready(&mut g);
    let (mut w, target, unit) = engineer_scene(&g);
    assert!(w.rules().quirks.pending_order_limit);
    // The house's pending list full (a stop leaves the behaviour as it is).
    for _ in 0..glasnost_sim::orders::PENDING_LIMIT {
        assert!(glasnost_sim::orders::issue(
            &mut w,
            HouseId(0),
            Order::Stop { actor: unit }
        ));
    }
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = brain.think(&w);
    assert!(garrison_orders(&orders, unit, target), "{orders:?}");
    for o in orders {
        assert!(
            !glasnost_sim::orders::issue(&mut w, HouseId(0), o),
            "the list is full"
        );
    }
    run_to_think(&mut w, 1);
    let again = brain.think(&w);
    assert!(garrison_orders(&again, unit, target), "{again:?}");
    assert_eq!(brain.given_up(), 0);
}

// PLAN §4.7 (memory): an engineer that gave up one building is sent at another it can see.
#[test]
fn an_engineer_that_gave_up_a_building_goes_for_another() {
    let mut g = game(5);
    ready(&mut g);
    let (mut w, target, unit) = engineer_scene(&g);
    let home = w.obj(unit).unwrap().pos.cell();
    let other_cell = glasnost_map::CellXY::new(home.x + 3, home.y + 9);
    let other = testkit::spawn(&mut w, Kind::Building, "CAPT", 1, other_cell, 0);
    reveal(&mut w, 0, other_cell);
    run_to_think(&mut w, 1);
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = brain.think(&w);
    let first = [target, other]
        .into_iter()
        .find(|&b| garrison_orders(&orders, unit, b))
        .expect("it picks one");
    run_to_think(&mut w, 1);
    let orders = brain.think(&w);
    let second = [target, other]
        .into_iter()
        .find(|&b| garrison_orders(&orders, unit, b))
        .expect("it picks the other");
    assert_ne!(first, second);
}

// PLAN §4.7 (memory): an engineer that took its order and is on its way leaves no memory.
#[test]
fn an_engineer_on_its_way_is_not_given_up() {
    let mut g = game(5);
    ready(&mut g);
    let (mut w, _target, unit) = engineer_scene(&g);
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    for o in brain.think(&w) {
        glasnost_sim::orders::issue(&mut w, HouseId(0), o);
    }
    for _ in 0..4 {
        run_to_think(&mut w, 1);
        let _ = brain.think(&w);
        assert_eq!(brain.given_up(), 0);
    }
    assert!(w.obj(unit).is_some());
}

/// Whether `orders` send `unit` at building `b` (any behaviour, by its
/// destination or target).
fn sent_at(orders: &[Order], unit: ObjectId, b: ObjectId) -> bool {
    orders.iter().any(|o| {
        matches!(o, Order::Behaviour { actor, dest, target, .. }
            if *actor == unit && (*dest == Some(Target::Object(b)) || *target == Some(Target::Object(b))))
    })
}

/// The cell of house 0's first object on the map.
fn home_of(w: &World) -> glasnost_map::CellXY {
    w.objects
        .values()
        .find(|o| o.owner == Some(HouseId(0)) && o.on_map)
        .map(|o| o.pos.cell())
        .expect("an object of house 0")
}

// design (M8 step A4 round 8, the lead's note 51; S-0509, S-0214 rows I4 to I7): the AI spends an engineer on an own building only for one it cannot do without (a factory here) at half its strength or below; not on a factory that is only scratched, and not on a building of no economy or production role (a power plant here), however damaged.
#[test]
fn an_engineer_repairs_only_an_important_badly_damaged_building() {
    let mut g = game(5);
    ready(&mut g);
    let mut w = g.world.clone();
    let home = home_of(&w);
    let pill = testkit::spawn(
        &mut w,
        Kind::Building,
        "PILL",
        0,
        glasnost_map::CellXY::new(home.x + 6, home.y),
        0,
    );
    let powr = testkit::spawn(
        &mut w,
        Kind::Building,
        "POWR",
        0,
        glasnost_map::CellXY::new(home.x + 6, home.y + 4),
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
    run_to_think(&mut w, 2);
    let full = |w: &World, b| glasnost_sim::combat::damage::strength(w, b);
    let set = |w: &mut World, b, num: i32, den: i32| {
        let s = full(w, b);
        w.obj_mut(b).unwrap().health = s * num / den;
    };
    // The factory at three quarters, the power plant at a quarter.
    set(&mut w, pill, 3, 4);
    set(&mut w, powr, 1, 4);
    let orders = Brain::new(&w, &fixture::map(), HouseId(0)).think(&w);
    assert!(!sent_at(&orders, unit, pill), "scratched: {orders:?}");
    assert!(!sent_at(&orders, unit, powr), "no role: {orders:?}");
    // The factory at a third: the engineer goes.
    set(&mut w, pill, 1, 3);
    let orders = Brain::new(&w, &fixture::map(), HouseId(0)).think(&w);
    assert!(sent_at(&orders, unit, pill), "{orders:?}");
    assert!(!sent_at(&orders, unit, powr));
}

/// A world copy where house 0 has a barracks, no engineer and no credits,
/// and sees `derricks` tech buildings that pay (DERR of the Neutral house).
fn derrick_scene(g: &Game, derricks: i32) -> (World, Vec<ObjectId>) {
    let mut w = g.world.clone();
    let home = home_of(&w);
    testkit::spawn(
        &mut w,
        Kind::Building,
        "PILL",
        0,
        glasnost_map::CellXY::new(home.x + 5, home.y),
        0,
    );
    let mut all = Vec::new();
    for i in 0..derricks {
        let c = glasnost_map::CellXY::new(home.x + 8 + 3 * i, home.y + 8);
        all.push(testkit::spawn(&mut w, Kind::Building, "DERR", 2, c, 0));
        reveal(&mut w, 0, c);
    }
    run_to_think(&mut w, 20);
    w.houses[0].credits = 0;
    (w, all)
}

fn orders_engineer(w: &World, orders: &[Order]) -> bool {
    let engi = w.rules().index_of(Kind::Infantry, "ENGI").unwrap();
    orders.iter().any(
        |o| matches!(o, Order::Build { what } if what.kind == Kind::Infantry && what.index == engi),
    )
}

// design (M8 step A4 round 8, the lead's note 48; S-0515 claims 3 and 5): a seen tech building that pays has the house queue an engineer at once, with no credits in hand (the line pays as money comes in), where before it waited for twice the price.
#[test]
fn queues_an_engineer_for_a_paying_tech_building_at_once() {
    let mut g = game_with_neutral(5);
    ready(&mut g);
    let (w, _) = derrick_scene(&g, 1);
    assert_eq!(w.houses[0].credits, 0);
    let orders = Brain::new(&w, &fixture::map(), HouseId(0)).think(&w);
    assert!(orders_engineer(&w, &orders), "{orders:?}");
}

// design (M8 step A4 round 8, the lead's note 48): with three paying tech buildings in sight the house keeps up to two engineers for them: one engineer, it queues another; two, it does not.
#[test]
fn keeps_up_to_two_engineers_for_paying_tech_buildings() {
    let mut g = game_with_neutral(5);
    ready(&mut g);
    for (have, want) in [(1, true), (2, false)] {
        let (mut w, _) = derrick_scene(&g, 3);
        let home = home_of(&w);
        for i in 0..have {
            testkit::spawn(
                &mut w,
                Kind::Infantry,
                "ENGI",
                0,
                glasnost_map::CellXY::new(home.x + 2, home.y + 2 + i),
                0,
            );
        }
        run_to_think(&mut w, 1);
        let orders = Brain::new(&w, &fixture::map(), HouseId(0)).think(&w);
        assert_eq!(orders_engineer(&w, &orders), want, "{have}: {orders:?}");
    }
}

// design (M8 step A4 round 8, the lead's note 48): an engineer sent to a tech building takes the nearest idle army unit within reach along, attack-moved to the building; a unit far away is not sent.
#[test]
fn an_engineer_for_a_tech_building_takes_an_escort() {
    let mut g = game_with_neutral(5);
    ready(&mut g);
    // Toward the middle of the map (house 0 starts in the west): one cell
    // off, and beyond the reach.
    for (dx, escorted) in [(-1, true), (18, false)] {
        let (mut w, _lab, derr, unit) = tech_scene(&g);
        let at = w.obj(unit).unwrap().pos.cell();
        let tank = testkit::spawn(
            &mut w,
            Kind::Vehicle,
            "GUNR",
            0,
            glasnost_map::CellXY::new(at.x + dx, at.y),
            0,
        );
        let goal = w.obj(derr).unwrap().pos.cell();
        run_to_think(&mut w, 1);
        let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
        let orders = brain.think(&w);
        assert!(garrison_orders(&orders, unit, derr), "{orders:?}");
        let escort = orders.iter().any(|o| {
            matches!(o, Order::Behaviour { actor, behaviour: Behaviour::Move, attack_move: true, dest: Some(Target::Cell(c)), .. }
                if *actor == tank && *c == goal)
        });
        assert_eq!(escort, escorted, "{dx}: {orders:?}");
        assert_eq!(brain.special_uses().escorts, u32::from(escorted));
    }
}
