//! Hover and amphibious movement (S-0503): the hovering locomotor.
//!
//! - **Model (claim 1).** The locomotor field alone chooses the model
//!   (`rules::Locomotor::Hover`); the amphibious class only feeds the land
//!   multipliers and passability. A driving amphibious vehicle drives.
//! - **Travel (claims 2 to 7).** [`tick`]: throttle (F1), the ramp of the
//!   hover speed (F2), the step length (F3), straight steps along the
//!   movement heading (F4) and the arrival test (F5), then the choice of the
//!   next heading point with the step check of table T1 (claims 5 and 12).
//! - **Wake (claim 9)**, **height (claim 8, F6)** and **spin (claims 10 and
//!   11, F7)** follow the travel in the same tick, in that order.
//! - **Switching off.** [`switched`] is called by the control-centre rule of
//!   S-0405 claim 13 when a vehicle is disabled or enabled.
//!
//! Interim rules of open requests are marked with their IDs: Q-0690 (bob
//! constants and default precision), Q-0691 (vehicle-present flags, between
//! steps, current speed), Q-0692 (search delay, final step after a failed
//! search), Q-0693 (cell-arrival updates) and Q-0694 (initial heading).

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_num::geom::{delta, direction, root_floor};
use glasnost_num::{F64, Rounding};
use serde::{Deserialize, Serialize};

use crate::drive;
use crate::occupancy::{self, Outcome};
use crate::path::{self, Mode, Search};
use crate::rules::{Behaviour, Kind, Locomotor, UnitRules};
use crate::turning::Turning;
use crate::world::{Layer, ObjectId, Pos, Target, World};

/// Ticks per game minute of the hover fields (F2, F6).
// spec: S-0503 Constants (900 ticks per game minute)
pub const TICKS_PER_MINUTE: i64 = 900;
/// Throttle heading tolerance, facing units (F1).
// spec: S-0503 Constants (throttle heading tolerance 8192 facing units, 45°)
pub const HEADING_TOLERANCE: i32 = 8192;
/// Half-throttle distance, leptons (F1).
// spec: S-0503 Constants (half-throttle distance 256 leptons)
pub const HALF_THROTTLE_DISTANCE: u64 = 256;
/// Bob amplitude factor, leptons per tick (F6).
// spec: S-0503 Constants (bob amplitude factor 2)
pub const BOB_AMPLITUDE: i64 = 2;
/// Spin facing step, facing units per count (F7).
// spec: S-0503 Constants (spin facing step 256)
pub const SPIN_STEP: i32 = 256;
/// The spin count draw R(20, 30) (claim 10).
// spec: S-0503 Constants (spin count draw R(20, 30))
pub const SPIN_DRAW: (i32, i32) = (20, 30);
/// The sign draw R(0, 99); below 50 the count is negated (claim 10).
// spec: S-0503 Constants (sign by R(0, 99) < 50)
pub const SIGN_DRAW: (i32, i32) = (0, 99);
/// The sign draw's threshold (claim 10).
// spec: S-0503 claim 10 (when the second draw is below 50, n is negated)
pub const SIGN_BELOW: i32 = 50;
/// Wake period, ticks (claim 9).
// spec: S-0503 Constants (wake period 10)
pub const WAKE_PERIOD: i32 = 10;
/// Sinking height limit, leptons (claim 11).
// spec: S-0503 Constants (sinking height limit 104 leptons)
pub const SINK_LIMIT: i32 = 104;
/// Deck-layer threshold of the new-path look, leptons (claim 12).
// spec: S-0503 Constants (deck-layer threshold of the new-path look 312)
pub const LOOK_DECK_ABOVE: i32 = 312;
/// Arrival height tolerance at the destination cell, leptons (claim 7).
// spec: S-0503 Constants (arrival height tolerance 208, strictly below)
pub const ARRIVAL_HEIGHT: i32 = 208;
/// The deck's height above the terrain, leptons (Terms, claim 6, F6).
// spec: S-0503 Terms (plus 416 leptons on a high-bridge cell or deck)
pub const DECK: i32 = 416;
/// The height above a cell's surface from which a heading point is raised
/// by [`DECK`] (Terms, `quirks.hover_elevated_heading_point`).
// spec: S-0503 Terms (at least 312 leptons above that surface)
pub const ELEVATED: i32 = 312;

/// A hovering vehicle's state (S-0503 Terms).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HoverState {
    /// The movement heading η (Terms; S-0206 claim 13).
    pub heading: Turning,
    /// The heading point *P* (Terms, claim 5).
    pub point: Option<Pos>,
    /// The throttle *T*: 0, ½ or 1 (claim 2).
    pub throttle: F64,
    /// The hover speed φ (claims 3 and 4).
    pub phi: F64,
    /// The boost *B* (claim 3).
    pub boost: F64,
    /// The vertical velocity *v*, leptons per tick (claim 8).
    pub v: F64,
    /// The speed fraction ρ of S-0207 claim 2's *s* (claim 4).
    pub rho: F64,
    /// The spin count *n*; 0 when no spin is active (claim 10, F7).
    pub spin: i32,
    /// The destination the locomotor last took (claim 4: ρ = 1 on a new move).
    pub for_dest: Option<Target>,
    /// Cells, with their layers, whose vehicle-present flag it set (S-0209
    /// claim 1; Q-0691 interim).
    pub claims: Vec<(CellXY, Layer)>,
    /// The scatter requests sent this tick (S-0207 Edge cases,
    /// `quirks.drive_duplicate_scatter_request` off).
    pub requests: Option<(i32, Vec<(CellXY, Layer)>)>,
    /// The bomb mark (S-0312 Terms), given by claim 11.
    pub bomb_mark: bool,
}

impl HoverState {
    /// A new hovering vehicle's state.
    pub fn new(mode: Rounding) -> Self {
        Self {
            heading: Turning::at(0),
            point: None,
            throttle: F64::ZERO,
            // spec: S-0503 Terms (hover speed starts at 0)
            phi: F64::ZERO,
            // spec: S-0503 Terms (boost starts at 1)
            boost: F64::from_i64(1, mode),
            // spec: S-0503 Terms (vertical velocity starts at 0)
            v: F64::ZERO,
            // spec: S-0207 Terms (new vehicles start with ρ = 0)
            rho: F64::ZERO,
            spin: 0,
            for_dest: None,
            claims: Vec::new(),
            requests: None,
            bomb_mark: false,
        }
    }
}

fn state(w: &World, id: ObjectId) -> Option<HoverState> {
    w.obj(id).and_then(|o| o.hover.clone())
}

fn with<R>(w: &mut World, id: ObjectId, f: impl FnOnce(&mut HoverState) -> R) -> Option<R> {
    w.obj_mut(id).and_then(|o| o.hover.as_mut()).map(f)
}

/// Whether `id` is a hovering vehicle.
pub fn is_hover(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.hover.is_some())
}

/// Whether the vehicle is switched off (claim 10): disabled by the loss of
/// its control centre (S-0405 claim 13).
pub fn switched_off(w: &World, id: ObjectId) -> bool {
    is_hover(w, id) && crate::econ::power::unit_disabled(w, id)
}

/// The surface the vehicle stands on or over, plus [`DECK`] on a deck.
fn base_height(w: &World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id) else { return 0 };
    // spec: S-0503 Terms (hover height: above the surface height at its position, less 416 on a deck)
    w.surface_at(o.pos.x, o.pos.y) + if o.layer == Layer::Deck { DECK } else { 0 }
}

/// The hover height *h* (Terms).
pub fn hover_height(w: &World, id: ObjectId) -> i32 {
    w.obj(id).map_or(0, |o| o.pos.z - base_height(w, id))
}

/// Powered (Terms): not switched off, or a hover height above 0.
pub fn powered(w: &World, id: ObjectId) -> bool {
    // spec: S-0503 Terms (powered: not switched off, or hover height above 0)
    !switched_off(w, id) || hover_height(w, id) > 0
}

/// Whether the vehicle is moving (Terms): a destination or a heading point.
pub fn moving(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .is_some_and(|o| o.dest.is_some() || o.hover.as_ref().is_some_and(|h| h.point.is_some()))
}

/// Whether the vehicle is moving now (Terms): moving and *T* ≠ 0.
pub fn moving_now(w: &World, id: ObjectId) -> bool {
    moving(w, id) && state(w, id).is_some_and(|h| !h.throttle.eq_num(F64::ZERO))
}

/// Whether a hovering vehicle is between steps (S-0209 Terms): while it has
/// no heading point (Q-0691 interim).
pub fn between_steps(w: &World, id: ObjectId) -> bool {
    // spec: S-0209 Terms (hovering vehicles are between steps at the end of each step); Q-0691 (interim: while it has no heading point)
    state(w, id).is_some_and(|h| h.point.is_none())
}

/// Whether a hovering vehicle's current speed is not zero for S-0212 claim
/// 8's heading cell: φ > 0 (Q-0691 interim).
pub fn has_speed(w: &World, id: ObjectId) -> bool {
    // spec: S-0212 claim 8 (speed not zero); Q-0691 (interim: the hover speed φ above 0)
    state(w, id).is_some_and(|h| h.phi.gt(F64::ZERO))
}

/// The heading point's cell, for the vehicle scatter centre (S-0207 Terms).
pub fn heading_cell(w: &World, id: ObjectId) -> Option<CellXY> {
    state(w, id).and_then(|h| h.point).map(|p| p.cell())
}

/// The destination point *D* (Terms): a cell's centre at its surface height,
/// plus 416 on a high-bridge cell; an object's position.
fn destination_point(w: &World, id: ObjectId) -> Option<Pos> {
    let o = w.obj(id)?;
    match o.dest? {
        Target::Cell(c) => {
            // spec: S-0204 claim 3 (the cell centre)
            let (x, y) = (256 * c.x + 128, 256 * c.y + 128);
            let deck = if w.high_bridge(c) { DECK } else { 0 };
            // spec: S-0503 Terms (D's height: the surface height of its cell, plus 416 leptons on a high-bridge cell)
            Some(Pos::new(x, y, w.surface_at(x, y) + deck))
        }
        // spec: S-0314 claim 18 (a vehicle in depot contact: the supplied docking point is the destination position, not the building's)
        Target::Object(t) => {
            crate::depot::heading_point(w, id, t).or_else(|| w.obj(t).map(|x| x.pos))
        }
    }
}

/// The heading point for the next cell `c` (Terms): its centre at the cell's
/// surface height, plus 416 when the vehicle is 312 or more above that
/// surface (`quirks.hover_elevated_heading_point`; off: on a high-bridge
/// cell only).
fn point_for(w: &World, id: ObjectId, c: CellXY) -> Pos {
    // spec: S-0204 claim 3 (the cell centre)
    let (x, y) = (256 * c.x + 128, 256 * c.y + 128);
    let h = w.surface_at(x, y);
    let z = w.obj(id).map_or(h, |o| o.pos.z);
    let elevated = z >= h + ELEVATED;
    // spec: S-0503 Terms and Edge cases (quirks.hover_elevated_heading_point: 416 whenever 312 or more above, bridge or not; off: a high-bridge cell is needed too)
    let raise = elevated && (w.rules().quirks.hover_elevated_heading_point || w.high_bridge(c));
    Pos::new(x, y, h + if raise { DECK } else { 0 })
}

fn dist(w: &World, a: Pos, b: Pos, height: bool) -> u64 {
    let dx = i64::from(b.x - a.x);
    let dy = i64::from(b.y - a.y);
    let dz = if height { i64::from(b.z - a.z) } else { 0 };
    let r = w.rules();
    // spec: S-0503 F1 and F5 (trunc(r(Δx² + Δy² [+ Δz²])), the table square root; squares summed exactly)
    root_floor(
        (dx * dx + dy * dy + dz * dz) as u64,
        r.quirks.table_root_distance,
        r.mode,
    )
}

/// The rate of the movement heading: S-0206 claim 6 applied to twice the
/// type's turn-rate field (S-0206 claim 13).
fn heading_rate(w: &World, u: &UnitRules) -> i32 {
    let q = w.rules().quirks;
    // spec: S-0206 claim 13 (twice the type's turn-rate field, then claim 6)
    crate::rules::turn_rate(
        u.raw.rot.unwrap_or(0).saturating_mul(2),
        q.turn_rate_negative_wrap,
    )
}

/// The locomotor takes a destination (claim 4): ρ = 1 when it is given a
/// move while powered and not moving now. Called when a destination is
/// assigned, and at the start of the vehicle's tick for any destination
/// set without that call.
pub fn take_destination(w: &mut World, id: ObjectId) {
    take(w, id, true);
}

/// `given`: a destination was assigned now (a move is given even when it
/// equals the last one); otherwise only a destination the locomotor has
/// not yet taken counts.
fn take(w: &mut World, id: ObjectId, given: bool) {
    let Some(o) = w.obj(id) else { return };
    let Some(h) = o.hover.as_ref() else { return };
    if !given && o.dest == h.for_dest {
        return;
    }
    let dest = o.dest;
    // spec: S-0503 claim 4 (ρ = 1 when it is given a move while powered and not moving now)
    let start = dest.is_some() && powered(w, id) && !moving_now(w, id);
    with(w, id, |h| {
        h.for_dest = dest;
        if start {
            h.rho = F64::from_i64(1, Rounding::TowardZero);
        }
    });
    // spec: S-0503 claim 5 (it chooses a new heading point when it starts a move); Q-0695 (interim: at once, when the move is taken, so S-0417's ρ = ½ set right after it persists, claim 4)
    if dest.is_some() && state(w, id).is_some_and(|h| h.point.is_none()) {
        let _ = choose_next(w, id);
    }
}

/// The fraction set after a factory exit instruction (S-0417 claim 5): it
/// stays until one of claim 4's events (H8).
pub fn set_exit_fraction(w: &mut World, id: ObjectId) {
    with(w, id, |h| {
        // spec: S-0417 claims 5 and 6 and Constants (K sets the speed fraction to 0.5); S-0503 claim 4 (it persists until a listed reset)
        h.rho = crate::consts::HALF;
    });
}

/// One tick of a hovering vehicle (S-0503 State and ordering). Returns
/// whether its ground position changed.
pub fn tick(w: &mut World, id: ObjectId) -> bool {
    let r = w.rules_arc();
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    if o.hover.is_none() {
        return false;
    }
    let Some(u) = r.unit(o.kind, o.type_index).cloned() else {
        return false;
    };
    take(w, id, false);
    let before = (o.pos.x, o.pos.y);
    // spec: S-0503 State and ordering (not moving: no travel; height and spin still update); S-0314 claim 22 (a hovering vehicle whose locomotor a depot switched off cannot move, `quirks.depot_loss_strands_hover`)
    if moving(w, id) && !crate::depot::locomotor_off(w, id) {
        travel_turn(w, id, &u);
    }
    if w.obj(id).is_none() {
        return false;
    }
    // spec: S-0503 claim 9 and State and ordering (the wake after the tick's travel, before the height change)
    wake(w, id);
    // spec: S-0503 claim 8 (every tick, moving or not)
    let settled = height(w, id);
    // spec: S-0503 claim 10 and State and ordering (the spin after the height update)
    spin(w, id, settled);
    w.obj(id).is_some_and(|o| (o.pos.x, o.pos.y) != before)
}

/// How a choice of the next heading point ended (claims 5, 7 and 12).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Choice {
    /// A heading point was chosen.
    Chosen,
    /// No heading point and no refusal: an already computed step is still taken (claim 7).
    NoPoint,
    /// A refused next cell (table T1): no travel on this tick, unless the
    /// move was given up.
    Refused { gave_up: bool },
}

/// The step length *d* = trunc(*s* × φ) (F3).
fn step_length(w: &World, id: ObjectId, u: &UnitRules) -> i32 {
    let Some(h) = state(w, id) else { return 0 };
    let mode = w.rules().mode;
    // spec: S-0503 claim 4 (s is S-0207 claim 2's per-tick speed with the hover's ρ)
    let s = drive::progress_with(w, id, u, h.rho);
    // spec: S-0503 F3 (d = trunc(s × φ))
    F64::from_i64(i64::from(s), mode)
        .mul(h.phi, mode)
        .to_i32_common()
}

fn travel_turn(w: &mut World, id: ObjectId, u: &UnitRules) {
    let t = w.tick;
    let r = w.rules_arc();
    let mode = r.mode;
    let q = r.quirks;
    let Some(o) = w.obj(id).cloned() else { return };
    let Some(h) = o.hover.clone() else { return };
    let one = F64::from_i64(1, mode);
    let choice;
    let mut d;
    if let Some(p) = h.point {
        // spec: S-0503 Terms (η turns towards the heading point at the rate of S-0206 claim 13)
        // spec: S-0204 claim 8 (β, the direction from the vehicle to P)
        let beta = direction(
            i64::from(p.x - o.pos.x),
            i64::from(p.y - o.pos.y),
            q.table_direction,
        );
        let omega = heading_rate(w, u);
        let mut heading = h.heading;
        heading.turn_to(beta, t, omega);
        let eta = heading.value(t);
        // spec: S-0503 F1 (δ = S-0204 claim 11's signed difference between η and β)
        let dlt = delta(eta, beta);
        let goal = destination_point(w, id).unwrap_or(p);
        let throttle = if !powered(w, id) || dlt.abs() > HEADING_TOLERANCE {
            // spec: S-0503 F1 (T = 0 when not powered or |δ| > 8192)
            F64::ZERO
        } else if dist(w, o.pos, goal, true) < HALF_THROTTLE_DISTANCE {
            // spec: S-0503 F1 (T = ½ when trunc(r(Δx² + Δy² + Δz²)) < 256, measured to D, else to P)
            crate::consts::HALF
        } else {
            one
        };
        // spec: S-0503 claim 3 (while T > 0, B = the boost factor when at least two stored steps remain and the first two are equal, else 1; while T = 0, B keeps its value)
        let boost = if throttle.gt(F64::ZERO) {
            let straight = o.path.steps.len() >= 2 && o.path.steps[0] == o.path.steps[1];
            if straight { r.general.hover.boost } else { one }
        } else {
            h.boost
        };
        let phi = ramp(w, h.phi, boost, throttle);
        with(w, id, |s| {
            s.heading = heading;
            s.throttle = throttle;
            s.boost = boost;
            s.phi = phi;
        });
        d = step_length(w, id, u);
        // spec: S-0503 Edge cases (quirks.hover_stalled_start_reset: d = 0 with η exactly at the heading point sets T = 1 and ρ = 1 and recomputes d)
        if d == 0 && q.hover_stalled_start_reset && eta == beta {
            with(w, id, |s| {
                s.throttle = one;
                s.rho = one;
            });
            d = step_length(w, id, u);
        }
        // spec: S-0503 F5 (arrived when trunc(r(Δx² + Δy²)) ≤ d, map plane only)
        if dist(w, o.pos, p, false) <= d as u64 {
            // spec: S-0503 claim 7 (no snap: it keeps its position and chooses its next heading point)
            with(w, id, |s| s.point = None);
            // spec: S-0507 claim 6 (arriving at the cell of the transport it is entering: it boards, or is turned away)
            if crate::transport::arrival(w, id) {
                return;
            }
            choice = choose_next(w, id);
            // spec: S-0417 claim 7; Q-0693 (interim: an arrival at the heading point is a cell-arrival update, after the next choice)
            crate::econ::release::arrival(w, id);
        } else {
            choice = Choice::Chosen;
        }
    } else {
        // spec: S-0503 claim 4 and State and ordering (moving with no heading point: ρ = 1; travel uses the retained hover speed, without that tick's ramp)
        with(w, id, |s| s.rho = one);
        d = step_length(w, id, u);
        choice = choose_next(w, id);
    }
    // spec: S-0503 claim 7 (a refused next cell suppresses travel on that tick, except that a give-up still takes the computed step; otherwise a positive d still gives a step); Q-0692 (interim: a failed search is a give-up)
    let travels = match choice {
        Choice::Chosen | Choice::NoPoint => true,
        Choice::Refused { gave_up } => gave_up,
    };
    if travels && d > 0 {
        travel(w, id, d);
    }
    aim_body(w, id, u);
}

/// F2: the hover speed after one tick of the ramp.
pub fn ramp(w: &World, phi: F64, boost: F64, throttle: F64) -> F64 {
    let r = w.rules();
    let mode = r.mode;
    let one = F64::from_i64(1, mode);
    let minute = F64::from_i64(TICKS_PER_MINUTE, mode);
    // spec: S-0503 F2 (G = min(1, B × T))
    let goal = one.min(boost.mul(throttle, mode));
    // spec: S-0503 F2 (φ₁ = min(G, φ + 1 / (a × 900)) when φ < G; the product formed before the division)
    let phi1 = if phi.lt(goal) {
        let step = one.div(r.general.hover.acceleration.mul(minute, mode), mode);
        goal.min(phi.add(step, mode))
    } else {
        phi
    };
    if !phi1.gt(goal) {
        return phi1;
    }
    let step = one.div(r.general.hover.brake.mul(minute, mode), mode);
    let down = phi1.sub(step, mode);
    if r.quirks.hover_brake_undershoot {
        // spec: S-0503 F2 (φ′ = max(0, φ₁ − 1 / (b × 900)); the floor is 0, not G)
        down.max(F64::ZERO)
    } else {
        // spec: S-0503 Edge cases (quirks.hover_brake_undershoot off: the braking step is floored at the goal)
        down.max(goal)
    }
}

/// Refused (table T1): no heading point, throttle, hover speed and ρ 0.
fn refuse(w: &mut World, id: ObjectId) {
    with(w, id, |s| {
        // spec: S-0503 claim 12 (every row other than clear: no heading point, throttle, hover speed and ρ at 0)
        s.point = None;
        s.throttle = F64::ZERO;
        s.phi = F64::ZERO;
        s.rho = F64::ZERO;
    });
}

/// Chooses the next heading point (claim 5), with S-0207 claim 19 c and d
/// and S-0212 claims 20 to 23 when no stored step is left.
fn choose_next(w: &mut World, id: ObjectId) -> Choice {
    let t = w.tick;
    let Some(o) = w.obj(id).cloned() else {
        return Choice::NoPoint;
    };
    if !o.path.steps.is_empty()
        && o.dest.is_some()
        && let Some(target) = o.target
    {
        // spec: S-0207 claim 19 (c) (a chased vehicle or infantry destination: the chase cut first; when made, no range test)
        let cut = chase_cut(w, id);
        // spec: S-0207 claim 19 (c) (target in range, by S-0215 claim 37 b: all stored steps discarded; the destination is kept)
        if !cut && crate::approach::in_range(w, id, target) {
            drive::discard_path(w, id);
        }
    }
    let empty = w.obj(id).is_none_or(|o| o.path.steps.is_empty());
    if empty {
        if at_destination(w, id) {
            if let Some(ob) = w.obj_mut(id) {
                // spec: S-0503 claim 7 (in the destination's cell at a height difference below 208: the destination is cleared and the move ends)
                ob.dest = None;
            }
            return Choice::NoPoint;
        }
        if w.obj(id).is_none_or(|o| o.dest.is_none()) {
            return Choice::NoPoint;
        }
        // spec: S-0212 claim 21 and S-0207 claim 19 (d) (a fresh search with no stored steps waits for R)
        if t >= o.path.retry_at {
            fresh_search(w, id);
        }
        let none = w
            .obj(id)
            .is_none_or(|o| o.path.steps.is_empty() || o.dest.is_none());
        if none {
            // spec: S-0207 claim 19 (d) (no next step chosen: the arrival-time stop of part a; destination cleared, target kept)
            if crate::walk::arrival_stop_holds(w, id)
                && let Some(ob) = w.obj_mut(id)
            {
                ob.dest = None;
                ob.path.steps.clear();
            }
            return Choice::NoPoint;
        }
    }
    step_check(w, id, false)
}

/// Whether the vehicle is in its destination's cell at a height difference
/// below 208 (claim 7).
fn at_destination(w: &World, id: ObjectId) -> bool {
    let (Some(o), Some(d)) = (w.obj(id), destination_point(w, id)) else {
        return false;
    };
    // spec: S-0503 claim 7 (the cell of its destination, at a height difference strictly less than 208)
    o.pos.cell() == d.cell() && (o.pos.z - d.z).abs() < ARRIVAL_HEIGHT
}

/// A fresh search (S-0212 claim 20). For the hovering locomotor every fresh
/// search starts *R* (S-0212 claim 21; Q-0692 interim for the immediate
/// searches of table T1); a found path gets the look at its first cell.
fn fresh_search(w: &mut World, id: ObjectId) -> Search {
    let t = w.tick;
    let res = path::compute(w, id, Mode::Fresh);
    if res == Search::Found {
        let delay = w.rules().general.retry_delay;
        if let Some(ob) = w.obj_mut(id) {
            // spec: S-0212 claim 21 (hovering: every fresh search starts R again from its tick, also one that finds a path); Q-0692
            ob.path.retry_at = t.wrapping_add(delay);
        }
        first_cell_look(w, id);
    } else {
        // spec: S-0212 claims 21 and 23 (a failed search or gate: R from the search's tick, and the hover's destination is cleared)
        path::failed(w, id, res, false);
    }
    res
}

/// The look at the first cell of a new path (claim 12): S-0207 claim 15's,
/// without the gate row and the train exception, with the nearest object
/// looked for in the deck layer above 312 leptons.
fn first_cell_look(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else { return };
    let Some(&d) = o.path.steps.first() else {
        return;
    };
    let here = o.pos.cell();
    let cell = drive::neighbour(here, d);
    let reference = occupancy::reference_level(w, id);
    // spec: S-0503 claim 12 (no gate row and no train exception; S-0207 claim 15's stationary-friendly row)
    if path::step_outcome(w, id, here, cell, reference) != Outcome::StationaryFriendly {
        return;
    }
    // spec: S-0503 claim 12 (the deck layer when the world height is more than 312 above the surface at that cell's centre)
    let Some(near) = drive::nearest_object(w, id, cell, LOOK_DECK_ABOVE) else {
        return;
    };
    let near_owner = w.obj(near).and_then(|x| x.owner);
    if !w.allied(o.owner, near_owner) {
        return;
    }
    // spec: S-0207 claim 15 and F7 (the look's give-up test adds contact with nothing)
    if drive::give_up_test(w, id, true) {
        drive::give_up(w, id);
    } else {
        drive::request_scatter(w, id, cell);
    }
}

/// The step check of the next cell (claim 12, table T1).
fn step_check(w: &mut World, id: ObjectId, repeated: bool) -> Choice {
    let r = w.rules_arc();
    let Some(o) = w.obj(id).cloned() else {
        return Choice::NoPoint;
    };
    let Some(&d) = o.path.steps.first() else {
        return Choice::NoPoint;
    };
    let here = o.pos.cell();
    let cell = drive::neighbour(here, d);
    // spec: S-0503 claim 5 (a crate in that cell is collected when the cell is chosen, before the check)
    collect_crate(w, id, cell);
    let reference = occupancy::reference_level(w, id);
    // spec: S-0503 claim 12 (the step check: the entry outcome of S-0209, without S-0209 claim 5f's driving substitutions)
    let outcome = path::step_outcome(w, id, here, cell, reference);
    let refused = |w: &mut World, gave_up: bool| {
        refuse(w, id);
        Choice::Refused { gave_up }
    };
    match outcome {
        Outcome::Clear => {
            let p = point_for(w, id, cell);
            if let Some(ob) = w.obj_mut(id) {
                // spec: S-0503 claim 5 (on clear the step is used up and the vehicle heads for that point)
                ob.path.steps.remove(0);
                // spec: S-0212 claim 22 (a blockage ends once the unit has taken a step again)
                ob.path.blocked_since = None;
            }
            with(w, id, |s| s.point = Some(p));
            // spec: S-0209 claim 1; Q-0691 (interim: the flag of the chosen cell is set at once, as for a driving step)
            claim(w, id, cell);
            Choice::Chosen
        }
        Outcome::CloakedBlocker => {
            // spec: S-0503 table T1 (cloaked blocker: the cloaked objects in the cell become visible to its house, S-0209 claim 9)
            reveal_cloaked(w, id, cell);
            if repeated {
                // spec: S-0503 table T1 (repeated check, cloaked blocker: it keeps its destination and waits)
                return refused(w, false);
            }
            // spec: S-0503 table T1 (first check: discard the path and search at once whatever the retry timer; a found path gets the repeated check)
            search_and_recheck(w, id)
        }
        Outcome::Blocked => {
            // spec: S-0503 table T1 (blocked: it reveals cloaked objects in the cell as in the cloaked-blocker row)
            reveal_cloaked(w, id, cell);
            if repeated {
                // spec: S-0503 table T1 (repeated check, blocked: it gives up the move, its destination cleared)
                drive::give_up(w, id);
                return refused(w, true);
            }
            search_and_recheck(w, id)
        }
        Outcome::MovingBlocker => {
            // spec: S-0503 table T1 (moving blocker: S-0212 claim 22's blockage timer and re-path rule, at both checks)
            drive::blocked_step(w, id);
            let gave_up = w.obj(id).is_none_or(|o| o.dest.is_none());
            refused(w, gave_up)
        }
        Outcome::ClosedFriendlyGate => {
            // spec: S-0503 table T1 (closed friendly gate: it tells the gate to open and waits)
            if let Some(gate) = crate::process::closed_friendly_gate(w, id, cell) {
                crate::process::gate_met(w, gate);
            }
            refused(w, false)
        }
        Outcome::FriendlyObstacle | Outcome::EnemyObstacle => {
            if !repeated {
                // spec: S-0503 table T1 (obstacle, first check: discard, search at once, repeated check on a found path)
                return search_and_recheck(w, id);
            }
            // spec: S-0503 table T1 (obstacle, repeated check: as S-0207 claim 15, attack what blocks by the override of S-0215 claim 10, else wait)
            drive::attack_obstacle(w, id, cell);
            refused(w, false)
        }
        Outcome::StationaryFriendly => {
            if !repeated {
                // spec: S-0503 table T1 (stationary friendly, first check: as for an obstacle)
                return search_and_recheck(w, id);
            }
            // spec: S-0207 Edge cases (quirks.drive_giveup_ignores_contact: the step check's give-up test does not ask about contact)
            let contact = !r.quirks.drive_giveup_ignores_contact;
            if drive::give_up_test(w, id, contact) {
                // spec: S-0503 table T1 (repeated check, stationary friendly: the give-up test holds, it gives up the move)
                drive::give_up(w, id);
                return refused(w, true);
            }
            // spec: S-0503 table T1 (otherwise the scatter request of S-0207 claim 15 to the cell layer it moves in, and it waits)
            drive::request_scatter(w, id, cell);
            refused(w, false)
        }
    }
}

/// A first check's discard, immediate fresh search and repeated check
/// (table T1).
fn search_and_recheck(w: &mut World, id: ObjectId) -> Choice {
    // spec: S-0503 claim 12 (the first check's row: no heading point, throttle, hover speed and ρ at 0)
    refuse(w, id);
    drive::discard_path(w, id);
    // spec: S-0503 table T1 (a fresh search at once, whatever its path retry timer shows)
    let res = fresh_search(w, id);
    let ready = w
        .obj(id)
        .is_some_and(|o| o.dest.is_some() && !o.path.steps.is_empty());
    if res == Search::Found && ready {
        // spec: S-0503 claim 12 (when the search finds a path, the repeated check follows on the same tick)
        return step_check(w, id, true);
    }
    let gave_up = w.obj(id).is_none_or(|o| o.dest.is_none());
    // spec: S-0503 claim 7; Q-0692 (interim: a failed search that clears the destination is a give-up and keeps the computed step)
    Choice::Refused { gave_up }
}

/// Sets the vehicle-present flag of `c` (S-0209 claim 1; Q-0691 interim).
fn claim(w: &mut World, id: ObjectId, c: CellXY) {
    let Some(o) = w.obj(id) else { return };
    let layer = o.layer;
    if o.hover
        .as_ref()
        .is_some_and(|h| h.claims.iter().any(|x| x.0 == c))
    {
        return;
    }
    w.cells.claim_vehicle(c, layer, true);
    with(w, id, |h| {
        h.claims.push((c, layer));
        h.claims.sort();
    });
}

/// Clears the flag of the cell left, unless it is the heading point's cell
/// (Q-0691 interim).
fn release_left(w: &mut World, id: ObjectId, c: CellXY) {
    let Some(h) = state(w, id) else { return };
    if h.point.is_some_and(|p| p.cell() == c) {
        return;
    }
    let held: Vec<(CellXY, Layer)> = h.claims.iter().copied().filter(|x| x.0 == c).collect();
    for (cell, layer) in held {
        w.cells.claim_vehicle(cell, layer, false);
    }
    with(w, id, |h| h.claims.retain(|x| x.0 != c));
}

/// Travels `d` leptons along the movement heading (claim 6, F4).
fn travel(w: &mut World, id: ObjectId, d: i32) {
    let t = w.tick;
    let r = w.rules_arc();
    let Some(o) = w.obj(id).cloned() else { return };
    let Some(h) = o.hover.as_ref() else { return };
    let eta = h.heading.value(t);
    // spec: S-0503 F4 (x′ = trunc(x + d cos θ), y′ = trunc(y − d sin θ), z′ = z: S-0208 claim 6's planar step, with quirks.table_direction)
    let (x, y) =
        crate::walk::step_point(o.pos.x, o.pos.y, eta, d, r.quirks.table_direction, r.mode);
    let old = o.pos.cell();
    let new = Pos::new(x, y, 0).cell();
    let mut layer = o.layer;
    if new != old {
        let surface = w.surface_at(x, y);
        if w.high_bridge(new) && o.pos.z - surface >= DECK {
            // spec: S-0503 claim 6 (entering a high-bridge cell with a hover height of 416 or more above its surface: on the deck)
            layer = Layer::Deck;
        } else if !w.high_bridge(new) {
            // spec: S-0503 claim 6 (entering a cell that is not a bridge cell: off any deck)
            layer = Layer::Ground;
        }
    }
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0503 claim 6 (the world height is not changed by this step)
        ob.pos = Pos::new(x, y, o.pos.z);
        ob.layer = layer;
    }
    if new != old {
        // spec: S-0503 claim 6 (its occupancy is moved from cell to cell as its position crosses a cell edge)
        w.cells.leave(old, o.layer, id);
        w.cells.enter(new, layer, id, false);
        crate::load::recalc_cells(w, &[old, new]);
        release_left(w, id, old);
    }
}

/// The body aim of claim 5 and S-0206 claim 13: while the throttle is above
/// 0, the body turns at its own rate towards the point one neighbour step
/// beyond the heading point in the next stored direction, or the heading
/// point.
fn aim_body(w: &mut World, id: ObjectId, u: &UnitRules) {
    let t = w.tick;
    let Some(o) = w.obj(id).cloned() else { return };
    let Some(h) = o.hover.as_ref() else { return };
    let Some(p) = h.point else { return };
    if !h.throttle.gt(F64::ZERO) {
        return;
    }
    let (ax, ay) = match o.path.steps.first() {
        Some(&s) => {
            let (dx, dy) = crate::regions::STEPS[usize::from(s % 8)]; // spec: S-0212 Terms (step directions 0 to 7)
            // spec: S-0503 claim 5 (one neighbour step: 256 leptons on each affected map axis)
            (p.x + 256 * dx, p.y + 256 * dy)
        }
        None => (p.x, p.y),
    };
    if (ax, ay) == (o.pos.x, o.pos.y) {
        return;
    }
    let q = w.rules().quirks;
    // spec: S-0204 claim 8 (the direction to the aim point)
    let f = direction(
        i64::from(ax - o.pos.x),
        i64::from(ay - o.pos.y),
        q.table_direction,
    );
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0206 claim 13 (the body facing turns at the type's own rate)
        ob.body.turn_to(f, t, u.turn_rate);
    }
}

/// The wake over water (claim 9).
fn wake(w: &mut World, id: ObjectId) {
    // spec: S-0503 claim 9 (ticks divisible by 10)
    if w.tick % WAKE_PERIOD != 0 {
        return;
    }
    // spec: S-0503 claim 9 (only while moving now)
    if !moving_now(w, id) {
        return;
    }
    let Some(anim) = w.rules().wake.clone() else {
        // spec: S-0503 claim 9 (provided the game-wide wake animation is defined)
        return;
    };
    let Some(o) = w.obj(id) else { return };
    let water = w
        .cells
        .cell(o.pos.cell())
        .is_some_and(|c| c.land == land::WATER);
    // spec: S-0503 claim 9 (off a deck, on a cell whose land type is water)
    if o.layer == Layer::Deck || !water {
        return;
    }
    let at = o.pos;
    // spec: S-0503 claim 9 (one instance at its position after the tick's travel)
    crate::combat::effects::anim(w, &anim, at);
}

/// The bob term's period *P*ᵈ (double) and *P*ᵢ (integer, at least 1).
fn bob_period(w: &World, n: u32) -> (F64, i64) {
    let r = w.rules();
    let mode = r.mode;
    // spec: S-0503 F6 (m = 1 for odd N, 1.1 for even N); Q-0690 (interim: the double nearest 1.1)
    let m = if n % 2 == 1 {
        F64::from_i64(1, mode)
    } else {
        crate::consts::EVEN_BOB_FACTOR
    };
    // spec: S-0503 F6 (Pᵈ = m × q × 900); Q-0690 (interim: formed left to right)
    let pd = m
        .mul(r.general.hover.bob, mode)
        .mul(F64::from_i64(TICKS_PER_MINUTE, mode), mode);
    // spec: S-0503 F6 (Pᵢ = trunc(Pᵈ)); Edge cases (a period of 0 is treated as 1 tick, a documented departure)
    let pi = i64::from(pd.to_i32_common());
    (pd, if pi == 0 { 1 } else { pi })
}

/// The height update (claim 8, F6). Returns whether the vehicle settled to
/// height 0 on this tick while switched off (claim 10).
fn height(w: &mut World, id: ObjectId) -> bool {
    let r = w.rules_arc();
    let mode = r.mode;
    let hr = r.general.hover;
    let t = w.tick;
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    let Some(st) = o.hover.clone() else {
        return false;
    };
    let h = hover_height(w, id);
    let off = switched_off(w, id);
    // spec: S-0503 F6 (A = trunc(h + v))
    let a = F64::from_i64(i64::from(h), mode)
        .add(st.v, mode)
        .to_i32_common();
    // spec: S-0503 Edge cases (N: our engine's own creation counter, a documented departure)
    let n = o.id.0;
    let (pd, pi) = bob_period(w, n);
    // spec: S-0503 F6 (ϱ = (t + 2N) mod Pᵢ, integer remainder with the sign of the dividend)
    let varrho = (i64::from(t) + 2 * i64::from(n)) % pi;
    // spec: S-0503 F6 (sin(ϱ × 2π / Pᵈ), table sine); Q-0690 (interim: (ϱ × 2π) / Pᵈ, 2π the double nearest)
    let arg = F64::from_i64(varrho, mode)
        .mul(crate::consts::TWO_PI, mode)
        .div(pd, mode);
    let arg = if pi == 1 && pd.to_i32_common() == 0 {
        // spec: S-0503 Edge cases (a zero period: the remainder is 0 and the bob term is 0)
        F64::ZERO
    } else {
        arg
    };
    let bob = F64::from_i64(BOB_AMPLITUDE, mode).mul(glasnost_num::trig::sin_t(arg, mode), mode);
    // spec: S-0503 F6 (u = trunc(2 × sin(...) + A))
    let u = bob
        .add(F64::from_i64(i64::from(a), mode), mode)
        .to_i32_common();
    // spec: S-0503 F6 (resulting height max(0, u); when u < 0 the velocity used next is 0)
    let (new_h, mut v) = if u < 0 { (0, F64::ZERO) } else { (u, st.v) };
    // spec: S-0503 F6 (hₜ = h − Hₕ with a stored step whose neighbour step ahead has a higher surface, else h)
    let climbing = o.path.steps.first().is_some_and(|&s| {
        let (dx, dy) = crate::regions::STEPS[usize::from(s % 8)]; // spec: S-0212 Terms (step directions 0 to 7)
        // spec: S-0503 F6 (the surface one neighbour step ahead, 256 leptons on each affected axis)
        w.surface_at(o.pos.x + 256 * dx, o.pos.y + 256 * dy) > w.surface_at(o.pos.x, o.pos.y)
    });
    let ht = if climbing { h - hr.height } else { h };
    let g = F64::from_i64(i64::from(hr.gravity), mode);
    if ht < hr.height && !off {
        // spec: S-0503 F6 (gains ((2Hₕ − hₜ) / Hₕ) × g when hₜ < Hₕ and not switched off, in double precision in the order shown)
        let lift = F64::from_i64(i64::from(2 * hr.height - ht), mode)
            .div(F64::from_i64(i64::from(hr.height), mode), mode)
            .mul(g, mode);
        v = v.add(lift, mode);
    }
    // spec: S-0503 F6 (also gains trunc(g / 3) when hₜ < Hₕ and hₜ < trunc(Hₕ / 4), switched off or not; integer divisions)
    if ht < hr.height && ht < hr.height / 4 {
        v = v.add(F64::from_i64(i64::from(hr.gravity / 3), mode), mode);
    }
    // spec: S-0503 F6 (subtract g, then multiply by k: the new vertical velocity)
    let v = v.sub(g, mode).mul(hr.dampen, mode);
    let base = base_height(w, id);
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0503 F6 (the world height becomes the surface height plus the resulting hover height, plus 416 on a deck)
        ob.pos.z = base + new_h;
        if let Some(s) = ob.hover.as_mut() {
            s.v = v;
        }
    }
    off && new_h == 0
}

/// Switching a hovering vehicle off or on (claim 10), from the disabling
/// and re-enabling of S-0405 claim 13. `off`: the new state. Called after
/// the vehicle's disabled flag changed.
pub fn switched(w: &mut World, id: ObjectId, off: bool, was_powered: bool) {
    if !off || !is_hover(w, id) {
        // spec: S-0503 claim 10 (switched on: the lift returns; an active spin is not cancelled)
        return;
    }
    let Some(o) = w.obj(id).cloned() else { return };
    let half = w.rules().quirks.ranged_draw_half_span;
    // spec: S-0503 claim 10 (a spin starts only when powered and the current behaviour is neither sleep nor enter)
    let quiet = matches!(o.beh.current, Some(Behaviour::Sleep | Behaviour::Enter));
    if was_powered && !quiet {
        // spec: S-0503 claim 10 (two ranged draws from the scenario stream: R(20, 30), then R(0, 99); below 50 negates n)
        let mut n = w.rng.ranged(SPIN_DRAW.0, SPIN_DRAW.1, half);
        let sign = w.rng.ranged(SIGN_DRAW.0, SIGN_DRAW.1, half);
        if sign < SIGN_BELOW {
            n = -n;
        }
        with(w, id, |s| s.spin = n);
    }
    // spec: S-0503 claim 10 (whenever it is switched off, a moving vehicle receives a stop request)
    if moving(w, id) {
        stop_request(w, id);
    }
}

/// Seam: a person's stop order (S-0213 claim 8) for a hovering vehicle.
/// No spec says what it does to the heading point, throttle, hover speed
/// and ρ (Q-0696); the order's own clearing of the destination and path is
/// all that happens, so the hover travels on to its heading point and
/// stops there.
pub fn stop_order(w: &mut World, id: ObjectId) {
    // Q-0696 (interim: the hover keeps its heading point, throttle, hover speed and ρ)
    let _ = (w, id);
}

/// The stop request of claim 10: the destination and stored path are
/// cleared only when the destination and heading point differ as full
/// three-dimensional points; throttle and hover speed are not zeroed.
pub fn stop_request(w: &mut World, id: ObjectId) {
    let d = destination_point(w, id);
    let p = state(w, id).and_then(|h| h.point);
    // spec: S-0503 claim 10 (cleared only when destination and heading point differ as full 3-D points)
    if d != p
        && let Some(ob) = w.obj_mut(id)
    {
        ob.dest = None;
        ob.path.steps.clear();
    }
}

/// The spin (claim 10, F7) and, when it ends, the sinking test (claim 11).
fn spin(w: &mut World, id: ObjectId, settled: bool) {
    let t = w.tick;
    let Some(st) = state(w, id) else { return };
    let n = st.spin;
    if n == 0 {
        return;
    }
    // spec: S-0503 claim 10 (a vehicle that settles to height 0 on this tick while switched off has no further rotation; the spin ends when it is no longer powered)
    let ended = if settled || !powered(w, id) {
        true
    } else {
        let keeps = w.rules().quirks.set_facing_keeps_target;
        if let Some(ob) = w.obj_mut(id) {
            let f = ob.body.value(t);
            // spec: S-0503 F7 (the body facing increases by 256n, modulo 65536)
            let nf = (i32::from(f) + SPIN_STEP * n).rem_euclid(65536) as u16; // spec: S-0503 F7 (modulo 65536)
            ob.body.set(nf, t, keeps);
        }
        // spec: S-0503 F7 (the count for the next tick is n − sign(n))
        let next = n - n.signum();
        with(w, id, |s| s.spin = next);
        next == 0
    };
    if ended {
        with(w, id, |s| s.spin = 0);
        sink_test(w, id);
    }
}

/// Claim 11: at the end of a spin, over a water cell below 104 leptons above
/// the surface, the vehicle is given the bomb mark and starts to fall.
fn sink_test(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let water = w
        .cells
        .cell(o.pos.cell())
        .is_some_and(|c| c.land == land::WATER);
    // spec: S-0503 claim 11 (a water cell, world height less than 104 leptons above the surface)
    // The world height above the surface, not the hover height: on a deck the vehicle is 416 or more above it and survives.
    let above = o.pos.z - w.surface_at(o.pos.x, o.pos.y);
    if !water || above >= SINK_LIMIT {
        return;
    }
    // spec: S-0503 claim 11 (it is given the bomb mark)
    with(w, id, |s| s.bomb_mark = true);
    // spec: S-0503 claim 11 (and starts to fall)
    start_fall(w, id);
}

/// Whether the vehicle carries the bomb mark (S-0312 Terms and claim 11).
pub fn bomb_mark(w: &World, id: ObjectId) -> bool {
    state(w, id).is_some_and(|h| h.bomb_mark)
}

/// Seam: the fall of a vehicle given the bomb mark (S-0503 claim 11, S-0511
/// claim 10). Falling objects are not built yet, so the vehicle stays where
/// it settled, with the mark; S-0312 claim 11's low-water death effects
/// for a marked vehicle are not built either.
pub fn start_fall(w: &mut World, id: ObjectId) {
    // spec: S-0511 claim 10 (the fall), not built: a seam
    let _ = (w, id);
}

/// Seam: the cloaked objects in `c` become visible to the vehicle's house
/// (S-0503 table T1, S-0209 claim 9). Cloaking is S-0516's and is not built,
/// so nothing is cloaked.
fn reveal_cloaked(w: &mut World, id: ObjectId, c: CellXY) {
    // spec: S-0516 (cloaking), not built: a seam
    let _ = (w, id, c);
}

/// Seam: a crate in the chosen cell is collected (S-0503 claim 5, S-0520).
/// Crates are not built yet.
fn collect_crate(w: &mut World, id: ObjectId, c: CellXY) {
    // spec: S-0520 (crates), not built: a seam
    let _ = (w, id, c);
}

/// Seam: the chase cut of S-0212 claim 20 for a vehicle or infantry
/// destination (S-0207 claim 19 c). Returns whether the cut was made; the
/// cut is not built for any locomotor yet, so it is never made.
fn chase_cut(w: &mut World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let chased = matches!(o.dest, Some(Target::Object(x)) if w.obj(x).is_some_and(|t| matches!(t.kind, Kind::Vehicle | Kind::Infantry)));
    // spec: S-0212 claim 20 (the chase cut), not built: a seam
    let _ = chased;
    false
}

/// Whether the vehicle moves by this module (the hovering locomotor).
pub fn uses_hover(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        .is_some_and(|u| u.locomotor == Locomotor::Hover)
}
