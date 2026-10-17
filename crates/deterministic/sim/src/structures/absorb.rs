//! Absorbing buildings (S-0605): who may enter (claims 1 and 2, F1), contact
//! and waiting in line (claims 3 to 5, F2), going inside (claims 6 to 8),
//! holding (claims 9 and 10), the voluntary release (claims 11 to 13, F3)
//! and the other ways out (claims 14 and 15: S-0312 claim 24, S-0408 claim
//! 8, S-0514 claim 12).
//!
//! State, all in [`super::StructState`]: a building's passengers in the
//! passenger order (front first), a passenger's building, and the building
//! an actor is waiting in line to enter. An entrant's contact is the
//! ordinary contact field; a building's contacts are the objects whose
//! contact it is.
//!
//! Interim rules of our requests:
//! - Q-1260: the bio-reactor fate of S-0601 claim 8 is a one-shot search
//!   ([`bio_reactor_fate`]); none found falls back to *hunt*.
//! - Q-1261: claim 3 runs when an enter order on an absorbing building is
//!   applied (after the order's ordinary effects), and a contact not granted
//!   for any reason other than a held contact also starts waiting in line;
//!   the retry of claim 4 runs in the actor's own turn after its movement,
//!   where the depot retry runs (S-0314 claim 23), and its *enter* is queued
//!   without an immediate start.
//! - Q-1262: at the eighth first-round direction the search ends only when
//!   both cell tests pass and the neighbour carries a deck (Edge cases).
//! - Q-1263: a passenger removed inside makes the building owner's power
//!   due when the type's extra output is above 0.
//! - Q-1264: a passenger let out by a destruction takes the first free
//!   standing position of its cell, without a draw (as S-0312 claim 13's
//!   crew placement).
//! - Q-1265: a building type carries passengers for S-0214 rows x and q
//!   when it is absorbing and its capacity is above 0.
//! - Q-1266: an entrant whose loading request is refused (claims 3 and 5)
//!   also forgets the building as its destination and stops walking, so
//!   that it stands (scenario A6).

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
/// verdicts *x* and *q*: an absorbing type with a capacity above 0 (Q-1265
/// interim).
pub fn carries_passengers(w: &World, b: ObjectId) -> bool {
    absorb_rules(w, b).is_some_and(|r| r.capacity > 0)
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
    // spec: S-0605 F1 and claim 2 (passengers counted, not sizes: exact integers; s ≤ L compared directly as doubles)
    i64::from(count(w, b)) + 1 <= i64::from(r.capacity) && size(w, u).le(r.size_limit)
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
    // spec: S-0605 claim 1 (otherwise it accepts exactly when F1 holds; no other entry feature is built, so a failed F1 refuses)
    if fits(w, b, u) {
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

/// A refused loading request (claims 3 and 5): the actor gives up the
/// contact and, under Q-1266's interim rule, forgets the building as its
/// destination and stops walking, so that the fall-back to rest makes it
/// stand (scenario A6) instead of walking on and going in under claim 6.
fn loading_given_up(w: &mut World, u: ObjectId) {
    give_up_contact(w, u);
    // Q-1266 interim (S-0605 claim 5 and scenario A6: it stops and stands, it does not go in)
    crate::econ::assign_destination(w, u, None);
    if let Some(ob) = w.obj_mut(u) {
        ob.path.steps.clear();
    }
}

fn set_waiting(w: &mut World, u: ObjectId, b: Option<ObjectId>) {
    if let Some(ob) = w.obj_mut(u) {
        ob.structure.waiting = b;
    }
}

// ---------------------------------------------------------------------------
// Getting contact (claims 3 to 5)

/// Claim 3 for actor `u` given absorbing building `b` as its destination
/// with *enter*: contact, or waiting in line.
fn seek_contact(w: &mut World, u: ObjectId, b: ObjectId) {
    if w.obj(u).is_none_or(|o| o.contact.is_some()) {
        return;
    }
    // spec: S-0605 claim 3 (the building already holds any contact: waiting in line, destination cleared, stays where it is); Q-1261 interim (a contact not granted for another reason also waits)
    if !contacts(w, b).is_empty() || !grants_contact(w, b, u) {
        crate::econ::assign_destination(w, u, None);
        if let Some(ob) = w.obj_mut(u) {
            ob.path.steps.clear();
        }
        set_waiting(w, u, Some(b));
        return;
    }
    if let Some(ob) = w.obj_mut(u) {
        ob.contact = Some(b);
    }
    // spec: S-0605 claim 3 (with contact it asks to load; switched off: it gives up the contact)
    if loading_refused(w, b) {
        loading_given_up(w, u);
    }
}

/// After an enter order's ordinary effects (claim 3, Q-1261 interim): an
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

/// The per-tick retry of an actor waiting in line (claim 4), in its own
/// turn after its movement (Q-1261 interim).
pub fn waiting_turn(w: &mut World, u: ObjectId) {
    let Some(b) = waiting_for(w, u) else { return };
    // spec: S-0605 claim 4 (a building that has left the game ends the wait)
    if w.obj(b).is_none_or(|o| !o.on_map) {
        set_waiting(w, u, None);
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
        // spec: S-0605 claim 4 (accept: it queues enter with that building as its destination and stops waiting)
        if let Some(ob) = w.obj_mut(u) {
            ob.beh.queue(Some(Behaviour::Enter));
            ob.path.steps.clear();
            ob.path.retry_at = 0;
        }
        crate::econ::assign_destination(w, u, Some(Target::Object(b)));
    } else {
        // spec: S-0605 claim 4 (refuse: it gives up the contact, its behaviour becomes none, its destination is cleared and it stops waiting)
        give_up_contact(w, u);
        if let Some(ob) = w.obj_mut(u) {
            ob.beh.current = None;
            ob.beh.queued = None;
        }
        crate::econ::assign_destination(w, u, None);
    }
}

/// The absorbing building an actor in *enter* is entering: its contact,
/// else the object it remembers entering (S-0215 claim 29).
fn partner(w: &World, u: ObjectId) -> Option<ObjectId> {
    let o = w.obj(u)?;
    // spec: S-0215 claim 29 (to its contact when present, otherwise to the object it remembers entering)
    o.contact
        .or(match o.dest {
            Some(Target::Object(k)) => Some(k),
            _ => None,
        })
        .filter(|&k| is_absorber(w, k))
}

/// Whether an actor in *enter* is entering an absorbing building.
pub fn entering_absorber(w: &World, u: ObjectId) -> bool {
    partner(w, u).is_some()
}

/// The *enter* processing toward an absorbing building (claim 5, S-0215
/// claim 29). Returns the delay.
pub fn process_enter(w: &mut World, u: ObjectId) -> i32 {
    let Some(b) = partner(w, u) else {
        crate::process::fall_back(w, u, false);
        return crate::process::standard_delay(w, u);
    };
    if loading_refused(w, b) {
        // spec: S-0605 claim 5 (switched off since then: it gives up the contact and falls back to rest)
        loading_given_up(w, u);
        crate::process::fall_back(w, u, false);
    }
    // spec: S-0215 claims 6 and 29 (the processing ends with the standard delay)
    crate::process::standard_delay(w, u)
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
    // spec: S-0605 claim 6 and S-0524 claim 12 (one entry raise on the building's tag, the entrant as causing object, while it is still outside)
    crate::mp_triggers::entry_raise(w, b, u);
    // The raise's actions may have removed either object.
    if !w.obj(u).is_some_and(|o| o.on_map) || !w.obj(b).is_some_and(|o| o.on_map) {
        return w.obj(u).is_none_or(|o| !o.on_map);
    }
    // spec: S-0605 claim 6 (the infantry absorber agrees unless it is being sold; nothing more happens at this arrival when it does not)
    let selling = w.obj(b).is_some_and(|o| {
        o.econ.sale.is_some() || crate::econ::placement::seen_behaviour(o) == Some(Behaviour::Selling)
    });
    if selling {
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
    if let Some(ob) = w.obj_mut(u) {
        // spec: S-0605 claim 7 (its contact ends, freeing the link place)
        ob.contact = None;
        ob.structure.absorbed_in = Some(b);
        ob.structure.waiting = None;
        ob.layer = Layer::Ground;
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
    let z = w.surface_at(256 * c.x + 128, 256 * c.y + 128);
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
            // spec: S-0605 claim 12 and Edge cases (quirks.absorber_release_bridge_ends_search: a deck neighbour at the eighth direction that passes both cell tests ends the search; Q-1262 interim)
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
/// S-0408 claim 8). `sale` chooses the sale's standing position (one
/// R(0, 3) draw, S-0408 claim 7). Returns whether it was placed.
fn place_released(w: &mut World, b: ObjectId, p: ObjectId, c: CellXY, sale: bool) -> bool {
    let Some(bo) = w.obj(b).cloned() else {
        return false;
    };
    let Some(kind) = w.obj(p).map(|o| o.kind) else {
        return false;
    };
    let (x, y) = if kind == Kind::Infantry {
        let k = if sale {
            // spec: S-0408 claims 7 and 8 (the desired point 128, 164 of the cell: class centre, one R(0, 3) draw unless the record blocks every position)
            crate::walk::choose_position(
                w,
                c,
                Layer::Ground,
                crate::cells::position::CENTRE,
                false,
            )
        } else if crate::occupancy::no_position_by_flags(w, c, Layer::Ground) {
            None
        } else {
            // spec: S-0312 claim 24 (an infantry unit on a free standing position); Q-1264 interim (the first free one of S-0208 table P, no draw)
            (0u8..5).find(|&k| {
                w.cells
                    .occ(c, Layer::Ground)
                    .is_none_or(|o| !o.taken(k))
            })
        };
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
/// building's charge planter. With no crew all die.
pub fn destroyed(w: &mut World, b: ObjectId, hit: &Hit) {
    let list = passengers(w, b);
    if list.is_empty() {
        return;
    }
    let Some(bo) = w.obj(b).cloned() else { return };
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
    // spec: S-0312 claim 24 and F5 (no crew: killing damage that ignores defences; the map setting is not modelled)
    let no_crew = hit.ignore_defences;
    for (i, &p) in list.iter().enumerate() {
        let placed = !no_crew
            // spec: S-0312 claim 24 and Edge cases (beyond the last cell: not placed)
            && cells
                .get(i)
                .is_some_and(|&c| place_released(w, b, p, c, false));
        if !placed {
            leave_order(w, b, p);
            // spec: S-0312 claim 24 (one that cannot be placed, or every one without a crew, dies)
            crate::combat::death::passenger_death(w, p, &credit);
        }
    }
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
        let placed = cells
            .get(i)
            .is_some_and(|&c| place_released(w, b, p, c, true));
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
        // Q-1263 interim (S-0605 claim 15 open question): the removal makes power due as a release does.
        power_due(w, b);
    }
    set_waiting(w, id, None);
    if count(w, id) > 0 {
        erased(w, id);
    }
}

// ---------------------------------------------------------------------------
// The bio-reactor fate (S-0601 claim 8, S-0215 claim 24)

/// The bio-reactor fate of S-0601 claim 8: S-0215 claim 24's search for
/// *enter the nearest absorbing building*, made once (Q-1260 interim):
/// the absorbing buildings of the object's own owner that accept it, the
/// nearest by F4's 3-D distance to the building's centre, ties to the
/// lowest object number. Found: that building as destination and *enter*
/// queued; returns whether one was found.
pub fn bio_reactor_fate(w: &mut World, u: ObjectId) -> bool {
    let Some(uo) = w.obj(u).cloned() else {
        return false;
    };
    if !uo.on_map || uo.owner.is_none() {
        return false;
    }
    let mut best: Option<(i64, ObjectId)> = None;
    for o in w.objects.values() {
        // spec: S-0215 claim 24 (the absorbing buildings of its own owner that accept it as a passenger)
        if o.owner != uo.owner || !o.on_map || answer(w, o.id, u) != Answer::Accepts {
            continue;
        }
        // spec: S-0215 claim 24 and F4 (the nearest by the 3-D distance ρ₃)
        let d = crate::approach::rho3(w, uo.pos, w.reference_point(o.id));
        if best.is_none_or(|(bd, _)| d < bd) {
            best = Some((d, o.id));
        }
    }
    let Some((_, b)) = best else {
        return false;
    };
    // spec: S-0215 claim 24 (it takes that building as its destination and queues enter, unless already in contact with it)
    if uo.contact == Some(b) {
        return true;
    }
    if let Some(ob) = w.obj_mut(u) {
        ob.beh.queue(Some(Behaviour::Enter));
        ob.path.steps.clear();
        ob.path.retry_at = 0;
    }
    crate::econ::assign_destination(w, u, Some(Target::Object(b)));
    seek_contact(w, u, b);
    true
}
