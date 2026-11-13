//! The depot cycle (S-0314 claims 6 and 16 to 24; S-0504 claims 8, 12 and
//! 14; S-0215 claim 29): a repair depot, or a naval yard with the repair
//! flag, accepts an allied vehicle's entry, the vehicle approaches its
//! docking point and docks, and the depot repairs it in three phases
//! (waiting, settling, working) until it is sent away; orders, sale,
//! destruction and power end or keep the contact as the claims say.
//!
//! State: the vehicle side is [`DepotState::link`] (the depot place it
//! holds, valid only while the vehicle's contact is that depot, and the
//! tether of the completed entry exchange), [`DepotState::queued`] (a
//! queued entry, claim 23), [`DepotState::entering`] (the remembered
//! depot, claim 23) and [`DepotState::settled`] (Terms, claims 21 and
//! 22). The depot side is its *repair* behaviour's progress count (the
//! phase), [`DepotState::serviced`] and [`DepotState::due`].
//!
//! Tether (claim 17): only the repeated entry request at a depot with the
//! pad flag tethers a vehicle ([`pad_exchange`]); an ordinary depot's
//! vehicle stays untethered while it is repaired. Reaching the docking
//! point's cell during movement queues *repair* and *sleep*
//! ([`movement_arrival`]).

use std::collections::BTreeMap;

use glasnost_map::CellXY;
use serde::{Deserialize, Serialize};

use crate::combat::heal::{self, StepOutcome};
use crate::rules::{Behaviour, Kind};
use crate::world::{ObjectId, Pos, Target, World};

/// A vehicle's depot contact (S-0314 claim 17): the place it holds,
/// whether the completed entry exchange has tethered it (*docked*) or not
/// yet (*approaching*, claims 20 to 24), and whether it has made its
/// movement arrival in the docking-point cell (claims 17 and 22).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Link {
    /// The depot.
    pub depot: ObjectId,
    /// The contact place, counted from 0.
    pub place: u32,
    /// Tethered: docked.
    pub tethered: bool,
    /// The vehicle has made its movement arrival in this depot's
    /// docking-point cell since it took this contact (claim 17): what
    /// claim 22's destruction row tells apart.
    pub arrived: bool,
}

/// Whether a vehicle is settled, and where (S-0314 Terms, *Settled
/// vehicle*). The variants are in this order so that the saved byte of a
/// vehicle that is not settled, or settled at a depot, is the one the
/// earlier flag had (postcard writes a unit variant as its index).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Settled {
    /// Not settled.
    #[default]
    No,
    /// Settled at a depot (Terms, *Settled at a depot*): the event that last
    /// made it settled was its movement arrival (claim 17), a settling
    /// processing (claim 21), or a disabling while it had contact with a
    /// depot or while a depot occupied its cell.
    AtDepot,
    /// Settled away from a depot (Terms, *Away from a depot*): the event
    /// that last made it settled was a disabling anywhere else.
    Away,
}

/// Depot-cycle state of an object (S-0314 claims 6 and 16 to 24).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepotState {
    /// A vehicle's depot contact; valid only while its contact is the
    /// depot (S-0215 Terms, contact).
    pub link: Option<Link>,
    /// A vehicle's queued entry for a depot (S-0314 claim 23).
    pub queued: Option<ObjectId>,
    /// A settled vehicle (S-0314 Terms, claims 21 and 22): made settled by
    /// its movement arrival, a settling processing or a disabling; it can
    /// outlast the contact. Which of these last made it settled decides
    /// whether it is settled at a depot (Terms, *Settled at a depot*).
    #[serde(alias = "loco_off")]
    pub settled: Settled,
    /// A depot's serviced vehicle at its last repair processing (S-0314
    /// Edge cases, several contact places: a change restarts the phases).
    pub serviced: Option<ObjectId>,
    /// A vehicle's remembered entry object (S-0215 claim 29; S-0605
    /// Terms): the depot that claim 23's order on a depot holding a contact
    /// makes a damaged vehicle remember (S-0314 claim 23).
    pub entering: Option<ObjectId>,
    /// A depot's running modelled slot animations (S-0108 claim 20), by
    /// slot: their numbers in the update order (`crate::depot_anim`).
    #[serde(default)]
    pub slots: BTreeMap<DepotSlot, ObjectId>,
    /// A depot's slot condition is *damaged* (S-0108 Terms, claim 12), for
    /// its modelled slots.
    #[serde(default)]
    pub slot_damaged: bool,
    /// A working depot's tick at which its next repair step falls due
    /// (S-0314 claim 6: the working phase is processed every tick, the steps
    /// spaced by the step count).
    #[serde(default)]
    pub due: i32,
}

/// The phases of a depot's repair, kept in its *repair* behaviour's
/// progress count, which every behaviour start sets to 0 (S-0215 claim 3).
mod phase {
    // nocite: our own numbering of S-0314 claim 6's three phases, in claim order
    pub const WAITING: i32 = 0;
    // nocite: our own numbering of S-0314 claim 6's three phases, in claim order
    pub const SETTLING: i32 = 1;
    // nocite: our own numbering of S-0314 claim 6's three phases, in claim order
    pub const WORKING: i32 = 2;
    // nocite: our own number for the 1-tick wait of S-0314 claim 6's waiting row with no contact, after the three phases
    pub const LEAVING: i32 = 3;
}

/// The settling hold distance (S-0314 claim 21), strict.
// spec: S-0314 Constants (settling hold distance: 200 leptons, strict, to the docking point)
const SETTLING_HOLD: i32 = 200;

/// The place-0 distance of the pad exchange (S-0314 claim 17), strict.
// spec: S-0314 claim 17 (the vehicle in contact place 0 more than 128 leptons from the foundation centre)
const PAD_PLACE0: i32 = 128;

/// The vertical difference that still stops a depot approach in the
/// docking-point cell (S-0314 claim 18), strict.
// spec: S-0314 claim 18 (a vertical difference strictly below 208 leptons)
const DOCK_HEIGHT: i32 = 208;

/// The early guard start distance (S-0314 claim 17), strict.
// spec: S-0314 Constants (early guard repair-start distance: 64 leptons, strict, to the foundation centre)
const EARLY_START: i32 = 64;

/// Whether `k` is a building whose type has the repair-depot flag
/// (S-0314 Import mapping, `UnitRepair`; S-0504 claim 8 for yards).
pub fn is_depot(w: &World, k: ObjectId) -> bool {
    w.obj(k).is_some_and(|b| {
        b.kind == Kind::Building
            && w.rules()
                .buildings
                .get(b.type_index as usize)
                .is_some_and(|r| r.building.unit_repair)
    })
}

/// A building slot that the simulation models (S-0108 Terms: the
/// building's animation slots): a repair depot's (S-0108 claim 20; S-0314
/// claims 6 and 22) and the docking, payment and unloading animations of
/// the building beside an unloading harvester (S-0402 claim 11), declared
/// in S-0108 claim 12's restart order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DepotSlot {
    /// The *pre-production* slot: the docking animation (S-0402 Terms).
    PreProduction,
    /// The *production* slot: while it runs, settling takes no step
    /// (S-0314 claim 6); a refinery-flag building's unloading animation
    /// (S-0402 Terms).
    Production,
    /// The *special-1* slot; the payment animation (S-0402 Terms).
    Special1,
    /// The *special-2* slot.
    Special2,
    /// The *special-3* slot: while it runs, a settling depot with no
    /// contact stays in its repair behaviour (S-0314 claim 22).
    Special3,
}

/// Whether depot `k`'s slot animation `slot` is running, as deterministic
/// simulation state with its duration from the art data (S-0108 claim 20;
/// S-0314 claims 6 and 22). Naval yards with the repair flag are included
/// (S-0504 claim 8).
pub fn slot_running(w: &World, k: ObjectId, slot: DepotSlot) -> bool {
    // spec: S-0108 claim 20 (the simulation models the slot animation's duration from the art data as deterministic simulation state)
    crate::depot_anim::running(w, k, slot)
}

/// Whether depot `k` is in contact (S-0108 claim 20): it holds a contact
/// with a vehicle in any of its contact places (S-0314 claim 17),
/// tethered or not.
pub fn in_contact(w: &World, k: ObjectId) -> bool {
    !contacts(w, k).is_empty()
}

/// A vehicle's depot link, when its contact is that depot.
pub fn link(w: &World, id: ObjectId) -> Option<Link> {
    let o = w.obj(id)?;
    o.depot.link.filter(|l| o.contact == Some(l.depot))
}

/// Whether vehicle `id` is docked (tethered) at a depot (S-0213 Terms,
/// tethered; S-0314 claims 20 to 24).
pub fn tethered(w: &World, id: ObjectId) -> bool {
    link(w, id).is_some_and(|l| l.tethered)
}

/// The vehicles in depot contact with `k`, by place.
fn contacts(w: &World, k: ObjectId) -> Vec<(u32, ObjectId)> {
    let mut v: Vec<(u32, ObjectId)> = w
        .objects
        .values()
        .filter_map(|o| {
            o.depot
                .link
                .filter(|l| l.depot == k && o.contact == Some(k))
                .map(|l| (l.place, o.id))
        })
        .collect();
    v.sort_unstable();
    v
}

/// The depot's contact places, max(*d*, 1) (S-0314 claim 17).
fn places(w: &World, k: ObjectId) -> u32 {
    // spec: S-0314 claim 17 and Import mapping (NumberOfDocks, absent 1: max(d, 1) contact places)
    crate::econ::release::contact_places(w, k)
}

/// The place vehicle `id` may take at `k`: the one it already holds, else
/// the first free one (S-0314 claim 16: a place assigned to this vehicle is
/// not occupied by another).
fn free_place(w: &World, k: ObjectId, id: ObjectId) -> Option<u32> {
    if let Some(l) = link(w, id).filter(|l| l.depot == k) {
        return Some(l.place);
    }
    let held: Vec<u32> = contacts(w, k).into_iter().map(|(p, _)| p).collect();
    // spec: S-0314 claim 17 (contact takes the first free place)
    (0..places(w, k)).find(|p| !held.contains(p))
}

/// Whether vehicle `id` stands on depot `k`: its cell is one of the
/// foundation's cells.
fn standing_on(w: &World, k: ObjectId, id: ObjectId) -> bool {
    let (Some(b), Some(o)) = (w.obj(k), w.obj(id)) else {
        return false;
    };
    crate::objects::foundation_cells(b.pos.cell(), b.foundation).contains(&o.pos.cell())
}

/// The depot's alliance and free-place tests (S-0314 claims 16 and 23).
fn alliance_and_place(w: &World, k: ObjectId, id: ObjectId) -> Option<u32> {
    let (Some(b), Some(o)) = (w.obj(k), w.obj(id)) else {
        return None;
    };
    // spec: S-0314 claim 16 (the depot's owner must count the vehicle as allied)
    if !w.allied(b.owner, o.owner) {
        return None;
    }
    free_place(w, k, id)
}

/// The depot's permission (S-0314 claim 16; S-0214 table V, building-entry
/// value n): whether depot `k` accepts vehicle `id`. `standing` applies the
/// refusal of a vehicle already standing on the depot: the click and the
/// queued-entry retry ask it; the repeated entry request of an entering
/// vehicle does not (claim 17).
pub fn accepts(w: &World, k: ObjectId, id: ObjectId, standing: bool) -> bool {
    if crate::structures::bunker::is_bunker(w, k) {
        // spec: S-0606 claim 20 (a type that is both a repair depot and a bunker: claim 2's permission rows apply, and S-0314 claim 16's depot rows do not)
        return is_depot(w, k) && crate::structures::bunker::permits(w, k, id);
    }
    // spec: S-0314 claim 16 (no contact place free for this vehicle prevents acceptance)
    permits_but_place(w, k, id, standing) && free_place(w, k, id).is_some()
}

/// Claim 16's permission without its free-place test: every other test,
/// the alliance test included.
fn permits_but_place(w: &World, k: ObjectId, id: ObjectId, standing: bool) -> bool {
    let (Some(b), Some(o)) = (w.obj(k), w.obj(id)) else {
        return false;
    };
    if !is_depot(w, k) || b.health <= 0 || !b.on_map || o.kind != Kind::Vehicle {
        return false;
    }
    // spec: S-0314 claim 16 (construction, selling and the build-up state refuse)
    if matches!(
        crate::econ::placement::seen_behaviour(b),
        Some(Behaviour::Construction | Behaviour::Selling)
    ) || b.econ.sale.is_some()
    {
        return false;
    }
    // spec: S-0314 claims 16 and 22 (a depot switched off refuses new acceptance; low power refuses nothing)
    if b.econ.building.switched_off {
        return false;
    }
    // spec: S-0314 claim 16 and S-0504 claim 7 (the naval match applies)
    if crate::naval::naval_match_refuses(w, k, id) {
        return false;
    }
    // spec: S-0314 claim 16 (a vehicle already standing on this very depot receives no new acceptance through this query); claim 17 (the repeated entry request is not a repeat of that test: entering a foundation cell does not trigger the refusal)
    if standing && standing_on(w, k, id) {
        return false;
    }
    // spec: S-0314 claim 16 (the alliance test; no damage or money gate)
    w.allied(b.owner, o.owner)
}

/// Takes depot contact in place `place` (S-0314 claim 17): a vehicle holds
/// one contact, so any other is released.
fn take(w: &mut World, id: ObjectId, k: ObjectId, place: u32) {
    let keep = link(w, id).is_some_and(|l| l.depot == k && l.tethered);
    // spec: S-0314 claims 17 and 22 (a repeated request that keeps this depot's contact does not undo the movement arrival already made)
    let arrived = link(w, id).is_some_and(|l| l.depot == k && l.arrived);
    let fresh = link(w, id).is_none_or(|l| l.depot != k);
    let mut to_depot = false;
    if let Some(o) = w.obj_mut(id) {
        o.contact = Some(k);
        o.depot.link = Some(Link {
            depot: k,
            place,
            tethered: keep,
            arrived,
        });
        to_depot = o.dest == Some(Target::Object(k));
    }
    if fresh && to_depot {
        // spec: S-0314 claim 18 and S-0207 claim 25 (the depot assigned as destination supplies its docking point, not its position, to driving or ship locomotion; our contact follows the destination's assignment within the same order)
        crate::drive::take_destination(w, id);
    }
}

/// An entry request (S-0215 claim 29, S-0314 claims 16 and 17): on
/// acceptance the vehicle takes contact in its place before it arrives.
pub fn request(w: &mut World, id: ObjectId, k: ObjectId, standing: bool) -> bool {
    if !accepts(w, k, id, standing) {
        return false;
    }
    let Some(place) = free_place(w, k, id) else {
        return false;
    };
    take(w, id, k, place);
    true
}

/// The reply to the repeated entry request of a vehicle in *enter*
/// (S-0215 claim 29; S-0314 claims 17 and 23).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Reply {
    /// Accepted, and the vehicle holds its contact place.
    Contact,
    /// Accepted with no place free for this vehicle: no contact taken.
    NoPlace,
    /// Not accepted.
    Refused,
}

/// The repeated entry request of an entering vehicle (S-0215 claim 29):
/// claim 16's free-place refusal governs the click and the queued-entry
/// retry, not this request, so with no free place the depot accepts and
/// gives no contact (S-0314 claim 23).
fn repeated_request(w: &mut World, id: ObjectId, k: ObjectId) -> Reply {
    // spec: S-0314 claim 17 (the repeated request is not a repeat of claim 16's test: standing on the depot does not refuse it)
    if !permits_but_place(w, k, id, false) {
        return Reply::Refused;
    }
    // spec: S-0314 claims 16 and 17 (revision 107: each repeated entry request to a repair depot also asks the vehicle whether it needs repair; at full health, a health ratio of exactly 1, it answers no and the depot refuses; a naval yard with the repair flag and a type that is also a bunker are repair depots, claim 17's "either")
    if health_full(w, id) {
        return Reply::Refused;
    }
    // spec: S-0606 claim 20 (a type that is both a repair depot and a bunker: the repeated request is also refused when claim 3's eligibility check refuses it)
    if crate::structures::bunker::is_bunker(w, k) && !crate::structures::bunker::eligible(w, id) {
        return Reply::Refused;
    }
    let Some(place) = free_place(w, k, id) else {
        // spec: S-0314 claim 23 (with no free place for this vehicle the depot answers accept and takes no contact; claim 16's free-place refusal governs the click and the retry, not this repeated request)
        return Reply::NoPlace;
    };
    // spec: S-0314 claim 23 (if a place is free when a processing runs, that request takes the contact)
    take(w, id, k, place);
    Reply::Contact
}

/// Releases vehicle `id`'s depot contact, its queued entry and its
/// remembered depot.
pub(crate) fn release(w: &mut World, id: ObjectId) {
    end_contact(w, id);
    if let Some(o) = w.obj_mut(id) {
        o.depot.queued = None;
        o.depot.entering = None;
    }
}

/// Ends vehicle `id`'s depot contact; its queued entry stays.
fn end_contact(w: &mut World, id: ObjectId) {
    let strand = w.rules().quirks.depot_loss_strands_hover;
    let mut had = false;
    if let Some(o) = w.obj_mut(id) {
        had = o.depot.link.is_some();
        if o.depot.link.is_some_and(|l| o.contact == Some(l.depot)) {
            o.contact = None;
        }
        o.depot.link = None;
    }
    if had && !strand {
        // spec: S-0314 Edge cases and Terms (quirks.depot_loss_strands_hover off: losing contact with a depot ends being settled, and the vehicle is switched on again)
        unsettle(w, id);
    }
}

/// Makes vehicle `id` settled (S-0314 Terms, claim 21): a hovering vehicle
/// is switched off in S-0503 claim 10's sense, its two draws taken when
/// that claim's conditions hold. Also the repeated switch-off of a settled
/// vehicle not yet down.
fn settle(w: &mut World, id: ObjectId) {
    if let Some(o) = w.obj_mut(id) {
        // spec: S-0314 Terms (Settled at a depot: the movement arrival, claim 17, and a settling processing, claim 21, make it settled at a depot)
        o.depot.settled = Settled::AtDepot;
    }
    // spec: S-0314 claim 21 and Changes for other specs (S-0503 claim 10: switched off when it becomes settled and at each later settling processing that finds it within 200 leptons, settled, standing still and not yet down; each switch-off meets claim 10's conditions on its own)
    crate::hover::switched(w, id, true);
}

/// Ends vehicle `id`'s being settled (S-0314 Terms): a hovering vehicle is
/// switched on again, also while it is still disabled (claim 21).
fn unsettle(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj_mut(id) else { return };
    if o.depot.settled == Settled::No {
        return;
    }
    o.depot.settled = Settled::No;
    // spec: S-0314 Terms and claim 21 (at every end of being settled a hovering vehicle is switched on again, with no exception for the disabled state)
    crate::hover::switched(w, id, false);
}

/// The disabling of S-0405 claim 20 made the vehicle settled with its
/// switch-off (S-0314 Terms, claim 21: whether or not it was settled or
/// switched on before), wherever it is (Terms, *Away from a depot*). It is
/// settled at a depot only when it has contact with a depot or a depot
/// occupies its cell at the disabling; a disabling anywhere else leaves it
/// settled away, even if an earlier event had made it settled at a depot.
/// The caller has already switched it off.
pub(crate) fn disabled(w: &mut World, id: ObjectId) {
    if !w.obj(id).is_some_and(|o| o.kind == Kind::Vehicle) {
        return;
    }
    // spec: S-0314 Terms (Settled at a depot: a disabling while it had contact with a depot or while a depot occupied its cell, claim 21 Disabling while docked; a disabling anywhere else makes it settled but not settled at a depot)
    let at = if link(w, id).is_some() || occupied_by(w, id, |r| r.building.unit_repair) {
        Settled::AtDepot
    } else {
        Settled::Away
    };
    if let Some(o) = w.obj_mut(id) {
        // spec: S-0314 Terms and claim 21 (the switch-off of a disabling of S-0405 claim 20 makes the vehicle settled, wherever it is)
        o.depot.settled = at;
    }
}

/// The enabling of S-0405 claim 20 ends the vehicle's being settled
/// (S-0314 Terms, claim 21).
pub(crate) fn enabled(w: &mut World, id: ObjectId) {
    // spec: S-0314 Terms and claim 21 (an enabling switches a settled vehicle on again and ends its being settled)
    unsettle(w, id);
}

/// Whether vehicle `id` is settled (S-0314 Terms), at a depot or away.
pub fn settled(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.depot.settled != Settled::No)
}

/// Whether vehicle `id` is settled at a depot (S-0314 Terms, *Settled at a
/// depot*). Only `quirks.docked_hover_water_loss` reads it.
pub fn settled_at_depot(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .is_some_and(|o| o.depot.settled == Settled::AtDepot)
}

/// Whether `k` has the obsolete-unloading flag: S-0314 claim 18 and S-0504
/// claim 12 give such a building the fixed docking point; the
/// vehicle-factory flag does not.
fn obsolete_unloading(w: &World, k: ObjectId) -> bool {
    w.obj(k)
        .and_then(|b| w.rules().buildings.get(b.type_index as usize))
        // spec: S-0314 Import mapping (building Weeder: the obsolete-unloading flag, absent no)
        .is_some_and(|r| r.raw.econ.weeder.unwrap_or(false))
}

/// The docking point depot `k` supplies for vehicle `id` (S-0314 claim 18;
/// S-0504 claim 12).
pub fn docking_point(w: &World, k: ObjectId, id: ObjectId) -> Pos {
    if crate::structures::bunker::is_bunker(w, k) {
        // spec: S-0606 claim 20 (a type that is both a repair depot and a bunker: claim 4's quadrant point, not S-0314 claim 18's)
        return crate::structures::bunker::docking_point(w, k, id);
    }
    if obsolete_unloading(w, k) {
        // spec: S-0314 claim 18 and S-0504 claim 12 (the obsolete-unloading flag, not the vehicle-factory flag, selects the centre of the cell (2, 1) from the top-left cell, at the building's height, before pad displacements)
        if let Some(b) = w.obj(k) {
            let c = b.pos.cell();
            let cell = CellXY::new(c.x.wrapping_add(2), c.y.wrapping_add(1));
            return Pos::centre(cell, b.pos.z);
        }
    }
    let place = link(w, id)
        .filter(|l| l.depot == k)
        .map_or_else(|| free_place(w, k, id).unwrap_or(0), |l| l.place);
    // spec: S-0314 claim 18 (the foundation centre with its height plus the displacement of the assigned place, S-0404 claim 15's selection rules, signed wrapping, not rotated)
    crate::econ::release::pad_position_at(w, k, place)
}

/// The heading point of a vehicle whose destination is the depot it is in
/// contact with (S-0314 claim 18: the docking point, not the building's
/// position); `None` for every other destination.
pub fn heading_point(w: &World, id: ObjectId, dest: ObjectId) -> Option<Pos> {
    let l = link(w, id)?;
    (l.depot == dest).then(|| docking_point(w, dest, id))
}

/// Whether a vehicle's *enter* goes to a repair depot (S-0215 claim 29).
pub fn entering_depot(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if o.kind != Kind::Vehicle {
        return false;
    }
    if link(w, id).is_some() {
        return true;
    }
    // spec: S-0215 claim 29 and S-0314 claim 23 (the request goes to the contact when present, else to the object it remembers entering: the depot of claim 23's order)
    if o.depot.entering.is_some_and(|k| is_depot(w, k)) {
        return true;
    }
    matches!(o.dest, Some(Target::Object(k)) if is_depot(w, k))
}

/// Whether the vehicle stands still: not moving and not turning, for the
/// early start of claim 17.
fn still(w: &World, id: ObjectId) -> bool {
    !crate::occupancy::in_progress(w, id) && w.obj(id).is_some_and(|o| !o.body.turning(w.tick))
}

/// A vehicle's *enter* processing at a repair depot (S-0215 claim 29;
/// S-0314 claims 16 to 18). Returns the delay: every outcome chooses the
/// standard delay (S-0215 claim 29).
pub fn process_enter(w: &mut World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id).cloned() else {
        return 1;
    };
    // spec: S-0215 claim 29 (the entry request goes to the contact when present, else to the object it remembers entering); S-0314 claim 23 (the depot of claim 23's order is the remembered entry object); otherwise the depot the order named as destination
    let remembered = o.depot.entering.filter(|&k| is_depot(w, k));
    let (k, from_memory) = match (link(w, id), remembered, o.dest) {
        (Some(l), _, _) => (l.depot, false),
        (None, Some(k), _) => (k, true),
        (None, None, Some(Target::Object(k))) => (k, false),
        _ => {
            crate::process::fall_back(w, id, false);
            return crate::process::standard_delay(w, id);
        }
    };
    if !tethered(w, id) {
        match repeated_request(w, id, k) {
            Reply::Refused => {
                // spec: S-0215 claim 29 and S-0314 claim 17 (only acceptance or an existing tether permits continued entry; without a tether a non-accepting reply ends contact and the actor falls back to rest); claim 23 (its queued entry stays and the retry still runs)
                end_contact(w, id);
                crate::process::fall_back(w, id, false);
                return crate::process::standard_delay(w, id);
            }
            // spec: S-0314 claim 23 (accepted without contact: the vehicle stays where it is, in enter, with no destination, and chooses the standard delay)
            Reply::NoPlace => return crate::process::standard_delay(w, id),
            Reply::Contact => {}
        }
        if pad_exchange(w, k, id)
            && let Some(ob) = w.obj_mut(id)
            && let Some(l) = ob.depot.link.as_mut()
        {
            // spec: S-0314 claim 17 (the pad depot's repeated request establishes tether; without the pad flag this exchange sets none, and neither reservation nor occupying a foundation or docking-point cell sets it)
            l.tethered = true;
        }
    }
    if !tethered(w, id) {
        if from_memory && o.dest.is_none() {
            // spec: S-0314 claim 23 (the request that takes a free place: the retry later in the same turn finds the contact held, asks permission and, granted, assigns the depot as destination and revision 28, Q-1158 answered: this processing gives no destination of its own)
            return crate::process::standard_delay(w, id);
        }
        let arrived = at_dock(w, k, id);
        // spec: S-0314 claim 18 (the depot directs the vehicle toward the building as its destination; the docking point is the destination position); claim 17 (an untethered vehicle that has reached the docking-point cell is not sent off again)
        if !arrived && (o.dest != Some(Target::Object(k)) || o.path.steps.is_empty()) {
            crate::econ::assign_destination(w, id, Some(Target::Object(k)));
            if let Some(ob) = w.obj_mut(id) {
                ob.path.steps.clear();
                // spec: S-0212 claim 21 and its state table, row "destination changes" (r97: for the driving and walk locomotors a different destination, or none, ends a running retry delay, path::destination_changed and path::waits; the ship and the other locomotors keep it, low confidence; the hovering locomotor's is S-0503's)
            }
        }
        return crate::process::standard_delay(w, id);
    }
    if o.depot.queued == Some(k) {
        // spec: S-0314 claim 20 (revision 28: the processing of a docked vehicle's enter on its own depot is only claim 17's repeated entry request: the depot accepts the vehicle it already holds, a pad depot sets the tether again; nothing is queued, neither repair at the depot nor sleep at the vehicle, and no destination is given)
        return crate::process::standard_delay(w, id);
    }
    // spec: S-0314 claim 17 (revision 29: an ordinary repair depot, without the resource-unloading or obsolete-unloading flag, supplies no completed-entry confirmation through its repeated entry request, with or without the pad flag or an existing tether: no turn toward 16384 facing units and no second arrival report; acceptance or the tether only lets entry continue)
    crate::process::standard_delay(w, id)
}

/// Whether the vehicle has a moving destination (S-0314 Terms): an
/// order-level destination while its locomotor gives claim 17's movement
/// report. Driving and ship: its locomotor destination point is present
/// (S-0207 claim 25), or its heading point, the track target, is present
/// and differs horizontally from its position. Hovering: S-0503 claim 5's
/// *is moving*.
pub(crate) fn moving_destination(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    // spec: S-0314 Terms (an order-level movement destination is needed in every row)
    if o.dest.is_none() {
        return false;
    }
    if o.hover.is_some() {
        // spec: S-0314 claim 17 (hovering: a locomotor destination point or heading point present, S-0503 claim 5's is moving, not the throttle-dependent is moving now)
        return crate::hover::moving(w, id);
    }
    if let Some(d) = o.drive.as_ref() {
        // spec: S-0314 claim 17 (driving or ship: the locomotor destination point present, or the heading point present and differing from the current horizontal position, x and y only; a turn in place, a track or stored steps are not sufficient by themselves)
        // spec: S-0314 claim 17 and S-0207 claim 25 (the separate point's presence, not goal-cell inequality or search success)
        let dest_point = crate::drive::goal_present(w, id);
        let heading = d
            .track
            .as_ref()
            .is_some_and(|t| t.target != (o.pos.x, o.pos.y));
        return dest_point || heading;
    }
    // Other locomotors: their own movement test.
    crate::occupancy::in_progress(w, id)
}

/// The pad depot's tether exchange in the repeated entry request (S-0314
/// claim 17).
pub fn pad_exchange(w: &World, k: ObjectId, id: ObjectId) -> bool {
    let Some(bt) = w
        .obj(k)
        .and_then(|b| w.rules().buildings.get(b.type_index as usize))
    else {
        return false;
    };
    // spec: S-0314 claim 17 and Import mapping (a depot with the pad flag, Helipad, and neither the resource-unloading, Refinery, nor the obsolete-unloading flag)
    if bt.raw.helipad != Some(true)
        || bt.raw.econ.refinery == Some(true)
        || obsolete_unloading(w, k)
    {
        return false;
    }
    let c = w.reference_point(k);
    // spec: S-0314 claim 17 (the vehicle occupies the cell containing the depot's foundation centre)
    if w.obj(id).is_none_or(|o| o.pos.cell() != c.cell()) {
        return false;
    }
    if !moving_destination(w, id) {
        return true;
    }
    // spec: S-0314 claim 17 (or the vehicle in contact place 0 is more than 128 leptons from that centre: its current world position with height, the floating-point Euclidean distance truncated toward zero; an empty place 0 fails, with no substitute position)
    contacts(w, k)
        .into_iter()
        .find(|&(p, _)| p == 0)
        .and_then(|(_, v)| w.obj(v))
        .is_some_and(|v| crate::naval::depot_distance(v.pos, c) > PAD_PLACE0)
}

/// Whether the vehicle stands in the docking-point cell of depot `k` at a
/// vertical difference strictly below 208 leptons (S-0314 claim 18).
fn at_dock(w: &World, k: ObjectId, id: ObjectId) -> bool {
    let dock = docking_point(w, k, id);
    w.obj(id).is_some_and(|o| {
        // spec: S-0314 claim 18 (being in the docking-point cell with a vertical difference strictly below 208 leptons stops the approach)
        o.pos.cell() == dock.cell() && (o.pos.z - dock.z).abs() < DOCK_HEIGHT
    })
}

/// The movement-arrival exchange (S-0314 claim 17): an entering vehicle
/// in depot contact that reaches the supplied docking-point cell during
/// movement makes the depot queue *repair* and itself queue *sleep*,
/// tethered or not. Called by the locomotors at a reached track target or
/// heading point.
pub fn movement_arrival(w: &mut World, id: ObjectId) {
    let Some(l) = link(w, id) else { return };
    let k = l.depot;
    let Some(o) = w.obj(id) else { return };
    if o.kind != Kind::Vehicle
        || o.beh.current != Some(Behaviour::Enter)
        || o.dest != Some(Target::Object(k))
        || !at_dock(w, k, id)
    {
        return;
    }
    // spec: S-0314 claim 17 and S-0215 claim 29 (reaching the supplied docking-point cell during movement sets the depot's ready mark and queues repair there and sleep at the vehicle, neither with an immediate start; no exact horizontal agreement and no tether required, and no tether created)
    if let Some(b) = w.obj_mut(k) {
        b.beh.queue(Some(Behaviour::Repair));
        b.beh.ready_mark = true;
    }
    if let Some(ob) = w.obj_mut(id) {
        ob.beh.queue(Some(Behaviour::Sleep));
        if let Some(l) = ob.depot.link.as_mut() {
            // spec: S-0314 claim 22 (revision 107: destruction tells a contacted vehicle before its movement arrival in the docking-point cell from one after it, docked or not)
            l.arrived = true;
        }
    }
    // spec: S-0314 Terms and claim 21 (the movement arrival in the docking-point cell makes the vehicle settled, whatever its state before; a hovering vehicle is switched off, with no draws since its behaviour is then enter)
    settle(w, id);
    // spec: S-0314 claim 17 (revision 29, Q-1162 answered: arrival itself requests no map-east body turn, even for a tethered vehicle, and schedules none for a later enter processing; an existing driving turn holds sleep only under S-0215 claim 11's ordinary readiness test)
}

/// The processing delay *D* of a depot (S-0314 F1): the rate delay of
/// *repair*, without a draw, counted as 1 when it is 1 or less.
fn delay_d(w: &World) -> i32 {
    // spec: S-0314 F1 (D = the rate delay of the repair behaviour, S-0215 F1, without a random draw; 1 or less counts as 1)
    w.rules().behaviour(Behaviour::Repair).rate_delay.max(1)
}

/// The vehicle repair count *K*<sub>u</sub> (S-0314 F1).
fn count_k(w: &World) -> i32 {
    // spec: S-0314 F1 (K_u from the vehicle repair rate)
    w.rules().combat.general.repair_count_vehicle.max(1)
}

/// The serviced vehicle: the one in the lowest-numbered occupied place
/// (S-0314 Edge cases, several contact places, our departure; the
/// original services place 0).
fn serviced(w: &World, k: ObjectId) -> Option<ObjectId> {
    // spec: S-0314 Edge cases (several contact places: our engine services the vehicle in the lowest-numbered occupied place)
    contacts(w, k).first().map(|&(_, v)| v)
}

/// The locomotor's movement report alone (S-0314 Terms, *moving
/// destination*, and claim 21): the moving-destination test without its
/// order-level destination. What decides, within 200 leptons of the
/// docking point, whether a vehicle that is not settled, or hovering not
/// down, stands still.
fn movement_report(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if o.hover.is_some() {
        // spec: S-0314 claims 17 and 21 (hovering: S-0503 claim 5's is moving)
        return crate::hover::moving(w, id);
    }
    if let Some(d) = o.drive.as_ref() {
        // spec: S-0314 claims 17 and 21 (driving or ship: the locomotor destination point present, or the heading point present and differing from the current horizontal position)
        let heading = d
            .track
            .as_ref()
            .is_some_and(|t| t.target != (o.pos.x, o.pos.y));
        return crate::drive::goal_present(w, id) || heading;
    }
    // Other locomotors: their own movement test.
    crate::occupancy::in_progress(w, id)
}

/// Whether a hovering vehicle is down on the ground (S-0314 Terms: S-0503
/// claim 10's coming down to height 0); a vehicle that does not hover
/// counts as down.
fn down(w: &World, id: ObjectId) -> bool {
    !crate::hover::is_hover(w, id) || crate::hover::hover_height(w, id) <= 0
}

/// A depot's *repair* processing (S-0314 claims 6, 21 and 22; S-0504 claim
/// 8). Returns the delay, by claim 6's processing spacing table.
pub fn process_repair(w: &mut World, k: ObjectId) -> i32 {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let t = w.tick;
    let d = delay_d(w);
    let ku = count_k(w);
    if w.obj(k)
        .is_some_and(|b| b.beh.seen() == Some(Behaviour::Selling))
    {
        return d;
    }
    if w.obj(k).is_some_and(|b| b.beh.progress == phase::LEAVING) {
        // spec: S-0314 claim 6 (revision 107, the waiting row with no contact: the depot is in guard from the tick its next processing falls due, whether or not special-3 is still running); measure: M-0007 (a loss before the first waiting processing)
        if let Some(b) = w.obj_mut(k) {
            b.depot.serviced = None;
            b.beh.queue_and_start_at_once(Behaviour::Guard, t, stale);
        }
        return 1;
    }
    let Some(v) = serviced(w, k) else {
        // spec: S-0314 claims 6 and 22 (a lost contact is met at the processing already scheduled: the loss neither brings it forward nor puts it back; the working phase is processed every tick)
        let stage = w.obj(k).map_or(phase::WAITING, |b| b.beh.progress);
        // spec: S-0108 claim 20 (no longer in contact in waiting or working, or in settling with production or special-2 running: production and special-2 stop, special-3 and active-1 start); S-0314 claim 22 (after a send-away the depot is already in settling, so that processing follows the settling row)
        let ended = match stage {
            phase::SETTLING => crate::depot_anim::production_or_special2(w, k),
            _ => true,
        };
        if ended {
            crate::depot_anim::service_ended(w, k);
        }
        if stage == phase::WORKING {
            // spec: S-0314 claim 22 (revision 31: a working depot left without any contact enters settling and rechecks after 1 tick; its serviced vehicle is kept until the return to guard, so a new contact meanwhile restarts the phases at waiting, Edge cases)
            if let Some(b) = w.obj_mut(k) {
                b.beh.progress = phase::SETTLING;
            }
            return 1;
        }
        if stage == phase::SETTLING && slot_running(w, k, DepotSlot::Special3) {
            // spec: S-0314 claims 6 and 22 (settling, no contact, special-3 running after this processing's slot changes: the no-contact settling hold, 1 tick later)
            return 1;
        }
        if stage == phase::WAITING {
            // spec: S-0314 claim 6 (revision 107, waiting, no contact: this processing sets the next one 1 tick later, and the depot is in guard from that tick, whether or not special-3 is still running); claim 22; measure: M-0007 (a loss before the first waiting processing: guard one tick after it)
            if let Some(b) = w.obj_mut(k) {
                b.beh.progress = phase::LEAVING;
            }
            return 1;
        }
        // spec: S-0314 claims 6 and 22 (settling with no special-3 running: a depot left without any contact returns to guard at that processing, without an animation wait)
        if let Some(b) = w.obj_mut(k) {
            b.depot.serviced = None;
            b.beh.queue_and_start_at_once(Behaviour::Guard, t, stale);
        }
        return 1;
    };
    if w.obj(k).is_some_and(|b| b.depot.serviced != Some(v))
        && let Some(b) = w.obj_mut(k)
    {
        // spec: S-0314 Edge cases (several contact places: when the serviced place changes, the phases restart at waiting)
        if b.depot.serviced.is_some() {
            b.beh.progress = phase::WAITING;
        }
        b.depot.serviced = Some(v);
    }
    let dock = docking_point(w, k, v);
    let stage = w.obj(k).map_or(phase::WAITING, |b| b.beh.progress);
    let set = |w: &mut World, s: i32| {
        if let Some(b) = w.obj_mut(k) {
            b.beh.progress = s;
        }
    };
    match stage {
        phase::WAITING => {
            // spec: S-0314 claim 6 and S-0504 claim 8 (waiting: standing still strictly within the starting distance of the docking point schedules settling 3 ticks later; otherwise D); claim 21 (the depot's question whether it is moving is the moving-destination test)
            if crate::naval::waiting_gate(w, v, dock) {
                set(w, phase::SETTLING);
                crate::naval::SETTLING_DELAY
            } else {
                // spec: S-0314 Terms and claim 21 (a waiting processing that does not accept the vehicle ends its being settled)
                unsettle(w, v);
                d
            }
        }
        phase::SETTLING => {
            if slot_running(w, k, DepotSlot::Production) {
                // spec: S-0314 claims 6 and 21 (while the depot's production slot animation runs, settling takes no repair step, does not make the vehicle settled whatever its distance, and rechecks after 1 tick)
                return 1;
            }
            let near = w
                .obj(v)
                .is_some_and(|o| crate::naval::depot_distance(o.pos, dock) < SETTLING_HOLD);
            if near {
                if !(settled(w, v) && down(w, v)) {
                    // spec: S-0314 Terms and claim 21 (within 200 leptons, not settled or hovering not down: no step and a recheck after 1 tick, moving or not; whether it stands still is its locomotor's movement report alone; standing still, this processing makes it settled, and a settled hovering vehicle not yet down is switched off again)
                    if !movement_report(w, v) {
                        settle(w, v);
                    }
                    return 1;
                }
                if let Some(o) = w.obj_mut(v) {
                    // spec: S-0314 Terms and claim 21 (a settled vehicle within 200 leptons, down if it hovers, takes its settling step at this processing, its order-level destination empty from then on; the removal is no receipt of a destination, neither stops its locomotor nor switches it on, and does not end its being settled; Open questions: so it counts as not moving)
                    o.dest = None;
                }
            } else if moving_destination(w, v) {
                // spec: S-0314 claim 21 (settling, 200 leptons or more and moving by the moving-destination test: no step, settling and contact kept, the vehicle stops being settled, and checked again after D ticks)
                unsettle(w, v);
                return d;
            }
            // spec: S-0314 claim 21 (at 200 leptons or more, being settled and being down play no part: a stationary vehicle proceeds to the settling repair step)
            match heal::repair_step(w, v) {
                // spec: S-0314 claim 6 (settling, not needed: sent away at once, no announcement, also for the retries after a failed departure; sent away or not, the next processing comes D ticks later)
                Some(StepOutcome::NotNeeded) | None => {
                    let _ = send_away(w, k, v);
                    d
                }
                // spec: S-0314 claim 6 (settling, cannot pay: asks again at its next processing, D ticks later)
                Some(StepOutcome::CannotPay) => d,
                // spec: S-0314 claim 6 (settling, more needed or done: working; the first working processing D ticks later, the second step falling due D + K_u − 1 ticks later)
                Some(StepOutcome::MoreNeeded | StepOutcome::Done) => {
                    set(w, phase::WORKING);
                    if let Some(b) = w.obj_mut(k) {
                        b.depot.due = t.wrapping_add(d).wrapping_add(ku).wrapping_sub(1);
                    }
                    // spec: S-0108 claim 20 (entering the working phase: special-1 starts, active-1 and idle stop)
                    crate::depot_anim::working_entered(w, k);
                    d
                }
            }
        }
        _ => {
            // spec: S-0314 claim 6 (working, every outcome: the next processing 1 tick later; the step spacing comes from K_u, not from the processings)
            if w.obj(k).is_some_and(|b| t < b.depot.due) {
                return 1;
            }
            if moving_destination(w, v) {
                // spec: S-0314 claims 6 and 21 (working: only movement defers a step; a step that falls due while the vehicle is moving is taken on the first tick it stands still)
                return 1;
            }
            // spec: S-0314 claim 21 and Edge cases (working: no distance test; a docked vehicle standing still anywhere receives its steps, quirks.depot_remote_repair)
            match heal::repair_step(w, v) {
                // spec: S-0314 claim 6 (more needed: the vehicle stays; the next step K_u ticks later)
                Some(StepOutcome::MoreNeeded) => {
                    if let Some(b) = w.obj_mut(k) {
                        b.depot.due = t.wrapping_add(ku);
                    }
                    1
                }
                // spec: S-0314 claim 6 (done or not needed: the repair-complete announcement for the depot's owner, attempted even when the departure then fails; the depot is back in settling on the tick of the step, whether or not the vehicle is sent away, and its next processing comes 1 tick later)
                Some(StepOutcome::Done | StepOutcome::NotNeeded) | None => {
                    announce(w, k, v, crate::combat::effects::Sound::UnitRepaired);
                    // spec: S-0108 claim 20 (a working step answers done or not needed: the slot changes come on the tick of that step, before the contact is released)
                    crate::depot_anim::service_ended(w, k);
                    set(w, phase::SETTLING);
                    let _ = send_away(w, k, v);
                    1
                }
                Some(StepOutcome::CannotPay) => {
                    let owner = w.obj(k).and_then(|b| b.owner);
                    // spec: S-0314 claim 6 (the depot's owner hears the low-funds announcement on leaving working when its available credits are exactly 0)
                    if owner.is_some_and(|h| crate::seams::available_credits(w, h) == 0) {
                        announce(w, k, v, crate::combat::effects::Sound::LowFunds);
                    }
                    // spec: S-0108 claim 20 (a working step answers cannot pay: production and special-2 stop, special-3 and active-1 start)
                    crate::depot_anim::service_ended(w, k);
                    // spec: S-0314 claim 6 (cannot pay: contact kept, back to settling, first retry on the next tick)
                    set(w, phase::SETTLING);
                    1
                }
            }
        }
    }
}

/// A presentation event of the depot's owner (S-0314 claim 6): the
/// repair-complete announcement (its *unit repaired* radar event, S-0701
/// claim 28) or the low-funds announcement.
fn announce(w: &mut World, k: ObjectId, v: ObjectId, sound: crate::combat::effects::Sound) {
    let of = w.obj(v).map(|o| crate::econ::TypeRef {
        kind: o.kind,
        index: o.type_index,
    });
    let house = w.obj(k).and_then(|b| b.owner);
    let pos = Some(w.reference_point(k));
    w.effects.push(crate::combat::effects::Effect::Sound {
        sound,
        of,
        house,
        pos,
    });
}

/// Sends vehicle `v` away from depot `k` (S-0314 claim 6): it moves to
/// the rally point, else to claim 25's exit cell; a vehicle of a house that
/// is not human-controlled with a guard spot moves to the guard spot, which
/// is cleared. With a destination, contact ends; `false` when there is no
/// destination, in which case nothing is ordered and contact is kept.
fn send_away(w: &mut World, k: ObjectId, v: ObjectId) -> bool {
    let Some(o) = w.obj(v).cloned() else {
        return true;
    };
    // spec: S-0314 claim 6 (the guard-spot return holds for the neutral and special houses only; no player house takes it, except a test-only computer-controlled one, glossary)
    let human = o
        .owner
        .and_then(|h| w.houses.get(usize::from(h.0)))
        .is_none_or(|h| h.human);
    let rally = w.obj(k).and_then(|b| b.guard_spot);
    // spec: S-0314 claim 6 (exit selection occurs before the rally-point override; it takes no draw, so only its result matters)
    let exit = exit_cell(w, k, v).map(Target::Cell);
    let dest = if !human && o.guard_spot.is_some() {
        // spec: S-0314 claim 6 (a vehicle of the neutral or special house with a guard spot moves to it, and the spot is cleared)
        if let Some(ob) = w.obj_mut(v) {
            ob.guard_spot = None;
        }
        o.guard_spot
    } else if rally.is_some() {
        // spec: S-0314 claim 6 (the depot's rally point when it has one, S-0213 claim 20, not subjected to the exit-cell suitability test)
        rally
    } else {
        // spec: S-0314 claims 6 and 25 (otherwise the exit cell selected by claim 25)
        exit
    };
    let Some(dest) = dest else {
        // spec: S-0314 claims 6 and 25 (with no selected cell and no rally point, no departure is ordered and contact is retained; no widening fallback, scatter or forced release)
        return false;
    };
    // spec: S-0314 claim 6 (a selected destination releases contact and tether and clears queued entry)
    release(w, v);
    // spec: S-0314 claims 6 and 21 (the destination the depot gives replaces any destination the player gave while docked)
    crate::econ::assign_destination(w, v, Some(dest));
    if let Some(ob) = w.obj_mut(v) {
        ob.path.steps.clear();
        // spec: S-0212 claim 21 and its state table, row "destination changes" (r97: for the driving and walk locomotors a different destination, or none, ends a running retry delay, path::destination_changed and path::waits; the ship and the other locomotors keep it, low confidence; the hovering locomotor's is S-0503's)
        // spec: S-0314 claim 6 (a selected destination queues move without immediate start)
        ob.beh.queue(Some(Behaviour::Move));
    }
    true
}

/// Claim 25's candidate cells of the repair departure, in preference order,
/// each with whether the locomotion-specific entry test also applies.
fn exit_candidates(w: &World, k: ObjectId, v: ObjectId) -> Vec<(CellXY, bool)> {
    let Some(b) = w.obj(k) else {
        return Vec::new();
    };
    let Some(bt) = w.rules().buildings.get(b.type_index as usize) else {
        return Vec::new();
    };
    let (x0, y0) = (b.pos.cell().x, b.pos.cell().y);
    let (fw, fh) = b.foundation;
    // spec: S-0314 claim 25 (integer cells with signed 16-bit wrapping, S-0205 claim 1)
    let cell = |a: i32, bb: i32| {
        CellXY::new(
            i32::from(x0.wrapping_add(a) as i16),
            i32::from(y0.wrapping_add(bb) as i16),
        )
    };
    let mut out = Vec::new();
    let e = &bt.raw.econ;
    // spec: S-0314 claim 25 (door kinds: (x0 + 1, y0 + 2), (x0 + 2, y0 + 2), (x0 + 2, y0 + 1) for the first, second and third kinds the type has; with the locomotion test)
    for (has, (a, bb)) in [
        (e.door_first, (1, 2)),
        (e.door_second, (2, 2)),
        (e.door_third, (2, 1)),
    ] {
        if has == Some(true) {
            out.push((cell(a, bb), true));
        }
    }
    if bt.naval && bt.factory == Some(crate::rules::FactoryKind::Vehicle) {
        // spec: S-0314 claim 25 (a naval vehicle factory: (qx, qy + 1), (qx, qy), (qx − 1, qy + 1) from the cell of the supplied docking point, claim 18; no locomotion test)
        let q = docking_point(w, k, v).cell();
        for (a, bb) in [(0, 1), (0, 0), (-1, 1)] {
            out.push((
                CellXY::new(
                    i32::from(q.x.wrapping_add(a) as i16),
                    i32::from(q.y.wrapping_add(bb) as i16),
                ),
                false,
            ));
        }
    }
    let hospital = bt.raw.hospital == Some(true);
    let refinery = bt.raw.foundation_refinery == Some(true);
    let exact = w.rules().quirks.foundation_exit_lists;
    let list = crate::econ::release::present_exit_list(fw, fh, refinery, exact);
    match list {
        // spec: S-0314 claim 25 (S-0404's foundation exit list, when present and the type is not a hospital, in its order with omissions and repeats; no locomotion test; exhausting it gives no cell)
        Some(l) if !hospital => {
            out.extend(l.into_iter().map(|(a, bb)| (cell(a, bb), false)));
        }
        _ => {
            // spec: S-0314 claim 25 (otherwise the perimeter: for i from −1 to W, (x0 + i, y0 + H) then (x0 + i, y0 − 1); then for j from −1 to H, (x0 + W, y0 + j) then (x0 − 1, y0 + j); corner repeats intended; with the locomotion test)
            for i in -1..=fw {
                out.push((cell(i, fh), true));
                out.push((cell(i, -1), true));
            }
            for j in -1..=fh {
                out.push((cell(fw, j), true));
                out.push((cell(-1, j), true));
            }
        }
    }
    out
}

/// The exit cell of a vehicle sent away (S-0314 claim 25): the first
/// candidate that is a legal map cell giving *free* from the vehicle's
/// cell-entry test. No draw, no route or region test.
fn exit_cell(w: &World, k: ObjectId, v: ObjectId) -> Option<CellXY> {
    let reference = crate::occupancy::reference_level(w, v);
    exit_candidates(w, k, v)
        .into_iter()
        .find(|&(c, _loco)| {
            // spec: S-0314 claim 25 (a legal map cell giving free from the vehicle's cell-entry test, S-0209 claim 5 and S-0211; every non-free result rejects it; the locomotion-specific test of the door and perimeter rows always gives free for driving, hovering and ship locomotion)
            w.cells.footprint().is_legal(c)
                && crate::occupancy::vehicle_outcome(w, v, c, reference)
                    == crate::occupancy::Outcome::Clear
        })
        .map(|(c, _)| c)
}

/// The early start of an unarmed depot in its guard processing (S-0314
/// claim 17; S-0215 claim 29): a contacted vehicle still in *enter*,
/// standing still strictly within 64 leptons of the foundation centre,
/// makes the depot queue *repair* with a 1-tick delay. `Some(1)` when it
/// applies.
pub fn early_start(w: &mut World, k: ObjectId) -> Option<i32> {
    if !is_depot(w, k) {
        return None;
    }
    let c = w.reference_point(k);
    let hit = contacts(w, k).into_iter().any(|(_, v)| {
        w.obj(v).is_some_and(|o| {
            o.beh.current == Some(Behaviour::Enter)
                && still(w, v)
                // spec: S-0314 claim 17 (the floating-point Euclidean distance truncated to integer leptons, to the foundation centre)
                && crate::naval::depot_distance(o.pos, c) < EARLY_START
        })
    });
    if !hit {
        return None;
    }
    if let Some(b) = w.obj_mut(k) {
        // spec: S-0314 claim 17 (the depot queues repair with a 1-tick processing delay; it sets no ready mark, so the start follows S-0215 claim 11)
        b.beh.queue(Some(Behaviour::Repair));
    }
    Some(1)
}

/// The per-tick queued-entry retry of a vehicle (S-0314 claim 23), in its
/// own turn after its locomotor movement.
pub fn vehicle_turn(w: &mut World, id: ObjectId) {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let t = w.tick;
    let Some(o) = w.obj(id) else { return };
    if let Some(l) = o.depot.link
        && o.contact != Some(l.depot)
    {
        // A link whose contact has ended is no longer held.
        if let Some(ob) = w.obj_mut(id) {
            ob.depot.link = None;
        }
        if !w.rules().quirks.depot_loss_strands_hover {
            // spec: S-0314 Edge cases and Terms (quirks.depot_loss_strands_hover off: losing contact with a depot ends being settled, and the vehicle is switched on again); design: a contact broken off outside this module (a deploy order's clear, S-0213 claim 10; an owner change's leave, S-0509 claim 14) is seen here, at the vehicle's next turn
            unsettle(w, id);
        }
    }
    let Some(k) = w.obj(id).and_then(|o| o.depot.queued) else {
        return;
    };
    if w.obj(k).is_none_or(|b| b.health <= 0 || !b.on_map) {
        // spec: S-0314 claim 23 (an inactive remembered depot clears the queued entry); our engine forgets the remembered depot with it, whose request would be refused anyway
        if let Some(ob) = w.obj_mut(id) {
            ob.depot.queued = None;
            ob.depot.entering = None;
        }
        return;
    }
    // spec: S-0314 claim 23 (once per eligible vehicle turn it asks again for a contact place: the alliance and free-place tests; while none is free it keeps waiting; a place its own processing took this turn is already held)
    let Some(place) = alliance_and_place(w, k, id) else {
        return;
    };
    take(w, id, k, place);
    // spec: S-0314 claim 23 (with a place it asks permission)
    if accepts(w, k, id, true) {
        // spec: S-0314 claim 23 (granted: it queues enter with immediate start, S-0215 claim 7, making it current in the same turn when ready, without processing it now; the depot becomes its destination and queued entry is cleared)
        let ready = crate::process::ready(w, id);
        if let Some(ob) = w.obj_mut(id) {
            ob.depot.queued = None;
            ob.beh
                .queue_and_start(Some(Behaviour::Enter), ready, t, stale);
        }
        crate::econ::assign_destination(w, id, Some(Target::Object(k)));
        if let Some(ob) = w.obj_mut(id) {
            ob.path.steps.clear();
            // spec: S-0212 claim 21 and its state table, row "destination changes" (r97: for the driving and walk locomotors a different destination, or none, ends a running retry delay, path::destination_changed and path::waits; the ship and the other locomotors keep it, low confidence; the hovering locomotor's is S-0503's)
        }
    } else {
        // spec: S-0314 claims 20 and 23 (refused: contact, queued entry and destination cleared; it falls back to rest)
        let standing = standing_on(w, k, id);
        release(w, id);
        crate::econ::assign_destination(w, id, None);
        if !standing {
            crate::process::fall_back(w, id, false);
        }
        // spec: S-0314 claim 20 (revision 28: refused for standing on the depot, claim 16, it queues no behaviour, and its enter falls back to rest at its next processing)
    }
}

/// What an order does after [`order_applied`] (S-0314 claims 20 and 23).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrderEffect {
    /// The order runs normally.
    Ordinary,
    /// The order ends here: no behaviour, no destination, no other effect.
    Ends,
    /// The order runs normally but gives no destination (claim 23's
    /// queued entry: *enter* queued without an immediate start).
    NoDestination,
}

/// What applying an order does to a vehicle's depot contact before the
/// order's ordinary effects (S-0314 claims 20 and 23; S-0504 claim 14).
pub fn order_applied(
    w: &mut World,
    id: ObjectId,
    b: Behaviour,
    dest: Option<Target>,
) -> OrderEffect {
    let Some(o) = w.obj(id).cloned() else {
        return OrderEffect::Ordinary;
    };
    if o.kind != Kind::Vehicle {
        return OrderEffect::Ordinary;
    }
    // spec: S-0314 claim 23 (any later order clears the queued entry); the remembered depot is replaced by the order's own entry object (none unless claim 23's order below names one)
    if let Some(ob) = w.obj_mut(id) {
        ob.depot.queued = None;
        ob.depot.entering = None;
    }
    let l = link(w, id);
    let target_depot = match (b, dest) {
        (Behaviour::Enter, Some(Target::Object(k))) if is_depot(w, k) => Some(k),
        _ => None,
    };
    let full = health_full(w, id);
    if let Some(l) = l
        && l.tethered
    {
        if target_depot == Some(l.depot) {
            // spec: S-0314 claim 20 (an enter order on the same depot ends the docking: a full-health vehicle loses its contact at once)
            if full {
                release(w, id);
                return OrderEffect::Ends;
            }
            if let Some(ob) = w.obj_mut(id) {
                // spec: S-0314 claim 20 (revision 28, Q-1158 answered: a damaged vehicle takes claim 23's queued entry in full although the depot holds its own contact: enter queued without an immediate start, the depot remembered, no destination given; the tether keeps the contact, S-0213 table T3 (b))
                ob.depot.queued = Some(l.depot);
                ob.depot.entering = Some(l.depot);
            }
            return OrderEffect::NoDestination;
        }
        if !w.rules().quirks.depot_remote_repair && matches!(b, Behaviour::Move | Behaviour::Attack)
        {
            // spec: S-0314 Edge cases (quirks.depot_remote_repair off: a docked vehicle given a move or attack order loses its contact as an approaching one does)
            release(w, id);
        }
        // spec: S-0314 claim 20 (otherwise a docked vehicle keeps contact and tether; the order runs normally)
        return OrderEffect::Ordinary;
    }
    if let Some(l) = l
        && target_depot == Some(l.depot)
        && o.beh.seen() == Some(Behaviour::Enter)
    {
        // spec: S-0314 claim 20 (an enter order on the same depot while approaching in enter is ignored entirely)
        return OrderEffect::Ends;
    }
    if l.is_some() {
        // spec: S-0314 claim 20 (an approaching vehicle given any other order loses its contact at once; its queued entry is cleared too)
        release(w, id);
    }
    let Some(k) = target_depot else {
        return OrderEffect::Ordinary;
    };
    if !contacts(w, k).is_empty() {
        if full {
            // spec: S-0314 claim 23 (a full-health vehicle gets no destination and no queued entry: the order ends, and the vehicle stays put)
            return OrderEffect::Ends;
        }
        if let Some(ob) = w.obj_mut(id) {
            // spec: S-0314 claim 23 (a later enter order on a depot that holds any contact: a damaged vehicle gets no depot destination and no contact; the order queues enter without an immediate start, S-0215 claim 12, records the depot as the object it is entering and gives it a queued entry for that depot)
            ob.depot.queued = Some(k);
            ob.depot.entering = Some(k);
        }
        return OrderEffect::NoDestination;
    }
    OrderEffect::Ordinary
}

/// A stop order applied to vehicle `id` (S-0314 claims 20 and 23): an
/// approaching vehicle loses its contact and any queued entry is cleared.
/// A docked vehicle ignores stop (S-0213 claim 8).
pub fn stop_order(w: &mut World, id: ObjectId) {
    if w.obj(id)
        .is_some_and(|o| o.depot.queued.is_some() || o.depot.link.is_some())
    {
        release(w, id);
    }
}

/// After an enter order's ordinary effects (S-0314 claim 23): with an
/// enter order on a depot that holds no contact, the vehicle asks for
/// entry at once; if refused, it takes no contact and does not queue.
pub fn order_entered(w: &mut World, id: ObjectId, b: Behaviour, dest: Option<Target>) {
    if b != Behaviour::Enter {
        return;
    }
    let Some(Target::Object(k)) = dest else {
        return;
    };
    if !is_depot(w, k) || w.obj(id).is_none_or(|o| o.kind != Kind::Vehicle) {
        return;
    }
    // spec: S-0314 claim 23 (an enter order on a depot that holds no contact asks for entry at once)
    let _ = request(w, id, k, true);
}

/// Whether an object is at full health (S-0314 Terms: a health ratio of
/// exactly 1).
fn health_full(w: &World, id: ObjectId) -> bool {
    let h = w.obj(id).map_or(0, |o| o.health);
    h >= crate::combat::damage::strength(w, id)
}

/// The run-away notice of a depot that starts selling (S-0314 claim 22).
pub fn sale_started(w: &mut World, k: ObjectId) {
    if !is_depot(w, k) {
        return;
    }
    for (_, v) in contacts(w, k) {
        run_away(w, k, v);
    }
}

/// The run-away notice to vehicle `v` (S-0314 claim 22).
fn run_away(w: &mut World, k: ObjectId, v: ObjectId) {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let t = w.tick;
    let Some(o) = w.obj(v).cloned() else { return };
    if o.dest == Some(Target::Object(k)) {
        // spec: S-0314 claim 22 (it clears its destination if that destination is the depot)
        crate::econ::assign_destination(w, v, None);
    }
    if let Some(ob) = w.obj_mut(v) {
        match ob.beh.current {
            // spec: S-0314 claim 22 (a vehicle in sleep switches to guard at once)
            Some(Behaviour::Sleep) => ob.beh.queue_and_start_at_once(Behaviour::Guard, t, stale),
            // spec: S-0314 claim 22 (a vehicle in enter switches to guard when its current processing ends)
            Some(Behaviour::Enter) => {
                ob.beh.queue(Some(Behaviour::Guard));
            }
            _ => {}
        }
    }
    if w.obj(v).is_some_and(|x| x.dest.is_none()) {
        // spec: S-0314 claim 22 (a vehicle with no destination left can be sent a scatter request, which S-0215 claim 19 accepts or refuses)
        crate::scatter::request(
            w,
            v,
            crate::scatter::ScatterRequest {
                threat: None,
                forced: false,
                anywhere: false,
            },
        );
    }
}

/// A selling depot's processing releases every contact (S-0314 claim 22).
pub fn selling_processing(w: &mut World, k: ObjectId) {
    if !is_depot(w, k) {
        return;
    }
    for (_, v) in contacts(w, k) {
        // spec: S-0314 claim 22 (at its next processing the selling depot releases every contact)
        release(w, v);
    }
}

/// A destroyed depot's contact `v` that survives (S-0314 claim 22; the
/// contacts within S-0312 claim 17's distance are destroyed): contact is
/// released without a run-away notice; a vehicle before its movement
/// arrival in the docking-point cell loses its target and falls back to
/// guard where it stands; one after that arrival, docked or not, keeps its
/// behaviour (`quirks.depot_loss_leaves_sleep`).
pub fn depot_destroyed(w: &mut World, k: ObjectId, v: ObjectId) {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let t = w.tick;
    let Some(l) = link(w, v).filter(|l| l.depot == k) else {
        return;
    };
    if l.arrived && !w.rules().quirks.depot_loss_leaves_sleep {
        // spec: S-0314 Edge cases (quirks.depot_loss_leaves_sleep off: destruction treats contacts as a sale does; revision 107: the switch covers every behaviour an arrived vehicle keeps)
        run_away(w, k, v);
        release(w, v);
        return;
    }
    release(w, v);
    if l.arrived {
        // spec: S-0314 claim 22 (revision 107: the release leaves the current behaviour of a contacted vehicle after its movement arrival as it was, docked or not: sleep stays sleep until an order, guard stays guard; quirks.depot_loss_leaves_sleep)
        return;
    }
    if let Some(ob) = w.obj_mut(v) {
        // spec: S-0314 claim 22 (a contacted vehicle before its movement arrival in the docking-point cell loses its target and falls back to guard where it stands, tethered or not)
        crate::spawn::clear_own_target(ob);
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
        ob.beh.queue_and_start_at_once(Behaviour::Guard, t, stale);
    }
    crate::econ::assign_destination(w, v, None);
    if let Some(ob) = w.obj_mut(v) {
        ob.path.steps.clear();
    }
}

/// Whether a building whose type meets `kind` occupies vehicle `id`'s cell
/// (a building occupies its foundation, glossary *building*).
fn occupied_by(
    w: &World,
    id: ObjectId,
    kind: impl Fn(&crate::rules::BuildingRules) -> bool,
) -> bool {
    let Some(o) = w.obj(id) else { return false };
    w.cells
        .list(o.pos.cell(), crate::world::Layer::Ground)
        .iter()
        .any(|&x| {
            w.obj(x).is_some_and(|b| {
                b.kind == Kind::Building
                    && w.rules()
                        .buildings
                        .get(b.type_index as usize)
                        .is_some_and(&kind)
            })
        })
}

/// Whether vehicle `id`'s locomotor counts as switched off for an end of
/// being settled (S-0314 Terms, *What ends it*): always for a vehicle that
/// does not hover; for a hovering one only once it is down, since above the
/// ground it is powered in S-0503's sense.
fn counts_switched_off(w: &World, id: ObjectId) -> bool {
    // spec: S-0314 Terms (for a vehicle that does not hover, every moment while it is settled; a hovering vehicle counts as switched off only once it is down: while it is still above the ground it is powered in S-0503's sense, S-0503 Terms)
    // design: S-0314 Terms (a settled hover switched on again with no end of being settled, as by S-0606's bunker release, is powered in S-0503's sense, so it does not count as switched off either)
    !crate::hover::is_hover(w, id) || !crate::hover::powered(w, id)
}

/// A destination assignment of vehicle `id` (S-0314 Terms, claim 22): one
/// that takes effect (`effective`: S-0207 claim 27) while a repair depot or
/// a tank bunker occupies the vehicle's cell, and while its locomotor
/// counts as switched off, ends its being settled, a cleared destination
/// included. This is also the end at a disabling's guard reset.
pub fn destination_given(w: &mut World, id: ObjectId, effective: bool) {
    // spec: S-0314 Terms (only an assignment that takes effect counts: S-0207 claim 27 rejects one that leaves the destination unchanged, empty to empty included, unless a destination-refresh state is present; a rejected assignment is not a receipt)
    if !effective || !settled(w, id) {
        return;
    }
    // spec: S-0314 Terms and claim 22 (only by a vehicle whose locomotor then counts as switched off: a destination a hovering vehicle receives while it is still above the ground does not end its being settled, wherever it is; the same condition for the guard reset of a disabling)
    if !counts_switched_off(w, id) {
        return;
    }
    // spec: S-0314 Terms and claim 22 (a destination received while a depot or tank bunker occupies its cell, an empty one included, ends being settled)
    if occupied_by(w, id, |r| {
        r.building.unit_repair || r.raw.bunker.unwrap_or(false)
    }) {
        // spec: S-0314 Terms and claim 21 (the vehicle is switched on again at the end, also a disabled hovering one in a tank bunker's cell, Away from a depot)
        unsettle(w, id);
    }
}
