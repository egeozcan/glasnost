//! S-0206 r19 claim 16 (Q-3001 answered): a body turn of a turret-less
//! track vehicle that has a target, no destination and no movement stops at
//! its target's direction when its value equals that direction exactly at
//! the end of one of its turns (`quirks.track_body_turn_stops_at_target`).
//! Synthetic data: the testkit's track vehicle CRUSHER without its turret,
//! an invented wheeled copy WHEELER, and BUGGY as the target.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::LocomotionClass;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::turning::Turning;
use glasnost_sim::world::{ObjectId, Target, World};

const N: i32 = 30;

/// The step of the body turn, facing units per tick.
const STEP: i32 = 1000;

/// The combat ruleset with CRUSHER (a track vehicle firing `Gun`) made
/// turret-less, and WHEELER, the same with locomotion class wheel.
fn rules(on: bool) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.globals.quirks.track_body_turn_stops_at_target = on;
    let c = r.sim.vehicles.get_mut("CRUSHER").unwrap();
    c.turret = Some(false);
    let mut wheel = c.clone();
    wheel.locomotion_class = Some(LocomotionClass::Wheel);
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("WHEELER".into());
    r.sim.vehicles.insert("WHEELER".into(), wheel);
    r
}

/// The direction from `a` to `b` now (S-0204 claim 8).
fn direction(w: &World, a: ObjectId, b: ObjectId) -> u16 {
    let p = w.obj(a).unwrap().pos;
    let q = w.obj(b).unwrap().pos;
    glasnost_num::geom::direction(
        i64::from(q.x - p.x),
        i64::from(q.y - p.y),
        w.rules().quirks.table_direction,
    )
}

/// A vehicle of type `ty` with BUGGY as its target and a body turn that
/// passes exactly through the target's direction four ticks after now, run
/// for twelve ticks. Returns the body's value at the end and that direction.
fn run(on: bool, ty: &str) -> (u16, u16) {
    let mut w = testkit::game_with(&rules(on), N, 7);
    let v = testkit::spawn(&mut w, Kind::Vehicle, ty, 0, testkit::mid(N, 0, 0), 0);
    let foe = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, testkit::mid(N, 3, 1), 0);
    let d = direction(&w, v, foe);
    let target = (i32::from(d) + 4 * STEP).rem_euclid(65536) as u16;
    let t0 = w.tick;
    let o = w.obj_mut(v).unwrap();
    o.target = Some(Target::Object(foe));
    // Rearming the whole run: the firing check's answer ranks above not
    // facing (S-0301 claim 9), so the firing does not turn the body itself
    // (S-0310 claim 9).
    o.combat.rearm_until = t0 + 1000;
    // S-0206 F1: the value at t₀ + ε is ψ₁ − β (Θ − ε), so d at ε = 4.
    o.body = Turning {
        start: (i32::from(d) - 4 * STEP).rem_euclid(65536) as u16,
        target,
        t0,
        duration: 8,
        step: STEP,
    };
    for _ in 0..12 {
        testkit::run(&mut w, 1);
    }
    let o = w.obj(v).unwrap();
    assert_eq!(o.target, Some(Target::Object(foe)), "the target is kept");
    assert!(o.dest.is_none(), "no destination");
    (o.body.value(w.tick), d)
}

// spec: S-0206 r19 claim 16 (the body's current facing becomes its new body target facing in the tick whose value equals the target's direction: claim 3 with Θ = 0; the test at the end of each vehicle turn)
#[test]
fn a_track_body_turn_stops_at_the_target_direction() {
    let (end, d) = run(true, "CRUSHER");
    assert_eq!(end, d, "the body stopped at the target's direction");
}

// spec: S-0206 Edge cases (quirks.track_body_turn_stops_at_target off: the body turn goes on to its own target facing)
#[test]
fn off_the_body_turn_goes_on() {
    let (end, d) = run(false, "CRUSHER");
    assert_eq!(i32::from(end), (i32::from(d) + 4 * STEP).rem_euclid(65536));
}

// spec: S-0206 r19 claim 16 (only a vehicle whose locomotion class is track: a wheel vehicle's turn goes on)
#[test]
fn a_wheel_body_turn_goes_on() {
    let (end, d) = run(true, "WHEELER");
    assert_eq!(i32::from(end), (i32::from(d) + 4 * STEP).rem_euclid(65536));
}
