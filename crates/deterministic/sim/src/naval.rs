//! Naval units (S-0504): the ship locomotor, wakes, sinking, underwater vessels and docking at yards.
//!
//! - **Ship locomotor (claim 1).** A ship carries the driving locomotor's
//!   state (`Object::drive`) and moves by [`crate::drive::tick`]: speed,
//!   tracks, turning in place, the speed fraction, stopping, step checks,
//!   scatter requests, the lead speed, the approach stop and cell-tag
//!   arrival are the driving ones. Its own crush mode and crush slowdown are
//!   S-0210 claim 9's ship case (`drive::crush_mode`, `crush::entered_cell`).
//! - **Launching hold (claim 3).** `drive::begin_track` waits while
//!   `spawn::track_held`, for ships as for drive vehicles.
//! - **Wakes (claim 2).** [`ship_wake`], after the ship's movement in its turn.
//! - **Sinking (claims 4 and 5).** [`start_sinking`] marks a vessel;
//!   [`sinking_turn`] runs the descent at the start of each of its turns,
//!   and the locomotor is not run while it lasts (`tick::object_turn`).
//! - **Underwater vessels (claim 6).** [`submerged`] is the Terms' state;
//!   cloaking (S-0516) is not modelled, so no vessel is submerged yet.
//! - **Docking (claims 7 and 8).** [`naval_match_refuses`] is the naval part
//!   of the docking answer; [`starting_distance`], [`depot_distance`] and
//!   [`waiting_gate`] are the depot's waiting gate. Nothing asks for them yet:
//!   repair depots and docking at them are not built (Q-0641).

use glasnost_map::terrain::land;
use glasnost_num::F64;
use glasnost_num::geom::root_floor;
use glasnost_rules::sim::{Locomotor as RsLocomotor, MovementZone};
use serde::{Deserialize, Serialize};

use crate::rules::Kind;
use crate::world::{ObjectId, Pos, World};

/// Ship wake period: tick numbers divisible by 8 (S-0504 claim 2, Constants).
// spec: S-0504 Constants (ship wake period 8)
pub const SHIP_WAKE_PERIOD: i32 = 8;
/// Sinking descent per update, leptons (S-0504 claim 4, Constants).
// spec: S-0504 Constants (sinking descent 5 leptons per tick)
pub const SINKING_DESCENT: i32 = 5;
/// The vessel leaves play once its height above the terrain is strictly below this (S-0504 claim 4).
// spec: S-0504 Constants (sinking removal height −400 leptons, strictly below)
pub const SINKING_REMOVAL_HEIGHT: i32 = -400;
/// Sinking wake period: tick numbers divisible by 4 (S-0504 claim 4).
// spec: S-0504 Constants (sinking wake period 4)
pub const SINKING_WAKE_PERIOD: i32 = 4;
/// The bound of each sinking-wake displacement draw, leptons (S-0504 F2).
// spec: S-0504 Constants (sinking wake displacement [−170, 170] per axis, inclusive)
pub const SINKING_WAKE_SPREAD: i32 = 170;
/// Depot starting distance for the ship and hovering locomotors (S-0504 claim 8).
// spec: S-0504 Constants (depot starting distance, ship and hovering locomotors: 200 leptons)
pub const START_DISTANCE_SHIP: i32 = 200;
/// Depot starting distance for every other locomotor (S-0504 claim 8).
// spec: S-0504 Constants (depot starting distance, other locomotors: 100 leptons)
pub const START_DISTANCE_OTHER: i32 = 100;
/// The delay from a successful waiting check to the settling processing (S-0504 claim 8).
// spec: S-0504 Constants (waiting-to-settling delay 3 ticks)
pub const SETTLING_DELAY: i32 = 3;

/// A vessel's naval state (S-0504 Terms).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavalState {
    /// The vessel is sinking (S-0504 Terms; S-0312 claim 10; S-0614 claim 11).
    #[serde(default)]
    pub sinking: bool,
}

/// Sets a vessel sinking (S-0312 claim 10 at a killing hit; S-0614 claim 11
/// for a cull, without a hit; S-0504 claim 5: other events that set a
/// vehicle sinking). Its health is not changed here. The descent starts at
/// its next own update (S-0504 claim 4); for a vessel already sinking it
/// continues from its current height (claim 13).
pub fn start_sinking(w: &mut World, id: ObjectId) {
    if let Some(o) = w.obj_mut(id) {
        // spec: S-0504 Terms (sinking: the vessel goes down while it stays in play) and S-0614 claim 11 (whatever its type's weight; no hit)
        o.naval.get_or_insert_with(Default::default).sinking = true;
    }
}

/// Whether a vessel is sinking (S-0504 Terms).
pub fn sinking(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .and_then(|o| o.naval.as_ref())
        .is_some_and(|n| n.sinking)
}

/// Whether a killed vehicle sinks instead of exploding (S-0312 claim 10):
/// a naval type that is not underwater and not organic, whose weight is at
/// least the game-wide sinking weight, on a water cell. A type without a
/// weight has weight 1.0 (S-0504 claim 11).
pub fn sinks_at_kill(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if o.kind != Kind::Vehicle {
        return false;
    }
    let Some(tc) = crate::combat::veterancy::type_combat(w, id) else {
        return false;
    };
    // spec: S-0312 claim 10 and S-0504 claim 6 (a naval type that is not an underwater type and not organic)
    if !tc.naval || tc.underwater || tc.organic {
        return false;
    }
    let mode = w.rules().mode;
    // spec: S-0312 claim 10 (the type's weight at least the game-wide sinking weight); S-0504 claim 11 (an absent weight is exactly 1.0)
    let heavy = w
        .rules()
        .unit(o.kind, o.type_index)
        .map(|u| u.vehicle.weight.unwrap_or_else(|| F64::from_i64(1, mode)))
        .is_some_and(|wt| wt.ge(w.rules().combat.general.sinking_weight));
    // spec: S-0312 claim 10 (on a water cell)
    let water = w
        .cells
        .cell(o.pos.cell())
        .is_some_and(|c| c.land == land::WATER);
    heavy && water
}

/// A sinking vessel's update at the start of its own turn (S-0504 claim 4,
/// F1, F2): it is lowered by 5 leptons; below −400 leptons above the
/// terrain it leaves play at once; otherwise on a tick number divisible by
/// 4 it takes two ranged draws over [−170, 170] and creates one wake at the
/// displaced point on the water surface (the draws are taken with no wake
/// animation defined, and then no wake is made). Returns whether the vessel
/// left play.
pub fn sinking_turn(w: &mut World, id: ObjectId) -> bool {
    if !sinking(w, id) {
        return false;
    }
    let Some(o) = w.obj_mut(id) else { return false };
    // spec: S-0504 claim 4 and F1 (each own update first lowers it by 5 leptons)
    o.pos.z -= SINKING_DESCENT;
    let pos = o.pos;
    let terrain = w.surface_at(pos.x, pos.y);
    let h = pos.z - terrain;
    if h < SINKING_REMOVAL_HEIGHT {
        // spec: S-0504 claim 4 (below −400 leptons, strictly: it leaves play at once, no draws; S-0312 claim 15) and claim 5 (no further destruction credit)
        crate::combat::death::leave_play(w, id);
        return true;
    }
    if w.tick % SINKING_WAKE_PERIOD == 0 {
        let half = w.rules().quirks.ranged_draw_half_span;
        // spec: S-0504 claim 4 and F2 (two ranged draws over [−170, 170] from the scenario stream; the first displaces y, the second x)
        let r1 = w
            .rng
            .ranged(-SINKING_WAKE_SPREAD, SINKING_WAKE_SPREAD, half);
        let r2 = w
            .rng
            .ranged(-SINKING_WAKE_SPREAD, SINKING_WAKE_SPREAD, half);
        if let Some(wake) = w.rules().wake.clone() {
            // spec: S-0504 F2 (wake position (x + r₂, y + r₁, z − h), at the water surface)
            let at = Pos::new(pos.x + r2, pos.y + r1, pos.z - h);
            crate::combat::effects::anim(w, &wake, at);
        }
        // spec: S-0504 claim 4 (no wake animation: the two draws are still taken and no wake is made; a design rule)
    }
    false
}

/// Moves a vehicle with the ship locomotor in its turn (S-0504 claim 1, by
/// the driving rules), then its wake (claim 2); whether it moved. A sinking
/// vessel's locomotor is not run (claim 5; `tick::object_turn`).
pub fn tick(w: &mut World, id: ObjectId) -> bool {
    if sinking(w, id) {
        // spec: S-0504 claim 5 (a sinking vessel makes no progress by its locomotor, turning and speed-fraction updates included)
        return false;
    }
    // spec: S-0504 claim 1 (a ship moves, turns and stops exactly as a drive vehicle of S-0207 with the same type fields)
    let moved = crate::drive::tick(w, id);
    // spec: S-0504 claim 2 (the wake check after that tick's locomotor movement and track transitions, in the ship's own turn)
    ship_wake(w, id);
    moved
}

/// The wake of a moving ship (S-0504 claim 2): a tick number divisible by
/// 8, moving now (a turning body, or an active track with s > 0 at the
/// check), not an underwater type, off a bridge deck, on water, with the
/// game-wide wake animation defined.
pub fn ship_wake(w: &mut World, id: ObjectId) {
    let underwater = crate::combat::veterancy::type_combat(w, id).is_some_and(|tc| tc.underwater);
    if underwater {
        // spec: S-0504 claims 2 and 6 (an underwater type makes no wakes while moving)
        return;
    }
    // spec: S-0504 claim 2 (tick numbers divisible by 8)
    crate::drive::wake_check(w, id, SHIP_WAKE_PERIOD);
}

/// Whether an object is *submerged* (S-0504 Terms, S-0516 claim 41): an
/// underwater-type vehicle whose cloak state is cloaking, cloaked or
/// uncloaking; an underwater type without the cloak ability is never
/// submerged (claim 6).
pub fn submerged(w: &World, id: ObjectId) -> bool {
    let tc = crate::combat::veterancy::type_combat(w, id);
    let underwater = tc.is_some_and(|tc| tc.underwater);
    // spec: S-0504 claim 6 (without the cloak ability an underwater type stays surfaced)
    let cloakable = tc.is_some_and(|tc| tc.cloakable);
    // spec: S-0504 Terms and claim 6, S-0516 claim 41 (submerged: underwater type and a cloak state of cloaking, cloaked or uncloaking)
    underwater && cloakable && crate::cloak::not_uncloaked(w, id)
}

/// Whether a type has the naval flag (S-0303 Terms; S-0504 claim 7), for a
/// unit or a building.
fn naval_flag(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if o.kind == Kind::Building {
        return w
            .rules()
            .buildings
            .get(o.type_index as usize)
            .is_some_and(|b| b.naval);
    }
    crate::combat::veterancy::type_combat(w, id).is_some_and(|tc| tc.naval)
}

/// The naval match of a docking answer (S-0504 claim 7; S-0214 Terms,
/// *docking answer*): a building refuses a unit that asks to enter or dock
/// when exactly one of the two has the naval flag, unless the unit's
/// movement zone is *amphibious*. Only this refusal; the building's other
/// reasons are its own.
pub fn naval_match_refuses(w: &World, building: ObjectId, unit: ObjectId) -> bool {
    let zone = w
        .obj(unit)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        .map(|u| u.zone);
    // spec: S-0504 claim 7 (only the plain amphibious movement zone is the exception; amphibious-crusher and amphibious-destroyer are not)
    if zone == Some(MovementZone::Amphibious) {
        return false;
    }
    // spec: S-0504 claim 7 (refuses when exactly one of the building and the unit has the naval flag)
    naval_flag(w, building) != naval_flag(w, unit)
}

/// The depot's starting distance for a docked vehicle (S-0504 claim 8):
/// 200 leptons for the ship and hovering locomotors, 100 for every other.
pub fn starting_distance(w: &World, vehicle: ObjectId) -> i32 {
    let Some(o) = w.obj(vehicle) else {
        return START_DISTANCE_OTHER;
    };
    let loco = w
        .rules()
        .unit(o.kind, o.type_index)
        .map(|u| u.type_locomotor());
    // spec: S-0504 claim 8 (200 leptons for the ship or hovering locomotor, 100 for every other)
    if matches!(loco, Some(RsLocomotor::Ship | RsLocomotor::Hover)) {
        START_DISTANCE_SHIP
    } else {
        START_DISTANCE_OTHER
    }
}

/// The depot distance of S-0504 F3: trunc(√((x − a)² + (y − b)² + (z − c)²))
/// in integer leptons, height included. The original works in double
/// precision; for integer radicands below 2⁵², which every map distance is,
/// the truncated correctly rounded root equals the integer square root, so
/// this is exact (not the table root of S-0204).
pub fn depot_distance(p: Pos, dock: Pos) -> i32 {
    let dx = i64::from(p.x - dock.x);
    let dy = i64::from(p.y - dock.y);
    let dz = i64::from(p.z - dock.z);
    // spec: S-0504 F3 (squared differences, their sum and the square root, truncated toward zero; height counts)
    let n = (dx * dx + dy * dy + dz * dz) as u64;
    i32::try_from(root_floor(n, false, glasnost_num::Rounding::TowardZero)).unwrap_or(i32::MAX)
}

/// The fixed docking point of a building with the obsolete-unloading flag
/// (S-0504 claim 12): the centre of the cell (2, 1) cells from its top-left
/// foundation cell, at the building's own height; `None` for any other
/// building, a naval vehicle factory included (it uses S-0314 claim 18's
/// point). It takes precedence over imported pad docking displacements.
pub fn yard_docking_point(w: &World, yard: ObjectId) -> Option<Pos> {
    let o = w.obj(yard)?;
    if o.kind != Kind::Building {
        return None;
    }
    let b = w.rules().buildings.get(o.type_index as usize)?;
    // spec: S-0504 claim 12 (the obsolete-unloading flag, not the vehicle-factory flag, selects the fixed point)
    if !b.raw.econ.weeder.unwrap_or(false) {
        return None;
    }
    let c = o.pos.cell();
    // spec: S-0504 claim 12 (the cell displaced by (2, 1) cells along x and y from the top-left foundation cell, wrapping signed 32-bit arithmetic)
    let cell = glasnost_map::CellXY::new(c.x.wrapping_add(2), c.y.wrapping_add(1));
    // spec: S-0504 claim 12 (its centre, at the yard's own world height)
    let p = Pos::centre(cell, o.pos.z);
    Some(p)
}

/// The depot's waiting gate (S-0504 claim 8): the docked vehicle stands
/// still and is strictly closer to its docking point `dock` than the
/// starting distance. On success the settling processing comes
/// [`SETTLING_DELAY`] ticks later; otherwise the depot's delay *D*.
pub fn waiting_gate(w: &World, vehicle: ObjectId, dock: Pos) -> bool {
    let Some(o) = w.obj(vehicle) else {
        return false;
    };
    let still = !crate::occupancy::in_progress(w, vehicle) && !o.body.turning(w.tick);
    // spec: S-0504 claim 8 (stands still and is closer than the starting distance, strictly)
    still && depot_distance(o.pos, dock) < starting_distance(w, vehicle)
}
