//! The corridor of S-0212 (claims 13, 13a, 13b and 14): sectors and their
//! connections, derived from the regions; the coarse search that picks a
//! chain of sectors at each level; and the retry after a failed cell search
//! inside a corridor.
//!
//! Sectors are derived data, rebuilt with the regions ([`crate::regions`]),
//! never saved. Enabled bridge links ([`crate::bridges`]) add their
//! connections; the coarse end cells of bridge cells are
//! [`crate::path`]'s.

use std::collections::{BTreeMap, BTreeSet, BinaryHeap};

use glasnost_map::CellXY;
use glasnost_num::{F32, F64};
use glasnost_rules::sim::MovementZone;

use crate::occupancy::{self, Outcome};
use crate::passability::{self, Answer, category};
use crate::regions::{Dsu, STEPS};
use crate::rules::Kind;
use crate::world::{ObjectId, World};

/// No sector.
pub const NONE: u32 = u32::MAX; // nocite: a marker of our own for "no sector", not behaviour

/// The levels of sectors (S-0212 Constants, corridor levels).
// spec: S-0212 Constants (corridor levels: 3)
pub const LEVELS: usize = 3;

/// The block size of level `l` (S-0212 Terms: 2^(L+1) cells).
pub fn block_size(l: usize) -> i32 {
    // spec: S-0212 Terms (block of level L: 2^(L+1) × 2^(L+1) cells)
    2 << l
}

/// The sectors of one level.
#[derive(Clone, Debug, Default)]
pub struct SectorLevel {
    /// The sector of each cell, by dense cell index ([`NONE`]: none).
    pub of_cell: Vec<u32>,
    /// The parent of each sector (the sector of the next level that holds
    /// it; [`NONE`] at the top level).
    pub parent: Vec<u32>,
    /// The category of each sector (its region's).
    pub category: Vec<u8>,
    /// The origin of each sector's block.
    pub block: Vec<CellXY>,
    /// The connections of each sector: (other sector, diagonal), ordered by
    /// the other sector, whose number follows its first cell (claim 12a).
    pub conns: Vec<Vec<(u32, bool)>>,
}

impl SectorLevel {
    /// The sector of a cell.
    pub fn sector(&self, w: &World, c: CellXY) -> u32 {
        w.cells
            .idx(c)
            .and_then(|i| self.of_cell.get(i).copied())
            .unwrap_or(NONE)
    }
}

/// The sectors of every level (derived from the regions).
#[derive(Clone, Debug, Default)]
pub struct Sectors {
    /// Levels 0 (finest) to 2.
    pub levels: Vec<SectorLevel>,
}

fn level_of(w: &World, c: CellXY) -> i32 {
    w.cells.cell(c).map_or(0, |x| i32::from(x.level))
}

fn block_of(c: CellXY, b: i32) -> (i32, i32) {
    (c.x.div_euclid(b), c.y.div_euclid(b))
}

/// Builds the sectors and connections of S-0212 claim 13b from the regions.
pub fn build(w: &World) -> Sectors {
    let region = &w.regions.region;
    // Cells of a region, row by row (increasing y, then x): sectors are
    // numbered in the order of their first cells (claim 12a).
    let mut cells: Vec<(CellXY, usize)> = w
        .cells
        .grid
        .iter()
        .filter_map(|(c, _)| w.cells.idx(c).map(|i| (c, i)))
        // spec: S-0212 claim 13b (off-map cells belong to no sector)
        .filter(|&(_, i)| region.get(i).is_some_and(|&r| r != u32::MAX))
        .collect();
    cells.sort_by_key(|&(c, _)| (c.y, c.x));
    let n = w.cells.grid.len();
    let mut levels: Vec<SectorLevel> = Vec::with_capacity(LEVELS);
    for l in 0..LEVELS {
        let b = block_size(l);
        let mut dsu = Dsu::new(n);
        for &(c, i) in &cells {
            for (dx, dy) in STEPS {
                let nc = CellXY::new(c.x + dx, c.y + dy);
                let Some(j) = w.cells.idx(nc) else { continue };
                // spec: S-0212 claim 13b (one region, one block, neighbours at most 1 level apart; no join across 2 or 3 levels)
                if region[j] == region[i]
                    && block_of(c, b) == block_of(nc, b)
                    && (level_of(w, c) - level_of(w, nc)).abs() <= 1
                {
                    dsu.union(i as u32, j as u32);
                }
            }
        }
        let mut lv = SectorLevel {
            of_cell: vec![NONE; n],
            ..Default::default()
        };
        let mut label: BTreeMap<u32, u32> = BTreeMap::new();
        for &(c, i) in &cells {
            let root = dsu.find(i as u32);
            let next = lv.category.len() as u32;
            let id = *label.entry(root).or_insert(next);
            if id == next {
                // spec: S-0212 claim 13b (a sector's category is its region's)
                lv.category.push(w.cells.category[i]);
                let (bx, by) = block_of(c, b);
                lv.block.push(CellXY::new(bx * b, by * b));
            }
            lv.of_cell[i] = id;
        }
        // Connections: true while some qualifying pair is not a diagonal pair
        // across a block's left or right edge.
        let mut straight: BTreeMap<(u32, u32), bool> = BTreeMap::new();
        for &(c, i) in &cells {
            let s = lv.of_cell[i];
            if !w.cells.usable(c) {
                continue;
            }
            for (d, (dx, dy)) in STEPS.iter().enumerate() {
                let nc = CellXY::new(c.x + dx, c.y + dy);
                let Some(j) = w.cells.idx(nc) else { continue };
                let t = lv.of_cell[j];
                // spec: S-0212 claim 13b (connected: neighbouring cells, both inside the usable area, at most 1 level apart, whatever the categories)
                if t == NONE
                    || t == s
                    || !w.cells.usable(nc)
                    || (level_of(w, c) - level_of(w, nc)).abs() > 1
                {
                    continue;
                }
                // spec: S-0212 claim 13b (a diagonal pair whose cells lie in different block columns crosses a left or right edge on the slant)
                let slant = d % 2 == 1 && block_of(c, b).0 != block_of(nc, b).0;
                *straight.entry((s, t)).or_insert(false) |= !slant;
            }
        }
        // spec: S-0212 claim 13b (an enabled link adds straight connections at every level, with no height or usable-area condition)
        for link in w.cells.links.iter().filter(|x| x.enabled) {
            for (a, b) in link_pairs(w, link) {
                let s = lv.sector(w, a);
                let t = lv.sector(w, b);
                if s == NONE || t == NONE || s == t {
                    continue;
                }
                // spec: S-0212 claim 13b (two sectors share at most one connection: joined by a link, it is straight)
                straight.insert((s, t), true);
                straight.insert((t, s), true);
            }
        }
        lv.conns = vec![Vec::new(); lv.category.len()];
        for (&(s, t), &st) in &straight {
            // spec: S-0212 claim 13b (diagonal when every such pair crosses on the slant, straight otherwise)
            lv.conns[s as usize].push((t, !st));
        }
        levels.push(lv);
    }
    // spec: S-0212 claim 13b (each sector lies inside one sector of the next level, its parent)
    for l in 0..LEVELS - 1 {
        let mut parent = vec![NONE; levels[l].category.len()];
        for &(_, i) in &cells {
            let s = levels[l].of_cell[i];
            parent[s as usize] = levels[l + 1].of_cell[i];
        }
        levels[l].parent = parent;
    }
    levels[LEVELS - 1].parent = vec![NONE; levels[LEVELS - 1].category.len()];
    Sectors { levels }
}

/// The two side displacements of S-0212 claim 13b's table for the place
/// *i* of the first end's tile in its bridge or wooden-bridge role.
// nocite: the array length is the claim's two sides, not a behavioural number
pub fn side_displacements(i: i32) -> Option<[(i32, i32); 2]> {
    // spec: S-0212 claim 13b (table: 0, 1, 6 to 10 map north then south; 3, 4, 11 to 15 map east then west; 2, 5 map north-west then south-east)
    match i {
        0 | 1 | 6..=10 => Some([(0, -1), (0, 1)]),
        3 | 4 | 11..=15 => Some([(1, 0), (-1, 0)]),
        2 | 5 => Some([(-1, -1), (1, 1)]),
        _ => None,
    }
}

/// The cell pairs whose sectors an enabled bridge link connects (S-0212
/// claim 13b): its two end cells, and for each of the two sides the two end
/// cells moved by the same displacement, which the place of the first end's
/// tile in its bridge role alone decides. A first end whose tile is in
/// neither role adds no side pairs (the claim gives side cells only by that
/// tile).
fn link_pairs(w: &World, l: &crate::cells::BridgeLink) -> Vec<(CellXY, CellXY)> {
    // spec: S-0212 claim 13b (the sectors of the two end cells)
    let mut out = vec![(l.first, l.second)];
    let place = match (w.cells.cell(l.first), w.catalogue.as_ref()) {
        (Some(c), Some(cat)) => glasnost_map::bridges::role_index(&cat.roles, c.tile),
        _ => None,
    };
    if let Some(sides) = place.and_then(side_displacements) {
        let by = |c: CellXY, (dx, dy): (i32, i32)| CellXY::new(c.x + dx, c.y + dy);
        for d in sides {
            // spec: S-0212 claim 13b (each side: the first end plus its displacement to the second end plus the same displacement, exact integer cells)
            out.push((by(l.first, d), by(l.second, d)));
        }
    }
    out
}

/// Connections closed by the retries of one path computation, per level,
/// as unordered sector pairs.
pub type Closed = [BTreeSet<(u32, u32)>; LEVELS];

fn pair(a: u32, b: u32) -> (u32, u32) {
    (a.min(b), a.max(b))
}

/// The entry cost κ of a sector's category (S-0212 table T3).
fn kappa(cat: u8) -> i64 {
    // spec: S-0212 table T3 (open 1, crushable 0, wall-like 0, beach 1, water 1, partial obstacle 0, blocked 1, off-map 1)
    match cat {
        category::CRUSHABLE | category::WALL_LIKE | category::PARTIAL => 0,
        _ => 1,
    }
}

/// A pending route of the coarse search: ordered by cost, then by
/// recording (claim 12a).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Route {
    cost: u32,
    seq: u32,
}

impl Ord for Route {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Min-heap on (cost, seq).
        (other.cost, other.seq).cmp(&(self.cost, self.seq))
    }
}

impl PartialOrd for Route {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// The coarse search of one level (S-0212 claim 13, F9): the chain of
/// sectors from `start` to `dest`, or `None`.
fn coarse_level(
    w: &World,
    lv: &SectorLevel,
    start: u32,
    dest: u32,
    allowed: &dyn Fn(u32) -> bool,
    closed: &BTreeSet<(u32, u32)>,
) -> Option<Vec<u32>> {
    let r = w.rules();
    let mode = r.mode;
    let keep_cost = r.quirks.coarse_limit_records_cost;
    // spec: S-0212 claim 13 (when start and destination lie in the same sector, the corridor is that sector)
    if start == dest {
        return Some(vec![start]);
    }
    let mut recorded: Vec<Option<F32>> = vec![None; lv.category.len()];
    // Every route: (sector, its predecessor route).
    let mut routes: Vec<(u32, u32)> = Vec::new();
    let mut heap: BinaryHeap<Route> = BinaryHeap::new();
    recorded[start as usize] = Some(F32::ZERO);
    routes.push((start, NONE));
    // spec: S-0212 claim 12a (the route to the start's sector is recorded first)
    heap.push(Route { cost: 0, seq: 0 });
    while let Some(rt) = heap.pop() {
        let (s, _) = routes[rt.seq as usize];
        if s == dest {
            // spec: S-0212 claim 13 (the level's search ends when the pending route taken next is a route to the destination's sector; its chain is the corridor)
            let mut chain = Vec::new();
            let mut k = rt.seq;
            while k != NONE {
                let (sec, pred) = routes[k as usize];
                chain.push(sec);
                k = pred;
            }
            chain.reverse();
            return Some(chain);
        }
        let cq = F32(rt.cost);
        // spec: S-0212 claim 12a (new routes recorded in the order of the sectors they enter, by first cell)
        for &(t, diagonal) in &lv.conns[s as usize] {
            // spec: S-0212 claim 13 (connections closed by a retry are not used)
            if closed.contains(&pair(s, t)) || !allowed(t) {
                continue;
            }
            // spec: S-0212 F9 (cost(s) = trunc₂₄(trunc₅₃(trunc₅₃(κ + cost(q)) + T) + λ); no threat records exist before M4, so T = 0)
            let lambda = if diagonal {
                crate::consts::DIAGONAL_CONNECTION
            } else {
                F64::ZERO
            };
            let cost = F64::from_i64(kappa(lv.category[t as usize]), mode)
                .add(cq.to_f64(), mode)
                .add(F64::ZERO, mode)
                .add(lambda, mode)
                .to_f32(mode);
            let better = match recorded[t as usize] {
                None => true,
                // spec: S-0212 F9 (a reached sector takes a new route only when strictly cheaper, an exact comparison)
                Some(old) => cost.to_f64().lt(old.to_f64()),
            };
            if !better {
                continue;
            }
            // spec: S-0212 claim 13 (at most 9999 routes pending)
            if heap.len() >= 9999 {
                // spec: S-0212 claim 13 (a route found at the limit is never pending; quirks.coarse_limit_records_cost)
                if keep_cost {
                    recorded[t as usize] = Some(cost);
                }
                continue;
            }
            recorded[t as usize] = Some(cost);
            let seq = routes.len() as u32;
            routes.push((t, rt.seq));
            heap.push(Route { cost: cost.0, seq });
        }
    }
    None
}

/// The corridor of every level, level 0 first.
pub type Chains = [Vec<u32>; LEVELS];

/// The coarse search of S-0212 claim 13 at every level, from the top level
/// down, for movement zone `zone`, from cell `start` to cell `dest` (the
/// coarse end cells).
pub fn coarse(
    w: &World,
    secs: &Sectors,
    zone: MovementZone,
    start: CellXY,
    dest: CellXY,
    closed: &Closed,
) -> Option<Chains> {
    let mut chains: Chains = Default::default();
    let mut upper: BTreeSet<u32> = BTreeSet::new();
    for l in (0..LEVELS).rev() {
        let lv = secs.levels.get(l)?;
        let s = lv.sector(w, start);
        let t = lv.sector(w, dest);
        if s == NONE || t == NONE {
            return None;
        }
        let top = l == LEVELS - 1;
        let allowed = |x: u32| {
            let cat = lv.category[x as usize];
            // spec: S-0212 claim 13 (on a corridor only with zone answer yes; below level 2 only with the parent on the corridor above, or crushable)
            passability::zone_table(zone, cat) == Answer::Yes
                && (top || upper.contains(&lv.parent[x as usize]) || cat == category::CRUSHABLE)
        };
        if !allowed(s) || !allowed(t) {
            return None;
        }
        let chain = coarse_level(w, lv, s, t, &allowed, &closed[l])?;
        // spec: S-0212 Constants and departure D2 (a chain of more than 500 sectors: a failed coarse search)
        if chain.len() > 500 {
            return None;
        }
        upper = chain.iter().copied().collect();
        chains[l] = chain;
    }
    Some(chains)
}

/// Whether a ground-layer cell has the corridor exemption (S-0212 claim
/// 13): one of its eight neighbours holds a vehicle, an infantry unit, an
/// aircraft, a terrain object or a wall section, on either layer.
pub fn exempt(w: &World, c: CellXY) -> bool {
    let r = w.rules();
    STEPS.iter().any(|&(dx, dy)| {
        let n = CellXY::new(c.x + dx, c.y + dy);
        let Some(cell) = w.cells.cell(n) else {
            return false;
        };
        // spec: S-0212 claim 13 (a wall section in a neighbour)
        let wall = cell.overlay != glasnost_map::NO_OVERLAY
            && r.overlays
                .get(usize::from(cell.overlay))
                .is_some_and(|o| o.wall);
        wall || [crate::world::Layer::Ground, crate::world::Layer::Deck]
            .iter()
            .flat_map(|&l| w.cells.list(n, l).iter())
            // spec: S-0212 claim 13 (a vehicle, an infantry unit, an aircraft or a terrain object in a neighbour, on its ground or its deck)
            .any(|&o| {
                w.obj(o).is_some_and(|x| {
                    matches!(
                        x.kind,
                        Kind::Vehicle | Kind::Infantry | Kind::Aircraft | Kind::Terrain
                    )
                })
            })
    })
}

/// The outcome-or-category condition of S-0212 claim 13a for cell `c`.
fn free_condition(w: &World, unit: ObjectId, zone: MovementZone, c: CellXY) -> bool {
    let Some(i) = w.cells.idx(c) else {
        return false;
    };
    let usable = passability::zone_table(zone, w.cells.category[i]) == Answer::Yes;
    let level = level_of(w, c);
    // spec: S-0212 claim 13a (the entry outcome for the unit at the cell's own height level)
    let clear = match w.obj(unit).map(|o| o.kind) {
        Some(Kind::Infantry) => occupancy::infantry_outcome(w, unit, c, level) == Outcome::Clear,
        Some(Kind::Aircraft) => true,
        _ => occupancy::vehicle_outcome(w, unit, c, level) == Outcome::Clear,
    };
    if w.rules().quirks.corridor_free_part_unusable_cells {
        // spec: S-0212 claim 13a (clear, or a category the unit's own zone cannot use; quirks.corridor_free_part_unusable_cells)
        clear || !usable
    } else {
        // spec: S-0212 claim 13a (switch off: usable and clear)
        clear && usable
    }
}

/// The effect of a failed cell search inside a corridor on level `l`
/// (S-0212 claims 13 and 13a). Closes connections in `closed`; returns
/// `false` when the corridor is abandoned.
#[allow(clippy::too_many_arguments)]
pub fn retry(
    w: &World,
    secs: &Sectors,
    unit: ObjectId,
    zone: MovementZone,
    l: usize,
    progress: CellXY,
    chain: &[u32],
    closed: &mut BTreeSet<(u32, u32)>,
) -> bool {
    let Some(lv) = secs.levels.get(l) else {
        return false;
    };
    let s = lv.sector(w, progress);
    if s == NONE {
        return false;
    }
    let b = block_size(l);
    // The free part of S: from the progress cell, by steps inside S through
    // cells that meet the condition.
    let mut free: BTreeSet<CellXY> = BTreeSet::new();
    let mut todo = vec![progress];
    free.insert(progress);
    let mut borders: BTreeSet<u32> = BTreeSet::new();
    while let Some(c) = todo.pop() {
        for (dx, dy) in STEPS {
            let n = CellXY::new(c.x + dx, c.y + dy);
            let t = lv.sector(w, n);
            if t == NONE || !free_condition(w, unit, zone, n) {
                continue;
            }
            if t == s {
                // spec: S-0212 claim 13a (the free part: reached from the progress cell by steps inside S)
                if free.insert(n) {
                    todo.push(n);
                }
            } else {
                // spec: S-0212 claim 13a (the free part borders a sector whose cell next to it meets the condition)
                borders.insert(t);
            }
        }
    }
    let split = if w.rules().quirks.corridor_split_probe {
        // spec: S-0212 claim 13a (the probe cell (x + x mod B, y + y mod B); split when it is inside, in S, and the free part does not cover the block)
        let probe = CellXY::new(
            progress.x + progress.x.rem_euclid(b),
            progress.y + progress.y.rem_euclid(b),
        );
        w.cells.usable(probe) && lv.sector(w, probe) == s && (free.len() as i32) < b * b
    } else {
        // spec: S-0212 claim 13a (switch off: split when a cell of S inside the usable area is not in the free part)
        let o = lv.block[s as usize];
        (0..b).any(|dy| {
            (0..b).any(|dx| {
                let c = CellXY::new(o.x + dx, o.y + dy);
                lv.sector(w, c) == s && w.cells.usable(c) && !free.contains(&c)
            })
        })
    };
    if !split {
        // spec: S-0212 claim 13 (whole: close the connections from S to every sector the free part does not border)
        for &(t, _) in &lv.conns[s as usize] {
            if !borders.contains(&t) {
                closed.insert(pair(s, t));
            }
        }
        return true;
    }
    let Some(k) = chain.iter().position(|&x| x == s) else {
        // spec: S-0212 claim 13 (split, S not on the corridor: the corridor is abandoned)
        return false;
    };
    // spec: S-0212 claim 13 (split, fewer than 2 sectors: the corridor is abandoned)
    if chain.len() < 2 {
        return false;
    }
    // spec: S-0212 claim 13 (split: close S to the next corridor sector, or the previous one to S when S is last)
    let (a, c) = if k + 1 < chain.len() {
        (s, chain[k + 1])
    } else {
        (chain[k - 1], s)
    };
    closed.insert(pair(a, c));
    // spec: S-0212 claim 13 (and the earlier of the two to every sector adjacent to both)
    let of = |x: u32| -> BTreeSet<u32> { lv.conns[x as usize].iter().map(|&(t, _)| t).collect() };
    let both: Vec<u32> = of(a).intersection(&of(c)).copied().collect();
    for t in both {
        closed.insert(pair(a, t));
    }
    true
}
