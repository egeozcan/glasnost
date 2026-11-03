//! Claim tests of S-0514 revision 16, claim 18 (*Order against the erasure
//! update*, Q-2377 answered; M8 step A3 intake 84): within one turn of an
//! object being erased, an infantry unit makes its sparkle before the
//! turn's erasure update and a vehicle, aircraft or building after it, so
//! a turn whose update removes the object still sparkles in a sparkle
//! tick, and a release in the update that ends the erasure leaves a
//! vehicle, aircraft or building with no sparkle. Scenario E11 part e.
//! Synthetic rules (`tests/temporal_common`): every name and number is
//! invented. The ledger is `docs/m8-claims/intake84.md`.

#[path = "temporal_common/mod.rs"]
mod temporal_common;

use glasnost_rules::schema::Ruleset;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::rules::Kind;
use glasnost_sim::temporal;
use glasnost_sim::testkit;
use glasnost_sim::transport;
use glasnost_sim::world::{ObjectId, Pos, World};
use temporal_common::*;

/// S-0514 Constants: the sparkle period, on the global tick number.
const SPARKLE: i32 = 24;

/// E11 part e1's targets: an infantry unit, a vehicle, an aircraft landed
/// on flat ground and a test building with no damage-fire positions.
const TARGETS: [(Kind, &str); 4] = [
    (Kind::Infantry, "BYSTANDER"),
    (Kind::Vehicle, "VICT"),
    (Kind::Aircraft, "PLANE"),
    (Kind::Building, "TOWER"),
];

/// The ruleset with a landed-aircraft type PLANE, every target strong
/// enough that no erasure completes by itself, and, with `ranged`, the
/// sparkle and warp-away animations given random rate pairs with different
/// ranges, so that each creation takes a draw (E11 part e2).
fn rules(ranged: bool) -> Ruleset {
    let mut r = temporal_common::ruleset();
    r.sim.aircraft.insert(
        "PLANE".into(),
        glasnost_rules::sim::UnitType {
            strength: Some(100_000),
            speed: Some(30),
            rot: Some(12),
            locomotor: Some(glasnost_rules::sim::Locomotor::Fly),
            ..Default::default()
        },
    );
    r.registries
        .entry("aircraft".into())
        .or_default()
        .push("PLANE".into());
    r.sim.vehicles.get_mut("VICT").unwrap().strength = Some(100_000);
    r.sim.buildings.get_mut("TOWER").unwrap().strength = Some(100_000);
    if ranged {
        r.sim.animations.get_mut("SPARK").unwrap().random_rate = Some([450, 225]);
        r.sim.animations.get_mut("WARP").unwrap().random_rate = Some([300, 100]);
    }
    r
}

/// S-0514 claim 18's place of a sparkle for a target of `kind` at `p`:
/// 120 leptons farther along each horizontal axis for an infantry unit or
/// vehicle, the position itself for an aircraft or a building with no
/// damage-fire positions.
fn placed(kind: Kind, p: Pos) -> Pos {
    match kind {
        Kind::Infantry | Kind::Vehicle => Pos::new(p.x + 120, p.y + 120, p.z), // spec: S-0514 Constants (mobile sparkle displacement 120 leptons)
        _ => p,
    }
}

/// The sparkle (SPARK) and warp-away (WARP) animations made in the last
/// tick, in order of creation, with their positions and whether their
/// creation took a draw.
fn made(w: &World) -> Vec<(&str, Pos, bool)> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Anim {
                name, pos, delay, ..
            } if name == "SPARK" || name == "WARP" => Some((name.as_str(), *pos, delay.is_some())),
            _ => None,
        })
        .collect()
}

/// Runs to the next tick divisible by 24 (still to run).
fn to_sparkle_tick(w: &mut World) {
    let next = (w.tick / SPARKLE + 1) * SPARKLE;
    run_to(w, next);
    assert_eq!(w.tick.rem_euclid(SPARKLE), 0);
}

/// A target of `kind` being erased by one infantry attacker, made to
/// complete in the next tick divisible by 24 (its amount set to 1 just
/// before that tick); runs that tick and returns the animations it made,
/// with the target's last position.
fn removal_tick(kind: Kind, name: &str, ranged: bool) -> (Vec<(String, Pos, bool)>, Pos) {
    let mut w = game(&rules(ranged));
    let x = spawn(&mut w, kind, name, 1, at(0, 0));
    let a = attacker(&mut w, "ERASER", 0, at(-12, -12));
    erase(&mut w, a, x);
    assert!(erased(&w, x), "{name}: set-up: being erased");
    to_sparkle_tick(&mut w);
    assert!(erased(&w, x), "{name}: set-up: still being erased");
    let last = w.obj(x).unwrap().pos;
    w.temporal.locks.get_mut(&x).unwrap().amount = 1;
    w.effects.clear();
    testkit::run(&mut w, 1);
    assert!(w.obj(x).is_none(), "{name}: removed in the sparkle tick");
    let got = made(&w)
        .into_iter()
        .map(|(n, p, d)| (n.to_string(), p, d))
        .collect();
    (got, last)
}

#[test]
fn e1_a_turn_whose_erasure_update_removes_the_target_sparkles_in_every_kind() {
    // spec: S-0514 r16 claim 18 (Order against the erasure update, Q-2377: a completion in the update does not end the state for this purpose, so a turn whose erasure update removes the object still makes the sparkle in a sparkle tick, in every kind, placed by the rule for its kind from the object's last position; E11 part e1: 120 leptons farther along each horizontal axis for the infantry unit and the vehicle, the own position for the aircraft and the building) and claim 12 (the warp-away animation at completion)
    for (kind, name) in TARGETS {
        let (got, last) = removal_tick(kind, name, false);
        let sparkles: Vec<Pos> = got
            .iter()
            .filter(|(n, ..)| n == "SPARK")
            .map(|(_, p, _)| *p)
            .collect();
        assert_eq!(
            sparkles,
            vec![placed(kind, last)],
            "{name}: one sparkle in the removal tick, from the last position"
        );
    }
}

#[test]
fn e2_the_sparkles_draw_comes_before_the_updates_for_infantry_and_after_them_for_other_kinds() {
    // spec: S-0514 r16 claim 18 (Order against the erasure update, Q-2377: an infantry unit makes its sparkle before that turn's erasure update, a vehicle, aircraft or building after it; any draw the sparkle's creation takes comes before every draw of the update, such as the creation of claim 12's warp-away animation, for infantry and after them for the other kinds, after the removal and the warp-away animation; E11 part e2) and S-0109 claim 3 (a random rate pair whose range holds more than one delay: one ranged creation draw)
    for (kind, name) in TARGETS {
        let (got, _) = removal_tick(kind, name, true);
        let order: Vec<(&str, bool)> = got.iter().map(|(n, _, d)| (n.as_str(), *d)).collect();
        let want = if kind == Kind::Infantry {
            vec![("SPARK", true), ("WARP", true)]
        } else {
            vec![("WARP", true), ("SPARK", true)]
        };
        assert_eq!(
            order, want,
            "{name}: the creation order, each with its draw"
        );
    }
}

/// A leading erasing attacker alone inside an open-topped transport whose
/// update in the next tick divisible by 24 releases its beam (claim 11):
/// whether the target sparkles in that tick.
fn release_tick_sparkles(kind: Kind, name: &str) -> bool {
    let mut w = game(&rules(false));
    let x = spawn(&mut w, kind, name, 1, at(0, 0));
    let t = spawn(&mut w, Kind::Vehicle, "OPEN", 0, at(-2, 0));
    let p = attacker(&mut w, "ERASER", 0, at(-2, 1));
    transport::board(&mut w, p, t);
    erase(&mut w, p, x);
    assert!(temporal::holds(&w, p, x), "{name}: set-up: locked");
    to_sparkle_tick(&mut w);
    assert!(erased(&w, x), "{name}: set-up: still being erased");
    // Beyond the warp distance: the tick's update releases the beam, the last one.
    let xp = w.obj(x).unwrap().pos;
    w.obj_mut(p).unwrap().pos = Pos::new(xp.x - 2000, xp.y, xp.z);
    w.effects.clear();
    testkit::run(&mut w, 1);
    assert!(!erased(&w, x), "{name}: released in the sparkle tick");
    assert!(w.obj(x).is_some());
    made(&w).iter().any(|(n, ..)| *n == "SPARK")
}

#[test]
fn e3_a_release_in_the_update_leaves_no_sparkle_but_at_an_infantry_target() {
    // spec: S-0514 r16 claim 18 (Order against the erasure update, Q-2377: the object's state at the point of its sparkle decides; a release in the update that ends the erasure, claim 11's last attacker letting go, leaves a vehicle, aircraft or building that is not phasing in with no sparkle in that turn, while an infantry unit makes it; E11 part e3, the second building aside) and claim 11 (a leading attacker inside an open-topped transport lets go beyond the warp distance)
    for (kind, name) in TARGETS {
        assert_eq!(
            release_tick_sparkles(kind, name),
            kind == Kind::Infantry,
            "{name}: a sparkle in the releasing tick only at the infantry target"
        );
    }
}

#[test]
fn a_joiner_taking_over_in_the_update_keeps_the_vehicles_sparkle() {
    // spec: S-0514 r16 claim 18 (the object's state after the update decides for a vehicle: still being erased under a new leader, it sparkles) and claims 9 and 11 (the leader's distance release hands the erasure to the most recent joiner)
    let mut w = game(&rules(false));
    let x = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(0, 0));
    let t = spawn(&mut w, Kind::Vehicle, "OPEN", 0, at(-2, 0));
    let p = attacker(&mut w, "ERASER", 0, at(-2, 1));
    transport::board(&mut w, p, t);
    let j: ObjectId = attacker(&mut w, "ERASER2", 0, at(-3, 0));
    erase(&mut w, p, x);
    erase(&mut w, j, x);
    to_sparkle_tick(&mut w);
    let xp = w.obj(x).unwrap().pos;
    w.obj_mut(p).unwrap().pos = Pos::new(xp.x - 2000, xp.y, xp.z);
    w.effects.clear();
    testkit::run(&mut w, 1);
    assert!(!temporal::holds(&w, p, x), "the leader let go");
    assert!(erased(&w, x), "the joiner leads on");
    let sparkles: Vec<Pos> = made(&w)
        .into_iter()
        .filter(|(n, ..)| *n == "SPARK")
        .map(|(_, p, _)| p)
        .collect();
    assert_eq!(sparkles, vec![placed(Kind::Vehicle, xp)]);
}
