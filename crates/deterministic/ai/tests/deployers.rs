//! The own AI's use of deployable units (S-0517, PLAN §4.7): it deploys an
//! in-place vehicle when a seen enemy is near and packs it up when none is,
//! deploys deploy-fire vehicles, deploys firing infantry when an enemy is in
//! its range and packs it up when none is, never gives such an order without
//! a seen enemy, and does not send a deployed unit with a wave.

use std::sync::Arc;

use glasnost_ai::brain::{Brain, default_plan};
use glasnost_ai::fixture;
use glasnost_ai::game::Game;
use glasnost_map::CellXY;
use glasnost_rules::sim::SequenceEntry;
use glasnost_sim::econ::inplace::{self, Form};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

/// The fixture's rules with DEPT (a gun vehicle that deploys in place), FIRE
/// (a deploy-fire vehicle), DEPI (a rifleman that deploys to fire, with
/// its rifle) and DEPR (DEPI immune to radiation, which deployed fires at
/// its own cell).
fn rules() -> Arc<SimRules> {
    let mut r = fixture::ruleset();
    r.registries
        .entry("vehicles".into())
        .or_default()
        .extend(["DEPT".to_string(), "FIRE".to_string()]);
    r.registries
        .entry("infantry".into())
        .or_default()
        .extend(["DEPI".to_string(), "DEPR".to_string()]);
    let gun = r.sim.vehicles["GUNR"].clone();
    let mut dept = gun.clone();
    dept.is_simple_deployer = Some(true);
    dept.econ.prerequisite = None;
    dept.tech_level = Some(99);
    let mut fire = gun;
    fire.deploy_fire = Some(true);
    fire.tech_level = Some(99);
    r.sim.vehicles.insert("DEPT".into(), dept);
    r.sim.vehicles.insert("FIRE".into(), fire);
    let mut inf = r.sim.infantry["WALKER"].clone();
    inf.deployer = Some(true);
    inf.deploy_fire = Some(true);
    inf.undeploy_delay = Some(-1);
    // Deployed, it fires its primary (S-0308 claim 8: the deploy weapon index).
    inf.deploy_fire_weapon = Some(0);
    inf.tech_level = Some(99);
    for (k, n) in [
        ("deploy", 4),
        ("deployed", 1),
        ("undeploy", 4),
        ("deployed_fire", 2),
    ] {
        inf.sequences.insert(
            k.into(),
            SequenceEntry {
                start: 0,
                count: n,
                multiplier: 0,
                direction: None,
            },
        );
    }
    let mut depr = inf.clone();
    depr.immune_to_radiation = Some(true);
    r.sim.infantry.insert("DEPI".into(), inf);
    r.sim.infantry.insert("DEPR".into(), depr);
    Arc::new(SimRules::new(&r, "synthetic-skirmish"))
}

fn game(seed: u32) -> Game {
    Game::new(
        rules(),
        &fixture::map(),
        "synthetic",
        fixture::catalogue(),
        fixture::setup(seed),
    )
    .expect("the skirmish loads")
}

fn ready(g: &mut Game) {
    while g.world.tick <= g.world.orders_blocked_through || g.world.tick % 15 != 0 {
        g.step();
    }
}

fn first_cell(w: &World, house: u8) -> CellXY {
    w.objects
        .values()
        .find(|o| o.owner == Some(HouseId(house)) && o.on_map)
        .map(|o| o.pos.cell())
        .expect("an object of the house")
}

/// A copy of the world where house 0 has `name` at a free spot near its base
/// and, when `enemy` is given, a house 1 gunner `enemy` cells away whose cell
/// the house has explored.
fn scene(g: &Game, kind: Kind, name: &str, enemy: Option<i32>) -> (World, ObjectId) {
    let mut w = g.world.clone();
    let home = first_cell(&w, 0);
    let at = CellXY::new(home.x, home.y + 6);
    let unit = testkit::spawn(&mut w, kind, name, 0, at, 0);
    if let Some(d) = enemy {
        let e = CellXY::new(at.x + d, at.y);
        testkit::spawn(&mut w, Kind::Vehicle, "GUNR", 1, e, 0);
        glasnost_sim::econ::shroud::area_reveal(&mut w, HouseId(0), Pos::centre(e, 0), 3);
    }
    testkit::run(&mut w, 2);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    (w, unit)
}

fn deploys(orders: &[Order], who: ObjectId) -> bool {
    orders
        .iter()
        .any(|o| matches!(o, Order::Deploy { actor } if *actor == who))
}

// S-0517, PLAN §4.7: an idle in-place vehicle with a seen enemy near is deployed, with the order a person gives; the counter counts it.
#[test]
fn deploys_an_in_place_vehicle_near_an_enemy() {
    let mut g = game(5);
    ready(&mut g);
    let (mut w, unit) = scene(&g, Kind::Vehicle, "DEPT", Some(5));
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = brain.think(&w);
    assert!(deploys(&orders, unit), "{orders:?}");
    assert_eq!(brain.special_uses().deploys, 1);
    for o in orders {
        glasnost_sim::orders::issue(&mut w, HouseId(0), o);
    }
    for _ in 0..60 {
        testkit::run(&mut w, 1);
    }
    assert_eq!(inplace::form(&w, unit), Form::Deployed);
}

// S-0517, PLAN §4.7: it sees only what its house sees: with no enemy on explored cells near, nothing is deployed.
#[test]
fn does_not_deploy_without_a_seen_enemy() {
    let mut g = game(5);
    ready(&mut g);
    let (w, unit) = scene(&g, Kind::Vehicle, "DEPT", None);
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    assert!(!deploys(&brain.think(&w), unit));
    // An enemy on a cell the house has not explored is not seen.
    let mut w2 = g.world.clone();
    let home = first_cell(&w2, 0);
    let at = CellXY::new(home.x, home.y + 6);
    let unit = testkit::spawn(&mut w2, Kind::Vehicle, "DEPT", 0, at, 0);
    testkit::spawn(
        &mut w2,
        Kind::Vehicle,
        "GUNR",
        1,
        CellXY::new(at.x + 5, at.y),
        0,
    );
    let mut brain = Brain::new(&w2, &fixture::map(), HouseId(0));
    let orders = brain.think(&w2);
    let seen = glasnost_sim::econ::shroud::explored(&w2, HouseId(0), CellXY::new(at.x + 5, at.y));
    assert_eq!(deploys(&orders, unit), seen);
}

// S-0517, PLAN §4.7: a deployed vehicle with no enemy near packs up so that it can move with the army, and a deployed vehicle is not sent with a wave.
#[test]
fn packs_up_when_the_enemy_is_gone() {
    let mut g = game(5);
    ready(&mut g);
    let (mut w, unit) = scene(&g, Kind::Vehicle, "DEPT", None);
    glasnost_sim::orders::issue(&mut w, HouseId(0), Order::Deploy { actor: unit });
    for _ in 0..30 {
        testkit::run(&mut w, 1);
    }
    assert_eq!(inplace::form(&w, unit), Form::Deployed);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    let mut plan = default_plan();
    plan.wave = 1;
    let mut brain = Brain::with_plan(&w, &fixture::map(), HouseId(0), plan);
    let orders = brain.think(&w);
    assert!(deploys(&orders, unit), "it packs up");
    assert!(
        !orders.iter().any(|o| matches!(
            o,
            Order::Behaviour { actor, behaviour: Behaviour::Move, .. } if *actor == unit
        )),
        "a deployed vehicle is not sent with a wave"
    );
    for o in orders {
        glasnost_sim::orders::issue(&mut w, HouseId(0), o);
    }
    for _ in 0..30 {
        testkit::run(&mut w, 1);
    }
    assert_eq!(inplace::form(&w, unit), Form::Mobile);
}

// S-0517, PLAN §4.7: a deploy-fire vehicle is deployed when a seen enemy is near, and a firing infantry unit when a seen enemy is in its range (the fixture rifle reaches 4 cells), each once.
#[test]
fn deploys_deploy_fire_vehicles_and_firing_infantry() {
    let mut g = game(5);
    ready(&mut g);
    for (kind, name, d) in [(Kind::Vehicle, "FIRE", 5), (Kind::Infantry, "DEPI", 3)] {
        let (mut w, unit) = scene(&g, kind, name, Some(d));
        let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
        let orders = brain.think(&w);
        assert!(deploys(&orders, unit), "{name}: {orders:?}");
        for o in orders {
            glasnost_sim::orders::issue(&mut w, HouseId(0), o);
        }
        for _ in 0..150 {
            testkit::run(&mut w, 1);
        }
        while w.tick % 15 != 0 {
            testkit::run(&mut w, 1);
        }
        let again = brain.think(&w);
        assert!(!deploys(&again, unit), "{name}: once");
    }
}

// design: own AI (#17), S-0517 claim 2 (unload stays current until the vehicle is given another behaviour) and S-0213 table T2 (the guard key gives area guard on the cell the unit stands in): a deploy-fire vehicle firing at its own cell with no enemy near is released, once, and leaves unload; it was left so for the rest of the game.
#[test]
fn releases_a_deploy_fire_vehicle_when_the_enemy_is_gone() {
    let mut g = game(5);
    ready(&mut g);
    let (mut w, unit) = scene(&g, Kind::Vehicle, "FIRE", None);
    glasnost_sim::orders::issue(&mut w, HouseId(0), Order::Deploy { actor: unit });
    testkit::run(&mut w, 30);
    let unloads = |w: &World| w.obj(unit).unwrap().beh.current == Some(Behaviour::Unload);
    assert!(unloads(&w), "it fires at its own cell");
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = brain.think(&w);
    assert!(ordered(&orders, unit), "it is not released: {orders:?}");
    assert!(!deploys(&orders, unit));
    for o in orders {
        glasnost_sim::orders::issue(&mut w, HouseId(0), o);
    }
    // The queued behaviour starts at unload's next processing, after its
    // delay of 14 to 16 ticks (S-0517 claim 2).
    testkit::run(&mut w, 17);
    assert!(!unloads(&w), "it still fires at its own cell");
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    let again = brain.think(&w);
    assert!(
        !again.iter().any(|o| matches!(
            o,
            Order::Behaviour { actor, behaviour: Behaviour::AreaGuard, .. } if *actor == unit
        )),
        "released again: {again:?}"
    );
}

fn deployed(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .and_then(|o| o.infantry.as_ref())
        .and_then(|s| s.seq)
        .is_some_and(glasnost_sim::infantry::Seq::is_deploy)
}

fn ordered(orders: &[Order], who: ObjectId) -> bool {
    orders
        .iter()
        .any(|o| matches!(o, Order::Behaviour { actor, .. } if *actor == who))
}

// S-0517 claim 16 and S-0215 claim 22, PLAN §4.7 (M8 step B7): deployed infantry takes no step towards its target and drops a destination, so a firing infantry unit is not deployed for a seen enemy beyond its range (the fixture rifle reaches 4 cells; this enemy stands 6 away).
#[test]
fn does_not_deploy_a_rifleman_out_of_range() {
    let mut g = game(5);
    ready(&mut g);
    let (w, unit) = scene(&g, Kind::Infantry, "DEPI", Some(6));
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    assert!(!deploys(&brain.think(&w), unit));
}

// S-0517 claim 16, S-0215 claim 22 and S-0213 claims 3 and 10, PLAN §4.7 (M8 step B7): a deployed firing infantry unit with a seen enemy in its range stays deployed; with none in its range it is packed up, with no wave order beside the deploy order (that would replace the unload), and then it can go with a wave.
#[test]
fn packs_up_a_deployed_rifleman_with_no_enemy_in_range() {
    let mut g = game(5);
    ready(&mut g);
    let mut plan = default_plan();
    plan.wave = 1;
    for (d, packs) in [(3, false), (7, true)] {
        let (mut w, unit) = scene(&g, Kind::Infantry, "DEPI", Some(d));
        glasnost_sim::orders::issue(&mut w, HouseId(0), Order::Deploy { actor: unit });
        for _ in 0..30 {
            testkit::run(&mut w, 1);
        }
        assert!(deployed(&w, unit), "{d}: deployed");
        while w.tick % 15 != 0 {
            testkit::run(&mut w, 1);
        }
        let mut brain = Brain::with_plan(&w, &fixture::map(), HouseId(0), plan.clone());
        let orders = brain.think(&w);
        assert_eq!(deploys(&orders, unit), packs, "{d}: {orders:?}");
        if !packs {
            continue;
        }
        assert!(!ordered(&orders, unit), "{orders:?}");
        for o in orders {
            glasnost_sim::orders::issue(&mut w, HouseId(0), o);
        }
        for _ in 0..30 {
            testkit::run(&mut w, 1);
        }
        assert!(!deployed(&w, unit), "packed up");
    }
}

fn attacks(orders: &[Order], who: ObjectId, at: ObjectId) -> bool {
    orders.iter().any(|o| {
        matches!(
            o,
            Order::Behaviour { actor, behaviour: Behaviour::Attack, target: Some(Target::Object(t)), .. }
                if *actor == who && *t == at
        )
    })
}

// S-0213 claim 10, S-0517 claim 16 and S-0309 table C (row *harmless building*), PLAN §4.7 (our own AI's design, M8 seam trace 4): the deploy order clears a rifleman's target and, deployed, its range search never takes an enemy building with no weapon and no threat, so it stands with no target beside it; the brain then clicks it on that building, once, as a person would, and it fires from its deployed form.
#[test]
fn clicks_a_deployed_rifleman_on_a_harmless_building_in_range() {
    let mut g = game(5);
    ready(&mut g);
    let (mut w, unit) = scene(&g, Kind::Infantry, "DEPI", None);
    let at = w.obj(unit).expect("the rifleman").pos.cell();
    let plant_cell = CellXY::new(at.x + 2, at.y);
    let plant = testkit::spawn(&mut w, Kind::Building, "POWR", 1, plant_cell, 0);
    glasnost_sim::econ::shroud::area_reveal(&mut w, HouseId(0), Pos::centre(plant_cell, 0), 3);
    glasnost_sim::orders::issue(&mut w, HouseId(0), Order::Deploy { actor: unit });
    for _ in 0..150 {
        testkit::run(&mut w, 1);
    }
    let full = w.obj(plant).expect("the plant").health;
    assert!(deployed(&w, unit), "deployed");
    assert_eq!(
        w.obj(unit).expect("the rifleman").target,
        None,
        "the sim gives it no target"
    );
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    let orders = brain.think(&w);
    assert!(attacks(&orders, unit, plant), "{orders:?}");
    assert!(!deploys(&orders, unit), "it stays deployed: {orders:?}");
    for o in orders {
        glasnost_sim::orders::issue(&mut w, HouseId(0), o);
    }
    for _ in 0..150 {
        testkit::run(&mut w, 1);
    }
    assert!(deployed(&w, unit), "still deployed");
    assert_eq!(
        w.obj(unit).expect("the rifleman").target,
        Some(Target::Object(plant))
    );
    assert!(
        w.obj(plant).expect("the plant").health < full,
        "it fires at the plant"
    );
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    assert!(!attacks(&brain.think(&w), unit, plant), "once");
}

// S-0215 claim 31 rows 3 to 5, PLAN §4.7 (M8 step B7): a radiation-immune firing infantry unit fires at its own cell when deployed, so it is deployed with a seen enemy near though beyond its rifle's range, stays deployed while one is near, and is packed up when none is.
#[test]
fn deploys_a_radiation_immune_unit_near_an_enemy_and_packs_it_up_without_one() {
    let mut g = game(5);
    ready(&mut g);
    let (w, unit) = scene(&g, Kind::Infantry, "DEPR", Some(6));
    let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
    assert!(deploys(&brain.think(&w), unit), "deploys near an enemy");
    for (enemy, packs) in [(Some(7), false), (None, true)] {
        let (mut w, unit) = scene(&g, Kind::Infantry, "DEPR", enemy);
        glasnost_sim::orders::issue(&mut w, HouseId(0), Order::Deploy { actor: unit });
        for _ in 0..30 {
            testkit::run(&mut w, 1);
        }
        assert!(deployed(&w, unit), "{enemy:?}: deployed");
        while w.tick % 15 != 0 {
            testkit::run(&mut w, 1);
        }
        let mut brain = Brain::new(&w, &fixture::map(), HouseId(0));
        assert_eq!(deploys(&brain.think(&w), unit), packs, "{enemy:?}");
    }
}

// design: own AI (#47) and S-0213 claim 3 (the behaviour orders are applied after the others, so a later order for the same unit replaces the deploy): an in-place vehicle deployed for a seen enemy is given the deploy order and no other order in the same think.
#[test]
fn a_deployed_vehicle_gets_no_second_order() {
    let mut g = game(5);
    ready(&mut g);
    let (w, unit) = scene(&g, Kind::Vehicle, "DEPT", Some(5));
    let orders = Brain::new(&w, &fixture::map(), HouseId(0)).think(&w);
    assert!(deploys(&orders, unit), "{orders:?}");
    let mine: Vec<&Order> = orders.iter().filter(|o| o.actor() == Some(unit)).collect();
    assert_eq!(mine.len(), 1, "one order for the vehicle: {mine:?}");
}
