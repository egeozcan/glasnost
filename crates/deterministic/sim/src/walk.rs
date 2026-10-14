//! Infantry movement, the walk locomotor (S-0208 claims 1 to 10, 14, 16
//! and 17), with a walker's reactions on its asking tick (S-0209 claims 6
//! to 10).

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_num::geom::{delta, direction, root_floor};
use glasnost_num::{F64, Rounding, trig};
use serde::{Deserialize, Serialize};

use crate::cells::position;
use crate::occupancy::{self, Outcome};
use crate::path::{self, Mode, Search};
use crate::regions::STEPS;
use crate::rules::{Behaviour, Kind};
use crate::scatter;
use crate::world::{Layer, ObjectId, Pos, Target, World};

/// A walking unit's locomotor state.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalkState {
    /// The walking target (a standing position, height included).
    pub target: Option<Pos>,
    /// The standing position number of the walking target or of the
    /// position the unit stands on.
    pub standing: u8,
    /// Tick of the last arrival on a standing position.
    pub arrived: Option<i32>,
    /// The unit moved on the current tick.
    pub moved_tick: Option<i32>,
    /// The infantry speed fraction of S-0310 (Terms), 0 or 1 for a walker:
    /// 1 from a walking step until the walker stops (S-0208 claims 16 and
    /// 17).
    #[serde(default)]
    pub fraction: bool,
}

/// The point of a standing position within a cell (S-0208 table P).
pub fn standing_point(p: u8) -> (i32, i32) {
    // spec: S-0208 table P (centre 128,128; top 64,64; right 192,64; left 64,192; bottom 192,192)
    match p {
        position::TOP => (64, 64),
        position::RIGHT => (192, 64),
        position::LEFT => (64, 192),
        position::BOTTOM => (192, 192),
        _ => (128, 128),
    }
}

/// The position class of a point within a cell (S-0208 claim 2). With
/// `quirks.top_position_counts_as_centre` off, a unit standing exactly at
/// the top position is recorded as the top (S-0208 Edge cases).
pub fn position_class(u: i32, v: i32, top_quirk: bool) -> u8 {
    if !top_quirk && (u, v) == standing_point(position::TOP) {
        return position::TOP;
    }
    let (du, dv) = (u - 128, v - 128); // spec: S-0208 claim 2 (relative to the cell centre, 128 per axis)
    // spec: S-0208 claim 2 (centre when (u − 128)² + (v − 128)² < 3600)
    if du * du + dv * dv < 3600 {
        return position::CENTRE;
    }
    // spec: S-0208 claim 2 (quadrants: right, left, bottom; the upper-left quadrant is the centre)
    match (u > 128, v > 128) {
        (true, false) => position::RIGHT,
        (false, true) => position::LEFT,
        (true, true) => position::BOTTOM,
        (false, false) => position::CENTRE,
    }
}

/// Table O of S-0208 (claim 3): the rows S2, S3, S4 and R0 … R3.
pub(crate) fn row_for(class: u8, draw: i32) -> &'static [u8] {
    use position::{BOTTOM, LEFT, RIGHT};
    // spec: S-0208 table O (fallback orders)
    match class {
        RIGHT => &[BOTTOM, LEFT],
        LEFT => &[BOTTOM, RIGHT],
        BOTTOM => &[RIGHT, LEFT],
        _ => match draw {
            0 | 1 => &[RIGHT, LEFT, BOTTOM],
            2 => &[LEFT, BOTTOM, RIGHT],
            _ => &[BOTTOM, RIGHT, LEFT],
        },
    }
}

/// The walking speed s of S-0208 F5 and claim 8, in leptons per tick.
pub fn walking_speed(w: &World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id) else { return 0 };
    let r = w.rules();
    let mode = r.mode;
    let Some(u) = r.unit(o.kind, o.type_index) else {
        return 0;
    };
    let kc = o
        .owner
        .and_then(|h| r.countries.get(w.houses[usize::from(h.0)].country as usize))
        .map_or(F64::from_i64(1, mode), |c| c.speed_infantry);
    // spec: S-0208 F5 (s₁ = trunc((b × k꜀) × m); s = trunc(s₁ × kᵥ) with the faster ability, else s₁)
    let s1 = F64::from_i64(i64::from(u.speed), mode)
        .mul(kc, mode)
        .mul(o.speed_bonus, mode)
        .to_i32_common();
    let s = if crate::combat::veterancy::has_ability(w, id, crate::combat::data::Ability::Faster) {
        F64::from_i64(i64::from(s1), mode)
            .mul(r.general.veteran_speed, mode)
            .to_i32_common()
    } else {
        s1
    };
    let prone = o.infantry.as_ref().is_some_and(|i| i.prone);
    if prone {
        if u.crawls {
            // spec: S-0208 claim 8 (crawling: s − ⌊s/3⌋)
            s - s / 3
        } else if r.quirks.prone_noncrawler_speedup {
            // spec: S-0208 claim 8 (a prone non-crawler: s + ⌊s/2⌋, quirks.prone_noncrawler_speedup)
            s + s / 2
        } else {
            s
        }
    } else {
        s
    }
}

/// The step of S-0208 F3 from (x, y) towards facing `f` at speed `s`.
pub fn step_point(x: i32, y: i32, f: u16, s: i32, table: bool, mode: Rounding) -> (i32, i32) {
    // spec: S-0208 F3 (f̂ = f or f − 65536)
    let fh = if f <= 32767 {
        i64::from(f)
    } else {
        i64::from(f) - 65536
    };
    let theta = if table {
        // spec: S-0208 F3 (θ = (f̂ − 16383) × c₂, c₂ = −2π / 65534 as a double)
        let c2 = crate::consts::WALK_C2;
        F64::from_i64(fh - 16383, mode).mul(c2, mode) // spec: S-0208 F3 (f̂ − 16383)
    } else {
        // spec: S-0208 F3 (`quirks.table_direction` off: θ = (f̂ − 16384) × (−2π / 65536))
        let c = crate::consts::WALK_C;
        F64::from_i64(fh - 16384, mode).mul(c, mode) // spec: S-0208 F3 (f̂ − 16384 with the switch off)
    };
    let sf = F64::from_i64(i64::from(s), mode);
    // spec: S-0208 F3 (x′ = trunc(x + s × cosT θ), y′ = trunc(y − s × sinT θ))
    let nx = F64::from_i64(i64::from(x), mode)
        .add(sf.mul(trig::cos_t(theta, mode), mode), mode)
        .to_i32_common();
    let ny = F64::from_i64(i64::from(y), mode)
        .sub(sf.mul(trig::sin_t(theta, mode), mode), mode)
        .to_i32_common();
    (nx, ny)
}

fn dist2(a: (i32, i32), b: (i32, i32)) -> u64 {
    let dx = i64::from(a.0 - b.0);
    let dy = i64::from(a.1 - b.1);
    (dx * dx + dy * dy) as u64
}

/// A walker has arrived when its distance to the walking target is below
/// this many leptons.
// spec: S-0208 F2 (arrival when D < 17)
const ARRIVAL: u64 = 17;

/// One tick of a walker (S-0208 claim 6). Returns whether it moved.
pub fn tick(w: &mut World, id: ObjectId) -> bool {
    let t = w.tick;
    let r = w.rules_arc();
    let q = r.quirks;
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    let Some(ws) = o.walk.clone() else {
        return false;
    };
    if let Some(target) = ws.target {
        let here = (o.pos.x, o.pos.y);
        let tp = (target.x, target.y);
        // spec: S-0208 F2 (D = trunc(√̃(Δx² + Δy²)); arrival when D < 17)
        let d = root_floor(dist2(here, tp), q.table_root_distance, r.mode);
        if d < ARRIVAL {
            arrive(w, id, target);
            // spec: S-0208 State and ordering (on the arrival tick, after the placement: claim 17's in-range stop, then claim 16's end-of-walk test)
            approach_stop(w, id);
            // spec: S-0208 claim 22 (the snap to the walking target raises the current cell's entered-by tag, after claim 17's destination clear; every intermediate standing-position arrival raises it)
            crate::objects::arrival_raise(w, id);
            end_of_walk(w, id);
            // spec: S-0208 claim 23 and S-0417 claim 10 (the arrival tick at each walking target is the cell-arrival update; a tethered infantry unit reports clear on it)
            crate::econ::release::arrival(w, id);
            return true;
        }
        let s = walking_speed(w, id);
        // spec: S-0208 F7 (the facing towards the walking target is S-0204's direction)
        let f = direction(
            i64::from(tp.0 - here.0),
            i64::from(tp.1 - here.1),
            q.table_direction,
        );
        let (mut nx, mut ny) = step_point(here.0, here.1, f, s, q.table_direction, r.mode);
        if !q.walker_overshoot {
            // spec: S-0208 Edge cases (switch off: a step that would end 17 or more beyond ends on the target)
            let before = i64::from(tp.0 - here.0) * i64::from(nx - here.0)
                + i64::from(tp.1 - here.1) * i64::from(ny - here.1);
            let travelled = dist2(here, (nx, ny));
            if before > 0
                && travelled > dist2(here, tp)
                && root_floor(dist2((nx, ny), tp), true, r.mode) >= ARRIVAL
            {
                nx = tp.0;
                ny = tp.1;
            }
        }
        if !q.slow_walker_stall && (nx, ny) == here {
            // spec: S-0208 Edge cases (switch off: a stalled step moves 1 lepton per axis towards the target)
            nx += (tp.0 - here.0).signum();
            ny += (tp.1 - here.1).signum();
        }
        let old_cell = o.pos.cell();
        let new_pos = Pos::new(nx, ny, o.pos.z);
        // spec: S-0206 claim 7 (the infantry body turns at the rate of its rules: 127 with quirks.infantry_fixed_turn_rate)
        let omega = r.unit(o.kind, o.type_index).map_or(0, |u| u.turn_rate);
        if let Some(ob) = w.obj_mut(id) {
            ob.body.turn_to(f, t, omega);
            ob.pos = new_pos;
            if let Some(s) = ob.walk.as_mut() {
                s.moved_tick = Some(t);
                // spec: S-0310 Terms (the infantry speed fraction is 1 from a walking step)
                s.fraction = true;
            }
        }
        if new_pos.cell() != old_cell {
            changed_cell(w, id, old_cell, new_pos.cell());
        }
        // spec: S-0209 claim 10 (the blockage delay starts afresh only after a walking step)
        if let Some(ob) = w.obj_mut(id) {
            ob.path.blocked_since = None;
        }
        return true;
    }
    // Standing on a standing position.
    if ws.arrived == Some(t) {
        // spec: S-0208 claim 6 (it does not choose its next walking target on its arrival tick)
        return false;
    }
    if o.dest.is_none() && o.path.steps.is_empty() {
        stopped(w, id);
        return false;
    }
    if o.path.steps.is_empty() {
        let goal = path::goal_cell(w, id);
        if goal == Some(o.pos.cell()) || goal.is_none() {
            // A standing walker given a destination in its own cell (no
            // arrival tick ends that walk under claim 16): the move is
            // finished.
            if let Some(ob) = w.obj_mut(id) {
                ob.dest = None;
            }
            stopped(w, id);
            return false;
        }
        // spec: S-0212 claims 20 and 21 (a fresh search once the retry delay is over)
        if t < o.path.retry_at {
            return false;
        }
        match path::compute(w, id, Mode::Fresh) {
            Search::Found => {}
            other => {
                path::failed(w, id, other, false);
                stopped(w, id);
                return false;
            }
        }
    }
    ask(w, id, true);
    if w.obj(id)
        .and_then(|o| o.walk.as_ref())
        .is_some_and(|s| s.target.is_none())
    {
        stopped(w, id);
    }
    false
}

/// The walker stands without a walking target on a tick it does not step
/// (no path, blocked, waiting, or no destination left): its speed fraction
/// is 0 (S-0310 Terms).
fn stopped(w: &mut World, id: ObjectId) {
    // spec: S-0310 Terms (the infantry speed fraction is 0 once the walker stops)
    if let Some(s) = w.obj_mut(id).and_then(|o| o.walk.as_mut()) {
        s.fraction = false;
    }
}

/// Whether the arrival-tick tests of S-0208 claims 16 and 17 apply: the
/// unit is alive and on the map. No unit falls in our engine yet.
fn arrival_tests_apply(w: &World, id: ObjectId) -> bool {
    // spec: S-0208 claim 16 (only for a unit that is alive, on the map and not falling)
    w.obj(id).is_some_and(|o| o.on_map && o.health > 0)
}

/// Whether an actor's target is in range for the arrival-time stop of a
/// walker (S-0208 claim 17) or of a driving vehicle (S-0207 claim 19 a),
/// whose arrival range tests are the same. For a type that is not
/// open-topped: S-0307's range test with the weapon S-0308 selects for the
/// target, run as if the target were the cell holding it (that cell's
/// centre at surface height is the measured point). For an open-topped
/// type: the 3-D approximate distance between the two reference points is
/// below the weapon range ρ of S-0307 claim 13.
pub fn walker_in_range(w: &World, id: ObjectId, target: ObjectId) -> bool {
    let (Some(o), Some(x)) = (w.obj(id), w.obj(target)) else {
        return false;
    };
    let r = w.rules();
    let open_topped = r
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.raw.open_topped.unwrap_or(false));
    let (slot, weapon) = crate::combat::weapon::would_use(w, id, Some(Target::Object(target)));
    if open_topped {
        let (a, b) = (w.reference_point(target), w.reference_point(id));
        let (dx, dy, dz) = (
            i64::from(a.x - b.x),
            i64::from(a.y - b.y),
            i64::from(a.z - b.z),
        );
        // spec: S-0208 claim 17 (open-topped: trunc(√̃(Δx² + Δy² + Δz²)) between the reference points, strictly below ρ)
        // spec: S-0207 claim 19 (a) (open-topped: trunc(r(Δx² + Δy² + Δz²)) < ρ between the reference points, r as in F5)
        let d = root_floor(
            (dx * dx + dy * dy + dz * dz) as u64,
            r.quirks.table_root_distance,
            r.mode,
        ) as i64;
        let rho = crate::combat::range::firer_weapon_range(w, id, slot);
        return d < i64::from(rho);
    }
    // spec: S-0208 claim 17 (S-0307's range test against the target's cell, whose centre is the measured point)
    // spec: S-0207 claim 19 (a) (S-0307's weapon range test to the target's cell centre at terrain surface height, as for a cell target)
    crate::combat::range::in_range(w, id, weapon, Some(Target::Cell(x.pos.cell())), None)
}

/// The conditions of the arrival-time stop shared by a walker (S-0208
/// claim 17) and a driving vehicle at a track target (S-0207 claim 19 a):
/// a mobile target, the current behaviour (else the queued one) *attack*,
/// *area guard*, *hunt* or *rescue*, a destination held, no approach
/// destinations, and the arrival range test.
pub fn arrival_stop_holds(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let (Some(Target::Object(x)), Some(_)) = (o.target, o.dest) else {
        return false;
    };
    // spec: S-0208 claim 17 (the target is an infantry unit, vehicle or aircraft)
    // spec: S-0207 claim 19 (a) (the target is an infantry unit, vehicle or aircraft, not a building, cell, overlay or terrain object)
    if !w
        .obj(x)
        .is_some_and(|t| matches!(t.kind, Kind::Infantry | Kind::Vehicle | Kind::Aircraft))
    {
        return false;
    }
    // spec: S-0208 claim 17 (the current behaviour, or the queued one when there is none: attack, area guard, hunt or rescue)
    // spec: S-0207 claim 19 (a) (the same behaviours)
    if !matches!(
        o.beh.current.or(o.beh.queued),
        Some(Behaviour::Attack | Behaviour::AreaGuard | Behaviour::Hunt | Behaviour::Rescue)
    ) {
        return false;
    }
    // spec: S-0207 claim 19 (a) (no approach destinations: no rule fills them yet, S-0215 Terms)
    walker_in_range(w, id, x)
}

/// S-0208 claim 17: on its arrival tick, a walker whose target is a mobile
/// actor in range, in *attack*, *area guard*, *hunt* or *rescue* (its
/// current behaviour, else its queued one), loses its destination and keeps
/// its target.
fn approach_stop(w: &mut World, id: ObjectId) {
    if !arrival_tests_apply(w, id) {
        return;
    }
    if arrival_stop_holds(w, id)
        && let Some(ob) = w.obj_mut(id)
    {
        // spec: S-0208 claim 17 (the unit loses its destination and keeps its target)
        ob.dest = None;
    }
}

/// S-0208 claim 16: on its arrival tick the walk ends when the unit has no
/// locomotor destination, or stands in that destination's cell less than
/// two height levels from it.
fn end_of_walk(w: &mut World, id: ObjectId) {
    if !arrival_tests_apply(w, id) {
        return;
    }
    let Some(o) = w.obj(id) else { return };
    let ends = match o.dest {
        // spec: S-0208 claim 16 (no locomotor destination: the walk ends)
        None => true,
        Some(d) => {
            let (cell, z) = match d {
                Target::Cell(c) => (c, cell_destination_height(w, id, c)),
                Target::Object(t) => match w.obj(t) {
                    Some(t) => (t.pos.cell(), t.pos.z),
                    None => return,
                },
            };
            // spec: S-0208 claim 16 and Constants (in the destination's cell, |Δz| < 208 leptons, strict)
            cell == o.pos.cell() && (z - o.pos.z).abs() < 208
        }
    };
    if ends && let Some(ob) = w.obj_mut(id) {
        // spec: S-0208 claim 16 (the destination is lost, the speed fraction becomes 0, the unit stands with no movement in progress)
        ob.dest = None;
        ob.path.steps.clear();
        if let Some(s) = ob.walk.as_mut() {
            s.fraction = false;
        }
    } else if let Some(ob) = w.obj_mut(id)
        && ob.path.steps.is_empty()
    {
        // spec: S-0208 claim 18 and its State table (the test fails with no path steps left: the destination is kept, the speed fraction is 0)
        if let Some(s) = ob.walk.as_mut() {
            s.fraction = false;
        }
    }
}

/// The height of a walker's cell locomotor destination (S-0208 claim 18,
/// F9): the terrain height at the cell's centre, plus 414 leptons when the
/// cell carries a high bridge, whatever the walker's layer. The decks are
/// fixed at load, so reading the bridge on arrival gives its state at
/// assignment.
pub fn cell_destination_height(w: &World, id: ObjectId, c: CellXY) -> i32 {
    // spec: S-0208 F9 (the cell centre (256cx + 128, 256cy + 128) and its terrain height H)
    let h = w.surface_at(256 * c.x + 128, 256 * c.y + 128);
    if w.rules().quirks.walker_bridge_destination_deck {
        // spec: S-0208 claim 18 and F9 (a = 414 leptons on a high-bridge cell, independent of the layer)
        h + if w.high_bridge(c) { 414 } else { 0 }
    } else {
        let deck = w.obj(id).is_some_and(|o| o.layer == Layer::Deck);
        // spec: S-0208 Edge cases (switch off: the centre's height on the arrival layer, 416 leptons up on the deck)
        h + if deck { 416 } else { 0 }
    }
}

/// The walker's asking tick (S-0209 claims 6 to 10). `first`: the first ask
/// of the tick.
fn ask(w: &mut World, id: ObjectId, first: bool) {
    let t = w.tick;
    let Some(o) = w.obj(id).cloned() else { return };
    let Some(&d) = o.path.steps.first() else {
        return;
    };
    let here = o.pos.cell();
    let (dx, dy) = STEPS[usize::from(d % 8)]; // spec: S-0212 Terms (step directions 0 to 7)
    let next = CellXY::new(here.x + dx, here.y + dy);
    let reference = occupancy::reference_level(w, id);
    let outcome = path::step_outcome(w, id, here, next, reference);
    match outcome {
        Outcome::Clear => {
            // spec: S-0209 claim 6 (clear: choose the walking target on the asking tick)
            if !choose_target(w, id, d, next) {
                // spec: S-0209 claim 6 (a clear cell with no free position: stop for this tick)
            }
        }
        Outcome::CloakedBlocker
        | Outcome::StationaryFriendly
        | Outcome::FriendlyObstacle
        | Outcome::EnemyObstacle
        | Outcome::Blocked => {
            if first {
                // spec: S-0209 claim 7 (stop, discard the path, search a new one at once whatever the retry timer)
                if let Some(ob) = w.obj_mut(id) {
                    ob.path.steps.clear();
                }
                match path::compute(w, id, Mode::Fresh) {
                    // spec: S-0209 State and ordering (the repeated ask on the same tick: clear with a position free chooses the walking target in the new path's first cell)
                    Search::Found => ask(w, id, false),
                    other => path::failed(w, id, other, false),
                }
                return;
            }
            match outcome {
                Outcome::StationaryFriendly => stationary_friendly(w, id, next),
                Outcome::FriendlyObstacle | Outcome::EnemyObstacle => {
                    // spec: S-0209 claim 9 (the first listed object not allied becomes the target; a wall cell otherwise)
                    let layer = occupancy::layers_for(w, next, reference).0;
                    let first_obj = w.cells.list(next, layer).first().copied();
                    let owner = o.owner;
                    let tgt = match first_obj {
                        Some(f) => (!w.allied(owner, w.obj(f).and_then(|x| x.owner)))
                            .then_some(Target::Object(f)),
                        None => Some(Target::Cell(next)),
                    };
                    if let Some(ob) = w.obj_mut(id) {
                        ob.path.steps.clear();
                        if tgt.is_some() {
                            ob.target = tgt;
                        }
                    }
                }
                // spec: S-0209 claim 7 (repeated ask blocked: keep the new path, wait; cloaked: wait)
                _ => {}
            }
        }
        Outcome::ClosedFriendlyGate => {
            // spec: S-0209 claim 9 (the walker makes the gate open and waits, without searching a new path)
            if let Some(gate) = crate::process::closed_friendly_gate(w, id, next) {
                crate::process::gate_met(w, gate);
            }
        }
        Outcome::MovingBlocker => {
            // spec: S-0209 claim 10 (wait; start the blockage delay on the first such tick)
            let since = o.path.blocked_since.unwrap_or(t);
            if let Some(ob) = w.obj_mut(id) {
                ob.path.blocked_since = Some(since);
            }
            if t >= o.path.retry_at {
                let delay = w.rules().general.blockage_delay;
                // spec: S-0209 claim 10 (re-path when the retry timer allows; avoid blockers once the delay has run out)
                let mode = if t.wrapping_sub(since) >= delay {
                    Mode::Avoid
                } else {
                    Mode::Blocked
                };
                let res = path::compute(w, id, mode);
                // spec: S-0212 claim 21 (walker: after a re-path for a moving blocker the delay R starts again)
                let r = w.rules().general.retry_delay;
                if let Some(ob) = w.obj_mut(id) {
                    ob.path.retry_at = t.wrapping_add(r);
                }
                if !matches!(res, Search::Found) {
                    path::failed(w, id, res, true);
                }
            }
        }
    }
}

/// Claim 8 of S-0209: give up within the close-enough distance, or ask the
/// cell's occupants to make room.
fn stationary_friendly(w: &mut World, id: ObjectId, next: CellXY) {
    let Some(o) = w.obj(id).cloned() else { return };
    let r = w.rules_arc();
    if let Some(dest) = path::heading_point(w, id) {
        let dx = i64::from(dest.x - o.pos.x);
        let dy = i64::from(dest.y - o.pos.y);
        let dz = i64::from(dest.z - o.pos.z);
        // spec: S-0209 F1 (D = trunc(√̃(Δx² + Δy² + Δz²)), give up when D < close-enough)
        let d = root_floor(
            (dx * dx + dy * dy + dz * dz) as u64,
            r.quirks.table_root_distance,
            r.mode,
        ) as i64;
        let tunnel = w
            .cells
            .cell(o.pos.cell())
            .is_some_and(|c| c.land == land::TUNNEL);
        // spec: S-0209 claim 8 (within the close-enough distance, in contact with nothing, |Δz| < 208, not a tunnel cell)
        if d < i64::from(r.general.close_enough) && o.contact.is_none() && dz.abs() < 208 && !tunnel
        {
            if let Some(ob) = w.obj_mut(id) {
                ob.dest = None;
                ob.path.steps.clear();
            }
            return;
        }
    }
    // spec: S-0209 claim 8 (a scatter request to every object listed there: no threat point, forced, move-anywhere)
    let reference = occupancy::reference_level(w, id);
    let layer = occupancy::layers_for(w, next, reference).0;
    scatter::cell_request(
        w,
        next,
        layer,
        scatter::ScatterRequest {
            threat: None,
            forced: true,
            anywhere: true,
        },
    );
}

/// The standing position a unit gets in cell `c` for a desired point of
/// class `class` (S-0208 claims 3 and 4), reading the occupancy record of
/// layer `l`. `ignore` is claim 4's case: occupants are ignored and the
/// position is the class's own, without a draw. A centre class otherwise
/// takes one ranged draw.
pub(crate) fn choose_position(
    w: &mut World,
    c: CellXY,
    l: Layer,
    class: u8,
    ignore: bool,
) -> Option<u8> {
    let half = w.rules().quirks.ranged_draw_half_span;
    if ignore {
        // spec: S-0208 claim 4 (occupants ignored: the standing position of the desired point's class, no draw)
        Some(class)
    } else if occupancy::no_position_by_flags(w, c, l) {
        // spec: S-0208 claim 3 (first row: vehicle-present, or object-present without an open gate)
        None
    } else if class != position::CENTRE {
        let rec = w.cells.occ(c, l).copied().unwrap_or_default();
        if !rec.taken(class) {
            Some(class)
        } else {
            occupancy::first_free(w, c, l, row_for(class, 0))
        }
    } else {
        // spec: S-0208 claim 3 (centre class: one ranged draw over 0 … 3 picks the row)
        let k = w.rng.ranged(0, 3, half);
        occupancy::first_free(w, c, l, row_for(class, k))
    }
}

/// Whether claim 4's first case holds for cell `c`: the unit's mission
/// target is a vehicle or aircraft positioned in it, or a building
/// occupying it (S-0208 claim 4).
pub(crate) fn mission_target_in(w: &World, id: ObjectId, c: CellXY) -> bool {
    // spec: S-0208 claim 4 (a mission target that is a vehicle or aircraft positioned in the cell, or a building occupying it; not a cell or an infantry unit)
    match occupancy::mission_target(w, id) {
        Some(Target::Object(t)) => w.obj(t).is_some_and(|m| match m.kind {
            Kind::Vehicle | Kind::Aircraft => m.pos.cell() == c,
            Kind::Building => w.cells.list(c, Layer::Ground).contains(&t),
            _ => false,
        }),
        _ => false,
    }
}

/// Chooses the walking target in the next cell (S-0208 claims 3 and 4).
/// Returns whether a target was chosen.
fn choose_target(w: &mut World, id: ObjectId, d: u8, next: CellXY) -> bool {
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    let q = w.rules().quirks;
    let (dx, dy) = STEPS[usize::from(d % 8)]; // spec: S-0212 Terms (step directions 0 to 7)
    // spec: S-0208 claim 3 (the desired point: the current position moved by one cell step)
    let desired = (o.pos.x + 256 * dx, o.pos.y + 256 * dy);
    let class = position_class(desired.0.rem_euclid(256), desired.1.rem_euclid(256), true); // spec: S-0204 claim 2 (the position within the cell)
    let mission = mission_target_in(w, id, next);
    // The second case of claim 4 (an enslaved unit at its master's unloading cell) needs
    // enslaved units, which no topic of M3 creates.
    // spec: S-0208 claim 4 (the mission target's cell: occupants ignored, no draw)
    let pos = choose_position(w, next, o.layer, class, mission);
    let Some(p) = pos else { return false };
    let (u, v) = standing_point(p);
    let (x, y) = (256 * next.x + u, 256 * next.y + v); // spec: S-0208 table P (a standing position within the cell)
    // spec: S-0208 claim 1 (the walking target lies at the surface height there, 416 above on a deck)
    let z = w.surface_at(x, y) + if o.layer == Layer::Deck { 416 } else { 0 };
    let target = Pos::new(x, y, z);
    // spec: S-0209 claim 1 (reservation: the old mark is cleared and the new target marked at once)
    let oq = crate::objects::occ_quirks(w);
    w.cells.clear_infantry(o.pos.cell(), o.layer, id, oq);
    let mark = position_class(u, v, q.top_position_counts_as_centre);
    if let Some(h) = o.owner {
        w.cells.mark_infantry(next, o.layer, mark, id, h);
    }
    if let Some(ob) = w.obj_mut(id) {
        ob.path.steps.remove(0);
        if let Some(s) = ob.walk.as_mut() {
            s.target = Some(target);
            s.standing = p;
        }
    }
    true
}

/// Arrival on the walking target (S-0208 claim 6).
fn arrive(w: &mut World, id: ObjectId, target: Pos) {
    let t = w.tick;
    let Some(o) = w.obj(id).cloned() else { return };
    let old = o.pos.cell();
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0208 claim 6 (placed exactly on the walking target, which it reaches with its height)
        ob.pos = target;
        if let Some(s) = ob.walk.as_mut() {
            s.target = None;
            s.arrived = Some(t);
            s.moved_tick = Some(t);
        }
    }
    if target.cell() != old {
        changed_cell(w, id, old, target.cell());
    }
}

/// The walker's position moved into another cell: it leaves the old cell's
/// list and enters the new one's (S-0209 claim 13); both cells are
/// recomputed (S-0211 claim 5).
fn changed_cell(w: &mut World, id: ObjectId, from: CellXY, to: CellXY) {
    let layer = w.obj(id).map_or(Layer::Ground, |o| o.layer);
    w.cells.leave(from, layer, id);
    w.cells.enter(to, layer, id, false);
    crate::load::recalc_cells(w, &[from, to]);
}

/// The 8-direction from one cell towards a neighbour.
pub fn step_dir(from: CellXY, to: CellXY) -> Option<u8> {
    STEPS
        .iter()
        .position(|&(dx, dy)| (from.x + dx, from.y + dy) == (to.x, to.y))
        .map(|i| i as u8)
}

/// Whether a facing is within `tol` of another.
pub fn near(a: u16, b: u16, tol: i32) -> bool {
    delta(a, b).abs() <= tol
}

#[cfg(test)]
mod tests {
    use super::*;

    const RZ: Rounding = Rounding::TowardZero;

    // spec: S-0208 claim 2 (position classes; the top position has class centre)
    #[test]
    fn classes() {
        assert_eq!(position_class(128, 128, true), position::CENTRE);
        assert_eq!(position_class(64, 64, true), position::CENTRE);
        assert_eq!(position_class(192, 64, true), position::RIGHT);
        assert_eq!(position_class(64, 192, true), position::LEFT);
        assert_eq!(position_class(192, 192, true), position::BOTTOM);
        assert_eq!(position_class(64, 64, false), position::TOP);
        // r < 60 exactly: (u − 128)² + (v − 128)² < 3600.
        assert_eq!(position_class(188, 128, true), position::RIGHT);
        assert_eq!(position_class(187, 128, true), position::CENTRE);
    }

    // spec: S-0208 claim 14 (due east exact; due north exact; due south drifts west by 1; due west drifts north by 1)
    #[test]
    fn axis_steps() {
        let east = step_point(1000, 1000, 16383, 9, true, RZ);
        assert_eq!(east, (1009, 1000));
        let north = step_point(1000, 1000, 0, 9, true, RZ);
        assert_eq!(north, (1000, 991));
        let south = step_point(1000, 1000, 32767, 9, true, RZ);
        assert_eq!(south, (999, 1009));
        let west = step_point(1000, 1000, 49153, 9, true, RZ);
        assert_eq!(west, (991, 999));
        // With round to nearest there is no drift.
        let rn = Rounding::Nearest;
        assert_eq!(step_point(1000, 1000, 32767, 9, true, rn), (1000, 1009));
        assert_eq!(step_point(1000, 1000, 49153, 9, true, rn), (991, 1000));
    }

    // spec: S-0208 F3 (c₂ is the double −9.587672516830327 × 10⁻⁵, nearest to −2π/65534)
    #[allow(clippy::disallowed_types, clippy::float_arithmetic)]
    #[test]
    fn step_constant() {
        let c2 = crate::consts::WALK_C2;
        let want = -2.0 * std::f64::consts::PI / 65534.0;
        assert_eq!(c2.0, want.to_bits());
        let c = crate::consts::WALK_C;
        assert_eq!(c.0, (-2.0 * std::f64::consts::PI / 65536.0).to_bits());
    }

    // spec: S-0208 claim 14 (a walker of speed 1 heading between east and south stalls)
    #[test]
    fn slow_walker_stalls() {
        let p = step_point(1000, 1000, 24451, 1, true, RZ);
        assert_eq!(p, (1000, 1000));
    }
}
