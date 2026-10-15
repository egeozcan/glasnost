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
use glasnost_rules::sim::LocomotionClass;
use serde::{Deserialize, Serialize};

use crate::combat::check::Answer;
use crate::rules::{Behaviour, Kind};
use crate::world::{Layer, ObjectId, Pos, Target, World};

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
    /// *V*, leptons per tick.
    speed: i32,
    /// *V*ₜ, the type's speed field.
    speed_field: i32,
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
        // spec: S-0501 claim 4 (V: the imported type speed, without bonuses)
        speed: u.speed,
        // spec: S-0501 F6 (Vₜ: the type's speed field; Q-0635)
        speed_field: u.raw.speed.unwrap_or(0),
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
    /// Drawn in the ground layer (Q-0632).
    pub ground: bool,
    /// The cell holding its object-present mark (S-0209 claim 1), while it
    /// is on the ground.
    pub mark: Option<CellXY>,
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
    /// Guidance turns toward the target rather than the destination (T1).
    pub face_target: bool,
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
    /// A pad building's servicing: the tick of its next service step.
    pub service: Option<i32>,
    /// The replacement target fraction for the next movement tick (claim 5).
    pub replaced: Option<F64>,
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

/// The height *h* above the terrain (S-0501 Terms). The bridge-deck rule is
/// a seam: no aircraft is flagged on a bridge.
pub fn height(w: &World, id: ObjectId) -> i32 {
    w.obj(id)
        .map_or(0, |o| o.pos.z - w.surface_at(o.pos.x, o.pos.y))
}

/// This spec's *on the ground* test (S-0501 Terms): *h* at most the
/// landing height 0, or drawn in the ground layer (Q-0632).
pub fn on_ground(w: &World, id: ObjectId) -> bool {
    // spec: S-0501 Terms (on the ground: h ≤ landing height 0, or in the ground layer)
    st(w, id).ground || height(w, id) <= 0
}

/// The in-the-air answer of an aircraft (S-0501 Terms): S-0307's
/// *airborne*.
pub fn in_the_air(w: &World, id: ObjectId) -> bool {
    crate::combat::weapon::airborne_by_height(w, id)
}

/// The passengers an aircraft carries. Seam: no object carries passengers
/// in our engine yet (S-0507).
fn passengers(_w: &World, _id: ObjectId) -> i32 {
    0
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

fn queue(w: &mut World, id: ObjectId, b: Behaviour) {
    if let Some(o) = w.obj_mut(id) {
        o.beh.queue(Some(b));
    }
}

fn forget(w: &mut World, id: ObjectId) {
    if let Some(o) = w.obj_mut(id) {
        o.target = None;
        o.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
        o.dest = None;
    }
}

fn set_dest(w: &mut World, id: ObjectId, d: Option<Target>) {
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
    // spec: S-0501 Terms (placement sets the visiting mark when the type is not both selectable and landable, or its primary weapon is a camera weapon; it never clears it)
    a.visiting |= !(t.selectable && t.landable) || t.camera;
    // spec: S-0501 claim 1 (a type placed carrying passengers is a passenger carrier for life)
    a.passenger_carrier |= passengers(w, id) > 0;
    a.entered = usable;
    let z = if ground {
        // Q-0632 (interim): ground placement at the terrain height
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
}

/// The marks an aircraft holds for removal (S-0209 claim 1): its
/// object-present mark while on the ground, none in flight. `None` for an
/// object that is not an aircraft of this spec.
pub fn removal_marks(w: &World, id: ObjectId) -> Option<Vec<(CellXY, Layer)>> {
    if !is_aircraft(w, id) {
        return None;
    }
    Some(
        st(w, id)
            .mark
            .map(|c| vec![(c, Layer::Ground)])
            .unwrap_or_default(),
    )
}

/// Clears an aircraft's object-present mark (going inside a carrier).
pub fn clear_mark(w: &mut World, id: ObjectId) {
    let Some(c) = w
        .obj_mut(id)
        .and_then(|o| o.air.as_mut())
        .and_then(|a| a.mark.take())
    else {
        return;
    };
    if let Some(rec) = w.cells.occ_mut(c, Layer::Ground) {
        rec.others = rec.others.saturating_sub(1);
    }
}

fn set_mark(w: &mut World, id: ObjectId) {
    let Some(c) = w.obj(id).map(|o| o.pos.cell()) else {
        return;
    };
    if st(w, id).mark.is_some() {
        return;
    }
    // spec: S-0209 claim 1 (an aircraft on the ground sets the object-present flag)
    if let Some(rec) = w.cells.occ_mut(c, Layer::Ground) {
        rec.others = rec.others.saturating_add(1);
    }
    if let Some(a) = st_mut(w, id) {
        a.mark = Some(c);
    }
}

/// A newly built aircraft stands docked at its pad (S-0404 claims 13 and
/// 15, S-0417 claim 11): settled on the ground at the pad position `p`.
pub fn released_docked(w: &mut World, id: ObjectId, p: Pos) {
    if !is_aircraft(w, id) {
        return;
    }
    clear_mark(w, id);
    let mut moved = false;
    if let Some(o) = w.obj_mut(id) {
        moved = o.pos.cell() != p.cell();
        let from = o.pos.cell();
        o.pos = p;
        if moved {
            w.cells.leave(from, Layer::Ground, id);
            w.cells.enter(p.cell(), Layer::Ground, id, false);
        }
    }
    let _ = moved;
    if let Some(a) = st_mut(w, id) {
        // Q-0632 (interim): a docked aircraft stands at its S-0404 claim 15 pad position, in the ground layer
        a.ground = true;
        a.held = 0;
        a.landing = false;
        a.taking_off = false;
        a.cur = F64::ZERO;
        a.tgt = F64::ZERO;
    }
    set_mark(w, id);
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
pub fn step(
    x: i32,
    y: i32,
    facing: Facing,
    v: i32,
    frac: F64,
    mode: glasnost_num::Rounding,
) -> (i32, i32) {
    // spec: S-0501 F3 (s = ⌊V × f⌋ in double precision, truncated toward zero)
    let s = F64::from_i64(i64::from(v), mode)
        .mul(frac, mode)
        .to_i32_common();
    // spec: S-0501 F3 (the table step of S-0208 F3 with θ = (Fₛ − 16383) × k, k = −2π / 65534)
    crate::walk::step_point(x, y, facing, s, true, mode)
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

/// F4: ordinary destination-dependent slowing. Returns (τ, adjusted
/// current fraction).
pub fn slowing(
    f0: F64,
    d: i32,
    s: i32,
    replacement: Option<F64>,
    r: &AirRules,
    mode: glasnost_num::Rounding,
) -> (F64, F64) {
    let one = F64::from_i64(1, mode);
    let df = F64::from_i64(i64::from(d), mode);
    // spec: S-0501 F4 (τ₀ = min(1, d / S); a slowdown distance of 0 or less gives no slowing); claim 5 (a replacement target from the previous tick stands in for it)
    let tau0 = if let Some(x) = replacement {
        x
    } else if s > 0 {
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

/// The flight point of a destination (Q-0631: the destination's own
/// point); for the aircraft's own pad contact, its pad position.
pub fn flight_point(w: &World, id: ObjectId, d: Target) -> Pos {
    if let Target::Object(k) = d
        && w.obj(id).is_some_and(|o| o.contact == Some(k))
        && w.obj(k).is_some_and(|b| b.kind == Kind::Building)
    {
        return crate::econ::release::pad_position(w, k, id).0;
    }
    // Q-0631 (interim): the destination's own point
    crate::process::target_pos(w, d)
}

/// The truncated table root of the planar squared difference (S-0204
/// claim 5).
fn planar(w: &World, a: Pos, b: Pos) -> i32 {
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

/// Whether the locomotor has stopped (Q-0631: current fraction 0 at the
/// held height, or on the ground).
pub fn stopped(w: &World, id: ObjectId) -> bool {
    let a = st(w, id);
    (a.cur.eq_num(F64::ZERO) && height(w, id) == a.held && !a.taking_off)
        || (a.ground && !a.taking_off)
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
    clear_mark(w, id);
    let dest = w.obj(id).and_then(|o| o.dest);
    if let Some(a) = st_mut(w, id) {
        // spec: S-0501 claim 8 (taking off: the held height becomes the cruise height)
        a.held = cruise;
        a.taking_off = true;
        a.landing = false;
        a.ground = false;
    }
    let Some(o) = w.obj(id).cloned() else { return };
    // spec: S-0501 claim 8 and claim 24 (standing on a pad, it leaves its reservation for any destination other than that pad); S-0417 claim 11 (the tether ends when it leaves the ground)
    if let Some(k) = o.contact
        && dest != Some(Target::Object(k))
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
    }
}

/// F7: the landing cell test at touchdown.
pub fn landing_accepted(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let c = o.pos.cell();
    // spec: S-0501 F7 (the touchdown position inside the usable area)
    if !w.cells.usable(c) {
        return false;
    }
    let t = ty(w, id).unwrap_or_default();
    // spec: S-0501 F7 (the ground cell layer, not the deck list; the first occupant found; Open question 7 for several)
    let found = w.cells.list(c, Layer::Ground).iter().copied().find(|&x| {
        x != id
            && w.obj(x).is_some_and(|ob| {
                matches!(
                    ob.kind,
                    Kind::Infantry | Kind::Vehicle | Kind::Aircraft | Kind::Building
                )
            })
    });
    match found {
        Some(x) => {
            // spec: S-0501 F7 (the lander's own contact is accepted)
            if o.contact == Some(x) {
                return true;
            }
            // spec: S-0501 F7 (a spawned lander and a found object with a spawn controller or of a spawned type)
            t.spawned
                && w.obj(x).is_some_and(|ob| {
                    ob.spawn.is_some()
                        || w.rules()
                            .unit(ob.kind, ob.type_index)
                            .is_some_and(|u| u.raw.spawned.unwrap_or(false))
                })
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
            // spec: S-0501 F7 (no other aircraft in play has this cell as its destination)
            let wanted = w.objects.values().any(|ob| {
                ob.id != id
                    && ob.kind == Kind::Aircraft
                    && ob.on_map
                    && ob.dest == Some(Target::Cell(c))
            });
            passable && empty && !wanted
        }
    }
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
    if let Some(ob) = w.obj_mut(id) {
        ob.pos.z = surface;
    }
    if let Some(a) = st_mut(w, id) {
        a.ground = true;
        a.landing = false;
        a.taking_off = false;
        a.held = 0;
        a.cur = F64::ZERO;
        a.tgt = F64::ZERO;
        a.flying_to = None;
    }
    set_mark(w, id);
    let pad = o
        .contact
        .filter(|&k| w.obj(k).is_some_and(|b| b.kind == Kind::Building) && pad_cell(w, k, id));
    set_dest(w, id, None);
    if let Some(k) = pad {
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

/// Docking at pad `k` (claim 24): put at the contact place's docking
/// position, then reports that it has entered.
fn dock(w: &mut World, id: ObjectId, k: ObjectId) {
    let (p, place) = crate::econ::release::pad_position(w, k, id);
    let facing = landing_facing(w, id);
    let t = w.tick;
    let keeps = w.rules().quirks.set_facing_keeps_target;
    released_docked(w, id, p);
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
        // Q-0632 (interim): the first service step at the pad's first turn at or after the report
        a.service = Some(t);
    }
}

/// Starts a crash (claim 9, S-0312 claim 16) of an aircraft above the
/// terrain: the tumble draws, then the fall counter. `living`: a crash
/// started without a kill (its locomotor switched off). Returns whether a
/// crash started.
pub fn start_crash(w: &mut World, id: ObjectId, living: bool) -> bool {
    if !is_aircraft(w, id) || height(w, id) <= 0 {
        return false;
    }
    if st(w, id).crash.is_some() {
        return true;
    }
    let mode = w.rules().mode;
    let half = w.rules().quirks.ranged_draw_half_span;
    // spec: S-0312 claim 16 (tumbling: one unit-interval value, one ranged draw over 0 to 1, one more unit-interval value)
    let _ = w.rng.unit(mode);
    let _ = w.rng.ranged(0, 1, half);
    let _ = w.rng.unit(mode);
    if let Some(a) = st_mut(w, id) {
        a.crash = Some(0);
        // Q-0635 (interim): a crash started without a kill has its locomotor switched off
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
    let Some(o) = w.obj(id).cloned() else { return };
    let surface = w.surface_at(o.pos.x, o.pos.y);
    if let Some(ob) = w.obj_mut(id) {
        ob.pos.z = surface;
    }
    if let Some(a) = st_mut(w, id) {
        a.ground = true;
    }
    let at = Pos::new(o.pos.x, o.pos.y, surface);
    if crate::combat::damage::dead(&o) {
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
        let surface = w.surface_at(o.pos.x, o.pos.y);
        if let Some(ob) = w.obj_mut(id) {
            ob.pos.z = surface + h1;
            ob.air = Some(a2.clone());
        }
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
    let dest = if dead { None } else { o.dest };
    let dp = dest.map(|d| flight_point(w, id, d));
    let here = o.pos;
    let d = dp.map(|p| planar(w, here, p));
    let dest_above = dp.map(|p| p.z - w.surface_at(p.x, p.y));
    // spec: S-0501 claim 6 (holding height: destination more than 120 leptons above terrain, a target with ammunition left, the run lock, or a type that is not landable)
    // spec: S-0501 claim 6 (an enter movement update clears the mark)
    let entering = o.beh.current == Some(Behaviour::Enter);
    a.holding = alive
        && !entering
        && (dest_above.is_some_and(|z| z > 120) // spec: S-0501 claim 6 (more than 120 leptons above terrain)
            || (o.target.is_some() && ammo_left(w, id))
            || a.run_lock
            || !t.landable);
    // Claim 8: a new destination makes an aircraft on the ground (or landing) take off.
    let new_dest = dest.is_some() && dest != a.flying_to;
    if alive && new_dest {
        a.flying_to = dest;
        if let Some(ob) = w.obj_mut(id) {
            ob.air = Some(a.clone());
        }
        if a.ground || a.landing {
            take_off(w, id, t.cruise);
        }
        a = st(w, id);
    }
    if alive && dest.is_none() {
        a.flying_to = None;
    }
    // The held height in ordinary flight (claim 6).
    if alive && !a.landing && !a.ground {
        a.held = t.cruise;
        // Q-0636 (interim): the destination height applies within 768 leptons only for a destination more than 120 leptons above the terrain
        if a.holding
            && d.is_some_and(|d| d < 768) // spec: S-0501 claim 6 (within 768 leptons of the destination)
            && let Some(z) = dest_above
            && z > HOLD_ABOVE
        {
            a.held = z;
        }
    }
    let h0 = height(w, id);
    // Requested heading (claims 4, 8 and T1).
    let cruise = t.cruise;
    let mut heading = None;
    if let Some(p) = dp {
        let aim = if a.face_target {
            o.target.map_or(p, |tg| crate::process::target_pos(w, tg))
        } else {
            p
        };
        heading = Some(toward(w, here, aim));
    }
    let mut full_request = false;
    if a.taking_off {
        let hh = cruise; // spec: S-0501 claim 8 (H = cruise height minus landing height 0)
        // spec: S-0501 claim 8 (above H − trunc(H / 3) the heading stays; from trunc(H / 2) up to that threshold it aims at the destination and requests full speed)
        if h0 > hh - hh / 3 {
            heading = None;
        } else if h0 >= hh / 2 {
            full_request = true;
        } else {
            // Q-0636 (interim): below trunc(H / 2) the heading stays
            heading = None;
        }
    }
    if a.ground && !a.taking_off {
        heading = None;
    }
    let tick_now = w.tick;
    if let Some(f) = heading
        && let Some(ob) = w.obj_mut(id)
    {
        // spec: S-0206 claim 1 (body turning)
        ob.body.turn_to(f, tick_now, t.turn_rate);
        ob.turret = ob.body;
    }
    // Horizontal motion (claim 4): forward along the body facing by ⌊V × f⌋.
    let mut removed = false;
    if !a.ground {
        let facing = w.obj(id).map_or(0, |ob| ob.body.value(tick_now));
        let (nx, ny) = step(here.x, here.y, facing, t.speed, a.cur, mode);
        if (nx, ny) != (here.x, here.y) {
            let mut np = Pos::new(nx, ny, here.z);
            let legal = |p: Pos| w.cells.cell(p.cell()).is_some();
            let may_leave = may_leave_now(w, id, &a);
            if !legal(np) && may_leave && !t.fly_by {
                // spec: S-0501 claim 4 (the position shifted 128 leptons in x back toward the map's middle)
                let middle = w.cells.footprint().width * 256 / 2; // spec: S-0204 claim 2 (256 leptons per cell)
                let back = if np.x > middle { -128 } else { 128 }; // spec: S-0501 Constants (edge push-back 128 leptons)
                let shifted = Pos::new(np.x + back, np.y, np.z);
                // Q-0633 (interim): no further fallback position; it does not move this tick
                np = if legal(shifted) { shifted } else { here };
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
        let nh = if h < a.held && alive {
            climb(h, a.held, passengers(w, id) > 0)
        } else if h > a.held {
            descend(h, a.held, dead)
        } else {
            h
        };
        let surface = w.surface_at(o.pos.x, o.pos.y);
        if let Some(ob) = w.obj_mut(id) {
            ob.pos.z = surface + nh;
        }
        if a.taking_off && nh >= a.held {
            a.taking_off = false;
        }
    }
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
            }
        }
        return w.obj(id).is_some_and(|ob| ob.pos != start);
    }
    let mut a = st(w, id);
    // Speed fraction (claim 5, F4), at the end of a living movement tick.
    if alive && !a.ground {
        let fixed = full_speed(w, id, &t, &a);
        let rep = a.replaced.take();
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
            let (tau, f) = slowing(a.cur, dd, t.slowdown, rep, ar, mode);
            a.tgt = tau;
            a.cur = f;
        } else {
            a.cur = approach_fraction(a.cur, a.tgt, ar, mode);
        }
        // spec: S-0501 claim 5 (the replacement target for the next movement tick, without the fixed-full-speed case or the fly-by flag)
        if let Some(p) = dp
            && !a.landing
            && !a.taking_off
            && !fixed
            && !t.fly_by
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
            a.replaced = replacement;
        }
        if let Some(ob) = w.obj_mut(id) {
            ob.air = Some(a.clone());
        }
        landing_trigger(w, id, &t, fixed);
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
        w.cells.leave(from, Layer::Ground, id);
        if w.cells.cell(to).is_some() {
            w.cells.enter(to, Layer::Ground, id, false);
        }
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
        // spec: S-0501 claim 8 (no destination in flight: it lands where it is)
        None => true,
        Some(dd) => {
            let p = flight_point(w, id, dd);
            let dist = planar(w, o.pos, p);
            // spec: S-0501 claim 6 (standing in its destination cell with a target fraction of 0)
            let exact = o.pos.cell() == p.cell() && a.tgt.eq_num(F64::ZERO);
            // spec: S-0501 claim 6 (close approach: below 128 leptons, ordinary slowing, current fraction below 0.05)
            let close = dist < 128 && !fixed && a.cur.lt(w.rules().air.twentieth);
            exact || close
        }
    };
    if !trigger {
        return;
    }
    // spec: S-0501 claim 6 (a further waypoint outside enter: wait is queued and it goes on)
    if o.beh.current != Some(Behaviour::Enter) && !o.queued_dests.is_empty() {
        let next = o.queued_dests[0];
        if let Some(ob) = w.obj_mut(id) {
            ob.queued_dests.remove(0);
            ob.beh.queue(Some(Behaviour::Wait));
        }
        set_dest(w, id, Some(next));
        return;
    }
    // spec: S-0501 claim 6 (a pad-bound aircraft whose cell holds no building it is in contact with falls back to rest instead)
    if t.bound && !o.contact.is_some_and(|k| pad_cell(w, k, id)) {
        fall_back(w, id);
        return;
    }
    start_landing(w, id);
}

// ---------------------------------------------------------------------------
// Pads (claims 21 to 28)

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

/// The landing spot without a pad (F8).
pub fn landing_spot(w: &mut World, id: ObjectId) -> Option<CellXY> {
    let o = w.obj(id)?.clone();
    let t = ty(w, id)?;
    let own = o.pos.cell();
    let zone = w.rules().unit(o.kind, o.type_index)?.zone;
    crate::regions::ensure(w);
    let settings = || crate::path::NearbySettings {
        // Q-0634 (interim): the nearby search with the track class, the type's movement zone, on the ground layer
        class: Some(LocomotionClass::Track),
        zone,
        reach: None,
        deck: false,
        height: false,
        reference: None,
        no_bridges: false,
        no_overlay: false,
    };
    let spot = |w: &World, c: CellXY| {
        crate::path::nearby(w, c, crate::path::NearbyUse::Settings(settings()))
    };
    if !t.docks.is_empty()
        && let Some(owner) = o.owner
    {
        let order = w.houses[usize::from(owner.0)].econ.acquired.clone();
        let mut best: Option<(i64, CellXY)> = None;
        let me = w.reference_point(id);
        for b in order {
            let Some(bo) = w.obj(b) else { continue };
            if bo.kind != Kind::Building || bo.owner != Some(owner) || !bo.on_map {
                continue;
            }
            let Some(s) = spot(w, bo.pos.cell()) else {
                continue;
            };
            let c = w.reference_point(b);
            let (dx, dy, dz) = (
                i64::from(c.x - me.x),
                i64::from(c.y - me.y),
                i64::from(c.z - me.z),
            );
            let q = w.rules().quirks;
            // spec: S-0501 F8 (the truncated table root of the spatial centre-to-centre squared difference)
            let mut d = root_floor(
                (dx * dx + dy * dy + dz * dz) as u64,
                q.table_root_distance,
                w.rules().mode,
            ) as i64;
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
            // spec: S-0501 F8 (below 256 leptons a free cell near its own cell; else the spot near the building)
            return if d < 256 {
                spot(w, own).or(Some(s))
            } else {
                Some(s)
            };
        }
    }
    // Q-0634 (interim): with no dock types or no building, the aircraft's own cell
    Some(own)
}

/// The edge cell of claim 34 (Q-0634: the nearest cell outside the usable
/// area in its own row or column, no draw). A legal cell is preferred,
/// since an aircraft that may leave the map is held back from illegal
/// cells (claim 4); with none, the first cell beyond the usable area.
pub fn edge_cell(w: &World, id: ObjectId) -> Option<CellXY> {
    let c = w.obj(id)?.pos.cell();
    let fp = w.cells.footprint();
    let limit = (fp.width + fp.height) * 2; // nocite: a search bound of our own, beyond any map's size
    let dirs = [(-1, 0), (1, 0), (0, -1), (0, 1)]; // nocite: the four axis directions in our own order (Q-0634)
    let mut legal: Option<(i32, CellXY)> = None;
    let mut any: Option<(i32, CellXY)> = None;
    for (dx, dy) in dirs {
        for n in 1..=limit {
            let p = CellXY::new(c.x + dx * n, c.y + dy * n);
            if !w.cells.usable(p) {
                // Q-0634 (interim): the nearest, the earlier direction on a tie
                if any.is_none_or(|(bn, _)| n < bn) {
                    any = Some((n, p));
                }
                if w.cells.cell(p).is_some() && legal.is_none_or(|(bn, _)| n < bn) {
                    legal = Some((n, p));
                }
                break;
            }
        }
    }
    legal.or(any).map(|(_, p)| p)
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
        if o.beh.current == Some(Behaviour::Enter) || !on_ground(w, x) || !pad_cell(w, k, x) {
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
            ob.target = tg;
            ob.dest = d;
            if ob.beh.current == Some(Behaviour::Patrol) {
                ob.beh.progress = 0;
            }
        }
        return true;
    }
    // spec: S-0501 T2 row 2 (retreat, the paradrop and spy-plane behaviours, not an airstrike aircraft: unchanged; no airstrike aircraft exists in our engine)
    if matches!(
        o.beh.current,
        Some(
            Behaviour::Retreat
                | Behaviour::ParadropApproach
                | Behaviour::ParadropOverfly
                | Behaviour::SpyPlaneApproach
                | Behaviour::SpyPlaneOverfly
        )
    ) {
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
    if on_ground(w, id) {
        if a.visiting && carrying {
            // spec: S-0501 T2 (on the ground, visiting, with passengers: unload)
            queue(w, id, Behaviour::Unload);
        } else if a.visiting {
            // spec: S-0501 T2 (on the ground, visiting, no passengers: forgets target and destination; retreat)
            forget(w, id);
            queue(w, id, Behaviour::Retreat);
        } else {
            forget(w, id);
            // spec: S-0501 T2 (on the ground otherwise: guard; area guard for an armed aircraft of a computer-controlled house)
            let b = if armed && !human(w, id) {
                Behaviour::AreaGuard
            } else {
                Behaviour::Guard
            };
            queue(w, id, b);
        }
    } else if carrying {
        // spec: S-0501 T2 (in flight with passengers: the landing spot; visiting: unload, else move)
        let s = landing_spot(w, id);
        set_dest(w, id, s.map(Target::Cell));
        queue(
            w,
            id,
            if a.visiting {
                Behaviour::Unload
            } else {
                Behaviour::Move
            },
        );
    } else if armed && a.visiting && !ammo {
        // spec: S-0501 T2 (in flight, armed, visiting, no ammunition: retreat)
        forget(w, id);
        queue(w, id, Behaviour::Retreat);
    } else if armed && a.visiting {
        // spec: S-0501 T2 (in flight, armed, visiting, ammunition left: hunt)
        queue(w, id, Behaviour::Hunt);
    } else if armed
        && (o.beh.queued == Some(Behaviour::Attack)
            || (o.beh.current == Some(Behaviour::Attack) && ammo && o.target.is_some()))
    {
        // spec: S-0501 T2 (in flight, armed, attack queued, or current with ammunition and a target: attack)
        queue(w, id, Behaviour::Attack);
    } else if armed
        && (!in_the_air(w, id)
            || (o.beh.current == Some(Behaviour::Enter)
                && o.dest
                    .is_some_and(|d| Some(d) == o.contact.map(Target::Object)))
            || t.docks.is_empty())
    {
        // spec: S-0501 T2 (in flight, armed: the in-the-air answer is no, already flying to its enter destination, or no dock types: guard)
        queue(w, id, Behaviour::Guard);
    } else if armed {
        if let Some(k) = seek_pad(w, id) {
            // spec: S-0501 T2 (a pad found that grants contact: the pad as destination; enter)
            set_dest(w, id, Some(Target::Object(k)));
            queue(w, id, Behaviour::Enter);
        } else if t.bound {
            // spec: S-0501 T2 (no pad, pad-bound: crash)
            start_crash(w, id, true);
            return;
        } else {
            let s = landing_spot(w, id);
            set_dest(w, id, s.map(Target::Cell));
            queue(w, id, Behaviour::Move);
        }
    } else {
        // spec: S-0501 T2 (in flight otherwise: the landing spot; move)
        let s = landing_spot(w, id);
        set_dest(w, id, s.map(Target::Cell));
        queue(w, id, Behaviour::Move);
    }
    // spec: S-0501 T2 (after the row: no ammunition, armed, no contact: a new pad search)
    let o2 = w.obj(id).cloned();
    if let Some(o2) = o2
        && armed
        && !ammo_left(w, id)
        && o2.contact.is_none()
    {
        if let Some(k) = seek_pad(w, id) {
            set_dest(w, id, Some(Target::Object(k)));
            if let Some(ob) = w.obj_mut(id) {
                ob.target = None;
                ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
            }
            queue(w, id, Behaviour::Enter);
        } else if t.bound && !on_ground(w, id) {
            start_crash(w, id, true);
            return;
        }
    }
    // spec: S-0501 T2 (after the row: an aircraft holding a contact queues enter)
    if w.obj(id).is_some_and(|x| x.contact.is_some()) {
        queue(w, id, Behaviour::Enter);
    }
    // spec: S-0501 claim 31 (the second start opportunity: the end of an ordinary fall-back)
    start_if_ready(w, id);
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
    // spec: S-0501 claim 32 (before processing, a target whose in-the-air answer is yes is cleared)
    if let Some(Target::Object(x)) = o.target
        && crate::combat::weapon::in_air(w, x)
        && let Some(ob) = w.obj_mut(id)
    {
        ob.target = None;
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
// spec: S-0501 T1 (approaching: at a distance of at least 512 leptons it turns toward the destination)
const STEER_FAR: i32 = 512;
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
        // spec: S-0215 claim 5 (sticky is processed as guard); area guard of a computer house's aircraft is processed as guard here (the original's computer-only choice, S-0501 claim 18)
        Some(Behaviour::Guard | Behaviour::Sticky | Behaviour::AreaGuard) => guard(w, id),
        Some(Behaviour::Hunt) => hunt(w, id),
        Some(Behaviour::Move) => move_(w, id),
        Some(Behaviour::Enter) => enter(w, id),
        Some(Behaviour::Retreat) => retreat(w, id),
        Some(Behaviour::SpyPlaneApproach) => spy_approach(w, id),
        Some(Behaviour::SpyPlaneOverfly) => spy_overfly(w, id),
        _ => NO_RULE_DELAY,
    }
}

/// *Move* (claim 33).
fn move_(w: &mut World, id: ObjectId) -> i32 {
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
    // spec: S-0501 claim 29 (an armed aircraft below cruise height, not full and without contact: a pad search; enter, target forgotten)
    if armed && !ground && h < t.cruise && !full_ammo(w, id) && o.contact.is_none() {
        if let Some(k) = seek_pad(w, id) {
            set_dest(w, id, Some(Target::Object(k)));
            if let Some(ob) = w.obj_mut(id) {
                ob.target = None;
                ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
            }
            queue(w, id, Behaviour::Enter);
            return standard(w, id);
        }
    }
    if !ground && h >= t.cruise {
        if o.dest.is_some() {
            // spec: S-0501 claim 29 (at cruise height with a destination: move)
            queue(w, id, Behaviour::Move);
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

/// *Enter* toward a pad (claim 24).
fn enter(w: &mut World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id).cloned() else {
        return NO_RULE_DELAY;
    };
    let pad = o
        .contact
        .filter(|&k| w.obj(k).is_some_and(|b| b.kind == Kind::Building));
    let Some(k) = pad else {
        // spec: S-0501 claim 24 (a lost reservation: falls back to rest)
        fall_back(w, id);
        return standard(w, id);
    };
    // spec: S-0501 claim 24 (each processing it asks to be loaded: the pad confirms the contact and the approach, or refuses)
    if !request_contact(w, id, k) {
        if let Some(ob) = w.obj_mut(id) {
            ob.contact = None;
            ob.econ.exit = None;
        }
        fall_back(w, id);
        return standard(w, id);
    }
    if on_ground(w, id) && pad_cell(w, k, id) {
        // Already standing on its pad: it reports that it has entered.
        dock(w, id, k);
        return standard(w, id);
    }
    if o.dest != Some(Target::Object(k)) {
        // spec: S-0501 claim 24 (on the ground but not on its destination pad: it takes off toward the pad)
        set_dest(w, id, Some(Target::Object(k)));
    }
    // Q-0635 (interim): enter returns the standard delay
    standard(w, id)
}

/// *Retreat* (claim 34).
fn retreat(w: &mut World, id: ObjectId) -> i32 {
    if w.obj(id).is_some_and(|o| o.dest.is_none()) {
        // spec: S-0501 claim 34 (no destination: a cell on its house's starting map edge; Q-0634)
        let c = edge_cell(w, id);
        set_dest(w, id, c.map(Target::Cell));
    }
    standard(w, id)
}

/// *Spy-plane approach* (claim 34).
fn spy_approach(w: &mut World, id: ObjectId) -> i32 {
    let (t, s) = (w.tick, stale(w));
    let Some(o) = w.obj(id).cloned() else {
        return NO_RULE_DELAY;
    };
    let frames = w.rules().air.camera_frames;
    let Some(tg) = o.target else {
        // spec: S-0501 claim 34 (no target: retreat starts)
        if let Some(ob) = w.obj_mut(id) {
            ob.beh.force(Behaviour::Retreat, t, s);
            ob.dest = None;
        }
        return frames;
    };
    let tp = crate::process::target_pos(w, tg);
    // spec: S-0501 claim 34 (the ground distance to the target below 769 leptons: overfly with the run lock and an edge destination)
    if planar(w, o.pos, tp) < 769 {
        let c = edge_cell(w, id);
        if let Some(ob) = w.obj_mut(id) {
            ob.beh.force(Behaviour::SpyPlaneOverfly, t, s);
        }
        set_dest(w, id, c.map(Target::Cell));
        if let Some(a) = st_mut(w, id) {
            a.run_lock = true;
        }
    } else {
        set_dest(w, id, Some(tg));
    }
    // spec: S-0501 claim 34 (the approach processing ends with the spy-plane camera frame count)
    frames
}

/// *Spy-plane overfly* (claim 34; the reveal belongs to a planned spec).
fn spy_overfly(w: &mut World, id: ObjectId) -> i32 {
    if w.obj(id).is_some_and(|o| o.dest.is_none()) {
        let c = edge_cell(w, id);
        set_dest(w, id, c.map(Target::Cell));
    }
    standard(w, id)
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

fn set_stage(w: &mut World, id: ObjectId, s: Stage) {
    if let Some(a) = st_mut(w, id) {
        a.stage = s;
    }
}

fn set_lock(w: &mut World, id: ObjectId, on: bool) {
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
    let threat = w.obj(id).map(|o| o.pos);
    // Q-0635 (interim): an unforced, not move-anywhere request with the aircraft's position as threat
    crate::scatter::cell_request(
        w,
        c,
        Layer::Ground,
        crate::scatter::ScatterRequest {
            threat,
            forced: false,
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

fn bare_rof(w: &World, id: ObjectId) -> i32 {
    primary(w, id).map_or(1, |wp| w.rules().combat.weapon(wp).rof)
}

/// F6: the wait after a strafing run's fifth shot.
pub fn strafe_wait(range: i32, speed_field: i32) -> i32 {
    // spec: S-0501 F6 (⌊(R + 1024) / Vₜ⌋; Q-0635: Vₜ at least 1)
    (range + 1024) / speed_field.max(1)
}

/// The processing of *attack* (table T1).
fn attack(w: &mut World, id: ObjectId) -> i32 {
    let start = w.obj(id).map_or(0, |o| o.beh.start_tick);
    let a = st(w, id);
    if a.run_of != Some(start) {
        if let Some(s) = st_mut(w, id) {
            s.run_of = Some(start);
            s.stage = Stage::Start;
        }
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
                if !is_strafer {
                    let half = w.rules().quirks.ranged_draw_half_span;
                    // spec: S-0501 claim 37 (a non-strafer finding a candidate: one U(0, 99) draw, before the standard delay)
                    let _ = w.rng.ranged(0, 99, half);
                }
                // spec: S-0501 claim 12 (a strafer's firing spot is the target); Q-0630 (interim: the target for every attacker)
                set_dest(w, id, Some(tg));
                if let Some(s) = st_mut(w, id) {
                    s.face_target = false;
                }
                set_stage(w, id, Stage::Approaching);
            } else {
                set_stage(w, id, Stage::Done);
            }
            Some(standard(w, id))
        }
        Stage::Approaching => {
            // spec: S-0501 T1 (approaching: the pending round spent, run lock off)
            if !a.run_lock || a.pending {
                set_lock(w, id, false);
                spend_pending(w, id);
            }
            let Some(tgv) = tg else {
                set_stage(w, id, Stage::Done);
                return Some(standard(w, id));
            };
            if !ammo_left(w, id) {
                set_stage(w, id, Stage::Done);
                return Some(standard(w, id));
            }
            let in_range = crate::combat::range::in_range(w, id, primary(w, id), Some(tgv), None);
            // spec: S-0501 T1 (a strafer within range, a fighter, or another aircraft whose locomotor has stopped: firing)
            if (is_strafer && in_range)
                || (!is_strafer && is_fighter)
                || (!is_strafer && !is_fighter && stopped(w, id))
            {
                set_stage(w, id, Stage::Firing);
                return None;
            }
            // The steer rows.
            let Some(d) = o.dest else {
                set_stage(w, id, Stage::Choosing);
                return None;
            };
            let p = flight_point(w, id, d);
            let dist = planar(w, o.pos, p);
            if dist >= STEER_FAR {
                // spec: S-0501 T1 (distance at least 512: turn toward the destination; 1 tick)
                if let Some(s) = st_mut(w, id) {
                    s.face_target = false;
                }
                return Some(1);
            }
            // spec: S-0501 T1 (below 512: turn toward the target; at least 16: 1 tick, else the destination is cleared and firing)
            if let Some(s) = st_mut(w, id) {
                s.face_target = true;
            }
            if dist >= STEER_NEAR {
                return Some(1);
            }
            set_dest(w, id, None);
            set_stage(w, id, Stage::Firing);
            None
        }
        Stage::Firing => {
            let Some(tgv) = tg else {
                set_stage(w, id, Stage::Done);
                return None;
            };
            if !ammo_left(w, id) {
                set_stage(w, id, Stage::Done);
                return None;
            }
            if !is_strafer {
                // spec: S-0501 T1 (a non-strafer turns its body to the target)
                let tp = crate::process::target_pos(w, tgv);
                let f = toward(w, o.pos, tp);
                let (tick, rate) = (w.tick, ty(w, id).map_or(0, |t| t.turn_rate));
                if let Some(ob) = w.obj_mut(id) {
                    ob.body.turn_to(f, tick, rate);
                    ob.turret = ob.body;
                }
            }
            let ans = crate::combat::check::check(w, id, Some(tgv), 0, true);
            match ans {
                Answer::CanFire => {
                    full_burst(w, id, tgv);
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
                // spec: S-0501 T1 (rearming: 1 tick; cloaked: uncloak, 1 tick; no object cloaks in our engine)
                Answer::Rearming | Answer::Cloaked => Some(1),
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
                set_stage(w, id, Stage::Done);
                return None;
            };
            let ans = crate::combat::check::check(w, id, Some(tgv), 0, true);
            match ans {
                Answer::CanFire => {
                    // spec: S-0501 T1 (refiring, ready: one shot, scatter)
                    if crate::combat::fire::shot(w, id, 0, tgv) {
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
                // spec: S-0501 claim 13 and T1 (later passes fire even when not facing, cloaked or out of range; one shot, scatter, the target becomes the destination)
                if crate::combat::fire::shot(w, id, 0, tgv) {
                    fired(w, id, tgv);
                }
                set_dest(w, id, Some(tgv));
                if n >= LAST_PASS {
                    // spec: S-0501 T1 (strafe pass 5: approaching; F6 ticks)
                    set_stage(w, id, Stage::Approaching);
                    let range = primary(w, id).map_or(0, |wp| w.rules().combat.weapon(wp).range);
                    let vt = ty(w, id).map_or(1, |t| t.speed_field);
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
            if let Some(s) = st_mut(w, id) {
                s.face_target = false;
            }
            run_end(w, id);
            Some(1)
        }
    }
}

/// "Not close enough" (Q-0630: the firing check's out-of-range answer).
fn not_close(w: &World, id: ObjectId, tg: Target) -> bool {
    !crate::combat::range::in_range(w, id, primary(w, id), Some(tg), None)
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
        ob.target = None;
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
    }
    // spec: S-0501 claim 17 (an airstrike aircraft with ammunition retreats; no airstrike exists in our engine; every other aircraft falls back to rest)
    set_stage(w, id, Stage::Start);
    fall_back(w, id);
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
    // spec: S-0501 claim 37 (smoke when u < 10 alive, u < 80 dead); the smoke animation is presentation and not created here
    let _shown = if dead { u < 80 } else { u < 10 };
}

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
    // spec: S-0501 T4 (receive the controller target and queue attack)
    queue(w, x, Behaviour::Attack);
}

/// A slot aircraft returns: target cleared, the spawner as destination,
/// *move* (T4).
pub fn carrier_return(w: &mut World, x: ObjectId, spawner: ObjectId) {
    if let Some(o) = w.obj_mut(x) {
        o.target = None;
        o.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
    }
    set_dest(w, x, Some(Target::Object(spawner)));
    // spec: S-0501 T4 (clear target, spawner as destination, move)
    queue(w, x, Behaviour::Move);
}
