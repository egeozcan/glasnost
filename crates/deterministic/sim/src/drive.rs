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
use crate::rules::{Behaviour, Kind, SimRules, UnitRules};
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
        // spec: S-0209 claim 1 (a vehicle taken out of a cell's list while between steps clears that cell's flag; one without the driving locomotor never advanced)
        Some(o) if o.drive.is_none() && o.kind == Kind::Vehicle => (vec![o.pos.cell()], o.layer),
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
        crate::load::recalc_cells(w, &[old, new]);
        crate::crush::entered_cell(w, id, new);
    }
}

/// The cell one step from `c` in step direction `d`.
fn neighbour(c: CellXY, d: u8) -> CellXY {
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
    let first = neighbour(o.pos.cell(), d);
    if let Some(gate) = process::closed_friendly_gate(w, id, first) {
        // spec: S-0207 claim 15 (an allied closed gate: told to open; the vehicle waits without a track and without turning)
        process::gate_met(w, gate);
        wait(w, id);
        return false;
    }
    let want = facing_of(d);
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
            wait(w, id);
            return false;
        }
        if let Some(dd) = w.obj_mut(id).and_then(|o| o.drive.as_mut()) {
            dd.turning_in_place = false;
            dd.turn = None;
        }
    }
    step_check(w, id, u, false)
}

/// The step check (S-0207 claim 15, tables T4 and T5): a first check, or
/// the repeated check that follows a search made at once. Begins the track
/// when the check lets it; returns whether it did.
fn step_check(w: &mut World, id: ObjectId, u: &UnitRules, repeated: bool) -> bool {
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
    match checked_outcome(w, id, u, here, first) {
        Outcome::Clear => {}
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
            // spec: S-0207 claim 15 table T4 (cloaked blocker; nothing is cloaked in M3, so there is nothing to reveal)
            if repeated {
                give_up(w, id);
                return false;
            }
            discard_path(w, id);
            wait(w, id);
            // spec: S-0207 claim 15 table T4 (the fresh search on the same tick only when the path retry timer has run out)
            if t >= o.path.retry_at {
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
            // spec: S-0207 claim 15 table T4 and S-0212 claim 22 (blocked clears the stored path; a fresh search follows after the delay)
            discard_path(w, id);
            wait(w, id);
            return false;
        }
    }
    // spec: S-0207 claim 7 (the next step chooses the track; crush mode drives straight)
    let e = o.path.steps.get(1).copied();
    let second = neighbour(first, e.unwrap_or(d));
    let crush = crush_mode(w, id, u, first, second);
    let mut use_e = if crush { None } else { e };
    let (_, two) = tracks::track(d, use_e);
    if two {
        // spec: S-0207 claim 15 table T5 (the second cell of a curve, checked after the first gave clear)
        match checked_outcome(w, id, u, first, second) {
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
                if t >= o.path.retry_at {
                    // Q-0035: no step check follows this search on the same tick (interim).
                    fresh_search(w, id, u);
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
                // spec: S-0207 table T5 (cloaked blocker: as in T4)
                if repeated {
                    give_up(w, id);
                    return false;
                }
                discard_path(w, id);
                wait(w, id);
                if t >= o.path.retry_at {
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
                next: 0,
                last: (ob.pos.x, ob.pos.y),
                cells: cells.clone(),
                crush,
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
fn discard_path(w: &mut World, id: ObjectId) {
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
    step_check(w, id, u, true)
}

/// A fresh search (S-0212 claim 20) and, when it finds a path, the look at
/// the first cell of the new path (S-0207 claim 15).
fn fresh_search(w: &mut World, id: ObjectId, u: &UnitRules) -> Search {
    let res = path::compute(w, id, Mode::Fresh);
    if res != Search::Found {
        path::failed(w, id, res, false);
        return res;
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
    let Some(near) = nearest_object(w, id, cell) else {
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

/// The object nearest the centre of `c`, looked for in the deck layer when
/// the vehicle's world height is more than 208 leptons above the surface
/// height at that centre, otherwise in the ground layer (S-0207 claim 15).
/// Which objects take part, how the distance is measured and how ties are
/// broken are not stated (request Q-0040). Interim: every listed object but
/// the vehicle itself, by S-0204's distance from its position to the
/// centre at the surface height, ties to the object listed first.
fn nearest_object(w: &World, id: ObjectId, c: CellXY) -> Option<ObjectId> {
    let o = w.obj(id)?;
    let r = w.rules();
    let cx = 256 * c.x + 128; // spec: S-0204 claim 3 (a cell centre: (256x + 128, 256y + 128))
    let cy = 256 * c.y + 128; // spec: S-0204 claim 3 (a cell centre)
    let h = w.surface_at(cx, cy);
    // spec: S-0207 claim 15 (the deck layer when z > H + 208 in exact integer leptons)
    let layer = if o.pos.z > h + 208 {
        Layer::Deck
    } else {
        Layer::Ground
    };
    let mut best: Option<(u64, ObjectId)> = None;
    for &x in w.cells.list(c, layer) {
        if x == id {
            continue;
        }
        let Some(ob) = w.obj(x) else { continue };
        let dx = i64::from(ob.pos.x - cx);
        let dy = i64::from(ob.pos.y - cy);
        let dz = i64::from(ob.pos.z - h);
        // spec: S-0204 claim 5 (the distance with the table square root; measure and ties: Q-0040)
        let dist = root_floor(
            (dx * dx + dy * dy + dz * dz) as u64,
            r.quirks.table_root_distance,
            r.mode,
        );
        if best.is_none_or(|(b, _)| dist < b) {
            best = Some((dist, x));
        }
    }
    best.map(|b| b.1)
}

/// The give-up test of S-0207 F7; `contact` adds the condition that the
/// vehicle is in contact with nothing.
fn give_up_test(w: &World, id: ObjectId, contact: bool) -> bool {
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
fn give_up(w: &mut World, id: ObjectId) {
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
fn request_scatter(w: &mut World, id: ObjectId, c: CellXY) {
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
fn attack_obstacle(w: &mut World, id: ObjectId, c: CellXY) {
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
            // Q-0035: the override's destination is not stated; interim none.
            ob.dest = None;
            ob.path.steps.clear();
        }
    }
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
/// 22, S-0207 table T4).
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
            if let Some(d2) = w.obj_mut(id).and_then(|ob| ob.drive.as_mut()) {
                d2.budget = a;
                d2.track = None;
                d2.between_steps = true;
            }
            set_pos(w, id, tr.target.0, tr.target.1, true);
            if let Some(&(_, _, f)) = pts.last() {
                set_facing(w, id, f);
            }
            crate::crush::at_cell_centre(w, id);
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
            // spec: S-0207 claim 3 (without a path the vehicle is not due to begin a track: its budget is kept)
            if t < o.path.retry_at {
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
        // spec: S-0207 claim 3 (due to begin a track but waiting for a gate or refused by the step check: the budget is 0)
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
