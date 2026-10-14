//! Claim tests for S-0215 claim 11a (a failed departure while harvesting)
//! on synthetic data: the invented harvest ruleset of `harvest_common`, no
//! stock data. House Alpha (0) of the testkit's sandbox game, human.

mod harvest_common;

use glasnost_map::CellXY;
use glasnost_sim::path::{self, Search};
use glasnost_sim::rules::Behaviour;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};
use harvest_common::*;

/// REFN's docking cell when its top-left cell is (−6, −1): (3, 1) from it
/// (S-0402 claim 10).
fn dock_cell() -> CellXY {
    at(-3, 0)
}

fn unblock(w: &mut World) {
    while w.tick <= w.orders_blocked_through {
        testkit::run(w, 1);
    }
}

/// A human-owned harvester on REFN's docking cell, without contact, with
/// *harvest* current, nothing queued and a processing timer that has not
/// expired, heading for `goal`.
fn harvesting(goal: CellXY) -> (World, ObjectId) {
    let r = ruleset();
    let mut w = game(&r);
    unblock(&mut w);
    place_building(&mut w, "REFN", 0, at(-6, -1));
    let h = place_vehicle(&mut w, "HARV", 0, dock_cell(), 0);
    let t = w.tick;
    let o = w.obj_mut(h).unwrap();
    o.beh.current = Some(Behaviour::Harvest);
    o.beh.queued = None;
    o.beh.expired = false;
    o.beh.last = Some((t, 1000));
    o.dest = Some(Target::Cell(goal));
    (w, h)
}

// spec: S-0215 claim 11a (a failed search that gives up queues guard for a human owner with no immediate start; harvest stays current and the processing timer is unchanged; guard starts at the next permitted per-tick start without waiting for the old timer, and its start expires the timer so that turn processes it; no docking-cell exception)
#[test]
fn claim_11a_give_up_queues_guard_and_starts_it_next_turn() {
    let (mut w, h) = harvesting(at(8, 8));
    let tick = w.tick;
    path::failed(&mut w, h, Search::SearchFailed, false);
    let b = &w.obj(h).unwrap().beh;
    assert_eq!(b.current, Some(Behaviour::Harvest));
    assert_eq!(b.queued, Some(Behaviour::Guard));
    assert_eq!(b.last, Some((tick, 1000)));
    assert!(!b.expired);
    assert!(!b.due(tick + 1), "the old timer has not run out");
    testkit::run(&mut w, 1);
    let b = &w.obj(h).unwrap().beh;
    assert_eq!(b.current, Some(Behaviour::Guard));
    assert_eq!(b.queued, None);
    let (at_tick, _) = b.last.expect("processed");
    assert_eq!(
        at_tick,
        w.tick - 1,
        "guard processed in the turn it started"
    );
}

// spec: S-0215 claim 11a (a gate refusal, or a failed search within one cell that does not give up, queues nothing; harvest remains current)
#[test]
fn claim_11a_other_failures_queue_nothing() {
    for (goal, res) in [
        (at(8, 8), Search::GateFailed),
        (at(-2, 0), Search::SearchFailed),
    ] {
        let (mut w, h) = harvesting(goal);
        path::failed(&mut w, h, res, false);
        let b = &w.obj(h).unwrap().beh;
        assert_eq!(b.current, Some(Behaviour::Harvest), "{res:?}");
        assert_eq!(b.queued, None, "{res:?}");
    }
}
