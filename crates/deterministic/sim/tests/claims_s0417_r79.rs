//! Claim tests of S-0417 revision 79: claim 17's request to a walking unit
//! whose destination gets no path (*No path*) once a different destination
//! ends a running retry delay (S-0212 claim 21, r97): no delay after the
//! gate's failure and the walk off without a wait (scenario U10 g run 2),
//! the kept destination's first request, and the cycle in which every
//! failing destination takes a request's draw in each turn. Synthetic data
//! only: the production ruleset's invented barracks and infantry types, our
//! own water cells, no game data.

mod econ_common;

use econ_common::*;
use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::econ::release::{self, Outcome};
use glasnost_sim::objects;
use glasnost_sim::occupancy::in_progress;
use glasnost_sim::orders::{self, Order};
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

/// What [`walk_off`] saw: the first accepted request's tick and
/// destination, the tick in which the unit first moved, and every failure
/// recorded (tick, search result, destination cleared).
type WalkOff = (Option<(i32, CellXY)>, Option<i32>, Vec<(i32, Search, bool)>);

/// Steps `w` up to `n` ticks; returns the first accepted request's tick
/// and destination, the tick in which `g` first moved from where it stood
/// at that request, and every failure recorded.
fn walk_off(w: &mut World, g: ObjectId, n: i32) -> WalkOff {
    let mut accepted = None;
    let mut from = None;
    let mut fails = Vec::new();
    let mut moved = None;
    for _ in 0..n {
        let k = w.tick;
        step(w);
        let ev = trace::take(w);
        fails.extend(failures(&ev, g));
        if accepted.is_none()
            && let Some(a) = answers(&ev, g).into_iter().find(|a| a.1)
        {
            accepted = Some((a.0, a.2.expect("a destination")));
            from = Some(w.obj(g).unwrap().pos);
            continue;
        }
        if let Some(p) = from
            && w.obj(g).unwrap().pos != p
        {
            moved = Some(k);
            break;
        }
    }
    (accepted, moved, fails)
}

// spec: S-0417 claim 17 (r79, No path, failed at the reachability gate: the destination cleared and movement in progress ended in the same movement update, no retry delay left; the first request after it accepted with its draw; the destination it gives, different, searched for without a wait)
// spec: S-0212 claim 21 (r97: the clearing at the gate's failure ends the delay that fresh search started)
// spec: S-0417 U10 g (run 2's expectations, R = 90: the first step as many ticks after the accepted request as part b's after its first request, and fewer than R − 2 = 88 ticks after the gate's failure)
#[test]
fn u10_g_run_2_gate_failure_then_the_walk_without_a_wait() {
    // Part b's set-up: the same release with no order.
    let (mut b, _, gb) = released(&ruleset_r90());
    let (acc_b, moved_b, fails_b) = walk_off(&mut b, gb, 300);
    assert!(fails_b.is_empty(), "{fails_b:?}");
    let (ab, _) = acc_b.expect("part b: an accepted request");
    let lag_b = moved_b.expect("part b: a first step") - ab;

    // Part g: a move order to the island, given directly to the unit after
    // its release and taken before its first turn (the part's harness).
    let (mut w, k, g) = released(&ruleset_r90());
    let release_pos = w.obj(g).unwrap().pos;
    let here = release_pos.cell();
    let isle = CellXY::new(here.x + 6, here.y + 1);
    island(&mut w, isle);
    let o = Order::Behaviour {
        actor: g,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(isle)),
    };
    assert!(orders::issue(&mut w, HouseId(0), o));
    orders::apply_pending(&mut w);
    assert_eq!(w.obj(g).unwrap().dest, Some(Target::Cell(isle)));
    // F: the unit's first turn, in which its fresh search fails at the gate.
    let f = w.tick;
    step(&mut w);
    let ev = trace::take(&w);
    assert_eq!(failures(&ev, g), vec![(f, Search::GateFailed, true)]);
    assert!(
        answers(&ev, g).iter().all(|a| !a.1),
        "no request accepted at F"
    );
    let o = w.obj(g).unwrap();
    assert_eq!(o.dest, None, "cleared in the failure's update");
    assert!(!in_progress(&w, g));
    assert!(
        o.path.retry_at <= w.tick,
        "no retry delay at the end of the failure's tick"
    );
    assert!(!path::delay_runs(&w, g));
    assert_eq!(o.pos, release_pos, "no step");
    // spec: S-0215 claim 22 (r132, End-of-update check: the order's move, processed in F before the movement update, saw the destination and chose the standard delay; at the end of F it is still current, not moving and with no destination, so the check queues guard, current at G = F + 1)
    assert_eq!(o.beh.current, Some(Behaviour::Move));
    assert_eq!(
        o.beh.queued,
        Some(Behaviour::Guard),
        "guard queued at the end of F"
    );
    // G: the first later turn in which guard is current on a cell of K.
    let (acc, moved, fails) = walk_off(&mut w, g, 200);
    assert!(fails.is_empty(), "{fails:?}");
    let (a, d) = acc.expect("an accepted request");
    assert_eq!(a, f + 1, "G = F + 1");
    assert_ne!(d, isle);
    assert!(!holds(&w, d, k), "off the factory");
    let m = moved.expect("a first step");
    assert_eq!(m - a, lag_b, "as soon after the request as part b's step");
    assert!(
        m - f < 88,
        "fewer than R - 2 ticks after the failure: {}",
        m - f
    );
}

// spec: S-0417 claim 17 (r79, No path, waiting for the retry delay: a walking unit that keeps its destination after a failed search waits for the delay with it; its movement in progress ended, so the first request is accepted with its draw, and a new destination different from the kept one ends the delay)
// spec: S-0212 claim 23 (row 3, within 1 cell: the walker keeps its destination) and claim 21 (r97)
#[test]
fn claim_17_kept_destination_request_ends_the_delay() {
    let (mut w, k, g) = released(&ruleset_r90());
    let here = w.obj(g).unwrap().pos.cell();
    // A kept destination on K, so that the request's destination, off K,
    // differs from it.
    let near = [(1, 0), (-1, 0), (0, 1), (0, -1)]
        .into_iter()
        .map(|(dx, dy)| CellXY::new(here.x + dx, here.y + dy))
        .find(|&c| holds(&w, c, k))
        .expect("a neighbouring cell of K");
    give(&mut w, g, near);
    let t = w.tick;
    path::failed(&mut w, g, Search::SearchFailed, false);
    assert_eq!(w.obj(g).unwrap().dest, Some(Target::Cell(near)), "kept");
    assert_eq!(w.obj(g).unwrap().path.retry_at, t + 90);
    assert!(path::delay_runs(&w, g), "it waits with it");
    assert!(!in_progress(&w, g), "its movement in progress ended");
    let mut p = w.clone();
    let before = p.rng.taken();
    assert!(scatter::request_resting(&mut p, g, REQ), "accepted");
    assert!(p.rng.taken() > before, "with its draw");
    let d = p.obj(g).unwrap().dest;
    assert!(d.is_some_and(|d| matches!(d, Target::Cell(c) if !holds(&p, c, k))));
    assert_ne!(d, Some(Target::Cell(near)));
    assert!(
        !path::delay_runs(&p, g),
        "the different destination ended it"
    );
    assert!(in_progress(&p, g));
    assert_eq!(probe(&p, g), (false, 0), "the next request refused");
    let from = p.obj(g).unwrap().pos;
    let mut moved = None;
    for _ in 0..5 {
        let k2 = p.tick;
        step(&mut p);
        if p.obj(g).unwrap().pos != from {
            moved = Some(k2);
            break;
        }
    }
    let m = moved.expect("it walks without a wait");
    assert!(m < t + 5, "not held back to {}", t + 90);
}

// spec: S-0212 claim 21 (r98, Q-2896: for the walk locomotor the destination it already holds, given again, ends the running retry delay, so the kept destination is not held unsearched; S-0212's note on S-0417 claim 17 and Open question 16)
// design: S-0417 Open question 16 (whether later requests are refused where a request gives the kept destination is not established; here that destination reaches the walk locomotor, which has movement in progress until its search, S-0107 Terms, so later requests are refused until then)
#[test]
fn open_question_16_the_kept_destination_given_again() {
    let (mut w, k, g) = released(&ruleset_r90());
    let here = w.obj(g).unwrap().pos.cell();
    let near = [(1, 0), (-1, 0), (0, 1), (0, -1)]
        .into_iter()
        .map(|(dx, dy)| CellXY::new(here.x + dx, here.y + dy))
        .find(|&c| holds(&w, c, k))
        .expect("a neighbouring cell of K");
    give(&mut w, g, near);
    let t = w.tick;
    path::failed(&mut w, g, Search::SearchFailed, false);
    assert!(path::delay_runs(&w, g), "the failed search started it");
    give(&mut w, g, near);
    assert!(!path::delay_runs(&w, g), "the delay has ended");
    assert_eq!(w.obj(g).unwrap().path.retry_at, t);
    assert!(in_progress(&w, g));
    assert_eq!(probe(&w, g), (false, 0), "refused until its search");
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

// spec: S-0417 claim 17 (r79, No path, the repeated cycle: each accepted destination that differs from any destination the unit kept is searched for without a wait for the delay; a unit on a building cell whose every scatter destination fails takes a request's draw in each turn in which a request comes, R = 90 as for R = 0)
#[test]
fn claim_17_every_failing_destination_takes_a_draw_each_turn() {
    let (mut w, k, g) = released(&ruleset_r90());
    moat(&mut w, k);
    for _ in 0..40 {
        step(&mut w);
    }
    let ev = trace::take(&w);
    let asked = answers(&ev, g);
    let yes = asked.iter().filter(|a| a.1).count();
    let no = asked.len() - yes;
    let failed = failures(&ev, g);
    assert!(failed.len() >= 30, "{failed:?}");
    assert!(
        failed.iter().all(|f| f.1 == Search::GateFailed && f.2),
        "{failed:?}"
    );
    assert!(no <= 1, "refused {no}");
    assert!(yes >= 39, "accepted {yes}");
}
