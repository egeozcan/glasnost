//! Claim tests of S-0212 revision 87: claim 21's running delay against a
//! new destination, and claim 23's gate row for the walk locomotor (after
//! a fresh search and after a re-path) with the end of its movement in
//! progress (scenario P7 run 7b). Synthetic data only: the test kit's
//! invented walker and buggy, our own water cells, no game data.

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::Decimal;
use glasnost_sim::occupancy::in_progress;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::path::{self, Search};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit::{self, SIZE, mid};
use glasnost_sim::tick;
use glasnost_sim::trace::{self, What};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

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

/// A move order to cell `c`.
fn move_to(w: &mut World, id: ObjectId, c: CellXY) {
    let o = Order::Behaviour {
        actor: id,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(c)),
    };
    assert!(orders::issue(w, HouseId(0), o), "move order refused");
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

/// Steps up to `n` ticks until `id` makes a path computation; returns its
/// tick and result.
fn next_search(w: &mut World, id: ObjectId, n: i32) -> Option<(i32, Search)> {
    for _ in 0..n {
        tick::step(w);
        if let Some(&x) = searches(&trace::take(w), id).first() {
            return Some(x);
        }
    }
    None
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
    assert_eq!(
        o.path.retry_at,
        t + 90,
        "the delay runs after the fresh search"
    );
    for _ in 0..30 {
        tick::step(&mut w);
        assert_eq!(w.obj(g).unwrap().pos.cell(), here, "no step");
    }
}

// spec: S-0212 claim 23 (gate row: for the walk locomotor the destination is cleared after a re-path for a moving blocker too, and its movement in progress ends in the same movement update; the retry delay runs after a walker's re-path, claim 21)
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
    assert_eq!(o.path.retry_at, t + 90);
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

// spec: S-0212 claim 21 (a new destination does not cut a running delay short for the walk locomotor: the first search for it waits for what remains of R)
#[test]
fn claim_21_walker_new_destination_waits_for_the_delay() {
    let (mut w, g) = one(Kind::Infantry, "WALKER");
    let isle = mid(SIZE, 8, 0);
    island(&mut w, isle);
    move_to(&mut w, g, isle);
    let (t, r) = next_search(&mut w, g, 5).expect("a search");
    assert_eq!(r, Search::GateFailed);
    assert_eq!(w.obj(g).unwrap().path.retry_at, t + 90);
    for _ in 0..20 {
        tick::step(&mut w);
    }
    let open = mid(SIZE, -6, 0);
    move_to(&mut w, g, open);
    tick::step(&mut w);
    assert_eq!(w.obj(g).unwrap().dest, Some(Target::Cell(open)));
    assert!(
        in_progress(&w, g),
        "the new destination reached the locomotor"
    );
    let here = w.obj(g).unwrap().pos.cell();
    while w.tick < t + 90 {
        tick::step(&mut w);
        assert_eq!(w.obj(g).unwrap().pos.cell(), here);
        assert!(in_progress(&w, g), "it waits with movement in progress");
    }
    assert_eq!(
        next_search(&mut w, g, 5),
        Some((t + 90, Search::Found)),
        "the search when R is over"
    );
}

// spec: S-0212 state table, row "destination changes" (for the locomotors other than walking the same rule, at low confidence: a fresh search for a new destination waits for a running retry delay)
#[test]
fn state_table_drive_new_destination_waits_for_the_delay() {
    let (mut w, v) = one(Kind::Vehicle, "BUGGY");
    let isle = mid(SIZE, 8, 0);
    island(&mut w, isle);
    move_to(&mut w, v, isle);
    let (failed_at, r) = next_search(&mut w, v, 5).expect("a search");
    assert_eq!(r, Search::GateFailed);
    assert_eq!(w.obj(v).unwrap().dest, None);
    assert_eq!(w.obj(v).unwrap().path.retry_at, failed_at + 90);
    move_to(&mut w, v, mid(SIZE, -6, 0));
    assert_eq!(
        next_search(&mut w, v, 120),
        Some((failed_at + 90, Search::Found))
    );
}
