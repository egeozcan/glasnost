//! Skirmish setup (S-0412) and country bonuses at game start (S-0413).

use serde::{Deserialize, Serialize};

/// The game's setup values (S-0412 claim 1).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {}

/// A house's setup values (S-0412 claims 4, 5 and 8).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HouseSetupState {}
