//! Skirmish start positions (S-0009 claims 17 to 20 and 33): start lists
//! with their random start cells, and the assignment of positions to
//! houses, in the early and late phases of map loading.

use glasnost_map::{CellXY, Header, Map};
use glasnost_num::geom::root_floor;

use crate::load::LoadError;
use crate::path::{self, StartAreas};
use crate::world::{HouseId, HouseRole, World};

/// Start waypoint markers 0 … 7 (S-0009 Terms).
const START_WAYPOINTS: u16 = 8; // spec: S-0009 Constants (start waypoint markers)

/// Failed random start candidates after which our engine refuses the game
/// (S-0009 claim 18: a limit of our engine).
const CANDIDATE_LIMIT: u32 = 1000; // spec: S-0009 Constants (failed random start candidates before our engine gives up)

/// The random start margin (S-0009 Formulas, *Random start candidate*).
const MARGIN: i32 = 10; // spec: S-0009 Constants (random start margin 10 cells)

/// The number of players P of a game: every house that is a player
/// (S-0009 claim 17; our lobby offers no observer rows).
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

/// Start positions that carry a mark, 0 … 15 (S-0009 claim 29).
pub const MARKED_POSITIONS: usize = 16; // spec: S-0009 Constants (start positions that carry a mark 16)

/// The mark of a free position (S-0009 claim 29).
pub const FREE: i32 = -1; // spec: S-0009 Constants (free mark −1)

/// The first header pair that acts as marks (S-0009 claim 29).
const FIRST_MARK_PAIR: i32 = 9; // spec: S-0009 Constants (header pairs that act as marks 9 … min(N_h, 16))

/// The last header pair our engine applies (S-0009 claim 29: pairs 17 and
/// beyond are unspecified; our engine ignores them, a departure).
const LAST_MARK_PAIR: i32 = 16; // spec: S-0009 Constants (header pairs that act as marks 9 … min(N_h, 16))

/// Whether the map's header pairs act as start position marks, and the
/// References rule reads the marks: a header count of 9 or more (S-0009
/// claim 29).
pub fn marks_apply(header: &Header) -> bool {
    header.count >= FIRST_MARK_PAIR
}

/// The marks of a game before any phase: every position free.
pub fn free_marks() -> Vec<i32> {
    vec![FREE; MARKED_POSITIONS]
}

/// Applies header pairs 9 … min(N_h, 16) that the header holds to `marks`
/// (S-0009 claim 29): pair i sets positions 2(i − 9) and 2(i − 9) + 1; an
/// absent pair, or an unread component, leaves the mark as it stands. A
/// mark at a position numbered `n` or above is kept for the References
/// rule, but no house holds a position by it (a departure, noted).
pub fn apply_pairs(w: &mut World, header: &Header, marks: &mut [i32], n: usize) {
    for i in FIRST_MARK_PAIR..=header.count.min(LAST_MARK_PAIR) {
        let Some(Some(pair)) = header.pairs.get(i as usize - 1) else {
            continue;
        };
        for (j, c) in pair.iter().enumerate() {
            // spec: S-0009 claim 29 (pair i marks positions 2(i − 9) and 2(i − 9) + 1)
            let pos = 2 * (i - FIRST_MARK_PAIR) as usize + j;
            let Some(m) = *c else { continue };
            let Some(slot) = marks.get_mut(pos) else {
                continue;
            };
            // spec: S-0009 claim 29 (References read the mark of each of positions A … H whatever N is; Q-3215's answer)
            *slot = m;
            if pos >= n && m != FREE {
                // spec: S-0009 claim 29 (a house that holds a position numbered N or above: unspecified; our engine ignores the mark for which position the house holds, a departure; the References rule still reads it)
                w.note("S-0009 claim 29: a header pair marks a start position numbered N or above; no house holds a position by it, and the References rule still reads it (a departure)");
            }
        }
    }
}

/// House `h`'s current place (S-0009 Terms): none for a house the
/// unused-house cleanup removed; otherwise its house number less the number
/// of removed houses before it in house order, or, with
/// `quirks.cleanup_shifts_start_marks` off, its house number.
pub fn current_place(w: &World, h: usize) -> Option<i32> {
    let house = w.houses.get(h)?;
    // spec: S-0009 claim 29 (After the cleanup: a house that the cleanup removes has no current place from then on)
    if house.removed {
        return None;
    }
    let shift = if w.rules().quirks.cleanup_shifts_start_marks {
        // spec: S-0009 Terms (current place: the house number minus the houses before it in house order that the cleanup removed) and claim 29 (After the cleanup, quirks.cleanup_shifts_start_marks)
        w.houses[..h].iter().filter(|x| x.removed).count()
    } else {
        // spec: S-0009 Edge cases (quirks.cleanup_shifts_start_marks off: current places never change)
        0
    };
    i32::try_from(h - shift).ok()
}

/// The house whose current place is `m` (S-0009 Terms), if any.
pub fn house_at_place(w: &World, m: i32) -> Option<usize> {
    (0..w.houses.len()).find(|&h| current_place(w, h) == Some(m))
}

/// What the cleanup leaves in the marks (S-0009 claim 29, *After the
/// cleanup*): nothing changes with `quirks.cleanup_shifts_start_marks` on;
/// off, every mark equal to a removed house's number becomes −1.
pub fn after_cleanup(w: &World, marks: &mut [i32]) {
    // spec: S-0009 claim 29 (After the cleanup: the cleanup changes no mark)
    if w.rules().quirks.cleanup_shifts_start_marks {
        return;
    }
    for (h, house) in w.houses.iter().enumerate() {
        if house.removed {
            // spec: S-0009 Edge cases (quirks.cleanup_shifts_start_marks off: at the cleanup every mark equal to a removed house's house number becomes −1)
            for m in marks.iter_mut().filter(|m| i64::from(**m) == h as i64) {
                *m = FREE;
            }
        }
    }
}

/// The highest-numbered position below `n` whose mark is `place` (S-0009
/// claim 29: a house holds a position whose mark is its current place, and
/// takes the highest-numbered it holds; positions numbered N or above are
/// not held, a departure).
pub fn highest_held(marks: &[i32], n: usize, place: i32) -> Option<u32> {
    (0..n.min(marks.len()))
        .rev()
        .find(|&p| marks[p] == place)
        .map(|p| p as u32)
}

/// Sets the start position of each house of `order` to the highest-numbered
/// position below `n` that it holds by its current place, or none (S-0009
/// claim 29).
pub fn take_held(w: &mut World, order: &[usize], marks: &[i32], n: usize) {
    for &h in order {
        let place = current_place(w, h);
        // spec: S-0009 claim 29 (a non-passive house holding positions takes the highest-numbered, by its current place)
        w.houses[h].start = place.and_then(|c| highest_held(marks, n, c));
    }
}

/// Whether "player at position X" reads the marks (S-0009 claims 21 and
/// 29): on a map whose header count is 9 or more, and on any map once the
/// cleanup of claim 28 has removed a house.
pub fn references_read_marks(w: &World, header: &Header) -> bool {
    // spec: S-0009 claim 21 (on a map whose N_h is 9 or more, and on any map once the cleanup of claim 28 has removed a house, the References rule of claim 29 replaces this one)
    marks_apply(header) || w.houses.iter().any(|h| h.removed)
}

/// "Player at position" `k` (S-0009 claims 21 and 29): with the marks in
/// force, the house whose current place is mark `k`, which may be passive,
/// and none when no house has that place (a removed house has none).
/// Otherwise the house that holds start position `k` and is not removed.
pub fn player_at(w: &World, k: u32) -> Option<HouseId> {
    match &w.start_marks {
        // spec: S-0009 claim 29 (References: the house whose current place equals the mark of position X, which may be a passive house; empty when no house has that place)
        Some(marks) => {
            let m = *marks.get(k as usize)?;
            house_at_place(w, m).map(|h| HouseId(h as u8))
        }
        // spec: S-0009 claim 21 (the house holding start position X)
        None => w
            .houses
            .iter()
            .position(|h| !h.removed && h.start == Some(k))
            .map(|i| HouseId(i as u8)),
    }
}

/// Gives each house of `order` that holds no position one, in that order
/// (S-0009 claim 19), against `list`: one ranged draw over 0 … N − 1 when
/// no position is taken, the farthest-sum rule otherwise. A position is
/// taken when its mark is not −1 (claim 29), and a position given out
/// carries the house's current place. A house that finds every position
/// below N taken gets none (claim 29, a departure).
pub fn assign(w: &mut World, order: &[usize], list: &[CellXY], marks: &mut [i32]) {
    let n = list.len().min(marks.len());
    let q = w.rules().quirks;
    for &h in order {
        if w.houses[h].start.is_some() {
            continue;
        }
        // spec: S-0009 claim 28 (a removed house takes no position)
        let Some(place) = current_place(w, h) else {
            continue;
        };
        // spec: S-0009 claim 29 (every position below N whose mark is not −1 counts as taken)
        let taken: Vec<usize> = (0..n).filter(|&p| marks[p] != FREE).collect();
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
        match pos {
            // spec: S-0009 claim 29 (the position a house is given carries its current place)
            Some(p) => marks[p] = place,
            // spec: S-0009 claim 29 (every position below N taken: unspecified; our engine gives the house no start position and so no starting units, a departure)
            None => w.note("S-0009 claim 29: every start position below N is taken; a house gets no start position and no starting units (a departure)"),
        }
        w.houses[h].start = pos.map(|p| p as u32);
    }
}
