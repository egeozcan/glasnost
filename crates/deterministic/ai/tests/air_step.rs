//! #443 (design: own AI): an AI house's unit that can hit a slow air unit
//! shooting it from close above keeps firing and steps aside, so the
//! attacker has to keep moving to stay over it. A slow attacker is one that
//! stays within two cells of the unit from one think to the next. The
//! step is a few cells away from the attacker; after it, retaliation
//! (S-0215 claim 18) brings the unit back to firing, which the first test
//! checks against a fresh attack order. Beta's FLAKT (an anti-air gun) is
//! shot by Alpha's hovering DISC. Synthetic data only.

mod air_common;

use air_common::*;
use glasnost_map::CellXY;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};

/// Where FLAKT stands, from Beta's home: within 30 cells, on the usable
/// area of the synthetic map (S-0009 claim 4).
const AT: (i32, i32) = (4, -16); // nocite: a synthetic scene

/// The disc hovers two cells east of FLAKT.
const OVER: i32 = 2; // nocite: a synthetic scene

/// Ticks between the disc's shots while FLAKT is in its reach.
const EVERY: i32 = 40; // nocite: a synthetic scene

/// The scene: the world, FLAKT and the disc over it.
fn scene() -> (World, ObjectId, ObjectId) {
    let mut w = world();
    let x = spawn(&mut w, "FLAKT", Kind::Vehicle, 1, AT.0, AT.1);
    let d = disc(&mut w, AT.0 + OVER, AT.1);
    (w, x, d)
}

/// What a tick showed of FLAKT: the tick, its cell and its last shot.
type Seen = Vec<(i32, CellXY, i32)>;

/// Runs `ticks` ticks of the scene with Beta's brain, the disc shooting
/// FLAKT every [`EVERY`] ticks while it is in reach when `shots`; returns
/// Beta's orders to FLAKT per think and what each tick showed of it.
fn brain_run(shots: bool, ticks: i32) -> (Log, Seen) {
    let (mut w, x, d) = scene();
    let mut br = brain(&w);
    let mut n = 0;
    let mut seen = Vec::new();
    let log = run(&mut w, &mut br, x, ticks, |w| {
        if shots && n % EVERY == 0 && in_reach(w, d, x) {
            shoot(w, d, x);
        }
        n += 1;
        let o = w.obj(x).unwrap();
        seen.push((w.tick, o.pos.cell(), o.last_fired));
    });
    (log, seen)
}

/// The move cells of a log.
fn moves(log: &Log) -> Vec<CellXY> {
    log.iter()
        .flat_map(|(_, o)| o.iter().filter_map(move_cell))
        .collect()
}

/// How FLAKT fares after a step given by hand: shot once, then sent three
/// cells west, then, once it stands again, with `attack` given a fresh
/// attack order on the disc and without it left to retaliate; the disc
/// shoots every [`EVERY`] ticks while FLAKT is in reach. Returns whether
/// it fired after the step and what was seen.
fn after_step(attack: bool) -> (bool, String) {
    let (mut w, x, d) = scene();
    let beta = BETA;
    shoot(&mut w, d, x);
    testkit::run(&mut w, EVERY);
    let start = w.obj(x).unwrap().pos.cell();
    let goal = CellXY::new(start.x - 3, start.y);
    orders::issue(
        &mut w,
        beta,
        Order::Behaviour {
            actor: x,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(goal)),
        },
    );
    let mut stood = None;
    for _ in 0..240 {
        testkit::run(&mut w, 1);
        let o = w.obj(x).unwrap();
        if o.pos.cell() == goal && (o.beh.current != Some(Behaviour::Move) || o.dest.is_none()) {
            stood = Some(w.tick);
            break;
        }
    }
    let o = w.obj(x).unwrap();
    let arrived = format!(
        "at {:?} (goal {goal:?}) tick {} stood {stood:?}: {:?} queued {:?} target {:?} dest {:?}",
        o.pos.cell(),
        w.tick,
        o.beh.current,
        o.beh.queued,
        o.target,
        o.dest
    );
    let Some(t0) = stood else {
        return (false, format!("it did not arrive: {arrived}"));
    };
    if attack {
        orders::issue(
            &mut w,
            beta,
            Order::Behaviour {
                actor: x,
                behaviour: Behaviour::Attack,
                attack_move: false,
                target: Some(Target::Object(d)),
                dest: None,
            },
        );
    }
    let mut n = 0;
    for _ in 0..(3 * EVERY) {
        if n % EVERY == 0 && in_reach(&w, d, x) {
            shoot(&mut w, d, x);
        }
        n += 1;
        testkit::run(&mut w, 1);
    }
    let o = w.obj(x).unwrap();
    let fired = o.last_fired > t0;
    (
        fired,
        format!(
            "{arrived}; then {:?} target {:?} last fired {} (stood at {t0})",
            o.beh.current, o.target, o.last_fired
        ),
    )
}

// spec: S-0215 claim 18 and F7 (an actor of a human-controlled owner in guard with no target retaliates, at once when its weapon reaches the attacker); design: own AI (#443): after a step the unit comes back to firing at the slow attacker over it by retaliation alone, as with a fresh attack order, so the brain gives no attack order after the step.
#[test]
fn after_a_step_retaliation_alone_brings_the_unit_back_to_firing() {
    let (fresh, fresh_seen) = after_step(true);
    assert!(fresh, "with a fresh attack order: {fresh_seen}");
    let (alone, alone_seen) = after_step(false);
    assert!(alone, "by retaliation alone: {alone_seen}");
}

// design: own AI (#443): a unit that can hit a slow air unit shooting it from close above steps aside and keeps firing.
#[test]
fn an_anti_air_unit_under_a_slow_bomber_steps_aside_and_keeps_firing() {
    let (quiet, _) = brain_run(false, 300);
    let quiet = moves(&quiet);
    let (log, seen) = brain_run(true, 300);
    let fresh: Vec<CellXY> = moves(&log)
        .into_iter()
        .filter(|c| !quiet.contains(c))
        .collect();
    assert!(!fresh.is_empty(), "it is sent aside: {log:?}");
    let start = seen[0].1;
    let left = seen
        .iter()
        .find(|&&(_, c, _)| dist2(c, start) >= 4)
        .map(|&(t, _, _)| t);
    let Some(left) = left else {
        panic!("it moves: {log:?}");
    };
    let last = seen.last().map_or(-1, |&(_, _, f)| f);
    assert!(
        last > left,
        "it fires after its step (left at {left}, last shot {last}): {log:?}"
    );
}

// design: own AI (#443): under the slow attacker, the house does not order the unit at every think.
#[test]
fn a_unit_under_a_slow_bomber_is_not_ordered_at_every_think() {
    let (quiet, _) = brain_run(false, 300);
    let quiet = moves(&quiet);
    let (log, _) = brain_run(true, 300);
    assert!(
        moves(&log).iter().any(|c| !quiet.contains(c)),
        "it is sent aside: {log:?}"
    );
    let ordered = log.iter().filter(|(_, o)| !o.is_empty()).count();
    assert!(ordered <= 3, "{ordered} of {} thinks: {log:?}", log.len());
}

// design: own AI (#443): a unit of another house under a slow air unit of the AI house is never ordered by its brain.
#[test]
fn a_unit_of_another_house_under_a_slow_bomber_is_never_ordered() {
    let mut w = world();
    let x = spawn(&mut w, "FLAKT", Kind::Vehicle, 0, AT.0, AT.1);
    let d = spawn(&mut w, "DISC", Kind::Vehicle, 1, AT.0 + OVER, AT.1);
    let (t, stale) = (w.tick, w.rules().quirks.stale_suspended_revert);
    w.obj_mut(d).unwrap().beh.force(Behaviour::Sleep, t, stale);
    glasnost_sim::jump::start_in_flight(&mut w, d, glasnost_sim::jump::Phase::Hovering, HOVER);
    testkit::run(&mut w, 2);
    let mut br = brain(&w);
    let mut n = 0;
    let log = run(&mut w, &mut br, x, 120, |w| {
        if n % EVERY == 0 && in_reach(w, d, x) {
            shoot(w, d, x);
        }
        n += 1;
    });
    assert!(log.iter().all(|(_, o)| o.is_empty()), "{log:?}");
}
