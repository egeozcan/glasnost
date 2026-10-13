//! The harvest cycle (S-0402): loading, field and refinery choice, docking,
//! unloading and income. Slave miners (claim 14) and chrono miners (claim
//! 17) are seams until their locomotors and units arrive.

use serde::{Deserialize, Serialize};

/// A harvester's load and cycle state (S-0402 claims 1 to 13).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarvestState {}

/// A refinery's docks and queue (S-0402 claims 9 to 11).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DockState {}
