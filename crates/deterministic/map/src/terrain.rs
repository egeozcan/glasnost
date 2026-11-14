//! A theater's tile catalogue and terrain recalculation (S-0010 claims 6 to
//! 12; S-0211 claims 5 to 8).
//!
//! The catalogue holds what a cell takes from its tile: each tile cell's
//! terrain code and slope shape, and the role starts that smoothing uses.
//! The importer fills it from the theater's files (S-0010 claims 1 to 5).
//! [`recalc`] is the terrain recalculation of one cell, which S-0211 calls
//! the *land recompute*; its list of triggers belongs to S-0211 claim 5 and
//! to the callers. The passability category (S-0211 claim 11) depends on the
//! objects in the cell and is the simulation's.

use serde::{Deserialize, Serialize};

use crate::{CellGrid, CellXY, NO_OVERLAY, NO_TILE};

/// Land types as S-0211 numbers them (Terms): labels with no behaviour of
/// their own.
pub mod land {
    /// Clear.
    pub const CLEAR: u8 = 0; // spec: S-0211 Terms (land type 0 clear)
    /// Road.
    pub const ROAD: u8 = 1; // spec: S-0211 Terms (land type 1 road)
    /// Water.
    pub const WATER: u8 = 2; // spec: S-0211 Terms (land type 2 water)
    /// Rock.
    pub const ROCK: u8 = 3; // spec: S-0211 Terms (land type 3 rock)
    /// Wall.
    pub const WALL: u8 = 4; // spec: S-0211 Terms (land type 4 wall)
    /// Tiberium.
    pub const TIBERIUM: u8 = 5; // spec: S-0211 Terms (land type 5 tiberium)
    /// Beach.
    pub const BEACH: u8 = 6; // spec: S-0211 Terms (land type 6 beach)
    /// Rough.
    pub const ROUGH: u8 = 7; // spec: S-0211 Terms (land type 7 rough)
    /// Ice.
    pub const ICE: u8 = 8; // spec: S-0211 Terms (land type 8 ice)
    /// Railroad.
    pub const RAILROAD: u8 = 9; // spec: S-0211 Terms (land type 9 railroad)
    /// Tunnel.
    pub const TUNNEL: u8 = 10; // spec: S-0211 Terms (land type 10 tunnel)
    /// Weeds.
    pub const WEEDS: u8 = 11; // spec: S-0211 Terms (land type 11 weeds)
}

/// One non-empty cell of a tile (S-0010 Terms, *tile cell*).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileCell {
    /// Terrain code, 0 … 15 in valid data.
    pub terrain_code: u8,
    /// Slope shape (S-0106), 0 … 20 in valid data; see [`TileCell::shape`].
    pub slope: u8,
}

/// The highest slope shape (S-0106: shapes 0 … 20).
const MAX_SHAPE: u8 = 20; // spec: S-0106 Edge cases (shape values 0 … 20)

impl TileCell {
    /// The slope shape this tile cell gives a cell: its slope code, or 0
    /// (flat) for a code outside 0 … 20.
    // spec: S-0106 Edge cases (a slope code outside 0 … 20: the cell is flat, shape 0)
    pub fn shape(self) -> u8 {
        if self.slope > MAX_SHAPE {
            0
        } else {
            self.slope
        }
    }
}

/// One tile of the theater.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileInfo {
    /// Width w in cells.
    pub width: u8,
    /// Height h in cells.
    pub height: u8,
    /// Whether the plain image was found (S-0010 claim 3). A tile without
    /// one has no tile cells.
    pub has_image: bool,
    /// The w × h cells, row by row; `None` for an empty cell.
    pub cells: Vec<Option<TileCell>>,
    /// Number of variant images besides the plain one (render only).
    pub variants: u8,
    /// The tile's animation, if its plain image names one (S-0010 claim 19).
    #[serde(default)]
    pub anim: Option<TileAnim>,
    /// Whether the tile's tileset has its allow-burrowing flag off (S-0010
    /// claim 17 and Import mapping; absent: allowed).
    #[serde(default)]
    pub no_burrowing: bool,
}

/// A tile's animation (S-0010 claim 19 and Import mapping): the animation
/// type's name as written and the sub-tile it attaches to (−1 when absent:
/// the animation never starts) and its screen displacement, which places
/// it (S-0010 Formulas, *Tile animation position*). Its depth adjustment
/// is the renderer's.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileAnim {
    /// The animation type's name as written (bytes 128 to 255 as U+F780 to
    /// U+F7FF).
    pub name: String,
    /// The attaches-to sub-tile.
    pub attaches_to: i32,
    /// The screen displacement *dx* in pixels (absent 0).
    #[serde(default)]
    pub dx: i32,
    /// The screen displacement *dy* in pixels (absent 0).
    #[serde(default)]
    pub dy: i32,
}

/// The role starts of S-0010 table R (claim 4), as tile numbers; an unset
/// role holds −1 (the destroyable-cliff role −2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[allow(missing_docs)]
pub struct Roles {
    pub clear: i32,
    pub water: i32,
    pub ramp: i32,
    pub smooth_ramp: i32,
    pub cliff: i32,
    pub shore: i32,
    pub water_bridge: i32,
    pub misc_pave: i32,
    pub pave_ground: i32,
    pub paved_road: i32,
    pub paved_road_end: i32,
    pub paved_road_slope: i32,
    pub median: i32,
    pub dirt_road_junction: i32,
    pub dirt_road_curve: i32,
    pub dirt_road_straight: i32,
    pub bridge: i32,
    pub wooden_bridge: i32,
    pub destroyable_cliff: i32,
    // spec: S-0010 Constants table R (four tunnel roles)
    pub tunnels: [i32; 4],
    pub rough_ground: i32,
    pub rough_transition: i32,
    pub sand_ground: i32,
    pub sand_transition: i32,
    pub green_ground: i32,
    pub green_transition: i32,
    pub pave_transition: i32,
}

/// The unset role start (S-0010 claim 4).
pub const UNSET: i32 = -1; // spec: S-0010 claim 4 (an unset role's start is −1)
/// The unset start of the destroyable-cliff role (S-0010 claim 4).
pub const UNSET_DESTROYABLE_CLIFF: i32 = -2; // spec: S-0010 claim 4 (destroyable cliff unset −2)

impl Default for Roles {
    fn default() -> Self {
        Self {
            clear: UNSET,
            water: UNSET,
            ramp: UNSET,
            smooth_ramp: UNSET,
            cliff: UNSET,
            shore: UNSET,
            water_bridge: UNSET,
            misc_pave: UNSET,
            pave_ground: UNSET,
            paved_road: UNSET,
            paved_road_end: UNSET,
            paved_road_slope: UNSET,
            median: UNSET,
            dirt_road_junction: UNSET,
            dirt_road_curve: UNSET,
            dirt_road_straight: UNSET,
            bridge: UNSET,
            wooden_bridge: UNSET,
            destroyable_cliff: UNSET_DESTROYABLE_CLIFF,
            // spec: S-0010 Constants table R (four tunnel roles)
            tunnels: [UNSET; 4],
            rough_ground: UNSET,
            rough_transition: UNSET,
            sand_ground: UNSET,
            sand_transition: UNSET,
            green_ground: UNSET,
            green_transition: UNSET,
            pave_transition: UNSET,
        }
    }
}

/// The ten bridge tile positions of a theater's general section (S-0511
/// Terms and Import mapping): 1-based positions *q* within a bridge role,
/// −1 when unset (matching no tile). Index 0 holds the key ending in 1,
/// index 1 the key ending in 2.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
// spec: S-0511 Import mapping (two keys per position, ending in 1 and 2)
pub struct BridgePositions {
    /// Start ramps along x (`BridgeTopLeft1`, `BridgeTopLeft2`).
    pub top_left: [i32; 2],
    /// Start ramps along y (`BridgeTopRight1`, `BridgeTopRight2`).
    pub top_right: [i32; 2],
    /// Far ramps along x (`BridgeBottomRight1`, `BridgeBottomRight2`).
    pub bottom_right: [i32; 2],
    /// Far ramps along y (`BridgeBottomLeft1`, `BridgeBottomLeft2`).
    pub bottom_left: [i32; 2],
    /// The pier bases *m*₁ (along x, `BridgeMiddle1`) and *m*₂ (along y,
    /// `BridgeMiddle2`).
    pub middle: [i32; 2],
}

/// An unset bridge position (S-0511 Import mapping).
pub const UNSET_POSITION: i32 = -1; // spec: S-0511 Import mapping (absent: −1, matching no tile)

impl Default for BridgePositions {
    fn default() -> Self {
        // spec: S-0511 Import mapping (two keys per position, both unset)
        let u = [UNSET_POSITION; 2];
        Self {
            top_left: u,
            top_right: u,
            bottom_right: u,
            bottom_left: u,
            middle: u,
        }
    }
}

/// A theater's tile catalogue.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileCatalogue {
    /// Tiles by tile number (S-0010 claim 2); the length is the tile count.
    pub tiles: Vec<TileInfo>,
    /// Role starts.
    pub roles: Roles,
    /// Each tile's tileset allow-tiberium flag, by tile number (S-0010
    /// claim 16); a tile without an entry has the flag off.
    #[serde(default)]
    pub allow_tiberium: Vec<bool>,
    /// The theater's bridge tile positions (S-0511 Terms).
    #[serde(default)]
    pub bridge_positions: BridgePositions,
}

impl TileCatalogue {
    /// The tile count.
    pub fn tile_count(&self) -> i32 {
        self.tiles.len() as i32
    }

    /// The tile cell a tile number and sub-tile name, if the tile is usable
    /// for that sub-tile (S-0010 claim 7).
    pub fn tile_cell(&self, tile: i32, sub_tile: u8) -> Option<TileCell> {
        let t = self.tiles.get(usize::try_from(tile).ok()?)?;
        if !t.has_image {
            return None;
        }
        t.cells.get(usize::from(sub_tile)).copied().flatten()
    }
}

/// What a cell's overlay contributes to its land type (S-0211 Terms,
/// *overlay flags*).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OverlayLand {
    /// The overlay type's land type (S-0211 claim 4; `none` already read as
    /// clear by the importer).
    pub land: u8,
    /// The own-land-type flag.
    pub own_land: bool,
    /// The tiberium flag.
    pub tiberium: bool,
}

impl OverlayLand {
    /// Row 1 of S-0211 claim 5: land type wall or railroad, or the
    /// own-land-type flag.
    pub fn is_row_one(&self) -> bool {
        self.own_land || self.land == land::WALL || self.land == land::RAILROAD
    }
}

/// The switches the recalculation reads (all default to the original).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerrainQuirks {
    /// `quirks.cliff_back_case_sets` (S-0211 claim 8).
    pub cliff_back_case_sets: bool,
    /// `quirks.unset_role_range` (S-0010 claims 5, 10, 11).
    pub unset_role_range: bool,
    /// `quirks.ramp_shape_zero` (S-0010 claim 11): a tile of the ramp
    /// range or of the smooth-ramp range whose slope shape is 0 becomes the
    /// tile just before the ramp range; off, it keeps its tile.
    pub ramp_shape_zero: bool,
}

impl Default for TerrainQuirks {
    fn default() -> Self {
        Self {
            cliff_back_case_sets: true,
            unset_role_range: true,
            ramp_shape_zero: true,
        }
    }
}

/// The rules a recalculation needs.
pub struct RecalcRules<'a> {
    /// The theater's catalogue.
    pub catalogue: &'a TileCatalogue,
    /// The overlay type at a list index, `None` when the index names no type.
    pub overlay: &'a dyn Fn(u8) -> Option<OverlayLand>,
    /// The cliff-back setting (S-0211 claim 8).
    pub cliff_back: i32,
    /// Switches.
    pub quirks: TerrainQuirks,
}

/// The land type of a tile land code (S-0211 claim 6). Codes outside 0 … 15
/// give clear (S-0211 claim 6, departure).
pub fn land_of_code(k: u8) -> u8 {
    // spec: S-0211 claim 6 (tile land code table; codes outside 0 … 15: clear, departure)
    match k {
        0 => land::CLEAR,
        1..=4 => land::ICE,
        5 => land::TUNNEL,
        6 => land::RAILROAD,
        7 | 8 => land::ROCK,
        9 => land::WATER,
        10 => land::BEACH,
        11 | 12 => land::ROAD,
        13 => land::CLEAR,
        14 => land::ROUGH,
        15 => land::ROCK,
        _ => land::CLEAR,
    }
}

/// The result of one cell's recalculation that callers act on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Recalc {
    /// The cell's ore or gem overlay was removed (S-0211 claim 7), which
    /// changes connectivity like any overlay change.
    pub overlay_removed: bool,
}

/// The six neighbours that the cliff-back rule looks at (S-0211 claim 8).
// spec: S-0211 claim 8 (the six neighbour cells)
const CLIFF_BACK_NEIGHBOURS: [(i32, i32); 6] = [(0, -1), (1, -1), (-1, 0), (-1, 1), (1, 1), (2, 2)];

/// The terrain recalculation of cell `c` (S-0010 claims 6 to 11, S-0211
/// claims 5 to 8). Neighbours outside the map read as blank cells: ground
/// level 0, flat, the no-tile marker (S-0009 claim 44; S-0211 claim 8 for
/// the cliff-back rule).
pub fn recalc(grid: &mut CellGrid, c: CellXY, rules: &RecalcRules<'_>) -> Recalc {
    let Some(cell) = grid.get(c).copied() else {
        return Recalc::default();
    };
    let cat = rules.catalogue;
    let count = cat.tile_count();
    let overlay = if cell.overlay == NO_OVERLAY {
        None
    } else {
        (rules.overlay)(cell.overlay)
    };
    let mut out = Recalc::default();
    let mut new = cell;
    let usable = cat.tile_cell(cell.tile, cell.sub_tile);
    // spec: S-0010 claim 8 (overlay case, no-tile case, tile case)
    let overlay_case = overlay.is_some_and(|o| o.is_row_one());
    let land_type;
    let smoothed;
    if overlay_case {
        let o = overlay.unwrap_or_default();
        // spec: S-0010 claim 7 (overlay case: tile kept; shape of the tile cell at t mod (w × h), else flat)
        if (0..count).contains(&cell.tile) {
            let t = &cat.tiles[cell.tile as usize];
            let n = usize::from(t.width) * usize::from(t.height);
            new.slope = if t.has_image && n > 0 {
                t.cells
                    .get(usize::from(cell.sub_tile) % n)
                    .copied()
                    .flatten()
                    .map_or(0, TileCell::shape)
            } else {
                0
            };
        }
        // spec: S-0211 claim 5 (row 1: the overlay's land type)
        land_type = o.land;
        smoothed = true;
    } else if let Some(tc) = usable.filter(|_| (0..count).contains(&cell.tile)) {
        // spec: S-0010 claim 6 (tile case: the slope shape of the tile cell), S-0106 Edge cases (a code outside 0 … 20 is flat)
        new.slope = tc.shape();
        let tib = overlay.filter(|o| o.tiberium);
        // spec: S-0211 claims 5 and 7 (row 4: ore on slope shapes 0 … 4; removed on 5 and above)
        land_type = match tib {
            Some(o) if new.slope <= 4 => {
                if o.land == land::CLEAR {
                    land::TIBERIUM
                } else {
                    o.land
                }
            }
            Some(_) => {
                new.overlay = NO_OVERLAY;
                out.overlay_removed = true;
                land_of_code(tc.terrain_code)
            }
            // spec: S-0211 claim 5 (row 5: the tile land code)
            None => land_of_code(tc.terrain_code),
        };
        smoothed = true;
    } else {
        // spec: S-0010 claim 7 (no-tile case: the no-tile marker, sub-tile 0, flat)
        let beyond = cell.tile < 0 || cell.tile >= count || cell.tile == NO_TILE;
        new.tile = NO_TILE;
        new.sub_tile = 0;
        new.slope = 0;
        // spec: S-0211 claim 5 (row 2: the overlay's land type or clear; row 3: clear)
        land_type = if beyond {
            overlay.map_or(land::CLEAR, |o| o.land)
        } else {
            land::CLEAR
        };
        smoothed = false;
    }
    // spec: S-0211 claim 7 (a tiberium overlay with a row-1 property is removed on any slope; land kept)
    if overlay_case
        && let Some(o) = overlay
        && o.tiberium
        && new.slope != 0
    {
        new.overlay = NO_OVERLAY;
        out.overlay_removed = true;
    }
    new.land = cliff_back(grid, c, cell.level, land_type, overlay_case, rules);
    if smoothed {
        new.tile = smooth_ground(grid, c, new.tile, rules);
        new.tile = smooth_ramp(grid, c, new.tile, new.slope, rules);
    }
    if let Some(slot) = grid.get_mut(c) {
        *slot = new;
    }
    out
}

/// The cliff-back rule (S-0211 claim 8).
fn cliff_back(
    grid: &CellGrid,
    c: CellXY,
    level: i8,
    land_type: u8,
    row_one: bool,
    rules: &RecalcRules<'_>,
) -> u8 {
    // spec: S-0211 claim 8 (active when the setting mod 256 is 2)
    if rules.cliff_back.rem_euclid(256) != 2 {
        return land_type;
    }
    let h = i32::from(level);
    let behind = CLIFF_BACK_NEIGHBOURS.iter().any(|&(dx, dy)| {
        let lv = grid
            .get(CellXY::new(c.x + dx, c.y + dy))
            .map_or(0, |n| i32::from(n.level));
        // spec: S-0211 claim 8 (a neighbour at h + 4 or more)
        lv >= h + 4
    });
    if !behind {
        return land_type;
    }
    let clear_set = matches!(
        land_type,
        land::CLEAR | land::WATER | land::BEACH | land::ICE
    );
    // spec: S-0211 claim 8 (row 1 always with quirks.cliff_back_case_sets; rows 2 to 5 the clear set)
    if (row_one && rules.quirks.cliff_back_case_sets) || clear_set {
        land::ROCK
    } else {
        land_type
    }
}

/// A role's range for smoothing: `n` tiles from `start`, or only −1 for an
/// unset role (S-0010 claims 10 and 11, `quirks.unset_role_range`).
fn in_range(tile: i32, start: i32, n: i32, q: TerrainQuirks) -> bool {
    if start < 0 {
        // spec: S-0010 claim 10 (in smoothing an unset role's range holds only −1)
        return q.unset_role_range && tile == UNSET;
    }
    (start..start + n).contains(&tile)
}

fn neighbour_tile(grid: &CellGrid, c: CellXY, dx: i32, dy: i32) -> i32 {
    grid.get(CellXY::new(c.x + dx, c.y + dy))
        .map_or(NO_TILE, |n| n.tile)
}

fn neighbour_slope(grid: &CellGrid, c: CellXY, dx: i32, dy: i32) -> u8 {
    grid.get(CellXY::new(c.x + dx, c.y + dy))
        .map_or(0, |n| n.slope)
}

/// Ground smoothing (S-0010 claim 10).
fn smooth_ground(grid: &CellGrid, c: CellXY, tile: i32, rules: &RecalcRules<'_>) -> i32 {
    let r = rules.catalogue.roles;
    let q = rules.quirks;
    // spec: S-0010 Constants table R (16 transition tiles per pair; 42 shore, 2 water bridge, 14 misc pave, 21 paved road, 14 median)
    let pairs: [(i32, i32, u8); 4] = [
        (r.rough_ground, r.rough_transition, 0),
        (r.sand_ground, r.sand_transition, 1),
        (r.green_ground, r.green_transition, 2),
        (r.pave_ground, r.pave_transition, 3),
    ];
    let mut tile = tile;
    // spec: S-0010 claim 10 (pairs in turn: rough, sand, green, pave)
    for (g, l, which) in pairs {
        // spec: S-0010 claim 10 (sand, green and pave skipped when their transition role is unset)
        if l < 0 && (which != 0 || !q.unset_role_range) {
            continue;
        }
        // spec: S-0010 Edge cases (quirks.unset_role_range off: a pair with an unset ground role is skipped too)
        if g < 0 && !q.unset_role_range {
            continue;
        }
        let belongs = |t: i32| {
            // spec: S-0010 claim 10 (belongs: the ground start, or the 16 transition tiles)
            (if g < 0 {
                q.unset_role_range && t == UNSET
            } else {
                t == g
            }) || in_range(t, l, 16, q)
        };
        if !belongs(tile) {
            continue;
        }
        let counts = |t: i32| {
            belongs(t)
                || match which {
                    // spec: S-0010 claim 10 (green: shore 42 and water bridge 2 count)
                    2 => in_range(t, r.shore, 42, q) || in_range(t, r.water_bridge, 2, q),
                    // spec: S-0010 claim 10 (pave: misc pave 14, paved road 21, median 14 count)
                    3 => {
                        in_range(t, r.misc_pave, 14, q)
                            || in_range(t, r.paved_road, 21, q)
                            || in_range(t, r.median, 14, q)
                    }
                    _ => false,
                }
        };
        // spec: S-0010 Formulas (m = [N out] + 2[E out] + 4[S out] + 8[W out])
        let m = i32::from(!counts(neighbour_tile(grid, c, 0, -1)))
            + 2 * i32::from(!counts(neighbour_tile(grid, c, 1, 0)))
            + 4 * i32::from(!counts(neighbour_tile(grid, c, 0, 1)))
            + 8 * i32::from(!counts(neighbour_tile(grid, c, -1, 0)));
        tile = if m == 0 {
            if g < 0 { UNSET } else { g }
        } else {
            l.max(UNSET) + m
        };
    }
    tile
}

/// Ramp smoothing (S-0010 claim 11).
fn smooth_ramp(grid: &CellGrid, c: CellXY, tile: i32, s: u8, rules: &RecalcRules<'_>) -> i32 {
    let r = rules.catalogue.roles;
    let q = rules.quirks;
    // spec: S-0010 Edge cases (quirks.unset_role_range off: ramp smoothing is skipped when either ramp role is unset)
    if !q.unset_role_range && (r.ramp < 0 || r.smooth_ramp < 0) {
        return tile;
    }
    // spec: S-0010 claim 11 (ramp range 20 tiles, smooth-ramp range 12 tiles)
    if !(in_range(tile, r.ramp, 20, q) || in_range(tile, r.smooth_ramp, 12, q)) {
        return tile;
    }
    let s = i32::from(s);
    // spec: S-0010 claim 11 (neighbours a (weight 1) and b (weight 2) by shape 1 … 4)
    let (a, b) = match s {
        1 => ((-1, 0), (1, 0)),
        2 => ((0, -1), (0, 1)),
        3 => ((1, 0), (-1, 0)),
        4 => ((0, 1), (0, -1)),
        _ => ((0, 0), (0, 0)),
    };
    // spec: S-0010 Formulas (m = [shape(a) = 0] + 2·[shape(b) = 0] for 1 ≤ s ≤ 4)
    let m = if (1..=4).contains(&s) {
        i32::from(neighbour_slope(grid, c, a.0, a.1) == 0)
            + 2 * i32::from(neighbour_slope(grid, c, b.0, b.1) == 0)
    } else {
        0
    };
    // spec: S-0010 claim 11 (shape 0 in either range: ramp start − 1 by the formula below, m being 0; quirks.ramp_shape_zero off keeps the tile)
    if s == 0 && !q.ramp_shape_zero {
        return tile;
    }
    // spec: S-0010 Formulas (smooth-ramp start + 3(s − 1) + m − 1 when m > 0, else ramp start + s − 1)
    if m > 0 {
        r.smooth_ramp.max(UNSET) + 3 * (s - 1) + m - 1
    } else {
        r.ramp.max(UNSET) + s - 1
    }
}

/// The map-load recalculation passes (S-0010 claim 12): every legal cell in
/// the visiting order. Returns the cells whose overlay was removed.
pub fn recalc_pass(grid: &mut CellGrid, order: &[CellXY], rules: &RecalcRules<'_>) -> Vec<CellXY> {
    let mut removed = Vec::new();
    for &c in order {
        if recalc(grid, c, rules).overlay_removed {
            removed.push(c);
        }
    }
    removed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cell, Footprint};

    fn flat_catalogue() -> TileCatalogue {
        let one = |code: u8, slope: u8| TileInfo {
            width: 1,
            height: 1,
            has_image: true,
            cells: vec![Some(TileCell {
                terrain_code: code,
                slope,
            })],
            variants: 0,
            anim: None,
            no_burrowing: false,
        };
        let mut tiles = vec![one(0, 0), one(9, 0), one(11, 0), one(14, 0)];
        for s in 1..=4 {
            tiles.push(one(0, s));
        }
        TileCatalogue {
            tiles,
            roles: Roles::default(),
            allow_tiberium: Vec::new(),
            bridge_positions: Default::default(),
        }
    }

    fn grid() -> CellGrid {
        let fp = Footprint::new(8, 8, [0, 0, 8, 8]);
        let mut g = CellGrid::new(fp);
        let cells: Vec<CellXY> = g.iter().map(|(c, _)| c).collect();
        for c in cells {
            if let Some(cell) = g.get_mut(c) {
                *cell = Cell {
                    tile: 0,
                    ..Cell::BLANK
                };
            }
        }
        g
    }

    fn no_overlay(_: u8) -> Option<OverlayLand> {
        None
    }

    fn rules<'a>(
        cat: &'a TileCatalogue,
        ov: &'a dyn Fn(u8) -> Option<OverlayLand>,
    ) -> RecalcRules<'a> {
        RecalcRules {
            catalogue: cat,
            overlay: ov,
            cliff_back: 0,
            quirks: TerrainQuirks::default(),
        }
    }

    // spec: S-0211 claim 6 (tile land codes)
    #[test]
    fn land_codes() {
        let want = [0, 8, 8, 8, 8, 10, 9, 3, 3, 2, 6, 1, 1, 0, 7, 3];
        for (k, w) in want.iter().enumerate() {
            assert_eq!(land_of_code(k as u8), *w, "code {k}");
        }
        assert_eq!(land_of_code(16), land::CLEAR);
    }

    // spec: S-0211 claim 5 (rows 2, 3 and 5) and S-0010 claim 7 (no-tile case)
    #[test]
    fn rows_of_claim_five() {
        let cat = flat_catalogue();
        let ov = no_overlay;
        let r = rules(&cat, &ov);
        let mut g = grid();
        let c = CellXY::new(8, 8);
        if let Some(cell) = g.get_mut(c) {
            cell.tile = 1;
        }
        recalc(&mut g, c, &r);
        assert_eq!(g.get(c).map(|x| x.land), Some(land::WATER));
        // A tile beyond the count: the no-tile marker, flat, clear.
        if let Some(cell) = g.get_mut(c) {
            cell.tile = 99;
        }
        recalc(&mut g, c, &r);
        let cell = g.get(c).copied().unwrap_or(Cell::BLANK);
        assert_eq!(
            (cell.tile, cell.slope, cell.land),
            (NO_TILE, 0, land::CLEAR)
        );
        // An unusable sub-tile: dropped, clear.
        if let Some(cell) = g.get_mut(c) {
            cell.tile = 1;
            cell.sub_tile = 3;
        }
        recalc(&mut g, c, &r);
        assert_eq!(g.get(c).map(|x| x.tile), Some(NO_TILE));
    }

    // spec: S-0211 claims 5 and 7 (ore on slopes 0 … 4 is tiberium land; removed on 5 and above)
    #[test]
    fn ore_rows() {
        let mut cat = flat_catalogue();
        cat.tiles.push(TileInfo {
            width: 1,
            height: 1,
            has_image: true,
            cells: vec![Some(TileCell {
                terrain_code: 0,
                slope: 7,
            })],
            variants: 0,
            anim: None,
            no_burrowing: false,
        });
        let ov = |i: u8| {
            (i == 3).then_some(OverlayLand {
                land: land::CLEAR,
                own_land: false,
                tiberium: true,
            })
        };
        let r = rules(&cat, &ov);
        let mut g = grid();
        let c = CellXY::new(8, 8);
        if let Some(cell) = g.get_mut(c) {
            cell.overlay = 3;
        }
        recalc(&mut g, c, &r);
        assert_eq!(g.get(c).map(|x| x.land), Some(land::TIBERIUM));
        if let Some(cell) = g.get_mut(c) {
            cell.tile = 8; // slope 7
        }
        let res = recalc(&mut g, c, &r);
        assert!(res.overlay_removed);
        assert_eq!(g.get(c).map(|x| x.overlay), Some(NO_OVERLAY));
    }

    // spec: S-0211 claim 8 (behind a cliff: clear set and row 1 become rock; road kept)
    #[test]
    fn cliff_back_rule() {
        let cat = flat_catalogue();
        let ov = no_overlay;
        let mut r = rules(&cat, &ov);
        r.cliff_back = 258; // acts as 2
        let mut g = grid();
        let c = CellXY::new(8, 8);
        if let Some(n) = g.get_mut(CellXY::new(8, 7)) {
            n.level = 4;
        }
        recalc(&mut g, c, &r);
        assert_eq!(g.get(c).map(|x| x.land), Some(land::ROCK));
        if let Some(cell) = g.get_mut(c) {
            cell.tile = 2; // road
        }
        recalc(&mut g, c, &r);
        assert_eq!(g.get(c).map(|x| x.land), Some(land::ROAD));
        r.cliff_back = 3;
        if let Some(cell) = g.get_mut(c) {
            cell.tile = 0;
        }
        recalc(&mut g, c, &r);
        assert_eq!(g.get(c).map(|x| x.land), Some(land::CLEAR));
    }

    // spec: S-0211 claim 8 (a neighbour 4 or more levels up counts: a rise of 3 does not, one of 6 does; scenario L5 runs L to O)
    #[test]
    fn cliff_back_rise_of_four_or_more() {
        let cat = flat_catalogue();
        let ov = no_overlay;
        let mut r = rules(&cat, &ov);
        r.cliff_back = 2;
        let c = CellXY::new(8, 8);
        for (rise, want) in [(3, land::CLEAR), (4, land::ROCK), (6, land::ROCK)] {
            let mut g = grid();
            if let Some(n) = g.get_mut(CellXY::new(8, 7)) {
                n.level = rise;
            }
            recalc(&mut g, c, &r);
            assert_eq!(g.get(c).map(|x| x.land), Some(want), "rise {rise}");
        }
    }

    // spec: S-0211 claim 8 (a neighbour outside the map reads level 0: behind a cliff only at level −4 or lower)
    // spec: S-0009 claim 44 (a cell outside the map reads as a blank cell)
    #[test]
    fn cliff_back_off_map_neighbour_reads_level_zero() {
        let cat = flat_catalogue();
        let ov = no_overlay;
        let mut r = rules(&cat, &ov);
        r.cliff_back = 2;
        let mut g = grid();
        let c = g
            .iter()
            .map(|(c, _)| c)
            .find(|c| {
                g.get(CellXY::new(c.x, c.y - 1)).is_none()
                    && CLIFF_BACK_NEIGHBOURS[1..].iter().all(|&(dx, dy)| {
                        g.get(CellXY::new(c.x + dx, c.y + dy))
                            .is_none_or(|n| n.level == 0)
                    })
            })
            .expect("an edge cell");
        for (level, want) in [(-4, land::ROCK), (-3, land::CLEAR)] {
            if let Some(cell) = g.get_mut(c) {
                cell.level = level;
                cell.tile = 0;
            }
            recalc(&mut g, c, &r);
            assert_eq!(g.get(c).map(|x| x.land), Some(want), "level {level}");
        }
    }

    // spec: S-0009 claim 44 (an off-map neighbour reads as a blank cell: level 0 in the cliff-back rule)
    #[test]
    fn claim_44_off_map_neighbour_is_blank() {
        let cat = flat_catalogue();
        let ov = no_overlay;
        let mut r = rules(&cat, &ov);
        r.cliff_back = 258; // acts as 2
        let mut g = grid();
        // (1, 8) lies on the edge: its neighbour (1, 7) is not a legal cell.
        let c = CellXY::new(1, 8);
        assert!(g.get(CellXY::new(1, 7)).is_none());
        for (level, expected) in [(-3, land::CLEAR), (-4, land::ROCK)] {
            if let Some(cell) = g.get_mut(c) {
                cell.level = level;
            }
            recalc(&mut g, c, &r);
            assert_eq!(g.get(c).map(|x| x.land), Some(expected), "level {level}");
        }
    }

    // spec: S-0010 claim 10 (ground smoothing: G inside a patch, L + m at its edge)
    #[test]
    fn ground_smoothing() {
        let mut cat = flat_catalogue();
        // Tiles 8 … 23 are a rough transition set; tile 3 is the rough ground.
        for _ in 0..16 {
            cat.tiles.push(TileInfo {
                width: 1,
                height: 1,
                has_image: true,
                cells: vec![Some(TileCell {
                    terrain_code: 14,
                    slope: 0,
                })],
                variants: 0,
                anim: None,
                no_burrowing: false,
            });
        }
        cat.roles.rough_ground = 3;
        cat.roles.rough_transition = 8;
        let ov = no_overlay;
        let r = rules(&cat, &ov);
        let mut g = grid();
        let c = CellXY::new(8, 8);
        if let Some(cell) = g.get_mut(c) {
            cell.tile = 3;
        }
        recalc(&mut g, c, &r);
        // All four neighbours out: m = 15.
        assert_eq!(g.get(c).map(|x| x.tile), Some(8 + 15));
        // The land type comes from the tile the recalculation began with.
        assert_eq!(g.get(c).map(|x| x.land), Some(land::ROUGH));
    }

    // spec: S-0010 claim 11 (a tile of shape 0 in the ramp range or in the smooth-ramp range becomes ramp start − 1; quirks.ramp_shape_zero off keeps it; scenario L4 parts A and B)
    #[test]
    fn ramp_shape_zero_in_both_ranges() {
        let mut cat = flat_catalogue();
        cat.roles.ramp = 4; // tiles 4 … 7 have shapes 1 … 4
        // Tiles 8 … 24 are flat (shape 0); tile 24 starts the smooth-ramp
        // range, outside the ramp range 4 … 23.
        while cat.tiles.len() < 25 {
            cat.tiles.push(TileInfo {
                width: 1,
                height: 1,
                has_image: true,
                cells: vec![Some(TileCell {
                    terrain_code: 0,
                    slope: 0,
                })],
                variants: 0,
                anim: None,
                no_burrowing: false,
            });
        }
        cat.roles.smooth_ramp = 24;
        let ov = no_overlay;
        let c = CellXY::new(8, 8);
        for (placed, quirk, want) in [
            (24, true, 3),   // smooth-ramp range, shape 0: ramp start − 1
            (24, false, 24), // switch off: kept
        ] {
            let mut r = rules(&cat, &ov);
            r.quirks.ramp_shape_zero = quirk;
            let mut g = grid();
            if let Some(cell) = g.get_mut(c) {
                cell.tile = placed;
            }
            recalc(&mut g, c, &r);
            assert_eq!(
                g.get(c).map(|x| x.tile),
                Some(want),
                "tile {placed}, switch {quirk}"
            );
        }
        // The ramp range: tile 4 carries shape 1 in the catalogue, so a
        // catalogue whose ramp-range tile 4 is flat shows part A.
        let mut cat_a = cat.clone();
        cat_a.tiles[4].cells = vec![Some(TileCell {
            terrain_code: 0,
            slope: 0,
        })];
        for (quirk, want) in [(true, 3), (false, 4)] {
            let mut r = rules(&cat_a, &ov);
            r.quirks.ramp_shape_zero = quirk;
            let mut g = grid();
            if let Some(cell) = g.get_mut(c) {
                cell.tile = 4;
            }
            recalc(&mut g, c, &r);
            assert_eq!(
                g.get(c).map(|x| x.tile),
                Some(want),
                "ramp range, switch {quirk}"
            );
        }
    }

    // spec: S-0010 claim 11 (ramp smoothing by the flat neighbours)
    #[test]
    fn ramp_smoothing() {
        let mut cat = flat_catalogue();
        cat.roles.ramp = 4; // tiles 4 … 7 have shapes 1 … 4
        cat.roles.smooth_ramp = 100;
        let ov = no_overlay;
        let r = rules(&cat, &ov);
        let mut g = grid();
        let c = CellXY::new(8, 8);
        if let Some(cell) = g.get_mut(c) {
            cell.tile = 4; // shape 1: a = W, b = E, both flat: m = 3
        }
        recalc(&mut g, c, &r);
        assert_eq!(g.get(c).map(|x| x.slope), Some(1));
        assert_eq!(g.get(c).map(|x| x.tile), Some(100 + 2));
    }
}
