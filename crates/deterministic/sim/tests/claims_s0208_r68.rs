//! Claim tests of S-0208 revision 68 (Q-3262 answered), claim 25: a
//! walker's searches for an object destination look for the cell of its
//! locomotor destination, the point the object gave at the assignment, not
//! for the object's place at the time; a failed fresh search clears the
//! destination near that point; a walker standing in that cell keeps its
//! destination otherwise, with no locomotor destination left. On a flat
//! synthetic map with an invented foot soldier; no game data. Ledger:
//! `docs/m8-claims/sim-issues.md`, *Batch S42*.

mod common;

use common::{FOOT, Scene, cell, open_orders, order_move, run, unit_type};
use glasnost_rules::sim::Decimal;
use glasnost_sim::path::{self, Mode};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{econ, occupancy};

/// A flat map with two soldiers: F of Alpha's at `f`, sub-cell 1, and I of
/// Beta's at `i`, sub-cell 2 (an enemy, so no stationary-friendly
/// substitution, S-0212 claim 24, touches the searches). `close_enough`:
/// the close-enough distance in cells, else the default. Returns (F, I).
fn scene(f: (i32, i32), i: (i32, i32), close_enough: Option<&str>) -> (World, ObjectId, ObjectId) {
    let mut s = Scene::new();
    s.infantry("FOOTMAN", unit_type(FOOT, 4, 0));
    if let Some(c) = close_enough {
        s.ruleset.sim.general.close_enough = Some(Decimal(c.into()));
    }
    s.place_infantry(0, "FOOTMAN", cell(f.0, f.1), 1);
    s.place_infantry(1, "FOOTMAN", cell(i.0, i.1), 2);
    let mut w = s.build();
    open_orders(&mut w);
    let of = |w: &World, h: u8| {
        w.objects
            .values()
            .find(|o| o.owner == Some(HouseId(h)))
            .map(|o| o.id)
            .unwrap()
    };
    let (fid, iid) = (of(&w, 0), of(&w, 1));
    (w, fid, iid)
}

fn dest(w: &World, id: ObjectId) -> Option<Target> {
    w.obj(id).unwrap().dest
}

/// Orders I away to a far cell and runs until it has left `from`.
fn walk_away(w: &mut World, i: ObjectId, from: glasnost_map::CellXY) {
    order_move(w, i, cell(4, 10));
    for _ in 0..300 {
        if w.obj(i).unwrap().pos.cell() != from {
            return;
        }
        glasnost_sim::tick::step(w);
    }
    panic!("void: I did not leave its cell");
}

// spec: S-0208 claim 25 (r68: while a walker's destination names an object, each search of its walking looks for the cell of its locomotor destination, the point the object gave at the last assignment; the object's place at the time of the search is not read; the fresh search of S-0212 claim 20) and Terms (locomotor destination)
#[test]
fn r68_the_fresh_search_looks_for_the_points_cell() {
    let (mut w, f, i) = scene((-8, 0), (4, 0), None);
    econ::assign_destination(&mut w, f, Some(Target::Object(i)));
    let p = w.obj(i).unwrap().pos;
    walk_away(&mut w, i, p.cell());
    assert_eq!(
        dest(&w, f),
        Some(Target::Object(i)),
        "void: F still holds I"
    );
    w.obj_mut(f).unwrap().path.steps.clear();
    assert_eq!(
        path::compute(&mut w, f, Mode::Fresh),
        path::Search::Found,
        "void: F finds a path"
    );
    assert_eq!(
        w.obj(f).unwrap().path.for_goal,
        Some(p.cell()),
        "the search looks for the point's cell, not I's cell now"
    );
}

// spec: S-0208 claim 25 (r68, A walker standing with no stored path: with movement in progress it makes the fresh search for its locomotor destination's cell; standing in that cell on the same search layer it gets no path, S-0212 claim 12, and S-0212 claim 23's third row applies; with D not less than the close-enough distance it keeps its destination with no movement in progress; with none in progress it has no locomotor destination, so its walking makes no search and tests no cell, and a walker never ends its move because it stands in the cell its object is in at that time, unless that is also its locomotor destination's cell)
#[test]
fn r68_a_walker_in_the_points_cell_keeps_its_destination_beyond_close_enough() {
    let (mut w, f, i) = scene((0, 0), (0, 0), Some("0"));
    econ::assign_destination(&mut w, f, Some(Target::Object(i)));
    run(&mut w, 3);
    assert_eq!(
        dest(&w, f),
        Some(Target::Object(i)),
        "the destination is kept"
    );
    assert!(!occupancy::in_progress(&w, f), "no movement in progress");
    let at = w.obj(f).unwrap().pos;
    walk_away(&mut w, i, at.cell());
    for _ in 0..200 {
        glasnost_sim::tick::step(&mut w);
        assert_eq!(dest(&w, f), Some(Target::Object(i)), "still kept");
        assert_eq!(w.obj(f).unwrap().pos, at, "F does not follow I");
        assert!(!occupancy::in_progress(&w, f));
    }
}

// spec: S-0208 claim 25 (r68, A failed search near the locomotor destination: a walker whose fresh search finds no path past the gate loses its destination in the same locomotion when its distance D of S-0209 F1 to its locomotor destination, heights included, is less than the close-enough distance and it is not held by a tether link)
#[test]
fn r68_a_walker_in_the_points_cell_within_close_enough_loses_its_destination() {
    let (mut w, f, i) = scene((0, 0), (0, 0), None);
    econ::assign_destination(&mut w, f, Some(Target::Object(i)));
    run(&mut w, 3);
    assert_eq!(dest(&w, f), None, "the destination is cleared");
    assert!(!occupancy::in_progress(&w, f));
}
