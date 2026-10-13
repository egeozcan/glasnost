//! Radar availability (S-0410 claims 1 to 3); what the radar map shows is
//! the interface's (S-0410 claims 4 to 11).

use serde::{Deserialize, Serialize};

/// A house's radar availability (S-0410 claims 1 to 3).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HouseRadar {}
