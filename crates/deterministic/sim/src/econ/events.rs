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
    // spec: S-0414 claim 4 (the on-map counts: an entry)
    crate::econ::victory::entered_map(w, id);
    crate::econ::power::entered_map(w, id);
    if kind == Kind::Building
        && let Some(h) = owner
    {
        // spec: S-0410 claim 2 (a house that gains a building is rechecked)
        crate::econ::radar::mark_due(w, h);
    }
    crate::econ::shroud::entered_map(w, id);
    if let Some(h) = owner {
        if kind == Kind::Building {
            // spec: S-0407 claim 12 (a) (a building put on the map)
            super::tech::recheck_due(w, h);
            // spec: S-0215 claim 38 (an owned building's entry refreshes the base-start cell and radius)
            crate::base::refresh(w, h, None);
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
    // spec: S-0414 claim 4 (the on-map counts: a leaving)
    crate::econ::victory::leaving_map(w, id);
    crate::econ::power::leaving_map(w, id);
    crate::econ::shroud::leaving_map(w, id, for_good);
    crate::econ::resource::removed(w, id);
    crate::econ::harvest::leaving_map(w, id);
    if kind == Kind::Building
        && let Some(h) = owner
    {
        // spec: S-0410 claim 2 (a house that loses a building is rechecked)
        crate::econ::radar::mark_due(w, h);
    }
    if let Some(h) = owner {
        if kind == Kind::Building {
            // spec: S-0407 claim 12 (b) (a building leaves the map)
            super::tech::recheck_due(w, h);
            // spec: S-0215 claim 38 (an owned building's removal refreshes the records from the surviving buildings)
            crate::base::refresh(w, h, Some(id));
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
    // spec: S-0414 claim 4 (a change of owner on the map: a leaving for the old owner, an entry for the new)
    crate::econ::victory::owner_changed(w, id, old);
    // spec: S-0414 claims 21 and 22 (an ownership gain's kill and score)
    crate::econ::stats::owner_changed(w, id, old);
    crate::econ::shroud::owner_changed(w, id);
    if kind.is_mobile() && !w.rules().quirks.unit_changes_no_recheck {
        // spec: S-0407 claim 12 (n) and quirks.unit_changes_no_recheck (both houses)
        for h in [old, new].into_iter().flatten() {
            super::tech::recheck_due(w, h);
        }
    }
    if kind != Kind::Building {
        return;
    }
    if old != new
        && let Some(ob) = w.obj_mut(id)
    {
        // spec: S-0314 Terms (captured mark: set by a successful ownership change, never cleared)
        ob.combat.captured = true;
    }
    crate::econ::power::owner_changed(w, id, old);
    for h in [old, new].into_iter().flatten() {
        // spec: S-0410 claim 2 (a house that gains or loses a building is rechecked)
        crate::econ::radar::mark_due(w, h);
        // spec: S-0407 claims 12 (a), (b) and 15 (a captured building counts for its new owner, no longer for the old)
        super::tech::recheck_due(w, h);
        // spec: S-0404 claim 8 (a factory building that changes owner: the lines of both houses)
        super::production::factory_event(w, h, id, false);
        // spec: S-0215 claim 38 (a building's owner change refreshes the records of both affected houses)
        crate::base::refresh(w, h, None);
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
    let Some(owner) = w.obj(id).and_then(|o| o.owner) else {
        return;
    };
    // spec: S-0405 claim 7 (a building switched on or off makes its house due)
    crate::econ::power::make_due(w, owner);
    // spec: S-0410 claim 2 (an accepted power-switch change switching one of its buildings off or on is an event)
    crate::econ::radar::mark_due(w, owner);
    // spec: S-0407 claim 12 (c) (a building switched on or off)
    super::tech::recheck_due(w, owner);
    // spec: S-0404 claim 8 (a factory building switched off or on: its lines checked at once)
    super::production::factory_event(w, owner, id, false);
}

/// An object's own turn, after its locomotor (`moved`: it moved in this
/// tick). The topics that act in an object's turn (S-0405 claims 7, 9 and
/// 13; S-0409 claim 8) are called from here.
pub fn turn(w: &mut World, id: ObjectId, moved: bool) {
    let Some(o) = w.obj(id) else { return };
    match o.kind {
        Kind::Building => {
            crate::econ::power::building_turn(w, id);
            // spec: S-0203 claim 12 (a building master's group update sees that turn's operation-state effects and animation advancement, and comes before its paid repair)
            crate::econ::slaves::group_turn(w, id);
            // spec: S-0314 claim 8 and S-0408 claim 13 (a building whose repair is on steps in its turn); S-0203 claim 12 (after the group update, so a delivery in it can pay this turn's step)
            crate::econ::sell::building_turn(w, id);
        }
        // A vehicle master's group update comes before its movement (S-0203 claim 12; `tick::object_turn`).
        Kind::Vehicle => crate::econ::power::vehicle_turn(w, id),
        // spec: S-0401 claim 15 (an ore mine draws and spreads in its own turn)
        Kind::Terrain => crate::econ::resource::mine_turn(w, id),
        // spec: S-0402 claim 21 (a slave group's updates come during the master's turns)
        _ => crate::econ::slaves::group_turn(w, id),
    }
    crate::econ::shroud::object_turn(w, id, moved);
}
