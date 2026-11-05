//! The own AI's use of jumpjets (S-0502, PLAN §4.7): on the strait map, where
//! no ground unit walks from one base to the other, jumpjet units fly over
//! the water to the enemy; the brain builds one when it sees an enemy it
//! cannot walk to, and sends its waves' jumpjets to the enemy's own cell.

use std::sync::Arc;

use glasnost_ai::brain::Brain;
use glasnost_ai::brain::default_plan;
use glasnost_ai::fixture;
use glasnost_ai::game::Game;
use glasnost_ai::jumpjets::across_obstacle;
use glasnost_map::CellXY;
use glasnost_rules::sim::{LocomotionClass, Locomotor, MovementZone, UnitType};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

/// The fixture's rules with JETI, an armed jumpjet rifleman the barracks
/// builds, and JETX, the same type nobody can build.
fn rules() -> Arc<SimRules> {
    let mut r = fixture::ruleset();
    r.registries
        .entry("infantry".into())
        .or_default()
        .extend(["JETI".to_string(), "JETX".to_string()]);
    let jumper = |buildable: bool| UnitType {
        strength: Some(100),
        speed: Some(6),
        locomotion_class: Some(LocomotionClass::Foot),
        movement_zone: Some(MovementZone::Fly),
        locomotor: Some(Locomotor::Jumpjet),
        jumpjet: Some(true),
        sight: Some(4),
        primary: Some("Rifle".into()),
        tech_level: Some(if buildable { 1 } else { 99 }),
        cost: Some(100),
        owner: Some(vec!["Alphaland".into(), "Betaland".into()]),
        jumpjet_height: Some(300),
        ..Default::default()
    };
    // The barracks has a door kind, as every real one has: only such a building qualifies for S-0417 claim 15's ascent report.
    r.sim.buildings.get_mut("PILL").unwrap().econ.door_first = Some(true);
    r.sim.infantry.insert("JETI".into(), jumper(true));
    r.sim.infantry.insert("JETX".into(), jumper(false));
    Arc::new(SimRules::new(&r, "synthetic-skirmish"))
}

fn game(seed: u32) -> Game {
    Game::new(
        rules(),
        &fixture::map_strait(),
        "synthetic-strait",
        fixture::catalogue(),
        fixture::setup(seed),
    )
    .expect("the strait skirmish loads")
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

/// A world copy where house 0 has an idle jumpjet near its base and sees an
/// enemy object across the strait; returns the world, the jumpjet and the
/// enemy objects across.
fn scene(g: &Game) -> (World, ObjectId, Vec<ObjectId>) {
    let mut w = g.world.clone();
    let home = first_cell(&w, 0);
    let there = first_cell(&w, 1);
    assert!(
        across_obstacle(&w, home, there),
        "the strait separates the bases"
    );
    let jet = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "JETX",
        0,
        CellXY::new(home.x, home.y + 5),
        0,
    );
    // A reveal of the area (as a map trigger gives) shows the enemy base to the house.
    let p = glasnost_sim::world::Pos::centre(there, 0);
    glasnost_sim::econ::shroud::area_reveal(&mut w, HouseId(0), p, 8);
    testkit::run(&mut w, 2);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    let across: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| o.owner == Some(HouseId(1)) && o.on_map)
        .map(|o| o.id)
        .collect();
    (w, jet, across)
}

// S-0502, PLAN §4.7: an idle armed jumpjet is sent against the nearest seen enemy that no walking unit of the house can reach, with the order a person gives, and it flies there over the water and attacks it.
#[test]
fn sends_a_jumpjet_over_the_strait() {
    let mut g = game(5);
    ready(&mut g);
    let (mut w, jet, across) = scene(&g);
    let mut brain = Brain::new(&w, &fixture::map_strait(), HouseId(0));
    let orders = brain.think(&w);
    let attack = orders.iter().find_map(|o| match o {
        Order::Behaviour {
            actor,
            behaviour: Behaviour::Attack,
            target: Some(Target::Object(e)),
            ..
        } if *actor == jet => Some(*e),
        _ => None,
    });
    let target = attack.expect("the jumpjet is sent against an enemy");
    assert!(across.contains(&target));
    let from = w.obj(jet).unwrap().pos.cell();
    let at = w.obj(target).unwrap().pos.cell();
    assert!(across_obstacle(&w, from, at), "across the water");
    assert_eq!(brain.special_uses().jumps, 1);
    for o in orders {
        glasnost_sim::orders::issue(&mut w, HouseId(0), o);
    }
    let near = |w: &World| {
        w.obj(jet).is_none_or(|o| {
            let c = o.pos.cell();
            let t = w.obj(target).map_or(at, |t| t.pos.cell());
            (i64::from(c.x - t.x)).pow(2) + (i64::from(c.y - t.y)).pow(2) <= 36
        })
    };
    for _ in 0..1500 {
        testkit::run(&mut w, 1);
        if near(&w) {
            break;
        }
    }
    assert!(near(&w), "it flew over the strait to its target");
}

// S-0502, PLAN §4.7: it sees only what its house sees: an enemy on a cell the house has not explored is not chosen.
#[test]
fn flies_only_at_what_it_has_explored() {
    let mut g = game(5);
    ready(&mut g);
    let mut w = g.world.clone();
    let home = first_cell(&w, 0);
    let jet = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "JETX",
        0,
        CellXY::new(home.x, home.y + 5),
        0,
    );
    let orders = Brain::new(&w, &fixture::map_strait(), HouseId(0)).think(&w);
    assert!(
        !orders.iter().any(|o| matches!(o, Order::Behaviour { actor, behaviour: Behaviour::Attack, .. } if *actor == jet)),
        "{orders:?}"
    );
}

// S-0502, PLAN §4.7: a wave's jumpjets go to the enemy's own cell, while a walking unit that cannot walk there waits at home for a ferry.
#[test]
fn a_wave_sends_its_jumpjets_across() {
    let mut g = game(5);
    ready(&mut g);
    let (mut w, jet, across) = scene(&g);
    let home = first_cell(&w, 0);
    let walker = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        0,
        CellXY::new(home.x + 1, home.y + 5),
        0,
    );
    // The house's guard stays home (the units nearest it), and no wave goes
    // before the opening is over (M8 step A4 round 8): two tanks beside
    // home, and the clock past the opening.
    for dx in [-2, 2] {
        testkit::spawn(
            &mut w,
            Kind::Vehicle,
            "GUNR",
            0,
            CellXY::new(home.x + dx, home.y),
            0,
        );
    }
    assert_eq!(glasnost_ai::scripted::GUARD_MIN, 2);
    let period = glasnost_ai::scripted::THINK_PERIOD;
    w.tick = (glasnost_ai::scripted::OPENING + period - 1) / period * period;
    let mut plan = default_plan();
    plan.wave = 1;
    let mut brain = Brain::with_plan(&w, &fixture::map_strait(), HouseId(0), plan);
    let orders = brain.think(&w);
    let dest_of = |who: ObjectId| {
        orders.iter().find_map(|o| match o {
            Order::Behaviour {
                actor,
                behaviour: Behaviour::Move,
                attack_move: true,
                dest: Some(Target::Cell(c)),
                ..
            } if *actor == who => Some(*c),
            _ => None,
        })
    };
    let goal = dest_of(jet).expect("the jumpjet joins the wave");
    let at = |id: ObjectId| w.obj(id).unwrap().pos.cell();
    assert!(
        across.iter().any(|&e| at(e) == goal),
        "the jumpjet goes to the enemy's own cell"
    );
    assert!(across_obstacle(&w, at(walker), goal));
    // The walker, which cannot walk there, stays at home for a ferry.
    assert_eq!(dest_of(walker), None, "the walker waits at home");
}

// spec: S-0417 claim 15 (answers Q-1070: a released jumpjet unit ends its factory contact and tether in its ascent, so no interim hold remains), PLAN §4.7: with no jumpjet unit, a barracks and a seen enemy no walking unit reaches, the house orders a jumpjet unit from its sidebar like a person, builds it, and the barracks is free for the next unit once the unit has climbed away.
#[test]
fn builds_a_jumpjet_unit_and_its_factory_stays_free() {
    let mut g = game(5);
    ready(&mut g);
    let (mut w, jet, _) = scene(&g);
    glasnost_sim::objects::remove(&mut w, jet);
    // The barracks the type needs (the fixture's pillbox-named building of the tech tree).
    let home = first_cell(&w, 0);
    testkit::spawn(
        &mut w,
        Kind::Building,
        "PILL",
        0,
        CellXY::new(home.x + 5, home.y),
        0,
    );
    w.houses[0].credits = 5000;
    testkit::run(&mut w, 1);
    while w.tick % 15 != 0 {
        testkit::run(&mut w, 1);
    }
    let idx = w.rules().index_of(Kind::Infantry, "JETI").unwrap();
    let t = glasnost_sim::econ::TypeRef {
        kind: Kind::Infantry,
        index: idx,
    };
    assert!(glasnost_sim::econ::tech::buildable(&w, HouseId(0), t));
    let mut brain = Brain::new(&w, &fixture::map_strait(), HouseId(0));
    let mut ordered = false;
    let mut built: Option<ObjectId> = None;
    let mut freed = false;
    'run: for _ in 0..400 {
        let orders = brain.think(&w);
        ordered |= orders.iter().any(
            |o| matches!(o, Order::Build { what } if what.kind == Kind::Infantry && what.index == idx),
        );
        for o in orders {
            glasnost_sim::orders::issue(&mut w, HouseId(0), o);
        }
        for _ in 0..15 {
            testkit::run(&mut w, 1);
            if built.is_none() {
                built = w
                    .objects
                    .values()
                    .find(|o| {
                        o.owner == Some(HouseId(0))
                            && o.kind == Kind::Infantry
                            && o.type_index == idx
                            && o.on_map
                    })
                    .map(|o| o.id);
            }
            if let Some(u) = built
                && w.obj(u)
                    .is_some_and(|o| o.contact.is_none() && o.econ.exit.is_none())
            {
                // The link ended while the unit was still climbing: the ascent report, not an arrival or an order.
                freed = glasnost_sim::jump::phase(&w, u)
                    == Some(glasnost_sim::jump::Phase::Ascending)
                    && w.height_above_surface(u) > 0;
                break 'run;
            }
        }
    }
    assert!(ordered, "the house orders the jumpjet unit");
    assert!(built.is_some(), "the unit is built and released");
    assert!(freed, "its factory contact and tether ended");
}
