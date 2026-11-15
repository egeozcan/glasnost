//! S-0212 claim 20 (revision 100, Q-3115): the recomputation a driving
//! vehicle makes as it begins the last step of its stored path, and the
//! two-cell curve over the last two stored steps, which makes none (#263).
//! Synthetic types on the flat test map of `tests/common`; a corridor of rock
//! one cell wide.

mod common;

use common::unit_type;
use common::{SIZE, Scene, WHEEL, cell, code_tile, objects_of, open_orders, order_move};
use glasnost_map::CellXY;
use glasnost_sim::rules::Kind;

const CAR: &str = "TESTCAR";

type Xy = (i32, i32);

fn at(c: Xy) -> CellXY {
    cell(c.0, c.1)
}

fn xy(c: CellXY) -> Xy {
    (c.x - SIZE, c.y - SIZE)
}

/// Rock on both sides of row 0 from column -19 to `end`: a corridor one cell
/// wide that opens onto flat ground at column `end` + 1.
fn scene(keeps_old_step: bool, end: i32) -> Scene {
    let mut s = Scene::new();
    s.vehicle(CAR, unit_type(WHEEL, 40, 6));
    s.ruleset.globals.quirks.path_refresh_keeps_old_step = keeps_old_step;
    for x in -19..=end {
        // Tile code 7: land type rock, no land row (blocked for every class).
        s.set_cell(cell(x, -1), code_tile(7), 0);
        s.set_cell(cell(x, 1), code_tile(7), 0);
    }
    s
}

/// What a drive showed: the cells the vehicle stood in, the cells of each
/// track it began, and, for each two-cell curve begun from a stored path
/// that did not use up exactly two of its steps, the stored step counts
/// before and after it began.
#[derive(Debug, Default)]
struct Drive {
    cells: Vec<Xy>,
    tracks: Vec<Vec<Xy>>,
    grew_in_a_curve: Vec<(usize, usize)>,
}

fn drive(keeps_old_step: bool, end: i32, from: Xy, to: Xy) -> Drive {
    let mut s = scene(keeps_old_step, end);
    s.place_unit(0, CAR, at(from), 64);
    let mut w = s.build();
    open_orders(&mut w);
    let id = objects_of(&w, Kind::Vehicle)[0];
    order_move(&mut w, id, at(to));
    let mut out = Drive {
        cells: vec![from],
        ..Drive::default()
    };
    let mut last_track: Option<(Xy, Vec<Xy>)> = None;
    let mut before = w.obj(id).unwrap().path.steps.len();
    for _ in 0..4000 {
        glasnost_sim::tick::step(&mut w);
        let o = w.obj(id).unwrap();
        let c = xy(o.pos.cell());
        if out.cells.last() != Some(&c) {
            out.cells.push(c);
        }
        let track = o.drive.as_ref().and_then(|d| d.track.as_ref());
        let now = o.path.steps.len();
        if let Some(t) = track {
            let key = (
                xy(t.target_cell),
                t.cells.iter().map(|&c| xy(c)).collect::<Vec<_>>(),
            );
            if last_track.as_ref() != Some(&key) {
                out.tracks.push(key.1.clone());
                // A curve begun from a stored path uses two of its steps.
                if key.1.len() == 2 && before >= 2 && now + 2 != before {
                    out.grew_in_a_curve.push((before, now));
                }
                last_track = Some(key);
            }
        } else {
            last_track = None;
        }
        before = now;
        if o.dest.is_none() && track.is_none() {
            break;
        }
    }
    out
}

// spec: S-0212 claim 20 (a two-cell curve that uses the last two stored steps begins as usual and makes no recomputation, whatever the switch; the stored path is empty when it ends, and the fresh search where the curve ended is followed from its first step) and Edge cases (`quirks.path_refresh_keeps_old_step`)
fn no_recomputation_at_a_curve_over_the_last_two_steps(keeps_old_step: bool) {
    // 23 stored steps east along the corridor and one south-east out of it:
    // the last two are begun as one curve from (7, 0) to (9, 1).
    let (from, to) = ((-15, 0), (16, 3));
    let d = drive(keeps_old_step, 8, from, to);
    assert!(
        d.tracks.contains(&vec![(8, 0), (9, 1)]),
        "the last two stored steps were not begun as one curve: {:?}",
        d.tracks
    );
    assert!(
        d.grew_in_a_curve.is_empty(),
        "a recomputation was made at a curve over the last two stored steps (stored steps before and after): {:?}",
        d.grew_in_a_curve
    );
    assert!(
        !d.cells.contains(&(17, 4)),
        "the vehicle drove past its destination: {:?}",
        d.cells
    );
    assert_eq!(d.cells.last(), Some(&to), "{:?}", d.cells);
}

#[test]
fn issue_263_on_a_curve_over_the_last_two_steps_makes_no_recomputation() {
    no_recomputation_at_a_curve_over_the_last_two_steps(true);
}

#[test]
fn issue_263_off_a_curve_over_the_last_two_steps_makes_no_recomputation() {
    no_recomputation_at_a_curve_over_the_last_two_steps(false);
}

// spec: S-0212 claim 20 (switch on: at the recomputation as the last stored step begins, the old step and the new path's second step choose the track as the current and next step of S-0207 claim 7; a two-cell curve takes the old step and then the new path's second step, and the unit continues with the new path's third step)
#[test]
fn issue_263_on_the_old_step_and_the_new_second_step_choose_a_curve() {
    // 24 stored steps east along the corridor to (9, 0); the destination lies
    // three steps south-east of it. The 24th step begins at (8, 0); the new
    // path from there is east, then south-east three times.
    let (from, to) = ((-15, 0), (12, 3));
    let d = drive(true, 9, from, to);
    assert!(
        d.tracks.contains(&vec![(9, 0), (10, 1)]),
        "the old step east and the new path's second step south-east were not begun as one curve: {:?}",
        d.tracks
    );
    assert!(
        !d.tracks.contains(&vec![(9, 0)]),
        "the old step was begun as a track of its own: {:?}",
        d.tracks
    );
    assert_eq!(d.cells.last(), Some(&to), "{:?}", d.cells);
}
