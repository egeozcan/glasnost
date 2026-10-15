//! The spy plane and the paradrop hand-off (S-0513 claims 42 to 45).

use glasnost_map::CellXY;

use crate::world::{HouseId, World};

/// A paradrop launch (claim 42).
pub fn paradrop(w: &mut World, h: HouseId, cell: CellXY, american: bool) {
    let _ = (w, h, cell, american);
}

/// A spy plane launch (claim 43).
pub fn spy_plane(w: &mut World, h: HouseId, cell: CellXY) {
    let _ = (w, h, cell);
}
