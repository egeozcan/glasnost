//! Overlays at map load: placing the map's overlay type stream with its
//! placement effects (S-0009 claims 12 and 13; S-0202 claim 10: low-bridge
//! end markers and their pieces; S-0211 claim 15: high-bridge cells;
//! S-0112 claim 7: wall connection bits) and the vein rebuild of S-0009
//! claim 14 (S-0202 claim 10, *vein patterns*).

use glasnost_map::terrain::land;
use glasnost_map::{CellXY, Map, NO_OVERLAY};

use crate::load::recalc_cells;
use crate::walls;
use crate::world::World;

/// The overlay list positions whose creation leaves the cell's overlay data
/// value as it was (S-0009 claim 12).
// spec: S-0009 Constants (overlay indices whose data value survives creation: 24, 25, 237, 238)
const DATA_KEEPING: [u8; 4] = [24, 25, 237, 238];

/// The largest slope shape on which an overlay is created (S-0009 claim 12).
const SLOPE_LIMIT: u8 = 4; // spec: S-0009 Constants (largest slope shape that takes an overlay)

/// The overlay list position exempt from the slope condition.
const SLOPE_EXEMPT: u8 = 178; // spec: S-0009 Constants (overlay index exempt from the slope condition)

/// The vein overlay's list position (S-0202 claim 10, S-0009 claim 14).
pub const VEIN: u8 = 126; // spec: S-0202 Constants (vein overlay position 126)

/// The overlay data value from which a vein cell is a pattern centre, and a
/// vein seed at map load (S-0202 Terms, S-0009 claim 14).
const PATTERN_CENTRE: u8 = 48; // spec: S-0009 Constants (vein seed threshold 48); S-0202 Terms (pattern centre)

/// The axis a low-bridge end marker builds along (S-0202 Terms, *line*).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Line {
    /// The map's x axis: the end row runs across y.
    X,
    /// The map's y axis: the end row runs across x.
    Y,
}

/// One low-bridge end marker (S-0202 Formulas table *Low-bridge end
/// markers*): its overlay list position, its line, the end-row type it
/// writes, its search step, the end-row type it looks for (with data 1),
/// and the base piece type t₀ of its pair.
struct Marker {
    pos: u8,
    line: Line,
    row_type: u8,
    search: (i32, i32),
    partner_type: u8,
    base: u8,
}

const fn mk(
    pos: u8,
    line: Line,
    row_type: u8,
    search: (i32, i32),
    partner_type: u8,
    base: u8,
) -> Marker {
    Marker {
        pos,
        line,
        row_type,
        search,
        partner_type,
        base,
    }
}

/// The low-bridge end markers (S-0202 Formulas).
// spec: S-0202 Formulas (table Low-bridge end markers; table Bridge piece type)
const MARKERS: [Marker; 8] = [
    mk(122, Line::X, 92, (-1, 0), 94, 74),
    mk(123, Line::X, 94, (1, 0), 92, 74),
    mk(124, Line::Y, 96, (0, 1), 98, 83),
    mk(125, Line::Y, 98, (0, -1), 96, 83),
    mk(233, Line::X, 223, (-1, 0), 225, 205),
    mk(234, Line::X, 225, (1, 0), 223, 205),
    mk(235, Line::Y, 227, (0, 1), 229, 214),
    mk(236, Line::Y, 229, (0, -1), 227, 214),
];

/// The end marker of overlay list position `o`, if it is one.
fn marker(o: u8) -> Option<&'static Marker> {
    MARKERS.iter().find(|m| m.pos == o)
}

/// The three cells across `line` centred on `c`, in the order of their
/// overlay data 0, 1, 2 (increasing y for the x axis, increasing x for the
/// y axis).
// spec: S-0202 Constants (end-row and piece width: 3 cells across the line)
fn across(line: Line, c: CellXY) -> [CellXY; 3] {
    // spec: S-0202 claim 10 (end row: (x, y − 1), (x, y), (x, y + 1) for the x axis; (x − 1, y), (x, y), (x + 1, y) for the y axis)
    match line {
        Line::X => [CellXY::new(c.x, c.y - 1), c, CellXY::new(c.x, c.y + 1)],
        Line::Y => [CellXY::new(c.x - 1, c.y), c, CellXY::new(c.x + 1, c.y)],
    }
}

/// Places the map's overlay type stream (S-0009 claims 12 and 13) on cells
/// that hold no overlay and data 0: each overlay in stream order, with the
/// slope condition and its placement effects, then, when the map has an
/// overlay data stream, every legal cell's data value from it.
/// `stream` holds each legal cell's overlay type and data value as read.
pub fn place_stream(w: &mut World, map: &Map, stream: &[(CellXY, u8, u8)]) {
    let r = w.rules_arc();
    let q = r.quirks;
    let mut order: Vec<&(CellXY, u8, u8)> = stream.iter().collect();
    // spec: S-0009 claim 12 (stream order: increasing y, then increasing x)
    order.sort_by_key(|(c, _, _)| (c.y, c.x));
    for &&(c, o, _) in &order {
        if o == NO_OVERLAY {
            continue;
        }
        // S-0009 claim 11 (no overlay below format version 2) is the importer's
        // (convert::map::MapQuirks, quirks.no_overlay_below_format_2).
        let Some(t) = r.overlays.get(usize::from(o)) else {
            // spec: S-0009 Edge cases (an index beyond the list: no overlay)
            continue;
        };
        // spec: S-0009 claim 12 (artwork or cell animation; crates only in campaign games, quirks.skirmish_drops_map_crates)
        if !(t.has_art || t.cell_anim.is_some()) || (t.crate_ && q.skirmish_drops_map_crates) {
            continue;
        }
        let Some(slope) = w.cells.cell(c).map(|x| x.slope) else {
            continue;
        };
        // spec: S-0009 claim 12 (the cell's slope shape as it stands is at most 4, or the type is at position 178)
        if slope > SLOPE_LIMIT && o != SLOPE_EXEMPT {
            continue;
        }
        place_overlay(w, c, o);
    }
    if map.has_overlay_data {
        // spec: S-0009 claim 13 (the data stream's values are final with respect to overlay creation; an immediate crater request made during placement acted before this overwrite, and the overwrite restores no overlay type it removed)
        for &(c, _, d) in stream {
            if let Some(cell) = w.cells.grid.get_mut(c) {
                cell.overlay_data = d;
            }
        }
    }
}

/// One overlay of the map's overlay list is created in `c` with its
/// placement effects (S-0009 claim 12; S-0202 claim 10).
fn place_overlay(w: &mut World, c: CellXY, o: u8) {
    if let Some(m) = marker(o) {
        // spec: S-0009 claim 12 and S-0202 claim 14 (a low-bridge end marker, its end-row writes and its piece writes start no cell animation; their recalculations can start tile animations)
        end_marker(w, c, m);
        // spec: S-0202 claim 10 (placing an overlay recalculates its cell)
        recalc_cells(w, &[c]);
        return;
    }
    if let Some(cell) = w.cells.grid.get_mut(c) {
        cell.overlay = o;
        // spec: S-0009 claim 12 (indices 24, 25, 237, 238 keep the cell's data value; 0 otherwise unless placement sets one)
        if !DATA_KEEPING.contains(&o) {
            cell.overlay_data = 0;
        }
    }
    // spec: S-0211 claim 15 (a placed high-bridge overlay marks its high-bridge cells)
    w.cells.mark_high_bridge(c, o);
    if walls::is_wall(w, o) {
        // spec: S-0112 claim 7 (a wall placed from the map's overlay list recomputes c and its edge neighbours)
        walls::wall_placed(w, c);
    }
    cell_animation(w, c, o);
    // spec: S-0202 claim 10 (placing an overlay recalculates its cell)
    recalc_cells(w, &[c]);
}

/// The cell animation of an overlay of type `o` just placed (S-0009 claim
/// 12, *Cell animations*): one animation of the type's cell animation
/// starts, with the creation draws of S-0202 claim 10, after the overlay is
/// in its cell and before that placement's recalculation of the cell (so
/// before the draws of any tile animation that recalculation starts).
/// Animation playback (S-0109) is not modelled, so the animation itself
/// leaves no state; its position, the overlay cell's centre moved by 384
/// leptons along +x and +y (the top corner of cell (x + 2, y + 2), where its
/// start event's effects act), at the surface height of the overlay cell's
/// centre (S-0202 claim 10, S-0009 claim 12), places its start event's
/// effects (`load::anim_created_at`).
fn cell_animation(w: &mut World, c: CellXY, o: u8) {
    let Some(name) = w
        .rules()
        .overlays
        .get(usize::from(o))
        .and_then(|t| t.cell_anim.clone())
    else {
        return;
    };
    // spec: S-0009 claim 12 (Where: the overlay cell's centre)
    let (cx, cy) = (256 * c.x + 128, 256 * c.y + 128);
    // spec: S-0009 claim 12 (Where: the height is the terrain surface height at the centre of the overlay's cell)
    let z = w.surface_at(cx, cy);
    // spec: S-0009 Constants (cell animation displacement: 384 leptons along +x and along +y)
    let at = (cx + 384, cy + 384, z);
    // spec: S-0009 claim 12 (Order: the cell animation's creation draws come before the tile animation of the same placement)
    crate::load::anim_created_at(w, &name, Some(at));
}

/// Whether a cell holds an overlay, for the end-row test (S-0202 claim
/// 10): a cell outside the map holds none (S-0009 claim 44).
fn holds_overlay(w: &World, c: CellXY) -> bool {
    w.cells.cell(c).is_some_and(|x| x.overlay != NO_OVERLAY)
}

/// A low-bridge end marker placed in `c` (S-0202 claim 10).
fn end_marker(w: &mut World, c: CellXY, m: &Marker) {
    let row = across(m.line, c);
    // spec: S-0202 claim 10 (any end-row cell holding an overlay: nothing is written, nothing follows)
    if row.iter().any(|&x| holds_overlay(w, x)) {
        return;
    }
    for (k, &x) in row.iter().enumerate() {
        // spec: S-0202 claim 10 (the end-row type with data 0, 1, 2; recalculated as written; nothing outside the map, S-0009 claim 44)
        if let Some(cell) = w.cells.grid.get_mut(x) {
            cell.overlay = m.row_type;
            cell.overlay_data = k as u8;
            recalc_cells(w, &[x]);
        }
    }
    // spec: S-0202 claim 10 (the partner: the nearest legal cell along the search direction holding the partner's end-row type with data 1, across legal cells only, no distance limit)
    let (sx, sy) = m.search;
    let mut dist = 0;
    let mut found = None;
    loop {
        dist += 1;
        let p = CellXY::new(c.x + sx * dist, c.y + sy * dist);
        let Some(cell) = w.cells.cell(p) else {
            break;
        };
        if cell.overlay == m.partner_type && cell.overlay_data == 1 {
            found = Some(dist);
            break;
        }
    }
    let Some(mdist) = found else {
        return;
    };
    // spec: S-0202 Formulas (d = max(m − 1, 1) steps; for m = 1 the step is the marker's own end row)
    let steps: Vec<i32> = if mdist >= 2 {
        // spec: S-0202 claim 10 (step by step from the step next to the partner's middle toward the marker)
        (1..mdist).rev().collect()
    } else {
        vec![0]
    };
    for k in steps {
        let step = CellXY::new(c.x + sx * k, c.y + sy * k);
        for (place, x) in across(m.line, step).into_iter().enumerate() {
            // spec: S-0202 Formulas (piece type t₀ + (x mod 4) for one raw value; data its place across)
            let raw = w.rng.raw();
            let piece = m.base + (raw % 4) as u8; // spec: S-0202 Formulas (bridge piece type: the lowest two bits)
            // spec: S-0202 claim 10 (a piece outside the map still takes its raw value; our engine writes nothing there, S-0009 claim 44)
            if let Some(cell) = w.cells.grid.get_mut(x) {
                cell.overlay = piece;
                cell.overlay_data = place as u8;
                recalc_cells(w, &[x]);
            }
        }
    }
}

/// Whether a land type is one the vein placement test refuses (S-0202
/// claim 10).
fn vein_refused_land(l: u8) -> bool {
    // spec: S-0202 claim 10 (land type none of water, rock, beach and ice)
    matches!(l, land::WATER | land::ROCK | land::BEACH | land::ICE)
}

/// Whether an overlay lets a vein placement pass: none, or a type with the
/// veins flag (S-0202 claim 10; the vein overlay itself only when it has
/// the flag).
fn vein_overlay_ok(w: &World, o: u8) -> bool {
    o == NO_OVERLAY
        || w.rules()
            .overlays
            .get(usize::from(o))
            .is_some_and(|t| t.veins)
}

/// The vein placement test of a cell (S-0202 claim 10). An edge neighbour
/// outside the map reads as a blank cell, which passes (S-0009 claim 44).
fn vein_test(w: &World, c: CellXY) -> bool {
    let Some(cell) = w.cells.cell(c) else {
        return false;
    };
    if cell.slope > SLOPE_LIMIT || vein_refused_land(cell.land) || !vein_overlay_ok(w, cell.overlay)
    {
        return false;
    }
    for (dx, dy) in walls::EDGE_STEPS {
        let Some(n) = w.cells.cell(CellXY::new(c.x + dx, c.y + dy)) else {
            continue;
        };
        if vein_refused_land(n.land) || !vein_overlay_ok(w, n.overlay) {
            return false;
        }
        // spec: S-0202 claim 10 (a flat cell: no edge neighbour with slope shape above 4 holding no overlay)
        if cell.slope == 0 && n.slope > SLOPE_LIMIT && n.overlay == NO_OVERLAY {
            return false;
        }
    }
    true
}

/// Whether a cell is a pattern centre: a vein cell with data 48 or more.
fn pattern_centre(o: u8, data: u8) -> bool {
    o == VEIN && data >= PATTERN_CENTRE
}

/// The vein connection pattern *c* of a cell (S-0202 claim 10, table *Vein
/// connections*).
fn connections(w: &World, c: CellXY) -> u8 {
    let r = w.rules();
    let veins_flag = |o: u8| r.overlays.get(usize::from(o)).is_some_and(|t| t.veins);
    let mut b = 0;
    for (i, (dx, dy)) in walls::EDGE_STEPS.iter().enumerate() {
        let Some(n) = w.cells.cell(CellXY::new(c.x + dx, c.y + dy)) else {
            continue;
        };
        let counts = if n.overlay == VEIN {
            // spec: S-0202 claim 10 (a pattern centre counts when flat, and when sloped only with the veins flag)
            pattern_centre(n.overlay, n.overlay_data) && (n.slope == 0 || veins_flag(VEIN))
        } else {
            // spec: S-0202 claim 10 (another type with the veins flag counts, flat or sloped)
            n.overlay != NO_OVERLAY && veins_flag(n.overlay)
        };
        if counts {
            b |= 1 << i;
        }
    }
    b
}

/// The overlay data of a sloped cell that a vein placement sets (S-0202
/// Formulas, *Vein pattern values*): 51 + 2s + (x mod 2).
fn sloped_value(slope: u8, x: u32) -> u8 {
    // spec: S-0202 Formulas (a sloped cell set by a placement: 51 + 2s + (x mod 2))
    (51 + 2 * u32::from(slope) + x % 2) as u8
}

/// Tries a vein placement on `c` (S-0202 claim 10, *the placement*).
fn vein_placement(w: &mut World, c: CellXY) {
    if !vein_test(w, c) {
        return;
    }
    let Some(slope) = w.cells.cell(c).map(|x| x.slope) else {
        return;
    };
    let overflow = w.rules().quirks.random_magnitude_overflow;
    let x = w.rng.raw();
    let data = if slope == 0 {
        // spec: S-0202 Formulas (the placed cell, flat: 48 + (|σ(x)| rem 3), "magnitude first")
        let s = x as i32;
        let m = if overflow {
            // spec: S-0202 Edge cases (quirks.random_magnitude_overflow: |−2147483648| stays negative)
            i64::from(s.wrapping_abs())
        } else {
            i64::from(s).abs()
        };
        (i64::from(PATTERN_CENTRE) + m % 3) as u8 // spec: S-0202 Formulas (rem 3)
    } else {
        sloped_value(slope, x)
    };
    if let Some(cell) = w.cells.grid.get_mut(c) {
        cell.overlay = VEIN;
        cell.overlay_data = data;
    }
    if slope != 0 {
        // spec: S-0202 claim 10 (a sloped placed cell does nothing more)
        return;
    }
    let veins_flag = |w: &World, o: u8| {
        w.rules()
            .overlays
            .get(usize::from(o))
            .is_some_and(|t| t.veins)
    };
    // spec: S-0202 claim 10 (the edge neighbours in the order map north, east, south, west)
    for (dx, dy) in walls::EDGE_STEPS {
        let n = CellXY::new(c.x + dx, c.y + dy);
        // spec: S-0009 claim 44 (a neighbour outside the map reads as a blank cell)
        let nc = w
            .cells
            .cell(n)
            .copied()
            .unwrap_or(glasnost_map::Cell::BLANK);
        if pattern_centre(nc.overlay, nc.overlay_data)
            || (nc.overlay != VEIN && nc.overlay != NO_OVERLAY && veins_flag(w, nc.overlay))
        {
            // spec: S-0202 claim 10 (a pattern centre or another veins-flag type: unchanged, no draw)
            continue;
        }
        let new = if nc.slope != 0 {
            // spec: S-0202 claim 10 (sloped, no overlay or a vein that is not a centre: one raw value)
            let x = w.rng.raw();
            sloped_value(nc.slope, x)
        } else {
            let cpat = connections(w, n);
            if nc.overlay == VEIN && nc.overlay_data / 3 == cpat {
                // spec: S-0202 claim 10 (a flat vein whose ⌊data / 3⌋ equals c: unchanged, no draw)
                continue;
            }
            // spec: S-0202 Formulas (a flat cell set or re-patterned: 3c + |σ(x) rem 3|, "magnitude after")
            let x = w.rng.raw();
            3 * cpat + ((x as i32) % 3).unsigned_abs() as u8 // spec: S-0202 Formulas (rem 3)
        };
        // spec: S-0009 claim 44 (our engine changes no cell outside the map; the raw value is taken)
        if let Some(cell) = w.cells.grid.get_mut(n) {
            cell.overlay = VEIN;
            cell.overlay_data = new;
        }
    }
}

/// The vein rebuild of S-0009 claim 14: every vein cell loses its vein;
/// the seeds (data 48 or more before this) are tried in the reverse of the
/// visiting order, each with the vein placement of S-0202 claim 10.
pub fn rebuild_veins(w: &mut World, order: &[CellXY]) {
    let mut seeds = Vec::new();
    for &c in order {
        if let Some(cell) = w.cells.grid.get_mut(c)
            && cell.overlay == VEIN
        {
            if cell.overlay_data >= PATTERN_CENTRE {
                seeds.push(c);
            }
            // spec: S-0009 claim 14 (every vein cell loses its overlay and its data value 0)
            cell.overlay = NO_OVERLAY;
            cell.overlay_data = 0;
        }
    }
    // spec: S-0009 claim 14 (the seeds in the reverse of the visiting order)
    for &c in seeds.iter().rev() {
        vein_placement(w, c);
    }
}
