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
    } else if !a.ground && (!on_ground(w, id) || a.released) {
        // spec: S-0501 claim 47 (c) and claim 48 (c) (so does an aircraft that a factory release has left above its pad, whatever its height at that update: the low release at height 0 is not at rest and becomes tracked all the same)
        // spec: S-0501 claim 47 (c) and Edge cases (quirks.aircraft_corner_sector_untracked on: an untracked aircraft in flight, or one a factory release has left above its pad, whose cell lies in sector (0, 0) of S-0303 F7, cell (0, 0) included, joins no list; off: it joins sector (0, 0)'s list like any other)
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
    // spec: S-0501 claim 31 (not ready with the run lock, except in spy-plane approach)
    !(st(w, id).run_lock && cur != Some(Behaviour::SpyPlaneApproach))
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
    // spec: S-0501 claim 1 (on the ground when not visiting, by the mark already present, and inside the usable area; else at cruise height)
    let ground = !a.visiting && usable;
    // spec: S-0501 claim 1 (a type placed carrying passengers is a passenger carrier for life)
    a.passenger_carrier |= passengers(w, id) > 0;
    a.entered = usable;
    // spec: S-0501 claim 43 (ordinary fresh ground placement replaces the requested height with the terrain height; a placement is no way to require a deck landing: the mark is clear)
    a.deck = false;
    let z = if ground {
        // spec: S-0501 claim 1 (ground placement replaces the requested world height with the terrain height)
        a.ground = true;
        a.held = 0;
        a.mark = Some(cell);
        surface
    } else {
        a.ground = false;
        a.held = t.cruise;
        surface + t.cruise
    };
    let mode = w.rules().mode;
    // spec: S-0501 claim 1 (the current speed fraction starts at 1 exactly at cruise height, else 0)
    a.cur = if z - surface == t.cruise {
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

/// The aircraft took destination `dest` (S-0501 claim 48 (c)): an aircraft
/// that a factory release has left above its pad ends its pad contact at
/// the order when the destination is not that pad and the cell holding its
/// horizontal position belongs to the pad, whatever its height. An
/// aircraft standing on its pad ends it at its take-off (claim 8).
pub(crate) fn destination_taken(w: &mut World, id: ObjectId, dest: Option<Target>) {
    if !is_aircraft(w, id) || !st(w, id).released {
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
    let s = step_length(v, frac, mode);
    // spec: S-0501 F3 (the table step of S-0208 F3 with θ = (Fₛ − 16383) × k, k = −2π / 65534; each new coordinate truncated, no remainder kept, so a 1-lepton step at facings 16391 to 32766 changes neither coordinate: quirks.aircraft_unit_step_stall)
    let (nx, ny) = crate::walk::step_point(x, y, facing, s, true, mode);
    if unit_stall || s < 1 || (nx, ny) != (x, y) {
        return (nx, ny);
    }
    // spec: S-0501 F3 and Edge cases (quirks.aircraft_unit_step_stall off: a step of at least 1 lepton that would change neither coordinate moves 1 lepton along the axis of the larger component, in its direction, x on equal magnitudes)
    let (dx, dy) = crate::walk::step_components(facing, s, true, mode);
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
    let held = w.obj(id).and_then(|o| o.air.as_ref());
    // spec: S-0501 claim 6 (the replacement is made when a movement point is accepted while the aircraft has a target and ammunition; it concerns the flight point held for movement and is not recomputed when the target or ammunition later goes)
    let raised = match held {
        Some(a) if a.flying_to == Some(d) => a.raised,
        _ => raise_now(w, id),
    };
    point_with(w, id, d, raised)
}

/// Whether a movement point accepted now has its height replaced (claim
/// 6): the aircraft has a target and ammunition.
fn raise_now(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.target.is_some()) && ammo_left(w, id)
}

/// [`flight_point`] with the height replacement given.
fn point_with(w: &World, id: ObjectId, d: Target, raised: bool) -> Pos {
    // spec: S-0501 claim 40 (the destination's heading point with the aircraft as requesting actor: a cell's centre at its surface, a mobile object's next movement point or position, a building's centre)
    let mut p = heading_point(w, d);
    // spec: S-0501 claim 6 (a movement point accepted while the aircraft has a target and ammunition: its world height becomes the terrain height there plus the cruise height)
    if raised {
        let cruise = ty(w, id).map_or(0, |t| t.cruise);
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
    let tick = w.tick;
    // spec: S-0206 claim 7 (an aircraft's body and turret facings turn at the type's turn rate)
    let rate = ty(w, id).map_or(0, |t| t.turn_rate);
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
/// flight (claim 42).
fn flight_target(w: &World, id: ObjectId) -> Option<Target> {
    let o = w.obj(id)?;
    if o.dest.is_some() {
        return o.dest;
    }
    let a = st(w, id);
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
    let Some(o) = w.obj(id).cloned() else { return };
    if crate::combat::damage::dead(&o) {
        return;
    }
    let p = flight_point(w, id, d);
    let here = o.pos;
    // spec: S-0501 claim 42 (the bearing of S-0204 claim 8 from its current position)
    let bearing = toward(w, here, p);
    // spec: S-0501 claim 42 (the distance to the flight point after any pad-position adjustment)
    let dist = planar(w, here, p);
    let is_strafer = strafer(w, id);
    // spec: S-0501 claim 42 (a strafer at most 768 leptons from the point requests its body only when the bearing differs from its current body facing by at most 8192 facing units)
    let strafer_skips = is_strafer
        && dist <= STRAFER_WINDOW
        && glasnost_num::geom::delta(o.body.value(w.tick), bearing).abs() > STRAFER_TURN;
    if !strafer_skips {
        // spec: S-0501 claim 42 (the body facing is requested toward the flight point)
        request(w, id, Facet::Body, bearing);
    }
    let armed_target = o.target.is_some() && ammo_left(w, id);
    // spec: S-0501 claim 42 (below 256 leptons, no fly-by flag, not a strafer with a target and ammunition left: the aiming facing toward the target with a target and ammunition left, otherwise toward the landing direction of claim 25; in every other case toward the flight point)
    let aim = if dist < AIM_NEAR && !t.fly_by && !(is_strafer && armed_target) {
        match o.target {
            // spec: S-0204 claim 8 and S-0310 F1 (the direction of the target: toward its reference point)
            Some(tg) if armed_target => toward(w, here, crate::combat::range::target_point(w, tg)),
            _ => landing_facing(w, id),
        }
    } else {
        bearing
    };
    request(w, id, Facet::Aim, aim);
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
            let (x, y) = crate::walk::step_point(tp.x, tp.y, f, radius, true, mode);
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
    if landing_cell_ok(w, id, centre, d) && reservation_ok(w, id, heading_point(w, d).cell(), false)
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
            let (x, y) = crate::walk::step_point(centre.x, centre.y, f, r, true, mode);
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

/// The aircraft touches down (claim 7) on an accepted cell; docks when it
/// lands at its pad contact (claim 24).
fn touchdown(w: &mut World, id: ObjectId) {
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
        // spec: S-0501 claim 40 (landing settled: the completion clears the unfinished flight and the destination)
        a.unfinished = false;
        a.released = false;
    }
    set_mark(w, id);
    // spec: S-0212 claim 13 (each landing records the aircraft in its landing cell)
    crate::threat::landed(w, id);
    let pad = o
        .contact
        .filter(|&k| w.obj(k).is_some_and(|b| b.kind == Kind::Building) && pad_cell(w, k, id));
    set_dest(w, id, None);
    // spec: S-0501 claim 24 (in enter, the docking stage's processing after touchdown reports that it has entered; outside enter it docks where it touched down, with no move to the docking position)
    let entering = o.beh.current == Some(Behaviour::Enter);
    if let Some(k) = pad
        && !entering
    {
        dock(w, id, k);
    }
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
    let facing = landing_facing(w, id);
    let t = w.tick;
    let keeps = w.rules().quirks.set_facing_keeps_target;
    if let Some(o) = w.obj_mut(id) {
        o.contact = Some(k);
        o.econ.exit = Some(crate::econ::release::Exit {
            factory: k,
            tethered: false,
            place,
        });
        // Claim 25's application time is open (Open question 4): applied at touchdown.
        o.body.set(facing, t, keeps);
        o.turret.set(facing, t, keeps);
    }
    report_entered(w, id, k);
}

/// One step of *enter*'s docking approach while the aircraft lands (claim
/// 24): toward its contact place's docking position at pad `k`, along each
/// map axis by the difference limited to −5 … +5 leptons, its world height
/// unchanged.
fn docking_step(w: &mut World, id: ObjectId, k: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let here = o.pos;
    let to = if w.obj(k).is_some_and(|b| b.kind == Kind::Building) {
        crate::econ::release::pad_position(w, k, id).0
    } else {
        // spec: S-0501 claim 24 (a contact that is not a building gives its centre instead of a docking position)
        w.reference_point(k)
    };
    // spec: S-0501 claim 24 and Constants (the docking approach step: each map axis by the difference to the docking position limited to −5 … 5 leptons, each axis separately, exact integer arithmetic; the world height unchanged)
    let step = |d: i32| d.clamp(-5, 5);
    let dx = step(to.x.wrapping_sub(here.x));
    let dy = step(to.y.wrapping_sub(here.y));
    if (dx, dy) != (0, 0) {
        move_to(w, id, Pos::new(here.x + dx, here.y + dy, here.z));
    }
}

/// The aircraft reports that it has entered pad `k` (claim 24): *sleep*
/// with *guard* queued, and the pad starts servicing.
fn report_entered(w: &mut World, id: ObjectId, k: ObjectId) {
    let (t, s) = (w.tick, stale(w));
    if let Some(o) = w.obj_mut(id) {
        // spec: S-0501 claim 24 (its behaviour becomes sleep, with guard queued)
        o.beh.force(Behaviour::Sleep, t, s);
        o.beh.queue(Some(Behaviour::Guard));
    }
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
    let moved = tick_flight(w, id);
    // spec: S-0209 claim 13 (taking off removes the listing when the listing height becomes positive, and landing adds it again)
    sync_listing(w, id);
    if released {
        released_mark(w, id);
    }
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
    let raised = raise_now(w, id);
    if let Some(a) = st_mut(w, id) {
        a.flying_to = dest;
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

fn tick_flight(w: &mut World, id: ObjectId) -> bool {
    if !is_aircraft(w, id) {
        return false;
    }
    let Some(t) = ty(w, id) else { return false };
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    if !o.on_map {
        return false;
    }
    let rules = w.rules_arc();
    let mode = rules.mode;
    let ar = &rules.air;
    let dead = crate::combat::damage::dead(&o);
    let start = o.pos;
    if o.air.is_none() {
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
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    // spec: S-0501 claim 42 (a destination cleared in attack with a target keeps the flight point: guidance and claim 5's slowing go on toward it)
    let dest = if dead { None } else { flight_target(w, id) };
    let dp = dest.map(|d| flight_point(w, id, d));
    let here = o.pos;
    let d = dp.map(|p| planar(w, here, p));
    let dest_above = dp.map(|p| p.z - w.surface_at(p.x, p.y));
    // spec: S-0501 claim 6 (holding height: destination more than 120 leptons above terrain, a target with ammunition left, the run lock, or a type that is not landable)
    // spec: S-0501 claim 6 (an enter movement update clears the mark)
    let entering = o.beh.current == Some(Behaviour::Enter);
    a.holding = alive
        && !entering
        && (dest_above.is_some_and(|z| z > HOLD_ABOVE)
            || (o.target.is_some() && ammo_left(w, id))
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
    }
    if alive && a.released && dest.is_none() && a.cur == F64::ZERO {
        // spec: S-0501 claim 48 (a) (its movement updates move it only toward a destination: while it has none, neither its height nor its position changes)
        if let Some(ob) = w.obj_mut(id) {
            ob.air = Some(a);
        }
        return false;
    }
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
    if alive && !a.landing && !a.ground {
        if !a.released {
            a.held = ordinary_held(&a);
        } else if !a.taking_off && a.held == 0 && a.tgt.eq_num(F64::from_i64(1, mode)) {
            // spec: S-0501 claim 48 (b) (a movement update that it begins neither landing nor taking off, with held height 0 and a target speed fraction of 1, gives the held height of ordinary flight: it climbs in that same update)
            a.held = ordinary_held(&a);
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
        let hh = cruise; // spec: S-0501 claim 8 (H = cruise height minus landing height 0)
        // spec: S-0501 claim 8 (the thresholds H − trunc(H / 3) and trunc(H / 2))
        let (upper, lower) = (hh - hh / 3, hh / 2);
        if h0 > upper {
            // spec: S-0501 claim 8 (above H − trunc(H / 3): the aiming facing is requested to match the current body facing; no body request)
            let body = w.obj(id).map_or(0, |ob| ob.body.value(tick_now));
            request(w, id, Facet::Aim, body);
        } else if h0 > lower {
            // spec: S-0501 claim 8 (above trunc(H / 2), up to and including the upper threshold: the body faces the destination and full speed is requested)
            if let Some(p) = dp {
                request(w, id, Facet::Body, toward(w, here, p));
            }
            full_request = true;
        }
        // spec: S-0501 claim 8 (at or below trunc(H / 2): no new heading request; the previous request persists)
        let r = st(w, id);
        (a.request_tick, a.body_was, a.aim_was) = (r.request_tick, r.body_was, r.aim_was);
    }
    // Horizontal motion (claim 4): forward along the body facing by ⌊V × f⌋.
    let mut removed = false;
    if !a.ground {
        let facing = w.obj(id).map_or(0, |ob| ob.body.value(tick_now));
        let unit_stall = w.rules().quirks.aircraft_unit_step_stall;
        let (nx, ny) = step(here.x, here.y, facing, t.speed, a.cur, unit_stall, mode);
        if (nx, ny) != (here.x, here.y) {
            let mut np = Pos::new(nx, ny, here.z);
            let legal = |p: Pos| w.cells.cell(p.cell()).is_some();
            let may_leave = may_leave_now(w, id, &a);
            if !legal(np) && may_leave && !t.fly_by {
                // spec: S-0501 claim 4 (the position shifted 128 leptons in x back toward the map's middle)
                let middle = w.cells.footprint().width * 256 / 2; // spec: S-0204 claim 2 (256 leptons per cell)
                let back = if np.x > middle { -128 } else { 128 }; // spec: S-0501 Constants (edge push-back 128 leptons)
                let shifted = Pos::new(np.x + back, np.y, np.z);
                np = if legal(shifted) {
                    shifted
                } else {
                    // spec: S-0501 claim 4 (a random fallback 64 leptons from the shifted point: one raw draw, its low eight bits times 256 as the facing)
                    let f = ((w.rng.raw() & 0xff) * 256) as Facing;
                    // spec: S-0501 claim 4 (F3's displacement and truncation with length 64, no snapping)
                    let (rx, ry) = crate::walk::step_point(shifted.x, shifted.y, f, 64, true, mode); // spec: S-0501 Constants (edge random fallback radius 64)
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
    let Some(o) = w.obj(id).cloned() else {
        return true;
    };
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
        // spec: S-0501 claim 4 (a living descending aircraft with a destination moves 5 more leptons toward its destination cell's centre, after the forward step and the height reduction, except in enter toward a helipad building)
        let toward_helipad = o.beh.current == Some(Behaviour::Enter)
            && matches!(dest, Some(Target::Object(k)) if helipad(w, k));
        if alive
            && nh < h
            && !toward_helipad
            && let Some(p) = dp
        {
            let c = p.cell();
            let centre = Pos::centre(c, 0);
            let now = w.obj(id).map_or(o.pos, |x| x.pos);
            // spec: S-0501 claim 4 (the direction by S-0204 claim 8, F3's facing conversion and table trigonometry with step length 5, no clamp; the world height kept)
            let f = toward(w, now, centre);
            let (nx, ny) = crate::walk::step_point(now.x, now.y, f, 5, true, mode); // spec: S-0501 Constants (descent horizontal nudge 5 leptons)
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
    // Touchdown (claim 7): changing into the ground layer.
    if alive && a.landing && height(w, id) <= 0 {
        if landing_accepted(w, id) {
            touchdown(w, id);
        } else {
            // spec: S-0501 claim 7 (a refused cell: it rises by 10 leptons and starts to take off again)
            if let Some(ob) = w.obj_mut(id) {
                ob.pos.z += 10;
            }
            if let Some(s) = st_mut(w, id) {
                s.landing = false;
                s.taking_off = true;
                s.held = cruise;
                // spec: S-0501 claim 43 (a touchdown refused under claim 7 clears the mark as take-off restarts)
                s.deck = false;
            }
            // spec: S-0501 claim 48 (e) and claim 44 (above height 0 it holds no object-present flag: a low release that carried its flag into the landing loses it as take-off restarts; any other lander has none)
            clear_mark(w, id);
            // spec: S-0102 claim 4 (a refused landing cell starts a take-off in that update, which re-files it; the landing update re-files it too, in tick)
        }
        return w.obj(id).is_some_and(|ob| ob.pos != start);
    }
    let mut a = st(w, id);
    // Speed fraction (claim 5, F4), at the end of a living movement tick.
    if alive && !a.ground {
        let fixed = full_speed(w, id, &t, &a);
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
        landing_trigger(w, id, &t, fixed);
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

/// The landing triggers of claim 6.
fn landing_trigger(w: &mut World, id: ObjectId, t: &Ty, fixed: bool) {
    let Some(o) = w.obj(id).cloned() else { return };
    let a = st(w, id);
    if a.holding || a.landing || a.taking_off || a.ground {
        return;
    }
    let no_fire = o.target.is_none() || !ammo_left(w, id);
    if !no_fire {
        return;
    }
    let trigger = match o.dest {
        // spec: S-0501 claim 8 (no destination in flight: it lands where it is) and Terms (claim 48 (c)'s low release at height 0 is on the ground by its height, not in flight, as after its fall-back to rest, which stays at height 0 in its cell)
        None => !low_release_at_height_0(w, id, &a),
        Some(dd) => {
            let p = flight_point(w, id, dd);
            let dist = planar(w, o.pos, p);
            // spec: S-0501 claim 6 (standing in its destination cell with a target fraction of 0)
            let exact = o.pos.cell() == p.cell() && a.tgt.eq_num(F64::ZERO);
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
    // spec: S-0501 claim 6 (a pad-bound aircraft whose cell holds no building it is in contact with falls back to rest instead)
    // design: S-0501 Open question 26 (a pad-bound aircraft released from a docking position outside the foundation that took its destination without a behaviour order still holds its contact in flight: in a destination cell holding no building it is in contact with it falls back, and T2's after-the-row enter is claim 24's, which takes the pad as its destination; in a destination cell of the pad it is in contact with the restriction does not hold, so it starts to land and its touchdown docks it there, claim 24)
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
    if let Some(a) = st_mut(w, id) {
        a.heading_pad = Some(k);
    }
    true
}

/// A pad search and contact request (claims 18 and 29); the pad on success.
fn seek_pad(w: &mut World, id: ObjectId) -> Option<ObjectId> {
    let k = find_pad(w, id)?;
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
        // spec: S-0501 claim 26 (an aircraft still flying in, in enter, or still moving is skipped)
        if o.beh.current == Some(Behaviour::Enter)
            || moving(w, x)
            || !st(w, x).ground
            || !pad_cell(w, k, x)
        {
            continue;
        }
        settled = true;
        let release = |w: &mut World| {
            // spec: S-0501 claim 26 (released: it falls back to rest and its guard is forced)
            fall_back(w, x);
            let (t, s) = (w.tick, stale(w));
            if let Some(ob) = w.obj_mut(x) {
                ob.beh.force(Behaviour::Guard, t, s);
            }
        };
        // spec: S-0501 claim 26 (ready for take-off, no target and full ammunition, with full health: released)
        if o.target.is_none() && full_ammo(w, x) && full_health(w, x) {
            release(w);
            continue;
        }
        // spec: S-0501 claim 26 (a settled aircraft not in sleep is told to queue sleep)
        if o.beh.current != Some(Behaviour::Sleep) {
            queue(w, x, Behaviour::Sleep);
            continue;
        }
        if !full_ammo(w, x) {
            let a = max_ammo(w, x);
            // spec: S-0501 claim 28 (with a target, served only up to ⌊A / 2⌋: the reload is confirmed without a round; quirks.aircraft_half_reload_with_target)
            if w.rules().quirks.aircraft_half_reload_with_target
                && o.target.is_some()
                && o.combat.ammo >= a / 2
            {
                continue;
            }
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
        // spec: S-0501 claim 26 (with no aircraft settled, servicing ends; otherwise every P ticks)
        a.service = settled.then(|| t.wrapping_add(p.max(1)));
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
    // spec: S-0501 T2 (guard; area guard for an armed aircraft of a computer-controlled house outside a team; our engine has no teams, claim 18)
    let resting = if armed && !human(w, id) {
        Behaviour::AreaGuard
    } else {
        Behaviour::Guard
    };
    // The row's next behaviour, queued after the after-the-row entries.
    let chosen = if ground {
        if a.visiting && carrying {
            // spec: S-0501 T2 (on the ground, visiting, with passengers: unload)
            Some(Behaviour::Unload)
        } else if a.visiting {
            // spec: S-0501 T2 (on the ground, visiting, no passengers: forgets target and destination; retreat)
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
            || (o.beh.current == Some(Behaviour::Enter)
                && o.dest
                    .is_some_and(|d| Some(d) == o.contact.map(Target::Object)))
            || t.docks.is_empty())
    {
        // spec: S-0501 T2 and claim 45 (in flight, armed: the in-the-air answer is no, already flying to its enter destination, or no dock types: the destination kept; guard, area guard for a computer-controlled house)
        Some(resting)
    } else if armed {
        if let Some(k) = seek_pad(w, id) {
            // spec: S-0501 T2 (a pad found that grants contact: the pad as destination; enter)
            set_dest(w, id, Some(Target::Object(k)));
            Some(Behaviour::Enter)
        } else if t.bound {
            // spec: S-0501 T2 and claim 46 (in flight, armed, no pad, pad-bound: it crashes at this fall-back)
            crash_fall_back(w, id);
            return;
        } else {
            // spec: S-0501 T2 (in flight, armed, no pad, not pad-bound: the landing spot of F8; move)
            let s = landing_spot(w, id);
            set_dest(w, id, s.map(Target::Cell));
            Some(Behaviour::Move)
        }
    } else {
        // spec: S-0501 T2 (in flight otherwise: the landing spot; move)
        let s = landing_spot(w, id);
        set_dest(w, id, s.map(Target::Cell));
        Some(Behaviour::Move)
    };
    let mut chosen = chosen;
    // spec: S-0501 T2 and its note (after every row from the fourth on, the visiting rows included: no ammunition, armed, no contact: a new pad search, which needs neither height test, claim 18)
    if armed && !ammo_left(w, id) && w.obj(id).is_some_and(|x| x.contact.is_none()) {
        if let Some(k) = seek_pad(w, id) {
            // spec: S-0501 claim 18 and T2 (found: the pad as destination, the target forgotten, enter)
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
        // spec: S-0501 claim 32 (an approaching aircraft whose destination is lost falls back to rest, looking for another pad)
        set_dest(w, id, None);
        fall_back(w, id);
        return standard(w, id);
    }
    let (expired, start) = w
        .obj(id)
        .map_or((false, 0), |o| (o.beh.expired, o.beh.start_tick));
    // spec: S-0501 claim 40 (the selection runs at the first processing of every such move, also one started inside another behaviour's processing, such as the fall-back at a run's end, whose start timer that processing's delay replaced, S-0215 claims 2 and 3)
    let first = expired || st(w, id).first_moved != Some(start);
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
    // spec: S-0501 claim 29 (an armed aircraft below cruise height, not full and without contact: a pad search; enter, target forgotten) and claim 48 (d) (guard compares the height with the cruise height for equality only: every other height, above it as well as below it, takes the same rules)
    if armed
        && !ground
        && h != t.cruise
        && !full_ammo(w, id)
        && o.contact.is_none()
        && let Some(k) = seek_pad(w, id)
    {
        set_dest(w, id, Some(Target::Object(k)));
        if let Some(ob) = w.obj_mut(id) {
            crate::spawn::clear_own_target(ob);
            ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
        }
        queue(w, id, Behaviour::Enter);
        return standard(w, id);
    }
    // spec: S-0501 claims 29 and 48 (d) (the cruise-height outcomes at exactly the cruise height only)
    if !ground && h == t.cruise {
        if o.dest.is_some() {
            if !t.bound {
                // spec: S-0501 claim 29 (at cruise height with a destination: move, if it is not pad-bound)
                queue(w, id, Behaviour::Move);
            }
        } else if armed {
            // spec: S-0501 claim 29 (at cruise height, armed, no destination: falls back to rest)
            fall_back(w, id);
        } else {
            // spec: S-0501 claim 29 (unarmed: its own cell and move, so it lands)
            let c = o.pos.cell();
            set_dest(w, id, Some(Target::Cell(c)));
            queue(w, id, Behaviour::Move);
        }
    } else if !ground && !t.bound && o.dest.is_some() && o.beh.queued.is_none() && o.health > 0 {
        // spec: S-0501 claim 33 (a living, not pad-bound aircraft in flight in guard with a destination and nothing queued: move)
        queue(w, id, Behaviour::Move);
    }
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
    // spec: S-0501 claim 24 (entry begins with a 1-tick delay)
    if st(w, id).enter_of != Some(o.beh.start_tick) {
        if let Some(a) = st_mut(w, id) {
            a.enter_of = Some(o.beh.start_tick);
            a.enter_docking = false;
        }
        return 1; // spec: S-0501 claim 24 (entry begins with a 1-tick delay)
    }
    let pad = o
        .contact
        .filter(|&k| w.obj(k).is_some_and(|b| b.kind == Kind::Building));
    let Some(k) = pad else {
        // spec: S-0501 T3 and claim 46 (the pad is gone: the destination is cleared before the fall-back)
        set_dest(w, id, None);
        // spec: S-0501 claim 24 (a lost reservation: falls back to rest; the unlinked and fallback paths return 1 tick)
        fall_back(w, id);
        return 1; // spec: S-0501 claim 24 (the unlinked retry and fallback paths return 1 tick)
    };
    // spec: S-0501 claim 24 (each processing it asks to be loaded: the pad confirms the contact and the approach, or refuses)
    if !request_contact(w, id, k) {
        if let Some(ob) = w.obj_mut(id) {
            ob.contact = None;
            ob.econ.exit = None;
        }
        if o.beh.queued.is_none() {
            // spec: S-0501 T3 and claim 46 (a refusal clears the destination when no behaviour is queued; Open question 19 otherwise, where it is kept)
            set_dest(w, id, None);
        }
        fall_back(w, id);
        return 1; // spec: S-0501 claim 24 (the fallback path returns 1 tick)
    }
    let landing = st(w, id).landing;
    // spec: S-0501 claim 24 (the processing of enter that finds it landing while in contact advances to the docking stage; one already down on its pad does too, and is not moved)
    let here = (landing || on_ground(w, id)) && pad_cell(w, k, id);
    if st(w, id).enter_docking && here {
        if landing {
            // spec: S-0501 claim 24 (from the next processing until touchdown, each processing, 1 tick after the one before, moves it horizontally toward its docking position)
            docking_step(w, id, k);
        } else {
            // spec: S-0501 claim 24 (after touchdown it reports that it has entered; an aircraft already on the ground and not landing when it reaches this stage is not moved)
            dock(w, id, k);
        }
        return 1; // spec: S-0501 claim 24 (the docking and entered-report stage returns 1 tick)
    }
    if here {
        // spec: S-0501 claim 24 (the processing that finds it landing in contact only advances to the docking stage and returns 3 ticks)
        if let Some(a) = st_mut(w, id) {
            a.enter_docking = true;
        }
        return 3; // spec: S-0501 claim 24 (in contact during the approaching stage: 3 ticks)
    }
    if let Some(a) = st_mut(w, id) {
        a.enter_docking = false;
    }
    if o.dest != Some(Target::Object(k)) {
        // spec: S-0501 claim 24 (on the ground but not on its destination pad: it takes off toward the pad)
        set_dest(w, id, Some(Target::Object(k)));
    }
    // spec: S-0501 claim 24 (in contact during the approaching stage it returns 3 ticks, with no standard-delay draw)
    3
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
        // spec: S-0501 claim 34 (an absent destination is replaced by a new choice on the exit edge)
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
/// them (claim 12).
fn full_burst(w: &mut World, id: ObjectId, tg: Target) -> bool {
    let b = primary(w, id).map_or(1, |wp| w.rules().combat.weapon(wp).burst.max(1));
    let mut any = false;
    for _ in 0..b {
        if !crate::combat::fire::shot(w, id, 0, tg) {
            break;
        }
        any = true;
    }
    any
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
                    full_burst(w, id, tgv);
                    if drain_cleared(w, id, tgv) {
                        if let Some(a) = st_mut(w, id) {
                            // spec: S-0501 claim 15 and S-0608 claim 16 (an admitted initial pass with a draining weapon makes its run's round pending, though its attempts make no ordinary shot; target clearing does not roll it back)
                            a.pending = true;
                        }
                        return drain_departure(w, id);
                    }
                    fired(w, id, tgv);
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
                    if crate::combat::fire::shot(w, id, 0, tgv) || drains(w, id) {
                        if drain_cleared(w, id, tgv) {
                            return drain_departure(w, id);
                        }
                        fired(w, id, tgv);
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
                if crate::combat::fire::shot(w, id, 0, tgv) || drains(w, id) {
                    if drain_cleared(w, id, tgv) {
                        return drain_departure(w, id);
                    }
                    fired(w, id, tgv);
                }
                set_dest(w, id, Some(tgv));
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
