//! Parasites (S-0614): latching weapons, the terror drone's and the squid's attachment to a host, release and exit.

use serde::{Deserialize, Serialize};

/// An object's parasite links (S-0614 Terms).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParasiteState {}
