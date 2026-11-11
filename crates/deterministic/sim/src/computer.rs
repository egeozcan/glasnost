//! The house-wide hunt order (S-0217 claim 1) and the periodic checks of a
//! test-only computer-controlled player house (S-0217 claims 2 to 4).
//!
//! The checks are the original's computer player's. In our engine only a
//! test-only computer-controlled player house makes them (S-0217, *Houses in
//! our engine*; S-0215 Terms): a player house whose `human` flag is clear,
//! as the headless arena and the oracle lanes mark theirs. Our AI houses and
//! human-controlled houses are `human` and make none, and neither do the
//! neutral and special houses. The hunt order itself is a rule of our engine
//! for every house that receives it: map action code 6 (S-0011) gives it too.

use serde::{Deserialize, Serialize};

use crate::rules::{Behaviour, FactoryKind, Kind};
use crate::world::{HouseId, HouseRole, ObjectId, World};

/// The check base delay, in ticks (S-0217 claim 2).
pub const CHECK_BASE_DELAY: i32 = 105; // spec: S-0217 Constants (check base delay 105 ticks, added to the draw of claim 2)
/// The lowest value of a check's delay draw (S-0217 claim 2).
pub const CHECK_DRAW_LOW: i32 = 1; // spec: S-0217 Constants (check draw range 1 to 7, one R(1, 7) on the scenario stream per check)
/// The highest value of a check's delay draw (S-0217 claim 2).
pub const CHECK_DRAW_HIGH: i32 = 7; // spec: S-0217 Constants (check draw range 1 to 7, one R(1, 7) on the scenario stream per check)
/// The base hit hold, in ticks (S-0217 claim 3).
pub const BASE_HIT_HOLD: i32 = 900; // spec: S-0217 Constants (base hit hold 900 ticks)

/// A house's check state (S-0217 claims 2 and 3). Every house carries it;
/// only a test-only computer-controlled player house reads it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checks {
    /// The tick of the next check. It is 0 when the game starts, so the
    /// first check is in the house update of tick 0 (S-0217 claim 2).
    pub next: i32,
    /// The last base hit tick *L* (S-0217 Terms and claim 3): the tick at
    /// which a building of the house was last hit by damage that has an
    /// attacker. It is 0 when the game starts.
    pub last_base_hit: i32,
}

/// The house-wide hunt order (S-0217 claim 1): in this tick, each actor of
/// house `h` on the map is reached once. A mobile actor has *hunt* queued
/// without an immediate start (S-0215 claim 7; an aircraft by S-0501 claim
/// 34's row); a building with occupants has them evicted with hunting and
/// without inside fallback (S-0508 claims 11 and 13), the buildings in
/// reverse creation order; any other building is not changed.
pub fn hunt_order(w: &mut World, h: HouseId) {
    // spec: S-0217 claim 1 (each actor on the map, not inside a building or a transport, is reached once: the mobile actors are taken before any eviction, so an occupant put out by this order is not reached a second time)
    let mobile: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| {
            o.owner == Some(h)
                && o.on_map
                && matches!(o.kind, Kind::Infantry | Kind::Vehicle | Kind::Aircraft)
        })
        .map(|o| o.id)
        .filter(|&id| crate::transport::riding(w, id).is_none())
        .collect();
    for id in mobile {
        // spec: S-0217 claim 1 (a mobile actor leaves its team, our engine has none, and has hunt queued without an immediate start, S-0215 claim 7; an aircraft in one of S-0501 claim 34's five behaviours ignores the request)
        crate::air::queue(w, id, Behaviour::Hunt);
    }
    // spec: S-0217 claim 1 (a building with occupants has them evicted, hunting, without inside fallback; S-0508 claims 11 and 13)
    crate::structures::garrison::hunt_order(w, h);
}

/// Whether house `h` makes the checks of S-0217 claims 2 to 4: a player
/// house that is not human-controlled, a test-only computer-controlled
/// player house (S-0215 Terms). A house the unused-house cleanup removed
/// (S-0009 claim 28) is no player of the game and makes none.
pub fn makes_checks(w: &World, h: HouseId) -> bool {
    // spec: S-0217 Behaviour, houses in our engine (only a test-only computer-controlled player house makes the checks; human-controlled houses, our AI houses, the neutral and the special house make none) and claim 2
    w.houses
        .get(usize::from(h.0))
        .is_some_and(|x| x.role == HouseRole::Player && !x.human && !x.removed)
}

/// The check of house `h` in its house update (S-0217 claim 2), when its
/// check delay has run out: claims 3 and 4, then the one delay draw that
/// sets the next check. The caller runs it after the raise of the house's
/// tags (S-0011 claim 35).
pub fn house_update(w: &mut World, h: HouseId) {
    if !makes_checks(w, h) {
        return;
    }
    let t = w.tick;
    let Some(c) = w.houses.get(usize::from(h.0)).map(|x| x.checks) else {
        return;
    };
    // spec: S-0217 claim 2 (a check in the house update of tick 0, then at each tick at which the check delay has run out)
    if t < c.next {
        return;
    }
    check(w, h, t, c.last_base_hit);
    // spec: S-0217 claim 2 (the check's only draw of its own: one R(1, 7) on the scenario stream, taken at the end of the check, after the draws of the order's evictions)
    let half = w.rules().quirks.ranged_draw_half_span; // spec: S-0202 claim 4 (the ranged draw's half-span rule)
    let r = w.rng.ranged(CHECK_DRAW_LOW, CHECK_DRAW_HIGH, half);
    if let Some(x) = w.houses.get_mut(usize::from(h.0)) {
        // spec: S-0217 claim 2 and Formulas (next check tick = t + 105 + R(1, 7))
        x.checks.next = t + CHECK_BASE_DELAY + r;
    }
}

/// Claims 3 and 4 at a check of house `h` at tick `t`, with last base hit
/// tick `last`.
fn check(w: &mut World, h: HouseId, t: i32, last: i32) {
    // spec: S-0217 claim 3 and Formulas (held at tick t when t <= L + 900)
    let held = t <= last + BASE_HIT_HOLD;
    let buildings: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| o.kind == Kind::Building && o.owner == Some(h))
        .map(|o| o.id)
        .collect();
    // spec: S-0217 claim 3 (no factory building of any kind on the map, building factories and occupied civilian buildings of a factory type included; S-0003 claim 21's factory kind other than none, the non-producing kinds included)
    let factory = buildings
        .iter()
        .any(|&b| w.obj(b).is_some_and(|o| o.on_map) && factory_building(w, b));
    // spec: S-0217 claim 3 (whatever construction vehicles it owns)
    let sale = !held && !factory;
    // spec: S-0217 claim 4 (no building other than occupied civilian buildings, and no construction vehicle: the order at every check, held or not)
    let bare = buildings.iter().all(|&b| occupied_civilian(w, b, h)) && !owns_construction_vehicle(w, h);
    if sale {
        sell_all(w, h);
    }
    // spec: S-0217 claims 3 and 4 (the order after the sell orders of claim 3, in the same house update; one order when both claims apply)
    if sale || bare {
        hunt_order(w, h);
    }
}

/// Claim 3's sell orders: every building house `h` owns on the map that has
/// health left, occupied civilian buildings included, one after another in
/// the house's order of ownership (S-0217 Terms).
fn sell_all(w: &mut World, h: HouseId) {
    let mut own: Vec<(u64, ObjectId)> = w
        .objects
        .values()
        .filter(|o| o.kind == Kind::Building && o.owner == Some(h) && o.on_map && o.health > 0)
        .map(|o| (o.combat.acquired, o.id))
        .collect();
    // spec: S-0217 claim 3 and Terms (order of ownership: the building the house came to own first comes first; S-0518 claim 3's acquisition number ranks a placement or an ownership change after every building the house already held)
    own.sort_unstable();
    for (_, b) in own {
        // spec: S-0217 claim 3 (a sell order, S-0408 claim 4, which decides whether it is sold)
        crate::econ::sell::sell(w, h, b);
    }
}

/// A factory building (S-0217 Terms): its type has any recognised factory
/// kind other than none (S-0003 claim 21), the non-producing kinds included.
fn factory_building(w: &World, b: ObjectId) -> bool {
    w.obj(b)
        .and_then(|o| w.rules().buildings.get(o.type_index as usize))
        // spec: S-0217 Terms (factory building: any recognised factory kind other than the no-factory value, S-0003 claim 21)
        .is_some_and(|t| t.factory_category.is_some())
}

/// An occupied civilian building of house `h` (S-0217 Terms): a civilian
/// garrison building that the house owns, which in our engine it owns only
/// through its own infantry's occupation (S-0508 claim 7).
fn occupied_civilian(w: &World, b: ObjectId, h: HouseId) -> bool {
    // spec: S-0217 Terms (occupied civilian building: a civilian garrison building that a house owns only because its own infantry occupy it, S-0508 claim 7)
    w.obj(b).is_some_and(|o| o.owner == Some(h))
        && crate::structures::garrison::civilian_building(w, b)
        && crate::structures::garrison::civilian_house(w) != Some(h)
}

/// Whether house `h` owns a construction vehicle (S-0217 Terms): a vehicle
/// whose type deploys into a building factory, a factory building whose
/// type makes buildings.
fn owns_construction_vehicle(w: &World, h: HouseId) -> bool {
    let r = w.rules();
    w.objects.values().any(|o| {
        o.owner == Some(h)
            && o.kind == Kind::Vehicle
            // spec: S-0217 Terms (construction vehicle: a vehicle whose type deploys into a building factory, S-0522 claim 5's term)
            && crate::econ::deploy::deploys_into(w, o.kind, o.type_index)
                .and_then(|b| r.buildings.get(b as usize))
                .is_some_and(|t| t.factory == Some(FactoryKind::Building))
    })
}

/// A hit on building `b` (S-0217 claim 3): when the hit has a known attacker
/// and lowered the building's health, its owner's last base hit tick
/// becomes the current tick, whatever the attacker's house. A building
/// whose type undeploys into a vehicle and covers one cell changes nothing.
/// The building's own damage reaches here only when its type damages
/// itself (S-0302 claim 8's first stage stops it otherwise).
pub fn building_hit(w: &mut World, b: ObjectId, owner: Option<HouseId>, attacker: Option<ObjectId>) {
    // spec: S-0217 claim 3 (damage that has an attacker; S-0302 claim 1: a known attacker is an object in the world) and Q-2986 (interim: only a hit that lowers the building's health sets L; the caller passes no other)
    if !attacker.is_some_and(|a| w.obj(a).is_some()) {
        return;
    }
    // spec: S-0217 claim 3 and Import mapping (a building whose type undeploys into a vehicle type and has a one-cell foundation does not change L)
    if crate::econ::sell::vehicle_like(w, b) {
        return;
    }
    let t = w.tick;
    if let Some(x) = owner.and_then(|h| w.houses.get_mut(usize::from(h.0))) {
        // spec: S-0217 claim 3 (L becomes the current tick)
        x.checks.last_base_hit = t;
    }
}
