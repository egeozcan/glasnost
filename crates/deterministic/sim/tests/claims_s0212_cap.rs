//! Claim tests for a unit routed through its own base (S-0212 claims 1 and
//! 13, Edge cases; Q-1771 and Q-1870 answered in r82), on synthetic data.
//!
//! The layout of scenario P31, our own buildings and types: a unit stands
//! at the exit just south of a 3 × 2 block of buildings (a factory), a row
//! of ten buildings lies 4 cells north of the block with open ground round
//! both ends, and the unit is sent 16 cells north and 3 west, behind the
//! row. The coarse search sees through buildings (claim 1), so the first
//! corridor runs into the block and the row; the cell search inside it is
//! not stopped by them, since every cell of each building's exemption
//! rectangle is exempt (claim 13). So the first cell search finds the way
//! round, at each of the 64 shifts of the layout against the sector grid,
//! for an infantry unit and a vehicle alike.
//!
//! The shape of the lead's pathing report 2 (M8 step A4), a vehicle on the
//! cell east of a 3 × 3 block (a refinery's docking cell) sent far west,
//! goes the same way. The last tests apply our own patch layer *Pathing and
//! other fixes* (`data/layers/play-fixes.jsonc`, M8 step C7): in these
//! shapes it changes nothing, since the first search finds the way.

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
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const BIG: i32 = 50;

/// The combat ruleset with BLOCK, a plain 1 × 1 building with no weapon.
fn rules() -> Ruleset {
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
    r
}

/// The combat ruleset with BLOCK and the quirk switches of `layers` applied
/// to the defaults (the file itself, through `Layers::apply_quirks`).
fn rules_with(layers: Layers) -> Ruleset {
    let mut r = rules();
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

/// The cells of P31's layout, as offsets from the map's middle before the
/// shift: the 3 × 2 block at (1..=3, 0..=1); a row of ten buildings 4 cells
/// north of the block's north row, from 5 cells west of its west column to
/// 2 cells east of its east column (y = −4, x = −4 to 5).
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

/// The unit, on the cell south of the block's middle building.
const UNIT: (i32, i32) = (2, 2);
/// Its destination: 16 cells north and 3 west of its own cell.
const GOAL: (i32, i32) = (-1, -14);

/// A unit of the layout: an infantry unit or a vehicle of movement zone
/// *normal*.
#[derive(Clone, Copy, Debug)]
enum Unit {
    Infantry,
    Vehicle,
}

/// Cell `(dx, dy)` of the layout shifted `e` cells east and `n` north.
fn at((dx, dy): (i32, i32), (e, n): (i32, i32)) -> glasnost_map::CellXY {
    testkit::mid(BIG, dx + e, dy - n)
}

/// The layout shifted by `shift` with `unit` of house 0 at the exit, the
/// trace on.
fn layout_with(r: &Ruleset, shift: (i32, i32), unit: Unit) -> (World, ObjectId) {
    let mut w = testkit::game_with(r, BIG, 7);
    testkit::run(&mut w, 2);
    for b in blocks() {
        testkit::spawn(&mut w, Kind::Building, "BLOCK", 0, at(b, shift), 0);
    }
    testkit::run(&mut w, 2);
    let (kind, name) = match unit {
        Unit::Infantry => (Kind::Infantry, "WALKER"),
        Unit::Vehicle => (Kind::Vehicle, "BUGGY"),
    };
    let id = testkit::spawn(&mut w, kind, name, 0, at(UNIT, shift), 0);
    testkit::run(&mut w, 1);
    while w.tick <= w.orders_blocked_through {
        testkit::run(&mut w, 1);
    }
    trace::switch_on(&mut w);
    (w, id)
}

/// What a person's move order gives: the tick (counted from the order) at
/// which the destination was cleared, the unit's cell then, its current
/// behaviour, and the cell searches of its first path computation.
struct Outcome {
    cleared: Option<i32>,
    cell: glasnost_map::CellXY,
    behaviour: Option<Behaviour>,
    searches: Vec<CellSearch>,
    result: Option<Search>,
}

/// Orders `id` to `to`, runs `ticks` ticks (stopping when the destination
/// is cleared) and gives the outcome.
fn order(w: &mut World, id: ObjectId, to: glasnost_map::CellXY, ticks: i32) -> Outcome {
    orders::issue(
        w,
        HouseId(0),
        Order::Behaviour {
            actor: id,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(to)),
        },
    );
    let mut cleared = None;
    for t in 0..ticks {
        tick::step(w);
        if w.obj(id).unwrap().dest.is_none() {
            cleared = Some(t);
            break;
        }
    }
    let (searches, result) = trace::take(w)
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

/// The move order of P31 under the rules `r`, the first 3 ticks.
fn exit_move(r: &Ruleset, shift: (i32, i32), unit: Unit) -> Outcome {
    let (mut w, id) = layout_with(r, shift, unit);
    order(&mut w, id, at(GOAL, shift), 3)
}

/// Whether the path computation found its path on its first cell search,
/// inside a corridor, and the destination stayed.
fn first_search_finds(x: &Outcome) -> bool {
    x.result == Some(Search::Found)
        && x.cleared.is_none()
        && x.searches.len() == 1
        && x.searches[0].corridor
        && x.searches[0].found.is_some()
}

// spec: S-0212 claim 13 (the corridor exemption: every cell of each building's exemption rectangle; the first cell search inside the first corridor finds the path, so no retry is made and the limit of 5 plays no part) and Edge cases (a unit routed through its own base, Q-1771 answered: at every shift and for an infantry unit and a vehicle, the unit keeps its destination and gets a route in the tick of the order), scenario P31
#[test]
fn claim_13_out_of_a_factory_exit_at_every_shift() {
    let r = rules();
    let mut fails = Vec::new();
    for unit in [Unit::Infantry, Unit::Vehicle] {
        for e in 0..8 {
            for n in 0..8 {
                let x = exit_move(&r, (e, n), unit);
                if !first_search_finds(&x) {
                    fails.push((unit, e, n, x.result, x.searches.len()));
                }
            }
        }
    }
    assert!(fails.is_empty(), "shifts not passed at once: {fails:?}");
}

// spec: S-0212 Edge cases (a unit routed through its own base, Q-1771 answered: it reaches the destination round one end of the row), scenario P31
#[test]
fn p31_the_unit_reaches_the_destination() {
    for unit in [Unit::Infantry, Unit::Vehicle] {
        let (mut w, id) = layout_with(&rules(), (0, 0), unit);
        let x = order(&mut w, id, at(GOAL, (0, 0)), 3);
        assert!(first_search_finds(&x), "{unit:?}: {:?}", x.searches);
        let mut arrived = false;
        for _ in 0..2000 {
            tick::step(&mut w);
            if w.obj(id).unwrap().pos.cell() == at(GOAL, (0, 0)) {
                arrived = true;
                break;
            }
        }
        assert!(arrived, "{unit:?} reaches the destination");
    }
}

// spec: S-0212 claim 13 (a cell search without a corridor is confined by nothing but the search limits) and claim 1 (buildings leave cells open): the way round exists
#[test]
fn claim_13_a_search_without_a_corridor_finds_the_way_round() {
    let (w, id) = layout_with(&rules(), (0, 0), Unit::Infantry);
    let found = S0212.find(
        &w,
        &Request {
            unit: id,
            from: at(UNIT, (0, 0)),
            to: at(GOAL, (0, 0)),
            mode: Mode::Fresh,
            congested: Default::default(),
        },
    );
    assert!(found.is_some(), "a path round the row exists");
}

// The shape of the lead's pathing report 2 (M8 step A4), our own layout: a
// vehicle stands on the cell east of a 3 × 3 block of buildings (a
// refinery's docking cell), two more blocks lie west of it, and it is sent
// to a cell far west of all three.

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
fn dock_layout_with(from: (i32, i32), r: &Ruleset) -> (World, ObjectId) {
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

/// The move west from `from` under the rules `r`; then `more` further
/// ticks, after which the vehicle's cell is given too.
fn dock_move_with(from: (i32, i32), r: &Ruleset, more: i32) -> (Outcome, glasnost_map::CellXY) {
    let (mut w, id) = dock_layout_with(from, r);
    let out = order(&mut w, id, dock_cell(WEST), 3);
    testkit::run(&mut w, more);
    (out, w.obj(id).unwrap().pos.cell())
}

// spec: S-0212 claim 13 (the corridor exemption includes the docking block's exemption rectangle and those of the blocks beyond it: the first cell search inside the first corridor finds the path) and Edge cases (a harvester leaving a refinery's docking cell across its base, Q-1870 answered: no rule depends on the harvester flag, so a vehicle on the docking cell goes alike)
#[test]
fn claim_13_a_vehicle_on_a_docking_cell_walks_out() {
    let (x, later) = dock_move_with(DOCK, &rules(), 40);
    assert!(first_search_finds(&x), "{:?}", x.searches);
    assert_ne!(
        later,
        dock_cell(DOCK),
        "the vehicle has left the docking cell"
    );
    assert_ne!(x.behaviour, Some(Behaviour::Guard));
}

// spec: S-0212 claim 13 (a cell search without a corridor is confined by nothing but the search limits) and claim 1 (buildings leave cells open): the way round exists from the docking cell
#[test]
fn claim_13_the_way_west_exists_from_the_docking_cell() {
    let (w, id) = dock_layout_with(DOCK, &rules());
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
    let x = dock_move_with(SOUTH, &rules(), 0).0;
    assert!(first_search_finds(&x), "{:?}", x.searches);
    assert_eq!(x.cell, dock_cell(SOUTH));
}

// The play-fixes layer (M8 step C7, data/layers/play-fixes.jsonc).

// spec: S-0212 claim 13 (the first cell search inside the corridor finds the path, so no retry is made and no switch of the retry table applies) and Edge cases (Q-1771 and Q-1870 answered: no switch is needed for these cases): with the play-fixes layer the computations of both reports' shapes are the original's
#[test]
fn play_fixes_change_nothing_in_the_reports_shapes() {
    let layer = rules_with(Layers::PLAY_FIXES);
    for from in [DOCK, SOUTH] {
        let plain = dock_move_with(from, &rules(), 0).0;
        let fixed = dock_move_with(from, &layer, 0).0;
        assert!(first_search_finds(&fixed), "{:?}", fixed.searches);
        assert_eq!(fixed.searches, plain.searches);
    }
    for unit in [Unit::Infantry, Unit::Vehicle] {
        for shift in [(0, 0), (3, 5), (7, 7)] {
            let plain = exit_move(&rules(), shift, unit);
            let fixed = exit_move(&layer, shift, unit);
            assert!(first_search_finds(&fixed), "{unit:?} {shift:?}");
            assert_eq!(fixed.searches, plain.searches);
        }
    }
}
