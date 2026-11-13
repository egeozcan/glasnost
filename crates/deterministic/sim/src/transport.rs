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
//! An air transport's unload phase is likewise its *unload* behaviour's
//! progress count, with [`air_phase`]'s numbering (claim 13).

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

/// The phases of an air transport's ordinary passenger unloading (claim
/// 13), kept in its *unload* behaviour's progress count.
pub mod air_phase {
    /// Selection, the initial phase.
    // nocite: our own numbering of claim 13's five phases, in the claim's order (0 is the start value of S-0215 claim 3)
    pub const SELECTION: i32 = 0;
    /// Approach.
    // nocite: our own numbering of claim 13's five phases, in the claim's order
    pub const APPROACH: i32 = 1;
    /// Stopped check.
    // nocite: our own numbering of claim 13's five phases, in the claim's order
    pub const STOPPED_CHECK: i32 = 2;
    /// Release.
    // nocite: our own numbering of claim 13's five phases, in the claim's order
    pub const RELEASE: i32 = 3;
    /// Reselect.
    // nocite: our own numbering of claim 13's five phases, in the claim's order
    pub const RESELECT: i32 = 4;
}

/// The step-direction preference of an air transport's release (claim 13).
// spec: S-0507 claim 13 (step-direction preference 4, 5, 3, 7, 1, 0, 6, 2)
const RELEASE_ORDER: [usize; 8] = [4, 5, 3, 7, 1, 0, 6, 2];
/// The facing direction when no release neighbour is clear (claim 13).
// spec: S-0507 claim 13 (if none is clear, direction 4 supplies the facing)
const RELEASE_FALLBACK_FACING: usize = 4;
/// The map waypoint at the centre of claim 13's replacement search.
// spec: S-0507 claim 13 and Import mapping (map waypoint 700)
const REPLACEMENT_WAYPOINT: u16 = 700;
/// The phase delay of claim 13's stopped check and reselect.
// spec: S-0507 claim 13 (stopped check and reselect return 1 tick)
const AIR_STEP: i32 = 1;

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

/// Mind-controlled (claim 4; glossary; S-0601 Terms).
pub fn mind_controlled(w: &World, id: ObjectId) -> bool {
    crate::mind::mind_controlled(w, id)
}

/// A mind controller that controls at least one object now (claim 4).
pub fn controls_objects(w: &World, id: ObjectId) -> bool {
    crate::mind::controls_objects(w, id)
}

/// Releases a boarding unit from any mind control that held it (claim 6,
/// S-0601 claim 10).
pub fn release_mind_control(w: &mut World, id: ObjectId) {
    crate::mind::boarding(w, id);
}

/// Resets a boarding unit's gattling spin value, stage and elapsed-tick
/// count (claim 6, S-0609 claim 11).
pub fn reset_gattling_spin(w: &mut World, id: ObjectId) {
    crate::gattling::reset(w, id);
}

/// Whether an object is inside a tube (claim 7). Seam: tubes are not
/// built.
pub fn in_tube(w: &World, id: ObjectId) -> bool {
    let _ = (w, id);
    false
}

/// Whether a building is a tank bunker (claims 7 and 8): its type has the
/// bunker flag (S-0606 Terms).
pub fn tank_bunker(w: &World, building: ObjectId) -> bool {
    // spec: S-0507 claim 7 and S-0606 Terms (a building whose type has the bunker flag)
    crate::structures::bunker::is_bunker(w, building)
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
    if o.kind == Kind::Infantry {
        // spec: S-0507 claim 6 and S-0524 claim 14 (an infantry unit's qualifying arrival with the transport as destination gives one entered-by raise on the transport's current tag, the infantry unit as causing object, before the final acceptance test: a refused boarding can already have notified; a vehicle's boarding gives none, and neither kind gets an own-tag destruction raise)
        crate::mp_triggers::entry_raise(w, t, p);
        // The raise's actions may have removed either object or moved the transport.
        let (Some(po), Some(to)) = (w.obj(p), w.obj(t)) else {
            return false;
        };
        if !po.on_map || !to.on_map || to.pos.cell() != po.pos.cell() {
            return false;
        }
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
    }
    // spec: S-0507 claim 6 and S-0609 claim 11 (it resets its gattling stage, spin value and elapsed-tick count to 0)
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
        // spec: S-0308 claim 6 and S-0514 claim 4 (a first passenger with a temporal beam hands it to the vehicle, attached or not)
        crate::temporal::hand_over(w, t, p);
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
        // spec: S-0107 r46 claim 11, Changes of target (whatever rule makes the change: an infantry passenger's loss of its target ends its attack state and makes its request at once)
        crate::infantry::watch_test(w, p);
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
        // spec: S-0301 claim 5, Moment of a retarget's let-go (revision 67, Q-2246: at the change of the firer's target)
        crate::emitters::retarget_let_go(w, p);
        // spec: S-0107 r46 claim 11, Changes of target (an order's change, when it takes effect: the passenger's new target)
        crate::infantry::watch_test(w, p);
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
    // spec: S-0507 claim 7 (a tunnel-marked cell or a tunnel edge for unloading, Terms)
    if tunnel_cell(w, c) {
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
    // spec: S-0507 claim 7 (water: refused for every player house; neutral and special houses keep the exemption, as does a test-only computer-controlled player house, glossary)
    let player = o
        .owner
        .is_some_and(|h| w.houses.get(usize::from(h.0)).is_some_and(|x| x.human));
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

/// Whether cell `c` is tunnel-marked (S-0507 Terms): its land type is
/// tunnel and it is recorded as the entrance of an existing map tube
/// (S-0212 claim 7). Tunnel land alone does not qualify.
pub fn tunnel_marked(w: &World, c: CellXY) -> bool {
    // spec: S-0507 Terms (tunnel-marked: tunnel land and a recorded tube entrance)
    w.cells.cell(c).is_some_and(|x| x.land == land::TUNNEL) && crate::seams::tube_entrance(w, c)
}

/// The tunnel-edge predicate of S-0507 Terms for cell `c`, given which
/// cells exist and which are tunnel-marked: in map north or map west,
/// either the first neighbour is marked and the second is not, or the
/// second is marked and the third is not, every cell of that sequence
/// through the non-marked cell existing.
pub fn tunnel_edge(
    c: CellXY,
    exists: impl Fn(CellXY) -> bool,
    marked: impl Fn(CellXY) -> bool,
) -> bool {
    // spec: S-0507 Terms (map north and map west only; no south, east or diagonal test)
    const NORTH: usize = 0; // spec: S-0204 Constants (step direction 0: map north)
    const WEST: usize = 6; // spec: S-0204 Constants (step direction 6: map west)
    [NORTH, WEST].iter().any(|&d| {
        let (dx, dy) = STEPS[d];
        let at = |k: i32| CellXY::new(c.x + k * dx, c.y + k * dy);
        // spec: S-0507 Terms (first marked and second not, both existing)
        let first = exists(at(1)) && exists(at(2)) && marked(at(1)) && !marked(at(2));
        // spec: S-0507 Terms (second marked and third not, all three existing)
        let second =
            exists(at(1)) && exists(at(2)) && exists(at(3)) && marked(at(2)) && !marked(at(3));
        first || second
    })
}

/// Whether cell `c` is a tunnel cell for unloading and deploy readiness:
/// tunnel-marked, or a tunnel edge (S-0507 claim 7 and Terms; S-0214
/// Terms *tunnel cell*).
pub fn tunnel_cell(w: &World, c: CellXY) -> bool {
    tunnel_marked(w, c) || tunnel_edge(c, |x| w.cells.cell(x).is_some(), |x| tunnel_marked(w, x))
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
    // spec: S-0507 claim 9 (the selected object, if any, is allied from the transport owner's side; with none selected the condition passes)
    alliance_object(w, c).is_none_or(|x| w.obj(x).is_some_and(|x| w.allied(o.owner, x.owner)))
}

/// The object claim 9's alliance test reads in cell `c`: the ground-layer
/// infantry unit, vehicle, aircraft or building nearest the cell's top
/// corner by its within-cell coordinates, ties to the one most recently
/// added to the layer, a building after mobile objects.
pub fn alliance_object(w: &World, c: CellXY) -> Option<ObjectId> {
    let mut best: Option<(ObjectId, u128)> = None;
    // spec: S-0507 claim 9 (ties favour the most recently added, a building following mobile objects: S-0209 claim 13's list order, non-buildings most recent first, then buildings)
    for &x in w.cells.list(c, Layer::Ground) {
        let Some(ob) = w.obj(x) else { continue };
        // spec: S-0507 claim 9 (an infantry unit, vehicle, aircraft or building; terrain objects supply no owner)
        if !matches!(
            ob.kind,
            Kind::Infantry | Kind::Vehicle | Kind::Aircraft | Kind::Building
        ) {
            continue;
        }
        // spec: S-0507 claim 9 (only the within-cell map coordinates), S-0204 claim 2 (0 … 255 per axis)
        let (u, v) = (
            u128::from(ob.pos.x.rem_euclid(256).unsigned_abs()),
            u128::from(ob.pos.y.rem_euclid(256).unsigned_abs()),
        );
        // spec: S-0507 claim 9 (the Euclidean length from the top corner, truncated toward zero to integer leptons; for these exact integer sums the truncated double-precision root is the integer square root)
        let d = glasnost_num::geom::isqrt(u * u + v * v);
        if best.is_none_or(|(_, b)| d < b) {
            best = Some((x, d));
        }
    }
    best.map(|(x, _)| x)
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
            // spec: S-0507 claim 11 and table U (the finished processing ends with the standard delay, including its draw; not a 1-tick completion)
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
        // spec: S-0507 claim 8 and table U (on water with no destination: the nearby-cell search's cell as destination; wait 10 ticks)
        // spec: S-0507 claim 8 and S-0212 claim 25 (with no result, the placeholder at map cell (0, 0) is still assigned, and the processing does not go on to claim 9)
        let c = shore(w, t).unwrap_or(CellXY::new(0, 0));
        crate::econ::assign_destination(w, t, Some(Target::Cell(c)));
        // A new destination starts a new path search (S-0212 claim 20).
        if let Some(ob) = w.obj_mut(t) {
            ob.path.steps.clear();
            // spec: S-0212 claim 21 and its state table, row "destination changes" (r97: for the driving and walk locomotors a different destination, or none, ends a running retry delay, path::destination_changed and path::waits; the ship and the other locomotors keep it, low confidence; the hovering locomotor's is S-0503's)
        }
        return SETTLE_WAIT;
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

/// The shore search of claim 8: S-0212 claim 25's nearby-cell search from
/// the transport's cell (its transport shore approach row).
fn shore(w: &mut World, t: ObjectId) -> Option<CellXY> {
    crate::regions::ensure(w);
    let c = w.obj(t)?.pos.cell();
    crate::path::nearby(
        w,
        c,
        crate::path::NearbyUse::Settings(crate::path::NearbySettings {
            // spec: S-0507 claim 8 and S-0212 claim 25 (transport shore approach: wheel, zone normal, no reach region, ground mode, bridges allowed, no height, no-overlay condition or reference cell)
            class: Some(LocomotionClass::Wheel),
            zone: MovementZone::Normal,
            reach: None,
            deck: false,
            height: false,
            reference: None,
            no_bridges: false,
            no_overlay: false,
        }),
    )
}

/// Whether passenger `p` could enter cell `c` from the transport (claim
/// 10): the entry outcome is clear.
pub(crate) fn enters(w: &World, p: ObjectId, c: CellXY, reference: i32) -> bool {
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
    let at = exit.and_then(|(_, n, _)| exit_position(w, p, n));
    let placed = match (exit, at) {
        (Some((d, _, dest)), Some(at)) => {
            // spec: S-0507 claim 10 (P is put on the map facing the exit direction)
            let facing = (d as i32 * DIRECTION_FACING) as Facing;
            put_out(w, t, p, at, facing, Some(dest));
            true
        }
        _ => false,
    };
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

/// Where passenger `p` is put on the map from serving neighbour `n` of a
/// vehicle transport (claim 10), or `None` when an infantry unit finds no
/// standing position.
pub(crate) fn exit_position(w: &mut World, p: ObjectId, n: CellXY) -> Option<Pos> {
    let po = w.obj(p)?.clone();
    let (x, y) = if po.kind == Kind::Infantry {
        // spec: S-0507 claim 10 and S-0208 claim 3 (the serving neighbour's centre as desired point: the centre class, one ranged draw over 0 … 3, right/left/bottom preferences; never centre or top; none available is a placement failure)
        let k = crate::walk::choose_position(
            w,
            n,
            Layer::Ground,
            crate::cells::position::CENTRE,
            false,
        )?;
        let (u, v) = crate::walk::standing_point(k);
        (256 * n.x + u, 256 * n.y + v) // spec: S-0208 table P (a standing position within the cell)
    } else {
        crate::regions::ensure(w);
        let class = w.rules().unit(po.kind, po.type_index).and_then(|u| u.class);
        // spec: S-0507 claim 10 and S-0212 claim 25 (vehicle passenger exit: its own class, zone normal, no reach region, ground mode, bridges allowed, no height, no-overlay condition or reference cell)
        let found = crate::path::nearby(
            w,
            n,
            crate::path::NearbyUse::Settings(crate::path::NearbySettings {
                class,
                zone: MovementZone::Normal,
                reach: None,
                deck: false,
                height: false,
                reference: None,
                no_bridges: false,
                no_overlay: false,
            }),
        );
        // spec: S-0507 claim 10 and S-0212 claim 25 (a failed search supplies the placeholder at cell (0, 0), not the serving neighbour's centre)
        let c = found.unwrap_or(CellXY::new(0, 0));
        (256 * c.x + 128, 256 * c.y + 128) // spec: S-0507 claim 10 (the selected cell's centre), S-0207 Terms (128 leptons in from its corner)
    };
    // spec: S-0507 claim 10 (placement puts it on the surface: world height 0 adjusted by normal placement)
    Some(Pos::new(x, y, w.surface_at(x, y)))
}

/// Puts passenger `p` of transport `t` on the map at `at` facing `facing`
/// (claims 10 and 13), queuing *move* to `dest` when given.
fn put_out(w: &mut World, t: ObjectId, p: ObjectId, at: Pos, facing: Facing, dest: Option<CellXY>) {
    let Some(po) = w.obj(p).cloned() else {
        return;
    };
    let Some(to) = w.obj(t).cloned() else {
        return;
    };
    // spec: S-0507 claims 10 and 3 (P is no longer a passenger; it leaves from position 1)
    if let Some(tb) = w.obj_mut(t) {
        tb.transport.passengers.retain(|&x| x != p);
    }
    if let Some(ob) = w.obj_mut(p) {
        ob.transport.riding = None;
        ob.layer = Layer::Ground;
    }
    crate::objects::place(w, p, at, facing);
    if open_topped(w, t)
        && po.owner != to.owner
        && let Some(ob) = w.obj_mut(p)
    {
        // spec: S-0507 claims 10 and 15 (open-topped, and P's owner differs from the transport's: P loses its target)
        crate::spawn::clear_own_target(ob);
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
    }
    if let Some(dest) = dest {
        if let Some(ob) = w.obj_mut(p) {
            // spec: S-0507 claim 10 (P queues move, to the far cell in the first round or the neighbour cell in the second) and claim 13 (move toward the chosen neighbour)
            ob.beh.queue(Some(Behaviour::Move));
        }
        crate::econ::assign_destination(w, p, Some(Target::Cell(dest)));
    }
    passenger_left(w, t, p);
}

/// Passenger `p` of aircraft `t` is dropped (S-0522 claim 9): it is no
/// longer a passenger and is put on the map at `at`, facing `facing`, in
/// cell layer `layer`.
pub(crate) fn drop_out(
    w: &mut World,
    t: ObjectId,
    p: ObjectId,
    at: Pos,
    facing: Facing,
    layer: Layer,
) {
    // spec: S-0507 claim 3 and S-0522 claim 9 (the first passenger leaves the passenger order)
    if let Some(tb) = w.obj_mut(t) {
        tb.transport.passengers.retain(|&x| x != p);
    }
    if let Some(ob) = w.obj_mut(p) {
        ob.transport.riding = None;
        ob.layer = layer;
    }
    crate::objects::place(w, p, at, facing);
    passenger_left(w, t, p);
}

/// What follows a passenger leaving transport `t` (claims 10 and 13).
fn passenger_left(w: &mut World, t: ObjectId, p: ObjectId) {
    // spec: S-0614 claim 12 (the host, a transport, lets a passenger out)
    crate::parasite::host_unloading(w, t);
    if count(w, t) == 0 && gunner(w, t) {
        // spec: S-0308 claim 6 (when its last passenger leaves, its weapon number becomes 0)
        crate::combat::weapon::set_weapon_number(w, t, 0);
        temporal_hand_back(w, t, p);
    }
}

/// The temporal beam hand-back to the last passenger of a gunner vehicle
/// (S-0308 claim 6): the beam goes back and lets go (S-0514).
pub fn temporal_hand_back(w: &mut World, t: ObjectId, p: ObjectId) {
    crate::temporal::hand_back(w, t, p);
}

fn set_air_phase(w: &mut World, t: ObjectId, phase: i32) {
    if let Some(ob) = w.obj_mut(t) {
        ob.beh.progress = phase;
    }
}

/// The *unload* processing of an air transport (claim 13): its phase
/// table. Returns the delay.
pub fn process_air_unload(w: &mut World, t: ObjectId) -> i32 {
    let Some(o) = w.obj(t).cloned() else {
        return crate::process::standard_delay(w, t);
    };
    let carryall = w
        .rules()
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.raw.carryall.unwrap_or(false));
    if carryall {
        // spec: S-0507 claim 13 (a carrier aircraft lifting a vehicle uses different unloading, outside this claim)
        w.note("S-0507 claim 13: a carrier aircraft's unloading is not this spec's");
        return crate::process::standard_delay(w, t);
    }
    // spec: S-0507 claim 13 (F7: S-0501 claim 40's landing test, the destination object used for the competing-aircraft comparison)
    let fails = |w: &World, d: Target| !crate::air::landing_test(w, t, d);
    match o.beh.progress {
        air_phase::APPROACH => {
            if o.dest.is_some_and(|d| fails(w, d)) {
                // spec: S-0507 claim 13 (approach: a destination failing F7 returns to selection, overriding the stopped transition)
                set_air_phase(w, t, air_phase::SELECTION);
            } else if crate::air::stopped(w, t) {
                // spec: S-0507 claim 13 (approach: the locomotor reports stopped, S-0501 claim 40's test: stopped check)
                set_air_phase(w, t, air_phase::STOPPED_CHECK);
            }
            // spec: S-0507 claim 13 (approach ends with S-0215 claim 6's standard delay)
            crate::process::standard_delay(w, t)
        }
        air_phase::STOPPED_CHECK => {
            if crate::air::stopped(w, t) {
                // spec: S-0507 claim 13 (stopped check: stopped, release)
                set_air_phase(w, t, air_phase::RELEASE);
            }
            // spec: S-0507 claim 13 and S-0215 claim 29 (stopped check: next processing in 1 tick, no delay draw)
            AIR_STEP
        }
        air_phase::RESELECT => {
            // spec: S-0507 claim 13 (reselect: selection, next processing in 1 tick)
            set_air_phase(w, t, air_phase::SELECTION);
            AIR_STEP
        }
        air_phase::RELEASE => {
            release(w, t);
            // spec: S-0507 claim 13 (the release processing ends with the standard delay)
            crate::process::standard_delay(w, t)
        }
        _ => {
            selection(w, t);
            // spec: S-0507 claim 13 (selection ends with the standard delay)
            crate::process::standard_delay(w, t)
        }
    }
}

/// Claim 13's *selection* phase.
fn selection(w: &mut World, t: ObjectId) {
    let Some(o) = w.obj(t).cloned() else { return };
    let h = crate::air::height(w, t);
    // spec: S-0507 claim 13 (height above ground 0 leptons and forward pitch 0, which is not modelled, S-0501 claim 38)
    let grounded = h == 0;
    let at_dest = o
        .dest
        .is_none_or(|d| crate::process::target_pos(w, d) == o.pos);
    if grounded && at_dest {
        // spec: S-0507 claim 13 (no destination, or exact world-position equality with it: release at a later processing)
        set_air_phase(w, t, air_phase::RELEASE);
        return;
    }
    let dropship = w
        .rules()
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.dropship);
    let Some(d) = o.dest else {
        if h > 0 && dropship {
            // spec: S-0507 claim 13 (above ground, no destination, a dropship type: an own usable dock by S-0501 claim 21's pad selection, else S-0501 claim 41's landing spot, as destination; selection stays)
            // spec: S-0501 claim 21 (r176, Q-2866: every use of the search records its result as the pad last heading for, this one included)
            let to = match crate::air::pad_search(w, t) {
                Some(k) => Some(Target::Object(k)),
                None => crate::air::landing_spot(w, t).map(Target::Cell),
            };
            crate::econ::assign_destination(w, t, to);
        } else {
            // spec: S-0507 claim 13 (no destination otherwise: stopped check; no landing-spot search)
            set_air_phase(w, t, air_phase::STOPPED_CHECK);
        }
        return;
    };
    let passes = crate::air::landing_test(w, t, d);
    // spec: S-0507 claim 13 and Edge cases (quirks.air_unload_reselects_clear_destination: a passing destination is replaced; off, a failing one)
    let replace = passes == w.rules().quirks.air_unload_reselects_clear_destination;
    if replace {
        // spec: S-0507 claim 13 and Import mapping (no original-team membership: map waypoint 700 is the centre; an undefined marker supplies cell (0, 0) and the search still runs)
        let centre = w
            .triggers
            .waypoints
            .get(&REPLACEMENT_WAYPOINT)
            .map_or(CellXY::new(0, 0), |&(x, y)| {
                CellXY::new(i32::from(x), i32::from(y))
            });
        // spec: S-0507 claim 13 (S-0501 claim 40's replacement-cell search, radii 0 … 31 with one draw each; failure retains the search centre, the placeholder (0, 0) with an undefined marker)
        let c = crate::air::replacement_search(w, t, centre).unwrap_or(centre);
        crate::econ::assign_destination(w, t, Some(Target::Cell(c)));
        return;
    }
    let cruise = crate::air::cruise_height(w, t);
    // spec: S-0507 claim 13 (approach when the current height in integer leptons above terrain equals the cruise height, otherwise reselect)
    let next = if h == cruise {
        air_phase::APPROACH
    } else {
        air_phase::RESELECT
    };
    set_air_phase(w, t, next);
}

/// Claim 13's *release* phase: position 1 gets out, unless tethered.
fn release(w: &mut World, t: ObjectId) {
    // spec: S-0507 claim 13 (release while tethered: nobody released)
    if crate::econ::tethered(w, t) {
        return;
    }
    let Some(o) = w.obj(t).cloned() else { return };
    let Some(&p) = o.transport.passengers.first() else {
        // spec: S-0507 claim 13 (release with no passengers: fall back to rest)
        crate::process::fall_back(w, t, false);
        return;
    };
    // spec: S-0507 claim 13 (position 1 is removed from the passenger order for this attempt)
    if let Some(tb) = w.obj_mut(t) {
        tb.transport.passengers.retain(|&x| x != p);
    }
    if let Some(ob) = w.obj_mut(p) {
        ob.transport.riding = None;
        ob.pos = o.pos;
        ob.layer = Layer::Ground;
    }
    let c = o.pos.cell();
    let reference = occupancy::reference_level(w, t);
    let step = |d: usize| CellXY::new(c.x + STEPS[d].0, c.y + STEPS[d].1);
    // spec: S-0507 claim 13 (the first neighbour whose entry outcome for this passenger is clear, in preference 4, 5, 3, 7, 1, 0, 6, 2; no entry direction or requested level)
    let clear = RELEASE_ORDER
        .iter()
        .copied()
        .find(|&d| enters(w, p, step(d), reference));
    // spec: S-0507 claim 13 (none clear: direction 4 supplies the facing and the last tested neighbour, direction 2, the destination)
    let (fd, dd) = match clear {
        Some(d) => (d, d),
        None => (
            RELEASE_FALLBACK_FACING,
            RELEASE_ORDER[RELEASE_ORDER.len() - 1],
        ),
    };
    // spec: S-0507 claim 13 (facing: the chosen direction times 8192, not the aircraft's body facing)
    let facing = (fd as i32 * DIRECTION_FACING) as Facing;
    let at = release_position(w, t, p);
    match at {
        Some(at) => put_out(w, t, p, at, facing, Some(step(dd))),
        None => {
            // spec: S-0507 claim 13 (failed placement: the unit stays off the map, out of the passenger order, with no retry and no kill awarded)
            w.note("S-0507 claim 13: a released passenger found no place; it stays off the map (its lifetime is unmeasured)");
            passenger_left(w, t, p);
        }
    }
    if count(w, t) == 0 {
        // spec: S-0507 claim 13 (fall back to rest when now empty)
        crate::process::fall_back(w, t, false);
    }
}

/// Where a passenger released by air transport `t` is put (claim 13), or
/// `None` when the placement fails: an infantry unit at surface height that
/// finds no standing position, or an aircraft cell whose entry outcome for
/// the passenger is not clear.
fn release_position(w: &mut World, t: ObjectId, p: ObjectId) -> Option<Pos> {
    let o = w.obj(t)?.clone();
    let kind = w.obj(p)?.kind;
    // spec: S-0507 claim 13 (the desired world position: the aircraft's exact world position, not its cell centre)
    let (x, y) = (o.pos.x, o.pos.y);
    // spec: S-0507 claim 13 (integer leptons: the terrain surface height at the aircraft's horizontal point)
    let surface = w.surface_at(x, y);
    let c = o.pos.cell();
    // spec: S-0507 claim 13 and S-0208 claim 3 (only the aircraft itself is excluded from occupancy: its object-present mark is set aside)
    let had = crate::air::set_aside_mark(w, t);
    let at = if kind == Kind::Infantry && o.pos.z == surface {
        // spec: S-0507 claim 13 (infantry exactly at surface height: S-0208 claim 3's occupant-aware choice in the same cell; a centre class takes the ranged draw, a side class its non-random row; refusal fails the placement)
        let class = crate::walk::position_class(x.rem_euclid(256), y.rem_euclid(256), true);
        crate::walk::choose_position(w, c, Layer::Ground, class, false).map(|k| {
            let (u, v) = crate::walk::standing_point(k);
            // spec: S-0208 table P (a standing position within the cell); S-0507 claim 13 (keeping the terrain height at the aircraft's original horizontal point)
            Pos::new(256 * c.x + u, 256 * c.y + v, surface)
        })
    } else {
        // spec: S-0507 claim 13 (infantry above surface height and every vehicle: the aircraft's exact horizontal position and height, with no standing-position draw and no nearby search; a height below the terrain is raised to it)
        Some(Pos::new(x, y, o.pos.z.max(surface)))
    };
    // spec: S-0507 claim 13 (for both kinds the ordinary placement entry test must also find the aircraft's own cell clear, with no requested entry direction or level; a refusal fails the placement)
    let reference = occupancy::reference_level(w, t);
    // spec: S-0507 claim 13 (the aircraft itself does not obstruct this placement: its listed entry is set aside too; other occupants count normally)
    let listed = w.cells.list(c, o.layer).iter().position(|&x| x == t);
    w.cells.leave(c, o.layer, t);
    let at = at.filter(|_| enters(w, p, c, reference));
    if let Some(i) = listed {
        w.cells.restore(c, o.layer, t, i);
    }
    if had {
        crate::air::restore_mark(w, t);
    }
    at
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
        died_with_transport(w, p, hit);
    }
}

/// A passenger's death with its transport (S-0312 claims 5 and 12): as a
/// leftover it keeps the health it had (S-0203 claim 13 (b)).
fn died_with_transport(w: &mut World, p: ObjectId, hit: &crate::combat::damage::Hit) {
    let health = w.obj(p).map(|o| o.health);
    crate::combat::death::passenger_death(w, p, hit);
    if let Some(hp) = health {
        // spec: S-0203 claim 13 (b) (a passenger killed with its transport keeps its health)
        crate::leftover::keep_health(w, p, hp);
    }
}

/// The removal cleanup of claim 16: every passenger still carried when
/// transport `t` is removed from play is killed, nested passengers too,
/// with no attacker credit and no death effects of its own (S-0312 claim
/// 5's passenger destruction with no attacker). This covers a sale
/// (S-0408 claim 11), an aircraft killed on the ground and a crashable
/// vehicle on the ground (S-0312 claims 12 and 15), the end of a sinking
/// (S-0504 claim 5) and every other removal.
pub fn remove_passengers(w: &mut World, t: ObjectId) {
    // spec: S-0507 claim 16 (killed, not released or preserved; no attacker credit)
    let none = crate::combat::damage::Hit {
        amount: 0,
        warhead: None,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: false,
        no_escape: false,
    };
    kill_passengers(w, t, &none);
}

/// An object is removed from the game: a removed passenger leaves its
/// transport's passenger order, and a removed transport's remaining
/// passengers die (claim 16).
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
    // spec: S-0312 claim 12 and S-0209 claim 4 (the dying vehicle contributes neither a listed blocker nor its vehicle-present flag to these escape tests; other occupants count normally)
    let at = w.cells.list(c, to.layer).iter().position(|&x| x == t);
    w.cells.leave(c, to.layer, t);
    // spec: S-0209 claim 1 (the flag is cleared in the record it was set in)
    let drive_flag = to
        .drive
        .as_ref()
        .and_then(|d| d.claims.iter().find(|x| x.0 == c))
        .map(|x| x.1);
    let hover_flag = to
        .hover
        .as_ref()
        .is_some_and(|h| h.claims.contains(&(c, to.layer)))
        .then_some(to.layer);
    let flag = drive_flag.or(hover_flag);
    if let Some(l) = flag {
        w.cells.claim_vehicle(c, l, false);
    }
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
            died_with_transport(w, p, hit);
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
        let player = w
            .obj(p)
            .and_then(|o| o.owner)
            .is_some_and(|h| w.houses.get(usize::from(h.0)).is_some_and(|x| x.human));
        if !player && let Some(ob) = w.obj_mut(p) {
            // spec: S-0312 claims 12 and 25 (a neutral or special house's passenger, or a test-only computer-controlled player house's (glossary), is put on hunt; a player house's gets no further order)
            ob.beh.queue(Some(Behaviour::Hunt));
        }
    }
    if let Some(l) = flag {
        w.cells.claim_vehicle(c, l, true);
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
