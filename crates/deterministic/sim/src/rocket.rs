//! Rocket flight (S-0506 claims 14 to 21): the rocket-flight movement mode
//! of a spawned missile, from its aim point to its detonation.
//!
//! A rocket missile is an aircraft-kind object placed by its spawner's
//! spawn controller ([`place`], S-0506 claim 7). Each of its turns runs
//! [`tick`]: the first one with a destination fixes the aim point, then
//! the phases of the Rocket table run, one tick at a time.
//!
//! A rocket missile is in no cell list and marks no occupancy; it is
//! found only through the flying-object search once tracked (claim 23). A
//! destroyed one runs its phases as an undamaged one and moves only by F1
//! (claim 20). The phase boundaries are claim 24's; the raw arctangent of
//! F6 and F7 is S-0204 F7's exact table.

use glasnost_num::geom::{direction, direction_constants, root_floor, table_root_real};
use glasnost_num::trig::{cos_tb, sin_tb};
use glasnost_num::{F32, F64, Rounding};
use serde::{Deserialize, Serialize};

use crate::combat::effects::{self, Effect, Sound};
use crate::rules::Kind;
use crate::spawn::{RocketSet, RocketSetId};
use crate::world::{ObjectId, Pos, Target, World};

/// A flight phase (S-0506 Terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// Standing still for the pause time.
    Pause,
    /// Turning the pitch from Pᵢ to P_f.
    Tilt,
    /// Climbing by the raise rate (set C after a pause).
    Raise,
    /// Accelerating along the pitch to the cruise altitude.
    Ascent,
    /// Levelling out or following the lazy curve.
    Cruise,
    /// Steering the nose onto the aim point.
    Descent,
}

/// A rocket missile's flight state (S-0506 Terms).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RocketState {
    /// The parameter set.
    pub set: RocketSetId,
    /// The spawner that launched it.
    pub spawner: Option<ObjectId>,
    /// The phase; `None` before the aim point is fixed.
    pub phase: Option<Phase>,
    /// The tick the current pause, tilt or raise began: claim 24's timer
    /// runs on the tick number, so its elapsed ticks are the tick less this
    /// one (S-0506 claim 14, r34).
    pub started: i32,
    /// The pitch p (single precision, radians).
    pub pitch: F32,
    /// The rocket speed v (double precision, leptons per tick).
    pub speed: F64,
    /// The fixed aim point.
    pub aim: Option<Pos>,
    /// The horizontal distance d₀ recorded at the start of cruise.
    pub d0: i32,
    /// The elite launch flag, read in every pause, tilt and raise tick.
    pub elite: bool,
    /// The tick of the next cosmetic animation, the timer shared by the
    /// trailer and the set-C take-off animations (claim 21).
    pub anim_next: i32,
}

fn set_of(w: &World, s: RocketSetId) -> RocketSet {
    *w.rules().missiles.set(s)
}

/// π/2 as a double (S-0204 F3 c₁).
fn half_pi() -> F64 {
    direction_constants().0
}

/// −2π / 65536 as a double: −4 c₁ / 65536, exact (S-0506 F1).
fn facing_radians() -> F64 {
    let rz = Rounding::TowardZero;
    // spec: S-0506 F1 (θ = (F − 16383) × (−2π / 65536); 2π = 4 × π/2, both factors powers of two, exact)
    half_pi()
        .mul(F64::from_i64(4, rz), rz)
        .div(F64::from_i64(65536, rz), rz)
        .negated()
}

/// θ of S-0506 F1 for a body facing.
fn theta(f: u16, mode: Rounding) -> F64 {
    // spec: S-0506 F1 (F read as a signed 16-bit value; the heading constant 16383)
    let signed = i64::from(f as i16);
    F64::from_i64(signed - 16383, mode).mul(facing_radians(), mode) // spec: S-0506 F1 and Constants (rocket heading constant 16383)
}

fn fi(v: i64, mode: Rounding) -> F64 {
    F64::from_i64(v, mode)
}

fn trunc(x: F64) -> i64 {
    x.trunc_i128().unwrap_or(0) as i64
}

/// The step of S-0506 Formulas F1: the new position for speed `v`, pitch
/// `p` and body facing `f`.
pub fn step(pos: Pos, v: F64, p: F32, f: u16, mode: Rounding) -> Pos {
    step_b(pos, v, p, f, mode, true)
}

/// [`step`] with `quirks.table_trig_negative_bias` at `bias` (S-0105 F1).
pub fn step_b(pos: Pos, v: F64, p: F32, f: u16, mode: Rounding, bias: bool) -> Pos {
    let cos_t = |x: F64| cos_tb(x, mode, bias);
    let sin_t = |x: F64| sin_tb(x, mode, bias);
    // spec: S-0506 F1 (s = trunc(v); h = trunc(cos(p) × s); x′ = trunc(x + h cos θ), y′ = trunc(y − h sin θ), z′ = trunc(z + sin(p) × s))
    let s = trunc(v);
    let pd = p.to_f64();
    let h = trunc(cos_t(pd).mul(fi(s, mode), mode));
    let th = theta(f, mode);
    let x = trunc(fi(pos.x.into(), mode).add(fi(h, mode).mul(cos_t(th), mode), mode));
    let y = trunc(fi(pos.y.into(), mode).sub(fi(h, mode).mul(sin_t(th), mode), mode));
    let z = trunc(fi(pos.z.into(), mode).add(sin_t(pd).mul(fi(s, mode), mode), mode));
    Pos::new(x as i32, y as i32, z as i32)
}

/// The nose point of S-0506 Formulas F9.
pub fn nose(pos: Pos, length: i32, p: F32, f: u16, mode: Rounding) -> Pos {
    nose_b(pos, length, p, f, mode, true)
}

/// [`nose`] with `quirks.table_trig_negative_bias` at `bias` (S-0105 F1).
pub fn nose_b(pos: Pos, length: i32, p: F32, f: u16, mode: Rounding, bias: bool) -> Pos {
    let cos_t = |x: F64| cos_tb(x, mode, bias);
    let sin_t = |x: F64| sin_tb(x, mode, bias);
    // spec: S-0506 F9 (b = trunc(L × cos(p)); nose = (trunc(x + b cos θ), trunc(y − b sin θ), trunc(z + L × sin(p))))
    let pd = p.to_f64();
    let l = fi(length.into(), mode);
    let b = trunc(l.mul(cos_t(pd), mode));
    let th = theta(f, mode);
    let x = trunc(fi(pos.x.into(), mode).add(fi(b, mode).mul(cos_t(th), mode), mode));
    let y = trunc(fi(pos.y.into(), mode).sub(fi(b, mode).mul(sin_t(th), mode), mode));
    let z = trunc(fi(pos.z.into(), mode).add(l.mul(sin_t(pd), mode), mode));
    Pos::new(x as i32, y as i32, z as i32)
}

/// The pitch P × π/2 narrowed once (S-0506 F3: α and β).
fn quarter_turns(p: F32, mode: Rounding) -> F64 {
    p.to_f64().mul(half_pi(), mode)
}

/// The tilt pitch of S-0506 Formulas F3 after `e` of `tt` ticks.
pub fn tilt_pitch(pi: F32, pf: F32, e: i32, tt: i32, mode: Rounding) -> F32 {
    let a = quarter_turns(pi, mode);
    let b = quarter_turns(pf, mode);
    // spec: S-0506 F3 (a zero tilt time takes the endpoint without division; at the end p = β)
    if tt == 0 || e >= tt {
        return b.to_f32(mode);
    }
    // spec: S-0506 F3 (p = α + (β − α) × u, u = e / Tₜ, stored in single precision)
    let u = fi(e.into(), mode).div(fi(tt.into(), mode), mode);
    a.add(b.sub(a, mode).mul(u, mode), mode).to_f32(mode)
}

/// The speed of S-0506 Formulas F4.
pub fn next_speed(v: F64, a: F32, cap: i32, mode: Rounding) -> F64 {
    // spec: S-0506 F4 (vₙ₊₁ = min(RZ₅₃(vₙ + a), V))
    v.add(a.to_f64(), mode).min(fi(cap.into(), mode))
}

/// The plain cruise pitch of S-0506 Formulas F5.
pub fn plain_pitch(p: F32, tau: F32, mode: Rounding) -> F32 {
    let zero = F64::ZERO;
    // spec: S-0506 F5 (a nonpositive pitch is unchanged)
    if !p.to_f64().gt(zero) {
        return p;
    }
    // spec: S-0506 F5 (p′ = max(0, RZ₂₄(RZ₅₃(p − τ))))
    let q = p.to_f64().sub(tau.to_f64(), mode).to_f32(mode);
    if q.to_f64().lt(zero) { F32(0) } else { q }
}

/// The raw table arctangent A of S-0506 Formulas F6, from S-0204 F7's
/// exact raw angles (no facing correction).
pub fn raw_atan(u: F64, mode: Rounding) -> F64 {
    // spec: S-0204 F3 (σ = 3276705 / 2^27)
    let sigma = F64::from_parts(false, 3276705, -27, Rounding::TowardZero);
    // spec: S-0506 F6 (the entry numbered by |trunc(RZ₅₃(u / σ))|, selected before the sign is narrowed; the sign from RZ₂₄(u); past the last entry of S-0204 claim 13's 4097-entry table, the single π/2)
    let i = u.div(sigma, mode).trunc_i128().map(i128::unsigned_abs);
    let mag = match i {
        Some(i) if i <= 4096 => F32(glasnost_num::tables::RAW_ATAN[i as usize]).to_f64(), // spec: S-0506 F6 and S-0204 claim 13 (the exact raw entry; the table's 4097 entries are numbered 0 to 4096)
        _ => half_pi().to_f32(mode).to_f64(),
    };
    if u.to_f32(mode).to_f64().is_sign_negative() {
        mag.negated()
    } else {
        mag
    }
}

/// ℓ of S-0506 Formulas F6 and F7: the line-of-sight angle, from the
/// integer squared-distance domain.
fn line_of_sight(d: (i32, i32, i32), table: bool, mode: Rounding) -> F64 {
    // spec: S-0506 F6 (nᵢ = wrap(wrap(Dₓ × Dₓ) + wrap(D_y × D_y)), ρ = r(nᵢ) after exact widening)
    let ni = d.0.wrapping_mul(d.0).wrapping_add(d.1.wrapping_mul(d.1));
    let rho = table_root_real(fi(ni.into(), mode), table, mode);
    // spec: S-0506 F6 (ℓ = A(RZ₅₃(D_z / ρ)) when ρ > 0, otherwise −π/2)
    if rho.gt(F64::ZERO) {
        raw_atan(fi(d.2.into(), mode).div(rho, mode), mode)
    } else {
        half_pi().negated()
    }
}

/// r(n_d) of S-0506 Formulas F6: the table root of the double sum of the
/// double squares.
fn root_nd(dx: i32, dy: i32, table: bool, mode: Rounding) -> F64 {
    // spec: S-0506 F6 (n_d = RZ₅₃(RZ₅₃(Dₓ²) + RZ₅₃(D_y²)) in double precision)
    let x = fi(dx.into(), mode);
    let y = fi(dy.into(), mode);
    let nd = x.mul(x, mode).add(y.mul(y, mode), mode);
    table_root_real(nd, table, mode)
}

/// The lazy curve pitch of S-0506 Formulas F6.
pub fn lazy_pitch(d: (i32, i32, i32), d0: i32, pf: F32, table: bool, mode: Rounding) -> F32 {
    let one = fi(1, mode);
    // spec: S-0506 F6 (q = RZ₅₃(trunc(r(n_d)) / d₀))
    let r = trunc(root_nd(d.0, d.1, table, mode));
    let q = fi(r, mode).div(fi(d0.into(), mode), mode);
    let l = line_of_sight(d, table, mode);
    // spec: S-0506 F6 (p′ = RZ₂₄(RZ₅₃(RZ₅₃(ℓ × RZ₅₃(1 − q)) + RZ₅₃(RZ₅₃(P_f × q) × π/2))))
    let a = l.mul(one.sub(q, mode), mode);
    let b = pf.to_f64().mul(q, mode).mul(half_pi(), mode);
    a.add(b, mode).to_f32(mode)
}

/// The descent pitch of S-0506 Formulas F7.
pub fn descent_pitch(d: (i32, i32, i32), p: F32, tau: F32, table: bool, mode: Rounding) -> F32 {
    let l = line_of_sight(d, table, mode);
    let pd = p.to_f64();
    let t = tau.to_f64();
    // spec: S-0506 F7 (δ = RZ₅₃(ℓ − p); |δ| ≤ τ: p + δ; otherwise p + τ × sign(δ))
    let delta = l.sub(pd, mode);
    if delta.abs().le(t) {
        pd.add(delta, mode).to_f32(mode)
    } else if delta.is_sign_negative() {
        pd.add(t.negated(), mode).to_f32(mode)
    } else {
        pd.add(t, mode).to_f32(mode)
    }
}

/// Whether a position is in a legal cell (S-0506 claim 18).
fn legal(w: &World, p: Pos) -> bool {
    w.cells.cell(p.cell()).is_some()
}

/// A position's height above the terrain.
fn height_above_terrain(w: &World, p: Pos) -> i32 {
    p.z - w.surface_at(p.x, p.y)
}

/// What the drawing of a rocket missile reads, as single-precision bit
/// patterns (S-0105 claim 41; read-only, the simulation holds no floats
/// outside its soft types).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Drawing {
    /// The current pitch *p*, radians, positive nose up (S-0506 claims 14
    /// to 18 and 24).
    pub pitch: u32,
    /// The selected set's initial pitch fraction *Pᵢ*, quarter turns.
    pub initial: u32,
    /// The selected set's final pitch fraction *P𝒇*, quarter turns.
    pub final_: u32,
}

/// The drawing inputs of rocket missile `id` (S-0105 claim 41), `None` for
/// an object that is not a rocket missile.
pub fn drawing(w: &World, id: ObjectId) -> Option<Drawing> {
    let rs = w.obj(id)?.rocket.as_ref()?;
    let set = set_of(w, rs.set);
    Some(Drawing {
        pitch: rs.pitch.0,
        initial: set.pitch_initial.0,
        final_: set.pitch_final.0,
    })
}

/// Places a launched rocket missile (S-0506 claim 7): on the map at
/// `pos` with `facing`, in the update order, in the rocket-flight mode of
/// its parameter set. It enters no cell list (claim 23).
pub fn place(w: &mut World, id: ObjectId, pos: Pos, facing: u16, spawner: ObjectId) {
    let t = w.tick;
    let keeps = w.rules().quirks.set_facing_keeps_target;
    let set = crate::spawn::rocket_type(w, id).unwrap_or_default();
    let Some(o) = w.obj_mut(id) else { return };
    o.pos = pos;
    o.on_map = true;
    // spec: S-0206 claim 5 (placing sets body and turret facings at once to the placement facing)
    o.body.set(facing, t, keeps);
    o.turret.set(facing, t, keeps);
    o.rocket = Some(RocketState {
        set,
        spawner: Some(spawner),
        anim_next: t,
        ..Default::default()
    });
    w.update.enter(id);
    crate::econ::events::entered_map(w, id, false);
}

/// Whether a rocket missile hit by a killing hit keeps flying (S-0506
/// claim 20): it is at least 1 lepton above the terrain and
/// `quirks.destroyed_rocket_still_detonates` is on.
pub fn keeps_flying_when_destroyed(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    o.rocket.is_some()
        && w.rules().quirks.destroyed_rocket_still_detonates
        // spec: S-0506 claim 20 and Constants (above the terrain: height 1 lepton or more)
        && height_above_terrain(w, o.pos) >= 1
}

/// The aim point of S-0506 claim 14: the destination's heading point.
pub fn heading_point(w: &World, dest: Target) -> Option<Pos> {
    heading_point_with(w, dest, w.rules().quirks.rocket_aim_at_heading_point)
}

/// The heading point of S-0506 claim 14; `moving_point` off samples a
/// unit's position instead of the point its locomotor moves it to
/// (`quirks.rocket_aim_at_heading_point` off, for a rocket's aim).
pub fn heading_point_with(w: &World, dest: Target, moving_point: bool) -> Option<Pos> {
    match dest {
        // spec: S-0506 claim 14 (a cell: its centre point, S-0204)
        Target::Cell(c) => Some(Pos::centre(
            c,
            w.surface_at(c.x * 256 + 128, c.y * 256 + 128),
        )), // spec: S-0204 claim 2 (the cell centre, 128 leptons in)
        Target::Object(x) => {
            let o = w.obj(x)?;
            if o.kind == Kind::Building {
                // spec: S-0506 claim 14 (a building: its centre, position + 128 × (W − 1), 128 × (H − 1)); the docking-point cases are a seam
                return Some(w.reference_point(x));
            }
            if !moving_point {
                // spec: S-0506 Edge cases (quirks.rocket_aim_at_heading_point off: the unit's position)
                return Some(o.pos);
            }
            // spec: S-0506 claim 14 (a unit: the point its locomotor is moving it to, the end of its step; its position when not moving); the tunnel case is a seam
            if let Some(tr) = o.drive.as_ref().and_then(|d| d.track.as_ref()) {
                let (x, y) = tr.target;
                return Some(Pos::new(x, y, w.surface_at(x, y)));
            }
            if let Some(p) = o.walk.as_ref().and_then(|s| s.target) {
                return Some(p);
            }
            // spec: S-0506 claim 14 (r41, Q-3044: a hovering vehicle's heading point is its locomotor's point P, S-0503 Terms, whenever it holds one; power, throttle and moving play no part, so a switched-off vehicle that keeps P, S-0503 claim 10, gives P; holding none, its position) (#191)
            if let Some(p) = o.hover.as_ref().and_then(|h| h.point) {
                return Some(p);
            }
            Some(o.pos)
        }
    }
}

/// Fixes a rocket missile's aim point and starts its flight at `t`: the
/// first phase, claim 24's timer, the initial pitch and zero speed.
fn start_flight(rs: &mut RocketState, aim: Pos, p: &RocketSet, t: i32, mode: Rounding) {
    rs.aim = Some(aim);
    // spec: S-0506 Rocket table (the flight begins in pause, or tilt when the pause time is 0, including set C; pitch = Pᵢ × π/2)
    rs.phase = Some(if p.pause > 0 {
        Phase::Pause
    } else {
        Phase::Tilt
    });
    // spec: S-0506 claim 24 (the timer starts in t₀ with zero elapsed ticks)
    rs.started = t;
    rs.pitch = quarter_turns(p.pitch_initial, mode).to_f32(mode);
    rs.speed = F64::ZERO;
}

/// Fixes a launched rocket missile's aim point at its launch (S-0506
/// claim 14, r34), from the heading point of `dest` at that moment, and
/// starts its flight in the launch tick (claim 24). For a destination whose
/// in-the-air answer is false at the launch; the caller decides that.
/// [`give_destination`] uses it for a later fixing (r37), in its own tick. No
/// effect on an object that is not a rocket missile, on one whose aim is
/// already fixed, or when `dest` gives no heading point.
pub fn fix_at_launch(w: &mut World, id: ObjectId, dest: Target) {
    let Some(rs) = w.obj(id).and_then(|o| o.rocket.as_ref()) else {
        return;
    };
    if rs.phase.is_some() {
        return;
    }
    let p = set_of(w, rs.set);
    let Some(aim) = heading_point(w, dest) else {
        return;
    };
    let (t, mode) = (w.tick, w.rules().mode);
    if let Some(rs) = w.obj_mut(id).and_then(|o| o.rocket.as_mut()) {
        // spec: S-0506 claim 14 (r34: the aim point is fixed at the launch, in the launch pass, in the tick of the placement; it does not change whatever the destination does afterwards)
        start_flight(rs, aim, &p, t, mode);
    }
}

/// A destination given to a launched rocket missile after its launch
/// (S-0506 claim 14, r37). One whose in-the-air answer is true is ignored,
/// so it never fixes an aim point. The first one whose answer is false
/// becomes the missile's destination and fixes its aim point from its
/// heading point at that moment, starting claim 24's timer in this tick. A
/// missile whose aim is already fixed ignores it, as any later move order.
/// No effect on an object that is not a rocket missile.
pub fn give_destination(w: &mut World, id: ObjectId, dest: Target) {
    let Some(rs) = w.obj(id).and_then(|o| o.rocket.as_ref()) else {
        return;
    };
    // spec: S-0506 claim 14 (r37: later move orders are ignored once the aim point is fixed)
    if rs.phase.is_some() {
        return;
    }
    // spec: S-0506 claim 14 (r37: a destination whose in-the-air answer is true leaves it without a destination; a destination in the air never fixes an aim point)
    if let Target::Object(d) = dest
        && crate::combat::weapon::in_air(w, d)
    {
        return;
    }
    if heading_point(w, dest).is_none() {
        return;
    }
    if let Some(o) = w.obj_mut(id) {
        o.dest = Some(dest);
    }
    // spec: S-0506 claim 14 (r37: the first destination not in the air fixes the aim point from its heading point at that moment); claim 24 (t₀ the tick of that fixing)
    fix_at_launch(w, id, dest);
}

/// A rocket missile's turn (S-0506 claims 14 to 21); whether it moved.
pub fn tick(w: &mut World, id: ObjectId) -> bool {
    let rules = w.rules_arc();
    let mode = rules.mode;
    let q = rules.quirks;
    let table = q.table_root_distance;
    let t = w.tick;
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    let Some(mut rs) = o.rocket.clone() else {
        return false;
    };
    let p = set_of(w, rs.set);
    let Some(u) = rules.unit(o.kind, o.type_index) else {
        return false;
    };
    let cap = u.speed;
    let omega = u.turn_rate;
    let trailer = u.raw.trailer.clone();
    // spec: S-0506 claim 14 (r37: a missile with no aim point makes no flight; it stays at its placement point, in none of claim 15's phases, with speed 0 and pitch 0, until give_destination fixes its aim)
    let Some(phase) = rs.phase else {
        return false;
    };
    let aim = rs.aim.unwrap_or(o.pos);
    let f = o.body.value(t);
    let mut detonate = false;
    match phase {
        Phase::Pause | Phase::Tilt | Phase::Raise => {
            // spec: S-0506 claim 19 (the elite launch is read in each pause, tilt and raise tick: the spawner exists and is elite)
            rs.elite = crate::spawn::launcher_elite(w, rs.spawner);
            // spec: S-0506 claim 14 (r34: claim 24's timer runs on the tick number, S-0203 claim 9, so a turn the missile does not make does not shift its ticks; its next update finds the timer run on)
            let elapsed = t.wrapping_sub(rs.started);
            match phase {
                Phase::Pause => {
                    if rs.set == RocketSetId::C {
                        set_c_anim(w, &mut rs, t);
                    }
                    if elapsed >= p.pause {
                        // spec: S-0506 Rocket table (pause over: tilt for sets A and B, raise for set C)
                        rs.phase = Some(if rs.set == RocketSetId::C {
                            Phase::Raise
                        } else {
                            Phase::Tilt
                        });
                        // spec: S-0506 claim 14 (r34: a phase's end is acted on at the first update at or after its timer runs out, and the next phase's timer starts at that update)
                        rs.started = t;
                    }
                }
                Phase::Tilt => {
                    // spec: S-0506 Rocket table (tilt: pitch by F3 each tick); claim 14 (r34: F3's pitch from the elapsed ticks, so an update lost in tilt leaves the next one's pitch as without the loss)
                    rs.pitch = tilt_pitch(p.pitch_initial, p.pitch_final, elapsed, p.tilt, mode);
                    if elapsed >= p.tilt {
                        end_of_tilt(w, id, &mut rs, &p, mode);
                    }
                }
                _ => {
                    set_c_anim(w, &mut rs, t);
                    if elapsed >= p.tilt {
                        // spec: S-0506 claim 24 (raise adds its rate only on updates before expiry: T − 1 increments)
                        end_of_tilt(w, id, &mut rs, &p, mode);
                    } else {
                        // spec: S-0506 Rocket table (raise: world height + raise rate, when that position is legal)
                        let up = Pos::new(o.pos.x, o.pos.y, o.pos.z.wrapping_add(p.raise_rate));
                        if legal(w, up)
                            && let Some(ob) = w.obj_mut(id)
                        {
                            ob.pos = up;
                        }
                    }
                }
            }
        }
        Phase::Ascent => {
            // spec: S-0506 Rocket table (ascent: speed by F4; cruise at the cruise altitude above the terrain, recording d₀)
            // spec: S-0506 claim 24 (ascent tests the altitude before F1 moves it; the update first finding cruise altitude records d₀ before that movement, still accelerates and moves with ascent pitch, and applies no cruise steering)
            if height_above_terrain(w, o.pos) >= p.altitude {
                rs.phase = Some(Phase::Cruise);
                let (dx, dy) = (aim.x - o.pos.x, aim.y - o.pos.y);
                // spec: S-0506 F6 (d₀ = trunc(r(n_d)))
                rs.d0 = trunc(root_nd(dx, dy, table, mode)) as i32;
            }
            rs.speed = next_speed(rs.speed, p.acceleration, cap, mode);
        }
        Phase::Cruise => {
            let pos = o.pos;
            let d = (aim.x - pos.x, aim.y - pos.y, aim.z - pos.z);
            // spec: S-0506 claim 17 (in cruise the body turns towards the direction to the aim point at the type's turn rate)
            let dir = direction(d.0.into(), d.1.into(), q.table_direction);
            if let Some(ob) = w.obj_mut(id) {
                ob.body.turn_to(dir, t, omega);
            }
            // spec: S-0506 claim 19 and Constants (cruise, any mode: height above the terrain below 1 lepton)
            if height_above_terrain(w, pos) < 1 {
                detonate = true;
            } else if !p.lazy || rs.d0 == 0 {
                // spec: S-0506 claim 17 (plain cruise: F4, F5, descent once ⌊r(dx² + dy²)⌋ ≤ z − z_aim)
                rs.speed = next_speed(rs.speed, p.acceleration, cap, mode);
                rs.pitch = plain_pitch(rs.pitch, p.turn_rate, mode);
                let n = i64::from(d.0) * i64::from(d.0) + i64::from(d.1) * i64::from(d.1);
                let horizontal = root_floor(n as u64, table, mode) as i64;
                if horizontal <= i64::from(pos.z) - i64::from(aim.z) {
                    rs.phase = Some(Phase::Descent);
                }
            } else if contact(w, pos, aim, &rs, mode) {
                // spec: S-0506 Rocket table (lazy cruise: the claim 19 test detonates)
                detonate = true;
            } else {
                // spec: S-0506 Rocket table (lazy cruise: speed by F4, pitch by F6)
                rs.speed = next_speed(rs.speed, p.acceleration, cap, mode);
                rs.pitch = lazy_pitch(d, rs.d0, p.pitch_final, table, mode);
            }
        }
        Phase::Descent => {
            let pos = o.pos;
            let d = (aim.x - pos.x, aim.y - pos.y, aim.z - pos.z);
            if contact(w, pos, aim, &rs, mode) {
                // spec: S-0506 Rocket table (descent: the claim 19 test detonates)
                detonate = true;
            } else {
                // spec: S-0506 Rocket table (descent: pitch by F7, speed unchanged)
                rs.pitch = descent_pitch(d, rs.pitch, p.turn_rate, table, mode);
            }
        }
    }
    if detonate {
        store(w, id, rs);
        detonation(w, id);
        return false;
    }
    let mut moved = false;
    // spec: S-0506 claim 18 (every tick with a rocket speed above 0: movement by F1)
    if rs.speed.gt(F64::ZERO) {
        let pos = w.obj(id).map_or(o.pos, |ob| ob.pos);
        let f = w.obj(id).map_or(f, |ob| ob.body.value(t));
        let next = step_b(pos, rs.speed, rs.pitch, f, mode, crate::table_trig::bias(w));
        if legal(w, next) {
            if let Some(ob) = w.obj_mut(id) {
                ob.pos = next;
            }
            if next.cell() != pos.cell() {
                // spec: S-0516 claim 22 and Edge cases (rocket flight supplies no cell-arrival update; quirks.flying_sensor_disc_stationary off: a cell-boundary crossing moves the sensor disc, with no adjacency test)
                crate::cloak::machine::flight_cell_crossed(w, id);
            }
            moved = next != pos;
            crate::combat::flying::moved(w, id);
        } else if !q.rocket_stalls_at_map_edge {
            // spec: S-0506 Edge cases (quirks.rocket_stalls_at_map_edge off: it detonates at the last legal position)
            detonate = true;
        }
        // spec: S-0506 claim 21 and Constants (a trailer animation every 3 ticks in positive-speed flight)
        if t >= rs.anim_next {
            rs.anim_next = t.wrapping_add(3);
            if let Some(name) = trailer.as_deref() {
                let at = w.obj(id).map_or(pos, |ob| ob.pos);
                effects::anim(w, name, at);
            }
        }
        // spec: S-0506 claim 19 (a positive-speed movement attempt with health below 1 detonates, even a rejected one)
        if w.obj(id).is_some_and(|ob| ob.health < 1) {
            detonate = true;
        }
    }
    store(w, id, rs);
    if detonate {
        detonation(w, id);
    }
    moved
}

fn store(w: &mut World, id: ObjectId, rs: RocketState) {
    if let Some(ob) = w.obj_mut(id) {
        ob.rocket = Some(rs);
    }
}

/// The set-C take-off animations in pause and raise every 24 ticks on the
/// shared timer (S-0506 claim 21). The animation's name is not imported
/// (S-0506 Import mapping), so only the timer runs: a seam.
fn set_c_anim(_w: &mut World, rs: &mut RocketState, t: i32) {
    if t >= rs.anim_next {
        // spec: S-0506 claim 21 and Constants (set-C pause and raise animation interval 24 ticks)
        rs.anim_next = t.wrapping_add(24);
    }
}

/// The end of tilt or raise (S-0506 claim 15): the final pitch, a tracked
/// flying object, the take-off sound; ascent.
fn end_of_tilt(w: &mut World, id: ObjectId, rs: &mut RocketState, p: &RocketSet, mode: Rounding) {
    // spec: S-0506 claim 15 (pitch = P_f × π/2; tracked flying object; take-off sound; ascent)
    rs.pitch = quarter_turns(p.pitch_final, mode).to_f32(mode);
    rs.phase = Some(Phase::Ascent);
    crate::combat::flying::register(w, id);
    let Some(o) = w.obj(id) else { return };
    let (of, pos) = (
        crate::econ::TypeRef {
            kind: o.kind,
            index: o.type_index,
        },
        o.pos,
    );
    w.effects.push(Effect::Sound {
        sound: Sound::TakeOff,
        of: Some(of),
        house: None,
        pos: Some(pos),
    });
}

/// The claim 19 test of lazy cruise and descent (S-0506 claim 19).
fn contact(w: &World, pos: Pos, aim: Pos, rs: &RocketState, mode: Rounding) -> bool {
    // spec: S-0506 claim 19 (the world height its move would reach)
    let s = trunc(rs.speed);
    let z_next = trunc(fi(pos.z.into(), mode).add(
        crate::table_trig::sin_q(w, rs.pitch.to_f64(), mode).mul(fi(s, mode), mode),
        mode,
    ));
    let aim_z = i64::from(aim.z);
    if z_next <= aim_z {
        return true;
    }
    let c = pos.cell();
    // spec: S-0506 claim 19 (over a high bridge: aim height equal to the cell's terrain height, and the next height at or below aim + 416, the bridge deck)
    if w.high_bridge(c) {
        let terrain = w.surface_at(c.x * 256 + 128, c.y * 256 + 128); // spec: S-0204 claim 2 (the cell centre)
        if aim.z == terrain && z_next <= aim_z + 416 {
            // spec: S-0506 claim 19 (the 416-lepton bridge deck height)
            return true;
        }
    }
    // spec: S-0506 claim 19 and Constants (height above the terrain below 1 lepton)
    height_above_terrain(w, pos) < 1
}

/// Detonates a rocket missile (S-0506 claim 19): it stops being a tracked
/// flying object, the detonation of S-0303 happens at its nose point with
/// the set's damage and warhead (elite ones for an elite launch), and the
/// missile is removed.
pub fn detonation(w: &mut World, id: ObjectId) {
    let mode = w.rules().mode;
    let Some(o) = w.obj(id).cloned() else { return };
    let Some(rs) = o.rocket.clone() else { return };
    let p = set_of(w, rs.set);
    crate::combat::flying::unregister(w, id);
    let bias = crate::table_trig::bias(w);
    let at = nose_b(
        o.pos,
        p.body_length,
        rs.pitch,
        o.body.value(w.tick),
        mode,
        bias,
    );
    // spec: S-0506 claim 19 (the set's damage and warhead, or the elite ones for an elite launch)
    let (damage, warhead) = if rs.elite {
        (p.elite_damage, p.elite_warhead)
    } else {
        (p.damage, p.warhead)
    };
    if let Some(wh) = warhead {
        // spec: S-0506 claim 19 (source object: the missile; source house: none; resource reduction allowed; the animation before the hits; the nose point exactly, with no height correction)
        crate::combat::projectile::detonate_rocket(w, wh, damage, at, Some(id));
    } else {
        w.note(
            "S-0506 claim 19: a rocket parameter set without a warhead detonates with no damage",
        );
    }
    // spec: S-0506 claim 19 (the missile is removed after the area damage)
    if w.obj(id).is_some() {
        crate::combat::death::leave_play(w, id);
    }
}
