//! The jumpjet locomotor (S-0502): the seven flight states, the speed and
//! height control, the straight-line step along the movement heading, the
//! cruise bands, hovering, landing, orders, the neighbour step and the crash
//! motion.
//!
//! One update of a jumpjet (`tick`) is a physics step (F1 speed, F2 to F4
//! height, F5 step) with the *previous* state's maximum speed and target
//! height, then the state's outcome (claims 5 and 7 to 15): a new state,
//! maximum speed or target height first affects the physics of the next
//! tick (claim 3). A jumpjet that holds no move order and is grounded or
//! hovering takes no update.
//!
//! Magnetic lift (S-0607) links a seized unit's temporary jumpjet
//! locomotor here and drives its pull, drop and fall through the helpers at
//! the end of this module; `crate::lift` owns the rules. Crate pickup at
//! touchdown is S-0520 claim 6 ([`crate_pickup`]); ion storms are out of
//! scope in a skirmish. The
//! deploy-to-land landing request of S-0517 is the field
//! [`JumpState::landing_request`], set and cleared by the deploy rules.

use crate::table_trig::sin_q;
use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_num::geom::{Facing, direction, eight_direction, root_floor};
use glasnost_num::{F64, Rounding};
use serde::{Deserialize, Serialize};

use crate::combat::damage::{self, Hit};
use crate::rules::{Behaviour, JumpFields, Kind};
use crate::turning::Turning;
use crate::walk::{position_class, standing_point};
use crate::world::{Layer, ObjectId, Pos, Target, World};

mod fly;

pub use fly::{impact, start_crash, tick};

/// The arrival radius: a cruising jumpjet arrives when ρ is below it.
// spec: S-0502 claim 11 and Constants (arrival radius 20 leptons)
const ARRIVAL_RADIUS: i32 = 20;

/// The height of a bridge deck above the terrain.
// spec: S-0502 Constants (bridge height 416 leptons) and F3
const BRIDGE_HEIGHT: i32 = 416;

/// The clearance a unit standing in a cell adds.
// spec: S-0502 Constants (occupied-cell clearance 85 leptons) and F3
const OCCUPIED_CLEARANCE: i32 = 85;

/// The clearance of one art-height level of a building.
// spec: S-0502 Constants (building clearance unit 104 leptons per art-height level) and F3
const BUILDING_LEVEL: i32 = 104;

/// The target height of a crashing jumpjet.
// spec: S-0502 Constants (crash target height −5 leptons) and claim 18
const CRASH_TARGET: i32 = -5;

/// The lead of the crash spin's heading target.
// spec: S-0502 Constants (crash spin lead 11520 facing units) and claim 18
const CRASH_SPIN: i32 = 11520;

/// The movement heading of a new jumpjet: map east.
// spec: S-0206 claim 14 (the heading is set at once to map east, 16384, when a unit gets the jumpjet locomotor)
const EAST: Facing = 16384;

/// The step directions of S-0204 and the glossary: north, north-east, east,
/// south-east, south, south-west, west, north-west.
// spec: S-0204 claim 10 and glossary (step direction: 0 is (x, y − 1), 1 is (x + 1, y − 1), 2 is (x + 1, y), 3 is (x + 1, y + 1), 4 is (x, y + 1), 5 is (x − 1, y + 1), 6 is (x − 1, y) and 7 is (x − 1, y − 1))
pub const STEP_DIRECTIONS: [(i32, i32); 8] = [
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
];

/// The seven flight states of a jumpjet (S-0502 Terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// On the ground or a deck, at rest.
    Grounded,
    /// Rising after a move order.
    Ascending,
    /// Holding position in the air.
    Hovering,
    /// Flying towards its destination.
    Cruising,
    /// Coming down to land.
    Descending,
    /// Falling after a crash start.
    Crashing,
    /// The fall has ended.
    Crashed,
}

/// An occupancy mark of a jumpjet: a standing position of an infantry unit
/// or the vehicle-present flag of a vehicle, in a cell layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mark {
    /// The cell.
    pub cell: CellXY,
    /// The cell layer whose record carries the mark.
    pub layer: Layer,
    /// The standing position of an infantry mark.
    pub position: Option<u8>,
}

/// The flight state of one jumpjet (S-0502 Terms).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JumpState {
    /// The jumpjet state.
    pub phase: Phase,
    /// It holds a move order (*moving*, Terms).
    pub moving: bool,
    /// The destination point (claim 16).
    pub dest: Option<Pos>,
    /// The order-level destination the point was taken from: a change of it
    /// is a new order.
    pub order: Option<Target>,
    /// The order-level destination the own-cell filter (claim 23) last
    /// ignored: the locomotor does not take it later (Q-3043 interim).
    #[serde(default)]
    pub filtered: Option<Target>,
    /// The current speed *v*<sub>c</sub> (leptons per tick).
    pub vc: F64,
    /// The maximum speed *v*<sub>m</sub>.
    pub vm: F64,
    /// The target height *h*<sub>t</sub> in leptons.
    pub ht: i32,
    /// The wobble phase φ in radians.
    pub phi: F64,
    /// The movement heading (S-0206 claim 14).
    pub heading: Turning,
    /// The speed fraction (claim 6).
    pub fraction: F64,
    /// The ground occupancy mark.
    pub mark: Option<Mark>,
    /// The landing reservation (Terms).
    pub reservation: Option<Mark>,
    /// The cell list the jumpjet is in (claims 4 and 22): a hovering or
    /// cruising jumpjet keeps the listing it had, in its old cell.
    pub listed: Option<(CellXY, Layer)>,
    /// The cell whose hover slot the jumpjet holds.
    pub slot: Option<CellXY>,
    /// It stands on a bridge deck.
    pub on_deck: bool,
    /// The landing request of a deploy-to-land vehicle (S-0517 claim 5).
    pub landing_request: bool,
    /// A crash has started (S-0312 claim 16) and its fall begins in the
    /// next update (claim 18).
    pub crash_started: bool,
    /// The last extra crash drop crossed a deck (claim 18).
    pub crossed_deck: bool,
}

impl JumpState {
    /// A new jumpjet: grounded, at rest, the heading east.
    pub fn new() -> Self {
        Self {
            phase: Phase::Grounded,
            moving: false,
            dest: None,
            order: None,
            filtered: None,
            vc: F64::ZERO,
            vm: F64::ZERO,
            ht: 0,
            phi: F64::ZERO,
            heading: Turning::at(EAST),
            fraction: F64::ZERO,
            mark: None,
            reservation: None,
            listed: None,
            slot: None,
            on_deck: false,
            landing_request: false,
            crash_started: false,
            crossed_deck: false,
        }
    }
}

impl Default for JumpState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Access

fn st(w: &World, id: ObjectId) -> Option<&JumpState> {
    w.obj(id).and_then(|o| o.jump.as_ref())
}

/// The cell of the unit's heading point, its movement point (claim 2):
/// for a gripped unit with the jumpjet stand-in, the point that the seize
/// fixed (S-0607 claim 7). None without one.
pub fn heading_cell(w: &World, id: ObjectId) -> Option<glasnost_map::CellXY> {
    st(w, id).and_then(|j| j.dest).map(|p| p.cell())
}

fn upd(w: &mut World, id: ObjectId, f: impl FnOnce(&mut JumpState)) {
    if let Some(j) = w.obj_mut(id).and_then(|o| o.jump.as_mut()) {
        f(j);
    }
}

fn int(v: i32, mode: Rounding) -> F64 {
    F64::from_i64(i64::from(v), mode)
}

/// The jumpjet fields of an object's type (claim 1).
pub fn fields(w: &World, id: ObjectId) -> Option<JumpFields> {
    let o = w.obj(id)?;
    o.jump.as_ref()?;
    Some(w.rules().unit(o.kind, o.type_index)?.jump)
}

/// Whether the object moves with the jumpjet locomotor.
pub fn is_jumpjet(w: &World, id: ObjectId) -> bool {
    st(w, id).is_some()
}

/// The jumpjet state of an object.
pub fn phase(w: &World, id: ObjectId) -> Option<Phase> {
    st(w, id).map(|j| j.phase)
}

/// Whether a jumpjet is *moving now*: every state except *grounded* and
/// *hovering* (Terms, claim 20).
pub fn moving_now(w: &World, id: ObjectId) -> bool {
    // spec: S-0502 Terms and claim 20 (moving now: every jumpjet state except grounded and hovering)
    st(w, id).is_some_and(|j| !matches!(j.phase, Phase::Grounded | Phase::Hovering))
}

/// Whether a jumpjet holds a move order (*moving*, Terms).
pub fn moving(w: &World, id: ObjectId) -> bool {
    st(w, id).is_some_and(|j| j.moving)
}

/// The jumpjet's speed fraction (claim 6); `None` for another locomotor.
pub fn speed_fraction(w: &World, id: ObjectId) -> Option<F64> {
    st(w, id).map(|j| j.fraction)
}

/// Whether the jumpjet is crashing or crashed (claim 18), or a dropped
/// unit marked as crashing (S-0607 claim 13).
pub fn crashing(w: &World, id: ObjectId) -> bool {
    // spec: S-0607 claim 13 (a dropped unit is marked as crashing, also during its cell-centre detour)
    st(w, id)
        .is_some_and(|j| matches!(j.phase, Phase::Crashing | Phase::Crashed) || j.crash_started)
        || (is_jumpjet(w, id) && crate::lift::dropped(w, id))
}

/// Whether a deploy-to-land vehicle holds a landing request (S-0517 claim 5,
/// S-0502 claim 13).
pub fn landing_requested(w: &World, id: ObjectId) -> bool {
    st(w, id).is_some_and(|j| j.landing_request)
}

/// Requests landing for deploy (S-0517 claim 5): a hovering deploy-to-land
/// vehicle then descends by S-0502 claim 13's fourth row. The request creates
/// no movement commitment and does not wake an idle hover: only a unit that
/// is *moving* takes the update (S-0502 claim 3).
pub fn request_landing(w: &mut World, id: ObjectId) {
    // spec: S-0517 claim 5 and S-0502 claims 3 and 24 (revision 7: without a movement commitment the clear cannot create one or wake idle hover; a landing request alone does not bypass claim 3's update eligibility)
    upd(w, id, |j| j.landing_request = true);
}

/// The deploy order's clear of the order-level destination (S-0213 claim 10,
/// S-0502 claim 24): the order has already emptied the object's destination.
/// For a jumpjet this is an effective clear like a stop order's, except that
/// an unchanged vehicle assignment is suppressed.
pub fn destination_cleared(w: &mut World, id: ObjectId) {
    // spec: S-0502 claim 24 and S-0213 claim 10 (the deploy order's destination clear has the stop order's consequences whenever a movement commitment is retained; a suppressed unchanged vehicle assignment keeps the preceding point and commitment)
    stop_ordered(w, id);
}

/// The factory ascent report's direct discard of the order-level destination
/// (S-0417 claim 15): the locomotor's movement point, commitment and flight
/// continue; this is not claim 24's effective clear and reroutes nothing.
pub fn discard_order_destination(w: &mut World, id: ObjectId) {
    if let Some(o) = w.obj_mut(id) {
        o.dest = None;
    }
    // spec: S-0417 claim 15 and S-0502 claim 24 (the discard is no assignment the locomotor takes: the point and the commitment stay)
    upd(w, id, |j| j.order = None);
}

/// Whether the unit is held by magnetic lift (*seized*, S-0607 Terms).
pub fn seized(w: &World, id: ObjectId) -> bool {
    crate::lift::seized(w, id)
}

/// A crate in the landing cell is picked up at touchdown (S-0520 claim 6,
/// S-0502 claim 15); the landing is never undone, so a block means nothing.
pub fn crate_pickup(w: &mut World, id: ObjectId) {
    let Some(c) = w.obj(id).map(|o| o.pos.cell()) else {
        return;
    };
    crate::crates::pickup(w, id, c);
}

fn balloon(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        .is_some_and(|u| u.raw.balloon_hover.unwrap_or(false))
}

fn vehicle(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.kind == Kind::Vehicle)
}

/// A balloon-hover vehicle (claim 7's quarter or half).
fn balloon_vehicle(w: &World, id: ObjectId) -> bool {
    vehicle(w, id) && balloon(w, id)
}

fn simple_deployer(w: &World, id: ObjectId) -> bool {
    vehicle(w, id)
        && w.obj(id)
            .and_then(|o| w.rules().unit(o.kind, o.type_index))
            .is_some_and(|u| u.simple_deployer)
}

fn deploy_to_land(w: &World, id: ObjectId) -> bool {
    simple_deployer(w, id)
        && w.obj(id)
            .and_then(|o| w.rules().unit(o.kind, o.type_index))
            .is_some_and(|u| u.deploy_to_land)
}

fn entering(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        o.beh.current == Some(Behaviour::Enter) || o.beh.queued == Some(Behaviour::Enter)
    })
}

fn has_target(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.target.is_some())
}

fn pos(w: &World, id: ObjectId) -> Pos {
    w.obj(id).map_or(Pos::default(), |o| o.pos)
}

/// The jumpjet's height above the surface *r* (S-0502 claim 7): above the
/// terrain, or above the deck when it stands on one.
pub fn height_above(w: &World, id: ObjectId) -> i32 {
    w.height_above_surface(id)
}

/// The bridge-adjusted height *h* of claims 7 and 15.
fn adjusted_height(w: &World, id: ObjectId) -> i32 {
    let r = height_above(w, id);
    let p = pos(w, id);
    let deck = st(w, id).is_some_and(|j| j.on_deck);
    // spec: S-0502 claim 7 (h = r − 416 when its cell has a high bridge, it is not on the deck and r ≥ 416; otherwise h = r)
    if w.high_bridge(p.cell()) && !deck && r >= BRIDGE_HEIGHT {
        r - BRIDGE_HEIGHT
    } else {
        r
    }
}

fn centre_height(w: &World, c: CellXY) -> i32 {
    w.surface_at(256 * c.x + 128, 256 * c.y + 128) // spec: S-0204 claim 3 (the centre of a cell is (256x + 128, 256y + 128))
}

// ---------------------------------------------------------------------------
// Occupancy marks and the cell listing (claim 4)

fn occ_quirks(w: &World) -> crate::cells::OccupancyQuirks {
    let q = &w.rules().quirks;
    crate::cells::OccupancyQuirks {
        side_positions_only: q.occupancy_house_side_positions_only,
        stale: q.occupancy_house_stale,
    }
}

/// A mark for the jumpjet at a point in a layer.
fn mark_at(w: &World, id: ObjectId, p: Pos, layer: Layer) -> Mark {
    let infantry = w.obj(id).is_some_and(|o| o.kind == Kind::Infantry);
    let top = w.rules().quirks.top_position_counts_as_centre;
    Mark {
        cell: p.cell(),
        layer,
        // spec: S-0204 claim 2 (the position within the cell, 0 … 255 per axis) and S-0208 claim 2 (its position class)
        position: infantry.then(|| position_class(p.x.rem_euclid(256), p.y.rem_euclid(256), top)),
    }
}

fn set_mark(w: &mut World, id: ObjectId, m: Mark) {
    let Some(o) = w.obj(id) else { return };
    match m.position {
        Some(p) => {
            if let Some(h) = o.owner {
                w.cells.mark_infantry(m.cell, m.layer, p, id, h);
            }
        }
        None => w.cells.claim_vehicle(m.cell, m.layer, true),
    }
}

/// Clears one of the jumpjet's marks: the actual-position mark and the
/// landing reservation can name the same standing position (claim 4: a
/// descent supplies both), which the other then still holds.
fn clear_mark(w: &mut World, id: ObjectId, m: Mark, other: Option<Mark>) {
    let q = occ_quirks(w);
    match m.position {
        Some(p) => {
            if other == Some(m) {
                return;
            }
            let mine = w
                .cells
                .occ(m.cell, m.layer)
                .and_then(|r| r.marks[usize::from(p)])
                .is_some_and(
                    |k| matches!(k.by, crate::cells::Marker::Infantry(who, _) if who == id),
                );
            if mine {
                w.cells.clear_position(m.cell, m.layer, p, q);
            }
        }
        None => w.cells.claim_vehicle(m.cell, m.layer, false),
    }
}

/// Lifts the ground occupancy mark.
fn lift_mark(w: &mut World, id: ObjectId) {
    let Some(j) = w.obj_mut(id).and_then(|o| o.jump.as_mut()) else {
        return;
    };
    let Some(m) = j.mark.take() else { return };
    let other = j.reservation;
    clear_mark(w, id, m, other);
}

/// Lays the ground mark at the present position (claim 4).
fn lay_mark(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let want = mark_at(w, id, o.pos, o.layer);
    if st(w, id).and_then(|j| j.mark) == Some(want) {
        return;
    }
    lift_mark(w, id);
    // spec: S-0502 claim 4 and S-0209 claim 1 (an actual-position mark of an infantry unit takes a standing position; a position that another unit holds stays that unit's: nothing is marked)
    if let Some(p) = want.position
        && w.cells.occ(want.cell, want.layer).is_some_and(|r| {
            r.marks[usize::from(p)].is_some_and(
                |k| !matches!(k.by, crate::cells::Marker::Infantry(who, _) if who == id),
            )
        })
    {
        return;
    }
    set_mark(w, id, want);
    upd(w, id, |j| j.mark = Some(want));
}

/// Releases the landing reservation.
fn release_reservation(w: &mut World, id: ObjectId) {
    let Some(j) = w.obj_mut(id).and_then(|o| o.jump.as_mut()) else {
        return;
    };
    let Some(m) = j.reservation.take() else {
        return;
    };
    let other = j.mark;
    clear_mark(w, id, m, other);
}

fn release_slot(w: &mut World, id: ObjectId) {
    let Some(c) = w
        .obj_mut(id)
        .and_then(|o| o.jump.as_mut())
        .and_then(|j| j.slot.take())
    else {
        return;
    };
    if w.jump_slots.get(&c) == Some(&id) {
        w.jump_slots.remove(&c);
    }
}

/// Whether another jumpjet holds the hover slot of a cell.
fn slot_held(w: &World, id: ObjectId, c: CellXY) -> bool {
    w.jump_slots
        .get(&c)
        .is_some_and(|&h| h != id && w.obj(h).is_some())
}

fn claim_slot(w: &mut World, id: ObjectId, c: CellXY) {
    release_slot(w, id);
    w.jump_slots.insert(c, id);
    upd(w, id, |j| j.slot = Some(c));
}

/// A jumpjet was put on the map (S-0502 claim 1; `objects::place`): it
/// stands grounded, its mark laid and its listing made by the placement.
pub fn placed(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if o.jump.is_none() {
        return;
    }
    let layer = o.layer;
    let m = mark_at(w, id, o.pos, layer);
    let cell = o.pos.cell();
    upd(w, id, |j| {
        j.phase = Phase::Grounded;
        j.on_deck = layer == Layer::Deck;
        j.mark = Some(m);
        j.listed = Some((cell, layer));
    });
}

/// A unit put on the map (S-0502 claim 29): it is registered as a tracked
/// flying object at its placement only when it is then airborne (claim 30
/// for a unit moving with the jumpjet locomotor, S-0307 otherwise) and its
/// type has the counts-as-aircraft flag (Import mapping,
/// `ConsideredAircraft`). Otherwise a jumpjet placed in the air is not
/// registered until it next takes off from *grounded*. Aircraft are left
/// to their own movement lane, which tracks them from the take-off to
/// touchdown and from a placement in the air (S-0501 claim 47).
pub fn placed_registration(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if !matches!(o.kind, Kind::Infantry | Kind::Vehicle) {
        return;
    }
    let flag = w
        .rules()
        .econ
        .tech
        .of(crate::econ::tech::type_of(o))
        .is_some_and(|t| t.considered_aircraft);
    // spec: S-0502 claim 29 (placed in the air: registered only when airborne, claim 30, and the type has the counts-as-aircraft flag; the same rule for every kind of unit)
    if flag && crate::combat::weapon::airborne_by_height(w, id) {
        crate::combat::flying::register(w, id);
    }
}

/// Puts a grounded jumpjet in the air in `phase` at `height` leptons above
/// the surface, as if it had flown there: its mark and listing go, its
/// target height is the height, a hovering one holds its cell's slot, and
/// a unit in any other state holds a move order. For scenes of other topics and
/// for tests.
pub fn start_in_flight(w: &mut World, id: ObjectId, phase: Phase, height: i32) {
    if !is_jumpjet(w, id) {
        return;
    }
    leave_ground(w, id);
    let Some(o) = w.obj(id) else { return };
    let z = w.surface_at(o.pos.x, o.pos.y) + height;
    if let Some(ob) = w.obj_mut(id) {
        ob.pos.z = z;
    }
    upd(w, id, |j| {
        j.phase = phase;
        j.ht = height;
        // spec: S-0502 Terms (moving: it holds a move order; a unit put in the air in a state other than hovering holds one)
        j.moving |= phase != Phase::Hovering;
    });
    if phase == Phase::Hovering {
        let c = pos(w, id).cell();
        claim_slot(w, id, c);
    }
    took_off(w, id);
}

/// The jumpjet leaves the map or goes inside: every mark, the reservation
/// and the hover slot are released (claims 4, 7, 13, 14 and 18).
pub fn leaving(w: &mut World, id: ObjectId) {
    let own_tracking = crate::air::is_aircraft(w, id) || crate::spawn::rocket_type(w, id).is_some();
    if !own_tracking {
        // spec: S-0502 claim 29 (revision 18, Q-1500 answered: going off the map for any reason ends registration, also for a seized unit whose own locomotor was restored, S-0607 claim 20)
        crate::combat::flying::unregister(w, id);
    }
    if !is_jumpjet(w, id) {
        return;
    }
    lift_mark(w, id);
    unlist(w, id);
    release_reservation(w, id);
    release_slot(w, id);
}

/// Takes the jumpjet out of the cell list it is in (claim 4): the cell of
/// its listing, which a hovering or cruising unit may have left.
fn unlist(w: &mut World, id: ObjectId) {
    let listed = w
        .obj_mut(id)
        .and_then(|o| o.jump.as_mut())
        .and_then(|j| j.listed.take());
    if let Some((c, l)) = listed {
        w.cells.leave(c, l, id);
    }
    // Another rule can have moved the listing without telling the jumpjet (a bridge collapse moves a deck listing to the ground, S-0511 claim 9): the cell of its position is cleared in both layers as well.
    if let Some(c) = w.obj(id).map(|o| o.pos.cell()) {
        w.cells.leave_both(c, id);
    }
}

/// Puts the jumpjet in the list of `layer` of the cell of its position.
fn list_at(w: &mut World, id: ObjectId, layer: Layer) {
    let Some(cell) = w.obj(id).map(|o| o.pos.cell()) else {
        return;
    };
    if let Some(ob) = w.obj_mut(id) {
        ob.layer = layer;
    }
    w.cells.enter(cell, layer, id, false);
    // spec: S-0212 claims 13 and 16a (recorded in a cell of another threat block: its threat amount moves there)
    crate::threat::relisted(w, id, cell);
    upd(w, id, |j| j.listed = Some((cell, layer)));
}

/// Takes the unit off the ground: its mark and cell listing go (claim 4).
fn leave_ground(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let cell = o.pos.cell();
    lift_mark(w, id);
    unlist(w, id);
    if let Some(ob) = w.obj_mut(id) {
        ob.layer = Layer::Ground;
    }
    upd(w, id, |j| j.on_deck = false);
    // spec: S-0211 claim 5 and S-0502 claim 26 (the old placement's removal recomputes its cell)
    crate::load::recalc_cells(w, &[cell]);
}

/// Puts the unit on the ground at its position: its listing, and a mark.
fn settle(w: &mut World, id: ObjectId, layer: Layer) {
    let Some(o) = w.obj(id) else { return };
    let cell = o.pos.cell();
    unlist(w, id);
    w.cells.leave_both(cell, id);
    list_at(w, id, layer);
    upd(w, id, |j| j.on_deck = layer == Layer::Deck);
    lay_mark(w, id);
    // spec: S-0211 claim 5 and S-0502 claim 26 (touchdown supplies its ordinary placement recompute)
    crate::load::recalc_cells(w, &[cell]);
}

/// The height below which a jumpjet's new position is listed (claim 4): a
/// height above the surface of less than 208 leptons.
// spec: S-0502 claim 4 (the listing test is r < 208 leptons, an exact integer comparison; the type's cruise height does not change it)
const LISTING_HEIGHT: i32 = 208;

/// Whether the jumpjet's position qualifies for ground-layer placement
/// (claim 4; Q-1010 is answered by revision 6): with *r* its height above
/// the surface (above the deck when the on-deck state is set) and *r*′ the
/// bridge-adjusted height, a new listing qualifies exactly when *r* < 208
/// leptons or *r*′ = 0. The 208 test reads *r*, not *r*′.
pub fn placement_qualifies(w: &World, id: ObjectId) -> bool {
    if w.obj(id).is_none() {
        return false;
    }
    let h = layer_heights(w, id);
    // spec: S-0502 claim 4 (exactly when r < 208 leptons or r′ = 0 leptons; the 208-lepton comparison uses r, not r′)
    h.r < LISTING_HEIGHT || h.adjusted == 0
}

/// The two heights of S-0502 claim 4's exact height predicate, which are
/// also those of S-0102's jumpjet layer rule (*State and ordering*): *r*,
/// the height above the surface, and the bridge-adjusted *r*′.
pub fn layer_heights(w: &World, id: ObjectId) -> crate::combat::effects::LayerHeights {
    let Some(o) = w.obj(id) else {
        return Default::default();
    };
    let on_deck = st(w, id).is_some_and(|j| j.on_deck);
    let r = height_above(w, id);
    // spec: S-0502 claim 4 and S-0102 claim 3 (r′ = r − 416 only over a high-bridge cell, with the on-deck state unset, r ≥ 416 and no bridge/parachute falling state, otherwise r′ = r; starting a crash does not set the falling state)
    let adjusted = if w.high_bridge(o.pos.cell())
        && !on_deck
        && !crate::fall::falling(w, id)
        && r >= BRIDGE_HEIGHT
    {
        r - BRIDGE_HEIGHT
    } else {
        r
    };
    crate::combat::effects::LayerHeights { r, adjusted }
}

// ---------------------------------------------------------------------------
// Orders (claims 16 and 17, 21)

/// The point of an order target.
fn point_of(w: &World, t: Target) -> Pos {
    match t {
        Target::Cell(c) => {
            let z = centre_height(w, c);
            // spec: S-0502 claim 16 (the centre of the cell at its terrain height, the deck height on a high-bridge cell)
            let z = if w.high_bridge(c) {
                z + BRIDGE_HEIGHT
            } else {
                z
            };
            Pos::centre(c, z)
        }
        Target::Object(x) => w.obj(x).map_or(Pos::default(), |o| o.pos),
    }
}

/// The standing point an infantry jumpjet takes in the found cell `c`
/// (claim 23): the desired point is the cell's centre class at world height
/// 0, so the choice reads ground occupancy unless the cell's high-bridge
/// deck at its centre is at or below world height 0, and the first free
/// side position wins after one ranged draw over 0 to 3 (S-0208 claim 3's
/// table O rows R0 to R3). `None` is a failed adjustment: the vehicle-present
/// or object-present refusal takes no draw; all side positions taken still
/// takes it.
fn standing_choice(w: &mut World, c: CellXY, z: i32) -> Option<Pos> {
    // spec: S-0502 claim 23 (the desired point has world height 0; ground occupancy, except when the cell has a high bridge whose deck at its centre is at or below world height 0, then deck occupancy)
    let layer = if w.high_bridge(c) && centre_height(w, c) + BRIDGE_HEIGHT <= 0 {
        Layer::Deck
    } else {
        Layer::Ground
    };
    // spec: S-0502 claim 23 (the vehicle-present refusal and the ground object-present or open-gate refusal of S-0208 claim 3 give no adjusted point and no draw; otherwise one ranged draw over 0 to 3 selects row R0 to R3, centre and top are never selected)
    let p = crate::walk::choose_position(w, c, layer, crate::cells::position::CENTRE, false)?;
    let (sx, sy) = standing_point(p);
    // spec: S-0204 claim 2 and S-0208 table P (a standing point within its cell), with the terrain height plus 416 on any high bridge (claim 23)
    Some(Pos::new(256 * c.x + sx, 256 * c.y + sy, z))
}

/// The three uses of the nearby-cell search of S-0502 claim 23.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Use {
    /// Claim 16, a vehicle that is not balloon-hover.
    MoveVehicle,
    /// Claim 16, an infantry unit or a balloon-hover vehicle.
    MoveInfantry,
    /// Claim 17, a stopped or failed landing.
    Stop,
}

/// The nearby-cell search of claims 16 and 17 (S-0212 claim 25) with the
/// settings of S-0502 claim 23: a 1 x 1-cell footprint, no further test, no
/// no-overlay condition, no reference cell, no reach region and no height
/// condition in every row.
fn search(w: &World, id: ObjectId, centre: CellXY, use_: Use) -> Option<CellXY> {
    let o = w.obj(id)?;
    let u = w.rules().unit(o.kind, o.type_index)?;
    let s = match use_ {
        Use::MoveVehicle | Use::MoveInfantry => crate::path::NearbySettings {
            // spec: S-0502 claims 16 and 23 (the hover locomotion class and the flying movement zone)
            class: Some(glasnost_rules::sim::LocomotionClass::Hover),
            zone: glasnost_rules::sim::MovementZone::Fly,
            reach: None,
            // spec: S-0502 claim 23 (deck mode never for a vehicle that is not balloon-hover; for an infantry unit or a balloon-hover vehicle when the centre carries a high bridge)
            deck: use_ == Use::MoveInfantry && w.high_bridge(centre),
            height: false,
            reference: None,
            // spec: S-0502 claim 23 (bridge cells excluded for the vehicle row, allowed for the other)
            no_bridges: use_ == Use::MoveVehicle,
            no_overlay: false,
        },
        // spec: S-0502 claim 23 (claim 17: the unit's current cell, its type's own class and its own movement zone, not simplified, no reach region, deck mode never, no height condition, bridge cells allowed)
        Use::Stop => crate::path::NearbySettings {
            class: u.class,
            zone: u.zone,
            reach: None,
            deck: false,
            height: false,
            reference: None,
            no_bridges: false,
            no_overlay: false,
        },
    };
    crate::path::nearby(w, centre, crate::path::NearbyUse::Settings(s))
}

/// The destination of a move order given to the locomotor (claim 16): the
/// order-level destination is already stored on the object.
pub fn take_destination(w: &mut World, id: ObjectId) {
    if !is_jumpjet(w, id) {
        return;
    }
    // spec: S-0607 claim 9 (a seized unit ignores every new destination; its pull and set-down are `crate::lift::steer`'s)
    if seized(w, id) {
        return;
    }
    let Some(o) = w.obj(id) else { return };
    let dest = o.dest;
    // spec: S-0502 claim 24 (Q-1030, revision 7: an infantry unit of a jumpjet type has no unchanged-destination suppression: repeating a nonempty destination repeats the movement-point adjustment and its draw, and an empty-to-empty assignment is an effective clear; only a vehicle ignores an unchanged order-level destination; equality is the same cell or the same object); S-0213 claim 21
    let infantry = o.kind == Kind::Infantry;
    // spec: S-0502 claim 27 and S-0207 claim 27 (pending destination refresh makes an unchanged vehicle assignment effective)
    let refreshed = crate::refresh::pending(w, id);
    if !infantry
        && !refreshed
        && st(w, id)
            .is_some_and(|j| j.order == dest && !(dest.is_some() && !j.moving && j.dest.is_none()))
    {
        return;
    }
    if let Some(t) = dest
        && own_cell_filtered(w, id, t)
    {
        // spec: S-0502 claim 23 and S-0213 claim 21 (outside an ion storm, a jumpjet-type infantry unit ignores a nonempty destination assignment whose cell equals its current cell, before any movement-point search, also while grounded; the order's other effects stand; the locomotor keeps its preceding order and point); Q-3043 (interim: the ignored destination is recorded so the locomotor does not take it later, #190)
        upd(w, id, |j| j.filtered = dest);
        return;
    }
    upd(w, id, |j| {
        j.order = dest;
        // Q-3043 (interim: an assignment that passes the filter ends the record, #190)
        j.filtered = None;
    });
    match dest {
        Some(t) => {
            // spec: S-0207 claim 27 and S-0502 claim 27 (the one-application suppression skips this nonempty movement point; the order-level destination changes without it)
            if !crate::refresh::skip_point(w, id) {
                order_point(w, id, t);
            }
        }
        None => {
            // spec: S-0502 claims 16, 17 and 24 (an effective order-level destination clear is not claim 16's direct no-point instruction: for a jumpjet with a movement commitment, whatever caused the clear, it invokes claim 17's reroute, which assigns a replacement movement point; with none, claim 17 does nothing; Q-1012 answered by revision 6)
            stop_with(w, id, false, false);
        }
    }
}

/// The own-cell filter of S-0502 claim 23: a jumpjet-type infantry unit
/// ignores a nonempty destination assignment whose cell is its current cell.
/// There is no ion storm in our engine, so the exception for one never
/// applies.
pub fn own_cell_filtered(w: &World, id: ObjectId, t: Target) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if o.kind != Kind::Infantry {
        return false;
    }
    // spec: S-0502 Import mapping (the filter concerns the imported JumpJet flag of the type, not merely the jumpjet locomotor)
    if !w
        .rules()
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.jumpjet)
    {
        return false;
    }
    point_of(w, t).cell() == o.pos.cell()
}

/// Whether the unit is a balloon-hover jumpjet vehicle (claim 28), the
/// kind of actor whose stop order makes a second destination request.
pub fn balloon_hover_vehicle(w: &World, id: ObjectId) -> bool {
    is_jumpjet(w, id) && balloon_vehicle(w, id)
}

/// Whether the unit is of a balloon-hover jumpjet type, vehicle or infantry
/// (S-0213 claim 8: stop makes a second destination request for the type).
pub fn balloon_hover_type(w: &World, id: ObjectId) -> bool {
    is_jumpjet(w, id) && balloon(w, id)
}

/// The signed wrapped difference of two facings, in -32768 to 32767.
fn facing_difference(a: Facing, b: Facing) -> i32 {
    // spec: S-0502 claim 28 (the signed wrapped difference in -32768 to 32767 facing units, integer arithmetic, no rounding)
    i32::from(a.wrapping_sub(b) as i16)
}

/// A request to clear the order-level destination (claim 28): a balloon-hover
/// vehicle with both a nonempty destination and a target rejects it when the
/// destination is the target itself, the destination is a cell whose first
/// ground-layer listed object is the target, or the horizontal direction to
/// the target is within 16384 facing units of the turret facing. A rejection
/// leaves the destination, movement point, commitment and landing reservation
/// as they are (no search, no draw) and makes *attack* current unless it is
/// already the seen behaviour. Returns whether the request was rejected; the
/// caller then does not clear the destination. Call it where the request is
/// made: after the order has installed its target, before the target is
/// cleared.
pub fn clear_rejected(w: &mut World, id: ObjectId) -> bool {
    if !balloon_hover_vehicle(w, id) {
        return false;
    }
    retention_rejects(w, id)
}

/// The deploy order's destination-clear request (S-0213 claim 10): as
/// [`clear_rejected`], and also for a vehicle of the hovering locomotor whose
/// type has the balloon-hover field, to which S-0503 claim 19 extends the
/// retention.
pub fn deploy_clear_rejected(w: &mut World, id: ObjectId) -> bool {
    // spec: S-0503 claim 19 (the balloon-hover retention of S-0213 claims 10 and 21 extended to the hovering locomotor: a hovering type with the balloon-hover field that meets the retention predicate keeps its order-level destination, D and stored path)
    if crate::hover::is_hover(w, id) && balloon_vehicle(w, id) {
        return retention_rejects(w, id);
    }
    clear_rejected(w, id)
}

/// Claim 28's retention predicate and its rejection, for a vehicle the
/// caller has found subject to it.
fn retention_rejects(w: &mut World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    // spec: S-0502 claim 28 (only an empty assignment is subject to it, and only with a nonempty destination and a target: having a target alone does not retain an empty destination)
    let (Some(dest), Some(target)) = (o.dest, o.target) else {
        return false;
    };
    let retain = match (dest, target) {
        // spec: S-0502 claim 28 (row 1: the destination is the target object itself)
        (Target::Object(d), Target::Object(t)) if d == t => true,
        // spec: S-0502 claim 28 (row 2: the destination is a cell and the target is its first ground-layer listed object)
        (Target::Cell(c), Target::Object(t))
            if w.cells.list(c, Layer::Ground).first() == Some(&t) =>
        {
            true
        }
        _ => false,
    } || {
        // spec: S-0502 claim 28 (row 3: the target direction, S-0204 claim 8's horizontal direction from the vehicle to its target, is at most 16384 facing units from the current turret facing, not the movement heading; a type without a turret uses the same facing; inclusive at 16384)
        let t = crate::process::target_pos(w, target);
        let d = direction(
            i64::from(t.x - o.pos.x),
            i64::from(t.y - o.pos.y),
            w.rules().quirks.table_direction,
        );
        facing_difference(d, o.turret.value(w.tick)).abs() <= 16384 // spec: S-0502 claim 28 (at most 16384 facing units)
    };
    if !retain {
        return false;
    }
    // spec: S-0502 claim 28 and S-0215 claim 13 (a rejection makes attack current and empties the queued behaviour unless the current behaviour, or the queued one when none is current, is already attack; claim 9's behaviour-start effects apply)
    let tick = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    if let Some(o) = w.obj_mut(id)
        && o.beh.seen() != Some(Behaviour::Attack)
    {
        o.beh.force(Behaviour::Attack, tick, stale);
    }
    true
}

/// A direct instruction giving the locomotor no point (claim 16, *No point*):
/// the movement point and the moving flag are cleared and nothing else
/// changes. While the jumpjet is moving now the instruction is refused and the
/// preceding movement stays (claim 24's safety departure).
pub fn no_point(w: &mut World, id: ObjectId) {
    if !is_jumpjet(w, id) {
        return;
    }
    // spec: S-0502 claim 24 (safety departure: refused while moving now, so the jumpjet never flies towards an absent point)
    if moving_now(w, id) {
        return;
    }
    // spec: S-0502 claim 16 (no point: the movement point and the moving flag are cleared, nothing else changes)
    upd(w, id, |j| {
        j.dest = None;
        j.moving = false;
    });
}

/// A destination given to the locomotor (claim 16), whether or not it is the
/// order-level destination.
fn order_point(w: &mut World, id: ObjectId, t: Target) {
    let phase = st(w, id).map_or(Phase::Grounded, |j| j.phase);
    let p = point_of(w, t);
    // spec: S-0502 claim 16 (it releases any landing reservation it holds, unless it is grounded or claim 24's safety refusal retains the preceding movement)
    let held = st(w, id).and_then(|j| j.reservation);
    if phase != Phase::Grounded {
        release_reservation(w, id);
    }
    let exact = balloon_vehicle(w, id) || (vehicle(w, id) && entering(w, id));
    let infantry = w.obj(id).is_some_and(|o| o.kind == Kind::Infantry);
    // spec: S-0502 claim 23 (the search settings of the use: infantry and balloon-hover vehicles have their own row)
    let use_ = if infantry || balloon_vehicle(w, id) {
        Use::MoveInfantry
    } else {
        Use::MoveVehicle
    };
    // spec: S-0502 claim 23 (Q-1011 answered by revision 6: claim 16's search does not lift a grounded unit's own occupancy; a nonempty searched record rejects the cell even when its only mark is the ordered unit's, S-0212 claim 25)
    let cell = search(w, id, p.cell(), use_);
    let found = cell.and_then(|c| {
        let z = point_of(w, Target::Cell(c)).z;
        if exact {
            // spec: S-0502 claim 16 (a balloon-hover vehicle and a vehicle whose current or queued behaviour is enter keep the exact point)
            Some(p)
        } else if infantry {
            // spec: S-0502 claim 16 and claim 23 (an infantry jumpjet takes a standing position in the found cell: the choice with its ranged draw)
            standing_choice(w, c, z)
        } else {
            Some(Pos::centre(c, z))
        }
    });
    let Some(d) = found else {
        // spec: S-0502 claim 24 (safety departure: a failed adjustment that would retain an off-map point is refused; the preceding point, moving flag, state, speeds, target height and landing reservation stay; no clamping to the edge)
        if w.cells.cell(p.cell()).is_none() {
            if let Some(m) = held
                && phase != Phase::Grounded
            {
                set_mark(w, id, m);
                upd(w, id, |j| j.reservation = Some(m));
            }
            return;
        }
        // spec: S-0502 claim 16 (adjustment finds no point: the ordered point itself remains the movement destination, with the moving flag, flight state, speeds and target height unchanged; a descending unit does not re-ascend; an idle unit does not start)
        upd(w, id, |j| j.dest = Some(p));
        return;
    };
    upd(w, id, |j| {
        j.dest = Some(d);
        j.moving = true;
    });
    // spec: S-0607 claim 13 (the stop's re-order changes neither a seized unit's flight state nor its speeds or target height)
    if phase == Phase::Descending && !seized(w, id) {
        // spec: S-0502 claim 16 (a descending jumpjet goes back to ascending with target height H; its landing request ends)
        let h = fields(w, id).map_or(0, |f| f.height);
        upd(w, id, |j| {
            j.phase = Phase::Ascending;
            j.ht = h;
            j.landing_request = false;
        });
        landing_cancelled(w, id);
    }
}

/// An *unload* in progress at a cancelled landing is replaced by *guard*
/// (S-0502 claim 16, S-0517 claim 8): *guard* is queued with the allowance
/// to interrupt movement.
fn landing_cancelled(w: &mut World, id: ObjectId) {
    if let Some(o) = w.obj_mut(id)
        && o.beh.current == Some(Behaviour::Unload)
    {
        // spec: S-0517 claim 8 (a destination given during descent: if unload is still current, guard is queued with permission to interrupt movement)
        o.beh.queue(Some(Behaviour::Guard));
        o.beh.allowance = true;
    }
}

/// Gives the jumpjet a new destination itself (a neighbour step or a
/// re-route): the order-level destination and the point.
fn give(w: &mut World, id: ObjectId, c: CellXY) {
    if let Some(o) = w.obj_mut(id) {
        o.dest = Some(Target::Cell(c));
    }
    upd(w, id, |j| j.order = None);
    take_destination(w, id);
}

/// Gives the locomotor a new movement point only (the reroute of a stop
/// order, claim 24): the order-level destination stays as it is.
fn give_point(w: &mut World, id: ObjectId, c: CellXY) {
    order_point(w, id, Target::Cell(c));
}

/// A neighbour step (claim 21): one ranged draw over 0 to 7 gives a step
/// direction whose neighbouring cell becomes the new destination; the
/// jumpjet enters *cruising*.
fn neighbour_step(w: &mut World, id: ObjectId) {
    let half = w.rules().quirks.ranged_draw_half_span;
    // spec: S-0502 claim 21 (one ranged draw over 0 to 7 from the scenario random stream; nothing tests the neighbour first)
    let d = w.rng.ranged(0, 7, half);
    let (dx, dy) = STEP_DIRECTIONS[(d.rem_euclid(8)) as usize]; // spec: S-0502 claim 21 (the step direction of the draw, 0 to 7)
    let c = pos(w, id).cell();
    give(w, id, CellXY::new(c.x + dx, c.y + dy));
    upd(w, id, |j| j.phase = Phase::Cruising);
}

/// Stops the movement of a moving jumpjet (claim 17): the reservation is
/// released and a nearby cell is searched; found, it re-orders itself to its
/// centre; not found, it destroys itself with no credit (or, with
/// `quirks.jumpjet_no_cell_self_destruct` off, clears its point and
/// commitment and hovers again at a height above the surface of 1 lepton or
/// more, or stays put and becomes grounded at 0 or less).
pub fn stop(w: &mut World, id: ObjectId) {
    stop_with(w, id, true, false);
}

/// A stop order (S-0213 claim 8, S-0502 claim 24) after it has cleared the
/// actor's order-level destination: a jumpjet with a movement commitment is
/// stopped by claim 17's reroute, which assigns a replacement movement point
/// and leaves the order-level destination empty. Without a commitment it does
/// nothing. It is not claim 16's direct instruction giving no point.
pub fn stop_ordered(w: &mut World, id: ObjectId) {
    if !is_jumpjet(w, id) {
        return;
    }
    // spec: S-0502 claims 24, 27 and 28 and S-0213 claims 8 and 21 (a redundant assignment is not an effective clear: an ordinary jumpjet vehicle ignores an unchanged order-level destination, empty-to-empty included, so no reroute happens and the preceding movement point and commitment survive; an infantry unit has no such suppression and reroutes again while its commitment lasts; pending destination refresh makes the repeat effective (S-0207 claim 27, answers Q-1090) and this assignment consumes it; the balloon-hover rejection is `clear_rejected`, made before this call)
    let refreshed = crate::refresh::pending(w, id);
    if vehicle(w, id) && !refreshed && st(w, id).is_some_and(|j| j.order.is_none()) {
        return;
    }
    if refreshed && let Some(o) = w.obj_mut(id) {
        o.econ.refresh = false;
    }
    // The order-level destination is empty, so the locomotor has nothing new to take from it.
    upd(w, id, |j| j.order = None);
    stop_with(w, id, false, false);
}

/// The stop that clearing a seized unit's destination runs when its grip
/// ends (S-0607 claim 13; S-0502 claims 17 and 24): an effective
/// order-level clear, so claim 17's stopped-movement rule runs on its
/// jumpjet movement in the same update. Not moving: nothing. A cell found:
/// its movement point becomes the re-order's point and it keeps its
/// movement commitment, with its flight state, speeds and target height
/// unchanged. No cell found: `lifter_ended` (every grip end of claim 12 but
/// claim 9's no-destination exception) takes S-0502 claim 17's
/// magnetic-release safety departure: no self-destruction, whatever
/// `quirks.jumpjet_no_cell_self_destruct` says, and the unit keeps no
/// movement point and is no longer moving; otherwise claim 17's ordinary
/// no-cell outcome applies, with its quirk switch and no credit.
pub fn lift_release_stop(w: &mut World, id: ObjectId, lifter_ended: bool) {
    if !is_jumpjet(w, id) {
        return;
    }
    // spec: S-0607 claim 13 (its destination becomes none: an effective order-level clear, S-0502 claim 24; the locomotor's record of the order follows)
    upd(w, id, |j| j.order = None);
    stop_with(w, id, false, lifter_ended);
}

/// Whether a jumpjet hovers with no movement point and no movement
/// commitment: for a seized set-down unit with the no-cell switch off, the
/// switch-off outcome of S-0502 claim 17 with no cell, whatever route
/// brought it there (S-0607 claims 9 and 20, revision 9). Restoration is not
/// tested for it; [`lift_retry_stop`] runs instead at the end of each of its
/// turns (revision 14).
pub fn hovering_uncommitted(w: &World, id: ObjectId) -> bool {
    // spec: S-0607 claim 20 (revision 9: hovering with no movement point and no movement commitment)
    st(w, id).is_some_and(|j| j.phase == Phase::Hovering && !j.moving && j.dest.is_none())
}

/// The repeated stop's search of a hovering set-down unit at the end of
/// each of its turns (S-0607 claim 20, revision 14, design): S-0502 claim 17's nearby-cell search
/// around its current cell. Found: it re-orders itself to that cell's
/// centre through S-0502 claim 16, as claim 13's stop does (a movement
/// point; the order-level destination stays none). Not found: nothing
/// changes. Returns whether a cell was found.
pub fn lift_retry_stop(w: &mut World, id: ObjectId) -> bool {
    let here = pos(w, id).cell();
    // spec: S-0607 claim 20 (revision 8, the hovering set-down unit: S-0502 claim 17's nearby-cell search around its current cell, for its type's own locomotion class and movement zone, with no draw)
    let Some(c) = search(w, id, here, Use::Stop) else {
        // spec: S-0607 claim 20 (revision 8: no cell found, it keeps hovering, is not restored and is not destroyed)
        return false;
    };
    // spec: S-0607 claim 20 (revision 8: a cell found, it re-orders itself to that cell's centre through S-0502 claim 16, exactly as claim 13's stop does; it is then claim 13's set-down unit that is still moving)
    give_point(w, id, c);
    true
}

fn stop_with(w: &mut World, id: ObjectId, order_level: bool, lifter_ended: bool) {
    if !moving(w, id) {
        // spec: S-0502 claim 17 (stopping a jumpjet that is not moving does nothing)
        return;
    }
    release_reservation(w, id);
    let here = pos(w, id).cell();
    // spec: S-0502 claims 17 and 23 (a nearby-cell search around its current cell, for its type's own locomotion class and own movement zone, with claim 23's settings)
    if let Some(c) = search(w, id, here, Use::Stop) {
        // spec: S-0502 claim 17 (it re-orders itself to that cell's centre, at its terrain height plus the bridge height on a high-bridge cell) and claim 24 (a player stop: the replacement is a movement point, the order-level destination stays empty)
        if order_level {
            give(w, id, c);
        } else {
            give_point(w, id, c);
        }
        return;
    }
    if lifter_ended {
        // spec: S-0502 claim 17 (revision 16, magnetic-release safety departure, design) and S-0607 claim 13 (revision 8: no cell found for a grip ended by its lifter: no self-destruction, whatever quirks.jumpjet_no_cell_self_destruct says; a dropped unit and a set-down unit alike keep no movement point, the one fixed at the seize cleared, and lose their movement commitment; a dropped unit falls as claim 15 says, a set-down one stays where it is)
        upd(w, id, |j| {
            j.dest = None;
            j.order = None;
            j.moving = false;
        });
        return;
    }
    if !w.rules().quirks.jumpjet_no_cell_self_destruct {
        // spec: S-0502 Edge cases (revision 17, quirks.jumpjet_no_cell_self_destruct off: the integer height above the surface decides, 1 lepton or more hovers, 0 or less stays put grounded) and S-0607 claims 9 and 20 (revision 9; answers Q-1344)
        if height_above(w, id) < 1 {
            // spec: S-0502 Edge cases (revision 17, quirks.jumpjet_no_cell_self_destruct off: at height 0 leptons or less it clears its movement point and commitment, stays put and becomes grounded, including from ascending at height 0; as at claim 15's touchdown, entering grounded releases the hover slot) and S-0607 claim 9 (revision 9: its flight state becomes grounded, even if it was ascending at height 0)
            release_slot(w, id);
            upd(w, id, |j| {
                j.phase = Phase::Grounded;
                j.ht = 0; // spec: S-0502 claims 5 and 15 (a grounded jumpjet's target height is 0, as touchdown leaves it, so a later take-off keeps claim 5's timing)
                j.dest = None;
                j.order = None;
                j.moving = false;
            });
            return;
        }
        // spec: S-0502 Edge cases (revision 17, quirks.jumpjet_no_cell_self_destruct off: at height 1 lepton or more it re-enters hovering over its current cell, with no movement point and no movement commitment, neither descending nor flying on)
        claim_slot(w, id, here);
        let h = fields(w, id).map_or(0, |f| f.height);
        upd(w, id, |j| {
            j.phase = Phase::Hovering;
            j.ht = h;
            j.dest = None;
            j.order = None;
            j.moving = false;
        });
        return;
    }
    if w.obj(id).is_some_and(|o| o.health > 0) {
        let health = w.obj(id).map_or(0, |o| o.health);
        let warhead = w.rules().combat.general.c4_warhead;
        // spec: S-0502 claim 17 (destroyed by damage equal to its remaining health with the demolition warhead, ignoring defences, with the no-escape marker and no attacker)
        damage::apply(
            w,
            id,
            Hit {
                amount: health,
                warhead,
                distance: 0,
                attacker: None,
                house: None,
                ignore_defences: true,
                no_escape: true,
            },
        );
    }
    if let Some(o) = w.obj_mut(id) {
        o.dest = None;
    }
    upd(w, id, |j| {
        j.dest = None;
        j.order = None;
    });
}

// ---------------------------------------------------------------------------
// Physics (claims 6, 8 to 10; F1 to F5)

/// The two-pi constant of F2, the stored double.
fn two_pi(mode: Rounding) -> F64 {
    let _ = mode;
    // spec: S-0502 F2 (2π denotes the stored double 6.283185307179586)
    glasnost_num::decimal::nearest_double("6.283185307179586")
}

/// F1: the speed update.
fn speed_step(j: &mut JumpState, f: &JumpFields, mode: Rounding) {
    let s = int(f.speed, mode);
    // spec: S-0502 F1 (q = min(v_c + a, S) when v_m > v_c, else v_c)
    let q = if j.vm.gt(j.vc) {
        j.vc.add(f.accel, mode).min(s)
    } else {
        j.vc
    };
    // spec: S-0502 F1 and claim 6 (braking by 1.5 × a, not below 0, when v_m < q)
    let brake = F64::from_i64(3, mode)
        .div(F64::from_i64(2, mode), mode)
        .mul(f.accel, mode);
    j.vc = if j.vm.lt(q) {
        q.sub(brake, mode).max(F64::ZERO)
    } else {
        q
    };
    // spec: S-0502 F1 and Edge cases (the speed fraction v_c′ / S; S = 0 divides by zero in the original, a deterministic 0 here, claim 25)
    j.fraction = if f.speed == 0 {
        F64::ZERO
    } else {
        j.vc.div(s, mode)
    };
}

/// The direct ground reference of F3.
fn direct_reference(w: &World, p: Pos, k: F64, mode: Rounding) -> i32 {
    let f = w.surface_at(p.x, p.y);
    // spec: S-0502 F3 (on a high-bridge cell, when z ≥ f + 416 − k use f + 416; otherwise f)
    if w.high_bridge(p.cell()) && int(p.z, mode).ge(int(f + BRIDGE_HEIGHT, mode).sub(k, mode)) {
        f + BRIDGE_HEIGHT
    } else {
        f
    }
}

/// The clearance of a cell (F3): its height at the centre plus the
/// allowance of what is listed in its ground layer (the jumpjet itself
/// included when it is listed there; S-0502 claim 22) and the bridge height.
fn clearance(w: &World, c: CellXY) -> (i32, i32) {
    let base = centre_height(w, c);
    let mut allowance = 0;
    let list = w.cells.list(c, Layer::Ground);
    // spec: S-0502 claim 22 (the actual ground-layer listings, not hover slots or reservations; a listed jumpjet supplies the 85-lepton allowance, including to itself, there is no self-exclusion; a deck-only listing supplies none)
    // spec: S-0502 claim 9 and F3 (a building listed in the cell's ground layer gives 104 leptons per level of its art height; otherwise infantry, a vehicle or an aircraft listed there gives 85; terrain objects, overlays and deck-only objects give nothing)
    if let Some(b) = list
        .iter()
        .filter_map(|&x| w.obj(x))
        .find(|x| x.kind == Kind::Building)
    {
        allowance = BUILDING_LEVEL
            * w.rules()
                .buildings
                .get(b.type_index as usize)
                .map_or(0, |t| t.building.art_height);
    } else if list
        .iter()
        .filter_map(|&x| w.obj(x))
        .any(|x| matches!(x.kind, Kind::Infantry | Kind::Vehicle | Kind::Aircraft))
    {
        allowance = OCCUPIED_CLEARANCE;
    }
    let bridge = if w.high_bridge(c) { BRIDGE_HEIGHT } else { 0 };
    (base + allowance, bridge)
}

/// F3: the ground reference with the look-ahead.
fn look_ahead(w: &World, id: ObjectId, j: &JumpState, mode: Rounding) -> i32 {
    let p = pos(w, id);
    let own = p.cell();
    let (c_base, c_bridge) = clearance(w, own);
    let mut c = c_base + c_bridge;
    let mut n = c;
    if j.vc.gt(F64::ZERO) {
        // spec: S-0502 F3 (N counts only while v_c > 0: the clearance of the neighbouring cell in the 8-direction of the movement heading, S-0204 claim 10)
        let e = usize::from(eight_direction(j.heading.value(w.tick)));
        let (dx, dy) = STEP_DIRECTIONS[e];
        let ahead = CellXY::new(own.x + dx, own.y + dy);
        let (n_base, n_bridge) = clearance(w, ahead);
        if w.rules().quirks.jumpjet_lookahead_bridge {
            // spec: S-0502 F3 and quirks.jumpjet_lookahead_bridge (the original adds the neighbouring cell's bridge height to the own cell's clearance instead of to N)
            c += n_bridge;
            n = n_base;
        } else {
            n = n_base + n_bridge;
        }
    }
    let _ = mode;
    // spec: S-0502 F3 (g = N when N > C; otherwise trunc((N + C) / 2))
    if n > c { n } else { (n + c) / 2 }
}

/// The ordinary height step of F2 to F4: the new world height, and whether
/// the ground hold stops the speed.
fn height_step(w: &World, id: ObjectId, j: &mut JumpState, f: &JumpFields, mode: Rounding) -> i32 {
    let p = pos(w, id);
    let phase = j.phase;
    let wobbles = matches!(phase, Phase::Hovering | Phase::Cruising) && !f.no_wobble;
    // spec: S-0502 F2 (φ′ = φ + 2π / (15 / W) in hovering and cruising without the no-wobble flag, otherwise 0; each division and the addition rounded separately)
    j.phi = if wobbles {
        let inner = int(15, mode).div(f.wobbles, mode);
        j.phi.add(two_pi(mode).div(inner, mode), mode)
    } else {
        F64::ZERO
    };
    // spec: S-0502 F2 (h_d = trunc(sin φ′ × d + h_t), table sine)
    let hd = sin_q(w, j.phi, mode)
        .mul(int(f.deviation, mode), mode)
        .add(int(j.ht, mode), mode)
        .to_i32_common();
    let direct = direct_reference(w, p, f.crash, mode);
    let over_dest = j.dest.is_some_and(|d| d.cell() == p.cell());
    // spec: S-0502 claim 9 and F3 (revision 14: the look-ahead in every state except grounded and descending, crashing included even at zero speed, when the movement point, not the order-level destination, is in another cell, and always for a balloon-hover type; otherwise, no movement point included, the direct reference)
    let elsewhere = j.dest.is_some_and(|d| d.cell() != p.cell());
    let look = !matches!(phase, Phase::Grounded | Phase::Descending)
        && (elsewhere || balloon_vehicle(w, id));
    let g = if look {
        look_ahead(w, id, j, mode)
    } else {
        direct
    };
    let u = p.z - g;
    let c = f.climb;
    // spec: S-0502 F4 (the climb or descent by c per tick, snapping to g + h_d when within one step; never below the terrain or the deck)
    let mut z = if u < hd {
        if int(u, mode).add(c, mode).ge(int(hd, mode)) {
            g + hd
        } else {
            int(p.z, mode).add(c, mode).to_i32_common()
        }
    } else if u > hd {
        if int(hd, mode).ge(int(u, mode).sub(c, mode)) {
            g + hd
        } else {
            int(p.z, mode).sub(c, mode).to_i32_common()
        }
    } else {
        p.z
    };
    z = z.max(direct);
    // spec: S-0502 claim 8 and F4 (ground hold: not over its destination cell and the height above the reference below half the desired height, truncated toward zero, sets the current speed to 0)
    if !over_dest && u.max(0) < hd / 2 {
        j.vc = F64::ZERO;
    }
    z
}

/// F5: the new horizontal position after a step of `v` leptons along the
/// heading `f`.
pub fn step_position(w: &World, p: Pos, heading: Facing, v: i32, mode: Rounding) -> (i32, i32) {
    if v == 0 {
        return (p.x, p.y);
    }
    if w.rules().quirks.jumpjet_step_truncation {
        // spec: S-0502 F5 and quirks.jumpjet_step_truncation (x′ = trunc(x + v cos θ), y′ = trunc(y − v sin θ), θ = (F_s − 16383) × (−2π / 65534), table trigonometry)
        let bias = crate::table_trig::bias(w);
        crate::walk::step_point_b(p.x, p.y, heading, v, true, mode, bias)
    } else {
        // spec: S-0502 Edge cases (step truncation off: x′ = x + round(v sin α), y′ = y − round(v cos α), α = 2πF / 65536, rounding half away from zero, exact functions)
        let (sx, cy) = glasnost_num::bigmath::rounded_sin_cos(heading, v);
        (p.x + sx, p.y - cy)
    }
}

/// Moves the jumpjet horizontally to (x, y). Its cell listing does not
/// follow: a hovering or cruising jumpjet keeps the listing it had in its
/// old cell, and any other state lists its new position again.
fn move_to(w: &mut World, id: ObjectId, x: i32, y: i32) {
    // spec: S-0502 claims 4 and 22 (hover and cruise cell crossings do not transfer a listing)
    if let Some(ob) = w.obj_mut(id) {
        ob.pos.x = x;
        ob.pos.y = y;
    }
}

/// Whether the unit's own type is a jumpjet type (glossary *jumpjet type*:
/// its type's locomotor is the jumpjet locomotor). A unit seized by a
/// magnetic beam flies with a temporary jumpjet locomotor but is not one.
pub fn jumpjet_type(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        w.rules()
            .unit(o.kind, o.type_index)
            .is_some_and(|u| u.locomotor == crate::rules::Locomotor::Jump)
    })
}

/// A unit moving with the jumpjet locomotor takes off (S-0502 claim 29):
/// it becomes a tracked flying object in the sector of the cell it is over,
/// read by the above-terrain detonations of S-0303 claim 6. A unit seized
/// by magnetic lift is registered by the same rule. Registration ends at
/// touchdown, at crash end or when it goes off the map ([`leaving`]); a
/// low unit can be registered and cell-listed at once (claim 31).
fn took_off(w: &mut World, id: ObjectId) {
    // spec: S-0502 claim 29 (revision 18, Q-1500 answered: registered in the update it takes off, leaving grounded for ascending, normally at height 0, seized units included; a unit already registered is not registered again)
    crate::combat::flying::register(w, id);
}

// ---------------------------------------------------------------------------
// Magnetic lift (S-0607): the jumpjet side of a seized unit's lift

/// Whether the last crash drop crossed a deck (claim 18).
pub fn crossed_deck(w: &World, id: ObjectId) -> bool {
    st(w, id).is_some_and(|j| j.crossed_deck)
}

/// Links the temporary jumpjet locomotor of a seized unit (S-0607 claim 5):
/// a unit of another locomotor gets a new jumpjet state, grounded at rest
/// with the heading east (S-0206 claim 14), listed where it stands; a unit
/// of a jumpjet type keeps its own state. Its own movement state is kept
/// for restoration.
pub fn link_lift(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let (cell, layer) = (o.pos.cell(), o.layer);
    if o.jump.is_none() {
        let mut j = JumpState::new();
        j.listed = Some((cell, layer));
        j.on_deck = layer == Layer::Deck;
        if let Some(ob) = w.obj_mut(id) {
            // spec: S-0607 claim 5 and S-0206 claim 14 (the jumpjet locomotor is linked; the heading is set at once to map east)
            ob.jump = Some(j);
        }
    } else {
        // spec: S-0607 claim 5 (it stops and stands still: a jumpjet type's own movement point and landing reservation go)
        release_reservation(w, id);
        upd(w, id, |j| {
            j.dest = None;
            j.order = None;
            j.landing_request = false;
        });
    }
}

/// A jumpjet type seized with another stand-in locomotor gives up its
/// marks (S-0607 claims 5 and 23): its ground mark, its landing
/// reservation and its hover slot go before its own state is kept aside.
pub fn give_up_marks(w: &mut World, id: ObjectId) {
    // spec: S-0607 claim 5 (it gives up the occupancy mark of its cell and stands still)
    lift_mark(w, id);
    release_reservation(w, id);
    release_slot(w, id);
}

/// A jumpjet type gets its own jumpjet locomotor back from another
/// stand-in (S-0607 claim 24, quirk off): its ground mark is laid at its
/// present position.
pub fn retake_marks(w: &mut World, id: ObjectId) {
    // spec: S-0502 claim 4 (the actual-position mark at its present position)
    lay_mark(w, id);
}

/// The movement point of a gripped unit, fixed at the seize (S-0607 claim
/// 7): S-0502 claim 16 applied to the lifter's point `p`. Returns the cell
/// that becomes the unit's order-level destination.
pub fn lift_point(w: &mut World, id: ObjectId, p: Pos) -> CellXY {
    // spec: S-0607 claim 7 (S-0502 claim 16's nearby-cell search around the cell of the lifter's point, with S-0212 table T5's jumpjet movement-point row for a vehicle that is not balloon-hover)
    let Some(c) = search(w, id, p.cell(), Use::MoveVehicle) else {
        // spec: S-0607 claim 7 (no cell found: the movement point is the lifter's point itself, with S-0502 claim 16's no-result consequences: the moving flag, flight state, speeds and target height unchanged) and claim 5 (its order-level destination becomes the cell of that point)
        upd(w, id, |j| {
            j.dest = Some(p);
            j.order = Some(Target::Cell(p.cell()));
        });
        return p.cell();
    };
    let z = point_of(w, Target::Cell(c)).z;
    // spec: S-0607 claim 7 (the centre of the cell found) and claim 5 (the order-level destination becomes the cell of that point, not the lifter)
    upd(w, id, |j| {
        j.dest = Some(Pos::centre(c, z));
        j.order = Some(Target::Cell(c));
        j.moving = true;
    });
    c
}

/// The fixed movement point's cell of a gripped unit (S-0607 claim 7), as
/// [`lift_point`] recorded it.
pub fn lift_order(w: &World, id: ObjectId) -> Option<Target> {
    st(w, id).and_then(|j| j.order)
}

/// A seized unit that is not gripped keeps no order-level destination
/// (S-0607 claim 9): the locomotor's record of it follows.
pub fn forget_order(w: &mut World, id: ObjectId) {
    upd(w, id, |j| j.order = None);
}

/// A dropped unit's orders stop (S-0607 claims 13 and 15): when it is not
/// exactly over its cell's centre it cruises there first. Returns whether
/// it does (the detour); otherwise its fall begins at once.
pub fn lift_dropped(w: &mut World, id: ObjectId) -> bool {
    let Some(p) = w.obj(id).map(|o| o.pos) else {
        return false;
    };
    upd(w, id, |j| {
        j.order = None;
        j.landing_request = false;
    });
    let c = p.cell();
    let centre = point_of(w, Target::Cell(c));
    // spec: S-0607 claim 15 (exactly over the centre of its current cell, S-0204 claim 3)
    if (p.x, p.y) == (centre.x, centre.y) {
        return false;
    }
    release_slot(w, id);
    release_reservation(w, id);
    // spec: S-0607 claims 13 and 15 (off its cell centre the detour replaces the stop's found point before the unit is steered by it: its destination becomes that centre and it keeps flying under S-0502's cruising rules)
    upd(w, id, |j| {
        j.dest = Some(centre);
        j.moving = true;
        j.phase = Phase::Cruising;
    });
    true
}

/// A dropped unit's fall begins (S-0607 claim 15, S-0502 claim 18):
/// *crashing* with target height 0, speeds 0, no move order. Its movement
/// point and commitment stay (S-0607 claim 13): a kept point in another
/// cell gives the fall S-0502 claim 9's look-ahead reference.
pub fn lift_fall(w: &mut World, id: ObjectId) {
    release_reservation(w, id);
    // spec: S-0502 claim 18 (crashing; a seized unit's speeds are 0) and S-0607 claim 15 (while a seized unit is crashing, its jumpjet target height is 0 in every update, not the ordinary crash's −5); S-0607 claim 13 (over its cell centre the fall begins with zero horizontal speed and the found point and the movement commitment are kept)
    upd(w, id, |j| {
        j.phase = Phase::Crashing;
        j.ht = 0;
        j.vc = F64::ZERO;
        j.vm = F64::ZERO;
        j.order = None;
        j.crash_started = false;
    });
}

/// A dropped unit's fall ends in *crashed* (S-0607 claim 17): it releases
/// its hover slot.
pub fn lift_crashed(w: &mut World, id: ObjectId) {
    // spec: S-0502 claim 29 (crash end: registration ends when it enters crashed)
    crate::combat::flying::unregister(w, id);
    release_slot(w, id);
    // spec: S-0607 claim 17 and S-0502 claim 18 (it releases its hover slot; maximum speed and target height 0; crashed)
    upd(w, id, |j| {
        j.vm = F64::ZERO;
        j.ht = 0;
        j.phase = Phase::Crashed;
        j.moving = false;
    });
}

/// A dropped unit is set sinking (S-0607 claim 17): its flight ends.
pub fn lift_sinking(w: &mut World, id: ObjectId) {
    lift_crashed(w, id);
}

/// A dropped unit whose fall crossed a bridge deck lands on it (S-0607
/// claims 15 and 17): it is put at deck height, on the deck.
pub fn lift_onto_deck(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let deck = w.surface_at(o.pos.x, o.pos.y) + BRIDGE_HEIGHT;
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0607 claim 15 (the fall ends when it crosses a bridge deck from at or above to below) and claim 17 (the unit landed on a bridge deck) and Edge cases (a survivor kept in play stands on the deck if its fall ended crossing a deck: it is put on the deck)
        ob.pos.z = deck;
        ob.layer = Layer::Deck;
    }
    upd(w, id, |j| j.on_deck = true);
}

/// A seized unit descends to land where it is (S-0607 claims 9 and 17): it
/// releases its hover slot and enters *descending*, its movement point the
/// surface beneath it, with no order-level destination.
pub fn lift_descend(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let r = height_above(w, id);
    let below = Pos::new(o.pos.x, o.pos.y, o.pos.z - r);
    release_slot(w, id);
    // spec: S-0607 claim 17 (it releases its hover slot and enters descending with no destination) and S-0502 claim 15 (descent: target height 0)
    upd(w, id, |j| {
        j.phase = Phase::Descending;
        j.ht = 0;
        j.vm = F64::ZERO;
        j.dest = Some(below);
        j.order = None;
        j.moving = false;
    });
}

/// A seized unit set down at the surface (S-0607 claim 13): grounded where
/// it stands.
pub fn lift_settle(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let layer = if o.layer == Layer::Deck || st(w, id).is_some_and(|j| j.on_deck) {
        Layer::Deck
    } else {
        Layer::Ground
    };
    let below = height_above(w, id).min(0);
    if below < 0
        && let Some(ob) = w.obj_mut(id)
    {
        // spec: S-0607 claim 13 and Edge cases (quirks.lift_impact_removes_survivor off: a survivor is placed on the surface below its impact point, grounded and set down)
        ob.pos.z -= below;
    }
    release_slot(w, id);
    release_reservation(w, id);
    upd(w, id, |j| {
        j.phase = Phase::Grounded;
        j.ht = 0;
        j.vc = F64::ZERO;
        j.vm = F64::ZERO;
        j.fraction = F64::ZERO;
        j.dest = None;
        j.order = None;
        j.moving = false;
    });
    // spec: S-0502 claim 29 (a set-down unit stops being registered at its touchdown)
    crate::combat::flying::unregister(w, id);
    settle(w, id, layer);
}

/// A seized unit gets its own locomotor back (S-0607 claim 20): the
/// temporary jumpjet locomotor's marks, reservation and slot go; a unit of
/// a jumpjet type keeps its jumpjet state, grounded at rest. Its cell
/// listing stays.
pub fn lift_restore(w: &mut World, id: ObjectId, jumpjet_type: bool) {
    if !is_jumpjet(w, id) {
        return;
    }
    if jumpjet_type {
        upd(w, id, |j| {
            j.dest = None;
            j.order = None;
            j.moving = false;
        });
        return;
    }
    lift_mark(w, id);
    release_reservation(w, id);
    release_slot(w, id);
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0607 claim 20 (the temporary jumpjet locomotor ends)
        ob.jump = None;
    }
}
