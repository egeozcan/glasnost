//! Transports (S-0507): who may board a vehicle or aircraft transport and
//! how capacity is counted (claims 1, 2 and 4, F1), the loading exchange
//! (claim 5), boarding (claim 6), the passenger order (claim 3), a vehicle
//! transport's unloading phases (claims 7 to 12, F2, F3, table U), an air
//! transport's release (claim 13) and what an open-topped transport passes
//! to its passengers (claims 14 and 15).
//!
//! State: [`TransportState`] on every object. On a transport, its
//! passengers (position 1 first) and the kept count fixed when an unload
//! starts; on a passenger, the transport it rides in. A vehicle
//! transport's unload phase is its *unload* behaviour's progress count,
//! which every behaviour start sets to 0, settling (S-0215 claim 3).
//!
//! Interim rules tied to requests: where a let-out passenger is put
//! (Q-0700), the shore search (Q-0701), passengers of a transport that
//! leaves play without a rule for them (Q-0702), the occupant of claim 9's
//! alliance test (Q-0703), the tunnel edge of claim 7 (Q-0704), an air
//! transport's airborne unload (Q-0705) and the delay of the *finished*
//! phase (Q-0706).

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_num::F64;
use glasnost_num::geom::{Facing, n_step};
use serde::{Deserialize, Serialize};

use crate::occupancy::{self, Outcome};
use crate::regions::STEPS;
use crate::rules::{Behaviour, Kind};
use crate::world::{Layer, ObjectId, Pos, Target, World};
use glasnost_rules::sim::{LocomotionClass, MovementZone};

/// Transport state of an object (S-0507 claims 3, 6 and 11).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportState {
    /// A transport's passengers in the passenger order, position 1 first
    /// (claim 3).
    pub passengers: Vec<ObjectId>,
    /// The number of passengers a vehicle transport keeps inside during
    /// the present unload, decided when it leaves settling (claim 11).
    pub kept: i32,
    /// The transport a passenger rides in (claim 6).
    pub riding: Option<ObjectId>,
}

/// The unloading phases of a vehicle transport (table U), kept in its
/// *unload* behaviour's progress count.
pub mod phase {
    /// Settling.
    // nocite: our own numbering of table U's four phases, in table order (0 is the start value of S-0215 claim 3)
    pub const SETTLING: i32 = 0;
    /// Turning.
    // nocite: our own numbering of table U's four phases, in table order
    pub const TURNING: i32 = 1;
    /// Letting out.
    // nocite: our own numbering of table U's four phases, in table order
    pub const LETTING_OUT: i32 = 2;
    /// Finished.
    // nocite: our own numbering of table U's four phases, in table order
    pub const FINISHED: i32 = 3;
}

/// The settling wait (claim 8).
// spec: S-0507 Constants (settling wait 10 ticks)
const SETTLE_WAIT: i32 = 10;
/// The turning step (claims 8 and 12).
// spec: S-0507 Constants (turning step 1 tick)
const TURN_STEP: i32 = 1;
/// The infantry positions of a cell (S-0208 table P).
// spec: S-0208 table P (five standing positions, 0 the centre)
const POSITIONS: u8 = 5;
/// The step directions (S-0204 Terms, eight directions).
// spec: S-0507 F3 (directions counted modulo 8)
const DIRECTIONS: usize = 8;
/// The straight step directions of claim 7.
// spec: S-0507 claim 7 (straight neighbours: step directions 0, 2, 4 and 6)
const STRAIGHT: [usize; 4] = [0, 2, 4, 6];
/// The open-cell base score (claim 9, F2).
// spec: S-0507 Constants (open-cell base score 128; −128 for a closed cell)
const OPEN: i32 = 128;
/// The south penalty (claim 9, F2).
// spec: S-0507 Constants (south penalty 100 score points, step direction 4)
const SOUTH_PENALTY: i32 = 100;
/// The step direction that takes the south penalty (F2).
// spec: S-0507 F2 (P(d) = 100 when d = 4)
const SOUTH: usize = 4;
/// Facing units per step direction in 256 steps (F2).
// spec: S-0507 Constants (facing per step direction: 32 of 256 steps)
const PER_DIRECTION: i32 = 32;
/// Facing units of the 65536-step facing per step direction (F2: the
/// 8-step facing ((d + 4) mod 8) × 8192).
// spec: S-0507 F2 (((d + 4) mod 8) × 8192)
const DIRECTION_FACING: i32 = 8192;

/// A docking answer (S-0214 Terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    /// Accepts.
    Accepts,
    /// Refuses.
    Refuses,
    /// Neither.
    Neither,
}

/// The reply of the loading exchange (claim 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reply {
    /// Acceptance: the passenger stays in *enter*.
    Accept,
    /// The transport takes the requester as its destination (not an
    /// acceptance).
    Approach,
    /// A refusal.
    Refuse,
    /// Neither.
    Neither,
}

fn st(w: &World, id: ObjectId) -> Option<&TransportState> {
    w.obj(id).map(|o| &o.transport)
}

/// The capacity of an object's type (Terms, Import mapping): 0 for a type
/// that is not a transport.
pub fn capacity(w: &World, id: ObjectId) -> i32 {
    w.obj(id)
        .filter(|o| matches!(o.kind, Kind::Vehicle | Kind::Aircraft))
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        .map_or(0, |u| u.passengers)
}

/// Whether an object is a transport (Terms): a vehicle or aircraft whose
/// type has a capacity above 0.
pub fn is_transport(w: &World, id: ObjectId) -> bool {
    // spec: S-0507 Terms (a transport: capacity above 0)
    capacity(w, id) > 0
}

/// Whether an object is a vehicle transport (Terms).
pub fn vehicle_transport(w: &World, id: ObjectId) -> bool {
    is_transport(w, id) && w.obj(id).is_some_and(|o| o.kind == Kind::Vehicle)
}

/// A transport's passengers, position 1 first (claim 3).
pub fn passengers(w: &World, id: ObjectId) -> Vec<ObjectId> {
    st(w, id).map(|s| s.passengers.clone()).unwrap_or_default()
}

/// The number of passengers a transport holds now.
pub fn count(w: &World, id: ObjectId) -> i32 {
    st(w, id).map_or(0, |s| s.passengers.len() as i32)
}

/// The transport an object rides in (claim 6).
pub fn riding(w: &World, id: ObjectId) -> Option<ObjectId> {
    st(w, id).and_then(|s| s.riding)
}

/// Whether an object's type is open-topped (claim 14).
pub fn open_topped(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        .is_some_and(|u| u.open_topped)
}

/// Whether an object rides in an open-topped transport (claim 14): a
/// passenger in play that takes turns inside (S-0203 claim 3).
pub fn open_topped_passenger(w: &World, id: ObjectId) -> bool {
    riding(w, id).is_some_and(|t| open_topped(w, t))
}

/// The open-topped transport an object rides in, if any.
pub fn open_topped_carrier(w: &World, id: ObjectId) -> Option<ObjectId> {
    riding(w, id).filter(|&t| open_topped(w, t))
}

fn size(w: &World, id: ObjectId) -> F64 {
    let mode = w.rules().mode;
    w.obj(id)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        // spec: S-0507 Import mapping (Size absent: 1.0)
        .map_or_else(|| F64::from_i64(1, mode), |u| u.size)
}

/// The load of a vehicle transport (claim 1, F1): each passenger's size is
/// added in turn, the sum rounded at each addition and, with
/// `quirks.transport_load_truncated`, truncated toward zero after it.
pub fn load(w: &World, id: ObjectId) -> F64 {
    let r = w.rules();
    let mode = r.mode;
    let mut l = F64::ZERO;
    for p in passengers(w, id) {
        // spec: S-0507 F1 (L_k = trunc(RZ₅₃(L_{k−1} + s_k)), the rounding of S-0205 claims 4 and 5)
        let sum = l.add(size(w, p), mode);
        l = if r.quirks.transport_load_truncated {
            // spec: S-0507 F1 and Edge cases (quirks.transport_load_truncated: trunc toward zero, S-0205 claim 7)
            F64::from_i64(i64::from(sum.to_i32_common()), mode)
        } else {
            // spec: S-0507 Edge cases (switch off: the double-precision sum, rounded at each addition)
            sum
        };
    }
    l
}

/// Claim 1's two tests for candidate `c` of vehicle transport `t`.
fn fits(w: &World, t: ObjectId, c: ObjectId) -> bool {
    let r = w.rules();
    let mode = r.mode;
    let Some(u) = w.obj(t).and_then(|o| r.unit(o.kind, o.type_index)) else {
        return false;
    };
    let s = size(w, c);
    let cap = F64::from_i64(i64::from(u.passengers), mode);
    // spec: S-0507 claim 1 and F1 (RZ₅₃(L + s) ≤ C and s ≤ S_max, exact comparisons of doubles)
    load(w, t).add(s, mode).le(cap) && s.le(u.size_limit)
}

/// Mind-controlled (claim 4; glossary). Seam: mind control belongs to
/// S-0601, which is not built, so no object is.
pub fn mind_controlled(w: &World, id: ObjectId) -> bool {
    let _ = (w, id);
    false
}

/// A mind controller that controls at least one object now (claim 4).
/// Seam: mind control belongs to S-0601, which is not built.
pub fn controls_objects(w: &World, id: ObjectId) -> bool {
    let _ = (w, id);
    false
}

/// Releases a boarding unit from any mind control that held it (claim 6).
/// Seam: mind control belongs to S-0601, which is not built.
pub fn release_mind_control(w: &mut World, id: ObjectId) {
    let _ = (w, id);
}

/// Resets a boarding unit's gattling spin value (claim 6, S-0609 claim
/// 11). Seam: the spin value belongs to S-0609, which is not built; the
/// stage our engine keeps is reset by [`board`].
pub fn reset_gattling_spin(w: &mut World, id: ObjectId) {
    let _ = (w, id);
}

/// Whether an object is inside a tube (claim 7). Seam: tubes are not
/// built.
pub fn in_tube(w: &World, id: ObjectId) -> bool {
    let _ = (w, id);
    false
}

/// Whether a building is a tank bunker (claims 7 and 8). Seam: tank
/// bunkers belong to the structures specs (S-0508 and its neighbours),
/// which are not built, so no building is one.
pub fn tank_bunker(w: &World, building: ObjectId) -> bool {
    let _ = (w, building);
    false
}

/// Whether a vehicle carries a bomb mark (S-0312 claims 11 and 12): the mark
/// of a fall from a lost deck (S-0511 claim 9) or of a sinking hover
/// vehicle (S-0503 claim 11). Planted bombs belong to the
/// special-abilities specs and are not built: a seam.
pub fn bomb_marked(w: &World, id: ObjectId) -> bool {
    // spec: S-0312 Terms (the bomb mark), S-0511 claims 9 and 10, S-0503 claim 11
    crate::hover::bomb_mark(w, id)
}

/// Disabled (S-0214 Terms): paralysed, or a vehicle playing its death
/// sequence. Seam: neither state is built.
fn disabled(w: &World, id: ObjectId) -> bool {
    let _ = (w, id);
    false
}

/// The docking answer of transport `t` to candidate `c` (claims 1, 2 and
/// 4; S-0214 Terms, *docking answer*).
pub fn answer(w: &World, t: ObjectId, c: ObjectId) -> Answer {
    let (Some(to), Some(co)) = (w.obj(t), w.obj(c)) else {
        return Answer::Neither;
    };
    if !is_transport(w, t) {
        return Answer::Neither;
    }
    // spec: S-0507 Terms (allied for boarding: the transport owner's view, a house allied with itself)
    let allied = w.allied(to.owner, co.owner);
    if to.kind == Kind::Aircraft {
        if !allied {
            // spec: S-0507 claim 2 (a candidate not allied with its owner: neither)
            return Answer::Neither;
        }
        // spec: S-0507 claim 2 (heads: accepts below its capacity, refuses at it)
        return if count(w, t) < capacity(w, t) {
            Answer::Accepts
        } else {
            Answer::Refuses
        };
    }
    let building_here = w
        .cells
        .list(to.pos.cell(), Layer::Ground)
        .iter()
        .any(|&b| w.obj(b).is_some_and(|x| x.kind == Kind::Building));
    // spec: S-0507 claim 4 (neither: not allied, being erased, a building on its cell, a mind-controlled candidate, one with a parasite inside it, a controller of objects)
    if !allied
        || crate::econ::power::being_erased(w, t)
        || building_here
        || mind_controlled(w, c)
        || crate::parasite::carries_parasite(w, c)
        || controls_objects(w, c)
    {
        return Answer::Neither;
    }
    // spec: S-0507 claim 4 (otherwise it accepts or refuses by claim 1)
    if fits(w, t, c) {
        Answer::Accepts
    } else {
        Answer::Refuses
    }
}

fn locomotor_moving(w: &World, id: ObjectId) -> bool {
    if w.obj(id).is_some_and(|o| o.kind == Kind::Aircraft) {
        return !crate::air::stopped(w, id);
    }
    occupancy::in_progress(w, id)
}

/// The loading exchange's reply of transport `t` to requester `p` (claim
/// 5). It may make `p` the transport's contact, give the transport a
/// destination, or call `p` to the transport's cell.
pub fn load_request(w: &mut World, t: ObjectId, p: ObjectId) -> Reply {
    let Some(to) = w.obj(t).cloned() else {
        return Reply::Neither;
    };
    let Some(po) = w.obj(p).cloned() else {
        return Reply::Neither;
    };
    let cap = capacity(w, t);
    let calls = |w: &World| {
        !locomotor_moving(w, t)
            && !to.body.turning(w.tick)
            && !crate::econ::tethered(w, t)
            && !occupancy::moving_or_dest(w, p)
    };
    if to.kind == Kind::Aircraft {
        // spec: S-0507 claim 5 (an air transport accepts below its positive capacity, even while moving; full: neither)
        if cap <= 0 || count(w, t) >= cap {
            return Reply::Neither;
        }
        // spec: S-0507 claim 5 (it calls a passenger not moving toward a destination only while its own locomotor is stopped)
        if !locomotor_moving(w, t) && !occupancy::moving_or_dest(w, p) {
            crate::econ::assign_destination(w, p, Some(Target::Object(t)));
        }
        return Reply::Accept;
    }
    // spec: S-0507 claim 5 (a vehicle transport in move with a destination, or with a moving locomotor, refuses)
    if (to.beh.current == Some(Behaviour::Move) && to.dest.is_some()) || locomotor_moving(w, t) {
        return Reply::Refuse;
    }
    let passes = fits(w, t, p);
    if to.contact != Some(p) {
        // spec: S-0507 claim 5 (a requester other than its current contact must pass claim 1's tests to proceed)
        if !passes {
            return Reply::Refuse;
        }
        if to.contact.is_none()
            && let Some(ob) = w.obj_mut(t)
        {
            // spec: S-0507 claim 5 (with no current contact, an eligible requester becomes its contact)
            ob.contact = Some(p);
        }
    }
    if to.dest.is_none() {
        let water = w
            .cells
            .cell(to.pos.cell())
            .is_some_and(|c| c.land == land::WATER);
        let zone = w.rules().unit(po.kind, po.type_index).map(|u| u.zone);
        let apart = zone.is_some_and(|z| {
            !crate::regions::same_region(w, z, to.pos.cell(), to.layer, po.pos.cell(), po.layer)
        });
        if water || apart {
            // spec: S-0507 claim 5 (no destination, on water or a different reach region for the requester's zone: the transport takes the requester as its destination; not an acceptance)
            crate::econ::assign_destination(w, t, Some(Target::Object(p)));
            return Reply::Approach;
        }
    }
    if !passes {
        // spec: S-0507 claim 5 (an already-contacted requester failing the capacity tests receives neither answer)
        return Reply::Neither;
    }
    if calls(w) {
        // spec: S-0507 claim 5 (stopped, not turning, untethered, the requester not moving toward a destination: it calls the requester to its own cell)
        crate::econ::assign_destination(w, p, Some(Target::Object(t)));
    }
    // spec: S-0507 claim 5 (otherwise it accepts when claim 1's tests pass)
    Reply::Accept
}

/// Whether an infantry unit or vehicle in *enter* is entering a
/// transport: its contact, else the object it remembers entering (its
/// destination), is a transport (S-0215 claim 29).
pub fn entering_transport(w: &World, id: ObjectId) -> bool {
    partner(w, id).is_some_and(|t| is_transport(w, t))
}

fn partner(w: &World, id: ObjectId) -> Option<ObjectId> {
    let o = w.obj(id)?;
    // spec: S-0215 claim 29 (to its contact when present, otherwise to the object it remembers entering)
    o.contact.or(match o.dest {
        Some(Target::Object(k)) => Some(k),
        _ => None,
    })
}

/// Breaks the contact between a requester and a transport, on both sides.
fn break_contact(w: &mut World, p: ObjectId, t: Option<ObjectId>) {
    if let Some(ob) = w.obj_mut(p) {
        ob.contact = None;
    }
    if let Some(t) = t
        && let Some(tb) = w.obj_mut(t)
        && tb.contact == Some(p)
    {
        tb.contact = None;
    }
}

/// The *enter* processing of a would-be passenger (S-0215 claim 29, S-0507
/// claim 5). Returns the delay.
pub fn process_enter(w: &mut World, p: ObjectId) -> i32 {
    let Some(t) = partner(w, p).filter(|&t| w.obj(t).is_some()) else {
        // spec: S-0507 claim 5 (neither contact nor a pending entry partner: fall back to rest)
        break_contact(w, p, None);
        crate::process::fall_back(w, p, false);
        return crate::process::standard_delay(w, p);
    };
    let reply = load_request(w, t, p);
    let tethered = crate::econ::tethered(w, p);
    if reply == Reply::Accept || tethered {
        // spec: S-0507 claim 5 (only acceptance, or an existing tether, keeps the passenger in enter)
        if let Some(ob) = w.obj_mut(p) {
            ob.contact = Some(t);
        }
    } else {
        // spec: S-0507 claim 5 and S-0215 claim 29 (every other answer breaks contact and falls back to rest)
        break_contact(w, p, Some(t));
        crate::process::fall_back(w, p, false);
    }
    // spec: S-0507 claim 5 and S-0215 claim 6 (processing ends with the standard delay)
    crate::process::standard_delay(w, p)
}

/// The moment of boarding (claim 6), at a would-be passenger's arrival
/// update. Returns whether it boarded (and so left the map); a unit turned
/// away goes on with its ordinary arrival.
pub fn arrival(w: &mut World, p: ObjectId) -> bool {
    let Some(o) = w.obj(p).cloned() else {
        return false;
    };
    let Some(Target::Object(t)) = o.dest else {
        return false;
    };
    let Some(to) = w.obj(t).cloned() else {
        return false;
    };
    // spec: S-0507 claim 6 (it boards when it arrives at the transport's cell while the transport is its destination)
    if !is_transport(w, t) || t == p || to.pos.cell() != o.pos.cell() || !to.on_map {
        return false;
    }
    if o.kind == Kind::Vehicle
        && !(o.beh.current == Some(Behaviour::Enter)
            && o.contact.is_some_and(|c| is_transport(w, c)))
    {
        // spec: S-0507 claim 6 (a vehicle boards only while its current behaviour is enter and its contact is a transport)
        return false;
    }
    if !matches!(o.kind, Kind::Infantry | Kind::Vehicle) {
        return false;
    }
    // spec: S-0507 claim 6 (at that moment the transport answers again by claims 1, 2 and 4; neither counts as a refusal)
    if answer(w, t, p) == Answer::Accepts {
        board(w, p, t);
        true
    } else {
        turned_away(w, p);
        false
    }
}

/// A would-be passenger that the transport does not accept (claim 6).
fn turned_away(w: &mut World, p: ObjectId) {
    crate::econ::assign_destination(w, p, None);
    if let Some(ob) = w.obj_mut(p) {
        // spec: S-0507 claim 6 (not accepted: it gives up its destination and queues guard)
        ob.beh.queue(Some(Behaviour::Guard));
    }
    // spec: S-0507 claim 6 (it scatters even if it would otherwise not: a forced request)
    crate::scatter::request(
        w,
        p,
        crate::scatter::ScatterRequest {
            threat: None,
            forced: true,
            anywhere: false,
        },
    );
}

/// Boards passenger `p` into transport `t` (claim 6).
pub fn board(w: &mut World, p: ObjectId, t: ObjectId) {
    // spec: S-0507 claim 6 (if the boarding unit is itself an open-topped transport, its own passengers lose their targets)
    clear_passenger_targets(w, p);
    if let Some(ob) = w.obj_mut(p) {
        // spec: S-0507 claim 6 (it forgets the target it had stored for later use)
        if let Some((_, d)) = ob.beh.aside {
            ob.beh.aside = Some((None, d));
        }
        // spec: S-0507 claim 6 (it resets its gattling stage to 0, S-0609 claim 11)
        ob.combat.gattling_stage = 0;
    }
    reset_gattling_spin(w, p);
    release_mind_control(w, p);
    // spec: S-0507 claim 6 and S-0203 claim 3 (it leaves the map; it leaves play)
    crate::objects::take_inside(w, p);
    let pos = w.obj(t).map(|o| o.pos);
    let only = count(w, t) == 0;
    if let Some(ob) = w.obj_mut(p) {
        // spec: S-0507 claim 6 (it leaves the bridge-deck layer if it was on it)
        ob.layer = Layer::Ground;
        if let Some(pos) = pos {
            ob.pos = pos;
        }
        ob.transport.riding = Some(t);
        // spec: glossary contact (contact lasts until the exchange ends: boarding ends it)
        ob.contact = None;
    }
    if let Some(tb) = w.obj_mut(t) {
        // spec: S-0507 claim 3 (the boarding unit goes to position 1; the others move one position back)
        tb.transport.passengers.insert(0, p);
        if tb.contact == Some(p) {
            // spec: glossary contact (the exchange has ended)
            tb.contact = None;
        }
    }
    if open_topped(w, t) {
        // spec: S-0203 claim 3 (boarding an open-topped transport: out of the update order and back in at its end)
        w.update.enter(p);
    }
    if only && gunner(w, t) {
        // spec: S-0308 claim 6 (an object enters a gunner vehicle and is then its only passenger: its weapon number becomes the passenger type's mode index)
        let m = mode_index(w, p);
        crate::combat::weapon::set_weapon_number(w, t, m);
    }
}

fn gunner(w: &World, t: ObjectId) -> bool {
    w.obj(t).is_some_and(|o| o.kind == Kind::Vehicle)
        && w.obj(t)
            .and_then(|o| w.rules().unit(o.kind, o.type_index))
            .is_some_and(|u| u.combat.gunner)
}

fn mode_index(w: &World, p: ObjectId) -> i32 {
    w.obj(p)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        .map_or(0, |u| u.combat.ifv_mode)
}

/// Every passenger of an open-topped transport loses its target (claim 15:
/// the transport leaves the map; claim 6: it boards another transport).
pub fn clear_passenger_targets(w: &mut World, t: ObjectId) {
    if !open_topped(w, t) {
        return;
    }
    for p in passengers(w, t) {
        if let Some(ob) = w.obj_mut(p) {
            crate::spawn::clear_own_target(ob);
            ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
        }
    }
}

/// An object leaves the map for any reason (claim 15): an open-topped
/// transport's passengers lose their targets.
pub fn leaving_map(w: &mut World, t: ObjectId) {
    // spec: S-0507 claim 15 (every passenger loses its target when the transport leaves the map for any reason)
    clear_passenger_targets(w, t);
}

/// An order set an open-topped transport's target (claim 15; S-0213 table
/// T3 (f)): it gives the target to its passengers when it is not nothing.
pub fn hand_down_target(w: &mut World, t: ObjectId) {
    if !open_topped(w, t) {
        return;
    }
    let Some(target) = w.obj(t).and_then(|o| o.target) else {
        return;
    };
    for p in passengers(w, t) {
        if let Some(ob) = w.obj_mut(p) {
            // spec: S-0213 table T3 (f) (an open-topped transport gives the same target to its passengers when the target is not nothing)
            crate::spawn::set_own_target(ob, Some(target));
        }
    }
}

/// Keeps a transport's passengers at its position (claim 15): whenever the
/// transport moves to a new position, its passengers' positions become
/// its position at once.
pub fn sync(w: &mut World, t: ObjectId) {
    let Some(o) = w.obj(t) else { return };
    if o.transport.passengers.is_empty() {
        return;
    }
    let pos = o.pos;
    for p in passengers(w, t) {
        if let Some(ob) = w.obj_mut(p) {
            // spec: S-0507 claim 15 (the passengers' positions become the transport's position)
            ob.pos = pos;
        }
        sync(w, p);
    }
}

/// Whether a vehicle transport has deploy readiness for unloading (claim 7;
/// S-0214 Terms).
pub fn may_unload(w: &World, t: ObjectId) -> bool {
    let Some(o) = w.obj(t) else { return false };
    let c = o.pos.cell();
    let Some(cell) = w.cells.cell(c) else {
        return false;
    };
    // spec: S-0507 claim 7 (a tunnel cell; Q-0704 interim: only cells of the tunnel land type, no tunnel edge)
    if cell.land == land::TUNNEL {
        return false;
    }
    // spec: S-0507 claim 7 (disabled, or inside a tube)
    if disabled(w, t) || in_tube(w, t) {
        return false;
    }
    // spec: S-0507 claim 7 (on a bridge)
    if o.layer == Layer::Deck {
        return false;
    }
    // spec: S-0507 claim 7 (its own cell or a straight neighbour, step directions 0, 2, 4 and 6, carries a bridge deck)
    let deck_near = std::iter::once((0, 0))
        .chain(STRAIGHT.iter().map(|&d| STEPS[d]))
        .any(|(dx, dy)| w.cells.is_bridge(CellXY::new(c.x + dx, c.y + dy)));
    if deck_near {
        return false;
    }
    // spec: S-0507 claim 7 (water: refused for every player house; neutral and special houses keep the exemption)
    let player = o.owner.is_some_and(|h| {
        w.houses
            .get(usize::from(h.0))
            .is_some_and(|x| x.role == crate::world::HouseRole::Player)
    });
    if cell.land == land::WATER && player {
        return false;
    }
    // spec: S-0507 claim 7 (a building's cell, unless that building is a tank bunker)
    let building = w
        .cells
        .list(c, Layer::Ground)
        .iter()
        .any(|&b| w.obj(b).is_some_and(|x| x.kind == Kind::Building) && !tank_bunker(w, b));
    !building
}

/// S-0214 table VD row VD7: a transport carrying passengers, not on a
/// bridge, whose first passenger cannot move on the land type of its cell
/// (F4, the land not being water), or that is in the air while its type
/// does not hover.
pub fn passenger_cannot_leave(w: &World, t: ObjectId) -> bool {
    let Some(o) = w.obj(t) else { return false };
    let Some(&p) = o.transport.passengers.first() else {
        return false;
    };
    if o.layer == Layer::Deck {
        return false;
    }
    let r = w.rules();
    let land_type = w.cells.cell(o.pos.cell()).map(|c| c.land);
    let class = w
        .obj(p)
        .and_then(|po| r.unit(po.kind, po.type_index))
        .and_then(|u| u.class);
    let stuck = match (land_type, class) {
        // spec: S-0214 F4 (m < the double nearest 0.01, m converted exactly from single precision; water excluded)
        (Some(l), Some(c)) if l != land::WATER => r
            .multiplier(l, c)
            .to_f64()
            .lt(crate::consts::LOW_MULTIPLIER),
        _ => false,
    };
    let hovers = r
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.locomotor == crate::rules::Locomotor::Hover);
    // spec: S-0214 table VD row VD7 (in the air while its type does not hover)
    let airborne = crate::combat::weapon::in_air(w, t) && !hovers;
    stuck || airborne
}

/// σ of F2: the signed value of a 256-step facing.
fn sigma(v: i32) -> i32 {
    // spec: S-0507 F2 (σ(v) = v when v < 128, else v − 256)
    if v < 128 { v } else { v - 256 }
}

/// The rear facing r of F2 in 256 steps.
pub fn rear_256(f: Facing) -> i32 {
    // spec: S-0507 F2 (r = ⌊(⌊((f + 32767) mod 65536) / 128⌋ + 1) / 2⌋ mod 256)
    (((i32::from(f) + 32767).rem_euclid(65536) / 128 + 1) / 2).rem_euclid(256)
}

/// The rear direction q of F3.
pub fn rear_direction(f: Facing) -> usize {
    // spec: S-0507 F3 (q = ⌊(⌊((f + 32767) mod 65536) / 4096⌋ + 1) / 2⌋ mod 8)
    (((i32::from(f) + 32767).rem_euclid(65536) / 4096 + 1) / 2).rem_euclid(8) as usize
}

/// Whether neighbour cell `c` is open for claim 9's +128.
fn open_cell(w: &World, t: ObjectId, c: CellXY) -> bool {
    let Some(o) = w.obj(t) else { return false };
    let Some(cell) = w.cells.cell(c) else {
        return false;
    };
    let r = w.rules();
    // spec: S-0507 claim 9 (its land type lets infantry walk on it: the foot land multiplier is not 0)
    if r.multiplier(cell.land, LocomotionClass::Foot) == glasnost_num::F32::ZERO {
        return false;
    }
    // spec: S-0507 claim 9 (no vehicle or building occupies it)
    let blocked = w.cells.list(c, Layer::Ground).iter().any(|&x| {
        w.obj(x)
            .is_some_and(|x| matches!(x.kind, Kind::Vehicle | Kind::Building))
    });
    if blocked {
        return false;
    }
    let rec = w.cells.occ(c, Layer::Ground).copied().unwrap_or_default();
    // spec: S-0507 claim 9 (not all of its infantry positions are taken); S-0208 table P (five positions)
    if (0..POSITIONS).all(|p| rec.taken(p)) {
        return false;
    }
    // spec: S-0507 claim 9 (the occupant used for the alliance test is allied from the transport owner's side); Q-0703 (interim: the cell's infantry house)
    rec.house.is_none_or(|h| w.allied(o.owner, Some(h)))
}

/// Chooses the unloading cell (claim 9, F2): the step direction, or none.
pub fn choose_direction(w: &World, t: ObjectId) -> Option<usize> {
    let o = w.obj(t)?;
    let c = o.pos.cell();
    let r = rear_256(o.body.value(w.tick));
    let wrap = !w.rules().quirks.unload_facing_gap_no_wrap;
    let mut best: Option<(usize, i32)> = None;
    for (d, &(dx, dy)) in STEPS.iter().enumerate() {
        let n = CellXY::new(c.x + dx, c.y + dy);
        let b = if open_cell(w, t, n) { OPEN } else { -OPEN };
        let dir = PER_DIRECTION * d as i32;
        let gap = if wrap {
            // spec: S-0507 Edge cases (switch off: the signed facing difference, a gap of at most 128)
            ((dir - r + 128).rem_euclid(256) - 128).abs()
        } else {
            // spec: S-0507 F2 and Edge cases (quirks.unload_facing_gap_no_wrap: |σ(32d) − σ(r)|, not reduced modulo 256)
            (sigma(dir) - sigma(r)).abs()
        };
        let p = if d == SOUTH { SOUTH_PENALTY } else { 0 };
        // spec: S-0507 F2 (score(d) = B(d) − gap − P(d))
        let score = b - gap - p;
        // spec: S-0507 F2 (the smallest d with the largest score)
        if best.is_none_or(|(_, s)| score > s) {
            best = Some((d, score));
        }
    }
    // spec: S-0507 F2 (used only if that score is above 0)
    best.filter(|&(_, s)| s > 0).map(|(d, _)| d)
}

/// Whether a vehicle transport is letting its passengers out under its own
/// *unload* processing (table U, phase letting out; S-0215 claim 11).
pub fn letting_out(w: &World, t: ObjectId) -> bool {
    w.obj(t).is_some_and(|o| {
        o.beh.current == Some(Behaviour::Unload) && o.beh.progress == phase::LETTING_OUT
    }) && vehicle_transport(w, t)
}

/// The *unload* processing of a vehicle transport (claims 8 to 12, table
/// U). Returns the delay.
pub fn process_unload(w: &mut World, t: ObjectId) -> i32 {
    let Some(o) = w.obj(t).cloned() else {
        return crate::process::standard_delay(w, t);
    };
    match o.beh.progress {
        phase::SETTLING => settle(w, t),
        phase::TURNING => {
            if o.body.turning(w.tick) {
                // spec: S-0507 claim 12 and table U (body still turning: standard delay)
                crate::process::standard_delay(w, t)
            } else {
                if let Some(ob) = w.obj_mut(t) {
                    ob.beh.progress = phase::LETTING_OUT;
                }
                // spec: S-0507 claim 12 and table U (body stopped: letting out, 1 tick)
                TURN_STEP
            }
        }
        phase::LETTING_OUT => {
            let kept = o.transport.kept;
            if count(w, t) <= kept {
                if let Some(ob) = w.obj_mut(t) {
                    ob.beh.progress = phase::FINISHED;
                }
                // spec: S-0507 claim 11 and table U (no more than kept: finished, standard delay)
                return crate::process::standard_delay(w, t);
            }
            let_out(w, t);
            // spec: S-0507 claim 10 (each processing ends with the standard delay)
            crate::process::standard_delay(w, t)
        }
        _ => {
            if let Some(ob) = w.obj_mut(t) {
                // spec: S-0507 claim 11 and table U (finished: queue guard with the allowance to interrupt movement, S-0215 claim 11)
                ob.beh.queue(Some(Behaviour::Guard));
                ob.beh.allowance = true;
            }
            // spec: S-0507 table U; Q-0706 (interim: the finished processing ends with the standard delay)
            crate::process::standard_delay(w, t)
        }
    }
}

fn settle(w: &mut World, t: ObjectId) -> i32 {
    let Some(o) = w.obj(t).cloned() else {
        return crate::process::standard_delay(w, t);
    };
    // spec: S-0507 claim 8 and table U (locomotor moving: wait 10 ticks)
    if occupancy::in_progress(w, t) {
        return SETTLE_WAIT;
    }
    let water = w
        .cells
        .cell(o.pos.cell())
        .is_some_and(|c| c.land == land::WATER);
    if water && o.dest.is_none() {
        // spec: S-0507 claim 8 and table U (on water with no destination: a nearby wheeled-standable location as destination; wait 10 ticks); Q-0701 (interim search)
        if let Some(c) = shore(w, t) {
            crate::econ::assign_destination(w, t, Some(Target::Cell(c)));
            // A new destination starts a new path search (S-0212 claim 20).
            if let Some(ob) = w.obj_mut(t) {
                ob.path.steps.clear();
                ob.path.retry_at = 0;
            }
            return SETTLE_WAIT;
        }
    }
    let d = choose_direction(w, t);
    let n = count(w, t);
    let Some(d) = d.filter(|_| n > 0) else {
        if let Some(ob) = w.obj_mut(t) {
            // spec: S-0507 claim 8 and table U (no passengers or no cell: queue guard, standard delay; the unload ends)
            ob.beh.queue(Some(Behaviour::Guard));
        }
        return crate::process::standard_delay(w, t);
    };
    let u = w.rules().unit(o.kind, o.type_index).cloned();
    // spec: S-0507 claim 11 (turret count above 0 keeps one passenger, unless exactly one is aboard when the unload starts)
    let kept = i32::from(u.as_ref().is_some_and(|u| u.turret_count > 0) && n != 1);
    let f = o.body.value(w.tick);
    let to: Facing = if u.as_ref().is_some_and(|u| u.train) {
        // spec: S-0507 claim 8 (a train turns to its present facing rounded to 8 steps, S-0204 claim 11 and F4)
        (n_step(f, 8) as i32 * DIRECTION_FACING) as Facing
    } else {
        // spec: S-0507 F2 (the transport turns to the 8-step facing ((d + 4) mod 8) × 8192)
        ((((d + 4) % 8) as i32) * DIRECTION_FACING) as Facing
    };
    let omega = u.as_ref().map_or(0, |u| u.turn_rate);
    let now = w.tick;
    if let Some(ob) = w.obj_mut(t) {
        ob.transport.kept = kept;
        ob.body.turn_to(to, now, omega);
        ob.beh.progress = phase::TURNING;
    }
    // spec: S-0507 claim 8 and table U (a cell is chosen: turning, processed again after 1 tick)
    TURN_STEP
}

/// The shore search of claim 8 (Q-0701 interim): the nearby-cell search
/// of S-0212 claim 25 from the transport's cell for the wheel class in
/// zone normal, with no reach region.
fn shore(w: &mut World, t: ObjectId) -> Option<CellXY> {
    crate::regions::ensure(w);
    let c = w.obj(t)?.pos.cell();
    crate::path::nearby(
        w,
        c,
        crate::path::NearbyUse::Settings(crate::path::NearbySettings {
            class: Some(LocomotionClass::Wheel),
            zone: MovementZone::Normal,
            reach: None,
            deck: false,
            height: true,
            reference: None,
            no_bridges: false,
            no_overlay: false,
        }),
    )
}

/// Whether passenger `p` could enter cell `c` from the transport (claim
/// 10): the entry outcome is clear.
fn enters(w: &World, p: ObjectId, c: CellXY, reference: i32) -> bool {
    let Some(o) = w.obj(p) else { return false };
    if w.cells.cell(c).is_none() {
        return false;
    }
    // spec: S-0507 claim 10 (the movement specs' entry test finds the cell open, with nothing in the way: clear, S-0209 claims 4 and 5)
    let out = match o.kind {
        Kind::Infantry => occupancy::infantry_outcome(w, p, c, reference),
        _ => occupancy::vehicle_outcome(w, p, c, reference),
    };
    out == Outcome::Clear
}

/// Lets passenger position 1 out (claim 10). Returns whether it got out.
fn let_out(w: &mut World, t: ObjectId) -> bool {
    let Some(o) = w.obj(t).cloned() else {
        return false;
    };
    let Some(&p) = o.transport.passengers.first() else {
        return false;
    };
    let pos = o.pos;
    if let Some(ob) = w.obj_mut(p) {
        ob.pos = pos;
    }
    let c = pos.cell();
    let reference = occupancy::reference_level(w, t);
    let q = rear_direction(o.body.value(w.tick));
    let deck = |w: &World, n: CellXY| w.cells.is_bridge(n);
    let step = |c: CellXY, d: usize| CellXY::new(c.x + STEPS[d].0, c.y + STEPS[d].1);
    let mut exit: Option<(usize, CellXY, CellXY)> = None;
    // spec: S-0507 claim 10 and F3 (first round: all 8 directions from the rear, clockwise; both the neighbour and the far cell)
    for k in 0..DIRECTIONS {
        let d = (q + k) % DIRECTIONS;
        let n = step(c, d);
        // spec: S-0507 claim 10 (a neighbouring cell that carries a bridge deck is passed over)
        if deck(w, n) {
            continue;
        }
        let far = step(n, d);
        if enters(w, p, n, reference) && enters(w, p, far, reference) {
            exit = Some((d, n, far));
            break;
        }
    }
    if exit.is_none() {
        // spec: S-0507 claim 10 and Edge cases (second round: the 7 directions after the rear, quirks.unload_second_round_skips_rear; off: the rear first)
        let first = usize::from(w.rules().quirks.unload_second_round_skips_rear);
        for k in first..DIRECTIONS {
            let d = (q + k) % DIRECTIONS;
            let n = step(c, d);
            if deck(w, n) {
                continue;
            }
            if enters(w, p, n, reference) {
                exit = Some((d, n, n));
                break;
            }
        }
    }
    let placed = exit.is_some_and(|(d, n, dest)| put_out(w, t, p, n, Some((d, dest))));
    if !placed {
        // spec: S-0507 claim 10 (on failure the passenger stays inside at position 1; the next processing tries it again)
        if gunner(w, t) && w.rules().quirks.unload_failed_exit_gunner_handover {
            // spec: S-0507 claim 10 and S-0308 claim 6 (quirks.unload_failed_exit_gunner_handover: the weapon number becomes the passenger's mode index)
            let m = mode_index(w, p);
            crate::combat::weapon::set_weapon_number(w, t, m);
        }
    }
    placed
}

/// Puts passenger `p` of transport `t` on the map in cell `n` (claims 10
/// and 13; Q-0700 interim placement). With `exit`, the exit direction and
/// the cell it then moves to. Returns whether it was placed.
fn put_out(
    w: &mut World,
    t: ObjectId,
    p: ObjectId,
    n: CellXY,
    exit: Option<(usize, CellXY)>,
) -> bool {
    let Some(po) = w.obj(p).cloned() else {
        return false;
    };
    let Some(to) = w.obj(t).cloned() else {
        return false;
    };
    let layer = Layer::Ground;
    let (x, y) = if po.kind == Kind::Infantry {
        // spec: S-0507 claim 10 (an infantry unit in a free infantry position of that cell); Q-0700 (interim: the first free of the five positions, centre first, S-0208 table P)
        let rec = w.cells.occ(n, layer).copied().unwrap_or_default();
        let Some(k) = (0..POSITIONS).find(|&k| !rec.taken(k)) else {
            return false;
        };
        let (u, v) = crate::walk::standing_point(k);
        (256 * n.x + u, 256 * n.y + v) // spec: S-0208 table P (a standing position within the cell)
    } else {
        // spec: S-0507 claim 10 (a vehicle at a nearby location where it can stand, searched from that centre); Q-0700 (interim: the centre of the serving cell)
        (256 * n.x + 128, 256 * n.y + 128) // spec: S-0207 Terms (a cell's centre: 128 leptons in from its corner)
    };
    let z = w.surface_at(x, y);
    // spec: S-0507 claim 10 (facing the exit direction); claim 13 has none, the transport's facing is kept (Q-0700 interim)
    let facing: Facing = match exit {
        Some((d, _)) => (d as i32 * DIRECTION_FACING) as Facing,
        None => to.body.value(w.tick),
    };
    // spec: S-0507 claims 10 and 3 (P is no longer a passenger; it leaves from position 1)
    if let Some(tb) = w.obj_mut(t) {
        tb.transport.passengers.retain(|&x| x != p);
    }
    if let Some(ob) = w.obj_mut(p) {
        ob.transport.riding = None;
        ob.layer = layer;
    }
    crate::objects::place(w, p, Pos::new(x, y, z), facing);
    if open_topped(w, t)
        && po.owner != to.owner
        && let Some(ob) = w.obj_mut(p)
    {
        // spec: S-0507 claims 10 and 15 (open-topped, and P's owner differs from the transport's: P loses its target)
        crate::spawn::clear_own_target(ob);
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
    }
    if let Some((_, dest)) = exit {
        if let Some(ob) = w.obj_mut(p) {
            // spec: S-0507 claim 10 (P queues move, to the far cell in the first round or the neighbour cell in the second)
            ob.beh.queue(Some(Behaviour::Move));
        }
        crate::econ::assign_destination(w, p, Some(Target::Cell(dest)));
    }
    // spec: S-0614 claim 12 (the host, a transport, lets a passenger out)
    crate::parasite::host_unloading(w, t);
    if count(w, t) == 0 && gunner(w, t) {
        // spec: S-0308 claim 6 (when its last passenger leaves, its weapon number becomes 0)
        crate::combat::weapon::set_weapon_number(w, t, 0);
        temporal_hand_back(w, t, p);
    }
    true
}

/// The temporal beam hand-back to the last passenger of a gunner vehicle
/// (S-0308 claim 6). Seam: temporal beams belong to S-0514, not built.
pub fn temporal_hand_back(w: &mut World, t: ObjectId, p: ObjectId) {
    let _ = (w, t, p);
}

/// The *unload* processing of an air transport (claim 13). Returns the
/// delay.
pub fn process_air_unload(w: &mut World, t: ObjectId) -> i32 {
    let Some(o) = w.obj(t).cloned() else {
        return crate::process::standard_delay(w, t);
    };
    let carryall = w
        .rules()
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.raw.carryall.unwrap_or(false));
    if carryall {
        // spec: S-0507 claim 13 (a carrier aircraft carrying a vehicle uses different unloading, out of scope)
        w.note("S-0507 claim 13: a carrier aircraft's unloading is not this spec's");
        return crate::process::standard_delay(w, t);
    }
    if count(w, t) == 0 {
        // spec: S-0507 claim 13 (no passengers left: it falls back to rest)
        crate::process::fall_back(w, t, false);
        return crate::process::standard_delay(w, t);
    }
    let here = o.pos.cell();
    let at_dest = match o.dest {
        None => true,
        Some(d) => crate::process::target_pos(w, d).cell() == here,
    };
    if crate::air::on_ground(w, t) && at_dest {
        if !crate::econ::tethered(w, t) {
            // spec: S-0507 claim 13 (landed at its destination or with none: release position 1 at its own cell, unless tethered); Q-0700 (interim placement; a failure keeps it inside)
            let p = o.transport.passengers[0];
            put_out(w, t, p, here, None);
        }
        // spec: S-0507 claim 13 (passenger-release processing ends with the standard delay)
        return crate::process::standard_delay(w, t);
    }
    if !crate::air::on_ground(w, t) && o.dest.is_none() {
        let dropship = w
            .rules()
            .unit(o.kind, o.type_index)
            .is_some_and(|u| u.dropship);
        if dropship {
            // spec: S-0507 claim 13 (in the air with no destination, a dropship type searches for a landing spot); Q-0705 (interim: the landing spot of S-0501 claim 41)
            let s = crate::air::landing_spot(w, t);
            crate::econ::assign_destination(w, t, s.map(Target::Cell));
        }
    }
    // Q-0705 (interim: an airborne air transport otherwise waits a standard delay while its flight approaches)
    w.note("Q-0705: an airborne air transport's unload transitions are interim");
    crate::process::standard_delay(w, t)
}

/// Every passenger of transport `t` dies with it, credited by `hit`, one
/// after another in the passenger order, a passenger's own passengers
/// before it (S-0312 claims 5, 12 and 16).
pub fn kill_passengers(w: &mut World, t: ObjectId, hit: &crate::combat::damage::Hit) {
    for p in passengers(w, t) {
        if let Some(tb) = w.obj_mut(t) {
            tb.transport.passengers.retain(|&x| x != p);
        }
        if let Some(ob) = w.obj_mut(p) {
            ob.transport.riding = None;
        }
        // spec: S-0312 claim 5 (the passengers of a passenger die before it does)
        kill_passengers(w, p, hit);
        crate::combat::death::passenger_death(w, p, hit);
    }
}

/// Removes every passenger of transport `t` without a death (Q-0702
/// interim): a transport that leaves play by a path no claim gives its
/// passengers' fate for (a sale, a sinking or crash that ends without
/// them, any other removal) takes them out of play with it.
pub fn remove_passengers(w: &mut World, t: ObjectId) {
    for p in passengers(w, t) {
        if let Some(tb) = w.obj_mut(t) {
            tb.transport.passengers.retain(|&x| x != p);
        }
        if let Some(ob) = w.obj_mut(p) {
            ob.transport.riding = None;
        }
        // Q-0702 (interim: removed with the transport, nested passengers first)
        remove_passengers(w, p);
        crate::objects::remove(w, p);
    }
}

/// An object is removed from the game (Q-0702 interim): a removed
/// passenger leaves its transport's passenger order, and a removed
/// transport's passengers are removed with it.
pub fn removing(w: &mut World, id: ObjectId) {
    if let Some(t) = riding(w, id)
        && let Some(tb) = w.obj_mut(t)
    {
        tb.transport.passengers.retain(|&x| x != id);
    }
    if let Some(ob) = w.obj_mut(id) {
        ob.transport.riding = None;
    }
    remove_passengers(w, id);
}

/// The escape of a destroyed vehicle's passengers (S-0312 claim 12, last
/// row): each in turn gets out at the vehicle's horizontal position, or
/// dies instead.
pub fn escape(w: &mut World, t: ObjectId, hit: &crate::combat::damage::Hit) {
    let Some(to) = w.obj(t).cloned() else { return };
    let c = to.pos.cell();
    // spec: S-0312 claim 12 (at the height of the ground surface at the centre of the vehicle's cell)
    let z = w.surface_at(256 * c.x + 128, 256 * c.y + 128); // spec: S-0207 Terms (a cell's centre: 128 leptons in)
    // spec: S-0312 claim 12 (the vehicle's body facing rounded to the nearest of 256 steps, S-0204 F4 with n = 256)
    let facing = (n_step(to.body.value(w.tick), 256) * 256) as Facing;
    let reference = occupancy::reference_level(w, t);
    // Q-0707 (interim: the destroyed vehicle itself is not counted by its passengers' entry test; it is out of its cell's list while they get out)
    let at = w.cells.list(c, to.layer).iter().position(|&x| x == t);
    w.cells.leave(c, to.layer, t);
    for p in passengers(w, t) {
        if let Some(tb) = w.obj_mut(t) {
            tb.transport.passengers.retain(|&x| x != p);
        }
        if let Some(ob) = w.obj_mut(p) {
            ob.transport.riding = None;
            ob.pos = Pos::new(to.pos.x, to.pos.y, z);
            ob.layer = Layer::Ground;
        }
        let kind = w.obj(p).map(|o| o.kind);
        let out = match kind {
            Some(Kind::Infantry) => occupancy::infantry_outcome(w, p, c, reference),
            Some(_) => occupancy::vehicle_outcome(w, p, c, reference),
            None => continue,
        };
        let free = (0..POSITIONS).any(|k| {
            // spec: S-0208 table P (five standing positions)
            w.cells.occ(c, Layer::Ground).is_none_or(|r| !r.taken(k))
        });
        let placeable = kind != Some(Kind::Infantry) || free;
        // spec: S-0312 claim 12 (it dies instead when the killing damage ignores defences, when its entry outcome is other than clear or moving blocker, or when it cannot be placed)
        if hit.ignore_defences
            || !matches!(out, Outcome::Clear | Outcome::MovingBlocker)
            || !placeable
        {
            kill_passengers(w, p, hit);
            crate::combat::death::passenger_death(w, p, hit);
            continue;
        }
        crate::objects::place(w, p, Pos::new(to.pos.x, to.pos.y, z), facing);
        // spec: S-0312 claim 12 (a scatter request with no threat point, forced, not move-anywhere)
        crate::scatter::request(
            w,
            p,
            crate::scatter::ScatterRequest {
                threat: None,
                forced: true,
                anywhere: false,
            },
        );
        let player = w.obj(p).and_then(|o| o.owner).is_some_and(|h| {
            w.houses
                .get(usize::from(h.0))
                .is_some_and(|x| x.role == crate::world::HouseRole::Player)
        });
        if !player && let Some(ob) = w.obj_mut(p) {
            // spec: S-0312 claims 12 and 25 (a neutral or special house's passenger is put on hunt; a player house's gets no further order)
            ob.beh.queue(Some(Behaviour::Hunt));
        }
    }
    if let Some(at) = at {
        w.cells.restore(c, to.layer, t, at);
    }
}

/// Whether a passenger of an open-topped transport has its own passenger
/// weapon index (S-0308 claim 10, table P row 4): the index, if any.
pub fn passenger_weapon_index(w: &World, firer: ObjectId) -> Option<usize> {
    if !open_topped_passenger(w, firer) {
        return None;
    }
    let i = w
        .obj(firer)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        .map_or(-1, |u| u.combat.open_transport_weapon);
    // spec: S-0308 claim 10 (−1 means choose normally)
    if i == -1 {
        return None;
    }
    // spec: S-0308 Edge cases (a passenger weapon index outside −1 … 17 is treated as 0, a departure)
    Some(
        usize::try_from(i)
            .ok()
            .filter(|&x| x < crate::combat::data::SLOTS)
            .unwrap_or(0),
    )
}
