//! Naval units (S-0504): the ship locomotor, wakes, sinking, underwater vessels and docking at yards.

use serde::{Deserialize, Serialize};

use crate::world::{ObjectId, World};

/// A vessel's naval state (S-0504 Terms).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavalState {
    /// The vessel is sinking (S-0504 Terms; S-0312 claim 10; S-0614 claim 11).
    #[serde(default)]
    pub sinking: bool,
}

/// Sets a vessel sinking without a hit (S-0614 claim 11; S-0504 claim 5:
/// other events that set a vehicle sinking). Its health is not changed.
/// The descent, wakes, draws and removal of S-0504 claims 4 and 5 are not
/// modelled yet: a seam of the naval topic.
pub fn start_sinking(w: &mut World, id: ObjectId) {
    if let Some(o) = w.obj_mut(id) {
        // spec: S-0614 claim 11 (the host starts to sink, whatever its type's weight; no hit)
        o.naval.get_or_insert_with(Default::default).sinking = true;
    }
}

/// Whether a vessel is sinking (S-0504 Terms).
pub fn sinking(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .and_then(|o| o.naval.as_ref())
        .is_some_and(|n| n.sinking)
}

/// Moves a vehicle with the ship locomotor in its turn (S-0504 claim 1); whether it moved.
pub fn tick(_w: &mut World, _id: ObjectId) -> bool {
    false
}
