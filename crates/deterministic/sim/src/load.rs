//! Loading a map into a new game (S-0009 claims 6, 12 to 14, 17 to 20, 27;
//! S-0010 claim 12; S-0202 claims 8 and 10 and its table "Draws during map
//! loading"; S-0009 claims 34 to 43 for the pre-placed objects).
//!
//! The groups of draws run in the table's order. Groups this module does
//! not implement yet (the bridge pieces and vein patterns of S-0202 claim 10
//! and S-0009 claim 14, animation creation draws, the starting units of
//! S-0009 claims 45 and 46 and the random start cells of claim 18) take no
//! draw here and are recorded in the world's notes when a map reaches them.

use std::sync::Arc;

use glasnost_map::terrain::{OverlayLand, RecalcRules, TerrainQuirks, TileCatalogue, recalc_pass};
use glasnost_map::{CellXY, Fill, Map, NO_OVERLAY, Theater};
use glasnost_num::geom::root_floor;

use crate::cells::Cells;
use crate::objects;
use crate::passability;
use crate::rng::Rng;
use crate::rules::{Kind, SimRules};
use crate::update::UpdateOrder;
use crate::world::{GameSetup, House, HouseId, HouseSetup, World};

/// Why a game could not be set up.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadError {
    /// A house names a country the rules do not have.
    UnknownCountry(String),
    /// The start list needs a random start cell (S-0009 claim 18), whose
    /// search, a use of S-0212 claim 25's nearby-cell search, is not
    /// implemented yet.
    RandomStartCell,
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownCountry(c) => write!(f, "unknown country {c:?}"),
            Self::RandomStartCell => write!(
                f,
                "the map needs a random start cell (S-0009 claim 18); its search is not implemented yet"
            ),
        }
    }
}

impl std::error::Error for LoadError {}

/// The overlay indices whose placement completes low bridges (S-0202 claim
/// 10).
// spec: S-0202 Constants (bridge end-marker positions 122 … 125 and 233 … 236)
const BRIDGE_MARKERS: [u8; 8] = [122, 123, 124, 125, 233, 234, 235, 236];

/// The vein overlay's position in the overlay list (S-0202 claim 10,
/// S-0009 claim 14).
const VEIN: u8 = 126; // spec: S-0202 Constants (vein overlay position 126)

/// The terrain-recalculation switches from the quirks.
pub fn terrain_quirks(r: &SimRules) -> TerrainQuirks {
    TerrainQuirks {
        cliff_back_case_sets: r.quirks.cliff_back_case_sets,
        unset_role_range: r.quirks.unset_role_range,
        ramp_shape_zero: r.quirks.ramp_shape_zero,
    }
}

/// The overlay land properties of an overlay list index.
pub fn overlay_land(r: &SimRules, i: u8) -> Option<OverlayLand> {
    r.overlays.get(usize::from(i)).map(|o| OverlayLand {
        land: o.land,
        own_land: o.own_land,
        tiberium: o.tiberium,
    })
}

/// Runs a land recompute over cells in order (S-0211 claim 5).
pub fn recalc_cells(w: &mut World, order: &[CellXY]) {
    let rules = w.rules_arc();
    let cat = w.catalogue_arc();
    let ov = |i: u8| overlay_land(&rules, i);
    let rr = RecalcRules {
        catalogue: &cat,
        overlay: &ov,
        cliff_back: rules.general.cliff_back,
        quirks: terrain_quirks(&rules),
    };
    recalc_pass(&mut w.cells.grid, order, &rr);
}

/// Sets up a new game on `map` (S-0009, S-0202 claim 8).
pub fn new_game(
    rules: Arc<SimRules>,
    map: &Map,
    catalogue: Arc<TileCatalogue>,
    setup: &GameSetup,
) -> Result<World, LoadError> {
    let mut houses = Vec::new();
    for h in &setup.houses {
        let country = rules
            .country_names
            .iter()
            .position(|n| *n == h.country)
            .ok_or_else(|| LoadError::UnknownCountry(h.country.clone()))?
            as u32;
        houses.push(House {
            name: h.name.clone(),
            country,
            allies: Vec::new(),
            human: h.human,
            skill: 0,
            start: h.start,
            pending: Vec::new(),
        });
    }
    for (i, h) in houses.iter_mut().enumerate() {
        // spec: S-0209 Terms (a house is allied with itself)
        h.allies.push(HouseId(i as u8));
    }
    for &(a, b) in &setup.alliances {
        for (x, y) in [(a, b), (b, a)] {
            if let Some(h) = houses.get_mut(usize::from(x)) {
                h.allies.push(HouseId(y));
            }
        }
    }
    for h in &mut houses {
        h.allies.sort();
        h.allies.dedup();
    }
    let mut w = World {
        tick: 0,
        // spec: S-0202 claim 8 (the scenario stream is seeded with the game's seed before the map is read)
        rng: Rng::new(setup.seed),
        game_speed: setup.game_speed,
        houses,
        objects: Default::default(),
        next_id: 1,
        update: UpdateOrder::default(),
        cells: Cells::new(map.cells.clone()),
        // spec: S-0201 claim 14 (orders blocked during the cycles of ticks 0 to 3)
        orders_blocked_through: 3,
        notes: Vec::new(),
        theater: map.theater,
        rules: Some(rules.clone()),
        catalogue: Some(catalogue.clone()),
        regions: Default::default(),
        scratch: Default::default(),
        pathfinder: Default::default(),
    };
    let q = rules.quirks;
    let order = map.cells.visiting_order(q.narrow_map_partial_pass);

    // Early phase: lists 1 and 2 and the random-position draws (S-0009 claim 20).
    let players: Vec<usize> = (0..w.houses.len())
        .filter(|&i| !setup.sandbox && w.houses[i].human)
        .collect();
    if !setup.sandbox {
        let list = start_list(map, players.len())?;
        assign_positions(&mut w, &players, &list);
    }

    // Initial fill (S-0009 claim 6): one draw per visited cell for a water fill.
    let (clear_base, water_base) = setup_bases(&rules, &catalogue, setup);
    for &c in &order {
        let tile = match map.fill {
            Fill::Clear => clear_base,
            // spec: S-0009 claim 6 (water base tile plus a ranged draw over 0 … 3, one per visited cell)
            Fill::Water => water_base + w.rng.ranged(0, 3, q.ranged_draw_half_span),
        };
        if let Some(cell) = w.cells.grid.get_mut(c)
            && !cell.stream
        {
            cell.tile = tile;
            cell.sub_tile = 0;
            cell.level = map.base_level;
        }
    }

    // Overlay type stream (S-0009 claims 11 to 13): creation rules and
    // placement recalculations, in stream order.
    let mut stream_cells: Vec<CellXY> = w.cells.grid.iter().map(|(c, _)| c).collect();
    stream_cells.sort_by_key(|c| (c.y, c.x));
    let mut placed = Vec::new();
    let mut bridge_note = false;
    for c in stream_cells {
        let Some(cell) = w.cells.grid.get(c).copied() else {
            continue;
        };
        if cell.overlay == NO_OVERLAY {
            continue;
        }
        // spec: S-0009 claim 11 (no overlay for format version below 2)
        let keep = map.format_version >= 2
            && rules
                .overlays
                .get(usize::from(cell.overlay))
                .is_some_and(|o| {
                    // spec: S-0009 claim 12 (artwork or cell animation; crates only in campaign games)
                    o.has_art && !o.crate_
                });
        if !keep {
            if let Some(cell) = w.cells.grid.get_mut(c) {
                cell.overlay = NO_OVERLAY;
            }
            continue;
        }
        bridge_note |= BRIDGE_MARKERS.contains(&cell.overlay);
        placed.push(c);
        // spec: S-0202 claim 10 (placing an overlay recalculates its cell)
        recalc_cells(&mut w, &[c]);
    }
    let _ = placed;
    if bridge_note {
        w.note("S-0202 claim 10: low-bridge end markers complete no bridge and take no draw (not implemented yet)");
    }

    // First recalculation pass (S-0010 claim 12).
    recalc_cells(&mut w, &order);

    // Veins are cleared; the vein rebuild of S-0009 claim 14 (S-0202 claim
    // 10's placement test and pattern draws) is not implemented yet.
    let mut veins = false;
    for &c in &order {
        if let Some(cell) = w.cells.grid.get_mut(c)
            && cell.overlay == VEIN
        {
            // spec: S-0009 claim 14 (every vein cell loses its overlay and its data value 0)
            cell.overlay = NO_OVERLAY;
            cell.overlay_data = 0;
            veins = true;
        }
    }
    if veins {
        w.note("S-0009 claim 14: vein seeds are cleared and not rebuilt (no draw; not implemented yet)");
    }

    // Terrain objects (no draws), then pre-placed objects kind by kind
    // (S-0009 claim 43; S-0203 claim 3; S-0202 table).
    objects::place_map_objects(&mut w, map, setup.sandbox);

    // Second recalculation pass (S-0010 claim 12).
    recalc_cells(&mut w, &order);

    // Late phase: list 3 (S-0009 claim 20).
    if !setup.sandbox && !players.is_empty() {
        let _list3 = start_list(map, players.len())?;
        w.note("S-0009 claims 45 and 46: starting units are not created (the rules fields they need are not imported yet)");
    }
    // spec: S-0009 claim 20 (after the last house one ranged draw over 0 … 65535 whose value has no effect)
    let _ = w.rng.ranged(0, 65535, q.ranged_draw_half_span);

    // Passability categories (S-0211 claim 11) after loading.
    let all: Vec<CellXY> = w.cells.grid.iter().map(|(c, _)| c).collect();
    for c in all {
        let cat = passability::compute_category(&w, c);
        if let Some(i) = w.cells.idx(c) {
            w.cells.category[i] = cat;
        }
    }
    crate::regions::rebuild(&mut w);
    Ok(w)
}

/// The fill's base tiles (S-0009 claim 6): with `quirks.stale_fill_tiles`
/// those of the theater loaded before (the setup's choice; 0 for both in a
/// new session), otherwise this theater's clear and water role starts.
fn setup_bases(r: &SimRules, cat: &TileCatalogue, setup: &GameSetup) -> (i32, i32) {
    let _ = setup;
    if r.quirks.stale_fill_tiles {
        // spec: S-0009 claim 6 (tile 0 for both when no theater has been loaded since the program started)
        (0, 0)
    } else {
        (cat.roles.clear, cat.roles.water)
    }
}

/// The start list of S-0009 claim 17 for P players.
fn start_list(map: &Map, players: usize) -> Result<Vec<CellXY>, LoadError> {
    // spec: S-0009 claim 17 (K: start waypoints defined consecutively from 0, at most 8)
    let mut k: usize = 0;
    // spec: S-0009 claim 17 (start waypoints 0 … 7, at most 8)
    while k < 8 && map.waypoints.contains_key(&(k as u16)) {
        k += 1;
    }
    let n = players.max(k);
    let mut list: Vec<CellXY> = (0..n as u16)
        .filter_map(|i| map.waypoints.get(&i))
        .map(|&(x, y)| CellXY::new(i32::from(x), i32::from(y)))
        .collect();
    if list.len() < n {
        // spec: S-0009 claim 18 (random start cells: their search is not implemented yet)
        return Err(LoadError::RandomStartCell);
    }
    list.truncate(n);
    Ok(list)
}

/// Assigns start positions (S-0009 claim 19).
fn assign_positions(w: &mut World, players: &[usize], list: &[CellXY]) {
    let n = list.len() as i32;
    let q = w.rules().quirks;
    let mut taken: Vec<u32> = players.iter().filter_map(|&p| w.houses[p].start).collect();
    for &p in players {
        if w.houses[p].start.is_some() {
            continue;
        }
        let pos = if taken.is_empty() {
            // spec: S-0009 claim 19 (no position taken: one ranged draw over 0 … N − 1)
            w.rng.ranged(0, n - 1, q.ranged_draw_half_span) as u32
        } else {
            // spec: S-0009 claim 19 (the free position whose distance sum is largest; ties to the lowest)
            let mut best: Option<(u64, u32)> = None;
            for i in 0..n as u32 {
                if taken.contains(&i) {
                    continue;
                }
                let sum: u64 = taken
                    .iter()
                    .map(|&t| {
                        let (a, b) = (list[i as usize], list[t as usize]);
                        let dx = i64::from(a.x - b.x);
                        let dy = i64::from(a.y - b.y);
                        root_floor(
                            (dx * dx + dy * dy) as u64,
                            q.table_root_distance,
                            w.rules().mode,
                        )
                    })
                    .sum();
                if best.is_none_or(|(s, _)| sum > s) {
                    best = Some((sum, i));
                }
            }
            best.map_or(0, |b| b.1)
        };
        w.houses[p].start = Some(pos);
        taken.push(pos);
    }
}

impl World {
    /// The theater's tile catalogue.
    pub fn catalogue_arc(&self) -> Arc<TileCatalogue> {
        self.catalogue.clone().expect("world has a catalogue")
    }

    /// Whether the map's theater is the snow theater (S-0211 claim 11).
    pub fn theater_snow(&self) -> bool {
        self.theater == Theater::Snow
    }

    /// A tile's size in cells, if the tile exists.
    pub fn tile_size(&self, tile: i32) -> Option<(u8, u8)> {
        let cat = self.catalogue.as_deref()?;
        let t = cat.tiles.get(usize::try_from(tile).ok()?)?;
        Some((t.width, t.height))
    }
}

/// The kind of a pre-placed object list.
pub fn kind_name(k: Kind) -> &'static str {
    match k {
        Kind::Infantry => "infantry",
        Kind::Vehicle => "vehicle",
        Kind::Aircraft => "aircraft",
        Kind::Building => "building",
        Kind::Terrain => "terrain object",
    }
}

/// Our viewer's sandbox setup for a map: one human-controlled house per
/// owner name that the map's object lists use, in order of first use
/// (vehicles, infantry, aircraft, structures), each with the country of the
/// same name, or the first country when no country has that name. This is
/// a rule of our viewer (no start positions, no players), not of the game;
/// [`GameSetup::sandbox`] is set. Returns the setup and the owner names
/// whose country was substituted.
pub fn sandbox_setup(map: &Map, rules: &SimRules, seed: u32) -> (GameSetup, Vec<String>) {
    let o = &map.objects;
    let owners = o
        .units
        .iter()
        .map(|x| &x.placed)
        .chain(o.infantry.iter().map(|x| &x.placed))
        .chain(o.aircraft.iter().map(|x| &x.placed))
        .chain(o.structures.iter().map(|x| &x.placed))
        .map(|p| p.owner.clone());
    let mut names: Vec<String> = Vec::new();
    for owner in owners {
        if let Ok(n) = String::from_utf8(owner)
            && !names.contains(&n)
        {
            names.push(n);
        }
    }
    let mut substituted = Vec::new();
    let houses = names
        .into_iter()
        .map(|name| {
            let country = if rules.country_names.contains(&name) {
                name.clone()
            } else {
                substituted.push(name.clone());
                rules.country_names.first().cloned().unwrap_or_default()
            };
            HouseSetup {
                name,
                country,
                human: true,
                start: None,
            }
        })
        .collect();
    (
        GameSetup {
            seed,
            // spec: S-0201 claim 7 (a skirmish starts at the lobby's index; the viewer takes the rules' lobby default)
            game_speed: rules.general.lobby_game_speed,
            houses,
            alliances: Vec::new(),
            sandbox: true,
        },
        substituted,
    )
}
