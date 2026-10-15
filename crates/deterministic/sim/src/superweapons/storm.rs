//! The weather storm (S-0513 claims 14 to 21).

use glasnost_map::CellXY;

use crate::world::{HouseId, World};

/// A storm request (claim 14).
pub fn request(w: &mut World, owner: Option<HouseId>, cell: CellXY, duration: i32, countdown: i32) {
    let _ = (w, owner, cell, duration, countdown);
}

/// A cloud made by a trigger action (claim 20; S-0524 claim 2).
pub fn cloud(w: &mut World, cell: CellXY) {
    let _ = (w, cell);
}
