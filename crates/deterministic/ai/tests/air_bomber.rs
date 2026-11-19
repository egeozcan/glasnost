//! #445: an AI house's balloon-hover bomber at an enemy base. The synthetic
//! M7b skirmish (`tests/air_common`) with BOMB, Alpha's balloon-hover
//! jumpjet vehicle whose only weapon BOMBW drops the vertical projectile
//! BOMBP (S-0307 claim 18, S-0306 claim 11), in variants of its reach, of
//! the mobile-fire flag (S-0310 claim 7) and of the side it comes from
//! (test data of our own). Beta's power plant POWR (two cells by two)
//! stands a few cells from the bomber, shown to Alpha. Each run keeps a
//! trace of the bomber's behaviour, target, destination, flight state, last
//! shot, the weapon it would use, the firing check's answers and its aim
//! error, and of the orders Alpha's brain gave it; the traces are the
//! assertions' messages.

mod air_common;

use std::sync::Arc;

use air_common::*;
use glasnost_ai::brain::Brain;
use glasnost_ai::fixture_m7b as kit;
use glasnost_rules::sim::{Decimal, ProjectileType, WeaponType};
use glasnost_sim::combat::check::{self, Answer};
use glasnost_sim::combat::{range, weapon};
use glasnost_sim::jump::{self, Phase};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

/// The AI house that owns the bomber.
const ALPHA: HouseId = HouseId(0);

/// Where Beta's power plant stands (its top-left cell), from Beta's home.
const PLANT: (i32, i32) = (6, -14); // nocite: a synthetic scene

/// Where the bomber starts, from the plant: west, east, north and south of
/// it, a few cells off its foundation.
const SIDES: [(&str, i32, i32); 4] = [
    ("west", -4, 0),
    ("east", 5, 0),
    ("north", 0, -4),
    ("south", 0, 5),
]; // nocite: a synthetic scene

/// Ticks a run lasts.
const TICKS: i32 = 600; // nocite: a synthetic scene

/// A bomber variant: the reach of its bomb in cells (a decimal of the
/// rules), whether its type has the mobile-fire flag (S-0310 Terms; the
/// flag is given either way, its absent-key default is yes) and the side
/// it starts on.
#[derive(Clone, Copy, Debug)]
struct Variant {
    range: &'static str,
    mobile_fire: bool,
    side: (&'static str, i32, i32),
}

/// The air_common ruleset with BOMB of variant `v`.
fn rules(v: Variant) -> Arc<SimRules> {
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
            range: Some(Decimal(v.range.into())),
            speed: Some(100),
            projectile: Some("BOMBP".into()),
            warhead,
            ..Default::default()
        },
    );
    let mut bomb = r.sim.vehicles["DISC"].clone();
    bomb.primary = Some("BOMBW".into());
    bomb.secondary = None;
    bomb.mobile_fire = Some(v.mobile_fire);
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("BOMB".into());
    r.sim.vehicles.insert("BOMB".into(), bomb);
    Arc::new(SimRules::new(&r, kit::MAP_NAME))
}

/// The scene: the world, the hovering bomber in *guard* and Beta's plant,
/// shown to Alpha (a reveal, as a map trigger gives).
fn scene(v: Variant) -> (World, ObjectId, ObjectId) {
    let mut w = world_with(rules(v));
    let p = spawn(&mut w, "POWR", Kind::Building, 1, PLANT.0, PLANT.1);
    let (_, dx, dy) = v.side;
    let b = spawn(&mut w, "BOMB", Kind::Vehicle, 0, PLANT.0 + dx, PLANT.1 + dy);
    jump::start_in_flight(&mut w, b, Phase::Hovering, HOVER);
    let c = w.obj(b).unwrap().pos.cell();
    glasnost_sim::econ::shroud::area_reveal(&mut w, ALPHA, Pos::centre(c, 0), 12);
    testkit::run(&mut w, 2);
    (w, b, p)
}

/// The bomber's aim error against its target in facing units (S-0310 F1:
/// the direction from its reference point to the point the check aims at,
/// S-0204 claim 8, against its body facing), when it has a target.
fn aim_error(w: &World, b: ObjectId) -> Option<i32> {
    let t = w.obj(b)?.target?;
    let from = w.reference_point(b);
    let to = range::target_point(w, t);
    let d = glasnost_num::geom::direction(
        i64::from(to.x - from.x),
        i64::from(to.y - from.y),
        w.rules().quirks.table_direction,
    );
    let f = w.body_facing(b);
    let e = (i32::from(d) - i32::from(f)).rem_euclid(65536);
    Some(if e > 32768 { 65536 - e } else { e })
}

/// The firing check's answer for the bomber now, range test included.
fn answer(w: &World, b: ObjectId) -> Answer {
    let t = w.obj(b).and_then(|o| o.target);
    let (slot, _) = weapon::would_use(w, b, t);
    check::check(w, b, t, slot, true)
}

/// One line of the trace: the bomber at this tick, and the orders given it.
fn line(w: &World, b: ObjectId, given: &[Order]) -> String {
    let Some(o) = w.obj(b) else {
        return format!("{}: gone", w.tick);
    };
    let (slot, wp) = weapon::would_use(w, b, o.target);
    let lasting = check::check_lasting(w, b, o.target, slot);
    let aim = o.target.map(|t| range::target_point(w, t));
    format!(
        "{} at {:?} {:?} queued {:?} target {:?} dest {:?} {:?} last shot {} slot {slot} weapon {} check {:?} lasting {lasting:?} body facing {} aims at {:?} error {:?} idle {} orders {given:?}",
        w.tick,
        (o.pos.x, o.pos.y),
        o.beh.current,
        o.beh.queued,
        o.target,
        o.dest,
        jump::phase(w, b),
        o.last_fired,
        wp.is_some(),
        answer(w, b),
        w.body_facing(b),
        aim.map(|p| (p.x, p.y)),
        aim_error(w, b),
        glasnost_ai::scripted::idle(w, b),
    )
}

/// Whether the bomber fired after `since`.
fn fired(w: &World, b: ObjectId, since: i32) -> bool {
    w.obj(b).is_some_and(|o| o.last_fired > since)
}

/// A run with an attack order on the plant given by hand: whether the
/// bomber fired, the world at the end and the trace.
fn by_hand(v: Variant) -> (bool, World, ObjectId, ObjectId, String) {
    let (mut w, b, p) = scene(v);
    let since = w.tick;
    orders::issue(
        &mut w,
        ALPHA,
        Order::Behaviour {
            actor: b,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(p)),
            dest: None,
        },
    );
    let mut trace = vec![format!("{v:?}")];
    for n in 0..TICKS {
        testkit::run(&mut w, 1);
        if n % 15 == 0 {
            trace.push(line(&w, b, &[]));
        }
        if fired(&w, b, since) {
            trace.push(line(&w, b, &[]));
            break;
        }
    }
    (fired(&w, b, since), w, b, p, trace.join("\n"))
}

// spec: S-0307 claim 18 (a balloon-hover vehicle with a vertical projectile takes its target as its destination, and keeps it), S-0310 claims 1, 2 and 7 (the body aims at the target's reference point; τ = 2048; for a balloon-hover type the fire-while-moving condition reads "its locomotor is moving it"), S-0204 claim 4 (a building's reference point is the centre of its foundation): a bomber whose bomb reaches two cells, of a type with the mobile-fire flag, bombs the plant on its way in only from the sides where its heading, towards the plant's position, stays within τ of the plant's reference point once in reach (Q-3257).
#[test]
fn a_bomber_that_reaches_from_afar_bombs_the_plant_from_some_sides() {
    let mut seen = Vec::new();
    let mut traces = Vec::new();
    for side in SIDES {
        let v = Variant {
            range: "2",
            mobile_fire: true,
            side,
        };
        let (fired, _, _, _, trace) = by_hand(v);
        seen.push((side.0, fired));
        traces.push(trace);
    }
    // spec: S-0502 claim 16 (r34): its heading points at the plant's reference point, so it bombs on its way in from every side
    let expected = vec![
        ("west", true),
        ("east", true),
        ("north", true),
        ("south", true),
    ];
    assert_eq!(seen, expected, "{}", traces.join("\n\n"));
}

// spec: S-0307 claim 18 (the target is kept as the destination), S-0502 claim 16 (r34: its movement point is the plant's reference point), S-0310 claims 1, 2 and 7 (its body aims, τ = 2048; its bomb has the fire-while-moving flag): a bomber whose bomb reaches half a cell flies towards the plant's reference point with its body on it and bombs the plant on its way in, from every side (Q-3257).
#[test]
fn a_bomber_that_reaches_half_a_cell_bombs_the_plant_on_its_way_in() {
    for side in SIDES {
        let v = Variant {
            range: "0.5",
            mobile_fire: true,
            side,
        };
        let (fired, w, b, p, trace) = by_hand(v);
        // spec: S-0502 claim 16 (r34)
        assert!(fired, "{trace}");
        let o = w.obj(b).unwrap();
        let point = w.reference_point(p);
        assert_eq!(o.beh.current, Some(Behaviour::Attack), "{trace}");
        assert_eq!(o.dest, Some(Target::Object(p)), "{trace}");
        // spec: S-0502 claim 16 (r34)
        let hover = o.jump.as_ref().and_then(|j| j.dest);
        // spec: S-0502 claim 16 (r34)
        assert_eq!(
            hover.map(|h| (h.x, h.y)),
            Some((point.x, point.y)),
            "{trace}"
        );
        // spec: S-0502 claim 16 (r34)
        assert!(aim_error(&w, b).is_some_and(|e| e <= 2048), "{trace}");
    }
}

// spec: S-0310 claim 7 (a type without the mobile-fire flag is refused while it has a destination) and S-0307 claim 18 (the target is kept as the destination): a bomber of a type without the mobile-fire flag never bombs the plant (Q-3257).
#[test]
fn a_bomber_without_mobile_fire_never_bombs_the_plant() {
    for range in ["0.5", "2"] {
        let v = Variant {
            range,
            mobile_fire: false,
            side: SIDES[0],
        };
        let (fired, w, b, p, trace) = by_hand(v);
        assert!(!fired, "{trace}");
        assert_eq!(w.obj(b).unwrap().dest, Some(Target::Object(p)), "{trace}");
        assert_eq!(answer(&w, b), Answer::Moving, "{trace}");
    }
}

// design: own AI (#445): Alpha's brain gives its idle bomber that sees an enemy building near it one attack order on it, which the bomber takes (its lasting check is neither illegal nor impossible), and gives it nothing more while the bomber holds that attack, stalled or not.
#[test]
fn an_ai_bomber_at_an_enemy_base_is_sent_at_it_once() {
    // spec: S-0502 claim 16 (r34)
    for (range, fires) in [("2", true), ("0.5", true)] {
        let v = Variant {
            range,
            mobile_fire: true,
            side: SIDES[0],
        };
        let (mut w, b, p) = scene(v);
        let since = w.tick;
        let mut br = Brain::new(&w, &kit::map(), ALPHA);
        let mut trace = vec![format!("{v:?}")];
        let mut given = Vec::new();
        for _ in 0..TICKS {
            let thinks = glasnost_ai::scripted::thinks_at(ALPHA, w.tick);
            let out = br.think(&w);
            let to = orders_to(&out, b);
            if thinks {
                trace.push(line(&w, b, &to));
            }
            given.extend(to);
            for o in out {
                orders::issue(&mut w, ALPHA, o);
            }
            testkit::run(&mut w, 1);
            if fired(&w, b, since) {
                break;
            }
        }
        let trace = trace.join("\n");
        let attack = Order::Behaviour {
            actor: b,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(p)),
            dest: None,
        };
        assert_eq!(given, vec![attack], "{trace}");
        let o = w.obj(b).unwrap();
        assert_eq!(o.target, Some(Target::Object(p)), "{trace}");
        let (slot, _) = weapon::would_use(&w, b, o.target);
        let lasting = check::check_lasting(&w, b, o.target, slot);
        assert!(
            !matches!(lasting, Answer::Illegal | Answer::Impossible),
            "{trace}"
        );
        assert_eq!(fired(&w, b, since), fires, "{trace}");
    }
}
