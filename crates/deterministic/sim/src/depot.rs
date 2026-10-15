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
//! queued entry, claim 23) and [`DepotState::loco_off`] (claim 22). The
//! depot side is its *repair* behaviour's progress count (the phase) and
//! [`DepotState::serviced`].
//!
//! Interim rules tied to requests: the tether moment and the stopping point
//! (Q-0680), settling while the vehicle moves far away (Q-0681) and the exit
//! cell of a vehicle sent away without a rally point (Q-0682).

use glasnost_map::CellXY;
use serde::{Deserialize, Serialize};

use crate::combat::heal::{self, StepOutcome};
use crate::rules::{Behaviour, FactoryKind, Kind};
use crate::world::{ObjectId, Pos, Target, World};

/// A vehicle's depot contact (S-0314 claim 17): the place it holds and
/// whether the completed entry exchange has tethered it (*docked*) or not
/// yet (*approaching*, claims 20 to 24).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Link {
    /// The depot.
    pub depot: ObjectId,
    /// The contact place, counted from 0.
    pub place: u32,
    /// Tethered: docked.
    pub tethered: bool,
}

/// Depot-cycle state of an object (S-0314 claims 6 and 16 to 24).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepotState {
    /// A vehicle's depot contact; valid only while its contact is the
    /// depot (S-0215 Terms, contact).
    pub link: Option<Link>,
    /// A vehicle's queued entry for a depot (S-0314 claim 23).
    pub queued: Option<ObjectId>,
    /// A vehicle's locomotor switched off by a settling step (S-0314
    /// claims 21 and 22).
    pub loco_off: bool,
    /// A depot's serviced vehicle at its last repair processing (S-0314
    /// Edge cases, several contact places: a change restarts the phases).
    pub serviced: Option<ObjectId>,
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
}

/// The facing a vehicle turns to in the completed entry exchange.
// spec: S-0314 Constants (arrival body facing: 16384 facing units, map east)
const ARRIVAL_FACING: u16 = 16384;

/// The settling hold distance (S-0314 claim 21), strict.
// spec: S-0314 Constants (settling hold distance: 200 leptons, strict, to the docking point)
const SETTLING_HOLD: i32 = 200;

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
/// refusal of a vehicle already standing on the depot (Q-0680 interim: the
/// click and the queued-entry retry ask it; the repeated entry request of
/// an approaching vehicle does not).
pub fn accepts(w: &World, k: ObjectId, id: ObjectId, standing: bool) -> bool {
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
    // spec: S-0314 claim 16 (a vehicle already standing on this very depot receives no new acceptance; Q-0680 interim: the click and the queued retry)
    if standing && standing_on(w, k, id) {
        return false;
    }
    // spec: S-0314 claim 16 (the alliance and contact-place tests; no damage or money gate)
    alliance_and_place(w, k, id).is_some()
}

/// Takes depot contact in place `place` (S-0314 claim 17): a vehicle holds
/// one contact, so any other is released.
fn take(w: &mut World, id: ObjectId, k: ObjectId, place: u32) {
    let keep = link(w, id).is_some_and(|l| l.depot == k && l.tethered);
    if let Some(o) = w.obj_mut(id) {
        o.contact = Some(k);
        o.depot.link = Some(Link {
            depot: k,
            place,
            tethered: keep,
        });
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

/// Releases vehicle `id`'s depot contact and queued entry.
fn release(w: &mut World, id: ObjectId) {
    let strand = w.rules().quirks.depot_loss_strands_hover;
    if let Some(o) = w.obj_mut(id) {
        if o.depot.link.is_some_and(|l| o.contact == Some(l.depot)) {
            o.contact = None;
        }
        o.depot.link = None;
        o.depot.queued = None;
        if !strand {
            // spec: S-0314 Edge cases (quirks.depot_loss_strands_hover off: losing depot contact switches the locomotor on)
            o.depot.loco_off = false;
        }
    }
}

/// Whether `k` is a vehicle factory: S-0314 claim 18 gives such a building
/// S-0504 claim 12's fixed docking point.
fn vehicle_factory(w: &World, k: ObjectId) -> bool {
    w.obj(k)
        .and_then(|b| w.rules().buildings.get(b.type_index as usize))
        .is_some_and(|r| r.factory == Some(FactoryKind::Vehicle))
}

/// The docking point depot `k` supplies for vehicle `id` (S-0314 claim 18;
/// S-0504 claim 12).
pub fn docking_point(w: &World, k: ObjectId, id: ObjectId) -> Pos {
    if vehicle_factory(w, k) {
        // spec: S-0314 claim 18 and S-0504 claim 12 (a vehicle-factory flag takes precedence: the centre of the cell (2, 1) from the top-left cell, at the building's height)
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
    matches!(o.dest, Some(Target::Object(k)) if is_depot(w, k))
}

/// Whether the vehicle stands still: not moving and not turning (S-0504
/// claim 8; S-0314 claim 6).
fn still(w: &World, id: ObjectId) -> bool {
    !crate::occupancy::in_progress(w, id) && w.obj(id).is_some_and(|o| !o.body.turning(w.tick))
}

/// A vehicle's *enter* processing at a repair depot (S-0215 claim 29;
/// S-0314 claims 16 to 18). Returns the delay: every outcome chooses the
/// standard delay (S-0215 claim 29).
pub fn process_enter(w: &mut World, id: ObjectId) -> i32 {
    let t = w.tick;
    let Some(o) = w.obj(id).cloned() else {
        return 1;
    };
    // spec: S-0215 claim 29 (the entry request goes to the contact when present, else to the object it remembers entering)
    let k = match (link(w, id), o.dest) {
        (Some(l), _) => l.depot,
        (None, Some(Target::Object(k))) => k,
        _ => {
            crate::process::fall_back(w, id, false);
            return crate::process::standard_delay(w, id);
        }
    };
    let dock = docking_point(w, k, id);
    let on_dock = standing_on(w, k, id) || o.pos.cell() == dock.cell();
    if link(w, id).is_some_and(|l| l.depot == k && !l.tethered) && on_dock {
        // spec: S-0314 claim 17 (the completed entry exchange tethers the vehicle to the contacted building; Q-0680 interim: once it stands in a foundation cell or the docking point's cell)
        if let Some(ob) = w.obj_mut(id)
            && let Some(l) = ob.depot.link.as_mut()
        {
            l.tethered = true;
        }
    }
    let tethered = tethered(w, id);
    // spec: S-0215 claim 29 (only acceptance or an existing tether permits continued entry; without a tether a non-accepting reply ends contact and the actor falls back to rest)
    if !tethered && !request(w, id, k, false) {
        release(w, id);
        crate::process::fall_back(w, id, false);
        return crate::process::standard_delay(w, id);
    }
    if !tethered {
        // spec: S-0314 claim 18 (the depot directs the vehicle toward the building as its destination; the docking point is the destination position)
        if o.dest != Some(Target::Object(k)) || o.path.steps.is_empty() && !on_dock {
            crate::econ::assign_destination(w, id, Some(Target::Object(k)));
            if let Some(ob) = w.obj_mut(id) {
                ob.path.steps.clear();
                ob.path.retry_at = 0;
            }
        }
        return crate::process::standard_delay(w, id);
    }
    if w.body_facing(id) != ARRIVAL_FACING {
        let rate = w
            .rules()
            .unit(o.kind, o.type_index)
            .map_or(0, |u| u.turn_rate);
        if crate::occupancy::in_progress(w, id) {
            return crate::process::standard_delay(w, id);
        }
        if let Some(ob) = w.obj_mut(id) {
            // spec: S-0314 claim 17 and S-0215 claim 29 (in the completed entry exchange the vehicle turns toward 16384 facing units)
            ob.body.turn_to(ARRIVAL_FACING, t, rate);
        }
        return crate::process::standard_delay(w, id);
    }
    if still(w, id)
        && w.obj(id)
            .is_some_and(|x| x.beh.current == Some(Behaviour::Enter))
    {
        // spec: S-0215 claim 29 and S-0314 claim 17 (no longer moving, tethered and still in enter: it reports arrival; the depot queues repair and sets its ready mark, the vehicle queues sleep, neither with an immediate start)
        if let Some(b) = w.obj_mut(k) {
            b.beh.queue(Some(Behaviour::Repair));
            b.beh.ready_mark = true;
        }
        if let Some(ob) = w.obj_mut(id) {
            ob.beh.queue(Some(Behaviour::Sleep));
        }
    }
    crate::process::standard_delay(w, id)
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

/// A depot's *repair* processing (S-0314 claims 6, 21 and 22; S-0504 claim
/// 8). Returns the delay.
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
    let Some(v) = serviced(w, k) else {
        // spec: S-0314 claim 22 (a depot left without any contact plays its idle animations and returns to guard)
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
            // spec: S-0314 claim 6 and S-0504 claim 8 (waiting: standing still strictly within the starting distance of the docking point schedules settling 3 ticks later; otherwise D)
            if crate::naval::waiting_gate(w, v, dock) {
                set(w, phase::SETTLING);
                crate::naval::SETTLING_DELAY
            } else {
                d
            }
        }
        phase::SETTLING => {
            let near = w
                .obj(v)
                .is_some_and(|o| crate::naval::depot_distance(o.pos, dock) < SETTLING_HOLD);
            if !still(w, v) {
                // spec: S-0314 claim 21 (settling: within 200 leptons and still moving, the depot checks again on the next tick)
                // Q-0681 interim: moving at 200 leptons or more, no step and the delay D.
                return if near { 1 } else { d };
            }
            // spec: S-0314 claim 21 (once it stands still, the depot switches its locomotor off before stepping; a vehicle whose locomotor is already off and which has a destination has its movement stopped)
            let off = w.obj(v).is_some_and(|o| o.depot.loco_off);
            if off && w.obj(v).is_some_and(|o| o.dest.is_some()) {
                crate::drive::stop(w, v);
            }
            if let Some(o) = w.obj_mut(v) {
                o.depot.loco_off = true;
            }
            match heal::repair_step(w, v) {
                // spec: S-0314 claim 6 (settling, not needed: sent away at once, no announcement)
                Some(StepOutcome::NotNeeded) | None => {
                    send_away(w, k, v);
                    1
                }
                // spec: S-0314 claim 6 (settling, cannot pay: asks again at its next processing, D ticks later)
                Some(StepOutcome::CannotPay) => d,
                // spec: S-0314 claim 6 (settling, more needed or done: working; the second step falls due D + K_u − 1 ticks later)
                Some(StepOutcome::MoreNeeded | StepOutcome::Done) => {
                    set(w, phase::WORKING);
                    d.wrapping_add(ku).wrapping_sub(1)
                }
            }
        }
        _ => {
            if !still(w, v) {
                // spec: S-0314 claims 6 and 21 (working: a step that falls due while the vehicle is moving is taken on the first tick it stands still)
                return 1;
            }
            // spec: S-0314 claim 21 and Edge cases (working: no distance test; a docked vehicle standing still anywhere receives its steps, quirks.depot_remote_repair)
            match heal::repair_step(w, v) {
                // spec: S-0314 claim 6 (more needed: the vehicle stays; the next step K_u ticks later)
                Some(StepOutcome::MoreNeeded) => ku,
                // spec: S-0314 claim 6 (done or not needed: the repair-complete announcement for the depot's owner, and the vehicle is sent away)
                Some(StepOutcome::Done | StepOutcome::NotNeeded) | None => {
                    announce(w, k, v, crate::combat::effects::Sound::UnitRepaired);
                    send_away(w, k, v);
                    1
                }
                Some(StepOutcome::CannotPay) => {
                    let owner = w.obj(k).and_then(|b| b.owner);
                    // spec: S-0314 claim 6 (the depot's owner hears the low-funds announcement on leaving working when its available credits are exactly 0)
                    if owner.is_some_and(|h| crate::seams::available_credits(w, h) == 0) {
                        announce(w, k, v, crate::combat::effects::Sound::LowFunds);
                    }
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

/// Sends vehicle `v` away from depot `k` (S-0314 claim 6): contact ends,
/// and it moves to the rally point, else an exit cell next to the depot
/// (Q-0682 interim); a vehicle of a house that is not human-controlled
/// with a guard spot moves to the guard spot, which is cleared.
fn send_away(w: &mut World, k: ObjectId, v: ObjectId) {
    let Some(o) = w.obj(v).cloned() else { return };
    release(w, v);
    if let Some(ob) = w.obj_mut(v) {
        // spec: S-0314 claims 21 and 22 (the locomotor is switched on when the vehicle receives a destination while a depot occupies its cell)
        ob.depot.loco_off = false;
    }
    // spec: S-0314 claim 6 (the guard-spot return holds for the neutral and special houses only; no player house takes it)
    let human = o
        .owner
        .and_then(|h| w.houses.get(usize::from(h.0)))
        .is_none_or(|h| h.role == crate::world::HouseRole::Player);
    let rally = w.obj(k).and_then(|b| b.guard_spot);
    let dest = if !human && o.guard_spot.is_some() {
        // spec: S-0314 claim 6 (a vehicle of the neutral or special house with a guard spot moves to it, and the spot is cleared)
        if let Some(ob) = w.obj_mut(v) {
            ob.guard_spot = None;
        }
        o.guard_spot
    } else if rally.is_some() {
        // spec: S-0314 claim 6 (the depot's rally point when it has one, S-0213 claim 20)
        rally
    } else {
        // spec: S-0314 claim 6 (otherwise an exit cell the depot picks next to itself; Q-0682 interim: the nearby-cell search of S-0212 claim 25, destination substitution, outside the foundation)
        exit_cell(w, k, v).map(Target::Cell)
    };
    // spec: S-0314 claim 21 (the destination the depot gives replaces any destination the player gave while docked)
    crate::econ::assign_destination(w, v, dest);
    if let Some(ob) = w.obj_mut(v) {
        ob.path.steps.clear();
        ob.path.retry_at = 0;
        if dest.is_some() {
            ob.beh.queue(Some(Behaviour::Move));
        } else {
            ob.beh.queue(Some(Behaviour::Guard));
        }
    }
}

/// The exit cell of a vehicle sent away without a rally point (Q-0682
/// interim).
fn exit_cell(w: &mut World, k: ObjectId, v: ObjectId) -> Option<CellXY> {
    let b = w.obj(k)?.clone();
    let foundation = crate::objects::foundation_cells(b.pos.cell(), b.foundation);
    crate::regions::ensure(w);
    let skip = move |_: &World, c: CellXY| !foundation.contains(&c);
    crate::path::nearby_filtered(
        w,
        b.pos.cell(),
        crate::path::NearbyUse::Substitution { unit: v },
        Some(&skip),
    )
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
/// own turn before its behaviour processing.
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
    }
    let Some(k) = w.obj(id).and_then(|o| o.depot.queued) else {
        return;
    };
    if w.obj(k).is_none_or(|b| b.health <= 0 || !b.on_map) {
        if let Some(ob) = w.obj_mut(id) {
            ob.depot.queued = None;
        }
        return;
    }
    // spec: S-0314 claim 23 (every tick it asks again for a contact place: the alliance and free-place tests; while none is free it keeps waiting)
    let Some(place) = alliance_and_place(w, k, id) else {
        return;
    };
    take(w, id, k, place);
    // spec: S-0314 claim 23 (with a place it asks permission)
    if accepts(w, k, id, true) {
        // spec: S-0314 claim 23 (granted: it switches to enter and approaches as in claim 17)
        if let Some(ob) = w.obj_mut(id) {
            ob.depot.queued = None;
            ob.beh.queue_and_start_at_once(Behaviour::Enter, t, stale);
        }
        crate::econ::assign_destination(w, id, Some(Target::Object(k)));
        if let Some(ob) = w.obj_mut(id) {
            ob.path.steps.clear();
            ob.path.retry_at = 0;
        }
    } else {
        // spec: S-0314 claims 20 and 23 (refused: contact, queued entry and destination cleared; it falls back to rest)
        release(w, id);
        crate::econ::assign_destination(w, id, None);
        crate::process::fall_back(w, id, false);
    }
}

/// What applying an order does to a vehicle's depot contact before the
/// order's ordinary effects (S-0314 claims 20 and 23; S-0504 claim 14).
/// Returns `true` when the order ends here (claim 23: no destination, no
/// ordinary effects).
pub fn order_applied(w: &mut World, id: ObjectId, b: Behaviour, dest: Option<Target>) -> bool {
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    if o.kind != Kind::Vehicle {
        return false;
    }
    // spec: S-0314 claim 23 (any later order clears the queued entry)
    if let Some(ob) = w.obj_mut(id) {
        ob.depot.queued = None;
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
            // spec: S-0314 claim 20 (an enter order on the same depot ends the docking: a damaged vehicle takes the queued entry, a full-health one loses its contact at once)
            if full {
                release(w, id);
            } else if let Some(ob) = w.obj_mut(id) {
                ob.depot.queued = Some(l.depot);
            }
            return true;
        }
        if !w.rules().quirks.depot_remote_repair && matches!(b, Behaviour::Move | Behaviour::Attack)
        {
            // spec: S-0314 Edge cases (quirks.depot_remote_repair off: a docked vehicle given a move or attack order loses its contact as an approaching one does)
            release(w, id);
        }
        // spec: S-0314 claim 20 (otherwise a docked vehicle keeps contact and tether; the order runs normally)
        return false;
    }
    if let Some(l) = l
        && target_depot == Some(l.depot)
        && o.beh.seen() == Some(Behaviour::Enter)
    {
        // spec: S-0314 claim 20 (an enter order on the same depot while approaching in enter is ignored entirely)
        return true;
    }
    if l.is_some() {
        // spec: S-0314 claim 20 (an approaching vehicle given any other order loses its contact at once; its queued entry is cleared too)
        release(w, id);
    }
    let Some(k) = target_depot else {
        return false;
    };
    if !contacts(w, k).is_empty() {
        if !full && let Some(ob) = w.obj_mut(id) {
            // spec: S-0314 claim 23 (a later enter order on a depot that holds any contact: a damaged vehicle gets no destination and no contact and stays with a queued entry)
            ob.depot.queued = Some(k);
        }
        // spec: S-0314 claim 23 (a full-health vehicle gets no destination and no queued entry: the order ends)
        return true;
    }
    false
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
/// released without a run-away notice; an approaching vehicle loses its
/// target and falls back to guard where it stands; a docked one stays in
/// *sleep* (`quirks.depot_loss_leaves_sleep`).
pub fn depot_destroyed(w: &mut World, k: ObjectId, v: ObjectId) {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let t = w.tick;
    let Some(l) = link(w, v).filter(|l| l.depot == k) else {
        return;
    };
    if l.tethered && !w.rules().quirks.depot_loss_leaves_sleep {
        // spec: S-0314 Edge cases (quirks.depot_loss_leaves_sleep off: destruction treats contacts as a sale does)
        run_away(w, k, v);
        release(w, v);
        return;
    }
    release(w, v);
    if l.tethered {
        // spec: S-0314 claim 22 (a docked vehicle stays in sleep until it receives an order, quirks.depot_loss_leaves_sleep)
        return;
    }
    if let Some(ob) = w.obj_mut(v) {
        // spec: S-0314 claim 22 (an approaching vehicle loses its target and falls back to guard where it stands)
        crate::spawn::clear_own_target(ob);
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
        ob.beh.queue_and_start_at_once(Behaviour::Guard, t, stale);
    }
    crate::econ::assign_destination(w, v, None);
    if let Some(ob) = w.obj_mut(v) {
        ob.path.steps.clear();
    }
}

/// A destination was given to vehicle `id` (S-0314 claim 22): one whose
/// cell holds a repair depot or a tank bunker has its locomotor switched on
/// again.
pub fn destination_given(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if !o.depot.loco_off || o.dest.is_none() {
        return;
    }
    let c = o.pos.cell();
    let on = w
        .cells
        .list(c, crate::world::Layer::Ground)
        .iter()
        .any(|&x| {
            w.obj(x).is_some_and(|b| {
                b.kind == Kind::Building
                    && w.rules()
                        .buildings
                        .get(b.type_index as usize)
                        .is_some_and(|r| r.building.unit_repair || r.raw.bunker.unwrap_or(false))
            })
        });
    if on && let Some(ob) = w.obj_mut(id) {
        // spec: S-0314 claim 22 (the locomotor is switched on again when the vehicle receives a destination while a depot or tank bunker occupies its cell)
        ob.depot.loco_off = false;
    }
}

/// Whether a vehicle's locomotor is switched off by a depot's settling step
/// (S-0314 claims 21 and 22). The hovering locomotor of S-0503 asks it
/// (`hover::tick`): a hovering vehicle with its locomotor switched off cannot move
/// (`quirks.depot_loss_strands_hover`); no such effect was found for other
/// land locomotors, so the driving and ship locomotors ignore it.
pub fn locomotor_off(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.depot.loco_off)
}
