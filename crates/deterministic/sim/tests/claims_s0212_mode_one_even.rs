//! S-0212 claim 19 (#306): a mode-1 search runs as mode 0 only when part
//! (i) counts no cell; with every count even it stays mode 1 with no marked
//! cell. Synthetic types and map from the shared `common` helpers (TESTCAR,
//! TESTSLOW); every name and number is made up.

mod common;

use common::unit_type;
use common::{Scene, WHEEL, cell, objects_of};
use glasnost_map::CellXY;
use glasnost_sim::path::{self, Mode, Search};
use glasnost_sim::rules::Kind;
use glasnost_sim::trace::{self, What};
use glasnost_sim::world::{ObjectId, Target, World};

const CAR: &str = "TESTCAR";
const SLOW: &str = "TESTSLOW";

/// T (TESTCAR, speed 40) at (0, 0) facing east; O (TESTSLOW, speed 20) at
/// (1, 0), standing with two stored steps east; with `third`, a parked
/// TESTCAR at (3, 0), the one cell O's remaining stored step leads through.
fn scene(third: bool) -> (World, ObjectId) {
    let mut s = Scene::new();
    s.vehicle(CAR, unit_type(WHEEL, 40, 6));
    s.vehicle(SLOW, unit_type(WHEEL, 20, 6));
    s.place_unit(0, CAR, cell(0, 0), 64);
    s.place_unit(0, SLOW, cell(1, 0), 64);
    if third {
        s.place_unit(0, CAR, cell(3, 0), 64);
    }
    let mut w = s.build();
    let at = |w: &World, c: CellXY| {
        objects_of(w, Kind::Vehicle)
            .into_iter()
            .find(|&v| w.obj(v).unwrap().pos.cell() == c)
            .unwrap()
    };
    let t = at(&w, cell(0, 0));
    let o = at(&w, cell(1, 0));
    let ob = w.obj_mut(o).unwrap();
    ob.dest = Some(Target::Cell(cell(-20, 10)));
    ob.path.retry_at = i32::MAX;
    ob.path.steps = vec![2, 2];
    (w, t)
}

/// The mode the computation ran in and its marked cells, from the trace.
fn run(w: &mut World, t: ObjectId, mode: Mode) -> (Option<Mode>, Vec<CellXY>) {
    trace::switch_on(w);
    w.obj_mut(t).unwrap().dest = Some(Target::Cell(cell(6, 0)));
    assert_eq!(path::compute(w, t, mode), Search::Found);
    for e in trace::take(w) {
        if let What::Search {
            mode, congested, ..
        } = e.what
        {
            return (mode, congested);
        }
    }
    panic!("no search recorded");
}

// spec: S-0212 claim 19 (in mode 1, when (i) counts no cell, the search runs as mode 0 instead; otherwise the marks follow the parity of the counts) (#306)
#[test]
fn mode_one_with_a_counted_path_cell_stays_mode_one() {
    // Control: (3, 0) empty. (i) counts it once, so it is marked; mode 1 runs.
    let (mut w, t) = scene(false);
    let (mode, marks) = run(&mut w, t, Mode::Blocked);
    assert_eq!(mode, Some(Mode::Blocked));
    assert_eq!(marks, vec![cell(3, 0)]);

    // (3, 0) occupied: (i) and (ii) count it, 2 in all; the cell ahead is
    // counted by (ii) and (iii). No cell is marked, but (i) counted a cell.
    let (mut w, t) = scene(true);
    let (mode, marks) = run(&mut w, t, Mode::Blocked);
    assert!(marks.is_empty());
    assert_eq!(mode, Some(Mode::Blocked), "claim 19: (i) counted (3, 0)");
}
