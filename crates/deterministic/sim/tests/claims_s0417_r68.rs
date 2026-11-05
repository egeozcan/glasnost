//! Claim tests of S-0417 revision 68 (answers Q-1980): claim 17's request
//! to a walking unit whose destination gets no path (*No path*): the
//! gate's failure, a failed search and the cycle with no delay, as
//! revision 79 leaves them (no wait for the retry delay after the gate's
//! failure; claims_s0417_r79.rs). Synthetic data only: the production ruleset's invented
//! barracks and infantry types, our own water cells, no game data.

mod econ_common;

use econ_common::*;
use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::econ::release::{self, Outcome};
use glasnost_sim::objects;
use glasnost_sim::occupancy::in_progress;
use glasnost_sim::path::{self, Search};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::scatter::{self, ScatterRequest};
use glasnost_sim::trace::{self, What};
use glasnost_sim::world::{HouseId, Layer, ObjectId, Target, World};

/// Claim 17's request: forced, move-anywhere, no threat point.
const REQ: ScatterRequest = ScatterRequest {
    threat: None,
    forced: true,
    anywhere: true,
};

/// The production ruleset with the path-retry delay field at 0.1 game
/// minutes, R = 90 ticks (S-0212 claim 21 and P25 a; S-0417 U10 g run 2).
fn ruleset_r90() -> Ruleset {
    let mut r = ruleset();
    r.sim.general.path_delay = dec("0.1");
    r
}

/// Whether cell `c` lists building `k`.
fn holds(w: &World, c: CellXY, k: ObjectId) -> bool {
    w.cells.list(c, Layer::Ground).contains(&k)
}

/// The scatter answers recorded for `id`: (tick, accepted, destination).
fn answers(events: &[trace::Event], id: ObjectId) -> Vec<(i32, bool, Option<CellXY>)> {
    events
        .iter()
        .filter(|e| e.unit == id)
        .filter_map(|e| match e.what {
            What::Asked { accepted, dest } => Some((e.tick, accepted, dest)),
            _ => None,
        })
        .collect()
}

/// The failed path computations recorded for `id`: (tick, result, destination gone).
fn failures(events: &[trace::Event], id: ObjectId) -> Vec<(i32, Search, bool)> {
    events
        .iter()
        .filter(|e| e.unit == id)
        .filter_map(|e| match e.what {
            What::Failed {
                result,
                dest_cleared,
                ..
            } => Some((e.tick, result, dest_cleared)),
            _ => None,
        })
        .collect()
}

/// Makes `c` an island: its eight neighbours become water.
fn island(w: &mut World, c: CellXY) {
    for dy in -1..=1 {
        for dx in -1..=1 {
            if (dx, dy) != (0, 0) {
                let n = CellXY::new(c.x + dx, c.y + dy);
                w.cells.grid.get_mut(n).unwrap().land = land::WATER;
                glasnost_sim::regions::cell_changed(w, n);
            }
        }
    }
}

/// A barracks with no rally point and one GRUNT released from it, the
/// trace on. Returns the world, K and the unit.
fn released(r: &Ruleset) -> (World, ObjectId, ObjectId) {
    let mut w = game(r);
    let k = building(&mut w, "BARRACKS", 0, 0, 0);
    ready(&mut w);
    trace::switch_on(&mut w);
    let ti = w.rules().index_of(Kind::Infantry, "GRUNT").unwrap();
    let g = objects::create(&mut w, Kind::Infantry, ti, Some(HouseId(0)));
    assert_eq!(release::release(&mut w, k, g), Outcome::Released);
    (w, k, g)
}

/// Gives `g` destination `c` as an order would, after its release and
/// before its first turn (S-0417 U10 g).
fn give(w: &mut World, g: ObjectId, c: CellXY) {
    let before = w.obj(g).unwrap().dest;
    let o = w.obj_mut(g).unwrap();
    o.dest = Some(Target::Cell(c));
    o.path.steps.clear();
    glasnost_sim::infantry::destination_assigned(w, g, before);
}

/// Draws taken by one claim 17 request on a copy of the world, and
/// whether it was accepted.
fn probe(w: &World, g: ObjectId) -> (bool, u64) {
    let mut p = w.clone();
    let before = p.rng.taken();
    let accepted = scatter::request_resting(&mut p, g, REQ);
    (accepted, p.rng.taken() - before)
}

// spec: S-0417 claim 17 (No path, failed at the reachability gate: the destination is cleared in the same movement update and movement in progress ends; the behaviour is unchanged; the first request after it is accepted with its draw; no request finds the destination that failed)
// spec: S-0212 claim 21 (r97: the clearing ends the delay R the failed fresh search started, so the accepted destination is searched for without a wait)
#[test]
fn claim_17_gate_failure_clears_then_accepts() {
    let (mut w, k, g) = released(&ruleset_r90());
    assert_eq!(w.rules().general.retry_delay, 90);
    let here = w.obj(g).unwrap().pos.cell();
    let isle = CellXY::new(here.x + 6, here.y + 1);
    island(&mut w, isle);
    give(&mut w, g, isle);
    let t = w.tick;
    step(&mut w);
    let ev = trace::take(&w);
    let failed = failures(&ev, g);
    assert_eq!(failed, vec![(t, Search::GateFailed, true)], "{failed:?}");
    let o = w.obj(g).unwrap();
    assert!(o.path.retry_at <= w.tick, "no delay left (r97)");
    assert_eq!(o.beh.current, Some(Behaviour::Guard), "behaviour unchanged");
    let asked = answers(&ev, g);
    assert_eq!(asked.len(), 1, "{asked:?}");
    assert!(asked[0].1, "accepted after the failure");
    let d = asked[0].2.expect("a new destination");
    assert_ne!(d, isle);
    assert!(!holds(&w, d, k));
    // The new destination is searched for at once and walked to.
    let mut off = false;
    for _ in 0..60 {
        assert_eq!(w.obj(g).unwrap().dest, Some(Target::Cell(d)));
        step(&mut w);
        if !holds(&w, w.obj(g).unwrap().pos.cell(), k) {
            off = true;
            break;
        }
    }
    assert!(off, "it walks off the foundation well before R is over");
    assert!(w.tick < t + 88, "fewer than R - 2 ticks after the failure");
    assert!(failures(&trace::take(&w), g).is_empty());
}

// spec: S-0417 claim 17 (No path, failed search: a fresh search that passes the gate but finds no path ends the walking unit's movement in progress whether or not it keeps its destination, so the first request after it is accepted with its draw)
// spec: S-0212 claim 23 (row 3, within 1 cell: the walker keeps its destination; the sentence after the table: its movement in progress ends)
#[test]
fn claim_17_failed_search_kept_destination_accepts() {
    let (mut w, k, g) = released(&ruleset_r90());
    let here = w.obj(g).unwrap().pos.cell();
    let near = CellXY::new(here.x + 1, here.y);
    give(&mut w, g, near);
    assert!(in_progress(&w, g));
    path::failed(&mut w, g, Search::SearchFailed, false);
    let o = w.obj(g).unwrap();
    assert_eq!(o.dest, Some(Target::Cell(near)), "kept within 1 cell");
    assert!(!in_progress(&w, g), "its movement in progress ended");
    let mut p = w.clone();
    let before = p.rng.taken();
    assert!(scatter::request_resting(&mut p, g, REQ), "accepted");
    assert!(p.rng.taken() > before, "with its draw");
    let d = p.obj(g).unwrap().dest;
    assert!(d.is_some_and(|d| matches!(d, Target::Cell(c) if !holds(&p, c, k))));
    // The accepted destination reaches the locomotor: moving again, even
    // were it the same cell, and the next request is refused.
    assert!(in_progress(&p, g));
    assert_eq!(probe(&p, g), (false, 0));
}

/// Water all round K's foundation, one cell out, so that every scatter
/// destination lies beyond it and fails the gate.
fn moat(w: &mut World, k: ObjectId) {
    let cells: Vec<CellXY> = (0..100)
        .flat_map(|y| (0..100).map(move |x| CellXY::new(x, y)))
        .filter(|&c| holds(w, c, k))
        .collect();
    let mut ring = Vec::new();
    for c in &cells {
        for dy in -1..=1 {
            for dx in -1..=1 {
                let n = CellXY::new(c.x + dx, c.y + dy);
                if !cells.contains(&n) && !ring.contains(&n) {
                    ring.push(n);
                }
            }
        }
    }
    for n in ring {
        w.cells.grid.get_mut(n).unwrap().land = land::WATER;
        glasnost_sim::regions::cell_changed(w, n);
    }
}

/// Runs `ticks` ticks of a released unit on a moated barracks; returns the
/// accepted requests, the refused ones and the failed fresh searches.
fn moated(r: &Ruleset, ticks: i32) -> (usize, usize, Vec<(i32, Search, bool)>, u64) {
    let (mut w, k, g) = released(r);
    moat(&mut w, k);
    let before = w.rng.taken();
    for _ in 0..ticks {
        step(&mut w);
    }
    let draws = w.rng.taken() - before;
    let ev = trace::take(&w);
    let asked = answers(&ev, g);
    let yes = asked.iter().filter(|a| a.1).count();
    (yes, asked.len() - yes, failures(&ev, g), draws)
}

// S-0417 r79 replaced the repeated cycle's wait (one draw per failed
// search): each differing destination is now searched for without a wait,
// so a draw is taken in each turn, R = 90 as for R = 0
// (claims_s0417_r79.rs).

// spec: S-0417 claim 17 (No path: when R is 0 there is no wait, and the unit takes one draw in each turn in which a request comes)
#[test]
fn claim_17_no_delay_one_draw_per_turn() {
    let mut r = ruleset();
    r.sim.general.path_delay = dec("0");
    let (yes, no, failed, _) = moated(&r, 40);
    assert!(failed.len() >= 30, "{failed:?}");
    assert!(no <= 1, "refused {no}");
    assert!(yes >= 39, "accepted {yes}");
}
