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
//! Seams to specs that are not built: magnetic lift (S-0607: [`seized`] is
//! never true), crate pickup at touchdown (S-0520 claim 6:
//! [`crate_pickup`]), ion storms (out of scope in a skirmish). The
//! deploy-to-land landing request of S-0517 is the field
//! [`JumpState::landing_request`], set and cleared by the deploy rules.

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_num::geom::{Facing, direction, eight_direction, root_floor};
use glasnost_num::trig::sin_t;
use glasnost_num::{F64, Rounding};
use serde::{Deserialize, Serialize};

use crate::combat::damage::{self, Hit};
use crate::rules::{Behaviour, JumpFields, Kind};
use crate::turning::Turning;
use crate::walk::{position_class, standing_point};
use crate::world::{Layer, ObjectId, Pos, Target, World};

mod fly;

pub use fly::{start_crash, tick};

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
            vc: F64::ZERO,
            vm: F64::ZERO,
            ht: 0,
            phi: F64::ZERO,
            heading: Turning::at(EAST),
            fraction: F64::ZERO,
            mark: None,
            reservation: None,
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

/// Whether the jumpjet is crashing or crashed (claim 18).
pub fn crashing(w: &World, id: ObjectId) -> bool {
    st(w, id)
        .is_some_and(|j| matches!(j.phase, Phase::Crashing | Phase::Crashed) || j.crash_started)
}

/// Whether the unit is held by magnetic lift (*seized*, S-0607 Terms).
/// Seam: magnetic lift is not built, so no unit is ever seized.
pub fn seized(w: &World, id: ObjectId) -> bool {
    let _ = (w, id);
    false
}

/// A crate in the landing cell is picked up at touchdown (S-0520 claim 6,
/// S-0502 claim 15). Seam: crates are not built.
pub fn crate_pickup(w: &mut World, id: ObjectId) {
    let _ = (w, id);
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

fn clear_mark(w: &mut World, id: ObjectId, m: Mark) {
    let q = occ_quirks(w);
    match m.position {
        Some(_) => w.cells.clear_infantry(m.cell, m.layer, id, q),
        None => w.cells.claim_vehicle(m.cell, m.layer, false),
    }
}

/// Lifts the ground occupancy mark.
fn lift_mark(w: &mut World, id: ObjectId) {
    let Some(m) = w
        .obj_mut(id)
        .and_then(|o| o.jump.as_mut())
        .and_then(|j| j.mark.take())
    else {
        return;
    };
    clear_mark(w, id, m);
}

/// Lays the ground mark at the present position (claim 4).
fn lay_mark(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let want = mark_at(w, id, o.pos, o.layer);
    if st(w, id).and_then(|j| j.mark) == Some(want) {
        return;
    }
    lift_mark(w, id);
    set_mark(w, id, want);
    upd(w, id, |j| j.mark = Some(want));
}

/// Releases the landing reservation.
fn release_reservation(w: &mut World, id: ObjectId) {
    let Some(m) = w
        .obj_mut(id)
        .and_then(|o| o.jump.as_mut())
        .and_then(|j| j.reservation.take())
    else {
        return;
    };
    clear_mark(w, id, m);
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
    upd(w, id, |j| {
        j.phase = Phase::Grounded;
        j.on_deck = layer == Layer::Deck;
        j.mark = Some(m);
    });
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
}

/// The jumpjet leaves the map or goes inside: every mark, the reservation
/// and the hover slot are released (claims 4, 7, 13, 14 and 18).
pub fn leaving(w: &mut World, id: ObjectId) {
    if !is_jumpjet(w, id) {
        return;
    }
    lift_mark(w, id);
    release_reservation(w, id);
    release_slot(w, id);
}

/// Takes the unit off the ground: its mark and cell listing go (claim 4).
fn leave_ground(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let (cell, layer) = (o.pos.cell(), o.layer);
    lift_mark(w, id);
    w.cells.leave(cell, layer, id);
    if let Some(ob) = w.obj_mut(id) {
        ob.layer = Layer::Ground;
    }
    upd(w, id, |j| j.on_deck = false);
    // spec: S-0211 claim 5 (an object leaving a cell recomputes it) and Q-0981 (interim: at take-off, not at crossings in the air)
    crate::load::recalc_cells(w, &[cell]);
}

/// Puts the unit on the ground at its position: its listing, and a mark.
fn settle(w: &mut World, id: ObjectId, layer: Layer) {
    let Some(o) = w.obj(id) else { return };
    let cell = o.pos.cell();
    w.cells.leave_both(cell, id);
    if let Some(ob) = w.obj_mut(id) {
        ob.layer = layer;
    }
    w.cells.enter(cell, layer, id, false);
    upd(w, id, |j| j.on_deck = layer == Layer::Deck);
    lay_mark(w, id);
    // spec: S-0211 claim 5 (an object put into a cell recomputes it) and Q-0981 (interim: at touchdown)
    crate::load::recalc_cells(w, &[cell]);
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

/// The standing point a free infantry position of a cell gives (claim 16).
fn free_standing_point(w: &World, id: ObjectId, c: CellXY, z: i32, from: Pos) -> Pos {
    let top = w.rules().quirks.top_position_counts_as_centre;
    let own = st(w, id).and_then(|j| j.reservation.or(j.mark));
    let layer = if w.high_bridge(c) && z > centre_height(w, c) {
        Layer::Deck
    } else {
        Layer::Ground
    };
    let free = |p: u8| {
        w.cells.occ(c, layer).is_some_and(|r| {
            !r.taken(p)
                || own.is_some_and(|m| m.cell == c && m.layer == layer && m.position == Some(p))
        })
    };
    // spec: S-0502 claim 16 (an infantry jumpjet takes a free standing position in the found cell) and Q-0975 (interim: the ordered point's own position, then the side positions in S-0208 table O's order for it, no draw, the centre when none is free)
    let class = position_class(from.x.rem_euclid(256), from.y.rem_euclid(256), top);
    let mut order = vec![class];
    order.extend(crate::walk::row_for(class, 0).iter().copied());
    let p = order.into_iter().find(|&p| free(p)).unwrap_or(0);
    let (sx, sy) = standing_point(p);
    Pos::new(256 * c.x + sx, 256 * c.y + sy, z) // spec: S-0204 claim 2 and S-0208 table P (a standing point within its cell)
}

/// The nearby-cell search of claims 16 and 17 (S-0212 claim 25).
fn search(w: &World, id: ObjectId, centre: CellXY, hover_class: bool) -> Option<CellXY> {
    let o = w.obj(id)?;
    let u = w.rules().unit(o.kind, o.type_index)?;
    let class = if hover_class {
        // spec: S-0502 claim 16 (for the hover locomotion class and the flying movement zone)
        Some(glasnost_rules::sim::LocomotionClass::Hover)
    } else {
        u.class
    };
    let zone = if hover_class {
        glasnost_rules::sim::MovementZone::Fly
    } else {
        u.zone
    };
    // spec: S-0502 claims 16 and 17 and Q-0975 (interim settings the claims leave open: no reach region, deck mode when the centre is a bridge cell, the height condition on, no reference cell, bridge cells allowed, no overlay condition)
    let s = crate::path::NearbySettings {
        class,
        zone,
        reach: None,
        deck: w.high_bridge(centre),
        height: true,
        reference: None,
        no_bridges: false,
        no_overlay: false,
    };
    crate::path::nearby(w, centre, crate::path::NearbyUse::Settings(s))
}

/// The destination of a move order given to the locomotor (claim 16): the
/// order-level destination is already stored on the object.
pub fn take_destination(w: &mut World, id: ObjectId) {
    if !is_jumpjet(w, id) {
        return;
    }
    let Some(o) = w.obj(id) else { return };
    let dest = o.dest;
    if st(w, id)
        .is_some_and(|j| j.order == dest && !(dest.is_some() && !j.moving && j.dest.is_none()))
    {
        return;
    }
    upd(w, id, |j| j.order = dest);
    let phase = st(w, id).map_or(Phase::Grounded, |j| j.phase);
    // spec: S-0502 claim 16 (it releases any landing reservation it holds, unless it is grounded)
    if phase != Phase::Grounded {
        release_reservation(w, id);
    }
    let Some(t) = dest else {
        // spec: S-0502 claim 16 (no point: the jumpjet stops being moving and nothing else happens)
        upd(w, id, |j| j.moving = false);
        return;
    };
    let p = point_of(w, t);
    let exact = balloon_vehicle(w, id) || (vehicle(w, id) && entering(w, id));
    let infantry = w.obj(id).is_some_and(|o| o.kind == Kind::Infantry);
    let found = if exact {
        // spec: S-0502 claim 16 (a balloon-hover vehicle and a vehicle whose current or queued behaviour is enter keep the exact point)
        Some(p)
    } else {
        // spec: S-0502 claim 16 and scenario J2 (an order to the unit's own cell keeps that cell: the unit's own mark does not count against the cell in the search; Q-0975)
        let own = st(w, id).and_then(|j| j.mark);
        lift_mark(w, id);
        let found = search(w, id, p.cell(), true);
        if own.is_some() {
            lay_mark(w, id);
        }
        found.map(|c| {
            let z = point_of(w, Target::Cell(c)).z;
            if infantry {
                free_standing_point(w, id, c, z, p)
            } else {
                Pos::centre(c, z)
            }
        })
    };
    let Some(d) = found else {
        // spec: S-0502 claim 16 (no cell found: the ordered point is kept as the destination and the moving state is left as it was; Q-0976)
        upd(w, id, |j| j.dest = Some(p));
        return;
    };
    upd(w, id, |j| {
        j.dest = Some(d);
        j.moving = true;
    });
    if phase == Phase::Descending {
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
/// `quirks.jumpjet_no_cell_self_destruct` off, hovers again).
pub fn stop(w: &mut World, id: ObjectId) {
    if !moving(w, id) {
        // spec: S-0502 claim 17 (stopping a jumpjet that is not moving does nothing)
        return;
    }
    release_reservation(w, id);
    let here = pos(w, id).cell();
    // spec: S-0502 claim 17 (a nearby-cell search around its current cell, for its type's own locomotion class and movement zone)
    if let Some(c) = search(w, id, here, false) {
        // spec: S-0502 claim 17 (it re-orders itself to that cell's centre, at its terrain height plus the bridge height on a high-bridge cell)
        give(w, id, c);
        return;
    }
    if !w.rules().quirks.jumpjet_no_cell_self_destruct {
        // spec: S-0502 Edge cases (quirks.jumpjet_no_cell_self_destruct off: it re-enters hovering over its current cell)
        claim_slot(w, id, here);
        let h = fields(w, id).map_or(0, |f| f.height);
        upd(w, id, |j| {
            j.phase = Phase::Hovering;
            j.ht = h;
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
    // spec: S-0502 F1 and Edge cases (the speed fraction v_c′ / S; S = 0 divides by zero in the original, a deterministic 0 here, Q-0977)
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
/// allowance of what stands in its ground layer (the jumpjet itself
/// excluded; Q-0978) and the bridge height.
fn clearance(w: &World, c: CellXY, skip: ObjectId) -> (i32, i32) {
    let base = centre_height(w, c);
    let mut allowance = 0;
    let list = w.cells.list(c, Layer::Ground);
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
        .filter(|&&x| x != skip)
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
    let (c_base, c_bridge) = clearance(w, own, id);
    let mut c = c_base + c_bridge;
    let mut n = c;
    if j.vc.gt(F64::ZERO) {
        // spec: S-0502 F3 (N counts only while v_c > 0: the clearance of the neighbouring cell in the 8-direction of the movement heading, S-0204 claim 10)
        let e = usize::from(eight_direction(j.heading.value(w.tick)));
        let (dx, dy) = STEP_DIRECTIONS[e];
        let ahead = CellXY::new(own.x + dx, own.y + dy);
        let (n_base, n_bridge) = clearance(w, ahead, id);
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
    let hd = sin_t(j.phi, mode)
        .mul(int(f.deviation, mode), mode)
        .add(int(j.ht, mode), mode)
        .to_i32_common();
    let direct = direct_reference(w, p, f.crash, mode);
    let over_dest = j.dest.is_some_and(|d| d.cell() == p.cell());
    // spec: S-0502 claim 9 (the look-ahead in every state except grounded and descending when the jumpjet is not over its destination cell, and always for a balloon-hover type; otherwise the direct reference)
    let look = !matches!(phase, Phase::Grounded | Phase::Descending)
        && (!over_dest || balloon_vehicle(w, id));
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
        crate::walk::step_point(p.x, p.y, heading, v, true, mode)
    } else {
        // spec: S-0502 Edge cases (step truncation off: x′ = x + round(v sin α), y′ = y − round(v cos α), α = 2πF / 65536, rounding half away from zero, exact functions)
        let (sx, cy) = glasnost_num::bigmath::rounded_sin_cos(heading, v);
        (p.x + sx, p.y - cy)
    }
}

/// Moves the jumpjet horizontally to (x, y), keeping its cell listing.
fn move_to(w: &mut World, id: ObjectId, x: i32, y: i32) {
    let Some(o) = w.obj(id) else { return };
    let (from, layer) = (o.pos.cell(), o.layer);
    let to = CellXY::new(x / 256, y / 256); // spec: S-0204 claim 2 (cell = (trunc(x / 256), trunc(y / 256)))
    if let Some(ob) = w.obj_mut(id) {
        ob.pos.x = x;
        ob.pos.y = y;
    }
    if from != to && w.cells.list(from, layer).contains(&id) {
        // A jumpjet on the ground (a slide at touchdown) keeps its listing.
        w.cells.leave(from, layer, id);
        w.cells.enter(to, layer, id, false);
    }
}
