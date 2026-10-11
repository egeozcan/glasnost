//! The line-of-fire walk (S-0306 claim 22), used by the range test
//! (S-0307 claim 11), invisible projectiles (S-0306 claim 9) and flight
//! (S-0306 T1).

use glasnost_map::CellXY;

use crate::combat::data::ProjectileRules;
use crate::world::{HouseId, Pos, World};

/// The elevation level of a cell (S-0307 Terms): its height level, plus 4
/// for a cell marked as carrying a bridge deck.
pub fn elevation_level(w: &World, c: CellXY) -> i32 {
    let level = w.cells.cell(c).map_or(0, |cell| i32::from(cell.level));
    // spec: S-0307 Terms (elevation level: height level plus 4 for a deck-marked cell)
    level + if w.high_bridge(c) { 4 } else { 0 }
}

/// The plain height level of a cell.
fn height_level(w: &World, c: CellXY) -> i32 {
    w.cells.cell(c).map_or(0, |cell| i32::from(cell.level))
}

/// Whether the cell holds a wall overlay, and the overlay's registration
/// index.
pub fn wall_overlay(w: &World, c: CellXY) -> Option<u8> {
    let cell = w.cells.cell(c)?;
    let o = cell.overlay;
    let r = w.rules();
    r.overlays.get(usize::from(o)).filter(|t| t.wall).map(|_| o)
}

/// Whether one sample cell blocks (S-0306 claim 22), with `prev` the
/// previous sample's cell and `x1`, `x2` the walk's end cells.
#[allow(clippy::too_many_arguments)]
pub fn sample_blocks(
    w: &World,
    p: &ProjectileRules,
    house: Option<HouseId>,
    cell: CellXY,
    prev: CellXY,
    x1: CellXY,
    x2: CellXY,
) -> bool {
    if p.cliffs {
        let e = elevation_level(w, cell);
        // spec: S-0306 claim 22 (a cliff: at least 4 above the previous sample's cell and strictly above the start cell's)
        if e >= elevation_level(w, prev) + 4 && e > elevation_level(w, x1) {
            return true;
        }
    }
    if p.walls
        && cell != x2
        && wall_overlay(w, cell).is_some()
        // spec: S-0306 claim 22 (a wall: the start cell's height level not above the end cell's)
        && height_level(w, x1) <= height_level(w, x2)
    {
        let transparent = w.rules().combat.general.allied_wall_transparency
            && w.cells
                .idx(cell)
                .and_then(|i| w.cells.wall_owner[i])
                .is_some_and(|owner| w.allied(Some(owner), house));
        // spec: S-0306 claim 22 (an allied wall does not block while allied wall transparency is on)
        if !transparent {
            return true;
        }
    }
    false
}

/// The walk from `a` to `b`: the first blocked sample's cell, if any.
pub fn walk(
    w: &World,
    p: &ProjectileRules,
    house: Option<HouseId>,
    a: Pos,
    b: Pos,
) -> Option<CellXY> {
    if !p.cliffs && !p.walls {
        return None;
    }
    let (ca, cb) = (a.cell(), b.cell());
    // spec: S-0306 claim 22 (n = max(|Δcₓ|, |Δc_y|) samples; none when both are in one cell)
    let n = (cb.x - ca.x).abs().max((cb.y - ca.y).abs());
    if n == 0 {
        return None;
    }
    // spec: S-0306 claim 22 (each component of (X₂ − X₁) / n truncated toward zero)
    let step = ((b.x - a.x) / n, (b.y - a.y) / n, (b.z - a.z) / n);
    let mut prev = ca;
    for i in 0..n {
        let s = Pos::new(a.x + i * step.0, a.y + i * step.1, a.z + i * step.2);
        let c = s.cell();
        if sample_blocks(w, p, house, c, prev, ca, cb) {
            return Some(c);
        }
        prev = c;
    }
    None
}
