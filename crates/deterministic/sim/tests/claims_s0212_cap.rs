//! Claim tests for the cap of 5 cell searches of a path computation with
//! the corridor (S-0212 claim 13), on synthetic data: the shape of the
//! lead's pathing report 1 (M8 step A4), our own layout. A unit stands at
//! the exit just south of a 3 × 2 block of buildings (a factory), a long
//! row of buildings lies further north with open ground round both ends,
//! and the unit is sent to a cell behind the row.
//!
//! The coarse search sees through buildings (claim 1), so every corridor
//! runs into the block or the row, each retry closes connections (claims
//! 13 and 13a), and the 5th cell search inside a corridor fails: the path
//! computation fails although a way round exists, and the unit gives up
//! (claim 23). `quirks.corridor_retry_no_fallback` does not change this: it
//! applies only when a repeated coarse search fails, never at the cap. Q-1771
//! asks whether the original's unit walks round.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::BuildingType;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::path::{Mode, Pathfinder, Request, S0212, Search};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::tick;
use glasnost_sim::trace::{self, CellSearch, What};
use glasnost_sim::world::{HouseId, Target, World};

const BIG: i32 = 50;

/// The combat ruleset with BLOCK, a plain 1 × 1 building, and the switch
/// `quirks.corridor_retry_no_fallback` set to `no_fallback`.
fn rules(no_fallback: bool) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("BLOCK".into());
    r.sim.buildings.insert(
        "BLOCK".into(),
        BuildingType {
            strength: Some(200),
            foundation: Some([1, 1]),
            ..Default::default()
        },
    );
    r.globals.quirks.corridor_retry_no_fallback = no_fallback;
    r
}

/// The cells of the layout, as offsets from the map's middle: the 3 × 2
/// block at (1..=3, 0..=1); the unit at its exit (2, 2); a row of ten
/// buildings at y = −4 from x = −4 to x = 5, open ground round both ends;
/// the destination (−1, −14), behind the row. The offsets place the block
/// and the row against the sector grid (claim 13b) so that the computation
/// reaches the cap; other placements fail earlier or succeed.
fn blocks() -> Vec<(i32, i32)> {
    let mut b = Vec::new();
    for x in 1..=3 {
        for y in 0..=1 {
            b.push((x, y));
        }
    }
    for x in -4..=5 {
        b.push((x, -4));
    }
    b
}

const UNIT: (i32, i32) = (2, 2);
const GOAL: (i32, i32) = (-1, -14);

fn at((dx, dy): (i32, i32)) -> glasnost_map::CellXY {
    testkit::mid(BIG, dx, dy)
}

/// The layout with an infantry unit of house 0 at the exit, the trace on.
fn layout(no_fallback: bool) -> (World, glasnost_sim::world::ObjectId) {
    let mut w = testkit::game_with(&rules(no_fallback), BIG, 7);
    testkit::run(&mut w, 2);
    for b in blocks() {
        testkit::spawn(&mut w, Kind::Building, "BLOCK", 0, at(b), 0);
    }
    testkit::run(&mut w, 2);
    let id = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, at(UNIT), 0);
    testkit::run(&mut w, 1);
    trace::switch_on(&mut w);
    (w, id)
}

/// What a person's move order to the destination gives: the tick (counted
/// from the order) at which the destination was cleared, the unit's cell
/// then, its current behaviour, and the cell searches of its first path
/// computation.
struct Outcome {
    cleared: Option<i32>,
    cell: glasnost_map::CellXY,
    behaviour: Option<Behaviour>,
    searches: Vec<CellSearch>,
    result: Option<Search>,
}

fn order_move(no_fallback: bool) -> Outcome {
    let (mut w, id) = layout(no_fallback);
    orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: id,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(at(GOAL))),
        },
    );
    let mut cleared = None;
    for t in 0..20 {
        tick::step(&mut w);
        if w.obj(id).unwrap().dest.is_none() {
            cleared = Some(t);
            break;
        }
    }
    let (searches, result) = trace::take(&w)
        .into_iter()
        .find_map(|e| match e.what {
            What::Search {
                searches, result, ..
            } if e.unit == id => Some((searches, Some(result))),
            _ => None,
        })
        .unwrap_or_default();
    let o = w.obj(id).unwrap();
    Outcome {
        cleared,
        cell: o.pos.cell(),
        behaviour: o.beh.current,
        searches,
        result,
    }
}

// spec: S-0212 claim 13 (the searches of a path computation with a corridor: the 5th failed cell search inside a corridor fails the computation, whatever the retry table would give) and claim 23 row 2 (the unit stops and gives up: its destination is cleared and, with a human-controlled owner, it queues guard); Edge cases (buildings do not separate regions, claim 1)
#[test]
fn claim_13_fifth_corridor_search_fails_the_computation() {
    let x = order_move(true);
    assert_eq!(x.result, Some(Search::SearchFailed));
    assert_eq!(x.searches.len(), 5, "five cell searches: {:?}", x.searches);
    assert!(
        x.searches.iter().all(|s| s.corridor && s.found.is_none()),
        "all inside a corridor, none found: {:?}",
        x.searches
    );
    // The order applies in the next tick (S-0213 claim 1); the path
    // computation fails in the one after.
    assert_eq!(x.cleared, Some(1), "the destination is cleared at once");
    assert_eq!(x.cell, at(UNIT), "the unit does not move");
    assert_eq!(x.behaviour, Some(Behaviour::Guard));
}

// spec: S-0212 claim 13 (a cell search without a corridor is confined by nothing but the search limits) and claim 1 (buildings leave cells open): the way round exists
#[test]
fn claim_13_a_search_without_a_corridor_finds_the_way_round() {
    let (w, id) = layout(true);
    let found = S0212.find(
        &w,
        &Request {
            unit: id,
            from: at(UNIT),
            to: at(GOAL),
            mode: Mode::Fresh,
            congested: Default::default(),
        },
    );
    assert!(found.is_some(), "a path round the row exists");
}

// spec: S-0212 claim 13 and Edge cases (quirks.corridor_retry_no_fallback off: one more cell search without a corridor only after a repeated coarse search fails; a retry is made only after fewer than 5 cell searches, so the cap still fails the computation)
#[test]
fn corridor_retry_no_fallback_off_does_not_lift_the_cap() {
    let on = order_move(true);
    let off = order_move(false);
    assert_eq!(off.result, Some(Search::SearchFailed));
    assert_eq!(off.searches, on.searches);
    assert_eq!(off.cleared, Some(1));
}
