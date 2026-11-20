//! Issue #456: a disguise-when-still vehicle (MIRAGE) and a defence that
//! cannot detect disguises (TOWER, of the other house, human-owned). Guard
//! tests of S-0516 r29 claim 44 (Q-3313 answered), with claims 33 to 35
//! and 39 and S-0309 claim 12's table C disguise row: the defence's search
//! reads claim 35's predicate, so it finds the vehicle on every tick it
//! moves, while it waits to re-disguise and after a damaging hit, and never
//! while it appears disguised, whatever its fake blink; retaliation is
//! tested at the hit with no draw; a held target is kept; the vehicle takes
//! its disguise in the turn that brings it to rest. Synthetic rules
//! (`cloak_common`); every name and number is invented.

mod cloak_common;

use cloak_common::*;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::cloak::{appears_disguised, disguise};
use glasnost_sim::combat::acquire::{self, SearchContext, SearchKind};
use glasnost_sim::occupancy;
use glasnost_sim::rules::Behaviour;
use glasnost_sim::world::{HouseRole, ObjectId, Target, World};

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

// spec: S-0516 r29 claim 44 (b) (a searcher of a player house whose type cannot detect disguises never picks a target that appears disguised to its house); claim 35 (appears disguised to H) and claim 36 (target acquisition uses it); S-0309 claim 12, table C, disguise row (a searcher whose type cannot detect disguises rejects T that appears disguised to its owner)
#[test]
fn issue_456_a_still_disguised_mirage_is_not_picked() {
    let (_, w, m, tower) = scene();
    assert_ne!(picks(&w, tower), Some(m));
}

// spec: S-0516 r29 claim 44 (e) (the vehicle counts as moving for claim 33 (a) while its locomotor's movement report is true) and claim 44 (b); claim 33 (a) and claim 34 (b) (a vehicle whose locomotor is moving loses any disguise on each tick on which it moves) and claim 35; S-0309 claim 12, table C, disguise row (no longer disguised, it is not rejected)
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

// spec: S-0516 r29 claim 44 (d) (each hit clears the disguise and starts the lockout; until the vehicle disguises again every house's searches can pick it); claim 34 (a) (a damaging hit clears the disguise and sets the re-disguise lockout, 2 ticks per point of damage, S-0302 claim 12) and claim 33 (c) (no disguise until the lockout has run out); claim 35 and S-0309 claim 12, table C, disguise row (not disguised, it is picked on every tick of the wait)
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

// spec: S-0516 r29 claim 44 (b) (whether or not the target's fake blink runs); claim 39 (the fake blink set on firing does not remove the disguise; while it runs claim 37 (a) shows others the real unit, and S-0309 claim 25 g applies, which is for searchers of the computer-owner houses only) and claim 37 (presentation) with claim 35 (game state): a human-owned defence's search reads claim 35, not the fake blink or the drawn image
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

// spec: S-0516 r29 claim 44 (e) (in a vehicle's turn claim 33 comes after its movement: in the turn whose movement brings it to rest claim 33 finds it still and it takes its disguise in that same turn, so no object finds it still and undisguised in that tick); claim 33 (a) and (c) and claim 35; S-0309 claim 12, table C, disguise row: in each state between ticks after the stop, the search follows claim 35's predicate
#[test]
fn issue_456_a_just_stopped_mirage_follows_the_predicate() {
    let (_, mut w, m, tower) = scene();
    move_to(&mut w, m, at(-3, 2));
    until(&mut w, 50, |w| occupancy::in_progress(w, m));
    until(&mut w, 300, |w| !occupancy::in_progress(w, m));
    // The first state between ticks with the vehicle still.
    assert!(
        w.obj(m).unwrap().cloak.disguise.is_some(),
        "disguised at the first tick end at which it is still (claim 44 e)"
    );
    for _ in 0..30 {
        expect_by_predicate(&w, m, tower, "after the stop");
        assert!(
            w.obj(m).unwrap().cloak.disguise.is_some(),
            "and at every later tick end (tick {})",
            w.tick
        );
        run(&mut w, 1);
    }
}

// spec: S-0516 r29 claim 44 (e) (a vehicle with the driving locomotor on a track when an order stops it keeps its heading point and finishes the track in its own movement, S-0207 claim 11; in the turn whose movement brings it to rest it takes its disguise); CS23 part d run d2 (a stop order given while it drives)
#[test]
fn claim_44_e_a_stop_order_on_a_track_disguises_in_the_turn_it_comes_to_rest() {
    let (_, mut w, m, _) = scene();
    move_to(&mut w, m, at(-3, 4));
    // The stop order is applied at the end of the tick after its issue (S-0213 claim 1), so it is issued while the vehicle is on a track that the next tick does not finish.
    until(&mut w, 100, |w| on_track(w, m));
    run(&mut w, 1);
    assert!(on_track(&w, m), "void unless it is still on a track");
    stop(&mut w, m);
    run(&mut w, 1);
    assert!(on_track(&w, m), "void unless the order found it on a track");
    assert_eq!(w.obj(m).unwrap().dest, None, "the order cleared it");
    until(&mut w, 300, |w| !occupancy::in_progress(w, m));
    let rest = w.obj(m).unwrap().pos;
    assert!(
        w.obj(m).unwrap().cloak.disguise.is_some(),
        "disguised at the first tick end at which it is still"
    );
    for _ in 0..30 {
        run(&mut w, 1);
        assert_eq!(w.obj(m).unwrap().pos, rest, "it stays at rest");
        assert!(
            w.obj(m).unwrap().cloak.disguise.is_some(),
            "still disguised"
        );
    }
}

/// Whether the vehicle is on a track of the driving locomotor (S-0207
/// Terms: it has a heading point).
fn on_track(w: &World, m: ObjectId) -> bool {
    w.obj(m)
        .and_then(|o| o.drive.as_ref())
        .is_some_and(|d| d.track.is_some())
}

/// A stop order for the vehicle from its owner.
fn stop(w: &mut World, m: ObjectId) {
    let owner = w.obj(m).unwrap().owner.unwrap();
    assert!(glasnost_sim::orders::issue(
        w,
        owner,
        glasnost_sim::orders::Order::Stop { actor: m }
    ));
}

/// A hit of the mirage on the tower: 100 points of the HE warhead with the
/// mirage as attacker, applied now. Returns the tower's health before.
fn mirage_hits_tower(r: &Ruleset, w: &mut World, m: ObjectId, tower: ObjectId) -> i32 {
    let he = r.sim.warheads.keys().position(|k| k == "HE").unwrap() as u32;
    let before = w.obj(tower).unwrap().health;
    glasnost_sim::combat::damage::apply(
        w,
        tower,
        glasnost_sim::combat::damage::Hit {
            amount: 100,
            warhead: Some(he),
            distance: 0,
            attacker: Some(m),
            house: Some(ALPHA),
            ignore_defences: false,
            no_escape: false,
        },
    );
    assert!(
        w.obj(tower).unwrap().health < before,
        "void unless the hit deals damage"
    );
    before
}

/// The scene with the tower in *guard* (a behaviour with the retaliate
/// property, S-0215 claim 18) and no target.
fn guard_scene() -> (Ruleset, World, ObjectId, ObjectId) {
    let (r, mut w, m, tower) = scene();
    let tick = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    w.obj_mut(tower)
        .unwrap()
        .beh
        .force(Behaviour::Guard, tick, stale);
    assert_eq!(w.obj(tower).unwrap().target, None);
    (r, w, m, tower)
}

// spec: S-0516 r29 claim 44 (a) (the disguise condition of retaliation, claim 36 and S-0309 claim 18, is tested when the hit is applied, for the attacker as it is then; the fake blink does not enter it and no draw is taken, whoever owns the victim: a player house, a marked test-only house, or the neutral or special house) and (b) (the hit does not make the attacker acquirable); the control: an undisguised attacker gets the ordinary retaliation of S-0309 claim 18
#[test]
fn claim_44_a_no_retaliation_against_a_disguised_attacker_at_the_hit() {
    for blink in [false, true] {
        for owner in ["human", "test-only", "neutral", "special"] {
            let (r, mut w, m, tower) = guard_scene();
            let beta = &mut w.houses[usize::from(BETA.0)];
            match owner {
                "test-only" => beta.human = false,
                "neutral" => beta.role = HouseRole::Neutral,
                "special" => beta.role = HouseRole::Special,
                _ => {}
            }
            if blink {
                let t = w.tick;
                w.obj_mut(m).unwrap().cloak.blink_until = t + BLINK;
                assert!(disguise::blinking(&w, m), "void unless blinking");
            }
            assert!(appears_disguised(&w, m, Some(BETA)), "void: disguised");
            mirage_hits_tower(&r, &mut w, m, tower);
            assert_eq!(
                w.obj(tower).unwrap().target,
                None,
                "{owner}, blink {blink}: no retaliation"
            );
            if owner == "human" {
                assert_ne!(
                    picks(&w, tower),
                    Some(m),
                    "blink {blink}: the hit does not make it acquirable"
                );
            }
        }
    }
    // The control: the same hit from an undisguised attacker.
    let (r, mut w, m, tower) = guard_scene();
    let t = w.tick;
    let c = &mut w.obj_mut(m).unwrap().cloak;
    c.disguise = None;
    c.lockout_until = t + 100;
    assert!(!appears_disguised(&w, m, Some(BETA)));
    mirage_hits_tower(&r, &mut w, m, tower);
    assert_eq!(
        w.obj(tower).unwrap().target,
        Some(Target::Object(m)),
        "an undisguised attacker is answered"
    );
}

// spec: S-0516 r29 claim 44 (d) (a hit on the vehicle clears its disguise; until it disguises again it appears disguised to no house, and every object it hits can retaliate against it under S-0309 claim 18's other conditions)
#[test]
fn claim_44_d_after_a_hit_on_it_its_hits_are_answered() {
    let (r, mut w, m, tower) = guard_scene();
    let sa = r.sim.warheads.keys().position(|k| k == "SA").unwrap() as u32;
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
    assert_eq!(w.obj(m).unwrap().cloak.disguise, None, "the hit clears it");
    assert!(!appears_disguised(&w, m, Some(ALPHA)));
    assert!(!appears_disguised(&w, m, Some(BETA)));
    mirage_hits_tower(&r, &mut w, m, tower);
    assert_eq!(
        w.obj(tower).unwrap().target,
        Some(Target::Object(m)),
        "the tower answers it"
    );
}

// spec: S-0516 r29 claim 44 (c) (taking a disguise drops no holder's target and whether a target appears disguised is not among the firing check's tests, so a holder that gained the vehicle while it moved keeps it after it disguises again, under S-0309's drop rules, claims 4 and 27 to 32, none of which concerns a disguise); S-0309 claim 29 (the check ticks: multiples of 16)
#[test]
fn claim_44_c_a_held_target_is_kept_after_the_vehicle_disguises_again() {
    // The tower in guard, its rearm countdown set far ahead so that it holds the vehicle without hitting it (CS23 part a's long rate of fire).
    let (_, mut w, m, tower) = guard_scene();
    let t = w.tick;
    w.obj_mut(tower).unwrap().combat.rearm_until = t + 10_000;
    move_to(&mut w, m, at(0, -3));
    // The tower's own search on a tick on which the vehicle moves gives it the vehicle.
    until(&mut w, 100, |w| {
        w.obj(tower).unwrap().target == Some(Target::Object(m))
    });
    assert!(
        !appears_disguised(&w, m, Some(BETA)),
        "taken while undisguised"
    );
    until(&mut w, 300, |w| {
        !occupancy::in_progress(w, m) && w.obj(m).unwrap().cloak.disguise.is_some()
    });
    let from = w.tick;
    let mut checks = 0;
    while w.tick < from + 40 {
        if w.tick % 16 == 0 {
            checks += 1;
        }
        run(&mut w, 1);
        assert!(appears_disguised(&w, m, Some(BETA)), "void: disguised");
        assert_eq!(
            w.obj(tower).unwrap().target,
            Some(Target::Object(m)),
            "kept (tick {})",
            w.tick - 1
        );
    }
    assert!(checks >= 2, "void unless two check ticks passed");
    assert_eq!(
        w.obj(tower).unwrap().combat.rearm_until,
        t + 10_000,
        "void unless it made no shot"
    );
}
