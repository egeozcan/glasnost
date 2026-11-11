//! Typed map consumed by the simulation and the renderer.
//!
//! **Milestone M2** defines the decoded map (S-0009): footprint and local
//! area, the cell grid with tiles, heights and overlays, waypoint markers,
//! the header block, the preview, the map's lighting values (S-0104) and the
//! objects the map places. `glasnost_convert::map` fills it from the user's
//! map file; this crate only holds integer data and the integer rules of
//! S-0009 that need no rules data. **M3** connects it to the simulation.
//! This is a simulation crate: the determinism lints in
//! `crates/deterministic/clippy.toml` apply.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub mod bridges;
pub mod geometry;
pub mod objects;
pub mod terrain;
pub mod triggers;

pub use geometry::{CellXY, Footprint};
pub use objects::{
    Aircraft, EntryFields, Infantry, ListKind, Objects, Smudge, Structure, TerrainObject, Unit,
};

/// Number of waypoint markers, numbered 0 to 701 (S-0009 claim 15).
pub const WAYPOINT_COUNT: u16 = 702; // spec: S-0009 Constants (waypoint count)

/// Default home waypoint (S-0009 claim 16).
pub const DEFAULT_HOME_WAYPOINT: u16 = 699; // spec: S-0009 Constants (default home waypoint)

/// Start waypoint markers are 0 to 7 (S-0009 Terms).
pub const START_WAYPOINTS: u16 = 8; // spec: S-0009 Constants (start waypoint markers)

/// The no-tile marker of the terrain stream (S-0009 claim 8).
pub const NO_TILE: i32 = 65535; // spec: S-0009 Constants (no-tile marker)

/// The "no overlay" value of the overlay type stream (S-0009 claim 12).
pub const NO_OVERLAY: u8 = 255; // spec: S-0009 Constants (no-overlay value)

/// One height level in leptons (S-0009 claim 9).
pub const LEVEL_HEIGHT: i32 = 104; // spec: S-0009 Constants (level height)

/// The six theaters, by index (S-0009 Import mapping).
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Theater {
    /// Index 0.
    #[default]
    Temperate,
    /// Index 1.
    Snow,
    /// Index 2.
    Urban,
    /// Index 3.
    Desert,
    /// Index 4.
    NewUrban,
    /// Index 5.
    Lunar,
}

impl Theater {
    /// The theater's index in the list of S-0009 Import mapping.
    pub fn index(self) -> u8 {
        // spec: S-0009 Import mapping ([Map] Theater: theaters 0 to 5)
        match self {
            Self::Temperate => 0,
            Self::Snow => 1,
            Self::Urban => 2,
            Self::Desert => 3,
            Self::NewUrban => 4,
            Self::Lunar => 5,
        }
    }
}

/// The map's initial fill setting (S-0009 claim 6).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Fill {
    /// Every visited cell gets the clear base tile.
    #[default]
    Clear,
    /// Every visited cell gets the water base tile plus a ranged draw.
    Water,
}

/// One cell's terrain (S-0009 claims 6 to 8) and overlay (claims 12, 13).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    /// Tile number as stored: signed, the no-tile marker [`NO_TILE`] kept
    /// (claim 8: the translated number is stored as it is).
    pub tile: i32,
    /// Sub-tile.
    pub sub_tile: u8,
    /// Height level, −128 to 127 (claim 7).
    pub level: i8,
    /// Ice-growth flag.
    pub ice: bool,
    /// Overlay type-list index, or [`NO_OVERLAY`]. This is the value of the
    /// overlay type stream; whether an overlay is created needs the rules
    /// data (claim 12) and is decided by the importer.
    pub overlay: u8,
    /// Overlay data value (claim 13).
    pub overlay_data: u8,
    /// Whether a terrain stream record set this cell (S-0009 claim 7); the
    /// cells it did not set keep the initial fill (claim 6).
    pub stream: bool,
    /// Slope shape after the last terrain recalculation (S-0010 claim 6).
    pub slope: u8,
    /// Land type after the last terrain recalculation (S-0211 claim 5), as
    /// [`terrain::land`] numbers it.
    pub land: u8,
}

impl Cell {
    // spec: S-0009 Terms (blank cell)
    /// A blank cell (S-0009 Terms): tile 65535, sub-tile 0, level 0, no
    /// overlay, data value 0.
    pub const BLANK: Cell = Cell {
        tile: NO_TILE,
        sub_tile: 0,
        level: 0,
        ice: false,
        overlay: NO_OVERLAY,
        overlay_data: 0,
        stream: false,
        slope: 0,
        land: terrain::land::CLEAR,
    };

    /// World height of the cell's flat floor in leptons (claim 9).
    pub fn floor_height(&self) -> i32 {
        // spec: S-0009 Formulas (z_floor = 104 × level)
        LEVEL_HEIGHT * i32::from(self.level)
    }
}

/// The cells of a map, stored densely over the coordinate square that holds
/// every legal cell (S-0009 claim 2: 1 ≤ x, y ≤ W + H − 1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellGrid {
    footprint: Footprint,
    side: u16,
    cells: Vec<Cell>,
}

impl CellGrid {
    /// A grid of blank cells for `footprint`.
    pub fn new(footprint: Footprint) -> Self {
        let side = footprint.coordinate_side();
        let n = usize::from(side) * usize::from(side);
        Self {
            footprint,
            side,
            cells: vec![Cell::BLANK; n],
        }
    }

    /// The number of dense indices (the coordinate square's area).
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Whether the grid holds together, as [`CellGrid::new`] makes it: its
    /// side is the footprint's coordinate side and it holds side × side
    /// cells. A grid decoded from bytes may not (#157).
    pub fn is_consistent(&self) -> bool {
        self.side == self.footprint.coordinate_side()
            && usize::from(self.side) * usize::from(self.side) == self.cells.len()
    }

    /// Whether the grid holds no cell.
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// The map's footprint.
    pub fn footprint(&self) -> &Footprint {
        &self.footprint
    }

    /// Replaces the local area (Lx, Ly, Lw, Lh), as the resize action of
    /// S-0524 claim 8 does in play. The legal cells and the dense indices
    /// depend only on W and H, so they stay.
    // spec: S-0524 claim 8 (four items: left, top, width, height)
    pub fn set_local(&mut self, local: [i32; 4]) {
        self.footprint.local = local;
    }

    /// The dense index of a legal cell (row-major over the coordinate
    /// square), for side tables kept beside the grid.
    pub fn index(&self, c: CellXY) -> Option<usize> {
        if !self.footprint.is_legal(c) {
            return None;
        }
        let x = usize::try_from(c.x).ok()?;
        let y = usize::try_from(c.y).ok()?;
        let side = usize::from(self.side);
        (x < side && y < side).then_some(y * side + x)
    }

    /// The cell at `c`, if it is legal.
    pub fn get(&self, c: CellXY) -> Option<&Cell> {
        self.index(c).and_then(|i| self.cells.get(i))
    }

    /// The cell at `c` for writing, if it is legal.
    pub fn get_mut(&mut self, c: CellXY) -> Option<&mut Cell> {
        let i = self.index(c)?;
        self.cells.get_mut(i)
    }

    /// Every legal cell in the visiting order of S-0009 claim 3, with the
    /// W = 1 edge case of `quirks.narrow_map_partial_pass` when `narrow_quirk`
    /// is on.
    pub fn visiting_order(&self, narrow_quirk: bool) -> Vec<CellXY> {
        self.footprint.visiting_order(narrow_quirk)
    }

    /// Every legal cell with its data, in increasing (y, x).
    pub fn iter(&self) -> impl Iterator<Item = (CellXY, &Cell)> + '_ {
        let side = i32::from(self.side);
        self.cells.iter().enumerate().filter_map(move |(i, cell)| {
            let i = i32::try_from(i).ok()?;
            let c = CellXY::new(i % side, i / side);
            self.footprint.is_legal(c).then_some((c, cell))
        })
    }
}

/// The header block that places the preview's start markers (S-0009 claim
/// 25) and, beyond the eighth pair, marks start positions (claim 29).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Header {
    /// Marker count N_h (−1 when absent).
    pub count: i32,
    /// Marker origin X0, Y0.
    pub origin: (i32, i32),
    /// Marker extent Wh, Hh.
    pub extent: (i32, i32),
    /// Pairs 1 … N_h as read; `None` for a pair the map does not hold
    /// (pairs 9 … 16 then leave the marks in force). A component is `None`
    /// when it cannot be read: the pair that applies it completes it from
    /// the matching component of the mark in force at that phase (S-0009
    /// claim 29, our engine's departure). Pairs 1 … 8 come completed from
    /// the value in force, (0, 0).
    pub pairs: Vec<Option<[Option<i32>; 2]>>, // spec: S-0009 Import mapping ([Header] WaypointN: two integers)
}

impl Default for Header {
    fn default() -> Self {
        // spec: S-0009 claim 25 (an absent header reads as −1 for the count, origin and extent)
        Self {
            count: -1,
            origin: (-1, -1),
            extent: (-1, -1),
            pairs: Vec::new(),
        }
    }
}

/// The preview image (S-0009 claim 24).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Preview {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Red, green, blue per pixel, row by row from the top.
    pub rgb: Vec<u8>,
}

/// One lighting set of S-0104 (claim 1): ambient and tint in hundredths,
/// ground bias and level factor in thousandths.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LightingSet {
    /// Ambient, hundredths.
    pub ambient: i32,
    /// Tint (red, green, blue), hundredths.
    pub tint: [i32; 3], // spec: S-0104 Terms (tint: a triple)
    /// Ground bias, thousandths.
    pub ground: i32,
    /// Level factor, thousandths per height level.
    pub level: i32,
}

/// The map's lighting values (S-0104 claim 1 and Import mapping).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lighting {
    /// Normal conditions.
    pub normal: LightingSet,
    /// Ion storm.
    pub ion: LightingSet,
    /// Psychic dominator.
    pub dominator: LightingSet,
    /// Dominator fade interval, stored in thousandths and used as ticks.
    pub dominator_rate: i32,
    /// The same map value truncated to whole ticks (⌊v⌋), the rate used
    /// with `quirks.dominator_rate_thousandths` off.
    #[serde(default = "one_tick")]
    pub dominator_rate_whole: i32,
    /// Nuke fade interval in ticks.
    pub nuke_rate: i32,
}

impl Default for Lighting {
    fn default() -> Self {
        // spec: S-0104 Constants, Import mapping (engine defaults for absent keys)
        Self {
            normal: LightingSet {
                ambient: 100,
                tint: [100, 100, 100],
                ground: 50,
                level: 8,
            },
            ion: LightingSet {
                ambient: 87,
                tint: [30, 40, 75],
                ground: 0,
                level: 0,
            },
            dominator: LightingSet {
                ambient: 150,
                tint: [85, 20, 30],
                ground: 0,
                level: 0,
            },
            dominator_rate: 1,
            // spec: S-0104 Edge cases (Dominator fade rate scale: switch off, the absent-key default stays 1 tick)
            dominator_rate_whole: 1,
            nuke_rate: 1,
        }
    }
}

/// The absent-key fade interval, one tick (S-0104 Constants).
fn one_tick() -> i32 {
    1 // spec: S-0104 Constants (default dominator fade interval 1 tick)
}

/// The map-list and map-wide settings of S-0009 Import mapping that the
/// viewer shows.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Basic {
    /// Map list name, as stored bytes.
    pub name: Vec<u8>,
    /// Minimum players shown in the map list.
    pub min_players: i32,
    /// Maximum players shown in the map list.
    pub max_players: i32,
    /// Game modes the map is listed under, as stored.
    pub game_modes: Vec<u8>,
    /// Official-map flag.
    pub official: bool,
    /// Multiplayer-only switch.
    pub multiplayer_only: bool,
    /// `FreeRadar`: every house has radar (S-0410 claim 1 and Import
    /// mapping; absent: no).
    #[serde(default)]
    pub free_radar: bool,
    /// The map growth switch (S-0401 claim 4, `[Basic]
    /// TiberiumGrowthEnabled`); `None`: the key is absent, and the switch
    /// is on.
    #[serde(default)]
    pub tiberium_growth: Option<bool>,
    /// The inert flag (S-0303 claim 2 and Import mapping, `[SpecialFlags]
    /// Inert`; absent: no).
    #[serde(default)]
    pub inert: bool,
    /// Harvester immunity (S-0303 claim 9 and Import mapping,
    /// `[SpecialFlags] HarvesterImmune`; absent: no).
    #[serde(default)]
    pub harvester_immune: bool,
}

/// A defined entry of the start list: waypoint number and cell.
pub type StartEntry = (u16, (i16, i16));

/// A decoded map.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Map {
    /// Format version (S-0009 Import mapping, default 0).
    pub format_version: i32,
    /// Theater.
    pub theater: Theater,
    /// Base height level L0 of the initial fill (claim 6), already reduced
    /// into −128 … 127.
    pub base_level: i8,
    /// Initial fill setting.
    pub fill: Fill,
    /// Map size and local area.
    pub footprint: Footprint,
    /// The cells after the initial fill and the terrain stream, with the
    /// overlay streams' values.
    pub cells: CellGrid,
    /// Whether the map has an overlay data stream (claim 13).
    pub has_overlay_data: bool,
    /// Defined waypoint markers by number, each a cell reduced into
    /// −32768 … 32767 per axis (claim 15). Includes the home waypoint once
    /// claim 16 has defined it.
    pub waypoints: BTreeMap<u16, (i16, i16)>,
    /// Home waypoint number (claim 16).
    pub home_waypoint: u16,
    /// Header block.
    pub header: Header,
    /// Preview, if the map has one with a positive size.
    pub preview: Option<Preview>,
    /// Lighting values.
    pub lighting: Lighting,
    /// Map-list settings.
    pub basic: Basic,
    /// Objects the map places, as read (owners are matched against houses
    /// by the caller, claim 27).
    pub objects: Objects,
    /// Cell tags: cell to tag id, the first tag a cell gets (claim 31).
    pub cell_tags: BTreeMap<(i16, i16), Vec<u8>>,
    /// The map's triggers, tags, cell tags in entry order and trigger
    /// variables (S-0011 Import mapping; S-0009 claims 31 and 32).
    #[serde(default)]
    pub triggers: triggers::Triggers,
    /// Number of draws the initial fill takes from the scenario random
    /// stream (claim 6). The stream itself belongs to the planned random-stream spec; the
    /// fill's random variants are not applied until it exists.
    pub fill_draws: u32,
}

impl Map {
    /// The start list's defined part (S-0009 claim 17): the cells of the
    /// defined start waypoint markers among 0 … N − 1 for `players` players,
    /// in waypoint order, and N. Entries past the list's length are random
    /// start cells, which need the scenario random stream (a planned spec).
    pub fn start_list(&self, players: u16) -> (Vec<StartEntry>, u16) {
        let mut k = 0u16;
        // spec: S-0009 claim 17 (K counts start waypoint markers defined consecutively from 0, at most 8)
        while k < START_WAYPOINTS && self.waypoints.contains_key(&k) {
            k += 1;
        }
        let n = players.max(k);
        let list = (0..n)
            .filter_map(|i| self.waypoints.get(&i).map(|c| (i, *c)))
            .collect();
        (list, n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// spec: S-0009 claim 17 (the gap example)
    #[test]
    fn start_list_skips_gaps() {
        let mut map = Map {
            format_version: 4,
            theater: Theater::Temperate,
            base_level: 0,
            fill: Fill::Clear,
            footprint: Footprint::new(20, 20, [0, 0, 20, 20]),
            cells: CellGrid::new(Footprint::new(20, 20, [0, 0, 20, 20])),
            has_overlay_data: false,
            waypoints: BTreeMap::new(),
            home_waypoint: DEFAULT_HOME_WAYPOINT,
            header: Header::default(),
            preview: None,
            lighting: Lighting::default(),
            basic: Basic::default(),
            objects: Objects::default(),
            cell_tags: BTreeMap::new(),
            fill_draws: 0,
            triggers: Default::default(),
        };
        for (i, c) in [(0, (10, 10)), (1, (12, 10)), (3, (14, 10)), (5, (16, 10))] {
            map.waypoints.insert(i, c);
        }
        let (list, n) = map.start_list(4);
        assert_eq!(n, 4);
        assert_eq!(list, vec![(0, (10, 10)), (1, (12, 10)), (3, (14, 10))]);
    }

    /// spec: S-0104 Constants (engine defaults)
    #[test]
    fn lighting_defaults() {
        let l = Lighting::default();
        assert_eq!(l.normal.ambient, 100);
        assert_eq!(l.normal.ground, 50);
        assert_eq!(l.ion.tint, [30, 40, 75]);
    }
}
