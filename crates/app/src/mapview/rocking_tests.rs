//! Tests of the view's body rocking (S-0113) on synthetic data: every type
//! name and number is made up.

use glasnost_num::{F32, F64, Rounding};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, Locomotor, WarheadType};
use glasnost_sim::combat::area::{self, Detonation};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::rules::Kind as SimKind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, World};

use super::build::Attitude;
use super::live::attitude_with;
use super::rocking::{
    Push, Rocking, SLOWED_PITCH_LIMIT, USUAL_LIMIT, area_strength, axis_step, crash_rates, drawn,
    push_rates,
};
use super::sinking_tests::images;

const SIZE: i32 = 24;
const RZ: Rounding = Rounding::TowardZero;

fn single(v: f32) -> F32 {
    F32(v.to_bits())
}

fn real(v: F32) -> f64 {
    f64::from(f32::from_bits(v.0))
}

fn ruleset(weight: Option<&str>) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.warheads.insert(
        "ROCK".into(),
        WarheadType {
            cell_spread: Some(Decimal("2".into())),
            verses: Some(vec![Decimal("0".into()); 11]),
            rocker: Some(true),
            ..Default::default()
        },
    );
    let cart = r.sim.vehicles.get_mut("CART").unwrap();
    cart.locomotor = Some(Locomotor::Drive);
    cart.weight = weight.map(|w| Decimal(w.into()));
    r
}

/// A world of `r` with one vehicle facing map north (facing 0) at the centre.
fn world_of(r: &Ruleset) -> (World, ObjectId) {
    let mut w = testkit::game_with(r, SIZE, 3);
    let cart = testkit::spawn(
        &mut w,
        SimKind::Vehicle,
        "CART",
        0,
        testkit::mid(SIZE, 0, 0),
        0,
    );
    (w, cart)
}

fn world(weight: Option<&str>) -> (World, ObjectId) {
    world_of(&ruleset(weight))
}

/// Sets off a rocking detonation of `damage` at `point` with no source
/// object, then runs one tick for the view.
fn blast(w: &mut World, rocking: &mut Rocking, point: Pos, damage: i32) {
    let warhead = w
        .rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == "ROCK")
        .map(|i| i as u32);
    w.effects.clear();
    area::detonate(
        w,
        Detonation {
            point,
            damage,
            warhead,
            source: None,
            house: None,
            resources: false,
        },
    );
    tick(w, rocking);
}

/// One game tick for the view, as `Live::take_effects` runs it: the
/// bodies step in the new tick, then the tick's reports set the rates.
fn tick(w: &mut World, rocking: &mut Rocking) {
    let effects = std::mem::take(&mut w.effects);
    w.tick += 1;
    rocking.advance(w);
    rocking.take(w, &effects);
}

fn offset(w: &World, id: ObjectId, dx: i32, dy: i32) -> Pos {
    let p = w.obj(id).unwrap().pos;
    Pos::new(p.x + dx, p.y + dy, p.z)
}

fn overload(w: &mut World, id: ObjectId, level: i32, draw: i32) {
    w.effects.push(Effect::OverloadRock {
        controller: id,
        level,
        draw,
    });
}

// spec: S-0113 Constants (usual limit π/4, crush-slowdown pitch limit π/10, single-precision constants)
#[test]
fn the_limits_are_single_constants() {
    assert_eq!(USUAL_LIMIT.0, std::f32::consts::FRAC_PI_4.to_bits());
    assert_eq!(
        SLOWED_PITCH_LIMIT.0,
        ((std::f64::consts::PI / 10.0) as f32).to_bits()
    );
}

// spec: S-0113 State and ordering (the simulated table of a push from level: peak, step of the peak, step at which the angle is level again), table R, claims 7 and 8
#[test]
fn a_push_from_level_follows_the_table() {
    // (roll, starting rate, peak, tolerance of the peak's digits, step of
    // the peak, level at step); the peak of the first row is π/4, held.
    let rows: [(bool, f32, f64, f64, i32, i32); 10] = [
        (true, 0.05, std::f64::consts::FRAC_PI_4, 0.0, 23, 41),
        (true, -0.05, -0.65, 0.005, 25, 42),
        (true, 0.03, 0.354, 0.0005, 19, 32),
        (true, -0.03, -0.24, 0.005, 15, 26),
        (true, 0.015, 0.125, 0.0005, 11, 19),
        (true, -0.015, -0.064, 0.0005, 8, 14),
        (false, 0.02, 0.095, 0.0005, 6, 13),
        (false, -0.05, -0.275, 0.0005, 10, 21),
        (false, 0.025, 0.13, 0.005, 7, 15),
        (false, -0.025, -0.075, 0.0005, 5, 11),
    ];
    for (roll, rate, peak, tolerance, peak_step, level_step) in rows {
        let (mut a, mut r) = (F32::ZERO, single(rate));
        let (mut best, mut best_step) = (0.0f64, 0);
        let mut level = None;
        for step in 1..=60 {
            (a, r) = axis_step(a, r, USUAL_LIMIT, roll, true, RZ);
            if step == 1 {
                assert_eq!(a, single(rate), "the first angle is the pushed rate");
            }
            if real(a).abs() > best.abs() {
                (best, best_step) = (real(a), step);
            }
            if a == F32::ZERO && r == F32::ZERO {
                level = Some(step);
                break;
            }
        }
        let name = format!("{} {rate}", if roll { "roll" } else { "pitch" });
        if tolerance == 0.0 {
            assert_eq!(best, real(USUAL_LIMIT), "{name}: held at the limit");
        } else {
            assert!((best - peak).abs() <= tolerance, "{name}: peak {best}");
        }
        assert_eq!(best_step, peak_step, "{name}: step of the peak");
        assert_eq!(level, Some(level_step), "{name}: level again");
    }
}

// spec: S-0113 claim 6 (a body held at its limit stays there for that one step only: the pull acts in the same step) and claim 7
#[test]
fn a_body_is_held_at_its_limit_for_one_step() {
    let (mut a, mut r) = (F32::ZERO, single(0.05));
    let mut held = Vec::new();
    for step in 1..=30 {
        (a, r) = axis_step(a, r, USUAL_LIMIT, true, true, RZ);
        if a == USUAL_LIMIT {
            held.push(step);
            assert!(real(r) < 0.0, "pulled back in the same step");
        }
    }
    assert_eq!(held, [23]);
    // The crush-slowdown pitch limit holds the same way.
    let (a, r) = axis_step(
        single(0.3),
        single(0.05),
        SLOWED_PITCH_LIMIT,
        false,
        true,
        RZ,
    );
    assert_eq!(a, SLOWED_PITCH_LIMIT);
    assert!(real(r) < 0.0);
    let (a, _) = axis_step(
        single(-0.3),
        single(-0.05),
        SLOWED_PITCH_LIMIT,
        false,
        true,
        RZ,
    );
    assert_eq!(a.0, SLOWED_PITCH_LIMIT.0 ^ 0x8000_0000);
}

// spec: S-0113 claim 15 and Edge cases (quirks.rocking_limit_escape: on, an angle at its limit is not held at the next step; off, held whatever the angle before the step); claim 9 (past the edge the pull is away from level)
#[test]
fn an_angle_at_its_limit_escapes_only_with_the_switch_on() {
    let push = single(0.05);
    let (on, r_on) = axis_step(USUAL_LIMIT, push, USUAL_LIMIT, true, true, RZ);
    assert!(real(on) > real(USUAL_LIMIT), "escaped");
    assert!(real(r_on) > 0.0);
    let (off, r_off) = axis_step(USUAL_LIMIT, push, USUAL_LIMIT, true, false, RZ);
    assert_eq!(off, USUAL_LIMIT);
    assert!(real(r_off) < 0.0, "held and pulled back");
    // Beyond its limit an angle is not held again until it comes back.
    let (a, _) = axis_step(single(1.0), single(0.01), USUAL_LIMIT, true, false, RZ);
    assert_eq!(a, USUAL_LIMIT, "off: held from beyond the limit");
    let (a, _) = axis_step(single(1.0), single(0.01), USUAL_LIMIT, true, true, RZ);
    assert!(real(a) > 1.0, "on: not held");
    // Past the edge (π/2) the pull is away from level.
    let (_, r) = axis_step(single(1.6), single(0.01), USUAL_LIMIT, true, true, RZ);
    assert!(real(r) > 0.01);
    let (_, r) = axis_step(single(-1.6), single(-0.01), USUAL_LIMIT, false, true, RZ);
    assert!(real(r) < -0.01);
}

/// Pulses an overload on the cart every tick for `ticks` ticks and gives
/// the largest roll seen and whether the body was dropped.
fn rolled_by_pulses(escape: bool, ticks: i32) -> (f64, bool) {
    let mut r = ruleset(None);
    r.globals.quirks.rocking_limit_escape = escape;
    let (mut w, cart) = world_of(&r);
    let mut rocking = Rocking::default();
    let mut most = 0.0f64;
    for _ in 0..ticks {
        overload(&mut w, cart, 2, 80);
        tick(&mut w, &mut rocking);
        // After a drop the tick's pulse starts a new body from level.
        let roll = rocking.body(cart).map_or(0.0, |b| real(b.roll));
        if most > std::f64::consts::FRAC_PI_2 && roll == 0.0 {
            return (most, true);
        }
        most = most.max(roll);
    }
    (most, false)
}

// spec: S-0113 claim 15 and Edge cases (quirks.rocking_limit_escape on: repeated pushes carry the roll past its limit and over; off: held at ±L); claim 9 (beyond ±π the body is destroyed; our view drops it, a seam)
#[test]
fn repeated_pushes_roll_a_body_over_only_with_the_escape() {
    let (most, dropped) = rolled_by_pulses(true, 200);
    assert!(most > std::f64::consts::FRAC_PI_2, "past the edge: {most}");
    assert!(dropped, "rolled over");
    let (most, dropped) = rolled_by_pulses(false, 200);
    assert_eq!(most, real(USUAL_LIMIT), "held at π/4");
    assert!(!dropped);
}

fn push(
    body: (i64, i64, i64),
    from: (i64, i64, i64),
    facing: u16,
    weight: F64,
) -> Option<(F32, F32)> {
    push_rates(
        &Push {
            body,
            from,
            strength: single(1.0),
            half: false,
            weight,
            facing,
        },
        true,
        true,
        RZ,
    )
}

fn one() -> F64 {
    F64::from_i64(1, RZ)
}

// spec: S-0113 claim 1 and F1 (the end or side of the body nearer the push point rises, at every facing: positive pitch lowers the bow, positive roll raises the left side); Terms
#[test]
fn a_push_lifts_the_nearer_end_or_side_at_every_facing() {
    let b = (6000, 6000, 0);
    // Facing 0 heads map north (−y), 16384 east; each eighth turn
    // clockwise.
    let headings: [(u16, (i64, i64)); 4] = [
        (0, (0, -200)),
        (8192, (141, -141)),
        (16384, (200, 0)),
        (40960, (-141, 141)),
    ];
    for (facing, (hx, hy)) in headings {
        // The right is a quarter turn clockwise of the heading.
        let (rx, ry) = (-hy, hx);
        let cases = [
            ("ahead", (hx, hy), -1, 0),
            ("behind", (-hx, -hy), 1, 0),
            ("on its right", (rx, ry), 0, -1),
            ("on its left", (-rx, -ry), 0, 1),
        ];
        for (name, (dx, dy), pitch, roll) in cases {
            let (p, r) = push(b, (b.0 + dx, b.1 + dy, 0), facing, one()).unwrap();
            let sign = |v: F32, other: F32| {
                let (v, o) = (real(v), real(other));
                if v.abs() <= o.abs() / 10.0 {
                    0
                } else {
                    v.signum() as i32
                }
            };
            assert_eq!(
                (sign(p, r), sign(r, p)),
                (pitch, roll),
                "facing {facing}, a push {name}: {p:?} {r:?}"
            );
        }
    }
    // k = (0.04 − 200 × 2.5 × 10⁻⁵) × 1 = 0.035 from behind at facing 0.
    let (p, _) = push(b, (b.0, b.1 + 200, 0), 0, one()).unwrap();
    assert!((real(p) - 0.035).abs() < 1e-5, "{p:?}");
}

// spec: S-0113 claims 1 and 11 (h = ½ for an area push and a drone delivery: the pitch rate halved, the roll rate not)
#[test]
fn the_pitch_factor_halves_only_the_pitch() {
    let b = (6000, 6000, 0);
    let rates = |half: bool, q: (i64, i64, i64)| {
        push_rates(
            &Push {
                body: b,
                from: q,
                strength: single(1.0),
                half,
                weight: one(),
                facing: 0,
            },
            true,
            true,
            RZ,
        )
        .unwrap()
    };
    let behind = (b.0, b.1 + 200, 0);
    let side = (b.0 + 200, b.1, 0);
    assert_eq!(
        real(rates(true, behind).0),
        real(rates(false, behind).0) / 2.0
    );
    assert_eq!(rates(true, side).1, rates(false, side).1);
}

// spec: S-0113 claim 3 and F1 (k = (0.04 − 2.5 × 10⁻⁵ d) × s / W, capped at 0.05; nothing below 0.01 or at the body's own position); Edge cases (weight of 0 gives the cap, a negative weight nothing within the reach)
#[test]
fn strength_distance_and_weight_set_the_magnitude() {
    let b = (6000, 6000, 0);
    let behind = (b.0, b.1 + 200, 0);
    let w = |v: i64| F64::from_i64(v, RZ);
    let (light, _) = push(b, behind, 0, one()).unwrap();
    let (heavy, _) = push(b, behind, 0, w(2)).unwrap();
    assert!(
        (real(heavy) - real(light) / 2.0).abs() < 1e-6,
        "the weight divides"
    );
    let (zero, _) = push(b, behind, 0, F64::ZERO).unwrap();
    assert!((real(zero) - 0.05).abs() < 1e-6, "weight 0: the cap");
    assert_eq!(push(b, behind, 0, w(-1)), None, "negative weight: nothing");
    assert_eq!(push(b, b, 0, one()), None, "the body's own position");
    // 0.04 − 2.5 × 10⁻⁵ d falls below 0.01 a little beyond d = 1200.
    assert!(push(b, (b.0, b.1 + 1100, 0), 0, one()).is_some());
    assert_eq!(push(b, (b.0, b.1 + 1300, 0), 0, one()), None, "k < 0.01");
    // A strength of 4 at 200 leptons is capped at 0.05.
    let capped = push_rates(
        &Push {
            body: b,
            from: behind,
            strength: single(4.0),
            half: false,
            weight: one(),
            facing: 0,
        },
        true,
        true,
        RZ,
    )
    .unwrap();
    assert!((real(capped.0) - 0.05).abs() < 1e-6);
}

// spec: S-0113 claim 4 (a push from straight above or below sets both rates to 0) and Edge cases (a rate of exactly 0 snaps the angle level: one step levels a rocking body)
#[test]
fn a_push_from_straight_above_levels_the_body() {
    let b = (6000, 6000, 0);
    assert_eq!(
        push(b, (b.0, b.1, 300), 0, one()),
        Some((F32::ZERO, F32::ZERO))
    );
    assert_eq!(
        push(b, (b.0, b.1, -300), 0, one()),
        Some((F32::ZERO, F32::ZERO))
    );
    assert_eq!(
        axis_step(single(0.4), F32::ZERO, USUAL_LIMIT, true, true, RZ),
        (F32::ZERO, F32::ZERO)
    );
}

// spec: S-0113 F2 (s = RZ₂₄(min(RZ₅₃(D × 0.01), 4.0)))
#[test]
fn the_area_strength_of_a_damage_amount() {
    assert_eq!(area_strength(50, RZ), single(0.5));
    assert_eq!(area_strength(100, RZ), single(1.0));
    assert_eq!(area_strength(400, RZ), single(4.0));
    assert_eq!(area_strength(5000, RZ), single(4.0));
    assert!((real(area_strength(73, RZ)) - 0.73).abs() < 1e-6);
}

// spec: S-0113 claim 2 (a push replaces both rates and the angles keep their values; of several before one turn only the last counts) and claim 17 (an overload replaces the roll rate only); State and ordering (a push shows from the next step: the first angle is the pushed rate)
#[test]
fn the_last_push_before_a_turn_counts() {
    let (mut w, cart) = world(None);
    let mut rocking = Rocking::default();
    overload(&mut w, cart, 1, 10);
    overload(&mut w, cart, 2, 90);
    tick(&mut w, &mut rocking);
    let b = rocking.body(cart).unwrap();
    assert_eq!(b.roll_rate, single(0.03), "the last pulse's rate");
    assert_eq!(b.roll, F32::ZERO, "it shows from the next step");
    tick(&mut w, &mut rocking);
    let b = rocking.body(cart).unwrap();
    assert_eq!(b.roll, single(0.03), "the first angle is the pushed rate");
    // A push from behind replaces both rates and leaves the roll angle.
    let point = offset(&w, cart, 0, 300);
    let before = rocking.body(cart).unwrap();
    blast(&mut w, &mut rocking, point, 400);
    let after = rocking.body(cart).unwrap();
    assert!(real(after.pitch_rate) > 0.0);
    assert!(
        real(after.roll_rate).abs() < 1e-9,
        "a push from straight behind has no roll: {after:?}"
    );
    let (stepped, _) = axis_step(before.roll, before.roll_rate, USUAL_LIMIT, true, true, RZ);
    assert_eq!(after.roll, stepped, "the angle keeps its value");
}

// spec: S-0601 F4 and S-0113 claim 17 (an overload pulse sets the roll rate: 0.015 at level 1, 0.03 from level 2, negative for a draw below 50; unscaled by the weight; the pitch left alone)
#[test]
fn an_overload_pulse_sets_the_roll_rate() {
    for (level, draw, rate) in [
        (1, 49, -0.015f32),
        (1, 50, 0.015),
        (2, 0, -0.03),
        (5, 100, 0.03),
    ] {
        let (mut w, cart) = world(Some("4"));
        let mut rocking = Rocking::default();
        overload(&mut w, cart, level, draw);
        tick(&mut w, &mut rocking);
        let b = rocking.body(cart).unwrap();
        assert_eq!(b.roll_rate, single(rate), "level {level}, draw {draw}");
        assert_eq!((b.pitch, b.pitch_rate), (F32::ZERO, F32::ZERO));
    }
}

// spec: S-0601 F4 (revision 20: the rate replaces the controller's roll rate, not added to a nonzero one, and is not scaled by the type's weight) and S-0113 claim 17
#[test]
fn an_overload_pulse_replaces_a_nonzero_roll_rate() {
    let (mut w, cart) = world(Some("4"));
    let mut rocking = Rocking::default();
    w.effects.push(Effect::DroneRock {
        host: cart,
        side: 0,
    });
    tick(&mut w, &mut rocking);
    tick(&mut w, &mut rocking);
    let before = rocking.body(cart).unwrap();
    assert!(real(before.roll_rate) > 0.0, "rolling: {before:?}");
    overload(&mut w, cart, 1, 0);
    tick(&mut w, &mut rocking);
    let b = rocking.body(cart).unwrap();
    assert_eq!(b.roll_rate, single(-0.015), "replaced, unscaled: {b:?}");
}

// spec: S-0113 claim 6 (the pitch limit π/10 in S-0210's crush-slowdown state) and Edge cases (quirks.rocking_limit_escape off: held at the limit under repeated pushes)
#[test]
fn a_crush_slowed_body_pitches_at_most_a_tenth_of_pi() {
    let most = |slowed: bool| {
        let mut r = ruleset(Some("0.2"));
        r.globals.quirks.rocking_limit_escape = false;
        let (mut w, cart) = world_of(&r);
        w.obj_mut(cart).unwrap().crush_slowdown = slowed;
        let mut rocking = Rocking::default();
        let mut most = 0.0f64;
        for _ in 0..60 {
            let point = offset(&w, cart, 0, 200);
            blast(&mut w, &mut rocking, point, 400);
            most = most.max(rocking.body(cart).map_or(0.0, |b| real(b.pitch)));
        }
        most
    };
    assert_eq!(most(true), real(SLOWED_PITCH_LIMIT));
    assert!(most(false) > real(SLOWED_PITCH_LIMIT));
}

// spec: S-0113 claim 13 and F3 (a drone delivery: a draw of 0 puts the point to the host's left, so its left side rises, a positive roll; a draw of 1 the other way; h = ½, S-0614 F2's strength)
#[test]
fn a_drone_rocks_its_host_to_the_drawn_side() {
    for (side, positive) in [(0, true), (1, false)] {
        let (mut w, cart) = world(None);
        let mut rocking = Rocking::default();
        w.effects.push(Effect::DroneRock { host: cart, side });
        tick(&mut w, &mut rocking);
        let b = rocking.body(cart).unwrap();
        assert_eq!(real(b.roll_rate) > 0.0, positive, "side {side}: {b:?}");
        assert!(real(b.roll_rate).abs() >= 0.01);
        assert!(real(b.pitch_rate).abs() < real(b.roll_rate).abs());
    }
}

fn squid_world() -> (World, ObjectId, ObjectId) {
    let mut r = ruleset(None);
    let mut squid = r.sim.vehicles.get("CART").unwrap().clone();
    squid.combat.naval = Some(true);
    squid.combat.organic = Some(true);
    r.sim.vehicles.insert("SQUID".into(), squid);
    r.registries
        .get_mut("vehicles")
        .expect("registry")
        .push("SQUID".into());
    let (mut w, cart) = world_of(&r);
    let squid = testkit::spawn(
        &mut w,
        SimKind::Vehicle,
        "SQUID",
        1,
        testkit::mid(SIZE, 3, 3),
        0,
    );
    w.obj_mut(cart).unwrap().parasite = Some(glasnost_sim::parasite::ParasiteState {
        attached: Some(squid),
        ..Default::default()
    });
    (w, cart, squid)
}

// spec: S-0113 claim 14 and F4 (a push phase writes no roll in turn 1, k = 0 in turns 2 and 3, each k from 1 to 9 in 4 turns, 0 in turn 40; σ = +1 for a way draw of 0; roll = RZ₂₄(sinT(k σ π/10) × π/4); the roll set to 0 at each strike and when the parasite leaves; the pitch unaffected)
#[test]
fn a_grappling_parasite_writes_its_hosts_roll() {
    for (way, sign) in [(0, 1.0f64), (1, -1.0)] {
        let (mut w, cart, _) = squid_world();
        let mut rocking = Rocking::default();
        tick(&mut w, &mut rocking);
        w.effects.push(Effect::GrapplePush { host: cart, way });
        tick(&mut w, &mut rocking);
        assert_eq!(rocking.body(cart), None, "turn 1: no roll written");
        for turn in 2..=45 {
            tick(&mut w, &mut rocking);
            let b = rocking.body(cart);
            let k = turn / 4;
            if turn >= 40 {
                assert!(b.is_none_or(|b| b.roll == F32::ZERO), "turn {turn}: ended");
                continue;
            }
            let b = b.unwrap_or_else(|| panic!("turn {turn}: written"));
            let want = (f64::from(k) * sign * std::f64::consts::PI / 10.0).sin()
                * std::f64::consts::FRAC_PI_4;
            assert!(
                (real(b.roll) - want).abs() < 1e-3,
                "way {way}, turn {turn}: {b:?}"
            );
            assert_eq!(b.pitch, F32::ZERO);
            assert_eq!(b.roll_rate, F32::ZERO, "the rate is never changed");
        }
    }
    // A strike sets the roll to 0; the parasite's leaving too.
    let (mut w, cart, _) = squid_world();
    let mut rocking = Rocking::default();
    tick(&mut w, &mut rocking);
    w.effects.push(Effect::GrapplePush { host: cart, way: 0 });
    for _ in 0..10 {
        tick(&mut w, &mut rocking);
    }
    assert!(real(rocking.body(cart).unwrap().roll) > 0.1);
    w.effects.push(Effect::GrapplePush { host: cart, way: 1 });
    tick(&mut w, &mut rocking);
    assert_eq!(rocking.body(cart).unwrap().roll, F32::ZERO, "struck");
    for _ in 0..10 {
        tick(&mut w, &mut rocking);
    }
    assert!(
        real(rocking.body(cart).unwrap().roll) < -0.1,
        "the next push's own draw"
    );
    w.obj_mut(cart).unwrap().parasite = None;
    tick(&mut w, &mut rocking);
    assert!(
        rocking.body(cart).is_none_or(|b| b.roll == F32::ZERO),
        "left"
    );
}

// spec: S-0113 claim 14 (a roll rate from a push keeps stepping underneath the parasite's writes, from its last value, and shows once it stops writing)
#[test]
fn a_rate_steps_under_the_parasites_roll() {
    let (mut w, cart, _) = squid_world();
    let mut rocking = Rocking::default();
    tick(&mut w, &mut rocking);
    w.effects.push(Effect::GrapplePush { host: cart, way: 0 });
    for _ in 0..8 {
        tick(&mut w, &mut rocking);
    }
    let written = rocking.body(cart).unwrap().roll;
    overload(&mut w, cart, 1, 80);
    tick(&mut w, &mut rocking);
    tick(&mut w, &mut rocking);
    let b = rocking.body(cart).unwrap();
    assert_eq!(
        b.roll, written,
        "the drawn roll is the parasite's (same frame)"
    );
    assert!(b.roll_rate != F32::ZERO, "the rate steps underneath");
}

// spec: S-0113 claim 5 and table S (no rocking step during a superweapon warp from the first pending update through the arrival effects; again in arrival hold)
#[test]
fn a_warped_body_holds_its_angles() {
    use glasnost_sim::superweapons::warp::{Member, Stage};
    for (stage, moves) in [
        (Stage::Pending, false),
        (Stage::Away, false),
        (Stage::Placed, false),
        (Stage::Hold(0), true),
    ] {
        let (mut w, cart) = world(None);
        let mut rocking = Rocking::default();
        overload(&mut w, cart, 2, 80);
        tick(&mut w, &mut rocking);
        tick(&mut w, &mut rocking);
        let before = rocking.body(cart).unwrap();
        let landing = w.obj(cart).unwrap().pos;
        w.obj_mut(cart).unwrap().sw.warp = Some(Member {
            stage,
            house: HouseId(0),
            landing,
            redirected: false,
            fell_back: false,
            refused: false,
            entry_ok: true,
        });
        tick(&mut w, &mut rocking);
        let after = rocking.body(cart).unwrap();
        assert_eq!(after != before, moves, "{stage:?}");
    }
}

// spec: S-0113 claim 5 and table S (sinking: the pitch rate is left as it is, the roll takes table R's roll step, so a push during the descent rolls the hull); S-0504 claim 15 (the lean starts from the pitch left over from rocking) and F4
#[test]
fn a_sinking_body_leans_from_its_rocked_pitch_and_rolls() {
    let (mut w, cart) = world(None);
    let mut rocking = Rocking::default();
    let point = offset(&w, cart, 0, 200);
    blast(&mut w, &mut rocking, point, 400);
    for _ in 0..3 {
        tick(&mut w, &mut rocking);
    }
    let rocked = rocking.body(cart).unwrap();
    assert!(real(rocked.pitch) > 0.04, "rocked bow down: {rocked:?}");
    glasnost_sim::naval::start_sinking(&mut w, cart);
    tick(&mut w, &mut rocking);
    assert_eq!(
        rocking.lean(cart),
        Some(rocked.pitch),
        "the lean starts from the rocked pitch"
    );
    assert_eq!(
        rocking.body(cart).unwrap().pitch_rate,
        rocked.pitch_rate,
        "the pitch rate is left as it is"
    );
    for _ in 0..5 {
        let before = rocking.lean(cart).unwrap();
        assert!(!glasnost_sim::naval::sinking_turn(&mut w, cart));
        tick(&mut w, &mut rocking);
        let lean = w.obj(cart).unwrap().naval.as_ref().unwrap().lean.unwrap();
        let expect = glasnost_sim::naval::pitch_step(before, lean, w.rules().mode);
        assert_eq!(rocking.lean(cart), Some(expect), "one F4 step per descent");
        let Attitude::Sinking { pitch, .. } = attitude_with(&w, w.obj(cart).unwrap(), &rocking)
        else {
            panic!("sinking")
        };
        assert_eq!(
            Some(F32(pitch)),
            rocking.lean(cart),
            "drawn from the view's lean"
        );
    }
    // A push from the left during descent rolls the hull.
    let point = offset(&w, cart, -200, 0);
    blast(&mut w, &mut rocking, point, 400);
    tick(&mut w, &mut rocking);
    let b = rocking.body(cart).unwrap();
    assert!(real(b.roll) > 0.0, "the nearer side rises: {b:?}");
    let Attitude::Sinking { roll, .. } = attitude_with(&w, w.obj(cart).unwrap(), &rocking) else {
        panic!("sinking")
    };
    assert!(f32::from_bits(roll) > 0.0, "the roll is drawn");
}

// spec: S-0113 claim 19 and State and ordering (one rocking step per game tick, not per drawn frame)
#[test]
fn the_angles_do_not_depend_on_the_frame_rate() {
    let run = |frames_per_tick: usize| {
        let (mut w, cart) = world(None);
        let mut rocking = Rocking::default();
        let point = offset(&w, cart, 150, 200);
        blast(&mut w, &mut rocking, point, 300);
        let mut seen = Vec::new();
        for _ in 0..30 {
            tick(&mut w, &mut rocking);
            for _ in 0..frames_per_tick {
                // A redraw without a tick moves nothing.
                rocking.advance(&w);
                seen.push(rocking.body(cart));
            }
            seen.dedup();
        }
        seen
    };
    assert_eq!(run(1), run(4));
}

// spec: S-0113 claim 5 (an angle that comes back into the level band stops there: a body rises, turns and settles, never swinging through level)
#[test]
fn a_rocked_body_settles_without_swinging_through_level() {
    let (mut w, cart) = world(None);
    let mut rocking = Rocking::default();
    let point = offset(&w, cart, 150, 150);
    blast(&mut w, &mut rocking, point, 400);
    let first = {
        tick(&mut w, &mut rocking);
        rocking.body(cart).unwrap()
    };
    let mut ticks = 0;
    while let Some(b) = rocking.body(cart) {
        for (a, a0) in [(b.pitch, first.pitch), (b.roll, first.roll)] {
            assert!(real(a) * real(a0) >= 0.0, "never through level: {b:?}");
        }
        tick(&mut w, &mut rocking);
        ticks += 1;
        assert!(ticks < 60, "settles");
    }
    assert!(rocking.is_empty());
    assert_eq!(rocking.angles(cart), None, "drawn level");
}

// spec: S-0106 Terms (a pitch or roll of 0.005 radians or more rocks); S-0105 claim 26 (each drawn pair is an image of its own); design: the drawn angles are held to that step
#[test]
fn drawn_angles_are_held_to_the_rocking_step() {
    assert_eq!(drawn(single(0.0049)).to_bits(), 0);
    assert_eq!(drawn(single(-0.0049)).to_bits(), 0, "never a negative zero");
    assert_eq!(
        drawn(single(0.005)),
        glasnost_render::voxel::ROCKING_THRESHOLD
    );
    assert_eq!(drawn(single(0.0123)), 0.01);
    assert!(drawn(USUAL_LIMIT) <= std::f32::consts::FRAC_PI_4);
}

// spec: S-0113 claim 18 and S-0106 claim 9 (the driving locomotor draws a body with its pitch and roll; the walking locomotor draws it without them, although its angles still step)
#[test]
fn a_rocking_vehicle_is_drawn_rocking() {
    let (mut w, cart) = world(None);
    let mut rocking = Rocking::default();
    assert_eq!(
        attitude_with(&w, w.obj(cart).unwrap(), &rocking),
        Attitude::None
    );
    let point = offset(&w, cart, 0, 200);
    blast(&mut w, &mut rocking, point, 400);
    tick(&mut w, &mut rocking);
    let Attitude::Rocking { pitch, roll, drawn } =
        attitude_with(&w, w.obj(cart).unwrap(), &rocking)
    else {
        panic!("rocking")
    };
    assert!(drawn);
    assert!(
        f32::from_bits(pitch) > 0.0,
        "a push from behind lowers the bow"
    );
    assert_eq!(roll, 0);
    let mut r = ruleset(None);
    r.sim.vehicles.get_mut("CART").unwrap().locomotor = Some(Locomotor::Walk);
    let (mut w2, walker) = world_of(&r);
    let mut rocking2 = Rocking::default();
    let point = offset(&w2, walker, 0, 200);
    blast(&mut w2, &mut rocking2, point, 400);
    tick(&mut w2, &mut rocking2);
    assert!(matches!(
        attitude_with(&w2, w2.obj(walker).unwrap(), &rocking2),
        Attitude::Rocking { drawn: false, .. }
    ));
}

// spec: S-0105 claim 26 and S-0106 claim 13 (a rocking body is drawn fresh: images of its own; below 0.005 radians the settled ones); S-0504 claim 16 and F5 (D for any rocking); design: a body that rocks and does not sink keeps its shadow (S-0105 claim 31's key has no pitch or roll)
#[test]
fn a_rocking_vehicle_has_images_of_its_own_and_keeps_its_shadow() {
    let rock = |p: f32, r: f32, drawn: bool| Attitude::Rocking {
        pitch: p.to_bits(),
        roll: r.to_bits(),
        drawn,
    };
    let a = images(&[
        Attitude::None,
        rock(0.2, 0.0, true),
        rock(0.0, -0.15, true),
        rock(0.2, 0.0, false),
        Attitude::None,
    ]);
    let (level, shadows) = &a[0];
    assert_eq!(*shadows, 1);
    for k in [1, 2] {
        assert_eq!(a[k].1, 1, "drawing {k}: a shadow");
        assert!(
            a[k].0.iter().filter(|i| !level.contains(i)).count() >= 2,
            "drawing {k}: body and turret rocked"
        );
    }
    assert_ne!(a[1].0, a[2].0);
    assert_eq!(a[3], a[0], "not drawn by its locomotor: settled");
    assert_eq!(a[4], a[0], "level again");
}

// spec: S-0113 F5 (roll rate = RZ₂₄(RZ₅₃(u₁ × 0.15) + 0.1), negated when b = 0; pitch rate = RZ₂₄(u₃ × 0.1); 0.15 and 0.1 double-precision constants) and RK9 (roll rate magnitude 0.1 to 0.25, pitch rate 0 to 0.1)
#[test]
fn the_crash_rates_follow_f5() {
    let (pitch, roll) = crash_rates(F64::ZERO, 1, F64::ZERO, RZ);
    assert_eq!(pitch, F32::ZERO);
    // 0.1 as a double, rounded to single toward zero.
    assert_eq!(roll, F32(0x3dcc_cccc));
    let (_, negative) = crash_rates(F64::ZERO, 0, F64::ZERO, RZ);
    assert_eq!(real(negative), -real(roll), "b = 0 negates the roll rate");
    let (pitch, roll) = crash_rates(one(), 1, one(), RZ);
    // The two doubles 0.15 and 0.1 sum to 0.25 exactly.
    assert_eq!(roll, F32(0x3e80_0000));
    assert_eq!(pitch, F32(0x3dcc_cccc));
    let half = F64(0.5f64.to_bits());
    let (pitch, roll) = crash_rates(half, 1, half, RZ);
    assert!((real(roll) - 0.175).abs() < 1e-7, "{}", real(roll));
    assert!((real(pitch) - 0.05).abs() < 1e-7, "{}", real(pitch));
}

/// A world with one flying-locomotor aircraft 1000 leptons up.
fn jet_world() -> (World, ObjectId) {
    use glasnost_rules::sim::{MovementZone, UnitType};
    let mut r = testkit::ruleset();
    r.registries.insert("aircraft".into(), vec!["JET".into()]);
    r.sim.aircraft.insert(
        "JET".into(),
        UnitType {
            strength: Some(100),
            speed: Some(10),
            locomotor: Some(Locomotor::Fly),
            movement_zone: Some(MovementZone::Fly),
            ..Default::default()
        },
    );
    let mut w = testkit::game_with(&r, SIZE, 1);
    let jet = testkit::spawn(
        &mut w,
        SimKind::Aircraft,
        "JET",
        0,
        testkit::mid(SIZE, 0, 0),
        0,
    );
    w.obj_mut(jet).unwrap().pos.z += 1000;
    (w, jet)
}

// spec: S-0113 claim 16 (an object that is not infantry starts to crash: S-0312 claim 16's three draws set its rates; while it crashes each angle adds its rate every turn, rounded to single, with no limit, pull or stop), claim 18 (the flying locomotor draws the pitch and roll only while crashing) and claim 20 (drawn fresh with those angles)
#[test]
fn a_crashing_aircraft_tumbles_by_its_draws() {
    let (mut w, jet) = jet_world();
    let mut rocking = Rocking::default();
    let mode = w.rules().mode;
    let half = w.rules().quirks.ranged_draw_half_span;
    let mut rng = w.rng.clone();
    let u1 = rng.unit(mode);
    let b = rng.ranged(0, 1, half);
    let u3 = rng.unit(mode);
    w.effects.clear();
    assert!(glasnost_sim::air::start_crash(&mut w, jet, false));
    assert_eq!(
        w.effects,
        vec![Effect::CrashTumble {
            id: jet,
            u1,
            sense: b,
            u3
        }]
    );
    assert_eq!(w.rng, rng, "the three draws, nothing else");
    let (pitch_rate, roll_rate) = crash_rates(u1, b, u3, mode);
    tick(&mut w, &mut rocking);
    let body = rocking.body(jet).unwrap();
    assert_eq!((body.pitch_rate, body.roll_rate), (pitch_rate, roll_rate));
    assert_eq!(
        (body.pitch, body.roll),
        (F32::ZERO, F32::ZERO),
        "shows from the next turn"
    );
    let (mut pitch, mut roll) = (F32::ZERO, F32::ZERO);
    for _ in 0..20 {
        tick(&mut w, &mut rocking);
        pitch = pitch.to_f64().add(pitch_rate.to_f64(), mode).to_f32(mode);
        roll = roll.to_f64().add(roll_rate.to_f64(), mode).to_f32(mode);
        let body = rocking.body(jet).unwrap();
        assert_eq!((body.pitch, body.roll), (pitch, roll));
        assert_eq!((body.pitch_rate, body.roll_rate), (pitch_rate, roll_rate));
    }
    assert!(real(roll).abs() > 1.9, "past every limit: {}", real(roll));
    // Drawn with the tumble: claim 20's exact angles, not held to the drawing step.
    let Attitude::Flying {
        crashing: true,
        tumble,
        ..
    } = attitude_with(&w, w.obj(jet).unwrap(), &rocking)
    else {
        panic!("a crashing aircraft");
    };
    assert_eq!(tumble, (pitch.0, roll.0));
    let (p, r) = rocking.exact_angles(jet).unwrap();
    assert_eq!(tumble, (p.to_bits(), r.to_bits()));
    assert!(r != 0.0);
}

// spec: S-0113 claim 18 (outside a crash the flying locomotor draws no pitch or roll)
#[test]
fn a_flying_aircraft_is_drawn_without_rocking() {
    let (w, jet) = jet_world();
    let rocking = Rocking::default();
    let Attitude::Flying {
        crashing: false,
        tumble,
        ..
    } = attitude_with(&w, w.obj(jet).unwrap(), &rocking)
    else {
        panic!("a flying aircraft");
    };
    assert_eq!(tumble, (0, 0));
}
