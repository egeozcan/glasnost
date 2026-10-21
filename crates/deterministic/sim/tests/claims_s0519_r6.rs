//! Claim tests for S-0519 revision 6 (answers Q-1524): the charge flash's
//! count and pattern (claim 25, F6), the air-strike mark's phase sequence
//! and its one scenario-stream draw (claims 21 and 26), and the combined
//! building brightness (F7). Synthetic data only.

#[path = "terror_common/mod.rs"]
mod terror_common;

use glasnost_sim::bomb;
use glasnost_sim::mark::{self, Adjust, Phase};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::strike::{self, Strike};
use glasnost_sim::superweapons::{ProtectKind, Protection, Tint};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};
use terror_common::*;

fn sabotage(w: &mut World, target: ObjectId) -> ObjectId {
    let cell = w.obj(target).unwrap().pos.cell();
    let s = quiet(w, Kind::Infantry, "SABO", 0, cell);
    let (tick, stale) = (w.tick, w.rules().quirks.stale_suspended_revert);
    let o = w.obj_mut(s).unwrap();
    o.beh.force(Behaviour::Sabotage, tick, stale);
    o.dest = Some(Target::Object(target));
    s
}

/// A quiet world with one enemy building, the charge delay absent (*L* = 13).
fn charged() -> (World, ObjectId) {
    let mut w = world(|r| r.sim.general.c4_delay = None);
    let b = quiet(&mut w, Kind::Building, "BLDG", 1, at(0, 0));
    let s = sabotage(&mut w, b);
    assert!(bomb::charge_arrival(&mut w, s));
    (w, b)
}

fn look(w: &World, b: ObjectId) -> mark::Look {
    w.obj(b).unwrap().combat.look
}

// spec: S-0519 claim 25 and F6 (planting sets c to L = 13 with the key absent, F4; each building turn lowers it by 1; on when c mod 4 is 2 or 3: after the 2nd, 3rd, 6th, 7th, 10th and 11th turns)
#[test]
fn claim_25_the_flash_count_and_pattern() {
    let (mut w, b) = charged();
    assert_eq!(bomb::charge_flash(&w), 13);
    assert_eq!(look(&w, b).flash, 13);
    assert!(
        !mark::flash_on(look(&w, b).flash),
        "off right after planting"
    );
    let mut on = Vec::new();
    for turn in 1..=16 {
        testkit::run(&mut w, 1);
        if mark::flash_on(look(&w, b).flash) {
            on.push(turn);
        }
    }
    assert_eq!(on, vec![2, 3, 6, 7, 10, 11]);
    assert_eq!(look(&w, b).flash, 0, "it stops at 0");
}

// spec: S-0519 claim 25 (curtain protection that cancels the countdown does not stop the flash: the count does not depend on the countdown; a later flash replaces the count, counts do not add)
#[test]
fn claim_25_the_flash_outlives_the_countdown_and_is_replaced() {
    let (mut w, b) = charged();
    w.obj_mut(b).unwrap().combat.delay_kill_until = None;
    testkit::run(&mut w, 2);
    assert_eq!(look(&w, b).flash, 11);
    mark::start_flash(&mut w, b, 5);
    assert_eq!(look(&w, b).flash, 5, "replaced, not 16");
}

// spec: S-0519 F6 (+500 ‰ while on and b ≤ 1500, −500 ‰ while on and b > 1500, unchanged while off; no clamp)
#[test]
fn f6_the_flash_step() {
    assert_eq!(mark::flashed(1000, true), 1500);
    assert_eq!(mark::flashed(1500, true), 2000);
    assert_eq!(mark::flashed(1501, true), 1001);
    assert_eq!(mark::flashed(1900, false), 1900);
    assert_eq!(mark::flashed(-200, true), 300, "no clamp");
}

/// A world with one enemy building marked by a hand-made strike of a
/// quiet designator, restarted in the current tick: the mark's draw is the
/// only draw.
fn marked() -> (World, ObjectId, ObjectId) {
    let mut w = world(|_| {});
    let b = quiet(&mut w, Kind::Building, "BLDG", 1, at(0, 0));
    let d = quiet(&mut w, Kind::Vehicle, "DESIG", 0, at(-6, 0));
    w.strike.strikes.insert(
        d,
        Strike {
            target: Some(b),
            progress: true,
            active: true,
            ..Strike::default()
        },
    );
    mark::restart(&mut w, b);
    (w, b, d)
}

// spec: S-0519 claims 21 and 26 (the restart puts the sequence before the first marked turn q; Initial rise at q, Bright hold at q + 6, and exactly one inclusive ranged draw from −5 through 5 on the scenario stream in turn q + 10, giving Initial dimming 20 + the draw; no other draw)
#[test]
fn claim_26_the_phases_and_the_one_draw() {
    let (mut w, b, _) = marked();
    assert_eq!(look(&w, b).phase, Phase::Before);
    testkit::run(&mut w, 1);
    let q = w.tick - 1;
    assert_eq!(look(&w, b).phase, Phase::Rise(q));
    let before = w.rng.clone();
    for _ in 1..10 {
        testkit::run(&mut w, 1);
    }
    assert_eq!(look(&w, b).phase, Phase::BrightHold(q + 6));
    assert_eq!(w.rng, before, "no draw before turn q + 10");
    let mut expect = w.rng.clone();
    let d = expect.ranged(-5, 5, w.rules().quirks.ranged_draw_half_span);
    testkit::run(&mut w, 1);
    assert_eq!(
        look(&w, b).phase,
        Phase::Dimming {
            since: q + 10,
            length: 20 + d
        }
    );
    assert_eq!(w.rng, expect, "exactly one draw in turn q + 10");
    let after = w.rng.clone();
    for _ in 0..(20 + d) {
        testkit::run(&mut w, 1);
    }
    assert_eq!(look(&w, b).phase, Phase::PulseRise(q + 30 + d));
    for _ in 0..64 {
        testkit::run(&mut w, 1);
    }
    assert_eq!(look(&w, b).phase, Phase::PulseFall(q + 94 + d));
    for _ in 0..64 {
        testkit::run(&mut w, 1);
    }
    assert_eq!(
        look(&w, b).phase,
        Phase::PulseRise(q + 158 + d),
        "a 128-tick pulse while the mark clock is far from its end"
    );
    assert_eq!(w.rng, after, "no further draw");
}

// spec: S-0519 claim 26 (a mark that ends before turn q + 10 makes no draw; any turn while not marked returns the sequence to before the first marked turn, without a draw)
#[test]
fn claim_26_unmarked_early_makes_no_draw() {
    let (mut w, b, d) = marked();
    testkit::run(&mut w, 5);
    assert!(matches!(look(&w, b).phase, Phase::Rise(_)));
    w.strike.strikes.get_mut(&d).unwrap().target = None;
    let before = w.rng.clone();
    testkit::run(&mut w, 20);
    assert_eq!(look(&w, b).phase, Phase::Before);
    assert_eq!(w.rng, before);
}

// spec: S-0519 claim 26 (the near-expiry phases: Pulse fall goes to Low hold when M < 158, Low hold lasts until a later turn finds M < 31, then Final rise 6, Final bright hold 4, Return 20 and Ordinary)
#[test]
fn claim_26_the_mark_clock_phases() {
    let (mut w, b, _) = marked();
    let t = w.tick;
    // A mark clock started long ago: 157 ticks left at the next turn.
    let o = w.obj_mut(b).unwrap();
    o.combat.look.clock_start = t + 157 - mark::MARK_CLOCK;
    o.combat.look.phase = Phase::PulseFall(t - 64);
    testkit::run(&mut w, 1);
    assert_eq!(look(&w, b).phase, Phase::LowHold(t));
    let mut seen = Vec::new();
    while look(&w, b).phase != Phase::Ordinary && w.tick < t + 400 {
        testkit::run(&mut w, 1);
        let p = look(&w, b).phase;
        if seen.last() != Some(&p) {
            seen.push(p);
        }
    }
    // M < 31 is first found in the turn of tick t + 127 (M = 30).
    assert_eq!(
        seen,
        vec![
            Phase::LowHold(t),
            Phase::FinalRise(t + 127),
            Phase::FinalHold(t + 133),
            Phase::Return(t + 137),
            Phase::Ordinary,
        ]
    );
    assert_eq!(w.tick, t + 158);
}

// spec: S-0519 claim 26 (the factor table: rise trunc((12 − r) × 256 / 6), holds 512, dimming and return trunc((r + 20) × 256 / 20), pulse rise 4 × (128 − r), pulse fall 4 × (r + 64), low hold 256; none before the first marked turn and in Ordinary)
#[test]
fn claim_26_the_factors() {
    assert_eq!(mark::factor(Phase::Before, 5), None);
    assert_eq!(mark::factor(Phase::Ordinary, 5), None);
    assert_eq!(mark::factor(Phase::Rise(10), 10), Some(256));
    assert_eq!(mark::factor(Phase::Rise(10), 13), Some(384));
    assert_eq!(mark::factor(Phase::Rise(10), 16), Some(512));
    assert_eq!(mark::factor(Phase::BrightHold(0), 3), Some(512));
    let dim = |t| {
        mark::factor(
            Phase::Dimming {
                since: 0,
                length: 25,
            },
            t,
        )
    };
    assert_eq!(dim(0), Some(576));
    assert_eq!(dim(25), Some(256));
    let dim15 = mark::factor(
        Phase::Dimming {
            since: 0,
            length: 15,
        },
        0,
    );
    assert_eq!(dim15, Some(448));
    assert_eq!(mark::factor(Phase::PulseRise(0), 0), Some(256));
    assert_eq!(mark::factor(Phase::PulseRise(0), 64), Some(512));
    assert_eq!(mark::factor(Phase::PulseFall(0), 0), Some(512));
    assert_eq!(mark::factor(Phase::PulseFall(0), 64), Some(256));
    assert_eq!(mark::factor(Phase::LowHold(0), 9), Some(256));
    assert_eq!(mark::factor(Phase::FinalRise(0), 0), Some(256));
    assert_eq!(mark::factor(Phase::FinalHold(0), 0), Some(512));
    assert_eq!(mark::factor(Phase::Return(0), 0), Some(512));
    assert_eq!(mark::factor(Phase::Return(0), 20), Some(256));
}

// spec: S-0519 claims 17, 19 and 26 (a start and a redirect restart the marked building's sequence and its mark clock; a mark that passes to another strike holding the same building restarts neither)
#[test]
fn claim_26_restart_by_start_and_redirect_not_by_a_shared_mark() {
    let mut w = world(|_| {});
    let a = quiet(&mut w, Kind::Building, "BLDG", 1, at(0, 0));
    let b = quiet(&mut w, Kind::Building, "BLDG", 1, at(4, 4));
    let d = quiet(&mut w, Kind::Vehicle, "DESIG", 0, at(-6, 0));
    let d2 = quiet(&mut w, Kind::Vehicle, "DESIG", 0, at(-6, 2));
    testkit::run(&mut w, 2);
    let t = w.tick;
    designate(&mut w, d, a);
    let started = look(&w, a).clock_start;
    assert_eq!(started, t, "a start marking it starts the mark clock");
    testkit::run(&mut w, 3);
    designate(&mut w, d2, a);
    let again = look(&w, a).clock_start;
    assert!(again > started, "a second start marking it restarts");
    testkit::run(&mut w, 3);
    let p = look(&w, a).phase;
    assert!(matches!(p, Phase::Rise(_) | Phase::BrightHold(_)));
    // Cancel the second strike: the first still holds the building.
    w.obj_mut(d2).unwrap().target = None;
    testkit::run(&mut w, 1);
    assert!(strike::marked(&w, a));
    assert_eq!(look(&w, a).clock_start, again, "no restart");
    assert_ne!(look(&w, a).phase, Phase::Before);
    // A redirect of the first strike to b marks b afresh.
    designate(&mut w, d, b);
    assert_eq!(w.strike.strikes[&d].target, Some(b));
    assert!(look(&w, b).clock_start > again);
}

// spec: S-0519 State and ordering and claim 26, S-0513 claim 29 (within one building turn the protection sequence steps first, so its draw precedes the mark's)
#[test]
fn claim_26_the_protection_draw_comes_first() {
    let (mut w, b, _) = marked();
    let t = w.tick;
    let o = w.obj_mut(b).unwrap();
    o.sw.protection = Some(Protection {
        start: t - 10,
        duration: 1000,
        kind: ProtectKind::ForceShield,
        tint: Tint::BrightHold(t - 4),
    });
    o.combat.look.phase = Phase::BrightHold(t - 4);
    let mut expect = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    let first = expect.ranged(-5, 5, half);
    let second = expect.ranged(-5, 5, half);
    testkit::run(&mut w, 1);
    let o = w.obj(b).unwrap();
    assert_eq!(
        o.sw.protection.unwrap().tint,
        Tint::Dimming {
            since: t,
            length: 20 + first
        }
    );
    assert_eq!(
        o.combat.look.phase,
        Phase::Dimming {
            since: t,
            length: 20 + second
        }
    );
}

// spec: S-0519 F7 (b₁ = F6; b₂ = P(b₁) of S-0513 F9; marked with a factor: min(2000, ⌊m × b₂ / 256⌋); marked without a factor or unmarked but protected: b₂; unmarked and unprotected: b₁; the flash itself has no clamp)
#[test]
fn f7_the_combined_brightness() {
    let a = |flash, protection, mark| Adjust {
        flash,
        protection,
        mark,
    };
    // Unmarked and unprotected: the flash only, no clamp.
    assert_eq!(a(3, None, None).brightness(1800), 1300);
    assert_eq!(a(2, None, None).brightness(1500), 2000);
    assert_eq!(a(1, None, None).brightness(1500), 1500, "off");
    // Protected but unmarked: b₂ (the envelope's own cap applies in its factor phases).
    assert_eq!(a(0, Some(Some(512)), None).brightness(1200), 2000);
    assert_eq!(a(0, Some(None), None).brightness(1200), 1200);
    // Marked in a factor phase: the mark acts on the protected, flashed value.
    assert_eq!(a(3, Some(Some(384)), Some(Some(384))).brightness(500), 2000);
    assert_eq!(a(3, Some(Some(256)), Some(Some(384))).brightness(500), 1500);
    assert_eq!(a(0, None, Some(Some(512))).brightness(900), 1800);
    // Marked in a phase without a factor: b₂, not newly capped.
    assert_eq!(a(2, None, Some(None)).brightness(1700), 1200);
    assert_eq!(a(2, None, Some(None)).brightness(1500), 2000);
    // A negative b₂ rounds toward minus infinity.
    assert_eq!(a(0, None, Some(Some(300))).brightness(-1), -2);
}

// spec: S-0519 Determinism (the flash count and the mark sequence survive a save and load, and the loaded game goes on alike)
#[test]
fn flash_and_mark_survive_a_save_and_load() {
    let (mut w, b, _) = marked();
    mark::start_flash(&mut w, b, 9);
    testkit::run(&mut w, 7);
    let bytes = glasnost_sim::save::save(&w);
    let h = glasnost_sim::save::state_hash(&w);
    let mut w2 =
        glasnost_sim::save::load(&bytes, w.rules_arc(), w.catalogue.clone().unwrap()).unwrap();
    assert_eq!(glasnost_sim::save::state_hash(&w2), h);
    assert_eq!(look(&w2, b), look(&w, b));
    testkit::run(&mut w, 30);
    testkit::run(&mut w2, 30);
    assert_eq!(
        glasnost_sim::save::state_hash(&w2),
        glasnost_sim::save::state_hash(&w)
    );
}
