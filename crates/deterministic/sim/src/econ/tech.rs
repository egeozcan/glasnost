//! The tech tree (S-0407): what a house may build.

use serde::{Deserialize, Serialize};

/// A house's offer, counts and flags (S-0407).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HouseTech {}
