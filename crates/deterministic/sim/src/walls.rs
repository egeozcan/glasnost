//! Wall connection bits (S-0112 claim 7): a wall overlay's data value is
//! 16 × damage stage + connection bits, and the bits are recomputed on the
//! events of claim 7's table. The data value is simulation state: crushing
//! (S-0210 claim 8) reads its stage and the map viewer draws it as the
//! wall's image frame (S-0112 claim 1, row 4).

use glasnost_map::{CellXY, NO_OVERLAY};

use crate::rules::Kind;
use crate::world::{Layer, ObjectId, World};

/// The edge neighbour steps in the order of the connection bits: bit k is
/// the neighbour in step direction 2k (map north, east, south, west).
// spec: S-0112 claim 7 (bit k for step direction 2k: north, east, south, west); S-0204 claim 9 (the steps)
pub const EDGE_STEPS: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

/// The overlay list positions whose walls join the Allied gates and the
/// wall tower (S-0112 claim 7).
// spec: S-0112 Constants (wall positions joining Allied gates and the wall tower: 0, 2)
const ALLIED_WALLS: [u8; 2] = [0, 2];

/// The overlay list position whose walls join the Soviet gates.
const SOVIET_WALL: u8 = 26; // spec: S-0112 Constants (wall position joining Soviet gates)

/// Overlay data values per damage stage (S-0112 claim 7: 16 × stage + bits).
const STAGE_STEP: u8 = 16; // spec: S-0112 claim 7 (16 × damage stage + connection bits)

/// What a gate or tower building is, for the wall rows of claim 7.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Joiner {
    /// A first gate type (joins walls map east and west of it).
    FirstGate,
    /// A second gate type (joins walls map north and south of it).
    SecondGate,
    /// The wall tower type (joins walls in every direction).
    Tower,
}

/// Whether the overlay type at list position `o` is a wall.
pub fn is_wall(w: &World, o: u8) -> bool {
    o != NO_OVERLAY
        && w.rules()
            .overlays
            .get(usize::from(o))
            .is_some_and(|t| t.wall)
}

/// What building type `t` is for walls, if anything.
pub fn joiner(w: &World, t: u32) -> Option<Joiner> {
    let g = w.rules().gates;
    let t = Some(t);
    if t == g.tower {
        Some(Joiner::Tower)
    } else if t == g.allied_one || t == g.soviet_one {
        Some(Joiner::FirstGate)
    } else if t == g.allied_two || t == g.soviet_two {
        Some(Joiner::SecondGate)
    } else {
        None
    }
}

/// Whether a building of type `t` in the neighbour at edge step `k` joins
/// a wall of overlay type `wall` (S-0112 claim 7).
fn building_joins(w: &World, wall: u8, t: u32, k: usize) -> bool {
    let g = w.rules().gates;
    let t = Some(t);
    // spec: S-0112 claim 7 (first gates map east or west, second gates map north or south)
    let east_west = k == 1 || k == 3;
    if ALLIED_WALLS.contains(&wall) {
        // spec: S-0112 claim 7 (overlay positions 0 and 2: the Allied gates and the wall tower)
        (t == g.allied_one && east_west) || (t == g.allied_two && !east_west) || t == g.tower
    } else if wall == SOVIET_WALL {
        // spec: S-0112 claim 7 (overlay position 26: the Soviet gates)
        (t == g.soviet_one && east_west) || (t == g.soviet_two && !east_west)
    } else {
        false
    }
}

/// The connection bits of the wall in `c` from its current neighbours.
fn bits(w: &World, c: CellXY, wall: u8) -> u8 {
    let mut b = 0;
    for (k, (dx, dy)) in EDGE_STEPS.iter().enumerate() {
        let n = CellXY::new(c.x + dx, c.y + dy);
        let Some(nc) = w.cells.cell(n) else {
            continue;
        };
        // spec: S-0112 claim 7 (a neighbour of the same overlay type; walls of different types never join)
        let same = nc.overlay == wall;
        // spec: S-0112 claim 7 (a gate or tower building with health above 0 in the neighbour)
        let gate = w.cells.list(n, Layer::Ground).iter().any(|&id| {
            w.obj(id).is_some_and(|o| {
                o.kind == Kind::Building && o.health > 0 && building_joins(w, wall, o.type_index, k)
            })
        });
        if same || gate {
            b |= 1 << k;
        }
    }
    b
}

/// Recomputes the connection bits of every wall cell among `cells`, keeping
/// each one's damage stage (S-0112 claim 7). Cells without a wall are left
/// alone.
pub fn recompute(w: &mut World, cells: &[CellXY]) {
    for &c in cells {
        let Some(cell) = w.cells.cell(c).copied() else {
            continue;
        };
        if !is_wall(w, cell.overlay) {
            continue;
        }
        let b = bits(w, c, cell.overlay);
        if let Some(m) = w.cells.grid.get_mut(c) {
            // spec: S-0112 claim 7 (data value 16 × stage + bits; the stage is kept)
            m.overlay_data = (m.overlay_data / STAGE_STEP) * STAGE_STEP + b;
        }
    }
}

/// A cell and its four edge neighbours.
pub fn with_neighbours(c: CellXY) -> Vec<CellXY> {
    let mut v = vec![c];
    v.extend(
        EDGE_STEPS
            .iter()
            .map(|(dx, dy)| CellXY::new(c.x + dx, c.y + dy)),
    );
    v
}

/// The cells within two edge steps of `c` (S-0112 Terms).
pub fn within_two(c: CellXY) -> Vec<CellXY> {
    let mut v = Vec::new();
    // spec: S-0112 Terms (within two edge steps: |x′ − x| + |y′ − y| ≤ 2)
    for dy in -2..=2_i32 {
        for dx in -2..=2_i32 {
            if dx.abs() + dy.abs() <= 2 {
                v.push(CellXY::new(c.x + dx, c.y + dy));
            }
        }
    }
    v
}

/// A wall overlay was placed in `c` (S-0112 claim 7, first row): `c` and
/// its four edge neighbours are recomputed.
pub fn wall_placed(w: &mut World, c: CellXY) {
    recompute(w, &with_neighbours(c));
}

/// The wall in `c` was removed by a wall hit, crushing included (S-0112
/// claim 7, third row): every cell within two edge steps is recomputed.
pub fn wall_removed(w: &mut World, c: CellXY) {
    recompute(w, &within_two(c));
}

/// A gate or wall tower building was placed, or was destroyed and left the
/// map (S-0112 claim 7, last three rows). `top_left` is its top-left
/// foundation cell and `foundation` its size.
pub fn joiner_changed(w: &mut World, j: Joiner, top_left: CellXY, foundation: (i32, i32)) {
    let (x, y) = (top_left.x, top_left.y);
    let mut cells = match j {
        // spec: S-0112 claim 7 (a wall tower: every cell within two edge steps of its top-left cell)
        Joiner::Tower => within_two(top_left),
        // spec: S-0112 claim 7 (a first gate: (x − 1, y) and (x + 3, y), each with its four edge neighbours)
        Joiner::FirstGate => {
            let mut v = with_neighbours(CellXY::new(x - 1, y));
            v.extend(with_neighbours(CellXY::new(x + 3, y)));
            v
        }
        // spec: S-0112 claim 7 (a second gate: (x, y − 1) and (x, y + 3), each with its four edge neighbours)
        Joiner::SecondGate => {
            let mut v = with_neighbours(CellXY::new(x, y - 1));
            v.extend(with_neighbours(CellXY::new(x, y + 3)));
            v
        }
    };
    if !w.rules().quirks.gate_tower_fixed_cells {
        // spec: S-0112 Edge cases (quirks.gate_tower_fixed_cells off: also every edge neighbour of any foundation cell)
        for fy in 0..foundation.1 {
            for fx in 0..foundation.0 {
                cells.extend(with_neighbours(CellXY::new(x + fx, y + fy)));
            }
        }
    }
    recompute(w, &cells);
}

/// The building `id` was just placed: a gate or tower recomputes its walls.
pub fn building_placed(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if o.kind != Kind::Building {
        return;
    }
    let (t, cell, f) = (o.type_index, o.pos.cell(), o.foundation);
    if let Some(j) = joiner(w, t) {
        joiner_changed(w, j, cell, f);
    }
}
