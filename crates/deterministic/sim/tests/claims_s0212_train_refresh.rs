//! S-0212 claim 20, the train rule (#263): "For a train type, the step in
//! progress is kept on purpose: the search starts where that step leads and
//! its result is stored after it, still up to 24 steps in total, so the
//! quirk does not arise." Scenario P9's train refresh run on synthetic data:
//! the flat test map of `tests/common`, an invented train type TESTTRAIN and
//! car type TESTCAR, and an invented wall overlay TESTFENCE at index 0.

mod common;

use common::unit_type;
use common::{SIZE, Scene, WHEEL, cell, objects_of, open_orders, order_move};
use glasnost_map::CellXY;
use glasnost_rules::sim::{OverlayType, UnitType};
use glasnost_sim::rules::Kind;
use glasnost_sim::world::World;

const CAR: &str = "TESTCAR";
const TRAIN: &str = "TESTTRAIN";

type Xy = (i32, i32);

fn at(c: Xy) -> CellXY {
    cell(c.0, c.1)
}

fn xy(c: CellXY) -> Xy {
    (c.x - SIZE, c.y - SIZE)
}

/// The start row: the unit stands in (0, `START`) facing north (−y). Column
/// 0 of the test map is legal from (0, 39) to (0, −39).
const START: i32 = 30;

/// The cell in column `x` of P9's row `k`, counted north from the start.
fn row(x: i32, k: i32) -> Xy {
    (x, START - k)
}

/// Makes `c` a wall cell at play time (overlay index 0, TESTFENCE).
fn wall(w: &mut World, c: CellXY) {
    w.cells.grid.get_mut(c).unwrap().overlay = 0;
    glasnost_sim::load::recalc_cells(w, &[c]);
    glasnost_sim::regions::cell_changed(w, c);
}

/// What [`drive`] saw.
#[derive(Debug)]
struct Run {
    /// The cells the unit stood in, in order.
    cells: Vec<Xy>,
    /// The longest stored path seen.
    longest: usize,
    /// Whether the move ended with no destination.
    ended: bool,
}

/// A unit of type `ty` facing north at row 0 is ordered `to_row` rows north.
/// Once its path is stored, walls are built on `walls` (P9: after the
/// order). Returns the cells it stood in.
fn drive(ty: &str, keeps_old_step: bool, to_row: i32, walls: &[Xy]) -> Run {
    let mut s = Scene::new();
    s.vehicle(CAR, unit_type(WHEEL, 40, 6));
    s.vehicle(
        TRAIN,
        UnitType {
            is_train: Some(true),
            ..unit_type(WHEEL, 40, 6)
        },
    );
    s.register("overlays", "TESTFENCE");
    s.ruleset.sim.overlays.insert(
        "TESTFENCE".into(),
        OverlayType {
            wall: Some(true),
            ..Default::default()
        },
    );
    s.map.has_overlay_data = true;
    s.ruleset.globals.quirks.path_refresh_keeps_old_step = keeps_old_step;
    s.place_unit(0, ty, at(row(0, 0)), 0);
    let mut w = s.build();
    open_orders(&mut w);
    let id = objects_of(&w, Kind::Vehicle)[0];
    order_move(&mut w, id, at(row(0, to_row)));
    let mut cells = vec![row(0, 0)];
    let mut longest = 0;
    let mut built = walls.is_empty();
    let mut ended = false;
    for _ in 0..6000 {
        glasnost_sim::tick::step(&mut w);
        if !built && w.obj(id).is_some_and(|o| !o.path.steps.is_empty()) {
            for &c in walls {
                wall(&mut w, at(c));
            }
            built = true;
        }
        let o = w.obj(id).unwrap();
        let c = xy(o.pos.cell());
        if cells.last() != Some(&c) {
            cells.push(c);
        }
        longest = longest.max(o.path.steps.len());
        let track = o.drive.as_ref().and_then(|d| d.track.as_ref()).is_some();
        if o.dest.is_none() && !track {
            ended = true;
            break;
        }
    }
    assert!(built, "no path was stored: {cells:?}");
    Run {
        cells,
        longest,
        ended,
    }
}

/// P9's walls: (x − 1, row 25), (x, row 25) and (x + 1, row 25), x = 0.
fn p9_walls() -> [Xy; 3] {
    [row(-1, 25), row(0, 25), row(1, 25)]
}

/// The cell entered right after `c`, if `c` was left.
fn after(cells: &[Xy], c: Xy) -> Option<Xy> {
    let i = cells.iter().position(|&x| x == c)?;
    cells.get(i + 1).copied()
}

// spec: S-0212 claim 20 (a train keeps the step in progress; the search starts where that step leads and its result is stored after it) and scenario P9's train refresh run (the train enters (x, row 24) from row 23, steps sideways from there and passes the walls without a refused step; falsified if it does not enter (x, row 24) from row 23), in both positions of quirks.path_refresh_keeps_old_step, which the rule says does not arise for a train
#[test]
fn issue_263_a_train_keeps_the_step_in_progress() {
    for keeps_old_step in [true, false] {
        let r = drive(TRAIN, keeps_old_step, 30, &p9_walls());
        let k = keeps_old_step;
        assert_eq!(after(&r.cells, row(0, 23)), Some(row(0, 24)), "{k}: {r:?}");
        let side = after(&r.cells, row(0, 24));
        assert!(
            side == Some(row(1, 24)) || side == Some(row(-1, 24)),
            "{k}: no sideways step from (x, row 24): {r:?}"
        );
        for c in p9_walls() {
            assert!(!r.cells.contains(&c), "{k}: entered a wall {c:?}: {r:?}");
        }
        assert!(r.ended, "{k}: {r:?}");
        assert_eq!(r.cells.last(), Some(&row(0, 30)), "{k}: {r:?}");
    }
}

// spec: S-0212 claim 20 and scenario P9's train refresh run, control (an ordinary vehicle never enters (x, row 24), goes diagonally to (x + 1, row 24) and does not give up the move), in both positions of quirks.path_refresh_keeps_old_step
#[test]
fn issue_263_a_car_at_the_p9_walls_turns_away_from_the_old_step() {
    for keeps_old_step in [true, false] {
        let r = drive(CAR, keeps_old_step, 30, &p9_walls());
        let k = keeps_old_step;
        assert!(!r.cells.contains(&row(0, 24)), "{k}: {r:?}");
        let next = after(&r.cells, row(0, 23));
        assert!(
            next == Some(row(1, 24)) || next == Some(row(-1, 24)),
            "{k}: not a diagonal step into row 24: {r:?}"
        );
        assert!(r.ended, "{k}: {r:?}");
        assert_eq!(r.cells.last(), Some(&row(0, 30)), "{k}: {r:?}");
    }
}

// spec: S-0212 claim 20 (a train's path after the recomputation: the step in progress and the search's result, still up to 24 steps in total) and Constants (stored path length 24 steps)
#[test]
fn issue_263_a_trains_stored_path_stays_within_24_steps() {
    for keeps_old_step in [true, false] {
        let r = drive(TRAIN, keeps_old_step, 60, &[]);
        let k = keeps_old_step;
        assert!(r.longest <= 24, "{k}: {} steps stored: {r:?}", r.longest);
        let straight: Vec<Xy> = (0..=60).map(|i| row(0, i)).collect();
        assert_eq!(r.cells, straight, "{k}");
        assert!(r.ended, "{k}: {r:?}");
    }
}
