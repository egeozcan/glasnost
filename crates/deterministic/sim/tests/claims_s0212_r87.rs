//! Claim tests of S-0212 revision 87, as revision 97 leaves them: claim
//! 23's gate row for the walk locomotor (after a fresh search and after a
//! re-path) with the end of its movement in progress (scenario P7 run 7b),
//! and no delay left after it (claim 21, r97). Synthetic data only: the
//! test kit's invented walker, our own water cells, no game data.

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::Decimal;
use glasnost_sim::occupancy::in_progress;
use glasnost_sim::path::{self, Search};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit::{self, SIZE, mid};
use glasnost_sim::tick;
use glasnost_sim::trace::{self, What};
use glasnost_sim::world::{ObjectId, Target, World};

/// The combat ruleset with the path-retry delay field at 0.1 game minutes,
/// R = 90 ticks (S-0212 claim 21 and P25 a).
fn ruleset_r90() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.general.path_delay = Some(Decimal("0.1".into()));
    r
}

/// A game on the test kit's empty map with one unit of type `name` of house
/// 0 at the map's middle, orders open (S-0201 claim 14), the trace on.
fn one(kind: Kind, name: &str) -> (World, ObjectId) {
    let mut w = testkit::game_with(&ruleset_r90(), SIZE, 1);
    let id = testkit::spawn(&mut w, kind, name, 0, mid(SIZE, 0, 0), 0);
    while w.tick < 4 {
        tick::step(&mut w);
    }
    trace::switch_on(&mut w);
    (w, id)
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

/// The ticks of `id`'s path computations and their results.
fn searches(events: &[trace::Event], id: ObjectId) -> Vec<(i32, Search)> {
    events
        .iter()
        .filter(|e| e.unit == id)
        .filter_map(|e| match e.what {
            What::Search { result, .. } => Some((e.tick, result)),
            _ => None,
        })
        .collect()
}

// spec: S-0212 claim 23 (gate row: a unit with the walk locomotor has its destination cleared after a fresh search, takes no step, does not give up and has nothing queued; its movement in progress ends in the same movement update)
// spec: S-0212 P7 run 7b (an infantry unit sent onto an island: no step; in its first movement update after the order its destination is cleared and its movement in progress ends; no behaviour is queued)
#[test]
fn p7_run_7b_walker_to_an_island() {
    let (mut w, g) = one(Kind::Infantry, "WALKER");
    let here = w.obj(g).unwrap().pos.cell();
    let isle = mid(SIZE, 10, 0);
    island(&mut w, isle);
    // As a team script's move would: the island cell itself, no
    // substitution before the path computation (S-0214 claim 11).
    let before = w.obj(g).unwrap().dest;
    let o = w.obj_mut(g).unwrap();
    o.dest = Some(Target::Cell(isle));
    o.path.steps.clear();
    glasnost_sim::infantry::destination_assigned(&mut w, g, before);
    assert!(
        in_progress(&w, g),
        "movement in progress from the assignment"
    );
    let queued = w.obj(g).unwrap().beh.queued;
    let t = w.tick;
    tick::step(&mut w);
    let ev = trace::take(&w);
    assert_eq!(searches(&ev, g), vec![(t, Search::GateFailed)]);
    let o = w.obj(g).unwrap();
    assert_eq!(o.dest, None, "cleared in the first movement update");
    assert!(!in_progress(&w, g), "its movement in progress ended");
    assert_eq!(o.beh.queued, queued, "nothing queued");
    // S-0212 r97 claim 21: the clearing ends the delay the fresh search
    // started, so none remains.
    assert!(
        o.path.retry_at <= w.tick,
        "no delay remains after the failed gate"
    );
    assert!(!path::delay_runs(&w, g));
    for _ in 0..30 {
        tick::step(&mut w);
        assert_eq!(w.obj(g).unwrap().pos.cell(), here, "no step");
    }
}

// spec: S-0212 claim 23 (gate row: for the walk locomotor the destination is cleared after a re-path for a moving blocker too, and its movement in progress ends in the same movement update)
// spec: S-0212 claim 21 (r97: the re-path's delay ends at that clearing, so none remains)
#[test]
fn claim_23_walker_repath_gate_failure_clears() {
    let (mut w, g) = one(Kind::Infantry, "WALKER");
    let isle = mid(SIZE, 6, 0);
    island(&mut w, isle);
    // A standing walker on its way, between steps, as at a re-path: a
    // destination and a stored path, no walking target.
    let t = w.tick;
    let o = w.obj_mut(g).unwrap();
    o.dest = Some(Target::Cell(isle));
    o.path.steps = vec![2, 2, 2];
    assert!(o.walk.as_ref().is_some_and(|s| s.target.is_none()));
    assert!(in_progress(&w, g));
    path::failed(&mut w, g, Search::GateFailed, true);
    let o = w.obj(g).unwrap();
    assert_eq!(o.dest, None);
    assert!(o.path.steps.is_empty());
    assert!(!in_progress(&w, g), "its movement in progress ended");
    assert_eq!(o.path.retry_at, t, "the delay ended at the clearing");
    assert!(!path::delay_runs(&w, g));
}

// spec: S-0212 claim 23 (the sentence after the table: for the walk locomotor, the stop of rows 2 and 3 ends its movement in progress in the same movement update, whether or not the unit keeps its destination)
#[test]
fn claim_23_walker_search_failure_ends_movement() {
    for (far, kept) in [(false, true), (true, false)] {
        let (mut w, g) = one(Kind::Infantry, "WALKER");
        let c = w.obj(g).unwrap().pos.cell();
        let d = if far {
            CellXY::new(c.x + 5, c.y)
        } else {
            CellXY::new(c.x + 1, c.y)
        };
        let o = w.obj_mut(g).unwrap();
        o.dest = Some(Target::Cell(d));
        o.path.steps = vec![2];
        assert!(in_progress(&w, g));
        path::failed(&mut w, g, Search::SearchFailed, false);
        let o = w.obj(g).unwrap();
        assert_eq!(o.dest.is_some(), kept, "row {}", if far { 2 } else { 3 });
        assert!(
            !in_progress(&w, g),
            "row {}: ended",
            if far { 2 } else { 3 }
        );
    }
}

// S-0212 r97 replaced claim 21's wait for a new destination: a different
// destination now ends a running delay for the driving and walk
// locomotors. The tests of r97 are in claims_s0212_r97.rs.
