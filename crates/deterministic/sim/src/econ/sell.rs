//! Selling and building repair (S-0408; repair itself is S-0314's).

use glasnost_map::CellXY;
use serde::{Deserialize, Serialize};

use crate::world::{HouseId, ObjectId, World};

/// A sale in progress (S-0408 claim 6).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sale {}

/// Applies a sell order (S-0213 claim 12, S-0408 claims 3 to 11).
pub fn sell(w: &mut World, house: HouseId, actor: ObjectId) {
    let _ = (w, house, actor);
}

/// Applies a sell-wall order (S-0213 claim 12, S-0408 claim 12).
pub fn sell_wall(w: &mut World, house: HouseId, cell: CellXY) {
    let _ = (w, house, cell);
}

/// Applies a repair order (S-0213 claim 13, S-0408 claim 14).
pub fn repair(w: &mut World, actor: ObjectId) {
    let _ = (w, actor);
}
