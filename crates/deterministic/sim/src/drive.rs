//! Ground vehicle movement, the drive locomotor (S-0207), with the turn in
//! place of S-0206 claim 12, a driving unit's blocked steps (S-0212 claims
//! 20 and 22) and crushing at cell centres (S-0210 claims 3, 7 to 9).

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_num::F64;
use glasnost_num::geom::root_floor;
use glasnost_rules::sim::{LocomotionClass, MovementZone};
use serde::{Deserialize, Serialize};

use crate::occupancy::{self, Outcome};
use crate::path::{self, Mode, Search};
use crate::process;
use crate::regions::STEPS;
use crate::rules::{Behaviour, Kind, Locomotor, SimRules, UnitRules};
use crate::trace;
use crate::tracks::{self, Point};
use crate::world::{Layer, ObjectId, Pos, Target, World};

/// The track a vehicle is on.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Track {
    /// Current step d.
    pub d: u8,
    /// The next step the track was chosen with (`None`: straight or
    /// diagonal for d alone).
    pub e: Option<u8>,
    /// The track target (a cell centre), map x and y.
    pub target: (i32, i32),
    /// The cell the track ends in.
    pub target_cell: CellXY,
    /// The heading point's stored world height: the vehicle's height when
    /// the track began (S-0207 claim 24).
    pub height: i32,
    /// Index of the next point to pass.
    pub next: usize,
    /// The last point passed (for `quirks.drive_subpoint_compounding` off).
    pub last: (i32, i32),
    /// The cells of the step or steps (first, and second for a curve).
    pub cells: Vec<CellXY>,
    /// Crush mode (S-0207 claim 7).
    pub crush: bool,
    /// The exit track of a factory (S-0417 table T1) instead of a track
    /// of S-0207.
    #[serde(default)]
    pub exit: bool,
    /// A tank bunker's entry or release track (S-0606 tables T1 and T2)
    /// instead of a track of S-0207.
    #[serde(default)]
    pub bunker: Option<crate::structures::bunker::BunkerTrack>,
}

impl Track {
    /// The points before the target, in driven order.
    fn points(&self) -> Vec<Point> {
        if let Some(b) = self.bunker {
            // spec: S-0606 Terms (the entry and release tracks of tables T1 and T2, special drive tracks under S-0207's track rules)
            crate::structures::bunker::track_points(b)
        } else if self.exit {
            // spec: S-0417 Terms (the exit track: table T1, distinct from the ordinary movement tracks)
            crate::econ::release::exit_track_points()
        } else {
            tracks::track(self.d, self.e).0
        }
    }
}

/// A driving vehicle's locomotor state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriveState {
    /// Progress budget A.
    pub budget: i32,
    /// Speed fraction ρ (a double).
    pub rho: F64,
    /// Target fraction ρₜ.
    pub rho_t: F64,
    /// The track it is on.
    pub track: Option<Track>,
    /// Turning in place before a step (S-0206 claim 12).
    pub turning_in_place: bool,
    /// Tick in which the turn in place was given, and its duration.
    pub turn: Option<(i32, i32)>,
    /// Cells whose vehicle-present flag it set (S-0209 claim 1).
    pub claims: Vec<CellXY>,
    /// Between steps (S-0209 Terms).
    pub between_steps: bool,
    /// Tick of the last movement.
    pub moved_tick: Option<i32>,
    /// The scatter requests sent this tick, cell and layer, kept only with
    /// `quirks.drive_duplicate_scatter_request` off (S-0207 Edge cases).
    #[serde(default)]
    pub requests: Option<(i32, Vec<(CellXY, Layer)>)>,
    /// The integer progress step s earned in the vehicle's latest drive
    /// tick (S-0207 claim 2), kept for the movement test of S-0402 claim 5a.
    #[serde(default)]
    pub step: i32,
    /// The locomotor destination point (S-0207 claim 25): the movement goal
    /// locomotion accepted, separate from the order-level destination and
    /// from the heading point.
    #[serde(default)]
    pub goal: Option<Pos>,
    /// The order-level destination the locomotor last took ([`take_destination`]).
    #[serde(default)]
    pub for_dest: Option<Target>,
}

impl DriveState {
    /// A new vehicle's state: ρ = 0 (S-0207 Terms).
    pub fn new() -> Self {
        Self {
            budget: 0,
            // spec: S-0207 Terms (new vehicles start with ρ = 0)
            rho: F64::ZERO,
            rho_t: F64::ZERO,
            track: None,
            turning_in_place: false,
            turn: None,
            claims: Vec::new(),
            // spec: S-0209 Terms (a vehicle that has not advanced since it was created is between steps)
            between_steps: true,
            moved_tick: None,
            requests: None,
            step: 0,
            // spec: S-0207 claim 25 (a newly created driving or ship locomotor has no locomotor destination point)
            goal: None,
            for_dest: None,
        }
    }
}

/// The height added to a high-bridge cell's surface for a supplied
/// destination point, leptons.
// spec: S-0207 claim 25 and Import mapping (the 416-lepton high-bridge lift, applied once at acceptance)
const GOAL_DECK_LIFT: i32 = 416;

/// The arrival height tolerance of the locomotor destination point,
/// leptons: a qualifying completion needs a height difference strictly
/// below it.
// spec: S-0207 claim 25 and Import mapping (the strict 208-lepton arrival tolerance)
const GOAL_HEIGHT_TOLERANCE: i32 = 208;

/// The movement point an order-level destination supplies to locomotion
/// (S-0207 claim 25): a cell's centre at its surface height, plus 416 on a
/// high-bridge cell; for an object, the depot's docking point when it
/// supplies one (S-0314 claim 18), else its position.
fn supplied_point(w: &World, id: ObjectId, dest: Target) -> Option<Pos> {
    match dest {
        Target::Cell(c) => {
            // spec: S-0204 claim 3 (the cell centre)
            let (x, y) = (256 * c.x + 128, 256 * c.y + 128);
            // spec: S-0207 claim 25 (a high-bridge target adds 416 leptons to the supplied height)
            let lift = if w.high_bridge(c) { GOAL_DECK_LIFT } else { 0 };
            Some(Pos::new(x, y, w.surface_at(x, y) + lift))
        }
        // spec: S-0314 claims 17 and 18 (an accepted enter order supplies the depot's docking point, even in the vehicle's own cell)
        // spec: S-0606 claim 4 (a vehicle approaching a bunker it is in contact with drives toward the docking point)
        Target::Object(t) => crate::depot::heading_point(w, id, t)
            .or_else(|| crate::structures::bunker::heading_point(w, id, t))
            .or_else(|| w.obj(t).map(|x| x.pos)),
    }
}

/// The locomotor takes the order-level destination just assigned (S-0207
/// claim 25): an accepted instruction with a point installs it at once,
/// before any path search, even in the vehicle's own cell; one with no
/// point (a stop) leaves none. Called when a destination is assigned, and
/// at the start of the vehicle's tick for a destination set without that
/// call.
pub fn take_destination(w: &mut World, id: ObjectId) {
    take(w, id, true);
}

fn take(w: &mut World, id: ObjectId, given: bool) {
    let Some(o) = w.obj(id) else { return };
    let Some(d) = o.drive.as_ref() else { return };
    if !given && o.dest == d.for_dest {
        return;
    }
    let dest = o.dest;
    // spec: S-0207 claim 27 (the one-application suppression skips a nonempty point: the order-level destination changes without a new locomotor destination point; claim 25's driving point is covered equally)
    if dest.is_some() && given && crate::refresh::skip_point(w, id) {
        if let Some(d) = w.obj_mut(id).and_then(|o| o.drive.as_mut()) {
            d.for_dest = dest;
        }
        return;
    }
    // spec: S-0207 claim 25 (an accepted instruction with a point replaces the old one; one with no point, or a stop that removes the order destination, leaves it absent)
    let goal = dest.and_then(|t| supplied_point(w, id, t));
    if let Some(d) = w.obj_mut(id).and_then(|o| o.drive.as_mut()) {
        d.for_dest = dest;
        d.goal = goal;
    }
}

/// The locomotor destination point goes absent at once (S-0207 claim 25,
/// claim 27's fallback stop); a begun track finishes.
pub fn stop_point(w: &mut World, id: ObjectId) {
    if let Some(d) = w.obj_mut(id).and_then(|o| o.drive.as_mut()) {
        d.goal = None;
    }
}

/// The driving movement report of a vehicle with a driving locomotor
/// (S-0207 claim 27, S-0417 claim 5): true unless there is no locomotor
/// destination point and no heading point horizontally different from the
/// vehicle's actual position. With `assigning`, only the stored point
/// counts (a destination being assigned has not been taken yet).
pub fn movement_report(w: &World, id: ObjectId, assigning: bool) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let Some(d) = o.drive.as_ref() else {
        return false;
    };
    let point = if assigning {
        d.goal.is_some()
    } else {
        goal_present(w, id)
    };
    // spec: S-0207 claim 27 (the heading point is the track's target; horizontally different from the actual position)
    let heading = d
        .track
        .as_ref()
        .is_some_and(|t| t.target != (o.pos.x, o.pos.y));
    point || heading
}

/// Whether the driving or ship locomotor destination point is present
/// (S-0207 claim 25; S-0314 claim 17). An order-level destination assigned
/// but not yet taken counts with the point it supplies, a removed one as
/// already cleared.
pub fn goal_present(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let Some(d) = o.drive.as_ref() else {
        return false;
    };
    if o.dest != d.for_dest {
        // spec: S-0207 claim 25 (assignment installs the point immediately; a stop clears it immediately)
        return o.dest.is_some_and(|t| supplied_point(w, id, t).is_some());
    }
    d.goal.is_some()
}

/// The locomotor destination point, if present.
pub fn goal(w: &World, id: ObjectId) -> Option<Pos> {
    let o = w.obj(id)?;
    let d = o.drive.as_ref()?;
    if o.dest != d.for_dest {
        return o.dest.and_then(|t| supplied_point(w, id, t));
    }
    d.goal
}

/// The qualifying track completion of S-0207 claim 25: with an order
/// destination still present, the vehicle in its supplied point's cell and
/// its height within 208 leptons of the point's, the point is cleared.
fn completion_clears_goal(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let Some(p) = o.drive.as_ref().and_then(|d| d.goal) else {
        return;
    };
    // spec: S-0207 claim 25 (an order destination still present, the vehicle in that destination's supplied movement-point cell, |vehicle height − point height| < 208, integer leptons)
    let qualifies = o.dest.is_some()
        && o.pos.cell() == p.cell()
        && (o.pos.z - p.z).abs() < GOAL_HEIGHT_TOLERANCE;
    if qualifies && let Some(d) = w.obj_mut(id).and_then(|o| o.drive.as_mut()) {
        d.goal = None;
    }
}

impl Default for DriveState {
    fn default() -> Self {
        Self::new()
    }
}

/// Whether a vehicle is between steps or at a chain point (S-0209 Terms).
pub fn between_steps_or_chain(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .and_then(|o| o.drive.as_ref())
        .is_some_and(|d| d.between_steps)
        || crate::hover::between_steps(w, id)
        || crate::lift::stand_in_between_steps(w, id)
        || at_chain_point(w, id)
}

/// Whether a vehicle is at a chain point (S-0209 Terms).
pub fn at_chain_point(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let Some(d) = o.drive.as_ref() else {
        return false;
    };
    let Some(t) = d.track.as_ref() else {
        return false;
    };
    let Some(e) = t.e else { return false };
    let Some(cp) = tracks::chain_point(t.d, e) else {
        return false;
    };
    // At the chain point: its position is that point (the step after the curve turns again: next stored step).
    let (pts, _) = tracks::track(t.d, Some(e));
    let p = pts[cp];
    let at = (t.target.0 + p.0, t.target.1 + p.1) == (o.pos.x, o.pos.y);
    let turns_again = o.path.steps.first().is_some_and(|&g| g != e);
    at && turns_again
}

/// Releases every vehicle-present claim of a vehicle.
pub fn release_claims(w: &mut World, id: ObjectId) {
    let claims = held_claims(w, id);
    for (c, layer) in claims {
        w.cells.claim_vehicle(c, layer, false);
    }
    if let Some(o) = w.obj_mut(id) {
        if let Some(d) = o.drive.as_mut() {
            d.claims.clear();
        }
        if let Some(h) = o.hover.as_mut() {
            h.claims.clear();
        }
        if let Some(t) = o.teleport.as_mut() {
            // The released records are no longer held.
            t.held = None;
            t.deck_mark = None;
        }
    }
}

/// The vehicle-present claims a vehicle holds, with their record layers:
/// the ones [`release_claims`] releases.
pub(crate) fn held_claims(w: &World, id: ObjectId) -> Vec<(CellXY, Layer)> {
    match w.obj(id) {
        // spec: S-0209 claim 1 and S-0503 claim 13 (a hovering vehicle clears the flags it set)
        Some(o) if o.hover.is_some() => o
            .hover
            .as_ref()
            .map(|h| h.claims.clone())
            .unwrap_or_default(),
        // spec: S-0209 claim 1 (a vehicle taken out of a cell's list while between steps clears that cell's flag; one without the driving locomotor never advanced)
        // spec: S-0207 claim 20 (a) and (e) (a teleporting vehicle's mark is on the cell of its landing point until the jump, in its reservation record; after it, the remembered reservation and any deck mark)
        Some(o) if o.drive.is_none() && o.kind == Kind::Vehicle => {
            crate::teleport::removal_marks(w, id)
        }
        Some(o) => o
            .drive
            .as_ref()
            .map(|d| d.claims.iter().map(|&c| (c, o.layer)).collect())
            .unwrap_or_default(),
        None => Vec::new(),
    }
}

/// Sets the vehicle-present flag of `c` for the vehicle (S-0209 claim 1).
fn claim(w: &mut World, id: ObjectId, c: CellXY) {
    let Some(o) = w.obj(id) else { return };
    let layer = o.layer;
    if o.drive.as_ref().is_some_and(|d| d.claims.contains(&c)) {
        return;
    }
    w.cells.claim_vehicle(c, layer, true);
    if let Some(d) = w.obj_mut(id).and_then(|o| o.drive.as_mut()) {
        d.claims.push(c);
        d.claims.sort();
    }
}

/// Clears the vehicle's flag of `c`, in the cell layer of its position
/// (S-0209 claim 1).
fn release(w: &mut World, id: ObjectId, c: CellXY) {
    let Some(o) = w.obj(id) else { return };
    let layer = o.layer;
    if !o.drive.as_ref().is_some_and(|d| d.claims.contains(&c)) {
        return;
    }
    w.cells.claim_vehicle(c, layer, false);
    if let Some(d) = w.obj_mut(id).and_then(|o| o.drive.as_mut()) {
        d.claims.retain(|x| *x != c);
    }
}

/// The braking floor F₀.₃ (single-precision 0.3 widened, S-0207 claim 10).
fn f03() -> F64 {
    // spec: S-0207 claim 10 (F₀.₃ = 0.300000011920928955078125, single-precision 0.3)
    crate::consts::BRAKE_FLOOR
}

/// The target fraction of a track entering `cell` (S-0207 claim 9, F6).
fn target_fraction(w: &World, id: ObjectId, u: &UnitRules, cell: CellXY) -> F64 {
    let r = w.rules();
    let mode = r.mode;
    let Some(o) = w.obj(id) else { return F64::ZERO };
    let here = o.pos.cell();
    // spec: S-0207 claim 9 (a) (λ: the current cell's level, plus 4 on a deck)
    let lambda = w.cells.cell(here).map_or(0, |c| i32::from(c.level))
        + if o.layer == Layer::Deck { 4 } else { 0 };
    let entered = w.cells.cell(cell);
    let lv = entered.map_or(0, |c| i32::from(c.level));
    // spec: S-0207 claim 9 (a) (a difference below 2 uses the cell's land type, otherwise road)
    let l = if (lv - lambda).abs() < 2 {
        entered.map_or(land::CLEAR, |c| c.land)
    } else {
        land::ROAD
    };
    let one = F64::from_i64(1, mode);
    let class = u.class.unwrap_or(LocomotionClass::Wheel);
    // spec: S-0207 claim 9 (b) (g = min(M(ℓ, c), 1), widened to double)
    let g = r.multiplier(l, class).to_f64().min(one);
    // spec: S-0207 claim 9 (c) (uphill or downhill factor by the surface heights; tracked or wheeled)
    let h_entered = w.surface_at(256 * cell.x + 128, 256 * cell.y + 128);
    let h_here = w.surface_at(o.pos.x, o.pos.y);
    let tracked = class == LocomotionClass::Track;
    let s = &r.general.slopes;
    let uf = if h_entered > h_here {
        if tracked { s[0] } else { s[2] } // spec: S-0207 claim 9 (c) (tracked uphill, wheeled uphill)
    } else if h_entered < h_here {
        if tracked { s[1] } else { s[3] } // spec: S-0207 claim 9 (c) (tracked downhill, wheeled downhill)
    } else {
        one
    };
    let gu = g.mul(uf, mode);
    // spec: S-0207 claim 9 (d) (0.5 where g × u = 0 exactly)
    let z = if gu.eq_num(F64::ZERO) {
        crate::consts::HALF
    } else {
        gu
    };
    // spec: S-0207 claim 9 (e) and S-0108 F3 (0.75 when health / strength ≤ yellow, divided in binary64)
    let strength = u.strength;
    let damaged = strength != 0
        && F64::from_i64(i64::from(o.health), mode)
            .div(F64::from_i64(i64::from(strength), mode), mode)
            .le(r.general.condition_yellow);
    let h = if damaged {
        crate::consts::THREE_QUARTERS
    } else {
        one
    };
    h.mul(z, mode)
}

/// One update of the speed fraction (S-0207 claim 10).
fn update_rho(w: &mut World, id: ObjectId, u: &UnitRules) {
    let r = w.rules_arc();
    let mode = r.mode;
    let Some(o) = w.obj(id).cloned() else { return };
    let Some(d) = o.drive.clone() else { return };
    let one = F64::from_i64(1, mode);
    let clamp = |x: F64| x.max(F64::ZERO).min(one);
    let exit = d.track.as_ref().is_some_and(|t| t.exit);
    let (rho, rho_t) = if exit && u.accelerates {
        // spec: S-0417 claim 6 (a type marked to accelerate gets no ordinary acceleration on the exit track)
        (d.rho, d.rho_t)
    } else if !u.accelerates {
        // spec: S-0207 claim 10 (acceleration flag off: ρ = clamp(ρₜ, 0, 1))
        (clamp(d.rho_t), d.rho_t)
    } else if u.passive {
        // spec: S-0207 claim 10 (passive with acceleration: no case applies)
        (d.rho, d.rho_t)
    } else {
        let v = F64::from_i64(i64::from(u.speed), mode);
        let dm = path::heading_point(w, id).map(|p| {
            let dx = i64::from(p.x - o.pos.x);
            let dy = i64::from(p.y - o.pos.y);
            let dz = i64::from(p.z - o.pos.z);
            // spec: S-0207 F5 (dₘ = ⌊r(Δx² + Δy² + Δz²)⌋)
            root_floor(
                (dx * dx + dy * dy + dz * dz) as u64,
                r.quirks.table_root_distance,
                mode,
            ) as i64
        });
        let braking = dm.is_some_and(|dm| dm < i64::from(u.slowdown_distance));
        let floor = f03();
        if o.crush_slowdown {
            // spec: S-0207 claim 10 case 1 and S-0210 F4 (min(ρₜ, 0.2); the target takes this value)
            let cap = crate::consts::CRUSH_CAP;
            let x = d.rho_t.min(cap);
            (clamp(x), x)
        } else if braking && (r.quirks.drive_brake_floor_raises || d.rho.gt(floor)) {
            // spec: S-0207 claim 10 case 2 (max(ρ − V × η, F₀.₃))
            (
                clamp(d.rho.sub(v.mul(u.deceleration, mode), mode).max(floor)),
                d.rho_t,
            )
        } else {
            // spec: S-0207 claim 10 cases 4 to 6 (accelerating, decelerating, steady)
            let x = if d.rho.lt(d.rho_t) {
                d.rho.add(u.acceleration, mode).min(d.rho_t)
            } else if d.rho.gt(d.rho_t) {
                d.rho.sub(v.mul(u.deceleration, mode), mode).max(d.rho_t)
            } else {
                d.rho
            };
            // spec: S-0207 Edge cases (`quirks.drive_brake_floor_raises` off: capped at F₀.₃ within the slowdown distance)
            let x = if braking { x.min(floor) } else { x };
            (clamp(x), d.rho_t)
        }
    };
    if let Some(dd) = w.obj_mut(id).and_then(|o| o.drive.as_mut()) {
        dd.rho = rho;
        dd.rho_t = rho_t;
    }
}

/// Progress per tick s (S-0207 claim 2, F2), with the speed fraction the
/// vehicle holds now.
pub(crate) fn progress(w: &World, id: ObjectId, u: &UnitRules) -> i32 {
    let rho = w
        .obj(id)
        .and_then(|o| o.drive.as_ref())
        .map_or(F64::ZERO, |d| d.rho);
    progress_with(w, id, u, rho)
}

/// Progress per tick s (S-0207 claim 2, F2) with the speed fraction `rho`;
/// the hovering locomotor's *s* (S-0503 claim 4) uses its own ρ.
pub(crate) fn progress_with(w: &World, id: ObjectId, u: &UnitRules, rho: F64) -> i32 {
    let r = w.rules();
    let mode = r.mode;
    let Some(o) = w.obj(id) else { return 0 };
    let kappa = o
        .owner
        .and_then(|h| r.countries.get(w.houses[usize::from(h.0)].country as usize))
        .map_or(F64::from_i64(1, mode), |c| c.speed_units);
    // spec: S-0207 F2 (s₁ = trunc((V × κ) × b); s₂ = trunc(s₁ × ν) with the faster ability, else s₁; s = trunc(s₂ × ρ))
    let s1 = F64::from_i64(i64::from(u.speed), mode)
        .mul(kappa, mode)
        .mul(o.speed_bonus, mode)
        .to_i32_common();
    let s2 = if crate::combat::veterancy::has_ability(w, id, crate::combat::data::Ability::Faster) {
        F64::from_i64(i64::from(s1), mode)
            .mul(r.general.veteran_speed, mode)
            .to_i32_common()
    } else {
        s1
    };
    F64::from_i64(i64::from(s2), mode)
        .mul(rho, mode)
        .to_i32_common()
}

/// Keeps the tick's step s (S-0207 claim 2) for S-0402 claim 5a.
fn record_step(w: &mut World, id: ObjectId, s: i32) {
    if let Some(d) = w.obj_mut(id).and_then(|o| o.drive.as_mut()) {
        d.step = s;
    }
}

/// The speed estimate of a vehicle for the lead of S-0306 claim 3 and F6
/// (S-0207 claim 18, F2a): F2 with the speed fraction the vehicle holds now,
/// read without updating the fraction or earning progress. The integer is
/// taken as leptons per tick by the lead, with no conversion. A hovering
/// vehicle holds its own fraction (S-0503 claim 4). A vehicle with neither
/// a driving nor a hovering state holds no fraction here, so its estimate is
/// 0 (the fraction of another locomotor is not defined yet, S-0306 Open
/// questions). No object carries a flag in this simulation, so the
/// flag-carrier halving of F2 never applies.
// spec: S-0207 claim 18 and F2a; S-0306 claim 3 (the speed estimate u)
pub fn speed_estimate(w: &World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id) else { return 0 };
    let Some(u) = w.rules().unit(o.kind, o.type_index) else {
        return 0;
    };
    if let Some(rho) = crate::hover::speed_fraction(w, id) {
        // spec: S-0306 claim 3 and Open questions (a hovering vehicle's speed fraction is S-0503's) and S-0503 claim 4 (its s contains its own ρ)
        return progress_with(w, id, u, rho);
    }
    progress(w, id, u)
}

fn cell_of(x: i32, y: i32) -> CellXY {
    Pos::new(x, y, 0).cell()
}

/// Moves the vehicle to a new ground position, handling the cell change
/// (S-0209 claim 13, S-0211 claim 5, S-0210 claim 9) and getting onto or
/// off a bridge deck (S-0207 claim 5). `at_point`: the position is a track
/// point or the track target, where the world height is set (claim 5);
/// between points (claim 4) the height is kept.
fn set_pos(w: &mut World, id: ObjectId, x: i32, y: i32, at_point: bool) {
    let Some(o) = w.obj(id).cloned() else { return };
    let old = o.pos.cell();
    let new = cell_of(x, y);
    let mut layer = o.layer;
    if new != old {
        let high = |c: CellXY| w.cells.idx(c).is_some_and(|i| w.cells.deck[i]);
        let level = |c: CellXY| w.cells.cell(c).map_or(0, |x| i32::from(x.level));
        // spec: S-0207 claim 5 (onto the deck: into a high-bridge cell exactly 4 levels below the cell left)
        if high(new) && level(new) == level(old) - 4 {
            layer = Layer::Deck;
        } else if high(old) && !high(new) {
            // spec: S-0207 claim 5 (off the deck: from a high-bridge cell into one that is not)
            layer = Layer::Ground;
        }
    }
    let z = if at_point {
        // spec: S-0207 claim 5 (the surface height H, plus 416 leptons on a bridge deck)
        w.surface_at(x, y) + if layer == Layer::Deck { 416 } else { 0 }
    } else {
        o.pos.z
    };
    if let Some(ob) = w.obj_mut(id) {
        ob.pos = Pos::new(x, y, z);
        ob.layer = layer;
    }
    if new != old {
        w.cells.leave(old, o.layer, id);
        w.cells.enter(new, layer, id, false);
        // spec: S-0212 claims 13 and 16a (recorded in a cell of another threat block: its threat amount moves there)
        crate::threat::relisted(w, id, new);
        crate::load::recalc_cells(w, &[old, new]);
        crate::crush::entered_cell(w, id, new);
    }
}

/// The cell one step from `c` in step direction `d`.
pub(crate) fn neighbour(c: CellXY, d: u8) -> CellXY {
    let (dx, dy) = STEPS[usize::from(d % 8)]; // spec: S-0212 Terms (step directions 0 to 7)
    CellXY::new(c.x + dx, c.y + dy)
}

/// The facing of 8-direction `d` (S-0207 claim 6: 8192 × d).
fn facing_of(d: u8) -> u16 {
    // spec: S-0207 claim 6 (a track begins only when the body facing equals 8192 × d exactly)
    u16::from(d % 8) * 8192
}

/// The entry outcome a step check uses, with the substitutions of S-0209
/// claim 5f (S-0207 Terms, *step check*).
fn checked_outcome(w: &World, id: ObjectId, u: &UnitRules, from: CellXY, to: CellXY) -> Outcome {
    let r = w.rules();
    let reference = occupancy::reference_level(w, id);
    let mut outcome = path::step_outcome(w, id, from, to, reference);
    if u.train && outcome != Outcome::Blocked {
        // spec: S-0209 claim 5f (a train: every outcome other than blocked counts as clear)
        outcome = Outcome::Clear;
    }
    if u.crusher
        && r.quirks.crusher_overlay_index0_clear
        && matches!(outcome, Outcome::FriendlyObstacle | Outcome::EnemyObstacle)
        && w.cells.cell(to).is_some_and(|c| c.overlay == 0)
    {
        // spec: S-0209 claim 5f (a crusher-flag vehicle: obstacles count as clear on overlay index 0)
        outcome = Outcome::Clear;
    }
    outcome
}

/// Begins a track from the vehicle's current cell (S-0207 claims 6, 7 and
/// 15). Returns `false` when the step cannot begin this tick.
fn begin_track(w: &mut World, id: ObjectId, u: &UnitRules) -> bool {
    let t = w.tick;
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    let Some(&d) = o.path.steps.first() else {
        return false;
    };
    if crate::spawn::track_held(w, id) {
        // spec: S-0207 claim 6 and S-0506 claim 10 (a launching vehicle begins no track: it waits, destination kept; precedence against other waits not established, checked first here)
        wait(w, id);
        trace::record(w, id, || trace::What::Waited { why: "launching" });
        return false;
    }
    let first = neighbour(o.pos.cell(), d);
    if let Some(gate) = process::closed_friendly_gate(w, id, first) {
        trace::step(
            w,
            id,
            trace::Mover::Drive,
            o.pos.cell(),
            first,
            Outcome::ClosedFriendlyGate,
            false,
        );
        // spec: S-0207 claim 15 (an allied closed gate: told to open; the vehicle waits without a track and without turning)
        process::gate_met(w, gate);
        wait(w, id);
        return false;
    }
    if o.body.value(t) != facing_of(d) {
        turn_in_place(w, id, u, d);
        return false;
    }
    if let Some((t0, dur)) = o.drive.as_ref().and_then(|x| x.turn)
        && o.drive.as_ref().is_some_and(|x| x.turning_in_place)
    {
        // spec: S-0206 claim 12 (a turn completing at once begins the track in tick t₀ + 1; otherwise in t₀ + Θ)
        let begin = if dur == 0 { t0 + 1 } else { t0 + dur };
        if t < begin {
            wait(w, id);
            return false;
        }
        if let Some(dd) = w.obj_mut(id).and_then(|o| o.drive.as_mut()) {
            dd.turning_in_place = false;
            dd.turn = None;
        }
    }
    step_check(w, id, u, false, false)
}

/// A turn in place towards the facing of step `d` (S-0207 claim 6). A turn
/// in place towards another facing (the path changed) is replaced.
fn turn_in_place(w: &mut World, id: ObjectId, u: &UnitRules, d: u8) {
    let t = w.tick;
    let Some(o) = w.obj(id) else { return };
    let want = facing_of(d);
    if o.drive.as_ref().is_some_and(|x| x.turning_in_place) && o.body.target == want {
        return;
    }
    let mut body = o.body;
    body.turn_to(want, t, u.turn_rate);
    let dur = body.duration;
    if let Some(ob) = w.obj_mut(id) {
        ob.body = body;
        if let Some(dd) = ob.drive.as_mut() {
            dd.turning_in_place = true;
            dd.turn = Some((t, dur));
            // spec: S-0207 claim 3 (the budget is set to 0 in every tick of a turn in place)
            dd.budget = 0;
        }
    }
}

/// The step check (S-0207 claim 15, tables T4 and T5): a first check, or
/// the repeated check that follows a search made at once. Begins the track
/// when the check lets it; returns whether it did. With `no_curve` the
/// check cannot begin a two-cell curve (the step check after the search of
/// T5's obstacle row).
fn step_check(w: &mut World, id: ObjectId, u: &UnitRules, repeated: bool, no_curve: bool) -> bool {
    let t = w.tick;
    let r = w.rules_arc();
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    let Some(&d) = o.path.steps.first() else {
        return false;
    };
    let here = o.pos.cell();
    let first = neighbour(here, d);
    let outcome = checked_outcome(w, id, u, here, first);
    trace::step(w, id, trace::Mover::Drive, here, first, outcome, repeated);
    match outcome {
        Outcome::Clear => {
            // spec: S-0520 claim 6 (a driving or ship mover collects the crate of the cell it commits to, before it arrives; a successful unit crate blocks the step and clears its movement target)
            if crate::crates::pickup(w, id, first) {
                discard_path(w, id);
                return false;
            }
        }
        Outcome::ClosedFriendlyGate => {
            // spec: S-0207 claim 15 table T4 (closed friendly gate: it tells the gate to open and waits)
            if let Some(gate) = process::closed_friendly_gate(w, id, first) {
                process::gate_met(w, gate);
            }
            wait(w, id);
            return false;
        }
        Outcome::MovingBlocker => {
            // spec: S-0207 claim 15 table T4 (moving blocker: S-0212 claim 22)
            blocked_step(w, id);
            return false;
        }
        outcome @ (Outcome::FriendlyObstacle
        | Outcome::EnemyObstacle
        | Outcome::StationaryFriendly) => {
            if !repeated {
                // spec: S-0207 claim 15 table T4 (first check: discard the path, search at once whatever the retry timer, then the repeated check)
                return search_and_recheck(w, id, u);
            }
            wait(w, id);
            if outcome == Outcome::StationaryFriendly {
                // spec: S-0207 claim 15 table T4 (repeated check, stationary friendly: give up, or a scatter request to the cell)
                // spec: S-0207 Edge cases (`quirks.drive_giveup_ignores_contact`: the step check's give-up test does not ask about contact)
                let contact = !r.quirks.drive_giveup_ignores_contact;
                if give_up_test(w, id, contact) {
                    give_up(w, id);
                } else {
                    request_scatter(w, id, first);
                }
            } else {
                // spec: S-0207 claim 15 table T4 (repeated check, obstacle: wait, and attack what blocks)
                attack_obstacle(w, id, first);
            }
            return false;
        }
        Outcome::CloakedBlocker => {
            // spec: S-0207 claim 15 table T4 (cloaked blocker); S-0516 claim 11 (e) (the cell's ground-layer cloaked objects start uncloaking)
            crate::cloak::machine::blocked_reveal(w, first);
            if repeated {
                give_up(w, id);
                return false;
            }
            discard_path(w, id);
            wait(w, id);
            // spec: S-0207 claim 15 table T4 (the fresh search on the same tick only when the path retry timer has run out); S-0212 claim 21 (a changed destination has ended it)
            if !path::waits(w, id) {
                return search_and_recheck(w, id, u);
            }
            return false;
        }
        Outcome::Blocked => {
            if repeated {
                // spec: S-0207 claim 15 table T4 (repeated check, blocked: it gives up the move)
                give_up(w, id);
                return false;
            }
            // design: C7-P4 stalled blockers (a step blocked by a unit with a destination, the close case of S-0212 claim 27 among them, starts the blockage timer as a moving blocker does, so a deadlock of such steps is seen)
            if !r.quirks.path_through_stalled_blockers
                && w.cells.list(first, Layer::Ground).iter().any(|&b| {
                    b != id
                        && w.obj(b).is_some_and(|x| {
                            matches!(x.kind, Kind::Vehicle | Kind::Infantry) && x.dest.is_some()
                        })
                })
                && let Some(ob) = w.obj_mut(id)
            {
                ob.path.blocked_since.get_or_insert(t);
            }
            // spec: S-0207 claim 15 table T4 and S-0212 claim 22 (first check, blocked: clear the stored path, a fresh search in mode 0 on the same tick whatever the retry timer, then the repeated check)
            return search_and_recheck(w, id, u);
        }
    }
    // spec: S-0207 claim 7 (the next step chooses the track; crush mode drives straight)
    let e = o.path.steps.get(1).copied();
    let second = neighbour(first, e.unwrap_or(d));
    let crush = crush_mode(w, id, u, first, second);
    // spec: S-0207 claim 15 (after the search of T5's obstacle row the step check cannot begin a two-cell curve)
    let mut use_e = if crush || no_curve { None } else { e };
    let (_, two) = tracks::track(d, use_e);
    if two {
        // spec: S-0207 claim 15 table T5 (the second cell of a curve, checked after the first gave clear)
        let outcome = checked_outcome(w, id, u, first, second);
        trace::step(w, id, trace::Mover::Drive, first, second, outcome, repeated);
        match outcome {
            Outcome::Clear => {}
            Outcome::MovingBlocker => {
                // spec: S-0207 claim 7 and table T5 (moving blocker: the straight or diagonal track into the first cell; path kept)
                use_e = None;
            }
            Outcome::ClosedFriendlyGate => {
                // spec: S-0207 table T5 (closed friendly gate: the gate is told to open; path discarded, stop at once with ρ = 0)
                if let Some(gate) = process::closed_friendly_gate(w, id, second) {
                    process::gate_met(w, gate);
                }
                stop_at_once(w, id);
                return false;
            }
            Outcome::FriendlyObstacle | Outcome::EnemyObstacle => {
                // spec: S-0207 table T5 (obstacle: discard the path; a fresh search once the retry timer has run out)
                discard_path(w, id);
                wait(w, id);
                // spec: S-0212 claim 21 (a changed destination has ended the retry timer)
                if !path::waits(w, id) && fresh_search(w, id, u) == Search::Found {
                    // spec: S-0207 claim 15 (after T5's obstacle row: the look at the new first cell, then, facing the new first step, a step check of the same kind that begins only a straight or diagonal track; not facing it, it turns and makes no step check)
                    let Some(nd) = w.obj(id).and_then(|x| x.path.steps.first().copied()) else {
                        return false;
                    };
                    if w.obj(id).is_none_or(|x| x.dest.is_none()) {
                        return false;
                    }
                    if w.obj(id).is_some_and(|x| x.body.value(t) != facing_of(nd)) {
                        turn_in_place(w, id, u, nd);
                        return false;
                    }
                    return step_check(w, id, u, repeated, true);
                }
                return false;
            }
            Outcome::StationaryFriendly => {
                if !repeated {
                    // spec: S-0207 table T5 (stationary friendly, first check: as in T4)
                    return search_and_recheck(w, id, u);
                }
                // spec: S-0207 table T5 (repeated check: the give-up test of F7 without contact, or the request to the second cell; stop at once)
                if give_up_test(w, id, false) {
                    give_up(w, id);
                } else {
                    request_scatter(w, id, second);
                }
                stop_at_once(w, id);
                return false;
            }
            Outcome::CloakedBlocker => {
                // spec: S-0207 table T5 (cloaked blocker: as in T4); S-0516 claim 11 (e)
                crate::cloak::machine::blocked_reveal(w, second);
                if repeated {
                    give_up(w, id);
                    return false;
                }
                discard_path(w, id);
                wait(w, id);
                // spec: S-0207 table T5 (cloaked blocker: as in T4); S-0212 claim 21 (a changed destination has ended the retry timer)
                if !path::waits(w, id) {
                    return search_and_recheck(w, id, u);
                }
                return false;
            }
            Outcome::Blocked => {
                if repeated {
                    // spec: S-0207 table T5 (repeated check, blocked: it gives up the move)
                    give_up(w, id);
                    return false;
                }
                // spec: S-0207 table T5 (blocked: discard the path and search at once)
                return search_and_recheck(w, id, u);
            }
        }
    }
    let (_, two) = tracks::track(d, use_e);
    let target_cell = if two { second } else { first };
    let target = (256 * target_cell.x + 128, 256 * target_cell.y + 128); // spec: S-0204 claim 3 (a cell centre: (256x + 128, 256y + 128))
    let cells = if two {
        vec![first, second]
    } else {
        vec![first]
    };
    // spec: S-0212 claim 20 (the drive locomotor recomputes as it begins the last stored step, heading point beyond 512)
    let used = if two { 2 } else { 1 };
    let last_step = o.path.steps.len() <= used;
    let rho_t = target_fraction(w, id, u, first);
    if let Some(ob) = w.obj_mut(id) {
        ob.path.steps.drain(..used.min(ob.path.steps.len()));
        if let Some(dd) = ob.drive.as_mut() {
            dd.track = Some(Track {
                d,
                e: use_e,
                target,
                target_cell,
                // spec: S-0207 claim 24 (the heading point receives the vehicle's current world height when the track begins, before any progress: an exact copy, no terrain sampling or deck lift)
                height: ob.pos.z,
                next: 0,
                last: (ob.pos.x, ob.pos.y),
                cells: cells.clone(),
                crush,
                exit: false,
                bunker: None,
            });
            dd.rho_t = rho_t;
        }
        // spec: S-0212 claim 22 (a blockage ends once the unit has taken a step again)
        ob.path.blocked_since = None;
    }
    // spec: S-0209 claim 1 (a step sets the flag of the cell it enters at once; a two-cell curve both cells)
    for c in cells {
        claim(w, id, c);
    }
    // spec: S-0210 claim 7 (a crusher warns the infantry of the first cell each time it begins a track, crush mode or not)
    if occupancy::is_crusher(w, id) {
        crate::crush::warn_ahead(w, id, first);
    }
    if last_step && let Some(hp) = path::heading_point(w, id) {
        let o2 = w.obj(id).cloned();
        if let Some(o2) = o2 {
            let dx = i64::from(hp.x - o2.pos.x);
            let dy = i64::from(hp.y - o2.pos.y);
            let dz = i64::from(hp.z - o2.pos.z);
            // spec: S-0212 F6c (the heading point more than 512 leptons away)
            let dist = root_floor(
                (dx * dx + dy * dy + dz * dz) as u64,
                r.quirks.table_root_distance,
                r.mode,
            );
            // spec: S-0212 claim 20 (the heading point more than 512 leptons away)
            if dist > 512 && hp.cell() != target_cell {
                refresh_from_target(w, id, target_cell);
            }
        }
    }
    true
}

/// A vehicle that waits gets no track on this tick and its budget is
/// emptied (S-0207 claims 3 and 15).
fn wait(w: &mut World, id: ObjectId) {
    set_budget_zero(w, id);
}

/// Discards the stored path.
pub(crate) fn discard_path(w: &mut World, id: ObjectId) {
    if let Some(ob) = w.obj_mut(id) {
        ob.path.steps.clear();
    }
}

/// Discards the path and stops at once with ρ = 0 (S-0207 table T5).
fn stop_at_once(w: &mut World, id: ObjectId) {
    discard_path(w, id);
    if let Some(d) = w.obj_mut(id).and_then(|o| o.drive.as_mut()) {
        // spec: S-0207 table T5 (it discards its stored path and stops at once with ρ = 0)
        d.rho = F64::ZERO;
        d.budget = 0;
    }
}

/// A first check's reaction to an obstacle or stationary friends (S-0207
/// table T4): the stored path is discarded, a fresh search is made at once,
/// and when it finds a path the repeated check follows on the same tick if
/// the vehicle faces the new step.
fn search_and_recheck(w: &mut World, id: ObjectId, u: &UnitRules) -> bool {
    let t = w.tick;
    discard_path(w, id);
    wait(w, id);
    if fresh_search(w, id, u) != Search::Found {
        return false;
    }
    let Some(o) = w.obj(id) else { return false };
    let Some(&d) = o.path.steps.first() else {
        return false;
    };
    // spec: S-0207 claim 15 (the repeated check follows on that tick when the vehicle faces the step)
    if o.body.value(t) != facing_of(d) || o.dest.is_none() {
        return false;
    }
    step_check(w, id, u, true, false)
}

/// A fresh search (S-0212 claim 20) and, when it finds a path, the look at
/// the first cell of the new path (S-0207 claim 15).
fn fresh_search(w: &mut World, id: ObjectId, u: &UnitRules) -> Search {
    let res = path::compute(w, id, Mode::Fresh);
    if res != Search::Found {
        path::failed(w, id, res, false);
        return res;
    }
    if crate::refresh::stopped(w, id) {
        // spec: S-0402 claim 10a (A chrono miner: a substitute the computation adopted got claim 27's fallback; the miner stops and the substitute is not driven to)
        if let Some(ob) = w.obj_mut(id) {
            ob.path.steps.clear();
        }
        return Search::SearchFailed;
    }
    first_cell_look(w, id, u);
    res
}

/// The look at the first cell of a new path (S-0207 claim 15).
fn first_cell_look(w: &mut World, id: ObjectId, u: &UnitRules) {
    let Some(o) = w.obj(id).cloned() else { return };
    let Some(&d) = o.path.steps.first() else {
        return;
    };
    let here = o.pos.cell();
    let cell = neighbour(here, d);
    // spec: S-0207 claim 15 (first cell of a new path: an allied closed gate is told to open)
    if let Some(gate) = process::closed_friendly_gate(w, id, cell) {
        process::gate_met(w, gate);
    }
    // spec: S-0207 claim 15 (stationary friendly, not a train, the object nearest the cell's centre allied)
    if u.train || checked_outcome(w, id, u, here, cell) != Outcome::StationaryFriendly {
        return;
    }
    // spec: S-0207 claim 15 (the deck layer when z > H + 208 in exact integer leptons)
    let Some(near) = nearest_object(w, id, cell, 208) else {
        return;
    };
    let near_owner = w.obj(near).and_then(|x| x.owner);
    if !w.allied(o.owner, near_owner) {
        return;
    }
    // spec: S-0207 claim 15 and F7 (the look's give-up test adds contact with nothing)
    if give_up_test(w, id, true) {
        give_up(w, id);
    } else {
        request_scatter(w, id, cell);
    }
}

/// The nearest object of the look at a new path's first cell `c` (S-0207
/// claim 15, F8), looked for in the deck layer when the vehicle's world
/// height is more than `deck_above` leptons (208 for driving, 312 for
/// hovering, S-0503 claim 12) above the surface height at the cell's
/// centre, otherwise in the ground layer. Only infantry units, vehicles,
/// aircraft and buildings take part; each one's distance dₙ is measured
/// along the map from the top corner of the cell holding its position;
/// the smallest dₙ wins, ties to the object listed first.
pub(crate) fn nearest_object(
    w: &World,
    id: ObjectId,
    c: CellXY,
    deck_above: i32,
) -> Option<ObjectId> {
    let o = w.obj(id)?;
    let cx = 256 * c.x + 128; // spec: S-0204 claim 3 (a cell centre: (256x + 128, 256y + 128))
    let cy = 256 * c.y + 128; // spec: S-0204 claim 3 (a cell centre)
    let h = w.surface_at(cx, cy);
    let layer = if o.pos.z > h + deck_above {
        Layer::Deck
    } else {
        Layer::Ground
    };
    let mut best: Option<(u64, ObjectId)> = None;
    for &x in w.cells.list(c, layer) {
        let Some(ob) = w.obj(x) else { continue };
        // spec: S-0207 claim 15 (only infantry units, vehicles, aircraft and buildings take part; the vehicle itself is not left out)
        if !matches!(
            ob.kind,
            Kind::Infantry | Kind::Vehicle | Kind::Aircraft | Kind::Building
        ) {
            continue;
        }
        let dist = nearest_distance(w, ob.pos.x, ob.pos.y);
        // spec: S-0207 claim 15 and F8 (the smallest dₙ, compared as integers; ties to the one listed first, in S-0209 claim 13's listing order)
        if best.is_none_or(|(b, _)| dist < b) {
            best = Some((dist, x));
        }
    }
    best.map(|b| b.1)
}

/// The distance dₙ of S-0207 F8 for an object at (`x`, `y`) in leptons:
/// ⌊r(u² + w²)⌋ with u = x mod 256 and w = y mod 256 (from 0 to 255), the
/// position within the cell holding it measured from that cell's top
/// corner; heights play no part.
pub fn nearest_distance(w: &World, x: i32, y: i32) -> u64 {
    let r = w.rules();
    // spec: S-0207 F8 (u = x mod 256, w = y mod 256, each the residue from 0 to 255)
    let (mut u, mut v) = (i64::from(x.rem_euclid(256)), i64::from(y.rem_euclid(256)));
    if !r.quirks.drive_nearest_from_top_corner {
        // spec: S-0207 F8 (`quirks.drive_nearest_from_top_corner` off: (x mod 256) − 128 and (y mod 256) − 128, from the centre)
        u -= 128; // spec: S-0207 F8 (switch off: (x mod 256) − 128)
        v -= 128; // spec: S-0207 F8 (switch off: (y mod 256) − 128)
    }
    // spec: S-0207 F8 (dₙ = ⌊r(n)⌋ with S-0204's table square root; `quirks.table_root_distance` off: ⌊√n⌋)
    root_floor((u * u + v * v) as u64, r.quirks.table_root_distance, r.mode)
}

/// The give-up test of S-0207 F7; `contact` adds the condition that the
/// vehicle is in contact with nothing.
pub(crate) fn give_up_test(w: &World, id: ObjectId, contact: bool) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let r = w.rules();
    let Some(dest) = path::heading_point(w, id) else {
        return false;
    };
    let dx = i64::from(dest.x - o.pos.x);
    let dy = i64::from(dest.y - o.pos.y);
    let dz = i64::from(dest.z - o.pos.z);
    // spec: S-0207 F7 (dₘ of F5 below the close-enough distance)
    let dm = root_floor(
        (dx * dx + dy * dy + dz * dz) as u64,
        r.quirks.table_root_distance,
        r.mode,
    ) as i64;
    let tunnel = w
        .cells
        .cell(o.pos.cell())
        .is_some_and(|c| c.land == land::TUNNEL);
    // spec: S-0207 F7 (dₘ < C, |zᵈ − zᵛ| < 208, the vehicle's own cell not a tunnel)
    dm < i64::from(r.general.close_enough)
        && dz.abs() < 208
        && !tunnel
        && (!contact || o.contact.is_none())
}

/// Giving up the move (S-0207 Terms): no track on this tick; with approach
/// destinations it stops and falls back to rest (S-0215 claim 13),
/// otherwise its destination is removed.
pub(crate) fn give_up(w: &mut World, id: ObjectId) {
    trace::record(w, id, || trace::What::GaveUp);
    let approach = w.obj(id).is_some_and(|o| !o.queued_dests.is_empty());
    if let Some(ob) = w.obj_mut(id) {
        ob.path.steps.clear();
        ob.dest = None;
        if let Some(d) = ob.drive.as_mut() {
            // spec: S-0207 claim 11 (a vehicle at a track target with no further step is at rest: ρ = 0)
            d.rho = F64::ZERO;
            d.budget = 0;
        }
    }
    if approach {
        // spec: S-0207 Terms (giving up with approach destinations: it stops and falls back to rest)
        crate::process::fall_back(w, id, false);
    }
}

/// The scatter request of a blocked vehicle (S-0207 claim 15): no threat
/// point, forced and move-anywhere, to every object listed in the cell layer
/// it drives on.
pub(crate) fn request_scatter(w: &mut World, id: ObjectId, c: CellXY) {
    trace::record(w, id, || trace::What::AskedRoom { cell: c });
    let Some(o) = w.obj(id) else { return };
    let deck = w.cells.idx(c).is_some_and(|i| w.cells.deck[i]);
    let cell_level = w.cells.cell(c).map_or(0, |x| i32::from(x.level));
    // spec: S-0207 claim 15 (the vehicle's height level trunc(z / 104), rounded toward zero)
    let level = o.pos.z / 104;
    // spec: S-0207 claim 15 (the deck layer of a high-bridge cell when the levels differ by 3 or more)
    let layer = if deck && (level - cell_level).abs() >= 3 {
        Layer::Deck
    } else {
        Layer::Ground
    };
    if !w.rules().quirks.drive_duplicate_scatter_request {
        // spec: S-0207 Edge cases (quirks.drive_duplicate_scatter_request off: of one vehicle's requests to the same cell and layer in a tick only the earliest is sent)
        let tick = w.tick;
        let Some(o) = w.obj_mut(id) else { return };
        // A hovering vehicle keeps its own record of the tick's requests (S-0503 claim 12).
        let requests = match (o.drive.as_mut(), o.hover.as_mut()) {
            (Some(d), _) => &mut d.requests,
            (None, Some(h)) => &mut h.requests,
            (None, None) => return,
        };
        let sent = requests.get_or_insert_with(|| (tick, Vec::new()));
        if sent.0 != tick {
            *sent = (tick, Vec::new());
        }
        if sent.1.contains(&(c, layer)) {
            return;
        }
        sent.1.push((c, layer));
    }
    crate::scatter::cell_request(
        w,
        c,
        layer,
        crate::scatter::ScatterRequest {
            threat: None,
            forced: true,
            anywhere: true,
        },
    );
}

/// The target of a vehicle refused by an obstacle at its repeated check
/// (S-0207 table T4): the first object listed in the cell when the
/// vehicle's house is not allied with it, else, with no object listed, the
/// cell's wall whatever its owner. It attacks by an override (S-0215 claim
/// 10).
pub(crate) fn attack_obstacle(w: &mut World, id: ObjectId, c: CellXY) {
    let Some(o) = w.obj(id).cloned() else { return };
    let r = w.rules_arc();
    let reference = occupancy::reference_level(w, id);
    let layer = occupancy::layers_for(w, c, reference).0;
    let target = match w.cells.list(c, layer).first().copied() {
        // spec: S-0207 table T4 (the first listed object when its house is not allied; a terrain object is never allied)
        Some(f) => {
            let fo = w.obj(f);
            let allied = fo.is_some_and(|x| x.kind != Kind::Terrain && w.allied(o.owner, x.owner));
            (!allied).then_some(Target::Object(f))
        }
        // spec: S-0207 table T4 (no object listed: the cell's wall, whatever its owner)
        None => w
            .cells
            .cell(c)
            .filter(|cell| {
                r.overlays
                    .get(usize::from(cell.overlay))
                    .is_some_and(|ov| ov.wall)
            })
            .map(|_| Target::Cell(c)),
    };
    let Some(target) = target else { return };
    let q = r.quirks;
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0215 claim 10 (an override puts the target and destination aside and replaces them)
        if ob.beh.override_with(
            Behaviour::Attack,
            q.refused_override_retargets,
            q.override_suspends_queued,
        ) {
            ob.beh.aside = Some((ob.target, ob.dest));
            ob.target = Some(target);
            // spec: S-0207 claim 15 (the override gives no destination: the move destination is put aside until the override is reverted)
            ob.dest = None;
            ob.path.steps.clear();
        }
    }
    // spec: S-0301 claim 5, Moment of a retarget's let-go (revision 67, Q-2246: at the change of the firer's target)
    crate::emitters::retarget_let_go(w, id);
}

/// The immediate recomputation of S-0212 claim 20, taken from where the
/// step in progress leads; with `quirks.path_refresh_keeps_old_step` the new
/// path's first step is dropped.
fn refresh_from_target(w: &mut World, id: ObjectId, from: CellXY) {
    let Some(o) = w.obj(id).cloned() else { return };
    // Search from the unit's cell (the original's rule), then align.
    let keep_pos = o.pos;
    let res = path::compute(w, id, Mode::Fresh);
    if res != Search::Found {
        return;
    }
    let q = w.rules().quirks;
    if let Some(ob) = w.obj_mut(id) {
        ob.pos = keep_pos;
        if q.path_refresh_keeps_old_step && !ob.path.steps.is_empty() {
            // spec: S-0212 claim 20 (the old step is still taken; the new path's first step is dropped)
            ob.path.steps.remove(0);
        }
    }
    let _ = from;
}

/// Crush mode (S-0207 claim 7).
fn crush_mode(w: &World, id: ObjectId, u: &UnitRules, first: CellXY, second: CellXY) -> bool {
    let r = w.rules();
    let all = u.zone == MovementZone::CrusherAll;
    let overlay_crush = |c: CellXY, walls: bool| -> bool {
        w.cells.cell(c).is_some_and(|cell| {
            r.overlays
                .get(usize::from(cell.overlay))
                .is_some_and(|o| o.crushable || (walls && all && o.wall))
        })
    };
    // spec: S-0607 claims 22 and 23 (a seized unit's stand-in, not its own type's locomotor, is the one that moves it)
    if crate::lift::current_locomotor(w, id) == Some(Locomotor::Ship) {
        // spec: S-0210 claim 9 (the ship locomotor's crush mode: the first cell, whose entry outcome is clear here, or the second holds a crushable overlay or, for crusher-all, a wall overlay; never set by a vehicle)
        return overlay_crush(first, true) || overlay_crush(second, true);
    }
    // spec: S-0207 claim 7 (the second cell: a crushable overlay, or for crusher-all a wall or a vehicle)
    let second_vehicle = all
        && w.cells.list(second, Layer::Ground).iter().any(|&x| {
            x != id
                && w.obj(x)
                    .is_some_and(|o| o.kind == crate::rules::Kind::Vehicle)
        });
    // spec: S-0207 claim 7 (the first cell: a crushable overlay, or for crusher-all a wall; a vehicle there does not count)
    overlay_crush(second, true) || second_vehicle || overlay_crush(first, true)
}

/// A driving, ship or hovering unit's next step is blocked by a moving
/// blocker (S-0212 claim 22, S-0207 table T4, S-0503 table T1).
pub(crate) fn blocked_step(w: &mut World, id: ObjectId) {
    let t = w.tick;
    let Some(o) = w.obj(id).cloned() else { return };
    let since = o.path.blocked_since.unwrap_or(t);
    if let Some(ob) = w.obj_mut(id) {
        ob.path.blocked_since = Some(since);
        if let Some(dd) = ob.drive.as_mut() {
            dd.budget = 0;
        }
    }
    let waiting = path::delay_runs(w, id);
    trace::record(w, id, || trace::What::Blocked {
        since,
        waiting_until: waiting.then_some(o.path.retry_at),
    });
    // spec: S-0212 claim 22 (a re-path for a blocked next step waits for the retry delay); claim 21 (a changed destination has ended it)
    if path::waits(w, id) {
        return;
    }
    let delay = w.rules().general.blockage_delay;
    // spec: S-0212 claim 22 (mode 1 while the blockage timer runs, mode 2 once it has run out)
    let mode = if t.wrapping_sub(since) >= delay {
        Mode::Avoid
    } else {
        Mode::Blocked
    };
    let res = path::compute(w, id, mode);
    let r = w.rules().general.retry_delay;
    match res {
        Search::GateFailed => path::failed(w, id, res, true),
        other => {
            // spec: S-0212 claim 21 (after a re-path for a blocked step the delay R starts again; a failed search that clears the destination ends it again for the driving locomotor, path::failed)
            if let Some(ob) = w.obj_mut(id) {
                path::start_delay(ob, t, r);
            }
            if other == Search::SearchFailed {
                path::failed(w, id, other, true);
            }
        }
    }
}

/// [`spend`], then at a reached track target the arrival-time stop of
/// S-0207 claim 19 (a): when its conditions hold the vehicle clears its
/// destination and discards its remaining steps, keeping its target, so
/// it begins no further track.
fn spend_and_stop(w: &mut World, id: ObjectId) -> bool {
    let ended = spend(w, id);
    // spec: S-0507 claim 6 (arriving at the cell of the transport it is entering: it boards, or is turned away)
    if ended && crate::transport::arrival(w, id) {
        return true;
    }
    // spec: S-0605 claim 8 (reaching a cell of the vehicle absorber it is entering: it goes in, or is refused)
    if ended && crate::structures::absorb::vehicle_arrival(w, id) {
        return true;
    }
    // spec: S-0603 claim 5 (a cell-arrival update in a cell of the building it is eaten by: it is consumed)
    if ended && crate::structures::grinder::vehicle_arrival(w, id) {
        return true;
    }
    if ended {
        // spec: S-0314 claim 17 (reaching the docking-point cell during movement: the depot's movement-arrival exchange)
        crate::depot::movement_arrival(w, id);
        // spec: S-0606 claim 5 (reaching the docking cell of the bunker it is in contact with: arrival)
        crate::structures::bunker::arrival(w, id);
    }
    // spec: S-0207 claim 19 (a) (at a track target, not between track targets; it does not wait for a behaviour processing)
    if ended
        && crate::walk::arrival_stop_holds(w, id)
        && let Some(ob) = w.obj_mut(id)
    {
        // spec: S-0207 claim 19 (a) (it clears its destination and discards all remaining path steps; it keeps its target)
        ob.dest = None;
        ob.path.steps.clear();
    }
    if ended {
        // spec: S-0417 claims 7 and 8 (a driving vehicle's cell-arrival update: reaching a track target, also when a further track follows in the same tick, Q-0223)
        crate::econ::release::arrival(w, id);
    }
    ended
}

/// Puts a released vehicle on the exit track towards cell `e`'s centre
/// (S-0417 claims 5 and 6), without a path search.
pub fn begin_exit_track(w: &mut World, id: ObjectId, e: CellXY) {
    let mode = w.rules().mode;
    let Some(o) = w.obj(id) else { return };
    let (x, y) = (o.pos.x, o.pos.y);
    // spec: S-0417 F3 (E's centre: (256 Ex + 128, 256 Ey + 128) leptons)
    let target = (256 * e.x + 128, 256 * e.y + 128);
    if let Some(ob) = w.obj_mut(id) {
        ob.path.steps.clear();
        if let Some(dd) = ob.drive.as_mut() {
            dd.track = Some(Track {
                // spec: S-0417 Constants (exit track facing 64: map east, step direction 2)
                d: 2,
                e: None,
                target,
                target_cell: e,
                // spec: S-0207 claim 24 and S-0417 claim 5 (the factory exit track's heading point stores world height 0, not the starting, terrain or deck height, with no deck lift)
                height: 0,
                next: 0,
                last: (x, y),
                cells: vec![e],
                crush: false,
                exit: true,
                bunker: None,
            });
            dd.turning_in_place = false;
            dd.turn = None;
            // spec: S-0417 claim 6 and Constants (the exit-track instruction sets the target fraction to 1.0)
            dd.rho_t = F64::from_i64(1, mode);
            // spec: S-0207 claim 25 and S-0417 claim 5 (a successful driving exit-track instruction supplies the locomotor destination point at E's horizontal centre with height 0)
            dd.goal = Some(Pos::new(target.0, target.1, 0));
            dd.for_dest = ob.dest;
        }
    }
    // spec: S-0209 claim 1 (a step sets the flag of the cell it enters at once)
    claim(w, id, e);
}

/// Puts a vehicle on a tank bunker's entry or release track toward
/// `target` (S-0606 claims 6, 12 and 13), without a path search: ρₜ and ρ
/// are set to 1.0 and no locomotor destination point is supplied.
pub fn begin_bunker_track(
    w: &mut World,
    id: ObjectId,
    kind: crate::structures::bunker::BunkerTrack,
    target: Pos,
) {
    let mode = w.rules().mode;
    let t = w.tick;
    let Some(o) = w.obj(id) else { return };
    let (x, y) = (o.pos.x, o.pos.y);
    let facing = o.body.value(t);
    let cell = target.cell();
    if let Some(ob) = w.obj_mut(id) {
        ob.path.steps.clear();
        if let Some(dd) = ob.drive.as_mut() {
            dd.track = Some(Track {
                // nocite: the step direction of the body facing, read by nothing on a bunker track
                d: glasnost_num::geom::n_step(facing, 8) as u8,
                e: None,
                // spec: S-0606 tables T1 and T2 (the track target: the reference point, or F4)
                target: (target.x, target.y),
                target_cell: cell,
                // spec: S-0606 F4 (height: the reference point's own)
                height: target.z,
                next: 0,
                last: (x, y),
                cells: vec![cell],
                crush: false,
                exit: false,
                bunker: Some(kind),
            });
            dd.turning_in_place = false;
            dd.turn = None;
            // spec: S-0606 claims 6, 12 and 13 (ρₜ and ρ set to 1.0)
            dd.rho_t = F64::from_i64(1, mode);
            dd.rho = F64::from_i64(1, mode);
        }
    }
    // spec: S-0209 claim 1 (a step sets the flag of the cell it enters at once)
    claim(w, id, cell);
}

/// Spends the budget on the current track (S-0207 claims 3 to 5). Returns
/// whether the track ended.
fn spend(w: &mut World, id: ObjectId) -> bool {
    let r = w.rules_arc();
    let mode = r.mode;
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    let Some(dd) = o.drive.clone() else {
        return false;
    };
    let Some(tr) = dd.track.clone() else {
        return false;
    };
    let pts = tr.points();
    let remaining = pts.len() - tr.next + 1; // the target included
    let mut a = dd.budget;
    // spec: S-0207 claim 3 (k = 0 when A ≤ 7, else the largest k with A − 7k ≥ 1, at most the points left)
    let k = if a <= 7 {
        0
    } else {
        (((a - 1) / 7) as usize).min(remaining)
    };
    a -= 7 * k as i32; // spec: S-0207 claim 3 (7 of the budget per point)
    let mut next = tr.next + k;
    let mut last = tr.last;
    if k > 0 && dd.between_steps && o.pos.cell() != tr.target_cell {
        // spec: S-0209 claim 1 (passing the first point of a track while between steps clears the flag of the cell it is leaving; a bunker track that ends in the cell it starts in leaves none, S-0606 tables T1 and T2)
        release(w, id, o.pos.cell());
        if let Some(d2) = w.obj_mut(id).and_then(|ob| ob.drive.as_mut()) {
            // spec: S-0209 Terms (between steps until it next advances along a track)
            d2.between_steps = false;
        }
    }
    if let Some(rp) = tr.e.and_then(|e| tracks::release_point(tr.d, e))
        && tr.next <= rp
        && next > rp
        && let Some(&c) = tr.cells.first()
    {
        // spec: S-0209 claim 1 (reaching a curve's release point clears the flag of the curve's first cell)
        release(w, id, c);
    }
    if k > 0 {
        if next > pts.len() {
            // spec: S-0207 claim 3 (the track target reached: A′ = A + trunc(7 × (1 − m/11)))
            a += tracks::end_correction(&pts);
            if let Some(d2) = w.obj_mut(id).and_then(|ob| ob.drive.as_mut()) {
                d2.budget = a;
                d2.track = None;
                d2.between_steps = true;
            }
            set_pos(w, id, tr.target.0, tr.target.1, true);
            if let Some(&(_, _, f)) = pts.last() {
                set_facing(w, id, f);
            }
            // spec: S-0207 claim 25 (a qualifying track completion clears the locomotor destination point before the cell-arrival report)
            completion_clears_goal(w, id);
            // spec: S-0516 claims 13 and 22 (arrival at the track target's cell centre is the completed-cell movement event: the sensor disc moves and a cloaked mover takes the adjacency check)
            crate::cloak::machine::cell_movement_completed(w, id);
            // spec: S-0402 claim 10 (second route: the arrival report at the track target, after the position and the cleared track, before cell-centre crushing and any next track)
            crate::econ::harvest::track_arrival(w, id);
            crate::crush::at_cell_centre(w, id);
            // spec: S-0207 claim 22 (reaching a track target raises the cell's entered-by tag, after the arrival report and cell-centre crushing; the cell it actually occupies)
            crate::objects::arrival_raise(w, id);
            return true;
        }
        let p: Point = pts[next - 1];
        let (x, y) = (tr.target.0 + p.0, tr.target.1 + p.1);
        // spec: S-0207 claim 5 (at a point: ground position target + displacement, height H, facing 256f)
        set_pos(w, id, x, y, true);
        set_facing(w, id, p.2);
        last = (x, y);
    }
    if next > pts.len() {
        next = pts.len();
    }
    // spec: S-0207 claim 4 (between points: moved towards P when A ≥ 1 and P is not the end)
    if a >= 1 && next < pts.len() {
        let p = pts[next];
        let pp = (tr.target.0 + p.0, tr.target.1 + p.1);
        let cur = w.obj(id).map_or((0, 0), |o| (o.pos.x, o.pos.y));
        let q0 = if r.quirks.drive_subpoint_compounding {
            cur
        } else {
            last
        };
        // spec: S-0207 F4 (t = single(A × (1/7)), 1/7 the double nearest; both roundings by the mode)
        let seventh = crate::consts::SEVENTH;
        let tt = F64::from_i64(i64::from(a), mode)
            .mul(seventh, mode)
            .to_f32(mode)
            .to_f64();
        let ox = F64::from_i64(i64::from(pp.0 - q0.0), mode)
            .mul(tt, mode)
            .to_i32_common();
        let oy = F64::from_i64(i64::from(pp.1 - q0.1), mode)
            .mul(tt, mode)
            .to_i32_common();
        let qn = (q0.0 + ox, q0.1 + oy);
        let qc = cell_of(q0.0, q0.1);
        let pc = cell_of(pp.0, pp.1);
        let nc = cell_of(qn.0, qn.1);
        // spec: S-0207 claim 4 (Q′ in the cell of Q or P; else P when A > 3; else Q)
        let dest = if nc == qc || nc == pc {
            qn
        } else if a > 3 {
            pp
        } else {
            q0
        };
        if dest != cur {
            set_pos(w, id, dest.0, dest.1, false);
        }
    }
    if let Some(d2) = w.obj_mut(id).and_then(|o| o.drive.as_mut()) {
        d2.budget = a;
        if let Some(t2) = d2.track.as_mut() {
            t2.next = next;
            t2.last = last;
        }
    }
    false
}

fn set_facing(w: &mut World, id: ObjectId, f: u8) {
    let t = w.tick;
    let keeps = w.rules().quirks.set_facing_keeps_target;
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0206 claim 12 (while driving, the body facing is set at once to the track's facing)
        ob.body.set(u16::from(f) * 256, t, keeps);
    }
}

/// Whether the vehicle's move goes on after a track ends: it has a stored
/// step and a destination.
fn move_goes_on(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .is_some_and(|o| o.dest.is_some() && !o.path.steps.is_empty())
}

/// One tick of a driving vehicle (S-0207 claim 12). Returns whether it
/// moved.
pub fn tick(w: &mut World, id: ObjectId) -> bool {
    let t = w.tick;
    let r = w.rules_arc();
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    let Some(u) = r.unit(o.kind, o.type_index).cloned() else {
        return false;
    };
    if o.drive.is_none() {
        return false;
    }
    // spec: S-0207 claim 25 (a destination assigned or removed without the assignment hook is taken before the movement)
    take(w, id, false);
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    let Some(dd) = o.drive.clone() else {
        return false;
    };
    let before = (o.pos.x, o.pos.y);
    let cap = r.quirks.drive_track_end_cap;
    if dd.track.is_some() {
        // spec: S-0207 claim 12 (a tick begun on a track: one update, s earned once)
        update_rho(w, id, &u);
        let s = progress(w, id, &u);
        record_step(w, id, s);
        add_budget(w, id, s);
        let mut ended = spend_and_stop(w, id);
        let mut finished = usize::from(ended);
        while ended && move_goes_on(w, id) {
            // spec: S-0207 claim 12 (the move goes on: a second update, the next track begins this tick)
            if !begin_track(w, id, &u) {
                break;
            }
            update_rho(w, id, &u);
            ended = spend_and_stop(w, id);
            finished += usize::from(ended);
            // spec: S-0207 claim 12 (at most two tracks when the tick began on one; quirks.drive_track_end_cap)
            if cap && finished >= 2 {
                break;
            }
        }
        if ended && !move_goes_on(w, id) {
            arrive_or_rest(w, id);
        }
    } else {
        if o.dest.is_none() {
            if dd.turning_in_place
                && let Some(d2) = w.obj_mut(id).and_then(|ob| ob.drive.as_mut())
            {
                // spec: S-0207 claim 11 (revision 39, Q-1160 question 2: a destination cancelled during the pre-step turn leaves no heading point and no independent pre-step movement hold; the cancelled step is not taken; the cancellation neither stops the body's turn nor retargets it, so the body completes its existing turn without displacement) and S-0206 Terms (a vehicle standing still with neither a move destination nor a next point is not being moved, even while its body turns)
                d2.turning_in_place = false;
                d2.turn = None;
            }
            return false;
        }
        // spec: S-0207 claim 26 (the first locomotion update without an active track removes a cell goal in the vehicle's cell, and a guard goal at the exact point, in that update's tick; also the assignment tick when assignment precedes the update)
        if goal_cleanup(w, id) {
            return false;
        }
        // spec: S-0207 claims 27 and 28 (stopped by the dock fallback: no locomotor destination point, so no track begins until a later effective assignment supplies one)
        if crate::refresh::stopped(w, id) {
            if let Some(ob) = w.obj_mut(id) {
                ob.path.steps.clear();
            }
            return false;
        }
        if o.path.steps.is_empty() {
            let goal = path::goal_cell(w, id);
            if goal == Some(o.pos.cell()) || goal.is_none() {
                // spec: S-0207 claim 26 (a building goal in its supplied point's cell is kept: cell equality alone removes neither destination)
                arrive_or_rest(w, id);
                return false;
            }
            // spec: S-0212 claims 20 and 21 (a fresh search once the retry delay is over; r97: for the driving locomotor a destination changed to a different one or cleared has ended it)
            // spec: S-0207 claim 3 (without a path the vehicle is not due to begin a track: its budget is kept)
            if path::waits(w, id) {
                return false;
            }
            // spec: S-0207 claim 15 (a fresh search that finds a path looks at the new path's first cell)
            if fresh_search(w, id, &u) != Search::Found {
                return false;
            }
        }
        if !begin_track(w, id, &u) {
            return false;
        }
        // spec: S-0207 claim 12 (a tick begun without a track: one update towards the new track, s once)
        update_rho(w, id, &u);
        let s = progress(w, id, &u);
        record_step(w, id, s);
        add_budget(w, id, s);
        let ended = spend_and_stop(w, id);
        let mut more = ended && move_goes_on(w, id);
        // spec: S-0207 claim 12 (a tick begun without a track finishes at most one; quirks.drive_track_end_cap off: no limit)
        while more && !cap {
            if !begin_track(w, id, &u) {
                break;
            }
            update_rho(w, id, &u);
            more = spend_and_stop(w, id) && move_goes_on(w, id);
        }
        if ended && !move_goes_on(w, id) {
            arrive_or_rest(w, id);
        }
    }
    let after = w.obj(id).map_or(before, |o| (o.pos.x, o.pos.y));
    let moved = after != before;
    if moved && let Some(d2) = w.obj_mut(id).and_then(|o| o.drive.as_mut()) {
        d2.moved_tick = Some(t);
    }
    moved
}

fn add_budget(w: &mut World, id: ObjectId, s: i32) {
    if let Some(d) = w.obj_mut(id).and_then(|o| o.drive.as_mut()) {
        // spec: S-0207 claim 3 (the vehicle adds s to its budget)
        d.budget = d.budget.wrapping_add(s);
    }
}

fn set_budget_zero(w: &mut World, id: ObjectId) {
    if let Some(d) = w.obj_mut(id).and_then(|o| o.drive.as_mut()) {
        // spec: S-0207 claim 3 (due to begin a track but waiting for a gate or refused by the step check: the budget is 0)
        d.budget = 0;
    }
}

/// A track target reached with no further step (S-0207 claim 11): the
/// vehicle is at rest. Completion alone leaves the order-level destination
/// in place (claim 26, final track); the next update that begins without a
/// track decides its removal ([`goal_cleanup`]).
fn arrive_or_rest(w: &mut World, id: ObjectId) {
    let keeps = w.rules().quirks.drive_leftover_kept_at_rest;
    if let Some(d) = w.obj_mut(id).and_then(|ob| ob.drive.as_mut()) {
        // spec: S-0207 claim 11 (a vehicle at rest has ρ = 0)
        d.rho = F64::ZERO;
        // spec: S-0207 claim 3 and Edge cases (a vehicle that comes to rest keeps its leftover budget; quirks.drive_leftover_kept_at_rest off: cleared)
        if !keeps {
            d.budget = 0;
        }
    }
}

/// The removal of the order-level goal by a locomotion update that begins
/// without an active track (S-0207 claim 26); whether the goal was removed.
/// A cell goal is removed while the vehicle occupies that cell, whatever the
/// behaviour and with no search, height tolerance or exact alignment. A
/// goal naming an object (a building, vehicle, infantry unit or aircraft,
/// a transport included, S-0504 claim 1, S-0507 claims 5 and 6) is not
/// removed by cell equality; only a vehicle at rest in
/// guard whose world point equals the locomotor destination point in all
/// three coordinates loses it too. With approach destinations the goal is
/// abandoned and the vehicle falls back to rest (S-0215 claim 13), as for a
/// give-up. Removing the destination clears the locomotor destination point
/// at once (claim 25, `goal`).
fn goal_cleanup(w: &mut World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let Some(dest) = o.dest else { return false };
    // spec: S-0207 claim 26 (Q-0830: a destination naming an object, a building, vehicle, infantry unit or aircraft, moving or stationary, is not a cell destination)
    let named = matches!(dest, Target::Object(t) if w.obj(t).is_some());
    let goal_c = path::goal_cell(w, id);
    let here = o.pos.cell();
    // spec: S-0207 claim 26 (an order-level cell destination, the vehicle occupying that cell: removed; a destination naming an object never, by cell equality alone)
    let cell_goal = !named && (goal_c.is_none() || goal_c == Some(here));
    // spec: S-0207 claim 26 (at rest in guard, the locomotor destination point present and equal to the vehicle's world point in x, y and z: the same removal, a destination naming a building, vehicle, infantry unit or aircraft included, compared with the retained locomotor point and not with the named unit's present position)
    let guard_goal = o.beh.current == Some(Behaviour::Guard) && goal(w, id) == Some(o.pos);
    if !cell_goal && !guard_goal {
        return false;
    }
    let approach = !o.queued_dests.is_empty();
    arrive_or_rest(w, id);
    if let Some(ob) = w.obj_mut(id) {
        ob.dest = None;
        ob.path.steps.clear();
    }
    if approach {
        // spec: S-0207 claim 26 (with approach destinations the current goal is abandoned and the vehicle falls back to rest, S-0215 claim 13, with any replacement goal that rule supplies)
        process::fall_back(w, id, false);
    }
    true
}

/// Whether a vehicle on the driving or ship locomotor is *moving now* for
/// its wake (S-0207 claim 23, S-0504 claim 2; Q-0600): its body is turning,
/// or it has a heading point (the active track's target) and its integer
/// progress rate s, worked out with its current speed fraction at the check,
/// is above 0.
pub(crate) fn moving_now(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if o.body.turning(w.tick) {
        // spec: S-0207 claim 23 and S-0504 claim 2 (moving now: its body is turning)
        return true;
    }
    let Some(u) = w.rules().unit(o.kind, o.type_index) else {
        return false;
    };
    // spec: S-0207 claim 23 and S-0504 claim 2 (a heading point, the active track's target, and s = trunc(s₂ × ρ) above 0 with the current ρ; not ρ itself nor a remembered rate)
    o.drive.as_ref().is_some_and(|d| d.track.is_some()) && progress(w, id, u) > 0
}

/// The shared wake test of S-0207 claim 23 and S-0504 claim 2: in a tick
/// whose number is divisible by `period`, a vehicle that is moving now, off
/// a bridge deck, on a cell of land type water, creates one game-wide wake
/// animation at its own position, if that animation is defined, with no
/// draw of its own beyond the animation's creation draws. The check reads
/// the state after the tick's locomotor movement.
pub(crate) fn wake_check(w: &mut World, id: ObjectId, period: i32) {
    if period <= 0 || w.tick % period != 0 {
        return;
    }
    let Some(wake) = w.rules().wake.clone() else {
        // spec: S-0207 claim 23 and S-0504 claim 2 (only when the game-wide wake animation is defined)
        return;
    };
    if !moving_now(w, id) {
        return;
    }
    let Some(o) = w.obj(id) else { return };
    // spec: S-0207 claim 23 and S-0504 claim 2 (off a bridge deck, on a cell whose land type is water)
    let water = w
        .cells
        .cell(o.pos.cell())
        .is_some_and(|c| c.land == land::WATER);
    if o.layer == Layer::Deck || !water {
        return;
    }
    let at = o.pos;
    // spec: S-0207 claim 23 and S-0504 claim 2 (one instance at its own post-movement position; only the animation's creation draws, S-0109 claim 3)
    crate::combat::effects::anim(w, &wake, at);
}

/// The driving wake of S-0207 claim 23: ticks divisible by 10, with no
/// underwater-type exclusion. Created in the vehicle's turn right after its
/// drive update, at its position after the move (Q-0600, answered).
pub fn driving_wake(w: &mut World, id: ObjectId) {
    // spec: S-0207 claim 23 (ticks whose number is divisible by 10)
    wake_check(w, id, 10);
}

/// A stop while on a track (S-0207 claim 11): the target fraction becomes
/// min(ρₜ, F₀.₃) and the track is finished.
pub fn stop(w: &mut World, id: ObjectId) {
    let floor = f03();
    if let Some(ob) = w.obj_mut(id) {
        ob.path.steps.clear();
        if let Some(d) = ob.drive.as_mut() {
            // spec: S-0207 claim 11 (stopping: ρₜ = min(ρₜ, F₀.₃); it finishes the track it is on)
            d.rho_t = d.rho_t.min(floor);
            d.turning_in_place = false;
            d.turn = None;
        }
    }
}

/// The rules of a unit (helper for tests).
pub fn unit_rules(r: &SimRules, w: &World, id: ObjectId) -> Option<UnitRules> {
    let o = w.obj(id)?;
    r.unit(o.kind, o.type_index).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::Kind;
    use crate::testkit;

    /// The world after `n` scatter requests from a vehicle to a friendly
    /// walker's cell in one tick, with the duplicate-request switch `on`.
    fn after_requests(on: bool, n: usize) -> World {
        let mut r = testkit::ruleset();
        r.globals.quirks.drive_duplicate_scatter_request = on;
        let mut w = testkit::game_with(&r, 40, 3);
        let v = testkit::spawn(&mut w, Kind::Vehicle, "CART", 0, testkit::mid(40, 0, 0), 0);
        let c = testkit::mid(40, 0, -1);
        let walker = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, c, 0);
        if let Some(o) = w.obj_mut(v) {
            o.drive.get_or_insert_with(DriveState::new);
        }
        for _ in 0..n {
            request_scatter(&mut w, v, c);
            // The walker that accepted holds a destination, so it has
            // movement in progress and would refuse a second request
            // (S-0417 claim 17, No path; S-0208 claim 11): clear it, so that
            // a request that reaches it is accepted and draws.
            if let Some(o) = w.obj_mut(walker) {
                o.dest = None;
            }
        }
        w
    }

    // spec: S-0207 claim 15 and Edge cases (quirks.drive_duplicate_scatter_request: on, a second request in the tick reaches the receiver again and takes its draw; off, it is omitted with its draws)
    #[test]
    fn duplicate_request_in_one_tick() {
        let once = after_requests(true, 1).rng;
        let twice = after_requests(true, 2).rng;
        assert_ne!(once, twice, "on: the second request draws again");
        let once = after_requests(false, 1).rng;
        let twice = after_requests(false, 2).rng;
        assert_eq!(once, twice, "off: the second request is not sent");
    }
}
