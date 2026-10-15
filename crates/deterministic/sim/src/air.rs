//! Aircraft flight, attack runs and pads (S-0501).

use serde::{Deserialize, Serialize};

use crate::world::{ObjectId, World};

/// An aircraft's flight state (S-0501 Terms).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AirState {}

/// Moves an aircraft with the flying locomotor in its turn (S-0501 claims 1 to 10); whether it moved.
pub fn tick(_w: &mut World, _id: ObjectId) -> bool {
    false
}
