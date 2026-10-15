//! The nuclear strike (S-0513 claims 7 to 13).

use glasnost_map::CellXY;

use crate::world::{HouseId, ObjectId, World};

/// The silo a launch of type `i` uses (claim 7).
pub fn silo_choice(w: &World, h: HouseId, i: u32) -> Option<ObjectId> {
    let _ = (w, h, i);
    None
}

/// Whether a silo accepts the launch (claim 7).
pub fn accepts(w: &World, s: ObjectId) -> bool {
    let _ = (w, s);
    false
}

/// A one-time strike (claim 10).
pub fn one_time(w: &mut World, h: Option<HouseId>, i: u32, cell: CellXY) {
    let _ = (w, h, i, cell);
}

/// A silo launch (claims 7 and 8).
pub fn silo_launch(w: &mut World, h: HouseId, i: u32, cell: CellXY, silo: Option<ObjectId>) {
    let _ = (w, h, i, cell, silo);
}

/// The nuclear strike at a waypoint (S-0524 claim 3).
pub fn waypoint(w: &mut World, cell: CellXY) {
    let _ = (w, cell);
}
