//! The own AI and tank bunkers (S-0606, PLAN §4.7): the house builds
//! bunkers near its base; an empty bunker gets the nearest suitable idle
//! vehicle by the enter click, never a harvester; a bunkered vehicle is let
//! out by a deploy click to repair, retreat or attack. The tests use
//! invented types on the synthetic skirmish.

use std::sync::Arc;

use glasnost_ai::brain::Brain;
use glasnost_ai::bunker::{self, REPAIR_PERCENT, RETREAT_PERCENT};
use glasnost_ai::fixture;
use glasnost_ai::game::Game;
use glasnost_map::CellXY;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::structures::bunker as sim_bunker;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{testkit, tick};

/// The fixture's rules with BUNK (a 2 × 2 tank bunker, cost 500, tech level
/// `tech`, no impassable rows).
fn rules(tech: i32) -> Arc<SimRules> {
    let mut r = fixture::ruleset();
    let mut b = fixture::building(2, 2, 500, 0);
    b.bunker = Some(true);
    b.number_impassable_rows = Some(0);
    b.tech_level = Some(tech);
    r.sim.buildings.insert("BUNK".into(), b);
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("BUNK".into());
    Arc::new(SimRules::new(&r, "synthetic-bunker"))
}

fn game(tech: i32) -> Game {
    let mut g = Game::new(
        rules(tech),
        &fixture::map(),
        "synthetic",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the skirmish loads");
    while g.world.tick <= g.world.orders_blocked_through || g.world.tick % 15 != 0 {
        g.step();
    }
    g
}

fn home(w: &World) -> CellXY {
    w.objects
        .values()
        .find(|o| o.owner == Some(HouseId(0)) && o.on_map)
        .map(|o| o.pos.cell())
        .unwrap()
}

fn spawn(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    let h = home(w);
    testkit::spawn(w, kind, name, house, CellXY::new(h.x + dx, h.y + dy), 0)
}

/// A world with house 0's bunker six cells south of its base.
fn scene(g: &Game) -> (World, ObjectId) {
    let mut w = g.world.clone();
    let b = spawn(&mut w, Kind::Building, "BUNK", 0, 0, 6);
    testkit::run(&mut w, 3);
    (w, b)
}

fn settle(w: &mut World) {
    testkit::run(w, 1);
    while w.tick % 15 != 0 {
        testkit::run(w, 1);
    }
}

fn brain(w: &World) -> Brain {
    Brain::new(w, &fixture::map(), HouseId(0))
}

/// The building unit `who`'s last enter order sends it into.
fn entered(orders: &[Order], who: ObjectId) -> Option<ObjectId> {
    orders.iter().rev().find_map(|o| match o {
        Order::Behaviour {
            actor,
            behaviour: Behaviour::Enter,
            dest: Some(Target::Object(x)),
            ..
        } if *actor == who => Some(*x),
        _ => None,
    })
}

fn deployed(orders: &[Order], who: ObjectId) -> bool {
    orders
        .iter()
        .any(|o| matches!(o, Order::Deploy { actor } if *actor == who))
}

/// Puts `v` into bunker `b` through the sim and returns once it is bunkered.
fn bunker_in(w: &mut World, v: ObjectId, b: ObjectId) {
    glasnost_sim::orders::issue(
        w,
        HouseId(0),
        Order::Behaviour {
            actor: v,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: None,
            dest: Some(Target::Object(b)),
        },
    );
    for _ in 0..1500 {
        tick::step(w);
        if sim_bunker::bunkered_in(w, v) == Some(b) {
            return;
        }
    }
    panic!("not bunkered");
}

// S-0606 claims 1 to 3 and S-0214 (enter cursor), PLAN §4.7: an empty bunker gets the nearest suitable idle vehicle by the enter click; the counter counts the order; a harvester is never sent.
#[test]
fn an_idle_vehicle_is_sent_in_never_a_harvester() {
    let g = game(99);
    let (mut w, b) = scene(&g);
    let minr = spawn(&mut w, Kind::Vehicle, "MINR", 0, 3, 8);
    let gun = spawn(&mut w, Kind::Vehicle, "GUNR", 0, 5, 9);
    let far = spawn(&mut w, Kind::Vehicle, "GUNR", 0, 7, 11);
    settle(&mut w);
    let mut br = brain(&w);
    let orders = br.think(&w);
    assert_eq!(entered(&orders, gun), Some(b), "{orders:?}");
    assert_eq!(entered(&orders, far), None, "one per bunker");
    assert_eq!(entered(&orders, minr), None, "never a harvester");
    assert_eq!(br.special_uses().bunker_entries, 1);
}

// S-0606 claims 1 and 2 (row P8), PLAN §4.7: a vehicle the bunker does not accept (no turret weapon) is not sent.
#[test]
fn an_ineligible_vehicle_is_not_sent() {
    let g = game(99);
    let (mut w, b) = scene(&g);
    let truck = spawn(&mut w, Kind::Vehicle, "TRUK", 0, 4, 9);
    settle(&mut w);
    let mut br = brain(&w);
    let orders = br.think(&w);
    assert_eq!(entered(&orders, truck), None, "{orders:?}");
    assert!(!sim_bunker::permits(&w, b, truck));
    assert_eq!(br.special_uses().bunker_entries, 0);
}

// S-0606 claim 3, PLAN §4.7: a bunker with a vehicle on its way or inside gets no second one.
#[test]
fn a_taken_bunker_gets_no_second_vehicle() {
    let g = game(99);
    let (mut w, b) = scene(&g);
    let a = spawn(&mut w, Kind::Vehicle, "GUNR", 0, 4, 9);
    bunker_in(&mut w, a, b);
    let c = spawn(&mut w, Kind::Vehicle, "GUNR", 0, 5, 11);
    settle(&mut w);
    assert!(bunker::taken(&w, b));
    let mut br = brain(&w);
    let orders = br.think(&w);
    assert_eq!(entered(&orders, c), None, "{orders:?}");
    assert!(!deployed(&orders, a), "a sound vehicle in a sound bunker stays");
}

// S-0606 claims 11 and 12 and S-0214 table VD row VD5, PLAN §4.7: a damaged bunkered vehicle is let out by the deploy click to be repaired; so is one whose bunker is badly damaged.
#[test]
fn let_out_to_repair_or_retreat() {
    let g = game(99);
    for (hurt_vehicle, why) in [(true, "repair"), (false, "retreat")] {
        let (mut w, b) = scene(&g);
        let a = spawn(&mut w, Kind::Vehicle, "GUNR", 0, 4, 9);
        bunker_in(&mut w, a, b);
        let (who, pct) = if hurt_vehicle {
            (a, REPAIR_PERCENT)
        } else {
            (b, RETREAT_PERCENT)
        };
        let s = glasnost_sim::combat::damage::strength(&w, who);
        w.obj_mut(who).unwrap().health = s * pct / 100;
        settle(&mut w);
        let mut br = brain(&w);
        let orders = br.think(&w);
        assert!(deployed(&orders, a), "{why}: {orders:?}");
        assert_eq!(br.special_uses().bunker_exits, 1, "{why}");
        // The deploy order lets it out (S-0606 claim 12).
        for o in orders {
            glasnost_sim::orders::issue(&mut w, HouseId(0), o);
        }
        testkit::run(&mut w, 60);
        assert!(!sim_bunker::bunkered(&w, a), "{why}");
    }
}

// PLAN §4.7: an enemy seen near the bunker that the vehicle has no target on lets it out to attack.
#[test]
fn let_out_to_attack() {
    let g = game(99);
    let (mut w, b) = scene(&g);
    let a = spawn(&mut w, Kind::Vehicle, "GUNR", 0, 4, 9);
    bunker_in(&mut w, a, b);
    // An enemy in sight and within reach of the bunker; the vehicle holds
    // no target on it (as when it is out of the weapon's range).
    let foe = spawn(&mut w, Kind::Vehicle, "GUNR", 1, 3, 6 + 4);
    settle(&mut w);
    w.obj_mut(a).unwrap().target = None;
    let mut br = brain(&w);
    let orders = br.think(&w);
    assert!(deployed(&orders, a), "{orders:?}");
    let _ = foe;
}

// S-0606, PLAN §4.7: the house builds tank bunkers once it can, no more than it wants.
#[test]
fn the_house_builds_bunkers() {
    let mut g = game(1);
    let bunk = g.world.rules().index_of(Kind::Building, "BUNK").unwrap();
    let count = |w: &World| {
        w.objects
            .values()
            .filter(|o| o.owner == Some(HouseId(0)) && o.kind == Kind::Building && o.type_index == bunk)
            .count()
    };
    for _ in 0..8000 {
        g.step();
        if count(&g.world) > 0 {
            break;
        }
    }
    assert!(count(&g.world) >= 1, "built");
    for _ in 0..4000 {
        g.step();
    }
    assert!(count(&g.world) <= bunker::WANTED, "no more than wanted");
}

// S-0606 claim 7 and Q-1304, PLAN §4.7: a bunkered vehicle is not idle, so the plan's waves leave it in its bunker.
#[test]
fn a_bunkered_vehicle_is_not_idle() {
    let g = game(99);
    let (mut w, b) = scene(&g);
    let a = spawn(&mut w, Kind::Vehicle, "GUNR", 0, 4, 9);
    assert!(glasnost_ai::scripted::idle(&w, a));
    bunker_in(&mut w, a, b);
    assert!(!glasnost_ai::scripted::idle(&w, a));
}
