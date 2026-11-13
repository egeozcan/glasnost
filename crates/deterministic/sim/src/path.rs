//! Pathfinding (S-0212), behind the [`Pathfinder`] interface.
//!
//! [`S0212`] implements the specified parts of S-0212: the reachability
//! gate (claim 6), the cell search with its costs, estimate, recording and
//! limits and order among equal entries (claims 7 to 12a, F1, F2), a
//! blocked destination (claim 15), the flattening of right-angle turns
//! (claim 17), the shortening of early detours (claim 18), congestion marks
//! (claim 19), path storage and retry delays (claims 20, 21), failures
//! (claim 23), destination substitution (claim 24) and the nearby-cell
//! search (claim 25), over (cell, search layer) pairs. The corridor (claims
//! 13 to 14, the estimate E of claim 24) is in [`crate::corridor`]; the
//! searches of a path computation with it are `corridor_search`'s.

use std::collections::{BTreeMap, BTreeSet, BinaryHeap};

use glasnost_map::CellXY;
use glasnost_num::geom::{root_floor, table_root};
use glasnost_num::{F32, F64, Rounding};
use glasnost_rules::sim::{LocomotionClass, MovementZone};
use serde::{Deserialize, Serialize};

use crate::corridor;
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
    /// A walker's destination when its current blockage began
    /// (`blocked_since`). A blockage met under another destination starts
    /// the blockage delay afresh (S-0209 claim 10, r60), also when the
    /// destination was written without reaching the locomotor. An
    /// assignment of the same destination restarts it at the assignment
    /// itself (r76, [`destination_changed`]).
    #[serde(default)]
    pub blocked_for: Option<Target>,
    /// The destination the stored path was computed for.
    pub for_goal: Option<CellXY>,
    /// A walker's destination at the failed search that ended its movement
    /// in progress while it kept that destination (S-0212 claim 23, the
    /// sentence after its table; S-0417 claim 17, *No path*). Cleared when
    /// a destination reaches the locomotor or a path is found.
    #[serde(default)]
    pub failed_for: Option<Target>,
    /// The destination the unit held when its running retry delay started
    /// ([`start_delay`]). For the driving and walk locomotors the delay
    /// ends once the destination is no longer this one (S-0212 claim 21,
    /// [`waits`]); none: the delay is tied to no destination and runs
    /// until its tick.
    #[serde(default)]
    pub retry_for: Option<Target>,
    /// The unit gives way in a deadlock with no way round
    /// (`quirks.deadlock_no_yield` off; our design, C7 part 5); none
    /// otherwise.
    #[serde(default)]
    pub yielding: Option<Yield>,
}

/// A unit giving way in a deadlock with no way round (our design, C7 part
/// 5, yield): it is sent to a free cell off the other side's way, waits
/// there and then resumes its order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Yield {
    /// The destination of the order it resumes.
    pub resume: Target,
    /// The units it gives way to.
    pub to: Vec<ObjectId>,
    /// The tick it began to give way.
    pub since: i32,
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
    /// Cells this search does not enter: the cells of stalled moving units,
    /// with `quirks.path_through_stalled_blockers` off (our design, C7 part
    /// 4, [`stalled_cells`]); empty otherwise. A search with such cells is
    /// also held to [`DETOUR_EXPANSIONS`].
    pub avoided: BTreeSet<CellXY>,
}

/// A pathfinder: finds the steps from one cell to another for a unit. The
/// simulation calls it through [`compute`]; implementations must be
/// deterministic.
pub trait Pathfinder {
    /// The steps of a path, or `None` when the search fails.
    fn find(&self, w: &World, req: &Request) -> Option<Vec<u8>>;
}

/// The cell search of S-0212, over the whole map (a path computation adds
/// the corridor, [`compute`]).
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
fn base_cost(w: &World, req: &Request, v: Outcome, cell: CellXY, layer: Layer, train: bool) -> i64 {
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
            Mode::Fresh => blocker_chain(w, cell, layer),
        },
    }
}

/// Where a unit is heading (S-0212 claim 8).
fn heading_cell(w: &World, id: ObjectId) -> Option<CellXY> {
    let o = w.obj(id)?;
    let here = o.pos.cell();
    let speed = o.drive.as_ref().is_some_and(|d| d.track.is_some())
        || o.walk.as_ref().is_some_and(|s| s.target.is_some())
        // spec: S-0503 claim 14 (a hovering vehicle's current speed is not zero while ρ ≠ 0; Q-0691 answered)
        || crate::hover::has_speed(w, id);
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

/// The blocker chain of S-0212 claim 8 (mode 0 moving blocker cost), from
/// the step's target cell on the step's search layer.
fn blocker_chain(w: &World, first: CellXY, first_layer: Layer) -> i64 {
    let mut c = first;
    let mut layer = first_layer;
    // spec: S-0212 claim 8 (a chain of at most 10 cells)
    for i in 1..=10 {
        let Some(&oid) = w.cells.list(c, layer).first() else {
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
                // spec: S-0212 claim 8 (a further bridge cell is read on its deck when the unit heading into it is on a deck or stands at least 3 levels higher)
                let higher = level(w, o.pos.cell()) - level(w, next) >= 3;
                layer = if is_bridge(w, next) && (o.layer == Layer::Deck || higher) {
                    Layer::Deck
                } else {
                    Layer::Ground
                };
                c = next;
            }
        }
    }
    1
}

/// A cell's height level (0 off the map).
fn level(w: &World, c: CellXY) -> i32 {
    w.cells.cell(c).map_or(0, |x| i32::from(x.level))
}

/// Whether a cell is a bridge cell: it carries a high bridge's deck
/// (S-0212 Terms; S-0211 claim 15).
fn is_bridge(w: &World, c: CellXY) -> bool {
    w.cells.idx(c).is_some_and(|i| w.cells.deck[i])
}

/// The height level of a search pair: the cell's, plus 4 on a deck (S-0212
/// Terms, *search layer*).
fn pair_level(w: &World, c: CellXY, l: Layer) -> i32 {
    // spec: S-0212 Terms (the deck layer sits 4 height levels above the cell's height level)
    level(w, c) + if l == Layer::Deck { 4 } else { 0 }
}

/// The search layer a step onto `c` from a pair at level `from_level` is
/// on (S-0212 claim 7).
fn candidate_layer(w: &World, from_level: i32, c: CellXY) -> Layer {
    // spec: S-0212 claim 7 (onto a bridge cell: its ground when within 1 level of the pair being expanded, its deck otherwise)
    if is_bridge(w, c) && (from_level - level(w, c)).abs() > 1 {
        Layer::Deck
    } else {
        Layer::Ground
    }
}

/// Whether the height rule of S-0212 claim 10 allows a step from the pair
/// at cell `f` (its deck when `deck`) to the neighbouring cell `t`. The
/// rule compares `t`'s height level with `f`'s, which for a deck pair is
/// the level of the ground under the deck; the layer of `t` entered is
/// claim 7's ([`candidate_layer`]).
pub(crate) fn height_ok(w: &World, f: CellXY, deck: bool, t: CellXY) -> bool {
    let (Some(cf), Some(ct)) = (w.cells.cell(f), w.cells.cell(t)) else {
        return false;
    };
    // spec: S-0212 claim 10 (Δ = level(T) − level(F), F's ground level even when the pair is its deck)
    let delta = i32::from(ct.level) - i32::from(cf.level);
    let deck = deck && is_bridge(w, f);
    let marked = w.cells.deck_entry_at(t);
    match delta {
        // spec: S-0212 claim 10 (Δ = 0: a ground pair; from a deck pair only onto a bridge cell with the deck-entry mark)
        0 => !deck || marked,
        1 | -1 => {
            let lower = if delta > 0 { cf } else { ct };
            // spec: S-0212 claim 10 (Δ = ±1: the lower of the two cells is a slope, from a ground pair and from a deck pair alike, no mark needed)
            let slope = lower.slope != 0;
            // spec: S-0212 Edge cases (`quirks.deck_side_step_ground_level` off: from a deck pair, Δ = ±1 onto a cell that is not a bridge cell is refused)
            let side_step =
                !deck || is_bridge(w, t) || w.rules().quirks.deck_side_step_ground_level;
            slope && side_step
        }
        // spec: S-0212 claim 10 (Δ = +4: off a deck onto ground at deck level; nothing is asked of T)
        4 => deck,
        // spec: S-0212 claim 10 (Δ = −4: onto a bridge cell with the deck-entry mark, from either layer of F)
        -4 => marked,
        // spec: S-0212 claim 10 (2, 3, 5 or more levels, either sign: never)
        _ => false,
    }
}

/// The entry outcome a unit gets for stepping from `from` into `to`, with
/// the height rule (S-0212 claim 10; S-0209 claims 4 and 5). `reference`
/// is the height level the step starts at (the unit's reference height
/// level for a unit's own step, the expanded pair's level in a search): it
/// gives the search layers of both cells (S-0212 claim 7) and the cell
/// layer the entry outcome reads (S-0209 Terms).
pub fn step_outcome(w: &World, id: ObjectId, from: CellXY, to: CellXY, reference: i32) -> Outcome {
    occupancy::Mover::of(w, id).map_or(Outcome::Blocked, |m| {
        step_outcome_of(w, &m, from, to, reference)
    })
}

/// [`step_outcome`] for a mover read once (a search reads it once for all
/// its steps).
pub fn step_outcome_of(
    w: &World,
    m: &occupancy::Mover<'_>,
    from: CellXY,
    to: CellXY,
    reference: i32,
) -> Outcome {
    // spec: S-0212 Terms (a pair 4 levels above its bridge cell is on the deck)
    let from_deck = is_bridge(w, from) && reference - level(w, from) >= 4;
    // spec: S-0212 claim 7 (the height rule of claim 10 is part of the entry outcome)
    if !height_ok(w, from, from_deck, to) {
        return Outcome::Blocked;
    }
    // spec: S-0211 Terms (a unit inside the usable area may not step outside it)
    if w.cells.usable(from) && !w.cells.usable(to) {
        return Outcome::Blocked;
    }
    match m.o.kind {
        Kind::Infantry => occupancy::infantry_outcome_of(w, m, to, reference),
        // spec: S-0209 claim 14 (an aircraft gets clear everywhere in skirmish games)
        Kind::Aircraft => Outcome::Clear,
        _ => occupancy::vehicle_outcome_of(w, m, to, reference),
    }
}

/// Reusable working memory of the cell search, indexed by search pair
/// (twice the dense cell index, plus 1 for a deck).
/// Not game state: a search starts from a fresh stamp, so nothing carries
/// over from one search to the next.
///
/// It also keeps what one path computation with the corridor has asked of
/// the world: the claim 13 exemption and the estimate h by dense cell
/// index, the entry outcome of a step by search pair and direction (our
/// tooling for speed, M8 C4 part 5). A computation's searches all read the
/// one world it was asked on (the world does not change while it runs), so
/// an answer kept is the answer asked again. A computation starts from a
/// fresh stamp of its own, so nothing carries over from one computation to
/// the next; a loaded or copied game starts empty, and nothing here is
/// saved or hashed.
#[derive(Debug, Default)]
pub struct SearchScratch {
    stamp: u32,
    /// `seen[i] == stamp`: pair i has a recorded g.
    seen: Vec<u32>,
    /// `closed[i] == stamp`: pair i has been expanded.
    closed: Vec<u32>,
    g: Vec<F32>,
    /// The predecessor pair and the step direction from it.
    pred: Vec<(CellXY, Layer, u8)>,
    /// The current path computation's stamp; 0 before the first.
    computation: u32,
    /// Whether a path computation is running (its memory may be kept).
    open: bool,
    /// `exempt_at[i] == computation`: `exempt[i]` is cell i's
    /// [`corridor::exempt`] in this computation's world.
    exempt_at: Vec<u32>,
    exempt: Vec<bool>,
    /// `outcome_at[i] == computation`: `outcomes[i][d]` is the entry
    /// outcome of the step from pair i in direction d, once asked.
    outcome_at: Vec<u32>,
    // spec: S-0212 claim 7 (candidates in step directions 0 through 7)
    outcomes: Vec<[Option<Outcome>; 8]>,
    /// `estimate_at[i] == computation`: `estimates[i]` is cell i's
    /// estimate h towards the computation's destination.
    estimate_at: Vec<u32>,
    estimates: Vec<F32>,
}

/// A copy of the world starts with empty working memory: nothing in it is
/// read before a search or computation starts it afresh.
impl Clone for SearchScratch {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl SearchScratch {
    fn begin(&mut self, len: usize) {
        if self.seen.len() != len || self.stamp == u32::MAX {
            self.stamp = 0;
            self.seen = vec![0; len];
            self.closed = vec![0; len];
            self.g = vec![F32::ZERO; len];
            self.pred = vec![(CellXY::new(0, 0), Layer::Ground, 0); len];
        }
        self.stamp += 1;
    }

    /// Starts a path computation's memory for a map of `cells` cells: what
    /// earlier computations kept is forgotten.
    fn begin_computation(&mut self, cells: usize) {
        if self.exempt_at.len() != cells || self.computation == u32::MAX {
            self.computation = 0;
            self.exempt_at = vec![0; cells];
            self.exempt = vec![false; cells];
            // spec: S-0212 Terms (two search layers per cell: ground and deck)
            self.outcome_at = vec![0; 2 * cells];
            // spec: S-0212 Terms and claim 7 (two search layers per cell; step directions 0 through 7)
            self.outcomes = vec![[None; 8]; 2 * cells];
            self.estimate_at = vec![0; cells];
            self.estimates = vec![F32::ZERO; cells];
        }
        self.computation += 1;
        self.open = true;
    }

    /// Ends the current path computation: until the next begins, every
    /// question is asked afresh and nothing is kept.
    fn end_computation(&mut self) {
        self.open = false;
    }

    /// [`corridor::exempt`] for cell `c` of `w`, the world of the current
    /// path computation: asked once per cell and computation.
    fn exempt(&mut self, w: &World, c: CellXY) -> bool {
        let Some(i) = w.cells.idx(c).filter(|_| self.open) else {
            return corridor::exempt(w, c);
        };
        if self.exempt_at.get(i) == Some(&self.computation) {
            return self.exempt[i];
        }
        let v = corridor::exempt(w, c);
        if let Some(at) = self.exempt_at.get_mut(i) {
            *at = self.computation;
            self.exempt[i] = v;
        }
        v
    }

    /// The entry outcome of the step from search pair `pair` in direction
    /// `d`, in the world of the current path computation: `ask` is asked
    /// once per step and computation.
    fn outcome(&mut self, pair: usize, d: usize, ask: impl FnOnce() -> Outcome) -> Outcome {
        if !self.open {
            return ask();
        }
        let Some(at) = self.outcome_at.get_mut(pair) else {
            return ask();
        };
        if *at != self.computation {
            *at = self.computation;
            // spec: S-0212 claim 7 (step directions 0 through 7)
            self.outcomes[pair] = [None; 8];
        }
        *self.outcomes[pair][d].get_or_insert_with(ask)
    }

    /// The estimate h of cell `c` of `w` towards the current path
    /// computation's destination (all its searches share it): `ask` is
    /// asked once per cell and computation.
    fn estimate(&mut self, w: &World, c: CellXY, ask: impl FnOnce() -> F32) -> F32 {
        let Some(i) = w.cells.idx(c).filter(|_| self.open) else {
            return ask();
        };
        match self.estimate_at.get_mut(i) {
            Some(at) if *at == self.computation => self.estimates[i],
            Some(at) => {
                *at = self.computation;
                let h = ask();
                self.estimates[i] = h;
                h
            }
            None => ask(),
        }
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
    layer: Layer,
    /// The cell's estimate h, kept so that the stale test on taking the
    /// entry does not work it out again (the same value: h depends on the
    /// cell alone).
    h: F32,
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
        let mut progress = req.from;
        search(w, req, None, &mut progress)
    }
}

/// A corridor confining a cell search (S-0212 claim 13).
pub struct CorridorFilter<'a> {
    /// The level-0 sectors.
    pub level0: &'a corridor::SectorLevel,
    /// The level-0 corridor, from the start's sector to the destination's.
    pub chain: &'a [u32],
    /// Whether each level-0 sector is on the corridor.
    pub on: Vec<bool>,
}

impl<'a> CorridorFilter<'a> {
    /// The filter of a level-0 corridor.
    pub fn new(level0: &'a corridor::SectorLevel, chain: &'a [u32]) -> Self {
        let mut on = vec![false; level0.category.len()];
        for &s in chain {
            if let Some(x) = on.get_mut(s as usize) {
                *x = true;
            }
        }
        Self { level0, chain, on }
    }
}

/// The cell search of S-0212 claims 7 to 12 and 15, inside a corridor when
/// `corr` is given (claim 13). `progress` receives the search's progress
/// cell (claim 13). With the movement trace on, its size is noted.
fn search(
    w: &World,
    req: &Request,
    corr: Option<&CorridorFilter<'_>>,
    progress: &mut CellXY,
) -> Option<Vec<u8>> {
    let mut size = SearchSize::default();
    let found = counted_search(w, req, corr, progress, &mut size);
    crate::trace::cell_search(w, || crate::trace::CellSearch {
        corridor: corr.is_some(),
        expansions: size.expansions,
        recorded: size.recorded,
        found: found.as_ref().map(|p| p.len() as u32),
        cost_bits: found.as_ref().and(size.cost_bits),
    });
    found
}

/// The size of a cell search, for the movement trace.
#[derive(Default)]
struct SearchSize {
    expansions: u32,
    recorded: u32,
    cost_bits: Option<u32>,
}

/// [`search`], counting its size into `size`.
fn counted_search(
    w: &World,
    req: &Request,
    corr: Option<&CorridorFilter<'_>>,
    progress: &mut CellXY,
    size: &mut SearchSize,
) -> Option<Vec<u8>> {
    {
        let r = w.rules();
        let q = r.quirks;
        let mode = r.mode;
        let o = w.obj(req.unit)?;
        let u = r.unit(o.kind, o.type_index)?;
        // The mover, read once for every step outcome of the search.
        let mover = occupancy::Mover::of(w, req.unit)?;
        let train = u.train;
        let passive = u.passive && o.kind == Kind::Vehicle;
        let from = req.from;
        let to = req.to;
        // spec: S-0212 claim 12 (a found path must contain at least one step)
        if from == to {
            return None;
        }
        let est = |c: CellXY| -> F32 {
            let dx = i64::from(c.x - to.x);
            let dy = i64::from(c.y - to.y);
            // spec: S-0212 F2 (h = r((x − xₜ)² + (y − yₜ)²) in cells, the table root whatever the switch; the search layer does not enter h)
            table_root((dx * dx + dy * dy) as u64, mode)
        };
        let mut sc = w.scratch.borrow_mut();
        // spec: S-0212 Terms (a bridge cell is two places for the search: its ground and its deck)
        sc.begin(2 * w.cells.grid.len());
        // spec: S-0212 Terms (two search layers per cell: ground and deck)
        let ix = |c: CellXY, l: Layer| {
            w.cells
                .idx(c)
                .map(|i| 2 * i + usize::from(l == Layer::Deck))
        };
        let (from_layer, start_level) = start_pair(w, req.unit, from);
        // spec: S-0212 claim 7 (the destination pair: a bridge cell's deck for every unit that is not an aircraft, else the ground)
        let to_layer = if is_bridge(w, to) && o.kind != Kind::Aircraft {
            Layer::Deck
        } else {
            Layer::Ground
        };
        let to_level = pair_level(w, to, to_layer);
        let fi = ix(from, from_layer)?;
        let mut heap: BinaryHeap<Pending> = BinaryHeap::new();
        let mut seq = 0u32;
        let stamp = sc.stamp;
        sc.seen[fi] = stamp;
        sc.g[fi] = F32::ZERO;
        // spec: S-0212 F1 (recorded-pair margin μ = 1.009, the double nearest)
        let mu = crate::consts::PATH_MARGIN;
        // The step cost of F1 below for b × τ = 1 (a clear cell without a
        // congestion mark, the usual step), worked out once per direction:
        // the same arithmetic, asked once instead of once per step.
        // spec: S-0212 F1 (c = trunc₂₄(b × τ + ε(d)), here for b × τ = 1)
        let unit_cost: [F32; 8] =
            std::array::from_fn(|d| t24(F64::from_i64(1, mode).add(epsilon(d), mode), mode));
        // spec: S-0212 claim 12a (the start pair is recorded first)
        let h_from = est(from);
        heap.push(Pending {
            f: t24(h_from.to_f64(), mode).0,
            seq,
            cell: from,
            layer: from_layer,
            h: h_from,
        });
        let expansions = &mut size.expansions;
        let recorded = &mut size.recorded;
        // spec: S-0212 claim 13 (progress: e₀ is the start cell)
        *progress = from;
        let mut next_sector: usize = 1;
        // spec: S-0212 claim 16 (a train may not enter the three cells behind it, on the ground layer unless the start's search layer lies at least 2 levels above the cell)
        let behind: Vec<(CellXY, Layer)> = if train {
            behind_cells(w, req.unit, from)
                .into_iter()
                .map(|c| {
                    let l = if is_bridge(w, c) && start_level - level(w, c) >= 2 {
                        Layer::Deck
                    } else {
                        Layer::Ground
                    };
                    (c, l)
                })
                .collect()
        } else {
            Vec::new()
        };
        loop {
            // spec: S-0212 claims 12 and 12a (the smallest f; among equal f the pair recorded first)
            let next = heap.pop()?;
            let Some(ni) = ix(next.cell, next.layer) else {
                continue;
            };
            if sc.closed[ni] == stamp {
                continue;
            }
            if !q.path_no_reopen {
                // A stale entry superseded by a cheaper record.
                let cur = sc.g(ni).unwrap_or(F32::ZERO);
                if t24(cur.to_f64().add(next.h.to_f64(), mode), mode).0 != next.f {
                    continue;
                }
            }
            if next.cell == to && next.layer == to_layer {
                // spec: S-0212 claim 12 (reaching the destination after exactly 10000 expansions fails)
                if q.path_search_10000_failure && *expansions == 10000 {
                    return None;
                }
                size.cost_bits = sc.g(ni).map(|g| g.0);
                return Some(reconstruct(&sc, &ix, (from, from_layer), (to, to_layer)));
            }
            // spec: S-0212 claim 12 (at most 65527 expansions)
            if *expansions >= 65527 {
                return None;
            }
            // design: C7-P4 stalled blockers (a search round stalled units looks near: a bounded detour)
            if !req.avoided.is_empty() && *expansions >= DETOUR_EXPANSIONS {
                return None;
            }
            *expansions += 1;
            sc.closed[ni] = stamp;
            let ge = sc.g(ni).unwrap_or(F32::ZERO);
            let here_level = if (next.cell, next.layer) == (from, from_layer) {
                start_level
            } else {
                pair_level(w, next.cell, next.layer)
            };
            for (d, (dx, dy)) in STEPS.iter().enumerate() {
                let c = CellXY::new(next.cell.x + dx, next.cell.y + dy);
                let cl = candidate_layer(w, here_level, c);
                let Some(ci) = ix(c, cl) else { continue };
                // spec: S-0212 claim 11 (with quirks.path_no_reopen a recorded pair is never recorded again)
                // Such a candidate leads nowhere below unless it is the
                // destination (claim 15's blocked destination), so it is
                // passed over before its outcome is asked: the same search
                // with less work.
                if q.path_no_reopen && c != to && sc.seen[ci] == stamp {
                    continue;
                }
                if w.cells.cell(c).is_none() || behind.contains(&(c, cl)) {
                    continue;
                }
                // design: C7-P4 stalled blockers (a stalled unit's cell is not entered, the destination aside)
                if c != to && req.avoided.contains(&c) {
                    continue;
                }
                // A candidate outside the corridor that is not the
                // destination leads nowhere below whatever its outcome, so
                // it is passed over before its outcome is asked: the same
                // search with less work. The destination's outcome comes
                // first (claim 15's blocked destination).
                if c != to && outside_corridor(w, corr, &mut sc, c, cl) {
                    continue;
                }
                // spec: S-0209 Terms (reference height level in a path search: the expanded pair's, plus 4 on its deck layer, with the start-pair adjustment of S-0212 claim 7; the step starts from that pair's cell)
                // The step from a pair in a direction has one outcome in the
                // computation's one world (the start pair's level is the
                // computation's too): asked once for all its searches.
                let v = sc.outcome(ni, d, || {
                    step_outcome_of(w, &mover, next.cell, c, here_level)
                });
                if v == Outcome::Blocked {
                    // spec: S-0212 claim 15 (a blocked destination is approached from a neighbour whose search layer is within 1 level of the destination's; not for passive vehicles)
                    if c == to && !passive && (here_level - to_level).abs() <= 1 {
                        if next.cell == from {
                            // spec: S-0212 claim 15 (already next to it: a path needs a step, claim 12, so the search fails)
                            return None;
                        }
                        let margin_ok = match ix(to, to_layer).and_then(|ti| sc.g(ti)) {
                            None => true,
                            Some(gr) => {
                                // spec: S-0212 F1 (recorded-pair margin: g_r ≥ trunc₅₃(g_e + 1.009))
                                !q.path_no_reopen
                                    || gr.to_f64().ge(ge.to_f64().add(mu, Rounding::TowardZero))
                            }
                        };
                        if margin_ok {
                            if *expansions == 10000 && q.path_search_10000_failure {
                                return None;
                            }
                            size.cost_bits = Some(ge.0);
                            return Some(reconstruct(
                                &sc,
                                &ix,
                                (from, from_layer),
                                (next.cell, next.layer),
                            ));
                        }
                    }
                    continue;
                }
                if c == to && outside_corridor(w, corr, &mut sc, c, cl) {
                    continue;
                }
                // spec: S-0212 F1 (c = trunc₂₄(b × τ + ε(d)); τ = 4 on a congestion mark)
                let tau = if req.congested.contains(&c) { 4 } else { 1 };
                let b = base_cost(w, req, v, c, cl, train) * tau;
                let cost = if b == 1 {
                    unit_cost[d]
                } else {
                    t24(F64::from_i64(b, mode).add(epsilon(d), mode), mode)
                };
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
                    *recorded += 1;
                    // spec: S-0212 claim 12 (departure D2: at most 65535 recorded pairs besides the start)
                    if *recorded > 65535 {
                        return None;
                    }
                }
                if better {
                    sc.closed[ci] = 0;
                }
                sc.seen[ci] = stamp;
                sc.g[ci] = gn;
                sc.pred[ci] = (next.cell, next.layer, d as u8);
                seq += 1;
                // spec: S-0212 F2 (f = trunc₂₄(g + h))
                let h = sc.estimate(w, c, || est(c));
                let f = t24(gn.to_f64().add(h.to_f64(), mode), mode).0;
                // spec: S-0212 claim 12a (pairs recorded in expansion order, an expansion's candidates in generation order)
                heap.push(Pending {
                    f,
                    seq,
                    cell: c,
                    layer: cl,
                    h,
                });
                // spec: S-0212 claim 13 (progress: the first recorded candidate in the next corridor sector)
                if let Some(cf) = corr
                    && fresh
                    && cf.chain.get(next_sector).copied() == Some(cf.level0.sector(w, c))
                {
                    next_sector += 1;
                    *progress = c;
                }
            }
        }
    }
}

/// Whether candidate `c` on search layer `cl` lies outside the corridor
/// `corr` (S-0212 claims 7 and 13). Without a corridor nothing is outside.
fn outside_corridor(
    w: &World,
    corr: Option<&CorridorFilter<'_>>,
    sc: &mut SearchScratch,
    c: CellXY,
    cl: Layer,
) -> bool {
    let Some(cf) = corr else { return false };
    // spec: S-0212 claim 7 (with a corridor, a ground-layer candidate only in a corridor sector or with the exemption; deck candidates are not restricted)
    if cl != Layer::Ground {
        return false;
    }
    let sec = cf.level0.sector(w, c);
    let on = cf.on.get(sec as usize).copied().unwrap_or(false);
    !on && !sc.exempt(w, c)
}

/// The start pair of a search from `from` (S-0212 claim 7): its search layer
/// and height level. The start pair is on the deck when the unit is on a
/// bridge deck and is not an aircraft; a train standing on a bridge cell
/// whose world height lies more than 2 levels from that pair's level starts
/// 4 levels higher.
pub fn start_pair(w: &World, id: ObjectId, from: CellXY) -> (Layer, i32) {
    let Some(o) = w.obj(id) else {
        return (Layer::Ground, level(w, from));
    };
    // spec: S-0212 claim 7 (the start pair is on the deck when the unit is on a bridge deck and is not an aircraft)
    let mut layer = if o.layer == Layer::Deck && is_bridge(w, from) && o.kind != Kind::Aircraft {
        Layer::Deck
    } else {
        Layer::Ground
    };
    let h0 = pair_level(w, from, layer);
    let train = w
        .rules()
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.train);
    // spec: S-0212 claim 7 (a train on a bridge cell: H = H₀ + 4 when |trunc(z / 104) − H₀| > 2; 104 leptons per height level)
    if train && is_bridge(w, from) && (o.pos.z / 104 - h0).abs() > 2 {
        layer = Layer::Deck;
        return (layer, h0 + 4);
    }
    (layer, h0)
}

/// The three neighbours of `from` behind a unit's body facing (S-0212
/// claim 16).
fn behind_cells(w: &World, id: ObjectId, from: CellXY) -> Vec<CellXY> {
    let Some(o) = w.obj(id) else {
        return Vec::new();
    };
    let f8 = i32::from(glasnost_num::geom::eight_direction(o.body.value(w.tick)));
    // spec: S-0212 claim 16 (the step directions d with (d − f₈) mod 8 equal to 3, 4 or 5)
    (0..8)
        .filter(|d| matches!((d - f8).rem_euclid(8), 3..=5))
        .map(|d| {
            let (dx, dy) = STEPS[d as usize];
            CellXY::new(from.x + dx, from.y + dy)
        })
        .collect()
}

fn reconstruct(
    sc: &SearchScratch,
    ix: &dyn Fn(CellXY, Layer) -> Option<usize>,
    from: (CellXY, Layer),
    to: (CellXY, Layer),
) -> Vec<u8> {
    let mut steps = Vec::new();
    let mut c = to;
    while c != from {
        let Some(i) = ix(c.0, c.1).filter(|&i| sc.seen[i] == sc.stamp) else {
            break;
        };
        let (p, pl, d) = sc.pred[i];
        steps.push(d);
        c = (p, pl);
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

/// One step of a found path as the detour pass of S-0212 claim 18 reads
/// it: a step in a step direction, or a tunnel step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathStep {
    /// A step in step direction 0 to 7.
    Dir(u8),
    /// A tunnel step from a tunnel entrance to its exit (S-0212 claim 9).
    Tunnel {
        /// The tunnel entrance.
        entrance: CellXY,
        /// The tunnel exit.
        exit: CellXY,
    },
}

/// The acceptance test of a shape in [`shorten`]: `ok(s, shape, limit)`
/// tells whether the steps `shape`, taken from cell `s`, enter only
/// acceptable cells with at most `limit` threatened ones.
pub type ShapeOk<'a> = dyn Fn(CellXY, &[u8], u32) -> bool + 'a;

fn step_of(d: u8) -> (i32, i32) {
    STEPS[usize::from(d % 8)] // spec: S-0212 Terms (step directions 0 to 7)
}

fn cheb(a: CellXY, b: CellXY) -> i32 {
    // spec: S-0212 Terms (Chebyshev distance)
    (a.x - b.x).abs().max((a.y - b.y).abs())
}

fn dir_of(dx: i32, dy: i32) -> u8 {
    STEPS
        .iter()
        .position(|&s| s == (dx.signum(), dy.signum()))
        .map_or(0, |d| d as u8)
}

/// The two shapes of S-0212 claim 18 from `s` to `e`, in the order they are
/// tried: diagonal-first (not when the ends lie on one straight line), then
/// straight-first (not when they lie on one diagonal line).
fn shapes(s: CellXY, e: CellXY) -> Vec<Vec<u8>> {
    let (dx, dy) = (e.x - s.x, e.y - s.y);
    let diag = dx.abs().min(dy.abs()) as usize;
    let straight = (dx.abs().max(dy.abs()) as usize) - diag;
    let dd = dir_of(dx, dy);
    let sd = if dx.abs() > dy.abs() {
        dir_of(dx, 0)
    } else {
        dir_of(0, dy)
    };
    let mut v = Vec::new();
    // spec: S-0212 claim 18 (diagonal-first: all diagonal steps, then all straight ones; not tried on a straight line)
    if diag > 0 {
        let mut a = vec![dd; diag];
        a.extend(std::iter::repeat_n(sd, straight));
        v.push(a);
    }
    // spec: S-0212 claim 18 (straight-first: all straight steps, then all diagonal ones; not tried on a diagonal line)
    if straight > 0 {
        let mut b = vec![sd; straight];
        b.extend(std::iter::repeat_n(dd, diag));
        v.push(b);
    }
    v
}

/// Replaces the stretch of `cur` from the earliest place `earliest` to the
/// place `last`, ending on cell `e` (S-0212 claim 18, *the stretch of a
/// trigger*). `starts[i]` is the cell where the step at place i of `cur`
/// starts.
fn replace_stretch(
    cur: &mut [Option<PathStep>],
    starts: &[CellXY],
    earliest: usize,
    last: usize,
    e: CellXY,
    limit: u32,
    ok: &ShapeOk<'_>,
) {
    // The steps that are not removed, σ₁ … σₜ, and their distances dᵢ from e.
    let sig: Vec<usize> = (earliest..=last).filter(|&i| cur[i].is_some()).collect();
    if sig.is_empty() {
        // spec: S-0212 claim 18 (every place removed: from e to e, the stretch stays as it is)
        return;
    }
    let d: Vec<i32> = sig.iter().map(|&i| cheb(e, starts[i])).collect();
    // spec: S-0212 claim 18 (σᵢ is a new maximum when dᵢ exceeds every later dⱼ)
    let new_max: Vec<bool> = (0..d.len())
        .map(|i| d[i + 1..].iter().all(|&dj| d[i] > dj))
        .collect();
    let u = (0..d.len()).rev().find(|&i| !new_max[i]);
    // spec: S-0212 claim 18 (with u the last that is not a new maximum and v the last new maximum before it, the stretch begins after σᵥ; else at the earliest place)
    let begin = match u.and_then(|u| (0..u).rev().find(|&i| new_max[i])) {
        Some(v) => sig[v] + 1,
        None => earliest,
    };
    let Some(first) = (begin..=last).find(|&i| cur[i].is_some()) else {
        return;
    };
    let s = starts[first];
    if s == e {
        // spec: S-0212 claim 18 (s and e the same cell: the stretch stays as it is)
        return;
    }
    // spec: S-0212 claim 18 (the diagonal-first shape when acceptable, else the straight-first one; neither: unchanged)
    let Some(shape) = shapes(s, e).into_iter().find(|sh| ok(s, sh, limit)) else {
        return;
    };
    // spec: S-0212 claim 18 (the new steps take the first places of the stretch, the rest become removed steps)
    for (k, place) in (begin..=last).enumerate() {
        cur[place] = shape.get(k).map(|&d| PathStep::Dir(d));
    }
}

/// The cell where each step of `cur` starts, walking from `start`, and last
/// the cell where the path ends; after a tunnel step the walk goes on from
/// `after_tunnel(entrance, exit)`.
fn start_cells(
    cur: &[Option<PathStep>],
    start: CellXY,
    after_tunnel: &dyn Fn(CellXY, CellXY) -> CellXY,
) -> Vec<CellXY> {
    let mut c = start;
    let mut v = Vec::with_capacity(cur.len());
    for st in cur {
        v.push(c);
        match st {
            Some(PathStep::Dir(d)) => {
                let (dx, dy) = step_of(*d);
                c = CellXY::new(c.x + dx, c.y + dy);
            }
            Some(PathStep::Tunnel { entrance, exit }) => c = after_tunnel(*entrance, *exit),
            None => {}
        }
    }
    v.push(c);
    v
}

/// The shortening of early detours (S-0212 claim 18), on the path as claim
/// 17 has left it, from cell `start`. `mirrored` is
/// `quirks.detour_mirrored_reference`, `tunnel_north` is
/// `quirks.detour_after_tunnel_position`.
pub fn shorten(
    steps: &[PathStep],
    start: CellXY,
    mirrored: bool,
    tunnel_north: bool,
    ok: &ShapeOk<'_>,
) -> Vec<PathStep> {
    // spec: S-0212 claim 18 (after a tunnel step: as on a new path from the cell north of the entrance, or from the exit with the switch off)
    let after_tunnel = |entrance: CellXY, exit: CellXY| {
        if tunnel_north {
            CellXY::new(entrance.x, entrance.y - 1)
        } else {
            exit
        }
    };
    // spec: S-0212 claim 18 (the first 20 places are examined)
    let n = steps.len().min(20);
    // The path as received decides the legs, references and triggers.
    let recv: Vec<Option<PathStep>> = steps.iter().copied().map(Some).collect();
    let rstarts = start_cells(&recv, start, &after_tunnel);
    let mut cur = recv.clone();
    // State of the pass, reset after a tunnel step.
    let mut seg_start: usize = 0; // place where the current fresh path begins
    let mut leg_first: Vec<usize> = vec![0]; // the first place of each leg
    let mut leg_cell: Vec<CellXY> = vec![start]; // the first cell of each leg
    let mut record = 0i32;
    let mut turned = false;
    let mut q_ref: Option<(CellXY, CellXY)> = None; // (q, p) from the third leg on
    for i in 0..n {
        let PathStep::Dir(dir) = steps[i] else {
            // spec: S-0212 claim 18 (a tunnel step ends every leg and with it the reference distances and records; never part of a stretch)
            seg_start = i + 1;
            let c = rstarts.get(i + 1).copied().unwrap_or(start);
            leg_first = vec![i + 1];
            leg_cell = vec![c];
            record = 0;
            turned = false;
            q_ref = None;
            continue;
        };
        let s = rstarts[i];
        let (dx, dy) = step_of(dir);
        let e = CellXY::new(s.x + dx, s.y + dy);
        // spec: S-0212 claim 18 (a step turns back when it shrinks |x| or |y| of its leg's displacement so far)
        let lc = *leg_cell.last().unwrap_or(&start);
        let (px, py) = (s.x - lc.x, s.y - lc.y);
        let back = (px + dx).abs() < px.abs() || (py + dy).abs() < py.abs();
        if back {
            turned = true;
            leg_first.push(i);
            leg_cell.push(s);
            // spec: S-0212 claim 18 (from the third leg: q the previous leg's first cell; a step's record is the largest of Chebyshev(q, p) and the reference distances of the leg's earlier examined steps)
            if leg_first.len() >= 3 {
                let q = leg_cell[leg_cell.len() - 2];
                q_ref = Some((q, s));
                record = cheb(q, s);
            }
        }
        // spec: S-0212 claim 18 (the reference distance: from the start in the first two legs; from q to the end cell mirrored through p later, quirks.detour_mirrored_reference)
        let reference = match q_ref {
            None => cheb(start_of_seg(&rstarts, seg_start, start), e),
            Some((q, p)) if mirrored => cheb(q, CellXY::new(2 * p.x - e.x, 2 * p.y - e.y)),
            Some((q, _)) => cheb(q, e),
        };
        if reference > record {
            // spec: S-0212 claim 18 (in the first two legs a step's record is the largest of 0 and the earlier steps' reference distances: kept as a running maximum; a step above its record does not trigger)
            record = reference;
            continue;
        }
        // spec: S-0212 claim 18 (the earliest place: the path's first in the first two legs, else the previous leg's first step)
        let earliest = if leg_first.len() >= 3 {
            leg_first[leg_first.len() - 2]
        } else {
            seg_start
        };
        let cstarts = start_cells(&cur, start, &after_tunnel);
        // spec: S-0212 claim 18 (a trigger: the stretch ending on the trigger step's end cell, threat limit 0)
        replace_stretch(&mut cur, &cstarts, earliest, i, e, 0, ok);
    }
    // spec: S-0212 claim 18 (the closing stretch, when a step turned back)
    if turned && n > 0 && n > seg_start {
        let p = *leg_cell.last().unwrap_or(&start);
        let lf = *leg_first.last().unwrap_or(&0);
        let last = n - 1;
        let e = rstarts.get(n).copied().unwrap_or(start);
        let m = (n - lf) as i32;
        // spec: S-0212 claim 18 (replaced when Chebyshev(p, e) < m − 1, from the last leg's first step, threat limit 3)
        if cheb(p, e) < m - 1 {
            let cstarts = start_cells(&cur, start, &after_tunnel);
            replace_stretch(&mut cur, &cstarts, lf, last, e, 3, ok);
        }
    }
    // spec: S-0212 claim 18 (removed steps are dropped when the pass ends)
    cur.into_iter().flatten().collect()
}

fn start_of_seg(starts: &[CellXY], seg: usize, start: CellXY) -> CellXY {
    starts.get(seg).copied().unwrap_or(start)
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
        // spec: S-0314 claim 18 (a vehicle in depot contact heads for the docking point, not the building's position)
        Target::Object(t) => {
            // spec: S-0606 claim 4 (a vehicle approaching a bunker heads for the docking point)
            crate::depot::heading_point(w, id, t)
                .or_else(|| crate::structures::bunker::heading_point(w, id, t))
                .or_else(|| w.obj(t).map(|x| x.pos))
        }
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
    /// A move or no-move order onto a cell or an object (S-0214 claim 11).
    MoveOrder {
        /// The unit.
        unit: ObjectId,
        /// The click was on an object (its cell is the centre).
        on_object: bool,
        /// A no-move order by a free mover (S-0214 Terms: a unit in the air,
        /// of movement zone subterranean, of a jumpjet infantry type, or an
        /// aircraft).
        free_no_move: bool,
    },
    /// The failed approach-position search of S-0307 claim 17.
    Approach {
        /// The actor.
        unit: ObjectId,
    },
    /// A use whose spec gives every setting itself (S-0402 claims 9, 13
    /// and 18).
    Settings(NearbySettings),
}

/// The settings of a nearby-cell search (S-0212 claim 25) that its user
/// gives in full.
#[derive(Clone, Copy, Debug)]
pub struct NearbySettings {
    /// The locomotion class.
    pub class: Option<LocomotionClass>,
    /// The movement zone.
    pub zone: MovementZone,
    /// The reach region the cells must lie in, if any.
    pub reach: Option<Reach>,
    /// Deck mode.
    pub deck: bool,
    /// The height condition.
    pub height: bool,
    /// The reference cell, if any.
    pub reference: Option<CellXY>,
    /// Bridge cells never qualify.
    pub no_bridges: bool,
    /// The further test: the cell holds no overlay of any kind.
    pub no_overlay: bool,
}

impl NearbySettings {
    /// The settings that S-0212 table T5 gives the idle harvester step-off
    /// (S-0402 claim 13) and the waiting absorber approach (S-0605 claim
    /// 4) for unit `id` around `centre`: the unit's own locomotion class,
    /// *winged* counted as *track*; its type's own movement zone; the reach
    /// region of the centre cell for that zone, read for its deck when the
    /// unit is on a bridge deck; deck mode then; no height condition, no
    /// further test, no reference cell, bridge cells allowed.
    /// The regions must be up to date ([`regions::ensure`]).
    pub fn own_class(w: &World, id: ObjectId, centre: CellXY) -> Option<NearbySettings> {
        let o = w.obj(id)?;
        let u = w.rules().unit(o.kind, o.type_index)?;
        let deck = o.layer == Layer::Deck;
        Some(NearbySettings {
            // spec: S-0212 table T5 (step-off and waiting absorber approach rows: the unit's own locomotion class, winged counted as track)
            class: u.class.map(|k| match k {
                LocomotionClass::Winged => LocomotionClass::Track,
                other => other,
            }),
            // spec: S-0212 table T5 (its own movement zone; the reach region of the centre cell for that zone, read for its deck in deck mode)
            zone: u.zone,
            reach: Some(regions::reach_read(w, centre, u.zone, deck)),
            // spec: S-0212 table T5 (deck mode when the unit is on a bridge deck; no height condition; no further test; no reference cell)
            deck,
            height: false,
            reference: None,
            no_bridges: false,
            no_overlay: false,
        })
    }
}

/// The nearby-cell search (S-0212 claim 25).
/// The regions must be up to date ([`regions::ensure`]).
pub fn nearby(w: &World, centre: CellXY, use_: NearbyUse) -> Option<CellXY> {
    nearby_filtered(w, centre, use_, None)
}

/// A cell test of a nearby-cell search's user.
pub type CellTest<'a> = &'a dyn Fn(&World, CellXY) -> bool;

/// The nearby-cell search (S-0212 claim 25) with a user's own extra test
/// `extra` that a cell must pass to be collected, besides the search's
/// own (S-0402 claim 24's footprint test of a slave-miner site).
pub fn nearby_filtered(
    w: &World,
    centre: CellXY,
    use_: NearbyUse,
    extra: Option<CellTest<'_>>,
) -> Option<CellXY> {
    let q = w.rules().quirks;
    let fp = *w.cells.footprint();
    // spec: S-0212 claim 25 (largest radius 31, or w + h − 1 when w + h < 32)
    let max_r = if fp.width + fp.height < 32 {
        fp.width + fp.height - 1
    } else {
        31
    };
    let is_deck = |c: CellXY| w.cells.idx(c).is_some_and(|i| w.cells.deck[i]);
    let (class, zone, reach_ref, deck_mode, height_cond, further, reference, centre_deck) =
        match use_ {
            NearbyUse::Substitution { unit } => {
                let o = w.obj(unit)?;
                let u = w.rules().unit(o.kind, o.type_index)?.clone();
                let z = simplified(u.zone);
                // spec: S-0212 table T5 (substitution: in deck mode when the unit is on a bridge deck)
                let deck = o.layer == Layer::Deck;
                // spec: S-0212 table T5 (the reach region of the unit's cell, read for its deck when the unit is on a bridge deck, claim 4)
                let rr = regions::reach_read(w, o.pos.cell(), z, deck);
                // spec: S-0212 claim 24 (quirks.substitution_centre_ground_level off: the centre's level includes the deck whenever it is a bridge cell, for a unit that is not an aircraft)
                let centre_deck =
                    deck || (!q.substitution_centre_ground_level && o.kind != Kind::Aircraft);
                // spec: S-0212 claim 25 (claim 24 asks for the further test for a subterranean unit whose destination passes it)
                let further = u.zone == MovementZone::Subterranean && burrow_ok(w, centre);
                (
                    u.class,
                    z,
                    Some(rr),
                    deck,
                    true,
                    further,
                    Some(o.pos.cell()),
                    centre_deck,
                )
            }
            NearbyUse::Scatter { class, deck } => {
                // spec: S-0212 table T5 (scatter: zone normal, no reach region, no further test, no reference cell)
                (
                    class,
                    MovementZone::Normal,
                    None,
                    deck,
                    true,
                    false,
                    None,
                    deck,
                )
            }
            NearbyUse::MoveOrder {
                unit,
                on_object,
                free_no_move,
            } => {
                let o = w.obj(unit)?;
                let u = w.rules().unit(o.kind, o.type_index)?.clone();
                // spec: S-0212 table T5 (orders: own zone, subterranean as normal; fly as infantry for a click on an object, and for a click on a cell only under an ion storm, which no topic of M3 starts)
                let z = match u.zone {
                    MovementZone::Subterranean => MovementZone::Normal,
                    MovementZone::Fly if on_object => MovementZone::Infantry,
                    other => other,
                };
                let on_deck = o.layer == Layer::Deck;
                // spec: S-0212 table T5 (the no-move rows: no reach region; the others: that of the unit's cell, read for its deck when the unit is on a bridge deck, for that zone)
                let rr = (!free_no_move).then(|| regions::reach_read(w, o.pos.cell(), z, on_deck));
                // spec: S-0212 table T5 (orders: in deck mode when the centre is a bridge cell)
                let deck = is_deck(centre);
                // spec: S-0212 claim 25 (the no-move rows ask for the further test when the unit is of zone subterranean and below the ground surface: its world height below the ground's at its position, a deck's height counted when it is on one)
                let ground = w.surface_at(o.pos.x, o.pos.y) + if on_deck { 416 } else { 0 }; // spec: S-0207 claim 5 (a bridge deck's height: H + 416)
                let further =
                    free_no_move && u.zone == MovementZone::Subterranean && o.pos.z < ground;
                (u.class, z, rr, deck, true, further, None, deck)
            }
            NearbyUse::Approach { unit } => {
                let o = w.obj(unit)?;
                let u = w.rules().unit(o.kind, o.type_index)?.clone();
                // spec: S-0307 claim 17 (its locomotion class, its own movement zone, its current ground reach region, deck mode off, the height condition on, no further test, no reference cell)
                let rr = regions::reach_read(w, o.pos.cell(), u.zone, false);
                (u.class, u.zone, Some(rr), false, true, false, None, false)
            }
            NearbyUse::Settings(x) => (
                x.class,
                x.zone,
                x.reach,
                x.deck,
                x.height,
                x.no_overlay,
                x.reference,
                x.deck,
            ),
        };
    let centre_level = w.cells.cell(centre).map_or(0, |c| i32::from(c.level))
        + if centre_deck && is_deck(centre) {
            4 // spec: S-0207 claim 9 (a deck counts 4 levels above its cell)
        } else {
            0
        };
    // spec: S-0212 claim 25 (the empty-radius approach-failure row of T5 excludes bridge cells, whatever the locomotion class)
    let no_bridges = match use_ {
        NearbyUse::Approach { .. } => true,
        NearbyUse::Settings(x) => x.no_bridges,
        _ => false,
    };
    let overlay_test = matches!(use_, NearbyUse::Settings(_));
    let mut collected: Vec<CellXY> = Vec::new();
    let mut stop_after_ring = false;
    'rings: for r in 0..=max_r {
        let ring = ring_cells(centre, r, q.nearby_centre_twice);
        for c in ring {
            if no_bridges && is_bridge(w, c) {
                continue;
            }
            // spec: S-0212 claim 25 (the further test, when the user asks for it)
            if qualifies(
                w,
                c,
                class,
                zone,
                reach_ref,
                deck_mode,
                height_cond,
                centre_level,
            ) && (!further
                || (if overlay_test {
                    no_overlay(w, c)
                } else {
                    burrow_ok(w, c)
                }))
                && extra.is_none_or(|f| f(w, c))
            {
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

/// The further test of S-0402 claim 18's first free-unit search: the
/// cell holds no overlay of any kind (a cell outside the map holds none).
pub(crate) fn no_overlay(w: &World, c: CellXY) -> bool {
    w.cells
        .cell(c)
        .is_none_or(|x| x.overlay == glasnost_map::NO_OVERLAY)
}

/// The further test of the nearby-cell search (S-0212 claim 25): the
/// burrowing test of S-0010 claim 17. Every cell outside the usable area
/// passes; a cell inside it passes when its tile's tileset allows burrowing
/// (a tile number outside the theater's tiles always does), it is flat, it
/// is neither a high-bridge cell nor marked as a bridge's body, and its
/// ground layer holds no building and no terrain object.
pub fn burrow_ok(w: &World, c: CellXY) -> bool {
    // spec: S-0010 claim 17 (every cell outside the playable area passes, by the height-aware test)
    if !w.cells.usable(c) {
        return true;
    }
    let Some(cell) = w.cells.cell(c) else {
        return true;
    };
    // spec: S-0010 claim 17 (the tileset's allow-burrowing flag; a tile number outside 0 … tile count − 1 passes this condition)
    let tileset_ok = w
        .catalogue
        .as_ref()
        .and_then(|cat| {
            usize::try_from(cell.tile)
                .ok()
                .and_then(|t| cat.tiles.get(t))
        })
        .is_none_or(|t| !t.no_burrowing);
    let i = w.cells.idx(c);
    // spec: S-0010 claim 17 (not a high-bridge cell and no bridge-body mark)
    let bridge = i.is_some_and(|i| w.cells.deck[i] || w.cells.bridge_body[i]);
    // spec: S-0010 claim 17 (flat, and no building and no terrain object on the ground layer)
    tileset_ok
        && cell.slope == 0
        && !bridge
        && !w.cells.list(c, Layer::Ground).iter().any(|&x| {
            w.obj(x)
                .is_some_and(|o| matches!(o.kind, Kind::Building | Kind::Terrain))
        })
}

/// Whether a cell qualifies for the nearby-cell search (S-0212 claim 25).
#[allow(clippy::too_many_arguments)]
pub(crate) fn qualifies(
    w: &World,
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
    // spec: S-0212 claim 25 (the cell lies in the user's reach region; a bridge cell read for its deck in deck mode, for its ground otherwise)
    if let Some(rr) = reach_ref
        && regions::reach_read(w, c, zone, deck_mode) != rr
    {
        return false;
    }
    let r = w.rules_arc();
    // spec: S-0212 claim 25 (enterable by class, a bridge cell judged as entered on its deck, in deck mode or not: its land type is not consulted)
    if !crate::passability::enterable_by_class(&r, w, c, class, zone, deck) {
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

/// The start area's side (S-0009 claim 18: 8 × 8 cells).
const START_AREA: i32 = 8; // spec: S-0009 Constants (random start clear area 8 × 8)

/// Which cells of the map are clear for a start area (S-0009 claim 18), as
/// prefix sums of the cells that are not, so that an 8 × 8 block is tested
/// at once. Built once per start list from the cells as they stand.
pub struct StartAreas {
    side: i32,
    /// Prefix sums over the coordinate square of the cells that are not
    /// clear, (side + 1)² entries.
    blocked: Vec<u32>,
}

impl StartAreas {
    /// The clear cells of `w` as they stand: a legal cell is clear when it
    /// passes the class rule (S-0211 claim 10) for class *track* with
    /// movement zone *normal*, a bridge cell judged on its deck, and its
    /// occupancy record (S-0209), the deck's for a bridge cell, is empty.
    /// Every other cell is clear (S-0009 claim 44).
    pub fn new(w: &World) -> Self {
        let r = w.rules_arc();
        let side = i32::from(w.cells.footprint().coordinate_side());
        let n = (side + 1) as usize;
        let mut blocked = vec![0u32; n * n];
        for y in 0..side {
            let mut row = 0u32;
            for x in 0..side {
                let c = CellXY::new(x, y);
                let clear = match w.cells.idx(c) {
                    None => true,
                    Some(i) => {
                        let deck = w.cells.deck[i];
                        let layer = if deck { Layer::Deck } else { Layer::Ground };
                        // spec: S-0009 claim 18 (clear: the class rule for track, zone normal, a bridge cell on its deck; an empty occupancy record)
                        crate::passability::enterable_by_class(
                            &r,
                            w,
                            c,
                            Some(LocomotionClass::Track),
                            MovementZone::Normal,
                            deck,
                        ) && w.cells.occ(c, layer).is_some_and(|o| o.empty())
                    }
                };
                row += u32::from(!clear);
                let above = blocked[y as usize * n + x as usize + 1];
                blocked[(y as usize + 1) * n + x as usize + 1] = above + row;
            }
        }
        Self { side, blocked }
    }

    fn sum(&self, x: i32, y: i32) -> u32 {
        let n = (self.side + 1) as usize;
        let x = x.clamp(0, self.side) as usize;
        let y = y.clamp(0, self.side) as usize;
        self.blocked[y * n + x]
    }

    /// Whether every cell of the start area with corner `c` is clear: the
    /// cells (x + i, y + j), i, j = 0 … 7 (S-0009 claim 18).
    pub fn clear(&self, c: CellXY) -> bool {
        let (x0, y0) = (c.x, c.y);
        let (x1, y1) = (c.x + START_AREA, c.y + START_AREA);
        let blocked = self.sum(x1, y1) + self.sum(x0, y0) - self.sum(x0, y1) - self.sum(x1, y0);
        blocked == 0
    }
}

/// The start-cell variant of the nearby-cell search (S-0009 claim 18): the
/// rings, scan order, 24-cell limit, visibility test and ring cut-off of
/// S-0212 claim 25 around `centre`, with no reach region, no deck mode, no
/// height condition, no further test and no reference cell. A cell
/// qualifies when it is usable (the height-aware test of S-0009 claim 4)
/// and its start area is clear. The result is the preferred cell at
/// position (t mod n), t the tick number. No draw is taken.
pub fn start_cell(w: &World, areas: &StartAreas, centre: CellXY) -> Option<CellXY> {
    let q = w.rules().quirks;
    let fp = *w.cells.footprint();
    // spec: S-0212 claim 25 (largest radius 31, or w + h − 1 when w + h < 32)
    let max_r = if fp.width + fp.height < 32 {
        fp.width + fp.height - 1
    } else {
        31
    };
    let mut collected: Vec<CellXY> = Vec::new();
    let mut stop_after_ring = false;
    'rings: for r in 0..=max_r {
        for c in ring_cells(centre, r, q.nearby_centre_twice) {
            // spec: S-0009 claim 18 (qualifies: a usable corner, height-aware, and a clear start area)
            if w.cells.usable(c) && areas.clear(c) {
                collected.push(c);
                // spec: S-0212 claim 25 (cut off at the 24th collected cell)
                if collected.len() >= 24 {
                    break 'rings;
                }
                // spec: S-0212 claim 25 (the ring with a collected cell passing the visibility test ends the collection; no deck mode)
                if crate::scatter::screen_cover_passes(w, c) {
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
    // spec: S-0212 claim 25 (no reference cell: the preferred cell at position t mod n); S-0009 claim 18 (t is 0 during loading)
    let n = preferred.len() as i32;
    preferred.get(w.tick.rem_euclid(n) as usize).copied()
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
        // A unit at rest moves into the cell of its first stored step, so its
        // remaining steps are the ones after that step (claim 8's heading cell).
        let at_rest = !(u.drive.as_ref().is_some_and(|d| d.track.is_some())
            || u.walk.as_ref().is_some_and(|s| s.target.is_some())
            || crate::hover::has_speed(w, oid));
        let skip = usize::from(at_rest);
        for &s in u.path.steps.iter().skip(skip).take(STORED) {
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

/// The coarse end cell of `c` (S-0212 claim 13b): the cell itself, except
/// for a bridge cell `c` read as on its deck (a start on a deck, a
/// destination that is a bridge cell), whose coarse end cell lies at an end
/// of the bridge on the same lane, by the table of claim 13b.
pub fn coarse_end(w: &World, c: CellXY, deck: bool) -> CellXY {
    if !deck || !is_bridge(w, c) {
        return c;
    }
    // spec: S-0212 claim 13b (the bridge's link: the first in creation order that X lies within 2 cells of)
    match crate::bridges::link_within(w, c, 2) {
        Some(l) => {
            // spec: S-0212 claim 13b (the lane cells: the two end cells moved by X's signed side offset o across the line)
            let o = l.side(c);
            let shift = |e: CellXY| {
                if l.along_x {
                    CellXY::new(e.x, e.y + o)
                } else {
                    CellXY::new(e.x + o, e.y)
                }
            };
            let (lane1, lane2) = (shift(l.first), shift(l.second));
            if l.enabled {
                let d = |e: CellXY| {
                    let (dx, dy) = (i64::from(e.x - c.x), i64::from(e.y - c.y));
                    // spec: S-0212 claim 13b (the table square root of the squared distance in cells, truncated to a whole number)
                    root_floor((dx * dx + dy * dy) as u64, true, w.rules().mode)
                };
                // spec: S-0212 claim 13b (found, enabled: the nearer lane cell; the second end's on equal distances)
                if d(lane1) < d(lane2) { lane1 } else { lane2 }
            } else {
                let y = crate::bridges::first_off_bridge(w, c, l.along_x);
                // spec: S-0212 claim 13b (found, not enabled: the second end's lane cell when Y meets the bridge-end condition, the first end's otherwise)
                if y.is_some_and(|y| crate::bridges::end_condition(w, y)) {
                    lane2
                } else {
                    lane1
                }
            }
        }
        None => {
            let ends = crate::bridges::ends(w, c);
            let inside: Vec<CellXY> = ends
                .iter()
                .flatten()
                .copied()
                .filter(|&e| w.cells.usable(e))
                .collect();
            let dist = |e: CellXY| (e.x - c.x).abs().max((e.y - c.y).abs());
            // spec: S-0212 claim 13b (none found: the nearer bridge end inside the usable area, the first on equal distance; neither: X itself, departure D6)
            let mut best: Option<CellXY> = None;
            for e in inside {
                if best.is_none_or(|b| dist(e) < dist(b)) {
                    best = Some(e);
                }
            }
            best.unwrap_or(c)
        }
    }
}

/// The anchor of the bridge cell `x` for sector `s` (S-0212 F8): the first
/// of x's first bridge end, the two cells beside it across the bridge's
/// axis, x's second bridge end and the two cells beside that, that lies
/// inside the usable area with level-0 sector `s`.
fn anchor(w: &World, x: CellXY, s: u32) -> Option<CellXY> {
    let level0 = w.regions.sectors.levels.first()?;
    // spec: S-0212 F8 (along cell x: east end, south of it, north of it, west end, south, north; along cell y: south end, west, east, north end, west, east)
    let beside: [(i32, i32); 3] = match w.cells.axis(x) {
        crate::cells::Axis::X => [(0, 0), (0, 1), (0, -1)],
        crate::cells::Axis::Y => [(0, 0), (-1, 0), (1, 0)],
        crate::cells::Axis::None => return None,
    };
    crate::bridges::ends(w, x)
        .iter()
        .flatten()
        .flat_map(|e| {
            beside
                .iter()
                .map(move |&(dx, dy)| CellXY::new(e.x + dx, e.y + dy))
        })
        .find(|&c| w.cells.usable(c) && level0.sector(w, c) == s)
}

/// The estimate test of S-0212 claim 24 for adopting `found` in place of
/// the destination `dest`: the coarse estimate E is at most their
/// Chebyshev distance plus 6 cells.
fn estimate_ok(
    w: &World,
    id: ObjectId,
    u: &crate::rules::UnitRules,
    found: CellXY,
    dest: CellXY,
) -> bool {
    // spec: S-0212 claim 24 (E for the unit's own zone; zone normal for a subterranean unit whose destination passes the further test)
    let zone = if u.zone == MovementZone::Subterranean && burrow_ok(w, dest) {
        MovementZone::Normal
    } else {
        u.zone
    };
    let direct = (found.x - dest.x).abs().max((found.y - dest.y).abs());
    // spec: S-0212 claim 24 (adopted when E ≤ Chebyshev distance + 6; not when the coarse search fails)
    coarse_estimate_layers(
        w,
        zone,
        found,
        true,
        dest,
        true,
        // spec: S-0212 claim 24 (Q-1975: the coarse search adds the threat term as a path computation's does: the unit's own coefficient and its owner's records, the zone alone changing for a subterranean unit)
        corridor::threat_of(w, id),
    )
    .is_some_and(|e| e <= direct + 6) // spec: S-0212 Constants (substitution detour allowance 6 cells)
}

/// The coarse estimate E of S-0212 claim 24 and F8 from `found` to `dest`
/// for movement zone `zone`, in cells; `None` when the coarse search fails.
/// The regions must be up to date ([`regions::ensure`]).
pub fn coarse_estimate(w: &World, zone: MovementZone, found: CellXY, dest: CellXY) -> Option<i32> {
    // spec: S-0212 claim 13b (the found cell counts as on a bridge deck when it is a bridge cell)
    coarse_estimate_layers(w, zone, found, true, dest, true, None)
}

/// [`coarse_estimate`] with each end read on its deck (`found_deck`,
/// `dest_deck`) or on the ground: a bridge cell read on the ground is its
/// own coarse end cell (S-0212 claim 13b); `threat` is the threat term of the
/// coarse search (claim 24, `None` for none); the legs of F8 follow the bridge
/// cells as before.
pub fn coarse_estimate_layers(
    w: &World,
    zone: MovementZone,
    found: CellXY,
    found_deck: bool,
    dest: CellXY,
    dest_deck: bool,
    threat: Option<corridor::Threat>,
) -> Option<i32> {
    // spec: S-0212 claim 13b (a bridge cell read as on its deck has its coarse end cell at a bridge end; any other cell is its own)
    let found_end = coarse_end(w, found, found_deck);
    let dest_end = coarse_end(w, dest, dest_deck);
    // spec: S-0212 claim 24 (the coarse search alone, between the coarse end cells; when it fails the cell is not adopted; the threat term of `threat` as in a path computation, F5)
    let chains = corridor::coarse_why(
        w,
        &w.regions.sectors,
        zone,
        found_end,
        dest_end,
        &Default::default(),
        threat,
    )
    .ok()?;
    let chain = &chains[0];
    let n = chain.len() as i32;
    let cheb = |a: CellXY, b: CellXY| (a.x - b.x).abs().max((a.y - b.y).abs());
    let direct = cheb(found, dest);
    let link3 = |c: CellXY| crate::bridges::link_within(w, c, 3); // spec: S-0212 Constants (lane reach for the same-bridge test: 3 cells)
    // spec: S-0212 F8 (both bridge cells of one bridge: the first link within 3 cells is the same for both; E is the Chebyshev distance alone)
    if is_bridge(w, found)
        && is_bridge(w, dest)
        && link3(found).is_some()
        && link3(found) == link3(dest)
    {
        return Some(direct);
    }
    let s = |k: i32| chain.get((k - 1) as usize).copied();
    // A leg: the Chebyshev distance from x to its anchor for the first
    // sector that has one, else to its coarse end cell.
    let leg = |x: CellXY, first: Option<u32>, then: Option<u32>, end: CellXY| -> i32 {
        let a = first
            .and_then(|t| anchor(w, x, t))
            .or_else(|| then.and_then(|t| anchor(w, x, t)))
            .unwrap_or(end);
        cheb(x, a)
    };
    let mut legs = 0;
    if is_bridge(w, dest) {
        // spec: S-0212 F8 (destination leg: its anchor for sₙ₋₁ when n ≥ 4, for sₙ when n < 4; then for sₙ; then its coarse end cell)
        let first = if n >= 4 { s(n - 1) } else { s(n) };
        legs += leg(dest, first, s(n), dest_end);
    }
    if is_bridge(w, found) {
        // spec: S-0212 F8 (found-cell leg: its anchor for s₂ when n ≥ 4, for s₁ when n < 4; then for s₁; then its coarse end cell)
        let first = if n >= 4 { s(2) } else { s(1) };
        legs += leg(found, first, s(1), found_end);
    }
    // spec: S-0212 claim 24 (E = max(Chebyshev distance, 2n − 2 + legs))
    Some(direct.max(2 * n - 2 + legs))
}

/// The cell searches of a fresh path computation with the corridor (S-0212
/// claims 13 and 14) for unit `id` from its cell to `to`, before
/// straightening. The regions must be up to date ([`regions::ensure`]).
pub fn corridor_find(w: &World, id: ObjectId, to: CellXY) -> Option<Vec<u8>> {
    let o = w.obj(id)?;
    let u = w.rules().unit(o.kind, o.type_index)?;
    let from = o.pos.cell();
    let req = Request {
        unit: id,
        from,
        to,
        mode: Mode::Fresh,
        congested: BTreeSet::new(),
        avoided: BTreeSet::new(),
    };
    // spec: S-0212 claim 13 (a jumpjet infantry type uses zone infantry for the coarse search)
    let coarse_zone = if o.kind == Kind::Infantry && u.jumpjet {
        MovementZone::Infantry
    } else {
        u.zone
    };
    corridor_search(
        w,
        &req,
        coarse_zone,
        u.zone,
        // spec: S-0212 claim 13b (the start's coarse end cell when the unit is on a bridge deck; the destination's when it is a bridge cell)
        coarse_end(w, from, o.layer == Layer::Deck),
        coarse_end(w, to, true),
    )
}

/// The searches of one path computation with the corridor (S-0212 claim
/// 13, the table of searches): the coarse search, a cell search inside the
/// corridor, and up to 5 cell searches with retries.
fn corridor_search(
    w: &World,
    req: &Request,
    coarse_zone: MovementZone,
    own_zone: MovementZone,
    start_end: CellXY,
    dest_end: CellXY,
) -> Option<Vec<u8>> {
    // One world for all the computation's searches: what they ask of it may
    // be kept for the computation (our tooling for speed, M8 C4 part 5).
    w.scratch.borrow_mut().begin_computation(w.cells.grid.len());
    let found = corridor_searches(w, req, coarse_zone, own_zone, start_end, dest_end);
    w.scratch.borrow_mut().end_computation();
    found
}

/// [`corridor_search`]'s searches.
fn corridor_searches(
    w: &World,
    req: &Request,
    coarse_zone: MovementZone,
    own_zone: MovementZone,
    start_end: CellXY,
    dest_end: CellXY,
) -> Option<Vec<u8>> {
    let secs = &w.regions.sectors;
    let mut closed: corridor::Closed = Default::default();
    let mut progress = req.from;
    let mut searches = 0;
    let mut first = true;
    loop {
        // spec: S-0212 claim 13 and F5 (the coarse search adds the threat term of the unit's owner's records when its coefficient exceeds the binary64 value nearest 0.00001)
        let threat = corridor::threat_of(w, req.unit);
        let chains = match corridor::coarse_why(
            w,
            secs,
            coarse_zone,
            start_end,
            dest_end,
            &closed,
            threat,
        ) {
            Ok(c) => c,
            Err(why) => {
                // spec: S-0212 claim 13 (a first coarse search that fails: one cell search without a corridor, final)
                if first {
                    return search(w, req, None, &mut progress);
                }
                // spec: S-0212 claim 13 and Edge cases (quirks.corridor_retry_no_fallback off: a failed repeated coarse search is followed by one cell search without a corridor, final; a retry is made only after fewer than 5 cell searches, so it is at most the 5th; departure D2's chain-length failure fails the computation whatever the switch)
                if !w.rules().quirks.corridor_retry_no_fallback
                    && why != corridor::CoarseFail::ChainLength
                {
                    return search(w, req, None, &mut progress);
                }
                // spec: S-0212 claim 13 and Edge cases (quirks.corridor_retry_no_fallback on: a repeated coarse search that fails fails the computation)
                return None;
            }
        };
        first = false;
        searches += 1;
        let level0 = secs.levels.first()?;
        let filter = CorridorFilter::new(level0, &chains[0]);
        // spec: S-0212 claim 13 (a cell search inside the corridor; a found path is the result)
        if let Some(p) = search(w, req, Some(&filter), &mut progress) {
            return Some(p);
        }
        // spec: S-0212 claim 13 (at most 5 cell searches per path computation)
        if searches >= 5 {
            return None;
        }
        let mut abandoned = false;
        // spec: S-0212 claim 13 (the retry table applies at each level; the free part uses the unit's own zone, claim 13a)
        for (l, chain) in chains.iter().enumerate() {
            if !corridor::retry(
                w,
                secs,
                req.unit,
                own_zone,
                l,
                progress,
                chain,
                &mut closed[l],
            ) {
                abandoned = true;
            }
        }
        if abandoned {
            // spec: S-0212 claim 13 (a corridor abandoned at some level: one cell search without a corridor, final, which counts towards the 5)
            return search(w, req, None, &mut progress);
        }
    }
}

/// A path computation for a unit (S-0212 claims 6, 20, 24): the gate,
/// destination substitution, the search, flattening and storage. With the
/// movement trace on, it is recorded ([`crate::trace`]).
pub fn compute(w: &mut World, id: ObjectId, mode: Mode) -> Search {
    if !crate::trace::on(w) {
        return compute_untraced(w, id, mode);
    }
    let from = w.obj(id).map(|o| o.pos.cell());
    crate::trace::begin_search(w);
    let result = compute_untraced(w, id, mode);
    let noted = crate::trace::pending(w);
    let (steps, to) = match (result, w.obj(id)) {
        (Search::Found, Some(o)) => (o.path.steps.clone(), o.path.for_goal),
        _ => (Vec::new(), None),
    };
    crate::trace::record(w, id, || crate::trace::What::Search {
        asked: mode,
        mode: noted.mode,
        from: from.unwrap_or(CellXY::new(0, 0)),
        goal: noted.goal,
        to,
        congested: noted.congested,
        corridor: noted.corridor,
        searches: noted.searches,
        result,
        steps,
    });
    result
}

/// [`compute`] without the trace.
fn compute_untraced(w: &mut World, id: ObjectId, mode: Mode) -> Search {
    let res = compute_once(w, id, mode);
    if res == Search::Found || w.rules().quirks.unreachable_move_dropped {
        return res;
    }
    // design: C7-P6 hold (a move held for units in its way keeps its destination: the nearest reachable cell is for a destination the unit cannot reach)
    if res == Search::SearchFailed && holds(w, id) {
        return res;
    }
    // design: C7-P4 nearest reachable cell (a failed move tries once more towards the nearest cell the unit can stand on in its own reach region; one try, as the new destination is reachable)
    let Some(c) = reachable_near(w, id) else {
        return res;
    };
    if let Some(ob) = w.obj_mut(id) {
        ob.dest = Some(Target::Cell(c));
    }
    compute_once(w, id, mode)
}

/// How far round a move's destination [`reachable_near`] looks, in rings.
// nocite: design: C7-P4 nearest reachable cell (our choice: two of the group spread's looks)
pub const NEAR_RADIUS: i32 = 12;

/// The expansions a search that keeps off stalled units' cells may make
/// (`quirks.path_through_stalled_blockers` off).
// nocite: design: C7-P4 stalled blockers (our choice: a detour near the stall, a sixteenth of the search limit of S-0212 claim 12)
pub const DETOUR_EXPANSIONS: u32 = 4096;

/// The nearest cell to a moving unit's destination cell that the unit can
/// stand on in its own reach region, for a move whose computation failed
/// (`quirks.unreachable_move_dropped` off; our design, C7 part 4): the
/// first ring round the destination, out to [`NEAR_RADIUS`], with a cell
/// in the map and the usable area, in the unit's reach region, whose entry
/// outcome is clear or a moving blocker and that no other unit of its house
/// has as its destination; of that ring the cell nearest the destination,
/// the first in row order on equal distances. None when the unit is not on
/// a move to a cell, when no ring has such a cell, or when the unit stands
/// as near the destination as that cell. It takes no draw.
pub fn reachable_near(w: &World, id: ObjectId) -> Option<CellXY> {
    let o = w.obj(id)?;
    if o.beh.current != Some(Behaviour::Move) {
        return None;
    }
    let Some(Target::Cell(goal)) = o.dest else {
        return None;
    };
    let u = w.rules().unit(o.kind, o.type_index)?;
    let from = o.pos.cell();
    let own = regions::reach_read(w, from, u.zone, o.layer == Layer::Deck);
    if matches!(own, Reach::OffMap | Reach::None) {
        return None;
    }
    let d2 = |c: CellXY| {
        let dx = i64::from(c.x - goal.x);
        let dy = i64::from(c.y - goal.y);
        dx * dx + dy * dy
    };
    let reference = occupancy::reference_level(w, id);
    let taken: BTreeSet<CellXY> = w
        .objects
        .iter()
        .filter(|(oid, x)| **oid != id && x.owner == o.owner)
        .filter_map(|(_, x)| match x.dest {
            Some(Target::Cell(c)) => Some(c),
            _ => None,
        })
        .collect();
    for r in 1..=NEAR_RADIUS {
        let mut best: Option<(i64, CellXY)> = None;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx.abs().max(dy.abs()) != r {
                    continue;
                }
                let c = CellXY::new(goal.x + dx, goal.y + dy);
                if w.cells.cell(c).is_none() || !w.cells.usable(c) || taken.contains(&c) {
                    continue;
                }
                if regions::reach_read(w, c, u.zone, true) != own {
                    continue;
                }
                let out = match o.kind {
                    Kind::Infantry => occupancy::infantry_outcome(w, id, c, reference),
                    _ => occupancy::vehicle_outcome(w, id, c, reference),
                };
                if !matches!(out, Outcome::Clear | Outcome::MovingBlocker) {
                    continue;
                }
                let d = d2(c);
                if best.is_none_or(|(b, _)| d < b) {
                    best = Some((d, c));
                }
            }
        }
        if let Some((d, c)) = best {
            return (d < d2(from)).then_some(c);
        }
    }
    None
}

/// Whether unit `x` is stalled: a vehicle or infantry unit on the map's
/// ground layer with a destination whose blockage (S-0212 claim 22) has
/// lasted the blockage delay.
fn stalled(w: &World, x: &crate::world::Object) -> bool {
    // design: C7-P4 stalled blockers (stalled: blocked for the blockage delay of S-0212 claim 22, as long as a unit's own wait before mode 2)
    let delay = w.rules().general.blockage_delay;
    x.on_map
        && x.layer == Layer::Ground
        && matches!(x.kind, Kind::Vehicle | Kind::Infantry)
        && x.dest.is_some()
        && x.path
            .blocked_since
            .is_some_and(|b| w.tick.wrapping_sub(b) >= delay)
}

/// The cells a stalled unit `id`'s search keeps off with
/// `quirks.path_through_stalled_blockers` off (our design, C7 part 4): the
/// cells of the other stalled units that wait in a ring. Each stalled unit
/// (`id` among them) waits on the stalled units in the cell it heads for
/// (S-0212 claim 8's heading cell); one that waits on none, or only on units
/// that do not wait in a ring themselves, is in a queue whose head still
/// moves, and is not kept off. What is left after taking such units away
/// until none is left are the units that wait, directly or through others,
/// on units that wait on each other: a deadlock. `id` with no stored path
/// waits on the stalled units round it. Empty unless `id` itself is left,
/// so a stalled unit elsewhere searches as the original does.
pub fn stalled_cells(w: &World, id: ObjectId) -> BTreeSet<CellXY> {
    stalled_ring(w, id)
        .into_iter()
        .filter(|&(u, _)| u != id)
        .map(|(_, c)| c)
        .collect()
}

/// The units of the deadlock [`stalled_cells`] finds for `id`, `id` among
/// them, with their cells; empty unless `id` is in it.
fn stalled_ring(w: &World, id: ObjectId) -> BTreeMap<ObjectId, CellXY> {
    let Some(me) = w.obj(id) else {
        return BTreeMap::new();
    };
    let here = me.pos.cell();
    // The cells `id` waits for, as below.
    let mine: Vec<CellXY> = match heading_cell(w, id) {
        Some(h) => vec![h],
        None => STEPS
            .iter()
            .map(|&(dx, dy)| CellXY::new(here.x + dx, here.y + dy))
            .collect(),
    };
    let units: BTreeMap<ObjectId, CellXY> = w
        .objects
        .iter()
        .filter(|(oid, x)| **oid == id || stalled(w, x))
        .map(|(oid, x)| (*oid, x.pos.cell()))
        .collect();
    // design: C7-P5 tick cost (a unit that waits on no stalled unit is in no deadlock: the graph is not built)
    if !units.iter().any(|(&u, c)| u != id && mine.contains(c)) {
        return BTreeMap::new();
    }
    let mut by_cell: BTreeMap<CellXY, Vec<ObjectId>> = BTreeMap::new();
    for (&u, &c) in &units {
        by_cell.entry(c).or_default().push(u);
    }
    // design: C7-P6 hold (quirks.jammed_move_dropped off: a unit as the hold keeps it, blocked on a move to a cell with no stored path, waits, like `id`, on every neighbour of its cell)
    let hold = !w.rules().quirks.jammed_move_dropped;
    let held = |u: ObjectId| hold && blocked_mover(w, u);
    let waits: BTreeMap<ObjectId, Vec<ObjectId>> = units
        .keys()
        .map(|&u| {
            // The cells it waits for: the one it heads for; for `id` with no
            // stored path (a blocked step discards it before the search),
            // and with the hold for a unit as it keeps one, every neighbour
            // of its cell.
            let cells: Vec<CellXY> = match heading_cell(w, u) {
                Some(h) => vec![h],
                None if u == id || held(u) => STEPS
                    .iter()
                    .map(|&(dx, dy)| CellXY::new(units[&u].x + dx, units[&u].y + dy))
                    .collect(),
                None => Vec::new(),
            };
            let on = cells
                .iter()
                .filter_map(|h| by_cell.get(h))
                .flatten()
                .copied()
                .filter(|&x| x != u)
                .collect();
            (u, on)
        })
        .collect();
    // design: C7-P5 tick cost (the units that wait on none left are taken away one at a time through a count of what each waits on, not by scanning every unit again each round; what is left is the same)
    let mut left: BTreeMap<ObjectId, usize> = waits.iter().map(|(&u, on)| (u, on.len())).collect();
    let mut by: BTreeMap<ObjectId, Vec<ObjectId>> = BTreeMap::new();
    for (&u, on) in &waits {
        for &x in on {
            by.entry(x).or_default().push(u);
        }
    }
    let mut free: Vec<ObjectId> = left
        .iter()
        .filter(|&(_, &n)| n == 0)
        .map(|(&u, _)| u)
        .collect();
    let mut ring: BTreeSet<ObjectId> = units.keys().copied().collect();
    while let Some(x) = free.pop() {
        ring.remove(&x);
        for &u in by.get(&x).into_iter().flatten() {
            if let Some(n) = left.get_mut(&u) {
                *n -= 1;
                if *n == 0 {
                    free.push(u);
                }
            }
        }
    }
    // design: C7-P4 stalled blockers (only a unit in the deadlock, or waiting on it, keeps off it)
    if !ring.contains(&id) {
        return BTreeMap::new();
    }
    ring.iter().map(|&u| (u, units[&u])).collect()
}

/// How far a unit giving way looks for a free cell, in steps.
// nocite: design: C7-P5 yield (our choice: the nearest reachable cell's look of C7-P4)
pub const YIELD_RADIUS: i32 = NEAR_RADIUS;

/// The longest a unit gives way before it resumes its order, in ticks.
// nocite: design: C7-P5 yield (our choice: about a minute of game time, long enough for a group to pass a long passage)
pub const YIELD_WAIT: i32 = 900;

/// The most cells [`shut`] looks at before it gives up.
// nocite: design: C7-P5 yield (our choice: two rooms and a long passage between them, small against a map's open ground)
pub const YIELD_FLOOD: usize = 2048;

/// Whether the cells `wall` names shut `id` off from `goal` (our design, C7
/// part 5): a fill from `id`'s cell and one from `goal`, a step at a time
/// each, in the eight directions through cells of the usable area in
/// `id`'s reach region that are not walls. Some(true) when one of them runs
/// out of cells before they meet, Some(false) when they meet; None when
/// they have looked at [`YIELD_FLOOD`] cells between them, or when `id` is
/// on a deck, outside the usable area or the fills meet a bridge cell (a
/// fill on the ground alone cannot say).
fn shut(
    w: &World,
    id: ObjectId,
    u: &crate::rules::UnitRules,
    wall: &dyn Fn(CellXY) -> bool,
    goal: CellXY,
) -> Option<bool> {
    let o = w.obj(id)?;
    let from = o.pos.cell();
    if o.layer == Layer::Deck || !w.cells.usable(from) {
        return None;
    }
    let own = regions::reach_read(w, from, u.zone, false);
    if matches!(own, Reach::OffMap | Reach::None) {
        return None;
    }
    let mut seen = [BTreeSet::from([from]), BTreeSet::from([goal])];
    let mut front = [vec![from], vec![goal]];
    loop {
        // nocite: design: C7-P5 yield (the fill's two sides, from the unit and from its goal)
        for side in 0..2 {
            let mut next = Vec::new();
            for &c in &front[side] {
                for &(dx, dy) in STEPS.iter() {
                    let n = CellXY::new(c.x + dx, c.y + dy);
                    if seen[1 - side].contains(&n) {
                        return Some(false);
                    }
                    if w.cells.cell(n).is_none()
                        || !w.cells.usable(n)
                        || wall(n)
                        || regions::reach_read(w, n, u.zone, true) != own
                    {
                        continue;
                    }
                    if is_bridge(w, n) {
                        return None;
                    }
                    if seen[side].insert(n) {
                        next.push(n);
                    }
                }
            }
            if next.is_empty() {
                return Some(true);
            }
            if seen[0].len() + seen[1].len() > YIELD_FLOOD {
                return None;
            }
            front[side] = next;
        }
    }
}

/// Whether the deadlock `ring` leaves `id` no way round to `goal` (our
/// design, C7 part 5, yield): whether the cells of the deadlock's units
/// not going `id`'s way ([`goes_with`]) shut it off from `goal`
/// ([`shut`]).
fn shut_in(
    w: &World,
    id: ObjectId,
    u: &crate::rules::UnitRules,
    ring: &BTreeMap<ObjectId, CellXY>,
    goal: CellXY,
) -> bool {
    let Some(h) = heading(w, id) else {
        return false;
    };
    // design: C7-P5 yield (no way round: the deadlock's units not going `id`'s way shut it off from its destination; those going its way move on with it)
    let walls: BTreeSet<CellXY> = ring
        .iter()
        .filter(|&(&x, _)| x != id && !goes_with(w, x, h))
        .map(|(_, &c)| c)
        .collect();
    shut(w, id, u, &|c| walls.contains(&c), goal) == Some(true)
}

/// Whether unit `x` goes the way of the heading `h` (our design, C7 parts 5
/// and 6, yield): its own heading to its destination makes a positive dot
/// product with `h`. Part 5 counted a unit at right angles (a dot product of
/// 0) as going the same way; at a crossing of passages it does not, so of
/// three groups meeting there two would never give way to each other.
fn goes_with(w: &World, x: ObjectId, (hx, hy): (i64, i64)) -> bool {
    // design: C7-P6 three ways (going the same way: a positive dot product of the headings; a unit at right angles crosses the way and is not going with it)
    heading(w, x).is_some_and(|(ax, ay)| ax * hx + ay * hy > 0)
}

/// The straight-line heading of a unit to its destination cell, in cells.
fn heading(w: &World, x: ObjectId) -> Option<(i64, i64)> {
    let o = w.obj(x)?;
    let g = goal_cell(w, x)?;
    let c = o.pos.cell();
    Some((i64::from(g.x - c.x), i64::from(g.y - c.y)))
}

/// Settles the deadlock `ring` (from [`stalled_ring`], `id` among its
/// units) that has no way round, with `quirks.deadlock_no_yield` off (our
/// design, C7 part 5, yield). The unit of the deadlock with the lowest
/// object id that is not already giving way has the right of way. The
/// units in play are the deadlock's and every unit of that unit's house
/// on a move to a cell within [`YIELD_RADIUS`] of a unit of the deadlock;
/// of these, every unit of the deadlock not going that unit's way
/// ([`goes_with`]) and every other one whose heading to its destination
/// points against that unit's (a negative dot product) gives way, the
/// others go with it.
/// A unit that gives way gets the nearest free cell off the way of the
/// units it gives way to (the cells of their paths by the original's
/// search, and their own cells) and saves its order to resume
/// ([`yield_turn`]). None when no unit gives way; otherwise whether `id`
/// itself gives way. It takes no draw.
fn give_way(w: &mut World, id: ObjectId, ring: &BTreeMap<ObjectId, CellXY>) -> Option<bool> {
    let free = |w: &World, x: ObjectId| w.obj(x).is_some_and(|o| o.path.yielding.is_none());
    // design: C7-P5 yield (the right of way: the lowest object id, the oldest unit, of the deadlock's units not already giving way)
    let first = ring.keys().copied().find(|&x| free(w, x))?;
    let (hx, hy) = heading(w, first)?;
    let house = w.obj(first)?.owner;
    let moving = |o: &crate::world::Object| {
        o.on_map
            && o.beh.current == Some(Behaviour::Move)
            && matches!(o.dest, Some(Target::Cell(_)))
    };
    // design: C7-P5 yield (the units in play: the deadlock's, and the moving units of the house with the right of way near it, so a whole group gives way or goes)
    let mut play: BTreeSet<ObjectId> = ring.keys().copied().collect();
    for (&x, o) in &w.objects {
        let c = o.pos.cell();
        if o.owner == house
            && o.path.yielding.is_none()
            && moving(o)
            && ring
                .values()
                .any(|r| (r.x - c.x).abs().max((r.y - c.y).abs()) <= YIELD_RADIUS)
        {
            play.insert(x);
        }
    }
    let mut yielders: Vec<ObjectId> = Vec::new();
    let mut winners: Vec<ObjectId> = Vec::new();
    for &x in &play {
        if !free(w, x) {
            continue;
        }
        // design: C7-P6 three ways (a unit of the deadlock that is not going the way of the unit with the right of way gives way, one at right angles among them; a unit near it but not in it gives way only when heading against it, so one crossing the way elsewhere is not stopped)
        let crosses = if ring.contains_key(&x) {
            !goes_with(w, x, (hx, hy))
        } else {
            heading(w, x).is_some_and(|(ax, ay)| ax * hx + ay * hy < 0)
        };
        let against = x != first
            && w.obj(x).is_some_and(|o| {
                o.beh.current == Some(Behaviour::Move) && matches!(o.dest, Some(Target::Cell(_)))
            })
            && crosses;
        // design: C7-P5 yield (the side that gives way: heading against the unit with the right of way)
        if against {
            yielders.push(x);
        } else {
            winners.push(x);
        }
    }
    if yielders.is_empty() {
        return None;
    }
    // design: C7-P5 yield (the way kept free: the cells of the other side and of their paths by the original's search)
    let mut lane: BTreeSet<CellXY> = BTreeSet::new();
    for &x in &winners {
        let Some(o) = w.obj(x) else { continue };
        let from = o.pos.cell();
        lane.insert(from);
        let Some(to) = goal_cell(w, x) else { continue };
        let req = Request {
            unit: x,
            from,
            to,
            mode: Mode::Fresh,
            congested: BTreeSet::new(),
            avoided: BTreeSet::new(),
        };
        let mut c = from;
        for d in S0212.find(w, &req).unwrap_or_default() {
            let (sx, sy) = STEPS[usize::from(d % 8)]; // spec: S-0212 Terms (step directions 0 to 7)
            c = CellXY::new(c.x + sx, c.y + sy);
            lane.insert(c);
        }
    }
    let walls: BTreeSet<CellXY> = winners
        .iter()
        .filter_map(|&x| w.obj(x).map(|o| o.pos.cell()))
        .collect();
    let mut reserved: BTreeSet<CellXY> = BTreeSet::new();
    let mut gives = false;
    let mut any = false;
    let t = w.tick;
    for &y in &yielders {
        let Some(refuge) = refuge(w, y, &lane, &walls, &reserved) else {
            continue;
        };
        reserved.insert(refuge);
        let Some(o) = w.obj(y) else { continue };
        let Some(resume) = o.dest else { continue };
        if let Some(ob) = w.obj_mut(y) {
            ob.path.yielding = Some(Yield {
                resume,
                to: winners.clone(),
                since: t,
            });
            // design: C7-P5 yield (a unit that gives way is no longer stalled and searches for its free cell at once)
            ob.path.blocked_since = None;
            ob.path.steps.clear();
            ob.path.retry_at = t;
        }
        crate::econ::assign_destination(w, y, Some(Target::Cell(refuge)));
        crate::trace::record(w, y, || crate::trace::What::GaveWay {
            refuge,
            to: winners.first().copied(),
        });
        gives |= y == id;
        any = true;
    }
    any.then_some(gives)
}

/// The free cell a unit `y` giving way goes to (our design, C7 part 5,
/// yield): the nearest cell in steps, out to [`YIELD_RADIUS`], reached
/// through cells of its own reach region that hold nothing it may not pass
/// (a moving unit aside) and none of `walls`, that is not on `lane`, not
/// `reserved`, no other unit of its house's destination, and whose entry
/// is clear; the first found in step order. None when there is none.
fn refuge(
    w: &World,
    y: ObjectId,
    lane: &BTreeSet<CellXY>,
    walls: &BTreeSet<CellXY>,
    reserved: &BTreeSet<CellXY>,
) -> Option<CellXY> {
    let o = w.obj(y)?;
    let u = w.rules().unit(o.kind, o.type_index)?;
    let from = o.pos.cell();
    let own = regions::reach_read(w, from, u.zone, o.layer == Layer::Deck);
    if matches!(own, Reach::OffMap | Reach::None) {
        return None;
    }
    let reference = occupancy::reference_level(w, y);
    let taken: BTreeSet<CellXY> = w
        .objects
        .iter()
        .filter(|(x, p)| **x != y && p.owner == o.owner)
        .filter_map(|(_, p)| match p.dest {
            Some(Target::Cell(c)) => Some(c),
            _ => None,
        })
        .collect();
    let mut seen: BTreeSet<CellXY> = BTreeSet::from([from]);
    let mut frontier = vec![from];
    for _ in 0..YIELD_RADIUS {
        let mut next = Vec::new();
        for &c in &frontier {
            for &(dx, dy) in STEPS.iter() {
                let n = CellXY::new(c.x + dx, c.y + dy);
                if !seen.insert(n) {
                    continue;
                }
                if w.cells.cell(n).is_none()
                    || !w.cells.usable(n)
                    || walls.contains(&n)
                    || regions::reach_read(w, n, u.zone, true) != own
                {
                    continue;
                }
                let out = match o.kind {
                    Kind::Infantry => occupancy::infantry_outcome(w, y, n, reference),
                    _ => occupancy::vehicle_outcome(w, y, n, reference),
                };
                match out {
                    Outcome::Clear => {
                        if !lane.contains(&n) && !reserved.contains(&n) && !taken.contains(&n) {
                            return Some(n);
                        }
                        next.push(n);
                    }
                    Outcome::MovingBlocker => next.push(n),
                    _ => {}
                }
            }
        }
        frontier = next;
    }
    None
}

/// A unit giving way (C7 part 5, yield) resumes its order once every unit
/// it gives way to has no destination, has left the map or has passed it
/// (stands ahead of it on its own heading to its destination), or after
/// [`YIELD_WAIT`] ticks: the saved destination again, with a move started
/// as a move order starts it. Called in each unit's own turn before its
/// movement.
pub fn yield_turn(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let Some(y) = o.path.yielding.clone() else {
        return;
    };
    if !o.on_map {
        return;
    }
    // design: C7-P5 yield (released when every unit of the other side has arrived, gone or passed it, ahead of it on its own heading, or after the longest wait)
    let here = o.pos.cell();
    let passed = y.to.iter().all(|&x| {
        w.obj(x).is_none_or(|p| {
            let c = p.pos.cell();
            !p.on_map
                || p.dest.is_none()
                || heading(w, x).is_some_and(|(hx, hy)| {
                    i64::from(c.x - here.x) * hx + i64::from(c.y - here.y) * hy > 0
                })
        })
    });
    if !passed && w.tick.wrapping_sub(y.since) < YIELD_WAIT {
        return;
    }
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let t = w.tick;
    if let Some(ob) = w.obj_mut(id) {
        ob.path.yielding = None;
        // spec: S-0213 table T3 (g) (as a move order: a mobile actor forgets its guard spot and queued destinations)
        ob.guard_spot = None;
        ob.queued_dests.clear();
        ob.path.steps.clear();
        ob.beh.queue_and_start_at_once(Behaviour::Move, t, stale);
    }
    crate::econ::assign_destination(w, id, Some(y.resume));
    crate::trace::record(w, id, || crate::trace::What::Resumed);
}

/// An order for `id` ends its giving way (C7 part 5, yield): the order
/// replaces the one it would have resumed.
pub fn order_given(w: &mut World, id: ObjectId) {
    // design: C7-P5 yield (a new order, a stop among them, replaces the order a unit giving way would resume)
    if let Some(ob) = w.obj_mut(id) {
        ob.path.yielding = None;
    }
}

/// The longest a unit holds a move whose search failed for units in its
/// way, counted from the start of its blockage, in ticks.
// nocite: design: C7-P6 hold (our choice: the yield's longest wait, so a unit held behind a group that gives way and passes is not given up before that group has passed)
pub const HOLD_WAIT: i32 = YIELD_WAIT;

/// The longest a unit holds a move whose search failed for units in its
/// way while no unit near it gives way, counted from the start of its
/// blockage, in ticks.
// nocite: design: C7-P6 hold (our choice: about twenty seconds of game time, time for the units in the way to stall and a deadlock's yield to start; a hold that sees no yield start by then gives up as the original does)
pub const HOLD_FIRST: i32 = 300;

/// Whether only moving units stand in the way of `id`'s move (our design,
/// C7 part 6, hold): a vehicle or infantry unit on the ground layer, on a
/// move to a cell, for which [`shut`] finds the destination cell reached
/// from the unit's cell when only cells holding a building, a terrain
/// object or another object that is not a unit, a unit with no destination
/// (one standing, which will not move off by itself), or a wall overlay,
/// are walls. A unit in a crowd of standing units so gives up as the
/// original does. False when the destination cell itself holds another
/// unit that stands or is blocked (two units that would swap cells), and
/// when the fills cannot say ([`shut`]'s None), so the unit then gives up
/// as the original does.
fn jammed(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if !o.on_map
        || o.layer != Layer::Ground
        || !matches!(o.kind, Kind::Vehicle | Kind::Infantry)
        || o.beh.current != Some(Behaviour::Move)
    {
        return false;
    }
    let Some(Target::Cell(goal)) = o.dest else {
        return false;
    };
    let r = w.rules();
    let Some(u) = r.unit(o.kind, o.type_index) else {
        return false;
    };
    // design: C7-P6 hold (not when the destination cell itself holds another unit that stands or is blocked: two units that would swap cells, or one whose cell is taken by a unit that will not leave, gain nothing by waiting where there is a way round, and give up as the original does)
    if w.cells.list(goal, Layer::Ground).iter().any(|&x| {
        x != id
            && w.obj(x)
                .is_some_and(|p| p.kind.is_mobile() && (p.dest.is_none() || blocked_mover(w, x)))
    }) {
        return false;
    }
    // design: C7-P6 hold (only moving units in the way: a fill that passes every unit with a destination and stops at buildings, terrain objects, standing units and walls reaches the destination cell)
    let wall = |c: CellXY| {
        w.cells.list(c, Layer::Ground).iter().any(|&x| {
            x != id
                && w.obj(x)
                    .is_some_and(|p| !p.kind.is_mobile() || p.dest.is_none())
        }) || w.cells.cell(c).is_some_and(|cell| {
            cell.overlay != glasnost_map::NO_OVERLAY
                && r.overlays
                    .get(usize::from(cell.overlay))
                    .is_some_and(|v| v.wall)
        })
    };
    shut(w, id, u, &wall, goal) == Some(false)
}

/// Whether unit `x` is on a move to a cell and blocked: its blockage has
/// begun (S-0212 claim 22). Our design, C7 part 6, hold.
fn blocked_mover(w: &World, x: ObjectId) -> bool {
    w.obj(x).is_some_and(|o| {
        o.beh.current == Some(Behaviour::Move)
            && matches!(o.dest, Some(Target::Cell(_)))
            && o.path.blocked_since.is_some()
    })
}

/// Whether `id`'s failed search is held instead of given up (our design,
/// C7 part 6, hold; `quirks.jammed_move_dropped` off): only moving units
/// stand in its way ([`jammed`]), the failure would give the move up
/// ([`search_gives_up`]) and its blockage has not lasted [`HOLD_FIRST`],
/// or, while a unit within [`YIELD_RADIUS`] of it gives way, [`HOLD_WAIT`].
fn holds(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    // design: C7-P6 hold (only a failure that gives the move up: one that does not takes the original's way, the nearest reachable cell of C7-P4 among it)
    if w.rules().quirks.jammed_move_dropped || !search_gives_up(w, id) {
        return false;
    }
    let blocked = o.path.blocked_since.map_or(0, |b| w.tick.wrapping_sub(b));
    // design: C7-P6 hold (held while blocked for less than the first hold; after it, up to the longest hold, only while a unit within the yield's reach gives way, so a unit waiting behind a yield is not given up before the other side has passed, and one in a crowd no yield settles gives up as the original does)
    let here = o.pos.cell();
    let yield_near = || {
        w.objects.values().any(|p| {
            p.path.yielding.is_some() && p.on_map && {
                let c = p.pos.cell();
                (c.x - here.x).abs().max((c.y - here.y).abs()) <= YIELD_RADIUS
            }
        })
    };
    (blocked < HOLD_FIRST || (blocked < HOLD_WAIT && yield_near())) && jammed(w, id)
}

/// One path computation of [`compute_untraced`].
fn compute_once(w: &mut World, id: ObjectId, mode: Mode) -> Search {
    let Some(o) = w.obj(id).cloned() else {
        return Search::SearchFailed;
    };
    let Some(mut goal) = goal_cell(w, id) else {
        return Search::SearchFailed;
    };
    crate::trace::note(w, |p| p.goal = Some(goal));
    let r = w.rules_arc();
    let Some(u) = r.unit(o.kind, o.type_index).cloned() else {
        return Search::SearchFailed;
    };
    let from = o.pos.cell();
    regions::ensure(w);
    let inside = w.cells.usable(from);
    // A unit inside the usable area may not step outside it (S-0211 Terms) and
    // nothing moves the area's edges yet, so a unit has entered the usable
    // area exactly when it stands inside it (S-0212 claim 14).
    let entered = inside;
    // spec: S-0212 claim 14 (the corridor-disabling state: entered, and a train or in retreat; no unit is a loaner or a team member yet)
    let disabling = entered && (u.train || o.beh.seen() == Some(Behaviour::Retreat));
    // spec: S-0212 claim 6 (the gate is skipped for a unit outside the usable area, and in the corridor-disabling state for a legal destination outside it)
    let gate_skipped =
        !inside || (disabling && w.cells.cell(goal).is_some() && !w.cells.usable(goal));
    let on_deck = o.layer == Layer::Deck;
    // spec: S-0212 claim 6 (the gate compares the reach regions of the unit's cell, read for its deck when the unit is on a bridge deck, and of the destination, read for its deck when it is a bridge cell, for an aircraft as well)
    let gate = |w: &World, goal: CellXY| regions::compare(w, u.zone, from, on_deck, goal, true);
    if !gate_skipped && !gate(w, goal) {
        return Search::GateFailed;
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
                // spec: S-0212 F6b (centre to centre)
                let d = root_floor(
                    (dx * dx + dy * dy) as u64,
                    r.quirks.table_root_distance,
                    r.mode,
                ) as i64;
                // spec: S-0212 claim 24 (adopted when nearer than D_u and the coarse estimate E allows it)
                if d < du && estimate_ok(w, id, &u, c, goal) {
                    goal = c;
                }
            }
        } else if out == Outcome::Blocked && holds_building {
            // spec: S-0212 claim 24 (a blocked building cell: the substitute always; none found keeps it, departure D3)
            if let Some(c) = nearby(w, goal, NearbyUse::Substitution { unit: id }) {
                goal = c;
            }
        }
        // spec: S-0212 claim 24 and departure D3 (the adopted cell replaces the unit's destination cell, so arrival is judged against it)
        if let Some(Target::Cell(d)) = o.dest
            && d != goal
            && let Some(ob) = w.obj_mut(id)
        {
            ob.dest = Some(Target::Cell(goal));
            // spec: S-0402 claim 10a (A chrono miner: an adopted substitute is a destination given while the miner holds contact, a qualified docking assignment, S-0207 claims 27 and 28)
            crate::refresh::adopted(w, id, o.dest);
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
    crate::trace::note(w, |p| {
        p.mode = Some(mode);
        p.congested = congested.iter().copied().collect();
    });
    let req = Request {
        unit: id,
        from,
        to: goal,
        mode,
        congested: congested.clone(),
        avoided: BTreeSet::new(),
    };
    // design: C7-P4 stalled blockers (a stalled unit's computation first looks for a way that keeps off the cells of stalled units in a deadlock, without a corridor, since the corridor reads terrain only; the computation's own searches follow when it finds none)
    let ring = if !r.quirks.path_through_stalled_blockers && stalled(w, &o) {
        stalled_ring(w, id)
    } else {
        BTreeMap::new()
    };
    let avoided: BTreeSet<CellXY> = ring
        .iter()
        .filter(|&(&x, _)| x != id)
        .map(|(_, &c)| c)
        .collect();
    let detour = if avoided.is_empty() {
        None
    } else {
        S0212.find(
            w,
            &Request {
                avoided,
                ..req.clone()
            },
        )
    };
    // design: C7-P5 yield (quirks.deadlock_no_yield off: a deadlock with no way round, no detour found, has one side give way; a unit of the side that gives way searches again for its free cell, one of the other side takes the original's searches)
    if detour.is_none()
        && !ring.is_empty()
        && !r.quirks.deadlock_no_yield
        && shut_in(w, id, &u, &ring, goal)
        && give_way(w, id, &ring) == Some(true)
    {
        return compute_once(w, id, Mode::Fresh);
    }
    // spec: S-0212 claim 13b (the coarse end cells: the start's when the unit is on a bridge deck, the destination's when it is a bridge cell)
    let start_end = coarse_end(w, from, on_deck);
    let dest_end = coarse_end(w, goal, true);
    // spec: S-0212 claim 14 (the corridor is used when the unit has entered the usable area, both coarse end cells are inside it, and it is not in the corridor-disabling state)
    let use_corridor =
        entered && w.cells.usable(start_end) && w.cells.usable(dest_end) && !disabling;
    crate::trace::note(w, |p| p.corridor = Some(use_corridor));
    if use_corridor && !gate(w, goal) {
        // spec: S-0212 claim 14 (with the corridor, reach regions that differ, read as in claim 6, fail the computation at once, as a failed search)
        return Search::SearchFailed;
    }
    // spec: S-0212 claim 13 (a jumpjet infantry type uses zone infantry for the coarse search of a path computation)
    let coarse_zone = if o.kind == Kind::Infantry && u.jumpjet {
        MovementZone::Infantry
    } else {
        u.zone
    };
    let found = match (detour, w.pathfinder) {
        (Some(p), _) => Some(p),
        (None, PathfinderChoice::S0212) if use_corridor => {
            corridor_search(w, &req, coarse_zone, u.zone, start_end, dest_end)
        }
        (None, PathfinderChoice::S0212) => S0212.find(w, &req),
    };
    let Some(steps) = found else {
        return Search::SearchFailed;
    };
    let k = u.threat_avoidance;
    let q = r.quirks;
    // spec: S-0212 claim 16 (quirks.train_exclusion_search_only off: straightening also treats the cells behind a train as not enterable)
    let excluded = if u.train && !q.train_exclusion_search_only {
        behind_cells(w, id, from)
    } else {
        Vec::new()
    };
    // The height level of the start pair (S-0212 claim 7).
    let reference = start_pair(w, id, from).1;
    // The search layer of each cell the found path passes through (claim 7).
    let mut layers: BTreeMap<CellXY, Layer> = BTreeMap::new();
    {
        let mut c = from;
        let mut lvl = reference;
        for &d in &steps {
            let (sx, sy) = STEPS[usize::from(d % 8)]; // spec: S-0212 Terms (step directions 0 to 7)
            c = CellXY::new(c.x + sx, c.y + sy);
            let l = candidate_layer(w, lvl, c);
            layers.insert(c, l);
            lvl = pair_level(w, c, l);
        }
    }
    let deck_at = |c: CellXY| layers.get(&c).copied().unwrap_or(Layer::Ground);
    let mode = r.mode;
    let owner = o.owner;
    // trunc₅₃(t × k) for the threat value t of cell `c` (S-0212 F7): the
    // record of the unit's owner (claim 16a; none without an owner).
    let tk = |c: CellXY| {
        let t = owner.map_or(0, |h| crate::threat::value(w, h, c));
        F64::from_i64(i64::from(t), mode).mul(k, mode)
    };
    // Whether the steps `shape` from cell `s` enter only clear cells without
    // a congestion mark, each passing `threat_ok` (S-0212 claims 17 and 18).
    let enters_clear = |s: CellXY,
                        shape: &mut dyn Iterator<Item = u8>,
                        threat_ok: &mut dyn FnMut(CellXY) -> bool|
     -> bool {
        let mut prev = s;
        // The height level of the search pair the shape starts from: the
        // unit's own at its start cell, else that of the ground or deck the
        // path reached (S-0212 claim 7).
        let mut lvl = if s == from {
            reference
        } else {
            pair_level(w, s, deck_at(s))
        };
        for d in shape {
            let (sx, sy) = STEPS[usize::from(d % 8)]; // spec: S-0212 Terms (step directions 0 to 7)
            let n = CellXY::new(prev.x + sx, prev.y + sy);
            // spec: S-0212 claims 17 and 18 (every cell entered clear and without a congestion mark)
            if step_outcome(w, id, prev, n, lvl) != Outcome::Clear
                || congested.contains(&n)
                || excluded.contains(&n)
                || !threat_ok(n)
            {
                return false;
            }
            lvl = pair_level(w, n, candidate_layer(w, lvl, n));
            prev = n;
        }
        true
    };
    let ok = |path: &[u8], start: usize, d3: u8, j: usize| -> bool {
        // The cell where the 2j new steps start.
        let mut c = from;
        for &s in &path[..start] {
            let (sx, sy) = STEPS[usize::from(s % 8)]; // spec: S-0212 Terms (step directions 0 to 7)
            c = CellXY::new(c.x + sx, c.y + sy);
        }
        // spec: S-0212 claim 17 (every cell the 2j new steps pass through has threat value × k below 1.0, F7, the value read at the cell where the new steps start; quirks.flatten_threat_start_cell off: each cell's own)
        let mut below = |n: CellXY| {
            let at = if q.flatten_threat_start_cell { c } else { n };
            tk(at).lt(crate::consts::ONE)
        };
        // spec: S-0212 claim 17 (2j new steps)
        enters_clear(c, &mut std::iter::repeat_n(d3, 2 * j), &mut below)
    };
    let steps = flatten(&steps, q.flatten_straight_run_v, &ok);
    // spec: S-0212 claim 18 and F7 (threat counted only when k exceeds the binary64 value nearest 0.00001)
    let counts = k.gt(crate::consts::THREAT_COEFFICIENT_MIN);
    let ok18 = |s: CellXY, shape: &[u8], limit: u32| {
        let mut n = 0u32;
        let mut within = |c: CellXY| {
            // spec: S-0212 claim 18 (the count of entered cells with threat value × k ≥ the binary64 value nearest 0.01, each cell's own value, stays within the limit)
            if counts && tk(c).ge(crate::consts::THREAT_COUNTED) {
                n += 1;
            }
            n <= limit
        };
        enters_clear(s, &mut shape.iter().copied(), &mut within)
    };
    let received: Vec<PathStep> = steps.iter().map(|&d| PathStep::Dir(d)).collect();
    // spec: S-0212 claim 18 (early detours are shortened, after claim 17)
    let steps: Vec<u8> = shorten(
        &received,
        from,
        q.detour_mirrored_reference,
        q.detour_after_tunnel_position,
        &ok18,
    )
    .into_iter()
    .filter_map(|p| match p {
        PathStep::Dir(d) => Some(d),
        PathStep::Tunnel { .. } => None,
    })
    .collect();
    if let Some(ob) = w.obj_mut(id) {
        ob.path.steps = steps.into_iter().take(STORED).collect();
        ob.path.for_goal = Some(goal);
        // spec: S-0417 claim 17 (No path: a found path gives the walker movement in progress again)
        ob.path.failed_for = None;
    }
    Search::Found
}

/// The final nearby-cell search of the house-selected destination after
/// give-up (S-0212 claim 23a, table T5) around `centre`. It takes no draw.
pub fn give_up_destination(w: &mut World, id: ObjectId, centre: CellXY) -> Option<CellXY> {
    let ground = w.obj(id)?.pos.cell();
    regions::ensure(w);
    // spec: S-0212 claim 23a and T5 (the reach region of the unit's ground cell for normal)
    let reach = regions::reach_read(w, ground, MovementZone::Normal, false);
    nearby(
        w,
        centre,
        // spec: S-0212 T5 (house-selected destination after give-up: track, normal, never in deck mode, no height condition, no further test, no reference cell)
        NearbyUse::Settings(NearbySettings {
            class: Some(LocomotionClass::Track),
            zone: MovementZone::Normal,
            reach: Some(reach),
            deck: false,
            height: false,
            reference: None,
            no_bridges: false,
            no_overlay: false,
        }),
    )
}

/// Whether a change of the unit's destination ends its running retry
/// delay: for the driving and walk locomotors (S-0212 claim 21). The ship
/// locomotor and every other one keep it (claim 21's carve-outs and the
/// state table, row *destination changes*, low confidence); the hovering
/// locomotor's delay is S-0503's ([`crate::hover`]).
fn ends_at_change(w: &World, id: ObjectId) -> bool {
    // spec: S-0212 claim 21 (for the driving and walk locomotors a running retry delay ends when the destination is changed to a different one or cleared)
    matches!(
        crate::lift::current_locomotor(w, id),
        Some(Locomotor::Drive | Locomotor::Walk)
    )
}

/// Starts the retry delay *R* at tick `t` (S-0212 claim 21), tied to the
/// destination the unit holds now ([`PathState::retry_for`]).
pub fn start_delay(ob: &mut crate::world::Object, t: i32, r: i32) {
    ob.path.retry_at = t.wrapping_add(r);
    ob.path.retry_for = ob.dest;
}

/// Whether a retry delay runs for `id` on this tick (S-0212 claim 21): its
/// tick is not reached and, for the driving and walk locomotors, the unit
/// still holds the destination the delay started with. Reads state only.
pub fn delay_runs(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if w.tick >= o.path.retry_at {
        return false;
    }
    // spec: S-0212 claim 21 (a destination changed to a different one or cleared ends the delay, a delay started earlier in the same tick included)
    o.path.retry_for.is_none() || o.dest == o.path.retry_for || !ends_at_change(w, id)
}

/// [`delay_runs`], ending a delay that a change of destination has ended
/// (S-0212 claim 21) so that it stays over. Every fresh search and re-path
/// of the driving and walk locomotors asks this before it waits.
pub fn waits(w: &mut World, id: ObjectId) -> bool {
    let runs = delay_runs(w, id);
    let t = w.tick;
    if !runs
        && let Some(ob) = w.obj_mut(id)
        && ob.path.retry_at > t
    {
        // spec: S-0212 claim 21 (the delay ended at the destination's change)
        ob.path.retry_at = t;
    }
    runs
}

/// What a change of `id`'s destination from `before` does to its running
/// delays. The retry delay (S-0212 claim 21 and its state table, row
/// *destination changes*): for the driving and walk locomotors a different
/// destination, or none, ends it, a delay started earlier in the same tick
/// included. For the walk locomotor `given` ends it too when the
/// destination is the one it already holds (r98, Q-2896); the driving
/// locomotor keeps it then (claim 21's Open questions). The other
/// locomotors keep it. The blockage delay of a walker (S-0209 claim 10):
/// `given` (an assignment was made) starts it afresh, the destination the
/// walker already holds included; without `given` only a destination that
/// differs from `before` does. Called with `given` where a destination
/// reaches the locomotor ([`crate::infantry::destination_assigned`]), and
/// without it after a failed search that may keep the destination
/// ([`failed`]); [`waits`] ends the retry delay too for a destination
/// written elsewhere, and the walker's wait compares
/// [`PathState::blocked_for`].
pub fn destination_changed(w: &mut World, id: ObjectId, before: Option<Target>, given: bool) {
    let t = w.tick;
    if !ends_at_change(w, id) {
        return;
    }
    let walker = crate::lift::current_locomotor(w, id) == Some(Locomotor::Walk);
    if walker
        && let Some(ob) = w.obj_mut(id)
        && (given || ob.dest != before)
    {
        // spec: S-0209 claim 10 (r76, Q-2822: a walker given a destination, by an order, a scatter or any other rule, starts the blockage delay afresh at every assignment that is made, whether the destination is a different one or the one it already has, the same cell or the same object; one whose destination is cleared does not wait) and S-0208 claim 6 (the scatter restarts the blockage delay; S-0209 claim 10 r76: a scatter whose cell is already the walker's destination too)
        ob.path.blocked_since = None;
    }
    if let Some(ob) = w.obj_mut(id)
        && (ob.dest != before || (walker && given && ob.dest.is_some()))
        && ob.path.retry_at > t
    {
        // spec: S-0212 claim 21 (a destination changed to a different one or cleared ends a running retry delay for the driving and walk locomotors, a delay started earlier in the same tick included)
        // spec: S-0212 claim 21 (r98, Q-2896: for the walk locomotor the destination it already holds, the same cell or the same object, given by an order, a scatter or any other rule at every assignment S-0209 claim 10 says is made, ends a running retry delay too, a delay started earlier in the same tick included; the other locomotors keep it, Open questions)
        ob.path.retry_at = t;
    }
}

/// The tick a running retry delay of `id` ends at, if one runs
/// ([`delay_runs`]); with [`keep_delay`], for an assignment whose new
/// destination leaves a running delay as it was.
pub fn running_delay(w: &World, id: ObjectId) -> Option<i32> {
    if !delay_runs(w, id) {
        return None;
    }
    w.obj(id).map(|o| o.path.retry_at)
}

/// Puts back a delay [`running_delay`] read before an assignment, now tied
/// to the new destination: a harvester's offer of its docking cell leaves a
/// running delay running (S-0212 claim 21 and its state table, which leave
/// those offers to S-0402 claim 10a; S-0402 claim 10a: the path computation
/// on such an offer waits for a running retry delay like any fresh search).
pub fn keep_delay(w: &mut World, id: ObjectId, held: Option<i32>) {
    let Some(at) = held else { return };
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0402 claim 10a (an offer of the docking cell waits for a running retry delay) and S-0212 claim 21 (the docking offers left to that claim)
        ob.path.retry_at = at;
        ob.path.retry_for = ob.dest;
    }
}

/// Whether a failed search of unit `id` gives its move up (S-0212 claim 23
/// row 2): its destination is not within one cell of it, or it is not on a
/// bridge deck while the destination is a bridge cell.
fn search_gives_up(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let goal = goal_cell(w, id);
    let near = goal.is_some_and(|g| {
        let c = o.pos.cell();
        (g.x - c.x).abs().max((g.y - c.y).abs()) <= 1
    });
    // spec: S-0212 claim 23 row 2 (or not on a bridge deck while the destination is a bridge cell)
    let off_deck = o.layer != Layer::Deck && goal.is_some_and(|g| w.high_bridge(g));
    // spec: S-0212 claim 23 row 2 (a unit gives up when its destination is not within one cell of it)
    !near || off_deck
}

/// What follows a failed path computation (S-0212 claims 21 and 23).
/// `repath`: the computation was a re-path for a blocked step.
pub fn failed(w: &mut World, id: ObjectId, res: Search, repath: bool) {
    let t = w.tick;
    let Some(o) = w.obj(id).cloned() else { return };
    let r = w.rules_arc();
    let walker = o.walk.is_some();
    // spec: S-0212 claim 23 (the same clearing ends every failed search of a unit with the driving, ship or hovering locomotor)
    let drive = o.drive.is_some() || o.hover.is_some();
    let mut gave_up = false;
    let mut house_given = false;
    match res {
        Search::Found => {}
        Search::GateFailed => {
            // spec: S-0212 claim 23 (gate row: a unit with the driving, ship or walk locomotor has its destination cleared; for the walk locomotor after a fresh search and after a re-path for a moving blocker, and its movement in progress ends in the same movement update)
            // spec: S-0417 claim 17 (No path: a walking unit whose fresh search fails at the gate has its destination cleared in the same movement update)
            if let Some(ob) = w.obj_mut(id) {
                ob.path.steps.clear();
                // spec: S-0212 claim 21 (the delay runs after a fresh search, and after a walker's re-path; for the driving and walk locomotors the clearing below ends it again in the same update, so none remains)
                if !repath || walker {
                    start_delay(ob, t, r.general.retry_delay);
                }
                if drive || walker {
                    ob.dest = None;
                }
            }
        }
        Search::SearchFailed => {
            // spec: S-0212 claim 23 row 2 and Departure D5 (guard for a human-controlled owner, AI houses included; the neutral and special houses keep area guard)
            let human = o.owner.is_none_or(|h| w.houses[usize::from(h.0)].human);
            let give_up = if human {
                Behaviour::Guard
            } else {
                Behaviour::AreaGuard
            };
            let gives_up = search_gives_up(w, id);
            // design: C7-P6 hold (quirks.jammed_move_dropped off: a move that would be given up while only units stand in its way is kept; the unit stops, counts as blocked from now if it was not, and searches again after the retry delay)
            if gives_up && holds(w, id) {
                if let Some(ob) = w.obj_mut(id) {
                    ob.path.steps.clear();
                    start_delay(ob, t, r.general.retry_delay);
                    ob.path.blocked_since = Some(ob.path.blocked_since.unwrap_or(t));
                    if walker {
                        // design: C7-P6 hold (a held walker ends its movement in progress as one that keeps its destination after a failed search does)
                        ob.path.failed_for = ob.dest;
                    }
                }
                crate::trace::record(w, id, || {
                    let after = w.obj(id);
                    crate::trace::What::Failed {
                        result: res,
                        gave_up: false,
                        dest_cleared: false,
                        retry_at: after.map_or(t, |x| x.path.retry_at),
                    }
                });
                return;
            }
            gave_up = gives_up;
            if let Some(ob) = w.obj_mut(id) {
                ob.path.steps.clear();
                // spec: S-0212 claim 23 (the unit stops and the retry delay R starts, which the clearing of its destination below ends again for the driving and walk locomotors, claim 21)
                start_delay(ob, t, r.general.retry_delay);
                if drive {
                    ob.dest = None;
                }
                if gives_up {
                    // spec: S-0212 claim 23 row 2 and claim 23a (it gives up: its destination is cleared and it queues guard or area guard only, neither move nor a forced start)
                    ob.dest = None;
                    ob.beh.queue(Some(give_up));
                    // spec: S-0309 claim 7 (giving the move up under S-0212 claim 23 row 2 also drops the actor's target; no limited-scan mark, claim 22)
                    crate::spawn::clear_own_target(ob);
                    ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
                }
                if walker {
                    // spec: S-0417 claim 17 (No path: a failed search that passes the gate ends the walking unit's movement in progress in that update, whether or not it keeps its destination; S-0212 claim 23, the sentence after its table)
                    ob.path.failed_for = ob.dest;
                }
            }
            if gives_up && !human {
                // spec: S-0212 claim 23a (a neutral or special house's unit gets the house-selected destination on the same tick: a nearby-cell result around the house's centre)
                if let Some(c) = crate::seams::give_up_centre(w, id) {
                    let dest = give_up_destination(w, id, c);
                    if let Some(ob) = w.obj_mut(id) {
                        ob.dest = dest.map(Target::Cell);
                        house_given = dest.is_some();
                    }
                }
            }
        }
    }
    // spec: S-0212 claims 21 and 23 (for the driving and walk locomotors the failure's clearing, or the new destination of claim 23a, ends the delay it started: no delay remains after a failed gate or a failure that clears the destination; a walker that keeps its destination, row 3, waits for R)
    // spec: S-0209 claim 10 (r76: only claim 23a's destination is an assignment here; a walker that keeps its destination, row 3, is given none, so its blockage delay runs on) and S-0212 claim 21 (r98: and its retry delay runs on too, unless a later assignment ends it)
    destination_changed(w, id, o.dest, house_given);
    // spec: S-0309 claim 22 (a route failure of a vehicle with the driving locomotor that does not give the move up, the vehicle standing still after it, S-0212 claim 23: a held target not within reach, claim 23, is dropped and the limited-scan mark set; the hovering and ship locomotors' conditions were not examined, Open questions, so they set none here)
    if res != Search::Found
        && !gave_up
        && o.kind == Kind::Vehicle
        && crate::lift::current_locomotor(w, id) == Some(Locomotor::Drive)
    {
        let held = w.obj(id).and_then(|x| x.target);
        if held.is_some()
            && !crate::combat::acquire::within_reach(w, id, held)
            && let Some(ob) = w.obj_mut(id)
        {
            crate::spawn::clear_own_target(ob);
            ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
            ob.combat.limited_scan = true;
        }
    }
    if res != Search::Found {
        crate::trace::record(w, id, || {
            let after = w.obj(id);
            crate::trace::What::Failed {
                result: res,
                gave_up,
                dest_cleared: after.is_none_or(|x| x.dest.is_none()),
                retry_at: after.map_or(t, |x| x.path.retry_at),
            }
        });
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
