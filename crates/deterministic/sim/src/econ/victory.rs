//! Victory and defeat (S-0414).

use serde::{Deserialize, Serialize};

/// A house's defeat and result (S-0414 claims 5 and 7).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HouseResult {}

/// The counts an object is in (S-0414 claim 4).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counted {}

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
