//! Pathfinding (S-0212), behind the [`Pathfinder`] interface.
//!
//! [`S0212`] implements the specified parts of S-0212: the reachability
//! gate (claim 6), the cell search with its costs, estimate, recording and
//! limits (claims 7 to 12, F1, F2), a blocked destination (claim 15), the
//! flattening of right-angle turns (claim 17), congestion marks (claim 19),
//! path storage and retry delays (claims 20, 21), failures (claim 23),
//! destination substitution (claim 24) and the nearby-cell search (claim 25).
//! The corridor (claims 13 and 14), the stretch selection of claim 18 and the
//! estimate E of claim 24 wait for request Q-0017: every search runs over
//! the whole map.

use std::collections::{BTreeMap, BTreeSet, BinaryHeap};

use glasnost_map::CellXY;
use glasnost_num::geom::{root_floor, table_root};
use glasnost_num::{F32, F64, Rounding};
use glasnost_rules::sim::{LocomotionClass, MovementZone};
use serde::{Deserialize, Serialize};

use crate::occupancy::{self, Outcome};
use crate::regions::{self, Reach, STEPS};
use crate::rules::{Behaviour, Kind, Locomotor};
use crate::world::{Layer, ObjectId, Pos, Target, World};

/// A unit's stored path and its timers (S-0212 claims 20 to 22).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathState {
    /// Stored step directions, first step first (at most 24).
    pub steps: Vec<u8>,
    /// Tick at which the retry delay is over.
    pub retry_at: i32,
    /// Tick at which the current blockage began, if any.
    pub blocked_since: Option<i32>,
    /// The destination the stored path was computed for.
    pub for_goal: Option<CellXY>,
}

/// Search mode (S-0212 Terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// 0: a fresh path.
    Fresh,
    /// 1: a re-path after a moving blocker.
    Blocked,
    /// 2: a re-path after the blockage delay.
    Avoid,
}

/// The result of a path computation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Search {
    /// A path was stored.
    Found,
    /// The gate of claim 6 failed.
    GateFailed,
    /// The search failed.
    SearchFailed,
}

/// A search request handed to a pathfinder.
#[derive(Clone, Debug)]
pub struct Request {
    /// The moving unit.
    pub unit: ObjectId,
    /// Start cell.
    pub from: CellXY,
    /// Destination cell.
    pub to: CellXY,
    /// Search mode.
    pub mode: Mode,
    /// Cells carrying the congestion mark during this search (claim 19).
    pub congested: BTreeSet<CellXY>,
}

/// A pathfinder: finds the steps from one cell to another for a unit. The
/// simulation calls it through [`compute`]; implementations must be
/// deterministic.
pub trait Pathfinder {
    /// The steps of a path, or `None` when the search fails.
    fn find(&self, w: &World, req: &Request) -> Option<Vec<u8>>;
}

/// The pathfinder of S-0212 (without the corridor, Q-0017).
pub struct S0212;

/// Which pathfinder a game uses.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PathfinderChoice {
    /// S-0212.
    #[default]
    S0212,
}

/// The stored path length (S-0212 claim 20).
// spec: S-0212 Constants (stored path length 24 steps)
pub const STORED: usize = 24;

/// The direction term ε(d) of S-0212 table T4.
fn epsilon(d: usize) -> F64 {
    // spec: S-0212 table T4 (the single-precision values nearest these decimals)
    crate::consts::EPSILON[d % 8]
}

/// The base cost b(v) of S-0212 claim 8 and table T2.
fn base_cost(w: &World, req: &Request, v: Outcome, cell: CellXY, train: bool) -> i64 {
    if train {
        // spec: S-0212 claim 8 (a train: every outcome but blocked costs 1)
        return 1;
    }
    // spec: S-0212 table T2 (base step cost by entry outcome)
    match v {
        Outcome::Clear | Outcome::ClosedFriendlyGate => 1,
        Outcome::CloakedBlocker => 1000,
        Outcome::FriendlyObstacle => 60,
        Outcome::EnemyObstacle => 20,
        Outcome::StationaryFriendly => 8,
        Outcome::Blocked => 0,
        Outcome::MovingBlocker => match req.mode {
            // spec: S-0212 claim 8 (moving blocker: mode 2 1000, mode 1 4, mode 0 the blocker chain)
            Mode::Avoid => 1000,
            Mode::Blocked => 4,
            Mode::Fresh => blocker_chain(w, cell),
        },
    }
}

/// Where a unit is heading (S-0212 claim 8).
fn heading_cell(w: &World, id: ObjectId) -> Option<CellXY> {
    let o = w.obj(id)?;
    let here = o.pos.cell();
    let speed = o.drive.as_ref().is_some_and(|d| d.track.is_some())
        || o.walk.as_ref().is_some_and(|s| s.target.is_some());
    if speed {
        // spec: S-0212 claim 8 (speed not zero: one step in the 8-direction of its body facing)
        let d = glasnost_num::geom::eight_direction(o.body.value(w.tick));
        let (dx, dy) = STEPS[usize::from(d)];
        Some(CellXY::new(here.x + dx, here.y + dy))
    } else {
        // spec: S-0212 claim 8 (zero speed: the cell of its first stored step; none when the path is empty)
        o.path.steps.first().map(|&d| {
            let (dx, dy) = STEPS[usize::from(d % 8)];
            CellXY::new(here.x + dx, here.y + dy)
        })
    }
}

/// The blocker chain of S-0212 claim 8 (mode 0 moving blocker cost).
fn blocker_chain(w: &World, first: CellXY) -> i64 {
    let mut c = first;
    // spec: S-0212 claim 8 (a chain of at most 10 cells)
    for i in 1..=10 {
        let Some(&oid) = w.cells.list(c, Layer::Ground).first() else {
            // spec: S-0212 claim 8 (last cell holds no object: 1)
            return 1;
        };
        let Some(o) = w.obj(oid) else { return 1 };
        if !o.kind.is_mobile() {
            // spec: S-0212 claim 8 (its object is not a unit: 4)
            return 4;
        }
        match heading_cell(w, oid) {
            // spec: S-0212 claim 8 (a unit heading nowhere ends the chain as an empty cell: 1)
            None => return 1,
            Some(next) => {
                if i == 10 {
                    // spec: S-0212 claim 8 (i = 10 and still heading on: 4)
                    return 4;
                }
                c = next;
            }
        }
    }
    1
}

/// Whether the height rule of S-0212 claim 10 allows a step.
fn height_ok(w: &World, a: CellXY, b: CellXY) -> bool {
    let (Some(ca), Some(cb)) = (w.cells.cell(a), w.cells.cell(b)) else {
        return false;
    };
    let diff = (i32::from(ca.level) - i32::from(cb.level)).abs();
    match diff {
        0 => true,
        // spec: S-0212 claim 10 (a difference of 1 only when the lower cell is a slope)
        1 => {
            let lower = if ca.level < cb.level { ca } else { cb };
            lower.slope != 0
        }
        // spec: S-0212 claim 10 (4 only between a deck and a bridge end; 2, 3 and above 4 blocked)
        _ => false,
    }
}

/// The entry outcome a unit gets for stepping from `from` into `to`, with
/// the height rule (S-0212 claim 10; S-0209 claims 4 and 5).
pub fn step_outcome(w: &World, id: ObjectId, from: CellXY, to: CellXY, reference: i32) -> Outcome {
    if !height_ok(w, from, to) {
        return Outcome::Blocked;
    }
    let Some(o) = w.obj(id) else {
        return Outcome::Blocked;
    };
    // spec: S-0211 Terms (a unit inside the usable area may not step outside it)
    if w.cells.usable(from) && !w.cells.usable(to) {
        return Outcome::Blocked;
    }
    match o.kind {
        Kind::Infantry => occupancy::infantry_outcome(w, id, to, reference),
        // spec: S-0209 claim 14 (an aircraft gets clear everywhere in skirmish games)
        Kind::Aircraft => Outcome::Clear,
        _ => occupancy::vehicle_outcome(w, id, to, reference),
    }
}

/// Reusable working memory of the cell search, indexed by dense cell index.
/// Not game state: a search starts from a fresh stamp, so nothing carries
/// over from one search to the next.
#[derive(Clone, Debug, Default)]
pub struct SearchScratch {
    stamp: u32,
    /// `seen[i] == stamp`: cell i has a recorded g.
    seen: Vec<u32>,
    /// `closed[i] == stamp`: cell i has been expanded.
    closed: Vec<u32>,
    g: Vec<F32>,
    pred: Vec<(CellXY, u8)>,
}

impl SearchScratch {
    fn begin(&mut self, len: usize) {
        if self.seen.len() != len || self.stamp == u32::MAX {
            *self = Self {
                stamp: 0,
                seen: vec![0; len],
                closed: vec![0; len],
                g: vec![F32::ZERO; len],
                pred: vec![(CellXY::new(0, 0), 0); len],
            };
        }
        self.stamp += 1;
    }

    fn g(&self, i: usize) -> Option<F32> {
        (self.seen[i] == self.stamp).then(|| self.g[i])
    }
}

/// An entry of the search's pending set: ordered by f, then by recording.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Pending {
    f: u32,
    seq: u32,
    cell: CellXY,
}

impl Ord for Pending {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Min-heap on (f, seq).
        (other.f, other.seq).cmp(&(self.f, self.seq))
    }
}

impl PartialOrd for Pending {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// trunc24 of a double (S-0212 F1: storing in single precision).
fn t24(x: F64, mode: Rounding) -> F32 {
    x.to_f32(mode)
}

impl Pathfinder for S0212 {
    fn find(&self, w: &World, req: &Request) -> Option<Vec<u8>> {
        let r = w.rules();
        let q = r.quirks;
        let mode = r.mode;
        let o = w.obj(req.unit)?;
        let u = r.unit(o.kind, o.type_index)?;
        let train = u.train;
        let passive = u.passive && o.kind == Kind::Vehicle;
        let from = req.from;
        let to = req.to;
        // spec: S-0212 claim 12 (a found path must contain at least one step)
        if from == to {
            return None;
        }
        let reference = occupancy::reference_level(w, req.unit);
        let est = |c: CellXY| -> F32 {
            let dx = i64::from(c.x - to.x);
            let dy = i64::from(c.y - to.y);
            // spec: S-0212 F2 (h = r((x − xₜ)² + (y − yₜ)²) in cells, the table root whatever the switch)
            table_root((dx * dx + dy * dy) as u64, mode)
        };
        let mut sc = w.scratch.borrow_mut();
        sc.begin(w.cells.grid.len());
        let ix = |c: CellXY| w.cells.idx(c);
        let fi = ix(from)?;
        let mut heap: BinaryHeap<Pending> = BinaryHeap::new();
        let mut seq = 0u32;
        let stamp = sc.stamp;
        sc.seen[fi] = stamp;
        sc.g[fi] = F32::ZERO;
        // spec: S-0212 F1 (recorded-pair margin μ = 1.009, the double nearest)
        let mu = crate::consts::PATH_MARGIN;
        let mut held: Option<Pending> = Some(Pending {
            f: t24(est(from).to_f64(), mode).0,
            seq,
            cell: from,
        });
        let mut expansions = 0u32;
        let mut recorded = 0u32;
        // spec: S-0212 claim 16 (a train may not enter the three cells behind it)
        let behind: Vec<CellXY> = if train {
            let f8 = i32::from(glasnost_num::geom::eight_direction(o.body.value(w.tick)));
            (0..8)
                .filter(|d| matches!((d - f8).rem_euclid(8), 3..=5))
                .map(|d| {
                    let (dx, dy) = STEPS[d as usize];
                    CellXY::new(from.x + dx, from.y + dy)
                })
                .collect()
        } else {
            Vec::new()
        };
        loop {
            // Next pair: the held candidate when strictly cheapest, else the
            // cheapest pending pair, earliest recorded first (Q-0017).
            let next = match (held, heap.peek().copied()) {
                (Some(h), Some(top)) if h.f < top.f => {
                    held = None;
                    h
                }
                (Some(h), Some(_)) => {
                    held = None;
                    heap.push(h);
                    heap.pop()?
                }
                (Some(h), None) => {
                    held = None;
                    h
                }
                (None, Some(_)) => heap.pop()?,
                (None, None) => return None,
            };
            let Some(ni) = ix(next.cell) else { continue };
            if sc.closed[ni] == stamp {
                continue;
            }
            if !q.path_no_reopen {
                // A stale entry superseded by a cheaper record.
                let cur = sc.g(ni).unwrap_or(F32::ZERO);
                if t24(cur.to_f64().add(est(next.cell).to_f64(), mode), mode).0 != next.f {
                    continue;
                }
            }
            if next.cell == to {
                // spec: S-0212 claim 12 (reaching the destination after exactly 10000 expansions fails)
                if q.path_search_10000_failure && expansions == 10000 {
                    return None;
                }
                return Some(reconstruct(&sc, &ix, from, to));
            }
            // spec: S-0212 claim 12 (at most 65527 expansions)
            if expansions >= 65527 {
                return None;
            }
            expansions += 1;
            sc.closed[ni] = stamp;
            let ge = sc.g(ni).unwrap_or(F32::ZERO);
            let mut best_new: Option<Pending> = None;
            for (d, (dx, dy)) in STEPS.iter().enumerate() {
                let c = CellXY::new(next.cell.x + dx, next.cell.y + dy);
                let Some(ci) = ix(c) else { continue };
                if w.cells.cell(c).is_none() || behind.contains(&c) {
                    continue;
                }
                let v = step_outcome(w, req.unit, next.cell, c, reference);
                if v == Outcome::Blocked {
                    // spec: S-0212 claim 15 (a blocked destination is approached; not for passive vehicles)
                    if c == to && !passive && next.cell != from {
                        let margin_ok = match sc.g(ci) {
                            None => true,
                            Some(gr) => {
                                // spec: S-0212 F1 (recorded-pair margin: g_r ≥ trunc₅₃(g_e + 1.009))
                                !q.path_no_reopen
                                    || gr.to_f64().ge(ge.to_f64().add(mu, Rounding::TowardZero))
                            }
                        };
                        if margin_ok {
                            if expansions == 10000 && q.path_search_10000_failure {
                                return None;
                            }
                            return Some(reconstruct(&sc, &ix, from, next.cell));
                        }
                    }
                    continue;
                }
                // spec: S-0212 F1 (c = trunc₂₄(b × τ + ε(d)); τ = 4 on a congestion mark)
                let tau = if req.congested.contains(&c) { 4 } else { 1 };
                let b = base_cost(w, req, v, c, train) * tau;
                let cost = t24(F64::from_i64(b, mode).add(epsilon(d), mode), mode);
                // spec: S-0212 F1 (g(new) = trunc₂₄(g(current) + c))
                let gn = t24(ge.to_f64().add(cost.to_f64(), mode), mode);
                let recorded_g = sc.g(ci);
                let fresh = recorded_g.is_none();
                // spec: S-0212 claim 11 (a pair is recorded once; quirks.path_no_reopen)
                let better =
                    recorded_g.is_some_and(|old| !q.path_no_reopen && gn.to_f64().lt(old.to_f64()));
                if !fresh && !better {
                    continue;
                }
                if fresh {
                    recorded += 1;
                    // spec: S-0212 claim 12 (departure D2: at most 65535 recorded pairs besides the start)
                    if recorded > 65535 {
                        return None;
                    }
                }
                if better {
                    sc.closed[ci] = 0;
                }
                sc.seen[ci] = stamp;
                sc.g[ci] = gn;
                sc.pred[ci] = (next.cell, d as u8);
                seq += 1;
                // spec: S-0212 F2 (f = trunc₂₄(g + h))
                let f = t24(gn.to_f64().add(est(c).to_f64(), mode), mode).0;
                let p = Pending { f, seq, cell: c };
                // The held candidate: smallest f, first in generation order.
                match best_new {
                    Some(b) if b.f <= f => heap.push(p),
                    Some(b) => {
                        heap.push(b);
                        best_new = Some(p);
                    }
                    None => best_new = Some(p),
                }
            }
            held = best_new;
        }
    }
}

fn reconstruct(
    sc: &SearchScratch,
    ix: &dyn Fn(CellXY) -> Option<usize>,
    from: CellXY,
    to: CellXY,
) -> Vec<u8> {
    let mut steps = Vec::new();
    let mut c = to;
    while c != from {
        let Some(i) = ix(c).filter(|&i| sc.seen[i] == sc.stamp) else {
            break;
        };
        let (p, d) = sc.pred[i];
        steps.push(d);
        c = p;
    }
    steps.reverse();
    steps
}

/// The acceptance test of a replacement in [`flatten`].
pub type ReplaceOk<'a> = dyn Fn(&[u8], usize, u8, usize) -> bool + 'a;

/// The flattening of right-angle turns (S-0212 claim 17). `ok(i, d3, j)`
/// tells whether the 2j new steps in direction d3 replacing the V that ends
/// with the first run at step index i are acceptable.
pub fn flatten(steps: &[u8], straight_v: bool, ok: &ReplaceOk<'_>) -> Vec<u8> {
    let mut path: Vec<u8> = steps.to_vec();
    // `fin`: steps before this index are final. `straight_ok`: the run that
    // begins at `fin` is a straight run that may open a V.
    let mut fin: usize = 0;
    let mut straight_ok = false;
    while fin < path.len() {
        // The run starting at fin.
        let d1 = path[fin];
        let mut n = 1;
        while fin + n < path.len() && path[fin + n] == d1 {
            n += 1;
        }
        let second = fin + n;
        if second >= path.len() {
            break;
        }
        let d2 = path[second];
        let mut m = 1;
        while second + m < path.len() && path[second + m] == d2 {
            m += 1;
        }
        let rel = (i32::from(d2) - i32::from(d1)).rem_euclid(8); // spec: S-0212 claim 17 (the turn between runs, in eighths)
        let diagonal = d1 % 2 == 1; // spec: S-0212 Terms (odd steps are diagonal)
        // spec: S-0212 claim 17 (a V: a run then a run at a right angle; diagonal runs always open one)
        let can_open = diagonal || (straight_v && straight_ok);
        // spec: S-0212 claim 17 (a right angle: two eighths either way)
        if !(rel == 2 || rel == 6) || !can_open {
            fin = second;
            straight_ok = false;
            continue;
        }
        // spec: S-0212 claim 17 (d₃ by the pair: diagonal pairs give the straight between; 6 and 0 give 0)
        let d3 = between(d1, d2);
        let mut j = n.min(m);
        while j >= 1 && !ok(&path, second - j, d3, j) {
            j -= 1;
        }
        if j == 0 {
            // spec: S-0212 claim 17 (a V not flattened makes final every step up to the end of its first run)
            fin = second;
            // spec: S-0212 claim 17 (the second run of an unflattened V of straight runs may open a V)
            straight_ok = !diagonal;
            continue;
        }
        let start = second - j;
        let ends_path = second + m == path.len();
        let mut np: Vec<u8> = path[..start].to_vec();
        np.extend(std::iter::repeat_n(d3, 2 * j)); // spec: S-0212 claim 17 (2j new steps)
        np.extend_from_slice(&path[second + j..]);
        path = np;
        // spec: S-0212 claim 17 (the last V judged is the one whose second run ends the path)
        if ends_path {
            break;
        }
        // spec: S-0212 claim 17 (judging resumes at the first new step; the kept steps are final)
        fin = start;
        // spec: S-0212 claim 17 (new straight steps, of a diagonal V or of the pair 6 and 0, may open a V)
        straight_ok = d3.is_multiple_of(2);
    }
    path
}

fn between(a: u8, b: u8) -> u8 {
    let (lo, hi) = if a < b { (a, b) } else { (b, a) };
    // spec: S-0212 claim 17 (7,1→0; 1,3→2; 3,5→4; 5,7→6; 0,2→1; 2,4→3; 4,6→5; 6,0→0)
    match (lo, hi) {
        (1, 7) => 0,
        (1, 3) => 2,
        (3, 5) => 4,
        (5, 7) => 6,
        (0, 2) => 1,
        (2, 4) => 3,
        (4, 6) => 5,
        (0, 6) => 0,
        _ => lo,
    }
}

/// The cell a unit's destination names (its heading point's cell).
pub fn goal_cell(w: &World, id: ObjectId) -> Option<CellXY> {
    heading_point(w, id).map(|p| p.cell())
}

/// The heading point (S-0212 Terms): the position of the destination.
pub fn heading_point(w: &World, id: ObjectId) -> Option<Pos> {
    let o = w.obj(id)?;
    match o.dest? {
        Target::Cell(c) => Some(Pos::centre(
            c,
            w.surface_at(256 * c.x + 128, 256 * c.y + 128), // spec: S-0204 claim 3 (the cell centre)
        )),
        Target::Object(t) => w.obj(t).map(|x| x.pos),
    }
}

/// The locomotor destination (S-0208 Terms).
pub fn locomotor_destination(w: &World, id: ObjectId) -> Option<Pos> {
    let o = w.obj(id)?;
    if !occupancy::in_progress(w, id) && o.dest.is_none() {
        return None;
    }
    heading_point(w, id)
}

/// The simplified movement zone of S-0212 claim 25.
fn simplified(z: MovementZone) -> MovementZone {
    use MovementZone as Z;
    // spec: S-0212 claim 25 (simplified zones: crushing and destroying variants map to their base)
    match z {
        Z::Normal | Z::Crusher | Z::Destroyer | Z::CrusherAll => Z::Normal,
        Z::Amphibious | Z::AmphibiousCrusher | Z::AmphibiousDestroyer => Z::Amphibious,
        Z::Infantry | Z::InfantryDestroyer => Z::Infantry,
        other => other,
    }
}

/// A use of the nearby-cell search (S-0212 table T5).
#[derive(Clone, Copy, Debug)]
pub enum NearbyUse {
    /// Destination substitution (claim 24).
    Substitution {
        /// The unit.
        unit: ObjectId,
    },
    /// A scatter without a threat point (S-0208 claim 13).
    Scatter {
        /// The unit's locomotion class.
        class: Option<LocomotionClass>,
        /// In deck mode.
        deck: bool,
    },
    /// A move order onto a cell (S-0214 claim 11).
    MoveOrder {
        /// The unit.
        unit: ObjectId,
    },
}

/// The nearby-cell search (S-0212 claim 25).
pub fn nearby(w: &mut World, centre: CellXY, use_: NearbyUse) -> Option<CellXY> {
    let q = w.rules().quirks;
    let fp = *w.cells.footprint();
    // spec: S-0212 claim 25 (largest radius 31, or w + h − 1 when w + h < 32)
    let max_r = if fp.width + fp.height < 32 {
        fp.width + fp.height - 1
    } else {
        31
    };
    let (class, zone, reach_ref, deck_mode, height_cond, reference) = match use_ {
        NearbyUse::Substitution { unit } => {
            let o = w.obj(unit)?;
            let u = w.rules().unit(o.kind, o.type_index)?.clone();
            let z = simplified(u.zone);
            let rr = regions::reach(w, o.pos.cell(), z);
            let o = w.obj(unit)?;
            (
                u.class,
                z,
                Some(rr),
                o.layer == Layer::Deck,
                true,
                Some(o.pos.cell()),
            )
        }
        NearbyUse::Scatter { class, deck } => {
            // spec: S-0212 table T5 (scatter: zone normal, no reach region, no reference cell)
            (class, MovementZone::Normal, None, deck, true, None)
        }
        NearbyUse::MoveOrder { unit } => {
            let o = w.obj(unit)?;
            let u = w.rules().unit(o.kind, o.type_index)?.clone();
            // spec: S-0212 table T5 (move order: own zone, subterranean as normal, fly as infantry under an ion storm)
            let z = if u.zone == MovementZone::Subterranean {
                MovementZone::Normal
            } else {
                u.zone
            };
            let rr = regions::reach(w, o.pos.cell(), z);
            let deck = w.cells.idx(centre).is_some_and(|i| w.cells.deck[i]);
            (u.class, z, Some(rr), deck, true, None)
        }
    };
    let centre_level = w.cells.cell(centre).map_or(0, |c| i32::from(c.level))
        + if deck_mode && w.cells.idx(centre).is_some_and(|i| w.cells.deck[i]) {
            4 // spec: S-0207 claim 9 (a deck counts 4 levels above its cell)
        } else {
            0
        };
    let mut collected: Vec<CellXY> = Vec::new();
    let mut stop_after_ring = false;
    'rings: for r in 0..=max_r {
        let ring = ring_cells(centre, r, q.nearby_centre_twice);
        for c in ring {
            if qualifies(
                w,
                c,
                class,
                zone,
                reach_ref,
                deck_mode,
                height_cond,
                centre_level,
            ) {
                collected.push(c);
                // spec: S-0212 claim 25 (cut off at the 24th collected cell)
                if collected.len() >= 24 {
                    break 'rings;
                }
                // spec: S-0212 claim 25 (a ring with a visible cell, or in deck mode any cell, ends the collection)
                if deck_mode || crate::scatter::screen_cover_passes(w, c) {
                    stop_after_ring = true;
                }
            }
        }
        if stop_after_ring {
            break;
        }
    }
    if collected.is_empty() {
        return None;
    }
    let visible: Vec<CellXY> = collected
        .iter()
        .copied()
        .filter(|&c| crate::scatter::screen_cover_passes(w, c))
        .collect();
    let preferred = if visible.is_empty() {
        collected
    } else {
        visible
    };
    match reference {
        Some(rc) => {
            // spec: S-0212 claim 25 (the preferred cell nearest the reference cell; the first collected on ties)
            let mut best: Option<(u64, CellXY)> = None;
            for c in preferred {
                let dx = i64::from(c.x - rc.x);
                let dy = i64::from(c.y - rc.y);
                let v = u64::from(table_root((dx * dx + dy * dy) as u64, w.rules().mode).0);
                if best.is_none_or(|(b, _)| v < b) {
                    best = Some((v, c));
                }
            }
            best.map(|b| b.1)
        }
        None => {
            // spec: S-0212 claim 25 (no reference cell: the preferred cell at position N mod n)
            let n = preferred.len() as i32;
            preferred.get(w.tick.rem_euclid(n) as usize).copied()
        }
    }
}

/// Whether a cell qualifies for the nearby-cell search (S-0212 claim 25).
#[allow(clippy::too_many_arguments)]
fn qualifies(
    w: &mut World,
    c: CellXY,
    class: Option<LocomotionClass>,
    zone: MovementZone,
    reach_ref: Option<Reach>,
    deck_mode: bool,
    height_cond: bool,
    centre_level: i32,
) -> bool {
    // spec: S-0212 claim 25 (inside the usable area)
    if !w.cells.usable(c) {
        return false;
    }
    let deck = w.cells.idx(c).is_some_and(|i| w.cells.deck[i]);
    let level = w.cells.cell(c).map_or(0, |x| i32::from(x.level)) + if deck { 4 } else { 0 }; // spec: S-0207 claim 9 (a deck counts 4 levels above its cell)
    // spec: S-0212 claim 25 (height condition: within 1 level of the centre)
    if height_cond && (level - centre_level).abs() > 1 {
        return false;
    }
    if class == Some(LocomotionClass::Winged) {
        // spec: S-0212 claim 25 (winged: only the area, height and further tests apply)
        return true;
    }
    if let Some(rr) = reach_ref
        && regions::reach(w, c, zone) != rr
    {
        return false;
    }
    let r = w.rules_arc();
    // spec: S-0212 claim 25 (enterable by class, a bridge cell judged on its deck)
    if !crate::passability::enterable_by_class(&r, w, c, class, zone, deck && deck_mode) {
        return false;
    }
    // spec: S-0212 claim 25 (the occupancy record is empty as a whole)
    let layer = if deck { Layer::Deck } else { Layer::Ground };
    w.cells.occ(c, layer).is_some_and(|o| o.empty())
}

/// The cells of ring r in scan order (S-0212 claim 25).
fn ring_cells(centre: CellXY, r: i32, centre_twice: bool) -> Vec<CellXY> {
    if r == 0 {
        // spec: S-0212 claim 25 (ring 0 is the centre, which appears twice with quirks.nearby_centre_twice)
        return if centre_twice {
            vec![centre, centre]
        } else {
            vec![centre]
        };
    }
    let mut rows = Vec::new();
    let mut cols = Vec::new();
    for x in centre.x - r..=centre.x + r {
        for y in [centre.y - r, centre.y + r] {
            rows.push(CellXY::new(x, y));
        }
    }
    for y in centre.y - r + 1..=centre.y + r - 1 {
        for x in [centre.x - r, centre.x + r] {
            cols.push(CellXY::new(x, y));
        }
    }
    // spec: S-0212 claim 25 (row cells by x then y, before column cells by y then x)
    rows.sort_by_key(|c| (c.x, c.y));
    cols.sort_by_key(|c| (c.y, c.x));
    rows.extend(cols);
    rows
}

/// The congestion marks of S-0212 claim 19 for a re-path.
fn congestion(w: &World, id: ObjectId, mode: Mode) -> Option<BTreeSet<CellXY>> {
    let o = w.obj(id)?;
    let me = w.rules().unit(o.kind, o.type_index)?;
    let here = o.pos.cell();
    let d = glasnost_num::geom::eight_direction(o.body.value(w.tick));
    let (dx, dy) = STEPS[usize::from(d)];
    let ahead = CellXY::new(here.x + dx, here.y + dy);
    let mut counts: BTreeMap<CellXY, u32> = BTreeMap::new();
    let mut from_paths = 0;
    for &oid in w.cells.list(ahead, Layer::Ground) {
        let Some(u) = w.obj(oid) else { continue };
        if !matches!(u.kind, Kind::Vehicle | Kind::Infantry) {
            continue;
        }
        let ut = w.rules().unit(u.kind, u.type_index);
        // spec: S-0212 claim 19 (mode 1: another type, lower speed number, inside the usable area)
        let other_type = u.type_index != o.type_index || u.kind != o.kind;
        let slower = ut.is_some_and(|t| t.speed < me.speed);
        let ok_mode = match mode {
            // spec: S-0212 claim 19 (mode 2: any vehicle or infantry unit)
            Mode::Avoid => true,
            // spec: S-0212 claim 19 (mode 1: another type, lower speed number, inside the usable area)
            _ => other_type && slower && w.cells.usable(u.pos.cell()),
        };
        // spec: S-0212 claim 19 (a vehicle needs two stored steps, an infantry unit three)
        let need = if u.kind == Kind::Vehicle { 2 } else { 3 };
        if !ok_mode || u.path.steps.len() < need {
            continue;
        }
        // spec: S-0212 claim 19 (i: the cells its remaining stored steps lead through, after the cell it moves into)
        let mut c = heading_cell(w, oid).unwrap_or(u.pos.cell());
        for &s in u.path.steps.iter().take(STORED) {
            let (sx, sy) = STEPS[usize::from(s % 8)]; // spec: S-0212 Terms (step directions 0 to 7)
            c = CellXY::new(c.x + sx, c.y + sy);
            *counts.entry(c).or_insert(0) += 1;
            from_paths += 1;
        }
    }
    if mode == Mode::Blocked && from_paths == 0 {
        // spec: S-0212 claim 19 (mode 1 with no cell from (i): the search runs as mode 0)
        return None;
    }
    // spec: S-0212 claim 19 (ii: occupied cells of the 5 × 5 block around the cell ahead, but the unit's own)
    for yy in -2..=2 {
        for xx in -2..=2 {
            let c = CellXY::new(ahead.x + xx, ahead.y + yy);
            if c == here {
                continue;
            }
            if !w.cells.list(c, Layer::Ground).is_empty() {
                *counts.entry(c).or_insert(0) += 1;
            }
        }
    }
    // spec: S-0212 claim 19 (iii: the cell ahead itself)
    *counts.entry(ahead).or_insert(0) += 1;
    let toggle = w.rules().quirks.path_congestion_toggle;
    Some(
        counts
            .into_iter()
            // spec: S-0212 claim 19 (marked when exactly one of: marked before, odd count; switch off: count ≥ 1)
            .filter(|&(_, n)| if toggle { n % 2 == 1 } else { n >= 1 })
            .map(|(c, _)| c)
            .collect(),
    )
}

/// A path computation for a unit (S-0212 claims 6, 20, 24): the gate,
/// destination substitution, the search, flattening and storage.
pub fn compute(w: &mut World, id: ObjectId, mode: Mode) -> Search {
    let Some(o) = w.obj(id).cloned() else {
        return Search::SearchFailed;
    };
    let Some(mut goal) = goal_cell(w, id) else {
        return Search::SearchFailed;
    };
    let r = w.rules_arc();
    let Some(u) = r.unit(o.kind, o.type_index).cloned() else {
        return Search::SearchFailed;
    };
    let from = o.pos.cell();
    // spec: S-0212 claim 6 (the gate compares the reach regions; skipped for a unit outside the usable area)
    let inside = w.cells.usable(from);
    if inside {
        let a = regions::reach(w, from, u.zone);
        let b = regions::reach(w, goal, u.zone);
        if a != b {
            return Search::GateFailed;
        }
    }
    // spec: S-0212 claim 24 (destination substitution after the gate, not for trains)
    if !u.train {
        let reference = occupancy::reference_level(w, id);
        let out = match o.kind {
            Kind::Infantry => occupancy::infantry_outcome(w, id, goal, reference),
            _ => occupancy::vehicle_outcome(w, id, goal, reference),
        };
        let holds_building = w
            .cells
            .list(goal, Layer::Ground)
            .iter()
            .any(|&b| w.obj(b).is_some_and(|x| x.kind == Kind::Building));
        let du = {
            let c = Pos::centre(goal, 0);
            let dx = i64::from(o.pos.x - c.x);
            let dy = i64::from(o.pos.y - c.y);
            // spec: S-0212 F6a (to the destination centre at world height 0, quirks.substitution_distance_flat_destination)
            let dz = if r.quirks.substitution_distance_flat_destination {
                i64::from(o.pos.z)
            } else {
                i64::from(o.pos.z - w.surface_at(c.x, c.y))
            };
            root_floor(
                (dx * dx + dy * dy + dz * dz) as u64,
                r.quirks.table_root_distance,
                r.mode,
            ) as i64
        };
        if out == Outcome::StationaryFriendly && du > i64::from(r.general.close_enough) {
            if let Some(c) = nearby(w, goal, NearbyUse::Substitution { unit: id }) {
                let dx = i64::from(256 * (c.x - goal.x));
                let dy = i64::from(256 * (c.y - goal.y));
                // spec: S-0212 F6b (centre to centre); the estimate E needs the corridor (Q-0017)
                let d = root_floor(
                    (dx * dx + dy * dy) as u64,
                    r.quirks.table_root_distance,
                    r.mode,
                ) as i64;
                if d < du {
                    goal = c;
                }
            }
        } else if out == Outcome::Blocked && holds_building {
            // spec: S-0212 claim 24 (a blocked building cell: the substitute always; none found keeps it, departure D3)
            if let Some(c) = nearby(w, goal, NearbyUse::Substitution { unit: id }) {
                goal = c;
            }
        }
    }
    let congested = match mode {
        Mode::Fresh => BTreeSet::new(),
        _ => congestion(w, id, mode).unwrap_or_default(),
    };
    let mode = if mode == Mode::Blocked && congested.is_empty() {
        Mode::Fresh
    } else {
        mode
    };
    let req = Request {
        unit: id,
        from,
        to: goal,
        mode,
        congested: congested.clone(),
    };
    let found = match w.pathfinder {
        PathfinderChoice::S0212 => S0212.find(w, &req),
    };
    let Some(steps) = found else {
        return Search::SearchFailed;
    };
    let k = u.threat_avoidance;
    let ok = |path: &[u8], start: usize, d3: u8, j: usize| -> bool {
        // Cells the 2j new steps pass through.
        let mut c = from;
        for &s in &path[..start] {
            let (sx, sy) = STEPS[usize::from(s % 8)]; // spec: S-0212 Terms (step directions 0 to 7)
            c = CellXY::new(c.x + sx, c.y + sy);
        }
        let reference = occupancy::reference_level(w, id);
        let mut prev = c;
        // spec: S-0212 claim 17 (2j new steps)
        for _ in 0..2 * j {
            let (sx, sy) = STEPS[usize::from(d3)];
            let n = CellXY::new(prev.x + sx, prev.y + sy);
            // spec: S-0212 claim 17 (every cell clear, no congestion mark; threat × k below 1.0, no threat records in M3)
            if step_outcome(w, id, prev, n, reference) != Outcome::Clear || congested.contains(&n) {
                return false;
            }
            prev = n;
        }
        let _ = k;
        true
    };
    let steps = flatten(&steps, r.quirks.flatten_straight_run_v, &ok);
    if let Some(ob) = w.obj_mut(id) {
        ob.path.steps = steps.into_iter().take(STORED).collect();
        ob.path.for_goal = Some(goal);
    }
    Search::Found
}

/// What follows a failed path computation (S-0212 claims 21 and 23).
/// `repath`: the computation was a re-path for a blocked step.
pub fn failed(w: &mut World, id: ObjectId, res: Search, repath: bool) {
    let t = w.tick;
    let Some(o) = w.obj(id).cloned() else { return };
    let r = w.rules_arc();
    let walker = o.walk.is_some();
    let drive = o.drive.is_some();
    match res {
        Search::Found => {}
        Search::GateFailed => {
            // spec: S-0212 claim 23 (gate fails: no path; a driving unit's destination is cleared)
            if let Some(ob) = w.obj_mut(id) {
                ob.path.steps.clear();
                if drive {
                    ob.dest = None;
                }
                // spec: S-0212 claim 21 (the delay runs after a fresh search, and after a walker's re-path)
                if !repath || walker {
                    ob.path.retry_at = t.wrapping_add(r.general.retry_delay);
                }
            }
        }
        Search::SearchFailed => {
            let goal = goal_cell(w, id);
            let near = goal.is_some_and(|g| {
                let c = o.pos.cell();
                (g.x - c.x).abs().max((g.y - c.y).abs()) <= 1
            });
            if let Some(ob) = w.obj_mut(id) {
                ob.path.steps.clear();
                // spec: S-0212 claim 23 (the unit stops and the retry delay R starts)
                ob.path.retry_at = t.wrapping_add(r.general.retry_delay);
                if drive {
                    ob.dest = None;
                }
                if !near {
                    // spec: S-0212 claim 23 (more than 1 cell away: it gives up, its destination is cleared and it queues guard)
                    ob.dest = None;
                    ob.beh.queue(Some(Behaviour::Guard));
                }
            }
        }
    }
    let _ = Locomotor::None;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(steps: &[u8], sv: bool) -> Vec<u8> {
        flatten(steps, sv, &|_, _, _, _| true)
    }

    // spec: S-0212 claim 17 (worked examples a and b, any setting of quirks.flatten_straight_run_v)
    #[test]
    fn examples_a_b() {
        assert_eq!(flat(&[1, 3, 3, 3, 1], true), vec![2, 2, 3, 2, 2]);
        assert_eq!(flat(&[1, 3, 3, 5], true), vec![2, 2, 4, 4]);
        assert_eq!(flat(&[1, 3, 3, 3, 1], false), vec![2, 2, 3, 2, 2]);
        assert_eq!(flat(&[1, 3, 3, 5], false), vec![2, 2, 4, 4]);
    }

    // spec: S-0212 claim 17 (worked examples c to f with quirks.flatten_straight_run_v on, and off)
    #[test]
    fn examples_c_to_f() {
        assert_eq!(
            flat(&[1, 3, 1, 3, 4, 4, 4], true),
            vec![2, 2, 3, 3, 3, 3, 4]
        );
        assert_eq!(flat(&[1, 3, 2, 4, 4, 4], true), vec![3, 3, 3, 3, 3, 3]);
        assert_eq!(flat(&[5, 7, 0], true), vec![6, 0, 0]);
        assert_eq!(flat(&[7, 1, 6], true), vec![0, 0, 0]);
        assert_eq!(
            flat(&[1, 3, 1, 3, 4, 4, 4], false),
            vec![2, 2, 2, 2, 4, 4, 4]
        );
        assert_eq!(flat(&[1, 3, 2, 4, 4, 4], false), vec![2, 2, 2, 4, 4, 4]);
        assert_eq!(flat(&[5, 7, 0], false), vec![6, 6, 0]);
        assert_eq!(flat(&[7, 1, 6], false), vec![0, 0, 6]);
    }

    // spec: S-0212 claim 25 (scan order: row cells by x then y, then column cells by y then x; the centre twice)
    #[test]
    fn ring_order() {
        let c = CellXY::new(10, 10);
        assert_eq!(ring_cells(c, 0, true), vec![c, c]);
        let r1 = ring_cells(c, 1, true);
        let want = [
            (9, 9),
            (9, 11),
            (10, 9),
            (10, 11),
            (11, 9),
            (11, 11),
            (9, 10),
            (11, 10),
        ];
        assert_eq!(
            r1,
            want.iter()
                .map(|&(x, y)| CellXY::new(x, y))
                .collect::<Vec<_>>()
        );
    }

    // spec: S-0212 table T4 (direction terms are single-precision values)
    #[test]
    fn epsilons() {
        let e = epsilon(0);
        assert_eq!(e.to_f32(Rounding::TowardZero).to_f64(), e);
    }
}
