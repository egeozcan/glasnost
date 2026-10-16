//! A building's change of owner (S-0509 claims 12 to 16).

use crate::world::{HouseId, ObjectId, World};

/// What caused a building ownership change (S-0509 claim 12): the cause
/// decides whether a capture or loss announcement is requested.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cause {
    /// An engineer capture (S-0509 claim 8): requests an announcement.
    Capture,
    /// The takeover of a vehicle-like building (S-0509 claim 17): requests
    /// an announcement.
    Takeover,
    /// A map trigger (S-0011 claim 24): no announcement.
    Trigger,
    /// The occupation rules of a civilian garrison building (S-0508 claims 7
    /// and 8): no announcement of this kind.
    Occupation,
}

/// A building changes owner to `new` (no effect when it is its owner
/// already).
pub fn change(w: &mut World, b: ObjectId, new: Option<HouseId>, cause: Cause) {
    let _ = cause;
    let Some(o) = w.obj(b) else { return };
    let old = o.owner;
    // spec: S-0509 Terms (a building ownership change has no effect when the new house is the old one)
    if old == new {
        return;
    }
    if let Some(ob) = w.obj_mut(b) {
        ob.owner = new;
    }
    crate::econ::events::owner_changed(w, b, old);
}
