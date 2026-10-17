//! Claim tests of S-0203 claim 4 (revision 32), S-0301 claims 11 and 13
//! (revision 18) and S-0215 claims 2 and 11 (revision 48): the warp update
//! is the whole member turn from the first pending update through arrival
//! effects, with elapsed clocks still running and ordinary eligibility back
//! in the arrival hold after the hold-end test. Engine-only contract tests
//! (S-0513 scenario SW23 part f, S-0301 scenarios F10 and F11), on
//! synthetic rules (`tests/sw_common`).

mod sw_common;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::rules::Behaviour;
use glasnost_sim::superweapons::warp::Stage;
use glasnost_sim::superweapons::{ProtectKind, Tint, protect};
use glasnost_sim::world::{ObjectId, World};
use sw_common::*;

/// The arrival hold's length in these tests.
const DELAY: i32 = 7;

fn rules() -> Ruleset {
    let mut r = ruleset();
    {
        let g = &mut r.sim.general.superweapons;
        g.chrono_placement = Some("MARK".into());
        g.chrono_blast = Some("BLASTS".into());
        g.chrono_blast_dest = Some("BLASTD".into());
        g.chrono_delay = Some(DELAY);
    }
    r.sim.general.warp_out = Some("WARPA".into());
    for n in ["MARK", "BLASTS", "BLASTD", "WARPA"] {
        r.sim
            .animations
            .insert(n.into(), glasnost_rules::sim::AnimationType::default());
        r.registries
            .entry("animations".into())
            .or_default()
            .push(n.into());
    }
    // The hold persists, so that several ordinary turns follow the hold-end test.
    r.globals.quirks.warp_hold_persists = true;
    let cart = r.sim.vehicles.get_mut("CART").unwrap();
    cart.combat.ammo = Some(3);
    cart.combat.reload = Some(20);
    r
}

fn stage(w: &World, id: ObjectId) -> Option<Stage> {
    w.obj(id).and_then(|o| o.sw.warp.as_ref()).map(|m| m.stage)
}

fn warp(w: &mut World, src: CellXY, dst: CellXY) {
    fire_now(w, ALPHA, SOURCE, src);
    fire(w, ALPHA, WARP, dst);
    step(w);
}

/// A game with a CART that will be held for DELAY ticks on arrival.
fn launched() -> (World, ObjectId) {
    let mut w = game(&rules());
    spawn(&mut w, "CHRONOB", ALPHA, -9, -9);
    ready_tick(&mut w, ALPHA, SOURCE, 300);
    let a = spawn(&mut w, "CART", ALPHA, 0, 0);
    w.obj_mut(a).unwrap().sw.held_delay = DELAY;
    (w, a)
}

/// Runs the member's warp, calling `f(world, id, stage_before_the_tick)`
/// after each tick, until the warp is over.
fn each_tick(w: &mut World, a: ObjectId, mut f: impl FnMut(&World, ObjectId, Option<Stage>)) {
    for _ in 0..200 {
        let s = stage(w, a);
        if s.is_none() {
            break;
        }
        step(w);
        f(w, a, s);
    }
}

#[test]
fn reload_countdown_keeps_its_clock_but_grants_nothing_until_the_hold() {
    // spec: S-0301 claim 11 (warp exclusion: no round is granted and no countdown restarted through arrival effects; elapsed time on an existing countdown still passes; in the hold one round on the first eligible turn, then the ordinary restart; no catch-up grants) and F10 warp part; S-0203 claim 4 (elapsed-tick clocks keep running)
    let (mut w, a) = launched();
    warp(&mut w, at(0, 0), at(5, 5));
    assert_eq!(stage(&w, a), Some(Stage::Pending));
    {
        let c = &mut w.obj_mut(a).unwrap().combat;
        c.ammo = 1;
        c.reload_left = Some(10);
    }
    let mut last = w.obj(a).unwrap().combat.reload_left.unwrap();
    assert_eq!(last, 10, "the launch tick is not a warp update");
    let mut first_hold = true;
    let mut hold_ticks = 0;
    let mut expired_before_hold = false;
    each_tick(&mut w, a, |w, a, s| {
        let c = &w.obj(a).unwrap().combat;
        match s {
            Some(Stage::Hold(_)) => {
                hold_ticks += 1;
                if first_hold {
                    first_hold = false;
                    assert_eq!(c.ammo, 2, "one round on the first eligible turn");
                    assert_eq!(c.reload_left, Some(20), "the ordinary restart");
                } else {
                    assert!(c.ammo <= 3 && c.ammo >= 2, "at most one round per turn");
                }
            }
            Some(st) => {
                assert_eq!(c.ammo, 1, "no grant in {st:?}");
                let left = c.reload_left.expect("never restarted or cleared");
                assert_eq!(left, last - 1, "elapsed time passes in {st:?}");
                last = left;
                expired_before_hold |= left <= 0;
            }
            None => unreachable!(),
        }
    });
    assert!(hold_ticks >= 2, "the hold lasts several turns");
    assert!(expired_before_hold, "the countdown expired during the warp");
}

#[test]
fn a_never_started_reload_countdown_first_grants_in_an_eligible_hold_turn() {
    // spec: S-0301 claim 11 and F10 warp part (a never-started countdown counts as over: its first grant likewise waits for an eligible hold turn); S-0203 claim 4
    let (mut w, a) = launched();
    warp(&mut w, at(0, 0), at(5, 5));
    assert_eq!(stage(&w, a), Some(Stage::Pending));
    {
        let c = &mut w.obj_mut(a).unwrap().combat;
        c.ammo = 1;
        c.reload_left = None;
    }
    let mut granted_in_hold = false;
    each_tick(&mut w, a, |w, a, s| {
        let c = &w.obj(a).unwrap().combat;
        match s {
            Some(Stage::Hold(_)) => granted_in_hold |= c.ammo > 1,
            Some(st) => {
                assert_eq!(c.ammo, 1, "no grant in {st:?}");
                assert_eq!(c.reload_left, None, "no restart in {st:?}");
            }
            None => unreachable!(),
        }
    });
    assert!(granted_in_hold, "the hold permits the first grant");
}

#[test]
fn the_firing_frame_counter_is_kept_and_the_rearm_deadline_is_not_postponed() {
    // spec: S-0301 claim 13 and F11 warp part (the counter retains its value through arrival effects; elapsed time on the rearm countdown still passes; in the hold the ordinary advancement resumes without replay); S-0203 claim 4
    let (mut w, a) = launched();
    warp(&mut w, at(0, 0), at(5, 5));
    assert_eq!(stage(&w, a), Some(Stage::Pending));
    {
        let t = w.tick;
        let c = &mut w.obj_mut(a).unwrap().combat;
        c.firing_frame = Some(4);
        c.rearm_until = t + 5;
    }
    let deadline = w.obj(a).unwrap().combat.rearm_until;
    let mut advanced_in_hold = false;
    each_tick(&mut w, a, |w, a, s| {
        let c = &w.obj(a).unwrap().combat;
        assert_eq!(
            c.rearm_until, deadline,
            "the rearm deadline is not postponed"
        );
        match s {
            Some(Stage::Hold(_)) => advanced_in_hold |= c.firing_frame != Some(4),
            Some(st) => assert_eq!(c.firing_frame, Some(4), "kept in {st:?}"),
            None => unreachable!(),
        }
    });
    assert!(advanced_in_hold, "the hold permits the advancement");
}

#[test]
fn a_queued_behaviour_waits_for_the_first_hold_turn() {
    // spec: S-0215 claim 11 and claim 2 (revision 48: the per-tick start takes no effect from the first pending warp update through arrival effects, before and after warp work; in the hold it regains its eligibility after the hold-end test; readiness alone cannot start a queued behaviour on an excluded turn) and S-0203 claim 4
    let (mut w, a) = launched();
    // Control: a stationary vehicle starts its queued behaviour at its next per-tick start.
    let c = spawn(&mut w, "CART", BETA, 8, 8);
    w.obj_mut(c).unwrap().beh.current = Some(Behaviour::Guard);
    w.obj_mut(c).unwrap().beh.queued = Some(Behaviour::Sticky);
    step(&mut w);
    assert_eq!(
        w.obj(c).unwrap().beh.queued,
        None,
        "control: started at once"
    );
    w.obj_mut(a).unwrap().beh.current = Some(Behaviour::Guard);
    warp(&mut w, at(0, 0), at(5, 5));
    w.obj_mut(a).unwrap().beh.queued = Some(Behaviour::Sticky);
    let mut started_in_hold = false;
    each_tick(&mut w, a, |w, a, s| {
        let b = &w.obj(a).unwrap().beh;
        match s {
            Some(Stage::Hold(_)) => started_in_hold |= b.queued.is_none(),
            Some(st) => {
                assert_eq!(b.queued, Some(Behaviour::Sticky), "still queued in {st:?}");
                assert_eq!(b.current, Some(Behaviour::Guard), "no start in {st:?}");
            }
            None => unreachable!(),
        }
    });
    assert!(started_in_hold, "the hold permits the per-tick start");
}

#[test]
fn expected_health_bookkeeping_is_skipped_and_resumes_in_the_hold() {
    // spec: S-0203 claim 4 (revision 32: expected-health turn bookkeeping is suppressed through arrival effects and eligible after the hold-end test, with no catch-up; it does not cancel actual damage) and S-0309 claim 15 (the per-turn change)
    let (mut w, a) = launched();
    warp(&mut w, at(0, 0), at(5, 5));
    assert_eq!(stage(&w, a), Some(Stage::Pending));
    {
        let o = w.obj_mut(a).unwrap();
        o.health = 100;
        o.combat.expected_health = -10;
    }
    let mut moved_in_hold = false;
    each_tick(&mut w, a, |w, a, s| {
        let o = w.obj(a).unwrap();
        match s {
            Some(Stage::Hold(_)) => moved_in_hold |= o.combat.expected_health != -10,
            Some(st) => assert_eq!(o.combat.expected_health, -10, "unchanged in {st:?}"),
            None => unreachable!(),
        }
    });
    assert!(moved_in_hold, "the hold permits the bookkeeping");
}

#[test]
fn the_protection_envelope_waits_for_the_hold_but_its_duration_elapses() {
    // spec: S-0203 claim 4 (revision 32: protection-brightness phase processing is suppressed through arrival effects, elapsed protection duration is not; the hold-end test precedes the phase processing) and S-0513 claim 29 (the first protected turn starts the rise)
    let (mut w, a) = launched();
    warp(&mut w, at(0, 0), at(5, 5));
    assert_eq!(stage(&w, a), Some(Stage::Pending));
    protect::start(&mut w, a, ProtectKind::Curtain, 5000);
    assert_eq!(
        w.obj(a).unwrap().sw.protection.unwrap().tint,
        Tint::BeforeFirstTurn
    );
    let mut rose_in_hold = false;
    each_tick(&mut w, a, |w, a, s| {
        let p = w
            .obj(a)
            .unwrap()
            .sw
            .protection
            .expect("the duration still runs");
        match s {
            Some(Stage::Hold(_)) => rose_in_hold |= p.tint != Tint::BeforeFirstTurn,
            Some(st) => assert_eq!(p.tint, Tint::BeforeFirstTurn, "no phase in {st:?}"),
            None => unreachable!(),
        }
    });
    assert!(rose_in_hold, "the hold permits the phase processing");
}

#[test]
fn berserk_countdown_is_ordinary_work_of_the_warp_turn() {
    // spec: S-0610 claim 5 (warp-only updates: no countdown step from the first pending update through arrival effects, none made up later, the remaining time kept; in arrival hold one step in each update after the hold-end test) and F2; S-0203 claim 4 (revision 34: the berserk countdown is among the suppressed ordinary work); S-0513 claims 39 and 51 (revision 15: the leaving stage, wait-expiry update included, takes no step)
    let (mut w, a) = launched();
    warp(&mut w, at(0, 0), at(5, 5));
    assert_eq!(stage(&w, a), Some(Stage::Pending));
    w.obj_mut(a).unwrap().combat.berserk = Some(1000);
    let mut last = 1000;
    let mut hold_ticks = 0;
    let mut saw_expired = false;
    each_tick(&mut w, a, |w, a, s| {
        let left = w.obj(a).unwrap().combat.berserk.expect("still berserk");
        match s {
            Some(Stage::Hold(_)) => {
                hold_ticks += 1;
                assert_eq!(left, last - 1, "one step per hold update");
            }
            Some(st) => {
                saw_expired |= st == Stage::Expired;
                assert_eq!(left, last, "no step in {st:?}");
            }
            None => unreachable!(),
        }
        last = left;
    });
    assert!(saw_expired, "the wait-expiry update was a warp-only update");
    assert!(hold_ticks >= 2, "the hold lasts several turns");
    assert_eq!(last, 1000 - hold_ticks, "no catch-up");
}
