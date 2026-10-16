//! The arrival of an infantry unit at a building (S-0508 claim 2, S-0509
//! claims 1 and 2, S-0510 claim 1): the one dispatcher that turns an arrival
//! into a garrison entry, an engineer outcome, an infiltration, a takeover or
//! a refusal.

use crate::rules::{Behaviour, Kind};
use crate::scatter::{self, ScatterRequest};
use crate::structures::{engineer, garrison};
use crate::world::{Layer, ObjectId, Target, World};

/// The building an arriving infantry unit enters (S-0509 claim 1): the unit
/// is in *capture*, *area guard* or *patrol*, stands in a cell that holds a
/// building, and that building is its destination or its target. Returns the
/// building and whether it is the destination.
fn entered_building(w: &World, id: ObjectId) -> Option<(ObjectId, bool)> {
    let o = w.obj(id)?;
    if o.kind != Kind::Infantry || !o.on_map {
        return None;
    }
    // spec: S-0509 claim 1 (the behaviour is capture, area guard or patrol)
    if !matches!(
        o.beh.current,
        Some(Behaviour::Capture | Behaviour::AreaGuard | Behaviour::Patrol)
    ) {
        return None;
    }
    let cell = o.pos.cell();
    let here = |t: Option<Target>| match t {
        Some(Target::Object(b))
            if w.obj(b).is_some_and(|x| x.kind == Kind::Building && x.on_map)
                // spec: S-0509 claim 1 (the cell holds the building: any cell of its foundation)
                && w.cells.list(cell, Layer::Ground).contains(&b) =>
        {
            Some(b)
        }
        _ => None,
    };
    // spec: S-0509 claim 1 (the building is the unit's destination or its target)
    here(o.dest)
        .map(|b| (b, true))
        .or_else(|| here(o.target).map(|b| (b, false)))
}

/// The entry update of an infantry unit's arrival at a standing position
/// (S-0208 claims 6 and 22): called by the walker after its boarding test
/// and before its own arrival effects. Returns whether the unit has left
/// the map (it went inside or was used up), in which case the arrival has
/// no further effect (S-0208 claim 22).
pub fn arrival(w: &mut World, id: ObjectId) -> bool {
    // spec: S-0519 claim 13 (a charge planter in sabotage that reaches its destination building plants a charge or is refused)
    if crate::bomb::charge_arrival(w, id) {
        return false;
    }
    let Some((b, by_destination)) = entered_building(w, id) else {
        return false;
    };
    let capturing = w
        .obj(id)
        .is_some_and(|o| o.beh.current == Some(Behaviour::Capture));
    // spec: S-0508 claim 2 and S-0509 claim 1 (an occupier type or an assaulter type in capture whose destination is a building makes the garrison entry attempt, whether or not it can occupy that building, a vehicle-like building included: a refusal there does not become a takeover)
    if capturing && by_destination && (garrison::occupier(w, id) || garrison::assaulter(w, id)) {
        return garrison::attempt(w, id, b);
    }
    // spec: S-0509 claim 1 and 17 (otherwise a destination building that is vehicle-like follows the takeover: any infantry unit, whatever its type)
    if by_destination && engineer::vehicle_like(w, b) {
        return engineer::takeover(w, id, b);
    }
    let is_engineer = engineer::is_engineer(w, id);
    // spec: S-0509 claim 3 row 1 and S-0511 claim 17 (a bridge repair hut, whoever owns it: the engineer's entry is S-0511's, whose entry raise is this spec's general raise, so the hut gets no second one)
    if is_engineer && crate::bridges::hut::is_hut(w, b) {
        if crate::bridges::hut::engineer_entered_hut(w, id, b) {
            return true;
        }
        // spec: S-0511 Edge cases (quirks.repair_consumes_engineer_always off: an entry that would repair nothing is refused)
        refuse(w, id, b);
        return false;
    }
    // spec: S-0509 claim 2 (the general entry raise: before any other effect, one raise on the building's tag with the entering unit as causing object, whatever the outcome)
    crate::mp_triggers::entry_raise(w, b, id);
    // The raise's actions may have removed either object.
    if !w.obj(id).is_some_and(|o| o.on_map) || w.obj(b).is_none() {
        return w.obj(id).is_none_or(|o| !o.on_map);
    }
    if is_engineer {
        // spec: S-0509 claim 2 (an engineer follows claims 3 to 11; a spy whose type also has the engineer flag is handled as an engineer)
        return engineer::entry(w, id, b);
    }
    // spec: S-0509 claim 2 (a spy or other infiltrator follows S-0510 claims 1 to 9)
    if crate::structures::spy::is_spy(w, id) {
        return crate::structures::spy::infiltrate(w, id, b);
    }
    // spec: S-0509 claim 2 and S-0510 claim 10 (any other unit is refused and scattered: no effect on the building)
    refuse(w, id, b);
    false
}

/// A refused entry (S-0508 claim 4, S-0509 claim 4, S-0510 claim 10): the
/// destination is cleared and the unit receives a scatter request whose
/// threat point is the building's centre, forced and move-anywhere. The unit
/// stays on the map and its behaviour is unchanged.
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
