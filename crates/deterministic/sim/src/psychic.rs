//! Psychic weapons (S-0610): the berserk state's start, countdown and end
//! (claims 4 and 5), the firing refusals of berserk weapons and berserk
//! firers (claims 3 and 6), the psychic sensor and its hostile-intent lines
//! (claims 9 to 12) and the psychic reveal superweapon (claims 13 to 15).
//!
//! The two immunities (claim 1) are type fields read where they apply:
//! damage resolution (`combat::damage`, S-0302 claims 8 and 10) and the
//! firing check (`combat::check`). Psychic pulses (claim 2) need no code of
//! their own: they are ordinary weapons with a psychic-damage warhead.
//!
//! A house's psychic sensors are derived, not stored: the buildings it owns
//! now whose type has a positive psychic-detection radius. So a placed
//! sensor joins its owner at once and a captured one moves to the new owner
//! at once (claim 9). The hostile-intent lines are a read-only query for the
//! drawing ([`intent_lines`]).
//!
//! The bunker half of claim 3 is [`bunker_refusal`] (S-0606).
//!
//! Seams:
//! - S-0613 (gas clouds): a gas particle's hit with a berserk-causing
//!   warhead goes through damage resolution like any other hit
//!   (`gas::damage_step`, S-0613 claim 13) and so reaches [`enter`].
//! - S-0612 (psychic domination, `crate::domination`) reads the same
//!   psionics immunity (claim 1) through `crate::mind::dominable`. S-0611 (mutation) is built
//!   (`crate::mutation`) and reads no psionics rule.

use glasnost_map::CellXY;
use glasnost_num::geom::root_floor;

use crate::combat::veterancy::type_combat;
use crate::rules::{Behaviour, Kind};
use crate::world::{HouseId, ObjectId, Pos, Target, World};

/// Leptons per cell of a sensor's reach (S-0610 F1).
pub const REACH_SCALE: i64 = 256; // spec: S-0610 Constants (sensor reach scale 256 leptons per cell)

/// The berserk tint of an object (claim 7, presentation): the position in
/// the colour-adjustment table (S-0006) of the extra additive tint a
/// berserk vehicle or infantry is drawn with; `None` for objects that are
/// not berserk and for aircraft and buildings. The view's drawing reads
/// [`berserk`] and the selector from the import (`mapview::protection`).
pub fn tint(w: &World, id: ObjectId) -> Option<i32> {
    let o = w.obj(id)?;
    // spec: S-0610 claim 7 (vehicles and infantry only; berserk aircraft and buildings get no tint)
    if !matches!(o.kind, Kind::Vehicle | Kind::Infantry) || o.combat.berserk.is_none() {
        return None;
    }
    // spec: S-0610 claim 7 and Import mapping (the berserk-colour field's table position, 0 when absent)
    Some(w.rules().superweapons.berserk_color)
}

/// Whether an object is berserk (S-0610 claims 4 and 5).
pub fn berserk(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.combat.berserk.is_some())
}

/// Drops an object's target, as entry and expiry do (claims 4 and 5),
/// resetting the burst counter with it (S-0301 claim 4).
fn drop_target(w: &mut World, id: ObjectId) {
    if let Some(ob) = w.obj_mut(id) {
        crate::spawn::clear_own_target(ob);
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
    }
    // spec: S-0107 r46 claim 11, Changes of target (the drop of the target when a hit makes the unit berserk, S-0302 claim 10 and S-0610 claim 4, ends an infantry unit's attack state and makes its request at the hit; the drop at the berserk state's end, in the unit's own update, is a change too)
    crate::infantry::watch_test(w, id);
}

/// A berserk hit takes effect (S-0302 claim 10) on `target` with duration
/// `t` (S-0610 claim 4). On a unit that is not berserk: the berserk state
/// starts with remaining time `t`, the unit drops its target and *hunt* is
/// queued, replacing whatever was queued. On a unit that is already
/// berserk: only the remaining time becomes `t`. With
/// `quirks.berserk_zero_duration` off a hit with `t` ≤ 0 has no effect.
/// Returns whether the hit had an effect.
pub fn enter(w: &mut World, target: ObjectId, t: i32) -> bool {
    if t <= 0 && !w.rules().quirks.berserk_zero_duration {
        // spec: S-0610 Edge cases (quirks.berserk_zero_duration off: a berserk hit with T ≤ 0 has no effect on the target)
        return false;
    }
    let Some(ob) = w.obj_mut(target) else {
        return false;
    };
    if ob.combat.berserk.is_some() {
        // spec: S-0610 claim 4 and S-0302 claim 10 (a hit on a unit already berserk only sets its remaining time to the new T; target and behaviour stay, as far as the berserk effect goes: the same hit's crush response comes after, S-0215 claim 39 and Changes for other specs)
        ob.combat.berserk = Some(t);
        return true;
    }
    // spec: S-0610 claim 4 (the remaining berserk time becomes T and the berserk state starts)
    ob.combat.berserk = Some(t);
    // spec: S-0610 claim 4 (hunt is queued as its next behaviour, replacing whatever was queued; it becomes current at the next behaviour change; the same hit's crush response then queues move in its place, S-0215 claim 39)
    ob.beh.queue(Some(Behaviour::Hunt));
    // spec: S-0610 claim 4 (it drops its target)
    drop_target(w, target);
    // Leaving a team (claim 4, the original) has no counterpart: the original's team system is out of scope.
    true
}

/// The berserk countdown in a unit's own update (S-0610 claim 5, F2): the
/// remaining time goes down by 1; in the update where it reaches 0 or less
/// the state ends, the remaining time is 0 (none), the unit drops its
/// target and *guard* is queued for a human-controlled owner (every player
/// house of our engine, AI houses included) or *hunt* otherwise (the
/// neutral and special houses).
pub fn countdown_turn(w: &mut World, id: ObjectId) {
    let Some(ob) = w.obj_mut(id) else { return };
    let Some(t) = ob.combat.berserk else { return };
    // spec: S-0610 F2 (t ← t − 1 in each update of the unit)
    let left = t.saturating_sub(1);
    if left > 0 {
        ob.combat.berserk = Some(left);
        return;
    }
    // spec: S-0610 claim 5 (berserk ends in the update where t reaches 0 or less; the remaining time is set to 0)
    ob.combat.berserk = None;
    let owner = ob.owner;
    // spec: S-0610 claim 5 and glossary AI house (guard for a human-controlled owner, an AI house included; hunt for the neutral and special houses)
    let human = owner.is_some_and(|h| w.houses.get(usize::from(h.0)).is_some_and(|x| x.human));
    let next = if human {
        Behaviour::Guard
    } else {
        Behaviour::Hunt
    };
    drop_target(w, id);
    if let Some(ob) = w.obj_mut(id) {
        ob.beh.queue(Some(next));
    }
}

/// The bunker half of claim 3: a berserk weapon is illegal against a
/// vehicle inside a tank bunker or a tank bunker that holds a vehicle
/// (S-0606 bunker link).
fn bunker_refusal(w: &World, target: ObjectId) -> bool {
    // spec: S-0610 claim 3 (a tank bunker that holds a vehicle, or a vehicle inside one)
    crate::structures::bunker::bunkered(w, target)
        || (crate::structures::bunker::is_bunker(w, target)
            && crate::structures::bunker::occupied(w, target))
}

/// The firing refusals of S-0610 at rank 4 of S-0301 claim 9: a weapon
/// whose warhead is berserk-causing against a target whose type has
/// psionics immunity or a bunkered vehicle (claim 3), and a berserk firer
/// against a target whose type is berserk-friendly (claim 6). Psychic-weapon
/// immunity adds no refusal (claim 1).
pub fn firing_refused(w: &World, firer: ObjectId, berserk_weapon: bool, target: Target) -> bool {
    let Target::Object(x) = target else {
        return false;
    };
    let Some(tc) = type_combat(w, x) else {
        return false;
    };
    // spec: S-0610 claim 3 (a berserk-causing warhead against a target whose type has psionics immunity: illegal)
    if berserk_weapon && (tc.immune_psionics || bunker_refusal(w, x)) {
        return true;
    }
    // spec: S-0610 claim 6 (a berserk firer against a target whose type is berserk-friendly: illegal)
    berserk(w, firer) && tc.berserk_friendly
}

// ---------------------------------------------------------------------------
// The psychic sensor (claims 9 to 12).

/// A building type's psychic-detection radius in cells (claim 9).
fn detection_radius(w: &World, type_index: u32) -> i32 {
    // spec: S-0610 Import mapping and Constants (PsychicDetectionRadius absent: 0, no sensor)
    w.rules()
        .buildings
        .get(type_index as usize)
        .and_then(|b| b.raw.econ.psychic_detection_radius)
        .unwrap_or(0)
}

/// A building's psychic-detection radius when it is a psychic sensor
/// (claim 9: its type's radius is above 0).
pub fn sensor_radius(w: &World, id: ObjectId) -> Option<i32> {
    let o = w.obj(id)?;
    if o.kind != Kind::Building {
        return None;
    }
    let r = detection_radius(w, o.type_index);
    (r > 0).then_some(r)
}

/// House `h`'s psychic sensors that count now (claim 9): the buildings on
/// the map it owns whose type has a psychic-detection radius above 0 and
/// that are in operation (S-0405 claim 8), in object order.
pub fn sensors(w: &World, h: HouseId) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|o| o.kind == Kind::Building && o.on_map && o.owner == Some(h))
        // spec: S-0610 claim 9 (a building whose type has a psychic-detection radius above 0 is one of its owner's sensors; captured, it is the new owner's at once)
        .filter(|o| sensor_radius(w, o.id).is_some())
        // spec: S-0610 claim 9 (a sensor counts only while it is in operation, S-0405)
        .filter(|o| crate::econ::power::in_operation(w, o.id))
        .map(|o| o.id)
        .collect()
}

/// Whether point `p` is within reach of sensor `s` (F1): the table root of
/// the squared spatial distance from the sensor's reference point,
/// truncated to whole leptons, at most 256 × R. No building size is
/// subtracted.
pub fn within_reach(w: &World, s: ObjectId, p: Pos) -> bool {
    let Some(r) = sensor_radius(w, s) else {
        return false;
    };
    let c = w.reference_point(s);
    let (dx, dy, dz) = (
        i64::from(p.x - c.x),
        i64::from(p.y - c.y),
        i64::from(p.z - c.z),
    );
    // spec: S-0610 F1 (Δx² + Δy² + Δz² in leptons, heights included; exact in double precision for map distances, S-0205)
    let n = (dx * dx + dy * dy + dz * dz) as u64;
    let rules = w.rules();
    // spec: S-0610 F1 and S-0204 claim 5 (the table root with its rounding, quirks.table_root_distance; truncated toward zero)
    let d = root_floor(n, rules.quirks.table_root_distance, rules.mode) as i64;
    // spec: S-0610 F1 (within reach when the distance is at most 256 × R; integer scaling and comparison)
    d <= REACH_SCALE * i64::from(r)
}

/// One hostile-intent line (claims 10 and 11): from the unit to its target
/// or destination, in world leptons, drawn in the unit owner's house
/// colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntentLine {
    /// The unit.
    pub unit: ObjectId,
    /// Its owner (the colour of the line's end squares).
    pub owner: Option<HouseId>,
    /// The start: the unit's firing point for a target, its position for
    /// a destination.
    pub from: Pos,
    /// The end: the target's aim point, or the destination raised by a
    /// bridge's height.
    pub to: Pos,
    /// The line points to an attack target (not a destination).
    pub to_target: bool,
}

/// A cell destination's end point: its centre, raised by a bridge's height
/// when the cell carries one (claim 11).
fn cell_end(w: &World, c: CellXY) -> Pos {
    // spec: S-0610 claim 11 (the destination raised by a bridge's height when the destination cell carries a bridge)
    crate::superweapons::cell_point(w, c, true)
}

/// The reference point of an intent's endpoint for the reach test (claim
/// 10): an object's reference point, a cell's centre.
fn reach_point(w: &World, t: Target) -> Pos {
    crate::combat::range::target_point(w, t)
}

/// Whether `viewer` sees the hostile-intent line of `unit` (claim 10), and
/// the line when it does. Eligibility is tested afresh at each call.
pub fn intent_line(w: &World, viewer: HouseId, unit: ObjectId) -> Option<IntentLine> {
    intent_line_with(w, viewer, unit, &sensors(w, viewer))
}

/// [`intent_line`] with the viewer's sensors that count now, `sensors`
/// ([`sensors`] of the same world and house), worked out by the caller.
fn intent_line_with(
    w: &World,
    viewer: HouseId,
    unit: ObjectId,
    sensors: &[ObjectId],
) -> Option<IntentLine> {
    let o = w.obj(unit)?;
    // spec: S-0610 claim 10 (an infantry, vehicle or aircraft in play)
    if !matches!(o.kind, Kind::Infantry | Kind::Vehicle | Kind::Aircraft)
        || !o.on_map
        || o.health <= 0
    {
        return None;
    }
    // spec: S-0610 claim 10 (not owned by the viewer's house, and the viewer's house does not count the owner as an ally: the viewer's side of the one-way alliance)
    if o.owner == Some(viewer) || w.allied(Some(viewer), o.owner) {
        return None;
    }
    if !w.rules().quirks.psychic_intent_ignores_visibility {
        // spec: S-0610 Edge cases (quirks.psychic_intent_ignores_visibility off: only a unit visible to the viewer's house: not hidden under S-0516 claim 18 and on a cell the viewer's house sees)
        let hidden = crate::cloak::hidden_from(w, unit, Some(viewer));
        if hidden || !crate::econ::shroud::seen(w, viewer, o.pos.cell()) {
            return None;
        }
    }
    // spec: S-0610 claim 10 (the destination: the last waypoint of a queued path, else the current destination; queued paths are not modelled, so the current destination)
    let (target, dest) = (o.target, o.dest);
    let reach = |t: Option<Target>| {
        t.is_some_and(|t| {
            let p = reach_point(w, t);
            sensors.iter().any(|&s| within_reach(w, s, p))
        })
    };
    // spec: S-0610 claim 10 (eligible when the target or the destination lies within a sensor's radius; the endpoint, not the unit's position, must be in reach)
    if !(reach(target) || reach(dest)) {
        return None;
    }
    // spec: S-0610 claim 10 (at most one line per unit: a target takes precedence over movement)
    let line = match (target, dest) {
        (Some(t), _) => {
            // spec: S-0610 claim 11 (from the unit's firing point to the target's aim point)
            let from = if crate::combat::weapon::slot_weapon(w, unit, 0).is_some() {
                crate::combat::position::fire_point(w, unit, 0, 0)
            } else {
                w.reference_point(unit)
            };
            IntentLine {
                unit,
                owner: o.owner,
                from,
                to: reach_point(w, t),
                to_target: true,
            }
        }
        (None, Some(d)) => IntentLine {
            unit,
            owner: o.owner,
            from: w.reference_point(unit),
            to: match d {
                Target::Cell(c) => cell_end(w, c),
                Target::Object(_) => reach_point(w, d),
            },
            to_target: false,
        },
        (None, None) => return None,
    };
    Some(line)
}

/// Every hostile-intent line `viewer` sees now (claim 10), in object order.
/// Visibility plays no part with `quirks.psychic_intent_ignores_visibility`
/// on (the original).
pub fn intent_lines(w: &World, viewer: HouseId) -> Vec<IntentLine> {
    // spec: S-0610 claim 9 and claim 10 (the sensors that count now, the same for every unit of one drawing: nothing between the units' tests changes the world)
    let sensors = sensors(w, viewer);
    if sensors.is_empty() {
        return Vec::new();
    }
    w.objects
        .keys()
        .filter_map(|&id| intent_line_with(w, viewer, id, &sensors))
        .collect()
}

/// The range ring a selected building shows (claim 12): its
/// psychic-detection radius in cells when its type shows a range ring and
/// the radius is at least 1, in preference to any cloak-sensor radius.
/// `None` when this claim gives no ring (the cloak-sensor ring is S-0516's).
pub fn range_ring(w: &World, id: ObjectId) -> Option<i32> {
    let o = w.obj(id)?;
    if o.kind != Kind::Building {
        return None;
    }
    let b = w.rules().buildings.get(o.type_index as usize)?;
    // spec: S-0610 Import mapping (HasRadialIndicator absent: no)
    let ring = b.raw.econ.has_radial_indicator.unwrap_or(false);
    let r = detection_radius(w, o.type_index);
    // spec: S-0610 claim 12 (a ring of the psychic-detection radius when the type shows one and the radius is at least 1)
    (ring && r >= 1).then_some(r)
}

// ---------------------------------------------------------------------------
// The psychic reveal (claims 13 to 15).

/// The psychic reveal's launch effect for house `h`, superweapon type `i`,
/// at `cell` (claims 13 to 15), in the launch tick: the area reveal of
/// S-0409 claim 17 without its high-ground test, and the activation sound
/// at the target cell's centre for every player. There is no launch
/// announcement and no radar event; the reveal adds no lasting sight, no
/// sensor coverage and no disguise detection, deals no damage and changes
/// no ownership or orders.
pub fn reveal(w: &mut World, h: HouseId, i: u32, cell: CellXY) {
    let p = crate::superweapons::cell_point(w, cell, false);
    let r = w.rules().superweapons.reveal_radius;
    // spec: S-0610 claim 14 (an area reveal of S-0409 claim 17 at the target cell's centre with the psychic-reveal radius; no high-ground test; for the launching house and the houses S-0409 claim 2 lets share it)
    crate::econ::shroud::psychic_reveal(w, h, p, r);
    // spec: S-0610 claim 13 (the activation sound at the target cell's centre, positionally, for every player; no launch announcement, no radar event)
    crate::superweapons::event(
        w,
        crate::superweapons::SwEvent::RevealSound,
        Some(h),
        Some(i),
        Some(p),
    );
}
