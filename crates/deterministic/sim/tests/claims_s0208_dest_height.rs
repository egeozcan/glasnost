//! S-0208 claim 18 and F9: the height of a walker's cell locomotor
//! destination is fixed when the destination is assigned, so a deck added
//! or lost during the walk does not change the end-of-walk test of claim 16
//! (#274). Synthetic scene, invented type name.

mod common;

use common::{FOOT, Scene, cell, open_orders, order_move, unit_type};
use glasnost_sim::rules::Kind;
use glasnost_sim::tick;
use glasnost_sim::walk;
use glasnost_sim::world::{Layer, Target, World};

fn mark_deck(w: &mut World, on: bool) {
    for x in 3..=5 {
        let i = w.cells.idx(cell(x, 0)).expect("cell");
        w.cells.deck[i] = on;
        w.cells.deck_entry[i] = on;
        w.cells.bridge_axis[i] = glasnost_sim::cells::Axis::X;
    }
    w.regions.dirty = true;
}

/// A soldier ordered from cell(0, 0) to cell(4, 0) on flat ground. With
/// `added`, cells (3 … 5, 0) get a high-bridge deck once the soldier has
/// left its start cell, as a repair in play marks them. Returns whether the
/// destination is still held on the tick the soldier comes to rest in the
/// ordered cell, the cell-destination height read then, and the terrain
/// height.
fn run(added: bool) -> (bool, i32, i32) {
    let mut s = Scene::new();
    s.infantry("TESTMAN", unit_type(FOOT, 4, 0));
    let c = cell(4, 0);
    s.place_infantry(0, "TESTMAN", cell(0, 0), 0);
    let mut w = s.build();
    let id = common::objects_of(&w, Kind::Infantry)[0];
    open_orders(&mut w);
    // The destination is assigned here: no cell carries a high bridge.
    order_move(&mut w, id, c);
    assert!(!w.high_bridge(c), "no deck at destination assignment");
    let h = w.surface_at(256 * c.x + 128, 256 * c.y + 128);
    let mut marked = false;
    for _ in 0..2000 {
        tick::step(&mut w);
        let o = w.obj(id).expect("soldier");
        if added && !marked && o.pos.cell() == cell(1, 0) {
            mark_deck(&mut w, true);
            marked = true;
            continue;
        }
        let resting = o.walk.as_ref().is_some_and(|s| !s.fraction);
        if o.pos.cell() == c && o.path.steps.is_empty() && resting {
            assert_eq!(o.layer, Layer::Ground, "it arrives on the ground");
            assert_eq!(o.pos.z, h, "at the terrain height");
            let kept = o.dest == Some(Target::Cell(c));
            return (kept, walk::cell_destination_height(&w, id, c), h);
        }
    }
    panic!("the soldier never came to rest in the ordered cell");
}

// spec: S-0208 claim 18 and F9 (a is 414 leptons when the cell carries a high bridge at destination assignment, and 0 otherwise; fixed when the destination is assigned)
// spec: S-0208 claim 16 (in the destination's cell less than two levels from the locomotor destination: the walk ends on the arrival tick)
#[test]
fn issue_274_deck_added_after_assignment_does_not_change_the_destination_height() {
    let (kept, z, h) = run(false);
    assert!(!kept, "control: the walk ends on arrival");
    assert_eq!(z, h, "control: z_D = H");
    let (kept, z, h) = run(true);
    assert_eq!(
        (kept, z),
        (false, h),
        "(destination kept on arrival, z_D): no deck at destination assignment, so z_D = H and the walk ends"
    );
}
