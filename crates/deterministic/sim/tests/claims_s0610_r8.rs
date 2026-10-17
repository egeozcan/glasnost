//! Claim tests for S-0610 revision 8 (claim 5: which updates are countdown
//! updates and what the expiry update does; F2; claim 6's healing bullet)
//! and S-0215 revision 57 (claim 11: the late start of infantry and
//! vehicles; scenario B46), on synthetic data: the chrono-weapon fixture
//! `tests/temporal_common` with a few more invented types. The warp rows
//! are in `claims_s0203_warp_turn.rs`, which has the warp fixture.

#[path = "temporal_common/mod.rs"]
mod temporal_common;

use std::collections::BTreeSet;

use glasnost_rules::schema::Ruleset;
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::occupancy;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::psychic;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::transport;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use temporal_common::*;

/// A player house (human-controlled in our engine: guard at expiry).
const BETA: u8 = 1;
/// The neutral house of `testkit::game_with_non_players`.
const NEUTRAL: u8 = 2;

/// The chrono fixture with BIG (a vehicle of strength 100 000, so that an
/// erasure lasts) and TANK (BIG with the fixture's gun).
fn rules() -> Ruleset {
    let mut r = ruleset();
    vehicle(&mut r, "BIG", |t| t.strength = Some(100_000));
    vehicle(&mut r, "TANK", |t| {
        t.strength = Some(100_000);
        t.primary = Some("Gun".into());
    });
    r
}

fn left(w: &World, id: ObjectId) -> Option<i32> {
    w.obj(id).unwrap().combat.berserk
}

fn set_berserk(w: &mut World, id: ObjectId, t: i32) {
    w.obj_mut(id).unwrap().combat.berserk = Some(t);
}

/// A unit of BETA ordered to move far east, run until it moves, then put
/// under *hunt* with nothing queued and its processing timer far off, so
/// that only the per-tick starts change its behaviour.
fn moving_under_hunt(w: &mut World, kind: Kind, name: &str) -> ObjectId {
    let id = spawn(w, kind, name, BETA, at(-10, 0));
    // Orders are refused in the first ticks (S-0213 claim 1).
    while w.tick < 4 {
        testkit::run(w, 1);
    }
    let o = Order::Behaviour {
        actor: id,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(at(12, 0))),
    };
    assert!(orders::issue(w, HouseId(BETA), o));
    for _ in 0..40 {
        testkit::run(w, 1);
        if occupancy::in_progress(w, id) {
            break;
        }
    }
    assert!(occupancy::in_progress(w, id), "moving");
    let t = w.tick;
    let ob = w.obj_mut(id).unwrap();
    ob.beh.current = Some(Behaviour::Hunt);
    ob.beh.queued = None;
    ob.beh.expired = false;
    ob.beh.last = Some((t, 100_000));
    id
}

// ---------------------------------------------------------------------------
// Claim 5: the effects in the expiry update; S-0215 claim 11's late start.

// spec: S-0610 claim 5 (Effects: the queued guard is not in charge in the expiry update; no processing of it in that update; the hunt move continues through it)
// spec: S-0215 claim 11 and scenario B46 (an infantry unit moving and ready under hunt: the late start makes the queued guard current at the end of the expiry update, nothing queued, first processed on the next update, claim 3b)
#[test]
fn b46_infantry_expiry_late_start() {
    let mut w = game(&rules());
    let u = moving_under_hunt(&mut w, Kind::Infantry, "BYSTANDER");
    set_berserk(&mut w, u, 1);
    testkit::run(&mut w, 1);
    let o = w.obj(u).unwrap();
    assert_eq!(o.combat.berserk, None, "expired");
    assert_eq!(o.beh.current, Some(Behaviour::Guard), "late start");
    assert_eq!(o.beh.queued, None);
    assert!(o.beh.expired, "made current, not processed in the expiry update");
    assert!(o.dest.is_some(), "the hunt move is not ended by a guard processing");
    assert!(occupancy::in_progress(&w, u), "still moving at the end of the expiry update");
}

// spec: S-0610 claim 5 (Effects: a vehicle moving under hunt is not ready, so it keeps hunting with guard queued until it stops; scenario P9 b)
// spec: S-0215 claim 11 (a vehicle moving under hunt is not ready to change: no late start)
#[test]
fn p9_b_vehicle_keeps_hunting_while_moving() {
    let mut w = game(&rules());
    let v = moving_under_hunt(&mut w, Kind::Vehicle, "BIG");
    set_berserk(&mut w, v, 1);
    testkit::run(&mut w, 1);
    let o = w.obj(v).unwrap();
    assert_eq!(o.combat.berserk, None);
    assert_eq!(o.beh.current, Some(Behaviour::Hunt));
    assert_eq!(o.beh.queued, Some(Behaviour::Guard));
    assert!(occupancy::in_progress(&w, v));
}

// spec: S-0215 claim 11 (the late start judges readiness with this update's movement counted: a vehicle that stops in its update is ready at the late start, and the queued behaviour is current at the end of that update without processing)
#[test]
fn claim_11_late_start_after_a_vehicle_stops() {
    let mut w = game(&rules());
    let v = moving_under_hunt(&mut w, Kind::Vehicle, "BIG");
    w.obj_mut(v).unwrap().beh.queued = Some(Behaviour::Guard);
    let mut started = None;
    for _ in 0..400 {
        let was_moving = occupancy::in_progress(&w, v);
        testkit::run(&mut w, 1);
        let o = w.obj(v).unwrap();
        if o.beh.current == Some(Behaviour::Guard) {
            started = Some((was_moving, occupancy::in_progress(&w, v), o.beh.expired));
            break;
        }
        assert_eq!(o.beh.current, Some(Behaviour::Hunt), "not ready while moving");
    }
    let (was_moving, moving, expired) = started.expect("stopped and started guard");
    assert!(was_moving && !moving, "the update in which it stopped");
    assert!(expired, "made current at the late start, not processed");
}

// spec: S-0610 claim 5 (A queued behaviour that never started: a unit never ready after the hit still has hunt queued; expiry replaces it by guard and the unit never hunts)
#[test]
fn claim_5_a_queued_hunt_that_never_started_is_replaced() {
    let mut w = game(&rules());
    let v = moving_under_hunt(&mut w, Kind::Vehicle, "BIG");
    // Moving under move: not ready (S-0215 claim 11).
    w.obj_mut(v).unwrap().beh.current = Some(Behaviour::Move);
    assert!(psychic::enter(&mut w, v, 3));
    assert_eq!(w.obj(v).unwrap().beh.queued, Some(Behaviour::Hunt));
    for _ in 0..3 {
        testkit::run(&mut w, 1);
        assert_ne!(w.obj(v).unwrap().beh.current, Some(Behaviour::Hunt));
    }
    let o = w.obj(v).unwrap();
    assert_eq!(o.combat.berserk, None);
    assert_eq!(o.beh.queued, Some(Behaviour::Guard));
}

// spec: S-0610 claim 5 (where expiry queues the behaviour that is already current and nothing else is queued, a neutral-house unit under hunt, nothing changes)
#[test]
fn claim_5_neutral_unit_under_hunt_is_unchanged() {
    let mut w = testkit::game_with_non_players(&rules(), SIZE, 1);
    let v = spawn(&mut w, Kind::Vehicle, "BIG", NEUTRAL, at(0, 0));
    let ob = w.obj_mut(v).unwrap();
    ob.beh.current = Some(Behaviour::Hunt);
    ob.beh.queued = None;
    set_berserk(&mut w, v, 1);
    psychic::countdown_turn(&mut w, v);
    let o = w.obj(v).unwrap();
    assert_eq!(o.combat.berserk, None);
    assert_eq!((o.beh.current, o.beh.queued), (Some(Behaviour::Hunt), None));
}

/// TANK of BETA attacking BIG of its own house in range; with
/// remaining berserk time `berserk` set before the first run (the attack
/// on the own-house unit stands for the berserk target; the expiry drops it). Returns the runs (1-based)
/// on which the tank fired a new shot, over `n` runs.
fn shots(berserk: i32, n: i32) -> Vec<i32> {
    let mut w = game(&rules());
    let tank = spawn(&mut w, Kind::Vehicle, "TANK", BETA, at(0, 0));
    let own = spawn(&mut w, Kind::Vehicle, "BIG", BETA, at(3, 0));
    let t = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    let o = w.obj_mut(tank).unwrap();
    o.target = Some(Target::Object(own));
    o.beh.force(Behaviour::Attack, t, stale);
    o.combat.berserk = Some(berserk);
    let mut seen: BTreeSet<ObjectId> = BTreeSet::new();
    let mut out = Vec::new();
    for run in 1..=n {
        testkit::run(&mut w, 1);
        let new: Vec<ObjectId> = w
            .projectiles
            .keys()
            .copied()
            .filter(|p| !seen.contains(p))
            .collect();
        if !new.is_empty() {
            out.push(run);
        }
        seen.extend(new);
    }
    out
}

// spec: S-0610 claim 5 (Effects: no berserk shot in the expiry update; the last berserk shot is in an earlier update) and F2 (a berserk shot in a countdown update is allowed only when t > 0 after its step); scenario P9 a
#[test]
fn p9_a_no_berserk_shot_in_the_expiry_update() {
    let always = shots(100_000, 60);
    let first = *always.first().expect("a berserk tank fires at its own house's unit");
    // Berserk ends on the run of the first shot: no shot then or later.
    assert_eq!(shots(first, 60), Vec::<i32>::new());
    // One more update of berserk: that shot is fired (t = 1 after its step).
    assert_eq!(shots(first + 1, 60).first(), Some(&first));
}

// ---------------------------------------------------------------------------
// Claim 5: which updates are countdown updates.

// spec: S-0610 claim 5 (excluded updates: a unit being erased takes no countdown step; its expiry is later than an unexcluded twin's by the erased updates) and S-0215 claim 2 (revision 57: on an erasing tick the berserk countdown takes no effect); scenario P9 d
#[test]
fn p9_d_no_countdown_while_being_erased() {
    let mut w = game(&rules());
    let a = attacker(&mut w, "ERASER", 0, at(-12, -12));
    let v = spawn(&mut w, Kind::Vehicle, "BIG", BETA, at(0, 0));
    let twin = spawn(&mut w, Kind::Vehicle, "BIG", BETA, at(6, 6));
    erase(&mut w, a, v);
    assert!(erased(&w, v));
    set_berserk(&mut w, v, 5);
    set_berserk(&mut w, twin, 5);
    for _ in 0..5 {
        testkit::run(&mut w, 1);
        assert!(erased(&w, v), "still being erased");
    }
    assert_eq!(left(&w, twin), None, "the twin expired");
    assert_eq!(left(&w, v), Some(5), "no step on an erasing update");
}

// spec: S-0610 claim 5 (out of play: a passenger of a transport that is not open-topped takes no updates and keeps its remaining time; a passenger of an open-topped transport stays in play and counts down); scenario P9 f
#[test]
fn claim_5_out_of_play_keeps_the_time_open_topped_counts() {
    let mut w = game(&rules());
    let closed = spawn(&mut w, Kind::Vehicle, "GUNNER", BETA, at(0, 0));
    let p = spawn(&mut w, Kind::Infantry, "BYSTANDER", BETA, at(0, 1));
    transport::board(&mut w, p, closed);
    let open = spawn(&mut w, Kind::Vehicle, "OPEN", BETA, at(6, 0));
    let q = spawn(&mut w, Kind::Infantry, "BYSTANDER", BETA, at(6, 1));
    transport::board(&mut w, q, open);
    assert!(transport::open_topped_passenger(&w, q));
    set_berserk(&mut w, p, 100);
    set_berserk(&mut w, q, 100);
    testkit::run(&mut w, 5);
    assert_eq!(left(&w, p), Some(100), "kept inside");
    assert_eq!(left(&w, q), Some(95), "in play: counts down");
}

// ---------------------------------------------------------------------------
// Claim 6: healing is unaffected.

// spec: S-0610 claim 6 (healing is unaffected: a healing hit from a warhead that is not berserk-causing heals a berserk unit and leaves the remaining time unchanged)
#[test]
fn claim_6_healing_hit_keeps_the_time() {
    let mut w = game(&rules());
    let v = spawn(&mut w, Kind::Vehicle, "BIG", BETA, at(0, 0));
    w.obj_mut(v).unwrap().health = 50_000;
    set_berserk(&mut w, v, 40);
    let he = w
        .rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == "HE")
        .unwrap() as u32;
    let hit = Hit {
        amount: -100,
        warhead: Some(he),
        distance: 0,
        attacker: None,
        house: Some(HouseId(0)),
        ignore_defences: false,
        no_escape: false,
    };
    damage::apply(&mut w, v, hit);
    assert!(health(&w, v) > 50_000, "healed");
    assert_eq!(left(&w, v), Some(40));
}
