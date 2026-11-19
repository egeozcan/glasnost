//! S-0312 claim 16, *Infantry crash end* (#252): a crashable jumpjet
//! infantry unit shot down in the air is put at ground height when its crash
//! ends, makes the forced request for *air-death-end* (S-0107 claims 6 and
//! 21), and leaves play with no corpse when that sequence completes (S-0107
//! State and ordering). It sets off no death weapon at its crash end.
//! Synthetic types on the jumpjet fixtures of `tests/jump_common`:
//! - JJEND, the fixtures' crashable JJCRASH with an *air-death-end* entry
//!   of 4 frames, exploding, with the fixtures' `Rifle` as its death weapon;
//! - JJNOEND, the same without an *air-death-end* entry (C = 0), so its
//!   request is turned down (Q-3239);
//! - the animation BODYA, the general corpse list.
//!
//! Every name and number is invented; no game data.

mod jump_common;

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{AnimationType, SequenceEntry};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::infantry::Seq;
use glasnost_sim::jump::{self, Phase};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, World};
use jump_common::*;

/// The number of frames of JJEND's *air-death-end* entry.
const END_FRAMES: i32 = 4;

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn game(twice: bool, keeps: bool) -> World {
    let mut r = ruleset();
    r.globals.quirks.crash_death_weapon_twice = twice;
    r.globals.quirks.refused_air_death_end_keeps_unit = keeps;
    register(&mut r, "animations", "BODYA");
    r.sim.animations.insert(
        "BODYA".into(),
        AnimationType {
            frame_count: Some(10),
            ..Default::default()
        },
    );
    r.sim.general.dead_bodies = Some(vec!["BODYA".into()]);
    let mut base = r.sim.infantry["JJCRASH"].clone();
    base.combat.explodes = Some(true);
    base.combat.death_weapon = Some("Rifle".into());
    let none = base.clone();
    let mut end = base;
    end.sequences.insert(
        "air_death_end".into(),
        SequenceEntry {
            start: 0,
            count: END_FRAMES,
            multiplier: 0,
            direction: None,
        },
    );
    register(&mut r, "infantry", "JJEND");
    r.sim.infantry.insert("JJEND".into(), end);
    register(&mut r, "infantry", "JJNOEND");
    r.sim.infantry.insert("JJNOEND".into(), none);
    let mut w = testkit::game_with(&r, SIZE, 7);
    testkit::run(&mut w, 4);
    w
}

fn detonations(w: &World) -> usize {
    w.effects
        .iter()
        .filter(|e| matches!(e, Effect::Detonation { .. }))
        .count()
}

fn corpses(w: &World) -> usize {
    w.effects
        .iter()
        .filter(|e| matches!(e, Effect::Anim { name, .. } if name == "BODYA"))
        .count()
}

/// What [`shot_down`] saw.
#[derive(Debug, Default)]
struct Crash {
    /// Death-weapon detonations at the killing hit.
    at_death: usize,
    /// Death-weapon detonations once the unit has left play.
    total: usize,
    /// The unit's height above the surface on the first tick it was seen
    /// in the crashed state, its current sequence then, and the tick.
    crashed: Option<(i32, Option<Seq>, i32)>,
    /// Whether it was still crashing on the tick before it left play.
    left_while_crashing: bool,
    /// The tick it left play.
    left: i32,
    /// The *air-death-end* stage on the last tick it was in play.
    last_stage: Option<i32>,
    /// Corpse animations made.
    corpses: usize,
}

/// A unit of type `name` alone at (0, 0), ordered to (10, 0) and killed at
/// 50 leptons or more during its ascent; run until it leaves play. `keeps`
/// is `quirks.refused_air_death_end_keeps_unit`.
fn shot_down(twice: bool, keeps: bool, name: &str) -> Crash {
    let mut w = game(twice, keeps);
    let u = put(&mut w, name, Kind::Infantry, 0, 0, 0);
    go(&mut w, u, at(10, 0));
    assert!(until(&mut w, 60, |w| height(w, u) >= 50));
    assert_eq!(phase(&w, u), Phase::Ascending);
    w.effects.clear();
    kill(&mut w, u);
    assert!(jump::crashing(&w, u), "it starts to crash");
    let mut c = Crash {
        at_death: detonations(&w),
        ..Default::default()
    };
    let t0 = w.tick;
    let mut was_crashing = true;
    while w.obj(u).is_some() && w.tick < t0 + 600 {
        was_crashing = jump::phase(&w, u) == Some(Phase::Crashing);
        let o = w.obj(u).unwrap();
        let s = o.infantry.as_ref().unwrap();
        if s.seq == Some(Seq::AirDeathEnd) {
            c.last_stage = Some(s.stage(w.tick));
        }
        if c.crashed.is_none() && jump::phase(&w, u) == Some(Phase::Crashed) {
            c.crashed = Some((height(&w, u), s.seq, w.tick));
        }
        testkit::run(&mut w, 1);
    }
    assert!(w.obj(u).is_none(), "it has left play");
    c.left_while_crashing = was_crashing;
    c.left = w.tick;
    c.total = detonations(&w);
    c.corpses = corpses(&w);
    c
}

fn kill(w: &mut World, id: ObjectId) {
    let health = w.obj(id).unwrap().health;
    damage::apply(
        w,
        id,
        Hit {
            amount: health * 10,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    );
}

// spec: S-0312 claim 16 (Infantry crash end: put at ground height, the forced request for air-death-end, S-0107 claims 6 and 21; when that sequence starts, its completion removes the unit with no corpse) and S-0107 State and ordering (air-death-end complete: unit removed, no corpse)
#[test]
fn issue_252_a_crashed_infantry_unit_plays_air_death_end_and_leaves_with_no_corpse() {
    let c = shot_down(true, true, "JJEND");
    let (h, seq, t) = c.crashed.expect("in play in the crashed state");
    assert_eq!(h, 0, "at ground height: {c:?}");
    assert_eq!(seq, Some(Seq::AirDeathEnd), "{c:?}");
    assert!(!c.left_while_crashing, "{c:?}");
    // It stays until the sequence is complete (S-0107 claim 8: the stage has
    // reached C), and no longer.
    assert!(c.left > t, "{c:?}");
    assert_eq!(c.last_stage, Some(END_FRAMES), "{c:?}");
    assert_eq!(c.corpses, 0, "{c:?}");
}

// spec: S-0312 claim 16 (Infantry crash end: no death weapon at its crash end, so an exploding one fires its death weapon only at death, claim 6) and Edge cases (quirks.crash_death_weapon_twice does not reach an infantry unit's death: it fires at death in both settings)
#[test]
fn issue_252_an_exploding_crashed_infantry_unit_fires_only_at_death() {
    for twice in [true, false] {
        let c = shot_down(twice, true, "JJEND");
        assert_eq!((c.at_death, c.total), (1, 1), "{twice}: {c:?}");
    }
}

// spec: S-0312 Edge cases (quirks.refused_air_death_end_keeps_unit off: when the forced air-death-end request is turned down, S-0107 claim 6 row 1, the unit is removed at its crash end with no corpse) and claim 16 (no death weapon at the crash end either way); the switch's on form is tests/claims_s0312_r91_crash_health.rs
#[test]
fn issue_252_a_turned_down_air_death_end_leaves_play_at_the_crash_end_with_the_switch_off() {
    for twice in [true, false] {
        let c = shot_down(twice, false, "JJNOEND");
        assert!(c.crashed.is_none(), "{twice}: {c:?}");
        assert!(c.left_while_crashing, "{twice}: {c:?}");
        assert_eq!(c.corpses, 0, "{twice}: {c:?}");
        assert_eq!((c.at_death, c.total), (1, 1), "{twice}: {c:?}");
    }
}
