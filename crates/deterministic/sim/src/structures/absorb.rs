//! Absorbing buildings (S-0605): who may enter (claims 1 and 2, F1), contact
//! and waiting in line (claims 3 to 5, F2), going inside (claims 6 to 8),
//! holding (claims 9 and 10), the voluntary release (claims 11 to 13, F3),
//! the other ways out (claims 14 and 15: S-0312 claim 24, S-0408 claim 8,
//! S-0514 claim 12), the bio-reactor fate (claim 16) and what ends an
//! entrant's contact (claim 17).
//!
//! State, all in [`super::StructState`]: a building's passengers in the
//! passenger order (front first), a passenger's building, the building an
//! actor is waiting in line to enter, and the enter-nearest instruction of
//! claim 16. An entrant's contact is the ordinary contact field; a
//! building's contacts are the objects whose contact it is. An enter order
//! on a building does not make an actor remember an entry object (claims 4
//! and 5), so only the contact completes an entry.
//!
//! The waiting actor's approach (claim 4) is S-0212 claim 25's
//! nearby-cell search with table T5's row for the waiting absorber
//! approach (Q-1152, answered by S-0605 revision 5).

use glasnost_map::CellXY;
use glasnost_num::F64;
use glasnost_num::geom::Facing;

use crate::combat::damage::Hit;
use crate::combat::effects::{Effect, Sound};
use crate::econ::TypeRef;
use crate::regions::STEPS;
use crate::rules::{AbsorbRules, Behaviour, Kind};
use crate::scatter::{self, ScatterRequest};
use crate::transport::Answer;
use crate::world::{Layer, ObjectId, Pos, Target, World};

/// The step directions of the exit search.
// spec: S-0605 Constants (exit rounds 8 then 7 directions, as S-0507)
const DIRECTIONS: usize = 8;
/// Facing units of the 65536-step facing per step direction: 32 of 256
/// steps.
// spec: S-0605 Constants and F3 (exit facing 32 × e in 256-step facing units, so 8192 × e of 65536)
const DIRECTION_FACING: i32 = 32 * 256;
/// The delay an empty *unload* processing chooses.
// spec: S-0605 claim 11 (an empty processing chooses 1 tick, without a standard-delay draw)
const EMPTY_UNLOAD_DELAY: i32 = 1;

// ---------------------------------------------------------------------------
// Queries

fn absorb_rules(w: &World, b: ObjectId) -> Option<AbsorbRules> {
    let o = w.obj(b)?;
    if o.kind != Kind::Building {
        return None;
    }
    w.rules()
        .buildings
        .get(o.type_index as usize)
        .map(|r| r.absorb)
        .filter(AbsorbRules::absorbing)
}

/// Whether an object is an absorbing building (Terms).
pub fn is_absorber(w: &World, b: ObjectId) -> bool {
    absorb_rules(w, b).is_some()
}

/// Whether an object is an infantry absorber (Terms).
pub fn infantry_absorber(w: &World, b: ObjectId) -> bool {
    absorb_rules(w, b).is_some_and(|r| r.infantry)
}

/// Whether an object is a vehicle absorber (Terms).
pub fn vehicle_absorber(w: &World, b: ObjectId) -> bool {
    absorb_rules(w, b).is_some_and(|r| r.vehicle)
}

/// Whether a building's type carries passengers for S-0214's transport
/// verdicts *x* and *q*: an absorbing type whose capacity is above 0
/// (claim 1).
pub fn carries_passengers(w: &World, b: ObjectId) -> bool {
    // spec: S-0605 claim 1 and Import mapping (rows x and q read the imported passenger count as above 0)
    absorb_rules(w, b).is_some_and(|r| r.capacity > 0)
}

/// For an absorbing building, whether it carries no passengers for S-0214
/// row I13: its capacity equals 0, so a negative capacity does not (claim
/// 1). `None` for any other object.
pub fn carries_no_passengers(w: &World, b: ObjectId) -> Option<bool> {
    // spec: S-0605 claim 1 (row I13 tests "carries no passengers" as equality with 0)
    absorb_rules(w, b).map(|r| r.capacity == 0)
}

/// Whether an actor holds the enter-nearest instruction for absorbing
/// buildings (claim 16, S-0215 Terms).
pub fn holds_instruction(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.structure.enter_nearest)
}

/// A building's passengers in the passenger order, the front first (Terms).
pub fn passengers(w: &World, b: ObjectId) -> Vec<ObjectId> {
    w.obj(b)
        .map(|o| o.structure.absorbed.clone())
        .unwrap_or_default()
}

/// The passenger count *n* (Terms, S-0405's *n*ₐ).
pub fn count(w: &World, b: ObjectId) -> i32 {
    w.obj(b).map_or(0, |o| o.structure.absorbed.len() as i32)
}

/// The absorbing building a passenger is inside (claim 9).
pub fn absorbed_in(w: &World, id: ObjectId) -> Option<ObjectId> {
    w.obj(id).and_then(|o| o.structure.absorbed_in)
}

/// The building an actor is waiting in line to enter (claim 4).
pub fn waiting_for(w: &World, id: ObjectId) -> Option<ObjectId> {
    w.obj(id).and_then(|o| o.structure.waiting)
}

fn size(w: &World, u: ObjectId) -> F64 {
    let mode = w.rules().mode;
    w.obj(u)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        // spec: S-0605 Import mapping (Size absent: 1.0, as S-0507)
        .map_or_else(|| F64::from_i64(1, mode), |t| t.size)
}

/// F1: accept ⇔ n + 1 ≤ c and s ≤ L.
fn fits(w: &World, b: ObjectId, u: ObjectId) -> bool {
    let Some(r) = absorb_rules(w, b) else {
        return false;
    };
    // spec: S-0605 F1 and claim 2 (passengers counted, not sizes: exact integers, so n + 1 ≤ c is n < c; s ≤ L compared directly as doubles)
    i64::from(count(w, b)) < i64::from(r.capacity) && size(w, u).le(r.size_limit)
}

/// The acceptance test (claim 1): the building's docking answer to would-be
/// entrant `u`. `Neither` for a building that does not absorb.
pub fn answer(w: &World, b: ObjectId, u: ObjectId) -> Answer {
    let (Some(bo), Some(uo)) = (w.obj(b), w.obj(u)) else {
        return Answer::Neither;
    };
    let Some(r) = absorb_rules(w, b) else {
        return Answer::Neither;
    };
    // spec: S-0605 claim 1 (the building's owner does not count U's house as allied: one-way, the building owner's view)
    if !w.allied(bo.owner, uo.owner) {
        return Answer::Refuses;
    }
    // spec: S-0605 claim 1 (under construction, being sold or not finished building up)
    if matches!(
        crate::econ::placement::seen_behaviour(bo),
        Some(Behaviour::Construction | Behaviour::Selling)
    ) || bo.econ.sale.is_some()
    {
        return Answer::Refuses;
    }
    // spec: S-0605 claim 1 and S-0504 claim 7 (exactly one naval, unless U's movement zone is amphibious)
    if crate::naval::naval_match_refuses(w, b, u) {
        return Answer::Refuses;
    }
    let balloon = w
        .rules()
        .unit(uo.kind, uo.type_index)
        .is_some_and(|t| t.raw.balloon_hover.unwrap_or(false));
    // spec: S-0605 claim 1 (a balloon-hover type)
    if balloon {
        return Answer::Refuses;
    }
    // spec: S-0605 claim 1 (switched off refuses; low power does not)
    if bo.econ.building.switched_off {
        return Answer::Refuses;
    }
    // spec: S-0605 claim 1 (kind: a vehicle at a building that is not a vehicle absorber, infantry at one that is not an infantry absorber; an aircraft is not refused by its kind)
    let wrong_kind = match uo.kind {
        Kind::Vehicle => !r.vehicle,
        Kind::Infantry => !r.infantry,
        Kind::Aircraft => false,
        _ => true,
    };
    if wrong_kind {
        return Answer::Refuses;
    }
    // spec: S-0605 claim 1 (a controller holding at least one victim; a controlled U is not refused)
    if crate::mind::controls_objects(w, u) {
        return Answer::Refuses;
    }
    // spec: S-0605 claim 1 and S-0603 claim 2 (otherwise it accepts when F1 holds or its recycling feature applies)
    if fits(w, b, u) || crate::structures::grinder::is_recycler(w, b) {
        Answer::Accepts
    } else {
        Answer::Refuses
    }
}

/// The link places of F2: max(d, 1).
pub fn link_places(w: &World, b: ObjectId) -> u32 {
    // spec: S-0605 F2 and Import mapping (NumberOfDocks absent 1; places = max(d, 1), minimum 1)
    crate::econ::release::contact_places(w, b)
}

/// The actors in contact with building `b`, in object order.
pub fn contacts(w: &World, b: ObjectId) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|o| o.contact == Some(b) && o.id != b)
        .map(|o| o.id)
        .collect()
}

/// Claim 3's contact grant for actor `u` at building `b`: both alliance
/// views, health above 0 and a free link place.
fn grants_contact(w: &World, b: ObjectId, u: ObjectId) -> bool {
    let (Some(bo), Some(uo)) = (w.obj(b), w.obj(u)) else {
        return false;
    };
    // spec: S-0605 claim 3 (the building counts the actor's house as allied and the actor's house the building's; health above 0; a link place free)
    w.allied(bo.owner, uo.owner)
        && w.allied(uo.owner, bo.owner)
        && bo.health > 0
        && bo.on_map
        && (contacts(w, b).len() as u32) < link_places(w, b)
}

/// The loading request of claims 3 and 5: an ordinary absorbing building
/// refuses it only while switched off.
fn loading_refused(w: &World, b: ObjectId) -> bool {
    // spec: S-0605 claims 3 and 5 (switching it off refuses loading)
    w.obj(b).is_none_or(|o| o.econ.building.switched_off)
}

fn give_up_contact(w: &mut World, u: ObjectId) {
    if let Some(ob) = w.obj_mut(u) {
        ob.contact = None;
    }
}

fn set_waiting(w: &mut World, u: ObjectId, b: Option<ObjectId>) {
    if let Some(ob) = w.obj_mut(u) {
        ob.structure.waiting = b;
        if b.is_some() {
            // spec: S-0605 claim 4 and Terms, remembered entry object (setting up the wait clears any object the actor remembered entering; in our engine only S-0314 claim 23's depot is one)
            ob.depot.entering = None;
        }
    }
}

/// Claim 3's start of a wait: destination cleared, the building named in
/// the waiting line, not told to walk yet.
fn start_waiting(w: &mut World, u: ObjectId, b: ObjectId) {
    crate::econ::assign_destination(w, u, None);
    if let Some(ob) = w.obj_mut(u) {
        ob.path.steps.clear();
    }
    set_waiting(w, u, Some(b));
}

// ---------------------------------------------------------------------------
// Getting contact (claims 3 to 5)

/// Claim 3 for actor `u` that has just been given absorbing building `b`
/// as its destination with *enter* current or queued: contact, or waiting
/// in line.
fn seek_contact(w: &mut World, u: ObjectId, b: ObjectId) {
    if w.obj(u).is_none_or(|o| o.contact.is_some()) {
        return;
    }
    if w.obj(u).is_some_and(|o| o.kind == Kind::Vehicle) && vehicle_absorber(w, b) {
        // spec: S-0605 claim 3 (a vehicle ordered in to a vehicle absorber, or assigned one by claim 16's fate, always starts waiting in line for it, whether or not the building holds contact: no destination and no contact, no remembered entry object, and no contact request; claim 4's approach and retry follow)
        start_waiting(w, u, b);
        return;
    }
    if !contacts(w, b).is_empty() {
        // spec: S-0605 claim 3 (the building already holds any contact, whatever the number of link places: waiting in line, destination cleared, not told to walk yet)
        start_waiting(w, u, b);
        return;
    }
    if !grants_contact(w, b, u) {
        // spec: S-0605 claim 3 (not granted: no waiting in line; the destination assignment goes on, so the actor holds the building as its destination without contact)
        return;
    }
    if let Some(ob) = w.obj_mut(u) {
        ob.contact = Some(b);
    }
    // spec: S-0605 claim 3 (granted: it asks to load; switched off refuses, it gives up the contact and keeps the building as its destination)
    if loading_refused(w, b) {
        give_up_contact(w, u);
    }
}

/// After an enter order's ordinary effects (claim 3; S-0213 table T3:
/// part (d) has queued *enter* when part (f) assigns the destination): an
/// enter order naming an absorbing building as destination seeks contact.
pub fn order_entered(w: &mut World, u: ObjectId, b: Behaviour, dest: Option<Target>) {
    if b != Behaviour::Enter {
        return;
    }
    let Some(Target::Object(k)) = dest else {
        return;
    };
    if !is_absorber(w, k) || w.obj(u).is_none_or(|o| !o.kind.is_mobile()) {
        return;
    }
    seek_contact(w, u, k);
}

/// Every behaviour order forgets the building the actor was waiting in
/// line to enter (S-0213 table T3 (b), claim 4).
pub fn order_applied(w: &mut World, u: ObjectId) {
    // spec: S-0605 claim 4 and S-0213 table T3 (b) (any new order ends the wait)
    set_waiting(w, u, None);
}

/// The retry of an actor waiting in line (claim 4), in its own turn after
/// its behaviour processing and its movement, where S-0314 claim 23's
/// retry runs.
///
/// First, with `quirks.absorber_stale_contact_kept` off, an actor in
/// contact with an absorbing building that has neither *enter* current nor
/// *enter* queued gives the contact up (claim 17, Edge cases).
pub fn waiting_turn(w: &mut World, u: ObjectId) {
    if !w.rules().quirks.absorber_stale_contact_kept {
        contact_outlived(w, u);
    }
    let Some(b) = waiting_for(w, u) else { return };
    // spec: S-0605 claim 3 and scenario A13 (g) (a vehicle victim put in line by claim 16's fate sends no contact request in the tick of the release; its first request is a later update's retry)
    if w.obj(u)
        .is_some_and(|o| o.structure.fate_tick == Some(w.tick))
    {
        return;
    }
    // spec: S-0605 claim 4 (a building that has left the game ends the wait)
    if w.obj(b).is_none_or(|o| !o.on_map) {
        set_waiting(w, u, None);
        return;
    }
    // spec: S-0605 claim 4 and S-0314 claim 23 (nothing is retried while the actor is still phasing in)
    if crate::teleport::phasing_in(w, u) {
        return;
    }
    // spec: S-0605 claim 4 (not granted, no free link place or any other reason: it keeps waiting)
    if !grants_contact(w, b, u) {
        return;
    }
    if let Some(ob) = w.obj_mut(u) {
        ob.contact = Some(b);
    }
    set_waiting(w, u, None);
    if answer(w, b, u) == Answer::Accepts {
        let (t, stale) = (w.tick, w.rules().quirks.stale_suspended_revert); // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
        let ready = crate::process::ready(w, u);
        // spec: S-0605 claim 4 and S-0215 claim 7 (accept: enter queued with an immediate start, current at once when ready; the building as destination; waiting ends)
        if let Some(ob) = w.obj_mut(u) {
            ob.beh
                .queue_and_start(Some(Behaviour::Enter), ready, t, stale);
            ob.path.steps.clear();
            // spec: S-0212 claim 21 and its state table, row "destination changes" (r97: for the driving and walk locomotors a different destination, or none, ends a running retry delay, path::destination_changed and path::waits; the ship and the other locomotors keep it, low confidence; the hovering locomotor's is S-0503's)
        }
        crate::econ::assign_destination(w, u, Some(Target::Object(b)));
    } else {
        // spec: S-0605 claim 4 (refuse: it gives up the contact, queues the none behaviour without a start, which leaves its behaviours as they are (S-0215 claim 7), clears its destination and stops waiting)
        give_up_contact(w, u);
        if let Some(ob) = w.obj_mut(u) {
            ob.beh.queue(None);
        }
        crate::econ::assign_destination(w, u, None);
    }
}

/// `quirks.absorber_stale_contact_kept` off (claim 17, Edge cases): the
/// contact of an actor that has neither *enter* current nor *enter* queued
/// ends. A waiting actor granted contact with *enter* queued but not yet
/// started keeps it.
fn contact_outlived(w: &mut World, u: ObjectId) {
    let Some(o) = w.obj(u) else { return };
    let Some(b) = o.contact else { return };
    let entering =
        o.beh.current == Some(Behaviour::Enter) || o.beh.queued == Some(Behaviour::Enter);
    // spec: S-0605 claim 17 and Edge cases (quirks.absorber_stale_contact_kept off: an actor in contact with an absorbing building with neither enter current nor queued gives the contact up)
    if !entering && is_absorber(w, b) {
        give_up_contact(w, u);
    }
}

/// The absorbing building an actor's contact is, if any. An enter order on
/// an absorbing building does not make an actor remember an entry object
/// (claims 4 and 5), so the contact is the only partner of S-0215 claim
/// 29's request.
fn partner(w: &World, u: ObjectId) -> Option<ObjectId> {
    let o = w.obj(u)?;
    // spec: S-0605 claims 4 and 5 and S-0215 claim 29 (the request goes to the contact; an enter order on an absorbing building leaves no remembered entry object)
    o.contact.filter(|&k| is_absorber(w, k))
}

/// Whether an actor in *enter* is entering an absorbing building: in
/// contact with one, waiting in line for one, or holding one as its
/// destination.
pub fn entering_absorber(w: &World, u: ObjectId) -> bool {
    let Some(o) = w.obj(u) else { return false };
    partner(w, u).is_some()
        || o.structure.waiting.is_some()
        || matches!(o.dest, Some(Target::Object(k)) if is_absorber(w, k))
}

/// The *enter* processing toward an absorbing building (claims 4 and 5,
/// S-0215 claim 29). Returns the delay.
pub fn process_enter(w: &mut World, u: ObjectId) -> i32 {
    if let Some(b) = partner(w, u) {
        if loading_refused(w, b) {
            // spec: S-0605 claim 5 (switched off since then: the loading request is refused; it gives up the contact and falls back to rest in the same processing, its destination and walk kept)
            give_up_contact(w, u);
            crate::process::fall_back(w, u, false);
        }
        // spec: S-0215 claims 6 and 29 (the processing ends with the standard delay)
        return crate::process::standard_delay(w, u);
    }
    if let Some(b) = waiting_for(w, u)
        && w.obj(b).is_some_and(|o| o.on_map)
    {
        // spec: S-0605 claim 4 (approach: the first processing of enter after the wait began, with no contact and no remembered entry object, the building still in the game)
        if !approach(w, u, b) {
            // spec: S-0605 claim 4 (no cell found: it falls back to rest in that same processing and stays where it is; nothing is kept, its destination was cleared when the wait began; the wait and its retry go on)
            crate::process::fall_back(w, u, false);
        }
        // spec: S-0215 claims 6 and 29 (the processing ends with the standard delay)
        return crate::process::standard_delay(w, u);
    }
    // spec: S-0605 claim 5 (no contact, no remembered entry object, not waiting in line: it falls back to rest at once, keeping the building as its destination; it never goes in)
    crate::process::fall_back(w, u, false);
    crate::process::standard_delay(w, u)
}

/// Claim 4's approach of an actor waiting in line for building `b`: the
/// nearby-cell search of S-0212 claim 25 with table T5's row for the
/// waiting absorber approach, from the cell of the building's reference
/// point; a found cell becomes its destination, with *move* queued and
/// started at once. The waiting goes on. Returns whether a cell was found.
pub(crate) fn approach(w: &mut World, u: ObjectId, b: ObjectId) -> bool {
    // spec: S-0605 claim 4 and S-0204 claim 4 (the centre: the cell of the building's reference point, the centre of its foundation, not its top-left cell and not S-0507 claim 10's exit ring)
    let centre = w.reference_point(b).cell();
    crate::regions::ensure(w);
    // spec: S-0605 claim 4 and S-0212 table T5 (the waiting absorber approach row: the actor's own class with winged counted as track, its type's own zone, the reach region of the centre cell for that zone read for its deck, deck mode when it is on a bridge deck, no height condition, no further test, no no-overlay condition, bridge cells allowed, no reference cell: the preferred cell at position N mod n with N the tick of this processing; no draw)
    let Some(settings) = crate::path::NearbySettings::own_class(w, u, centre) else {
        return false;
    };
    let found = crate::path::nearby(w, centre, crate::path::NearbyUse::Settings(settings));
    let Some(c) = found else { return false };
    let (t, stale) = (w.tick, w.rules().quirks.stale_suspended_revert); // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    // spec: S-0605 claim 4 (it assigns the location as its destination, queues move and starts it at once; the waiting itself does not end)
    crate::econ::assign_destination(w, u, Some(Target::Cell(c)));
    if let Some(ob) = w.obj_mut(u) {
        ob.beh.queue_and_start_at_once(Behaviour::Move, t, stale);
        ob.path.steps.clear();
        // spec: S-0212 claim 21 and its state table, row "destination changes" (r97: for the driving and walk locomotors a different destination, or none, ends a running retry delay, path::destination_changed and path::waits; the ship and the other locomotors keep it, low confidence; the hovering locomotor's is S-0503's)
    }
    true
}

// ---------------------------------------------------------------------------
// Going inside (claims 6 to 8)

/// The absorbing building an arriving infantry unit tries to enter (claim
/// 6): it is in *enter*, stands in a cell that holds the building, and the
/// building is its destination, its target, or the building in its
/// destination cell.
fn arrival_building(w: &World, u: ObjectId) -> Option<ObjectId> {
    let o = w.obj(u)?;
    if o.kind != Kind::Infantry || !o.on_map || o.beh.current != Some(Behaviour::Enter) {
        return None;
    }
    let cell = o.pos.cell();
    let here = |b: ObjectId| {
        infantry_absorber(w, b)
            && w.obj(b).is_some_and(|x| x.on_map)
            // spec: S-0605 claim 6 (a cell that holds the building: any of its foundation cells)
            && w.cells.list(cell, Layer::Ground).contains(&b)
    };
    let from = |t: Option<Target>| match t {
        Some(Target::Object(b)) => Some(b),
        Some(Target::Cell(c)) => w
            .cells
            .list(c, Layer::Ground)
            .iter()
            .copied()
            .find(|&b| is_absorber(w, b)),
        None => None,
    };
    // spec: S-0605 claim 6 (the destination, or the target, or the building in its destination cell; unlike S-0508 claim 2 the target counts)
    from(o.dest)
        .filter(|&b| here(b))
        .or_else(|| match o.target {
            Some(Target::Object(b)) if here(b) => Some(b),
            _ => None,
        })
}

/// An infantry unit's arrival at the end of a walking step (claim 6).
/// Returns whether it went inside.
pub fn arrival(w: &mut World, u: ObjectId) -> bool {
    let Some(b) = arrival_building(w, u) else {
        return false;
    };
    // spec: S-0605 claim 6 and S-0524 claim 12 (one entry raise on the building's tag, the entrant as causing object, while it is still outside; not conditional on contact)
    crate::mp_triggers::entry_raise(w, b, u);
    // The raise's actions may have removed either object.
    if !w.obj(u).is_some_and(|o| o.on_map) || !w.obj(b).is_some_and(|o| o.on_map) {
        return w.obj(u).is_none_or(|o| !o.on_map);
    }
    // spec: S-0605 claim 6 (the infantry absorber agrees unless it is being sold; nothing more happens at this arrival when it does not)
    let selling = w.obj(b).is_some_and(|o| {
        o.econ.sale.is_some()
            || crate::econ::placement::seen_behaviour(o) == Some(Behaviour::Selling)
    });
    if selling {
        return false;
    }
    // spec: S-0605 claim 6 (the entrant asks its contact to complete its entry: one that holds no contact gets no answer and does not go in, the raise made all the same)
    if w.obj(u).and_then(|o| o.contact) != Some(b) {
        return false;
    }
    if !w.rules().quirks.absorber_contact_no_capacity_recheck && answer(w, b, u) != Answer::Accepts
    {
        // spec: S-0605 Edge cases (quirks.absorber_contact_no_capacity_recheck off: claim 1 again at completion; a refused entrant is treated as a refused vehicle, claim 8)
        turned_away(w, u);
        return false;
    }
    // spec: S-0605 claim 6 and Edge cases (quirks.absorber_contact_no_capacity_recheck: no re-check of capacity, size or the switch)
    go_in(w, u, b, true);
    true
}

/// A vehicle's arrival at a cell (claim 8, S-0209 row VP5). Returns whether
/// it went inside.
pub fn vehicle_arrival(w: &mut World, u: ObjectId) -> bool {
    let Some(o) = w.obj(u) else { return false };
    if o.kind != Kind::Vehicle || !o.on_map || o.beh.current != Some(Behaviour::Enter) {
        return false;
    }
    let Some(Target::Object(b)) = o.dest else {
        return false;
    };
    let cell = o.pos.cell();
    // spec: S-0605 claim 8 (it reaches a cell of a vehicle absorber with enter and that building as its destination)
    if !vehicle_absorber(w, b)
        || !w.obj(b).is_some_and(|x| x.on_map)
        || !w.cells.list(cell, Layer::Ground).contains(&b)
    {
        return false;
    }
    // spec: S-0605 claim 8 (unlike infantry it asks the full acceptance test again)
    if answer(w, b, u) == Answer::Accepts {
        go_in(w, u, b, false);
        true
    } else {
        turned_away(w, u);
        false
    }
}

/// A refused vehicle (claim 8): destination cleared, *guard* queued, a
/// forced scatter request.
fn turned_away(w: &mut World, u: ObjectId) {
    // spec: S-0605 claim 8 (refuse: its destination is cleared and it queues guard)
    crate::econ::assign_destination(w, u, None);
    if let Some(ob) = w.obj_mut(u) {
        ob.beh.queue(Some(Behaviour::Guard));
    }
    // spec: S-0605 claim 8 and S-0208 claims 11 to 13 (a forced scatter request)
    scatter::request(
        w,
        u,
        ScatterRequest {
            threat: None,
            forced: true,
            anywhere: false,
        },
    );
}

fn extra_output_positive(w: &World, b: ObjectId) -> bool {
    w.obj(b)
        .and_then(|o| w.rules().econ.power.buildings.get(o.type_index as usize))
        .is_some_and(|p| p.extra_output > 0)
}

fn power_due(w: &mut World, b: ObjectId) {
    // spec: S-0605 claims 7 and 13 and S-0405 claim 7 (power due only when the type's extra output is above 0; the building's owner)
    if extra_output_positive(w, b)
        && let Some(h) = w.obj(b).and_then(|o| o.owner)
    {
        crate::econ::power::make_due(w, h);
    }
}

/// The effects of going in (claims 7 and 8). `infantry` is claim 7's
/// infantry entry; otherwise claim 8's vehicle entry.
fn go_in(w: &mut World, u: ObjectId, b: ObjectId, infantry: bool) {
    if !infantry && let Some(ob) = w.obj_mut(u) {
        // spec: S-0605 claim 8 (it forgets the object it remembers entering and leaves any bridge deck)
        ob.dest = None;
        ob.layer = Layer::Ground;
    }
    // spec: S-0605 claims 7 and 8 and S-0609 claim 11 (spin value, stage and elapsed-tick count become 0)
    crate::gattling::reset(w, u);
    // spec: S-0605 claim 7 and S-0601 claim 9 (a controlled entrant is released first: release sound, owner back, fate decision; the release sound precedes the entry sound)
    crate::mind::release_one(w, u);
    let pos = w.obj(u).map(|o| o.pos);
    if infantry {
        let of = w.obj(u).map(|o| TypeRef {
            kind: o.kind,
            index: o.type_index,
        });
        let house = w.obj(u).and_then(|o| o.owner);
        // spec: S-0605 claim 7 and Import mapping (the entry sound at the entrant's position; none when not set)
        w.effects.push(Effect::Sound {
            sound: Sound::EnterBioReactor,
            of,
            house,
            pos,
        });
    }
    // spec: S-0605 claims 7 and 9 (it leaves the map and play; its destination is cleared)
    crate::objects::take_inside(w, u);
    let keep = w.rules().quirks.enter_nearest_survives_absorption;
    if let Some(ob) = w.obj_mut(u) {
        // spec: S-0605 claim 7 (its contact ends, freeing the link place)
        ob.contact = None;
        ob.structure.absorbed_in = Some(b);
        ob.structure.waiting = None;
        ob.layer = Layer::Ground;
        if !keep {
            // spec: S-0605 Edge cases (quirks.enter_nearest_survives_absorption off: going in clears the enter-nearest instruction)
            ob.structure.enter_nearest = false;
        }
    }
    if let Some(bb) = w.obj_mut(b) {
        // spec: S-0605 claim 7 and Terms (passenger order: it becomes the front passenger; n goes up by 1)
        bb.structure.absorbed.insert(0, u);
    }
    power_due(w, b);
}

// ---------------------------------------------------------------------------
// The voluntary release (claims 11 to 13)

/// The *unload* processing of an absorbing building (claim 11). Returns
/// the delay.
pub fn process_unload(w: &mut World, b: ObjectId) -> i32 {
    if count(w, b) == 0 {
        if crate::structures::garrison::garrisonable(w, b) {
            // A garrison feature's own unload (S-0508 claim 13).
            return crate::structures::garrison::process_unload(w, b);
        }
        if let Some(ob) = w.obj_mut(b) {
            // spec: S-0605 claim 11 (an empty processing of an ordinary absorber queues guard)
            ob.beh.queue(Some(Behaviour::Guard));
        }
        return EMPTY_UNLOAD_DELAY;
    }
    // spec: S-0605 claim 11 (garrison occupants are put out first, in the same processing)
    if crate::structures::garrison::count(w, b) > 0 {
        crate::structures::garrison::evict(w, b, crate::structures::garrison::Evict::PLAIN);
    }
    // spec: S-0605 claim 11 (the front passenger tries to come out)
    let_out(w, b);
    // spec: S-0605 claim 11 and S-0215 claim 6 (a processing that began with passengers ends with the standard delay, placed or not)
    crate::process::standard_delay(w, b)
}

/// The exit search of claim 12 for the front passenger. Returns whether it
/// came out.
fn let_out(w: &mut World, b: ObjectId) -> bool {
    let Some(bo) = w.obj(b).cloned() else {
        return false;
    };
    let Some(&p) = bo.structure.absorbed.first() else {
        return false;
    };
    // spec: S-0605 claim 12 and F3 (the neighbouring cells of the building's top-left foundation cell)
    let c = bo.pos.cell();
    let z = w.surface_at(256 * c.x + 128, 256 * c.y + 128); // spec: S-0204 claim 3 (a cell centre: (256x + 128, 256y + 128))
    if let Some(ob) = w.obj_mut(p) {
        // The passenger is tested from the top-left cell (S-0507 claim 10's entry test).
        ob.pos = Pos::centre(c, z);
        ob.layer = Layer::Ground;
    }
    let reference = w.cells.cell(c).map_or(0, |x| i32::from(x.level));
    // spec: S-0605 claim 12 and F3 (the rear direction q of S-0507 F3 from the building's facing)
    let q = crate::transport::rear_direction(bo.body.value(w.tick));
    let deck = |w: &World, n: CellXY| w.cells.is_bridge(n);
    let step = |c: CellXY, d: usize| CellXY::new(c.x + STEPS[d].0, c.y + STEPS[d].1);
    let q_ = &w.rules().quirks;
    let (deck_ends, skip_rear) = (
        q_.absorber_release_bridge_ends_search,
        q_.absorber_release_second_round_skips_rear,
    );
    let mut exit: Option<(usize, CellXY, CellXY)> = None;
    let mut ended = false;
    // spec: S-0605 claim 12 and S-0507 claim 10 (first round: all 8 directions from the rear, clockwise; the neighbour and the far cell)
    for k in 0..DIRECTIONS {
        let d = (q + k) % DIRECTIONS;
        let n = step(c, d);
        let far = step(n, d);
        let serves = || {
            crate::transport::enters(w, p, n, reference)
                && crate::transport::enters(w, p, far, reference)
        };
        if deck(w, n) {
            // spec: S-0605 claim 12 and Edge cases (quirks.absorber_release_bridge_ends_search: a deck neighbour at the eighth direction that passes both cell tests ends the search; a deck neighbour that fails a cell test there takes the ordinary advance)
            if k == DIRECTIONS - 1 && deck_ends && serves() {
                ended = true;
            }
            // spec: S-0605 claim 12 (a neighbouring cell with a bridge deck is passed over)
            continue;
        }
        if serves() {
            exit = Some((d, n, far));
            break;
        }
    }
    if exit.is_none() && !ended {
        // spec: S-0605 claim 12 and Edge cases (second round: the 7 directions after the rear, quirks.absorber_release_second_round_skips_rear; off: the rear first)
        let first = usize::from(skip_rear);
        for k in first..DIRECTIONS {
            let d = (q + k) % DIRECTIONS;
            let n = step(c, d);
            if deck(w, n) {
                continue;
            }
            if crate::transport::enters(w, p, n, reference) {
                exit = Some((d, n, n));
                break;
            }
        }
    }
    // spec: S-0605 claim 12 (placement at the first serving direction ends the search whether or not it succeeds)
    let Some((d, n, dest)) = exit else {
        return false;
    };
    // spec: S-0605 claim 12 (an infantry unit on a free standing position of that cell, S-0208 claim 3 with the cell centre as desired point; a vehicle by a nearby search from that cell)
    let Some(at) = crate::transport::exit_position(w, p, n) else {
        // spec: S-0605 claim 13 (on failure the passenger goes back to the front and stays inside)
        return false;
    };
    // spec: S-0605 F3 (exit facing 32 × e in 256-step units)
    let facing = (d as i32 * DIRECTION_FACING) as Facing;
    put_out(w, b, p, at, facing);
    if let Some(ob) = w.obj_mut(p) {
        // spec: S-0605 claim 13 (it queues move to the far cell, first round, or the neighbour cell, second round)
        ob.beh.queue(Some(Behaviour::Move));
    }
    crate::econ::assign_destination(w, p, Some(Target::Cell(dest)));
    if infantry_absorber(w, b) {
        let of = w.obj(b).map(|o| TypeRef {
            kind: o.kind,
            index: o.type_index,
        });
        // spec: S-0605 claim 13 and Import mapping (the exit sound at the building, at an infantry absorber only)
        w.effects.push(Effect::Sound {
            sound: Sound::LeaveBioReactor,
            of,
            house: bo.owner,
            pos: Some(bo.pos),
        });
    }
    // spec: S-0605 claim 13 (power due as in claim 7)
    power_due(w, b);
    let human = w
        .obj(p)
        .and_then(|o| o.owner)
        .and_then(|h| w.houses.get(usize::from(h.0)))
        .is_some_and(|x| x.human);
    if human && let Some(ob) = w.obj_mut(p) {
        // spec: S-0605 claim 13 (a passenger of a human-controlled house drops an enter-nearest instruction; one of the neutral or special house keeps it, quirks.enter_nearest_survives_absorption)
        ob.structure.enter_nearest = false;
    }
    true
}

/// Takes passenger `p` out of `b`'s passenger order and puts it on the map.
fn put_out(w: &mut World, b: ObjectId, p: ObjectId, at: Pos, facing: Facing) {
    leave_order(w, b, p);
    if let Some(ob) = w.obj_mut(p) {
        ob.layer = Layer::Ground;
    }
    crate::objects::place(w, p, at, facing);
}

/// Passenger `p` leaves `b`'s passenger order (n goes down by 1).
fn leave_order(w: &mut World, b: ObjectId, p: ObjectId) {
    if let Some(bb) = w.obj_mut(b) {
        // spec: S-0605 claims 13 and 15 (n goes down by 1)
        bb.structure.absorbed.retain(|&x| x != p);
    }
    if let Some(ob) = w.obj_mut(p) {
        ob.structure.absorbed_in = None;
    }
}

// ---------------------------------------------------------------------------
// Other ways out (claims 14 and 15)

/// The orders of a passenger let out by a destruction or a sale (S-0312
/// claim 25, S-0408 claim 8): none for a player house, *hunt* for the
/// neutral or special house.
fn released_orders(w: &mut World, p: ObjectId) {
    let human = w
        .obj(p)
        .and_then(|o| o.owner)
        .is_none_or(|h| w.houses.get(usize::from(h.0)).is_some_and(|x| x.human));
    if !human && let Some(ob) = w.obj_mut(p) {
        // spec: S-0312 claim 25 (an absorbed passenger that came out: hunt for the neutral or special house; a player house gives no order beyond its scatter request)
        ob.beh.queue(Some(Behaviour::Hunt));
    }
}

/// Puts passenger `p` of `b` into foundation cell `c` facing the
/// building's facing with a forced scatter request (S-0312 claim 24,
/// S-0408 claim 8). An infantry passenger takes the centre placement of
/// S-0208 claim 3, for a destruction as for a sale (claim 14). Returns
/// whether it was placed.
fn place_released(w: &mut World, b: ObjectId, p: ObjectId, c: CellXY) -> bool {
    let Some(bo) = w.obj(b).cloned() else {
        return false;
    };
    let Some(kind) = w.obj(p).map(|o| o.kind) else {
        return false;
    };
    let (x, y) = if kind == Kind::Infantry {
        // spec: S-0605 claim 14, S-0408 claims 7 and 8 and S-0208 claim 3 (destruction and sale alike: the aim point 36 leptons from the cell's centre along its second axis, the centre class; an open cell makes one R(0, 3) draw and takes the first free position of the chosen row; a cell that refuses the infantry unit makes no draw)
        let k = crate::walk::choose_position(
            w,
            c,
            Layer::Ground,
            crate::cells::position::CENTRE,
            false,
        );
        let Some(k) = k else {
            return false;
        };
        let (u, v) = crate::walk::standing_point(k);
        (256 * c.x + u, 256 * c.y + v) // spec: S-0208 table P (a standing position within the cell)
    } else {
        // spec: S-0312 claim 24 and S-0408 claim 8 (a vehicle at the cell centre), S-0204 claim 3
        (256 * c.x + 128, 256 * c.y + 128)
    };
    let z = w.surface_at(x, y);
    // spec: S-0312 claim 24 (it gets out facing the building's facing)
    put_out(w, b, p, Pos::new(x, y, z), bo.body.value(w.tick));
    // spec: S-0312 claims 12 and 24 and S-0408 claim 8 (a scatter request: no threat point, forced, not move-anywhere)
    scatter::request(
        w,
        p,
        ScatterRequest {
            threat: None,
            forced: true,
            anywhere: false,
        },
    );
    released_orders(w, p);
    true
}

/// A destroyed absorbing building's passengers (S-0312 claim 24, S-0605
/// claim 14), at claim 19 (g) before the survivors: front first, one per
/// foundation cell in S-0302's order; one not placed dies, credited to the
/// building's charge planter. With no crew all die. Returns the number of
/// foundation cells the passengers used, one each whether placed or not,
/// after which claim 18's survivors start.
pub fn destroyed(w: &mut World, b: ObjectId, hit: &Hit) -> usize {
    let list = passengers(w, b);
    if list.is_empty() {
        return 0;
    }
    let Some(bo) = w.obj(b).cloned() else {
        return 0;
    };
    let cells = crate::combat::damage::foundation_order(bo.pos.cell(), bo.foundation);
    let planter = bo.combat.charge_planter.filter(|&x| w.obj(x).is_some());
    let credit = Hit {
        amount: 0,
        warhead: None,
        distance: 0,
        // spec: S-0312 claim 24 (credited to the building's charge planter, if any, otherwise to no one)
        attacker: planter,
        house: planter.and_then(|x| w.obj(x)).and_then(|o| o.owner),
        ignore_defences: false,
        no_escape: false,
    };
    // spec: S-0312 claims 18 and 24 and F5 (revision 51: the passengers' release tests only the no-crew state, not the crewed flag; that state arises only from killing damage that ignores defences, no map field sets it)
    let no_crew = hit.ignore_defences || bo.combat.no_crew;
    for (i, &p) in list.iter().enumerate() {
        let placed = !no_crew
            // spec: S-0312 claim 24 and Edge cases (beyond the last cell: not placed)
            && cells
                .get(i)
                .is_some_and(|&c| place_released(w, b, p, c));
        if !placed {
            leave_order(w, b, p);
            // spec: S-0312 claim 24 (one that cannot be placed, or every one without a crew, dies)
            crate::combat::death::passenger_death(w, p, &credit);
        }
    }
    // spec: S-0312 claim 24 (each passenger uses the next cell whether or not the previous one could be placed; the survivors start after the last cell used)
    list.len()
}

/// A sold absorbing building's passengers (S-0408 claim 8, S-0605 claim
/// 14), before the garrison: as for a destruction, with the sale's
/// standing-position choice; one not placed is deleted.
pub fn sold(w: &mut World, b: ObjectId) {
    let list = passengers(w, b);
    if list.is_empty() {
        return;
    }
    let Some(bo) = w.obj(b).cloned() else { return };
    let cells = crate::combat::damage::foundation_order(bo.pos.cell(), bo.foundation);
    for (i, &p) in list.iter().enumerate() {
        // spec: S-0408 claim 8 (one per foundation cell in the cell order, front first, even if an earlier placement fails; beyond the last cell, S-0312 Edge cases)
        let placed = cells.get(i).is_some_and(|&c| place_released(w, b, p, c));
        if !placed {
            leave_order(w, b, p);
            // spec: S-0408 claim 8 (a passenger that cannot be placed is deleted)
            crate::objects::remove(w, p);
        }
    }
}

/// An erased absorbing building deletes its passengers (S-0605 claim 14,
/// S-0514 claim 12).
pub fn erased(w: &mut World, b: ObjectId) {
    for p in passengers(w, b) {
        leave_order(w, b, p);
        // spec: S-0605 claim 14 and S-0514 claim 12 (erasure deletes the passengers)
        crate::objects::remove(w, p);
    }
}

/// An object leaves the game (claim 15): a passenger removed inside leaves
/// its building's passenger order; a building removed with passengers
/// still inside deletes them; a waiting actor stops waiting.
pub fn removing(w: &mut World, id: ObjectId) {
    if let Some(b) = absorbed_in(w, id) {
        leave_order(w, b, id);
        if !w.rules().quirks.absorber_removal_keeps_stale_power {
            // spec: S-0605 claim 15 and Edge cases (quirks.absorber_removal_keeps_stale_power off: the removal makes the building owner's power due as a release does, when the extra output is above 0)
            power_due(w, b);
        }
        // spec: S-0605 claim 15 and Edge cases (quirks.absorber_removal_keeps_stale_power on: nothing marks the building owner's power due; its totals keep the removed passenger's contribution until another event)
    }
    set_waiting(w, id, None);
    if count(w, id) > 0 {
        erased(w, id);
    }
}

// ---------------------------------------------------------------------------
// The bio-reactor fate (claim 16, S-0601 claim 8, S-0215 claims 24 and 26)

/// Claim 16's search for unit `u`: its owner's absorbing buildings in the
/// order of the absorber list (the house's holding order, S-0402 Terms,
/// filtered to absorbing buildings), each asked claim 1's acceptance test
/// (a free link place does not count), the smallest ρ₃ from the unit's
/// position to the building's reference point winning, and on equal values
/// the later one in the list. Draws nothing.
fn nearest_accepting(w: &World, u: ObjectId) -> Option<ObjectId> {
    let uo = w.obj(u)?;
    let owner = uo.owner?;
    let list = &w.houses.get(usize::from(owner.0))?.econ.acquired;
    let mut best: Option<(i64, ObjectId)> = None;
    // spec: S-0605 claim 16 and Terms (candidates: the owner's absorbing buildings in absorber-list order, the order they were placed or came to the house)
    for &b in list {
        let Some(bo) = w.obj(b) else { continue };
        // spec: S-0605 claim 16 (each asked claim 1's acceptance test: alliance, build state, kind, switch, capacity, size; a link place is not asked)
        if bo.owner != Some(owner)
            || !bo.on_map
            || !is_absorber(w, b)
            || answer(w, b, u) != Answer::Accepts
        {
            continue;
        }
        // spec: S-0605 claim 16 and S-0215 F4 (ρ₃ from the unit's own position to the building's reference point, the foundation centre, S-0204 claim 4; quirks.table_root_distance)
        let d = crate::approach::rho3(w, uo.pos, w.reference_point(b));
        // spec: S-0605 claim 16 (the smallest value wins; on equal values the building later in the absorber list)
        if best.is_none_or(|(bd, _)| d <= bd) {
            best = Some((d, b));
        }
    }
    best.map(|(_, b)| b)
}

/// The search and its result (claim 16, S-0215 claim 24): found, the unit
/// takes the instruction and, unless already in contact with that building
/// or waiting in line for it, queues *enter* without a start and takes the
/// building as its destination, which applies claim 3. None found, the
/// instruction is cleared. Returns whether a building was found.
fn search_and_go(w: &mut World, u: ObjectId) -> bool {
    let Some(b) = nearest_accepting(w, u) else {
        if let Some(ob) = w.obj_mut(u) {
            // spec: S-0605 claim 16 (none found: the instruction is cleared)
            ob.structure.enter_nearest = false;
        }
        return false;
    };
    let Some(o) = w.obj_mut(u) else { return false };
    // spec: S-0605 claim 16 (found: the unit takes the enter-nearest instruction for absorbing buildings)
    o.structure.enter_nearest = true;
    // spec: S-0605 claim 16 and S-0215 claim 24 (unless already bound for it: in contact with it, or waiting in line for it)
    if o.contact == Some(b) || o.structure.waiting == Some(b) {
        return true;
    }
    // spec: S-0605 claim 16 (it queues enter without a start and assigns the building as its destination)
    o.beh.queue(Some(Behaviour::Enter));
    o.path.steps.clear();
    // spec: S-0212 claim 21 and its state table, row "destination changes" (r97: for the driving and walk locomotors a different destination, or none, ends a running retry delay, path::destination_changed and path::waits; the ship and the other locomotors keep it, low confidence; the hovering locomotor's is S-0503's)
    crate::econ::assign_destination(w, u, Some(Target::Object(b)));
    // spec: S-0605 claims 3 and 16 (the assignment applies claim 3 at once: contact, or waiting in line)
    seek_contact(w, u, b);
    true
}

/// The bio-reactor fate of S-0601 claim 8 (claim 16), once, at the moment
/// of the fate. Returns whether it succeeded; a failure becomes *hunt* in
/// the caller.
pub fn bio_reactor_fate(w: &mut World, u: ObjectId) -> bool {
    if w.obj(u).is_none_or(|o| !o.on_map || o.owner.is_none()) {
        return false;
    }
    let found = search_and_go(w, u);
    if w.obj(u).is_some_and(|o| o.kind == Kind::Vehicle) && waiting_for(w, u).is_some() {
        let t = w.tick;
        if let Some(ob) = w.obj_mut(u) {
            ob.structure.fate_tick = Some(t);
        }
    }
    found
}

/// The *guard* or *area guard* processing of an actor that holds the
/// enter-nearest instruction (claim 16, S-0215 claims 24 and 26 row 3): the
/// same search again; none found ends the instruction. The caller chooses
/// the delay: D_r of the behaviour being processed, no draw.
pub fn instruction_turn(w: &mut World, u: ObjectId) {
    // spec: S-0605 claim 16 and S-0215 claim 24 (at each guard or area guard processing the holder searches again and ends the instruction when no building is found)
    search_and_go(w, u);
}
