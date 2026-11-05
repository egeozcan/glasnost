//! Vehicle bunkers (S-0606): a tank bunker takes in one bunker-eligible
//! vehicle through the entry stages of table S1, holds it as its bunkered
//! vehicle, and lets it out or releases it in place (claims 12 to 14).
//!
//! State: the bunker link is two-way, [`StructState::bunker_occupant`] on
//! the bunker and [`StructState::bunkered_in`] on the vehicle; the bunker's
//! entry stage is [`StructState::bunker_stage`], kept apart from the
//! behaviour's progress count because every behaviour start resets that
//! (S-0215 claim 3). Contact (S-0215 Terms) is the object's ordinary
//! `contact` field: a bunkered vehicle is also its bunker's contact
//! (S-0606 Terms).
//!
//! Revision 4 of S-0606 answers our requests Q-1300 to Q-1308: the chosen
//! object's distance (F5, `quirks.bunker_clear_far_point`), arrival
//! (claim 5), orders during entry and to a bunkered vehicle (claims 16
//! and 17), the destination refusal (claim 18), the release's open details
//! (claim 12), the track ends (claim 21), other removals (claim 19, our
//! departure) and a type that is also a repair depot (claim 20, whose
//! outcome stays open). Revision 7 answers Q-1163 (claim 20's order: one
//! S1 row, then the depot's decision on the state it left) and Q-1164
//! (claim 12: a released vehicle skips the deploy-fire rule).
//!
//! [`StructState::bunker_occupant`]: super::StructState::bunker_occupant
//! [`StructState::bunkered_in`]: super::StructState::bunkered_in
//! [`StructState::bunker_stage`]: super::StructState::bunker_stage

use glasnost_map::CellXY;
use glasnost_num::F64;
use glasnost_rules::sim::{LocomotionClass, MovementZone};
use serde::{Deserialize, Serialize};

use crate::combat::effects::{Effect, Sound};
use crate::rules::{Behaviour, Kind};
use crate::tracks::Point;
use crate::world::{Layer, ObjectId, Pos, Target, World};

/// A bunker's entry stage (S-0606 Terms, table S1).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stage {
    /// Clearing (also: not taking in a vehicle).
    #[default]
    Clearing,
    /// Waiting for clear cells.
    Waiting,
    /// Turning in.
    TurningIn,
    /// Driving in.
    DrivingIn,
    /// Turning to face south.
    TurningSouth,
    /// Raising walls.
    RaisingWalls,
    /// Occupied.
    Occupied,
}

/// Which bunker track a vehicle is on (S-0606 tables T1 and T2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BunkerTrack {
    /// An entry track, row E1 to E4 (1 to 4).
    Entry(u8),
    /// The release track.
    Release,
}

/// The entry processing delay (S-0606 claim 6), ticks, no draw.
// spec: S-0606 Constants (bunker entry processing delay: 15 ticks, no draw)
const ENTRY_DELAY: i32 = 15;

/// The docking displacement per axis (S-0606 F1), leptons.
// spec: S-0606 Constants (docking displacement: 128 leptons per axis)
const DOCK: i32 = 128;

/// The turning-in addend (S-0606 F3), facing units, exact.
// spec: S-0606 Constants (turning-in addend: 32767 facing units, not 32768)
const TURN_IN: i32 = 32767;

/// The bunkered body facing (S-0606 claim 7), map south.
// spec: S-0606 Constants (bunkered body facing: 32768 facing units, map south)
const SOUTH: u16 = 32768;

/// The clear-report keep distance (S-0606 claim 8), leptons, strict.
// spec: S-0606 Constants (clear-report keep distance: 384 leptons, strict)
const KEEP: i32 = 384;

/// The earlier guard distance (S-0606 claim 5, S-0215 claim 29), strict.
// spec: S-0606 Constants (earlier guard distance: 64 leptons)
const EARLY_START: i32 = 64;

/// The release point displacement (S-0606 F4), leptons.
// spec: S-0606 Constants (release point displacement: (−128, +128, 0) leptons)
const RELEASE: (i32, i32) = (-128, 128);

/// The exit search centre displacement (S-0606 claim 12), cells.
// spec: S-0606 Constants (exit search centre displacement: (−1, +1) cells from the top-left foundation cell)
const EXIT_CENTRE: (i32, i32) = (-1, 1);

/// The placeholder destination of a release with no nearby cell (S-0606
/// claim 12).
// spec: S-0606 Constants (no-cell placeholder destination: map cell (0, 0))
const NO_CELL: CellXY = CellXY { x: 0, y: 0 };

/// The *unload* delay after a release, ordinary type (S-0606 claim 12).
// spec: S-0606 Constants (unload delay after a release, ordinary type: 450 ticks, no draw)
const UNLOAD_DELAY: i32 = 450;

// ---------------------------------------------------------------------------
// Queries

/// Whether `k` is a tank bunker: a building whose type has the bunker flag
/// (S-0606 Terms and Import mapping, `Bunker`, absent off).
pub fn is_bunker(w: &World, k: ObjectId) -> bool {
    w.obj(k).is_some_and(|b| {
        b.kind == Kind::Building
            && w.rules()
                .buildings
                .get(b.type_index as usize)
                // spec: S-0606 Import mapping (Bunker: boolean, absent off)
                .is_some_and(|r| r.raw.bunker.unwrap_or(false))
    })
}

/// The bunker vehicle `id` is bunkered in (S-0606 Terms, bunker link).
pub fn bunkered_in(w: &World, id: ObjectId) -> Option<ObjectId> {
    w.obj(id).and_then(|o| o.structure.bunkered_in)
}

/// Whether `id` is a bunkered vehicle (S-0606 Terms).
pub fn bunkered(w: &World, id: ObjectId) -> bool {
    bunkered_in(w, id).is_some()
}

/// The bunkered vehicle of bunker `k` (S-0606 Terms).
pub fn occupant(w: &World, k: ObjectId) -> Option<ObjectId> {
    w.obj(k).and_then(|b| b.structure.bunker_occupant)
}

/// Whether `k` is an occupied bunker (S-0606 Terms).
pub fn occupied(w: &World, k: ObjectId) -> bool {
    occupant(w, k).is_some()
}

/// The objects in contact with bunker `k`, in object order.
fn contacts(w: &World, k: ObjectId) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|o| o.contact == Some(k) && o.id != k)
        .map(|o| o.id)
        .collect()
}

/// Whether bunker `k` has a contact place free for `id` (S-0606 claim 2
/// row P4: max(*d*, 1) places, S-0314 claim 17).
fn place_free(w: &World, k: ObjectId, id: ObjectId) -> bool {
    // spec: S-0606 claim 2 row P4 (contact places are max(d, 1) for the imported docking-place count d, S-0314 claim 17)
    let places = crate::econ::release::contact_places(w, k) as usize;
    contacts(w, k).into_iter().filter(|&x| x != id).count() < places
}

/// Whether vehicle `id` stands on a cell of bunker `k`'s foundation.
pub fn standing_on(w: &World, k: ObjectId, id: ObjectId) -> bool {
    let (Some(b), Some(o)) = (w.obj(k), w.obj(id)) else {
        return false;
    };
    crate::objects::foundation_cells(b.pos.cell(), b.foundation).contains(&o.pos.cell())
}

/// Whether `id` is bunker-eligible (S-0606 claim 1).
pub fn eligible(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let Some(u) = w.rules().unit(o.kind, o.type_index) else {
        return false;
    };
    // spec: S-0606 claim 1 and Import mapping (Bunkerable: absent on for vehicle types, off for every other kind)
    let flag = u.raw.bunkerable.unwrap_or(o.kind == Kind::Vehicle);
    // spec: S-0606 claim 1 (the turret flag)
    let turret = u.turret;
    // spec: S-0606 claim 1 (a turret weapon: its current weapon names a weapon type, S-0308 claims 1 and 2)
    let weapon = crate::combat::weapon::current_weapon(w, id).is_some();
    // spec: S-0606 claim 1 (its locomotion class is not hover, S-0211)
    let not_hover = u.class != Some(LocomotionClass::Hover);
    // spec: S-0606 claim 1 (no parasite is inside it, S-0614)
    let no_parasite = crate::parasite::parasite_of(w, id).is_none();
    flag && turret && weapon && not_hover && no_parasite
}

/// Bunker `k`'s answer to actor `id`'s permission query (S-0606 claim 2,
/// first matching row of P1 to P10): `true` only for P10's acceptance.
pub fn permits(w: &World, k: ObjectId, id: ObjectId) -> bool {
    if !is_bunker(w, k) {
        return false;
    }
    let (Some(b), Some(o)) = (w.obj(k), w.obj(id)) else {
        return false;
    };
    if b.health <= 0 || !b.on_map {
        return false;
    }
    // spec: S-0606 claim 2 row P1 (not allied with the actor's house: no answer, not permitted)
    if !w.allied(b.owner, o.owner) {
        return false;
    }
    // spec: S-0606 claim 2 row P2 (current behaviour construction or selling: refuses)
    if matches!(
        crate::econ::placement::seen_behaviour(b),
        Some(Behaviour::Construction | Behaviour::Selling)
    ) || b.econ.sale.is_some()
    {
        return false;
    }
    // spec: S-0606 claim 2 row P3 (build-up not finished: refuses)
    if b.econ.construction.building_up {
        return false;
    }
    // spec: S-0606 claim 2 row P4 (no free contact place for the actor: refuses; the initialisation exemption needs no rule, no order can query permission then)
    if !place_free(w, k, id) {
        return false;
    }
    // spec: S-0606 claim 2 row P5 (S-0504 claim 7's naval-match test fails: refuses)
    if crate::naval::naval_match_refuses(w, k, id) {
        return false;
    }
    // spec: S-0606 claim 2 row P6 (the actor's type has the balloon-hover flag: refuses)
    if w.rules()
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.raw.balloon_hover.unwrap_or(false))
    {
        return false;
    }
    // spec: S-0606 claim 2 row P7 (switched off: refuses; low power alone is not a refusal)
    if b.econ.building.switched_off {
        return false;
    }
    // spec: S-0606 claim 2 row P8 (not bunker-eligible: refuses)
    if !eligible(w, id) {
        return false;
    }
    // spec: S-0606 claim 2 row P9 (the actor stands on a cell of this bunker's foundation: refuses)
    if standing_on(w, k, id) {
        return false;
    }
    // spec: S-0606 claim 2 row P10 (otherwise accepts)
    true
}

/// The direction *d* of vehicle `id` from bunker `k`'s reference point
/// (S-0204 claim 8, F3).
fn direction_from_reference(w: &World, k: ObjectId, id: ObjectId) -> u16 {
    let c = w.reference_point(k);
    let p = w.obj(id).map_or(c, |o| o.pos);
    // spec: S-0606 F1 (d: the direction of the vehicle from the reference point, S-0204 F3)
    glasnost_num::geom::direction(
        i64::from(p.x - c.x),
        i64::from(p.y - c.y),
        w.rules().quirks.table_direction,
    )
}

/// A facing rounded to 1/256 turns, halves up (S-0606 F1 and F2).
fn round_256(f: u16) -> i32 {
    // spec: S-0606 F1 and F2 (⌊(⌊f / 128⌋ + 1) / 2⌋ mod 256, floor division of non-negative values)
    ((i32::from(f) / 128 + 1) / 2) % 256
}

/// The docking displacement of a quadrant *q* (S-0606 F1).
fn quadrant(q: i32) -> (i32, i32) {
    // spec: S-0606 F1 (q 0 … 63: (+128, −128); 64 … 127: (+128, +128); 128 … 191: (−128, +128); 192 … 255: (−128, −128))
    match q {
        0..=63 => (DOCK, -DOCK),
        64..=127 => (DOCK, DOCK),
        128..=191 => (-DOCK, DOCK),
        _ => (-DOCK, -DOCK),
    }
}

/// The docking point bunker `k` gives vehicle `id` (S-0606 claim 4): the
/// reference point plus the quadrant's displacement, from the vehicle's
/// position now.
pub fn docking_point(w: &World, k: ObjectId, id: ObjectId) -> Pos {
    let c = w.reference_point(k);
    let q = round_256(direction_from_reference(w, k, id));
    let (dx, dy) = quadrant(q);
    // spec: S-0606 claim 4 and F4 (exact integer-lepton additions, at the reference point's height)
    Pos::new(c.x + dx, c.y + dy, c.z)
}

/// The heading point of a vehicle whose destination is the bunker it is in
/// contact with (S-0606 claim 4: the docking point); `None` otherwise.
pub fn heading_point(w: &World, id: ObjectId, dest: ObjectId) -> Option<Pos> {
    let o = w.obj(id)?;
    // spec: S-0606 claim 4 (a vehicle approaching a bunker drives toward the docking point)
    (o.contact == Some(dest) && is_bunker(w, dest) && o.kind == Kind::Vehicle)
        .then(|| docking_point(w, dest, id))
}

// ---------------------------------------------------------------------------
// Entry

/// An entry request of `id` to bunker `k` (S-0606 claim 3). Returns
/// whether the bunker accepts.
pub fn request(w: &mut World, id: ObjectId, k: ObjectId) -> bool {
    let Some(b) = w.obj(k) else { return false };
    if b.health <= 0 || !b.on_map {
        return false;
    }
    // spec: S-0606 claim 3 (switched off: refuses)
    if b.econ.building.switched_off {
        return false;
    }
    let Some(o) = w.obj(id) else { return false };
    if o.contact == Some(k) {
        // spec: S-0606 claim 3 (already in contact: refuses if the actor is no longer bunker-eligible, otherwise accepts)
        return eligible(w, id);
    }
    if place_free(w, k, id) {
        // spec: S-0606 claim 3 (not yet in contact, a contact place free: the bunker takes the actor as a contact and accepts)
        let to_bunker = o.dest == Some(Target::Object(k));
        if let Some(ob) = w.obj_mut(id) {
            ob.contact = Some(k);
        }
        if to_bunker {
            // spec: S-0606 claim 4 and S-0207 claim 25 (the bunker as destination supplies its docking point once contact holds)
            crate::drive::take_destination(w, id);
        }
        return true;
    }
    // spec: S-0606 claim 3 (not yet in contact, no contact place free: acceptance without contact)
    true
}

/// After an enter order's ordinary effects (S-0606 claim 3): a vehicle in
/// *enter* holding no contact that is given a bunker as destination.
pub fn order_entered(w: &mut World, id: ObjectId, b: Behaviour, dest: Option<Target>) {
    if b != Behaviour::Enter {
        return;
    }
    let Some(Target::Object(k)) = dest else {
        return;
    };
    if !is_bunker(w, k) || crate::depot::is_depot(w, k) {
        return;
    }
    let Some(o) = w.obj(id) else { return };
    if o.kind != Kind::Vehicle || o.contact.is_some() {
        return;
    }
    if !contacts(w, k).is_empty() {
        // spec: S-0606 claim 3 (the bunker already holds any contact: the destination is cleared and it does not move toward the bunker)
        clear_destination(w, id);
        return;
    }
    // spec: S-0606 claim 3 (otherwise one entry request at once; on refusal it ends contact and its destination is cleared)
    if !request(w, id, k) {
        end_contact(w, id, k);
        clear_destination(w, id);
    }
}

fn clear_destination(w: &mut World, id: ObjectId) {
    crate::econ::assign_destination(w, id, None);
    if let Some(ob) = w.obj_mut(id) {
        ob.path.steps.clear();
        // spec: S-0212 claim 21 and its state table, row "destination changes" (r97: for the driving and walk locomotors a different destination, or none, ends a running retry delay, path::destination_changed and path::waits; the ship and the other locomotors keep it, low confidence; the hovering locomotor's is S-0503's)
    }
}

/// Ends `id`'s contact with `k`, if it holds it.
fn end_contact(w: &mut World, id: ObjectId, k: ObjectId) {
    if let Some(ob) = w.obj_mut(id)
        && ob.contact == Some(k)
    {
        ob.contact = None;
    }
}

/// The bunker a vehicle's *enter* goes to: its contact when that is a
/// bunker, else its destination when that is one (S-0215 claim 29).
fn partner(w: &World, id: ObjectId) -> Option<ObjectId> {
    let o = w.obj(id)?;
    if let Some(c) = o.contact
        && is_bunker(w, c)
    {
        return Some(c);
    }
    match o.dest {
        Some(Target::Object(k)) if is_bunker(w, k) => Some(k),
        _ => None,
    }
}

/// Whether a vehicle's *enter* goes to a tank bunker (S-0606 claim 3).
pub fn entering_bunker(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.kind == Kind::Vehicle) && partner(w, id).is_some()
}

/// A vehicle's *enter* processing toward a tank bunker (S-0606 claims 3 to
/// 5; S-0215 claim 29). Returns the delay.
pub fn process_enter(w: &mut World, id: ObjectId) -> i32 {
    let Some(k) = partner(w, id) else {
        crate::process::fall_back(w, id, false);
        return crate::process::standard_delay(w, id);
    };
    // spec: S-0606 claim 3 and S-0215 claim 29 (the repeated entry request; a refusal ends the untethered vehicle's contact and it falls back)
    if !request(w, id, k) {
        end_contact(w, id, k);
        crate::process::fall_back(w, id, false);
        return crate::process::standard_delay(w, id);
    }
    // spec: S-0606 claim 5 (revision 4, Q-1301 answered: the enter processing makes no arrival test; standing still in the docking cell is not arrival)
    let Some(o) = w.obj(id) else {
        return crate::process::standard_delay(w, id);
    };
    let to_bunker = o.dest == Some(Target::Object(k));
    if !standing_on(w, k, id) && (!to_bunker || o.path.steps.is_empty()) && o.contact == Some(k) {
        // spec: S-0606 claim 4 and S-0215 claim 29 (the bunker is kept as destination while the vehicle approaches; the docking point is the movement point)
        crate::econ::assign_destination(w, id, Some(Target::Object(k)));
        if let Some(ob) = w.obj_mut(id) {
            ob.path.steps.clear();
            // spec: S-0212 claim 21 and its state table, row "destination changes" (r97: for the driving and walk locomotors a different destination, or none, ends a running retry delay, path::destination_changed and path::waits; the ship and the other locomotors keep it, low confidence; the hovering locomotor's is S-0503's)
        }
    }
    // spec: S-0215 claim 29 (enter chooses the standard delay)
    crate::process::standard_delay(w, id)
}

/// The arrival of an entering vehicle at its bunker (S-0606 claim 5): the
/// movement-arrival test, called at a reached track target only.
/// Returns whether it arrived.
pub fn arrival(w: &mut World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let Some(k) = o.contact else { return false };
    // spec: S-0606 claim 20 (a type that is also a repair depot: arrival is the depot's movement-arrival exchange, S-0314 claim 17)
    if o.kind != Kind::Vehicle
        || !is_bunker(w, k)
        || bunkered(w, id)
        || crate::depot::is_depot(w, k)
    {
        return false;
    }
    // spec: S-0215 claim 29 (revision 64: the movement-arrival test also applies at a tank bunker, made by movement into a cell, not by the enter processing)
    // spec: S-0606 claim 5 (in enter, or in patrol, which the same test accepts; contact and destination both the bunker; its cell the docking cell)
    if !matches!(o.beh.current, Some(Behaviour::Enter | Behaviour::Patrol))
        || o.dest != Some(Target::Object(k))
        || o.pos.cell() != docking_point(w, k, id).cell()
    {
        return false;
    }
    // spec: S-0606 claim 5 (it ends its route and stops, but its destination stays the bunker, so a later movement into the docking cell can report again) and S-0207 claim 25 (a stop clears the locomotor destination point)
    crate::drive::stop(w, id);
    crate::drive::stop_point(w, id);
    if let Some(ob) = w.obj_mut(id) {
        ob.path.steps.clear();
        // spec: S-0212 claim 21 and its state table, row "destination changes" (r97: for the driving and walk locomotors a different destination, or none, ends a running retry delay, path::destination_changed and path::waits; the ship and the other locomotors keep it, low confidence; the hovering locomotor's is S-0503's)
    }
    // spec: S-0606 claim 5 (the completion report: accepted unless the bunker's current behaviour is selling; it sets its ready mark and queues repair without an immediate start; the vehicle stays in enter and queues no sleep)
    if let Some(b) = w.obj_mut(k)
        && b.beh.current != Some(Behaviour::Selling)
    {
        b.beh.ready_mark = true;
        b.beh.queue(Some(Behaviour::Repair));
    }
    true
}

/// The early start of a bunker in its guard processing (S-0606 claim 5;
/// S-0215 claim 29): a contacted actor in *enter* standing still strictly
/// within 64 leptons of the reference point makes the bunker queue
/// *repair*. `Some(1)` when it applies.
pub fn early_start(w: &mut World, k: ObjectId) -> Option<i32> {
    if !is_bunker(w, k) || crate::depot::is_depot(w, k) {
        return None;
    }
    let c = w.reference_point(k);
    let t = w.tick;
    let hit = contacts(w, k).into_iter().any(|v| {
        w.obj(v).is_some_and(|o| {
            o.beh.current == Some(Behaviour::Enter)
                && !crate::occupancy::in_progress(w, v)
                && !o.body.turning(t)
                // spec: S-0606 claim 5 and S-0215 claim 29 (standing still within 64 leptons of the reference point)
                && crate::naval::depot_distance(o.pos, c) < EARLY_START
        })
    });
    if !hit {
        return None;
    }
    if let Some(b) = w.obj_mut(k) {
        // spec: S-0606 claim 5 (a bunker in guard queues repair)
        b.beh.queue(Some(Behaviour::Repair));
    }
    // spec: S-0215 claim 29 (the earlier guard transition's 1-tick delay)
    Some(1)
}

// ---------------------------------------------------------------------------
// Entry stages (table S1)

fn set_stage(w: &mut World, k: ObjectId, s: Stage) {
    if let Some(b) = w.obj_mut(k) {
        b.structure.bunker_stage = s;
    }
}

/// The vehicle being taken in (S-0606 claim 6): the linked vehicle, else
/// the first contact.
fn taken_in(w: &World, k: ObjectId) -> Option<ObjectId> {
    // spec: S-0606 claim 6 (the bunker's linked vehicle if any, otherwise its first contact)
    occupant(w, k).or_else(|| contacts(w, k).first().copied())
}

/// The candidates of foundation cell `c` of bunker `k` (S-0606 claim 6):
/// the infantry, vehicles, aircraft and buildings listed in the cell's
/// ground layer, the bunker itself excepted, in list order.
fn candidates(w: &World, k: ObjectId, c: CellXY) -> Vec<ObjectId> {
    w.cells
        .list(c, Layer::Ground)
        .iter()
        .copied()
        .filter(|&x| {
            // spec: S-0606 claim 6 (candidates: the infantry, vehicles, aircraft and buildings listed in the cell's ground layer, except the bunker itself; terrain objects, overlays, crates, animations and deck-layer objects never)
            x != k
                && w.obj(x).is_some_and(|o| {
                    o.on_map
                        && o.health > 0
                        && matches!(
                            o.kind,
                            Kind::Infantry | Kind::Vehicle | Kind::Aircraft | Kind::Building
                        )
                })
        })
        .collect()
}

/// F5's chosen-object distance *e* of a candidate at `p` (S-0606 F5).
pub fn chosen_distance(w: &World, p: Pos) -> u64 {
    let r = w.rules();
    // spec: S-0606 F5 and Constants (a = 896 leptons per axis from the cell's top corner; 128, the centre, with quirks.bunker_clear_far_point off)
    let a: i64 = if r.quirks.bunker_clear_far_point {
        896
    } else {
        128
    };
    // spec: S-0606 F5 (u = x mod 256, w = y mod 256, the residues 0 to 255; z plays no part)
    let u = i64::from(p.x.rem_euclid(256));
    let v = i64::from(p.y.rem_euclid(256)); // spec: S-0606 F5 (w = y mod 256)
    // spec: S-0606 F5 (n = (u − a)² + (w − a)², exact; e = ⌊r(n)⌋, the table root of S-0204 F1, quirks.table_root_distance)
    let n = (u - a) * (u - a) + (v - a) * (v - a);
    glasnost_num::geom::root_floor(n as u64, r.quirks.table_root_distance, r.mode)
}

/// The objects examined on foundation cell `c` of bunker `k` for vehicle
/// `v` (S-0606 claim 6 and Edge cases, `quirks.bunker_clear_chosen_only`):
/// with the switch on, the cell's chosen object (F5), unless it is the
/// entering vehicle; off, every candidate other than the entering vehicle.
fn examined(w: &World, k: ObjectId, v: ObjectId, c: CellXY) -> Vec<ObjectId> {
    let listed = candidates(w, k, c);
    if !w.rules().quirks.bunker_clear_chosen_only {
        // spec: S-0606 Edge cases (quirks.bunker_clear_chosen_only off: every candidate of claim 6 other than the entering vehicle on the cell)
        return listed.into_iter().filter(|&x| x != v).collect();
    }
    let mut best: Option<(u64, ObjectId)> = None;
    for x in listed {
        let Some(o) = w.obj(x) else { continue };
        let e = chosen_distance(w, o.pos);
        // spec: S-0606 claim 6 (the smallest e, compared as integers; on a tie the candidate listed first wins)
        if best.is_none_or(|(be, _)| e < be) {
            best = Some((e, x));
        }
    }
    match best {
        // spec: S-0606 claim 6 and S1 (the chosen object, unless it is the entering vehicle)
        Some((_, x)) if x != v => vec![x],
        _ => Vec::new(),
    }
}

fn foundation(w: &World, k: ObjectId) -> Vec<CellXY> {
    w.obj(k).map_or_else(Vec::new, |b| {
        crate::objects::foundation_cells(b.pos.cell(), b.foundation)
    })
}

/// Whether the bunker's health ratio is at or below the yellow condition
/// ratio (S-0606 Terms, walls effects; S-0108).
fn damaged(w: &World, k: ObjectId) -> bool {
    let r = w.rules();
    let mode = r.mode;
    let h = w.obj(k).map_or(0, |b| b.health);
    let s = crate::combat::damage::strength(w, k);
    // spec: S-0606 Terms (the damaged variant when the health ratio is at or below the yellow condition ratio, as in S-0108)
    s != 0
        && F64::from_i64(i64::from(h), mode)
            .div(F64::from_i64(i64::from(s), mode), mode)
            .le(r.general.condition_yellow)
}

fn type_of(w: &World, id: ObjectId) -> Option<crate::econ::TypeRef> {
    w.obj(id).map(|o| crate::econ::TypeRef {
        kind: o.kind,
        index: o.type_index,
    })
}

/// A walls sound at the bunker's position (S-0606 claims 7, 12 and 14).
fn walls_sound(w: &mut World, k: ObjectId, sound: Sound) {
    let of = type_of(w, k);
    let house = w.obj(k).and_then(|b| b.owner);
    let pos = Some(w.reference_point(k));
    w.effects.push(Effect::Sound {
        sound,
        of,
        house,
        pos,
    });
}

/// The walls-down effects (S-0606 claim 12): *special-1* and *special-2*
/// stop, the walls-down sound, *special-3* and *special-4* start.
fn walls_down(w: &mut World, k: ObjectId) {
    let d = damaged(w, k);
    // spec: S-0606 claim 12 (special-1 and special-2 stop)
    w.effects.push(Effect::SpecialAnims {
        building: k,
        slots: [1, 2],
        start: false,
        damaged: d,
    });
    // spec: S-0606 claim 12 (the imported walls-down sound at the bunker's position, if one is set)
    walls_sound(w, k, Sound::BunkerWallsDown);
    // spec: S-0606 claim 12 (special-3 and special-4 start, normal or damaged variant)
    w.effects.push(Effect::SpecialAnims {
        building: k,
        slots: [3, 4],
        start: true,
        damaged: d,
    });
}

/// A bunker's *repair* processing: one row of table S1 (S-0606 claim 6).
/// Returns the delay.
pub fn process_repair(w: &mut World, k: ObjectId) -> i32 {
    let t = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let stage = w
        .obj(k)
        .map_or(Stage::Clearing, |b| b.structure.bunker_stage);
    let v = taken_in(w, k).filter(|&v| w.obj(v).is_some_and(|o| o.kind == Kind::Vehicle));
    let Some(v) = v else {
        if stage == Stage::RaisingWalls {
            // spec: S-0606 claim 16 and S1 (an entry abandoned after raising walls started special-1 and special-2: they stop on the return to clearing, with no sound and no special-3 or special-4; the original's running animations are a display defect, fixed without a switch)
            let d = damaged(w, k);
            w.effects.push(Effect::SpecialAnims {
                building: k,
                slots: [1, 2], // spec: S-0606 claim 16 (special-1 and special-2)
                start: false,
                damaged: d,
            });
        }
        // spec: S-0606 S1 (no vehicle to take in, or it is not a vehicle: clearing, the bunker queues guard without an immediate start)
        set_stage(w, k, Stage::Clearing);
        if let Some(b) = w.obj_mut(k) {
            b.beh.queue(Some(Behaviour::Guard));
        }
        return ENTRY_DELAY;
    };
    let rate = w
        .obj(v)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        .map_or(0, |u| u.turn_rate);
    let still = |w: &World| !crate::occupancy::in_progress(w, v);
    let turning = |w: &World| w.obj(v).is_some_and(|o| o.body.turning(t));
    match stage {
        Stage::Clearing => {
            // spec: S-0606 S1 (clearing: the vehicle stands on a foundation cell of this bunker and is not moving)
            if standing_on(w, k, v) && still(w) {
                set_stage(w, k, Stage::Waiting);
                let threat = w.obj(k).map(|b| b.pos);
                for c in foundation(w, k) {
                    for x in examined(w, k, v, c) {
                        // spec: S-0606 claim 6 (each request forced, not move-anywhere, with the bunker's position as threat point; S-0215 claim 17 carries them out)
                        crate::scatter::request(
                            w,
                            x,
                            crate::scatter::ScatterRequest {
                                threat,
                                forced: true,
                                anywhere: false,
                            },
                        );
                    }
                }
            }
        }
        Stage::Waiting => {
            // spec: S-0606 S1 (waiting: each foundation cell's chosen object, F5, is absent or is the entering vehicle)
            let clear = foundation(w, k)
                .into_iter()
                .all(|c| examined(w, k, v, c).is_empty());
            if clear {
                set_stage(w, k, Stage::TurningIn);
                let d = direction_from_reference(w, k, v);
                // spec: S-0606 F3 ((d + 32767) mod 65536, d measured when the stage is reached)
                let f = ((i32::from(d) + TURN_IN) % 65536) as u16;
                if let Some(o) = w.obj_mut(v) {
                    o.body.turn_to(f, t, rate);
                }
            }
        }
        Stage::TurningIn => {
            // spec: S-0606 S1 (turning in: the body is not turning)
            if !turning(w) {
                set_stage(w, k, Stage::DrivingIn);
                // spec: S-0606 F2 (the row from the body facing rounded to 1/256 turns: 32, 96 and 224 select E1, E2 and E4; every other value E3)
                let row = match round_256(w.body_facing(v)) {
                    32 => 1,
                    96 => 2,
                    224 => 4,
                    _ => 3,
                };
                let target = w.reference_point(k);
                // spec: S-0606 claim 6 (the entry track toward the reference point as its track target, ρₜ = ρ = 1.0)
                crate::drive::begin_bunker_track(w, v, BunkerTrack::Entry(row), target);
            }
        }
        Stage::DrivingIn => {
            // spec: S-0606 S1 (driving in: on a foundation cell and not moving)
            if standing_on(w, k, v) && still(w) {
                set_stage(w, k, Stage::TurningSouth);
                if let Some(o) = w.obj_mut(v) {
                    // spec: S-0606 S1 (desired body facing 32768)
                    o.body.turn_to(SOUTH, t, rate);
                }
            }
        }
        Stage::TurningSouth => {
            // spec: S-0606 S1 (turning to face south: the body is not turning)
            if !turning(w) {
                set_stage(w, k, Stage::RaisingWalls);
                let d = damaged(w, k);
                // spec: S-0606 S1 (special-1 and special-2 start, normal or damaged variant)
                w.effects.push(Effect::SpecialAnims {
                    building: k,
                    slots: [1, 2],
                    start: true,
                    damaged: d,
                });
            }
        }
        Stage::RaisingWalls => {
            // spec: S-0606 S1 (raising walls: always occupied)
            set_stage(w, k, Stage::Occupied);
            // spec: S-0606 claim 7 (the bunker link is set)
            if let Some(b) = w.obj_mut(k) {
                b.structure.bunker_occupant = Some(v);
            }
            if let Some(o) = w.obj_mut(v) {
                o.structure.bunkered_in = Some(k);
            }
            // spec: S-0606 claim 7 (the vehicle leaves any control group and is removed from the selection)
            w.effects.push(Effect::Bunkered {
                vehicle: v,
                bunker: k,
            });
            // spec: S-0606 claim 7 (the vehicle queues guard with an immediate start, S-0215 claim 7)
            let ready = crate::process::ready(w, v);
            if let Some(o) = w.obj_mut(v) {
                o.beh
                    .queue_and_start(Some(Behaviour::Guard), ready, t, stale);
            }
            // spec: S-0606 claim 7 (the imported walls-up sound at the bunker's position, if one is set)
            walls_sound(w, k, Sound::BunkerWallsUp);
        }
        // spec: S-0606 S1 (occupied: nothing)
        Stage::Occupied => {}
    }
    // spec: S-0606 claim 6 (a delay of 15 ticks, no draw)
    ENTRY_DELAY
}

// ---------------------------------------------------------------------------
// Clear report (claim 8)

/// A clear report of `v` to bunker `k` (S-0606 claim 8). Returns `true`
/// when the bunker gives the accepting answer and contact is kept. No
/// route of ours makes the report: S-0417 claim 7's report comes only from
/// a tethered unit, and a bunker never tethers (claim 8 is `unmeasured`
/// for the same reason).
pub fn clear_report(w: &mut World, k: ObjectId, v: ObjectId) -> bool {
    let c = w.reference_point(k);
    let Some(p) = w.obj(v).map(|o| o.pos) else {
        return false;
    };
    // spec: S-0606 claim 8 (the spatial distance from the reference point, truncated toward zero, not the table root; strictly below 384 keeps contact)
    if crate::naval::depot_distance(p, c) < KEEP {
        return true;
    }
    // spec: S-0606 claim 8 (otherwise contact ends and the bunker gives the run-away reply, S-0417 claim 8, with its destination rules)
    end_contact(w, v, k);
    crate::econ::release::run_away(w, v);
    false
}

// ---------------------------------------------------------------------------
// Releases (claims 12 to 15)

/// Ends the bunker link on both sides and the bunker's contact with `v`.
fn unlink(w: &mut World, k: ObjectId, v: ObjectId) {
    if let Some(b) = w.obj_mut(k)
        && b.structure.bunker_occupant == Some(v)
    {
        b.structure.bunker_occupant = None;
    }
    if let Some(o) = w.obj_mut(v) {
        if o.structure.bunkered_in == Some(k) {
            o.structure.bunkered_in = None;
        }
        if o.contact == Some(k) {
            o.contact = None;
        }
    }
}

/// The bunker returns to *clearing* and queues *guard* without an immediate
/// start (S-0606 claims 12 and 14).
fn back_to_clearing(w: &mut World, k: ObjectId) {
    set_stage(w, k, Stage::Clearing);
    if let Some(b) = w.obj_mut(k) {
        b.beh.queue(Some(Behaviour::Guard));
    }
}

/// Puts `v` on the release track toward bunker `k`'s release point (S-0606
/// claims 12 and 13), its locomotion switched back on.
fn release_track(w: &mut World, k: ObjectId, v: ObjectId) {
    let c = w.reference_point(k);
    // spec: S-0606 F4 (the reference point plus (−128, +128, 0), at the reference point's height)
    let target = Pos::new(c.x + RELEASE.0, c.y + RELEASE.1, c.z);
    // spec: S-0606 claims 12 and 13 (after switching its locomotion back on if it was off; no pulse check); S-0314 claim 22 (leaving a tank bunker switches a vehicle on but is no end of its being settled)
    crate::hover::switched(w, v, false);
    // spec: S-0606 claims 12 and 13 (the release track, table T2, ρₜ = ρ = 1.0)
    crate::drive::begin_bunker_track(w, v, BunkerTrack::Release, target);
}

/// A bunkered vehicle's *unload* (S-0606 claim 12). Returns the delay.
pub fn process_unload(w: &mut World, v: ObjectId) -> i32 {
    let Some(k) = bunkered_in(w, v) else {
        return crate::process::standard_delay(w, v);
    };
    // spec: S-0606 claim 12 (the walls-down effects)
    walls_down(w, k);
    // spec: S-0606 claim 12 (the vehicle's link ends, and it is put on the release track)
    if let Some(o) = w.obj_mut(v) {
        o.structure.bunkered_in = None;
    }
    release_track(w, k, v);
    // spec: S-0606 claim 12 (the nearby-cell search centred one cell along −x and +y from the top-left foundation cell: the vehicle's class, zone normal, no reach region, never deck mode, bridge cells allowed, no other condition, as S-0212 T5's bunker exit row)
    let top = w.obj(k).map(|b| b.pos.cell());
    let class = w
        .obj(v)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        .and_then(|u| u.class);
    let cell = top.and_then(|c| {
        crate::regions::ensure(w);
        crate::path::nearby(
            w,
            CellXY::new(c.x + EXIT_CENTRE.0, c.y + EXIT_CENTRE.1),
            crate::path::NearbyUse::Settings(crate::path::NearbySettings {
                class,
                zone: MovementZone::Normal,
                reach: None,
                deck: false,
                height: false,
                reference: None,
                no_bridges: false,
                no_overlay: false,
            }),
        )
    });
    // spec: S-0606 claim 12 (the result as destination; no cell found: the placeholder map cell (0, 0), S-0009 claim 44's stand-in, not a search result)
    let dest = cell.unwrap_or(NO_CELL);
    crate::econ::assign_destination(w, v, Some(Target::Cell(dest)));
    if let Some(o) = w.obj_mut(v) {
        o.path.steps.clear();
        // spec: S-0212 claim 21 and its state table, row "destination changes" (r97: for the driving and walk locomotors a different destination, or none, ends a running retry delay, path::destination_changed and path::waits; the ship and the other locomotors keep it, low confidence; the hovering locomotor's is S-0503's)
        // spec: S-0606 claim 12 (queues move, without an immediate start; it starts under S-0215 claim 11 when the vehicle is ready, whatever the delay)
        o.beh.queue(Some(Behaviour::Move));
    }
    // spec: S-0606 claim 12 (the bunker's link ends, it returns to clearing, queues guard without an immediate start, and ends contact with every vehicle in contact with it)
    unlink(w, k, v);
    for x in contacts(w, k) {
        end_contact(w, x, k);
    }
    back_to_clearing(w, k);
    continue_unload(w, v)
}

/// The rest of a released vehicle's *unload* processing (S-0606 claim 12,
/// S-0415 claim 1): an ordinary type does nothing more and chooses 450
/// ticks, no draw; a type with the harvester or weeder flag, a deploy-into
/// building type or the simple-deployer flag goes on with that property's
/// *unload* rule, in that order of precedence. Neither the passenger
/// unloading of S-0507 nor the deploy-fire rule of S-0517 claim 2 is
/// reached (revision 7, answering Q-1164).
fn continue_unload(w: &mut World, v: ObjectId) -> i32 {
    let Some(o) = w.obj(v) else {
        return UNLOAD_DELAY;
    };
    let Some(u) = w.rules().unit(o.kind, o.type_index).cloned() else {
        return UNLOAD_DELAY;
    };
    // spec: S-0606 claim 12 (revision 7, Q-1164 answered: passenger unloading and the deploy-fire rule are skipped, even for a type with passenger capacity or the deploy-fire flag) and S-0415 claim 1
    // spec: S-0606 claim 12 and S-0415 claim 1 (the harvester or weeder flag: S-0402's unload rule)
    if u.harvester || u.raw.weeder.unwrap_or(false) {
        return crate::econ::harvest::process_unload(w, v);
    }
    // spec: S-0606 claim 12 and S-0415 claim 1 (revision 18: then a deploy-into building type makes its deploy attempt whatever its passenger capacity or deploy-fire flag; the standard delay of S-0215 claim 6)
    if let Some(b) = crate::econ::deploy::deploys_into(w, o.kind, o.type_index) {
        let _ = crate::econ::deploy::attempt(w, v, b);
        return crate::process::standard_delay(w, v);
    }
    // spec: S-0606 claim 12 (revision 7) and S-0517 claim 1 (revision 10: then the simple-deployer flag: the in-place cycle in this same processing, whatever the passenger capacity; under quirks.deploy_transition_discards_orders its return to guard replaces the queued move, S-0517 claims 3 and 8)
    if u.simple_deployer {
        return crate::econ::inplace::process(w, v);
    }
    // spec: S-0606 claim 12 (revision 7: none of the three properties, a deploy-fire type included: nothing more, exactly 450 ticks, no draw) and S-0415 claim 1
    UNLOAD_DELAY
}

/// An occupied bunker lets its vehicle out (S-0606 claim 13: destroyed,
/// sold or erased): the release track, both links and the contact end; no
/// walls-down effects, no new behaviour or destination.
pub fn let_out(w: &mut World, k: ObjectId) {
    let Some(v) = occupant(w, k) else { return };
    if w.obj(v).is_none() {
        if let Some(b) = w.obj_mut(k) {
            b.structure.bunker_occupant = None;
        }
        return;
    }
    // spec: S-0606 claim 13 (the release track toward F4, ρₜ = ρ = 1.0, locomotion switched back on)
    release_track(w, k, v);
    // spec: S-0606 claim 13 (the bunker link ends on both sides, and the bunker ends contact with the vehicle)
    unlink(w, k, v);
}

/// A bunkered vehicle is released in place (S-0606 claim 14: it is
/// destroyed, completes erasure or joins a warp group): the walls-down
/// effects, both links and the contact end, the bunker returns to
/// *clearing* with *guard* queued. The vehicle is not moved.
pub fn release_in_place(w: &mut World, v: ObjectId) {
    let Some(k) = bunkered_in(w, v) else { return };
    if w.obj(k).is_none() {
        if let Some(o) = w.obj_mut(v) {
            o.structure.bunkered_in = None;
        }
        return;
    }
    // spec: S-0606 claim 14 (the walls-down effects of claim 12)
    walls_down(w, k);
    // spec: S-0606 claim 14 (the bunker link ends on both sides, and the bunker ends contact)
    unlink(w, k, v);
    // spec: S-0606 claim 14 (the bunker returns to clearing and queues guard, without an immediate start)
    back_to_clearing(w, k);
}

/// An object leaving play by another route (S-0606 claim 19): our engine
/// ends the bunker link on both sides silently (a documented departure from
/// the original's stale link, no switch): no walls-down effects, no release
/// track; a surviving vehicle gets no new behaviour or destination, and a
/// surviving bunker ends contact with the removed vehicle, returns to
/// *clearing* and queues *guard* without an immediate start.
pub fn removing(w: &mut World, id: ObjectId) {
    if let Some(k) = bunkered_in(w, id) {
        // spec: S-0606 claim 19 (our engine: the link ends on both sides; the surviving bunker ends contact with the removed vehicle, returns to clearing and queues guard without an immediate start)
        unlink(w, k, id);
        back_to_clearing(w, k);
    }
    if let Some(v) = occupant(w, id) {
        // spec: S-0606 claim 19 (our engine: a surviving vehicle loses the link and gets no new behaviour or destination)
        unlink(w, id, v);
    }
}

/// Whether a bunkered vehicle refuses a destination (S-0606 claim 18:
/// every non-empty destination, whatever its source, while the link
/// holds; clearing it is accepted).
pub fn refuses_destination(w: &World, id: ObjectId) -> bool {
    // spec: S-0606 claim 18 (while the bunker link holds, every non-empty destination is refused; the refusal ends with the link)
    bunkered(w, id)
}

/// Enforces [`refuses_destination`] on a destination written without the
/// assignment: called at the start of the vehicle's behaviour turn.
pub fn enforce(w: &mut World, id: ObjectId) {
    if refuses_destination(w, id)
        && let Some(o) = w.obj_mut(id)
        && o.dest.is_some()
    {
        o.dest = None;
        o.path.steps.clear();
    }
}

// ---------------------------------------------------------------------------
// Tracks (tables T1 and T2)

/// The points of a bunker track, displacements from its target, facings in
/// 1/256 turns. Every listed point is driven, the last, (0, 0), included,
/// so the track ends on its target with that point's facing (S-0606
/// claim 21).
pub fn track_points(t: BunkerTrack) -> Vec<Point> {
    match t {
        BunkerTrack::Entry(row) => (1..=16) // spec: S-0606 table T1 (n = 1 … 16)
            .map(|n| {
                // spec: S-0606 table T1 (B: point n = (−128 + 8n, 128 − 8n), facing 32)
                let (x, y, f) = (-128 + 8 * n, 128 - 8 * n, 32i32);
                match row {
                    // spec: S-0606 table T1 row E1 (no transform)
                    1 => (x, y, f as u8),
                    // spec: S-0606 table T1 row E2 ((x, −y), φ → (−φ − 128) mod 256)
                    2 => (x, -y, (-f - 128).rem_euclid(256) as u8),
                    // spec: S-0606 table T1 row E4 ((−x, y), φ → (−φ) mod 256)
                    4 => (-x, y, (-f).rem_euclid(256) as u8),
                    // spec: S-0606 table T1 row E3 ((y, x), φ → (−φ − 64) mod 256)
                    _ => (y, x, (-f - 64).rem_euclid(256) as u8),
                }
            })
            .collect(),
        BunkerTrack::Release => {
            // spec: S-0606 table T2 (x of points 1 to 17)
            const X: [i32; 17] = [
                128, 124, 119, 115, 111, 106, 101, 96, 91, 84, 77, 70, 64, 48, 32, 16, 0,
            ];
            // spec: S-0606 table T2 (y of points 1 to 17)
            const Y: [i32; 17] = [
                -128, -112, -96, -80, -64, -57, -50, -44, -37, -32, -27, -22, -17, -12, -8, -4, 0,
            ];
            // spec: S-0606 table T2 (points 1 to 17; facing 128 + 4 (n − 1), from 128 to 192)
            (0..17).map(|i| (X[i], Y[i], (128 + 4 * i) as u8)).collect()
        }
    }
}

// ---------------------------------------------------------------------------
// Ownership (claim 15)

/// Whether `v` is the bunkered vehicle of building `k`: S-0509's contact
/// treatment of an owner change leaves it (S-0606 claim 15: the link holds).
///
/// The psychic domination's sweep (S-0612 claim 13) reaches a bunkered
/// vehicle of a two by two bunker as its cell's start object and changes
/// its owner through `set_owner`, so the same contact treatment applies and
/// the link stays (S-0612 claim 13: "The bunker link stays").
pub fn holds_through_owner_change(w: &World, k: ObjectId, v: ObjectId) -> bool {
    // spec: S-0606 claim 15 (either side changes owner: the link holds)
    occupant(w, k) == Some(v) || bunkered_in(w, v) == Some(k)
}
