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
    if let Some(h) = owner {
        if kind == Kind::Building {
            // spec: S-0407 claim 12 (a) (a building put on the map)
            super::tech::recheck_due(w, h);
            // spec: S-0404 claim 8 (a factory building placed: its lines checked at once)
            super::production::factory_event(w, h, id, false);
        } else if kind.is_mobile() && !w.rules().quirks.unit_changes_no_recheck {
            // spec: S-0407 claim 12 (l) and quirks.unit_changes_no_recheck (off: a unit's entry is an event)
            super::tech::recheck_due(w, h);
        }
    }
}

/// An object is about to leave the map. `for_good`: it stops existing
/// (destroyed, sold, removed) rather than leaving the map for a while.
pub fn leaving_map(w: &mut World, id: ObjectId, for_good: bool) {
    let Some(o) = w.obj(id) else { return };
    let (kind, owner) = (o.kind, o.owner);
    if let Some(h) = owner {
        if kind == Kind::Building {
            // spec: S-0407 claim 12 (b) (a building leaves the map)
            super::tech::recheck_due(w, h);
            // spec: S-0404 claim 8 (a factory building leaving the map: its lines checked at once, without it)
            super::production::factory_event(w, h, id, true);
            if for_good {
                // spec: S-0404 claim 8 (a factory building removed for good)
                super::production::factory_removed(w, h, id);
            }
        } else if kind.is_mobile() && !for_good && !w.rules().quirks.unit_changes_no_recheck {
            // spec: S-0407 claim 12 (m) and quirks.unit_changes_no_recheck
            super::tech::recheck_due(w, h);
        }
        if for_good {
            // spec: S-0407 claim 12 (g) and (o) (an object stops existing)
            super::tech::object_ends(w, id);
        }
    }
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
    if kind.is_mobile() && !w.rules().quirks.unit_changes_no_recheck {
        // spec: S-0407 claim 12 (n) and quirks.unit_changes_no_recheck (both houses)
        for h in [old, new].into_iter().flatten() {
            super::tech::recheck_due(w, h);
        }
    }
    if kind != Kind::Building {
        return;
    }
    for h in [old, new].into_iter().flatten() {
        // spec: S-0407 claims 12 (a), (b) and 15 (a captured building counts for its new owner, no longer for the old)
        super::tech::recheck_due(w, h);
        // spec: S-0404 claim 8 (a factory building that changes owner: the lines of both houses)
        super::production::factory_event(w, h, id, false);
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
    let Some(h) = w.obj(id).and_then(|o| o.owner) else {
        return;
    };
    // spec: S-0407 claim 12 (c) (a building switched on or off)
    super::tech::recheck_due(w, h);
    // spec: S-0404 claim 8 (a factory building switched off or on: its lines checked at once)
    super::production::factory_event(w, h, id, false);
}
