//! The own AI's use of deployable units (S-0517, PLAN §4.7): it deploys an
//! in-place vehicle when a seen enemy is near and packs it up when none is,
//! deploys deploy-fire vehicles and firing infantry, never gives such an
//! order without a seen enemy, and does not send a deployed vehicle with a
//! wave.

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
use glasnost_sim::world::{HouseId, ObjectId, Pos, World};

/// The fixture's rules with DEPT (a gun vehicle that deploys in place), FIRE
/// (a deploy-fire vehicle) and DEPI (a rifleman that deploys to fire).
fn rules() -> Arc<SimRules> {
    let mut r = fixture::ruleset();
    r.registries
        .entry("vehicles".into())
        .or_default()
        .extend(["DEPT".to_string(), "FIRE".to_string()]);
    r.registries
        .entry("infantry".into())
        .or_default()
        .push("DEPI".into());
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
    r.sim.infantry.insert("DEPI".into(), inf);
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

// S-0517, PLAN §4.7: a deploy-fire vehicle and a firing infantry unit are deployed when a seen enemy is near, each once.
#[test]
fn deploys_deploy_fire_vehicles_and_firing_infantry() {
    let mut g = game(5);
    ready(&mut g);
    for (kind, name) in [(Kind::Vehicle, "FIRE"), (Kind::Infantry, "DEPI")] {
        let (mut w, unit) = scene(&g, kind, name, Some(5));
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
