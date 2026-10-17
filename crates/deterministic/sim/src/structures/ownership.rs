//! A building's change of owner (S-0509 claims 12 to 16): the effects that
//! the owning specs give are run by `econ::events::owner_changed`; what this
//! module adds is the ordering around it: the contacts of the building
//! (claim 14), the planted bomb (claim 13, a seam), the spawner of a silo
//! (S-0506 claim 13) and the presentation event for the announcements and
//! the local player's building placement (claims 12 and 15).

use glasnost_num::geom::root_floor;

use crate::combat::effects::Effect;
use crate::econ::TypeRef;
use crate::rules::Kind;
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
    if spawner {
        // spec: S-0509 claim 12 and S-0506 claim 13 (a missile silo's spawner clears its slots when the building changes house)
        crate::spawn::owner_changed(w, b);
    }
    w.effects.push(Effect::Captured {
        building: b,
        of,
        old,
        new,
        pos,
        // spec: S-0509 claim 12 (an engineer capture, a vehicle-like takeover and the mind-control takeover and release of a building request an announcement; a trigger and the occupation rules do not)
        announce: cause.announces(),
    });
}

/// The docking point of `building` for the contact `v` (S-0314 claim 18,
/// S-0404 claim 15).
fn docking_point(w: &World, building: ObjectId, v: ObjectId) -> crate::world::Pos {
    if crate::depot::is_depot(w, building) {
        return crate::depot::docking_point(w, building, v);
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
