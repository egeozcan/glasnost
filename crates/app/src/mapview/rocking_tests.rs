//! Tests of the view's body rocking (S-0303 claim 20, S-0614 claims 9 and
//! 10, S-0210 claim 12, S-0504 claim 15) on synthetic data: every type name
//! and number is made up. The interim rules under test are ours (Q-2081).

use std::f64::consts::{FRAC_PI_4, PI};

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, Locomotor, WarheadType};
use glasnost_sim::combat::area::{self, Detonation};
use glasnost_sim::rules::Kind as SimKind;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Pos, World};

use super::build::Attitude;
use super::live::attitude_with;
use super::rocking::{
    Body, DRONE_STRENGTH, FORWARD_LIMIT, Rocking, SIDEWAYS_LIMIT, SLOWED_FORWARD_LIMIT, drawn,
    strength,
};
use super::sinking_tests::images;

const SIZE: i32 = 24;

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

/// A world with one vehicle facing map north (facing 0) at the centre.
fn world(weight: Option<&str>) -> (World, ObjectId) {
    let mut w = testkit::game_with(&ruleset(weight), SIZE, 3);
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

/// Sets off a rocking detonation of `damage` at `point`, then lets the
/// view take the tick's events and move one tick, as `Live::after_tick`
/// does after the game's tick.
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

/// One game tick for the view: its events taken, then one step.
fn tick(w: &mut World, rocking: &mut Rocking) {
    let effects = std::mem::take(&mut w.effects);
    w.tick += 1;
    rocking.take(w, &effects, w.tick - 1);
    rocking.advance(w);
}

fn offset(w: &World, id: ObjectId, dx: i32, dy: i32) -> Pos {
    let p = w.obj(id).unwrap().pos;
    Pos::new(p.x + dx, p.y + dy, p.z)
}

// spec: S-0303 claim 20 (strength min(D × 0.01, 4.0), exceeding 0.3 from D = 31, capped at 4.0 from D = 400)
#[test]
fn the_strength_of_a_damage_amount() {
    assert!(strength(31) > 0.3);
    assert!(strength(30) <= 0.3);
    assert_eq!(strength(400), 4.0);
    assert_eq!(strength(5000), strength(400));
    assert_eq!(strength(100), 1.0);
}

// spec: S-0504 Terms (a positive pitch lowers the bow; the roll about x, S-0105's Rx lifting the left, +y, side); S-0303 claim 20 (pushed away from the point); design: Q-2081 question 1 (the push's direction relative to the body splits the rates)
#[test]
fn a_blast_lifts_the_side_it_comes_from() {
    // Map north is −y; the body faces north (facing 0), so its right is +x.
    let cases = [
        ("ahead", (0, -200), (-1, 0)),
        ("behind", (0, 200), (1, 0)),
        ("on its left", (-200, 0), (0, 1)),
        ("on its right", (200, 0), (0, -1)),
    ];
    for (name, (dx, dy), (pitch, roll)) in cases {
        let (mut w, cart) = world(None);
        let mut rocking = Rocking::default();
        let point = offset(&w, cart, dx, dy);
        blast(&mut w, &mut rocking, point, 200);
        tick(&mut w, &mut rocking);
        let b = rocking.body(cart).unwrap();
        let sign = |v: f64| {
            if v.abs() < 1e-9 { 0 } else { v.signum() as i32 }
        };
        assert_eq!(
            (sign(b.pitch), sign(b.roll)),
            (pitch, roll),
            "a blast {name}"
        );
    }
}

// spec: S-0303 claim 20 (scaled by the type's weight; S-0504 claim 11: absent, 1.0); design: Q-2081 question 3 (interim: divided, so a heavier hull rocks less)
#[test]
fn a_heavier_hull_rocks_less() {
    let peak = |weight: Option<&str>| {
        let (mut w, cart) = world(weight);
        let mut rocking = Rocking::default();
        let point = offset(&w, cart, 0, 200);
        blast(&mut w, &mut rocking, point, 200);
        let mut peak = 0.0f64;
        for _ in 0..40 {
            tick(&mut w, &mut rocking);
            peak = peak.max(rocking.body(cart).map_or(0.0, |b| b.pitch.abs()));
        }
        peak
    };
    let (none, one, heavy, zero) = (
        peak(None),
        peak(Some("1")),
        peak(Some("4")),
        peak(Some("0")),
    );
    assert_eq!(none, one, "an absent weight is 1.0");
    assert!(heavy < one / 2.0, "{heavy} against {one}");
    assert_eq!(zero, one, "a weight that is not positive scales as 1.0");
}

// spec: S-0210 claim 12 (π/4 forward, π/10 while crush-slowed; at the limit the angle stays and the rate becomes level; the sideways limit unchanged); design: Q-2081 question 4 (sideways π/4)
#[test]
fn the_angles_stop_at_their_limits() {
    for (slowed, limit) in [(false, FORWARD_LIMIT), (true, SLOWED_FORWARD_LIMIT)] {
        let mut b = Body::default();
        // A strong push ahead on a very light body.
        b.push(0, 0, 4.0, 0.1);
        let mut reached = false;
        for _ in 0..20 {
            b.step(limit, false);
            assert!(b.pitch.abs() <= limit + 1e-12);
            if b.pitch == limit {
                reached = true;
                break;
            }
        }
        assert!(reached, "slowed {slowed}: the limit is reached");
        assert!(
            b.pitch_rate.abs() > 0.0,
            "the spring then pulls the level rate back"
        );
        assert!(b.pitch_rate < 0.0, "towards level");
        assert!(drawn(b.pitch) as f64 <= limit);
    }
    // The sideways angle keeps its own limit while the forward one is slowed.
    let mut b = Body::default();
    b.push(0, 16384, 4.0, 0.1);
    b.step(SLOWED_FORWARD_LIMIT, false);
    assert_eq!(b.roll, SIDEWAYS_LIMIT);
    const { assert!(SIDEWAYS_LIMIT > SLOWED_FORWARD_LIMIT) };
}

// spec: S-0210 claim 12 (while crush-slowed the forward angle is limited to π/10, whatever rocks it)
#[test]
fn a_crush_slowed_vehicle_pitches_at_most_a_tenth_of_pi() {
    let (mut w, cart) = world(Some("0.2"));
    w.obj_mut(cart).unwrap().crush_slowdown = true;
    let mut rocking = Rocking::default();
    let point = offset(&w, cart, 0, 200);
    blast(&mut w, &mut rocking, point, 400);
    // The hit's own tick takes the body to the limit, where the rate is
    // made level before the spring pulls it back.
    assert_eq!(
        rocking.body(cart).unwrap().pitch,
        PI / 10.0,
        "held at the limit"
    );
    let mut peak = 0.0f64;
    for _ in 0..40 {
        tick(&mut w, &mut rocking);
        peak = peak.max(rocking.body(cart).map_or(0.0, |b| b.pitch.abs()));
    }
    assert!(peak <= PI / 10.0, "never past it: {peak}");
}

// design: Q-2081 question 2 (interim: a damped return to level within about two seconds of game time); S-0106 Terms (below 0.005 radians a body does not rock)
#[test]
fn a_rocked_body_comes_back_to_level() {
    let (mut w, cart) = world(None);
    let mut rocking = Rocking::default();
    let point = offset(&w, cart, 150, 150);
    blast(&mut w, &mut rocking, point, 400);
    let mut crossed = false;
    let mut last_sign = 0.0;
    let mut ticks = 0;
    while rocking.body(cart).is_some() {
        let b = rocking.body(cart).unwrap();
        if last_sign != 0.0 && b.pitch.signum() != last_sign {
            crossed = true;
        }
        if b.pitch.abs() > 0.01 {
            last_sign = b.pitch.signum();
        }
        tick(&mut w, &mut rocking);
        ticks += 1;
        assert!(ticks < 120, "settles");
    }
    assert!(crossed, "it swings through level");
    assert!((40..=90).contains(&ticks), "{ticks} ticks");
    assert!(rocking.is_empty());
    assert_eq!(rocking.angles(cart, w.tick), None, "drawn level");
}

// spec: S-0504 claim 16 (the angles advance once per game tick, not once per drawn frame)
#[test]
fn the_angles_do_not_depend_on_the_frame_rate() {
    let run = |frames_per_tick: usize| {
        let (mut w, cart) = world(None);
        let mut rocking = Rocking::default();
        let point = offset(&w, cart, 0, 200);
        blast(&mut w, &mut rocking, point, 300);
        let mut seen = Vec::new();
        for _ in 0..30 {
            tick(&mut w, &mut rocking);
            for _ in 0..frames_per_tick {
                // A redraw without a tick moves nothing.
                rocking.advance(&w);
                seen.push(rocking.angles(cart, w.tick - 1));
            }
            seen.dedup();
        }
        seen
    };
    assert_eq!(run(1), run(4));
}

// spec: S-0106 Terms (a pitch or roll of 0.005 radians or more rocks); S-0105 claim 26 (each drawn pair is an image of its own); design: S-0303 claim 20 (the strength's precision is not fixed: the drawn angles are held to that step)
#[test]
fn drawn_angles_are_held_to_the_rocking_step() {
    assert_eq!(drawn(0.0049).to_bits(), 0);
    assert_eq!(drawn(-0.0049).to_bits(), 0, "never a negative zero");
    assert_eq!(drawn(0.005), glasnost_render::voxel::ROCKING_THRESHOLD);
    assert_eq!(drawn(0.0123), 0.01);
    assert!(drawn(FRAC_PI_4) as f64 <= FRAC_PI_4);
    // A full rock from the cap draws a bounded number of distinct poses.
    let (mut w, cart) = world(None);
    let mut rocking = Rocking::default();
    let point = offset(&w, cart, 90, 200);
    blast(&mut w, &mut rocking, point, 400);
    let mut poses = std::collections::BTreeSet::new();
    for _ in 0..120 {
        tick(&mut w, &mut rocking);
        if let Some((p, r)) = rocking.angles(cart, w.tick - 1) {
            poses.insert((p.to_bits(), r.to_bits()));
        }
    }
    assert!(poses.len() < 120, "{} poses", poses.len());
}

// spec: S-0303 claim 20 and S-0106 claim 9 (a rocking body of the driving locomotor is drawn with its pitch and roll); S-0504 claim 16 (only the driving, ship, hovering and jumpjet locomotors draw it)
#[test]
fn a_rocking_vehicle_is_drawn_rocking() {
    let (mut w, cart) = world(None);
    let mut rocking = Rocking::default();
    assert_eq!(
        attitude_with(&w, w.obj(cart).unwrap(), &rocking, w.tick),
        Attitude::None
    );
    let point = offset(&w, cart, 0, 200);
    blast(&mut w, &mut rocking, point, 300);
    tick(&mut w, &mut rocking);
    let Attitude::Rocking { pitch, roll, drawn } =
        attitude_with(&w, w.obj(cart).unwrap(), &rocking, w.tick - 1)
    else {
        panic!("rocking")
    };
    assert!(drawn);
    assert!(f32::from_bits(pitch) > 0.0, "a blast behind lowers the bow");
    assert_eq!(roll, 0);
    // Under a locomotor that draws no rocking the angles are kept, not drawn.
    let mut r = ruleset(None);
    r.sim.vehicles.get_mut("CART").unwrap().locomotor = Some(Locomotor::Walk);
    let mut w2 = testkit::game_with(&r, SIZE, 3);
    let walker = testkit::spawn(
        &mut w2,
        SimKind::Vehicle,
        "CART",
        0,
        testkit::mid(SIZE, 0, 0),
        0,
    );
    let mut rocking2 = Rocking::default();
    let point = offset(&w2, walker, 0, 200);
    blast(&mut w2, &mut rocking2, point, 300);
    tick(&mut w2, &mut rocking2);
    assert!(matches!(
        attitude_with(&w2, w2.obj(walker).unwrap(), &rocking2, w2.tick - 1),
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

// spec: S-0504 claim 15 (while sinking the forward rate is not added and the pitch is the lean, which starts from the pitch left over from rocking; the roll is left free and a rocking hit during descent rolls the hull) and F4
#[test]
fn a_sinking_vessel_leans_from_its_rocked_pitch_and_rolls_freely() {
    let (mut w, cart) = world(None);
    let mut rocking = Rocking::default();
    let point = offset(&w, cart, 0, 200);
    blast(&mut w, &mut rocking, point, 200);
    for _ in 0..3 {
        tick(&mut w, &mut rocking);
    }
    let p0 = rocking.body(cart).unwrap().pitch;
    assert!(p0 > 0.05, "rocked bow down: {p0}");
    glasnost_sim::naval::start_sinking(&mut w, cart);
    tick(&mut w, &mut rocking);
    let seeded = rocking.lean(cart).expect("the view steps the lean");
    assert_eq!(
        seeded.0,
        (p0 as f32).to_bits(),
        "the lean starts from the rocked pitch"
    );
    // Each descent takes one F4 step from there.
    let mut expect = seeded;
    for _ in 0..5 {
        assert!(!glasnost_sim::naval::sinking_turn(&mut w, cart));
        tick(&mut w, &mut rocking);
        let lean = w.obj(cart).unwrap().naval.as_ref().unwrap().lean.unwrap();
        expect = glasnost_sim::naval::pitch_step(expect, lean, w.rules().mode);
        assert_eq!(rocking.lean(cart), Some(expect));
        let Attitude::Sinking { pitch, roll, .. } =
            attitude_with(&w, w.obj(cart).unwrap(), &rocking, w.tick - 1)
        else {
            panic!("sinking")
        };
        assert_eq!(pitch, expect.0, "drawn from the view's lean");
        let _ = roll;
    }
    // A blast from the left during descent rolls the hull and leaves the
    // lean to its steps.
    let before = rocking.lean(cart).unwrap();
    let point = offset(&w, cart, -200, 0);
    blast(&mut w, &mut rocking, point, 300);
    tick(&mut w, &mut rocking);
    let lean = w.obj(cart).unwrap().naval.as_ref().unwrap().lean.unwrap();
    let mode = w.rules().mode;
    let after = glasnost_sim::naval::pitch_step(
        glasnost_sim::naval::pitch_step(before, lean, mode),
        lean,
        mode,
    );
    assert_eq!(rocking.lean(cart), Some(after));
    let b = rocking.body(cart).unwrap();
    assert!(b.roll > 0.0, "rolled away from the blast");
    assert_eq!(b.pitch_rate, 0.0, "no forward rate while sinking");
    let Attitude::Sinking { pitch, roll, .. } =
        attitude_with(&w, w.obj(cart).unwrap(), &rocking, w.tick - 1)
    else {
        panic!("sinking")
    };
    assert_eq!(pitch, after.0);
    assert!(f32::from_bits(roll) > 0.0, "the roll is drawn");
}

// spec: S-0614 claim 9 and F2 (a drone's delivery rocks the host sideways with strength 1.5, to the side the draw chose); design: Q-2081 question 6 (+s on the host's right; pushed away from it)
#[test]
fn a_drone_rocks_its_host_sideways() {
    use glasnost_sim::combat::effects::Effect;
    for (side, sign) in [(0, -1.0), (1, 1.0)] {
        let (mut w, cart) = world(None);
        let mut rocking = Rocking::default();
        w.effects.push(Effect::DroneRock { host: cart, side });
        tick(&mut w, &mut rocking);
        let b = rocking.body(cart).unwrap();
        assert!(b.pitch.abs() < 1e-12, "no pitch");
        assert_eq!(b.roll.signum(), sign, "side {side}");
        // The first step's roll is the rate the push set.
        let mut c = Body::default();
        c.push(0, 16384, DRONE_STRENGTH, 1.0);
        assert!((b.roll - sign * c.roll_rate).abs() < 1e-12);
    }
}

// spec: S-0614 claim 10 (the push's tilt sin(k × (±π/10)) × π/4 at frame k, 4 ticks a frame, 0 at the strike; only while the host still has its grappling parasite); design: Q-2081 question 7 (the roll; draw 1 positive)
#[test]
fn a_squid_rocks_its_host_through_each_push() {
    use glasnost_sim::combat::effects::Effect;
    for (way, sign) in [(1, 1.0), (0, -1.0)] {
        let (w, cart) = world(None);
        let mut rocking = Rocking::default();
        let start = 7;
        rocking.take(&w, &[Effect::GrapplePush { host: cart, way }], start);
        assert_eq!(rocking.angles(cart, start), None, "0 at the strike");
        for k in 1..10 {
            let tilt = (f64::from(k) * sign * PI / 10.0).sin() * FRAC_PI_4;
            for t in 0..4 {
                let (pitch, roll) = rocking.angles(cart, start + 4 * k + t).unwrap();
                assert_eq!(pitch, 0.0);
                assert_eq!(roll, drawn(tilt), "way {way}, frame {k}");
            }
        }
        assert_eq!(rocking.angles(cart, start + 40), None, "after ten frames");
        // No parasite holds the host in this scene, so its next tick ends the push.
        rocking.advance(&w);
        assert!(rocking.is_empty());
    }
}

// spec: S-0601 claim 16 and F4 (an overload pulse sets the controller's sideways rate: 0.015 radians per tick at level 1, 0.03 from level 2, negative for a draw below 50; the nearest single values; set, not scaled by weight; the pitch is left alone)
#[test]
fn an_overload_pulse_sets_the_sideways_rate() {
    use glasnost_sim::combat::effects::Effect;
    for (level, draw, rate) in [
        (1, 49, -0.015f32),
        (1, 50, 0.015),
        (2, 0, -0.03),
        (5, 100, 0.03),
    ] {
        let (mut w, cart) = world(Some("4"));
        let mut rocking = Rocking::default();
        w.effects.push(Effect::OverloadRock {
            controller: cart,
            level,
            draw,
        });
        tick(&mut w, &mut rocking);
        let b = rocking.body(cart).unwrap();
        assert_eq!(b.roll, f64::from(rate), "level {level}, draw {draw}");
        assert_eq!((b.pitch, b.pitch_rate), (0.0, 0.0));
    }
    // A pulse on a body already rocking replaces its sideways rate only.
    let (mut w, cart) = world(None);
    let mut rocking = Rocking::default();
    let point = offset(&w, cart, 150, 150);
    blast(&mut w, &mut rocking, point, 200);
    let before = rocking.body(cart).unwrap();
    w.effects.push(Effect::OverloadRock {
        controller: cart,
        level: 2,
        draw: 80,
    });
    let mut expect = before;
    expect.roll_rate = f64::from(0.03f32);
    expect.step(FORWARD_LIMIT, false);
    tick(&mut w, &mut rocking);
    assert_eq!(rocking.body(cart), Some(expect));
}
