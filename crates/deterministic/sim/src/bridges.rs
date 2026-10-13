//! Bridge spans and links (S-0010 claim 25; S-0212 Terms, claims 3 and 4).
//!
//! Once at game start the map's bridge spans are found from the tiles of
//! the bridge and wooden-bridge roles. Each span is a bridge link of the
//! path search (enabled when the span is high); each low span gives the
//! cells of its band the bridge-body mark and its bridge axis. The links,
//! marks and axes are state ([`crate::cells::Cells`]); regions and sectors
//! read them. The changes of links in play (S-0212 claim 3's table) wait for
//! the bridge specs, which name the cell each destruction or repair touches.

use glasnost_map::CellXY;
use glasnost_map::terrain::{Roles, land};

use crate::cells::{Axis, BridgeLink};
use crate::world::World;

/// Tiles per bridge role (S-0010 table R and table B).
// spec: S-0010 Constants table R (bridge and wooden bridge: 16 tiles each)
const ROLE_TILES: i32 = 16;

/// The direction of a span (S-0010 table B).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Dir {
    /// Towards x + 1.
    East,
    /// Towards y + 1.
    South,
}

/// The place `i` of `tile` in the bridge role's range, or else in the
/// wooden-bridge role's (S-0010 claim 25). An unset role has no tiles.
fn place(roles: &Roles, tile: i32) -> Option<i32> {
    let in_role = |start: i32| {
        // spec: S-0010 claim 25 (an unset role has no tiles here; these tests never count from the unset start)
        (start >= 0 && (start..start + ROLE_TILES).contains(&tile)).then(|| tile - start)
    };
    // spec: S-0010 claim 25 (a tile in the bridge role's range is judged by the bridge role's i alone)
    in_role(roles.bridge).or_else(|| in_role(roles.wooden_bridge))
}

/// The start sub-tile and direction of tile place `i` (S-0010 table B).
fn start_of(i: i32) -> Option<(u8, Dir)> {
    // spec: S-0010 Constants table B (start sub-tiles and directions)
    match i {
        0 | 1 => Some((7, Dir::East)),
        3 | 4 => Some((7, Dir::South)),
        6..=10 => Some((4, Dir::East)),
        11..=15 => Some((2, Dir::South)),
        _ => None,
    }
}

/// The end sub-tile of tile place `j` (S-0010 table B).
fn end_of(j: i32) -> Option<u8> {
    // spec: S-0010 Constants table B (end sub-tiles)
    match j {
        2 | 6..=10 => Some(4),
        5 | 11..=15 => Some(2),
        _ => None,
    }
}

/// Whether `c` meets the condition of a bridge end (S-0212 Terms, *bridge
/// ends*): its tile is one of the 16 tiles of the bridge or wooden-bridge
/// role of its theater and its land type is not rock.
pub fn end_condition(w: &World, c: CellXY) -> bool {
    let Some(cell) = w.cells.cell(c) else {
        return false;
    };
    let Some(cat) = w.catalogue.as_ref() else {
        return false;
    };
    // spec: S-0212 Terms (bridge ends: a tile of the 16 of the bridge or wooden-bridge role, and land that is not rock)
    place(&cat.roles, cell.tile).is_some() && cell.land != land::ROCK
}

/// The two bridge ends of the bridge cell `x` (S-0212 Terms), first the one
/// towards x + 1 (along cell x) or y + 1 (along cell y), then the other;
/// `None` for a direction without one.
// nocite: the two directions along an axis, not a behavioural number
pub fn ends(w: &World, x: CellXY) -> [Option<CellXY>; 2] {
    let (dx, dy) = match w.cells.axis(x) {
        Axis::X => (1, 0),
        Axis::Y => (0, 1),
        Axis::None => return [None, None],
    };
    let walk = |sx: i32, sy: i32| {
        let mut c = x;
        loop {
            c = CellXY::new(c.x + sx, c.y + sy);
            w.cells.cell(c)?;
            if !w.cells.is_bridge(c) {
                // spec: S-0212 Terms (the first cell that is not a bridge cell, when it meets the condition)
                return end_condition(w, c).then_some(c);
            }
        }
    };
    [walk(dx, dy), walk(-dx, -dy)]
}

/// A span found at game start.
struct Span {
    start: CellXY,
    far: CellXY,
    dir: Dir,
    high: bool,
}

/// Finds the map's bridge spans (S-0010 claim 25), in the visiting order
/// `order` of their start cells, and makes them the bridge links (S-0212
/// claim 3); low spans set the bridge-body mark and the bridge axis on
/// their bands.
pub fn find_spans(w: &mut World, order: &[CellXY]) {
    let roles = w.catalogue_arc().roles;
    let mut spans: Vec<Span> = Vec::new();
    for &c in order {
        let Some(cell) = w.cells.cell(c) else {
            continue;
        };
        let Some((sub, dir)) = place(&roles, cell.tile).and_then(start_of) else {
            continue;
        };
        // spec: S-0010 claim 25 (a start cell: tile i of a bridge role with the start sub-tile of table B)
        if cell.sub_tile != sub {
            continue;
        }
        let (sx, sy) = match dir {
            Dir::East => (1, 0),
            Dir::South => (0, 1),
        };
        let mut far = None;
        let mut n = c;
        loop {
            n = CellXY::new(n.x + sx, n.y + sy);
            // spec: S-0010 claim 25 (a cell that is not a legal cell before a far end: no span)
            let Some(nc) = w.cells.cell(n) else { break };
            // spec: S-0010 claim 25 (the far end: the nearest cell with tile j of a role and the end sub-tile of table B for j)
            if place(&roles, nc.tile).and_then(end_of) == Some(nc.sub_tile) {
                far = Some(n);
                break;
            }
        }
        let Some(far) = far else { continue };
        // spec: S-0010 claim 25 (high when every cell strictly between whose tile lies in neither range is a high-bridge cell)
        let mut high = true;
        let mut b = CellXY::new(c.x + sx, c.y + sy);
        while b != far {
            let outside = w
                .cells
                .cell(b)
                .is_some_and(|x| place(&roles, x.tile).is_none());
            if outside && !w.cells.is_bridge(b) {
                high = false;
            }
            b = CellXY::new(b.x + sx, b.y + sy);
        }
        spans.push(Span {
            start: c,
            far,
            dir,
            high,
        });
    }
    // The band of each low span; where bands overlap, the span whose start
    // cell comes first in the visiting order decides the axis, so the
    // spans are applied last to first.
    for s in spans.iter().rev().filter(|s| !s.high) {
        let (sx, sy) = match s.dir {
            Dir::East => (1, 0),
            Dir::South => (0, 1),
        };
        let axis = match s.dir {
            // spec: S-0212 Terms (a low span's band sets the axis: along cell x for a span running east, along cell y for one running south)
            Dir::East => Axis::X,
            Dir::South => Axis::Y,
        };
        let mut b = CellXY::new(s.start.x + sx, s.start.y + sy);
        while b != s.far {
            if !w.cells.is_bridge(b) {
                // spec: S-0010 claim 25 (the band: the cell, two cells to the low side and one to the high side, across the span)
                for k in [0, -1, -2, 1] {
                    let m = CellXY::new(b.x + k * sy, b.y + k * sx);
                    if let Some(i) = w.cells.idx(m) {
                        w.cells.bridge_body[i] = true;
                        w.cells.bridge_axis[i] = axis;
                    }
                }
            }
            b = CellXY::new(b.x + sx, b.y + sy);
        }
    }
    // spec: S-0212 claim 3 (one link per span in the visiting order of the start cells; enabled when the span is high)
    w.cells.links = spans
        .iter()
        .map(|s| BridgeLink {
            first: s.start,
            second: s.far,
            along_x: s.dir == Dir::East,
            enabled: s.high,
        })
        .collect();
}

/// The first bridge link, in creation order, that `c` lies within `k`
/// cells of (S-0212 Terms).
pub fn link_within(w: &World, c: CellXY, k: i32) -> Option<BridgeLink> {
    w.cells.links.iter().copied().find(|l| l.within(c, k))
}

/// Going from the bridge cell `x` one cell at a time east along cell x (a
/// link along cell x) or south along cell y, the first cell that is not a
/// bridge cell (S-0212 claims 4 and 13b), if any.
pub fn first_off_bridge(w: &World, x: CellXY, along_x: bool) -> Option<CellXY> {
    let (sx, sy) = if along_x { (1, 0) } else { (0, 1) };
    let mut c = x;
    loop {
        c = CellXY::new(c.x + sx, c.y + sy);
        w.cells.cell(c)?;
        if !w.cells.is_bridge(c) {
            return Some(c);
        }
    }
}
