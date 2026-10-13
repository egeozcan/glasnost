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
    // spec: S-0405 claim 7 and S-0403 claim 8 (power worked out when due; every line of the house then works out its step period)
    if super::power::work_out_if_due(w, h) {
        super::production::on_power_reevaluated(w, h);
    }
    // spec: S-0411 claim 2 (the removal of icons and its orders, at the house update)
    super::sidebar::remove_unbuildable(w, h);
    // spec: S-0407 claim 12 (a recheck in the first house update after an event)
    super::tech::house_update(w, h);
    // spec: S-0411 claim 9 (the automatic release order)
    super::sidebar::release_orders(w, h);
}
