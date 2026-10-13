//! Power (S-0405): output and drain, low power, blackouts, the power switch,
//! and which buildings stop.

use serde::{Deserialize, Serialize};

use crate::world::{HouseId, ObjectId, World};

/// A house's power totals (S-0405 claim 4) and blackout (claims 5, 6).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HousePower {}

/// A building's switches: the power switch (S-0405 Terms, claim 10), the
/// primary-factory mark (S-0213 claim 13) and its upgrades (S-0406 claim 12).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildingSwitches {
    /// Switched off by the power switch (S-0405 Terms).
    pub switched_off: bool,
    /// The primary factory of its kind (S-0213 claim 13).
    pub primary: bool,
    /// Upgrade types recorded on the building (S-0406 claim 12).
    pub upgrades: Vec<u32>,
    /// Repair switched on (S-0213 claim 13, S-0314).
    pub repairing: bool,
}

/// Applies a power-off or power-on order (S-0213 claim 13, S-0405 claim 10).
pub fn switch(w: &mut World, actor: ObjectId, on: bool) {
    let _ = (w, actor, on);
}

/// Whether the house has low power (S-0405 claim 4).
pub fn low_power(w: &World, house: HouseId) -> bool {
    let _ = (w, house);
    false
}
