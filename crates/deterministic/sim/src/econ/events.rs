//! Event hooks: the moments that several M5a topics react to. Each hook
//! calls the topics in a fixed order; a topic that reacts to an event adds
//! its call here and keeps its rule in its own module.
//!
//! The hooks are called by `crate::objects` (placing and removing objects),
//! by the power switch (S-0405 claim 10) and by owner changes.

use crate::rules::Kind;
use crate::world::{HouseId, ObjectId, World};

/// An object entered the map (placed by the map, built, released, deployed,
/// unloaded). `by_order`: a building put on the map by a place order
/// (S-0407 claim 10).
pub fn entered_map(w: &mut World, id: ObjectId, by_order: bool) {
    let Some(o) = w.obj(id) else { return };
    let (kind, owner) = (o.kind, o.owner);
    if kind == Kind::Building
        && let Some(h) = owner
        && let Some(house) = w.houses.get_mut(usize::from(h.0))
        && !house.econ.acquired.contains(&id)
    {
        house.econ.acquired.push(id);
    }
    let _ = by_order;
}

/// An object is about to leave the map. `for_good`: it stops existing
/// (destroyed, sold, removed) rather than leaving the map for a while.
pub fn leaving_map(w: &mut World, id: ObjectId, for_good: bool) {
    let Some(o) = w.obj(id) else { return };
    let (kind, owner) = (o.kind, o.owner);
    if kind == Kind::Building
        && for_good
        && let Some(h) = owner
        && let Some(house) = w.houses.get_mut(usize::from(h.0))
    {
        house.econ.acquired.retain(|&x| x != id);
    }
}

/// An object changed owner from `old` to its present owner (capture, mind
/// control, defeat hand-over).
pub fn owner_changed(w: &mut World, id: ObjectId, old: Option<HouseId>) {
    let Some(o) = w.obj(id) else { return };
    let (kind, new) = (o.kind, o.owner);
    if kind != Kind::Building {
        return;
    }
    if let Some(h) = old
        && let Some(house) = w.houses.get_mut(usize::from(h.0))
    {
        house.econ.acquired.retain(|&x| x != id);
    }
    if let Some(h) = new
        && let Some(house) = w.houses.get_mut(usize::from(h.0))
        && !house.econ.acquired.contains(&id)
    {
        house.econ.acquired.push(id);
    }
}

/// A building was switched off or on (S-0405 claim 10).
pub fn switched(w: &mut World, id: ObjectId) {
    let _ = (w, id);
}
