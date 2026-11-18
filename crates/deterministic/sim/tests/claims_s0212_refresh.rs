//! S-0212 claim 20: the recomputation a driving vehicle makes as it begins
//! the last step of its stored path, with `quirks.path_refresh_keeps_old_step`
//! on and off (#263): a straight recomputation, the switch-off turn, and
//! the moving-blocker difference at S-0207 table T5. Synthetic types on the
//! flat test map of `tests/common`.

mod common;

use common::unit_type;
use common::{SIZE, Scene, WHEEL, cell, objects_of, open_orders, order_move};
use glasnost_map::CellXY;
use glasnost_sim::regions::STEPS;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::Target;

const CAR: &str = "TESTCAR";

type Xy = (i32, i32);

fn at(c: Xy) -> CellXY {
    cell(c.0, c.1)
}

fn xy(c: CellXY) -> Xy {
    (c.x - SIZE, c.y - SIZE)
}

fn walk(from: Xy, steps: &[u8]) -> Xy {
    steps.iter().fold(from, |c, &d| {
        let (dx, dy) = STEPS[usize::from(d)];
        (c.0 + dx, c.1 + dy)
    })
}

/// A recomputation seen while a track is in progress: the cell the track
/// ends in and the cell the stored steps lead to from there.
#[derive(Debug)]
struct Refresh {
    track_end: Xy,
    stored: Vec<u8>,
    leads_to: Xy,
}

/// Drives a vehicle from `from` to `to` on open ground; returns the cells it
/// stood in and the recomputations made during a track.
fn drive(keeps_old_step: bool, from: Xy, to: Xy) -> (Vec<Xy>, Vec<Refresh>) {
    let mut s = Scene::new();
    s.vehicle(CAR, unit_type(WHEEL, 40, 6));
    s.ruleset.globals.quirks.path_refresh_keeps_old_step = keeps_old_step;
    s.place_unit(0, CAR, at(from), 64);
    let mut w = s.build();
    open_orders(&mut w);
    let id = objects_of(&w, Kind::Vehicle)[0];
    order_move(&mut w, id, at(to));
    let mut cells = vec![from];
    let mut refreshes = Vec::new();
    let mut before = 0;
    for _ in 0..4000 {
        glasnost_sim::tick::step(&mut w);
        let o = w.obj(id).unwrap();
        let c = xy(o.pos.cell());
        if cells.last() != Some(&c) {
            cells.push(c);
        }
        let track = o.drive.as_ref().and_then(|d| d.track.as_ref());
        let now = o.path.steps.len();
        if before > 0
            && now > before
            && let Some(t) = track
        {
            let end = xy(t.target_cell);
            refreshes.push(Refresh {
                track_end: end,
                stored: o.path.steps.clone(),
                leads_to: walk(end, &o.path.steps),
            });
        }
        before = now;
        if o.dest.is_none() && track.is_none() {
            break;
        }
    }
    (cells, refreshes)
}

// spec: S-0212 claim 20 (the drive locomotor recomputes as it begins the last stored step, heading point beyond 512) and Edge cases (`quirks.path_refresh_keeps_old_step` off: the unit takes the new path's first step instead of the old one)
#[test]
fn issue_263_off_the_new_first_step_replaces_the_old_one() {
    let (from, to) = ((-15, -15), (15, 15));
    let (cells, refreshes) = drive(false, from, to);
    assert!(!refreshes.is_empty(), "no recomputation during a track");
    for r in &refreshes {
        assert_eq!(
            r.leads_to, to,
            "the step in progress ends on {:?}; the stored steps {:?} lead from there to {:?}, not to the destination",
            r.track_end, r.stored, r.leads_to
        );
    }
    assert!(
        !cells.contains(&(16, 16)),
        "the vehicle drove past its destination: {cells:?}"
    );
    assert_eq!(cells.last(), Some(&to));
}

// spec: S-0212 claim 20 (switch on: the old step is still taken and the new path's first step is dropped; when the new path's first step is the old step this has no visible effect)
#[test]
fn issue_263_on_a_straight_refresh_has_no_visible_effect() {
    let (from, to) = ((-15, -15), (15, 15));
    let (cells, refreshes) = drive(true, from, to);
    assert!(!refreshes.is_empty(), "no recomputation during a track");
    for r in &refreshes {
        assert_eq!(r.leads_to, to, "{r:?}");
    }
    assert!(!cells.contains(&(16, 16)), "{cells:?}");
    assert_eq!(cells.last(), Some(&to));
}

/// The cell a vehicle driving north from (0, 12) stands in as its 24th and
/// last stored step begins (S-0212 claim 20: 24 steps are stored).
const LAST_FROM: Xy = (0, -11);
/// The cell that last step leads to.
const LAST_TO: Xy = (0, -12);
/// The destination that turns the recomputation's new path east along the
/// row of [`LAST_FROM`]: twelve east steps, none of them the old step north.
const EAST: Xy = (12, -11);

/// What [`redirected`] saw.
#[derive(Debug)]
struct Redirected {
    /// The cells the vehicle stood in, in order.
    cells: Vec<Xy>,
    /// The stored path on each tick the vehicle stood in [`LAST_FROM`]
    /// without a track after its destination changed.
    waiting: Vec<Vec<u8>>,
}

/// A vehicle facing north at (0, 12) is ordered to (0, −18). Once it has
/// driven 12 cells its destination becomes [`EAST`] with its stored path
/// kept, a test-only change of state, so the recomputation as the last
/// stored step begins at [`LAST_FROM`] finds a new path whose first step is
/// not the old one. With `blocker`, an allied vehicle that counts as moving
/// stands in (1, −12), the second cell of the two-cell curve the old step
/// north and the new path's second step east would form (S-0207 claim 7).
fn redirected(keeps_old_step: bool, blocker: bool) -> Redirected {
    let mut s = Scene::new();
    s.vehicle(CAR, unit_type(WHEEL, 40, 6));
    s.ruleset.globals.quirks.path_refresh_keeps_old_step = keeps_old_step;
    s.place_unit(0, CAR, at((0, 12)), 0);
    if blocker {
        s.place_unit(0, CAR, at((1, -12)), 0);
    }
    let mut w = s.build();
    open_orders(&mut w);
    let ids = objects_of(&w, Kind::Vehicle);
    let id = ids
        .iter()
        .copied()
        .find(|&v| xy(w.obj(v).unwrap().pos.cell()) == (0, 12))
        .unwrap();
    if let Some(&b) = ids.iter().find(|&&v| v != id) {
        // Counts as moving (S-0212 claim 27) while it stays where it is: a
        // far destination, no stored path, a retry delay that never ends.
        let o = w.obj_mut(b).unwrap();
        o.dest = Some(Target::Cell(cell(-20, 10)));
        o.path.steps.clear();
        o.path.retry_at = i32::MAX;
    }
    order_move(&mut w, id, at((0, -18)));
    let mut cells = vec![(0, 12)];
    let mut waiting = Vec::new();
    let mut redirected = false;
    for _ in 0..4000 {
        glasnost_sim::tick::step(&mut w);
        let o = w.obj(id).unwrap();
        let c = xy(o.pos.cell());
        if cells.last() != Some(&c) {
            cells.push(c);
        }
        let track = o.drive.as_ref().and_then(|d| d.track.as_ref()).is_some();
        if redirected && c == LAST_FROM && !track {
            waiting.push(o.path.steps.clone());
        }
        let done = o.dest.is_none() && !track;
        if !redirected && c == (0, 0) {
            let o = w.obj_mut(id).unwrap();
            assert!(
                o.path.steps.len() > 2 && o.path.steps.iter().all(|&d| d == 0),
                "the stored path at (0, 0): {:?}",
                o.path.steps
            );
            o.dest = Some(Target::Cell(at(EAST)));
            redirected = true;
        }
        if done {
            break;
        }
    }
    assert!(redirected, "never reached (0, 0): {cells:?}");
    Redirected { cells, waiting }
}

/// The cell entered right after `c`, if `c` was left.
fn after(cells: &[Xy], c: Xy) -> Option<Xy> {
    let i = cells.iter().position(|&x| x == c)?;
    cells.get(i + 1).copied()
}

// spec: S-0212 claim 20 and Edge cases (`quirks.path_refresh_keeps_old_step` off: the unit takes the new path's first step instead of the old one, after a turn when it does not face it, S-0207 claim 6; on: the old step and the new path's second step choose the track, S-0207 claim 7)
#[test]
fn issue_263_off_turns_to_the_new_first_step() {
    let off = redirected(false, false);
    // Off: it turns east where it stands, holding the whole new path, and
    // never takes the old step north.
    assert_eq!(after(&off.cells, LAST_FROM), Some((1, -11)), "{off:?}");
    assert!(!off.cells.contains(&LAST_TO), "{off:?}");
    assert!(
        off.waiting.iter().any(|p| p == &vec![2; 12]),
        "no turn in {LAST_FROM:?} with the twelve east steps stored: {off:?}"
    );
    assert_eq!(off.cells.last(), Some(&EAST), "{off:?}");
    // On: the old step north and the new path's second step east form a
    // two-cell curve through (0, −12) into (1, −12), with no turn first.
    let on = redirected(true, false);
    assert_eq!(after(&on.cells, LAST_FROM), Some(LAST_TO), "{on:?}");
    assert_eq!(after(&on.cells, LAST_TO), Some((1, -12)), "{on:?}");
    assert!(on.waiting.is_empty(), "{on:?}");
}

// spec: S-0212 claim 20 (T5's moving-blocker row at the curve's second cell: the step taken is the new path's first step, from the unit's cell, after a turn when the unit does not face it and a step check of the cell it enters; no step of the new path is dropped) and S-0207 table T5 (moving blocker: the straight or diagonal track into the first cell)
#[test]
fn issue_263_moving_blocker_at_the_curve_takes_the_new_first_step() {
    for keeps_old_step in [true, false] {
        let r = redirected(keeps_old_step, true);
        let k = keeps_old_step;
        assert_eq!(after(&r.cells, LAST_FROM), Some((1, -11)), "{k}: {r:?}");
        assert!(!r.cells.contains(&LAST_TO), "{k}: {r:?}");
        assert!(
            r.waiting.iter().any(|p| p == &vec![2; 12]),
            "{k}: no turn in {LAST_FROM:?} with the twelve east steps stored: {r:?}"
        );
        assert_eq!(r.cells.last(), Some(&EAST), "{k}: {r:?}");
    }
}
