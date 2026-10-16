//! The arrival of an infantry unit at a building (S-0508 claim 2, S-0509
//! claims 1 and 2, S-0510 claim 1): the one dispatcher that turns an arrival
//! into a garrison entry, an engineer outcome, an infiltration or a refusal.

use crate::rules::{Behaviour, Kind};
use crate::scatter::{self, ScatterRequest};
use crate::world::{Layer, ObjectId, Target, World};

/// The destination building of an infantry unit that has just arrived in
/// one of its cells, if the three conditions of a building entry hold: its
/// behaviour is *capture* and the cell it stands in holds the building that
/// is its destination (S-0508 claim 2).
fn destination_building(w: &World, id: ObjectId) -> Option<ObjectId> {
    let o = w.obj(id)?;
    if o.kind != Kind::Infantry || !o.on_map {
        return None;
    }
    // spec: S-0508 claim 2 (the behaviour is capture and the building is the unit's destination; its target does not count)
    if o.beh.current != Some(Behaviour::Capture) {
        return None;
    }
    let Some(Target::Object(b)) = o.dest else {
        return None;
    };
    let bo = w.obj(b)?;
    if bo.kind != Kind::Building || !bo.on_map {
        return None;
    }
    // spec: S-0508 claim 2 (it arrives in a cell that holds the building)
    w.cells
        .list(o.pos.cell(), Layer::Ground)
        .contains(&b)
        .then_some(b)
}

/// The entry update of an infantry unit's arrival at a standing position
/// (S-0208 claims 6 and 22): called by the walker after its boarding test
/// and before its own arrival effects. Returns whether the unit has left
/// the map (it went inside or was used up), in which case the arrival has
/// no further effect (S-0208 claim 22).
pub fn arrival(w: &mut World, id: ObjectId) -> bool {
    let Some(b) = destination_building(w, id) else {
        return false;
    };
    // spec: S-0508 claim 2 (an occupier type or an assaulter type in capture whose destination is a building makes the garrison entry attempt, whether or not it can occupy that building, a vehicle-like building included)
    if crate::structures::garrison::occupier(w, id) || crate::structures::garrison::assaulter(w, id)
    {
        return crate::structures::garrison::attempt(w, id, b);
    }
    // Engineers, spies and other entrants: S-0509 and S-0510.
    false
}

/// A refused entry (S-0508 claim 4, S-0509 claim 4): the destination is
/// cleared and the unit receives a scatter request whose threat point is the
/// building's centre, forced and move-anywhere. The unit stays on the map
/// and its behaviour is unchanged.
pub fn refuse(w: &mut World, unit: ObjectId, b: ObjectId) {
    // spec: S-0508 claim 4 (the unit has its destination cleared)
    crate::econ::assign_destination(w, unit, None);
    let threat = w.reference_point(b);
    // spec: S-0508 claim 4 (a scatter request whose threat point is the building's centre, forced and move-anywhere, S-0208 claims 11 to 13)
    scatter::request(
        w,
        unit,
        ScatterRequest {
            threat: Some(threat),
            forced: true,
            anywhere: true,
        },
    );
}
