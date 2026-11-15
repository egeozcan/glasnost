//! S-0212 claim 5 (#298): a factory's rally spread (`orders::rally_cell`,
//! `quirks.rally_point_shared` off, a cell rally point) reads the reach
//! regions as the new map content gives them, even after a category change
//! earlier in the same tick. The regions' own contract (`regions::reach_in`)
//! asserts in a build with debug assertions that no read happens while they
//! are out of date. The econ test kit's BARRACKS and GRUNT; every name and
//! number is made up.

mod econ_common;

use econ_common::*;
use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_sim::econ::release::{self, Outcome};
use glasnost_sim::objects;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, Target};

// spec: S-0212 claim 5 (the rally spread's reach read after a category change in the same tick sees the new regions) (#298)
#[test]
fn rally_spread_after_a_category_change() {
    let mut r = ruleset();
    r.globals.quirks.rally_point_shared = false;
    let mut w = game(&r);
    let k = building(&mut w, "BARRACKS", 0, 0, 0);
    ready(&mut w);
    let kc = w.obj(k).unwrap().pos.cell();
    let spot = CellXY::new(kc.x + 6, kc.y + 2);
    w.obj_mut(k).unwrap().guard_spot = Some(Target::Cell(spot));
    // A category change far away, earlier in the same tick.
    let far = CellXY::new(kc.x - 8, kc.y - 8);
    w.cells.grid.get_mut(far).unwrap().land = land::WATER;
    glasnost_sim::regions::cell_changed(&mut w, far);
    assert!(w.regions.dirty);
    let ti = w.rules().index_of(Kind::Infantry, "GRUNT").unwrap();
    let g = objects::create(&mut w, Kind::Infantry, ti, Some(HouseId(0)));
    assert_eq!(release::release(&mut w, k, g), Outcome::Released);
    assert!(
        !w.regions.dirty,
        "the rally read brought the regions up to date"
    );
}
