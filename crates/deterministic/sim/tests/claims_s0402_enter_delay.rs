//! S-0402 claim 10 and S-0215 claims 6 and 29: every processing of a
//! harvester's *enter* chooses the standard delay, also when it finds no
//! accepting refinery and falls back to rest (#268). Synthetic types from
//! `tests/harvest_common`.

mod harvest_common;

use glasnost_map::CellXY;
use glasnost_sim::econ::harvest::{self, DockState};
use glasnost_sim::rules::Behaviour;
use glasnost_sim::world::{ObjectId, Target, World};
use harvest_common::*;

fn refinery_cell() -> CellXY {
    at(-6, -1)
}

fn enter_now(w: &mut World, h: ObjectId) {
    let t = w.tick;
    let o = w.obj_mut(h).unwrap();
    o.beh.current = Some(Behaviour::Enter);
    o.beh.start_tick = t;
}

// spec: S-0402 claim 10 (each processing of enter is followed by the next after the standard delay of enter, one draw) and S-0215 claim 29 (every outcome of this enter processing chooses the standard delay)
#[test]
fn issue_268_enter_without_a_refinery_takes_the_standard_delay() {
    // Control: row 1 (in contact, the refinery switched off, not tethered).
    let r = ruleset();
    let mut w = game(&r);
    let refn = place_building(&mut w, "REFN", 0, refinery_cell());
    let h = place_vehicle(&mut w, "HARV", 0, at(0, 4), 0);
    w.obj_mut(h).unwrap().contact = Some(refn);
    w.obj_mut(refn).unwrap().econ.dock = Some(DockState { holders: vec![h] });
    w.obj_mut(refn).unwrap().econ.building.switched_off = true;
    enter_now(&mut w, h);
    let rng = w.rng.clone();
    let d_row1 = harvest::process_enter(&mut w, h);
    assert_ne!(w.rng, rng, "row 1 takes the draw");

    // Case A: no contact, destination a refinery whose only slot another harvester holds.
    let mut w = game(&r);
    let refn = place_building(&mut w, "REFN", 0, refinery_cell());
    let a = place_vehicle(&mut w, "HARV", 0, at(-3, 0), 16384);
    w.obj_mut(a).unwrap().contact = Some(refn);
    w.obj_mut(refn).unwrap().econ.dock = Some(DockState { holders: vec![a] });
    let b = place_vehicle(&mut w, "HARV", 0, at(0, 4), 0);
    w.obj_mut(b).unwrap().dest = Some(Target::Object(refn));
    enter_now(&mut w, b);
    let rng = w.rng.clone();
    let d_a = harvest::process_enter(&mut w, b);
    let drew_a = w.rng != rng;

    // Case B: no contact and a cell destination.
    let mut w = game(&r);
    place_building(&mut w, "REFN", 0, refinery_cell());
    let b = place_vehicle(&mut w, "HARV", 0, at(0, 4), 0);
    w.obj_mut(b).unwrap().dest = Some(Target::Cell(at(-3, 0)));
    enter_now(&mut w, b);
    let rng = w.rng.clone();
    let d_b = harvest::process_enter(&mut w, b);
    let drew_b = w.rng != rng;

    assert!(drew_a, "case A: no draw, delay {d_a} (row 1 gave {d_row1})");
    assert!(drew_b, "case B: no draw, delay {d_b} (row 1 gave {d_row1})");
    assert_ne!(d_a, 1, "case A: the 1-tick delay");
    assert_ne!(d_b, 1, "case B: the 1-tick delay");
}
