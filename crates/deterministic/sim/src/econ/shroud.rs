//! Shroud and sight (S-0409): explored cells per player house.

use serde::{Deserialize, Serialize};

/// The explored state of every player house (S-0409 claim 1).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shroud {}

/// The sight an object has applied (S-0409 claim 8).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectSight {}
