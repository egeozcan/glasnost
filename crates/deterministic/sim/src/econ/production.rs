//! Production lines, payment and build time (S-0403, S-0404).

use serde::{Deserialize, Serialize};

use super::TypeRef;
use crate::world::{HouseId, World};

/// Every production line of every house, in the order the lines were
/// created (S-0404 claim 1).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lines {}

/// Production state of a house that is not a line (S-0404 claim 2).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HouseProduction {}

/// Row 6 of S-0203: every production line, in creation order.
pub fn part(w: &mut World) {
    let _ = w;
}

/// The house has worked out its power again (S-0405 claim 11): the step
/// period of each of its production lines must be worked out again at once
/// (S-0403 claim 8), from the power fraction `crate::econ::power::fraction`.
/// Stub for the production topic (S-0403, S-0404).
pub fn power_changed(w: &mut World, house: HouseId) {
    let _ = (w, house);
}

/// Applies a build order (S-0404 claim 3).
pub fn build(w: &mut World, house: HouseId, t: TypeRef) {
    let _ = (w, house, t);
}

/// Applies a hold order (S-0404 claim 5).
pub fn hold(w: &mut World, house: HouseId, t: TypeRef) {
    let _ = (w, house, t);
}

/// Applies a cancel order (S-0404 claim 6); `all`: a cancel-all order.
pub fn cancel(w: &mut World, house: HouseId, t: TypeRef, all: bool) {
    let _ = (w, house, t, all);
}

/// Applies a place order (S-0404 claim 7, S-0406): a done unit is released
/// (no cell), a done building is put on the map at `cell`.
pub fn place(w: &mut World, house: HouseId, t: TypeRef, cell: Option<glasnost_map::CellXY>) {
    let _ = (w, house, t, cell);
}
