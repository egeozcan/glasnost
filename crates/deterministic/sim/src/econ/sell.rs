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

/// A building type's adjusted cost *C*′ (S-0408 claim 2, F2), which the
/// price of S-0404 claim 2 uses. Owned by S-0408's topic.
pub fn adjusted_cost(w: &World, b: u32) -> i32 {
    use crate::econ::production as p;
    let cost = |t: Option<crate::econ::TypeRef>| t.map_or(0, |t| p::cost_field(w, t));
    // spec: S-0408 F2 (d = trunc((C₀ + C₁) / 2) when the pad deduction applies; a missing entry counts 0)
    let d = if p::pad_term_applies(w, b) {
        (cost(p::pad(w, 0)) + cost(p::pad(w, 1))) / 2
    } else {
        0
    };
    let k = cost(Some(crate::econ::TypeRef {
        kind: crate::rules::Kind::Building,
        index: b,
    }));
    match p::free_unit(w, b) {
        // spec: S-0408 F2 (C = max(0, K − d − F) with a free unit, K − d without one)
        Some(f) => (k - d - cost(Some(f))).max(0),
        None => k - d,
    }
}
