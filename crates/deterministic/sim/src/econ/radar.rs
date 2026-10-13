//! Radar availability (S-0410 claims 1 to 3): when a house has radar and
//! when that is worked out again. What the radar map shows is the pure
//! functions of [`super::minimap`], which the interface calls.

use glasnost_num::Rounding;
use glasnost_rules::schema::Ruleset;
use serde::{Deserialize, Serialize};

use crate::rules::{Behaviour, Kind};
use crate::world::{HouseId, ObjectId, World};

/// The radar rules (S-0410 Import mapping).
#[derive(Clone, Debug, Default)]
pub struct Rules {
    /// Radar flag of each building type (claim 1).
    pub radar: Vec<bool>,
}

impl Rules {
    /// The radar rules of a resolved ruleset.
    pub fn new(r: &Ruleset, mode: Rounding) -> Self {
        let _ = mode;
        let reg = r.registries.get("buildings").cloned().unwrap_or_default();
        Self {
            radar: reg
                .iter()
                // spec: S-0410 Import mapping (Radar absent: no)
                .map(|n| {
                    r.sim
                        .buildings
                        .get(n)
                        .and_then(|b| b.econ.radar)
                        .unwrap_or(false)
                })
                .collect(),
        }
    }
}

/// A house's radar availability (S-0410 claims 1 to 3).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HouseRadar {
    /// The house has radar (claim 1), as last rechecked.
    pub available: bool,
    /// An event since the last recheck asks for one (claim 2).
    pub due: bool,
    /// The map's free-radar flag (claim 1), fixed for the game.
    pub free: bool,
    /// A running radar outage: the tick at which it ends (Terms; the
    /// superweapon and trigger specs start it).
    pub outage_until: Option<i32>,
    /// The house is defeated: radar on for good (claim 1).
    pub defeated: bool,
    /// The tick of the last change of `available`, for the panel's sound
    /// and animation (claim 3).
    pub changed: Option<i32>,
}

/// Asks for a recheck of a house's radar in its next update (claim 2).
pub fn mark_due(w: &mut World, h: HouseId) {
    if let Some(house) = w.houses.get_mut(usize::from(h.0)) {
        house.econ.radar.due = true;
    }
}

/// Whether the house has radar (claim 1), as last rechecked.
pub fn available(w: &World, h: HouseId) -> bool {
    w.houses
        .get(usize::from(h.0))
        .is_some_and(|x| x.econ.radar.available)
}

/// Starts a radar outage of `ticks` ticks (Terms; the weather storm and the
/// radar-outage trigger action call it). Its start and end are events
/// (claim 2).
pub fn start_outage(w: &mut World, h: HouseId, ticks: i32) {
    let t = w.tick;
    if let Some(house) = w.houses.get_mut(usize::from(h.0)) {
        house.econ.radar.outage_until = Some(t.saturating_add(ticks.max(0)));
        house.econ.radar.due = true;
    }
}

/// Whether the building qualifies for radar (claim 1): a radar building
/// that is switched on, on the map, alive and not being sold.
fn qualifies(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        o.kind == Kind::Building
            && w.rules()
                .econ
                .radar
                .radar
                .get(o.type_index as usize)
                .copied()
                .unwrap_or(false)
            // spec: S-0410 claim 1 (switched on, on the map, alive, neither current nor queued behaviour selling)
            && !o.econ.building.switched_off
            && o.on_map
            && o.health > 0
            && o.beh.current != Some(Behaviour::Selling)
            && o.beh.queued != Some(Behaviour::Selling)
    })
}

/// Claim 1 at this moment for an undefeated house.
pub fn claim_1(w: &World, h: HouseId) -> bool {
    let Some(house) = w.houses.get(usize::from(h.0)) else {
        return false;
    };
    let r = &house.econ.radar;
    // spec: S-0410 claim 1 (no radar outage running)
    if r.outage_until.is_some_and(|u| w.tick < u) {
        return false;
    }
    // spec: S-0410 claim 1 (the free-radar flag)
    if r.free {
        return true;
    }
    // spec: S-0410 claim 1 (not on low power, tested once for the whole house)
    if crate::econ::power::low_power(w, h) {
        return false;
    }
    let ok = |id: ObjectId| {
        // spec: S-0410 claim 1 (not under an electromagnetic lock and not being erased)
        !crate::econ::power::under_pulse(w, id) && !crate::econ::power::being_erased(w, id)
    };
    let q: Vec<ObjectId> = house
        .econ
        .acquired
        .iter()
        .copied()
        .filter(|&id| w.obj(id).is_some_and(|o| o.owner == Some(h)) && qualifies(w, id))
        .collect();
    // spec: S-0410 claim 1 (the first qualifying building in the order of ownership decides, quirks.radar_first_building_decides)
    if w.rules().quirks.radar_first_building_decides {
        q.first().is_some_and(|&id| ok(id))
    } else {
        q.iter().any(|&id| ok(id))
    }
}

/// The radar part of a house's update (claim 2): an outage that ends here
/// is an event; a due house is rechecked, and the spy satellite with it
/// (S-0409 claim 15).
pub fn house_update(w: &mut World, h: HouseId) {
    let t = w.tick;
    let hi = usize::from(h.0);
    let Some(house) = w.houses.get_mut(hi) else {
        return;
    };
    let r = &mut house.econ.radar;
    if let Some(u) = r.outage_until
        && t >= u
    {
        // spec: S-0410 claim 2 (a radar outage that ends is an event)
        r.outage_until = None;
        r.due = true;
    }
    // spec: S-0410 claim 2 (after its defeat a house's radar is no longer re-evaluated)
    if r.defeated || !r.due {
        return;
    }
    r.due = false;
    let now = claim_1(w, h);
    set(w, h, now);
    // spec: S-0409 claim 15 (the spy satellite is checked whenever the radar state is rechecked)
    crate::econ::shroud::spy_satellite_check(w, h);
}

fn set(w: &mut World, h: HouseId, now: bool) {
    let t = w.tick;
    let r = &mut w.houses[usize::from(h.0)].econ.radar;
    if r.available != now {
        r.available = now;
        // spec: S-0410 claim 3 (the panel plays the radar-on or radar-off sound and animation when availability changes)
        r.changed = Some(t);
    }
}

/// A house is defeated (S-0414 claim 5): its radar comes on in the tick of
/// the defeat and stays on (S-0410 claim 1); the map is revealed for it
/// (S-0409 claim 15). S-0414 calls this.
pub fn on_defeat(w: &mut World, h: HouseId) {
    crate::econ::shroud::defeat_reveal(w, h);
    if w.houses.get(usize::from(h.0)).is_none() {
        return;
    }
    set(w, h, true);
    let r = &mut w.houses[usize::from(h.0)].econ.radar;
    // spec: S-0410 claim 1 (radar on from the defeat for the rest of the game, without a recheck)
    r.defeated = true;
    r.due = false;
}
