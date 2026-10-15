//! Rocket flight (S-0506 claims 14 to 21): the rocket-flight movement mode of a spawned missile, from its aim point to its detonation.

use serde::{Deserialize, Serialize};

use crate::world::{ObjectId, World};

/// A rocket missile's flight state (S-0506 Terms).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RocketState {}

/// Moves a rocket missile in its turn (S-0506 claims 14 to 19); whether it moved.
pub fn tick(_w: &mut World, _id: ObjectId) -> bool {
    false
}
