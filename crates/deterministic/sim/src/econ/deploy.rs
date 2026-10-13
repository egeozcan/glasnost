//! Deploying a vehicle into a building and packing it up again (S-0415).

use serde::{Deserialize, Serialize};

/// A deploy or pack-up in progress (S-0415 claims 2 to 11).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeployState {}
