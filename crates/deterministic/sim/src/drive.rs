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
use crate::regions::STEPS;
use crate::rules::{SimRules, UnitRules};
use crate::tracks::{self, Point};
use crate::world::{Layer, ObjectId, Pos, World};

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
    /// Index of the next point to pass.
    pub next: usize,
    /// The last point passed (for `quirks.drive_subpoint_compounding` off).
    pub last: (i32, i32),
    /// The cells of the step or steps (first, and second for a curve).
    pub cells: Vec<CellXY>,
    /// Crush mode (S-0207 claim 7).
    pub crush: bool,
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
        }
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
    let (claims, layer) = match w.obj(id) {
        Some(o) => (
            o.drive
                .as_ref()
                .map(|d| d.claims.clone())
                .unwrap_or_default(),
            o.layer,
        ),
        None => return,
    };
    for c in claims {
        w.cells.claim_vehicle(c, layer, false);
    }
    if let Some(d) = w.obj_mut(id).and_then(|o| o.drive.as_mut()) {
        d.claims.clear();
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
    let (rho, rho_t) = if !u.accelerates {
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

/// Progress per tick s (S-0207 claim 2, F2).
fn progress(w: &World, id: ObjectId, u: &UnitRules) -> i32 {
    let r = w.rules();
    let mode = r.mode;
    let Some(o) = w.obj(id) else { return 0 };
    let rho = o.drive.as_ref().map_or(F64::ZERO, |d| d.rho);
    let kappa = o
        .owner
        .and_then(|h| r.countries.get(w.houses[usize::from(h.0)].country as usize))
        .map_or(F64::from_i64(1, mode), |c| c.speed_units);
    // spec: S-0207 F2 (s₁ = trunc((V × κ) × b); s₂ = s₁ without the faster ability; s = trunc(s₂ × ρ))
    let s1 = F64::from_i64(i64::from(u.speed), mode)
        .mul(kappa, mode)
        .mul(o.speed_bonus, mode)
        .to_i32_common();
    F64::from_i64(i64::from(s1), mode)
        .mul(rho, mode)
        .to_i32_common()
}

fn cell_of(x: i32, y: i32) -> CellXY {
    Pos::new(x, y, 0).cell()
}

/// Moves the vehicle to a new ground position, handling the cell change
/// (S-0209 claim 13, S-0211 claim 5, S-0210 claim 9).
fn set_pos(w: &mut World, id: ObjectId, x: i32, y: i32, z: Option<i32>) {
    let Some(o) = w.obj(id).cloned() else { return };
    let old = o.pos.cell();
    let z = z.unwrap_or(o.pos.z);
    let np = Pos::new(x, y, z);
    if let Some(ob) = w.obj_mut(id) {
        ob.pos = np;
    }
    let new = np.cell();
    if new != old {
        w.cells.leave(old, o.layer, id);
        w.cells.enter(new, o.layer, id, false);
        crate::load::recalc_cells(w, &[old, new]);
        crate::crush::entered_cell(w, id, new);
    }
}

/// Begins a track from the vehicle's current cell (S-0207 claims 6 and 7).
/// Returns `false` when the step cannot begin this tick.
fn begin_track(w: &mut World, id: ObjectId, u: &UnitRules) -> bool {
    let t = w.tick;
    let r = w.rules_arc();
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    let Some(&d) = o.path.steps.first() else {
        return false;
    };
    let here = o.pos.cell();
    let (dx, dy) = STEPS[usize::from(d % 8)]; // spec: S-0212 Terms (step directions 0 to 7)
    let first = CellXY::new(here.x + dx, here.y + dy);
    // spec: S-0207 claim 6 (a track begins only when the body facing equals 8192 × d exactly)
    let want = u16::from(d % 8) * 8192;
    if o.body.value(t) != want {
        // A turn in place towards another facing (the path changed) is replaced.
        if !o.drive.as_ref().is_some_and(|x| x.turning_in_place) || o.body.target != want {
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
        return false;
    }
    if let Some((t0, dur)) = o.drive.as_ref().and_then(|x| x.turn)
        && o.drive.as_ref().is_some_and(|x| x.turning_in_place)
    {
        // spec: S-0206 claim 12 (a turn completing at once begins the track in tick t₀ + 1; otherwise in t₀ + Θ)
        let begin = if dur == 0 { t0 + 1 } else { t0 + dur };
        if t < begin {
            if let Some(dd) = w.obj_mut(id).and_then(|o| o.drive.as_mut()) {
                dd.budget = 0;
            }
            return false;
        }
        if let Some(dd) = w.obj_mut(id).and_then(|o| o.drive.as_mut()) {
            dd.turning_in_place = false;
            dd.turn = None;
        }
    }
    // The entry check at the start of a step (S-0209 claim 5f).
    let reference = occupancy::reference_level(w, id);
    let mut outcome = path::step_outcome(w, id, here, first, reference);
    if u.train && outcome != Outcome::Blocked {
        // spec: S-0209 claim 5f (a train: every outcome other than blocked counts as clear)
        outcome = Outcome::Clear;
    }
    if u.crusher
        && r.quirks.crusher_overlay_index0_clear
        && matches!(outcome, Outcome::FriendlyObstacle | Outcome::EnemyObstacle)
        && w.cells.cell(first).is_some_and(|c| c.overlay == 0)
    {
        // spec: S-0209 claim 5f (a crusher-flag vehicle: obstacles count as clear on overlay index 0)
        outcome = Outcome::Clear;
    }
    match outcome {
        Outcome::Clear => {}
        Outcome::Blocked => {
            // spec: S-0212 claim 22 (outcome blocked clears the stored path; a fresh search follows after the delay)
            if let Some(ob) = w.obj_mut(id) {
                ob.path.steps.clear();
                if let Some(dd) = ob.drive.as_mut() {
                    dd.budget = 0;
                }
            }
            return false;
        }
        _ => {
            // spec: S-0212 claim 22 (moving blocker: blockage timer, re-path modes 1 and 2); Q-0019 for the others
            blocked_step(w, id);
            return false;
        }
    }
    // spec: S-0207 claim 7 (the next step chooses the track; crush mode drives straight)
    let e = o.path.steps.get(1).copied();
    let second = {
        let g = e.unwrap_or(d);
        let (sx, sy) = STEPS[usize::from(g % 8)]; // spec: S-0212 Terms (step directions 0 to 7)
        CellXY::new(first.x + sx, first.y + sy)
    };
    let crush = crush_mode(w, id, u, first, second);
    let mut use_e = if crush { None } else { e };
    let (_, two) = tracks::track(d, use_e);
    if two {
        // spec: S-0207 claim 7 (the curve's second cell occupied or being entered: the straight track instead)
        let rec = w.cells.occ(second, o.layer);
        let busy = rec.is_some_and(|x| x.vehicle_present())
            || w.cells
                .list(second, o.layer)
                .iter()
                .any(|&x| x != id && w.obj(x).is_some_and(|ob| ob.kind.is_mobile()));
        if busy {
            use_e = None;
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
                next: 0,
                last: (ob.pos.x, ob.pos.y),
                cells,
                crush,
            });
            dd.rho_t = rho_t;
        }
        // spec: S-0212 claim 22 (a blockage ends once the unit has taken a step again)
        ob.path.blocked_since = None;
    }
    // spec: S-0209 claim 1 (a step sets the flag of the cell it enters at once; a two-cell curve both cells)
    for c in if two {
        vec![first, second]
    } else {
        vec![first]
    } {
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

/// A driving unit's next step is blocked by a moving blocker (S-0212 claim
/// 22); the other outcomes take the same course until Q-0019 is answered.
fn blocked_step(w: &mut World, id: ObjectId) {
    let t = w.tick;
    let Some(o) = w.obj(id).cloned() else { return };
    let since = o.path.blocked_since.unwrap_or(t);
    if let Some(ob) = w.obj_mut(id) {
        ob.path.blocked_since = Some(since);
        if let Some(dd) = ob.drive.as_mut() {
            dd.budget = 0;
        }
    }
    if t < o.path.retry_at {
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
            // spec: S-0212 claim 21 (after a re-path for a blocked step the delay R starts again)
            if let Some(ob) = w.obj_mut(id) {
                ob.path.retry_at = t.wrapping_add(r);
            }
            if other == Search::SearchFailed {
                path::failed(w, id, other, true);
            }
        }
    }
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
    let (pts, _) = tracks::track(tr.d, tr.e);
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
    if k > 0 && dd.between_steps {
        // spec: S-0209 claim 1 (passing the first point of a track while between steps clears the flag of the cell it is leaving)
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
            let z = w.surface_at(tr.target.0, tr.target.1);
            if let Some(d2) = w.obj_mut(id).and_then(|ob| ob.drive.as_mut()) {
                d2.budget = a;
                d2.track = None;
                d2.between_steps = true;
            }
            set_pos(w, id, tr.target.0, tr.target.1, Some(z));
            if let Some(&(_, _, f)) = pts.last() {
                set_facing(w, id, f);
            }
            crate::crush::at_cell_centre(w, id);
            return true;
        }
        let p: Point = pts[next - 1];
        let (x, y) = (tr.target.0 + p.0, tr.target.1 + p.1);
        // spec: S-0207 claim 5 (at a point: ground position target + displacement, height H, facing 256f)
        let z = w.surface_at(x, y);
        set_pos(w, id, x, y, Some(z));
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
            set_pos(w, id, dest.0, dest.1, None);
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
    let Some(dd) = o.drive.clone() else {
        return false;
    };
    let before = (o.pos.x, o.pos.y);
    let cap = r.quirks.drive_track_end_cap;
    if dd.track.is_some() {
        // spec: S-0207 claim 12 (a tick begun on a track: one update, s earned once)
        update_rho(w, id, &u);
        let s = progress(w, id, &u);
        add_budget(w, id, s);
        let mut ended = spend(w, id);
        let mut finished = usize::from(ended);
        while ended && move_goes_on(w, id) {
            // spec: S-0207 claim 12 (the move goes on: a second update, the next track begins this tick)
            if !begin_track(w, id, &u) {
                break;
            }
            update_rho(w, id, &u);
            ended = spend(w, id);
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
            return false;
        }
        if o.path.steps.is_empty() {
            let goal = path::goal_cell(w, id);
            if goal == Some(o.pos.cell()) || goal.is_none() {
                arrive_or_rest(w, id);
                return false;
            }
            // spec: S-0212 claims 20 and 21 (a fresh search once the retry delay is over)
            if t < o.path.retry_at {
                set_budget_zero(w, id);
                return false;
            }
            let res = path::compute(w, id, Mode::Fresh);
            if res != Search::Found {
                path::failed(w, id, res, false);
                set_budget_zero(w, id);
                return false;
            }
        }
        if !begin_track(w, id, &u) {
            return false;
        }
        // spec: S-0207 claim 12 (a tick begun without a track: one update towards the new track, s once)
        update_rho(w, id, &u);
        let s = progress(w, id, &u);
        add_budget(w, id, s);
        let ended = spend(w, id);
        let mut more = ended && move_goes_on(w, id);
        // spec: S-0207 claim 12 (a tick begun without a track finishes at most one; quirks.drive_track_end_cap off: no limit)
        while more && !cap {
            if !begin_track(w, id, &u) {
                break;
            }
            update_rho(w, id, &u);
            more = spend(w, id) && move_goes_on(w, id);
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
        // spec: S-0207 claim 3 (a move to make but no track driven or begun: the budget is 0)
        d.budget = 0;
    }
}

/// A track target reached with no further step (S-0207 claim 11).
fn arrive_or_rest(w: &mut World, id: ObjectId) {
    let at_goal = {
        let goal = path::goal_cell(w, id);
        let here = w.obj(id).map(|o| o.pos.cell());
        goal.is_none() || goal == here
    };
    if let Some(ob) = w.obj_mut(id) {
        if let Some(d) = ob.drive.as_mut() {
            // spec: S-0207 claim 11 (a vehicle at rest has ρ = 0; it keeps its leftover budget)
            d.rho = F64::ZERO;
        }
        if at_goal {
            ob.dest = None;
            ob.path.steps.clear();
        }
    }
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
