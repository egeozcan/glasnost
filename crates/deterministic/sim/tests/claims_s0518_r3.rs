//! Claim tests for S-0518 revision 3 (answers Q-0985): the acquisition order
//! of equally distant supporters (claim 3), the start of charging with its
//! delay (claim 14) and the break-off search (claim 15). Synthetic data only.

#[path = "linked_common/mod.rs"]
mod linked_common;

use glasnost_sim::linked;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::ownership::{self, Cause};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use linked_common::*;

fn quiet(w: &mut World, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    let b = spawn(w, Kind::Building, name, house, at(dx, dy));
    let tick = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    if let Some(o) = w.obj_mut(b) {
        o.beh.force(Behaviour::Sleep, tick, stale);
    }
    b
}

/// A master at the origin and a victim to fire at; the supporter that has
/// a pending beam once the master has recruited one.
fn recruited(w: &mut World, m: ObjectId, cands: &[ObjectId]) -> Option<ObjectId> {
    let v = spawn(w, Kind::Vehicle, "VICT", 1, at(4, 0));
    run_to(w, w.tick + 2);
    attack(w, m, v);
    for _ in 0..20 {
        testkit::run(w, 1);
        if support(w, m) > 0 {
            break;
        }
    }
    cands
        .iter()
        .copied()
        .find(|&c| w.obj(c).is_some_and(|o| o.combat.support_beam.is_some()))
}

// spec: S-0518 claim 3 (revision 3, answers Q-0985): of equally distant eligible buildings the one the house acquired first supports; with both placed for the house, that is the earlier placement
#[test]
fn equally_distant_supporters_go_by_placement_order() {
    let mut w = world(|_| {});
    let m = quiet(&mut w, "PRISM", 0, 0, 0);
    let a = quiet(&mut w, "PRISM", 0, -1, 0);
    let b = quiet(&mut w, "PRISM", 0, 0, -1);
    assert_eq!(recruited(&mut w, m, &[a, b]), Some(a));
}

// spec: S-0518 claim 3 (revision 3): a captured building ranks after every building the new owner already held, even one created earlier: A is created first but captured after B was placed, so B supports
#[test]
fn a_captured_building_ranks_after_the_buildings_already_held() {
    let mut w = world(|_| {});
    let m = quiet(&mut w, "PRISM", 0, 0, 0);
    let a = quiet(&mut w, "PRISM", 1, -1, 0);
    let b = quiet(&mut w, "PRISM", 0, 0, -1);
    ownership::change(&mut w, a, Some(HouseId(0)), Cause::Capture);
    assert_eq!(recruited(&mut w, m, &[a, b]), Some(b));
}

// spec: S-0518 claim 3 (revision 3): losing a building and reacquiring it gives it a new, later rank; an unchanged-owner assignment is no acquisition
#[test]
fn reacquiring_gives_a_later_rank_and_an_unchanged_owner_gives_none() {
    let mut w = world(|_| {});
    let m = quiet(&mut w, "PRISM", 0, 0, 0);
    let a = quiet(&mut w, "PRISM", 0, -1, 0);
    let b = quiet(&mut w, "PRISM", 0, 0, -1);
    // A same-owner assignment changes nothing.
    ownership::change(&mut w, a, Some(HouseId(0)), Cause::Capture);
    let first = w.obj(a).unwrap().combat.acquired;
    // B is lost and taken back: it is now newer than A.
    ownership::change(&mut w, b, Some(HouseId(1)), Cause::Capture);
    ownership::change(&mut w, b, Some(HouseId(0)), Cause::Capture);
    assert_eq!(w.obj(a).unwrap().combat.acquired, first);
    assert!(w.obj(b).unwrap().combat.acquired > first);
    assert_eq!(recruited(&mut w, m, &[a, b]), Some(a));
}

// spec: S-0518 claim 3 (revision 3): acquisitions within one tick rank in the order their events take effect, not by object number: B is created later than A but captured first
#[test]
fn acquisitions_in_one_tick_rank_by_event_order() {
    let mut w = world(|_| {});
    let m = quiet(&mut w, "PRISM", 0, 0, 0);
    let a = quiet(&mut w, "PRISM", 1, -1, 0);
    let b = quiet(&mut w, "PRISM", 1, 0, -1);
    ownership::change(&mut w, b, Some(HouseId(0)), Cause::Capture);
    ownership::change(&mut w, a, Some(HouseId(0)), Cause::Capture);
    assert_eq!(recruited(&mut w, m, &[a, b]), Some(b));
}

// spec: S-0518 claim 14 (revision 3): when a guarding charger takes a building the idle action does not run, but the processing keeps the delay of guard (its rate plus one draw over 0 to 2), not a fixed 1 tick and not the attack rate; attack is queued without starting
#[test]
fn a_charger_keeps_the_guard_delay_after_taking_a_building() {
    let mut w = world(|_| {});
    let coil = spawn(&mut w, Kind::Building, "COIL", 0, at(1, 0));
    let u = spawn(&mut w, Kind::Infantry, "CHARGER", 0, at(0, 0));
    let rate = w.rules().behaviour(Behaviour::Guard).rate_delay;
    let mut seen = None;
    for _ in 0..60 {
        testkit::run(&mut w, 1);
        if w.obj(u).unwrap().target == Some(Target::Object(coil)) {
            seen = w.obj(u).unwrap().beh.last;
            let o = w.obj(u).unwrap();
            assert_eq!(
                o.beh.current,
                Some(Behaviour::Guard),
                "attack is only queued"
            );
            assert_eq!(o.beh.queued, Some(Behaviour::Attack));
            break;
        }
    }
    let (_, delay) = seen.expect("it took the building");
    assert!(
        (rate..=rate + 2).contains(&delay),
        "the guard rate {rate} plus a draw over 0 to 2, got {delay}"
    );
}

// spec: S-0518 claim 15 (revision 3): the break-off search takes no targeting-timer reset draw; with nothing in range the held building target and the charging mark stay unchanged, and with a result it becomes the target and clears the mark
#[test]
fn the_break_off_search_takes_no_draw_and_keeps_the_target_when_empty() {
    let mut w = world(|_| {});
    let coil = spawn(&mut w, Kind::Building, "COIL", 0, at(1, 0));
    let u = spawn(&mut w, Kind::Infantry, "CHARGER", 0, at(0, 0));
    assert!(linked::start_charging(&mut w, u));
    let before = w.rng.clone();
    linked::break_off(&mut w, u);
    assert!(w.rng == before, "no draw");
    let o = w.obj(u).unwrap();
    assert_eq!(o.target, Some(Target::Object(coil)));
    assert!(o.combat.charging);
    // An enemy in range: it becomes the target, the mark clears, and still no draw.
    let enemy = spawn(&mut w, Kind::Vehicle, "VICT", 1, at(4, 0));
    let before = w.rng.clone();
    linked::break_off(&mut w, u);
    assert!(w.rng == before, "no draw for the result either");
    let o = w.obj(u).unwrap();
    assert_eq!(o.target, Some(Target::Object(enemy)));
    assert!(!o.combat.charging);
}
