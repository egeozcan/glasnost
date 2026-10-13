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

/// Tiles per bridge role (S-0010 claim 25 and table R).
// spec: S-0010 claim 25 (16 tiles each)
const ROLE_TILES: i32 = 16;

/// Whether `tile` is one of the tiles of the bridge or wooden-bridge role
/// (S-0010 claim 25). An unset role has no tiles.
fn in_roles(roles: &Roles, tile: i32) -> bool {
    // spec: S-0010 claim 25 (an unset role has no tiles here)
    [roles.bridge, roles.wooden_bridge]
        .into_iter()
        .any(|s| s >= 0 && (s..s + ROLE_TILES).contains(&tile))
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
    in_roles(&cat.roles, cell.tile) && cell.land != land::ROCK
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

/// Finds the map's bridge spans (S-0010 claim 25, through
/// [`glasnost_map::bridges::find_spans`]), in the visiting order `order` of
/// their start cells, and makes them the bridge links (S-0212 claim 3);
/// low spans set the bridge-body mark and the bridge axis on their bands.
pub fn find_spans(w: &mut World, order: &[CellXY]) {
    let roles = w.catalogue_arc().roles;
    let cells = &w.cells;
    let deck = |c: CellXY| cells.is_bridge(c);
    let mut spans = glasnost_map::bridges::find_spans(&cells.grid, &roles, deck);
    // spec: S-0212 claim 3 (links in the visiting order of the start cells)
    let rank: std::collections::BTreeMap<CellXY, usize> =
        order.iter().enumerate().map(|(i, &c)| (c, i)).collect();
    spans.sort_by_key(|s| rank.get(&s.start).copied().unwrap_or(usize::MAX));
    // The band of each low span; where bands overlap, the span whose start
    // cell comes first in the visiting order decides the axis, so the
    // spans are applied last to first.
    for s in spans.iter().rev().filter(|s| !s.high) {
        let axis = if s.east {
            // spec: S-0212 Terms (a low span's band sets the axis: along cell x for a span running east, along cell y for one running south)
            Axis::X
        } else {
            Axis::Y
        };
        let band = glasnost_map::bridges::body_cells(&w.cells.grid, std::slice::from_ref(s), |c| {
            w.cells.is_bridge(c)
        });
        for m in band {
            if let Some(i) = w.cells.idx(m) {
                w.cells.bridge_body[i] = true;
                w.cells.bridge_axis[i] = axis;
            }
        }
    }
    // spec: S-0212 claim 3 (one link per span; enabled when the span is high)
    w.cells.links = spans
        .iter()
        .map(|s| BridgeLink {
            first: s.start,
            second: s.end,
            along_x: s.east,
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
