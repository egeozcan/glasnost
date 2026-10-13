//! Skirmish start positions (S-0009 claims 17 to 20 and 33): start lists
//! with their random start cells, and the assignment of positions to
//! houses, in the early and late phases of map loading.

use glasnost_map::{CellXY, Map};
use glasnost_num::geom::root_floor;

use crate::load::LoadError;
use crate::path::{self, StartAreas};
use crate::world::{HouseRole, World};

/// Start waypoint markers 0 … 7 (S-0009 Terms).
const START_WAYPOINTS: u16 = 8; // spec: S-0009 Constants (start waypoint markers)

/// Failed random start candidates after which our engine refuses the game
/// (S-0009 claim 18: a limit of our engine).
const CANDIDATE_LIMIT: u32 = 1000; // spec: S-0009 Constants (failed random start candidates before our engine gives up)

/// The random start margin (S-0009 Formulas, *Random start candidate*).
const MARGIN: i32 = 10; // spec: S-0009 Constants (random start margin 10 cells)

/// The number of players P of a game: every house that is a player
/// (S-0009 claim 17; our engine has no observers).
pub fn player_count(w: &World) -> usize {
    w.houses
        .iter()
        .filter(|h| h.role == HouseRole::Player)
        .count()
}

/// One random start cell (S-0009 claim 18): candidates from two ranged
/// draws each, until the start-cell search around one succeeds.
fn random_cell(w: &mut World, areas: &StartAreas) -> Result<CellXY, LoadError> {
    let q = w.rules().quirks;
    let fp = *w.cells.footprint();
    let span = fp.width + fp.height;
    for _ in 0..CANDIDATE_LIMIT {
        let c = if q.random_start_asymmetric_range {
            // spec: S-0009 claim 18, Formulas (u from 0 … W + H − 11, then v from 10 … W + H − 11; candidate (v + 1, u + 11))
            let u = w.rng.ranged(0, span - 11, q.ranged_draw_half_span); // spec: S-0009 Formulas (u ~ U[0, W + H − 11])
            let v = w.rng.ranged(MARGIN, span - 11, q.ranged_draw_half_span); // spec: S-0009 Formulas (v ~ U[10, W + H − 11])
            CellXY::new(v + 1, u + 11) // spec: S-0009 Formulas (candidate (v + 1, u + 11))
        } else {
            // spec: S-0009 Edge cases (quirks.random_start_asymmetric_range off: both from 11 … W + H − 10; the first draw gives y, the second x; bounds in either order when W + H < 21)
            let y = w
                .rng
                .ranged(MARGIN + 1, span - MARGIN, q.ranged_draw_half_span);
            let x = w
                .rng
                .ranged(MARGIN + 1, span - MARGIN, q.ranged_draw_half_span);
            CellXY::new(x, y)
        };
        if let Some(found) = path::start_cell(w, areas, c) {
            return Ok(found);
        }
    }
    // spec: S-0009 claim 18 (after 1000 failed candidates our engine refuses to start the game)
    Err(LoadError::NoStartCell)
}

/// A start list for `players` players (S-0009 claims 17 and 18), built on
/// the cells as they stand, with new random entries and their draws.
pub fn build_list(w: &mut World, map: &Map, players: usize) -> Result<Vec<CellXY>, LoadError> {
    let q = w.rules().quirks;
    // spec: S-0009 claim 17 (K: start waypoint markers defined consecutively from 0, at most 8)
    let mut k: u16 = 0;
    while k < START_WAYPOINTS && map.waypoints.contains_key(&k) {
        k += 1;
    }
    // spec: S-0009 claim 17 (N = max(P, K))
    let n = players.max(usize::from(k));
    let defined = |i: usize| {
        map.waypoints
            .get(&(i as u16))
            .map(|&(x, y)| CellXY::new(i32::from(x), i32::from(y)))
    };
    let mut list: Vec<Option<CellXY>> = if q.start_waypoint_gap {
        // spec: S-0009 claim 17 (the defined waypoints among 0 … N − 1 in order; random entries added until N)
        let mut v: Vec<Option<CellXY>> = (0..n).filter_map(defined).map(Some).collect();
        v.resize(n, None);
        v
    } else {
        // spec: S-0009 Edge cases (quirks.start_waypoint_gap off: position i is waypoint i when defined; N = max(P, K) with K counted as with the switch on; random entries in increasing position order)
        (0..n).map(defined).collect()
    };
    let missing = list.iter().filter(|e| e.is_none()).count();
    if missing > 0 {
        let areas = StartAreas::new(w);
        for e in list.iter_mut() {
            if e.is_none() {
                *e = Some(random_cell(w, &areas)?);
            }
        }
    }
    Ok(list.into_iter().flatten().collect())
}

/// The distance sum of a free position (S-0009 Formulas, *Distance sum*):
/// per taken position, w(⌊r(Δx² + Δy²)⌋) with the differences reduced by w.
fn distance_sum(w: &World, list: &[CellXY], p: usize, taken: &[usize]) -> i64 {
    let q = w.rules().quirks;
    // spec: S-0009 Formulas (w(c) = ((c + 32768) mod 65536) − 32768)
    let wrap = |v: i64| ((v + 32768).rem_euclid(65536)) - 32768;
    taken
        .iter()
        .map(|&t| {
            let dx = wrap(i64::from(list[p].x) - i64::from(list[t].x));
            let dy = wrap(i64::from(list[p].y) - i64::from(list[t].y));
            // spec: S-0009 Formulas (n = Δx² + Δy², exact; the table square root, quirks.table_root_distance)
            let n = (dx * dx + dy * dy) as u64;
            wrap(root_floor(n, q.table_root_distance, w.rules().mode) as i64)
        })
        .sum()
}

/// Gives each house of `order` that holds no position one, in that order
/// (S-0009 claim 19), against `list`: one ranged draw over 0 … N − 1 when
/// no position is taken, the farthest-sum rule otherwise. A house that
/// finds every position below N taken gets none (claim 29, a departure).
pub fn assign(w: &mut World, order: &[usize], list: &[CellXY]) {
    let n = list.len();
    let q = w.rules().quirks;
    for &h in order {
        if w.houses[h].start.is_some() {
            continue;
        }
        let taken: Vec<usize> = w
            .houses
            .iter()
            .filter_map(|x| x.start.map(|s| s as usize))
            .filter(|&s| s < n)
            .collect();
        let pos = if taken.is_empty() {
            if n == 0 {
                None
            } else {
                // spec: S-0009 claim 19 (no position taken: one ranged draw over 0 … N − 1)
                Some(w.rng.ranged(0, n as i32 - 1, q.ranged_draw_half_span) as usize)
            }
        } else {
            // spec: S-0009 claim 19 (the free position with the largest distance sum; ties to the lowest; all below 0: the lowest free, a departure)
            let mut best: Option<(i64, usize)> = None;
            let mut first_free = None;
            for p in (0..n).filter(|p| !taken.contains(p)) {
                first_free.get_or_insert(p);
                let sum = distance_sum(w, list, p, &taken);
                if sum >= 0 && best.is_none_or(|(s, _)| sum > s) {
                    best = Some((sum, p));
                }
            }
            best.map(|b| b.1).or(first_free)
        };
        w.houses[h].start = pos.map(|p| p as u32);
    }
}
