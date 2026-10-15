//! Naval units (S-0504): the ship locomotor, wakes, sinking, underwater vessels and docking at yards.

use serde::{Deserialize, Serialize};

use crate::world::{ObjectId, World};

/// A vessel's naval state (S-0504 Terms).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavalState {}

/// Moves a vehicle with the ship locomotor in its turn (S-0504 claim 1); whether it moved.
pub fn tick(_w: &mut World, _id: ObjectId) -> bool {
    false
}
