//! S-0208 claims 6, 16 and 18 with S-0212 claims 7 and 12: a walker on the
//! ground under a high-bridge deck whose cell destination is its own cell
//! has not completed its destination, so it searches for it (#273).
//! Synthetic scene, invented type name.

mod common;

use common::{FOOT, Scene, cell, unit_type};
use glasnost_sim::rules::Kind;
use glasnost_sim::tick;
use glasnost_sim::trace::{self, What};
use glasnost_sim::world::{Layer, ObjectId, Target, World};

/// A soldier standing at the centre of cell(4, 0) with no walking target,
/// no stored path and that cell as destination (S-0208 claim 6: a standing
/// walker searches in its first locomotion). With `deck`, cells (3 … 5, 0)
/// of the flat ground carry a high-bridge deck and the soldier stands on
/// the ground under it.
fn standing(deck: bool) -> (World, ObjectId) {
    let mut s = Scene::new();
    s.infantry("TESTMAN", unit_type(FOOT, 4, 0));
    let c = cell(4, 0);
    s.place_infantry(0, "TESTMAN", c, 0);
    let mut w = s.build();
    if deck {
        for x in 3..=5 {
            let i = w.cells.idx(cell(x, 0)).expect("cell");
            w.cells.deck[i] = true;
            w.cells.deck_entry[i] = true;
            w.cells.bridge_axis[i] = glasnost_sim::cells::Axis::X;
        }
        w.regions.dirty = true;
    }
    let id = common::objects_of(&w, Kind::Infantry)[0];
    let o = w.obj_mut(id).expect("soldier");
    o.dest = Some(Target::Cell(c));
    o.path.steps.clear();
    let ws = o.walk.as_mut().expect("walk state");
    ws.target = None;
    ws.fraction = false;
    trace::switch_on(&mut w);
    (w, id)
}

/// The number of path computations `id` makes in the next tick.
fn searches_in_one_tick(w: &mut World, id: ObjectId) -> usize {
    tick::step(w);
    trace::take(w)
        .iter()
        .filter(|e| e.unit == id && matches!(e.what, What::Search { .. }))
        .count()
}

// spec: S-0208 claim 18 (a cell destination is not a request to stand on the layer the unit already occupies; an arrival on the ground under the deck does not complete it, and an exhausted path needs another path computation under S-0212); claim 16 (|Δz| < 208 against the locomotor destination); S-0212 claim 7 (the destination pair of a bridge cell is its deck) and claim 12 (no path only when the destination cell is the unit's own cell on the same search layer)
#[test]
fn issue_273_own_cell_under_a_deck_is_searched_for() {
    let (mut w, id) = standing(true);
    assert_eq!(w.obj(id).expect("s").layer, Layer::Ground);
    assert_eq!(
        searches_in_one_tick(&mut w, id),
        1,
        "a walker under the deck makes a path computation for the deck of its own cell"
    );
}

// spec: S-0208 claim 16 (control: a walker on the ground with its own cell as destination and no deck there has completed it; no search)
#[test]
fn issue_273_control_own_cell_without_a_deck_is_complete() {
    let (mut w, id) = standing(false);
    assert_eq!(searches_in_one_tick(&mut w, id), 0, "no search");
    assert_eq!(
        w.obj(id).expect("s").dest,
        None,
        "the destination is complete"
    );
}
