//! Invulnerability and the force shield (S-0513 claims 22 to 32).

use glasnost_map::CellXY;

use crate::world::{HouseId, World};

/// The curtain (claim 22).
pub fn curtain(w: &mut World, h: Option<HouseId>, cell: CellXY, waypoint: bool) {
    let _ = (w, h, cell, waypoint);
}

/// The force shield (claims 30 to 32).
pub fn force_shield(w: &mut World, h: HouseId, i: u32, cell: CellXY) {
    let _ = (w, h, i, cell);
}

/// The fading sound's countdown in the house update (claim 32).
pub fn fade_update(w: &mut World, h: HouseId) {
    let _ = (w, h);
}
