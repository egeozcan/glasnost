//! Claim tests for S-0109 revision 41 (M8 step A3 intake 22) on synthetic
//! data: invented animation, vehicle and warhead types, every number our
//! own. The warhead of an animation's damage by its current type's name
//! (claim 17 and Import mapping, Q-1717), the order of an advance's damage,
//! middle event and turn and pass tests (State and ordering, claims 14 and
//! 17, Q-1716), and an attached animation whose own detonation removes its
//! owner (Open questions, our engine's design). The animation that leaves
//! play in its own detonation is tested with the drain in `claims_s0608.rs`.
//! The ledger is `docs/m8-claims/intake22-S-0109.md`.
//!
//! Animations are made with `anims::spawn` between ticks: tick *c* gives
//! them their first update (no advance, S-0109 claim 5) and tick *c* + 1
//! their first advance at the rate 900 (*d* = 1, F1).

mod harvest_common;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{AnimationType, Decimal, UnitType, WarheadType};
use glasnost_sim::anims::{self, Spawn};
use glasnost_sim::rng::Rng;
use glasnost_sim::rules::Kind;
use glasnost_sim::tick;
use glasnost_sim::world::{ObjectId, Pos, World};
use harvest_common::{at, game_with_ore, put, stage};

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n.eq_ignore_ascii_case(name)) {
        l.push(name.into());
    }
}

fn anim(r: &mut Ruleset, name: &str, t: AnimationType) {
    register(r, "animations", name);
    r.sim.animations.insert(name.into(), t);
}

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

/// `frames` image frames at rate 900 (d = 1, F1), one pass.
fn frames(frames: i32) -> AnimationType {
    AnimationType {
        frame_count: Some(frames),
        rate: Some(900),
        ..Default::default()
    }
}

/// Damage δ = 1.0 over `n` frames, μ = 2 with no middle-event effects.
fn hurting(n: i32) -> AnimationType {
    AnimationType {
        damage: Some(d("1")),
        middle_frame: Some(2),
        ..frames(n)
    }
}

/// The rules: the resource fixture of `harvest_common` plus
/// - the warheads FLAME (the game-wide flame warhead, 0 % against every
///   armour) and BOOM (the demolition warhead, 100 %);
/// - CAN: a vehicle with no weapon; BLOWN: one of strength 1 whose
///   explosion list has two entries (a list pick at its death, S-0312
///   claim 9);
/// - PLAIN: four frames, μ = 2, no effects; HURT: eight frames of δ = 1.0.
fn rules() -> Ruleset {
    let mut r = harvest_common::ruleset();
    for (name, v) in [("FLAME", "0"), ("BOOM", "1")] {
        register(&mut r, "warheads", name);
        r.sim.warheads.insert(
            name.into(),
            WarheadType {
                verses: Some((0..11).map(|_| d(v)).collect()),
                ..Default::default()
            },
        );
    }
    r.sim.general.flame_damage2 = Some("FLAME".into());
    r.sim.general.c4_warhead = Some("BOOM".into());
    let can = UnitType {
        strength: Some(300),
        primary: None,
        ..r.sim.vehicles["CART"].clone()
    };
    let blown = UnitType {
        strength: Some(1),
        combat: glasnost_rules::sim::CombatFields {
            explosion: Some(vec!["PLAIN".into(), "PLAIN".into()]),
            ..can.combat.clone()
        },
        ..can.clone()
    };
    for (name, v) in [("CAN", can), ("BLOWN", blown)] {
        register(&mut r, "vehicles", name);
        r.sim.vehicles.insert(name.into(), v);
    }
    anim(
        &mut r,
        "PLAIN",
        AnimationType {
            middle_frame: Some(2),
            ..frames(4)
        },
    );
    anim(&mut r, "HURT", hurting(8));
    r
}

fn game_of(r: &Ruleset, seed: u32) -> World {
    game_with_ore(r, &[], seed)
}

/// The ground point at the centre of cell `c`.
fn ground(w: &World, c: CellXY) -> Pos {
    let (x, y) = (c.x * 256 + 128, c.y * 256 + 128);
    Pos::new(x, y, w.surface_at(x, y))
}

fn make(w: &mut World, name: &str, c: CellXY, s: Spawn) -> ObjectId {
    let p = ground(w, c);
    anims::spawn(w, name, Some(p), s)
        .id
        .expect("a listed type is in play")
}

fn vehicle(w: &mut World, name: &str, c: CellXY) -> ObjectId {
    glasnost_sim::testkit::spawn(w, Kind::Vehicle, name, 0, c, 0)
}

fn health(w: &World, id: ObjectId) -> i32 {
    w.obj(id).map_or(0, |o| o.health)
}

fn steps(w: &mut World, n: usize) {
    for _ in 0..n {
        tick::step(w);
    }
}

/// The scorch-or-crater choice of a raw stream state: a crater when the
/// ranged draw over [0, 2147483646] is 1073741824 or more (S-0109 claim 14).
fn crater_from(mut e: Rng, half: bool) -> bool {
    e.ranged(0, 2147483646, half) >= 1073741824
}

// ---------------------------------------------------------------------------
// Claim 17 and Import mapping: the warhead by the current type's name (Q-1717).

#[test]
fn claim17_the_invisibility_type_takes_the_demolition_warhead() {
    // spec: S-0109 claim 17 and Import mapping (C4Warhead for the type whose name is exactly the built-in invisibility-animation type's, FlameDamage2 for every other; the type's own warhead field plays no part)
    let mut r = rules();
    anim(&mut r, "INVISO", hurting(8));
    let mut w = game_of(&r, 3);
    let (ci, co) = (at(-4, 0), at(4, 0));
    let vi = vehicle(&mut w, "CAN", ci);
    let vo = vehicle(&mut w, "CAN", co);
    let (hi, ho) = (health(&w, vi), health(&w, vo));
    make(&mut w, "INVISO", ci, Spawn::default());
    make(&mut w, "HURT", co, Spawn::default());
    // Tick c: the first update; then every advance delivers (A₀ = 1.0, δ = 1.0).
    tick::step(&mut w);
    assert_eq!((health(&w, vi), health(&w, vo)), (hi, ho), "no advance yet");
    let mut last = hi;
    for _ in 0..4 {
        tick::step(&mut w);
        let now = health(&w, vi);
        assert!(now < last, "a loss on every advance under 100 %");
        last = now;
    }
    assert_eq!(health(&w, vo), ho, "the flame warhead's 0 % does nothing");
}

#[test]
fn claim17_a_type_first_registered_in_other_case_never_matches() {
    // spec: S-0109 Import mapping (the comparison is exact, letter case included, with the spelling the type was first registered with, S-0004 claims 2 and 7; a type first registered as Inviso takes FlameDamage2, and with no type of exactly that name C4Warhead is never chosen)
    let mut r = rules();
    anim(&mut r, "Inviso", hurting(8));
    let mut w = game_of(&r, 3);
    assert!(
        w.rules()
            .animation("INVISO")
            .is_some_and(|a| a.name == "Inviso"),
        "found without regard to letter case, under its first spelling"
    );
    let c = at(0, 0);
    let v = vehicle(&mut w, "CAN", c);
    let h = health(&w, v);
    make(&mut w, "INVISO", c, Spawn::default());
    steps(&mut w, 9);
    assert_eq!(health(&w, v), h, "every delivery took the flame warhead");
}

#[test]
fn claim17_a_follow_up_switch_changes_the_warhead() {
    // spec: S-0109 claim 17 (the warhead is chosen on each delivery by the current type's name: a follow-up switch to the invisibility-animation type changes it from the follow-up's first delivery on) and claim 15 and F5 (the accumulator restarts at 0.0 at the switch); the follow-up name in other letter case finds the type registered in capitals
    let mut r = rules();
    anim(&mut r, "INVISO", hurting(8));
    anim(
        &mut r,
        "LEAD",
        AnimationType {
            next: Some("inviso".into()),
            ..hurting(4)
        },
    );
    let mut w = game_of(&r, 3);
    let c = at(0, 0);
    let v = vehicle(&mut w, "CAN", c);
    let h = health(&w, v);
    let id = make(&mut w, "LEAD", c, Spawn::default());
    // Tick c, then LEAD's four advances (c + 1 … c + 4), the last switching.
    steps(&mut w, 5);
    assert_eq!(health(&w, v), h, "no loss during the first type's advances");
    assert_eq!(
        anims::shown(&w, id).map(|s| s.0.to_vec()),
        Some(b"inviso".to_vec())
    );
    // The follow-up's first advance: A = 0.0 + 1.0 delivers 1 under 100 %.
    tick::step(&mut w);
    assert!(
        health(&w, v) < h,
        "a loss from the follow-up's first delivery"
    );
}

// ---------------------------------------------------------------------------
// State and ordering (Q-1716): damage, then the middle event, then the turn and pass tests.

/// The rules of the order tests: BANG, four frames of δ = 9.0 with μ = 1
/// (reached by the first advance) and both smudge flags; QUIET, the same
/// without the flags; and the variants of `extra`.
fn order_rules(extra: impl FnOnce(&mut Ruleset)) -> Ruleset {
    let mut r = rules();
    let bang = AnimationType {
        damage: Some(d("9")),
        middle_frame: Some(1),
        scorch: Some(true),
        crater: Some(true),
        ..frames(4)
    };
    let quiet = AnimationType {
        scorch: None,
        crater: None,
        ..bang.clone()
    };
    anim(&mut r, "BANG", bang);
    anim(&mut r, "QUIET", quiet);
    // The damage takes the demolition warhead's 100 %, so that BLOWN dies in the detonation.
    r.sim.general.flame_damage2 = Some("BOOM".into());
    extra(&mut r);
    r
}

/// Plays `name` on a BLOWN vehicle's cell holding resource at stage 8 (an
/// overlay without the chain-reaction flag, so the detonation leaves it),
/// up to the tick of the first advance: the stream state before that tick,
/// the world after it and the animation.
fn first_advance(r: &Ruleset, name: &str, seed: u32) -> (Rng, World, ObjectId) {
    let mut w = game_of(r, seed);
    let c = at(0, 0);
    put(&mut w, c, 0, 8);
    vehicle(&mut w, "BLOWN", c);
    let id = make(&mut w, name, c, Spawn::default());
    tick::step(&mut w);
    let before = w.rng.clone();
    tick::step(&mut w);
    (before, w, id)
}

#[test]
fn state_and_ordering_the_detonation_comes_before_the_middle_event() {
    // spec: S-0109 State and ordering and claims 14 and 17 (Q-1716: the advance's damage detonation, its kills and their draws included, is complete before the middle event's scorch-or-crater choice draws)
    let r = order_rules(|_| {});
    let half = glasnost_sim::rules::SimRules::new(&r, "t")
        .quirks
        .ranged_draw_half_span;
    let mut told = false;
    for seed in 1..40 {
        // The detonation alone: its kill takes the explosion list pick.
        let (s0, quiet, _) = first_advance(&r, "QUIET", seed);
        assert_ne!(quiet.rng, s0, "the detonation's kill takes draws");
        let (b0, w, _) = first_advance(&r, "BANG", seed);
        assert_eq!(b0, s0);
        // Damage first: the choice draws from the state after the detonation's draws.
        let first = crater_from(quiet.rng.clone(), half);
        let wrong = crater_from(s0.clone(), half);
        told |= first != wrong;
        assert_eq!(
            stage(&w, at(0, 0)),
            Some(if first { 2 } else { 8 }),
            "seed {seed}"
        );
    }
    assert!(told, "some seed tells the two orders apart");
}

#[test]
fn state_and_ordering_the_middle_event_comes_before_the_end() {
    // spec: S-0109 State and ordering and claim 14 (Q-1716: reaching μ on the advance that ends the animation still produces the middle event, before the end)
    let r = order_rules(|r| {
        let t = r.sim.animations.get_mut("BANG").unwrap();
        t.end = Some(1);
        t.damage = None;
    });
    let half = glasnost_sim::rules::SimRules::new(&r, "t")
        .quirks
        .ranged_draw_half_span;
    let mut w = game_of(&r, 5);
    let c = at(0, 0);
    put(&mut w, c, 0, 8);
    let id = make(&mut w, "BANG", c, Spawn::default());
    tick::step(&mut w);
    let mut e = w.rng.clone();
    let crater = e.clone();
    e.ranged(0, 2147483646, half);
    tick::step(&mut w);
    assert_eq!(w.rng, e, "one choice draw on the ending advance");
    assert_eq!(
        stage(&w, c),
        Some(if crater_from(crater, half) { 2 } else { 8 })
    );
    assert!(anims::shown(&w, id).is_none(), "removed in that update");
}

#[test]
fn state_and_ordering_the_middle_event_comes_before_a_follow_up_switch() {
    // spec: S-0109 State and ordering and claims 14, 15 and 28 (Q-1716: on the advance before a follow-up switch the old type's middle event comes first, then the switch with its delay draw)
    let r = order_rules(|r| {
        let t = r.sim.animations.get_mut("BANG").unwrap();
        t.end = Some(1);
        t.damage = None;
        t.next = Some("SLOW".into());
        anim(
            r,
            "SLOW",
            AnimationType {
                random_rate: Some([450, 150]),
                ..frames(4)
            },
        );
    });
    let half = glasnost_sim::rules::SimRules::new(&r, "t")
        .quirks
        .ranged_draw_half_span;
    let (lo, hi) = glasnost_sim::rules::SimRules::new(&r, "t")
        .animation("SLOW")
        .unwrap()
        .random_delay;
    assert!(lo < hi, "a range to draw from");
    let mut told = false;
    for seed in 1..40 {
        let mut w = game_of(&r, seed);
        let c = at(0, 0);
        let id = make(&mut w, "BANG", c, Spawn::default());
        tick::step(&mut w);
        let s0 = w.rng.clone();
        tick::step(&mut w);
        let mut e = s0.clone();
        e.ranged(0, 2147483646, half);
        let right = e.ranged(lo, hi, half);
        let wrong = s0.clone().ranged(lo, hi, half);
        told |= right != wrong;
        assert_eq!(w.rng, e, "seed {seed}");
        let a = &w.anims.anims[&id];
        assert_eq!(a.player.name, b"SLOW");
        assert_eq!(a.player.delay(), right, "seed {seed}");
    }
    assert!(told, "some seed tells the two orders apart");
}

// ---------------------------------------------------------------------------
// Open questions: an attached animation whose own detonation removes its owner.

#[test]
fn open_question_owner_removed_by_its_own_detonation() {
    // spec: S-0109 Open questions and claim 22 (an attached animation whose own damage detonation removes its owner goes on with the rest of that update, its middle event at the owner's last position, our engine's design; it is deleted at its next update)
    let r = order_rules(|_| {});
    let half = glasnost_sim::rules::SimRules::new(&r, "t")
        .quirks
        .ranged_draw_half_span;
    for seed in [1, 2, 3, 4] {
        let mut runs = Vec::new();
        for name in ["QUIET", "BANG"] {
            let mut w = game_of(&r, seed);
            let c = at(0, 0);
            put(&mut w, c, 0, 8);
            let owner = vehicle(&mut w, "BLOWN", c);
            let s = Spawn {
                owner: Some((owner, Pos::new(0, 0, 0))),
                ..Default::default()
            };
            let id = make(&mut w, name, c, s);
            tick::step(&mut w);
            tick::step(&mut w);
            assert!(w.obj(owner).is_none_or(|o| o.health == 0), "the owner died");
            runs.push((w, id));
        }
        let id = runs[1].1;
        let (quiet, _) = &runs[0];
        let w = &runs[1].0;
        // One more draw than the detonation alone: the middle event's choice.
        let mut e = quiet.rng.clone();
        let crater = crater_from(e.clone(), half);
        e.ranged(0, 2147483646, half);
        assert_eq!(w.rng, e, "seed {seed}");
        assert_eq!(stage(w, at(0, 0)), Some(if crater { 2 } else { 8 }));
        assert!(anims::shown(w, id).is_some(), "still in play this tick");
        let mut w = runs.pop().unwrap().0;
        tick::step(&mut w);
        assert!(anims::shown(&w, id).is_none(), "deleted at its next update");
    }
}
