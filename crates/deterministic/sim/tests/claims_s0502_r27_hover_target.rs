//! Claim tests of S-0502 r27 (Q-3252, #447): a hovering jumpjet vehicle and
//! its target. Claim 10, *A vehicle with a target*, and the Edge cases entry
//! *Body on the heading while hovering*
//! (`quirks.hovering_jumpjet_body_on_heading`). Synthetic rules on the
//! jumpjet fixtures: every name and number is invented; no game data.

mod jump_common;

use glasnost_num::geom::{Facing, delta, direction};
use glasnost_rules::sim::WeaponType;
use glasnost_sim::jump::{self, Phase};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::turning::Turning;
use glasnost_sim::world::{ObjectId, Target, World};
use jump_common::*;

/// The fixtures plus three jumpjet vehicles with a target to shoot:
/// HVGUN (no turret, a weapon that is not omni-fire), HVOMNI (no turret,
/// an omni-fire weapon) and HVTUR (a turret, the weapon that is not
/// omni-fire). Each has the body turn-rate field 4.
fn game_with_hoverers(on: bool) -> World {
    let mut r = ruleset();
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .extend(["HVGUN", "HVOMNI", "HVTUR"].map(String::from));
    let s = &mut r.sim;
    let base = s.vehicles["JV"].clone();
    for (name, weapon, turret) in [
        ("HVGUN", "Gun", false),
        ("HVOMNI", "OmniGun", false),
        ("HVTUR", "Gun", true),
    ] {
        let mut v = base.clone();
        v.primary = Some(weapon.into());
        v.rot = Some(4);
        v.turret = Some(turret);
        s.vehicles.insert(name.into(), v);
    }
    let gun = WeaponType {
        damage: Some(1),
        rof: Some(20),
        range: Some(d("8")),
        speed: Some(100),
        projectile: Some("Bullet".into()),
        warhead: Some("SA".into()),
        ..Default::default()
    };
    let omni = WeaponType {
        omni_fire: Some(true),
        ..gun.clone()
    };
    s.weapons.insert("Gun".into(), gun);
    s.weapons.insert("OmniGun".into(), omni);
    let mut w = testkit::game_with(&r, SIZE, 7);
    let mut rules = (*w.rules_arc()).clone();
    rules.quirks.hovering_jumpjet_body_on_heading = on;
    w.rules = Some(std::sync::Arc::new(rules));
    testkit::run(&mut w, 4);
    w
}

/// A hovering vehicle of type `name` at height 300 with an enemy CART as its
/// target four cells away, a quarter turn off its movement heading, its body
/// on that heading, and no order-level destination. With `committed` it
/// keeps a movement commitment to the centre of its own cell (S-0502 Terms,
/// *moving*: the state a flight that ended in a hover to attack leaves).
fn hoverer(w: &mut World, name: &str, committed: bool) -> (ObjectId, ObjectId) {
    let e = put(w, "CART", Kind::Vehicle, 1, 0, 4);
    let u = put(w, name, Kind::Vehicle, 0, 0, 0);
    jump::start_in_flight(w, u, Phase::Hovering, 300);
    if committed {
        commit_here(w, u);
    }
    let h = heading(w, u);
    let o = w.obj_mut(u).unwrap();
    o.body = Turning::at(h);
    o.dest = None;
    o.target = Some(Target::Object(e));
    (u, e)
}

fn heading(w: &World, u: ObjectId) -> Facing {
    w.obj(u)
        .unwrap()
        .jump
        .as_ref()
        .unwrap()
        .heading
        .value(w.tick)
}

fn body(w: &World, u: ObjectId) -> Facing {
    w.obj(u).unwrap().body.value(w.tick)
}

/// The direction of the target from the vehicle (S-0204 claim 8), as the
/// firing check reads it.
fn to_target(w: &World, u: ObjectId, e: ObjectId) -> Facing {
    let from = w.reference_point(u);
    let to = glasnost_sim::combat::range::target_point(w, Target::Object(e));
    direction(
        i64::from(to.x - from.x),
        i64::from(to.y - from.y),
        w.rules().quirks.table_direction,
    )
}

fn last_fired(w: &World, u: ObjectId) -> i32 {
    w.obj(u).unwrap().last_fired
}

/// The vehicle's body turn rate ω (S-0206 claims 6 and 7).
fn omega(w: &World, u: ObjectId) -> i32 {
    let o = w.obj(u).unwrap();
    w.rules().unit(o.kind, o.type_index).unwrap().turn_rate
}

/// The ticks of its shots over `n` ticks.
fn shots(w: &mut World, u: ObjectId, n: i32) -> Vec<i32> {
    let mut v = Vec::new();
    let mut last = last_fired(w, u);
    for _ in 0..n {
        step(w);
        let now = last_fired(w, u);
        if now != last {
            v.push(now);
            last = now;
        }
    }
    v
}

// spec: S-0502 claim 10 (A vehicle with a target: a hovering jumpjet vehicle that keeps a movement commitment, with no turret and a weapon that is not omni-fire, keeps its body on its movement heading and does not fire while the direction of its target is more than the tolerance of S-0310 claim 2 from the heading; for the reply to not facing without a turret its locomotor is moving it whenever it is moving, so that reply gives it no body turn) and Edge cases (quirks.hovering_jumpjet_body_on_heading on)
#[test]
fn a_committed_hovering_vehicle_keeps_its_body_on_the_heading_and_holds_its_fire() {
    let mut w = game_with_hoverers(true);
    let (u, e) = hoverer(&mut w, "HVGUN", true);
    step(&mut w);
    let h = heading(&w, u);
    // spec: S-0310 claim 2 (τ = 2048 for a projectile that does not home)
    assert!(
        delta(h, to_target(&w, u, e)).abs() > 2048,
        "set-up: the target is off the heading"
    );
    for _ in 0..120 {
        step(&mut w);
        let o = w.obj(u).unwrap();
        assert_eq!(phase(&w, u), Phase::Hovering);
        assert_eq!(heading(&w, u), h, "the heading does not follow the target");
        assert_eq!(
            body(&w, u),
            h,
            "the body is on the heading at tick {}",
            w.tick
        );
        assert!(
            !o.body.turning(w.tick),
            "no body turn towards the target at tick {}",
            w.tick
        );
        assert!(
            !o.turret.turning(w.tick),
            "nor the turret that follows the body at tick {}",
            w.tick
        );
    }
    assert_eq!(
        last_fired(&w, u),
        -100,
        "no shot at a target outside the heading's tolerance"
    );
}

// spec: S-0502 Edge cases (quirks.hovering_jumpjet_body_on_heading off, fixed behaviour: in hovering the physics steps do not set a vehicle's body facing from the heading, and for the reply to not facing without a turret its locomotor does not count as moving it; with no destination its body turns towards its target at its body turn rate, S-0206 claim 7, and it fires once the target lies within the tolerance; its movement heading is unchanged)
#[test]
fn switch_off_a_hovering_vehicle_turns_its_body_and_fires_within_the_tolerance() {
    let mut w = game_with_hoverers(false);
    let (u, e) = hoverer(&mut w, "HVGUN", true);
    let h = heading(&w, u);
    let start = body(&w, u);
    let d = to_target(&w, u, e);
    assert!(
        delta(start, d).abs() > 2048,
        "set-up: the target is off the body"
    );
    step(&mut w);
    // spec: S-0206 claim 7 and F1 (the body turns at the type's turn rate: Θ = ⌊|Δ| / (256 ω)⌋)
    let o = w.obj(u).unwrap();
    assert_eq!(
        o.body.target, d,
        "the body's target facing is the direction of the target"
    );
    let theta = delta(start, d).abs() / (256 * omega(&w, u));
    assert!(theta > 1, "set-up: a turn of more than one tick");
    assert_eq!(o.body.duration, theta);
    let mut first = None;
    for _ in 0..(theta + 10) {
        let before = body(&w, u);
        let fired = last_fired(&w, u);
        step(&mut w);
        assert_eq!(phase(&w, u), Phase::Hovering);
        assert_eq!(heading(&w, u), h, "the movement heading is unchanged");
        if last_fired(&w, u) != fired {
            // spec: S-0310 claim 2 and S-0206 claim 10 (d) (the check of the shot's tick saw the body facing of that tick, within τ = 2048)
            assert!(delta(before, d).abs() <= 2048 || delta(body(&w, u), d).abs() <= 2048);
            first = Some(w.tick);
            break;
        }
    }
    assert!(
        first.is_some(),
        "it fires once the body has turned to the target"
    );
    assert!(delta(body(&w, u), d).abs() <= 2048);
}

// spec: S-0502 Edge cases (quirks.hovering_jumpjet_body_on_heading off: the first physics step outside hovering sets the body facing from the heading again, at once)
#[test]
fn switch_off_the_body_returns_to_the_heading_at_the_first_step_outside_hovering() {
    let mut w = game_with_hoverers(false);
    let (u, e) = hoverer(&mut w, "HVGUN", true);
    let d = to_target(&w, u, e);
    assert!(
        testkit_until_body(&mut w, u, d, 60),
        "the body turns to the target first"
    );
    // A move order far to the east: claim 13 row 1 turns the heading and the vehicle cruises.
    go(&mut w, u, at(12, 0));
    let mut out = false;
    for _ in 0..6 {
        let was = phase(&w, u);
        step(&mut w);
        if was != Phase::Hovering {
            // spec: S-0502 claim 10 (outside hovering each physics step sets the body facing to the movement heading)
            assert_eq!(
                body(&w, u),
                heading(&w, u),
                "the body is on the heading again"
            );
            out = true;
            break;
        }
    }
    assert!(out, "it left hovering");
}

fn testkit_until_body(w: &mut World, u: ObjectId, d: Facing, max: i32) -> bool {
    until(w, max, |w| {
        let o = w.obj(u).unwrap();
        o.body.value(w.tick) == d && !o.body.turning(w.tick)
    })
}

// spec: S-0502 claim 10 (Without a commitment: a hovering jumpjet vehicle without a movement commitment takes no physics step, claim 3; with no destination and a not facing answer the body turns towards the target at the vehicle's body turn rate, S-0206 claim 7, and it can fire once the target lies within the tolerance) and Edge cases (under either setting of quirks.hovering_jumpjet_body_on_heading)
#[test]
fn without_a_commitment_the_body_turns_to_the_target_under_either_setting() {
    for on in [true, false] {
        let mut w = game_with_hoverers(on);
        let (u, e) = hoverer(&mut w, "HVGUN", false);
        assert!(!jump::moving(&w, u), "set-up: no movement commitment");
        let h = heading(&w, u);
        let d = to_target(&w, u, e);
        assert!(
            delta(body(&w, u), d).abs() > 2048,
            "set-up: the target is off the body"
        );
        step(&mut w);
        assert_eq!(
            w.obj(u).unwrap().body.target,
            d,
            "switch {on}: the body turns to the target"
        );
        let fired = shots(&mut w, u, 40);
        assert!(
            !fired.is_empty(),
            "switch {on}: it fires once within the tolerance"
        );
        assert!(delta(body(&w, u), d).abs() <= 2048);
        assert_eq!(
            heading(&w, u),
            h,
            "switch {on}: no physics step turns the heading"
        );
        assert_eq!(phase(&w, u), Phase::Hovering);
    }
}

// spec: S-0502 claim 10 (With a turret: the turret aims by S-0206 claim 9 as on any vehicle, and the body stays on the heading) and Edge cases (quirks.hovering_jumpjet_body_on_heading on)
#[test]
fn a_turreted_hovering_vehicle_aims_its_turret_with_the_body_on_the_heading() {
    let mut w = game_with_hoverers(true);
    let (u, e) = hoverer(&mut w, "HVTUR", true);
    step(&mut w);
    let h = heading(&w, u);
    let d = to_target(&w, u, e);
    assert!(
        delta(h, d).abs() > 2048,
        "set-up: the target is off the heading"
    );
    let mut fired = false;
    for _ in 0..60 {
        let before = last_fired(&w, u);
        step(&mut w);
        assert_eq!(
            body(&w, u),
            h,
            "the body stays on the heading at tick {}",
            w.tick
        );
        fired |= last_fired(&w, u) != before;
    }
    assert_eq!(
        w.obj(u).unwrap().turret.target,
        d,
        "the turret aims at the target"
    );
    assert!(fired, "it fires once its turret points at the target");
}

// spec: S-0502 claim 10 (Omni-fire weapon: it fires whatever the direction of its target, S-0310 claim 2, with its body on its movement heading) and Edge cases (on; off: the physics steps in hovering leave the body alone and the check never answers not facing, so it keeps the body facing it had when it entered hovering)
#[test]
fn an_omni_fire_hovering_vehicle_fires_with_its_body_where_the_setting_puts_it() {
    for on in [true, false] {
        let mut w = game_with_hoverers(on);
        let (u, e) = hoverer(&mut w, "HVOMNI", true);
        // The body away from both the heading and the target.
        let h = heading(&w, u);
        let away = h.wrapping_add(8192);
        w.obj_mut(u).unwrap().body = Turning::at(away);
        assert!(delta(away, to_target(&w, u, e)).abs() > 2048);
        let mut ticks = Vec::new();
        for _ in 0..100 {
            let before = last_fired(&w, u);
            step(&mut w);
            let want = if on { h } else { away };
            assert_eq!(body(&w, u), want, "switch {on}: body at tick {}", w.tick);
            assert!(!w.obj(u).unwrap().body.turning(w.tick));
            if last_fired(&w, u) != before {
                ticks.push(last_fired(&w, u));
            }
        }
        assert!(
            ticks.len() >= 3,
            "switch {on}: it fires at its rearm rate: {ticks:?}"
        );
        assert_eq!(heading(&w, u), h);
    }
}

// spec: S-0502 claim 10 (Exception: a hovering infantry jumpjet with the jumpjet-turn flag and a target snaps its heading to the target; The jumpjet-turn flag: a vehicle has none) and Edge cases (the switch concerns jumpjet vehicles only)
#[test]
fn the_switch_leaves_a_jumpjet_turn_infantry_unit_alone() {
    let run = |on: bool| {
        let mut w = game_with_hoverers(on);
        let e = put(&mut w, "CART", Kind::Vehicle, 1, 0, 4);
        let u = put(&mut w, "JJTURN", Kind::Infantry, 0, 0, 0);
        jump::start_in_flight(&mut w, u, Phase::Hovering, 300);
        commit_here(&mut w, u);
        let o = w.obj_mut(u).unwrap();
        o.dest = None;
        o.target = Some(Target::Object(e));
        let mut trace = Vec::new();
        for _ in 0..20 {
            step(&mut w);
            trace.push((heading(&w, u), body(&w, u), w.obj(u).unwrap().pos));
        }
        assert_eq!(
            trace.last().unwrap().0,
            to_target(&w, u, e),
            "the heading snaps to the target"
        );
        trace
    };
    assert_eq!(run(true), run(false));
}
