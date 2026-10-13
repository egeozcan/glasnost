//! Building placement (S-0406): foundation cells, the cell test, adjacency,
//! explored ground, walls and upgrades.

use glasnost_map::CellXY;

use super::TypeRef;
use crate::world::{HouseId, World};

/// Puts the house's done building item of type `t` on the map at `cell`
/// (S-0406; the place order of S-0404 claim 7). It must place the item's
/// existing object, `crate::econ::production::item_object(w, house, t)`,
/// and create none (its creation draw was taken when the item started,
/// S-0404 claim 11); an upgrade is added to the building at `cell` instead
/// (S-0406 claim 12). Returns whether the placement succeeded. A stub until
/// S-0406 is implemented: every placement fails, so the item stays done.
pub fn place_building(w: &mut World, house: HouseId, t: TypeRef, cell: CellXY) -> bool {
    let _ = (w, house, t, cell);
    false
}

/// Whether a place order with a cell is legal when it is issued: adjacency
/// and explored ground (S-0406 claim 6, S-0411 claim 10; S-0404 claim 10).
/// A stub until S-0406 is implemented: legal.
pub fn place_legal(w: &World, house: HouseId, t: TypeRef, cell: CellXY) -> bool {
    let _ = (w, house, t, cell);
    true
}
