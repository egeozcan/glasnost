//! #445, the AI side: an AI house's balloon-hover bomber that holds over
//! its target without firing is moved on. The synthetic M7b skirmish
//! (`tests/air_common`) with BOMB, Alpha's balloon-hover jumpjet vehicle
//! whose only weapon BOMBW drops the vertical projectile BOMBP (S-0307
//! claim 18), in variants of the mobile-fire flag (S-0310 claim 7) and of
//! the weapon's fire-while-moving flag (test data of our own). Beta's
//! power plant POWR stands a few cells from the bomber, shown to Alpha;
//! Alpha's brain alone plays.

mod air_common;

use std::sync::Arc;

use air_common::*;
use glasnost_ai::brain::Brain;
use glasnost_ai::fixture_m7b as kit;
use glasnost_map::CellXY;
use glasnost_rules::sim::{Decimal, ProjectileType, WeaponType};
use glasnost_sim::jump::{self, Phase};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

const ALPHA: HouseId = HouseId(0);

/// Where Beta's power plant stands (its top-left cell), from Beta's home:
/// out in the field, a dozen cells from Beta's other objects, so that it is
/// the enemy nearest the bomber.
const PLANT: (i32, i32) = (-10, -28); // nocite: a synthetic scene

/// Where the bomber starts, from the plant: north of it.
const NORTH: (i32, i32) = (0, -4); // nocite: a synthetic scene

/// Where the bomber starts, from the plant: west of it, so that it flies in
/// towards map east.
const WEST: (i32, i32) = (-4, 0); // nocite: a synthetic scene

/// The air_common ruleset with BOMB: the mobile-fire flag `mobile` and the
/// weapon's fire-while-moving flag `fwm` (absent: the default).
fn rules(mobile: bool, fwm: Option<bool>) -> Arc<SimRules> {
    let mut r = air_common::ruleset();
    r.sim.projectiles.insert(
        "BOMBP".into(),
        ProjectileType {
            vertical: Some(true),
            ..Default::default()
        },
    );
    let warhead = r.sim.weapons["Gun"].warhead.clone();
    r.sim.weapons.insert(
        "BOMBW".into(),
        WeaponType {
            damage: Some(40),
            rof: Some(50),
            range: Some(Decimal("2".into())),
            speed: Some(100),
            projectile: Some("BOMBP".into()),
            warhead,
            fire_while_moving: fwm,
            ..Default::default()
        },
    );
    let mut bomb = r.sim.vehicles["DISC"].clone();
    bomb.primary = Some("BOMBW".into());
    bomb.secondary = None;
    bomb.mobile_fire = Some(mobile);
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("BOMB".into());
    r.sim.vehicles.insert("BOMB".into(), bomb);
    Arc::new(SimRules::new(&r, kit::MAP_NAME))
}

/// The scene: the hovering bomber at `from` off Beta's plant, both shown
/// to Alpha. Alpha's start discs are taken out of the game first, so that
/// none of them holds the bomber's hover slot over the plant (S-0502 claim
/// 12): the bomber is Alpha's only flier.
fn scene(mobile: bool, fwm: Option<bool>, from: (i32, i32)) -> (World, ObjectId, ObjectId) {
    let mut w = world_with(rules(mobile, fwm));
    let discs: Vec<ObjectId> = w
        .objects
        .iter()
        .filter(|(id, o)| o.owner == Some(ALPHA) && o.on_map && jump::is_jumpjet(&w, **id))
        .map(|(id, _)| *id)
        .collect();
    for d in discs {
        glasnost_sim::objects::remove(&mut w, d);
    }
    let p = spawn(&mut w, "POWR", Kind::Building, 1, PLANT.0, PLANT.1);
    let b = spawn(
        &mut w,
        "BOMB",
        Kind::Vehicle,
        0,
        PLANT.0 + from.0,
        PLANT.1 + from.1,
    );
    jump::start_in_flight(&mut w, b, Phase::Hovering, HOVER);
    let c = w.obj(b).unwrap().pos.cell();
    glasnost_sim::econ::shroud::area_reveal(&mut w, ALPHA, Pos::centre(c, 0), 12);
    testkit::run(&mut w, 2);
    (w, b, p)
}

/// Alpha's brain plays `ticks` ticks: the orders it gave the bomber, with
/// their ticks, whether the bomber fired, and a trace.
fn play(w: &mut World, b: ObjectId, ticks: i32) -> (Vec<(i32, Order)>, bool, String) {
    let since = w.tick;
    let mut br = Brain::new(w, &kit::map(), ALPHA);
    let mut given = Vec::new();
    let mut trace = Vec::new();
    let mut fired = false;
    for _ in 0..ticks {
        let out = br.think(w);
        for o in orders_to(&out, b) {
            let x = w.obj(b).unwrap();
            trace.push(format!(
                "{} at {:?} {:?} {:?} target {:?} last shot {}: {o:?}",
                w.tick,
                x.pos.cell(),
                x.beh.current,
                jump::phase(w, b),
                x.target,
                x.last_fired
            ));
            given.push((w.tick, o));
        }
        for o in out {
            orders::issue(w, ALPHA, o);
        }
        testkit::run(w, 1);
        // the bomber may fall with the plant it bombs, so its shots are watched on every tick
        fired |= w.obj(b).is_some_and(|o| o.last_fired > since);
    }
    if let Some(x) = w.obj(b) {
        let t = x.target.and_then(|t| match t {
            Target::Object(t) => w
                .obj(t)
                .map(|o| (o.kind, o.type_index, o.owner, o.pos.cell())),
            Target::Cell(_) => None,
        });
        trace.push(format!(
            "end {} at {:?} {:?} {:?} target {:?} {t:?} dest {:?} last shot {}",
            w.tick,
            x.pos.cell(),
            x.beh.current,
            jump::phase(w, b),
            x.target,
            x.dest,
            x.last_fired
        ));
    }
    (given, fired, trace.join("\n"))
}

fn attacks_on(given: &[(i32, Order)], p: ObjectId) -> usize {
    given
        .iter()
        .filter(|(_, o)| {
            matches!(o, Order::Behaviour { behaviour: Behaviour::Attack, target: Some(Target::Object(t)), .. } if *t == p)
        })
        .count()
}

/// The plain moves (not attack-moves, so not a wave's) among `given`, with
/// their ticks.
fn moves(given: &[(i32, Order)]) -> Vec<(i32, CellXY)> {
    given
        .iter()
        .filter(|(_, o)| {
            matches!(
                o,
                Order::Behaviour {
                    attack_move: false,
                    ..
                }
            )
        })
        .filter_map(|(t, o)| move_cell(o).map(|c| (*t, c)))
        .collect()
}

/// Ticks within which a bomber that holds over its target without a shot
/// is moved on in these scenes: the house's waves come much later.
const MOVED_WITHIN: i32 = 1500; // nocite: a synthetic scene

// design: own AI (#445), S-0310 claim 7 and S-0307 claim 18 (a balloon-hover type without the mobile-fire flag gets *moving* while its target is its destination, so no attack order makes it fire): the brain sees its bomber hold over the plant without a shot and moves it away, and over a long window sends it at the plant no more than twice.
#[test]
fn an_ai_bomber_that_never_fires_over_its_target_is_moved_on() {
    let (mut w, b, p) = scene(false, None, NORTH);
    let plant = w.reference_point(p).cell();
    let start = w.tick;
    let (given, fired, trace) = play(&mut w, b, 9000);
    assert!(!fired, "{trace}");
    let moved = moves(&given);
    assert!(
        moved
            .first()
            .is_some_and(|&(t, _)| t - start <= MOVED_WITHIN),
        "moved on: {trace}"
    );
    assert!(attacks_on(&given, p) <= 2, "{trace}");
    assert!(given.len() <= 6, "a bounded number of orders: {trace}");
    // design: own AI (#445): once given up, it is kept out of the house's waves and attacks
    assert_eq!(
        given.last().map(|(t, _)| *t),
        moved.last().map(|(t, _)| *t),
        "the move home is its last order: {trace}"
    );
    let at = w.obj(b).unwrap().pos.cell();
    assert!(
        dist2(at, plant) > 4 * 4,
        "not left over the plant: {at:?} {trace}"
    );
}

// design: own AI (#445) and S-0502 claim 10 (Q-3257: over the plant's reference point the plant lies straight below the bomber, a direction of map east, so a bomb without the fire-while-moving flag falls only after an approach flown towards map east): a bomber that holds over the plant from the north without a shot is sent round to the west and in again, and bombs it.
#[test]
fn an_ai_bomber_whose_bomb_needs_its_heading_comes_again_from_the_west() {
    let (mut w, b, p) = scene(true, Some(false), NORTH);
    let plant = w.reference_point(p).cell();
    let start = w.tick;
    let (given, fired, trace) = play(&mut w, b, 3000);
    assert!(fired, "{trace}");
    let west = moves(&given);
    assert!(
        west.iter()
            .any(|&(t, c)| t - start <= MOVED_WITHIN && c.x < plant.x && c.y == plant.y),
        "sent round to the west: {trace}"
    );
    assert_eq!(attacks_on(&given, p), 2, "in again: {trace}");
}

// design: own AI (#445) and S-0502 claim 10 (Q-3257: flown in towards map east, its body stays on the plant below it): a bomber that keeps bombing the plant is not moved on.
#[test]
fn an_ai_bomber_that_fires_is_left_alone() {
    let (mut w, b, _) = scene(true, None, WEST);
    // a run that ends before the plant falls to its bombs
    let (given, fired, trace) = play(&mut w, b, 600);
    assert!(fired, "{trace}");
    let last = w.obj(b).map_or(i32::MIN, |o| o.last_fired);
    assert!(w.tick - last <= 300, "still bombing: {trace}");
    assert!(moves(&given).is_empty(), "{trace}");
}
