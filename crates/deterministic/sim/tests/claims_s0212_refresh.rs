//! S-0212 claim 20: the recomputation a driving vehicle makes as it begins
//! the last step of its stored path, with `quirks.path_refresh_keeps_old_step`
//! on and off (#263). Synthetic types on the flat test map of `tests/common`.

mod common;

use common::unit_type;
use common::{SIZE, Scene, WHEEL, cell, objects_of, open_orders, order_move};
use glasnost_map::CellXY;
use glasnost_sim::regions::STEPS;
use glasnost_sim::rules::Kind;

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
