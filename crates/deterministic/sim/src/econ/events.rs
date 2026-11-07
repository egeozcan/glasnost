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
    // spec: S-0212 claim 13 (a building's exemption rectangle and an aircraft's exemption from its placement)
    crate::threat::entered(w, id);
    if kind == Kind::Building {
        // spec: S-0501 claim 30 (a placed helipad building adds to its house's pad total)
        super::tech::pad_placed(w, id);
    }
    // spec: S-0414 claim 4 (the on-map counts: an entry)
    crate::econ::victory::entered_map(w, id);
    crate::econ::power::entered_map(w, id);
    // spec: S-0515 claim 4 (a) (a building put on the map for the first time restarts its income countdown)
    crate::structures::tech::entered_map(w, id);
    if kind == Kind::Building
        && let Some(h) = owner
    {
        // spec: S-0410 claim 2 (a house that gains a building is rechecked)
        crate::econ::radar::mark_due(w, h);
    }
    crate::econ::shroud::entered_map(w, id);
    // spec: S-0516 claims 16, 22, 23 and 40 (entering play: a cloaked vehicle, sensor discs, arrays and detectors)
    crate::cloak::entered_map(w, id);
    if kind == Kind::Building {
        // spec: S-0616 claim 1 (natural smoke: a building whose type names a natural particle system gets one when placed in play, if it has none)
        crate::emitters::natural_smoke(w, id, false);
    }
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
    // spec: S-0212 claim 13 (the exemption a building or an aircraft gave is taken away when it leaves the map)
    crate::threat::leaving(w, id);
    if kind == Kind::Building {
        // spec: S-0501 claim 30 (a helipad building removed from the map takes its count from the pad total)
        super::tech::pad_removed(w, id);
    }
    // spec: S-0414 claim 4 (the on-map counts: a leaving)
    crate::econ::victory::leaving_map(w, id);
    crate::econ::power::leaving_map(w, id);
    // spec: S-0512 claim 2 (d) (a building giving a superweapon being removed: the availability check at once)
    crate::superweapons::charge::building_removed(w, id);
    crate::econ::shroud::leaving_map(w, id, for_good);
    // spec: S-0516 claims 20, 22, 23 and 40 (leaving play: discs, fields and building coverage)
    crate::cloak::leaving_map(w, id);
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
    if old != new
        && let Some(ob) = w.obj_mut(id)
    {
        // spec: S-0309 claim 30 (a holder that changes owner to another house, whatever the cause, drops its target at that moment, inside the change, whatever the target; no targeting-timer draw, the timer counts on; a change to the same house drops nothing)
        crate::spawn::clear_own_target(ob);
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
    }
    // spec: S-0414 claim 4 (a change of owner on the map: a leaving for the old owner, an entry for the new)
    crate::econ::victory::owner_changed(w, id, old);
    // spec: S-0414 claims 21 and 22 (an ownership gain's kill and score)
    crate::econ::stats::owner_changed(w, id, old);
    crate::econ::shroud::owner_changed(w, id);
    // spec: S-0516 claims 20, 22 and 23 (a change of owner moves discs and fields)
    crate::cloak::owner_changed(w, id, old);
    // spec: S-0601 claim 11 (a controller that changes owner keeps its victims; the switch off releases them)
    crate::mind::owner_changed(w, id);
    // spec: S-0212 claim 16a (an owner change moves the object's threat amount between the recording houses)
    crate::threat::owner_changed(w, id, old);
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
    // spec: S-0515 claims 3, 4 and 7 (an ownership change: the acquisition bonus from a passive old owner and the income countdown restarted)
    crate::structures::tech::owner_changed(w, id, old);
    // spec: S-0501 claim 30 (the pad total moves with a helipad building that changes owner)
    super::tech::pad_owner_changed(w, id, old);
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
            // spec: S-0508 claims 7 to 9 (a civilian garrison building's own turn: forced evacuation, the owner following the first occupant, the return of an abandoned building); claim 7 (r13: the owner change comes after the building's behaviour processing, its per-tick search and all its firing in that turn, which `tick::object_turn` runs before this)
            crate::structures::garrison::turn(w, id);
            crate::econ::power::building_turn(w, id);
            // The income cycle of a cash-producing building (S-0515 claims 5 and 6) comes earlier in the building's turn, before its behaviour processing (S-0608 claim 15; `tick::object_turn`).
            // spec: S-0402 claim 21 and S-0203 claim 12 (a building master's group update sees that turn's operation-state effects and animation advancement, and comes before its paid repair)
            crate::econ::slaves::group_turn(w, id);
            // spec: S-0314 claim 8 and S-0408 claim 13 (a building whose repair is on steps in its turn); S-0203 claim 12 (after the group update, so a delivery in it can pay this turn's step)
            crate::econ::sell::building_turn(w, id);
        }
        // A vehicle master's group update comes before its movement (S-0203 claim 12; `tick::object_turn`); its cell-entry disabling is at its arrivals (S-0405 claim 13).
        Kind::Vehicle => {}
        // spec: S-0401 claim 15 (an ore mine draws and spreads in its own turn)
        Kind::Terrain => crate::econ::resource::mine_turn(w, id),
        // spec: S-0402 claim 21 (a slave group's updates come during the master's turns)
        _ => crate::econ::slaves::group_turn(w, id),
    }
    crate::econ::shroud::object_turn(w, id, moved);
    if w.obj(id).is_some_and(|o| o.kind == Kind::Building) {
        // spec: S-0516 claims 18, 20, 23 and 42 (a building's ordinary cloak update follows its behaviour-fired shots, S-0203)
        crate::cloak::turn(w, id);
    }
    // spec: S-0516 claims 25 and 33 (an aircraft's hidden target and disguise when still, late in the object's own turn)
    crate::cloak::machine::late_turn(w, id);
}
