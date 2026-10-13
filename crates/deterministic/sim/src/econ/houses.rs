//! The houses' part of the tick (S-0203 row 7): each house's own update,
//! one house after another in house order (S-0203 claim 8). The order of the
//! topics inside one house's update follows the specs that own them.

use crate::world::{HouseId, World};

/// Row 7 of S-0203: every house's update, in house order.
pub fn part(w: &mut World) {
    // spec: S-0203 claim 8 (the houses one after another in house order)
    for hi in 0..w.houses.len() {
        house_update(w, HouseId(hi as u8));
    }
}

/// One house's update.
fn house_update(w: &mut World, h: HouseId) {
    let _ = (w, h);
}
