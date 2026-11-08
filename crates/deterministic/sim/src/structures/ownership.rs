//! A building's change of owner (S-0509 claims 12 to 16): the effects that
//! the owning specs give are run by `econ::events::owner_changed`; what this
//! module adds is the ordering around it: the contacts of the building
//! (claim 14), the planted bomb (claim 13, a seam), the spawner of a silo
//! (S-0506 claim 13) and the presentation event for the announcements and
//! the local player's building placement (claims 12 and 15).

use glasnost_num::geom::root_floor;

use crate::combat::effects::Effect;
use crate::econ::TypeRef;
use crate::rules::{Behaviour, Kind};
use crate::world::{HouseId, ObjectId, Target, World};

/// What caused a building ownership change (S-0509 claim 12): the cause
/// decides whether a capture or loss announcement is requested.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cause {
    /// An engineer capture (S-0509 claim 8): requests an announcement.
    Capture,
    /// The takeover of a vehicle-like building (S-0509 claim 17): requests
    /// an announcement.
    Takeover,
    /// A map trigger (S-0011 claim 24): no announcement.
    Trigger,
    /// The occupation rules of a civilian garrison building (S-0508 claims 7
    /// and 8): no announcement of this kind.
    Occupation,
    /// The mind-control takeover and release of a building (S-0601 claims
    /// 6 and 9): requests an announcement (S-0509 claim 12).
    MindControl,
}

impl Cause {
    /// Whether the cause requests a capture or loss announcement (S-0509
    /// claim 12: an engineer capture, a vehicle-like takeover and the
    /// mind-control takeover and release of a building do).
    pub fn announces(self) -> bool {
        matches!(self, Cause::Capture | Cause::Takeover | Cause::MindControl)
    }
}

/// How close, in leptons, a staying contact must be to the building's
/// docking point to change owner with it.
// spec: S-0509 Constants (contact follow distance 63 leptons)
const FOLLOW_DISTANCE: i32 = 63;

/// A building changes owner to `new` (no effect when it is its owner
/// already).
pub fn change(w: &mut World, b: ObjectId, new: Option<HouseId>, cause: Cause) {
    change_by(w, b, new, cause, None);
}

/// As [`change`], made in the turn of `actor` (the capturing engineer, the
/// arriving unit, the controller), so that the change can say whether the
/// building's slot animations take their turns before it (S-0108 claim 26,
/// Q-2186). `None` for a change not made in an object's turn.
pub fn change_by(
    w: &mut World,
    b: ObjectId,
    new: Option<HouseId>,
    cause: Cause,
    actor: Option<ObjectId>,
) {
    let Some(o) = w.obj(b) else { return };
    let old = o.owner;
    // spec: S-0509 Terms (a building ownership change has no effect when the new house is the old one)
    if old == new {
        return;
    }
    let of = TypeRef {
        kind: o.kind,
        index: o.type_index,
    };
    let pos = o.pos;
    let spawner = o.spawn.is_some();
    // spec: S-0509 claim 13 and S-0519 claim 7 (a planted timed bomb is disarmed without a detonation, except on a garrisonable type)
    crate::bomb::building_owner_changed(w, b);
    // spec: S-0509 claim 14 (the objects in contact with the building are asked whether they move: staying ones near the docking point change owner with it, every other one is told to leave)
    contacts(w, b, new);
    if let Some(ob) = w.obj_mut(b) {
        ob.owner = new;
    }
    // spec: S-0518 claim 3 (an ownership change makes the building the new owner's newest acquisition)
    crate::linked::acquire(w, b);
    crate::econ::events::owner_changed(w, b, old);
    // spec: S-0309 claim 31 (r124, Q-2747: every cause of claim 30's list, the vehicle-like takeover of S-0509 claim 17 included, resets the building's orders inside the change, after the target drop)
    reset_orders(w, b);
    if spawner {
        // spec: S-0509 claim 12 and S-0506 claim 13 (a missile silo's spawner clears its slots when the building changes house)
        crate::spawn::owner_changed(w, b);
    }
    // spec: S-0108 claim 26 (the slots of a building the map places stand directly after the building in the update order: a change made in the turn of an object that stands after the building comes after the slots' turns of its tick, one made in the turn of an object before it, or before the turn pass, comes before them; the building's own turn, the occupation rules, is before its slots)
    let after_slots = match actor {
        Some(a) => {
            let order = &w.update.order;
            match (
                order.iter().position(|&x| x == a),
                order.iter().position(|&x| x == b),
            ) {
                (Some(pa), Some(pb)) => pa > pb,
                // design: Q-2186 (an actor or building not in the update order: the announcing causes act after the slots, as before)
                _ => cause.announces(),
            }
        }
        // design: Q-2186 (no actor given: the announcing causes act after the slots, a trigger's and the occupation rules' change before them)
        None => cause.announces(),
    };
    w.effects.push(Effect::Captured {
        building: b,
        of,
        old,
        new,
        pos,
        // spec: S-0509 claim 12 (an engineer capture, a vehicle-like takeover and the mind-control takeover and release of a building request an announcement; a trigger and the occupation rules do not)
        announce: cause.announces(),
        after_slots,
    });
}

/// The orders reset of an owner change (S-0309 claim 31), called inside
/// the change after its owner is set and its target dropped (claim 30):
/// for every cause of claim 30's list, the vehicle-like takeover and the
/// contacts that follow a building. The caller has made the change to
/// another house; a change to the same house does none of this.
pub(crate) fn reset_orders(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let (kind, on_map) = (o.kind, o.on_map);
    let q = &w.rules().quirks;
    let (keep_override, keep_memory) = (
        q.owner_change_keeps_override,
        q.owner_change_keeps_attack_move,
    );
    let t = w.tick;
    let stale = q.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    if let Some(ob) = w.obj_mut(id) {
        if !keep_override {
            // spec: S-0309 Edge cases (quirks.owner_change_keeps_override off: the change empties the suspended slot and forgets the put-aside target and destination, before the guard request)
            ob.beh.suspended = None;
            ob.beh.aside = None;
        }
        if !keep_memory {
            // spec: S-0309 Edge cases (quirks.owner_change_keeps_attack_move off: the change removes the memory with its remembered destination and target and its engaged mark, before the guard request)
            ob.attack_move = None;
            ob.combat.engaged = false;
        }
    }
    // spec: S-0601 r23 claim 9 and S-0107 r46 claim 11, Changes of target (the clear of an infantry victim's target is a change of target: an attack state ends at the clear, with its request for stand, prone or deployed; a victim in the attack state with no target has no change and keeps the state)
    crate::infantry::watch_test(w, id);
    if w.obj(id).is_some_and(|o| o.dest.is_some()) {
        // spec: S-0309 claim 31 (the destination is cleared; clearing a destination the object does not have changes nothing)
        crate::econ::assign_destination(w, id, None);
    }
    // spec: S-0309 claim 31 (a vehicle in the middle of deploying or undeploying in place keeps its guard spot, S-0517 Terms, claims 4 and 6)
    let keeps_spot = crate::econ::inplace::in_transition(w, id);
    if let Some(ob) = w.obj_mut(id)
        && !keeps_spot
    {
        // spec: S-0309 claim 31 (it forgets its guard spot: a factory its rally point, S-0213 claim 20, and a released vehicle the rally point it saved, S-0417 claim 8)
        ob.guard_spot = None;
    }
    // spec: S-0309 claim 31 (a harvester forgets its remembered cell, S-0402 claim 3)
    crate::econ::harvest::forget_remembered(w, id);
    // spec: S-0309 claim 31 (a simple-deployer vehicle whose current behaviour is unload gets no guard request)
    if !crate::mind::deploying_in_place(w, id) {
        // spec: S-0309 claim 31 (the readiness is read after the clears above)
        let ready = crate::process::ready(w, id);
        if let Some(ob) = w.obj_mut(id) {
            // spec: S-0309 claim 31 and S-0215 claim 7 (guard is requested with a readiness-dependent immediate start; selling ignores the request, wait refuses guard)
            ob.beh
                .queue_and_start(Some(Behaviour::Guard), ready, t, stale);
        }
    }
    // design: Q-2801 (questions 3 and 4: nothing is added for a unit that a vehicle factory whose unloading ended here has not yet sent out, or for a vehicle docked at a captured repair building; S-0417's and S-0314's rules go on as they stand)
    if let Some(ob) = w.obj_mut(id)
        && (ob.beh.current == Some(Behaviour::Rescue) || ob.beh.queued == Some(Behaviour::Rescue))
    {
        // spec: S-0309 claim 31 (rescue current, or queued after the guard request: guard is made current at once, without the readiness test); design: Q-2801 (question 6: the force of S-0215 claim 9, which wait refuses)
        ob.beh.force(Behaviour::Guard, t, stale);
    }
    // spec: S-0309 claim 31 (an object off the map gets no fall-back)
    if !on_map {
        return;
    }
    // spec: S-0309 claim 31 (no fall-back for an object in contact with a vehicle factory)
    let contact = w.obj(id).and_then(|o| o.contact);
    if contact.is_some_and(|k| crate::econ::release::vehicle_factory(w, k)) {
        return;
    }
    // spec: S-0309 claim 31 (no fall-back for a vehicle factory whose current behaviour is still unload after the guard request; one whose guard started gets it)
    let current = w.obj(id).and_then(|o| o.beh.current);
    if kind == Kind::Building
        && crate::econ::release::vehicle_factory(w, id)
        && current == Some(Behaviour::Unload)
    {
        return;
    }
    if kind.is_mobile() {
        // spec: S-0309 claim 31 (an infantry unit, vehicle or aircraft falls back to rest in the ordinary way, S-0215 claims 13 to 15, an aircraft with S-0501 claim 18: an override still in effect takes T2's first row)
        // design: Q-2801 (question 1: an attack resumed here with a target the change made its own side's holds it as any target, S-0309 claims 25 part k and 29)
        crate::process::fall_back(w, id, false);
        return;
    }
    if kind != Kind::Building {
        return;
    }
    let Some(o) = w.obj(id) else { return };
    let building_up = o.beh.current == Some(Behaviour::Construction)
        || o.beh.queued == Some(Behaviour::Construction);
    // design: Q-2801 (questions 2 and 5: no idle-body request for a building being sold, or while construction is current or queued)
    let idle_request = current != Some(Behaviour::Selling) && !building_up;
    // spec: S-0108 claim 6 (a deferred request for the idle state leaves a running idle range unchanged)
    if idle_request && o.econ.idle_body.is_none() {
        // spec: S-0309 claim 31 (a building requests its idle body state, as at the end of S-0417 claim 2)
        crate::econ::body::enter_idle(w, id, false);
    }
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0309 claim 31 (and queues guard without an immediate start, S-0215 claim 7)
        ob.beh.queue(Some(Behaviour::Guard));
    }
}

/// The docking point of `building` for the contact `v` (S-0314 claim 18,
/// S-0404 claim 15).
fn docking_point(w: &World, building: ObjectId, v: ObjectId) -> crate::world::Pos {
    if crate::depot::is_depot(w, building) {
        return crate::depot::docking_point(w, building, v);
    }
    if super::bunker::is_bunker(w, building) {
        // spec: S-0606 claim 4 (a bunker's docking point)
        return super::bunker::docking_point(w, building, v);
    }
    let place = w.obj(v).and_then(|o| o.econ.exit).map_or(0, |e| e.place);
    crate::econ::release::pad_position_at(w, building, place)
}

/// The contacts of a changing building (claim 14).
fn contacts(w: &mut World, b: ObjectId, new: Option<HouseId>) {
    let list: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| o.contact == Some(b) && o.id != b)
        .map(|o| o.id)
        // spec: S-0606 claim 15 (the bunker link holds through the bunker's owner change: the bunkered vehicle is not asked to move)
        .filter(|&v| !super::bunker::holds_through_owner_change(w, b, v))
        .collect();
    if list.is_empty() {
        return;
    }
    let factory = crate::econ::release::vehicle_factory(w, b);
    let (mode, table) = (w.rules().mode, w.rules().quirks.table_root_distance);
    for v in list {
        let Some(o) = w.obj(v).cloned() else { continue };
        // spec: S-0509 claim 14 (it counts as staying when it has no destination or its movement has stopped)
        let staying = o.dest.is_none() || !crate::occupancy::in_progress(w, v);
        let near = || {
            // spec: S-0509 F3 (the three-dimensional distance between the docking point and the contact's position, by the table square root; within when at most 63)
            let d = docking_point(w, b, v);
            let (dx, dy, dz) = (
                i64::from(d.x.wrapping_sub(o.pos.x)),
                i64::from(d.y.wrapping_sub(o.pos.y)),
                i64::from(d.z.wrapping_sub(o.pos.z)),
            );
            root_floor((dx * dx + dy * dy + dz * dz) as u64, table, mode)
                <= u64::from(FOLLOW_DISTANCE.unsigned_abs())
        };
        // spec: S-0509 claim 14 (for a factory of vehicles the distance is not tested)
        if staying && (factory || near()) {
            let old = o.owner;
            if old != new {
                if let Some(ob) = w.obj_mut(v) {
                    ob.owner = new;
                }
                // The contact and its tether link are kept.
                crate::econ::events::owner_changed(w, v, old);
                // spec: S-0309 claim 31 (each object that changes owner together with a building it is in contact with has its orders reset)
                reset_orders(w, v);
            }
        } else {
            leave(w, b, v);
        }
    }
}

/// A contact told to leave (claim 14): it loses its contact and its tether
/// link with the building and gives up the building as its destination.
fn leave(w: &mut World, b: ObjectId, v: ObjectId) {
    if let Some(ob) = w.obj_mut(v) {
        ob.contact = None;
        // spec: S-0509 claim 14 (every other contact loses its contact and tether link with the building)
        if ob.econ.exit.is_some_and(|e| e.factory == b) {
            ob.econ.exit = None;
        }
    }
    if let Some(bb) = w.obj_mut(b)
        && bb.contact == Some(v)
    {
        bb.contact = None;
    }
    if w.obj(v).is_some_and(|o| o.dest == Some(Target::Object(b))) {
        crate::econ::assign_destination(w, v, None);
    }
    // A harvester or a vehicle on a depot is sent on its way like a released one.
    if w.obj(v).is_some_and(|o| o.kind == Kind::Vehicle) {
        crate::scatter::request(
            w,
            v,
            crate::scatter::ScatterRequest {
                threat: None,
                forced: true,
                anywhere: false,
            },
        );
    }
}
