//! Victory and defeat (S-0414).

use serde::{Deserialize, Serialize};

/// A house's defeat and result (S-0414 claims 5 and 7).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HouseResult {}

/// The counts an object is in (S-0414 claim 4).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counted {}

/// The present count of a don't-score vehicle type for a house (S-0414
/// claim 4, `quirks.unscored_vehicle_stays_counted`; S-0407 Terms reads
/// it): the times a vehicle of the type has entered the map as the house's
/// since game start, or 0 with the switch off. A stub until S-0414's counts
/// are implemented: 0.
pub fn unscored_vehicle_entries(
    w: &crate::world::World,
    house: crate::world::HouseId,
    vehicle: u32,
) -> i32 {
    let _ = (w, house, vehicle);
    0
}

/// The end of the game (S-0414 claims 6 to 10).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameEnd {}

/// Applies a surrender order (S-0414 claim 11).
pub fn surrender(w: &mut crate::world::World, house: crate::world::HouseId) {
    let _ = (w, house);
}

/// Applies an alliance order towards `other` (S-0414 claims 12 to 14).
pub fn alliance(
    w: &mut crate::world::World,
    house: crate::world::HouseId,
    other: crate::world::HouseId,
) {
    let _ = (w, house, other);
}
