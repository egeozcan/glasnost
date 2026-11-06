//! Claim tests for pathfinding (S-0212), on synthetic data.
//!
//! Invented types (TESTCAR, TESTHUT, TESTTREE, ...) on the flat 40 × 40 map
//! of `common`. Rock cells use a tile whose land type has no land row, so
//! every locomotion class has multiplier 0 there (category *blocked*).
//!
//! Not tested here because the simulation does not implement them yet:
//! follower cars (claim 26), tunnels and their links and steps, and the
//! changes of bridge links in play (claim 3's table: no bridge is destroyed
//! or repaired before the bridge specs).

mod common;

use common::unit_type;
use common::{Scene, WHEEL, cell, code_tile, objects_of, open_orders, order_move, slope_tile};
use glasnost_map::CellXY;
use glasnost_rules::sim::{
    BuildingType, Decimal, LandRow, LandType, MovementZone, TerrainObjectType, UnitType,
};
use glasnost_sim::occupancy::{self, Outcome};
use glasnost_sim::passability::category;
use glasnost_sim::path::{self, Mode, PathStep, Pathfinder, Request, S0212, Search};
use glasnost_sim::regions::{self, Reach, STEPS};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{ObjectId, Target, World};

const CAR: &str = "TESTCAR";
const HUT: &str = "TESTHUT";
const TREE: &str = "TESTTREE";

/// A scene with the vehicle type TESTCAR (wheel, invented speed 40, turn
/// rate 6, no weapon) and the one-cell building type TESTHUT.
fn scene() -> Scene {
    let mut s = Scene::new();
    s.vehicle(CAR, unit_type(WHEEL, 40, 6));
    s.register("buildings", HUT);
    s.ruleset.sim.buildings.insert(
        HUT.into(),
        BuildingType {
            strength: Some(200),
            ..Default::default()
        },
    );
    s
}

/// Makes `c` a flat rock cell at height level `level` (land type rock, no
/// land row: multiplier 0 for every class).
fn rock(s: &mut Scene, c: CellXY, level: i8) {
    s.set_cell(c, code_tile(7), level);
}

/// The cells at Chebyshev distance `r` from `centre`.
fn ring(centre: CellXY, r: i32) -> Vec<CellXY> {
    let mut v = Vec::new();
    for dy in -r..=r {
        for dx in -r..=r {
            if dx.abs().max(dy.abs()) == r {
                v.push(CellXY::new(centre.x + dx, centre.y + dy));
            }
        }
    }
    v
}

/// All cells of the map grid.
fn all_cells(s: &Scene) -> Vec<CellXY> {
    s.map.cells.iter().map(|(c, _)| c).collect()
}

fn vehicles(w: &World) -> Vec<ObjectId> {
    objects_of(w, Kind::Vehicle)
}

/// Runs the S-0212 cell search for unit `id` from its own cell.
fn find(w: &World, id: ObjectId, to: CellXY, mode: Mode, marks: &[CellXY]) -> Option<Vec<u8>> {
    let from = w.obj(id).unwrap().pos.cell();
    S0212.find(
        w,
        &Request {
            unit: id,
            from,
            to,
            mode,
            congested: marks.iter().copied().collect(),
            avoided: Default::default(),
        },
    )
}

/// The cells a path visits after its start, in order.
fn visits(from: CellXY, steps: &[u8]) -> Vec<CellXY> {
    let mut c = from;
    steps
        .iter()
        .map(|&d| {
            let (dx, dy) = STEPS[usize::from(d)];
            c = CellXY::new(c.x + dx, c.y + dy);
            c
        })
        .collect()
}

fn region(w: &World, c: CellXY) -> u32 {
    w.regions.region[w.cells.idx(c).unwrap()]
}

/// Gives unit `id` the destination cell `c` and runs a path computation.
fn compute_to(w: &mut World, id: ObjectId, c: CellXY, mode: Mode) -> Search {
    w.obj_mut(id).unwrap().dest = Some(Target::Cell(c));
    path::compute(w, id, mode)
}

/// Makes unit `id` count as moving while it stays where it is: it has a
/// far destination, no stored path and a retry delay that never ends.
fn hold_as_moving(w: &mut World, id: ObjectId) {
    let o = w.obj_mut(id).unwrap();
    o.dest = Some(Target::Cell(cell(-20, 10)));
    o.path.steps.clear();
    o.path.retry_at = i32::MAX;
}

fn chebyshev(a: CellXY, b: CellXY) -> i32 {
    (a.x - b.x).abs().max((a.y - b.y).abs())
}

// ---------------------------------------------------------------- connectivity

// spec: S-0212 claim 1 (buildings and units leave cells open, so a ring of buildings separates nothing)
#[test]
fn ring_of_buildings_does_not_split_regions() {
    let mut s = scene();
    for c in ring(cell(0, 0), 1) {
        s.place_structure(0, HUT, c);
    }
    s.place_unit(0, CAR, cell(0, 0), 64);
    let w = s.build();
    assert_eq!(objects_of(&w, Kind::Building).len(), 8);
    for c in ring(cell(0, 0), 1).into_iter().chain([cell(0, 0)]) {
        assert_eq!(w.cells.category[w.cells.idx(c).unwrap()], category::OPEN);
    }
    let inside = regions::reach_in(&w, cell(0, 0), MovementZone::Normal);
    assert!(matches!(inside, Reach::Region(_)));
    assert_eq!(
        inside,
        regions::reach_in(&w, cell(6, 3), MovementZone::Normal)
    );
}

// spec: S-0212 claim 2 (no and never both keep cells out of the reach regions; off-map cells form no region)
#[test]
fn no_and_never_keep_cells_out_of_reach_regions() {
    let mut s = scene();
    // Shrink the usable area so that the grid's edge cells are off-map.
    let fp = glasnost_map::Footprint::new(
        common::SIZE,
        common::SIZE,
        [4, 4, common::SIZE - 8, common::SIZE - 8],
    );
    let mut grid = glasnost_map::CellGrid::new(fp);
    for c in all_cells(&s) {
        if let Some(x) = grid.get_mut(c) {
            *x = *s.map.cells.get(c).unwrap();
        }
    }
    s.map.footprint = fp;
    s.map.cells = grid;
    s.set_cell(cell(3, 0), code_tile(9), 0); // water
    s.place_unit(0, CAR, cell(0, 0), 64);
    let mut w = s.build();
    let id = vehicles(&w)[0];
    let off = all_cells(&s)
        .into_iter()
        .find(|&c| !w.cells.usable(c))
        .expect("an off-map cell");
    assert_eq!(
        w.cells.category[w.cells.idx(off).unwrap()],
        category::OFF_MAP
    );
    assert_eq!(region(&w, off), u32::MAX);
    let z = MovementZone::Normal;
    assert_eq!(regions::reach_in(&w, off, z), Reach::OffMap);
    assert_eq!(regions::reach_in(&w, cell(3, 0), z), Reach::None);
    assert_ne!(Reach::OffMap, Reach::None);
    // Both make a destination unreachable for the gate.
    assert_eq!(
        compute_to(&mut w, id, cell(3, 0), Mode::Fresh),
        Search::GateFailed
    );
    assert_eq!(compute_to(&mut w, id, off, Mode::Fresh), Search::GateFailed);
}

/// A scene whose cells at `pick` are raised to `level`.
fn raised(pick: impl Fn(CellXY) -> bool, level: i8) -> World {
    let mut s = scene();
    for c in all_cells(&s) {
        if pick(c) {
            s.set_cell(c, 0, level);
        }
    }
    s.build()
}

// spec: S-0212 claim 3 (same category joins within 1 level; along cell x also at 2 or 3 levels; otherwise never)
#[test]
fn regions_join_by_height_and_along_cell_x() {
    let row = |c: CellXY| c.y == cell(0, 1).y; // neighbours across it lie along cell y
    let col = |c: CellXY| c.x == cell(1, 0).x; // neighbours across it lie along cell x
    // A raised row: joined to the ground only within 1 level.
    let w = raised(row, 1);
    assert_eq!(region(&w, cell(0, 1)), region(&w, cell(0, 0)));
    let w = raised(row, 2);
    assert_ne!(region(&w, cell(0, 1)), region(&w, cell(0, 0)));
    assert_eq!(region(&w, cell(0, 1)), region(&w, cell(5, 1)));
    // A raised column: joined along cell x up to 3 levels, not at 4.
    for level in [2, 3] {
        let w = raised(col, level);
        assert_eq!(
            region(&w, cell(1, 0)),
            region(&w, cell(0, 0)),
            "level {level}"
        );
    }
    let w = raised(col, 4);
    assert_ne!(region(&w, cell(1, 0)), region(&w, cell(0, 0)));
}

// spec: S-0212 claim 3 (a region of category blocked is adjacent to its neighbours whatever the heights)
#[test]
fn blocked_region_adjacent_across_any_height() {
    let island = |code: u8, level: i8| {
        let mut s = scene();
        for c in ring(cell(0, 0), 2) {
            s.set_cell(c, code_tile(code), level);
        }
        s.build()
    };
    // Rock ring three levels up: subterranean (open and blocked) still joins
    // inside and outside; normal does not.
    let w = island(7, 3);
    let sub = MovementZone::Subterranean;
    assert_eq!(
        regions::reach_in(&w, cell(0, 0), sub),
        regions::reach_in(&w, cell(5, 0), sub)
    );
    assert_ne!(
        regions::reach_in(&w, cell(0, 0), MovementZone::Normal),
        regions::reach_in(&w, cell(5, 0), MovementZone::Normal)
    );
    // A water ring three levels up is not adjacent to the open ground for
    // amphibious; at level 0 it is.
    let amph = MovementZone::Amphibious;
    let w = island(9, 3);
    assert_ne!(
        regions::reach_in(&w, cell(0, 0), amph),
        regions::reach_in(&w, cell(5, 0), amph)
    );
    let w = island(9, 0);
    assert_eq!(
        regions::reach_in(&w, cell(0, 0), amph),
        regions::reach_in(&w, cell(5, 0), amph)
    );
}

// spec: S-0212 claim 4 (reach regions per movement zone; cells of a no-answer region lie in no reach region)
#[test]
fn reach_regions_differ_by_movement_zone() {
    let mut s = scene();
    for c in ring(cell(0, 0), 2) {
        s.set_cell(c, code_tile(9), 0); // water
    }
    let w = s.build();
    let (inside, outside, water, water2) = (cell(0, 0), cell(5, 0), cell(2, 0), cell(-2, 1));
    let n = MovementZone::Normal;
    assert_ne!(
        regions::reach_in(&w, inside, n),
        regions::reach_in(&w, outside, n)
    );
    assert_eq!(regions::reach_in(&w, water, n), Reach::None);
    // Two cells of no reach region compare as the same.
    assert_eq!(
        regions::reach_in(&w, water, n),
        regions::reach_in(&w, water2, n)
    );
    // The same regions joined into one reach region for amphibious.
    let a = MovementZone::Amphibious;
    let r = regions::reach_in(&w, inside, a);
    assert!(matches!(r, Reach::Region(_)));
    assert_eq!(r, regions::reach_in(&w, outside, a));
    assert_eq!(r, regions::reach_in(&w, water, a));
}

// spec: S-0212 claim 5 (a destroyed terrain object updates connectivity for a path computation later in the same tick)
#[test]
fn destroyed_terrain_objects_reconnect_at_once() {
    let mut s = scene();
    s.register("terrain_objects", TREE);
    s.ruleset
        .sim
        .terrain_objects
        .insert(TREE.into(), TerrainObjectType::default());
    for c in ring(cell(0, 0), 1) {
        s.place_terrain(TREE, c);
    }
    s.place_unit(0, CAR, cell(0, 0), 64);
    let mut w = s.build();
    let id = vehicles(&w)[0];
    assert_eq!(
        compute_to(&mut w, id, cell(5, 0), Mode::Fresh),
        Search::GateFailed
    );
    for t in objects_of(&w, Kind::Terrain) {
        glasnost_sim::objects::remove(&mut w, t);
    }
    // No tick in between.
    assert_eq!(
        compute_to(&mut w, id, cell(5, 0), Mode::Fresh),
        Search::Found
    );
}

// spec: S-0212 claim 5 (a destroyed building changes no category, so connectivity stays as it was)
#[test]
fn destroyed_building_changes_no_category() {
    let mut s = scene();
    s.place_structure(0, HUT, cell(2, 0));
    let mut w = s.build();
    let i = w.cells.idx(cell(2, 0)).unwrap();
    let before = w.regions.region.clone();
    assert_eq!(w.cells.category[i], category::OPEN);
    let b = objects_of(&w, Kind::Building)[0];
    glasnost_sim::objects::remove(&mut w, b);
    assert_eq!(w.cells.category[i], category::OPEN);
    assert!(!w.regions.dirty);
    assert_eq!(w.regions.region, before);
}

// ------------------------------------------------------------------- the gate

// spec: S-0212 claim 6 (no search across reach regions; a destination failing the gate is never replaced)
#[test]
fn gate_refuses_destination_in_another_reach_region() {
    let mut s = scene();
    for c in ring(cell(0, 0), 2) {
        rock(&mut s, c, 0);
    }
    s.place_unit(0, CAR, cell(6, 0), 64);
    // An allied parked vehicle on the destination would start substitution
    // if the gate passed.
    s.place_unit(0, CAR, cell(0, 0), 64);
    let mut w = s.build();
    let id = vehicles(&w)
        .into_iter()
        .find(|&v| w.obj(v).unwrap().pos.cell() == cell(6, 0))
        .unwrap();
    assert_eq!(
        compute_to(&mut w, id, cell(0, 0), Mode::Fresh),
        Search::GateFailed
    );
    let o = w.obj(id).unwrap();
    assert!(o.path.steps.is_empty());
    assert_eq!(o.path.for_goal, None);
    // The same destination passes when the ring has a gap.
    let mut s2 = scene();
    for c in ring(cell(0, 0), 2) {
        if c != cell(2, 0) {
            rock(&mut s2, c, 0);
        }
    }
    s2.place_unit(0, CAR, cell(6, 0), 64);
    let mut w2 = s2.build();
    let id2 = vehicles(&w2)[0];
    assert_eq!(
        compute_to(&mut w2, id2, cell(0, 0), Mode::Fresh),
        Search::Found
    );
}

// ----------------------------------------------------------------- the search

// spec: S-0212 claim 7 (open ground searched without a corridor: all diagonal steps first, then straight ones; Chebyshev distance steps)
#[test]
fn open_ground_path_is_diagonal_first() {
    let mut s = scene();
    s.place_unit(0, CAR, cell(0, 0), 64);
    let mut w = s.build();
    let id = vehicles(&w)[0];
    // In retreat the unit is in the corridor-disabling state (claim 14), so
    // its path computation searches without a corridor.
    w.obj_mut(id).unwrap().beh.current = Some(Behaviour::Retreat);
    let from = cell(0, 0);
    for (dx, dy) in [
        (5, 2),
        (-3, 7),
        (14, -9),
        (0, 6),
        (-8, 0),
        (-14, -14),
        (2, -13),
    ] {
        let to = cell(dx, dy);
        let steps = find(&w, id, to, Mode::Fresh, &[]).expect("a path");
        assert_eq!(steps.len() as i32, chebyshev(from, to), "to {dx},{dy}");
        assert_eq!(*visits(from, &steps).last().unwrap(), to);
        let diag = steps.iter().take_while(|&&d| d % 2 == 1).count();
        assert_eq!(diag as i32, dx.abs().min(dy.abs()), "to {dx},{dy}");
        assert!(steps[diag..].iter().all(|&d| d % 2 == 0));
        // Neither straightening pass changes it.
        assert_eq!(compute_to(&mut w, id, to, Mode::Fresh), Search::Found);
        assert_eq!(w.obj(id).unwrap().path.steps, steps);
    }
}

// spec: S-0212 claim 7 (candidates whose entry outcome is blocked are not considered)
#[test]
fn path_goes_around_blocked_cells() {
    let mut s = scene();
    for dy in -3..=3 {
        rock(&mut s, cell(2, dy), 0);
    }
    s.place_unit(0, CAR, cell(0, 0), 64);
    let w = s.build();
    let id = vehicles(&w)[0];
    let steps = find(&w, id, cell(4, 0), Mode::Fresh, &[]).expect("a path");
    let cells = visits(cell(0, 0), &steps);
    assert_eq!(*cells.last().unwrap(), cell(4, 0));
    assert!(
        cells
            .iter()
            .all(|c| c.x != cell(2, 0).x || (c.y - cell(0, 0).y).abs() > 3)
    );
}

// spec: S-0212 claim 8 (a stationary friendly costs 8, more than a one-step detour)
#[test]
fn stationary_friendly_is_avoided_when_a_detour_is_cheaper() {
    let mut s = scene();
    s.place_unit(0, CAR, cell(0, 0), 64);
    s.place_unit(0, CAR, cell(2, 0), 64);
    let w = s.build();
    let id = vehicles(&w)
        .into_iter()
        .find(|&v| w.obj(v).unwrap().pos.cell() == cell(0, 0))
        .unwrap();
    let r = occupancy::reference_level(&w, id);
    assert_eq!(
        path::step_outcome(&w, id, cell(1, 0), cell(2, 0), r),
        Outcome::StationaryFriendly
    );
    let steps = find(&w, id, cell(4, 0), Mode::Fresh, &[]).expect("a path");
    assert!(!visits(cell(0, 0), &steps).contains(&cell(2, 0)));
    assert_eq!(steps.len(), 4);
}

// spec: S-0212 claim 8 (for a train every outcome other than blocked costs 1)
#[test]
fn train_steps_cost_one_whatever_the_outcome() {
    let mut s = scene();
    s.vehicle(
        "TESTTRAIN",
        UnitType {
            is_train: Some(true),
            ..unit_type(WHEEL, 40, 6)
        },
    );
    s.place_unit(0, "TESTTRAIN", cell(0, 0), 64);
    s.place_unit(0, CAR, cell(2, 0), 64);
    let w = s.build();
    let id = vehicles(&w)
        .into_iter()
        .find(|&v| w.obj(v).unwrap().pos.cell() == cell(0, 0))
        .unwrap();
    assert_eq!(
        find(&w, id, cell(4, 0), Mode::Fresh, &[]),
        Some(vec![2, 2, 2, 2])
    );
}

// spec: S-0212 claim 8 (land multipliers play no part in the step cost)
#[test]
fn slow_land_costs_the_same_as_clear() {
    let base = || {
        let mut s = scene();
        s.place_unit(0, CAR, cell(0, 0), 64);
        s
    };
    let w = base().build();
    let id = vehicles(&w)[0];
    let plain = find(&w, id, cell(6, 3), Mode::Fresh, &[]).unwrap();
    // Every cell of that path, and its whole neighbourhood, made slow land.
    let mut s = base();
    s.ruleset.sim.land_types.insert(
        LandType::Rough,
        LandRow {
            wheel: Some(Decimal("0.25".into())),
            ..Default::default()
        },
    );
    for c in visits(cell(0, 0), &plain) {
        s.set_cell(c, code_tile(14), 0); // rough
    }
    let w = s.build();
    let id = vehicles(&w)[0];
    assert_eq!(find(&w, id, cell(6, 3), Mode::Fresh, &[]).unwrap(), plain);
}

/// A unit at cell(0, 0) facing east, a rock wall across cell x + 2 from
/// cell y − `half` to y + `half` with a gap at cell(2, 0), and an allied
/// unit of the same type held in the gap as a moving unit with no path
/// when `blocker` is set.
fn gap_scene(half: i32, blocker: bool) -> (World, ObjectId, Option<ObjectId>) {
    let mut s = scene();
    for dy in -half..=half {
        if dy != 0 {
            rock(&mut s, cell(2, dy), 0);
        }
    }
    s.place_unit(0, CAR, cell(0, 0), 64);
    if blocker {
        s.place_unit(0, CAR, cell(2, 0), 64);
    }
    let mut w = s.build();
    let at = |w: &World, c: CellXY| {
        vehicles(w)
            .into_iter()
            .find(|&v| w.obj(v).unwrap().pos.cell() == c)
    };
    let id = at(&w, cell(0, 0)).unwrap();
    let b = at(&w, cell(2, 0));
    if let Some(b) = b {
        hold_as_moving(&mut w, b);
    }
    (w, id, b)
}

// spec: S-0212 claim 8 (moving blocker: mode 2 costs 1000, mode 1 costs 4, mode 0 follows the blocker chain)
#[test]
fn moving_blocker_cost_by_search_mode() {
    // The detour around the wall is 6 steps longer than the way through the gap.
    let (w, id, _) = gap_scene(4, true);
    let r = occupancy::reference_level(&w, id);
    assert_eq!(
        path::step_outcome(&w, id, cell(1, 0), cell(2, 0), r),
        Outcome::MovingBlocker
    );
    let through =
        |p: &Option<Vec<u8>>| visits(cell(0, 0), p.as_ref().unwrap()).contains(&cell(2, 0));
    // Mode 0: the blocker heads nowhere, so the chain ends as an empty cell (1).
    let p0 = find(&w, id, cell(4, 0), Mode::Fresh, &[]);
    assert_eq!(p0, Some(vec![2, 2, 2, 2]));
    // Mode 1: 4, still cheaper than the detour.
    let p1 = find(&w, id, cell(4, 0), Mode::Blocked, &[]);
    assert!(through(&p1), "mode 1 path {p1:?}");
    // Mode 2: 1000, the detour is taken.
    let p2 = find(&w, id, cell(4, 0), Mode::Avoid, &[]);
    assert!(!through(&p2), "mode 2 path {p2:?}");
    // Without the wall, mode 1's cost 4 already loses to a one-step detour.
    let mut s = scene();
    s.place_unit(0, CAR, cell(0, 0), 64);
    s.place_unit(0, CAR, cell(2, 0), 64);
    let mut w = s.build();
    let vs = vehicles(&w);
    let (id, b) = if w.obj(vs[0]).unwrap().pos.cell() == cell(0, 0) {
        (vs[0], vs[1])
    } else {
        (vs[1], vs[0])
    };
    hold_as_moving(&mut w, b);
    let p1 = find(&w, id, cell(4, 0), Mode::Blocked, &[]);
    assert!(!through(&p1), "open mode 1 path {p1:?}");
}

// spec: S-0212 claim 9 (a congestion mark makes a step cost four times its base cost)
#[test]
fn congestion_mark_costs_four_times() {
    // Open ground: a marked cell (4) loses to a one-step detour.
    let (w, id, _) = gap_scene(0, false);
    assert_eq!(
        find(&w, id, cell(4, 0), Mode::Fresh, &[]),
        Some(vec![2, 2, 2, 2])
    );
    let p = find(&w, id, cell(4, 0), Mode::Fresh, &[cell(2, 0)]).unwrap();
    assert!(!visits(cell(0, 0), &p).contains(&cell(2, 0)), "{p:?}");
    // A marked gap (4) still beats a detour 6 steps longer.
    let (w, id, _) = gap_scene(4, false);
    let p = find(&w, id, cell(4, 0), Mode::Fresh, &[cell(2, 0)]).unwrap();
    assert!(visits(cell(0, 0), &p).contains(&cell(2, 0)), "{p:?}");
}

// spec: S-0212 claim 10 (a one-level step needs a slope as the lower cell; two or more levels are blocked)
#[test]
fn height_rule_for_a_step() {
    let mut s = scene();
    // Row y + 0: flat 0 -> flat 1.
    s.set_cell(cell(1, 0), 0, 1);
    // Row y + 2: slope 0 -> flat 1.
    s.set_cell(cell(0, 2), slope_tile(1), 0);
    s.set_cell(cell(1, 2), 0, 1);
    // Row y + 4: flat 0 -> slope 1 (the slope is the higher cell).
    s.set_cell(cell(1, 4), slope_tile(1), 1);
    // Row y + 6: slope 0 -> flat 2.
    s.set_cell(cell(0, 6), slope_tile(1), 0);
    s.set_cell(cell(1, 6), 0, 2);
    s.place_unit(0, CAR, cell(-3, 0), 64);
    let w = s.build();
    assert_ne!(w.cells.cell(cell(0, 2)).unwrap().slope, 0);
    let id = vehicles(&w)[0];
    let r = occupancy::reference_level(&w, id);
    let out = |a: CellXY, b: CellXY| path::step_outcome(&w, id, a, b, r);
    assert_eq!(out(cell(0, 0), cell(1, 0)), Outcome::Blocked);
    assert_eq!(out(cell(0, 2), cell(1, 2)), Outcome::Clear);
    // Whichever cell the step starts from.
    assert_eq!(out(cell(1, 2), cell(0, 2)), Outcome::Clear);
    assert_eq!(out(cell(0, 4), cell(1, 4)), Outcome::Blocked);
    assert_eq!(out(cell(1, 4), cell(0, 4)), Outcome::Blocked);
    assert_eq!(out(cell(0, 6), cell(1, 6)), Outcome::Blocked);
}

/// The cost of a path in thousandths of a cost unit by F1 on clear cells:
/// 1000 × τ plus the direction term of table T4.
fn clear_cost_milli(from: CellXY, steps: &[u8], marks: &[CellXY]) -> i64 {
    // spec: S-0212 table T4 (direction terms 0.001 … 0.008 by step direction)
    const EPS: [i64; 8] = [1, 5, 2, 6, 3, 7, 4, 8];
    visits(from, steps)
        .iter()
        .zip(steps)
        .map(|(c, &d)| {
            // spec: S-0212 F1 (τ = 4 on a congestion mark)
            let tau = if marks.contains(c) { 4 } else { 1 };
            1000 * tau + EPS[usize::from(d)]
        })
        .sum()
}

// spec: S-0212 claim 11 (a recorded pair keeps the cost of its first recording, so the path found need not be the cheapest)
#[test]
fn recorded_pair_is_not_reopened() {
    let mk = |no_reopen: bool| {
        let mut s = scene();
        s.ruleset.globals.quirks.path_no_reopen = no_reopen;
        s.place_unit(0, CAR, cell(0, 0), 64);
        s.build()
    };
    let (on, off) = (mk(true), mk(false));
    let id = vehicles(&on)[0];
    let (to, marks) = (cell(6, -2), [cell(4, -2)]);
    let a = find(&on, id, to, Mode::Fresh, &marks).unwrap();
    let b = find(&off, id, to, Mode::Fresh, &marks).unwrap();
    for p in [&a, &b] {
        let v = visits(cell(0, 0), p);
        assert_eq!(p.len(), 6);
        assert_eq!(*v.last().unwrap(), to);
        assert!(!v.contains(&marks[0]));
    }
    // A cheaper route to a cell of the path was found later and ignored.
    let (ca, cb) = (
        clear_cost_milli(cell(0, 0), &a, &marks),
        clear_cost_milli(cell(0, 0), &b, &marks),
    );
    assert!(
        ca > cb,
        "no-reopen {a:?} costs {ca}, reopen {b:?} costs {cb}"
    );
}

// spec: S-0212 claim 12 (a found path has at least one step; a search with no route fails)
#[test]
fn search_needs_a_step_and_fails_without_a_route() {
    let mut s = scene();
    for c in ring(cell(5, 0), 1) {
        rock(&mut s, c, 0);
    }
    s.place_unit(0, CAR, cell(0, 0), 64);
    let w = s.build();
    let id = vehicles(&w)[0];
    assert_eq!(find(&w, id, cell(0, 0), Mode::Fresh, &[]), None);
    assert_eq!(find(&w, id, cell(5, 0), Mode::Fresh, &[]), None);
}

// spec: S-0212 claim 15 (a blocked destination is approached to a neighbour, not for passive vehicles)
#[test]
fn blocked_destination_is_approached() {
    let mut s = scene();
    s.vehicle(
        "TESTSHY",
        UnitType {
            passive: Some(true),
            ..unit_type(WHEEL, 40, 6)
        },
    );
    s.place_structure(0, HUT, cell(4, 0));
    s.place_unit(0, CAR, cell(0, 0), 64);
    s.place_unit(0, "TESTSHY", cell(0, 3), 64);
    let w = s.build();
    let at = |c: CellXY| {
        vehicles(&w)
            .into_iter()
            .find(|&v| w.obj(v).unwrap().pos.cell() == c)
            .unwrap()
    };
    let (car, shy) = (at(cell(0, 0)), at(cell(0, 3)));
    let r = occupancy::reference_level(&w, car);
    assert_eq!(
        path::step_outcome(&w, car, cell(5, 0), cell(4, 0), r),
        Outcome::Blocked
    );
    let p = find(&w, car, cell(4, 0), Mode::Fresh, &[]).expect("approach path");
    let end = *visits(cell(0, 0), &p).last().unwrap();
    assert_eq!(chebyshev(end, cell(4, 0)), 1);
    // A passive vehicle does not approach.
    assert_eq!(find(&w, shy, cell(4, 0), Mode::Fresh, &[]), None);
}

// spec: S-0212 claim 16 (a train never enters the three cells behind it during the search)
#[test]
fn train_search_excludes_cells_behind() {
    let mut s = scene();
    s.vehicle(
        "TESTTRAIN",
        UnitType {
            is_train: Some(true),
            ..unit_type(WHEEL, 40, 6)
        },
    );
    s.place_unit(0, "TESTTRAIN", cell(0, 0), 64); // facing east
    s.place_unit(0, CAR, cell(0, 6), 64);
    let w = s.build();
    let at = |c: CellXY| {
        vehicles(&w)
            .into_iter()
            .find(|&v| w.obj(v).unwrap().pos.cell() == c)
            .unwrap()
    };
    let train = at(cell(0, 0));
    let f8 = glasnost_num::geom::eight_direction(w.obj(train).unwrap().body.value(w.tick));
    assert_eq!(f8, 2);
    let behind = [cell(-1, 1), cell(-1, 0), cell(-1, -1)];
    for c in behind {
        assert_eq!(find(&w, train, c, Mode::Fresh, &[]), None, "{c:?}");
    }
    let p = find(&w, train, cell(-4, 0), Mode::Fresh, &[]).expect("a path");
    let d = i32::from(p[0]);
    assert!(!matches!((d - 2).rem_euclid(8), 3..=5), "first step {d}");
    assert!(visits(cell(0, 0), &p).iter().all(|c| !behind.contains(c)));
    // A car facing the same way reverses at once.
    let car = at(cell(0, 6));
    assert_eq!(
        find(&w, car, cell(-4, 6), Mode::Fresh, &[]),
        Some(vec![6, 6, 6, 6])
    );
}

// ------------------------------------------------------------ early detours

fn dirs(v: &[u8]) -> Vec<PathStep> {
    v.iter().map(|&d| PathStep::Dir(d)).collect()
}

/// Claim 18 on a path from cell(0, 0); `bad` cells are not acceptable.
fn shorten(steps: &[u8], mirrored: bool, bad: &[CellXY]) -> Vec<PathStep> {
    let ok =
        |s: CellXY, shape: &[u8], _limit: u32| visits(s, shape).iter().all(|c| !bad.contains(c));
    path::shorten(&dirs(steps), cell(0, 0), mirrored, true, &ok)
}

// spec: S-0212 claim 18 (worked example a: 1, 3, 4, 5, 4, 4 with (x + 1, y) not clear gives 3, 4, 4, 4; quirks.detour_mirrored_reference off gives 3, 2, 5, 4, 4)
#[test]
fn detour_example_a() {
    let bad = [cell(1, 0)];
    assert_eq!(
        shorten(&[1, 3, 4, 5, 4, 4], true, &bad),
        dirs(&[3, 4, 4, 4])
    );
    assert_eq!(
        shorten(&[1, 3, 4, 5, 4, 4], false, &bad),
        dirs(&[3, 2, 5, 4, 4])
    );
}

// spec: S-0212 claim 18 (worked example b: 2, 4, 4, 5, 4, 4, 4 gives 3, 4, 5, 4, 4, 4)
#[test]
fn detour_example_b() {
    assert_eq!(
        shorten(&[2, 4, 4, 5, 4, 4, 4], true, &[]),
        dirs(&[3, 4, 5, 4, 4, 4])
    );
}

// spec: S-0212 claim 18 (only the first 20 places are examined; removed steps keep their places)
#[test]
fn detour_examines_twenty_places() {
    // A detour at places 19 and 20 (south, then west) is shortened...
    let mut p = vec![2; 18];
    p.extend([4, 6]);
    let mut want = vec![2; 16];
    want.push(3);
    assert_eq!(shorten(&p, true, &[]), dirs(&want));
    // ... the same detour at places 21 and 22 is not examined.
    let mut p = vec![2; 20];
    p.extend([4, 6]);
    assert_eq!(shorten(&p, true, &[]), dirs(&p));
    // A shape that is not acceptable leaves the stretch as it is.
    assert_eq!(
        shorten(&[2, 4, 4, 5, 4, 4, 4], true, &[cell(1, 1)]),
        dirs(&[2, 4, 4, 5, 4, 4, 4])
    );
}

// spec: S-0212 claim 18 (after a tunnel step the pass starts afresh from the cell north of the entrance, quirks.detour_after_tunnel_position; off: from the exit)
#[test]
fn detour_after_a_tunnel_step() {
    let tunnel = PathStep::Tunnel {
        entrance: cell(0, 0),
        exit: cell(10, 10),
    };
    let steps = [tunnel, PathStep::Dir(2), PathStep::Dir(4)];
    // Cells near the exit are not acceptable: only the shape judged from
    // the cell north of the entrance passes.
    let ok =
        |s: CellXY, shape: &[u8], _limit: u32| visits(s, shape).iter().all(|c| c.x < cell(5, 0).x);
    assert_eq!(
        path::shorten(&steps, cell(0, 0), true, true, &ok),
        vec![tunnel, PathStep::Dir(3)]
    );
    assert_eq!(
        path::shorten(&steps, cell(0, 0), true, false, &ok),
        steps.to_vec()
    );
}

/// A train type at cell(0, 0) facing map north, with wall sections on
/// `walls`, ordered to `to`: the stored path.
fn train_path(walls: &[CellXY], to: CellXY) -> Vec<u8> {
    let mut s = scene();
    s.vehicle(
        "TESTTRAIN",
        UnitType {
            is_train: Some(true),
            ..unit_type(WHEEL, 40, 6)
        },
    );
    s.register("overlays", "TESTFENCE");
    s.ruleset.sim.overlays.insert(
        "TESTFENCE".into(),
        glasnost_rules::sim::OverlayType {
            wall: Some(true),
            ..Default::default()
        },
    );
    s.map.has_overlay_data = true;
    for &c in walls {
        s.cell_mut(c).overlay = 0;
    }
    s.place_unit(0, "TESTTRAIN", cell(0, 0), 0);
    let mut w = s.build();
    let t = vehicles(&w)[0];
    assert_eq!(compute_to(&mut w, t, to, Mode::Fresh), Search::Found);
    w.obj(t).unwrap().path.steps.clone()
}

// spec: S-0212 claim 18 (scenario P9 train runs 2 to 4: the opening detour of a train ordered behind itself is shortened)
#[test]
fn train_detours_are_shortened() {
    // Run 2: 6 cells, the first behind the train.
    assert_eq!(train_path(&[], cell(0, 6)), vec![3, 4, 5, 4, 4, 4]);
    // Run 3: the opening 2, 3, 4 becomes 3, 3.
    assert_eq!(train_path(&[], cell(2, 6)), vec![3, 3, 4, 4, 4, 4]);
    // Run 4: claim 17 cannot flatten 1, 3 past the wall; claim 18 gives 3, 4, 4, 4.
    let walls = [cell(-2, 0), cell(-1, 0), cell(1, 0), cell(0, 2), cell(0, 3)];
    assert_eq!(train_path(&walls, cell(1, 4)), vec![3, 4, 4, 4]);
}

// ------------------------------------------------------- re-paths, congestion

/// A unit A at cell(0, 0) facing east and a unit B of type `b_type` at
/// cell(1, 0) facing east with `b_steps` stored steps east.
fn congestion_scene(b_type: &str, b_steps: usize) -> (World, ObjectId) {
    let mut s = scene();
    s.vehicle("TESTSLOW", unit_type(WHEEL, 20, 6));
    s.place_unit(0, CAR, cell(0, 0), 64);
    s.place_unit(0, b_type, cell(1, 0), 64);
    let mut w = s.build();
    let at = |w: &World, c: CellXY| {
        vehicles(w)
            .into_iter()
            .find(|&v| w.obj(v).unwrap().pos.cell() == c)
            .unwrap()
    };
    let a = at(&w, cell(0, 0));
    let b = at(&w, cell(1, 0));
    hold_as_moving(&mut w, b);
    w.obj_mut(b).unwrap().path.steps = vec![2; b_steps];
    (w, a)
}

// spec: S-0212 claim 19 (mode 2: the stored path of a vehicle with two or more steps in the cell ahead is made costly)
#[test]
fn mode_two_marks_the_path_of_the_unit_ahead() {
    let marked = [cell(3, 0), cell(4, 0), cell(5, 0)];
    let dest = cell(6, 0);
    // One stored step: the unit does not qualify, the route rejoins the row.
    let (mut w, a) = congestion_scene(CAR, 1);
    assert_eq!(compute_to(&mut w, a, dest, Mode::Avoid), Search::Found);
    let short = visits(cell(0, 0), &w.obj(a).unwrap().path.steps);
    assert!(short.iter().any(|c| marked.contains(c)), "{short:?}");
    // Seven stored steps: those cells carry the mark and are avoided.
    let (mut w, a) = congestion_scene(CAR, 7);
    assert_eq!(compute_to(&mut w, a, dest, Mode::Avoid), Search::Found);
    let long = visits(cell(0, 0), &w.obj(a).unwrap().path.steps);
    assert!(!long.iter().any(|c| marked.contains(c)), "{long:?}");
    assert_eq!(*long.last().unwrap(), dest);
}

// spec: S-0212 claim 19 (mode 1: a unit of the same type does not qualify, so the search runs as mode 0)
#[test]
fn mode_one_without_a_qualifying_unit_runs_as_mode_zero() {
    let dest = cell(6, 0);
    let (mut w, a) = congestion_scene(CAR, 7);
    assert_eq!(compute_to(&mut w, a, dest, Mode::Blocked), Search::Found);
    let same = w.obj(a).unwrap().path.steps.clone();
    let (mut w0, a0) = congestion_scene(CAR, 7);
    assert_eq!(compute_to(&mut w0, a0, dest, Mode::Fresh), Search::Found);
    assert_eq!(same, w0.obj(a0).unwrap().path.steps);
    // A slower unit of another type qualifies: its path is marked.
    let (mut w, a) = congestion_scene("TESTSLOW", 7);
    assert_eq!(compute_to(&mut w, a, dest, Mode::Blocked), Search::Found);
    let cells = visits(cell(0, 0), &w.obj(a).unwrap().path.steps);
    assert!(
        !cells
            .iter()
            .any(|c| [cell(3, 0), cell(4, 0), cell(5, 0)].contains(c)),
        "{cells:?}"
    );
    assert_ne!(w.obj(a).unwrap().path.steps, same);
}

// -------------------------------------------------------- storage and timing

// spec: S-0212 claim 20 (a unit keeps at most 24 steps of a found path)
#[test]
fn stored_path_keeps_24_steps() {
    let mut s = scene();
    s.place_unit(0, CAR, cell(-15, -15), 64);
    let mut w = s.build();
    let id = vehicles(&w)[0];
    let full = find(&w, id, cell(15, 15), Mode::Fresh, &[]).unwrap();
    assert_eq!(full.len(), 30);
    assert_eq!(
        compute_to(&mut w, id, cell(15, 15), Mode::Fresh),
        Search::Found
    );
    assert_eq!(w.obj(id).unwrap().path.steps, full[..24].to_vec());
}

// spec: S-0212 claim 20 (the drive locomotor computes a new path as it begins the last stored step, so it arrives beyond 24 steps)
#[test]
fn driving_unit_arrives_beyond_24_steps() {
    let mut s = scene();
    s.place_unit(0, CAR, cell(-15, -15), 64);
    let mut w = s.build();
    open_orders(&mut w);
    let id = vehicles(&w)[0];
    order_move(&mut w, id, cell(15, 15));
    let mut longest = 0;
    let n = common::run_until(&mut w, 3000, |w| {
        longest = longest.max(w.obj(id).unwrap().path.steps.len());
        w.obj(id).unwrap().pos.cell() == cell(15, 15)
    });
    assert!(n < 3000, "did not arrive");
    assert!(longest <= 24);
}

// spec: S-0212 claim 21 (R = trunc(P × 900); it runs after a failed fresh search, not after a found path)
#[test]
fn retry_delay_after_fresh_searches() {
    let mut s = scene();
    s.ruleset.sim.general.path_delay = Some(Decimal("0.05".into()));
    for c in ring(cell(5, 0), 1) {
        rock(&mut s, c, 0);
    }
    s.place_unit(0, CAR, cell(0, 0), 64);
    let mut w = s.build();
    common::run(&mut w, 7);
    let t = w.tick;
    let id = vehicles(&w)[0];
    assert_eq!(w.rules().general.retry_delay, 45);
    // Found: no delay.
    let r = compute_to(&mut w, id, cell(-5, 0), Mode::Fresh);
    assert_eq!(r, Search::Found);
    path::failed(&mut w, id, r, false);
    assert!(w.obj(id).unwrap().path.retry_at <= t);
    // Gate failed after a fresh search: r97, the driving locomotor's
    // destination is cleared, which ends the delay, so none remains.
    // spec: S-0212 claims 21 and 23 (r97: for the driving and walk locomotors the clearing ends the retry delay, so none remains)
    let r = compute_to(&mut w, id, cell(5, 0), Mode::Fresh);
    assert_eq!(r, Search::GateFailed);
    path::failed(&mut w, id, r, false);
    assert!(w.obj(id).unwrap().path.retry_at <= t);
    assert!(!path::delay_runs(&w, id));
    // Gate failed after a re-path of the drive locomotor: no delay.
    w.obj_mut(id).unwrap().path.retry_at = 0;
    let r = compute_to(&mut w, id, cell(5, 0), Mode::Blocked);
    assert_eq!(r, Search::GateFailed);
    path::failed(&mut w, id, r, true);
    assert_eq!(w.obj(id).unwrap().path.retry_at, 0);
}

// spec: S-0212 claim 21 (a re-path for a blocked step waits for the delay and restarts it)
// spec: S-0212 claim 22 (moving blocker: re-paths in mode 1 while the blockage timer runs, mode 2 once it has run out)
#[test]
fn moving_blocker_wait_then_detour() {
    let mut s = scene();
    s.place_unit(0, CAR, cell(0, 0), 64);
    s.place_unit(0, CAR, cell(1, 0), 64);
    let mut w = s.build();
    open_orders(&mut w);
    let vs = vehicles(&w);
    let (a, b) = if w.obj(vs[0]).unwrap().pos.cell() == cell(0, 0) {
        (vs[0], vs[1])
    } else {
        (vs[1], vs[0])
    };
    order_move(&mut w, a, cell(4, 0));
    hold_as_moving(&mut w, b);
    let start = w.obj(a).unwrap().pos;
    let r = w.rules().general.retry_delay;
    let delay = w.rules().general.blockage_delay;
    let mut since = None;
    let mut retry_log = Vec::new();
    let mut moved_at = None;
    for _ in 0..200 {
        common::run(&mut w, 1);
        let t = w.tick - 1;
        let o = w.obj(a).unwrap();
        assert!(
            w.obj(b).unwrap().dest.is_some(),
            "the blocker stays a moving unit"
        );
        if since.is_none() {
            since = o.path.blocked_since;
        }
        if retry_log.last() != Some(&o.path.retry_at) {
            retry_log.push(o.path.retry_at);
        }
        if o.pos != start {
            moved_at = Some(t);
            break;
        }
    }
    let since = since.expect("a blockage began");
    let moved_at = moved_at.expect("the unit went round");
    // Re-paths every R ticks from the blockage, each restarting the delay.
    let expected: Vec<i32> = (0..)
        .map(|k| since + k * r)
        .take_while(|&t| t - since < delay + r)
        .map(|t| t + r)
        .collect();
    assert_eq!(&retry_log[1..], &expected[..], "retry log {retry_log:?}");
    // The detour comes only from the first mode-2 re-path.
    let first_mode2 = (0..)
        .map(|k| since + k * r)
        .find(|&t| t - since >= delay)
        .unwrap();
    assert!(
        moved_at >= first_mode2,
        "moved at {moved_at}, mode 2 at {first_mode2}"
    );
    let o = w.obj(a).unwrap();
    assert_ne!(o.pos.cell(), cell(1, 0));
}

// spec: S-0212 claim 22 (at the first check of a track's first cell, blocked clears the stored path and makes a fresh search in mode 0 on the same tick, regardless of the path retry timer)
#[test]
fn blocked_next_step_clears_the_path() {
    let mut s = scene();
    rock(&mut s, cell(1, 0), 0);
    s.place_unit(0, CAR, cell(0, 0), 64);
    let mut w = s.build();
    open_orders(&mut w);
    let id = vehicles(&w)[0];
    let t = w.tick;
    {
        let o = w.obj_mut(id).unwrap();
        o.dest = Some(Target::Cell(cell(4, 0)));
        o.path.steps = vec![2, 2, 2, 2]; // the first step leads onto the rock
        o.path.retry_at = t + 5;
    }
    common::run(&mut w, 1);
    // On the same tick, although the retry delay still runs, a fresh path
    // goes round the rock.
    let o = w.obj(id).unwrap();
    assert!(o.dest.is_some());
    let first = o.path.steps.first().copied();
    assert!(
        first.is_some_and(|d| d != 2),
        "a new path: {:?}",
        o.path.steps
    );
    assert_ne!(o.pos.cell(), cell(1, 0));
}

// ------------------------------------------------------------------ failures

// spec: S-0212 claim 23 (gate failure: the driving unit's destination is cleared and it does not give up)
#[test]
fn gate_failure_clears_destination_only() {
    let mut s = scene();
    for c in ring(cell(5, 0), 1) {
        rock(&mut s, c, 0);
    }
    s.place_unit(0, CAR, cell(0, 0), 64);
    let mut w = s.build();
    let id = vehicles(&w)[0];
    w.obj_mut(id).unwrap().beh.queued = None;
    let r = compute_to(&mut w, id, cell(5, 0), Mode::Fresh);
    assert_eq!(r, Search::GateFailed);
    path::failed(&mut w, id, r, false);
    let o = w.obj(id).unwrap();
    assert_eq!(o.dest, None);
    assert_eq!(o.beh.queued, None);
}

// spec: S-0212 claim 23 (search failure more than 1 cell away: the unit gives up and queues guard)
#[test]
fn search_failure_far_away_gives_up() {
    let mut s = scene();
    // Allied buildings leave the ring's cells open: the gate passes, the
    // search cannot get in.
    for c in ring(cell(5, 0), 1) {
        s.place_structure(0, HUT, c);
    }
    s.place_unit(0, CAR, cell(0, 0), 64);
    let mut w = s.build();
    let id = vehicles(&w)[0];
    w.obj_mut(id).unwrap().beh.queued = None;
    let r = compute_to(&mut w, id, cell(5, 0), Mode::Fresh);
    assert_eq!(r, Search::SearchFailed);
    path::failed(&mut w, id, r, false);
    let o = w.obj(id).unwrap();
    assert_eq!(o.dest, None);
    assert_eq!(o.beh.queued, Some(Behaviour::Guard));
    // spec: S-0212 claim 23 row 2 (r97: R starts, and the clearing of the destination ends it again for the driving locomotor, claim 21)
    assert!(o.path.retry_at <= w.tick);
    assert!(!path::delay_runs(&w, id));
}

// spec: S-0212 claim 23 (search failure within 1 cell: the unit stops without giving up; a driving unit's destination is cleared)
#[test]
fn search_failure_within_one_cell_does_not_give_up() {
    let mut s = scene();
    s.place_unit(0, CAR, cell(0, 0), 64);
    let mut w = s.build();
    let id = vehicles(&w)[0];
    {
        let o = w.obj_mut(id).unwrap();
        o.beh.queued = None;
        o.dest = Some(Target::Cell(cell(1, 1)));
    }
    path::failed(&mut w, id, Search::SearchFailed, false);
    let o = w.obj(id).unwrap();
    assert_eq!(o.dest, None);
    assert_eq!(o.beh.queued, None);
    // spec: S-0212 claim 23 row 3 (r97: a driving unit ends up without a destination and with no retry delay, claim 21)
    assert!(o.path.retry_at <= w.tick);
    assert!(!path::delay_runs(&w, id));
}

// spec: S-0212 claim 15 (a unit already next to its blocked destination gets no path: the search fails)
#[test]
fn next_to_blocked_destination_gets_no_path() {
    let mut s = scene();
    s.place_unit(1, CAR, cell(1, 0), 64); // an enemy: blocked for an unarmed car
    s.place_unit(0, CAR, cell(0, 0), 64);
    s.place_structure(0, HUT, cell(4, 0));
    s.place_unit(0, CAR, cell(3, 0), 64);
    let w = s.build();
    let at = |c: CellXY| {
        vehicles(&w)
            .into_iter()
            .find(|&v| w.obj(v).unwrap().pos.cell() == c)
            .unwrap()
    };
    let r = occupancy::reference_level(&w, at(cell(0, 0)));
    assert_eq!(
        occupancy::vehicle_outcome(&w, at(cell(0, 0)), cell(1, 0), r),
        Outcome::Blocked
    );
    assert_eq!(find(&w, at(cell(0, 0)), cell(1, 0), Mode::Fresh, &[]), None);
    assert_eq!(find(&w, at(cell(3, 0)), cell(4, 0), Mode::Fresh, &[]), None);
}

// ------------------------------------------------------------- substitution

/// A unit at `unit` and an allied parked TESTCAR at cell(6, 0), or an allied
/// TESTHUT there when `hut` is set; `kind` is the moving unit's type.
fn substitution_scene(unit: CellXY, hut: bool, kind: &str) -> (World, ObjectId) {
    let mut s = scene();
    s.vehicle(
        "TESTTRAIN",
        UnitType {
            is_train: Some(true),
            ..unit_type(WHEEL, 40, 6)
        },
    );
    if hut {
        s.place_structure(0, HUT, cell(6, 0));
    } else {
        s.place_unit(0, CAR, cell(6, 0), 64);
    }
    s.place_unit(0, kind, unit, 64);
    let w = s.build();
    let id = vehicles(&w)
        .into_iter()
        .find(|&v| w.obj(v).unwrap().pos.cell() == unit)
        .unwrap();
    (w, id)
}

// spec: S-0212 claim 24 (a stationary friendly on the destination: the nearby cell nearest the unit is adopted when farther than close enough)
#[test]
fn stationary_friendly_destination_is_replaced() {
    let (mut w, id) = substitution_scene(cell(0, 0), false, CAR);
    assert_eq!(
        compute_to(&mut w, id, cell(6, 0), Mode::Fresh),
        Search::Found
    );
    let o = w.obj(id).unwrap();
    assert_eq!(o.path.for_goal, Some(cell(5, 0)));
    // The adopted cell becomes the unit's destination (departure D3's "replaces").
    assert_eq!(o.dest, Some(Target::Cell(cell(5, 0))));
    assert_eq!(
        *visits(cell(0, 0), &o.path.steps).last().unwrap(),
        cell(5, 0)
    );
    // Within the close-enough distance (2 cells = 512 leptons < 640): kept.
    let (mut w, id) = substitution_scene(cell(4, 0), false, CAR);
    assert_eq!(
        compute_to(&mut w, id, cell(6, 0), Mode::Fresh),
        Search::Found
    );
    assert_eq!(w.obj(id).unwrap().path.for_goal, Some(cell(6, 0)));
    assert_eq!(w.obj(id).unwrap().dest, Some(Target::Cell(cell(6, 0))));
}

// spec: S-0212 claim 24 (a blocked building cell is always replaced, with no distance test; trains are never substituted)
#[test]
fn building_destination_is_always_replaced() {
    for start in [cell(0, 0), cell(4, 0)] {
        let (mut w, id) = substitution_scene(start, true, CAR);
        assert_eq!(
            compute_to(&mut w, id, cell(6, 0), Mode::Fresh),
            Search::Found
        );
        assert_eq!(
            w.obj(id).unwrap().path.for_goal,
            Some(cell(5, 0)),
            "{start:?}"
        );
        assert_eq!(
            w.obj(id).unwrap().dest,
            Some(Target::Cell(cell(5, 0))),
            "{start:?}"
        );
    }
    let (mut w, id) = substitution_scene(cell(0, 0), false, "TESTTRAIN");
    let _ = compute_to(&mut w, id, cell(6, 0), Mode::Fresh);
    assert_ne!(w.obj(id).unwrap().path.for_goal, Some(cell(5, 0)));
    assert_eq!(w.obj(id).unwrap().dest, Some(Target::Cell(cell(6, 0))));
}

// --------------------------------------------------------- entry outcomes

// spec: S-0212 claim 27 (allied vehicles: stationary friendly, moving blocker, the close case blocked; allied buildings blocked)
#[test]
fn vehicle_entry_outcomes() {
    let mut s = scene();
    s.place_unit(0, CAR, cell(0, 0), 64); // mover, facing east
    s.place_unit(0, CAR, cell(1, 0), 64); // allied, parked, same facing
    s.place_unit(0, CAR, cell(0, 2), 64); // allied, will be moving
    s.place_unit(0, CAR, cell(1, -2), 192); // allied, facing west, ahead and close
    s.place_unit(0, CAR, cell(2, -4), 192); // allied, facing west, ahead but 2 cells away
    s.place_structure(0, HUT, cell(-1, 0));
    let mut w = s.build();
    let at = |w: &World, c: CellXY| {
        vehicles(w)
            .into_iter()
            .find(|&v| w.obj(v).unwrap().pos.cell() == c)
            .unwrap()
    };
    let m = at(&w, cell(0, 0));
    for c in [cell(0, 2), cell(1, -2), cell(2, -4)] {
        let id = at(&w, c);
        hold_as_moving(&mut w, id);
    }
    let r = occupancy::reference_level(&w, m);
    let out = |w: &World, c: CellXY| occupancy::vehicle_outcome(w, m, c, r);
    assert_eq!(out(&w, cell(1, 0)), Outcome::StationaryFriendly);
    assert_eq!(out(&w, cell(-1, 0)), Outcome::Blocked);
    // Move the mover next to each moving unit in turn (same facing east).
    let place = |w: &mut World, c: CellXY| {
        let o = w.obj_mut(m).unwrap();
        o.pos = glasnost_sim::world::Pos::centre(c, 0);
    };
    place(&mut w, cell(-1, 2));
    assert_eq!(out(&w, cell(0, 2)), Outcome::MovingBlocker);
    place(&mut w, cell(0, -2));
    assert!(occupancy::close_case(&w, m, at(&w, cell(1, -2))));
    assert_eq!(out(&w, cell(1, -2)), Outcome::Blocked);
    place(&mut w, cell(0, -4));
    assert!(!occupancy::close_case(&w, m, at(&w, cell(2, -4))));
    assert_eq!(out(&w, cell(2, -4)), Outcome::MovingBlocker);
}

// spec: S-0212 claim 27 (a wall section gives blocked to a vehicle without a weapon that does not crush it)
#[test]
fn wall_blocks_an_unarmed_vehicle() {
    let mut s = scene();
    s.register("overlays", "TESTFENCE");
    s.ruleset.sim.overlays.insert(
        "TESTFENCE".into(),
        glasnost_rules::sim::OverlayType {
            wall: Some(true),
            ..Default::default()
        },
    );
    s.map.has_overlay_data = true;
    s.cell_mut(cell(1, 0)).overlay = 0;
    s.place_unit(0, CAR, cell(0, 0), 64);
    let w = s.build();
    let id = vehicles(&w)[0];
    assert_eq!(w.cells.cell(cell(1, 0)).unwrap().overlay, 0);
    let r = occupancy::reference_level(&w, id);
    assert_eq!(
        occupancy::vehicle_outcome(&w, id, cell(1, 0), r),
        Outcome::Blocked
    );
    assert_eq!(
        occupancy::vehicle_outcome(&w, id, cell(1, 1), r),
        Outcome::Clear
    );
}

// ------------------------------------------------- the nearby-cell search, T5

/// A unit of `ty` at cell(0, 0) and a car on cell(6, 0), inside a ring of
/// rock cells at Chebyshev distance 2 from it: the island of cell(6, 0)
/// and its eight neighbours is a region of its own.
fn island(ty: &str, t: UnitType) -> (World, ObjectId) {
    let mut s = scene();
    s.vehicle(ty, t);
    for c in ring(cell(6, 0), 2) {
        rock(&mut s, c, 0);
    }
    s.place_unit(0, ty, cell(0, 0), 64);
    s.place_unit(1, CAR, cell(6, 0), 64);
    let mut w = s.build();
    regions::ensure(&mut w);
    let id = vehicles(&w)
        .into_iter()
        .find(|&v| w.obj(v).unwrap().pos.cell() == cell(0, 0))
        .unwrap();
    (w, id)
}

fn order_search(w: &World, id: ObjectId, on_object: bool, free_no_move: bool) -> Option<CellXY> {
    path::nearby(
        w,
        cell(6, 0),
        path::NearbyUse::MoveOrder {
            unit: id,
            on_object,
            free_no_move,
        },
    )
}

// spec: S-0212 table T5 (move orders: the unit's reach region; the no-move rows of a free mover: no reach region)
#[test]
fn order_search_reach_region() {
    let (w, id) = island("TESTWHEEL", unit_type(WHEEL, 40, 6));
    let near = |c: Option<CellXY>| chebyshev(c.expect("a cell"), cell(6, 0));
    // With the reach region, the island's cells are out: ring 3.
    assert_eq!(near(order_search(&w, id, false, false)), 3);
    assert_eq!(near(order_search(&w, id, true, false)), 3);
    // Without it (a free mover's no-move), the island's ring 1 qualifies.
    assert_eq!(near(order_search(&w, id, false, true)), 1);
    assert_eq!(near(order_search(&w, id, true, true)), 1);
}

// spec: S-0212 table T5 (fly counted as infantry for a click on an object always, for a click on a cell only under an ion storm)
#[test]
fn order_search_fly_zone() {
    let (w, id) = island(
        "TESTFLY",
        UnitType {
            movement_zone: Some(MovementZone::Fly),
            ..unit_type(WHEEL, 40, 6)
        },
    );
    let near = |c: Option<CellXY>| chebyshev(c.expect("a cell"), cell(6, 0));
    // Zone fly reaches across the rock ring: the island's ring 1.
    assert_eq!(near(order_search(&w, id, false, false)), 1);
    // As infantry, the island is another reach region: ring 3.
    assert_eq!(near(order_search(&w, id, true, false)), 3);
}

// spec: S-0212 claim 25 (the further test is S-0010's burrowing test: outside the usable area every cell passes; inside, flat, no bridge cell, no building or terrain object)
// spec: S-0212 claim 24 (a subterranean unit whose destination passes the further test gets only substitutes that pass it)
#[test]
fn further_test_is_the_burrowing_test() {
    let mut s = scene();
    s.vehicle(
        "TESTMOLE",
        UnitType {
            movement_zone: Some(MovementZone::Subterranean),
            ..unit_type(WHEEL, 40, 6)
        },
    );
    s.set_cell(cell(5, 1), slope_tile(1), 0);
    s.place_structure(1, HUT, cell(4, 4));
    s.place_unit(0, "TESTMOLE", cell(0, 0), 64);
    s.place_unit(1, CAR, cell(6, 0), 64);
    let mut w = s.build();
    regions::ensure(&mut w);
    assert!(path::burrow_ok(&w, cell(6, 0)));
    assert!(!path::burrow_ok(&w, cell(5, 1)), "a slope fails");
    assert!(!path::burrow_ok(&w, cell(4, 4)), "a building cell fails");
    let outside = CellXY::new(-5, -5);
    assert!(!w.cells.usable(outside));
    assert!(
        path::burrow_ok(&w, outside),
        "outside the usable area passes"
    );
}

// spec: S-0212 claim 25 (the no-move rows ask for the further test only for a subterranean unit below the ground surface; every candidate, the centre included, must pass, whatever the clicked cell gives)
#[test]
fn no_move_further_test_for_a_unit_underground() {
    let mut s = scene();
    s.vehicle(
        "TESTMOLE",
        UnitType {
            movement_zone: Some(MovementZone::Subterranean),
            ..unit_type(WHEEL, 40, 6)
        },
    );
    // The clicked cell is a slope: it qualifies, but fails the burrowing test.
    s.set_cell(cell(6, 0), slope_tile(1), 0);
    s.place_unit(0, "TESTMOLE", cell(0, 0), 64);
    let mut w = s.build();
    regions::ensure(&mut w);
    let id = vehicles(&w)[0];
    // On the surface: no further test, the clicked cell itself.
    assert_eq!(order_search(&w, id, false, true), Some(cell(6, 0)));
    // Below the ground surface: the slope fails, a flat ring-1 cell is taken.
    w.obj_mut(id).unwrap().pos.z = -1;
    let c = order_search(&w, id, false, true).expect("a cell");
    assert_eq!(chebyshev(c, cell(6, 0)), 1);
    assert!(path::burrow_ok(&w, c));
    // The move row (not a free mover's no-move) never asks for it.
    assert_eq!(
        chebyshev(order_search(&w, id, false, false).unwrap(), cell(6, 0)),
        0
    );
}

// spec: S-0010 claim 17 (a tile of a tileset without the allow-burrowing flag refuses; a tile number outside the theater's tiles passes the tileset condition)
#[test]
fn allow_burrowing_flag_of_the_tileset() {
    let mut s = scene();
    let n = s.catalogue.tiles.len() as i32;
    let mut t = s.catalogue.tiles[0].clone();
    t.no_burrowing = true;
    s.catalogue.tiles.push(t);
    s.set_cell(cell(2, 0), n, 0);
    s.set_cell(cell(3, 0), 0, 0);
    let w = s.build();
    assert!(!path::burrow_ok(&w, cell(2, 0)));
    assert!(path::burrow_ok(&w, cell(3, 0)));
}

// ------------------------------------------------------------- bridge decks

/// Makes `c` a bridge cell along cell x, with the deck-entry mark when
/// `entry` (S-0212 Terms), as a high-bridge overlay would.
fn lay_deck(w: &mut World, c: CellXY, entry: bool) {
    let i = w.cells.idx(c).unwrap();
    w.cells.deck[i] = true;
    w.cells.deck_entry[i] = entry;
    w.cells.bridge_axis[i] = glasnost_sim::cells::Axis::X;
    w.regions.dirty = true;
}

/// Plateau at height level 4 everywhere but a pit at level 0 under
/// x = 3 … 5, y = −1 … 1; with `deck`, the pit's middle row carries a
/// bridge deck. A car stands on the plateau at cell(2, 0) facing east.
fn bridge_scene(deck: bool) -> (World, ObjectId) {
    let mut s = scene();
    for c in all_cells(&s) {
        let pit = (cell(3, 0).x..=cell(5, 0).x).contains(&c.x) && (c.y - cell(0, 0).y).abs() <= 1;
        s.set_cell(c, 0, if pit { 0 } else { 4 });
    }
    s.place_unit(0, CAR, cell(2, 0), 64);
    let mut w = s.build();
    if deck {
        for x in 3..=5 {
            lay_deck(&mut w, cell(x, 0), true);
        }
    }
    regions::ensure(&mut w);
    let car = vehicles(&w)[0];
    (w, car)
}

// spec: S-0212 claim 10 (Δ = −4 onto a marked deck, Δ = 0 along marked deck cells, Δ = +4 off a deck; 4 levels from ground otherwise blocked)
// spec: S-0212 claim 7 (a step onto a bridge cell from a pair more than 1 level higher is on its deck; the destination of a bridge cell is its deck)
#[test]
fn bridge_deck_steps() {
    let (w, car) = bridge_scene(true);
    let r = occupancy::reference_level(&w, car);
    assert_eq!(r, 4);
    // From the plateau onto the deck, and along the deck.
    assert_eq!(
        path::step_outcome(&w, car, cell(2, 0), cell(3, 0), 4),
        Outcome::Clear
    );
    assert_eq!(
        path::step_outcome(&w, car, cell(3, 0), cell(4, 0), 4 + 4),
        Outcome::Clear
    );
    // From the ground under the deck up to the plateau: blocked.
    assert_eq!(
        path::step_outcome(&w, car, cell(3, 0), cell(2, 0), 0),
        Outcome::Blocked
    );
    // Into a pit cell without a deck: 4 levels down, blocked.
    assert_eq!(
        path::step_outcome(&w, car, cell(2, 0), cell(3, 1), 4),
        Outcome::Blocked
    );
    // The search crosses the deck straight; without the deck it goes round.
    assert_eq!(
        find(&w, car, cell(6, 0), Mode::Fresh, &[]),
        Some(vec![2; 4])
    );
    let (w, car) = bridge_scene(false);
    let p = find(&w, car, cell(6, 0), Mode::Fresh, &[]).expect("a way round");
    assert!(p.len() > 4, "{p:?}");
}

// spec: S-0207 claim 5 (a vehicle drives onto the deck from land 4 levels higher, at H + 416, and off it again)
#[test]
fn a_vehicle_drives_across_a_deck() {
    use glasnost_sim::world::Layer;
    let (mut w, car) = bridge_scene(true);
    open_orders(&mut w);
    order_move(&mut w, car, cell(6, 0));
    let mut on_deck = false;
    let n = common::run_until(&mut w, 600, |w| {
        let o = w.obj(car).unwrap();
        if o.pos.cell() == cell(4, 0) && o.layer == Layer::Deck {
            on_deck = true;
        }
        o.pos.cell() == cell(6, 0)
    });
    assert!(n < 600, "the car never crossed");
    assert!(on_deck, "the car crossed on the deck");
    assert_eq!(w.obj(car).unwrap().layer, Layer::Ground);
}

// spec: S-0212 claim 10 (the table: a deck pair is judged by its ground level; deck to deck at Δ = 0 needs the mark, onto a deck at Δ = −4 needs the mark, off a deck at Δ = +4 needs nothing, a deck never steps onto the ground under it)
#[test]
fn deck_steps_and_the_deck_entry_mark() {
    let (mut w, car) = bridge_scene(true);
    // The pit's row y + 1 carries a deck without the deck-entry mark.
    for x in 3..=5 {
        lay_deck(&mut w, cell(x, 1), false);
    }
    regions::ensure(&mut w);
    let out = |a: CellXY, b: CellXY, r: i32| path::step_outcome(&w, car, a, b, r);
    // Deck to deck with the mark (Δ = 0), and not onto the unmarked row.
    assert_eq!(out(cell(4, 0), cell(5, 0), 4), Outcome::Clear);
    assert_eq!(out(cell(4, 0), cell(4, 1), 4), Outcome::Blocked);
    // Onto the unmarked row from the plateau at deck level (Δ = −4): refused.
    assert_eq!(out(cell(4, 2), cell(4, 1), 4), Outcome::Blocked);
    // Onto the marked row from the plateau: allowed; off a deck sideways onto
    // the plateau (Δ = +4): allowed.
    assert_eq!(
        out(cell(4, -2), cell(4, -1), 4),
        Outcome::Blocked,
        "no deck at y − 1"
    );
    assert_eq!(out(cell(2, -1), cell(3, 0), 4), Outcome::Clear);
    assert_eq!(out(cell(3, 0), cell(2, -1), 4), Outcome::Clear);
    // From a deck pair onto the ground under the deck of a neighbour that is
    // not a bridge cell: refused (Δ = 0 from a deck pair).
    assert_eq!(out(cell(4, 0), cell(4, -1), 4), Outcome::Blocked);
    // Under the deck the steps are those between ground cells.
    assert_eq!(out(cell(4, 0), cell(4, 1), 0), Outcome::Clear);
    assert_eq!(out(cell(4, 0), cell(4, -1), 0), Outcome::Clear);
}

// spec: S-0212 claim 10 (Δ = ±1 from a deck pair: the slope rule on the ground levels; onto a cell that is not a bridge cell it lands on that cell's ground)
// spec: S-0212 Edge cases (`quirks.deck_side_step_ground_level` off: such a step onto a cell that is not a bridge cell is refused; onto another deck it is allowed either way)
#[test]
fn deck_side_step_onto_ground_one_level_away() {
    for on in [true, false] {
        let mut s = scene();
        s.ruleset.globals.quirks.deck_side_step_ground_level = on;
        // A sloped bridge cell at level 0 with a flat cell at level 1 beside
        // it, and a flat bridge cell at level 1 beyond that.
        s.set_cell(cell(0, 0), slope_tile(1), 0);
        s.set_cell(cell(1, 0), 0, 1);
        s.set_cell(cell(0, 1), 0, 1);
        s.place_unit(0, CAR, cell(-5, 0), 64);
        let mut w = s.build();
        lay_deck(&mut w, cell(0, 0), true);
        lay_deck(&mut w, cell(0, 1), true);
        regions::ensure(&mut w);
        let car = vehicles(&w)[0];
        let out = |b: CellXY| path::step_outcome(&w, car, cell(0, 0), b, 4);
        let expect = if on { Outcome::Clear } else { Outcome::Blocked };
        assert_eq!(
            out(cell(1, 0)),
            expect,
            "onto ground 3 levels below, switch {on}"
        );
        assert_eq!(
            out(cell(0, 1)),
            Outcome::Clear,
            "onto another deck, switch {on}"
        );
    }
}

// spec: S-0212 claim 7 (the start pair: on the deck for a unit on a bridge deck that is not an aircraft; a train at deck height on a bridge cell starts on the deck)
#[test]
fn start_pair_of_a_train_at_deck_height() {
    use glasnost_sim::world::Layer;
    let mut s = scene();
    s.vehicle(
        "TESTTRAIN",
        UnitType {
            is_train: Some(true),
            ..unit_type(WHEEL, 40, 6)
        },
    );
    s.place_unit(0, "TESTTRAIN", cell(0, 0), 64);
    s.place_unit(0, CAR, cell(3, 0), 64);
    let mut w = s.build();
    lay_deck(&mut w, cell(0, 0), true);
    lay_deck(&mut w, cell(3, 0), true);
    regions::ensure(&mut w);
    let train = objects_of(&w, Kind::Vehicle)
        .into_iter()
        .find(|&v| w.obj(v).unwrap().pos.cell() == cell(0, 0))
        .unwrap();
    let car = objects_of(&w, Kind::Vehicle)
        .into_iter()
        .find(|&v| v != train)
        .unwrap();
    // On the ground at ground height: the ground pair.
    assert_eq!(path::start_pair(&w, train, cell(0, 0)), (Layer::Ground, 0));
    // At deck height (z = 416, height level 4), not marked as on the deck:
    // |4 − 0| > 2, so the start pair is raised to the deck.
    w.obj_mut(train).unwrap().pos.z = 416;
    assert_eq!(path::start_pair(&w, train, cell(0, 0)), (Layer::Deck, 4));
    // At 2 levels (z = 208): within 2, the ground pair.
    w.obj_mut(train).unwrap().pos.z = 208;
    assert_eq!(path::start_pair(&w, train, cell(0, 0)), (Layer::Ground, 0));
    // An ordinary vehicle at deck height that is not on the deck: ground.
    w.obj_mut(car).unwrap().pos.z = 416;
    assert_eq!(path::start_pair(&w, car, cell(3, 0)), (Layer::Ground, 0));
    w.obj_mut(car).unwrap().layer = Layer::Deck;
    assert_eq!(path::start_pair(&w, car, cell(3, 0)), (Layer::Deck, 4));
}

// ------------------------------------------------------------- bridge links

/// First tile number of the bridge role in [`river`]'s catalogue.
const BRIDGE_ROLE: i32 = 42;

/// A rock river over the columns x = 1 … 5 (all rows) between two banks,
/// and a bridge span from A = (0, 0) to B = (6, 0): both hold tile 6 of the
/// bridge role (S-0010 table B: start and end sub-tile 4, running east). A
/// high-bridge overlay of index 24 stands in each river cell of row 0 but
/// those in `gaps` (S-0211 claim 15: each marks rows 1, 0, −1 and −2 of its
/// column). A car stands on bank A at (−2, 0).
fn river(gaps: &[i32]) -> (World, ObjectId) {
    river_in(scene(), gaps)
}

/// [`river`] built from scene `s`.
fn river_in(s: Scene, gaps: &[i32]) -> (World, ObjectId) {
    river_in_with(s, gaps, &[])
}

/// [`river_in`] with the cells of `clear` left open ground, not rock.
fn river_in_with(mut s: Scene, gaps: &[i32], clear: &[CellXY]) -> (World, ObjectId) {
    for i in 0..=24 {
        let name = format!("OV{i}");
        s.register("overlays", &name);
        s.ruleset.sim.overlays.insert(
            name,
            glasnost_rules::sim::OverlayType {
                has_art: Some(true),
                ..Default::default()
            },
        );
    }
    // 16 bridge tiles of 3 × 2 clear cells.
    assert_eq!(s.catalogue.tiles.len() as i32, BRIDGE_ROLE);
    for _ in 0..16 {
        s.catalogue.tiles.push(glasnost_map::terrain::TileInfo {
            width: 3,
            height: 2,
            has_image: true,
            cells: vec![
                Some(glasnost_map::terrain::TileCell {
                    terrain_code: 0,
                    slope: 0,
                });
                6
            ],
            ..Default::default()
        });
    }
    s.catalogue.roles.bridge = BRIDGE_ROLE;
    for c in all_cells(&s) {
        if (cell(1, 0).x..=cell(5, 0).x).contains(&c.x) && !clear.contains(&c) {
            rock(&mut s, c, 0);
        }
    }
    for x in [0, 6] {
        s.set_cell(cell(x, 0), BRIDGE_ROLE + 6, 0);
        s.cell_mut(cell(x, 0)).sub_tile = 4;
    }
    for x in 1..=5 {
        if !gaps.contains(&x) {
            s.cell_mut(cell(x, 0)).overlay = 24;
        }
    }
    s.place_unit(0, CAR, cell(-2, 0), 64);
    let mut w = s.build();
    regions::ensure(&mut w);
    let car = vehicles(&w)[0];
    (w, car)
}

// spec: S-0010 claim 25 (a span from a start cell to the far end by table B; high when every between cell outside the roles' ranges is a high-bridge cell)
// spec: S-0212 claim 3 (one link per span, enabled when high; an enabled link makes the regions of its two end cells adjacent)
#[test]
fn a_high_span_is_an_enabled_link_joining_the_banks() {
    let (w, _) = river(&[]);
    assert_eq!(
        w.cells.links,
        vec![glasnost_sim::cells::BridgeLink {
            first: cell(0, 0),
            second: cell(6, 0),
            along_x: true,
            enabled: true,
        }]
    );
    let z = MovementZone::Normal;
    assert!(matches!(
        regions::reach_in(&w, cell(0, 0), z),
        Reach::Region(_)
    ));
    assert_eq!(
        regions::reach_in(&w, cell(-3, 5), z),
        regions::reach_in(&w, cell(9, -5), z),
        "the link joins the two banks"
    );
    // A high span sets no bridge-body mark.
    assert!(w.cells.bridge_body.iter().all(|&b| !b));
}

// spec: S-0212 claim 13b (the side displacements by the first end's tile place i: 0, 1, 6 to 10 north then south; 3, 4, 11 to 15 east then west; 2 and 5 north-west then south-east)
#[test]
fn link_side_displacement_table() {
    for i in 0..16 {
        let want = match i {
            0 | 1 | 6..=10 => [(0, -1), (0, 1)],
            3 | 4 | 11..=15 => [(1, 0), (-1, 0)],
            _ => [(-1, -1), (1, 1)],
        };
        assert_eq!(corridor::side_displacements(i), Some(want), "place {i}");
    }
    assert_eq!(corridor::side_displacements(16), None);
}

/// Whether a level-0 connection joins the sectors of `a` and `b`.
fn connected(w: &World, a: CellXY, b: CellXY) -> bool {
    let (s, t) = (sector(w, 0, a), sector(w, 0, b));
    w.regions.sectors.levels[0].conns[s as usize]
        .iter()
        .any(|&(x, _)| x == t)
}

// spec: S-0212 claim 25 and table T5 (the empty-radius approach failure's search excludes bridge cells, whatever the class)
#[test]
fn approach_failure_search_skips_bridge_cells() {
    let (mut w, car) = river(&[]);
    // The river bed turns to clear ground, so the cells under the deck
    // share the car's ground reach region.
    let bed: Vec<CellXY> = w
        .cells
        .grid
        .iter()
        .map(|(c, _)| c)
        .filter(|c| (cell(1, 0).x..=cell(5, 0).x).contains(&c.x))
        .collect();
    for &c in &bed {
        w.cells.grid.get_mut(c).expect("cell").land = glasnost_map::terrain::land::CLEAR;
        regions::cell_changed(&mut w, c);
    }
    regions::ensure(&mut w);
    let centre = cell(3, 0);
    assert!(w.cells.is_bridge(centre));
    let got = glasnost_sim::path::nearby(
        &w,
        centre,
        glasnost_sim::path::NearbyUse::Approach { unit: car },
    )
    .expect("a cell off the bridge");
    assert!(!w.cells.is_bridge(got), "{got:?} is a bridge cell");
}

// spec: S-0212 claim 13b (an enabled link joins the sectors of its first end and second end each moved by the same displacement, which the first end's tile alone decides)
#[test]
fn link_side_cells_follow_the_first_end_tile() {
    // Place 6: the cells map north and south of both ends.
    let (w, _) = river(&[]);
    assert!(connected(&w, cell(0, -1), cell(6, -1)));
    assert!(connected(&w, cell(0, 1), cell(6, 1)));
    assert!(!connected(&w, cell(-1, -1), cell(5, -1)));
    // The first end's tile at place 2 (the second end's unchanged): the
    // north-west and south-east neighbours of both ends.
    let (mut w, _) = river(&[]);
    w.cells.grid.get_mut(cell(0, 0)).unwrap().tile = BRIDGE_ROLE + 2;
    regions::rebuild(&mut w);
    assert!(connected(&w, cell(-1, -1), cell(5, -1)));
    assert!(connected(&w, cell(1, 1), cell(7, 1)));
}

// spec: S-0010 claim 25 (a low span: its band of four cells across each between cell that is not a high-bridge cell carries the bridge-body mark and, S-0212 Terms, the span's axis)
// spec: S-0212 claim 3 (a low span's link is not enabled: the banks stay apart)
// spec: S-0010 claim 17 (the bridge-body mark refuses burrowing)
#[test]
fn a_low_span_marks_its_band_and_stays_disabled() {
    use glasnost_sim::cells::Axis;
    let (w, _) = river(&[3]);
    assert_eq!(w.cells.links.len(), 1);
    assert!(!w.cells.links[0].enabled);
    let z = MovementZone::Normal;
    assert_ne!(
        regions::reach_in(&w, cell(-3, 5), z),
        regions::reach_in(&w, cell(9, -5), z)
    );
    let body = |c: CellXY| w.cells.bridge_body[w.cells.idx(c).unwrap()];
    for y in [1, 0, -1, -2] {
        assert!(body(cell(3, y)), "band cell (3, {y})");
        assert_eq!(w.cells.axis(cell(3, y)), Axis::X);
    }
    assert!(!body(cell(3, 2)) && !body(cell(3, -3)));
    assert!(!body(cell(2, 0)), "a high-bridge cell sets no band");
    assert!(!body(cell(0, 0)) && !body(cell(6, 0)), "the end cells");
    assert!(
        !path::burrow_ok(&w, cell(3, 1)),
        "the mark refuses burrowing"
    );
    assert!(path::burrow_ok(&w, cell(3, 2)));
}

// spec: S-0309 claim 7 (revision 21, Q-1502 answered: the searcher's own cell is read for its deck whenever it is a bridge cell, whatever layer it is in; when that read finds no bridge link the connected-region restriction is omitted for the whole search)
#[test]
fn the_search_region_reads_the_searchers_deck() {
    let (mut w, car) = river(&[]);
    let mut c = w.obj(car).unwrap().clone();
    c.pos = glasnost_sim::world::Pos::centre(cell(3, 0), 0);
    let a = glasnost_sim::world::ObjectId(9000);
    w.objects.insert(a, c);
    // The car stands on the near bank; the searcher on the ground under the deck at (3, 0), a cell of no reach region read for its ground.
    assert_eq!(
        regions::reach_in(&w, cell(3, 0), MovementZone::Normal),
        Reach::None
    );
    assert!(
        glasnost_sim::combat::acquire::same_zone(&w, a, car),
        "the deck read through the link gives the bank's region"
    );
    // Off every link's line by 2 cells: the deck read finds no link.
    w.obj_mut(a).unwrap().pos = glasnost_sim::world::Pos::centre(cell(3, -2), 0);
    w.obj_mut(car).unwrap().pos = glasnost_sim::world::Pos::centre(cell(12, 9), 0);
    assert!(
        glasnost_sim::combat::acquire::same_zone(&w, a, car),
        "no-link deck exception: no object is rejected for its region"
    );
}

// spec: S-0212 claim 4 (a bridge cell read for its deck: through the first link within 1 cell; enabled: the first end's reach region; none found: a reach region of its own)
// spec: S-0212 claim 6 (the destination read for its deck when it is a bridge cell: an order onto a deck over the river passes the gate)
#[test]
fn a_deck_is_read_through_its_link() {
    let z = MovementZone::Normal;
    let (mut w, car) = river(&[]);
    let bank = regions::reach_in(&w, cell(0, 0), z);
    assert_eq!(regions::reach_read(&w, cell(3, 0), z, true), bank);
    assert_eq!(regions::reach_read(&w, cell(3, 1), z, true), bank);
    assert_eq!(
        regions::reach_in(&w, cell(3, 0), z),
        Reach::None,
        "its ground"
    );
    // Row −2 lies 2 cells to the side of the link: no link within 1.
    assert_eq!(
        regions::reach_read(&w, cell(3, -2), z, true),
        Reach::LoneDeck
    );
    assert_eq!(
        regions::reach_read(&w, cell(4, -2), z, true),
        regions::reach_read(&w, cell(3, -2), z, true)
    );
    // spec: S-0212 claim 25 (in deck mode a bridge cell's reach region is read for its deck: a move order onto the deck collects the deck cell itself)
    let order = |w: &World| {
        path::nearby(
            w,
            cell(3, 0),
            path::NearbyUse::MoveOrder {
                unit: car,
                on_object: false,
                free_no_move: false,
            },
        )
    };
    assert_eq!(order(&w), Some(cell(3, 0)));
    assert_ne!(
        compute_to(&mut w, car, cell(3, 0), Mode::Fresh),
        Search::GateFailed
    );
    assert_eq!(
        compute_to(&mut w, car, cell(3, -2), Mode::Fresh),
        Search::GateFailed
    );
    // A link that is not enabled: the first cell off the bridge going east
    // (the gap at x = 3) is no bridge end, so the first end's region.
    let (w, _) = river(&[3]);
    assert_eq!(
        regions::reach_read(&w, cell(2, 0), z, true),
        regions::reach_in(&w, cell(0, 0), z)
    );
    // From x = 4 the first cell off the bridge is B, a bridge end: the
    // second end's region.
    assert_eq!(
        regions::reach_read(&w, cell(4, 0), z, true),
        regions::reach_in(&w, cell(6, 0), z)
    );
}

/// The river of [`river`] with the combat test rules: the car of house 0
/// carries the straight-shot `Rifle` (range 4 cells), and an unarmed car of
/// house 1 stands on bank B at (12, 0).
fn armed_river(gaps: &[i32]) -> (World, ObjectId, ObjectId) {
    armed_river_with(gaps, &[], None, cell(12, 0))
}

/// [`armed_river`] with the cells of `clear` left open ground, the Rifle's
/// range set to `range` cells when given, and the van at `van_at`.
fn armed_river_with(
    gaps: &[i32],
    clear: &[CellXY],
    range: Option<&str>,
    van_at: CellXY,
) -> (World, ObjectId, ObjectId) {
    let mut s = scene();
    let c = glasnost_sim::testkit::combat_ruleset();
    s.ruleset.sim.weapons = c.sim.weapons;
    s.ruleset.sim.projectiles = c.sim.projectiles;
    s.ruleset.sim.warheads = c.sim.warheads;
    if let Some(r) = range {
        s.ruleset.sim.weapons.get_mut("Rifle").unwrap().range =
            Some(glasnost_rules::sim::Decimal(r.into()));
    }
    s.ruleset.sim.vehicles.get_mut(CAR).unwrap().primary = Some("Rifle".into());
    s.vehicle("TESTVAN", unit_type(WHEEL, 40, 6));
    s.place_unit(1, "TESTVAN", van_at, 64);
    let (w, _) = river_in_with(s, gaps, clear);
    let of = |h: u8| {
        vehicles(&w)
            .into_iter()
            .find(|&v| w.obj(v).unwrap().owner == Some(glasnost_sim::world::HouseId(h)))
            .unwrap()
    };
    let (car, van) = (of(0), of(1));
    (w, car, van)
}

/// Stands `id` on the deck of bridge cell `c` (S-0307 claim 19: the actor's
/// layer) and gives it `target`.
fn on_deck_attacking(w: &mut World, id: ObjectId, c: CellXY, target: ObjectId) {
    use glasnost_sim::world::{Layer, Pos};
    let o = w.obj_mut(id).unwrap();
    o.pos = Pos::centre(c, 416); // spec: S-0307 Constants (bridge deck height, 416 leptons above the terrain)
    o.layer = Layer::Deck;
    o.target = Some(Target::Object(target));
}

// spec: S-0307 claim 19 (the clear-cell test requires the actor's current reach region, determined from its actual cell and its current ground/deck layer; the candidate's region is the ground-layer region; RG18 run c) and S-0212 claim 4 (a bridge cell read for its deck: through the first link within 1 cell; enabled: the first end's reach region). The lead's play note 37 (M8 A4 round 6): units on a high-bridge deck over water dropped an attack on a building off the bridge.
#[test]
fn an_attacker_on_a_deck_takes_its_reach_region_from_the_deck() {
    let z = MovementZone::Normal;
    let approach = |w: &mut World, id: ObjectId| {
        glasnost_sim::approach::step(w, id);
        let o = w.obj(id).unwrap();
        (o.target, o.dest)
    };
    let (mut w, car, van) = armed_river(&[]);
    on_deck_attacking(&mut w, car, cell(3, 0), van);
    assert_eq!(
        regions::reach_in(&w, cell(3, 0), z),
        Reach::None,
        "its ground"
    );
    assert_eq!(
        regions::reach_read(&w, cell(3, 0), z, true),
        regions::reach_in(&w, cell(12, 0), z),
        "its deck: the banks' region, joined by the enabled link"
    );
    let got = approach(&mut w, car);
    assert!(
        matches!(got, (Some(Target::Object(t)), Some(Target::Cell(c))) if t == van && c.x >= cell(6, 0).x),
        "the target kept, a firing cell on bank B the destination: {got:?}"
    );
    // RG18 run c's control: the regions differ. A gap at x = 3 leaves the
    // link not enabled, the banks two regions; the deck at (2, 0) reads the
    // first end's, bank A's, and bank B's candidates are refused.
    let (mut w, car, van) = armed_river(&[3]);
    on_deck_attacking(&mut w, car, cell(2, 0), van);
    assert_ne!(
        regions::reach_read(&w, cell(2, 0), z, true),
        regions::reach_in(&w, cell(12, 0), z)
    );
    let got = approach(&mut w, car);
    assert_eq!(got.0, None, "the regions differ: the target is dropped");
}

/// Stands `id` on the deck of bridge cell `c`, as [`on_deck_attacking`]
/// does, without a target.
fn on_deck(w: &mut World, id: ObjectId, c: CellXY) {
    use glasnost_sim::world::{Layer, Pos};
    let o = w.obj_mut(id).unwrap();
    o.pos = Pos::centre(c, 416); // spec: S-0307 Constants (bridge deck height, 416 leptons above the terrain)
    o.layer = Layer::Deck;
}

/// Stands `id` on the ground of `c` with `target`.
fn on_ground_attacking(w: &mut World, id: ObjectId, c: CellXY, target: ObjectId) {
    use glasnost_sim::world::{Layer, Pos};
    let o = w.obj_mut(id).unwrap();
    o.pos = Pos::centre(c, 0);
    o.layer = Layer::Ground;
    o.target = Some(Target::Object(target));
}

// spec: S-0307 claim 19 (revision 20, Q-2171, a deck over water: the candidate's reach region is read for its deck, S-0212 claim 4, when its cell carries a high bridge; an attacker on the deck, read for its deck too, or on a bank qualifies a deck cell when the two reads agree; RG18 run e). The lead's play note 37 (M8 A4 round 6): units on a deck dropped an attack on a target on the same deck.
#[test]
fn deck_candidates_are_read_for_their_deck() {
    let z = MovementZone::Normal;
    let approach = |w: &mut World, id: ObjectId| {
        glasnost_sim::approach::step(w, id);
        let o = w.obj(id).unwrap();
        (o.target, o.dest)
    };
    // Run e: the van in the middle of the deck, the Rifle's range 1 cell,
    // so that every primary passing the range test is a deck cell; the car
    // on the same deck, 2 cells away.
    let (mut w, car, van) = armed_river_with(&[], &[], Some("1"), cell(3, 0));
    on_deck(&mut w, van, cell(3, 0));
    on_deck_attacking(&mut w, car, cell(1, 0), van);
    assert_eq!(
        regions::reach_in(&w, cell(3, 0), z),
        Reach::None,
        "the water under the deck"
    );
    let got = approach(&mut w, car);
    assert!(
        matches!(got, (Some(Target::Object(t)), Some(Target::Cell(c))) if t == van && w.high_bridge(c)),
        "the target kept, a deck primary the destination: {got:?}"
    );
    // Repeated with the car on a bank cell next to the bridge's first end.
    let (mut w, car, van) = armed_river_with(&[], &[], Some("1"), cell(3, 0));
    on_deck(&mut w, van, cell(3, 0));
    on_ground_attacking(&mut w, car, cell(-1, 0), van);
    assert_eq!(
        regions::reach_in(&w, cell(-1, 0), z),
        regions::reach_read(&w, cell(3, 0), z, true),
        "the bank lies in the region the deck read gives"
    );
    let got = approach(&mut w, car);
    assert!(
        matches!(got, (Some(Target::Object(t)), Some(Target::Cell(c))) if t == van && w.high_bridge(c)),
        "from the bank too: {got:?}"
    );
}

// spec: S-0307 claim 19 (revision 20, a deck over other ground: an attacker on the ground under the bridge qualifies a high-bridge cell only when its region is the deck's, whatever the region of the cell's ground; RG18 run c, second part)
#[test]
fn a_deck_over_other_ground_is_read_for_its_deck() {
    let z = MovementZone::Normal;
    // Open ground under the deck at x = 2 to 4, walled in by rock: a reach
    // region of its own, not the banks'.
    let island = [cell(2, 0), cell(3, 0), cell(4, 0)];
    let (mut w, car, van) = armed_river_with(&[], &island, Some("1"), cell(4, 0));
    on_deck(&mut w, van, cell(4, 0));
    on_ground_attacking(&mut w, car, cell(2, 0), van);
    let ground = regions::reach_in(&w, cell(3, 0), z);
    assert!(matches!(ground, Reach::Region(_)), "{ground:?}");
    assert_eq!(regions::reach_in(&w, cell(2, 0), z), ground);
    assert_ne!(ground, regions::reach_read(&w, cell(3, 0), z, true));
    glasnost_sim::approach::step(&mut w, car);
    let o = w.obj(car).unwrap();
    assert!(
        !matches!(o.dest, Some(Target::Cell(c)) if w.high_bridge(c)),
        "no deck cell chosen, as a ground read would choose: {:?}",
        o.dest
    );
    assert_eq!(
        o.target, None,
        "every primary is a deck cell: the target is dropped"
    );
}

// spec: S-0215 claim 37 (a1) (revision 91, Q-2171 question 4: the per-tick test is S-0212 claim 6's comparison; the actor's cell read for its deck when it is on a bridge deck, the target's cell whenever that cell carries a high bridge, whichever layer the target is on; B32 h runs 1 to 4)
#[test]
fn the_per_tick_reach_test_reads_decks() {
    use glasnost_sim::approach::reaches;
    let (mut w, car, van) = armed_river_with(&[], &[], None, cell(5, 0));
    // Run 1: the actor and the target on one deck.
    on_deck(&mut w, van, cell(5, 0));
    on_deck_attacking(&mut w, car, cell(1, 0), van);
    assert!(reaches(&w, car, cell(5, 0)), "run 1");
    // Run 4 (control): the actor on the deck, the target's cell beside the
    // bridge with no bridge over it.
    assert!(!w.high_bridge(cell(3, 4)));
    assert!(!reaches(&w, car, cell(3, 4)), "run 4");
    // Runs 2 and 3: the actor on a bank; the target on the deck, or on the
    // ground under it (its cell carries a high bridge: read for its deck).
    on_ground_attacking(&mut w, car, cell(-1, 0), van);
    assert!(reaches(&w, car, cell(5, 0)), "run 2");
    let o = w.obj_mut(van).unwrap();
    o.layer = glasnost_sim::world::Layer::Ground;
    assert!(reaches(&w, car, cell(5, 0)), "run 3");
    // The per-tick rule keeps a reachable target and gives no destination.
    on_deck_attacking(&mut w, car, cell(1, 0), van);
    glasnost_sim::approach::per_tick(&mut w, car);
    let o = w.obj(car).unwrap();
    assert_eq!((o.target, o.dest), (Some(Target::Object(van)), None));
}

// spec: S-0215 claim 37 (a1) and S-0212 claims 4 and 6 (in the comparison two cells of no reach region count as one region; an off-map cell, a cell of no reach region and a reach region all differ)
#[test]
fn the_per_tick_comparison_shares_no_region() {
    use glasnost_sim::approach::reaches;
    let (mut w, car, van) = armed_river_with(&[], &[], None, cell(12, 0));
    let z = MovementZone::Normal;
    // The actor on rock beside the bridge, the target's cell rock too.
    on_ground_attacking(&mut w, car, cell(3, 4), van);
    assert_eq!(regions::reach_in(&w, cell(3, 4), z), Reach::None);
    assert!(reaches(&w, car, cell(3, 6)), "two cells of no reach region");
    assert!(
        !reaches(&w, car, cell(12, 0)),
        "no reach region and a reach region"
    );
}

// spec: S-0215 claim 37 (a1) and S-0212 claim 6 (the exceptions skip the comparison and the target counts as reachable: the actor's cell outside the usable area but a legal cell; in the corridor-disabling state, the actor's cell inside the usable area and the target's a legal cell outside it)
#[test]
fn the_per_tick_comparison_exceptions() {
    use glasnost_sim::approach::reaches;
    let build = |train: bool| {
        let mut s = scene();
        if train {
            s.ruleset.sim.vehicles.get_mut(CAR).unwrap().is_train = Some(true);
        }
        let fp = glasnost_map::Footprint::new(
            common::SIZE,
            common::SIZE,
            [4, 4, common::SIZE - 8, common::SIZE - 8],
        );
        let mut grid = glasnost_map::CellGrid::new(fp);
        for c in all_cells(&s) {
            if let Some(x) = grid.get_mut(c) {
                *x = *s.map.cells.get(c).unwrap();
            }
        }
        s.map.footprint = fp;
        s.map.cells = grid;
        s.set_cell(cell(3, 0), code_tile(9), 0); // water
        s.place_unit(0, CAR, cell(0, 0), 64);
        let mut w = s.build();
        regions::ensure(&mut w);
        let off = all_cells(&s)
            .into_iter()
            .find(|&c| !w.cells.usable(c) && w.cells.cell(c).is_some())
            .expect("a legal cell outside the usable area");
        let id = vehicles(&w)[0];
        (w, id, off)
    };
    let (mut w, id, off) = build(false);
    let z = MovementZone::Normal;
    // Inside the usable area: the comparison decides.
    assert!(!reaches(&w, id, cell(3, 0)), "a reach region and water");
    assert!(!reaches(&w, id, off), "a reach region and an off-map cell");
    // The actor outside the usable area: the target counts as reachable,
    // although the comparison would differ.
    w.obj_mut(id).unwrap().pos = glasnost_sim::world::Pos::centre(off, 0);
    assert!(!regions::compare(&w, z, off, false, cell(3, 0), false));
    assert!(reaches(&w, id, cell(3, 0)), "the second exception");
    // A train type inside the usable area, the target's cell outside it.
    let (w, id, off) = build(true);
    assert!(reaches(&w, id, off), "the third exception");
    assert!(
        !reaches(&w, id, cell(3, 0)),
        "a target cell inside: compared"
    );
}

// spec: S-0212 claim 13b (coarse end cells of a bridge cell: the lane cells; enabled: the nearer, the second end's on equal distances; not enabled: by the first cell off the bridge going east)
#[test]
fn coarse_end_cells_of_bridge_cells() {
    let (w, _) = river(&[]);
    assert_eq!(path::coarse_end(&w, cell(2, 0), true), cell(0, 0));
    assert_eq!(path::coarse_end(&w, cell(3, 0), true), cell(6, 0), "a tie");
    assert_eq!(
        path::coarse_end(&w, cell(4, 1), true),
        cell(6, 1),
        "the lane"
    );
    assert_eq!(path::coarse_end(&w, cell(2, -1), true), cell(0, -1));
    // Not on the deck, or not a bridge cell: the cell itself.
    assert_eq!(path::coarse_end(&w, cell(2, 0), false), cell(2, 0));
    assert_eq!(path::coarse_end(&w, cell(-2, 0), true), cell(-2, 0));
    let (w, _) = river(&[3]);
    assert_eq!(path::coarse_end(&w, cell(2, 0), true), cell(0, 0));
    assert_eq!(path::coarse_end(&w, cell(4, 0), true), cell(6, 0));
    // In row 1 the first cell off the bridge east, (6, 1), holds no bridge
    // tile: the first end's lane cell.
    assert_eq!(path::coarse_end(&w, cell(5, 1), true), cell(0, 1));
}

// spec: S-0212 F8 (both cells bridge cells of one bridge, the first link within 3 cells the same: E is their Chebyshev distance)
// spec: S-0212 claim 24 (E = max(Chebyshev distance, 2n − 2 + legs))
#[test]
fn coarse_estimate_on_one_bridge_is_the_chebyshev_distance() {
    let (w, _) = river(&[]);
    let z = MovementZone::Normal;
    assert_eq!(
        path::coarse_estimate(&w, z, cell(2, 0), cell(5, 1)),
        Some(3)
    );
    // Away from bridges, at least the Chebyshev distance.
    let e = path::coarse_estimate(&w, z, cell(-10, 4), cell(-2, 4)).unwrap();
    assert!(e >= 8, "{e}");
}

// spec: S-0212 claim 13 (the corridor exemption: an aircraft in a neighbour gives it, as a vehicle does)
#[test]
fn an_aircraft_gives_the_corridor_exemption() {
    let mut s = scene();
    s.aircraft(
        "TESTJET",
        unit_type(glasnost_rules::sim::LocomotionClass::Winged, 40, 6),
    );
    s.place_aircraft(0, "TESTJET", cell(4, 4), 0);
    let w = s.build();
    assert!(corridor::exempt(&w, cell(5, 5)));
    assert!(!corridor::exempt(&w, cell(8, 8)));
}

// ------------------------------------------------------------- the corridor

use glasnost_sim::corridor::{self, NONE};

fn sector(w: &World, l: usize, c: CellXY) -> u32 {
    w.regions.sectors.levels[l].sector(w, c)
}

// spec: S-0212 claim 13b (on open ground each block is one sector, with straight connections to the four blocks beside it and diagonal ones to the four at its corners; each sector lies inside its parent)
#[test]
fn sectors_on_open_ground() {
    let mut s = scene();
    s.place_unit(0, CAR, cell(0, 0), 64);
    let mut w = s.build();
    regions::ensure(&mut w);
    for l in 0..corridor::LEVELS {
        let b = corridor::block_size(l);
        assert_eq!(b, 2 << l);
        let a = sector(&w, l, cell(0, 0));
        assert_ne!(a, NONE);
        // The whole block is one sector; the next block is another.
        assert_eq!(sector(&w, l, cell(b - 1, b - 1)), a, "level {l}");
        assert_ne!(sector(&w, l, cell(b, 0)), a, "level {l}");
        let lv = &w.regions.sectors.levels[l];
        let conns = &lv.conns[a as usize];
        assert_eq!(conns.len(), 8, "level {l}");
        for (dx, dy, diagonal) in [
            (b, 0, false),
            (-b, 0, false),
            (0, b, false),
            (0, -b, false),
            (b, b, true),
            (-b, -b, true),
            (b, -b, true),
            (-b, b, true),
        ] {
            let t = sector(&w, l, cell(dx, dy));
            assert!(conns.contains(&(t, diagonal)), "level {l} {dx},{dy}");
        }
        if l + 1 < corridor::LEVELS {
            assert_eq!(lv.parent[a as usize], sector(&w, l + 1, cell(0, 0)));
        }
    }
}

// spec: S-0212 claim 13b (inside a block, cells joined only across a step of 2 levels along cell x are different sectors, with no connection between them)
#[test]
fn sectors_split_by_a_two_level_step() {
    let mut s = scene();
    s.set_cell(cell(1, 0), 0, 2);
    s.set_cell(cell(1, 1), 0, 2);
    s.place_unit(0, CAR, cell(-4, 0), 64);
    let mut w = s.build();
    regions::ensure(&mut w);
    // One region (claim 3 joins along x across 2 levels) ...
    assert_eq!(region(&w, cell(0, 0)), region(&w, cell(1, 0)));
    // ... but two sectors of one level-0 block, not connected.
    let a = sector(&w, 0, cell(0, 0));
    let b = sector(&w, 0, cell(1, 0));
    assert_ne!(a, b);
    let lv = &w.regions.sectors.levels[0];
    assert!(lv.conns[a as usize].iter().all(|&(t, _)| t != b));
}

// spec: S-0212 claim 13 (the coarse search: per-category entry costs plus 0.001 across a diagonal connection, so a diagonal neighbour block is entered directly; F9)
#[test]
fn coarse_search_chains() {
    let mut s = scene();
    s.place_unit(0, CAR, cell(0, 0), 64);
    let mut w = s.build();
    regions::ensure(&mut w);
    let secs = w.regions.sectors.clone();
    let chains = corridor::coarse(
        &w,
        &secs,
        MovementZone::Normal,
        cell(0, 0),
        cell(2, 2),
        &Default::default(),
    )
    .expect("a corridor");
    // Level 0: the start's block, then the diagonal block (1.001 < 2).
    assert_eq!(
        chains[0],
        vec![sector(&w, 0, cell(0, 0)), sector(&w, 0, cell(2, 2))]
    );
    // Levels 1 and 2: start and destination share a sector.
    assert_eq!(chains[1], vec![sector(&w, 1, cell(0, 0))]);
    assert_eq!(chains[2], vec![sector(&w, 2, cell(0, 0))]);
    // Straight across three blocks.
    let chains = corridor::coarse(
        &w,
        &secs,
        MovementZone::Normal,
        cell(0, 0),
        cell(4, 0),
        &Default::default(),
    )
    .expect("a corridor");
    assert_eq!(
        chains[0],
        vec![
            sector(&w, 0, cell(0, 0)),
            sector(&w, 0, cell(2, 0)),
            sector(&w, 0, cell(4, 0))
        ]
    );
    // Every level-0 corridor sector's parent is on the level-1 corridor.
    let lv0 = &secs.levels[0];
    for &x in &chains[0] {
        assert!(chains[1].contains(&lv0.parent[x as usize]));
    }
    // A zone that cannot use the destination's category: no corridor.
    assert!(
        corridor::coarse(
            &w,
            &secs,
            MovementZone::Water,
            cell(0, 0),
            cell(4, 0),
            &Default::default(),
        )
        .is_none()
    );
}

// spec: S-0212 claim 13 (a cell search with a corridor considers ground cells of corridor sectors only, apart from cells with the corridor exemption)
#[test]
fn corridor_confines_the_search() {
    let mut s = scene();
    s.place_unit(0, CAR, cell(0, 0), 64);
    let mut w = s.build();
    regions::ensure(&mut w);
    let id = vehicles(&w)[0];
    let to = cell(9, 4);
    let p = path::corridor_find(&w, id, to).expect("a path");
    assert_eq!(*visits(cell(0, 0), &p).last().unwrap(), to);
    let secs = w.regions.sectors.clone();
    let chains = corridor::coarse(
        &w,
        &secs,
        MovementZone::Normal,
        cell(0, 0),
        to,
        &Default::default(),
    )
    .unwrap();
    for c in visits(cell(0, 0), &p) {
        assert!(
            chains[0].contains(&sector(&w, 0, c)) || corridor::exempt(&w, c),
            "{c:?} off the corridor"
        );
    }
    // The cells next to the unit have the exemption: it is listed in its cell.
    assert!(corridor::exempt(&w, cell(1, 1)));
    assert!(!corridor::exempt(&w, cell(5, 5)));
}

// spec: S-0212 claims 13 and 15 (the corridor exemption and the entry outcomes are read from the world of the computation; a blocked destination is approached from a neighbour) and claim 13a (quirks.corridor_free_part_ignores_height off: a ledge stops the free part, so the retries lead round it); our tooling for speed (M8 C4 part 5): a path computation keeps what its searches asked of the world, and nothing it kept reaches a later computation on a changed world, with another destination, or in a copy of the world
#[test]
fn a_computation_keeps_nothing_for_the_next() {
    for (barrier, held) in [(false, false), (true, false), (false, true), (true, true)] {
        let mut s = scene();
        if barrier {
            // A ledge: flat clear cells one level up with no slope, which
            // the coarse search connects to the ground round it (claim
            // 13b) and the height rule closes to every step (claim 10).
            // The first corridor runs into it and, with
            // `quirks.corridor_free_part_ignores_height` off, the
            // computation retries round it (claim 13a). A wall of huts no
            // longer makes it retry: their exemption rectangles let the
            // first cell search round it (claim 13).
            for y in -6..=5 {
                s.set_cell(cell(5, y), 0, 1);
            }
            s.ruleset.globals.quirks.corridor_free_part_ignores_height = false;
        }
        let to = cell(9, 4);
        if held {
            // An enemy hut on the destination: blocked, so the path ends
            // next to it (claim 15).
            s.place_structure(1, HUT, to);
        }
        s.place_unit(0, CAR, cell(0, 0), 64);
        let mut w = s.build();
        regions::ensure(&mut w);
        let id = vehicles(&w)[0];
        let car = w.rules().index_of(Kind::Vehicle, CAR).unwrap();
        let mut paths = std::collections::BTreeSet::new();
        let mut off_corridor = 0;
        let mut most_searches = 0;
        for y in -3..=8 {
            for x in -1..=10 {
                let at = cell(x, y);
                if at == cell(0, 0) || at == to || (barrier && x == 5 && y <= 5) {
                    continue;
                }
                // A stone (a second car) in `at` gives its neighbours the
                // exemption and blocks its cell, in the world the
                // computations before it read without it.
                let owner = w.obj(id).unwrap().owner;
                let stone = glasnost_sim::objects::create(&mut w, Kind::Vehicle, car, owner);
                glasnost_sim::objects::place(
                    &mut w,
                    stone,
                    glasnost_sim::world::Pos::centre(at, 0),
                    0,
                );
                regions::ensure(&mut w);
                // A copy starts with empty working memory; `w` keeps what
                // its last computation asked.
                let fresh = w.clone();
                // Another destination first: the computations between differ
                // in their goal too.
                let other = cell(8, -3);
                assert_eq!(
                    path::corridor_find(&w, id, other),
                    path::corridor_find(&fresh, id, other),
                    "stone in {at:?}, the other destination"
                );
                let kept = path::corridor_find(&w, id, to);
                assert_eq!(kept, path::corridor_find(&fresh, id, to), "stone in {at:?}");
                // The stone goes: `w` answers the changed world afresh.
                glasnost_sim::objects::remove(&mut w, stone);
                regions::ensure(&mut w);
                let plain = w.clone();
                assert_eq!(
                    path::corridor_find(&w, id, to),
                    path::corridor_find(&plain, id, to),
                    "stone gone from {at:?}"
                );
                if let Some(p) = kept {
                    let cells = visits(cell(0, 0), &p);
                    if held {
                        // spec: S-0212 claim 15 (the path ends next to the blocked destination)
                        assert_eq!(chebyshev(*cells.last().unwrap(), to), 1, "stone in {at:?}");
                    }
                    let secs = fresh.regions.sectors.clone();
                    let chains = corridor::coarse(
                        &fresh,
                        &secs,
                        MovementZone::Normal,
                        cell(0, 0),
                        to,
                        &Default::default(),
                    );
                    off_corridor += cells
                        .into_iter()
                        .filter(|&c| {
                            chains.as_ref().is_some_and(|ch| {
                                !ch[0].contains(&sector(&fresh, 0, c))
                                    && corridor::exempt(&fresh, c)
                            })
                        })
                        .count();
                    paths.insert(p);
                }
                // How many cell searches the computation made (retries).
                let mut probe = fresh.clone();
                glasnost_sim::trace::switch_on(&mut probe);
                compute_to(&mut probe, id, to, Mode::Fresh);
                for e in glasnost_sim::trace::take(&probe) {
                    if let glasnost_sim::trace::What::Search { searches, .. } = e.what {
                        most_searches = most_searches.max(searches.len());
                    }
                }
            }
        }
        let case = format!("barrier {barrier}, held {held}");
        assert!(paths.len() >= 2, "{case}: the stones change the answer");
        assert!(off_corridor > 0, "{case}: some path uses the exemption");
        assert!(
            most_searches > usize::from(barrier),
            "{case}: the barrier makes the computation retry: {most_searches}"
        );
    }
}

// spec: S-0212 claim 14 (the corridor is not used in the corridor-disabling state: a unit in retreat, or a train)
#[test]
fn corridor_use() {
    let mut s = scene();
    s.place_unit(0, CAR, cell(0, 0), 64);
    let mut w = s.build();
    let id = vehicles(&w)[0];
    let to = cell(9, 4);
    regions::ensure(&mut w);
    let with = path::corridor_find(&w, id, to).expect("a path");
    let without = find(&w, id, to, Mode::Fresh, &[]).expect("a path");
    assert_ne!(with, without, "the corridor shapes this path");
    // A path computation uses the corridor ...
    assert_eq!(compute_to(&mut w, id, to, Mode::Fresh), Search::Found);
    let stored = w.obj(id).unwrap().path.steps.clone();
    // ... and in retreat it does not.
    w.obj_mut(id).unwrap().beh.current = Some(Behaviour::Retreat);
    assert_eq!(compute_to(&mut w, id, to, Mode::Fresh), Search::Found);
    assert_eq!(w.obj(id).unwrap().path.steps, without);
    assert_ne!(stored, without);
}

// spec: S-0212 claim 13a (a whole progress sector closes its connections to every sector its free part does not border)
#[test]
fn retry_closes_unbordered_connections() {
    let mut s = scene();
    // The level-0 block east of the unit's block is full of enemy buildings,
    // which give an unarmed vehicle the outcome blocked.
    for (dx, dy) in [(2, 0), (3, 0), (2, 1), (3, 1)] {
        s.place_structure(1, HUT, cell(dx, dy));
    }
    s.place_unit(0, CAR, cell(0, 0), 64);
    let mut w = s.build();
    regions::ensure(&mut w);
    let id = vehicles(&w)[0];
    let secs = w.regions.sectors.clone();
    let me = sector(&w, 0, cell(0, 0));
    let east = sector(&w, 0, cell(2, 0));
    let mut closed = Default::default();
    let chain = vec![me, east];
    assert!(corridor::retry(
        &w,
        &secs,
        id,
        MovementZone::Normal,
        0,
        cell(0, 0),
        &chain,
        &mut closed,
    ));
    let pair = (me.min(east), me.max(east));
    assert!(closed.contains(&pair), "{closed:?}");
    assert_eq!(closed.len(), 1, "{closed:?}");
}

// spec: S-0212 claim 24 (a substitute is adopted only when the coarse estimate E is at most the Chebyshev distance plus 6)
#[test]
fn substitution_estimate() {
    for wall in [false, true] {
        let mut s = scene();
        // The destination cell(4, 0) holds an idle friendly car and its eight
        // neighbours are rock or taken, so the nearby search finds cell(2, 0).
        for c in [
            cell(3, -1),
            cell(3, 0),
            cell(3, 1),
            cell(4, -1),
            cell(4, 1),
            cell(5, -1),
            cell(5, 1),
        ] {
            rock(&mut s, c, 0);
        }
        if wall {
            // A long wall west of the destination: the coarse route from
            // cell(2, 0) goes round it.
            for y in -12..=11 {
                rock(&mut s, cell(3, y), 0);
            }
        }
        s.place_unit(0, CAR, cell(0, 0), 64);
        s.place_unit(0, CAR, cell(4, 0), 64);
        s.place_unit(0, CAR, cell(5, 0), 64);
        let mut w = s.build();
        let id = vehicles(&w)
            .into_iter()
            .find(|&v| w.obj(v).unwrap().pos.cell() == cell(0, 0))
            .unwrap();
        compute_to(&mut w, id, cell(4, 0), Mode::Fresh);
        let goal = w.obj(id).unwrap().path.for_goal;
        if wall {
            assert_eq!(goal, Some(cell(4, 0)), "not adopted behind the wall");
        } else {
            assert_eq!(goal, Some(cell(2, 0)), "adopted");
        }
    }
}

// spec: S-0212 claim 12a (coarse routes of exactly equal cost: the one recorded first goes first, and a route's new routes are recorded in the order of their sectors' first cells)
#[test]
fn coarse_tie_goes_to_the_first_recorded() {
    let mut s = scene();
    // Rock on the corner where the level-0 blocks of cell(0, 0) and
    // cell(2, 2) touch: no diagonal connection between them, so the routes
    // through the block east (first cell on an earlier row) and the block
    // south both cost exactly 2.
    rock(&mut s, cell(2, 2), 0);
    s.place_unit(0, CAR, cell(0, 0), 64);
    let mut w = s.build();
    regions::ensure(&mut w);
    let secs = w.regions.sectors.clone();
    // One level-1 sector holds all four level-0 blocks.
    assert_eq!(sector(&w, 1, cell(0, 0)), sector(&w, 1, cell(3, 3)));
    let a = sector(&w, 0, cell(0, 0));
    let d = sector(&w, 0, cell(3, 3));
    assert!(
        secs.levels[0].conns[a as usize]
            .iter()
            .all(|&(t, _)| t != d)
    );
    let chains = corridor::coarse(
        &w,
        &secs,
        MovementZone::Normal,
        cell(0, 0),
        cell(3, 3),
        &Default::default(),
    )
    .expect("a corridor");
    assert_eq!(chains[0], vec![a, sector(&w, 0, cell(2, 0)), d]);
}

// spec: S-0212 claim 23 row 2 (a unit on the ground within 1 cell of a bridge-cell destination gives up all the same)
#[test]
fn search_failure_next_to_a_bridge_cell_gives_up() {
    let mut s = scene();
    s.place_unit(0, CAR, cell(0, 0), 64);
    let mut w = s.build();
    lay_deck(&mut w, cell(1, 1), true);
    let id = vehicles(&w)[0];
    {
        let o = w.obj_mut(id).unwrap();
        o.beh.queued = None;
        o.dest = Some(Target::Cell(cell(1, 1)));
    }
    path::failed(&mut w, id, Search::SearchFailed, false);
    let o = w.obj(id).unwrap();
    assert_eq!(o.dest, None);
    assert_eq!(o.beh.queued, Some(Behaviour::Guard));
}

// spec: S-0212 claim 23 row 2 and Departure D5 (a unit of a house that is not human-controlled, the neutral or special house, queues area guard when it gives up; an AI house counts as human-controlled and queues guard)
#[test]
fn search_failure_of_a_neutral_unit_queues_area_guard() {
    for (human, want) in [(false, Behaviour::AreaGuard), (true, Behaviour::Guard)] {
        let mut s = scene();
        s.place_unit(0, CAR, cell(0, 0), 64);
        let mut w = s.build();
        w.houses[0].human = human;
        let id = vehicles(&w)[0];
        {
            let o = w.obj_mut(id).unwrap();
            o.beh.queued = None;
            o.dest = Some(Target::Cell(cell(5, 0)));
        }
        // spec: S-0215 claim 38 (the neutral or special house's centre choice, then the final search on the same tick)
        let mut v = w.clone();
        let expected = if human {
            None
        } else {
            let c = glasnost_sim::base::selected_centre(&mut v, w.obj(id).unwrap().owner.unwrap());
            c.and_then(|c| path::give_up_destination(&mut v, id, c))
        };
        path::failed(&mut w, id, Search::SearchFailed, false);
        let o = w.obj(id).unwrap();
        assert_eq!(o.dest, expected.map(Target::Cell), "human {human}");
        assert_eq!(format!("{:?}", w.rng), format!("{:?}", v.rng));
        assert_eq!(o.beh.queued, Some(want), "human {human}");
        assert_ne!(o.beh.current, Some(Behaviour::Move));
    }
}

// spec: S-0212 claim 23a and T5 (the house-selected destination is a nearby-cell result around the centre: track, normal, the reach region of the unit's ground cell, no deck mode; the search takes no draw)
#[test]
fn give_up_destination_is_a_nearby_cell() {
    let mut s = scene();
    s.place_unit(0, CAR, cell(0, 0), 64);
    s.place_unit(0, CAR, cell(5, 0), 64);
    let mut w = s.build();
    let id = vehicles(&w)[0];
    let rng = w.rng.clone();
    // A clear centre is its own result.
    assert_eq!(
        path::give_up_destination(&mut w, id, cell(3, 3)),
        Some(cell(3, 3))
    );
    // An occupied centre gives a nearby cell, not the centre itself.
    let c = path::give_up_destination(&mut w, id, cell(5, 0)).unwrap();
    assert_ne!(c, cell(5, 0));
    let o5 = cell(5, 0);
    assert!((c.x - o5.x).abs().max((c.y - o5.y).abs()) <= 1, "{c:?}");
    assert_eq!(w.rng, rng);
}

/// A north–south ledge of `2 × half + 1` flat clear cells one level up at
/// x = 0, shifted `shift` cells south, the car 12 cells west of it and its
/// destination 12 cells east, under the two switches as given: the result
/// of the path computation and, for each of its cell searches, whether it
/// ran inside a corridor and whether it found the path.
fn across_a_ledge(
    half: i32,
    shift: i32,
    ignores_height: bool,
    no_fallback: bool,
) -> (Search, Vec<(bool, bool)>) {
    let mut s = scene();
    for y in -half..=half {
        s.set_cell(cell(0, y + shift), 0, 1);
    }
    s.ruleset.globals.quirks.corridor_free_part_ignores_height = ignores_height;
    s.ruleset.globals.quirks.corridor_retry_no_fallback = no_fallback;
    s.place_unit(0, CAR, cell(-12, shift), 64);
    let mut w = s.build();
    regions::ensure(&mut w);
    let id = vehicles(&w)[0];
    glasnost_sim::trace::switch_on(&mut w);
    let r = compute_to(&mut w, id, cell(12, shift), Mode::Fresh);
    let mut n = Vec::new();
    for e in glasnost_sim::trace::take(&w) {
        if let glasnost_sim::trace::What::Search { searches, .. } = e.what {
            n = searches
                .iter()
                .map(|s| (s.corridor, s.found.is_some()))
                .collect();
        }
    }
    (r, n)
}

// spec: S-0212 Edge cases (a ledge: open to the coarse searches, refused by the cell search's height rule, claim 10; the free part of claim 13a ignores height, so a retry closes nothing, each new coarse search finds the same corridor, and the 5th cell search inside it fails the computation whatever quirks.corridor_retry_no_fallback is set to; the 7-ledges fail at every shift) and claim 13 (the limit of 5 cell searches)
#[test]
fn a_ledge_reaches_the_limit_of_five_searches() {
    for half in [1, 3] {
        for shift in 0..8 {
            for no_fallback in [true, false] {
                let (r, n) = across_a_ledge(half, shift, true, no_fallback);
                let case = format!(
                    "ledge {} shift {shift} no_fallback {no_fallback}",
                    2 * half + 1
                );
                assert_eq!(r, Search::SearchFailed, "{case}: {n:?}");
                assert_eq!(n, vec![(true, false); 5], "{case}");
            }
        }
    }
}

// spec: S-0212 Edge cases (quirks.corridor_free_part_ignores_height off: the free part and its borders apply the height rule of claim 10, so a ledge stops the free part as a strip does and the retries close connections and lead a new coarse search round it) and claim 13a
#[test]
fn corridor_free_part_ignores_height_off_goes_round_a_ledge() {
    for half in [1, 3] {
        for shift in 0..8 {
            for no_fallback in [true, false] {
                let (r, n) = across_a_ledge(half, shift, false, no_fallback);
                let case = format!(
                    "ledge {} shift {shift} no_fallback {no_fallback}",
                    2 * half + 1
                );
                assert_eq!(r, Search::Found, "{case}: {n:?}");
                let (last, before) = n.split_last().unwrap();
                assert_eq!(*last, (true, true), "{case}: found inside a corridor");
                assert!(before.iter().all(|&s| s == (true, false)), "{case}: {n:?}");
                assert!(n.len() < 5, "{case}: before the limit");
            }
        }
    }
}

// spec: S-0212 Edge cases (quirks.corridor_free_part_ignores_height off: a ledge stops the free part as a strip does and the retries lead a new coarse search round it) and claim 13 (the limit of 5 cell searches); M8 step C7 part 3: the play-fixes layer sets the switch off, read from the layer file, so a ledge across the route is gone round and not given up at the 5th search
#[test]
fn play_fixes_layer_goes_round_a_ledge() {
    for half in [1, 3] {
        for shift in 0..8 {
            for layered in [false, true] {
                let mut s = scene();
                for y in -half..=half {
                    s.set_cell(cell(0, y + shift), 0, 1);
                }
                if layered {
                    let q = s.ruleset.globals.quirks;
                    s.ruleset.globals.quirks = glasnost_rules::layers::Layers::PLAY_FIXES
                        .apply_quirks(q)
                        .unwrap();
                }
                s.place_unit(0, CAR, cell(-12, shift), 64);
                let mut w = s.build();
                regions::ensure(&mut w);
                let id = vehicles(&w)[0];
                let r = compute_to(&mut w, id, cell(12, shift), Mode::Fresh);
                let case = format!("ledge {} shift {shift} layered {layered}", 2 * half + 1);
                if layered {
                    assert_eq!(r, Search::Found, "{case}");
                } else {
                    assert_eq!(r, Search::SearchFailed, "{case}");
                }
            }
        }
    }
}

/// The tracked test vehicle of the strip tests.
const TANK: &str = "TESTTANK";

/// A north–south strip of `2 × half + 1` cells of rough ground at x = 6,
/// from y = 10 + `shift` − `half` to 10 + `shift` + `half`, whose land row
/// has multiplier 0 for the track class only (so the category, which reads
/// the wheel column, is *open*: S-0211 claim 11), the tracked TESTTANK at
/// (−10, 10) and its destination (8, 10), 2 cells behind the strip, under
/// the two switches as given: the result of the path computation and, for
/// each of its cell searches, whether it ran inside a corridor and whether
/// it found the path.
fn across_a_strip(
    half: i32,
    shift: i32,
    no_fallback: bool,
    split: bool,
) -> (Search, Vec<(bool, bool)>) {
    let mut s = scene();
    s.vehicle(TANK, unit_type(common::TRACK, 40, 6));
    let one = || Some(Decimal("1".into()));
    let row = LandRow {
        foot: one(),
        track: Some(Decimal("0".into())),
        wheel: one(),
        hover: one(),
        float: one(),
        amphibious: one(),
        float_beach: one(),
        buildable: None,
    };
    s.ruleset.sim.land_types.insert(LandType::Rough, row);
    for y in -half..=half {
        // The rough terrain code of the scene's tile catalogue.
        s.set_cell(cell(6, 10 + y + shift), code_tile(14), 0);
    }
    s.ruleset.globals.quirks.corridor_retry_no_fallback = no_fallback;
    s.ruleset.globals.quirks.corridor_split_probe = split;
    s.place_unit(0, TANK, cell(-10, 10), 64);
    let mut w = s.build();
    regions::ensure(&mut w);
    let id = vehicles(&w)[0];
    glasnost_sim::trace::switch_on(&mut w);
    let r = compute_to(&mut w, id, cell(8, 10), Mode::Fresh);
    let mut n = Vec::new();
    for e in glasnost_sim::trace::take(&w) {
        if let glasnost_sim::trace::What::Search { searches, .. } = e.what {
            n = searches
                .iter()
                .map(|s| (s.corridor, s.found.is_some()))
                .collect();
        }
    }
    (r, n)
}

/// The (length, shift) pairs of strips of 3, 5, 7 and 9 cells at shifts 0
/// to 7 whose path computation fails under the two switches.
fn failing_strips(no_fallback: bool, split: bool) -> Vec<(i32, i32)> {
    let mut fails = Vec::new();
    for half in 1..=4 {
        for shift in 0..8 {
            let (r, n) = across_a_strip(half, shift, no_fallback, split);
            if r != Search::Found {
                assert_eq!(r, Search::SearchFailed);
                // Cell searches inside corridors fail until a new coarse
                // search after a retry fails, and with it the computation
                // (claim 13's table), before the limit of 5.
                assert!(
                    n.len() < 5 && n.iter().all(|&s| s == (true, false)),
                    "{} {shift}: {n:?}",
                    2 * half + 1
                );
                fails.push((2 * half + 1, shift));
            }
        }
    }
    fails
}

// spec: S-0212 Edge cases (a short strip of ground the unit cannot enter, open to the coarse searches through the wheel column: strips of 3 and 5 are passed, and some alignment of the 7- and 9-strips fails at once, the new coarse search after a retry failing at level 1 while the level-2 corridor stays the same; which shifts fail is not predicted, so these are this synthetic geometry's) and claim 13 (the table: a repeated coarse search that fails fails the computation), scenario P12 run T
#[test]
fn a_strip_fails_at_some_alignments() {
    assert_eq!(failing_strips(true, true), vec![(7, 0), (7, 1), (9, 0)]);
}

// spec: S-0212 Edge cases (quirks.corridor_retry_no_fallback off: when the new coarse search after a retry fails, one more cell search without a corridor, so an open way round a short strip is found at every alignment)
#[test]
fn corridor_retry_no_fallback_off_goes_round_a_strip() {
    assert_eq!(failing_strips(false, true), Vec::<(i32, i32)>::new());
    // With quirks.corridor_split_probe off as well (the play-fixes layer).
    assert_eq!(failing_strips(false, false), Vec::<(i32, i32)>::new());
    for (half, shift) in [(3, 0), (3, 1), (4, 0)] {
        let (r, n) = across_a_strip(half, shift, false, true);
        assert_eq!(r, Search::Found);
        let (last, before) = n.split_last().unwrap();
        assert_eq!(*last, (false, true), "found without a corridor: {n:?}");
        assert!(before.iter().all(|&s| s == (true, false)), "{n:?}");
    }
}

// spec: S-0212 Edge cases (quirks.corridor_split_probe off: every block that holds a strip cell reads as split whatever the probe cell, so the failure is expected at fewer alignments) and claim 13a
#[test]
fn corridor_split_probe_off_fails_at_fewer_alignments() {
    let on = failing_strips(true, true);
    let off = failing_strips(true, false);
    assert!(off.len() < on.len(), "{off:?} against {on:?}");
    assert!(off.iter().all(|f| on.contains(f)), "{off:?} against {on:?}");
}
