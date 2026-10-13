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
    // S-0414 claims 1 and 11: a pending surrender's destruction, then the
    // defeat test; first in the house update (interim, request Q-0151).
    crate::econ::victory::house_update(w, h);
    // Power, then radar (S-0405 claim 7, S-0410 claim 2): the radar recheck
    // after an event sees the power worked out in the same update (S-0410
    // scenario R1, the power changes at (d) and (e)).
    if crate::econ::power::house_update(w, h) {
        // spec: S-0410 claim 2 (a change of the power balance asks for a recheck)
        crate::econ::radar::mark_due(w, h);
    }
    crate::econ::radar::house_update(w, h);
    // spec: S-0407 claim 12 (a recheck in the first house update after an event; S-0404 claim 9 and S-0411 claim 2: the removal orders in that house update)
    super::tech::house_update(w, h);
    // spec: S-0411 claim 9 (the automatic release order)
    super::sidebar::release_orders(w, h);
}
