//! Aircraft flight, attack runs and pads (S-0501).
//!
//! An *aircraft* here is an object of an aircraft type that moves with the
//! flying locomotor and is not a rocket missile (S-0501 Terms). Its flight
//! state lives in [`AirState`] on the object; a pad building keeps its
//! servicing state in the same record. The tick pipeline calls, in an
//! aircraft's turn, [`behaviour_turn`] (its processing), [`tick`] (the
//! locomotor) and [`end_turn`] (the end-of-update start, the pending round,
//! the run lock and the damage-smoke draw); in a pad building's turn,
//! [`pad_turn`].

use crate::rules::Locomotor;
use glasnost_map::CellXY;
use glasnost_num::F64;
use glasnost_num::geom::{Facing, direction, root_floor};
use glasnost_rules::sim::{LocomotionClass, MovementZone};

use crate::regions::{Reach, reach_read};
use serde::{Deserialize, Serialize};

use crate::combat::check::Answer;
use crate::rules::{Behaviour, Kind};
use crate::world::{HouseId, Layer, ObjectId, Pos, Target, World};

// ---------------------------------------------------------------------------
// Rules

/// The game-wide aircraft values (S-0501 Import mapping).
#[derive(Clone, Debug, Default)]
pub struct AirRules {
    /// The game-wide cruise height *FL*, leptons.
    pub flight_level: i32,
    /// The service cadence *P* of Formula F10, ticks.
    pub service: i32,
    /// The re-approach flag (claim 14).
    pub curley_shuffle: bool,
    /// The attacking-aircraft sight range, cells (claim 20).
    pub sight_range: i32,
    /// The spy-plane approach delay, ticks (claim 34).
    pub camera_frames: i32,
    /// The double 0.1 (F4's slow-speed floor and fraction step).
    pub tenth: F64,
    /// The double 0.05 (F4's creep fraction, claim 6's close-approach test).
    pub twentieth: F64,
    /// The double 0.5 (F4's halving, claim 5's middle band).
    pub half: F64,
    /// The double 0.75 (claim 5's outer band).
    pub three_quarters: F64,
}

impl AirRules {
    /// The aircraft rules of a resolved ruleset.
    pub fn new(r: &glasnost_rules::schema::Ruleset, mode: glasnost_num::Rounding) -> Self {
        let g = &r.sim.general.economy;
        let d = crate::rules::nearest_double;
        // spec: S-0501 Import mapping (ReloadRate, a double in game minutes, absent 0.05)
        let rate = g
            .reload_rate
            .as_ref()
            .map_or_else(|| d("0.05"), |x| x.value(mode));
        Self {
            // spec: S-0501 Import mapping ([General] FlightLevel absent: 500)
            flight_level: g.flight_level.unwrap_or(500),
            // spec: S-0501 F10 (P = ⌊r × 900⌋, the product in double precision truncated toward zero)
            service: crate::rules::rate_delay(rate, mode),
            // spec: S-0501 Import mapping (CurleyShuffle absent: no)
            curley_shuffle: g.curley_shuffle.unwrap_or(false),
            // spec: S-0501 Import mapping (AttackingAircraftSightRange absent: 5 cells)
            sight_range: g.attacking_aircraft_sight_range.unwrap_or(5),
            // spec: S-0501 Import mapping (SpyPlaneCameraFrames absent: 16 ticks)
            camera_frames: g.spy_plane_camera_frames.unwrap_or(16),
            // spec: S-0501 Constants (slow-speed floor and speed fraction step 0.1)
            tenth: d("0.1"),
            // spec: S-0501 Constants (creep fraction 0.05; close-approach landing fraction below 0.05)
            twentieth: d("0.05"),
            // spec: S-0501 F4 (f₀ × 0.5) and claim 5 (the 0.5 band)
            half: d("0.5"),
            // spec: S-0501 claim 5 (the 0.75 band)
            three_quarters: d("0.75"),
        }
    }
}

/// The per-type fields of an aircraft type that this spec reads.
#[derive(Clone, Debug, Default)]
struct Ty {
    landable: bool,
    bound: bool,
    fighter: bool,
    fly_by: bool,
    fly_back: bool,
    /// Cruise height *FL*.
    cruise: i32,
    /// Slowdown distance *S*.
    slowdown: i32,
    /// *V* (and F6's *V*ₜ), the converted type speed, leptons per tick.
    speed: i32,
    /// Dock types, in list order (building registration indices).
    docks: Vec<u32>,
    selectable: bool,
    spawned: bool,
    camera: bool,
    turn_rate: i32,
}

fn ty(w: &World, id: ObjectId) -> Option<Ty> {
    let o = w.obj(id)?;
    let r = w.rules();
    let u = r.unit(o.kind, o.type_index)?;
    let e = &u.raw.econ;
    let camera = u
        .primary
        .as_ref()
        .and_then(|n| r.weapons.get(n))
        .is_some_and(|wt| wt.camera.unwrap_or(false));
    Some(Ty {
        // spec: S-0501 Import mapping (Landable, AirportBound, Fighter, FlyBy, FlyBack: booleans, absent no)
        landable: e.landable.unwrap_or(false),
        bound: e.airport_bound.unwrap_or(false),
        fighter: e.fighter.unwrap_or(false),
        fly_by: e.fly_by.unwrap_or(false),
        fly_back: e.fly_back.unwrap_or(false),
        // spec: S-0501 Import mapping (a type's FlightLevel absent: the game-wide value)
        cruise: e.flight_level.unwrap_or(r.air.flight_level),
        // spec: S-0501 Import mapping (SlowdownDistance absent: 500, S-0207)
        slowdown: u.slowdown_distance,
        // spec: S-0501 claim 4 (V: the imported type speed, without bonuses); F6 (Vₜ: the same converted speed of S-0207 claim 1, not the raw field)
        speed: u.speed,
        // spec: S-0501 Import mapping (Dock: building types in list order, absent empty)
        docks: e
            .dock
            .iter()
            .flatten()
            .filter_map(|n| r.names.get(&(Kind::Building, n.clone())).copied())
            .collect(),
        // spec: S-0501 Import mapping (Selectable absent: yes; Spawned absent: no; Camera absent: no)
        selectable: u.raw.selectable.unwrap_or(true),
        spawned: u.raw.spawned.unwrap_or(false),
        camera,
        turn_rate: u.turn_rate,
    })
}

/// The type record `t` that a caller already holds for aircraft `id`, or,
/// without one, the record built now (a missing one reads as the default).
/// An object's type never changes, so a record held for the update is the
/// one [`ty`] would build again (#387).
fn ty_or<'a>(w: &World, id: ObjectId, t: Option<&'a Ty>) -> std::borrow::Cow<'a, Ty> {
    match t {
        Some(t) => std::borrow::Cow::Borrowed(t),
        None => std::borrow::Cow::Owned(ty(w, id).unwrap_or_default()),
    }
}

// ---------------------------------------------------------------------------
// State

/// The stage of an attack run (S-0501 table T1).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stage {
    /// Start.
    #[default]
    Start,
    /// Choosing a firing spot.
    Choosing,
    /// Approaching it.
    Approaching,
    /// Firing.
    Firing,
    /// Refiring (the single shot after a full burst).
    Refiring,
    /// A strafing pass, 2 to 5.
    Strafe(u8),
    /// Done.
    Done,
}

/// An aircraft's flight state (S-0501 Terms), or a pad building's
/// servicing state (claim 26).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AirState {
    /// The held height *FL*ₕ.
    pub held: i32,
    /// The current speed fraction.
    pub cur: F64,
    /// The target speed fraction.
    pub tgt: F64,
    /// Taking off.
    pub taking_off: bool,
    /// Landing.
    pub landing: bool,
    /// Settled on the ground: placed there, touched down or released onto
    /// its pad at the landing height, until it takes off. An aircraft that
    /// a factory release leaves above its pad is not settled (claim 48,
    /// [`AirState::released`]).
    pub ground: bool,
    /// The cell holding its object-present mark (S-0209 claim 1), while it
    /// is on the ground.
    pub mark: Option<CellXY>,
    /// The aircraft deck mark (claim 43): its own standing-on-deck state,
    /// not inferred from a layer or an occupancy flag.
    pub deck: bool,
    /// The holding-height mark (claim 6).
    pub holding: bool,
    /// The run lock (claim 31).
    pub run_lock: bool,
    /// The fall counter *k* of a crash (claim 9, F5).
    pub crash: Option<i32>,
    /// The locomotor is switched off (a crash started without a kill).
    pub switched_off: bool,
    /// The attack run's stage (table T1).
    pub stage: Stage,
    /// The start tick of the *attack* whose stage is kept.
    pub run_of: Option<i32>,
    /// A round of the run is pending (claim 15).
    pub pending: bool,
    /// Marked as a passenger carrier (claim 1).
    pub passenger_carrier: bool,
    /// A visiting aircraft (Terms).
    pub visiting: bool,
    /// Has entered the usable area since it appeared (claim 10).
    pub entered: bool,
    /// The pad it was last heading for (claim 21).
    pub heading_pad: Option<ObjectId>,
    /// The destination the locomotor was last given (claim 8: a new one
    /// makes an aircraft on the ground take off).
    pub flying_to: Option<Target>,
    /// For an object held in [`AirState::flying_to`], its heading point at
    /// the last hand-over (claims 8, 40 and 53): nothing reads the object's
    /// position or heading point again for the flight between hand-overs.
    pub handed: Option<Pos>,
    /// The destination object left play since the flight point was last
    /// handed over (claim 53): its flight flies on to that point with no
    /// destination object, and *move* reads it.
    pub lost: bool,
    /// The held flight point's world height was replaced by the terrain
    /// height there plus the cruise height when the point was accepted
    /// (claim 6: the aircraft then had a target and ammunition). Fixed with
    /// the point; cleared with it.
    pub raised: bool,
    /// A pad building's servicing: the tick of its next service step.
    pub service: Option<i32>,
    /// An unfinished flight (claim 40): a movement point accepted and not
    /// yet completed by settling at its landing destination.
    pub unfinished: bool,
    /// The start tick of the *enter* whose first processing was made
    /// (claim 24's 1-tick entry delay).
    pub enter_of: Option<i32>,
    /// *Enter* has recognised arrival on its pad and docks at its next
    /// processing (claim 24).
    pub enter_docking: bool,
    /// The ready mark (claim 31, r110, Q-2438) is cleared: state kept from
    /// one processing to the next, not a reading of the last one. Not
    /// cleared (the default) means set, as when the aircraft is made.
    pub mark_cleared: bool,
    /// The tick of the facing requests remembered below (claim 42).
    pub request_tick: i32,
    /// The pass counter (S-0522 Terms): passes it may still start without
    /// a successful drop. A paradrop plane starts with 5 (S-0522 claims 6
    /// to 8); a successful drop sets it to 5 (claims 8 and 11).
    pub passes: i32,
    /// A paradrop plane sent by a launch (S-0522 Terms): its passengers
    /// leave play silently with it when it leaves the map (claim 8).
    pub paradrop: bool,
    /// The body facing before its first request in `request_tick`, so that
    /// a later request in the same tick replaces it without a trace
    /// (claim 42, S-0206 claim 1).
    pub body_was: Option<crate::turning::Turning>,
    /// The aiming facing before its first request in `request_tick`.
    pub aim_was: Option<crate::turning::Turning>,
    /// Left above its pad by a factory release (claim 48), with the landing
    /// height 0 as its held height, until it takes the held height of
    /// ordinary flight (claim 48 (b)), starts to land, takes off or
    /// crashes.
    pub released: bool,
    /// Still filed in the ground render layer's list by its ground
    /// placement, with no re-file since (claim 48 (a) and (c), S-0102
    /// claim 4): set by a factory release that leaves it above the terrain,
    /// cleared at its next re-file moment. S-0309 claim 7's ground-layer
    /// pass and the drawing read it.
    pub ground_filed: bool,
    /// An aircraft whose type cannot land, still in the ground render
    /// layer's list while it flies (S-0306 r72 claim 17, *An aircraft that
    /// cannot land, in flight*; S-0102 claim 4: no take-off re-files such a
    /// type): set by a ground placement and at the end of a warp member's
    /// arrival hold, cleared at its next re-file moment, which for such a
    /// type is only a crash (`effects::layer_refile`). Only S-0309 claim
    /// 7's ground-layer pass reads it ([`kept_filed`]); the rules that read
    /// the layer from state and height do not.
    pub kept_filed: bool,
    /// The start tick of the *move* whose first processing gave the
    /// aircraft its destination, until that move's next processing gives
    /// its flight point once more (claim 40).
    pub regive: Option<i32>,
    /// The start tick of the *move* whose first processing has run (claim
    /// 40). A *move* started inside another behaviour's processing, such as
    /// the fall-back at a run's end (claim 17), has its "expired now" timer
    /// replaced by that processing's delay (S-0215 claims 2 and 3), so the
    /// start flag alone does not mark its first processing.
    pub first_moved: Option<i32>,
    /// The flight point was given with no destination object: being given
    /// no destination in flight, moving and not landing, the aircraft lands
    /// where it is (claim 6, r120, Q-2507). While it has no destination its
    /// flight flies to [`AirState::flying_to`]; cleared with that point.
    pub own_point: bool,
    /// The destination claim 8's exception ignored when it was handed to
    /// the flight (r120: the test is made only at hand-over): while the
    /// aircraft's destination is this one, its flight keeps its flight
    /// point.
    pub ignored: Option<Target>,
    /// A refused touchdown forgot the destination and the flight point
    /// (claim 7, r120): nothing moves the aircraft, which holds 10 leptons
    /// above the refused cell until it gets a destination.
    pub stranded: bool,
    /// On the ground by its height, not landing and not settled, with held
    /// height 0, after an untested touchdown that kept its flight point or
    /// the drop after a refusal that kept the destination (claim 7, r132,
    /// Q-2578, `quirks.aircraft_refused_kept_drop` on): it slides on with
    /// claim 4's forward step until the movement update after the first one
    /// that leaves its target fraction at 1, which climbs.
    pub sliding: bool,
    /// The flight point is the map's origin point, the point whose three
    /// coordinates are 0 (claim 7, r146, Q-2676): the survivor of the hit a
    /// landing check's refusal gives where no free cell is found. Its
    /// flight is unfinished, so it is moving, and [`AirState::flying_to`]
    /// holds no destination; while its destination is
    /// [`AirState::ignored`], the one it had at the hit, no movement update
    /// hands that destination to its flight. Claim 5's calculation of the
    /// target fraction, guidance and the close-approach trigger do not act
    /// on it. It ends when the aircraft is given a destination (claim 8)
    /// or a flight point of its own (claim 6).
    pub origin: bool,
}

// ---------------------------------------------------------------------------
// Tests on the aircraft

/// Whether `id` is an aircraft in this spec's sense: an aircraft-type
/// object with the flying locomotor that is not a rocket missile (S-0501
/// Terms).
pub fn is_aircraft(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        o.kind == Kind::Aircraft
            && o.rocket.is_none()
            && w.rules()
                .unit(o.kind, o.type_index)
                .is_some_and(|u| u.locomotor == Locomotor::Fly)
    })
}

fn st(w: &World, id: ObjectId) -> AirState {
    w.obj(id).and_then(|o| o.air.clone()).unwrap_or_default()
}

fn st_mut(w: &mut World, id: ObjectId) -> Option<&mut AirState> {
    let o = w.obj_mut(id)?;
    Some(o.air.get_or_insert_with(AirState::default))
}

/// The deck allowance in leptons: the height of a bridge deck above the
/// terrain.
// spec: S-0501 claim 43 (the 416-lepton deck allowance and reference)
const DECK_ALLOWANCE: i32 = 416;

/// Whether the aircraft carries its deck mark (S-0501 claim 43).
pub fn deck_marked(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .and_then(|o| o.air.as_ref())
        .is_some_and(|a| a.deck)
}

/// Whether altitude calculations use the bridge deck as the reference:
/// the aircraft is marked, or it is unmarked, over a high-bridge cell and at
/// least 416 leptons above the terrain (S-0501 Terms, *height*).
fn deck_reference(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if o.air.as_ref().is_some_and(|a| a.deck) {
        return true;
    }
    // spec: S-0501 Terms (height: an unmarked aircraft at least 416 leptons above terrain on a high-bridge cell also uses the deck as its reference; claim 43 distinguishes this temporary reference from the mark)
    w.high_bridge(o.pos.cell()) && o.pos.z - w.surface_at(o.pos.x, o.pos.y) >= DECK_ALLOWANCE
}

/// The world height that *h* = 0 stands for (S-0501 Terms): the terrain
/// surface, plus 416 leptons when the deck is the reference.
fn base(w: &World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id) else { return 0 };
    w.surface_at(o.pos.x, o.pos.y)
        + if deck_reference(w, id) {
            DECK_ALLOWANCE
        } else {
            0
        }
}

/// The height *h* above the terrain (S-0501 Terms): the signed world height
/// minus the terrain surface height at the aircraft's horizontal position,
/// minus 416 leptons when the deck is its reference (claim 43).
pub fn height(w: &World, id: ObjectId) -> i32 {
    w.obj(id).map_or(0, |o| o.pos.z - base(w, id))
}

/// The listing height in leptons (S-0209 claim 13): world height minus the
/// terrain surface height minus the deck allowance of 416 leptons when the
/// aircraft's own deck mark is set, in exact integer arithmetic.
pub fn listing_height(w: &World, id: ObjectId) -> i32 {
    // spec: S-0209 claim 13 (listing height = world height − terrain surface height − deck allowance; the allowance is 416 when the aircraft's own deck mark is set, else 0)
    w.obj(id).map_or(0, |o| {
        o.pos.z
            - w.surface_at(o.pos.x, o.pos.y)
            - if deck_marked(w, id) {
                DECK_ALLOWANCE
            } else {
                0
            }
    })
}

/// This spec's *on the ground* test (S-0501 Terms): *h* at most the
/// landing height 0, or the render layer its height gives it (claim 7) is
/// the ground layer, the same condition. It reads the height, never the
/// render-layer list the aircraft was last filed in: claim 48's released
/// aircraft, still filed in the ground render layer at a positive height,
/// is in flight.
pub fn on_ground(w: &World, id: ObjectId) -> bool {
    // spec: S-0501 Terms (on the ground: h ≤ landing height 0, or the ground layer its height gives it; the test reads height, not the render layer's list, r52) and claim 7 (the ground layer exactly when the height above the surface, the deck mark included, is at most 0; no hysteresis)
    listing_height(w, id) <= 0
}

/// Claim 48 (c)'s low release while it stays at height 0 (*At height 0*):
/// released above its pad and not yet climbing. It is on the ground by its
/// height (Terms), so claim 8's landing of an aircraft in flight that has
/// no destination does not apply to it.
fn low_release_at_height_0(w: &World, id: ObjectId, a: &AirState) -> bool {
    // spec: S-0501 claim 48 (c) (At height 0: while the low-released aircraft stays at height 0) and Terms (on the ground: h at most the landing height 0)
    a.released && height(w, id) <= 0
}

/// Whether the aircraft is still filed in the ground render layer's list
/// by its ground placement at a positive height, with no re-file since
/// (S-0501 claim 48 (a) and (c)): a factory release left it above its pad.
pub fn ground_filed(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .and_then(|o| o.air.as_ref())
        .is_some_and(|a| a.ground_filed)
}

/// Whether an aircraft whose type cannot land is still in the ground
/// render layer's list from a ground placement or a warp's arrival hold,
/// with no re-file since ([`AirState::kept_filed`]), whatever its height.
pub fn kept_filed(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .and_then(|o| o.air.as_ref())
        .is_some_and(|a| a.kept_filed)
}

/// The cell layer the aircraft is listed in (S-0209 claim 13), or none.
fn listing_layer(w: &World, id: ObjectId) -> Option<Layer> {
    // spec: S-0209 claim 13 (a positive listing height excludes the aircraft from both cell-layer lists; at 0 or below it is listed on deck when deck-marked, and on ground otherwise)
    if listing_height(w, id) > 0 {
        None
    } else if deck_marked(w, id) {
        Some(Layer::Deck)
    } else {
        Some(Layer::Ground)
    }
}

/// Brings the aircraft's cell listing in line with its listing height and
/// mark (S-0209 claim 13): taking off removes the listing when the listing
/// height becomes positive, and landing adds it again. A listing already
/// right is left alone, so the listing order does not change. Tracking as
/// a flying object is separate (S-0501 claim 47): it changes only at the
/// events of [`start_tracking`], [`end_tracking`] and [`sector_update`].
pub fn sync_listing(w: &mut World, id: ObjectId) {
    if !is_aircraft(w, id) {
        return;
    }
    let Some(o) = w.obj(id) else { return };
    if !o.on_map {
        // spec: S-0501 claim 47 (b) (it stops being tracked when it leaves the map in any way)
        end_tracking(w, id);
        return;
    }
    let cell = o.pos.cell();
    let want = listing_layer(w, id);
    for l in [Layer::Ground, Layer::Deck] {
        let listed = w.cells.list(cell, l).contains(&id);
        if want == Some(l) {
            if !listed {
                w.cells.enter(cell, l, id, false);
            }
        } else if listed {
            w.cells.leave(cell, l, id);
        }
    }
}

/// Whether the aircraft's type has the counts-as-aircraft flag (S-0501
/// Import mapping: absent, yes for an aircraft type).
fn counts_as_aircraft(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        w.rules()
            .econ
            .tech
            .of(crate::econ::tech::type_of(o))
            .is_some_and(|t| t.considered_aircraft)
    })
}

/// The aircraft starts being a tracked flying object (S-0501 claim 47
/// (a)), at the end of its current sector's list. One already tracked is
/// not added a second time.
fn start_tracking(w: &mut World, id: ObjectId) {
    // spec: S-0501 claim 47 (a) (an aircraft already tracked is not added a second time) and S-0303 claim 11 (a newly tracked object follows every object already registered, in its sector and globally)
    crate::combat::flying::register(w, id);
}

/// The aircraft stops being a tracked flying object (S-0501 claim 47 (b)).
fn end_tracking(w: &mut World, id: ObjectId) {
    crate::combat::flying::unregister(w, id);
}

/// The sector update at the start of a movement update, before its forward
/// step (S-0501 claim 47 (c)), from the speed fraction carried from the
/// previous update.
fn sector_update(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let cell = o.pos.cell();
    let a = st(w, id);
    // spec: S-0501 claim 47 (c) (no sector update while the speed fraction carried from the previous update is 0)
    if a.cur == F64::ZERO {
        return;
    }
    if crate::combat::flying::tracked(w, id) {
        // spec: S-0501 claim 47 (c) (in another cell than at its last start of tracking or sector update: when its cell lies in another sector than its list's, it joins the end of its current sector's list, S-0303 claim 11; otherwise its place does not change). Its list's sector is the sector of that last cell, so only a change of cell can change the sector.
        crate::combat::flying::moved(w, id);
    } else if !a.ground {
        // spec: S-0501 claim 47 (c) and claim 48 (c) (so does an aircraft that a factory release has left above its pad, whatever its height at that update: the low release at height 0 is not at rest and becomes tracked all the same) and r134 (Q-2647: so does one that an untested touchdown left holding its flight point, on the ground as in flight: an aircraft not at rest)
        // spec: S-0501 claim 47 (c) and Edge cases (quirks.aircraft_corner_sector_untracked on: an untracked aircraft in flight, one a factory release has left above its pad, or, r134, one an untested touchdown left holding its flight point, on the ground as in flight, whose cell lies in sector (0, 0) of S-0303 F7, cell (0, 0) included, joins no list; off: it joins sector (0, 0)'s list like any other)
        if w.rules().quirks.aircraft_corner_sector_untracked
            && crate::combat::flying::sector(w, cell) == (0, 0)
        {
            return;
        }
        // spec: S-0501 claim 47 (c) (an aircraft in flight that is not tracked becomes tracked at its first movement update with a positive carried fraction, at the end of its current sector's list)
        start_tracking(w, id);
    }
}

/// An aircraft goes off the map (inside a carrier, a transport or a
/// building): it stops being a tracked flying object (S-0501 claim 47
/// (b)).
pub fn leaving_map(w: &mut World, id: ObjectId) {
    if is_aircraft(w, id) {
        // spec: S-0501 claim 47 (b) (leaving the map in any way ends tracking, going inside a carrier, a transport or a building included)
        end_tracking(w, id);
    }
}

/// The in-the-air answer of an aircraft (S-0501 Terms): S-0307's
/// *airborne*.
pub fn in_the_air(w: &World, id: ObjectId) -> bool {
    crate::combat::weapon::airborne_by_height(w, id)
}

/// The number of passengers an aircraft carries now (S-0507 claim 3).
fn passengers(w: &World, id: ObjectId) -> i32 {
    crate::transport::count(w, id)
}

/// A passenger carrier (claim 1) that now carries none (claim 19).
pub fn empty_passenger_carrier(w: &World, id: ObjectId) -> bool {
    is_aircraft(w, id) && st(w, id).passenger_carrier && passengers(w, id) == 0
}

/// The primary weapon (S-0501 Terms, *armed*).
fn primary(w: &World, id: ObjectId) -> Option<crate::combat::data::WeaponId> {
    crate::combat::weapon::slot_weapon(w, id, 0)
}

fn armed(w: &World, id: ObjectId) -> bool {
    primary(w, id).is_some()
}

/// A strafer: its primary weapon's projectile has a signed turn rate at
/// most 1 and is not invisible (S-0501 Terms, claim 11).
pub fn strafer(w: &World, id: ObjectId) -> bool {
    primary(w, id).is_some_and(|wp| {
        w.rules()
            .combat
            .projectile_of(wp)
            // spec: S-0501 Terms (strafing weapon: projectile turn rate at most 1, not invisible)
            .is_some_and(|p| p.rot <= 1 && !p.inviso)
    })
}

/// A fighter (claim 11).
pub fn fighter(w: &World, id: ObjectId) -> bool {
    ty(w, id).is_some_and(|t| t.fighter)
}

/// The type's maximum ammunition *A* (−1: unlimited).
fn max_ammo(w: &World, id: ObjectId) -> i32 {
    crate::combat::veterancy::type_combat(w, id).map_or(-1, |t| t.ammo)
}

/// "Ammunition left", counting the round of the run still to be spent
/// (S-0501 T1).
pub fn ammo_left(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let pending = i32::from(o.air.as_ref().is_some_and(|a| a.pending));
    // spec: S-0501 T1 ("ammunition left" counts the round of the run still to be spent; −1 is unlimited)
    o.combat.ammo < 0 || o.combat.ammo - pending > 0
}

fn full_ammo(w: &World, id: ObjectId) -> bool {
    let a = max_ammo(w, id);
    w.obj(id).is_some_and(|o| a <= 0 || o.combat.ammo >= a)
}

fn full_health(w: &World, id: ObjectId) -> bool {
    let s = crate::combat::damage::strength(w, id);
    w.obj(id).is_some_and(|o| o.health >= s)
}

/// A human-controlled house's aircraft (our AI houses count as human,
/// glossary *AI house*).
fn human(w: &World, id: ObjectId) -> bool {
    crate::combat::acquire::player_owned(w, id)
}

/// Ready to change (S-0501 claim 31).
pub fn ready(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let cur = o.beh.current;
    // spec: S-0501 claim 31 (not ready in sticky or rescue)
    if matches!(cur, Some(Behaviour::Sticky | Behaviour::Rescue)) {
        return false;
    }
    let a = st(w, id);
    // spec: S-0501 claim 31 (not ready with the run lock, except in spy-plane approach)
    if a.run_lock && cur != Some(Behaviour::SpyPlaneApproach) {
        return false;
    }
    // spec: S-0501 claim 31 (r110, Q-2438: not ready when a processing has cleared its ready mark, a kept mark, until a processing sets it again; every change of behaviour, forced or from the queue, leaves it as it is) and Edge cases (quirks.aircraft_ready_mark_outlives_enter off: the cleared mark holds it not ready only while enter is current)
    !(a.mark_cleared
        && (w.rules().quirks.aircraft_ready_mark_outlives_enter || cur == Some(Behaviour::Enter)))
}

/// Sets the ready mark (claim 31, r110): at the processings of *enter* and
/// *guard* that claim names.
fn set_ready_mark(w: &mut World, id: ObjectId) {
    if let Some(a) = st_mut(w, id) {
        a.mark_cleared = false;
    }
}

/// With `quirks.aircraft_ready_mark_outlives_enter` off, a change out of
/// *enter* leaves the ready mark set (claim 31, Edge cases): applied at
/// the start and the end of the aircraft's update, so that it holds once
/// *enter* is current again. A change out and back within one update is
/// not seen.
fn ready_mark_after_enter(w: &mut World, id: ObjectId) {
    // spec: S-0501 Edge cases (quirks.aircraft_ready_mark_outlives_enter off: any change out of enter, forced or from the queue, leaves its ready mark set)
    if !w.rules().quirks.aircraft_ready_mark_outlives_enter
        && w.obj(id)
            .is_some_and(|o| o.beh.current != Some(Behaviour::Enter))
    {
        set_ready_mark(w, id);
    }
}

fn stale(w: &World) -> bool {
    w.rules().quirks.stale_suspended_revert // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
}

/// A start opportunity of claim 31: the queued behaviour becomes current
/// when the aircraft is ready.
fn start_if_ready(w: &mut World, id: ObjectId) {
    if !ready(w, id) {
        return;
    }
    let (t, s) = (w.tick, stale(w));
    if let Some(o) = w.obj_mut(id) {
        o.beh.start_next(t, s);
    }
}

pub(crate) fn queue(w: &mut World, id: ObjectId, b: Behaviour) {
    if queue_ignored(w, id, b) {
        return;
    }
    if let Some(o) = w.obj_mut(id) {
        o.beh.queue(Some(b));
    }
}

/// The five behaviours of claim 34 that hold an aircraft that is not a
/// strike aircraft.
fn exit_flight(b: Option<Behaviour>) -> bool {
    matches!(
        b,
        Some(
            Behaviour::Retreat
                | Behaviour::ParadropApproach
                | Behaviour::ParadropOverfly
                | Behaviour::SpyPlaneApproach
                | Behaviour::SpyPlaneOverfly
        )
    )
}

/// Whether a request to queue `b` for aircraft `id` is ignored entirely
/// (claim 34): an aircraft that is not a strike aircraft, in *retreat*,
/// *paradrop approach*, *paradrop overfly*, *spy-plane approach* or
/// *spy-plane overfly*, accepts only one of those five.
pub(crate) fn queue_ignored(w: &World, id: ObjectId, b: Behaviour) -> bool {
    // spec: S-0501 claim 34 (not a strike aircraft, in one of the five: a request to queue any other behaviour is ignored entirely, with no start, and its current behaviour goes on)
    is_aircraft(w, id)
        && w.obj(id).is_some_and(|o| exit_flight(o.beh.current))
        && !exit_flight(Some(b))
        && !crate::strike::linked(w, id)
}

pub(crate) fn forget(w: &mut World, id: ObjectId) {
    if let Some(o) = w.obj_mut(id) {
        crate::spawn::clear_own_target(o);
        o.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
        o.dest = None;
    }
}

pub(crate) fn set_dest(w: &mut World, id: ObjectId, d: Option<Target>) {
    crate::econ::assign_destination(w, id, d);
}

// ---------------------------------------------------------------------------
// Placement (claim 1)

/// An aircraft was put on the map at its requested position (S-0501 claim
/// 1); called by [`crate::objects::place`] before the placement fall-back.
///
/// A superweapon warp member's appearance at its landing point
/// (`superweapons::warp`, which calls this while the object is still a
/// member) does not follow claim 1's rule (S-0501 r111 claim 1, S-0513
/// r115 claim 36): it appears on the ground at its landing point's height,
/// whatever its visiting mark and the usable area. That height is its
/// surface, or the deck when the landing cell is a high-bridge cell, so the
/// deck mark is set then (claim 43's own standing-on-deck state).
pub fn placed(w: &mut World, id: ObjectId) {
    if !is_aircraft(w, id) {
        return;
    }
    let Some(t) = ty(w, id) else { return };
    let Some(o) = w.obj(id).cloned() else { return };
    let mut a = o.air.clone().unwrap_or_default();
    let cell = o.pos.cell();
    let usable = w.cells.usable(cell);
    let surface = w.surface_at(o.pos.x, o.pos.y);
    let warped = crate::superweapons::warp::is_member(w, id);
    // spec: S-0501 r111 claim 1 (a superweapon warp member's appearance at its landing point does not follow this rule: it appears at no height above its surface whatever its visiting mark, so an aircraft whose type cannot land does not appear at its cruise height) and S-0513 r115 claims 36 and 38 (the appearance at the height of claim 36, whatever its kind and visiting mark; not S-0501 claim 1's rule)
    // spec: S-0501 claim 1 (on the ground when not visiting, by the mark already present, and inside the usable area; else at cruise height)
    let ground = warped || (!a.visiting && usable);
    // spec: S-0306 r72 claim 17 (An aircraft that cannot land, in flight: the same holds for any such aircraft placed on the ground, warped or not; it stays in the ground list, S-0102 claim 4) and S-0102 claim 4 (a placement files it; no take-off of a type that cannot land re-files it)
    // spec: S-0306 r75 claim 17 (a warp member's appearance sets its height at no height above its surface, whatever its type's landing flag and visiting mark: an aircraft member whose type cannot land is then on the ground and keeps the ground list from there while it flies, until a crash)
    a.kept_filed = ground && !t.landable;
    // spec: S-0501 claim 1 (a type placed carrying passengers is a passenger carrier for life)
    a.passenger_carrier |= passengers(w, id) > 0;
    a.entered = usable;
    // spec: S-0513 r115 claim 36 (a member that does not move from one bridge deck to another appears on the deck when it lands on a high-bridge cell from the ground; on a high-bridge cell the bridge lift is added) and S-0501 claim 43 (the deck mark is the aircraft's own standing-on-deck state)
    let on_deck = warped && w.high_bridge(cell) && o.pos.z - surface >= DECK_ALLOWANCE;
    // spec: S-0501 claim 43 (ordinary fresh ground placement replaces the requested height with the terrain height; a placement is no way to require a deck landing: the mark is clear)
    a.deck = on_deck;
    let z = if ground {
        // spec: S-0501 claim 1 (ground placement replaces the requested world height with the terrain height)
        a.ground = true;
        a.held = 0;
        a.mark = Some(cell);
        if warped {
            // spec: S-0513 r115 claim 36 (the landing point's height: the surface, plus the bridge lift on a high-bridge cell; a member moving from one bridge deck to another is as that claim gives) and S-0306 r75 claim 17 (it stands at no height above its surface, on the deck when it lands on a high-bridge cell from the ground)
            o.pos.z
        } else {
            surface
        }
    } else {
        a.ground = false;
        a.held = t.cruise;
        surface + t.cruise
    };
    let mode = w.rules().mode;
    // spec: S-0501 claim 1 (the current speed fraction starts at 1 exactly at cruise height, else 0)
    // spec: S-0306 r75 claim 17 (an aircraft member stays at rest at that height after its hold, as an aircraft the map places on the ground does)
    a.cur = if !warped && z - surface == t.cruise {
        F64::from_i64(1, mode)
    } else {
        F64::ZERO
    };
    a.tgt = a.cur;
    if !ground {
        // An aircraft placed in flight sets no object-present mark (S-0209 claim 1).
        if let Some(rec) = w.cells.occ_mut(cell, Layer::Ground) {
            rec.others = rec.others.saturating_sub(1);
        }
        a.mark = None;
    }
    if let Some(ob) = w.obj_mut(id) {
        ob.pos.z = z;
        ob.layer = Layer::Ground;
        ob.air = Some(a);
    }
    sync_listing(w, id);
    // spec: S-0501 claim 47 (a) (placed with its in-the-air answer yes and its type's counts-as-aircraft flag set: tracked from the placement; placed on the ground, it is first tracked when its take-off starts)
    if !ground && in_the_air(w, id) && counts_as_aircraft(w, id) {
        start_tracking(w, id);
    }
}

/// The type test of a placement (S-0501 Terms and claim 1), called by
/// [`crate::objects::place`] after the placement fall-back: it sets the
/// visiting mark when the type is not both selectable and landable, or its
/// primary weapon is a camera weapon, and never clears it. The placement
/// height and the placement fall-back have seen only the mark already
/// present.
pub fn placed_type_test(w: &mut World, id: ObjectId) {
    if !is_aircraft(w, id) {
        return;
    }
    let Some(t) = ty(w, id) else { return };
    // spec: S-0501 Terms (placement sets the visiting mark when the type is not both selectable and landable, or its primary weapon is a camera weapon; it never clears it; the placement fall-back comes before this test, revision 41)
    let mark = !(t.selectable && t.landable) || t.camera;
    if let Some(a) = w.obj_mut(id).and_then(|o| o.air.as_mut()) {
        a.visiting |= mark;
    }
}

/// The marks an aircraft holds for removal (S-0209 claim 1): its
/// object-present mark while on the ground, none in flight. `None` for an
/// object that is not an aircraft of this spec.
pub fn removal_marks(w: &World, id: ObjectId) -> Option<Vec<(CellXY, Layer)>> {
    if !is_aircraft(w, id) {
        return None;
    }
    let layer = mark_layer(w, id);
    Some(st(w, id).mark.map(|c| vec![(c, layer)]).unwrap_or_default())
}

/// The occupancy record that carries an ordinary aircraft's object-present
/// flag at a marking or clearing event.
fn mark_layer(w: &World, id: ObjectId) -> Layer {
    // spec: S-0209 claim 1 (other objects) and S-0501 claim 44 (deck when a high bridge is present and the actual world height is at least the terrain surface height plus 416 leptons, signed integer leptons; otherwise ground; neither the listing nor the aircraft's own deck mark substitutes for this test)
    let Some(o) = w.obj(id) else {
        return Layer::Ground;
    };
    if w.high_bridge(o.pos.cell()) && o.pos.z - w.surface_at(o.pos.x, o.pos.y) >= DECK_ALLOWANCE {
        Layer::Deck
    } else {
        Layer::Ground
    }
}

/// Clears an aircraft's object-present mark (going inside a carrier).
pub fn clear_mark(w: &mut World, id: ObjectId) {
    let layer = mark_layer(w, id);
    let Some(c) = w
        .obj_mut(id)
        .and_then(|o| o.air.as_mut())
        .and_then(|a| a.mark.take())
    else {
        return;
    };
    if let Some(rec) = w.cells.occ_mut(c, layer) {
        rec.others = rec.others.saturating_sub(1);
    }
}

/// Sets a landed aircraft's object-present mark again (S-0607 claim 20: a
/// seized aircraft gets its own locomotor back).
pub fn ground_mark(w: &mut World, id: ObjectId) {
    set_mark(w, id);
}

fn set_mark(w: &mut World, id: ObjectId) {
    let Some(c) = w.obj(id).map(|o| o.pos.cell()) else {
        return;
    };
    if st(w, id).mark.is_some() {
        return;
    }
    // spec: S-0209 claim 1 (an aircraft on the ground sets the object-present flag) and S-0501 claim 44 (the deck record only at rest on a high-bridge deck, the ground record otherwise)
    let layer = mark_layer(w, id);
    if let Some(rec) = w.cells.occ_mut(c, layer) {
        rec.others = rec.others.saturating_add(1);
    }
    if let Some(a) = st_mut(w, id) {
        a.mark = Some(c);
    }
}

/// A newly built aircraft is released at its pad (S-0404 claims 13 and
/// 15, S-0417 claim 11, S-0501 claims 1 and 48): moved, in the tick of its
/// ground placement, to the pad position `p`, its docking position.
///
/// With `quirks.aircraft_released_above_pad` on, the height is `p`'s:
/// where that is above the terrain, the aircraft rests in the air there,
/// not settled, in neither cell layer, not tracked, still filed in the
/// ground render layer and holding the object-present flag in no cell
/// (claim 48). At or below the terrain it is settled there. Off, it is
/// settled at the landing height above the terrain at `p`'s horizontal
/// location, holding its flag there, like an aircraft docked by landing.
pub fn released_docked(w: &mut World, id: ObjectId, p: Pos) {
    if !is_aircraft(w, id) {
        return;
    }
    clear_mark(w, id);
    let above = w.rules().quirks.aircraft_released_above_pad;
    // spec: S-0501 claim 1 and S-0404 claim 15 (a factory release moves it to its docking position, height included, in the same tick) and Edge cases (quirks.aircraft_released_above_pad off: at the docking position's horizontal location, at the landing height 0 above the terrain there)
    let z = if above { p.z } else { w.surface_at(p.x, p.y) };
    if let Some(o) = w.obj_mut(id) {
        let from = o.pos.cell();
        o.pos = Pos::new(p.x, p.y, z);
        if from != p.cell() {
            w.cells.leave(from, Layer::Ground, id);
            w.cells.enter(p.cell(), Layer::Ground, id, false);
        }
    }
    // spec: S-0501 claim 48 (its height there, Terms, is positive: it rests above its pad)
    let raised = above && height(w, id) > 0;
    if let Some(a) = st_mut(w, id) {
        // spec: S-0501 claim 40 (settled on its contact building: no unfinished flight) and claim 48 (a) and (b) (released above its pad: not landing, no take-off under way, the landing height 0 held from its ground placement, its speed fractions 0)
        a.ground = !raised;
        a.held = 0;
        a.landing = false;
        a.taking_off = false;
        a.cur = F64::ZERO;
        a.tgt = F64::ZERO;
        a.unfinished = false;
        a.flying_to = None;
        a.raised = false;
        a.deck = false; // spec: S-0501 claim 43 (a pad grants no mark: the mark is clear on a pad)
        a.released = raised;
        // spec: S-0501 claim 48 (a) (its ground placement filed it in the ground render layer and the release raises it without a re-file, S-0102 claim 4)
        a.ground_filed = raised;
    }
    if raised {
        // spec: S-0501 claim 48 (e) (from the release tick on, resting above its pad, it holds the object-present flag in neither record of any cell: the clear_mark above took its ground placement's flag and none is set; the ground record of the cell holding the docking position's horizontal location ends with no object-present flag, even if another object had contributed one before the release, S-0209 claim 1: the flags are not counts)
        if let Some(rec) = w.cells.occ_mut(p.cell(), Layer::Ground) {
            rec.others = 0;
        }
    } else {
        // spec: S-0501 claim 44 and claim 48 (e) and Edge cases (released at or below the terrain, or with quirks.aircraft_released_above_pad off: it rests at the landing height in its cell layer, holding its flag in that cell like an aircraft docked by landing)
        set_mark(w, id);
    }
    // spec: S-0209 claim 13 (a positive listing height keeps it out of both cell layers: claim 48 (a))
    sync_listing(w, id);
    // spec: S-0501 claim 47 (b) (a landed aircraft resting on a pad is never tracked) and claim 48 (a) (nothing has made the released aircraft a tracked flying object)
    end_tracking(w, id);
}

/// Claim 24's contact at acceptance: aircraft `id` that accepts pad `dest`
/// as its destination while *enter* is current or queued and it holds no
/// contact at once asks the pad to be loaded; the pad takes it into contact
/// when it accepts it and has a free place (claims 22 and 23).
fn pad_accepted(w: &mut World, id: ObjectId, dest: Option<Target>) {
    let Some(Target::Object(k)) = dest else {
        return;
    };
    let Some(o) = w.obj(id) else { return };
    let entering =
        o.beh.current == Some(Behaviour::Enter) || o.beh.queued == Some(Behaviour::Enter);
    if !entering || o.contact.is_some() {
        return;
    }
    // spec: S-0501 Terms (a pad: a building whose type is one of the aircraft type's dock types)
    let pad = w.obj(k).is_some_and(|b| {
        b.kind == Kind::Building && ty(w, id).is_some_and(|t| t.docks.contains(&b.type_index))
    });
    if pad {
        // spec: S-0501 claim 24 (r93, Q-2251: the contact is made when the pad is accepted as destination while enter is current or queued and no contact is held: it asks the pad to be loaded, claim 23's request)
        request_contact(w, id, k);
    }
}

/// The aircraft took destination `dest` (S-0501 claim 48 (c)): an aircraft
/// that a factory release has left above its pad ends its pad contact at
/// the order when the destination is not that pad and the cell holding its
/// horizontal position belongs to the pad, whatever its height. An
/// aircraft standing on its pad ends it at its take-off (claim 8).
pub(crate) fn destination_taken(w: &mut World, id: ObjectId, dest: Option<Target>) {
    if !is_aircraft(w, id) {
        return;
    }
    // spec: S-0306 r80 claim 17 (Q-2526: for an aircraft warp member, a destination given while it phases in, its arrival hold included, does not hold; it is dropped as soon as it is given, as S-0207 claim 20 (a) drops one for an object that phases in after an own jump)
    if dest.is_some() && crate::superweapons::warp::phasing(w, id) {
        if let Some(o) = w.obj_mut(id) {
            o.dest = None;
        }
        return;
    }
    pad_accepted(w, id, dest);
    if let Some(d) = dest
        && st(w, id).flying_to == dest
        && !ignores(w, id, d)
    {
        // spec: S-0501 claims 8 and 53 (r183: the destination the flight already holds, given again by an order, a processing or another rule, is handed to the flight once more, so its heading point is taken again; claim 8's exception, when it ignores the hand-over, keeps the earlier point)
        let handed = handed_point(w, id, d);
        if let Some(a) = st_mut(w, id) {
            a.handed = handed;
            a.lost = false;
        }
    }
    if dest.is_some()
        && let Some(a) = st_mut(w, id)
        && a.origin
    {
        // spec: S-0501 claim 7 (r146, Q-2676: the survivor's origin point lasts until it is given a destination, by an order or by a processing of its behaviour, which is handed to its flight as any destination is, claim 8: the next movement update hands it over, the destination it kept included)
        a.ignored = None;
    }
    if dest.is_none() {
        given_no_destination(w, id);
    }
    if !st(w, id).released {
        return;
    }
    let Some(d) = dest else { return };
    let Some(k) = w.obj(id).and_then(|o| o.contact) else {
        return;
    };
    // spec: S-0501 claim 48 (c) and S-0417 claim 11 (its contact ends at the order when it takes a destination other than the pad; the rule reads the cell of its horizontal position, which must belong to the pad it is in contact with, not its height; a docking position outside the foundation leaves the contact held)
    // spec: S-0501 claim 48 (c) (Pad contact: a behaviour order, from a person or from our AI, has already ended a pad-bound type's contact by S-0213 table T3 (b), since S-0417 claim 11 leaves it untethered; so only a destination without a behaviour order, such as the release's rally move, or a tethered type, reaches this with its contact held outside the foundation, Q-1940)
    if d != Target::Object(k)
        && w.obj(k).is_some_and(|b| b.kind == Kind::Building)
        && pad_cell(w, k, id)
        && let Some(o) = w.obj_mut(id)
    {
        o.contact = None;
        o.econ.exit = None;
    }
}

/// The tick in which aircraft warp member `id`'s arrival hold ended, after
/// its behaviour processing (S-0306 r80 claim 17, Q-2526): with
/// `quirks.warp_hold_end_forgets_destination` on, a destination that
/// processing gave (`before` is the destination before it) is forgotten,
/// so it starts no take-off and the aircraft stays where it is until its
/// first destination that holds. Off, nothing is forgotten: S-0513 claim
/// 51's order alone, and the destination holds.
pub(crate) fn hold_end_forget(w: &mut World, id: ObjectId, before: Option<Target>) {
    if !is_aircraft(w, id) {
        // spec: S-0306 Edge cases (quirks.warp_hold_end_forgets_destination: for a member of another kind the switch is not defined)
        return;
    }
    // spec: S-0306 Edge cases (quirks.warp_hold_end_forgets_destination, default on = original; off: no step forgets a destination after the member's behaviour processing in that tick)
    if !w.rules().quirks.warp_hold_end_forgets_destination {
        return;
    }
    if let Some(o) = w.obj_mut(id)
        && o.dest.is_some()
        && o.dest != before
    {
        o.dest = None;
    }
}

/// Aircraft `id` is given no destination (claims 6 and 8, r120, Q-2507;
/// r132, Q-2576 and Q-2577): in flight and moving, it lands where it is. It
/// gets a new flight point with no destination object, the centre of the
/// cell claim 40's selection gives around the cell it is over now (or, over
/// a cell outside the usable area, around a cell on the local area's edge),
/// with that selection's draws; in *attack* with a target it keeps its
/// flight point (claim 42), and in *attack* with no target it takes an F8
/// landing spot. A landing aircraft whose selected cell is its flight
/// point's goes on landing; over another it takes off toward the new point.
/// An aircraft that is not moving (no flight point: after a refused
/// touchdown that forgot it, claim 7, or above claim 45's edge cell) is
/// left as it is. The stop order, which clears the destination without
/// [`crate::econ::assign_destination`], calls this too; the first movement
/// update after any other clear does ([`tick_flight`]).
pub(crate) fn given_no_destination(w: &mut World, id: ObjectId) {
    if !is_aircraft(w, id) {
        return;
    }
    let Some(o) = w.obj(id).cloned() else { return };
    let Some(a) = o.air.clone() else { return };
    if o.dest.is_some() || !o.on_map || crate::combat::damage::dead(&o) {
        return;
    }
    // spec: S-0501 claim 6 (r120, Q-2507: moving means that it holds a flight point or that its pitch is above zero; pitch is not modelled, claim 38, so it counts as 0; an aircraft claim 7 leaves with nothing to move it, which comes only with claim 7's quirks.aircraft_descent_touchdown_untested off, claim 45's above its edge cell included, holds none: being given no destination changes nothing for it; r132, Q-2576: a landing aircraft always holds one)
    // spec: S-0501 claim 7 (r146, Q-2676: the survivor of a refused check's hit holds the origin point as flight point and its flight stays unfinished, so it is still moving, claim 6)
    if a.flying_to.is_none() && !a.origin {
        return;
    }
    // spec: S-0501 claim 6 (r120: an aircraft in flight, Terms: not on the ground by its height; r134, Q-2647: one on the ground by its height that is moving, as one sliding on after an untested touchdown that kept its flight point, gets the new flight point like one in flight; an aircraft at rest holds none)
    if a.ground {
        return;
    }
    let grounded = on_ground(w, id) && !a.landing;
    let attack_current = o.beh.current == Some(Behaviour::Attack);
    let attack = attack_current || o.beh.queued == Some(Behaviour::Attack);
    if attack && o.target.is_some() {
        // spec: S-0501 claim 6 (r120; r132, Q-2576: landing too) and claim 42 (an aircraft in attack, current or queued, that holds a target keeps its flight point and unfinished flight when its destination is cleared)
        return;
    }
    let here = o.pos.cell();
    let cell = if attack_current {
        // spec: S-0501 claim 6 (r120, Q-2507: with attack current and no target, the cleared destination gives the flight an F8 landing spot as its flight point; r132, Q-2576: landing too, the F8 spot being the point handed to its flight and its cell the one compared) and claim 41 (F8 always gives a cell: at worst its own current cell)
        landing_spot(w, id).unwrap_or(here)
    } else {
        // spec: S-0501 claim 6 (r120, Q-2507: otherwise the centre of the cell claim 40's selection gives around the cell it is over at that moment; with attack queued, another behaviour current and no target too; r132, Q-2576: the same selection for a landing aircraft)
        let start = stop_start(w, id, &a, here);
        // spec: S-0501 claim 6 (r120: that cell itself when F7 and the flight-reservation test accept it, claim 40's test: F7 with the cell as the destination object, the reservation test without self; r132, Q-2577: from the start above, which F7 refuses outside the usable area)
        if landing_cell_ok(w, id, start, Target::Cell(start)) && reservation_ok(w, id, start, false)
        {
            start
        } else {
            // spec: S-0501 claim 6 (r120: else claim 40's replacement search around it, with that search's direction draws; r132, Q-2577: when every radius fails, the start cell's centre) and claim 40 (failure of all 32 radii retains the original cell)
            replacement_search(w, id, start).unwrap_or(start)
        }
    };
    if a.landing {
        let fc = a.flying_to.map(|f| flight_point(w, id, f).cell());
        if fc == Some(cell) {
            // spec: S-0501 claim 6 (r132, Q-2576: a landing aircraft is always moving; when the selected cell is its flight point's cell its flight ignores the centre, as claim 8's exception ignores a destination in that cell: its landing goes on with that flight point kept, and with no destination claim 4's descent move does not apply)
            if let Some(s) = st_mut(w, id) {
                s.own_point = true;
                s.ignored = None;
            }
            return;
        }
    }
    let raised = raise_now(w, id);
    if let Some(s) = st_mut(w, id) {
        // spec: S-0501 claim 6 (r120, Q-2507: a new flight point with no destination object, the centre of that cell; the aircraft flies to it)
        s.flying_to = Some(Target::Cell(cell));
        s.own_point = true;
        s.ignored = None;
        // spec: S-0501 claim 7 (r146, Q-2676: the new flight point replaces the origin point)
        s.origin = false;
        // spec: S-0501 claim 6 (the height replacement is fixed with the point when it is accepted: a target and ammunition now)
        s.raised = raised;
        // spec: S-0501 claim 40 (a movement point accepted by the locomotor is an unfinished flight)
        s.unfinished = true;
    }
    if a.landing {
        // spec: S-0501 claim 6 (r132, Q-2576: over another cell than its flight point's, it takes off toward that centre, handed to its flight only and not given as a destination, and lands where it is there) and claim 8 (a landing aircraft that gets a new point takes off: the held height becomes its cruise height)
        if let Some(t) = ty(w, id) {
            take_off(w, id, t.cruise);
        }
    } else if grounded && let Some(t) = ty(w, id) {
        // spec: S-0501 claim 6 (r134, Q-2647: being on the ground and not landing, it takes off toward that point, claim 8, which starts its tracking when it is not tracked yet, claim 47 (a), and lands where it is there, so its slide does not end where it was given no destination)
        take_off(w, id, t.cruise);
        if t.landable {
            // spec: S-0102 claim 4 (the first movement update after a take-off starts re-files, as at claim 8's take-off of an aircraft given a destination)
            crate::combat::effects::layer_refile(w, id);
        }
    }
}

/// The cell claim 6's selection starts from for an aircraft given no
/// destination over cell `here` (r132, Q-2577): `here`, except over a cell
/// outside the usable area for an aircraft that is not visiting and may not
/// leave the map now, with `quirks.aircraft_offmap_stop_start` on.
fn stop_start(w: &World, id: ObjectId, a: &AirState, here: CellXY) -> CellXY {
    // spec: S-0501 claim 6 (r132, Q-2577: over a cell outside the usable area by its height-aware test, S-0009 claims 4 and 5, for an aircraft that is not visiting and may not leave the map now, claim 10) and Edge cases (quirks.aircraft_offmap_stop_start on; off: the selection starts from the cell it is over, as anywhere else)
    if !w.rules().quirks.aircraft_offmap_stop_start
        || a.visiting
        || may_leave_now(w, id, a)
        || w.cells.usable(here)
    {
        return here;
    }
    let fp = w.cells.footprint();
    let [lx, ly, lw, lh] = fp.local;
    // spec: S-0501 claim 6 (r132: (a, b) the local point whose cell by S-0522 F2 is that cell, F2 being one-to-one) and S-0522 F2 (x + y = W + Y and x − y = 2X − W + (Y mod 2), so X = ⌊(x − y + W) / 2⌋ and Y = x + y − W, with X = a + Lx and Y = b + Ly)
    let mut la = (here.x - here.y + fp.width).div_euclid(2) - lx;
    let mut lb = here.x + here.y - fp.width - ly;
    // spec: S-0501 claim 6 (r132: a is compared with ⌊Lw / 2⌋ and b with ⌊Lh / 2⌋)
    let (ca, cb) = (lw.div_euclid(2), lh.div_euclid(2));
    if (la - ca).abs() < (lb - cb).abs() {
        // spec: S-0501 claim 6 (r132: when |a − ⌊Lw / 2⌋| < |b − ⌊Lh / 2⌋|, a becomes 1 if a < ⌊Lw / 2⌋ and Lw − 1 otherwise)
        la = if la < ca { 1 } else { lw - 1 };
    } else {
        // spec: S-0501 claim 6 (r132: otherwise, ties included, b becomes 1 if b < ⌊Lh / 2⌋ and Lh − 1 otherwise; the row compared and set in units of Lh, although the local area's rows run to 2Lh)
        lb = if lb < cb { 1 } else { lh - 1 };
    }
    local_cell(w, la, lb)
}

// ---------------------------------------------------------------------------
// Flight (claims 2 to 10)

/// F1: the climb step (claim 2).
pub fn climb(h: i32, held: i32, passengers: bool) -> i32 {
    // spec: S-0501 F1 (c = 10 with passengers, else 20)
    let c = if passengers { 10 } else { 20 };
    h + (held - h).min(c)
}

/// F2: the descent step (claim 3), with the dead-aircraft floor.
pub fn descend(h: i32, held: i32, dead: bool) -> i32 {
    // spec: S-0501 F2 (δ = min(50, max(20, ⌊(h − FLₕ) / 20⌋)))
    let delta = ((h - held) / 20).clamp(20, 50);
    let mut n = h - delta;
    if dead {
        // spec: S-0501 F2 (a dead aircraft: at least 1 lepton)
        n = n.max(1);
    }
    // spec: S-0501 F2 (then at least 0)
    n.max(0)
}

/// F3: the horizontal step from (x, y) along facing `f` by ⌊V × f⌋.
/// `unit_stall` is `quirks.aircraft_unit_step_stall`.
pub fn step(
    x: i32,
    y: i32,
    facing: Facing,
    v: i32,
    frac: F64,
    unit_stall: bool,
    mode: glasnost_num::Rounding,
) -> (i32, i32) {
    step_b(x, y, facing, v, frac, unit_stall, mode, true)
}

/// [`step`] with `quirks.table_trig_negative_bias` at `bias` (S-0105 F1).
#[allow(clippy::too_many_arguments)]
pub fn step_b(
    x: i32,
    y: i32,
    facing: Facing,
    v: i32,
    frac: F64,
    unit_stall: bool,
    mode: glasnost_num::Rounding,
    bias: bool,
) -> (i32, i32) {
    let s = step_length(v, frac, mode);
    // spec: S-0501 F3 (the table step of S-0208 F3 with θ = (Fₛ − 16383) × k, k = −2π / 65534; each new coordinate truncated, no remainder kept, so a 1-lepton step at facings 16391 to 32766 changes neither coordinate: quirks.aircraft_unit_step_stall)
    let (nx, ny) = crate::walk::step_point_b(x, y, facing, s, true, mode, bias);
    if unit_stall || s < 1 || (nx, ny) != (x, y) {
        return (nx, ny);
    }
    // spec: S-0501 F3 and Edge cases (quirks.aircraft_unit_step_stall off: a step of at least 1 lepton that would change neither coordinate moves 1 lepton along the axis of the larger component, in its direction, x on equal magnitudes)
    let (dx, dy) = crate::walk::step_components_b(facing, s, true, mode, bias);
    let unit = |c: F64| if c.is_sign_negative() { -1 } else { 1 };
    if dx.abs().ge(dy.abs()) {
        (x + unit(dx), y)
    } else {
        (x, y + unit(dy))
    }
}

/// F3's step length s = ⌊V × f⌋.
pub fn step_length(v: i32, frac: F64, mode: glasnost_num::Rounding) -> i32 {
    // spec: S-0501 F3 (s = ⌊V × f⌋ in double precision, truncated toward zero)
    F64::from_i64(i64::from(v), mode)
        .mul(frac, mode)
        .to_i32_common()
}

/// One adjustment of the current fraction toward τ by at most 0.1 (claim 5).
fn approach_fraction(f: F64, tau: F64, r: &AirRules, mode: glasnost_num::Rounding) -> F64 {
    // spec: S-0501 F4 (min(τ, f + 0.1) below τ, max(τ, f − 0.1) above, f when equal)
    if f.lt(tau) {
        tau.min(f.add(r.tenth, mode))
    } else if f.gt(tau) {
        tau.max(f.sub(r.tenth, mode))
    } else {
        f
    }
}

/// F4: ordinary destination-dependent slowing. `f0` is the fraction the
/// same tick's forward step has just used and `d` the distance from the
/// position after that step; the adjusted fraction moves the aircraft in
/// the next tick. Returns (τ, adjusted current fraction).
pub fn slowing(f0: F64, d: i32, s: i32, r: &AirRules, mode: glasnost_num::Rounding) -> (F64, F64) {
    let one = F64::from_i64(1, mode);
    let df = F64::from_i64(i64::from(d), mode);
    // spec: S-0501 F4 (S signed: for S ≠ 0, τ₀ = min(1, d / S) with floating-point division, so a negative S gives a negative quotient; for S = 0, τ₀ = 1 at every distance, d = 0 included, without a quotient); claim 5 (the ordinary calculation recomputes τ, so an earlier approach replacement has no effect here)
    let tau0 = if s != 0 {
        one.min(df.div(F64::from_i64(i64::from(s), mode), mode))
    } else {
        one
    };
    // spec: S-0501 F4 (τ₀ < 0.1 and d < 86: τ = 0, fₕ = f₀ × 0.5; else τ = max(0.1, τ₀))
    let (tau, fh) = if tau0.lt(r.tenth) && d < 86 {
        (F64::ZERO, f0.mul(r.half, mode))
    } else {
        (r.tenth.max(tau0), f0)
    };
    // spec: S-0501 F4 (f꜀ = min(fₕ, d))
    let fc = fh.min(df);
    // spec: S-0501 F4 (fₚ = 0.05 when τ = 0, f꜀ = 0 and d > 0)
    let fp = if tau.eq_num(F64::ZERO) && fc.eq_num(F64::ZERO) && d > 0 {
        r.twentieth
    } else {
        fc
    };
    (tau, approach_fraction(fp, tau, r, mode))
}

/// The crash contribution of F5: the counter's new value and *h*₁.
pub fn crash_step(k: i32, h: i32, dead: bool) -> (i32, i32) {
    // spec: S-0501 F5 (k ← k + 1 dead, k + 3 alive; h₁ = h − k)
    let k = k + if dead { 1 } else { 3 };
    (k, h - k)
}

/// The flight point of a destination (claim 40): its heading point with
/// the aircraft as requesting actor (S-0506 claim 14), the docking position
/// for its own pad contact (S-0404 claim 15), and claim 6's height for an
/// armed approach. For the held point (the destination the locomotor
/// accepted) the height replacement is the one fixed at acceptance
/// ([`AirState::raised`]); any other destination is read as it would be
/// accepted now.
pub fn flight_point(w: &World, id: ObjectId, d: Target) -> Pos {
    flight_point_of(w, id, d, None)
}

/// [`flight_point`] with the aircraft's type record `t` when the caller
/// holds it (#387).
fn flight_point_of(w: &World, id: ObjectId, d: Target, t: Option<&Ty>) -> Pos {
    let held = w.obj(id).and_then(|o| o.air.as_ref());
    // spec: S-0501 claim 6 (the replacement is made when a movement point is accepted while the aircraft has a target and ammunition; it concerns the flight point held for movement and is not recomputed when the target or ammunition later goes)
    let raised = match held {
        Some(a) if a.flying_to == Some(d) => a.raised,
        _ => raise_now(w, id),
    };
    // spec: S-0501 claim 53 (r183, Q-3041: nothing reads the object's position or heading point again for the flight between hand-overs: the held object's heading point is the one taken at the last hand-over) and claims 8 and 40
    let handed = match held {
        Some(a) if a.flying_to == Some(d) && matches!(d, Target::Object(_)) => a.handed,
        _ => None,
    };
    point_from(w, id, d, raised, handed, t)
}

/// Whether a movement point accepted now has its height replaced (claim
/// 6): the aircraft has a target and ammunition.
fn raise_now(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.target.is_some()) && ammo_left(w, id)
}

/// [`flight_point`] with the height replacement given and the
/// destination's heading point `handed` at the last hand-over, or read now
/// when `None`, and the type record `t` when the caller holds it.
fn point_from(
    w: &World,
    id: ObjectId,
    d: Target,
    raised: bool,
    handed: Option<Pos>,
    t: Option<&Ty>,
) -> Pos {
    // spec: S-0501 claim 40 (the destination's heading point with the aircraft as requesting actor: a cell's centre at its surface, a mobile object's next movement point or position, a building's centre)
    let mut p = handed.unwrap_or_else(|| heading_point_of(w, id, d, t));
    // spec: S-0501 claim 6 (a movement point accepted while the aircraft has a target and ammunition: its world height becomes the terrain height there plus the cruise height)
    if raised {
        let cruise = ty_or(w, id, t).cruise;
        p.z = w.surface_at(p.x, p.y) + cruise;
    }
    if let Target::Object(k) = d
        && w.obj(id).is_some_and(|o| o.contact == Some(k))
        && w.obj(k).is_some_and(|b| b.kind == Kind::Building)
    {
        // spec: S-0501 claim 40 (pad guidance then replaces a building point with the contacting aircraft's docking position, S-0404 claim 15, but only its horizontal location: the docking position's height is not applied, claim 24, r52)
        let q = crate::econ::release::pad_position(w, k, id).0;
        p.x = q.x;
        p.y = q.y;
    }
    p
}

/// The heading point of destination `d` with aircraft `id` as requesting
/// actor (claims 24 and 40): for an aircraft without a contact, a pad's
/// is its docking position without a contact place.
fn heading_point_for(w: &World, id: ObjectId, d: Target) -> Pos {
    heading_point_of(w, id, d, None)
}

/// [`heading_point_for`] with the type record `t` when the caller holds it
/// (#387).
fn heading_point_of(w: &World, id: ObjectId, d: Target, t: Option<&Ty>) -> Pos {
    let t = ty_or(w, id, t);
    // spec: S-0501 Terms (a pad: a building whose type is one of the aircraft type's dock types)
    if let Target::Object(k) = d
        && w.obj(id).is_some_and(|o| o.contact.is_none())
        && w.obj(k)
            .is_some_and(|b| b.kind == Kind::Building && t.docks.contains(&b.type_index))
    {
        // spec: S-0501 claim 24 (r103, Q-2366: for an aircraft without a contact, the pad's heading point is the docking position of its one docking place, height included, or its foundation centre when it has several places or none, S-0404 claim 15 and S-0506 claim 14; a pad that is neither a helipad nor a unit repair building gives its centre, S-0404 claim 15). Any other building keeps S-0506 claim 14's centre, whose docking-point cases are a seam there
        return crate::econ::release::pad_position_at(w, k, NO_PLACE);
    }
    heading_point(w, d)
}

/// The heading point of a destination (S-0506 claim 14).
fn heading_point(w: &World, d: Target) -> Pos {
    // spec: S-0501 claim 40 (S-0506 claim 14's destination heading point)
    crate::rocket::heading_point_with(w, d, true)
        .unwrap_or_else(|| crate::process::target_pos(w, d))
}

/// The truncated table root of the planar squared difference (S-0204
/// claim 5).
pub(crate) fn planar(w: &World, a: Pos, b: Pos) -> i32 {
    let dx = i64::from(a.x - b.x);
    let dy = i64::from(a.y - b.y);
    let q = w.rules().quirks;
    root_floor(
        (dx * dx + dy * dy) as u64,
        q.table_root_distance,
        w.rules().mode,
    )
    .min(i32::MAX as u64) as i32
}

fn toward(w: &World, from: Pos, to: Pos) -> Facing {
    direction(
        i64::from(to.x - from.x),
        i64::from(to.y - from.y),
        w.rules().quirks.table_direction,
    )
}

/// Whether the locomotor has stopped (claim 40): no unfinished flight and
/// no positive forward pitch. Pitch is not modelled (claim 38 leaves the
/// pitch variants out), so it counts as 0.
pub fn stopped(w: &World, id: ObjectId) -> bool {
    // spec: S-0501 claim 40 (stopped only with no unfinished flight and no positive forward pitch; not the zero-current-fraction test)
    !st(w, id).unfinished
}

/// Which facing of an aircraft a request turns (claim 42).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Facet {
    /// The body facing, which gives the direction of travel (claim 4).
    Body,
    /// The aiming facing, which the firing check compares (claim 19).
    Aim,
}

/// Requests facing `which` of aircraft `id` toward `f`. Both facings turn
/// at the type's turn rate (S-0206 claim 7). A facing turns toward the
/// last direction requested in a tick; an earlier request of the same tick
/// is undone before the new one, so it has no lasting effect (claim 42,
/// S-0206 claim 1).
fn request(w: &mut World, id: ObjectId, which: Facet, f: Facing) {
    // spec: S-0206 claim 7 (an aircraft's body and turret facings turn at the type's turn rate)
    let rate = ty(w, id).map_or(0, |t| t.turn_rate);
    request_at(w, id, which, f, rate);
}

/// [`request`] with the type's turn rate given by a caller that holds the
/// type record (#387).
fn request_at(w: &mut World, id: ObjectId, which: Facet, f: Facing, rate: i32) {
    let tick = w.tick;
    let mut a = st(w, id);
    if a.request_tick != tick {
        a.request_tick = tick;
        a.body_was = None;
        a.aim_was = None;
    }
    let Some(o) = w.obj_mut(id) else { return };
    let (facing, was) = match which {
        Facet::Body => (&mut o.body, &mut a.body_was),
        Facet::Aim => (&mut o.turret, &mut a.aim_was),
    };
    // spec: S-0501 claim 42 (a request replaced later in the same tick has no lasting effect)
    match was {
        Some(x) => *facing = *x,
        None => *was = Some(*facing),
    }
    // spec: S-0206 claim 1 (the facing turns toward the requested direction)
    facing.turn_to(f, tick, rate);
    o.air = Some(a);
}

/// The destination whose flight point the aircraft flies to: its
/// destination, or, after that was cleared in *attack* (current or
/// queued) while it has a target, the destination of its still unfinished
/// flight (claim 42), or, landing where it is, the flight point its cleared
/// destination gave it (claim 6, [`AirState::own_point`]).
fn flight_target(w: &World, id: ObjectId) -> Option<Target> {
    let o = w.obj(id)?;
    let a = st(w, id);
    if a.origin && o.dest == a.ignored {
        // spec: S-0501 claim 7 (r146, Q-2676: the survivor's movement updates treat a flight point at the origin point as no flight point for claim 5's calculation of the target fraction and for guidance, claim 42; no movement update hands its destination to its flight again, with a destination kept or with none, until it is given a destination, claim 8)
        return None;
    }
    if let Some(d) = o.dest {
        if let Target::Object(x) = d
            && w.obj(x).is_none()
        {
            // spec: S-0501 claim 53 (r183: no movement update reads the gone object; `destination_lost` clears such a destination when the object leaves play, so this guards only an object taken out of the world by a path that skips that moment, for which there is no point to read) (#160)
            return None;
        }
        return Some(d);
    }
    if a.own_point {
        // spec: S-0501 claim 6 (r120, Q-2507: landing where it is, the flight flies to the new flight point with no destination object)
        return a.flying_to;
    }
    let attack =
        o.beh.current == Some(Behaviour::Attack) || o.beh.queued == Some(Behaviour::Attack);
    // spec: S-0501 claim 42 (clearing the destination of an aircraft in attack, current or queued, that has a target does not end its flight: the flight point and unfinished flight remain)
    if attack && o.target.is_some() && a.unfinished {
        a.flying_to
    } else {
        None
    }
}

/// Claim 42's guidance, after the movement tick of a living aircraft that
/// has a flight point, is neither landing nor taking off and is above the
/// terrain: the body toward the flight point, the aiming facing toward the
/// target, the landing direction or the flight point. These are the last
/// facing requests of the update.
fn guidance(w: &mut World, id: ObjectId, t: &Ty) {
    let a = st(w, id);
    // spec: S-0501 claim 42 (after each movement tick of a living aircraft that has a flight point, is neither landing nor taking off and is above the terrain; a settled aircraft has completed its flight, claim 40) and claim 48 (c) (At height 0: guidance applies only above the terrain, so it requests no facing for the low release at height 0, whose take-off steering, claim 8, keeps the released facing up to trunc(H / 2))
    if a.landing || a.taking_off || a.ground || height(w, id) <= 0 {
        return;
    }
    // spec: S-0501 claim 42 (with the run lock neither the body nor the aiming facing is requested)
    if a.run_lock {
        return;
    }
    let Some(d) = flight_target(w, id) else {
        return;
    };
    let Some(o) = w.obj(id) else { return };
    if crate::combat::damage::dead(o) {
        return;
    }
    let (here, body, target) = (o.pos, o.body, o.target);
    let p = flight_point_of(w, id, d, Some(t));
    // spec: S-0501 claim 42 (the bearing of S-0204 claim 8 from its current position)
    let bearing = toward(w, here, p);
    // spec: S-0501 claim 42 (the distance to the flight point after any pad-position adjustment)
    let dist = planar(w, here, p);
    let is_strafer = strafer(w, id);
    // spec: S-0501 claim 42 (a strafer at most 768 leptons from the point requests its body only when the bearing differs from its current body facing by at most 8192 facing units)
    let strafer_skips = is_strafer
        && dist <= STRAFER_WINDOW
        && glasnost_num::geom::delta(body.value(w.tick), bearing).abs() > STRAFER_TURN;
    if !strafer_skips {
        // spec: S-0501 claim 42 (the body facing is requested toward the flight point)
        request_at(w, id, Facet::Body, bearing, t.turn_rate);
    }
    let armed_target = target.is_some() && ammo_left(w, id);
    // spec: S-0501 claim 42 (below 256 leptons, no fly-by flag, not a strafer with a target and ammunition left: the aiming facing toward the target with a target and ammunition left, otherwise toward the landing direction of claim 25; in every other case toward the flight point)
    let aim = if dist < AIM_NEAR && !t.fly_by && !(is_strafer && armed_target) {
        match target {
            // spec: S-0204 claim 8 and S-0310 F1 (the direction of the target: toward its reference point)
            Some(tg) if armed_target => toward(w, here, crate::combat::range::target_point(w, tg)),
            _ => landing_facing(w, id),
        }
    } else {
        bearing
    };
    request_at(w, id, Facet::Aim, aim, t.turn_rate);
}

/// Moving in the sense of S-0501 claim 26 (its locomotor is moving).
pub fn moving(w: &World, id: ObjectId) -> bool {
    !stopped(w, id)
}

/// Whether the fixed-full-speed case of claim 5 applies.
fn full_speed(w: &World, id: ObjectId, t: &Ty, a: &AirState) -> bool {
    // spec: S-0501 claim 5 (the target is fixed at 1 with the run lock, or holding height with the fly-by flag or as a strafer or fighter with ammunition left)
    a.run_lock || (a.holding && (t.fly_by || ((strafer(w, id) || t.fighter) && ammo_left(w, id))))
}

/// Takes off toward a new destination (claim 8).
fn take_off(w: &mut World, id: ObjectId, cruise: i32) {
    // spec: S-0501 claim 47 (a) (tracking starts when the take-off starts, at the height it has at that moment, before it rises; leaving a pad is such a take-off)
    start_tracking(w, id);
    clear_mark(w, id);
    let dest = w.obj(id).and_then(|o| o.dest);
    if let Some(a) = st_mut(w, id) {
        // spec: S-0501 claim 8 (taking off: the held height becomes the cruise height) and claim 48 (b) (a take-off gives the held height of ordinary flight)
        a.held = cruise;
        a.taking_off = true;
        a.landing = false;
        a.ground = false;
        a.released = false;
        a.stranded = false;
        // spec: S-0501 claim 8 (an aircraft on the ground and not landing that gets a destination takes off, whatever the cell: a sliding one too, claim 7)
        a.sliding = false;
    }
    let Some(o) = w.obj(id).cloned() else { return };
    // spec: S-0501 claim 8 and claim 24 (standing on a pad, it leaves its reservation for any destination other than that pad); S-0417 claim 11 (the tether ends when it leaves the ground)
    // spec: S-0501 claim 48 (c) (Pad contact: claim 8's rule reads the cell that holds the aircraft's horizontal position, which must belong to the pad it is in contact with, so a low release from a docking position outside the foundation keeps its contact as it takes off, Q-1940)
    if let Some(k) = o.contact
        && dest != Some(Target::Object(k))
        && (w.obj(k).is_none_or(|b| b.kind != Kind::Building) || pad_cell(w, k, id))
        && let Some(ob) = w.obj_mut(id)
    {
        ob.contact = None;
        ob.econ.exit = None;
    }
    if let Some(ob) = w.obj_mut(id)
        && let Some(e) = ob.econ.exit.as_mut()
    {
        e.tethered = false;
    }
}

/// Starts to land (claims 6 and 7).
fn start_landing(w: &mut World, id: ObjectId) {
    if let Some(a) = st_mut(w, id) {
        // spec: S-0501 claim 6 (a landing sets the held height to the landing height 0) and claim 7 (the target fraction to 0)
        a.landing = true;
        a.taking_off = false;
        a.held = 0;
        a.tgt = F64::ZERO;
        a.released = false;
        a.sliding = false;
    }
}

/// F7: the landing cell test at touchdown.
pub fn landing_accepted(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let c = o.pos.cell();
    landing_cell_ok(w, id, c, Target::Cell(c))
}

/// The object F7 finds in cell `c`'s ground layer for lander `id`: the
/// first infantry, vehicle, aircraft or building other than the lander.
fn f7_found(w: &World, id: ObjectId, c: CellXY) -> Option<ObjectId> {
    // spec: S-0501 F7 (the ground cell layer, not the deck list; the first occupant found; Open question 7 for several)
    w.cells.list(c, Layer::Ground).iter().copied().find(|&x| {
        x != id
            && w.obj(x).is_some_and(|ob| {
                matches!(
                    ob.kind,
                    Kind::Infantry | Kind::Vehicle | Kind::Aircraft | Kind::Building
                )
            })
    })
}

/// Whether an object is spawned or has a spawn controller (F7).
fn spawn_related(w: &World, x: ObjectId) -> bool {
    w.obj(x).is_some_and(|ob| {
        ob.spawn.is_some()
            || w.rules()
                .unit(ob.kind, ob.type_index)
                .is_some_and(|u| u.raw.spawned.unwrap_or(false))
    })
}

/// F7 for lander `id` on cell `c`; `wanted` is the destination object the
/// competing-aircraft test compares against (the cell object itself at
/// touchdown; claim 40's initial test uses the original destination).
fn landing_cell_ok(w: &World, id: ObjectId, c: CellXY, wanted: Target) -> bool {
    let Some(o) = w.obj(id) else { return false };
    // spec: S-0501 F7 (the touchdown position inside the usable area)
    if !w.cells.usable(c) {
        return false;
    }
    let t = ty(w, id).unwrap_or_default();
    // spec: S-0501 F7 (the found object may be the lander itself: claim 48 (c)'s low release is listed in its cell's ground layer at height 0, and claim 48 (e) gives it its flag there, when its landing starts; finding itself accepts the cell)
    let found = w.cells.list(c, Layer::Ground).iter().copied().find(|&x| {
        w.obj(x).is_some_and(|ob| {
            matches!(
                ob.kind,
                Kind::Infantry | Kind::Vehicle | Kind::Aircraft | Kind::Building
            )
        })
    });
    if found == Some(id) {
        return true;
    }
    match f7_found(w, id, c) {
        Some(x) => {
            // spec: S-0501 F7 (the lander's own contact is accepted)
            if o.contact == Some(x) {
                return true;
            }
            // spec: S-0501 F7 (a spawned lander and a found object with a spawn controller or of a spawned type)
            t.spawned && spawn_related(w, x)
        }
        None => {
            let r = w.rules();
            // spec: S-0501 F7 (passable for tracked ground movement, S-0211 claim 3, with no occupants)
            let passable = crate::passability::enterable_by_class(
                r,
                w,
                c,
                Some(LocomotionClass::Track),
                glasnost_rules::sim::MovementZone::Normal,
                false,
            );
            let empty = w.cells.occ(c, Layer::Ground).is_none_or(|rec| rec.empty());
            // spec: S-0501 F7 (no other aircraft in play has this cell object as its destination) and claim 40 (the initial test compares against the original destination object)
            let competing = w.objects.values().any(|ob| {
                ob.id != id && ob.kind == Kind::Aircraft && ob.on_map && ob.dest == Some(wanted)
            });
            passable && empty && !competing
        }
    }
}

/// The flight-reservation test of claim 39 for aircraft `id` on cell `c`;
/// `with_self`: the searching aircraft itself counts as a reservation
/// (firing spots).
fn reservation_ok(w: &World, id: ObjectId, c: CellXY, with_self: bool) -> bool {
    let Some(t) = ty(w, id) else { return false };
    // spec: S-0501 claim 39 (a pad-bound aircraft passes)
    if t.bound {
        return true;
    }
    // spec: S-0501 claim 39 (a spawned aircraft passes when F7's found ground occupant is spawned or has a spawn controller)
    if t.spawned && f7_found(w, id, c).is_some_and(|x| spawn_related(w, x)) {
        return true;
    }
    // spec: S-0501 claim 39 (a placed, non-limbo infantry unit, vehicle or aircraft in the cell, or whose destination is that cell object, refuses; in firing-spot selection the searching aircraft counts too)
    !w.objects.values().any(|ob| {
        (with_self || ob.id != id)
            && ob.on_map
            && matches!(ob.kind, Kind::Infantry | Kind::Vehicle | Kind::Aircraft)
            && (ob.pos.cell() == c || ob.dest == Some(Target::Cell(c)))
    })
}

/// The firing spot of a non-strafer (claim 39) against target `tg`, with
/// its selection draw; `None` when no circle has an eligible candidate.
fn firing_spot(w: &mut World, id: ObjectId, tg: Target) -> Option<CellXY> {
    let o = w.obj(id)?.clone();
    let mode = w.rules().mode;
    // spec: S-0501 claim 39 (T: the target's current position)
    let tp = crate::process::target_pos(w, tg);
    // spec: S-0501 claim 39 (rho: the firer's primary-weapon range of S-0307 claim 13)
    let rho = crate::combat::range::firer_weapon_range(w, id, 0);
    // spec: S-0501 claim 39 (the reference: the target's destination position for a mobile unit with a destination, else the aircraft's position)
    let reference = match tg {
        Target::Object(x) => w
            .obj(x)
            .filter(|t| matches!(t.kind, Kind::Infantry | Kind::Vehicle | Kind::Aircraft))
            .and_then(|t| t.dest)
            .map(|d| crate::process::target_pos(w, d)),
        Target::Cell(_) => None,
    }
    .unwrap_or(o.pos);
    // spec: S-0501 claim 39 (radii rho − 256, rho − 512, …, only those strictly above 256; the largest circle with any eligible candidate)
    let mut radius = rho - 256; // spec: S-0501 Constants (firing-spot radius interval 256)
    // spec: S-0501 claim 39 (only radii strictly above 256)
    while radius > 256 {
        let mut records: Vec<CellXY> = Vec::new();
        let mut best: Option<i32> = None;
        // spec: S-0501 claim 39 (facings 0, 4096, …, 61440 in order)
        for k in 0..16 {
            let f = (k * 4096) as Facing; // spec: S-0501 Constants (firing-spot angular interval 4096)
            // spec: S-0501 claim 39 (F3's trigonometry and truncation applied to T and the radius)
            let bias = w.rules().quirks.table_trig_negative_bias; // spec: S-0105 F1 (quirks.table_trig_negative_bias)
            let (x, y) = crate::walk::step_point_b(tp.x, tp.y, f, radius, true, mode, bias);
            // spec: S-0501 claim 39 (the cell: integer division toward zero by 256)
            let c = CellXY::new(x / 256, y / 256);
            // spec: S-0501 claim 39 (inside the height-aware usable area and passing the reservation test, the aircraft itself counting; no passability or F7 test)
            if !w.cells.usable(c) || !reservation_ok(w, id, c, true) {
                continue;
            }
            // spec: S-0501 claim 39 (the score: the planar distance of the unsnapped candidate to the reference, S-0204 claim 5)
            let score = planar(w, Pos::new(x, y, 0), reference);
            // spec: S-0501 claim 39 (a record: the first eligible candidate or one scoring strictly below all before it)
            if best.is_none_or(|b| score < b) {
                best = Some(score);
                records.push(c);
            }
        }
        if let Some(&last) = records.last() {
            let half = w.rules().quirks.ranged_draw_half_span;
            // spec: S-0501 claim 37 (one U(0, 99) draw) and claim 39 (below 50 the last record's cell, else the preceding record's; with one record both use it)
            let u = w.rng.ranged(0, 99, half);
            let prev = records.len().checked_sub(2).map_or(last, |i| records[i]); // spec: S-0501 claim 39 (the preceding record; with one record the last)
            return Some(if u < 50 { last } else { prev }); // spec: S-0501 claim 39 (below 50 the last record's cell)
        }
        radius -= 256; // spec: S-0501 Constants (firing-spot radius interval 256)
    }
    None
}

/// Claim 40's destination selection for *move*: the destination retained,
/// or a replacement cell, with its draws.
fn move_destination(w: &mut World, id: ObjectId, d: Target) -> Target {
    // spec: S-0501 claim 40 (the cell at the destination's current position)
    let centre = crate::process::target_pos(w, d).cell();
    // spec: S-0501 claim 40 (retained when F7 accepts that cell, compared against the original destination, and the reservation test without self accepts the cell of its heading point)
    if landing_cell_ok(w, id, centre, d)
        && reservation_ok(w, id, heading_point_for(w, id, d).cell(), false)
    {
        return d;
    }
    // spec: S-0501 claim 40 (failure of all 32 radii retains the original destination)
    replacement_search(w, id, centre).map_or(d, Target::Cell)
}

/// Claim 40's replacement-cell search around `centre`: the first cell
/// accepted, with the search's draws, or `None` when all 32 radii fail.
/// S-0507 claim 13 uses it around map waypoint 700.
pub fn replacement_search(w: &mut World, id: ObjectId, centre: CellXY) -> Option<CellXY> {
    let mode = w.rules().mode;
    let half = w.rules().quirks.ranged_draw_half_span;
    // spec: S-0501 claim 40 (radii 0 … 31 cells in increasing order)
    for r in 0..32 {
        // spec: S-0501 claim 40 (one U(0, 7) draw per radius: the first step direction)
        let first = w.rng.ranged(0, 7, half);
        // spec: S-0501 claim 40 (consecutive duplicates are not tested again; the remembered pair starts at (0, 0))
        let mut last = (0, 0);
        for i in 0..8 {
            // spec: S-0501 claim 40 (eight clockwise directions wrapping modulo eight; facing 8192 times the index)
            let dir = (first + i).rem_euclid(8);
            let f = (dir * 8192) as Facing;
            // spec: S-0501 claim 40 (F3's trigonometry in cell units with length equal to the radius, the sums truncated toward zero)
            let bias = w.rules().quirks.table_trig_negative_bias; // spec: S-0105 F1 (quirks.table_trig_negative_bias)
            let (x, y) = crate::walk::step_point_b(centre.x, centre.y, f, r, true, mode, bias);
            if (x, y) == last {
                continue;
            }
            last = (x, y);
            let c = CellXY::new(x, y);
            // spec: S-0501 claim 40 (a height-aware usable cell passing F7 and the reservation test without self; the first accepted)
            if w.cells.usable(c)
                && landing_cell_ok(w, id, c, Target::Cell(c))
                && reservation_ok(w, id, c, false)
            {
                return Some(c);
            }
        }
    }
    None
}

/// F7, the landing test of claim 40, for destination `d` of aircraft `id`:
/// the cell at the destination's current position, with the destination
/// object used for the competing-aircraft comparison (S-0507 claim 13).
pub fn landing_test(w: &World, id: ObjectId, d: Target) -> bool {
    let c = crate::process::target_pos(w, d).cell();
    landing_cell_ok(w, id, c, d)
}

/// The type's cruise height in leptons above the terrain (claims 5 and 6).
pub fn cruise_height(w: &World, id: ObjectId) -> i32 {
    ty(w, id).map_or(0, |t| t.cruise)
}

/// Sets aside an aircraft's object-present mark, as S-0507 claim 13's
/// release does for the occupancy choice; returns whether it had one.
pub fn set_aside_mark(w: &mut World, id: ObjectId) -> bool {
    let had = st(w, id).mark.is_some();
    clear_mark(w, id);
    had
}

/// Restores a mark set aside by [`set_aside_mark`].
pub fn restore_mark(w: &mut World, id: ObjectId) {
    set_mark(w, id);
}

/// The landing direction (claim 25): the contact's body facing rounded to
/// eight directions; with passengers its own; else the pose direction.
pub fn landing_facing(w: &World, id: ObjectId) -> Facing {
    let t = w.tick;
    let round8 = |f: Facing| -> Facing {
        // spec: S-0501 claim 25 (rounded to 8 directions of 8192)
        ((((i32::from(f) + 4096) / 8192) % 8) * 8192) as Facing
    };
    let Some(o) = w.obj(id) else { return 0 };
    if let Some(k) = o.contact
        && let Some(c) = w.obj(k)
    {
        return round8(c.body.value(t));
    }
    if passengers(w, id) > 0 {
        return round8(o.body.value(t));
    }
    let pose = w.rules().econ.production.pose_dir;
    // spec: S-0501 claim 25 (the pose-direction integer times 8192, modulo 65536)
    (pose.wrapping_mul(8192)).rem_euclid(65536) as Facing
}

/// The aircraft touches down (claim 7) and rests: on an accepted cell of a
/// tested touchdown, which forgets its destination in any case, or where an
/// untested one forgot it or left it no flight point (`forget_dest` false:
/// an untested touchdown has none of an accepted one's effects). Its next
/// docking-stage processing docks it at its pad contact (claim 24).
fn touchdown(w: &mut World, id: ObjectId, forget_dest: bool) {
    let Some(o) = w.obj(id).cloned() else { return };
    let surface = w.surface_at(o.pos.x, o.pos.y);
    // spec: S-0501 claim 43 (landing finishes on a high-bridge cell while the world height is at least the terrain surface height plus 416 leptons: the deck mark is set before settling at landing height; an already marked aircraft keeps it)
    let marked =
        deck_marked(w, id) || (w.high_bridge(o.pos.cell()) && o.pos.z - surface >= DECK_ALLOWANCE);
    if let Some(ob) = w.obj_mut(id) {
        ob.pos.z = surface + if marked { DECK_ALLOWANCE } else { 0 };
    }
    // spec: S-0501 claim 47 (b) (an accepted touchdown ends tracking, on open ground and on a pad alike; the start of landing and a refused touchdown do not)
    end_tracking(w, id);
    if let Some(a) = st_mut(w, id) {
        a.deck = marked;
        a.ground = true;
        a.landing = false;
        a.taking_off = false;
        a.held = 0;
        a.cur = F64::ZERO;
        a.tgt = F64::ZERO;
        a.flying_to = None;
        a.raised = false;
        a.own_point = false;
        a.ignored = None;
        a.stranded = false;
        // spec: S-0501 claim 40 (landing settled: the completion clears the unfinished flight and the destination)
        a.unfinished = false;
        a.released = false;
        a.sliding = false;
    }
    // spec: S-0209 claim 1 and S-0501 claim 47 (b) (r134: at rest it holds the object-present flag of the cell it touched down in; one that slid there carries its flag to that cell)
    clear_mark(w, id);
    set_mark(w, id);
    // spec: S-0212 claim 13 (each landing records the aircraft in its landing cell)
    crate::threat::landed(w, id);
    if forget_dest {
        // spec: S-0501 claim 7 (r103, Q-2366: an accepted touchdown forgets the aircraft's destination in that same movement update; neither an accepted nor a refused touchdown makes a fall-back to rest or an entry report itself: for an aircraft in enter its next processing makes the docking stage's step of claim 24)
        set_dest(w, id, None);
    }
}

/// An untested touchdown whose own forget test kept the flight point
/// (claim 7, r132): the aircraft is on the ground by its height, not
/// landing and not at rest, with its destination (if any) and its flight
/// point; from its next movement update it goes on as after a refusal that
/// kept the destination.
fn untested_kept(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else { return };
    let surface = w.surface_at(o.pos.x, o.pos.y);
    // spec: S-0501 claim 43 (landing finishes on a high-bridge cell while the world height is at least the terrain surface height plus 416 leptons: the deck mark is set; an already marked aircraft keeps it)
    let marked =
        deck_marked(w, id) || (w.high_bridge(o.pos.cell()) && o.pos.z - surface >= DECK_ALLOWANCE);
    if let Some(ob) = w.obj_mut(id) {
        ob.pos.z = surface + if marked { DECK_ALLOWANCE } else { 0 };
    }
    // spec: S-0501 claim 47 (b) (r134, Q-2647: every touchdown ends the aircraft's tracking and returns its listing in its cell layer)
    end_tracking(w, id);
    let drop = w.rules().quirks.aircraft_refused_kept_drop;
    if let Some(a) = st_mut(w, id) {
        a.deck = marked;
        a.landing = false;
        a.taking_off = false;
        a.ground = false;
        a.stranded = false;
        a.released = false;
        a.ignored = None;
        // spec: S-0501 claim 7 (r132: on the ground by its height with its destination and its flight point; from its next movement update it goes on as after a refusal that kept the destination, Q-2578: held height 0, sliding until the update after one that leaves its target fraction at 1) and Edge cases (quirks.aircraft_refused_kept_drop on; off: that untested touchdown gives it the held height of ordinary flight, which its next movement update's start sets, so it climbs from that update on)
        a.sliding = drop;
        a.held = 0;
        // spec: S-0501 claim 7 (r134: every touchdown, tested or not, sets the aircraft's current and target speed fractions to 0)
        a.cur = F64::ZERO;
        a.tgt = F64::ZERO;
    }
    // spec: S-0501 claim 47 (b) (r134, Q-2647: the listing it returns sets the object-present flag of the cell it is in, S-0209 claim 13 and claim 44's rule for the record; while it slides each update moves that flag to the cell it ends the update in, [`kept_mark`])
    clear_mark(w, id);
    set_mark(w, id);
    // spec: S-0212 claim 13 (each landing records the aircraft in its landing cell)
    crate::threat::landed(w, id);
}

/// Claim 7's forget at touchdown (r120): whether the touchdown forgets the
/// aircraft's destination and flight point before its cell is tested. It
/// does when the aircraft is in the flight point's cell, and, wherever it
/// is, when that cell holds the building it is in contact with; for an
/// aircraft with no contact, a flight point's cell with no building counts
/// as holding it. With no flight point there is nothing to compare.
fn touchdown_forgets(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let Some(f) = st(w, id).flying_to else {
        return false;
    };
    let fc = flight_point(w, id, f).cell();
    // spec: S-0501 claim 7 (r120: in the cell of that flight point, claim 40)
    if o.pos.cell() == fc {
        return true;
    }
    // The buildings a cell holds: those listed in its ground layer, which
    // every foundation cell of a building lists.
    let mut buildings = w
        .cells
        .list(fc, Layer::Ground)
        .iter()
        .copied()
        .filter(|&x| w.obj(x).is_some_and(|b| b.kind == Kind::Building));
    match o.contact {
        // spec: S-0501 claim 7 (r120: wherever it is, when the flight point's cell holds the building it is in contact with; the owner, the pad-bound status and the kind of destination do not enter this test)
        Some(k) => buildings.any(|x| x == k),
        // spec: S-0501 claim 7 (r120: for an aircraft with no contact, a flight point's cell with no building counts as holding it)
        None => buildings.next().is_none(),
    }
}

/// Claim 7's check of a landing's flight point cell (r132): whether it lets
/// the landing go on. It refuses a cell only to a pad-bound aircraft, when
/// that cell holds no building the aircraft is in contact with.
fn landing_cell_check(w: &World, id: ObjectId, t: &Ty, a: &AirState) -> bool {
    // spec: S-0501 claim 7 (r132: for an aircraft that is not pad-bound the check takes no account of the cell's occupants, its terrain or the usable area: it refuses a cell only in a campaign mission, which this spec does not cover, so it refuses none here)
    if !t.bound {
        return true;
    }
    let Some(f) = a.flying_to else { return true };
    let fc = flight_point(w, id, f).cell();
    let contact = w.obj(id).and_then(|o| o.contact);
    // spec: S-0501 claim 7 (r132: for a pad-bound aircraft it refuses a cell that holds no building the aircraft is in contact with; the buildings a cell holds are those listed in its ground layer, which every foundation cell of a building lists)
    contact.is_some_and(|k| {
        w.cells
            .list(fc, Layer::Ground)
            .iter()
            .any(|&x| x == k && w.obj(x).is_some_and(|b| b.kind == Kind::Building))
    })
}

/// Whether the aircraft stands in a cell of pad `k`'s foundation.
fn pad_cell(w: &World, k: ObjectId, id: ObjectId) -> bool {
    let (Some(b), Some(o)) = (w.obj(k), w.obj(id)) else {
        return false;
    };
    crate::econ::placement::foundation_cells(w, b.type_index, b.pos.cell()).contains(&o.pos.cell())
}

/// Docking at pad `k` after touchdown (claim 24): the aircraft rests where
/// it touched down, at the landing height, in contact with its contact
/// place, and reports that it has entered. Only a factory release puts an
/// aircraft at a docking position ([`released_docked`]); the docking stage
/// of *enter* has moved it toward that position horizontally while it
/// landed ([`docking_step`]).
fn dock(w: &mut World, id: ObjectId, k: ObjectId) {
    // spec: S-0501 claim 24 (docking by landing never applies the docking position: it rests where it touched down, short of that position if the horizontal difference was not used up) and S-0404 claim 15 (its contact place)
    let (_, place) = crate::econ::release::pad_position(w, k, id);
    if let Some(o) = w.obj_mut(id) {
        o.contact = Some(k);
        o.econ.exit = Some(crate::econ::release::Exit {
            factory: k,
            tethered: false,
            place,
        });
    }
    // spec: S-0501 claim 24 (r134, Q-2649: neither the report after a touchdown nor the pad's answer to it requests a facing) and claim 25 (the landing direction's one named use is claim 42's aiming request) (#162)
    report_entered(w, id, k);
}

/// A contact place index outside every pad's displacement count: the
/// docking position of an aircraft without a contact (S-0404 claim 15).
const NO_PLACE: u32 = u32::MAX; // spec: S-0501 claim 24 (r99: the docking position without a contact; r103: the pad's heading point without a contact) and S-0404 claim 15 (a contact index outside the displacement count gives c; a count of 1 gives the sole displacement whatever the index)

/// One step of *enter*'s docking approach while the aircraft lands (claim
/// 24): toward its contact place's docking position at pad `to`, along each
/// map axis by the difference limited to −5 … +5 leptons, its world height
/// unchanged.
fn docking_step(w: &mut World, id: ObjectId, to: Target) {
    let Some(o) = w.obj(id) else { return };
    let here = o.pos;
    let to = match to {
        Target::Object(k) if w.obj(k).is_some_and(|b| b.kind == Kind::Building) => {
            if o.contact == Some(k) {
                crate::econ::release::pad_position(w, k, id).0
            } else {
                // spec: S-0501 claim 24 (r99 and r103: in the docking stage without a contact, an aircraft aims at a docking position that does not depend on whether the pad has a free contact place: with one docking place that place's docking position, otherwise the pad's foundation centre with no displacement) and S-0404 claim 15 (r23: an aircraft that lands over the pad without a contact is moved toward that same docking position; a count of 1 gives the sole displacement whatever the index, a contact index outside the count gives c)
                crate::econ::release::pad_position_at(w, k, NO_PLACE)
            }
        }
        // spec: S-0501 claim 24 (a contact that is not a building gives its centre instead of a docking position)
        Target::Object(k) => w.reference_point(k),
        // spec: S-0501 claim 24 (r110, Q-2437: without a contact, toward a destination that is not a building, such as the landing spot the run-away notice gives, it moves it toward that destination's centre)
        Target::Cell(c) => Pos::centre(c, here.z),
    };
    // spec: S-0501 claim 24 and Constants (the docking approach step: each map axis by the difference to the docking position limited to −5 … 5 leptons, each axis separately, exact integer arithmetic; the world height unchanged)
    let step = |d: i32| d.clamp(-5, 5);
    let dx = step(to.x.wrapping_sub(here.x));
    let dy = step(to.y.wrapping_sub(here.y));
    if (dx, dy) != (0, 0) {
        move_to(w, id, Pos::new(here.x + dx, here.y + dy, here.z));
    }
}

/// The aircraft reports that it has entered pad `k` (claim 24): the pad
/// starts servicing and has it queue *sleep*, and it then queues *guard*,
/// which replaces the *sleep*.
fn report_entered(w: &mut World, id: ObjectId, k: ObjectId) {
    let t = w.tick;
    // spec: S-0501 claim 24 (r110, Q-2438: the pad has the aircraft queue sleep, and the aircraft then queues guard, which replaces the sleep, S-0215 claim 7; the processing leaves it ready, so guard becomes current by the end of the tick; neither the report nor the pad tests where it touched down)
    queue(w, id, Behaviour::Sleep);
    queue(w, id, Behaviour::Guard);
    if let Some(a) = st_mut(w, k)
        && a.service.is_none()
    {
        // spec: S-0501 claim 26 (an accepted entered report queues servicing without resetting the pad's timer; it becomes current at the pad's next per-tick start with its timer expired: in the report tick when the pad's turn has not passed, else the next tick)
        a.service = Some(t);
    }
}

/// Starts a crash (claim 9, S-0312 claim 16) of an aircraft above the
/// terrain: the tumble draws, then the fall counter. `living`: a crash
/// started without a kill (its locomotor switched off). Returns whether a
/// crash started.
pub fn start_crash(w: &mut World, id: ObjectId, living: bool) -> bool {
    start_crash_hit(w, id, living, None)
}

/// As [`start_crash`], with the killing hit that credits the passengers'
/// deaths (S-0312 claim 16); without one, they are credited to no one.
pub fn start_crash_hit(
    w: &mut World,
    id: ObjectId,
    living: bool,
    hit: Option<&crate::combat::damage::Hit>,
) -> bool {
    if !is_aircraft(w, id) || height(w, id) <= 0 {
        return false;
    }
    if st(w, id).crash.is_some() {
        return true;
    }
    let none = crate::combat::damage::Hit {
        amount: 0,
        warhead: None,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: false,
        no_escape: false,
    };
    // spec: S-0312 claim 16 (an object that starts to crash loses all its passengers, credited to the attacker, before its tumbling draws)
    crate::transport::kill_passengers(w, id, hit.unwrap_or(&none));
    let mode = w.rules().mode;
    let half = w.rules().quirks.ranged_draw_half_span;
    // spec: S-0312 claim 16 (tumbling: one unit-interval value, one ranged draw over 0 to 1, one more unit-interval value)
    let u1 = w.rng.unit(mode);
    let sense = w.rng.ranged(0, 1, half);
    let u3 = w.rng.unit(mode);
    // spec: S-0113 claim 16 (the three draws set the crashing body's rates; the view keeps them, presentation only)
    w.effects
        .push(crate::combat::effects::Effect::CrashTumble { id, u1, sense, u3 });
    if let Some(a) = st_mut(w, id) {
        a.crash = Some(0);
        // spec: S-0501 claim 9 (a living crash: its locomotor is switched off; which states start one is Open question 9) and claim 36 (a release crash is a dead crash, not a living disabled fall)
        a.switched_off = living;
        a.landing = false;
        a.taking_off = false;
        a.run_lock = false;
        // A crashing aircraft descends toward the landing height (claim 3).
        a.held = 0;
    }
    true
}

/// Whether the aircraft is crashing.
pub fn crashing(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .and_then(|o| o.air.as_ref())
        .is_some_and(|a| a.crash.is_some())
}

/// Impact of a crashing aircraft (claim 9, S-0312 claim 16).
fn impact(w: &mut World, id: ObjectId) {
    // spec: S-0501 claim 47 (b) (a crash impact ends tracking, before the impact's own effects)
    end_tracking(w, id);
    let Some(o) = w.obj(id).cloned() else { return };
    let surface = w.surface_at(o.pos.x, o.pos.y);
    if let Some(ob) = w.obj_mut(id) {
        ob.pos.z = surface;
    }
    if let Some(a) = st_mut(w, id) {
        a.ground = true;
        a.released = false;
        a.sliding = false;
    }
    let at = Pos::new(o.pos.x, o.pos.y, surface);
    if crate::combat::damage::dead(&o) {
        if crate::bomb::carries(w, id) {
            if w.rules().quirks.crash_drops_planted_bomb {
                // spec: S-0501 claim 46 and Edge cases (quirks.crash_drops_planted_bomb on: a bomb that has not gone off by the impact is removed then, with no detonation, animation or damage, as S-0519 claim 7's removal other than by death)
                if let Some(ob) = w.obj_mut(id) {
                    ob.combat.bomb = None;
                }
            } else {
                // spec: S-0501 Edge cases (quirks.crash_drops_planted_bomb off: at impact, before the death weapon, the bomb goes off by S-0519 claim 5 at the aircraft's position)
                crate::bomb::detonate(w, id);
            }
        }
        // spec: S-0312 claim 16 (shot down: its death weapon goes off at the impact point, and it leaves play)
        crate::combat::death::impact_death_weapon(w, id, at);
        crate::combat::death::leave_play(w, id);
        return;
    }
    let warhead = w.rules().combat.general.c4_warhead;
    let amount = o.health;
    // spec: S-0312 claim 16 (alive: destroyed by damage equal to its health, the demolition warhead, ignoring defences, no-escape, no attacker)
    crate::combat::damage::apply(
        w,
        id,
        crate::combat::damage::Hit {
            amount,
            warhead,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: true,
        },
    );
    if let Some(h) = warhead {
        // spec: S-0312 claim 16 (the explosion for that amount, S-0303 F5, and a detonation with no source object or house at the impact point)
        crate::combat::projectile::detonate_area(w, h, amount, at, None, None, true);
    }
}

/// Moves an aircraft with the flying locomotor in its turn (S-0501 claims
/// 1 to 10); whether it moved.
pub fn tick(w: &mut World, id: ObjectId) -> bool {
    let landing = w
        .obj(id)
        .filter(|o| is_aircraft(w, id) && o.on_map && !crate::combat::damage::dead(o))
        .and_then(|o| o.air.as_ref())
        .is_some_and(|a| a.landing);
    // Claim 48 (e) governs an aircraft that a factory release left above its pad, through the update that ends that state.
    let released = st_opt(w, id).is_some_and(|a| a.released);
    let start_cell = w.obj(id).map(|o| o.pos.cell());
    let moved = tick_flight(w, id, landing);
    // spec: S-0209 claim 13 (taking off removes the listing when the listing height becomes positive, and landing adds it again)
    sync_listing(w, id);
    if released {
        released_mark(w, id);
    }
    kept_mark(w, id, start_cell);
    let landing = landing || st_opt(w, id).is_some_and(|a| a.landing);
    // spec: S-0102 claim 4 (every movement update of a landing re-files, from the update that starts it to the one that ends it, while the aircraft has strength left; a type that cannot land is never re-filed by landing)
    if landing && landable(w, id) && w.obj(id).is_some_and(|o| o.on_map) {
        crate::combat::effects::layer_refile(w, id);
    }
    moved
}

/// The object-present flag of a released aircraft at the end of a movement
/// update (S-0501 claim 48 (e)): above height 0 it holds none; at height 0
/// it holds one in the cell that then holds it (the ground record in the
/// low release; claim 44 chooses the record), and in no other cell, so a
/// cell it has left no longer holds its flag.
fn released_mark(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if !o.on_map || crate::combat::damage::dead(o) {
        return;
    }
    let cell = o.pos.cell();
    // spec: S-0501 claim 48 (e) (while it is above height 0, resting or leaving, it holds no flag; it next holds one at the end of a movement update that leaves it at height 0, in the cell that then holds it: at the end of its touchdown, or in the low release; a climb in its second update leaves it with no flag, as after claim 44's take-off)
    let want = (height(w, id) <= 0).then_some(cell);
    if st(w, id).mark != want {
        clear_mark(w, id);
    }
    if want.is_some() {
        set_mark(w, id);
    }
}

/// The cell listing and object-present flag at the end of a movement update
/// of an aircraft that a touchdown left on the ground by its height and not
/// at rest, holding its flight point (S-0501 claim 47 (b), r134, Q-2647):
/// the flag that touchdown set ([`untested_kept`], [`refusal_touchdown_kept`])
/// marks it. While it is at height 0, each update takes it out of the cell
/// layer of the cell it was in (`from`), clearing that cell's flag, and
/// lists it in the cell layer of the cell it ends the update in, setting
/// that cell's flag; the first update whose climb leaves its height
/// positive ends the listing ([`sync_listing`]) and clears the flag.
fn kept_mark(w: &mut World, id: ObjectId, from: Option<CellXY>) {
    let Some(o) = w.obj(id) else { return };
    if !o.on_map || crate::combat::damage::dead(o) {
        return;
    }
    let a = st(w, id);
    if a.mark.is_none() || a.ground || a.released || a.landing {
        return;
    }
    let cell = o.pos.cell();
    if height(w, id) > 0 {
        // spec: S-0501 claim 47 (b) (r134: that listing ends in the first movement update whose climb leaves its height positive) and S-0209 claim 1 (above the ground it holds no object-present flag)
        clear_mark(w, id);
        return;
    }
    if from == Some(cell)
        && let Some(l) = listing_layer(w, id)
    {
        // spec: S-0501 claim 47 (b) (r134, Q-2647: each of its movement updates takes it out of the cell layer of the cell it was in and lists it in the cell layer of the cell it ends the update in, the same cell included: listed again, it goes to the front of that list, S-0209 claim 13); a change of cell has done so in its forward step
        w.cells.leave(cell, l, id);
        w.cells.enter(cell, l, id, false);
    }
    // spec: S-0501 claim 47 (b) (r134, Q-2647: clearing the flag of the cell it was in and setting that of the cell it ends the update in, by claim 44's rule for the record)
    clear_mark(w, id);
    set_mark(w, id);
}

/// Whether the aircraft's type is landable (S-0501 Terms).
pub fn landable(w: &World, id: ObjectId) -> bool {
    ty(w, id).is_some_and(|t| t.landable)
}

fn st_opt(w: &World, id: ObjectId) -> Option<&AirState> {
    w.obj(id).and_then(|o| o.air.as_ref())
}

/// The locomotor accepts the flight point of `dest` (claims 6, 8 and 40):
/// at a movement update that finds a new destination, and at *move*'s
/// first and next processing, which give it once more ([`move_`]). An
/// aircraft that is then on the ground by its height, or landing, takes
/// off.
fn give_point(w: &mut World, id: ObjectId, t: &Ty, dest: Option<Target>) {
    if dest.is_some_and(|d| ignores(w, id, d)) {
        // spec: S-0501 claim 8 (r110, Q-2437: a landing aircraft given a destination in the cell of its flight point ignores it: no take-off, the flight point stays as it was and it goes on landing there) and r120 (Q-2507: the test is made only when the destination is handed to the flight; an ignored destination is still the aircraft's destination)
        if let Some(a) = st_mut(w, id) {
            a.ignored = dest;
        }
        return;
    }
    let raised = raise_now(w, id);
    let handed = dest.and_then(|d| handed_point(w, id, d));
    if let Some(a) = st_mut(w, id) {
        a.flying_to = dest;
        a.handed = handed;
        a.lost = false;
        a.own_point = false;
        a.ignored = None;
        // spec: S-0501 claim 7 (r120: the hold after a refused touchdown that forgot the destination lasts until it gets a destination; r146, Q-2676: so does the survivor's origin point, the destination handed to its flight as any destination is, claim 8)
        a.stranded = false;
        a.origin = false;
        // spec: S-0501 claim 6 (the height replacement is fixed with the point when it is accepted: a target and ammunition now)
        a.raised = raised;
        // spec: S-0501 claim 40 (a movement point accepted by the locomotor is an unfinished flight)
        a.unfinished = true;
    }
    let a = st(w, id);
    // spec: S-0501 claim 8 and Terms (it takes off only when landing or on the ground by its height, whichever render-layer list it is filed in: claim 48 (c)'s released aircraft above the terrain starts no take-off)
    if a.ground || a.landing || on_ground(w, id) {
        take_off(w, id, t.cruise);
        if t.landable {
            // spec: S-0102 claim 4 (the first movement update after a take-off starts re-files, after that update's climb: the drawing evaluates after the tick)
            crate::combat::effects::layer_refile(w, id);
        }
    }
}

/// Claim 53 (r183, Q-3041): object `gone` leaves play (at its killing hit,
/// going off the map or its removal; [`crate::combat::held::holders_drop`]
/// calls this after any revert). Every aircraft holding it as destination
/// loses it now, without the destination assignment, so claim 6's
/// land-where-it-is does not follow: its flight is not told. A destination
/// not yet handed to the flight (a revert, or an assignment earlier in the
/// tick) is handed over first, unless claim 8's exception ignores it, so
/// its flight point is the object's heading point now. The flight keeps
/// its point as one held with no destination object and its unfinished
/// flight; [`AirState::lost`] records the loss for *move*.
pub(crate) fn destination_lost(w: &mut World, gone: ObjectId) {
    let lost = Target::Object(gone);
    // spec: S-0501 claim 53 (every aircraft that holds the object as its destination, whatever its behaviour and however the object left play; in key order, which is creation order)
    let ids: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| o.kind == Kind::Aircraft && o.air.is_some() && o.dest == Some(lost))
        .map(|o| o.id)
        .collect();
    for id in ids {
        let a = st(w, id);
        if a.flying_to != Some(lost)
            && a.ignored != Some(lost)
            && let Some(t) = ty(w, id)
        {
            // spec: S-0501 claim 53 (the revert's destination is one the aircraft gets in claim 8's sense: its flight point is whatever that hand-over leaves, the object's heading point at the moment of the loss unless claim 8's exception ignores it) and claim 8 (an assignment is a hand-over: at the order, at a processing that gives one, or by another rule)
            give_point(w, id, &t, Some(lost));
        }
        let held = st(w, id).flying_to == Some(lost) && st(w, id).handed.is_none();
        // spec: S-0501 claim 53 (no movement update reads the gone object: a building's heading point, read at each update since it does not move, is kept as it is now)
        let now = held.then(|| heading_point_for(w, id, lost));
        if let Some(o) = w.obj_mut(id) {
            // spec: S-0501 claim 53 (from that moment in the tick it does not hold that object as its destination; no destination assignment is made)
            o.dest = None;
        }
        if let Some(a) = st_mut(w, id) {
            if now.is_some() {
                a.handed = now;
            }
            if a.ignored == Some(lost) {
                a.ignored = None;
            }
            // spec: S-0501 claim 53 (its flight point and unfinished flight stay as they were: its movement updates go on toward that point as toward any flight point held with no destination object, claims 4, 5, 6 and 42)
            if a.flying_to.is_some() {
                a.own_point = true;
            }
            a.lost = true;
        }
    }
}

/// The heading point of a mobile destination object, taken when it is
/// handed to the flight (claims 8, 40 and 53); `None` for a cell or a
/// building, whose heading point does not move (S-0506 claim 14: a cell's
/// centre, a building's centre) and is read as before.
fn handed_point(w: &World, id: ObjectId, d: Target) -> Option<Pos> {
    let Target::Object(k) = d else { return None };
    let o = w.obj(k)?;
    if o.kind == Kind::Building {
        return None;
    }
    // spec: S-0501 claim 53 (r183, Q-3041: when the hand-over that set the flight point gave the object as destination, the point is the object's heading point at that hand-over, claim 40; nothing reads it again for the flight between hand-overs)
    Some(heading_point_for(w, id, d))
}

/// Claim 8's exception: whether landing aircraft `id`'s flight ignores
/// destination `d`, given in the cell of the point it is landing at (claim
/// 40's flight point: its docking position when it docks, claim 24; the
/// flight point its cleared destination gave it when it lands where it is,
/// claim 6). The test is made only when a destination is handed to the
/// flight ([`give_point`]); an ignored one is kept in
/// [`AirState::ignored`].
fn ignores(w: &World, id: ObjectId, d: Target) -> bool {
    let Some(a) = st_opt(w, id) else { return false };
    // spec: S-0501 claim 8 (the test is made at each hand-over, at each processing of move that gives a destination, claim 40, so also for the destination the flight already holds) (#159)
    if !a.landing {
        return false;
    }
    // spec: S-0501 claim 8 (r120, Q-2507: the point it is landing at is its flight point, claim 40, the one its cleared destination gave it when it lands where it is, claim 6, not its position; with no flight point there is no such point)
    let Some(f) = a.flying_to else { return false };
    // spec: S-0501 claim 8 (r120: the cell compared is that of the new destination's heading point, claim 40, with that of the flight point)
    heading_point_for(w, id, d).cell() == flight_point(w, id, f).cell()
}

/// One movement update; `started_landing`: whether the aircraft was landing
/// at its start.
fn tick_flight(w: &mut World, id: ObjectId, started_landing: bool) -> bool {
    if !is_aircraft(w, id) {
        return false;
    }
    let Some(t) = ty(w, id) else { return false };
    let Some(o) = w.obj(id) else {
        return false;
    };
    if !o.on_map {
        return false;
    }
    let dead = crate::combat::damage::dead(o);
    let start = o.pos;
    let unplaced = o.air.is_none();
    let rules = w.rules_arc();
    let mode = rules.mode;
    let ar = &rules.air;
    if unplaced {
        placed(w, id);
    }
    // spec: S-0501 claim 47 (c) (the sector update comes at the start of each movement update, before its forward step)
    sector_update(w, id);
    // Claim 9: the crash contribution comes first; impact before any ordinary adjustment.
    let mut a = st(w, id);
    if dead && a.crash.is_none() && !start_crash(w, id, false) {
        return false;
    }
    let mut a2 = st(w, id);
    if let Some(k) = a2.crash {
        let h = height(w, id);
        let (k, h1) = crash_step(k, h, dead);
        a2.crash = Some(k);
        let surface = base(w, id);
        if let Some(ob) = w.obj_mut(id) {
            ob.pos.z = surface + h1;
            ob.air = Some(a2.clone());
        }
        // spec: S-0102 claim 4 (every tick of an aircraft's crash fall re-files)
        crate::combat::effects::layer_refile(w, id);
        // spec: S-0501 claim 9 (a contribution bringing height below 1: impact before ordinary movement)
        if h1 < 1 {
            impact(w, id);
            return true;
        }
        if a2.switched_off {
            return w.obj(id).is_some_and(|ob| ob.pos != start);
        }
    }
    a = st(w, id);
    let alive = !dead;
    // spec: S-0501 claim 31 (the first start opportunity: in the locomotor's step, alive, neither landing nor taking off)
    if alive && !a.landing && !a.taking_off {
        start_if_ready(w, id);
    }
    if alive {
        guard_move(w, id, &t);
    }
    let Some(o) = w.obj(id) else {
        return false;
    };
    // The object's fields as this point of the update left them.
    let (o_dest, here, current, target) = (o.dest, o.pos, o.beh.current, o.target);
    // spec: S-0501 claim 7 (r146, Q-2676: the survivor holding the origin point as flight point is moving, claim 6, so a clear of the destination it kept is being given no destination)
    let holds_point = a.flying_to.is_some() || (a.origin && a.ignored.is_some());
    if alive && o_dest.is_none() && holds_point && !a.own_point && flight_target(w, id).is_none() {
        // spec: S-0501 claim 6 (r120, Q-2507: an aircraft in flight, moving and not landing that is given no destination lands where it is; a clear that did not go through the destination assignment is met at this movement update, with the aircraft where the clear left it)
        let was_landing = a.landing;
        given_no_destination(w, id);
        let s = st(w, id);
        if was_landing || (s.taking_off && !a.taking_off) {
            // spec: S-0501 claim 6 (r132, Q-2576: a landing aircraft given no destination over another cell than its flight point's takes off toward the new point, in this movement update; r134, Q-2647: so does one on the ground by its height that is moving)
            a = s;
        } else {
            (
                a.flying_to,
                a.own_point,
                a.ignored,
                a.raised,
                a.unfinished,
                a.origin,
            ) = (
                s.flying_to,
                s.own_point,
                s.ignored,
                s.raised,
                s.unfinished,
                s.origin,
            );
        }
    }
    // spec: S-0501 claim 42 (a destination cleared in attack with a target keeps the flight point: guidance and claim 5's slowing go on toward it)
    let mut dest = if dead { None } else { flight_target(w, id) };
    if dest.is_some() && dest == a.ignored {
        // spec: S-0501 claim 8 (r110, Q-2437: its flight ignores a destination in the cell of the point it is landing at: the flight point stays as it was and it goes on landing there) and r120 (the test is made only at hand-over, not at the movement updates in between)
        dest = a.flying_to;
    }
    let dp = dest.map(|d| flight_point_of(w, id, d, Some(&t)));
    let d = dp.map(|p| planar(w, here, p));
    let dest_above = dp.map(|p| p.z - w.surface_at(p.x, p.y));
    // spec: S-0501 claim 6 (holding height: destination more than 120 leptons above terrain, a target with ammunition left, the run lock, or a type that is not landable)
    // spec: S-0501 claim 6 (an enter movement update clears the mark)
    let entering = current == Some(Behaviour::Enter);
    a.holding = alive
        && !entering
        && (dest_above.is_some_and(|z| z > HOLD_ABOVE)
            || (target.is_some() && ammo_left(w, id))
            || a.run_lock
            || !t.landable);
    // Claim 8: a new destination makes an aircraft on the ground (or landing) take off.
    let new_dest = dest.is_some() && dest != a.flying_to;
    if alive && new_dest {
        if let Some(ob) = w.obj_mut(id) {
            ob.air = Some(a.clone());
        }
        give_point(w, id, &t, dest);
        a = st(w, id);
    } else if alive
        && dest.is_some()
        // spec: S-0501 claim 53 (r183: the flight point a loss left is held with no destination; nothing hands it to the flight again)
        && !(a.lost && o_dest.is_none())
        && !a.landing
        && !a.taking_off
        && low_release_at_height_0(w, id, &a)
        && w.obj(id)
            .is_some_and(|o| o.beh.current != Some(Behaviour::Move))
    {
        // design: S-0501 claim 48 (c) and Open question 26 (a low release whose destination came with a behaviour other than move, such as the attack of (d), takes off at the start of its next movement update at height 0 with that destination, as a move's first processing takes it off before that update: the low release makes no movement update at height 0 with a destination but without a take-off, whatever gave it the destination; on a move, claim 40's processing alone gives the take-off)
        if let Some(ob) = w.obj_mut(id) {
            ob.air = Some(a.clone());
        }
        give_point(w, id, &t, dest);
        a = st(w, id);
    }
    if alive && dest.is_none() {
        a.flying_to = None;
        a.raised = false;
        a.own_point = false;
    }
    if alive && a.released && dest.is_none() && a.cur == F64::ZERO {
        // spec: S-0501 claim 48 (a) (its movement updates move it only toward a destination: while it has none, neither its height nor its position changes)
        if let Some(ob) = w.obj_mut(id) {
            ob.air = Some(a);
        }
        return false;
    }
    let mut slide_began = false;
    if alive && !a.landing && a.taking_off && a.held == 0 {
        // spec: S-0501 claim 7 (r120: the take-off a refused touchdown starts ends in the next movement update after the one that refused it, whose take-off step clears it, and leaves its held height as it was, the landing's; only that take-off keeps the landing height 0 as held height, claim 8's take-off gives the cruise height)
        a.taking_off = false;
        // spec: S-0501 claim 7 (r132, Q-2578: after a refusal that kept the destination, this update's descent toward the held height 0 brings it back to height 0, which is no touchdown: it is then on the ground by its height, not landing, with its destination, its flight point and the held height 0) and Edge cases (quirks.aircraft_refused_kept_drop on; off: the held height of ordinary flight, so it climbs from this update, the next after the refusal)
        if w.rules().quirks.aircraft_refused_kept_drop && !a.stranded {
            a.sliding = true;
            slide_began = true;
        }
    }
    if alive && a.stranded && dest.is_none() {
        // spec: S-0501 claim 7 (r120: a refused touchdown that forgot the destination and the flight point leaves the aircraft with nothing to move it: its movement updates change neither its height nor its position, and claim 6's landing triggers do not apply; it stays in flight, not landing, 10 leptons above the refused cell until it gets a destination)
        if let Some(ob) = w.obj_mut(id) {
            ob.air = Some(a);
        }
        return false;
    }
    a.stranded = false;
    // The held height in ordinary flight (claim 6).
    let ordinary_held = |a: &AirState| {
        // spec: S-0501 claim 6 (below 768 leptons from the destination every marked approach uses the destination height minus the terrain height there)
        if a.holding
            && d.is_some_and(|d| d < 768) // spec: S-0501 claim 6 (below 768 leptons of the destination)
            && let Some(z) = dest_above
        {
            z
        } else {
            t.cruise
        }
    };
    // spec: S-0501 claim 48 (b)'s third case and claim 6 (r167, Q-2802: a movement update that it begins neither landing nor taking off, with held height 0 and a target speed fraction of 1, the fraction held from the end of the update before, gives the held height of ordinary flight before its height changes in that update)
    let third_case =
        |a: &AirState| !a.taking_off && a.held == 0 && a.tgt.eq_num(F64::from_i64(1, mode));
    // spec: S-0501 claim 6 (r167: a 768-lepton held height of 0, as for a point at the terrain's surface, does not last into such an update: it gives way to that of ordinary flight, the cruise height, so the aircraft does not descend toward that point's height in that update)
    let flight_held = |a: &AirState| match ordinary_held(a) {
        0 => t.cruise,
        z => z,
    };
    if alive && !a.landing && !a.ground {
        if a.sliding {
            // spec: S-0501 claim 7 (r132, Q-2578: at the start of the movement update after the first one that leaves its target speed fraction at 1, it takes the held height of ordinary flight, claim 48 (b)'s third case, and climbs in that update; until then the held height 0)
            if !slide_began && third_case(&a) {
                a.held = flight_held(&a);
                a.sliding = false;
            }
        } else if !a.released {
            // spec: S-0501 claim 6 (r167, Q-2802: the held height of 0 from the 768-lepton rule, which it holds between updates, gives way to the held height of ordinary flight in an update begun with the third case's other conditions; above the terrain within 768 leptons claim 5's replacement keeps the fraction below 1, so it lasts unless claim 5 fixes the target at 1) and claim 51 (r167: the spy plane's held height is 0 again between updates, and each update changes its height toward that of ordinary flight)
            a.held = ordinary_held(&a);
            if third_case(&a) {
                a.held = t.cruise;
            }
        } else if third_case(&a) {
            // spec: S-0501 claim 48 (b) (a movement update that it begins neither landing nor taking off, with held height 0 and a target speed fraction of 1, gives the held height of ordinary flight: it climbs in that same update)
            a.held = flight_held(&a);
            a.released = false;
        }
    }
    let h0 = height(w, id);
    // Take-off's facing requests (claim 8). In ordinary flight the facings
    // are requested by guidance after the movement tick (claim 42).
    let cruise = t.cruise;
    let tick_now = w.tick;
    let mut full_request = false;
    if a.taking_off {
        full_request = take_off_requests(w, id, cruise, h0, here, dp);
        let r = st(w, id);
        (a.request_tick, a.body_was, a.aim_was) = (r.request_tick, r.body_was, r.aim_was);
    }
    // Horizontal motion (claim 4): forward along the body facing by ⌊V × f⌋.
    let mut removed = false;
    if !a.ground {
        let facing = w.obj(id).map_or(0, |ob| ob.body.value(tick_now));
        let unit_stall = w.rules().quirks.aircraft_unit_step_stall;
        let bias = w.rules().quirks.table_trig_negative_bias; // spec: S-0105 F1 (quirks.table_trig_negative_bias)
        let (nx, ny) = step_b(
            here.x, here.y, facing, t.speed, a.cur, unit_stall, mode, bias,
        );
        if (nx, ny) != (here.x, here.y) {
            let mut np = Pos::new(nx, ny, here.z);
            let legal = |p: Pos| w.cells.cell(p.cell()).is_some();
            let may_leave = may_leave_now(w, id, &a);
            // spec: S-0501 Edge cases (quirks.aircraft_edge_hold on: the push-back of claim 4; off: an aircraft that may leave the map now is not pushed back, whatever its flags, with no candidate tried and no draw taken, so claim 10's removal on leaving the legal map applies)
            let hold = w.rules().quirks.aircraft_edge_hold;
            if !legal(np) && may_leave && !t.fly_by && hold {
                // spec: S-0501 claim 4 (r93, Q-2291: the side rule: +128 when the new position's cell has local column a < T, else −128)
                let back = if edge_column_low(w, np.cell()) {
                    128 // spec: S-0501 Constants (edge push-back 128 leptons)
                } else {
                    -128 // spec: S-0501 Constants (edge push-back 128 leptons)
                };
                let shifted = Pos::new(np.x + back, np.y, np.z);
                np = if legal(shifted) {
                    shifted
                } else {
                    // spec: S-0501 claim 4 (a random fallback 64 leptons from the shifted point: one raw draw, its low eight bits times 256 as the facing)
                    let f = ((w.rng.raw() & 0xff) * 256) as Facing;
                    // spec: S-0501 claim 4 (F3's displacement and truncation with length 64, no snapping)
                    let (rx, ry) =
                        crate::walk::step_point_b(shifted.x, shifted.y, f, 64, true, mode, bias); // spec: S-0501 Constants (edge random fallback radius 64)
                    let rc = Pos::new(rx, ry, np.z).cell();
                    // spec: S-0501 claim 4 (either cell coordinate outside 0 … 511: the candidate is the shifted point itself)
                    let cand = if (0..=511).contains(&rc.x) && (0..=511).contains(&rc.y) {
                        Pos::new(rx, ry, np.z)
                    } else {
                        shifted
                    };
                    // spec: S-0501 claim 4 (neither candidate legal: no forward relocation this tick)
                    if legal(cand) { cand } else { here }
                };
            }
            move_to(w, id, np);
        }
        removed = leave_map_test(w, id, &t);
    }
    if removed {
        return true;
    }
    let Some(o) = w.obj(id) else {
        return true;
    };
    // The object's fields after the forward step.
    let (stepped, current, o_dest) = (o.pos, o.beh.current, o.dest);
    // Altitude (claims 2 and 3).
    let h = height(w, id);
    if !a.ground {
        let climbing = h < a.held && alive;
        let nh = if climbing {
            climb(h, a.held, passengers(w, id) > 0)
        } else if h > a.held {
            descend(h, a.held, dead)
        } else {
            h
        };
        // The reference of this adjustment: the terrain, or the deck (S-0501 Terms, claim 43).
        let surface = base(w, id);
        let was_marked = a.deck;
        let temporary_deck = !was_marked && deck_reference(w, id);
        if climbing {
            // spec: S-0501 claim 43 (an ordinary altitude adjustment that climbs toward the held height clears the mark in that movement tick, the first climbing tick after a deck take-off included)
            a.deck = false;
        } else if h > a.held && temporary_deck && nh <= 0 {
            // spec: S-0501 claim 43 (ordinary descent that uses the high bridge as its reference and whose step reaches or passes height 0 above that deck sets the mark, with the resulting height 0 above the deck)
            a.deck = true;
        }
        // spec: S-0501 claim 43 (horizontal flight or hovering alone, without one of these altitude or landing transitions, leaves the mark unchanged)
        if let Some(ob) = w.obj_mut(id) {
            ob.pos.z = surface + nh;
        }
        if a.taking_off && nh >= a.held {
            a.taking_off = false;
        }
        // spec: S-0501 claim 4 (r183: a living descending aircraft that holds a flight point, with or without a destination object, moves 5 more leptons toward the centre of its flight point's cell, after the forward step and the height reduction, except during enter while its destination is a helipad building)
        // spec: S-0501 claims 6 and 8 (r183, Q-3041: the descent move needs a flight point, not a destination: landing where it is, after an ignored destination, and after a destination object left play, claim 53, it draws the aircraft toward the cell of the flight point it holds)
        let toward_helipad = current == Some(Behaviour::Enter)
            && matches!(o_dest, Some(Target::Object(k)) if helipad(w, k));
        if alive
            && nh < h
            && !toward_helipad
            && let Some(p) = dp
        {
            let c = p.cell();
            let centre = Pos::centre(c, 0);
            let now = w.obj(id).map_or(stepped, |x| x.pos);
            // spec: S-0501 claim 4 (the direction by S-0204 claim 8, F3's facing conversion and table trigonometry with step length 5, no clamp; the world height kept)
            let f = toward(w, now, centre);
            let bias = w.rules().quirks.table_trig_negative_bias; // spec: S-0105 F1 (quirks.table_trig_negative_bias)
            let (nx, ny) = crate::walk::step_point_b(now.x, now.y, f, 5, true, mode, bias); // spec: S-0501 Constants (descent horizontal nudge 5 leptons)
            let z = now.z;
            move_to(w, id, Pos::new(nx, ny, z));
            if let Some(ob) = w.obj_mut(id) {
                ob.pos.z = z;
            }
            // spec: S-0501 claim 10 (removed at once when its position leaves the usable area: the nudge's displacement too)
            if leave_map_test(w, id, &t) {
                return true;
            }
        }
    }
    // spec: S-0501 claim 10 (the aircraft has entered the usable area since it appeared: the mark this update's forward step set is kept when the update's own state is written back)
    a.entered |= st(w, id).entered;
    if let Some(ob) = w.obj_mut(id) {
        ob.air = Some(a.clone());
    }
    let mut a = st(w, id);
    // Speed fraction (claim 5, F4), at the end of a living movement tick.
    if alive && !a.ground {
        // spec: S-0501 claim 7 (r146, Q-2676: for the survivor holding the origin point, claim 5's calculation of the target fraction does not act, so it no longer changes the target fraction it was left with; only the step of the current fraction toward it goes on) and claim 5 (the fixed-full-speed case is that calculation's, which applies while the aircraft has a flight point)
        let fixed = !a.origin && full_speed(w, id, &t, &a);
        // The target of this tick's ordinary calculation, before any approach target replacement.
        let mut ordinary = None;
        let early_take_off = a.taking_off && height(w, id) < cruise / 2; // spec: S-0501 claim 5 (taking off: the target is kept until half the cruise height)
        if a.landing {
            a.cur = approach_fraction(a.cur, a.tgt, ar, mode);
        } else if full_request || fixed {
            a.tgt = F64::from_i64(1, mode);
            a.cur = approach_fraction(a.cur, a.tgt, ar, mode);
        } else if let Some(dd) = d
            && !early_take_off
        {
            let dd = dp.map_or(dd, |p| planar(w, w.obj(id).map_or(here, |x| x.pos), p));
            // spec: S-0501 claim 5 and F4 (the forward step comes first with the previous tick's fraction; d is measured from the position after it)
            let (tau, f) = slowing(a.cur, dd, t.slowdown, ar, mode);
            a.tgt = tau;
            a.cur = f;
            ordinary = Some(tau);
        } else {
            a.cur = approach_fraction(a.cur, a.tgt, ar, mode);
        }
        // spec: S-0501 claim 5 (the approach target replacement, without the fixed-full-speed case or the fly-by flag; the next ordinary calculation recomputes τ, so it governs only an adjustment in a later tick that skips that calculation)
        // spec: S-0501 claims 5 and 42 and claim 48 (c) (At height 0: the replacement is part of guidance, which runs only for an aircraft above the terrain at the end of that movement update, whatever the aircraft; at height 0 its target fraction is claim 5's ordinary one, Q-1941)
        if let Some(p) = dp
            && !a.landing
            && !a.taking_off
            && !fixed
            && !t.fly_by
            && height(w, id) > 0
        {
            let dd = planar(w, w.obj(id).map_or(here, |x| x.pos), p);
            let replacement = if dd < 128 {
                Some(F64::ZERO) // spec: S-0501 claim 5 (0 below 128 leptons)
            } else if dd < 512 {
                Some(ar.half) // spec: S-0501 claim 5 (0.5 from 128 to below 512)
            } else if dd < 768 {
                Some(ar.three_quarters) // spec: S-0501 claim 5 (0.75 from 512 to below 768)
            } else {
                None
            };
            if let Some(x) = replacement {
                a.tgt = x;
            }
        }
        if let Some(ob) = w.obj_mut(id) {
            ob.air = Some(a.clone());
        }
        landing_trigger(w, id, &t, fixed, ordinary);
    }
    // spec: S-0501 claim 7 (r134, Q-2646: the landing step, with the check of the flight point's cell and the touchdown, comes after the update's forward step, claim 3's descent and claim 6's triggers; Q-2648: a landing that the exact-cell trigger starts at height 0 touches down in that update's landing step, without waiting for the next update)
    if alive && landing_step(w, id, &t, started_landing) {
        return w.obj(id).is_some_and(|ob| ob.pos != start);
    }
    if alive
        && let Some(mut a) = st_opt(w, id).cloned()
        && a.released
        && !a.landing
        && !a.taking_off
        && flight_target(w, id).is_some()
        && height(w, id) > 0
    {
        // spec: S-0501 claim 48 (b) (a movement update that leaves it alive, with a destination, neither landing nor taking off and above the terrain gives it the held height of ordinary flight, claim 6, at the end of the update: it climbs from its next movement update on, whatever its target fraction and its distance; the next update's start recomputes claim 6's approach height)
        a.held = t.cruise;
        a.released = false;
        if let Some(ob) = w.obj_mut(id) {
            ob.air = Some(a);
        }
    }
    // spec: S-0501 claim 42 (guidance follows the movement tick)
    if alive {
        guidance(w, id, &t);
    }
    w.obj(id).is_some_and(|ob| ob.pos != start)
}

/// The take-off step's facing requests (claim 8) for an aircraft taking off
/// at height `h` from position `here`, toward point `to` (none: no body
/// request); whether full speed is requested.
fn take_off_requests(
    w: &mut World,
    id: ObjectId,
    cruise: i32,
    h: i32,
    here: Pos,
    to: Option<Pos>,
) -> bool {
    let hh = cruise; // spec: S-0501 claim 8 (H = cruise height minus landing height 0)
    // spec: S-0501 claim 8 (the thresholds H − trunc(H / 3) and trunc(H / 2))
    let (upper, lower) = (hh - hh / 3, hh / 2);
    if h > upper {
        // spec: S-0501 claim 8 (above H − trunc(H / 3): the aiming facing is requested to match the current body facing; no body request)
        let body = w.obj(id).map_or(0, |ob| ob.body.value(w.tick));
        request(w, id, Facet::Aim, body);
        false
    } else if h > lower {
        // spec: S-0501 claim 8 (above trunc(H / 2), up to and including the upper threshold: the body faces the destination and full speed is requested)
        if let Some(p) = to {
            request(w, id, Facet::Body, toward(w, here, p));
        }
        true
    } else {
        // spec: S-0501 claim 8 (at or below trunc(H / 2): no new heading request; the previous request persists)
        false
    }
}

/// The landing step of a movement update (claim 7), after its forward
/// step, claim 3's descent and claim 6's triggers: the check of the
/// landing's flight point cell, then the touchdown when the update has
/// brought the aircraft to the landing's held height 0. `started` is
/// whether the aircraft was landing at the start of the update. Returns
/// whether the update ends here (a touchdown, or a refused touchdown).
fn landing_step(w: &mut World, id: ObjectId, t: &Ty, started: bool) -> bool {
    let a = st(w, id);
    if !a.landing {
        return false;
    }
    if !started && t.landable {
        // spec: S-0102 claim 4 (every movement update of a landing re-files, from the update that starts it to the one that ends it: a landing that starts and ends in one update is re-filed in that update, which [`tick`] sees neither before nor after it)
        crate::combat::effects::layer_refile(w, id);
    }
    // spec: S-0501 claim 7 (r134, Q-2646: before touchdown each movement update of a landing checks its flight point's cell, at the update's landing step, after its forward step, claim 3's descent and claim 6's triggers, so a landing that the same update's exact-cell trigger starts is checked in that update, and the update whose descent reaches height 0 can still be refused)
    if !landing_cell_check(w, id, t, &a) {
        return check_refused(w, id, t);
    }
    if height(w, id) > 0 {
        return false;
    }
    // Touchdown (claim 7): changing into the ground layer.
    // spec: S-0501 claim 7 (r120: before its cell is tested, the touchdown forgets the destination and the flight point when the aircraft is in the flight point's cell, or, wherever it is, when that cell holds the building it is in contact with)
    let forgot = touchdown_forgets(w, id);
    if forgot {
        forget_point(w, id);
    }
    if w.rules().quirks.aircraft_descent_touchdown_untested {
        // spec: S-0501 claim 7 (r132: claim 3's descent brings every landing aircraft to height 0 before its landing step, so the touchdown is untested: no cell test, no refusal and none of an accepted touchdown's effects, the forget of the destination in any case included; only its own forget test applies) and Edge cases (quirks.aircraft_descent_touchdown_untested on; off: every touchdown is tested)
        if !forgot && st(w, id).flying_to.is_some() {
            // spec: S-0501 claim 7 (r132: an untested touchdown that its own forget test does not make forget its destination leaves the aircraft on the ground by its height with its destination and its flight point; r134, Q-2647: one it leaves holding its flight point with no destination goes on in the same way, since its movement follows its flight point and not its destination)
            untested_kept(w, id);
        } else {
            // spec: S-0501 claim 7 (r132: a touchdown whose forget test forgot the destination and the flight point, or with no flight point to compare, rests where it touched down)
            touchdown(w, id, false);
        }
    } else if landing_accepted(w, id) {
        touchdown(w, id, true);
    } else {
        // spec: S-0501 claim 7 (a refused cell: it rises by 10 leptons and starts to take off again)
        if let Some(ob) = w.obj_mut(id) {
            ob.pos.z += 10;
        }
        if let Some(s) = st_mut(w, id) {
            s.landing = false;
            s.taking_off = true;
            // spec: S-0501 claim 7 (r134: every touchdown, tested or not, sets the aircraft's current and target speed fractions to 0)
            s.cur = F64::ZERO;
            s.tgt = F64::ZERO;
            // spec: S-0501 claim 7 (r120: that take-off leaves its held height as it was, the landing's)
            s.ignored = None;
            // spec: S-0501 claim 7 (r120: a refusal that forgot the destination and the flight point leaves nothing to move it)
            s.stranded = forgot;
            // spec: S-0501 claim 43 (a touchdown refused under claim 7 clears the mark as take-off restarts)
            s.deck = false;
        }
        // spec: S-0501 claim 48 (e) and claim 44 (above height 0 it holds no object-present flag: a low release that carried its flag into the landing loses it as take-off restarts; any other lander has none)
        clear_mark(w, id);
        // spec: S-0102 claim 4 (a refused landing cell starts a take-off in that update, which re-files it; the landing update re-files it too, in tick)
    }
    true
}

/// The touchdown's forget of the destination and the flight point (claim
/// 7, r120).
fn forget_point(w: &mut World, id: ObjectId) {
    // spec: S-0501 claim 7 (r120: the destination and the flight point are forgotten; the landing is still under way, so claim 6's landing where it is does not apply). The flight point goes first, so that the cleared destination finds the aircraft holding none (claim 6's given-no-destination rule then changes nothing)
    if let Some(s) = st_mut(w, id) {
        s.flying_to = None;
        s.raised = false;
        s.unfinished = false;
        s.own_point = false;
        s.ignored = None;
        s.origin = false;
    }
    set_dest(w, id, None);
}

/// A landing's check of its flight point's cell has refused that cell
/// (claim 7, r134, Q-2646): the landing ends with a take-off toward cruise
/// height that ends in the same update, and the aircraft gets a free cell
/// near the cell it is over as its new flight point, its destination kept;
/// or, with none found, a hit of its current health. Returns whether the
/// update ends here (the touchdown that still follows the check).
fn check_refused(w: &mut World, id: ObjectId, t: &Ty) -> bool {
    // spec: S-0501 claim 7 (r134: a cell the check refuses ends the landing and starts a take-off toward cruise height, claim 8) and claim 47 (a) (which starts the aircraft's tracking only when it is not tracked already)
    take_off(w, id, t.cruise);
    let Some(o) = w.obj(id).cloned() else {
        return true;
    };
    let here = o.pos.cell();
    // spec: S-0501 claim 7 (r134: a free cell from S-0212 claim 25's nearby search around the cell it is over, which never gives a cell that holds a building) and Changes for other specs (S-0212 claim 25, table T5: centre the aircraft's cell, locomotion class track, movement zone fly, no reach region, never in deck mode, no height condition, no further test, no reference cell, bridge cells accepted)
    let settings = crate::path::NearbySettings {
        class: Some(LocomotionClass::Track),
        zone: MovementZone::Fly,
        reach: None,
        deck: false,
        height: false,
        reference: None,
        no_bridges: false,
        no_overlay: false,
    };
    let Some(c) = crate::path::nearby(w, here, crate::path::NearbyUse::Settings(settings)) else {
        // spec: S-0501 claim 7 (r134: when no cell is found, the aircraft takes a hit of its current health with the demolition warhead, which ignores its defences, lets no passenger escape and has no attacker and no house, S-0302 claim 1 deciding its outcome) and Changes for other specs (S-0302 claim 1: distance 0 leptons) and Import mapping (C4Warhead)
        crate::combat::damage::apply(
            w,
            id,
            crate::combat::damage::Hit {
                amount: o.health,
                warhead: w.rules().combat.general.c4_warhead,
                distance: 0,
                attacker: None,
                house: None,
                ignore_defences: true,
                no_escape: true,
            },
        );
        // spec: S-0501 claim 7 (r146, Q-2676: its flight point becomes the map's origin point, the point whose three coordinates are 0, and no touchdown follows in that update; its flight stays unfinished, so it is still moving, claim 6, and its locomotor does not report stopped, claim 40; its destination is kept, and no movement update hands it to its flight again)
        let kept = w.obj(id).and_then(|o| o.dest);
        if let Some(s) = st_mut(w, id) {
            s.flying_to = None;
            s.own_point = false;
            s.ignored = kept;
            s.raised = false;
            s.unfinished = true;
            s.origin = true;
        }
        if w.obj(id).is_none_or(crate::combat::damage::dead) {
            return true;
        }
        // spec: S-0501 claim 7 (r134: the refusal's take-off ends at the update's take-off step, with claim 8's requests for the height it has, when no touchdown follows; r146, Q-2676: those requests are made toward the origin point: above the upper threshold the aiming request, in the middle band the body facing the origin point from where the aircraft is and full speed, lower none)
        end_refusal_take_off(w, id, t, Some(ORIGIN));
        return false;
    };
    let raised = raise_now(w, id);
    if let Some(s) = st_mut(w, id) {
        // spec: S-0501 claim 7 (r134: when one is found, its flight point becomes the centre of that cell, at the height of its floor, or of the deck for a bridge cell, while its destination is kept; it flies there in ordinary flight with claim 6's triggers). Its flight ignores the kept destination while it is unchanged, as claim 8's exception does ([`AirState::ignored`]); with none, the point is its own
        s.flying_to = Some(Target::Cell(c));
        s.own_point = o.dest.is_none();
        s.ignored = o.dest;
        // spec: S-0501 claim 6 (the height replacement is fixed with the point when it is accepted: a target and ammunition now)
        s.raised = raised;
        // spec: S-0501 claim 40 (a movement point accepted by the locomotor is an unfinished flight)
        s.unfinished = true;
    }
    if height(w, id) > 0 {
        let to = flight_point(w, id, Target::Cell(c));
        end_refusal_take_off(w, id, t, Some(to));
        return false;
    }
    // spec: S-0501 claim 7 (r134: when a cell is found and the refusing update leaves the aircraft at or below the landing's held height, that update's touchdown still follows the check; it makes no cell test with quirks.aircraft_descent_touchdown_untested on or off, since the refusal has already ended the landing, and its forget test is made against the new flight point)
    if touchdown_forgets(w, id) {
        forget_point(w, id);
        touchdown(w, id, false);
    } else {
        refusal_touchdown_kept(w, id);
    }
    true
}

/// The take-off of a landing check's refusal ends in its update (claim 7,
/// r134): at its take-off step, which makes claim 8's take-off requests for
/// the height the aircraft has, toward point `to`.
fn end_refusal_take_off(w: &mut World, id: ObjectId, t: &Ty, to: Option<Pos>) {
    let (h, here) = (
        height(w, id),
        w.obj(id).map_or(Pos::new(0, 0, 0), |o| o.pos),
    );
    // spec: S-0501 claim 7 (r134: that take-off, unlike a refused touchdown's, ends in the same movement update, at its take-off step, which makes claim 8's take-off requests for the height the aircraft has; the climb comes from the next movement update, since the refusing update has already changed the height)
    let full = take_off_requests(w, id, t.cruise, h, here, to);
    let one = F64::from_i64(1, w.rules().mode);
    if let Some(s) = st_mut(w, id) {
        s.taking_off = false;
        if full {
            // spec: S-0501 claim 8 (above trunc(H / 2): full speed is requested)
            s.tgt = one;
        }
    }
}

/// The touchdown in a refusing update (claim 7, r134) that its forget test
/// against the new flight point does not make forget: on the ground by its
/// height, with its destination and that flight point, the held height of
/// ordinary flight from its take-off, so it climbs from its next movement
/// update with no slide first.
fn refusal_touchdown_kept(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else { return };
    let surface = w.surface_at(o.pos.x, o.pos.y);
    // spec: S-0501 claim 43 (landing finishes on a high-bridge cell while the world height is at least the terrain surface height plus 416 leptons: the deck mark is set; an already marked aircraft keeps it)
    let marked =
        deck_marked(w, id) || (w.high_bridge(o.pos.cell()) && o.pos.z - surface >= DECK_ALLOWANCE);
    if let Some(ob) = w.obj_mut(id) {
        ob.pos.z = surface + if marked { DECK_ALLOWANCE } else { 0 };
    }
    // spec: S-0501 claim 47 (b) (r134: the touchdown that follows a landing check's refusal in the same movement update ends the aircraft's tracking, whether or not the refusal's take-off started it, and returns its listing in its cell layer)
    end_tracking(w, id);
    if let Some(a) = st_mut(w, id) {
        a.deck = marked;
        a.landing = false;
        a.taking_off = false;
        a.ground = false;
        a.stranded = false;
        a.released = false;
        // spec: S-0501 claim 7 (r134: one that it does not make forget climbs from its next movement update, with no slide first, since its take-off gave it the held height of ordinary flight)
        a.sliding = false;
        // spec: S-0501 claim 7 (r134: every touchdown, tested or not, sets the aircraft's current and target speed fractions to 0)
        a.cur = F64::ZERO;
        a.tgt = F64::ZERO;
    }
    // spec: S-0501 claim 47 (b) (r134: the listing it returns sets that cell's object-present flag, S-0209 claim 13 and claim 44's rule for the record; the climb ends that listing in its next movement update, [`kept_mark`])
    clear_mark(w, id);
    set_mark(w, id);
    // spec: S-0212 claim 13 (each landing records the aircraft in its landing cell)
    crate::threat::landed(w, id);
}

/// Moves the aircraft to `np`, keeping its cell listing.
fn move_to(w: &mut World, id: ObjectId, np: Pos) {
    let Some(o) = w.obj(id) else { return };
    let from = o.pos.cell();
    let to = np.cell();
    if let Some(ob) = w.obj_mut(id) {
        ob.pos.x = np.x;
        ob.pos.y = np.y;
    }
    if from != to {
        w.cells.leave_both(from, id);
        // The listing in the new cell follows the listing height (S-0209 claim 13).
        sync_listing(w, id);
        // spec: S-0516 claim 22 and Edge cases (quirks.flying_sensor_disc_stationary: a crossing in flight relocates no sensor disc; off, it does)
        crate::cloak::machine::flight_cell_crossed(w, id);
    }
    if w.cells.usable(to)
        && let Some(a) = st_mut(w, id)
    {
        a.entered = true;
    }
}

/// Claim 10: may leave the map now.
fn may_leave_now(w: &World, id: ObjectId, a: &AirState) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let b = o.beh.current;
    // spec: S-0501 claim 10 (visiting, entered once, no target or in spy-plane overfly, not in either paradrop behaviour)
    a.visiting
        && a.entered
        && (o.target.is_none() || b == Some(Behaviour::SpyPlaneOverfly))
        && !matches!(
            b,
            Some(Behaviour::ParadropApproach | Behaviour::ParadropOverfly)
        )
}

/// Claim 4's side rule for the edge push-back: whether cell `c` has a local
/// column below *T*, so the shift is +128 (else −128).
fn edge_column_low(w: &World, c: CellXY) -> bool {
    let fp = w.cells.footprint();
    let lx = fp.local[0];
    // spec: S-0501 claim 4 (r93, Q-2291: d = cₓ − cᵧ, S-0009 claim 1; a = ⌊(d + W) / 2⌋ − Lx, the inverse of S-0522 F2, floor division)
    let a = (c.x - c.y + fp.width).div_euclid(2) - lx;
    // spec: S-0501 claim 4 (r93: T = ⌊(W + H − 1) / 2⌋, half the largest legal cell coordinate, S-0009 claim 2)
    let t = (fp.width + fp.height - 1).div_euclid(2);
    a < t
}

/// The removal tests of claim 10; whether the aircraft was removed.
fn leave_map_test(w: &mut World, id: ObjectId, t: &Ty) -> bool {
    let Some(o) = w.obj(id) else { return true };
    let a = o.air.clone().unwrap_or_default();
    let c = o.pos.cell();
    let usable = w.cells.usable(c);
    let legal = w.cells.cell(c).is_some();
    let retreat = o.beh.current == Some(Behaviour::Retreat);
    // spec: S-0501 claim 10 (eligible: removed outside the usable area without either flag, and in any case off the map; in retreat, removed outside the usable area)
    let gone = (may_leave_now(w, id, &a) && ((!usable && !t.fly_by && !t.fly_back) || !legal))
        || (retreat && !usable);
    if gone {
        if a.paradrop {
            // spec: S-0522 claim 8 (paratrooper still on board leave play with the plane without a death, loss count, kill credit or experience award; this overrides S-0507 claim 16's killing at removal)
            for p in crate::transport::passengers(w, id) {
                crate::objects::remove(w, p);
            }
        }
        crate::objects::remove(w, id);
    }
    gone
}

/// The landing triggers of claim 6. `ordinary` is the target that this
/// tick's ordinary calculation of claim 5 gave, before any approach target
/// replacement (none when the calculation did not run).
fn landing_trigger(w: &mut World, id: ObjectId, t: &Ty, fixed: bool, ordinary: Option<F64>) {
    let Some(o) = w.obj(id).cloned() else { return };
    let a = st(w, id);
    if a.holding || a.landing || a.taking_off || a.ground {
        return;
    }
    let no_fire = o.target.is_none() || !ammo_left(w, id);
    if !no_fire {
        return;
    }
    // spec: S-0501 claim 6 (r93, Q-2252: the cell tested is that of the flight point the flight holds, claim 40, not that of the destination object; with no flight point the test does not apply)
    let trigger = match flight_target(w, id) {
        // spec: S-0501 claim 6 (with no flight point the test does not apply) and r120 (Q-2507: an aircraft given no destination in flight, moving and not landing lands where it is through the flight point that gives it, [`given_no_destination`]; one that is not moving holds no flight point and starts no landing)
        // spec: S-0501 claim 7 (r146, Q-2676: the survivor holding the origin point: claim 6's exact-cell trigger tests the origin point's cell and never finds it there with a target fraction of 0, as it stops where it was hit or flies on at fraction 1; this test reads the target of the update's claim 5 calculation, which does not run for it, with the same outcome; the close-approach trigger belongs to guidance, which does not act; so no movement update starts a landing or repeats the check or the hit)
        None => false,
        Some(dd) => {
            let p = flight_point_of(w, id, dd, Some(t));
            let dist = planar(w, o.pos, p);
            // spec: S-0501 claim 6 (r93, Q-2252: standing in its flight point's cell with the target fraction of the same tick's ordinary calculation of claim 5 at 0; a replacement target is never tested, so within 128 leptons it does not start to land while its ordinary target is positive) and claim 5 (r93: that calculation runs in every tick in which the test can apply)
            // spec: S-0501 claim 7 (r134, Q-2648: claim 6's exact-cell trigger applies to an aircraft sliding on the ground too, as at any height: in an update in which it stands in its flight point's cell with that update's target fraction at 0 it starts a landing, and that update's landing step touches down at once, at height 0, [`landing_step`])
            let exact = o.pos.cell() == p.cell() && ordinary.is_some_and(|x| x.eq_num(F64::ZERO));
            // spec: S-0501 claim 6 (close approach: below 128 leptons, ordinary slowing, current fraction below 0.05) and claims 42 and 48 (c) (At height 0: the close-approach trigger is a request of guidance, which runs only for an aircraft above the terrain, whatever the aircraft; the exact-cell trigger applies at any height, Q-1941)
            let close =
                dist < 128 && !fixed && a.cur.lt(w.rules().air.twentieth) && height(w, id) > 0;
            exact || close
        }
    };
    if !trigger {
        return;
    }
    // spec: S-0501 claim 6 (a further waypoint outside enter: wait is queued and it goes on)
    if o.beh.current != Some(Behaviour::Enter) && !o.queued_dests.is_empty() {
        let next = o.queued_dests[0];
        let ignored = queue_ignored(w, id, Behaviour::Wait);
        if let Some(ob) = w.obj_mut(id) {
            ob.queued_dests.remove(0);
            if !ignored {
                ob.beh.queue(Some(Behaviour::Wait));
            }
        }
        set_dest(w, id, Some(next));
        return;
    }
    // spec: S-0213 claim 16d (revision 84, Q-2192: an aircraft issues its next route point when a flight ends, at the moment it would begin to land, at either landing trigger, unless its behaviour is then enter) and revision 89 (Q-2229: this issue is S-0501 claim 6's going on to a further waypoint of a planned route, and it comes before that claim's pad-bound test, so a pad-bound aircraft away from its pad that has a point to issue issues it and does not fall back to rest; an aircraft that issues a point begins no landing in that tick and stays in flight; the issue is preceded by claim 16c's queue of wait with an immediate start) and Changes for other specs (S-0501 claim 6, planned-route exception). The waypoint step above serves only destinations queued by other paths, which give an aircraft none
    if o.beh.current != Some(Behaviour::Enter) && crate::routes::issue_later(w, id) {
        return;
    }
    // spec: S-0501 claim 6 (a pad-bound aircraft whose cell holds no building it is in contact with falls back to rest instead: an armed one to a pad or a crash, an unarmed one to an F8 landing spot near which it stays in the air, claim 50)
    // design: S-0501 Open question 26 (a pad-bound aircraft released from a docking position outside the foundation that took its destination without a behaviour order still holds its contact in flight: in a destination cell holding no building it is in contact with it falls back, and T2's after-the-row enter is claim 24's, which takes the pad as its destination; in a destination cell of the pad it is in contact with the restriction does not hold, so it starts to land; its touchdown makes no entry report itself, claim 7 r103: outside enter it rests there in contact)
    if t.bound && !o.contact.is_some_and(|k| pad_cell(w, k, id)) {
        if t.docks.is_empty() && armed(w, id) && ammo_left(w, id) {
            // spec: S-0501 claims 6 and 45 (except a pad-bound type with no dock types and ammunition left: it keeps its cell and stays above it)
            return;
        }
        fall_back(w, id);
        return;
    }
    start_landing(w, id);
}

// ---------------------------------------------------------------------------
// Pads (claims 21 to 28)

/// Whether building `k` is a helipad building (claim 22).
fn helipad(w: &World, k: ObjectId) -> bool {
    w.obj(k).is_some_and(|b| {
        b.kind == Kind::Building
            && w.rules()
                .buildings
                .get(b.type_index as usize)
                .is_some_and(|r| r.raw.helipad.unwrap_or(false))
    })
}

/// Whether pad `k` accepts aircraft `id` (claim 22).
pub fn accepts(w: &World, k: ObjectId, id: ObjectId) -> bool {
    let (Some(b), Some(o)) = (w.obj(k), w.obj(id)) else {
        return false;
    };
    // spec: S-0501 claim 22 (a dead building accepts nothing)
    if b.kind != Kind::Building || b.health <= 0 || !b.on_map {
        return false;
    }
    // spec: S-0501 claim 22 (the pad's house allied with the aircraft's)
    if !w.allied(b.owner, o.owner) {
        return false;
    }
    // spec: S-0501 claim 22 (not being built up or sold, build-up finished)
    if matches!(
        b.beh.current.or(b.beh.queued),
        Some(Behaviour::Construction | Behaviour::Selling)
    ) || b.econ.sale.is_some()
    {
        return false;
    }
    // spec: S-0501 claim 22 (switched on, S-0405; low power does not matter)
    if b.econ.building.switched_off {
        return false;
    }
    // spec: S-0501 claim 22 (a helipad building accepts only aircraft)
    let helipad = w
        .rules()
        .buildings
        .get(b.type_index as usize)
        .is_some_and(|r| r.raw.helipad.unwrap_or(false));
    if helipad && o.kind != Kind::Aircraft {
        return false;
    }
    free_place(w, k, id)
}

/// A free contact place for the aircraft: one that is empty or already holds
/// it (claim 22).
fn free_place(w: &World, k: ObjectId, id: ObjectId) -> bool {
    if w.obj(id).is_some_and(|o| o.contact == Some(k)) {
        return true;
    }
    let n = crate::econ::release::contact_places(w, k);
    crate::econ::release::first_free_place(w, k, n) < n
}

/// F9: the squared planar distance between centres, signed 32-bit and
/// wrapping (`quirks.pad_distance_overflow`), or exact.
pub fn pad_distance(dx: i32, dy: i32, overflow: bool) -> i64 {
    if overflow {
        // spec: S-0501 F9 (a signed 32-bit integer that wraps on overflow, S-0205 claim 1)
        i64::from(dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy)))
    } else {
        // spec: S-0501 Edge cases (quirks.pad_distance_overflow off: exact squared distances)
        i64::from(dx) * i64::from(dx) + i64::from(dy) * i64::from(dy)
    }
}

/// The distance-or-primary preference of claim 21 over `cands` in order.
fn prefer(cands: &[(ObjectId, i64, bool)]) -> Option<ObjectId> {
    let mut best: Option<(ObjectId, i64)> = None;
    for &(b, d, primary) in cands {
        // spec: S-0501 claim 21 (a saved distance of −1 is no choice; strictly smaller, or primary, replaces)
        let replace = match best {
            None => true,
            Some((_, bd)) => bd == -1 || d < bd || primary,
        };
        if replace {
            best = Some((b, d));
        }
    }
    best.map(|(b, _)| b)
}

/// Chooses a pad (claim 21). No random draw.
pub fn find_pad(w: &World, id: ObjectId) -> Option<ObjectId> {
    let o = w.obj(id)?;
    let t = ty(w, id)?;
    let a = st(w, id);
    // spec: S-0501 claim 21 (a pad-bound aircraft first asks the pad it was last heading for)
    if t.bound
        && let Some(k) = a.heading_pad
        && accepts(w, k, id)
    {
        return Some(k);
    }
    let owner = o.owner?;
    let overflow = w.rules().quirks.pad_distance_overflow;
    let order = &w.houses.get(usize::from(owner.0))?.econ.acquired;
    let centre = w.reference_point(id);
    let mut bests = Vec::new();
    for &dt in &t.docks {
        let mut cands = Vec::new();
        // spec: S-0501 claim 21 (within a dock type, house building-list order; not in limbo, with a free place and accepting)
        for &b in order {
            let Some(bo) = w.obj(b) else { continue };
            if bo.kind != Kind::Building
                || bo.type_index != dt
                || bo.owner != Some(owner)
                || !bo.on_map
            {
                continue;
            }
            if !accepts(w, b, id) {
                continue;
            }
            let c = w.reference_point(b);
            let d = pad_distance(c.x - centre.x, c.y - centre.y, overflow);
            cands.push((b, d, bo.econ.building.primary));
        }
        if let Some(best) = prefer(&cands) {
            let c = w.reference_point(best);
            let d = pad_distance(c.x - centre.x, c.y - centre.y, overflow);
            let primary = w.obj(best).is_some_and(|x| x.econ.building.primary);
            bests.push((best, d, primary));
        }
    }
    // spec: S-0501 claim 21 (among the per-type candidates, dock-list order and the same preference)
    prefer(&bests)
}

/// Claim 21's pad search as the simulation makes it: [`find_pad`]'s
/// choice, recorded as the pad the aircraft was last heading for.
pub fn pad_search(w: &mut World, id: ObjectId) -> Option<ObjectId> {
    let k = find_pad(w, id);
    // spec: S-0501 claim 21 (r176, Q-2866: every search records its result as the pad the aircraft was last heading for, the pad it keeps, the pad it chooses or no pad when it finds none, in every behaviour and at every use, pad-bound or not; a pad-bound aircraft whose recorded pad no longer accepts it loses that record and the result replaces it)
    record_heading(w, id, k);
    k
}

/// Records `k` as the pad aircraft `id` was last heading for (claim 21).
pub(crate) fn record_heading(w: &mut World, id: ObjectId, k: Option<ObjectId>) {
    if let Some(a) = st_mut(w, id) {
        a.heading_pad = k;
    }
}

/// Building `k` is destroyed or removed from the game (claim 21): every
/// aircraft that recorded it as the pad it was last heading for is left
/// with no pad recorded.
pub(crate) fn heading_pad_gone(w: &mut World, k: ObjectId) {
    for o in w.objects.values_mut() {
        if let Some(a) = o.air.as_mut()
            && a.heading_pad == Some(k)
        {
            // spec: S-0501 claim 21 (r176, Q-2866: the record changes when the recorded building is destroyed or removed from the game, which leaves no pad recorded)
            a.heading_pad = None;
        }
    }
}

/// Asks pad `k` for contact (claim 23); whether it was granted.
pub fn request_contact(w: &mut World, id: ObjectId, k: ObjectId) -> bool {
    let (Some(b), Some(o)) = (w.obj(k), w.obj(id)) else {
        return false;
    };
    // spec: S-0501 claim 23 (the pad accepts it, has a free place, the two houses mutually allied)
    let mutual = w.allied(b.owner, o.owner) && w.allied(o.owner, b.owner);
    if !mutual || !accepts(w, k, id) {
        return false;
    }
    let place = if o.contact == Some(k) {
        o.econ
            .exit
            .filter(|e| e.factory == k)
            .map(|e| e.place)
            .unwrap_or_else(|| {
                let n = crate::econ::release::contact_places(w, k);
                crate::econ::release::first_free_place(w, k, n)
            })
    } else {
        let n = crate::econ::release::contact_places(w, k);
        crate::econ::release::first_free_place(w, k, n)
    };
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0501 claim 23 (an aircraft holding another contact drops it first; it then holds the pad's place)
        ob.contact = Some(k);
        ob.econ.exit = Some(crate::econ::release::Exit {
            factory: k,
            tethered: false,
            place,
        });
    }
    // spec: S-0501 claim 21 (r176, Q-2866: the record of the pad last heading for is the search's; a contact granted after it, in sticky or not, by claim 18's in-flight pad row or claim 24's contact rule, records nothing)
    true
}

/// T2's in-flight pad row (claim 18, r176, Q-2867): claim 21's search,
/// then the moment with no destination, then the contact request to the pad
/// found; the pad when it granted contact.
fn seek_pad(w: &mut World, id: ObjectId) -> Option<ObjectId> {
    // spec: S-0501 claim 18 (r176, Q-2867: the search comes first, so the pad it finds and records does not depend on the moment)
    let k = pad_search(w, id);
    // spec: S-0501 claim 18, The moment with no destination (r176, Q-2867: every fall-back that reaches the in-flight pad search gives the aircraft no destination for a moment after that search, whatever its current behaviour, whether the search finds a pad or none and whether the pad grants or refuses the contact; claim 6's effects by its state)
    set_dest(w, id, None);
    let k = k?;
    // spec: S-0501 claim 18 (asks the pad found for contact by the row's own request) and claim 23
    request_contact(w, id, k).then_some(k)
}

/// Claim 41's landing-search footprint test (S-0212 claim 25a) of the
/// `size` × `size` rectangle at top-left `tl`, against reference reach
/// region `reach` (none: no restriction).
fn landing_footprint_ok(w: &World, tl: CellXY, size: i32, reach: Option<Reach>) -> bool {
    let r = w.rules_arc();
    for j in 0..size {
        for i in 0..size {
            // spec: S-0212 claim 25a (the cells (x + i, y + j), exact integer additions from the top-left)
            let c = CellXY::new(tl.x + i, tl.y + j);
            if w.cells.cell(c).is_none() {
                return false;
            }
            let deck = w.cells.idx(c).is_some_and(|k| w.cells.deck[k]);
            // spec: S-0212 claim 25a (the ground reach region for zone normal matches the reference's, unless the reference has none; no deck mode)
            if let Some(rr) = reach
                && reach_read(w, c, MovementZone::Normal, false) != rr
            {
                return false;
            }
            // spec: S-0212 claim 25a (the occupancy record empty as a whole: the deck record on a bridge cell, else the ground record)
            let layer = if deck { Layer::Deck } else { Layer::Ground };
            if !w.cells.occ(c, layer).is_some_and(|o| o.empty()) {
                return false;
            }
            // spec: S-0212 claim 25a (enterable by foot and zone normal under S-0211 claim 10, a bridge cell as entered on its deck)
            if !crate::passability::enterable_by_class(
                &r,
                w,
                c,
                Some(LocomotionClass::Foot),
                MovementZone::Normal,
                deck,
            ) {
                return false;
            }
        }
    }
    true
}

/// One landing search of claim 41 around `centre`: footprint `size`, the
/// ground reach region of cell `reach_of` for zone normal, and an optional
/// reference cell.
fn landing_search(
    w: &World,
    centre: CellXY,
    size: i32,
    reach_of: CellXY,
    reference: Option<CellXY>,
) -> Option<CellXY> {
    // spec: S-0501 claim 41 (the ground reach region for zone normal of the search's cell); S-0212 claim 25a (a reference without a reach region removes the restriction)
    let rr = reach_read(w, reach_of, MovementZone::Normal, false);
    let reach = (rr != Reach::None).then_some(rr);
    // spec: S-0501 claim 41 (class foot, zone normal, no deck mode, bridges allowed, no height, no-overlay or burrowing condition); S-0212 claim 25a (only the top-left cell takes the height-aware usable-area test; the class, region and occupancy tests are the footprint's, on every cell)
    let settings = crate::path::NearbySettings {
        class: Some(LocomotionClass::Winged),
        zone: MovementZone::Normal,
        reach: None,
        deck: false,
        height: false,
        reference,
        no_bridges: false,
        no_overlay: false,
    };
    let extra = |w: &World, c: CellXY| landing_footprint_ok(w, c, size, reach);
    // spec: S-0212 claim 25a (the result is the selected top-left cell itself)
    crate::path::nearby_filtered(
        w,
        centre,
        crate::path::NearbyUse::Settings(settings),
        Some(&extra),
    )
}

/// The landing spot without a pad (F8, claim 41).
pub fn landing_spot(w: &mut World, id: ObjectId) -> Option<CellXY> {
    let o = w.obj(id)?.clone();
    let t = ty(w, id)?;
    let own = o.pos.cell();
    crate::regions::ensure(w);
    let me = w.reference_point(id);
    let q = w.rules().quirks;
    let mode = w.rules().mode;
    // spec: S-0501 F8 (the truncated table root of the spatial centre-to-centre squared difference)
    let dist = |w: &World, x: ObjectId| -> i64 {
        let c = w.reference_point(x);
        let (dx, dy, dz) = (
            i64::from(c.x - me.x),
            i64::from(c.y - me.y),
            i64::from(c.z - me.z),
        );
        root_floor(
            (dx * dx + dy * dy + dz * dz) as u64,
            q.table_root_distance,
            mode,
        ) as i64
    };
    let owner = o.owner;
    if !t.docks.is_empty()
        && let Some(owner) = owner
    {
        let order = w.houses[usize::from(owner.0)].econ.acquired.clone();
        let mut best: Option<(i64, CellXY)> = None;
        for b in order {
            let Some(bo) = w.obj(b) else { continue };
            if bo.kind != Kind::Building || bo.owner != Some(owner) || !bo.on_map {
                continue;
            }
            let bc = bo.pos.cell();
            // spec: S-0501 claim 41 (the building search: a 3 × 3 free footprint, the building cell's ground reach region, no reference cell)
            let Some(s) = landing_search(w, bc, 3, bc, None) else {
                continue;
            };
            let mut d = dist(w, b);
            if bo.type_index == t.docks[0] {
                // spec: S-0501 F8 (for the first dock type divided by 4, truncating)
                d /= 4;
            }
            // spec: S-0501 F8 (the smallest, the earlier on a tie)
            if best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, s));
            }
        }
        if let Some((d, s)) = best {
            // spec: S-0501 F8 (below 256 leptons)
            if d >= 256 {
                // spec: S-0501 F8 (otherwise the free spot near the chosen building)
                return Some(s);
            }
            // spec: S-0501 F8 (below 256 leptons a free cell near its own cell) and claim 41 (a 1 × 1 footprint, the own cell's ground reach region, the own cell as reference)
            if let Some(c) = landing_search(w, own, 1, own, Some(own)) {
                return Some(c);
            }
            if q.landing_own_cell_no_result {
                // spec: S-0501 claim 41 and F8 (Q-0650: an own-cell search with no result makes the landing destination the placeholder at map cell (0, 0), not either fallback; quirks.landing_own_cell_no_result)
                return Some(PLACEHOLDER);
            }
            // spec: S-0501 Edge cases (quirks.landing_own_cell_no_result off: no successful building choice; the other-object fallback follows, the chosen building counted like any other own-house object)
        }
    }
    // spec: S-0501 claim 41 (the fallback: other own-house infantry, vehicles, aircraft and buildings, not self and not in limbo; the smallest spatial table-root distance whose 1 × 1 search succeeds)
    let mut cands: Vec<(i64, ObjectId)> = w
        .objects
        .values()
        .filter(|x| {
            x.id != id
                && x.on_map
                && owner.is_some()
                && x.owner == owner
                && matches!(
                    x.kind,
                    Kind::Infantry | Kind::Vehicle | Kind::Aircraft | Kind::Building
                )
        })
        .map(|x| (dist(w, x.id), x.id))
        .collect();
    // spec: S-0501 claim 41 (the distance truncated to integer leptons; equal distances favour the object created earliest, across all four kinds; object numbers are given in creation order)
    cands.sort();
    for (_, x) in cands {
        let Some(c) = w.obj(x).map(|x| x.pos.cell()) else {
            continue;
        };
        // spec: S-0501 claim 41 (the other object's cell, a 1 × 1 footprint, otherwise the building-search settings); S-0212 claim 25a (no reference: tick modulo)
        if let Some(s) = landing_search(w, c, 1, c, None) {
            return Some(s);
        }
    }
    // spec: S-0501 claim 41 (no successful object choice: the aircraft's own current cell, without another free-cell search)
    Some(own)
}

// ---------------------------------------------------------------------------
// Edge cells (claims 34 and 37; S-0522 claims 4 and 5)

/// The cell of local point (a, b) (S-0522 F2).
fn local_cell(w: &World, a: i32, b: i32) -> CellXY {
    let fp = w.cells.footprint();
    let [lx, ly, _, _] = fp.local;
    // spec: S-0522 F2 (X = a + Lx, Y = b + Ly; x = ⌊(Y + 1) / 2⌋ + X, y = W + ⌊Y / 2⌋ − X, floor division)
    let (x, y) = (a + lx, b + ly);
    CellXY::new((y + 1).div_euclid(2) + x, fp.width + y.div_euclid(2) - x) // spec: S-0522 F2 (⌊(Y + 1) / 2⌋ + X, W + ⌊Y / 2⌋ − X)
}

/// A house's computed edge is computed again (S-0522 claim 5) because
/// `building`, a building factory of the house, was marked primary: the
/// edge whose midpoint (F3) is nearest the cell of its centre.
pub fn recompute_edge(w: &mut World, building: ObjectId) {
    let Some(b) = w.obj(building) else { return };
    let Some(owner) = b.owner else { return };
    // spec: S-0522 claim 5 (the reference cell is the cell of that building's centre)
    let c = w.reference_point(building).cell();
    let [_, _, lw, lh] = w.cells.footprint().local;
    // spec: S-0522 F3 (the midpoints: north (⌊Lw / 2⌋, 1), east (Lw, Lh), south (⌊Lw / 2⌋, 2Lh), west (0, Lh))
    let mids = [
        (lw.div_euclid(2), 1),
        (lw, lh),
        (lw.div_euclid(2), 2 * lh),
        (0, lh),
    ];
    let (q, mode) = (w.rules().quirks.table_root_distance, w.rules().mode);
    let mut best: Option<(u64, u8)> = None;
    for (e, &(a, bb)) in (0u8..).zip(mids.iter()) {
        let m = local_cell(w, a, bb);
        let (dx, dy) = (i64::from(c.x - m.x), i64::from(c.y - m.y));
        // spec: S-0522 F3 (D = trunc(r((x − xₑ)² + (y − yₑ)²)) in cells, the table root of S-0204 claim 5)
        let d = root_floor((dx * dx + dy * dy) as u64, q, mode);
        // spec: S-0522 F3 (the smallest distance; ties to the smaller edge number)
        if best.is_none_or(|(bd, _)| d < bd) {
            best = Some((d, e));
        }
    }
    if let (Some((_, e)), Some(h)) = (best, w.houses.get_mut(usize::from(owner.0))) {
        h.computed_edge = e;
    }
}

/// The edge-cell selection of S-0522 claim 4 on edge `edge` (0 north,
/// 1 east, 2 south, 3 west), with its draws.
pub fn edge_cell_on(w: &mut World, edge: u8) -> CellXY {
    let [_, _, lw, lh] = w.cells.footprint().local;
    let q = w.rules().quirks;
    let half = q.ranged_draw_half_span; // spec: S-0202 claim 4 (the ranged draw's half-span rule)
    // spec: S-0522 claim 4 (a candidate is acceptable when its cell is outside the usable area by the height-aware test)
    let ok = |w: &World, a: i32, b: i32| !w.cells.usable(local_cell(w, a, b));
    let found = if lw <= 0 || lh <= 0 {
        // A local area without positive size has no candidate points; the
        // fallback point below applies (S-0522 claim 4).
        None
    } else {
        match edge {
            0 => {
                // spec: S-0522 claim 4 (north: r = R(1, Lw); points ((i + r − 1) mod Lw, 0) in cyclic order)
                let r = w.rng.ranged(1, lw, half);
                (0..lw)
                    .map(|i| ((i + r - 1).rem_euclid(lw), 0))
                    .find(|&(a, b)| ok(w, a, b))
            }
            1 => {
                // spec: S-0522 claim 4 (east: r = R(1, 2Lh); points (Lw, (i + r − 1) mod 2Lh))
                let r = w.rng.ranged(1, 2 * lh, half);
                (0..2 * lh) // spec: S-0522 claim 4 (east: 2Lh points)
                    .map(|i| (lw, (i + r - 1).rem_euclid(2 * lh))) // spec: S-0522 claim 4 (east: (i + r − 1) mod 2Lh)
                    .find(|&(a, b)| ok(w, a, b))
            }
            // spec: S-0522 claim 4 (edge 2 is south)
            2 => {
                // spec: S-0522 claim 4 (south: one R(1, Lw) draw discarded)
                let _ = w.rng.ranged(1, lw, half);
                // spec: S-0522 claim 4 (south: per column a, the first acceptable (a, 2Lh + j), j = 0 … 14)
                let kept: Vec<(i32, i32)> = (0..lw)
                    .filter_map(|a| {
                        (0..=14)
                            .map(|j| (a, 2 * lh + j))
                            .find(|&(a, b)| ok(w, a, b))
                    })
                    .collect();
                if kept.is_empty() {
                    None
                } else {
                    // spec: S-0522 claim 4 (south: k = R(0, n − 1), the retained point k by increasing column)
                    let k = w.rng.ranged(0, kept.len() as i32 - 1, half);
                    kept.get(k as usize).copied()
                }
            }
            _ => {
                // spec: S-0522 claim 4 (west: r = R(0, 2Lh); rows rem(r − 1 + i, 2Lh) with the dividend's sign); Edge cases (quirks.paradrop_west_edge_start_row off: the first row is r mod 2Lh, then cyclic)
                let r = w.rng.ranged(0, 2 * lh, half);
                (0..2 * lh) // spec: S-0522 claim 4 (west: 2Lh rows)
                    .map(|i| {
                        let b = if q.paradrop_west_edge_start_row {
                            (r - 1 + i) % (2 * lh) // spec: S-0522 claim 4 and Edge cases (rem(r − 1 + i, 2Lh))
                        } else {
                            (r + i).rem_euclid(2 * lh) // spec: S-0522 Edge cases (r mod 2Lh, then cyclic)
                        };
                        (0, b)
                    })
                    .find(|&(a, b)| ok(w, a, b))
            }
        }
    };
    let (a, b) = found.unwrap_or(if q.paradrop_fallback_point {
        // spec: S-0522 claim 4 (no acceptable candidate: local point (1, ⌊Lw / 2⌋)); Edge cases (quirks.paradrop_fallback_point)
        (1, lw.div_euclid(2))
    } else {
        // spec: S-0522 Edge cases (quirks.paradrop_fallback_point off: (⌊Lw / 2⌋, 1))
        (lw.div_euclid(2), 1)
    });
    local_cell(w, a, b)
}

/// The edge cell of claim 34 for an aircraft: on its house's arrival edge
/// (S-0522 claim 5), or with `opposite` on the edge opposite its house's
/// computed edge (the spy plane's exit), selected as in S-0522 claim 4.
fn arrival_edge_cell(w: &mut World, id: ObjectId, opposite: bool) -> Option<CellXY> {
    let owner = w.obj(id)?.owner?;
    // spec: S-0522 claim 5 (the computed edge; a map section's named edge is not read, S-0522 Open questions 1)
    let e = w.houses.get(usize::from(owner.0))?.computed_edge;
    // spec: S-0501 claim 34 (the spy exit is on the edge opposite the computed edge: north and south, east and west)
    let e = if opposite { (e + 2) % 4 } else { e };
    Some(edge_cell_on(w, e))
}

/// A cell on house `h`'s arrival edge (S-0522 claim 5), selected with the
/// draws of S-0522 claim 4: a paradrop or spy plane's arrival cell.
pub fn house_edge_cell(w: &mut World, h: HouseId) -> CellXY {
    // spec: S-0522 claim 5 (the computed edge; a map section's named edge is not read, S-0522 Open questions 1)
    let e = w
        .houses
        .get(usize::from(h.0))
        .map_or(0, |x| x.computed_edge);
    edge_cell_on(w, e)
}

/// The spy plane's camera (S-0513 claim 44) in a processing of its
/// approach or overfly: when it had its destination at the start of the
/// processing and its ground distance to the target is at most its
/// primary weapon's range. `sound` plays the camera sound (approach).
fn spy_camera(w: &mut World, o: &crate::world::Object, id: ObjectId, sound: bool) {
    let (Some(tg), Some(_)) = (o.target, o.dest) else {
        return;
    };
    let Some(wp) = crate::combat::weapon::slot_weapon(w, id, 0) else {
        // spec: S-0513 claim 44 (a spy type with no primary weapon gives no camera; our engine warns, departure)
        w.note("S-0513 claim 44: a spy plane has no primary weapon; it takes no camera");
        return;
    };
    let (range, damage) = {
        let wr = w.rules().combat.weapon(wp);
        (wr.range, wr.damage)
    };
    // spec: S-0513 claim 44 (the ground distance to the target, S-0204 claim 6, at most the primary weapon's range)
    if planar(w, o.pos, crate::process::target_pos(w, tg)) > range {
        return;
    }
    // spec: S-0513 claim 44 (the radius in cells is the primary weapon's damage)
    crate::econ::shroud::camera(w, id, damage);
    if sound {
        let owner = o.owner;
        let p = w.obj(id).map(|x| x.pos);
        // spec: S-0513 claim 44 (in approach each camera plays the camera sound at the plane; presentation)
        crate::superweapons::event(w, crate::superweapons::SwEvent::Camera, owner, None, p);
    }
}

/// The pad's service step (claims 26 to 28), in the pad building's turn.
pub fn pad_turn(w: &mut World, k: ObjectId) {
    let Some(next) = w
        .obj(k)
        .and_then(|b| b.air.as_ref())
        .and_then(|a| a.service)
    else {
        return;
    };
    let t = w.tick;
    if t < next {
        return;
    }
    let p = w.rules().air.service;
    let mut list: Vec<(u32, ObjectId)> = w
        .objects
        .values()
        .filter(|o| o.contact == Some(k) && o.kind == Kind::Aircraft)
        .map(|o| {
            (
                o.econ
                    .exit
                    .filter(|e| e.factory == k)
                    .map_or(u32::MAX, |e| e.place),
                o.id,
            )
        })
        .collect();
    // spec: S-0501 State and ordering (in a service step, aircraft in contact-place order, one step each)
    list.sort();
    let mut settled = false;
    for (_, x) in list {
        if !is_aircraft(w, x) {
            continue;
        }
        let Some(o) = w.obj(x).cloned() else { continue };
        // spec: S-0501 claim 26 (an aircraft still flying in, in enter, or still moving is skipped) and claim 24 (r110, Q-2436: neither the report nor the pad tests where it touched down, so one settled off the foundation is serviced there)
        // spec: S-0501 claims 26 and 52 and Edge cases (r154, quirks.aircraft_pad_sleep_ends_retaliation off: one whose current behaviour is attack is skipped too, as one in enter)
        let attack_skipped = !w.rules().quirks.aircraft_pad_sleep_ends_retaliation
            && o.beh.current == Some(Behaviour::Attack);
        let skipped = o.beh.current == Some(Behaviour::Enter)
            || attack_skipped
            || moving(w, x)
            || !st(w, x).ground;
        let release = |w: &mut World| {
            // spec: S-0501 claim 26 (released: it falls back to rest and its guard is forced)
            fall_back(w, x);
            let (t, s) = (w.tick, stale(w));
            if let Some(ob) = w.obj_mut(x) {
                ob.beh.force(Behaviour::Guard, t, s);
            }
        };
        // spec: S-0501 claim 26 (ready for take-off, no target and full ammunition, with full health: released) and r134 (Q-2649: before the skip, in the claim's order: one that slides after its report, not settled, is skipped unless claim 26 releases it first)
        if o.target.is_none() && full_ammo(w, x) && full_health(w, x) {
            release(w);
            continue;
        }
        if skipped {
            continue;
        }
        // spec: S-0501 claim 26 (r154: servicing goes on while a settled aircraft is left that the first bullet has not released)
        settled = true;
        // spec: S-0501 claim 26 (a settled aircraft not in sleep is told to queue sleep; r154: one in the attack of a retaliation included, claim 52, quirks.aircraft_pad_sleep_ends_retaliation)
        if o.beh.current != Some(Behaviour::Sleep) {
            queue(w, x, Behaviour::Sleep);
            continue;
        }
        let a = max_ammo(w, x);
        // spec: S-0501 claim 28 and claim 26 (r154: with a target at or above ⌊A / 2⌋ the reload is confirmed without a round, and that step neither repairs nor releases it, full ammunition included; T3, target held; quirks.aircraft_half_reload_with_target)
        if w.rules().quirks.aircraft_half_reload_with_target
            && o.target.is_some()
            && o.combat.ammo >= a / 2
        {
            continue;
        }
        if !full_ammo(w, x) {
            if let Some(ob) = w.obj_mut(x) {
                // spec: S-0501 claim 26 (one round per service step)
                ob.combat.ammo += 1;
            }
            continue;
        }
        // spec: S-0501 claim 27 (a repair step: F11 cost, the repair step; released when full or when the house cannot pay)
        match crate::combat::heal::repair_step(w, x) {
            Some(crate::combat::heal::StepOutcome::MoreNeeded) => {}
            _ => release(w),
        }
    }
    if let Some(a) = st_mut(w, k) {
        // spec: S-0501 claim 26 (r154: when no aircraft was settled, or the first bullet released every settled one, servicing ends; otherwise every P ticks)
        a.service = settled.then(|| t.wrapping_add(p.max(1)));
    }
}

/// The run-away notice of pad `k` that starts selling (claim 24, r110,
/// Q-2437; S-0314 claim 22): to every aircraft in contact with it,
/// approaching, docking or on the pad, in contact-place order.
pub fn sale_started(w: &mut World, k: ObjectId) {
    let mut list: Vec<(u32, ObjectId)> = w
        .objects
        .values()
        .filter(|o| o.contact == Some(k) && o.kind == Kind::Aircraft)
        .map(|o| {
            (
                o.econ
                    .exit
                    .filter(|e| e.factory == k)
                    .map_or(u32::MAX, |e| e.place),
                o.id,
            )
        })
        .collect();
    // spec: S-0501 claim 24 (r120, Q-2508: the pad sends its notices in contact-place order, from place 0 up, S-0404 claim 15, each handled in full, its landing spot given and its contact ended, before the next is sent; ending a contact moves no other contact to another place; the order of granting counts only through the places taken; F8 takes no draws and reads no other aircraft's destination, so no spot depends on those given before it). A contact with no place of this pad, and equal places, follow in object order, which keeps the sort total
    list.sort_unstable();
    for (_, x) in list {
        if is_aircraft(w, x) {
            run_away(w, x);
        }
    }
}

/// The run-away notice to aircraft `x` (claim 24, r110).
fn run_away(w: &mut World, x: ObjectId) {
    // spec: S-0501 claim 24 (r110, Q-2437: the notice ends the contact)
    if let Some(o) = w.obj_mut(x) {
        o.contact = None;
        o.econ.exit = None;
    }
    // spec: S-0501 claim 24 (r110: it gives the aircraft a landing spot near its house's buildings, Formula F8, as its destination, which makes one that is landing or on the ground take off, claim 8, at its next movement update)
    let s = landing_spot(w, x);
    set_dest(w, x, s.map(Target::Cell));
    // spec: S-0501 claim 24 (r110: it then has the aircraft queue move, S-0215 claim 7 deciding the slot) and claim 34 (a request that claim ignores is ignored entirely)
    queue(w, x, Behaviour::Move);
    let cur = w.obj(x).and_then(|o| o.beh.current);
    match cur {
        Some(Behaviour::Enter) => {
            // spec: S-0501 claim 24 (r110: in enter it queues guard after that, which replaces the move)
            queue(w, x, Behaviour::Guard);
        }
        Some(Behaviour::Sleep) if !queue_ignored(w, x, Behaviour::Guard) => {
            // spec: S-0501 claim 24 (r110: in sleep that guard comes with an immediate start)
            let (t, s) = (w.tick, stale(w));
            if let Some(o) = w.obj_mut(x) {
                o.beh.queue_and_start_at_once(Behaviour::Guard, t, s);
            }
        }
        // spec: S-0501 claim 24 (r110: in any other current behaviour, such as the guard of an aircraft released on its pad, the move stays queued; the notice makes no fall-back itself)
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Losing a target that leaves the map (claims 32 and 37)

// Claim 37's draw and claim 32's loss are `combat::held`'s, which S-0309
// claim 27 extends to every holder.

// ---------------------------------------------------------------------------
// Falling back to rest (claim 18, table T2)

/// The early exits of table T2 (its first three rows): whether one applied,
/// so that S-0215 claim 13's common effects do not occur.
pub fn fall_back_early(w: &mut World, id: ObjectId) -> bool {
    if !is_aircraft(w, id) {
        return false;
    }
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    // spec: S-0501 T2 row 1 (an overridden behaviour is restored; a restored patrol starts from its first stage)
    if o.beh.suspended.is_some() {
        if let Some(ob) = w.obj_mut(id)
            && let Some((tg, d)) = ob.beh.revert()
        {
            crate::spawn::set_own_target(ob, tg);
            ob.dest = d;
            if ob.beh.current == Some(Behaviour::Patrol) {
                ob.beh.progress = 0;
            }
        }
        // spec: S-0301 claim 5, Moment of a retarget's let-go (revision 67, Q-2246: at the change of the firer's target)
        crate::emitters::retarget_let_go(w, id);
        crate::combat::held::note_revert(w, id);
        return true;
    }
    // spec: S-0501 T2 row 2 and S-0519 claim 19 (retreat, the paradrop and spy-plane behaviours, not a strike aircraft: unchanged; a strike aircraft in retreat is not exempt from falling back)
    if !crate::strike::linked(w, id)
        && matches!(
            o.beh.current,
            Some(
                Behaviour::Retreat
                    | Behaviour::ParadropApproach
                    | Behaviour::ParadropOverfly
                    | Behaviour::SpyPlaneApproach
                    | Behaviour::SpyPlaneOverfly
            )
        )
    {
        return true;
    }
    // spec: S-0501 T2 row 3 (a queued paradrop or spy-plane approach starts now)
    if matches!(
        o.beh.queued,
        Some(Behaviour::ParadropApproach | Behaviour::SpyPlaneApproach)
    ) {
        let (t, s) = (w.tick, stale(w));
        if let Some(ob) = w.obj_mut(id) {
            ob.beh.start_next(t, s);
        }
        return true;
    }
    false
}

/// Falls back to rest through the full S-0215 claim 13 path.
fn fall_back(w: &mut World, id: ObjectId) {
    crate::process::fall_back(w, id, false);
}

/// The rows of table T2 after the common effects (claim 18).
pub fn fall_back_rows(w: &mut World, id: ObjectId) {
    if !is_aircraft(w, id) {
        return;
    }
    let Some(o) = w.obj(id).cloned() else { return };
    let Some(t) = ty(w, id) else { return };
    let a = st(w, id);
    let carrying = passengers(w, id) > 0;
    let armed = armed(w, id);
    let ammo = ammo_left(w, id);
    let ground = on_ground(w, id);
    // spec: S-0501 claim 18, Missile types (r154: an object of a missile type, S-0506 Terms, the missile-spawn flag, takes the rows for an aircraft on the ground whatever its height; the after-the-row entries read its height as for any aircraft) and T2's note
    // spec: S-0501 claim 18, Missile types (r172, Q-2821: the type's missile-spawn flag alone decides, not whether it is one of S-0506's rocket types nor whether a carrier spawned it, so a flagged type that is not a rocket type takes these rows in flight too, placed by a map, built at a pad or a carrier's slot object; its dock types and pad-bound field play no part in the row) and claim 35 (r172: a flagged slot type's run end falls back by these rows)
    let ground_rows = ground || crate::spawn::missile_type(w, id);
    // spec: S-0501 T2 (guard; area guard for an armed aircraft of a computer-controlled house outside a team; our engine has no teams, claim 18)
    let resting = if armed && !human(w, id) {
        Behaviour::AreaGuard
    } else {
        Behaviour::Guard
    };
    // The row's next behaviour, queued after the after-the-row entries.
    let chosen = if ground_rows {
        if a.visiting && carrying {
            // spec: S-0501 T2 (on the ground, visiting, with passengers: unload)
            Some(Behaviour::Unload)
        } else if a.visiting {
            // spec: S-0501 T2 (on the ground, visiting, no passengers, not in a team: forgets target and destination; retreat) and r121 (the row in a team keeps target and destination and gives guard; our engine has no teams, claim 18, so it never applies)
            forget(w, id);
            Some(Behaviour::Retreat)
        } else {
            // spec: S-0501 claim 18 and T2 (on the ground otherwise: forgets its target and destination; guard, or area guard for a computer-controlled house's armed aircraft)
            forget(w, id);
            Some(resting)
        }
    } else if carrying {
        // spec: S-0501 T2 (in flight with passengers: the landing spot; visiting: unload, else move)
        let s = landing_spot(w, id);
        set_dest(w, id, s.map(Target::Cell));
        Some(if a.visiting {
            Behaviour::Unload
        } else {
            Behaviour::Move
        })
    } else if armed && a.visiting && !ammo {
        // spec: S-0501 T2 (in flight, armed, visiting, no ammunition: forgets target and destination; retreat)
        forget(w, id);
        Some(Behaviour::Retreat)
    } else if armed && a.visiting {
        // spec: S-0501 T2 (in flight, armed, visiting, ammunition left, not in a team: hunt)
        Some(Behaviour::Hunt)
    } else if armed
        && (o.beh.queued == Some(Behaviour::Attack)
            || (o.beh.current == Some(Behaviour::Attack) && ammo && o.target.is_some()))
    {
        // spec: S-0501 T2 (in flight, armed, attack queued, or current with ammunition and a target: attack)
        Some(Behaviour::Attack)
    } else if armed
        && (!in_the_air(w, id)
            || (o.beh.current == Some(Behaviour::Enter) && o.dest.is_some())
            || t.docks.is_empty())
    {
        // spec: S-0501 T2 and claim 45 (in flight, armed: the in-the-air answer is no, already flying to its enter destination, or no dock types: the destination kept; guard, area guard for a computer-controlled house) and claims 24 and 46 (r110, Q-2437: the row of an aircraft already flying to its enter destination needs that destination, not a contact: in enter's docking stage without a contact it keeps the pad, or the landing spot the run-away notice gave, and gives guard) and claim 29 (r167, Q-2802: that row needs enter current; an enter only queued, as in sticky, does not make it apply)
        Some(resting)
    } else if armed {
        // spec: S-0501 claim 18 and T2 (r176, Q-2867: the in-flight pad row, the rows after the guard row for an armed aircraft in flight: claim 21's search, then the moment with no destination whatever the current behaviour, guard, area guard, sticky, move, an attack the attack row does not keep and enter with no destination alike, then the contact request to the pad found) and claim 29 (r167, Q-2802: for an aircraft in sticky that holds a pad's contact with enter queued at cruise height too; it already holds a contact then, so claim 24's contact rule asks nothing and enter stays queued)
        let found = seek_pad(w, id);
        if let Some(k) = found {
            // spec: S-0501 T2 (r176: pad found, contact granted: destination none, then the pad; enter) and claim 18 (r176: the pad given right after the moment is handed to its flight as any destination is, claim 8, and replaces the flight point the moment gave, but does not give back a landing the moment undid)
            set_dest(w, id, Some(Target::Object(k)));
            if let Some(s) = st_mut(w, id)
                && s.flying_to == Some(Target::Object(k))
            {
                // spec: S-0501 claim 6 (r132, Q-2576: a landing aircraft given no destination over its flight point's cell keeps that flight point) and claim 29 (r167: the pad it then gets as destination is that flight point's destination object again, so the point is no longer its own)
                s.own_point = false;
            }
            Some(Behaviour::Enter)
        } else if t.bound {
            // spec: S-0501 T2 and claim 46 (r176: in flight, armed, no pad grants contact, pad-bound: destination none, then it crashes at this fall-back; claim 18: its crash begins with the flight point and unfinished flight that the moment left it)
            crash_fall_back(w, id);
            return;
        } else {
            // spec: S-0501 T2 (r176: in flight, armed, no pad grants contact, not pad-bound: destination none, then the landing spot of F8; move)
            let s = landing_spot(w, id);
            set_dest(w, id, s.map(Target::Cell));
            Some(Behaviour::Move)
        }
    } else {
        // spec: S-0501 Edge cases (quirks.unarmed_pad_bound_hover off: for an unarmed aircraft of a landable pad-bound type with dock types, falling back in flight with no contact, only this last in-flight row changes: it first looks for a pad, claim 21, and asks it for contact as T2's in-flight pad row does, with that row's moment with no destination, r176, claim 18; granted: the pad as destination and enter; otherwise the row's landing spot and move, with no crash. A type that is not landable is unchanged)
        let pad = if !w.rules().quirks.unarmed_pad_bound_hover
            && t.bound
            && t.landable
            && !t.docks.is_empty()
            && o.contact.is_none()
        {
            seek_pad(w, id)
        } else {
            None
        };
        if let Some(k) = pad {
            set_dest(w, id, Some(Target::Object(k)));
            if let Some(s) = st_mut(w, id)
                && s.flying_to == Some(Target::Object(k))
            {
                // spec: S-0501 claim 6 (r132, Q-2576) and claim 18 (r176: as in the armed row, the pad given after the moment over the landing flight point's cell is that point's destination object again)
                s.own_point = false;
            }
            Some(Behaviour::Enter)
        } else {
            // spec: S-0501 T2 (in flight otherwise: the landing spot; move) and claim 50 (r93, Q-2252: an unarmed aircraft, pad-bound with dock types included, makes no pad search and never crashes for want of a pad: T2's pad rows and its after-the-row search are for armed aircraft only, so it hovers at its F8 spot)
            let s = landing_spot(w, id);
            set_dest(w, id, s.map(Target::Cell));
            Some(Behaviour::Move)
        }
    };
    let mut chosen = chosen;
    // spec: S-0501 T2 and its note (after every row from the fourth on, the visiting rows included: no ammunition, armed, no contact: a new pad search, which needs neither height test, claim 18)
    if armed && !ammo_left(w, id) && w.obj(id).is_some_and(|x| x.contact.is_none()) {
        // spec: S-0501 claim 18 (r93, Q-2251: this search asks the pad for no contact: it only finds a pad that accepts the aircraft, claims 21 and 22) and claim 21 (r176, Q-2866: it records its result, a pad or none) and claim 18 (r176, Q-2867: no moment with no destination here)
        if let Some(k) = pad_search(w, id) {
            // spec: S-0501 claim 18 and T2 (found: the pad as destination, the target forgotten, enter) and claim 24 (r93: that acceptance makes a contact only when enter is already current or queued, destination_taken)
            set_dest(w, id, Some(Target::Object(k)));
            if let Some(ob) = w.obj_mut(id) {
                crate::spawn::clear_own_target(ob);
                ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
            }
            chosen = Some(Behaviour::Enter);
        } else if t.bound {
            // spec: S-0501 claim 18 and T2 (none and pad-bound: the row's next behaviour is not queued and the current behaviour stays, target and destination as the row left them)
            chosen = None;
            if !ground {
                // spec: S-0501 claims 18 and 46 (in flight it crashes; on the ground it does not, claim 9)
                crash_fall_back(w, id);
                return;
            }
        }
    }
    if w.obj(id).is_some_and(|x| x.contact.is_some()) {
        // spec: S-0501 T2 (after the row: an aircraft holding a contact queues enter)
        chosen = Some(Behaviour::Enter);
    }
    if let Some(b) = chosen {
        queue(w, id, b);
    }
    // spec: S-0501 claim 31 (the second start opportunity: the end of an ordinary fall-back)
    start_if_ready(w, id);
}

/// The fall-back crash of a pad-bound aircraft in flight with no pad
/// granting it contact (claim 46): its destruction without a hit, credited
/// to nobody, then S-0312 claim 16's crash start, with none of a killing
/// hit's effects. It then falls as a dead aircraft (claim 9).
fn crash_fall_back(w: &mut World, id: ObjectId) {
    if height(w, id) <= 0 {
        // spec: S-0501 claim 9 (a crash starts only above the ground)
        return;
    }
    let none = crate::combat::damage::Hit {
        amount: 0,
        warhead: None,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: false,
        no_escape: false,
    };
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0501 claim 46 (the crash begins with the aircraft's destruction without a hit: no damage dealt, health set to 0)
        ob.health = 0;
    }
    // spec: S-0501 claims 32 and 46 (destroyed: those targeting it lose it at that moment)
    crate::combat::death::mark_killed(w, id);
    // spec: S-0501 claim 46 and S-0414 claim 21 (credited to nobody: no house's kills, score or experience change; one loss of its own house unless its type is not scored)
    crate::combat::damage::credit(w, id, &none);
    // spec: S-0501 claim 46 (a tag on the aircraft is raised as for other destructions, destroyed-by-anything; low, unmeasured)
    crate::combat::death::destruction_raises(w, id, &none);
    // spec: S-0501 claim 46 and S-0312 claim 16 (the crash-start effects: its passengers are lost, three tumbling draws; no debris, explosion-list pick or bomb detonation, as no hit kills it)
    start_crash_hit(w, id, false, None);
}

// ---------------------------------------------------------------------------
// Processing (claims 12 to 17, 24, 29, 33, 34)

/// An aircraft's behaviour part of its turn (S-0215 claim 2, S-0501
/// claims 31 and 32): no per-tick start; a target in the air is cleared
/// before processing.
pub fn behaviour_turn(w: &mut World, id: ObjectId) {
    if !is_aircraft(w, id) {
        return;
    }
    let t = w.tick;
    ready_mark_after_enter(w, id);
    let Some(o) = w.obj(id).cloned() else { return };
    // spec: S-0501 claim 32 (before processing, a target whose in-the-air answer is yes is cleared; an aircraft in attack whose target is gone, removed and no longer existing (glossary, *removed*), ends its run: the target is cleared here too, for a removal that was not a kill)
    if let Some(Target::Object(x)) = o.target
        && (w.obj(x).is_none() || crate::combat::weapon::in_air(w, x))
        && let Some(ob) = w.obj_mut(id)
    {
        crate::spawn::clear_own_target(ob);
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
    }
    let Some(o) = w.obj(id) else { return };
    // spec: S-0215 claim 2 (alive with health above 0 and the timer expired)
    if o.health <= 0 || !o.beh.due(t) {
        return;
    }
    let d = process(w, id);
    if let Some(ob) = w.obj_mut(id) {
        ob.beh.processed(t, d);
    }
}

// spec: S-0215 claim 4 (no processing rule: a delay of exactly 450 ticks, no draw)
const NO_RULE_DELAY: i32 = 450;
// spec: S-0501 claim 24 (while in contact during the approaching stage enter returns 3 ticks, including the processing that recognizes arrival and advances to docking)
const APPROACH_DELAY: i32 = 3;
// spec: S-0501 claim 6 (a destination more than 120 leptons above terrain)
const HOLD_ABOVE: i32 = 120;
// spec: S-0501 T1 (approaching: at a distance of at least 512 leptons the aiming facing is requested toward the destination)
const STEER_FAR: i32 = 512;
// spec: S-0501 claim 42 and Constants (guidance aiming distance: below 256 leptons)
const AIM_NEAR: i32 = 256;
// spec: S-0501 claim 42 and Constants (strafer body re-aim window: at most 768 leptons)
const STRAFER_WINDOW: i32 = 768;
// spec: S-0501 claim 42 and Constants (strafer body re-aim window: at most 8192 facing units)
const STRAFER_TURN: i32 = 8192;
// spec: S-0501 claim 41 and Constants (no-result landing cell: map cell coordinates (0, 0))
const PLACEHOLDER: CellXY = CellXY::new(0, 0);
// spec: S-0501 T1 (approaching: below 16 leptons the destination is cleared)
const STEER_NEAR: i32 = 16;
// spec: S-0501 claim 7 (r146, Q-2676: the map's origin point, the point whose three coordinates are 0)
const ORIGIN: Pos = Pos { x: 0, y: 0, z: 0 };
// spec: S-0501 claim 13 and T1 (five strafing passes)
const LAST_PASS: u8 = 5;

fn standard(w: &mut World, id: ObjectId) -> i32 {
    crate::process::standard_delay(w, id)
}

fn process(w: &mut World, id: ObjectId) -> i32 {
    let cur = w.obj(id).and_then(|o| o.beh.current);
    match cur {
        Some(Behaviour::Attack) => attack(w, id),
        // spec: S-0215 claim 5 (sticky is processed as guard)
        Some(Behaviour::Guard | Behaviour::Sticky) => guard(w, id),
        Some(Behaviour::AreaGuard) => area_guard(w, id),
        Some(Behaviour::Hunt) => hunt(w, id),
        Some(Behaviour::Move) => move_(w, id),
        Some(Behaviour::Enter) => enter(w, id),
        Some(Behaviour::Retreat) => retreat(w, id),
        Some(Behaviour::SpyPlaneApproach) => spy_approach(w, id),
        Some(Behaviour::SpyPlaneOverfly) => spy_overfly(w, id),
        // spec: S-0522 claim 6 (paradrop approach)
        Some(Behaviour::ParadropApproach) => crate::paradrop::approach(w, id),
        // spec: S-0522 claim 7 (paradrop overfly)
        Some(Behaviour::ParadropOverfly) => crate::paradrop::overfly(w, id),
        // spec: S-0507 claim 13 (an air transport in unload)
        Some(Behaviour::Unload) => crate::transport::process_air_unload(w, id),
        // spec: S-0215 claim 28, claim 5 and table T1 (an aircraft's capture and sabotage are claim 28's table; its fall-back is claim 18 here)
        Some(Behaviour::Capture | Behaviour::Sabotage) => {
            crate::process::process_capture(w, id, false)
        }
        // spec: S-0215 claim 28 and table T1 (an aircraft's eaten: the same table with its eaten differences; S-0603 claim 3)
        Some(Behaviour::Eaten) => crate::structures::grinder::process_eaten(w, id),
        _ => NO_RULE_DELAY,
    }
}

/// Claim 33's *move* for an aircraft in *guard* (r146, Q-2677), at a
/// movement update after claim 31's first start opportunity, so a *guard*
/// that becomes current there gets its *move* in the same update. It
/// becomes current at claim 31's third point, when the aircraft is ready.
fn guard_move(w: &mut World, id: ObjectId, t: &Ty) {
    let Some(o) = w.obj(id) else { return };
    // spec: S-0501 claim 33 (r146, Q-2677: a living aircraft that is not pad-bound and not crashing, claim 9, in guard with a destination and nothing queued at a movement update: that update queues move for it, at any height, on the ground included; the test comes after claim 31's first point in that update)
    // spec: S-0501 claim 33 (r150, Q-2726: the test is for guard itself as the current behaviour; an aircraft in sticky, processed as guard by S-0215 claim 5, gets no move at a movement update and keeps sticky with its destination; any queued behaviour stops the test, a guard that a row of T2 queued included)
    let applies = o.health > 0
        && !t.bound
        && o.beh.current == Some(Behaviour::Guard)
        && o.dest.is_some()
        && o.beh.queued.is_none()
        && !crashing(w, id);
    if applies {
        // spec: S-0501 claim 34 (a request that claim ignores is ignored entirely)
        queue(w, id, Behaviour::Move);
    }
}

/// *Move*'s processing gives the aircraft its destination's flight point
/// (claim 40): the locomotor accepts it as at a new destination, so an
/// aircraft on the ground by its height, or landing, takes off (claim 8).
fn regive(w: &mut World, id: ObjectId) {
    let Some(t) = ty(w, id) else { return };
    let dest = flight_target(w, id);
    if dest.is_some() {
        give_point(w, id, &t, dest);
    }
}

/// *Move* (claim 33).
fn move_(w: &mut World, id: ObjectId) -> i32 {
    let lost = w
        .obj(id)
        .and_then(|o| o.dest)
        .is_some_and(|d| matches!(d, Target::Object(k) if w.obj(k).is_none()));
    if lost {
        // spec: S-0501 claim 32 (an approaching aircraft whose destination is lost falls back to rest, looking for another pad); claim 53 clears such a destination when the object leaves play, so this guards only an object taken out of the world by a path that skips that moment
        set_dest(w, id, None);
        fall_back(w, id);
        return standard(w, id);
    }
    let (expired, start) = w
        .obj(id)
        .map_or((false, 0), |o| (o.beh.expired, o.beh.start_tick));
    // spec: S-0501 claim 40 (the selection runs at the first processing of every such move, also one started inside another behaviour's processing, such as the fall-back at a run's end, whose start timer that processing's delay replaced, S-0215 claims 2 and 3)
    let first = expired || st(w, id).first_moved != Some(start);
    let s = st(w, id);
    if s.lost && w.obj(id).is_some_and(|o| o.dest.is_none()) && (first || s.regive == Some(start)) {
        // spec: S-0501 claim 53 (r183, Q-3041: in an ordinary move, a loss of the destination object before the move's second processing, the one that gives its flight point once more, makes its next processing fall back to rest, claim 18)
        if let Some(a) = st_mut(w, id) {
            a.regive = None;
        }
        fall_back(w, id);
        return standard(w, id);
    }
    if first && let Some(a) = st_mut(w, id) {
        a.first_moved = Some(start);
    }
    if let Some(o) = w.obj(id)
        && first
        && let Some(orig) = o.dest
    {
        // spec: S-0501 claim 40 (the first processing of every ordinary move applies the selection to its destination, also one the aircraft already flies to; its own earlier choice does not count against it: retained, or a replacement cell with its draws)
        let sel = move_destination(w, id, orig);
        if sel != orig {
            set_dest(w, id, Some(sel));
        }
        // spec: S-0501 claim 40 (whether kept or replaced, the selected destination is given to the aircraft at that processing, as any destination is: a destination it gets in claim 8's sense, so one then on the ground by its height, or landing, takes off; claim 48 (c), Q-1942)
        regive(w, id);
        if let Some(a) = st_mut(w, id) {
            a.regive = Some(start);
        }
    } else if !first && st(w, id).regive == Some(start) {
        // spec: S-0501 claim 40 (the move's next processing gives its flight point to the flight once more, a destination it gets in claim 8's sense)
        if let Some(a) = st_mut(w, id) {
            a.regive = None;
        }
        regive(w, id);
    }
    let has_dest = w.obj(id).is_some_and(|o| o.dest.is_some());
    // spec: S-0501 claim 33 (when its locomotor has stopped there it falls back to rest)
    if !has_dest && stopped(w, id) {
        fall_back(w, id);
    }
    // spec: S-0501 claim 33 (move uses the standard delay)
    standard(w, id)
}

/// *Hunt* (claim 33).
fn hunt(w: &mut World, id: ObjectId) -> i32 {
    if !armed(w, id) || !ammo_left(w, id) {
        fall_back(w, id);
        // spec: S-0501 claim 33 (no ammunition: the standard delay)
        return standard(w, id);
    }
    crate::combat::acquire::search(
        w,
        id,
        crate::combat::acquire::SearchKind::MapWide,
        crate::combat::acquire::SearchContext::Hunt,
    );
    if w.obj(id).is_some_and(|o| o.target.is_some()) {
        // spec: S-0501 claim 33 (a target: attack queued without an immediate start)
        queue(w, id, Behaviour::Attack);
    } else {
        fall_back(w, id);
    }
    // spec: S-0501 claim 33 (hunt returns a delay of 1 tick after finding a target or failing to)
    1
}

/// *Guard* (claims 28, 29 and 33).
fn guard(w: &mut World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id).cloned() else {
        return NO_RULE_DELAY;
    };
    let Some(t) = ty(w, id) else {
        return NO_RULE_DELAY;
    };
    let armed = armed(w, id);
    let h = height(w, id);
    let ground = on_ground(w, id);
    let a_max = max_ammo(w, id);
    // spec: S-0501 claim 28 (on the pad with a target: stays below ⌊A / 2⌋, takes off into attack at it)
    if ground && o.target.is_some() && armed {
        let threshold = if a_max > 0 { a_max / 2 } else { 0 };
        let enough = o.combat.ammo < 0 || o.combat.ammo >= threshold;
        if enough && ammo_left(w, id) {
            queue(w, id, Behaviour::Attack);
            start_if_ready(w, id);
            return 1;
        }
    }
    // spec: S-0501 claim 29 (an armed aircraft below cruise height, not full and without contact: a pad search; enter, target forgotten; r150: on the ground too, where it takes off for the pad, claim 8) and claim 48 (d) (guard compares the height with the cruise height for equality only: every other height, above it as well as below it, takes the same rules)
    if armed
        && h != t.cruise
        && !full_ammo(w, id)
        && o.contact.is_none()
        // spec: S-0501 claim 21 (r176, Q-2866: claim 29's search below cruise height records its result, a pad or none)
        && let Some(k) = pad_search(w, id)
    {
        // spec: S-0501 claim 29 (r167, Q-2802: a pad found below cruise height: the aircraft queues enter first and is given the pad as destination after that, so claim 24's contact rule applies: it asks the pad to be loaded and the pad, which has a free contact place since the search found it, claim 21, takes it into contact at that processing, destination_taken)
        let before = destination(w, id);
        queue(w, id, Behaviour::Enter);
        set_dest(w, id, Some(Target::Object(k)));
        if let Some(ob) = w.obj_mut(id) {
            crate::spawn::clear_own_target(ob);
            ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
        }
        // spec: S-0501 claim 29 (r167: the pad then asks whether it has settled, with the destination it held before the pad, claim 24; one that has settled, as one standing landed has, is made to start enter at once although it is not ready to change, so sticky ends; one that has not keeps sticky with enter queued, the contact and the pad as destination, and flies to the pad)
        let confirmed = w.obj(id).is_some_and(|x| x.contact == Some(k))
            && w.obj(k).is_some_and(|b| !b.econ.building.switched_off);
        if confirmed && settled(w, id, &t, before) && accepts(w, k, id) {
            start_enter_at_once(w, id);
        }
        return standard(w, id);
    }
    // spec: S-0501 claims 29 and 48 (d) (the cruise-height outcomes at exactly the cruise height only)
    // spec: S-0501 claim 29 (r110, Q-2437: at exactly cruise height the test is team membership, not the destination; our engine has no teams, so every aircraft takes the outcomes outside a team)
    if !ground && h == t.cruise {
        if armed {
            // spec: S-0501 claim 29 (r110: outside a team, an armed aircraft falls back to rest, claim 18, whatever its destination) and claim 31 (r110, Q-2438: this processing sets the ready mark)
            // spec: S-0501 claim 29 (r167, Q-2802: a contact held at cruise height: each later processing at exactly that height of an armed aircraft with dock types that holds a pad's contact with enter queued is that fall-back again; with sticky current T2's row for an aircraft already flying to its enter destination does not apply, so the in-air answer no keeps the pad by T2's guard row and otherwise the in-flight pad row applies again, fall_back_rows)
            set_ready_mark(w, id);
            fall_back(w, id);
        } else {
            // spec: S-0501 claim 29 (unarmed, outside a team: its own cell and move, so it lands, whatever its destination) and claim 31 (r110: this processing leaves the ready mark as it is)
            // spec: S-0501 claim 29 (r167, Q-2802: being given its flight point again changes nothing in its speed fraction, whatever its slowdown distance and speed, give_point; with a slowdown distance of 0 it never lands, and with V below 20 it lands unless claim 5's 1-lepton hover or zero step holds it at cruise height; a type that is not landable that claim 5 holds at full speed keeps its cruise height by claim 48 (b)'s third case, tick_flight)
            let c = o.pos.cell();
            set_dest(w, id, Some(Target::Cell(c)));
            queue(w, id, Behaviour::Move);
        }
        // spec: S-0501 claim 29 (r150, Q-2726: outside a team, a processing at exactly cruise height ends with a delay of 1 tick)
        return 1;
    }
    // spec: S-0501 claim 29 (r146: apart from these processings, claim 33 queues move for an aircraft in guard that is not pad-bound, with a destination and nothing queued, at any height: at a movement update, [`guard_move`], not at this processing)
    // spec: S-0501 claim 29 (45 ticks for an unarmed aircraft and an aircraft on the ground with no contact; otherwise the standard delay)
    if !armed || (ground && o.contact.is_none()) {
        45
    } else {
        standard(w, id)
    }
}

/// *Area guard* of an aircraft (claim 45; S-0215 claim 26 with *k* = 2).
fn area_guard(w: &mut World, id: ObjectId) -> i32 {
    let Some(t) = ty(w, id) else {
        return NO_RULE_DELAY;
    };
    // spec: S-0501 claim 45 (each processing compares the aircraft's height, Terms, taken at its start, with its cruise height, not with its held height; Q-1837)
    let h = height(w, id);
    if h == t.cruise {
        // spec: S-0501 claim 45 (at exactly its cruise height each processing is the fall-back to rest instead of S-0215 claim 26's rows; a delay of 1 tick)
        fall_back(w, id);
        return 1; // spec: S-0501 claim 45 (a delay of 1 tick at its cruise height)
    }
    let spent = armed(w, id) && !ammo_left(w, id) && w.obj(id).is_some_and(|o| o.contact.is_none());
    if spent {
        // spec: S-0501 claim 45 (at any other height, above or below, an armed aircraft with its ammunition spent and no contact makes the same fall-back first)
        fall_back(w, id);
    }
    let Some(o) = w.obj(id) else {
        return NO_RULE_DELAY;
    };
    if o.target.is_some() {
        // spec: S-0501 claim 45 and its Changes for S-0215 claim 26 (a held target: attack queued; the next processing 1 tick later)
        queue(w, id, Behaviour::Attack);
        return 1; // spec: S-0501 claim 45 (1 tick with a held target)
    }
    // spec: S-0501 claim 45 and S-0215 claim 26 (away from its cruise height, above or below, and with no target: claim 26's rows and table, its delay with k = 2 for aircraft)
    crate::process::process_area_guard(w, id)
}

/// *Enter* toward a pad (claim 24).
fn enter(w: &mut World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id).cloned() else {
        return NO_RULE_DELAY;
    };
    // spec: S-0501 claim 24 (r103, Q-2366: one rule comes before every other rule of this claim, at every processing of enter, its first included)
    if bound_away(w, id) {
        return 1; // spec: S-0501 claim 24 (r103: a processing that finds no contact returns 1 tick, and so does the pad-bound rule that queues move)
    }
    let Some(t) = ty(w, id) else {
        return NO_RULE_DELAY;
    };
    if !on_ground(w, id)
        && let Some(Target::Object(b)) = destination(w, id)
        && w.obj(b).is_some_and(|x| x.kind == Kind::Building)
        && st(w, id).heading_pad != Some(b)
    {
        // spec: S-0501 claim 24 (r176, Q-2866: at every processing of enter, its first included, an aircraft in flight, pad-bound or not, whose destination is a building other than the pad it was last heading for records that building as that pad, claim 21; this record changes nothing else in the processing; it comes after the pad-bound rule on the ground and before the rule for an aircraft in flight with no destination)
        record_heading(w, id, Some(b));
    }
    // spec: S-0501 claim 24 (r110, Q-2437: a second rule comes next, at every processing, its first included, and in either stage: an aircraft in flight with no destination asks its contact to be loaded; unless that request is confirmed it falls back to rest at once, claim 18, whatever is queued, and the processing returns 1 tick and does nothing else; an aircraft without a contact has nobody to confirm it, so this is what any aircraft in flight whose destination is gone does) and claim 31 (that fall-back leaves the ready mark as it is)
    let in_flight = !on_ground(w, id);
    let asked = in_flight && destination(w, id).is_none();
    if asked && !load_request(w, id, &t) {
        fall_back(w, id);
        return 1; // spec: S-0501 claim 24 (r110: the second rule's fall-back returns 1 tick)
    }
    if asked && restarted(w, id, o.beh.start_tick) {
        return 1; // spec: S-0501 claim 24 (entry begins with a 1-tick delay)
    }
    // A confirmed request goes on by the rules below in this processing,
    // which is still that of the enter it found current: an entry started
    // again by the pad's answer has its first processing at the next one.
    // spec: S-0501 claim 24 (entry begins with a 1-tick delay)
    if st(w, id).enter_of != Some(o.beh.start_tick) {
        if let Some(a) = st_mut(w, id) {
            a.enter_of = Some(o.beh.start_tick);
            a.enter_docking = false;
        }
        // spec: S-0501 claim 31 (r110, Q-2438: the processing of enter that starts entry sets the ready mark)
        set_ready_mark(w, id);
        return 1; // spec: S-0501 claim 24 (entry begins with a 1-tick delay)
    }
    let pad = o
        .contact
        .filter(|&k| w.obj(k).is_some_and(|b| b.kind == Kind::Building));
    let Some(k) = pad else {
        // spec: S-0501 claim 31 (r110, Q-2438: every processing of enter without a contact that the two rules let through sets the ready mark: the wait, the fall-back, the move on to the docking stage, and every processing of the docking stage)
        set_ready_mark(w, id);
        if st(w, id).enter_docking {
            // spec: S-0501 claim 24 (r103: in the docking stage without a contact, its contact having ended after it reached that stage, or its pad's sale having replaced the destination, r110)
            return docking_without_contact(w, id);
        }
        return without_contact(w, id);
    };
    let docking = st(w, id).enter_docking;
    // spec: S-0501 claim 24 (r110, Q-2436: a processing asks the pad it is in contact with to be loaded in two cases only: in flight with no destination, the second rule above, and for a pad-bound aircraft in flight in the approaching stage)
    if !asked && !docking && t.bound && in_flight && !load_request(w, id, &t) {
        // spec: S-0501 claim 24 (r99: a refusal by which the pad ends the contact during the approach while it stays the aircraft's destination leaves it to the processing without a contact; our engine meets the refusal in this processing, so that processing follows at once) and Open question 19 (the refusal itself stays open: our engine meets it at this request) and claim 31 (r110: that processing sets the ready mark)
        set_ready_mark(w, id);
        return without_contact(w, id);
    }
    if !asked && !docking && t.bound && in_flight && restarted(w, id, o.beh.start_tick) {
        return 1; // spec: S-0501 claim 24 (entry begins with a 1-tick delay)
    }
    let landing = st(w, id).landing;
    let ground = on_ground(w, id);
    if docking {
        // spec: S-0501 claim 31 (r110, Q-2438: every processing of the docking stage that the two rules let through sets the ready mark, the one that reports entry included)
        set_ready_mark(w, id);
        if landing {
            // spec: S-0501 claim 24 (from the next processing until touchdown, each processing, 1 tick after the one before, moves it horizontally toward its docking position)
            docking_step(w, id, Target::Object(k));
            return 1; // spec: S-0501 claim 24 (the docking and entered-report stage returns 1 tick)
        }
        if ground {
            let s = st(w, id);
            // An aircraft on the ground by its height that is not at rest,
            // not released and not taking off: an untested touchdown kept
            // its destination, with either position of the drop switch.
            if s.sliding || (!s.ground && !s.released && !s.taking_off) {
                // spec: S-0501 claim 24 (r132, Not pad-bound: after a touchdown that kept the pad as destination its next processing finds it not landing and reports entry; its movement follows claim 7's after an untested touchdown that kept the destination) and r134 (Q-2649: neither the report nor the pad's answer to it requests a facing, after a touchdown as in flight; while it slides it has not settled, so the pad's service steps skip it unless claim 26 releases it first)
                report_entered(w, id, k);
            } else {
                // spec: S-0501 claim 24 (after touchdown it reports that it has entered; an aircraft already on the ground and not landing when it reaches this stage is not moved; r110, Q-2436: neither the report nor the pad tests where it touched down)
                dock(w, id, k);
            }
            return 1; // spec: S-0501 claim 24 (the docking and entered-report stage returns 1 tick)
        }
        // spec: S-0501 claim 24 (r120, Q-2506: the docking stage's processing tests whether the aircraft is landing, not its height: one in contact that finds it in flight and not landing, after a refused touchdown, reports entry at once, in flight, as after a touchdown; it is neither kept in the docking stage nor sent back to the approaching stage, and that processing does not move it)
        report_entered(w, id, k);
        return 1; // spec: S-0501 claim 24 (r120: the processing returns 1 tick and leaves it ready to change)
    }
    // spec: S-0501 claim 24 (r110, Q-2436: in the approaching stage, a processing in contact that finds the aircraft landing moves it on to the docking stage, and so does one that finds a pad-bound aircraft on the ground by its height and not landing; neither tests the position) and Edge cases (quirks.aircraft_grounded_enter_stall on: one that is not pad-bound, on the ground and not landing is never moved on; off: it is moved on as a pad-bound one is)
    let stall = w.rules().quirks.aircraft_grounded_enter_stall;
    if landing || (ground && (t.bound || !stall)) {
        if let Some(a) = st_mut(w, id) {
            a.enter_docking = true;
            // spec: S-0501 claim 31 (r110, Q-2438: the ready mark is cleared only when a processing in contact moves the aircraft on to the docking stage, claim 24, and by move for a type that carries vehicles, a seam: our engine has no such move)
            a.mark_cleared = true;
        }
        // spec: S-0501 claim 24 (the processing that moves it on to the docking stage returns 3 ticks)
        return APPROACH_DELAY;
    }
    // spec: S-0501 claim 24 (r110, Q-2436: a processing in contact that finds neither returns 3 ticks and changes nothing else: it tests neither the destination nor the position, and leaves a cell or no destination as it is, apart from the pad's answer) and claim 31 (it leaves the ready mark as it is)
    APPROACH_DELAY
}

/// Whether the pad's answer to a load request in this processing of
/// *enter*, which started at tick `was`, started entry again (claim 24,
/// r134): a behaviour queued was replaced by *enter* and started at once.
/// If so, entry's beginning is made at once: the aircraft is put in the
/// approaching stage and left ready to change, and no rule of the stage it
/// was in follows in that processing.
fn restarted(w: &mut World, id: ObjectId, was: i32) -> bool {
    let Some(s) = w.obj(id).map(|x| x.beh.start_tick) else {
        return false;
    };
    if s == was {
        return false;
    }
    // spec: S-0501 claim 24 (r132, Q-2579; r134, Q-2650: a queued behaviour is replaced by enter and lost, and entry starts again within that processing, in the approaching stage as in the docking stage: it makes entry's beginning, with its 1-tick delay, puts the aircraft in the approaching stage and leaves it ready to change, and no rule of the stage it was in follows in that processing; its next processing, 1 tick later, is the approaching stage's) and claim 31 (r110, Q-2438: the processing of enter that starts entry sets the ready mark)
    if let Some(a) = st_mut(w, id) {
        a.enter_of = Some(s);
        a.enter_docking = false;
    }
    set_ready_mark(w, id);
    true
}

/// The destination of aircraft `id` as claim 24 reads it (r110, Q-2437): a
/// destination that is gone, its pad destroyed or removed, leaves none.
fn destination(w: &World, id: ObjectId) -> Option<Target> {
    // spec: S-0501 claim 24 (r110, Q-2437: a destination that is gone, its pad destroyed or removed, leaves the aircraft none)
    w.obj(id)?
        .dest
        .filter(|d| !matches!(*d, Target::Object(k) if w.obj(k).is_none()))
}

/// Claim 24's load request (r110, Q-2436): aircraft `id` asks the pad it is
/// in contact with to be loaded. Returns whether the pad confirmed it; a
/// refusal ends the contact. A confirmed request gives a settled aircraft
/// that the pad accepts the pad as destination, *enter* queued and its
/// queued behaviour started at once.
fn load_request(w: &mut World, id: ObjectId, t: &Ty) -> bool {
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    let pad = o
        .contact
        .filter(|&k| w.obj(k).is_some_and(|b| b.kind == Kind::Building));
    // spec: S-0501 claim 24 (r110: a pad that is switched on confirms the request, and a switched-off one refuses it; an aircraft without a contact has nobody to confirm it)
    let Some(k) = pad.filter(|&k| w.obj(k).is_some_and(|b| !b.econ.building.switched_off)) else {
        if o.contact.is_some_and(|k| Some(k) == pad)
            && let Some(ob) = w.obj_mut(id)
        {
            // spec: S-0501 claim 24 (a refusal by which the pad ends the contact, claim 32)
            ob.contact = None;
            ob.econ.exit = None;
        }
        return false;
    };
    let dest = destination(w, id);
    if settled(w, id, t, dest) && accepts(w, k, id) {
        set_dest(w, id, Some(Target::Object(k)));
        start_enter_at_once(w, id);
    }
    true
}

/// Claim 24's settled question, which a pad asks before it confirms a
/// request to be loaded, with `dest` as the aircraft's destination.
fn settled(w: &World, id: ObjectId, t: &Ty, dest: Option<Target>) -> bool {
    let Some(o) = w.obj(id) else { return false };
    // spec: S-0501 claim 24 (r110: before it confirms, the pad asks whether the aircraft has settled: its locomotor reports stopped, claim 40, it has no destination, or it is pad-bound and its destination is the cell it stands in)
    stopped(w, id) || dest.is_none() || (t.bound && dest == Some(Target::Cell(o.pos.cell())))
}

/// The pad's answer to a settled aircraft (claim 24): it has the aircraft
/// queue *enter* and start its queued behaviour at once.
fn start_enter_at_once(w: &mut World, id: ObjectId) {
    // spec: S-0501 claim 24 (r110: when it has, and the pad accepts it, claim 22, the pad gives itself as destination and has it queue enter and start its queued behaviour at once, whether or not it is ready; with enter current and nothing queued that changes only the destination, S-0215 claim 7; a queued behaviour is replaced and lost, and entry starts again; an aircraft that has not settled keeps its destination: the answer changes nothing for it)
    // spec: S-0501 claim 34 (a request that claim ignores is ignored entirely)
    if !queue_ignored(w, id, Behaviour::Enter) {
        let (tick, s) = (w.tick, stale(w));
        if let Some(ob) = w.obj_mut(id) {
            ob.beh.queue_and_start_at_once(Behaviour::Enter, tick, s);
        }
    }
}

/// Claim 24's rule before every other (r103, Q-2366): a pad-bound aircraft
/// on the ground by its height whose destination is not the building in
/// its own cell leaves its contact, records the destination as the pad it
/// was last heading for and queues *move* with an immediate start. Returns
/// whether the rule applied.
fn bound_away(w: &mut World, id: ObjectId) -> bool {
    let Some(t) = ty(w, id) else { return false };
    let Some(d) = destination(w, id) else {
        return false;
    };
    let building = |k: ObjectId| w.obj(k).is_some_and(|b| b.kind == Kind::Building);
    // spec: S-0501 claim 24 (r103: a pad-bound aircraft on the ground by its height, Terms, with a destination other than the building in its own cell: its destination pad while it stands anywhere else)
    let own = matches!(d, Target::Object(k) if building(k) && pad_cell(w, k, id));
    if !t.bound || !on_ground(w, id) || own {
        return false;
    }
    let pad = match d {
        Target::Object(k) if building(k) => Some(k),
        _ => None,
    };
    // spec: S-0501 claim 24 (r103: it leaves any contact it holds)
    if let Some(o) = w.obj_mut(id) {
        o.contact = None;
        o.econ.exit = None;
    }
    if let Some(a) = st_mut(w, id) {
        // spec: S-0501 claim 24 (r103: it records that destination as the pad it was last heading for, the pad claim 21 asks first, when it is a building, of its dock types or not; r110, Q-2436: any other destination, such as a cell, is recorded as no pad, and the earlier record is lost)
        a.heading_pad = pad;
    }
    // spec: S-0501 claim 24 (r103: it queues move with an immediate start, S-0215 claim 7, which replaces any queued behaviour) and claim 34 (a request that claim ignores is ignored entirely)
    if !queue_ignored(w, id, Behaviour::Move) {
        let (r, tick, s) = (ready(w, id), w.tick, stale(w));
        if let Some(o) = w.obj_mut(id) {
            o.beh.queue_and_start(Some(Behaviour::Move), r, tick, s);
        }
    }
    true
}

/// The docking stage without a contact (claim 24, r103 and r110, Q-2366
/// and Q-2437): the contact ended after the aircraft reached that stage, or
/// its pad's sale replaced the destination, with *enter* still current.
fn docking_without_contact(w: &mut World, id: ObjectId) -> i32 {
    // spec: S-0501 claim 24 (r103: each of its processings there, 1 tick apart, makes no test of contact or of a queued behaviour)
    if st(w, id).landing {
        if let Some(d) = destination(w, id) {
            // spec: S-0501 claim 24 (r103: while it is landing, the processing moves it as the docking stage moves one in contact, toward a docking position that does not depend on whether the pad has a free contact place; r110, Q-2437: toward a destination that is not a building, such as the run-away notice's landing spot, toward that destination's centre)
            docking_step(w, id, d);
        }
    } else {
        // spec: S-0501 claim 24 (r103: the first processing that finds it no longer landing makes it fall back to rest, claim 18, whatever is queued: after an accepted touchdown, whose own movement update makes no fall-back, claim 7, or after a refused one that kept the destination; r110, Q-2437: it does not forget the destination first, so the fall-back sees the destination as it stands, none after an accepted touchdown, and in flight the destination it still has, for which T2's row of an armed aircraft already flying to its enter destination keeps it and gives guard; r120: a refused touchdown in its flight point's cell has made it forget its destination, so the second rule decides first, enter, and it falls back to rest at once with no destination)
        fall_back(w, id);
    }
    1 // spec: S-0501 claim 24 (r103: the docking stage's processings are 1 tick apart; the docking and entered-report stage returns 1 tick)
}

/// A processing of *enter* after its first that finds the aircraft without
/// a contact (claim 24, r99, r103 and r110, Q-2326, Q-2366 and Q-2437), in
/// flight and on the ground alike, whether or not the pad has a free contact
/// place. It makes no test of the destination.
fn without_contact(w: &mut World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id) else {
        return NO_RULE_DELAY;
    };
    let queued = o.beh.queued.is_some();
    if st(w, id).landing
        && let Some(a) = st_mut(w, id)
    {
        // spec: S-0501 claim 24 (r103, Q-2366: a processing without a contact that finds the aircraft landing also moves it on to the docking stage but does not move the aircraft itself; it then waits or falls back as below; r110, Q-2437: it does so wherever the aircraft lands, with no test of the cell under it or of its destination)
        a.enter_docking = true;
    }
    // spec: S-0501 claim 24 (r103, Q-2366: with a behaviour queued it keeps the destination and enter, asks no pad and waits; that processing leaves it ready to change and in enter it carries no run lock, so the queued behaviour becomes current at the end of that tick, claim 31's third point: the wait lasts that one processing; r110, Q-2437: this processing makes no test of the destination, so a pad being sold, a building not of the dock types or a gone pad is waited with or forgotten as any pad is)
    if !queued {
        // spec: S-0501 claim 24 (r99: with no behaviour queued it forgets the destination and falls back to rest, claim 18)
        set_dest(w, id, None);
        fall_back(w, id);
    }
    1 // spec: S-0501 claim 24 (r99: a processing that finds no contact returns 1 tick, whether it waits or falls back; no standard-delay draw)
}

/// *Retreat* (claim 34).
fn retreat(w: &mut World, id: ObjectId) -> i32 {
    if w.obj(id).is_some_and(|o| o.dest.is_none()) {
        // spec: S-0501 claim 34 (no destination: a cell on its house's arrival edge, S-0522 claim 5, selected as in S-0522 claim 4)
        let c = arrival_edge_cell(w, id, false);
        set_dest(w, id, c.map(Target::Cell));
    }
    3 // spec: S-0501 claim 34 (retreat processing has a delay of 3 ticks)
}

/// *Spy-plane approach* (claim 34).
fn spy_approach(w: &mut World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id).cloned() else {
        return NO_RULE_DELAY;
    };
    let frames = w.rules().air.camera_frames;
    // spec: S-0513 claim 44 (a camera in a processing of spy-plane approach in which it already has its destination, with the camera sound)
    spy_camera(w, &o, id, true);
    // spec: S-0501 claim 34 (the ground distance to the target below 769 leptons, or no target: overfly queued, the run lock held and an exit destination chosen)
    let close = match o.target {
        None => true,
        Some(tg) => planar(w, o.pos, crate::process::target_pos(w, tg)) < 769,
    };
    if close {
        let c = arrival_edge_cell(w, id, true);
        queue(w, id, Behaviour::SpyPlaneOverfly);
        set_dest(w, id, c.map(Target::Cell));
        if let Some(a) = st_mut(w, id) {
            a.run_lock = true;
        }
    } else if let Some(tg) = o.target {
        set_dest(w, id, Some(tg));
    }
    // spec: S-0501 claim 34 (the approach processing ends with the spy-plane camera frame count)
    frames
}

/// *Spy-plane overfly* (claim 34; the camera is S-0513 claim 44).
fn spy_overfly(w: &mut World, id: ObjectId) -> i32 {
    if let Some(o) = w.obj(id).cloned() {
        // spec: S-0513 claim 44 (a camera in every processing of spy-plane overfly in which the gate holds; no sound)
        spy_camera(w, &o, id, false);
    }
    if w.obj(id).is_some_and(|o| o.dest.is_none()) {
        // spec: S-0501 claim 34 (an absent destination is replaced by a new choice on the exit edge); S-0513 r87 claim 45 (after its target it keeps its approach heading and is not steered toward its exit cell, S-0501 claims 4, 10, 34 and 51)
        let c = arrival_edge_cell(w, id, true);
        set_dest(w, id, c.map(Target::Cell));
    }
    3 // spec: S-0501 claim 34 (overfly processing has a delay of 3 ticks)
}

// ---------------------------------------------------------------------------
// The attack run (claims 12 to 17, table T1)

/// Spends the pending round (claim 15).
fn spend_pending(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj_mut(id) else { return };
    let Some(a) = o.air.as_mut() else { return };
    if !a.pending {
        return;
    }
    a.pending = false;
    // spec: S-0501 claim 15 (one round per run in which it fired, taken from the count)
    if o.combat.ammo > 0 {
        o.combat.ammo -= 1;
    }
}

/// Restarts the attack run's stage (table T1) without changing the
/// behaviour: an aircraft whose target is lost in *attack* (S-0511 claim 22).
pub fn restart_run(w: &mut World, id: ObjectId) {
    // spec: S-0511 claim 22 (the attack-run substate restarts); S-0501 table T1 (the run's first stage)
    set_stage(w, id, Stage::Start);
}

fn set_stage(w: &mut World, id: ObjectId, s: Stage) {
    if let Some(a) = st_mut(w, id) {
        a.stage = s;
    }
}

pub(crate) fn set_lock(w: &mut World, id: ObjectId, on: bool) {
    if let Some(a) = st_mut(w, id) {
        a.run_lock = on;
    }
}

/// A full burst: *B* shots in the same tick with no firing check between
/// them (claim 12). Returns whether one of its attempts was not a drop
/// (claim 20's kamikaze removal).
fn full_burst(w: &mut World, id: ObjectId, tg: Target) -> bool {
    let b = primary(w, id).map_or(1, |wp| w.rules().combat.weapon(wp).burst.max(1));
    let mut ordinary = false;
    for _ in 0..b {
        ordinary |= not_a_drop(w, id);
        if !crate::combat::fire::shot(w, id, 0, tg) {
            break;
        }
    }
    ordinary
}

/// Whether the attempt aircraft `id` is about to make is one that counts
/// for claim 20's kamikaze removal: any attempt with its weapon other than
/// a drop attempt, in the order `combat::fire::shot` takes its paths (a
/// suicide or drain attempt first, then the drop of a carrying aircraft).
fn not_a_drop(w: &World, id: ObjectId) -> bool {
    let Some(wp) = crate::combat::weapon::slot_weapon(w, id, 0) else {
        return false;
    };
    let wt = w.rules().combat.weapon(wp);
    // spec: S-0501 claim 20 (r178, Q-2868: every ordinary shot counts, with a projectile or none; a drain attempt counts, and a suicide attempt, whose own hit comes first; a drop attempt does not count)
    wt.suicide || wt.drain || !crate::paradrop::shot_drops(w, id)
}

/// Claim 20's kamikaze removal (r178, Q-2868), at the end of a firing pass
/// of aircraft `id` in which `ordinary` says an attempt counted: a
/// kamikaze (S-0506 Terms) is removed from the game without dying, after
/// the pass's last attempt and its other effects, the scatter request
/// included. Returns whether it was removed.
fn kamikaze_removal(w: &mut World, id: ObjectId, ordinary: bool) -> bool {
    // spec: S-0501 claim 20 (r178, Q-2868: every flagged object that a carrier releases is a kamikaze, a flagged type that is not a rocket type included; removed at the end of the first firing pass in which it makes an ordinary shot; a refused firing check makes no attempt, so it stays in play)
    if !ordinary || !crate::spawn::kamikaze(w, id) || w.obj(id).is_none_or(|o| !o.on_map) {
        return false;
    }
    for p in w.projectiles.values_mut() {
        if p.attacker == Some(id) {
            // spec: S-0501 claim 20 (r178: every projectile of that pass has no firer when it detonates, S-0306 Terms no firer; its detonation carries neither attacker nor attacking house, S-0302 claim 1, so a target it kills gives no house a kill)
            p.attacker = None;
            p.house = None;
        }
    }
    // spec: S-0501 claim 20 (r178: the removal is not a death: no hit, explosion, debris, death weapon or crash, no kill or loss for any house, none of S-0011's destruction raises, a tag stays attached, S-0011 claim 2; no round spent, claim 15's pending round never spent; it leaves the map and play as any removed object does, S-0414 claim 4's count and S-0506 claim 26's group, and is a leftover for the rest of the tick, S-0203 claim 13; a planted bomb removed with no detonation, S-0519 claim 7; the loss notification changes no slot, S-0506 claim 9, since the kamikaze left its slot when enrolled)
    crate::objects::remove(w, id);
    true
}

/// The scatter request of a firing pass (claim 16).
fn scatter(w: &mut World, id: ObjectId, tg: Target) {
    let c = crate::process::target_pos(w, tg).cell();
    // spec: S-0501 claim 16 (the threat point is the aircraft's current position)
    let threat = w.obj(id).map(|o| o.pos);
    // spec: S-0501 claim 16 (to the target's ground cell layer; forced and not move-anywhere; S-0209 claim 11's forwarding gates still apply)
    crate::scatter::cell_request(
        w,
        c,
        Layer::Ground,
        crate::scatter::ScatterRequest {
            threat,
            forced: true,
            anywhere: false,
        },
    );
}

fn fired(w: &mut World, id: ObjectId, tg: Target) {
    if let Some(a) = st_mut(w, id) {
        // spec: S-0501 claim 15 (a run in which it fired: a round becomes pending)
        a.pending = true;
    }
    scatter(w, id, tg);
}

/// Whether the aircraft's primary weapon is a draining weapon (S-0608
/// Terms): its attempts have S-0608 claims 1, 2 and 16's effects instead of
/// an ordinary shot's.
fn drains(w: &World, id: ObjectId) -> bool {
    primary(w, id).is_some_and(|wp| w.rules().combat.weapon(wp).drain)
}

/// Claim 12's drain continuation: a drain attempt of this pass cleared the
/// object target `tg` of an aircraft that remains in play.
fn drain_cleared(w: &World, id: ObjectId, tg: Target) -> bool {
    // spec: S-0501 claim 12 (Drain continuation: an attempt that clears the object target of an aircraft that remains in play; a cell-target attempt keeps its target and uses the normal pass transitions)
    drains(w, id)
        && matches!(tg, Target::Object(_))
        && w.obj(id).is_some_and(|o| o.on_map && o.target.is_none())
}

/// Claim 12's drain continuation (departure): no remaining burst attempts
/// (the burst stopped at the attempt), no scatter request for the pass,
/// *done* with a 1-tick processing delay. Any formed link and the pending
/// round are kept; the target is not restored, the destination not
/// cleared and no rearm countdown started.
fn drain_departure(w: &mut World, id: ObjectId) -> Option<i32> {
    // spec: S-0501 claim 12 and Edge cases (Drain clears an aircraft's object target: cancel the remaining burst attempts and that pass's scatter request, keep the pending round; done is due after 1 tick)
    set_stage(w, id, Stage::Done);
    Some(1)
}

fn bare_rof(w: &World, id: ObjectId) -> i32 {
    primary(w, id).map_or(1, |wp| w.rules().combat.weapon(wp).rof)
}

/// F6: the wait after a strafing run's fifth shot, `speed` the converted
/// type speed.
pub fn strafe_wait(range: i32, speed: i32) -> i32 {
    if speed <= 0 {
        // spec: S-0501 F6 (our engine's S-0205 claim 3 departure: a converted speed of 0 gives quotient 0; the importer warns)
        return 0;
    }
    // spec: S-0501 F6 (⌊(R + 1024) / Vₜ⌋, integer division toward zero)
    (range + 1024) / speed
}

/// The processing of *attack* (table T1).
fn attack(w: &mut World, id: ObjectId) -> i32 {
    let start = w.obj(id).map_or(0, |o| o.beh.start_tick);
    let a = st(w, id);
    if a.run_of != Some(start)
        && let Some(s) = st_mut(w, id)
    {
        s.run_of = Some(start);
        s.stage = Stage::Start;
    }
    // Rows without a delay go straight on to the next stage in the same processing.
    // nocite: a loop bound of our own (a few stage changes per processing)
    for _ in 0..8 {
        if let Some(d) = attack_stage(w, id) {
            return d;
        }
    }
    1
}

/// One row of T1; `None` when it changed stage without a delay.
fn attack_stage(w: &mut World, id: ObjectId) -> Option<i32> {
    let o = w.obj(id)?.clone();
    let a = st(w, id);
    let tg = o.target;
    let curley = w.rules().air.curley_shuffle;
    let endless = w.rules().quirks.aircraft_endless_refire;
    let is_strafer = strafer(w, id);
    let is_fighter = fighter(w, id);
    match a.stage {
        Stage::Start => {
            // spec: S-0501 T1 (start: choosing with a target, done without; 1 tick)
            set_stage(
                w,
                id,
                if tg.is_some() {
                    Stage::Choosing
                } else {
                    Stage::Done
                },
            );
            Some(1)
        }
        Stage::Choosing => {
            // spec: S-0501 T1 (choosing: run lock off; the pending round is spent)
            set_lock(w, id, false);
            spend_pending(w, id);
            if let Some(tg) = tg
                && ammo_left(w, id)
            {
                // spec: S-0501 claim 12 (a strafer's firing spot is the target) and claim 39 (a non-strafer's firing spot, its draw before the standard delay)
                let spot = if is_strafer {
                    Some(tg)
                } else {
                    firing_spot(w, id, tg).map(Target::Cell)
                };
                // spec: S-0501 T1 (the firing spot becomes the destination; approaching when it exists, else done)
                set_dest(w, id, spot);
                set_stage(
                    w,
                    id,
                    if spot.is_some() {
                        Stage::Approaching
                    } else {
                        Stage::Done
                    },
                );
            } else {
                set_stage(w, id, Stage::Done);
            }
            Some(standard(w, id))
        }
        Stage::Approaching => {
            // spec: S-0501 T1 (every processing of approaching first spends a pending round and turns the run lock off, whatever else it then finds)
            set_lock(w, id, false);
            spend_pending(w, id);
            let Some(tgv) = tg.filter(|_| ammo_left(w, id)) else {
                // spec: S-0501 T1 and claim 32 (approaching, no target or no ammunition, a strafer arriving from strafe pass 5 included: done; 1 tick)
                set_stage(w, id, Stage::Done);
                return Some(1);
            };
            let in_range = crate::combat::range::in_range(w, id, primary(w, id), Some(tgv), None);
            // spec: S-0501 T1 and claim 42 (a strafer within range, a fighter, or another aircraft whose current speed fraction, left by the previous movement tick, is exactly 0: firing, the destination kept; 1 tick)
            let ordinary = !is_strafer && !is_fighter;
            // spec: S-0501 claim 42 and Edge cases (quirks.aircraft_zero_step_attack_stall off: an ordinary attacker whose positive current fraction gives F3 step length 0 leptons also moves to firing, the destination kept; 1 tick. A 1-lepton step that changes neither coordinate does not count)
            let zero_step = ordinary
                && !w.rules().quirks.aircraft_zero_step_attack_stall
                && a.cur.gt(F64::ZERO)
                && step_length(ty(w, id).map_or(0, |t| t.speed), a.cur, w.rules().mode) == 0;
            if (is_strafer && in_range)
                || (!is_strafer && is_fighter)
                || (ordinary && a.cur.eq_num(F64::ZERO))
                || zero_step
            {
                set_stage(w, id, Stage::Firing);
                return Some(1);
            }
            // The steer rows.
            let Some(d) = o.dest else {
                set_stage(w, id, Stage::Choosing);
                return None;
            };
            let p = flight_point(w, id, d);
            let dist = planar(w, o.pos, p);
            if dist >= STEER_FAR {
                // spec: S-0501 T1 (distance at least 512: the aiming facing is requested toward the destination; 1 tick)
                request(w, id, Facet::Aim, toward(w, o.pos, p));
                return Some(1);
            }
            // spec: S-0501 T1 (below 512: the aiming facing is requested toward the target, the body is not turned); S-0204 claim 8 and S-0310 F1 (toward its reference point)
            let tp = crate::combat::range::target_point(w, tgv);
            request(w, id, Facet::Aim, toward(w, o.pos, tp));
            if dist >= STEER_NEAR {
                // spec: S-0501 T1 (at least 16 leptons: 1 tick)
                return Some(1);
            }
            // spec: S-0501 T1 and claim 42 (below 16 leptons: the destination is cleared, the flight point kept, and firing; 1 tick)
            set_dest(w, id, None);
            set_stage(w, id, Stage::Firing);
            Some(1)
        }
        Stage::Firing => {
            let Some(tgv) = tg.filter(|_| ammo_left(w, id)) else {
                // spec: S-0501 T1 (firing, no target or no ammunition: done; 1 tick)
                set_stage(w, id, Stage::Done);
                return Some(1);
            };
            if !is_strafer {
                // spec: S-0501 T1 (a non-strafer requests its body and aiming facing toward the target; claim 42 decides which request lasts); S-0204 claim 8 and S-0310 F1 (toward its reference point)
                let tp = crate::combat::range::target_point(w, tgv);
                let f = toward(w, o.pos, tp);
                request(w, id, Facet::Body, f);
                request(w, id, Facet::Aim, f);
            }
            let ans = crate::combat::check::check(w, id, Some(tgv), 0, true);
            match ans {
                Answer::CanFire => {
                    let ordinary = full_burst(w, id, tgv);
                    if drain_cleared(w, id, tgv) {
                        if let Some(a) = st_mut(w, id) {
                            // spec: S-0501 claim 15 and S-0608 claim 16 (an admitted initial pass with a draining weapon makes its run's round pending, though its attempts make no ordinary shot; target clearing does not roll it back)
                            a.pending = true;
                        }
                        let d = drain_departure(w, id);
                        // spec: S-0501 claim 20 (r178, Q-2868: a drain attempt counts: claim 12's Drain continuation applies first, with no scatter request, and the removal follows)
                        kamikaze_removal(w, id, ordinary);
                        return d;
                    }
                    fired(w, id, tgv);
                    // spec: S-0501 claim 20 (r178, Q-2868: the removal neither cuts the pass short nor adds to it: it comes after the last attempt and the scatter request, in the same processing and tick)
                    if kamikaze_removal(w, id, ordinary) {
                        return Some(1);
                    }
                    if is_strafer {
                        // spec: S-0501 T1 (strafer: strafe pass 2, run lock on, one rate of fire)
                        set_lock(w, id, true);
                        set_stage(w, id, Stage::Strafe(2)); // spec: S-0501 claim 13 (the second of five passes)
                        Some(bare_rof(w, id))
                    } else if is_fighter {
                        // spec: S-0501 T1 (fighter: run lock on, choosing, or done with no ammunition; one rate of fire)
                        set_lock(w, id, true);
                        let next = if ammo_left(w, id) {
                            Stage::Choosing
                        } else {
                            Stage::Done
                        };
                        set_stage(w, id, next);
                        Some(bare_rof(w, id))
                    } else {
                        // spec: S-0501 T1 (other: refiring, 1 tick)
                        set_stage(w, id, Stage::Refiring);
                        Some(1)
                    }
                }
                Answer::NotFacing => {
                    let next = if !ammo_left(w, id) {
                        Stage::Done
                    } else if is_strafer || not_close(w, id, tgv) {
                        Stage::Choosing
                    } else if is_fighter {
                        Stage::Firing
                    } else if curley {
                        Stage::Choosing
                    } else {
                        Stage::Firing
                    };
                    set_stage(w, id, next);
                    // spec: S-0501 T1 and claim 13 (45 ticks for a strafer, else 1)
                    Some(if is_strafer { 45 } else { 1 })
                }
                // spec: S-0501 T1 (rearming: 1 tick; cloaked: uncloak, 1 tick); S-0516 claim 11 (a)
                Answer::Rearming | Answer::Cloaked => {
                    crate::cloak::machine::firing_answer(w, id, 0, tgv, ans);
                    Some(1)
                }
                _ => {
                    if !ammo_left(w, id) {
                        set_stage(w, id, Stage::Done);
                        Some(1)
                    } else if is_strafer {
                        Some(1)
                    } else {
                        set_stage(w, id, Stage::Refiring);
                        Some(1)
                    }
                }
            }
        }
        Stage::Refiring => {
            let Some(tgv) = tg else {
                // spec: S-0501 T1 and claim 32 (refiring, no target: done; 1 tick)
                set_stage(w, id, Stage::Done);
                return Some(1);
            };
            let ans = crate::combat::check::check(w, id, Some(tgv), 0, true);
            match ans {
                Answer::CanFire => {
                    // spec: S-0501 T1 (refiring, ready: one shot, scatter); a drain attempt at a cell is a pass like any other (claim 12)
                    let ordinary = not_a_drop(w, id);
                    if crate::combat::fire::shot(w, id, 0, tgv) || drains(w, id) {
                        if drain_cleared(w, id, tgv) {
                            let d = drain_departure(w, id);
                            // spec: S-0501 claim 20 (r178, Q-2868: a drain attempt counts; the removal follows the Drain continuation)
                            kamikaze_removal(w, id, ordinary);
                            return d;
                        }
                        fired(w, id, tgv);
                    }
                    // spec: S-0501 claim 20 (r178, Q-2868: an ordinary shot counts with or without a projectile; the removal follows the pass's attempt and scatter request)
                    if kamikaze_removal(w, id, ordinary) {
                        return Some(1);
                    }
                    let next = if !ammo_left(w, id) {
                        Stage::Done
                    } else if curley || !endless {
                        // spec: S-0501 claim 14 (the re-approach flag on, or quirks.aircraft_endless_refire off: choose a new firing spot)
                        Stage::Choosing
                    } else {
                        // spec: S-0501 claim 14 (flag off: firing again, quirks.aircraft_endless_refire)
                        Stage::Firing
                    };
                    set_stage(w, id, next);
                    Some(standard(w, id))
                }
                Answer::Cloaked | Answer::Rearming => Some(standard(w, id)),
                other => {
                    let next = if !ammo_left(w, id) {
                        Stage::Done
                    } else if not_close(w, id, tgv)
                        || curley
                        || (is_strafer && other == Answer::NotFacing)
                    {
                        Stage::Choosing
                    } else {
                        Stage::Firing
                    };
                    set_stage(w, id, next);
                    if is_strafer && other == Answer::NotFacing {
                        Some(45) // spec: S-0501 T1 (a strafer's not facing: chooses again and waits 45 ticks)
                    } else {
                        Some(standard(w, id))
                    }
                }
            }
        }
        Stage::Strafe(n) => {
            let Some(tgv) = tg else {
                // spec: S-0501 T1 (any strafe pass with no target: done; 1 tick)
                set_stage(w, id, Stage::Done);
                return Some(1);
            };
            let ans = crate::combat::check::check(w, id, Some(tgv), 0, true);
            if matches!(
                ans,
                Answer::CanFire | Answer::NotFacing | Answer::Cloaked | Answer::OutOfRange
            ) {
                // spec: S-0501 claim 13 and T1 (later passes fire even when not facing, cloaked or out of range; one shot, scatter, the target becomes the destination); a drain attempt at a cell is a pass like any other (claim 12)
                let ordinary = not_a_drop(w, id);
                if crate::combat::fire::shot(w, id, 0, tgv) || drains(w, id) {
                    if drain_cleared(w, id, tgv) {
                        let d = drain_departure(w, id);
                        // spec: S-0501 claim 20 (r178, Q-2868: a drain attempt counts; the removal follows the Drain continuation)
                        kamikaze_removal(w, id, ordinary);
                        return d;
                    }
                    fired(w, id, tgv);
                }
                set_dest(w, id, Some(tgv));
                // spec: S-0501 claim 20 (r178, Q-2868: the removal follows the pass's attempt, scatter request and its other effects)
                if kamikaze_removal(w, id, ordinary) {
                    return Some(1);
                }
                if n >= LAST_PASS {
                    // spec: S-0501 T1 (strafe pass 5: approaching; F6 ticks)
                    set_stage(w, id, Stage::Approaching);
                    let range = primary(w, id).map_or(0, |wp| w.rules().combat.weapon(wp).range);
                    let vt = ty(w, id).map_or(0, |t| t.speed);
                    Some(strafe_wait(range, vt))
                } else {
                    set_stage(w, id, Stage::Strafe(n + 1));
                    Some(bare_rof(w, id))
                }
            } else if !ammo_left(w, id) && ans == Answer::OutOfAmmo {
                // spec: S-0501 T1 (another refusal with no ammunition: done, run lock off; 1 tick)
                set_lock(w, id, false);
                set_stage(w, id, Stage::Done);
                Some(1)
            } else {
                // spec: S-0501 T1 (otherwise retry the same pass; 1 tick)
                Some(1)
            }
        }
        Stage::Done => {
            // spec: S-0501 T1 (done: run lock off, the pending round spent, then claim 17; 1 tick)
            set_lock(w, id, false);
            spend_pending(w, id);
            run_end(w, id);
            Some(1)
        }
    }
}

/// Not close enough (claim 12): the range test of S-0307 claims 7 to 11
/// fails for the weapon selected against the target.
fn not_close(w: &World, id: ObjectId, tg: Target) -> bool {
    // spec: S-0501 claim 12 (close enough: S-0307 claims 7 to 11's range test for the weapon selected against the current target, from claim 8's firing source point; not the last firing-check answer)
    let wp = crate::combat::weapon::would_use(w, id, Some(tg)).1;
    !crate::combat::range::in_range(w, id, wp, Some(tg), None)
}

/// The end of a run (claim 17).
fn run_end(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else { return };
    let ammo = ammo_left(w, id);
    if o.target.is_some() && ammo {
        // spec: S-0501 claim 17 (ammunition and a target: straight into the next run)
        set_stage(w, id, Stage::Choosing);
        return;
    }
    let visiting = st(w, id).visiting;
    if !ammo
        && (visiting || human(w, id))
        && let Some(ob) = w.obj_mut(id)
    {
        // spec: S-0501 claim 17 (no ammunition: a visiting or human-controlled house's aircraft forgets its target)
        crate::spawn::clear_own_target(ob);
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
    }
    // spec: S-0501 claims 17 and 37 (every other run end gives a destination after the target is forgotten and before retreat or the fall-back: a cell on the house's computed edge, selected as in S-0522 claim 4 with its draws)
    if let Some(c) = arrival_edge_cell(w, id, false) {
        set_dest(w, id, Some(Target::Cell(c)));
    }
    // spec: S-0501 claim 17 (a strike aircraft with ammunition retreats; every other aircraft falls back to rest)
    set_stage(w, id, Stage::Start);
    if ammo && crate::strike::linked(w, id) {
        queue(w, id, Behaviour::Retreat);
    } else {
        fall_back(w, id);
    }
}

// ---------------------------------------------------------------------------
// The end of an aircraft's update (claims 15, 31 and 37)

/// The end of an aircraft's update: the third start opportunity, the
/// pending round of a left run, the run lock and the damage-smoke draw.
pub fn end_turn(w: &mut World, id: ObjectId) {
    if !is_aircraft(w, id) {
        return;
    }
    let Some(o) = w.obj(id).cloned() else { return };
    let alive = !crate::combat::damage::dead(&o);
    if alive {
        discard_placeholder(w, id);
        let Some(o) = w.obj(id).cloned() else { return };
        let cur = o.beh.current;
        // spec: S-0501 claim 31 (the run lock is cleared in every update whose behaviour is none of attack, paradrop overfly, spy-plane approach and overfly)
        if !matches!(
            cur,
            Some(
                Behaviour::Attack
                    | Behaviour::ParadropOverfly
                    | Behaviour::SpyPlaneApproach
                    | Behaviour::SpyPlaneOverfly
            )
        ) {
            set_lock(w, id, false);
        }
        ready_mark_after_enter(w, id);
        // spec: S-0501 claim 31 (the third start opportunity: once more each update, after its processing and movement)
        start_if_ready(w, id);
        // spec: S-0501 claim 15 (a left run's pending round is spent at the end of that update, a start in this update included)
        if w.obj(id)
            .is_some_and(|x| x.beh.current != Some(Behaviour::Attack))
        {
            spend_pending(w, id);
        }
    }
    smoke(w, id);
}

/// Whether `t` is claim 41's placeholder destination: the cell position
/// with map coordinates (0, 0), where no map has a cell.
pub fn is_placeholder(w: &World, t: Option<Target>) -> bool {
    // spec: S-0501 claim 41 (the placeholder: map cell coordinates (0, 0), where no map has a cell)
    t == Some(Target::Cell(PLACEHOLDER)) && w.cells.cell(PLACEHOLDER).is_none()
}

/// Claim 41's discard: an aircraft whose destination or target is the
/// placeholder discards both at the end of its update and falls back to
/// rest again.
fn discard_placeholder(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if !is_placeholder(w, o.dest) && !is_placeholder(w, o.target) {
        return;
    }
    // spec: S-0501 claim 41 (Q-0650: the destination and target are discarded at the end of its update, after its behaviour processing and movement)
    forget(w, id);
    if let Some(a) = st_mut(w, id)
        && is_placeholder_target(a.flying_to)
    {
        // The placeholder given again is a new movement point (claim 40).
        a.flying_to = None;
        a.raised = false;
    }
    // spec: S-0501 claim 41 (it falls back to rest again, claim 18)
    fall_back(w, id);
}

fn is_placeholder_target(t: Option<Target>) -> bool {
    t == Some(Target::Cell(PLACEHOLDER))
}

/// The damage-smoke chance draw (claim 37; `quirks.aircraft_smoke_scenario_draw`).
fn smoke(w: &mut World, id: ObjectId) {
    let rules = w.rules_arc();
    // spec: S-0501 Edge cases (quirks.aircraft_smoke_scenario_draw off: no damage smoke and no draw)
    if !rules.quirks.aircraft_smoke_scenario_draw {
        return;
    }
    let Some(o) = w.obj(id).cloned() else { return };
    if height(w, id) <= 0 {
        return;
    }
    let mode = rules.mode;
    let s = crate::combat::damage::strength(w, id).max(1);
    let ratio =
        F64::from_i64(i64::from(o.health), mode).div(F64::from_i64(i64::from(s), mode), mode);
    // spec: S-0501 claim 37 (a health ratio below the condition-red ratio and a height above 0: one U(0, 99) draw)
    if !ratio.lt(rules.general.condition_red) {
        return;
    }
    let u = w.rng.ranged(0, 99, rules.quirks.ranged_draw_half_span); // spec: S-0501 claim 37 (a U(0, 99) draw)
    let dead = crate::combat::damage::dead(&o);
    // spec: S-0501 claim 37 (smoke when u < 10 alive, u < 80 dead)
    let shown = if dead { u < 80 } else { u < 10 };
    if !shown {
        return;
    }
    // spec: S-0501 claim 37 (Q-2237: one animation of the fixed damage-smoke type, the same for a living and a dead aircraft and for every aircraft type, found by its name without regard to letter case; at the aircraft's position when u is taken, after its own movement in this tick, height included, with no raise or lowering; not attached; no initial loop delay, repeat factor 1, no reverse request; its creation draws, S-0109 claims 3 and 13, right after u in this turn; no type of that name: nothing created, u still taken)
    let made = crate::anims::spawn(
        w,
        DAMAGE_SMOKE,
        Some(o.pos),
        crate::anims::Spawn {
            creation: crate::anim_clock::Creation {
                loop_delay: 0,
                repeat: 1,
                reverse: false,
            },
            owner: None,
            role: crate::anims::Role::Ordinary,
            looped: false,
        },
    );
    if made.id.is_some() {
        // The presentation event: the view draws the animation the simulation plays, under
        // the type's registered name.
        let name = made
            .id
            .and_then(|id| w.anims.anims.get(&id))
            .and_then(|a| a.anim)
            .and_then(|i| w.rules().animations.get(i as usize))
            .map_or_else(|| DAMAGE_SMOKE.to_owned(), |t| t.name.clone());
        w.effects.push(crate::combat::effects::Effect::Anim {
            name,
            pos: o.pos,
            delay: made.delay,
            reverse: false,
            z_adjust: None,
            id: made.id,
        });
    }
}

/// The fixed name of the damage-smoke animation type (S-0501 Import
/// mapping): no rules entry names it.
// spec: S-0501 Import mapping (animation type `SGRYSMK1`, a fixed animation type name, found without regard to letter case, S-0004 claims 2 and 7) and claim 37
pub const DAMAGE_SMOKE: &str = "SGRYSMK1";

// ---------------------------------------------------------------------------
// Carrier slot aircraft (claims 35 and 36, table T4)

/// A slot aircraft is sent toward `cell` with *move* queued (T4: launch and
/// the preparing re-send).
pub fn carrier_move(w: &mut World, x: ObjectId, cell: CellXY) {
    set_dest(w, x, Some(Target::Cell(cell)));
    // spec: S-0501 T4 (the destination and move queued)
    queue(w, x, Behaviour::Move);
}

/// A slot aircraft receives the controller target and queues *attack*
/// (T4).
pub fn carrier_attack(w: &mut World, x: ObjectId, target: Target) {
    if let Some(o) = w.obj_mut(x) {
        o.target = Some(target);
    }
    // spec: S-0301 claim 5, Moment of a retarget's let-go (revision 67, Q-2246: at the change of the firer's target)
    crate::emitters::retarget_let_go(w, x);
    // spec: S-0501 T4 (receive the controller target and queue attack)
    queue(w, x, Behaviour::Attack);
}

/// A slot aircraft returns: target cleared, the spawner as destination,
/// *move* (T4).
pub fn carrier_return(w: &mut World, x: ObjectId, spawner: ObjectId) {
    if let Some(o) = w.obj_mut(x) {
        crate::spawn::clear_own_target(o);
        o.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
    }
    set_dest(w, x, Some(Target::Object(spawner)));
    // spec: S-0501 T4 (clear target, spawner as destination, move)
    queue(w, x, Behaviour::Move);
}
