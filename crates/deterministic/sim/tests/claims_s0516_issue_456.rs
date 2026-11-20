//! Issue #456: a disguise-when-still vehicle (MIRAGE) and a defence that
//! cannot detect disguises (TOWER, of the other house, human-owned). Guard
//! tests of S-0516 claims 33 to 35 and 39 with S-0309 claim 12's table C
//! disguise row: the defence's search reads claim 35's predicate, so it
//! finds the vehicle on every tick it moves, while it waits to re-disguise
//! and after a damaging hit, and never while it appears disguised, whatever
//! its fake blink. Synthetic rules (`cloak_common`); every name and number
//! is invented.

mod cloak_common;

use cloak_common::*;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::cloak::{appears_disguised, disguise};
use glasnost_sim::combat::acquire::{self, SearchContext, SearchKind};
use glasnost_sim::occupancy;
use glasnost_sim::rules::Behaviour;
use glasnost_sim::world::{ObjectId, World};

/// A game with a still, disguised MIRAGE of Alpha at (−3, 0) and a TOWER of
/// Beta at (3, 0), six cells off, within its weapon's range. The tower is
/// held in *sleep*, so only the searches the tests make act.
fn scene() -> (Ruleset, World, ObjectId, ObjectId) {
    let r = ruleset();
    let mut w = game(&r);
    let m = spawn(&mut w, "MIRAGE", ALPHA, -3, 0);
    let tower = spawn(&mut w, "TOWER", BETA, 3, 0);
    let tick = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    w.obj_mut(tower)
        .unwrap()
        .beh
        .force(Behaviour::Sleep, tick, stale);
    until(&mut w, 100, |w| w.obj(m).unwrap().cloak.disguise.is_some());
    assert!(
        acquire::player_owned(&w, tower),
        "void unless the searcher is human-owned (claim 25 g is not in play)"
    );
    assert!(
        appears_disguised(&w, m, Some(BETA)),
        "void unless it appears disguised"
    );
    (r, w, m, tower)
}

/// What the tower's range search picks now, run on a copy with its target
/// cleared (the search restarts the timer and takes draws).
fn picks(w: &World, tower: ObjectId) -> Option<ObjectId> {
    let mut probe = w.clone();
    probe.obj_mut(tower).unwrap().target = None;
    acquire::search(&mut probe, tower, SearchKind::Range, SearchContext::PerTick)
}

/// Asserts that the pick follows claim 35's predicate for Beta: the mirage
/// is picked exactly when it does not appear disguised to Beta.
fn expect_by_predicate(w: &World, m: ObjectId, tower: ObjectId, when: &str) {
    let hidden = appears_disguised(w, m, Some(BETA));
    assert_eq!(
        picks(w, tower) == Some(m),
        !hidden,
        "{when} (tick {}): picked exactly when it does not appear disguised",
        w.tick
    );
}

// spec: S-0516 claim 35 (appears disguised to H) and claim 36 (target acquisition uses it); S-0309 claim 12, table C, disguise row (a searcher whose type cannot detect disguises rejects T that appears disguised to its owner)
#[test]
fn issue_456_a_still_disguised_mirage_is_not_picked() {
    let (_, w, m, tower) = scene();
    assert_ne!(picks(&w, tower), Some(m));
}

// spec: S-0516 claim 33 (a) and claim 34 (b) (a vehicle whose locomotor is moving loses any disguise on each tick on which it moves) and claim 35; S-0309 claim 12, table C, disguise row (no longer disguised, it is not rejected)
#[test]
fn issue_456_a_moving_mirage_is_undisguised_and_picked_on_every_moving_tick() {
    let (_, mut w, m, tower) = scene();
    // A path that stays within the tower's range (7.25 cells): every point between (−3, 0) and (0, −3) is under 6.1 cells from (3, 0).
    move_to(&mut w, m, at(0, -3));
    until(&mut w, 50, |w| occupancy::in_progress(w, m));
    let mut moving_ticks = 0;
    for _ in 0..300 {
        run(&mut w, 1);
        if occupancy::in_progress(&w, m) {
            moving_ticks += 1;
            assert_eq!(
                w.obj(m).unwrap().cloak.disguise,
                None,
                "no disguise on a moving tick (tick {})",
                w.tick
            );
            assert_eq!(
                picks(&w, tower),
                Some(m),
                "picked while moving (tick {})",
                w.tick
            );
        }
        expect_by_predicate(&w, m, tower, "on the way");
        if !occupancy::in_progress(&w, m) && w.obj(m).unwrap().cloak.disguise.is_some() {
            break;
        }
    }
    assert!(moving_ticks > 5, "void unless it moved for several ticks");
    assert!(
        w.obj(m).unwrap().cloak.disguise.is_some(),
        "it disguises again once still (claim 33 c)"
    );
    assert_ne!(picks(&w, tower), Some(m), "and is no longer picked");
}

// spec: S-0516 claim 34 (a) (a damaging hit clears the disguise and sets the re-disguise lockout, 2 ticks per point of damage, S-0302 claim 12) and claim 33 (c) (no disguise until the lockout has run out); claim 35 and S-0309 claim 12, table C, disguise row (not disguised, it is picked on every tick of the wait)
#[test]
fn issue_456_after_a_damaging_hit_it_is_picked_until_it_disguises_again() {
    let (r, mut w, m, tower) = scene();
    let sa = r.sim.warheads.keys().position(|k| k == "SA").unwrap() as u32;
    let t = w.tick;
    glasnost_sim::combat::damage::apply(
        &mut w,
        m,
        glasnost_sim::combat::damage::Hit {
            amount: 10,
            warhead: Some(sa),
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: false,
            no_escape: false,
        },
    );
    assert!(w.obj(m).unwrap().health < 200, "a damaging hit");
    assert_eq!(w.obj(m).unwrap().cloak.disguise, None);
    assert_eq!(w.obj(m).unwrap().cloak.lockout_until, t + 20);
    assert_eq!(picks(&w, tower), Some(m), "picked at once after the hit");
    let mut waited = 0;
    while w.obj(m).unwrap().cloak.disguise.is_none() {
        assert!(!occupancy::in_progress(&w, m), "it stays still");
        assert_eq!(
            picks(&w, tower),
            Some(m),
            "picked in the lockout (tick {})",
            w.tick
        );
        run(&mut w, 1);
        waited += 1;
        assert!(waited < 60, "it disguises again");
    }
    assert!(waited >= 19, "the lockout holds");
    expect_by_predicate(&w, m, tower, "disguised again");
    assert_ne!(picks(&w, tower), Some(m));
}

// spec: S-0516 claim 39 (the fake blink set on firing does not remove the disguise; while it runs claim 37 (a) shows others the real unit, and S-0309 claim 25 g applies, which is for searchers of the computer-owner houses only) and claim 37 (presentation) with claim 35 (game state): a human-owned defence's search reads claim 35, not the fake blink or the drawn image
#[test]
fn issue_456_the_fake_blink_does_not_let_a_human_defence_pick_it() {
    let (_, mut w, m, tower) = scene();
    let t = w.tick;
    w.obj_mut(m).unwrap().cloak.blink_until = t + BLINK;
    assert!(disguise::blinking(&w, m), "void unless blinking");
    assert!(
        appears_disguised(&w, m, Some(BETA)),
        "still appears disguised"
    );
    assert_ne!(picks(&w, tower), Some(m));
}

// spec: S-0516 claim 33 (a) and (c) (moving clears the disguise; it takes one again on a still tick with no contact and its lockout run out) and claim 35; S-0309 claim 12, table C, disguise row: in each state between ticks after the stop, the search follows claim 35's predicate (whether a still, undisguised state is seen between ticks depends on the tick pipeline: the re-disguise is taken in the vehicle's own turn of its first still tick)
// The test prints its observation for Q-3313 (a test harness line, not
// simulation output).
#[test]
#[allow(clippy::disallowed_macros)]
fn issue_456_a_just_stopped_mirage_follows_the_predicate() {
    let (_, mut w, m, tower) = scene();
    move_to(&mut w, m, at(-3, 2));
    until(&mut w, 50, |w| occupancy::in_progress(w, m));
    until(&mut w, 300, |w| !occupancy::in_progress(w, m));
    // The first state between ticks with the vehicle still.
    let first_still_disguised = w.obj(m).unwrap().cloak.disguise.is_some();
    for _ in 0..30 {
        expect_by_predicate(&w, m, tower, "after the stop");
        run(&mut w, 1);
    }
    assert!(
        w.obj(m).unwrap().cloak.disguise.is_some(),
        "it disguises again"
    );
    // Printed for the request (Q-3313): whether it was already disguised in the first still state between ticks.
    eprintln!("first still state between ticks disguised: {first_still_disguised}");
}
