//! Skirmish setup (S-0412) and country bonuses at game start (S-0413).

use serde::{Deserialize, Serialize};

/// The game's setup values (S-0412 claim 1).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {}

/// The game's superweapon setting (S-0412 claim 9; S-0407 claim 7 reads
/// it). A stub until S-0412 is implemented: superweapons on.
pub fn superweapons_allowed(w: &crate::world::World) -> bool {
    let _ = w;
    true
}

/// A house's setup values (S-0412 claims 4, 5 and 8).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HouseSetupState {}
