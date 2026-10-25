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
//!
//! The last tests apply our own patch layer *Pathing and other fixes*
//! (`data/layers/play-fixes.jsonc`, M8 step C7), which turns
//! `quirks.corridor_retry_no_fallback` and `quirks.corridor_split_probe` off:
//! with both off the same shapes find the way, with either alone they do not.

use glasnost_rules::layers::Layers;
use glasnost_rules::quirks::Quirks;
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

/// The combat ruleset with BLOCK and the quirk switches of `layers` applied
/// to the defaults (the file itself, through `Layers::apply_quirks`).
fn rules_with(layers: Layers) -> Ruleset {
    let mut r = rules(true);
    assert_eq!(
        r.globals.quirks,
        Quirks::default(),
        "the test ruleset keeps the defaults"
    );
    r.globals.quirks = layers
        .apply_quirks(r.globals.quirks)
        .expect("the layer applies");
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
    layout_with(&rules(no_fallback))
}

fn layout_with(r: &Ruleset) -> (World, glasnost_sim::world::ObjectId) {
    let mut w = testkit::game_with(r, BIG, 7);
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
    order_move_with(&rules(no_fallback), 0).0
}

/// [`order_move`] under the rules `r`; then `more` further ticks, after
/// which the unit's cell is given too.
fn order_move_with(r: &Ruleset, more: i32) -> (Outcome, glasnost_map::CellXY) {
    let (mut w, id) = layout_with(r);
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
    let out = Outcome {
        cleared,
        cell: o.pos.cell(),
        behaviour: o.beh.current,
        searches,
        result,
    };
    testkit::run(&mut w, more);
    (out, w.obj(id).unwrap().pos.cell())
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

// The shape of the lead's pathing report 2 (M8 step A4), our own layout: a
// vehicle stands on the cell east of a 3 × 3 block of buildings (a
// refinery's docking cell), two more blocks lie west of it, and it is sent
// to a cell far west of all three. Every corridor runs into the blocks
// (claim 1), the 5th cell search inside a corridor fails, and the vehicle
// gives up although a way round exists. From a start south of the blocks
// the first corridor search finds the way. Q-1771 asks whether the
// original's unit walks round.

/// The cells of report 2's shape, on a map of size 64 shifted so that the
/// blocks sit against the sector grid (claim 13b) as in the report: the
/// 3 × 3 block at (52..=54, 98..=100), a 4 × 3 block at (50..=53, 94..=96)
/// and a 4 × 4 block at (46..=49, 96..=99).
fn dock_blocks() -> Vec<(i32, i32)> {
    let mut b = Vec::new();
    for (x0, x1, y0, y1) in [(52, 54, 98, 100), (50, 53, 94, 96), (46, 49, 96, 99)] {
        for x in x0..=x1 {
            for y in y0..=y1 {
                b.push((x, y));
            }
        }
    }
    b
}

const DOCK: (i32, i32) = (55, 99);
const SOUTH: (i32, i32) = (56, 102);
const WEST: (i32, i32) = (36, 95);

fn dock_cell((x, y): (i32, i32)) -> glasnost_map::CellXY {
    glasnost_map::CellXY::new(x + 16, y - 32)
}

/// Report 2's shape with a vehicle of house 0 at `from`, the trace on and
/// orders accepted.
fn dock_layout(from: (i32, i32)) -> (World, glasnost_sim::world::ObjectId) {
    dock_layout_with(from, &rules(true))
}

fn dock_layout_with(from: (i32, i32), r: &Ruleset) -> (World, glasnost_sim::world::ObjectId) {
    let mut w = testkit::game_with(r, 64, 7);
    testkit::run(&mut w, 2);
    for b in dock_blocks() {
        testkit::spawn(&mut w, Kind::Building, "BLOCK", 0, dock_cell(b), 0);
    }
    testkit::run(&mut w, 2);
    let id = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 0, dock_cell(from), 0);
    testkit::run(&mut w, 1);
    while w.tick <= w.orders_blocked_through {
        testkit::run(&mut w, 1);
    }
    trace::switch_on(&mut w);
    (w, id)
}

fn dock_move(from: (i32, i32)) -> Outcome {
    dock_move_with(from, &rules(true), 0).0
}

/// [`dock_move`] under the rules `r`; then `more` further ticks, after which
/// the vehicle's cell is given too.
fn dock_move_with(from: (i32, i32), r: &Ruleset, more: i32) -> (Outcome, glasnost_map::CellXY) {
    let (mut w, id) = dock_layout_with(from, r);
    orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: id,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(dock_cell(WEST))),
        },
    );
    let mut cleared = None;
    for t in 0..3 {
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
    let out = Outcome {
        cleared,
        cell: o.pos.cell(),
        behaviour: o.beh.current,
        searches,
        result,
    };
    testkit::run(&mut w, more);
    (out, w.obj(id).unwrap().pos.cell())
}

// spec: S-0212 claim 13 (the 5th failed cell search inside a corridor fails the computation) and claim 23 row 2 (the unit gives up: its destination is cleared and, with a human-controlled owner, it queues guard); claim 1 (the coarse search sees through buildings)
#[test]
fn claim_13_cap_for_a_vehicle_on_a_docking_cell() {
    let x = dock_move(DOCK);
    assert_eq!(x.result, Some(Search::SearchFailed));
    assert_eq!(x.searches.len(), 5, "five cell searches: {:?}", x.searches);
    assert!(
        x.searches.iter().all(|s| s.corridor && s.found.is_none()),
        "all inside a corridor, none found: {:?}",
        x.searches
    );
    assert_eq!(x.cleared, Some(1), "the destination is cleared at once");
    assert_eq!(x.cell, dock_cell(DOCK), "the vehicle does not move");
    assert_eq!(x.behaviour, Some(Behaviour::Guard));
}

// spec: S-0212 claim 13 (a cell search without a corridor is confined by nothing but the search limits) and claim 1 (buildings leave cells open): the way round exists from the docking cell
#[test]
fn claim_13_the_way_west_exists_from_the_docking_cell() {
    let (w, id) = dock_layout(DOCK);
    let found = S0212.find(
        &w,
        &Request {
            unit: id,
            from: dock_cell(DOCK),
            to: dock_cell(WEST),
            mode: Mode::Fresh,
            congested: Default::default(),
        },
    );
    assert!(found.is_some(), "a path round the blocks exists");
}

// spec: S-0212 claim 13 (the first cell search inside the corridor finds the path): from south of the blocks the vehicle goes
#[test]
fn claim_13_from_south_of_the_blocks_the_first_search_finds_the_way() {
    let x = dock_move(SOUTH);
    assert_eq!(x.result, Some(Search::Found));
    assert_eq!(x.searches.len(), 1, "one cell search: {:?}", x.searches);
    assert!(x.searches[0].corridor && x.searches[0].found.is_some());
    assert_eq!(x.cleared, None, "the destination stays");
}

// The play-fixes layer (M8 step C7, data/layers/play-fixes.jsonc).

/// The test ruleset with only the two corridor switches as given (both
/// `false` is what the play-fixes layer gives).
fn with_switches(retry: bool, split: bool) -> Ruleset {
    let mut r = rules(retry);
    r.globals.quirks.corridor_split_probe = split;
    r
}

/// A path computation that the layer lets through: the cell searches inside
/// corridors fail, the new coarse search fails, and the one cell search
/// without a corridor finds the path, before the cap.
fn assert_found_by_the_fallback(x: &Outcome) {
    assert_eq!(x.result, Some(Search::Found), "searches: {:?}", x.searches);
    assert_eq!(x.cleared, None, "the destination stays");
    let (last, before) = x.searches.split_last().expect("a cell search");
    assert!(!last.corridor && last.found.is_some(), "{:?}", x.searches);
    assert!(
        before.iter().all(|s| s.corridor && s.found.is_none()),
        "{:?}",
        x.searches
    );
    assert!(x.searches.len() < 5, "{:?}", x.searches);
}

// spec: S-0212 claim 13 and Edge cases (quirks.corridor_retry_no_fallback off: after a failed repeated coarse search one cell search without a corridor, final) with claim 13a and Edge cases (quirks.corridor_split_probe off: split when some usable cell of the sector is not in the free part): with the play-fixes layer the vehicle on the docking cell finds its way west and leaves the cell (Q-1870, pathing report 2)
#[test]
fn play_fixes_the_vehicle_on_the_docking_cell_finds_its_way() {
    let (x, later) = dock_move_with(DOCK, &rules_with(Layers::PLAY_FIXES), 40);
    assert_found_by_the_fallback(&x);
    assert_ne!(
        later,
        dock_cell(DOCK),
        "the vehicle has left the docking cell"
    );
}

// spec: S-0212 claims 13 and 13a and Edge cases (both switches off, as the play-fixes layer sets them): the unit at the factory's exit finds its way round the row and walks (Q-1771, pathing report 1)
#[test]
fn play_fixes_the_unit_at_the_factory_exit_finds_its_way() {
    let (x, later) = order_move_with(&rules_with(Layers::PLAY_FIXES), 60);
    assert_found_by_the_fallback(&x);
    assert_ne!(later, at(UNIT), "the unit has left the exit");
}

// spec: S-0212 claim 13 and Edge cases (quirks.corridor_retry_no_fallback off alone: the cap of 5 still ends the computation) and claim 13a and Edge cases (quirks.corridor_split_probe off alone: the new coarse search fails and, with the other switch on, so does the computation): the layer needs both switches
#[test]
fn play_fixes_needs_both_switches() {
    for (retry, split) in [(false, true), (true, false)] {
        let r = with_switches(retry, split);
        let dock = dock_move_with(DOCK, &r, 0).0;
        let exit = order_move_with(&r, 0).0;
        for x in [&dock, &exit] {
            assert_eq!(
                x.result,
                Some(Search::SearchFailed),
                "retry {retry}, split {split}: {:?}",
                x.searches
            );
            assert_eq!(x.cleared, Some(1));
        }
    }
    // Both off, set by hand, is what the layer gives.
    let by_hand = dock_move_with(DOCK, &with_switches(false, false), 0).0;
    let layer = dock_move_with(DOCK, &rules_with(Layers::PLAY_FIXES), 0).0;
    assert_eq!(by_hand.searches, layer.searches);
}

// spec: S-0212 claim 13 (the first cell search inside the corridor finds the path): where the original finds the way at once the layer changes nothing
#[test]
fn play_fixes_change_nothing_where_the_first_search_finds_the_way() {
    let plain = dock_move(SOUTH);
    let fixed = dock_move_with(SOUTH, &rules_with(Layers::PLAY_FIXES), 0).0;
    assert_eq!(fixed.result, Some(Search::Found));
    assert_eq!(fixed.searches, plain.searches);
}
