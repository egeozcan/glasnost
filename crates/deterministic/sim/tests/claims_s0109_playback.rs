//! Claim tests for animation playback in the simulation (S-0109 claims 5,
//! 11, 13 to 17, 22, 28 and 29; S-0203 claim 10 for the removal skip), on
//! synthetic data: invented animation and warhead types, every number our
//! own. The ledger is `docs/m8-claims/S-0109-playback.md`.
//!
//! Animations are made with `anims::spawn` between ticks, so tick `c` (the
//! tick number at creation) is the tick whose turn pass gives them their
//! first update (S-0109 claim 5, *Which update is first*: created before
//! the turn pass).

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{AnimationType, Decimal, WarheadType};
use glasnost_sim::anim_clock::Creation;
use glasnost_sim::anims::{self, Spawn};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::world::{ObjectId, Pos, World};
use glasnost_sim::{save, testkit, tick};

const SIZE: i32 = 16;

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn anim(r: &mut Ruleset, name: &str, t: AnimationType) {
    register(r, "animations", name);
    r.sim.animations.insert(name.into(), t);
}

/// Four frames at rate 900 (d = 1, F1), one pass.
fn four() -> AnimationType {
    AnimationType {
        frame_count: Some(4),
        rate: Some(900),
        ..Default::default()
    }
}

/// The rules: testkit's combat rules plus
/// - PLAIN: four frames, d = 1, μ = 2 with no middle-event effects;
/// - MID: as PLAIN with the scorch and crater flags (μ = 2);
/// - START: as MID with μ = 0;
/// - LOOPS: four frames, d = 1, two passes, random loop delay (2, 5);
/// - HURT: eight frames, d = 1, damage 0.375;
/// - FIRST and NEXT: FIRST four frames with follow-up NEXT (four frames,
///   damage 0.375);
/// - the flame warhead FLAME.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    anim(
        &mut r,
        "PLAIN",
        AnimationType {
            middle_frame: Some(2),
            ..four()
        },
    );
    anim(
        &mut r,
        "MID",
        AnimationType {
            middle_frame: Some(2),
            scorch: Some(true),
            crater: Some(true),
            ..four()
        },
    );
    anim(
        &mut r,
        "START",
        AnimationType {
            middle_frame: Some(0),
            scorch: Some(true),
            crater: Some(true),
            ..four()
        },
    );
    anim(
        &mut r,
        "LOOPS",
        AnimationType {
            loop_count: Some(2),
            random_loop_delay: Some([2, 5]),
            ..four()
        },
    );
    anim(
        &mut r,
        "HURT",
        AnimationType {
            frame_count: Some(8),
            damage: Some(Decimal("0.375".into())),
            ..four()
        },
    );
    anim(
        &mut r,
        "FIRST",
        AnimationType {
            next: Some("NEXT".into()),
            ..four()
        },
    );
    anim(
        &mut r,
        "NEXT",
        AnimationType {
            damage: Some(Decimal("0.375".into())),
            ..four()
        },
    );
    register(&mut r, "warheads", "FLAME");
    r.sim.warheads.insert(
        "FLAME".into(),
        WarheadType {
            verses: Some((0..11).map(|_| Decimal("1".into())).collect()),
            ..Default::default()
        },
    );
    r.sim.general.flame_damage2 = Some("FLAME".into());
    r
}

fn game() -> World {
    testkit::game_with(&rules(), SIZE, 3)
}

/// The ground point at the centre of cell `c`.
fn ground(w: &World, c: CellXY) -> Pos {
    let (x, y) = (c.x * 256 + 128, c.y * 256 + 128);
    Pos::new(x, y, w.surface_at(x, y))
}

fn centre(w: &World) -> Pos {
    ground(w, testkit::mid(SIZE, 0, 0))
}

fn make(w: &mut World, name: &str, s: Spawn) -> ObjectId {
    let at = centre(w);
    anims::spawn(w, name, Some(at), s)
        .id
        .expect("a listed type is in play")
}

/// The image frame `id` shows after each of `n` ticks, `None` once removed.
fn frames(w: &mut World, id: ObjectId, n: usize) -> Vec<Option<i32>> {
    (0..n)
        .map(|_| {
            tick::step(w);
            anims::shown(w, id).map(|s| s.1)
        })
        .collect()
}

#[test]
fn an_empty_tick_takes_no_draw() {
    // The fixtures below count draws against an empty world.
    let mut w = game();
    let before = w.rng.clone();
    tick::step(&mut w);
    assert_eq!(w.rng, before);
}

#[test]
fn claim29_appended_to_the_update_order() {
    // spec: S-0109 claim 29 (an animation is created at its event and appended to the update order)
    let mut w = game();
    let id = make(&mut w, "PLAIN", Spawn::default());
    assert_eq!(w.update.order.last(), Some(&id));
    assert!(w.anims.anims.contains_key(&id));
}

#[test]
fn claims5_16_first_update_and_lifetime() {
    // spec: S-0109 claim 5 (the first update makes no advance; with d = 1 and u = c the first advance is in c + 1) and claim 16 (lifetime E × d = 4 ticks; the frame at the end limit is never drawn)
    let mut w = game();
    let id = make(&mut w, "PLAIN", Spawn::default());
    assert_eq!(anims::shown(&w, id).map(|s| s.1), Some(0));
    assert_eq!(
        frames(&mut w, id, 5),
        [Some(0), Some(1), Some(2), Some(3), None]
    );
    assert!(!w.update.order.contains(&id), "removed in its own turn");
}

#[test]
fn claim14_middle_event_on_the_advance_to_mu() {
    // spec: S-0109 claim 14 (the middle event on the advance after which a + k = μ, μ ≠ 0: both flags take one ranged draw over [0, 2147483646]); claim 13 (μ ≠ 0: the creation start event has no middle event)
    let mut w = game();
    let before = w.rng.clone();
    let id = make(&mut w, "MID", Spawn::default());
    assert_eq!(w.rng, before, "no draw at creation when μ ≠ 0");
    let half = w.rules().quirks.ranged_draw_half_span;
    // Ticks c (first update) and c + 1 (k = 1): no draw.
    tick::step(&mut w);
    tick::step(&mut w);
    assert_eq!(w.rng, before);
    // Tick c + 2: k = 2 = μ.
    let mut expected = before.clone();
    expected.ranged(0, 2147483646, half);
    tick::step(&mut w);
    assert_eq!(w.rng, expected);
    assert_eq!(anims::shown(&w, id).map(|s| s.1), Some(2));
    tick::step(&mut w);
    tick::step(&mut w);
    assert_eq!(w.rng, expected, "no more middle events");
    assert!(anims::shown(&w, id).is_none());
}

#[test]
fn claim13_start_event_at_creation_with_mu_zero() {
    // spec: S-0109 claim 13 (with no loop delay the start event comes at creation and produces the middle event at once when μ = 0)
    let mut w = game();
    let half = w.rules().quirks.ranged_draw_half_span;
    let mut expected = w.rng.clone();
    expected.ranged(0, 2147483646, half);
    make(&mut w, "START", Spawn::default());
    assert_eq!(w.rng, expected);
}

#[test]
fn claims11_13_29_trailer_loop_delay_defers_the_start_event() {
    // spec: S-0109 claim 29 (a projectile trailer: an initial loop delay of 1 update) and claim 11 (counted from the second update; its start event comes on update D + 1 after creation, with no advance)
    let mut w = game();
    let before = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    let s = Spawn {
        creation: Creation {
            loop_delay: 1,
            repeat: 1,
            reverse: false,
        },
        ..Default::default()
    };
    let id = make(&mut w, "START", s);
    assert_eq!(w.rng, before, "no start event at creation");
    tick::step(&mut w);
    assert_eq!(w.rng, before, "the first update counts no delay down");
    let mut expected = before.clone();
    expected.ranged(0, 2147483646, half);
    tick::step(&mut w);
    assert_eq!(w.rng, expected, "the start event at the delay's end");
    assert_eq!(anims::shown(&w, id).map(|s| s.1), Some(0), "no advance");
    // The clock was not restarted: the advance due since c + 1 comes now.
    tick::step(&mut w);
    assert_eq!(anims::shown(&w, id).map(|s| s.1), Some(1));
}

#[test]
fn claims11_28_loop_delay_draw_from_the_scenario_stream() {
    // spec: S-0109 claim 11 (a pass followed by a new pass: a ranged draw over the pair; D updates without advance) and claim 28 (the draw is from the scenario stream, in the animation's own turn)
    let mut w = game();
    let half = w.rules().quirks.ranged_draw_half_span;
    let id = make(&mut w, "LOOPS", Spawn::default());
    let before = w.rng.clone();
    // Tick c is the first update; the advances in c + 1 … c + 4, the one in
    // c + 4 ending the first pass.
    for _ in 0..5 {
        tick::step(&mut w);
    }
    let mut expected = before.clone();
    let d = expected.ranged(2, 5, half);
    assert_eq!(w.rng, expected);
    assert_eq!(anims::shown(&w, id).map(|s| s.1), Some(0), "the new pass");
    // D updates count down without advancing; then the clock's due advance.
    for _ in 0..d {
        tick::step(&mut w);
        assert_eq!(anims::shown(&w, id).map(|s| s.1), Some(0));
    }
    tick::step(&mut w);
    assert_eq!(anims::shown(&w, id).map(|s| s.1), Some(1));
}

/// The damage amounts delivered at `at` in each of `n` ticks.
fn deliveries(w: &mut World, at: Pos, n: usize) -> Vec<i32> {
    (0..n)
        .map(|_| {
            w.effects.clear();
            tick::step(w);
            w.effects
                .iter()
                .filter_map(|e| match e {
                    Effect::Detonation { pos, damage, .. } if *pos == at => Some(*damage),
                    _ => None,
                })
                .sum()
        })
        .collect()
}

#[test]
fn claim17_damage_over_time_pattern() {
    // spec: S-0109 claim 17 and F5 (A₀ = 1.0, δ = 0.375 gives 1, 0, 1, 0, 0, 1, 0, 1 over the first advances; the ending advance delivers too)
    let mut w = game();
    let at = centre(&w);
    make(&mut w, "HURT", Spawn::default());
    // Tick c: the first update, no advance.
    let got = deliveries(&mut w, at, 9);
    assert_eq!(got, [0, 1, 0, 1, 0, 0, 1, 0, 1]);
}

#[test]
fn claim17_damage_accumulator_quirk_off() {
    // spec: S-0109 F5 (quirks.anim_damage_starts_at_one off: A₀ = 0.0, so δ = 0.375 first delivers on the third advance)
    let mut r = rules();
    r.globals.quirks.anim_damage_starts_at_one = false;
    let mut w = testkit::game_with(&r, SIZE, 3);
    let at = centre(&w);
    make(&mut w, "HURT", Spawn::default());
    let got = deliveries(&mut w, at, 4);
    assert_eq!(got, [0, 0, 0, 1]);
}

#[test]
fn claim15_follow_up_restarts_the_accumulator_at_zero() {
    // spec: S-0109 claim 15 (the same animation continues as the follow-up type, k = a′ = 0) and F5 (after a follow-up switch the accumulator restarts at 0.0: δ = 0.375 first delivers on the third advance of NEXT)
    let mut w = game();
    let at = centre(&w);
    let id = make(&mut w, "FIRST", Spawn::default());
    // FIRST: advances in c + 1 … c + 4, the last switching to NEXT.
    let got = deliveries(&mut w, at, 5);
    assert_eq!(got, [0, 0, 0, 0, 0]);
    assert_eq!(
        anims::shown(&w, id).map(|s| s.0.to_vec()),
        Some(b"NEXT".to_vec())
    );
    let got = deliveries(&mut w, at, 3);
    assert_eq!(got, [0, 0, 1]);
}

#[test]
fn claim22_owner_removed_deletes_at_the_next_update() {
    // spec: S-0109 claim 22 (an attached animation follows its owner at its displacement; the owner removed from the game: deleted at its next update, with nothing else)
    let mut w = game();
    let c = testkit::mid(SIZE, 2, 0);
    let owner = testkit::spawn(&mut w, glasnost_sim::rules::Kind::Vehicle, "CART", 0, c, 0);
    let d = Pos::new(10, -20, 30);
    let s = Spawn {
        owner: Some((owner, d)),
        ..Default::default()
    };
    let id = make(&mut w, "PLAIN", s);
    tick::step(&mut w);
    let p = w.obj(owner).unwrap().pos;
    assert_eq!(
        anims::shown(&w, id).and_then(|s| s.2),
        Some(Pos::new(p.x + d.x, p.y + d.y, p.z + d.z))
    );
    glasnost_sim::objects::remove(&mut w, owner);
    assert!(anims::shown(&w, id).is_some(), "until its next update");
    tick::step(&mut w);
    assert!(anims::shown(&w, id).is_none());
    assert!(!w.update.order.contains(&id));
}

#[test]
fn claim29_removal_in_own_turn_costs_the_next_animation_a_turn() {
    // spec: S-0109 claim 29 and S-0203 claim 10 (an animation removed in its own turn is a counting removal: the next object misses that turn, so its end comes a tick later)
    let mut w = game();
    let a = make(&mut w, "PLAIN", Spawn::default());
    let b = make(&mut w, "PLAIN", Spawn::default());
    let mut ends = (None, None);
    for t in 0..8 {
        tick::step(&mut w);
        if ends.0.is_none() && anims::shown(&w, a).is_none() {
            ends.0 = Some(t);
        }
        if ends.1.is_none() && anims::shown(&w, b).is_none() {
            ends.1 = Some(t);
        }
    }
    assert_eq!(ends, (Some(4), Some(5)));
    // quirks.update_order_removal_skip off: both end together.
    let mut r = rules();
    r.globals.quirks.update_order_removal_skip = false;
    let mut w = testkit::game_with(&r, SIZE, 3);
    let a = make(&mut w, "PLAIN", Spawn::default());
    let b = make(&mut w, "PLAIN", Spawn::default());
    for _ in 0..5 {
        tick::step(&mut w);
    }
    assert!(anims::shown(&w, a).is_none() && anims::shown(&w, b).is_none());
}

#[test]
fn claim8_15_reverse_request_is_kept_at_a_follow_up() {
    // spec: S-0109 Terms and claim 15 (reversed is decided afresh from the follow-up type's flag and the creator's request) and claim 8 (k₀ = λₑ − 1)
    let mut w = game();
    let s = Spawn {
        creation: Creation {
            loop_delay: 0,
            repeat: 1,
            reverse: true,
        },
        ..Default::default()
    };
    let id = make(&mut w, "FIRST", s);
    // Reversed: k₀ = λₑ − 1 = 3, then 2, 1, and k = 0 ends the pass (claim 8).
    assert_eq!(anims::shown(&w, id).map(|s| s.1), Some(3));
    let got = frames(&mut w, id, 4);
    assert_eq!(got, [Some(3), Some(2), Some(1), Some(0)]);
    // The switch to NEXT shows 2a′ = 0 and stays reversed: its next advance (k = −1) ends its pass.
    let a = &w.anims.anims[&id];
    assert_eq!(a.player.name, b"NEXT");
    assert!(
        a.player.play.reverse,
        "the creator's request survives the switch"
    );
}

#[test]
fn state_survives_a_save_round_trip() {
    // spec: S-0109 claim 28 (playback state is simulation state: it is saved and hashed)
    let mut w = game();
    make(&mut w, "LOOPS", Spawn::default());
    make(&mut w, "HURT", Spawn::default());
    for _ in 0..3 {
        tick::step(&mut w);
    }
    let bytes = save::save(&w);
    let mut back = save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("loads");
    assert_eq!(save::state_hash(&back), save::state_hash(&w));
    for _ in 0..6 {
        tick::step(&mut w);
        tick::step(&mut back);
    }
    assert_eq!(save::state_hash(&back), save::state_hash(&w));
}

#[test]
fn claim7_repeat_factor_multiplies_the_passes() {
    // spec: S-0109 claim 7 and F3 (r₀ = max(1, (L × max(m, 1)) mod 256): loop count 1 with repeat factor 3 plays three passes of E × d = 4 ticks)
    let mut r = rules();
    r.sim.animations.get_mut("PLAIN").unwrap().loop_count = Some(1);
    let mut w = testkit::game_with(&r, SIZE, 3);
    let s = Spawn {
        creation: Creation {
            loop_delay: 0,
            repeat: 3,
            reverse: false,
        },
        ..Default::default()
    };
    let id = make(&mut w, "PLAIN", s);
    let got = frames(&mut w, id, 13);
    // Tick c shows frame 0; each pass ends on its fourth advance.
    assert_eq!(got[4], Some(0), "the second pass");
    assert_eq!(got[8], Some(0), "the third pass");
    assert_eq!(got[11], Some(3), "the third pass's last frame");
    assert_eq!(got[12], None, "removed after three passes");
}
