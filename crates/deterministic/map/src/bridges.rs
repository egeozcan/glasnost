//! Bridge spans and the bridge-body mark (S-0010 claim 25 and table B).
//!
//! Once at game start, right after the game-start pass of terrain
//! recalculation, the map's bridge spans are found from the tiles of the
//! bridge role and of the wooden-bridge role, and every low span marks a
//! band of cells with the bridge-body mark. The spans are also the bridge
//! links of S-0212 (claims 3 and 13b): a high span is a link enabled at game
//! start, a low span one that is not.

use serde::{Deserialize, Serialize};

use crate::terrain::Roles;
use crate::{CellGrid, CellXY};

/// Tiles in each of the two bridge roles (S-0010 claim 25).
const ROLE_TILES: i32 = 16; // spec: S-0010 claim 25 (16 tiles each)

/// One bridge span (S-0010 Terms and claim 25).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    /// The start cell.
    pub start: CellXY,
    /// The far end.
    pub end: CellXY,
    /// The direction: east (towards x + 1) or, when false, south (towards
    /// y + 1).
    pub east: bool,
    /// Whether the span is high (every between-cell whose tile lies in
    /// neither role's range is a high-bridge cell).
    pub high: bool,
}

/// A tile's place *i* in the bridge role's range, else in the wooden-bridge
/// role's (S-0010 claim 25: the bridge role decides when both ranges hold
/// the tile; an unset role has no tiles here).
pub fn role_index(roles: &Roles, tile: i32) -> Option<i32> {
    // spec: S-0010 claim 25 (an unset role has no tiles; the bridge role is read before the wooden-bridge role)
    [roles.bridge, roles.wooden_bridge]
        .into_iter()
        .find(|&s| s >= 0 && (s..s + ROLE_TILES).contains(&tile))
        .map(|s| tile - s)
}

/// Table B's start sub-tile and direction (east: true) for *i*.
fn start_of(i: i32) -> Option<(u8, bool)> {
    // spec: S-0010 table B (start sub-tile and direction of the span)
    match i {
        0 | 1 => Some((7, true)),
        3 | 4 => Some((7, false)),
        6..=10 => Some((4, true)),
        11..=15 => Some((2, false)),
        _ => None,
    }
}

/// Table B's end sub-tile for *j*.
fn end_of(j: i32) -> Option<u8> {
    // spec: S-0010 table B (end sub-tile)
    match j {
        2 | 6..=10 => Some(4),
        5 | 11..=15 => Some(2),
        _ => None,
    }
}

/// The map's bridge spans (S-0010 claim 25), in the visiting order of the
/// grid's cells (the result does not depend on the order). `deck` tells
/// whether a cell is a high-bridge cell (S-0211 claim 15) as it stands.
pub fn find_spans(grid: &CellGrid, roles: &Roles, deck: impl Fn(CellXY) -> bool) -> Vec<Span> {
    let mut spans = Vec::new();
    for (c, cell) in grid.iter() {
        let Some(i) = role_index(roles, cell.tile) else {
            continue;
        };
        let Some((sub, east)) = start_of(i) else {
            continue;
        };
        // spec: S-0010 claim 25 (a start cell's sub-tile equals table B's start sub-tile)
        if cell.sub_tile != sub {
            continue;
        }
        let (dx, dy) = if east { (1, 0) } else { (0, 1) };
        let mut between = Vec::new();
        let mut end = None;
        let mut p = c;
        loop {
            p = CellXY::new(p.x + dx, p.y + dy);
            // spec: S-0010 claim 25 (a cell that is not legal before the far end: no span)
            let Some(q) = grid.get(p) else {
                break;
            };
            let j = role_index(roles, q.tile);
            // spec: S-0010 claim 25 (the far end: the nearest cell holding a tile j of either role with table B's end sub-tile)
            if j.and_then(end_of) == Some(q.sub_tile) {
                end = Some(p);
                break;
            }
            between.push((p, j.is_some()));
        }
        let Some(end) = end else {
            continue;
        };
        // spec: S-0010 claim 25 (high when every between-cell whose tile lies in neither range is a high-bridge cell; none: high)
        let high = between
            .iter()
            .filter(|(_, in_role)| !in_role)
            .all(|&(b, _)| deck(b));
        spans.push(Span {
            start: c,
            end,
            east,
            high,
        });
    }
    spans
}

/// The cells that the low spans give the bridge-body mark (S-0010 claim
/// 25): for each cell strictly between a low span's ends that is not a
/// high-bridge cell, the cell itself, the two cells beyond it towards the
/// low side across the span and the one cell towards the high side. Cells
/// that are not legal are left out. A cell may appear more than once.
pub fn body_cells(grid: &CellGrid, spans: &[Span], deck: impl Fn(CellXY) -> bool) -> Vec<CellXY> {
    let mut out = Vec::new();
    for s in spans.iter().filter(|s| !s.high) {
        let (dx, dy) = if s.east { (1, 0) } else { (0, 1) };
        // Across the span: towards y for an eastward span, towards x for a southward one.
        let (ax, ay) = (dy, dx);
        let mut p = CellXY::new(s.start.x + dx, s.start.y + dy);
        while p != s.end {
            if !deck(p) {
                // spec: S-0010 claim 25 and Constants (band of 4 cells: 2 to the low side, the cell, 1 to the high side)
                for k in [-2, -1, 0, 1] {
                    let m = CellXY::new(p.x + k * ax, p.y + k * ay);
                    if grid.get(m).is_some() {
                        out.push(m);
                    }
                }
            }
            p = CellXY::new(p.x + dx, p.y + dy);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cell, Footprint};

    fn grid() -> CellGrid {
        let mut g = CellGrid::new(Footprint::new(30, 30, [0, 0, 30, 30]));
        let all: Vec<CellXY> = g.iter().map(|(c, _)| c).collect();
        for c in all {
            *g.get_mut(c).unwrap() = Cell {
                tile: 0,
                ..Cell::BLANK
            };
        }
        g
    }

    fn roles() -> Roles {
        Roles {
            bridge: 100,
            wooden_bridge: 200,
            ..Roles::default()
        }
    }

    fn put(g: &mut CellGrid, c: CellXY, tile: i32, sub: u8) {
        let cell = g.get_mut(c).unwrap();
        cell.tile = tile;
        cell.sub_tile = sub;
    }

    /// spec: S-0010 claim 25 and table B (start cell, far end, direction; low span band of four cells across; high span marks nothing)
    #[test]
    fn claim_25_low_and_high_spans() {
        let mut g = grid();
        let (x0, y0) = (20, 20);
        // Wooden-bridge tile i = 6 (start sub-tile 4, east) and j = 6 ten cells east (end sub-tile 4).
        put(&mut g, CellXY::new(x0, y0), 206, 4);
        put(&mut g, CellXY::new(x0 + 10, y0), 206, 4);
        let spans = find_spans(&g, &roles(), |_| false);
        assert_eq!(spans.len(), 1);
        let s = spans[0];
        assert_eq!(
            (s.start, s.end, s.east, s.high),
            (CellXY::new(x0, y0), CellXY::new(x0 + 10, y0), true, false)
        );
        let marks = body_cells(&g, &spans, |_| false);
        let col: Vec<i32> = marks
            .iter()
            .filter(|m| m.x == x0 + 5)
            .map(|m| m.y)
            .collect();
        assert_eq!(col, [y0 - 2, y0 - 1, y0, y0 + 1]);
        assert!(!marks.contains(&CellXY::new(x0, y0)));
        assert!(!marks.contains(&CellXY::new(x0 + 10, y0)));
        // A high span: every between-cell is a high-bridge cell.
        let spans = find_spans(&g, &roles(), |c| c.y == y0);
        assert!(spans[0].high);
        assert!(body_cells(&g, &spans, |c| c.y == y0).is_empty());
        // No far end: no span.
        put(&mut g, CellXY::new(x0 + 10, y0), 0, 0);
        assert!(find_spans(&g, &roles(), |_| false).is_empty());
    }

    /// spec: S-0010 claim 25 (southward spans, mixed roles, the bridge role read first, an unset role has no tiles)
    #[test]
    fn claim_25_south_mixed_and_unset() {
        let mut g = grid();
        // Bridge tile i = 11 (start sub-tile 2, south), ending on a wooden tile j = 5 (end sub-tile 2).
        put(&mut g, CellXY::new(20, 15), 111, 2);
        put(&mut g, CellXY::new(20, 19), 205, 2);
        let spans = find_spans(&g, &roles(), |_| false);
        assert_eq!(spans.len(), 1);
        assert!(!spans[0].east);
        let marks = body_cells(&g, &spans, |_| false);
        let row: Vec<i32> = marks.iter().filter(|m| m.y == 17).map(|m| m.x).collect();
        assert_eq!(row, [18, 19, 20, 21]);
        // Overlapping ranges: the bridge role's i decides.
        let both = Roles {
            bridge: 100,
            wooden_bridge: 95,
            ..Roles::default()
        };
        assert_eq!(role_index(&both, 100), Some(0));
        // An unset role has no tiles.
        let unset = Roles::default();
        assert!(find_spans(&g, &unset, |_| false).is_empty());
    }
}
