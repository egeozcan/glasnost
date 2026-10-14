//! Loading a map into a new game (S-0009 claims 6, 12 to 14, 17 to 20, 27, 28,
//! 33 and 45 to 46; S-0010 claims 12 and 25; S-0202 claims 8 and 10 and its table
//! "Draws during map loading"; S-0009 claims 34 to 43 for the pre-placed
//! objects).
//!
//! The groups of draws run in the table's order. The groups that belong to
//! specs not implemented here (the tags of S-0011 claim 34, the start-event
//! draws of animations, S-0109 claim 13) take no draw and are recorded in the
//! world's notes when a map reaches them.

use std::sync::Arc;

use glasnost_map::terrain::{OverlayLand, RecalcRules, TerrainQuirks, TileCatalogue, recalc_pass};
use glasnost_map::{CellGrid, CellXY, Fill, Map, NO_OVERLAY, Theater};

use crate::cells::Cells;
use crate::passability;
use crate::rng::Rng;
use crate::rules::{Kind, SimRules};
use crate::update::UpdateOrder;
use crate::world::{GameSetup, House, HouseId, HouseRole, HouseSetup, World};
use crate::{crush, objects, overlays, starting, starts};

/// Why a game could not be set up.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadError {
    /// A house names a country the rules do not have.
    UnknownCountry(String),
    /// No random start cell could be found: 1000 candidates in a row failed
    /// (S-0009 claim 18, a departure from the original's endless load).
    NoStartCell,
    /// The skirmish setup is refused (S-0412).
    Setup(crate::econ::setup::SetupError),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownCountry(c) => write!(f, "unknown country {c:?}"),
            Self::Setup(e) => write!(f, "{e}"),
            Self::NoStartCell => write!(
                f,
                "no start cell could be found on the map: 1000 random candidates in a row had no clear 8 × 8 area (S-0009 claim 18)"
            ),
        }
    }
}

impl std::error::Error for LoadError {}

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

/// Runs a land recompute over cells in order (S-0211 claim 5), each cell
/// starting its tile animation, with that animation's creation draws, when
/// S-0010 claim 19 says (see [`tile_anim_due`]).
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
    for &c in order {
        let anim = tile_anim_due(w, &cat, c);
        recalc_pass(&mut w.cells.grid, &[c], &rr);
        if let Some(name) = anim {
            anim_creation_draws(w, &name);
        }
    }
}

/// Whether the recalculation of `c` that is about to run starts the cell's
/// tile animation (S-0010 claim 19): the cell is of the tile case (claim
/// 8: a usable tile and no row-1 overlay) as the recalculation begins, its
/// tile names an animation and its sub-tile is the tile's attaches-to
/// value. While the map loads, before the second pass, only a cell's first
/// recalculation can start one (S-0202 table); from the second pass on, a
/// cell starts at most one. Returns the animation type's name. Claim 19
/// leaves out of these once-only rules a type with the vein-animation flag
/// and leaves what happens then open (S-0010 Open questions); the sim
/// rules carry no such flag for animations, so our engine applies the
/// rules to every tile animation.
fn tile_anim_due(w: &mut World, cat: &TileCatalogue, c: CellXY) -> Option<String> {
    let i = w.cells.idx(c)?;
    let cell = *w.cells.cell(c)?;
    let first = match &mut w.cells.anim_first {
        Some(seen) => {
            // spec: S-0202 table (before the second pass a tile animation starts at the first recalculation of its cell)
            let was = seen[i];
            seen[i] = true;
            !was
        }
        // spec: S-0010 claim 19 (from the second pass until the next map load, at most one per cell)
        None => !w.cells.anim_started[i],
    };
    if !first {
        return None;
    }
    let tile = cat.tiles.get(usize::try_from(cell.tile).ok()?)?;
    let anim = tile.anim.as_ref()?;
    // spec: S-0010 claim 19 (the cell's sub-tile equals the tile's attaches-to value)
    if anim.attaches_to != i32::from(cell.sub_tile) {
        return None;
    }
    let row_one = cell.overlay != NO_OVERLAY
        && overlay_land(w.rules(), cell.overlay).is_some_and(|o| o.is_row_one());
    // spec: S-0010 claim 8 (only a tile-case cell starts a tile animation)
    if row_one || cat.tile_cell(cell.tile, cell.sub_tile).is_none() {
        return None;
    }
    if w.cells.anim_first.is_none() {
        w.cells.anim_started[i] = true;
    }
    Some(anim.name.clone())
}

/// The creation draws of an animation created with no loop delay (S-0202
/// claim 10, *Animation creation draws*): the stage-delay draw of S-0109
/// claim 3 when its range holds more than one value, then a bouncing type's
/// three raw values and three ranged draws over −65535 … 65535. A start
/// event's chain reaction on a cell holding no resource takes no draw
/// (S-0202 claim 10); one on a resource cell (S-0401 claim 20, the economy
/// topic's seam) and the middle event's particles and smudges (S-0109
/// claim 14 gives their gates, no approved spec their draws: request
/// Q-0182) are not taken. An animation type the rules do not list has no fields
/// (S-0004 claim 3) and takes no draw.
pub fn anim_creation_draws(w: &mut World, name: &str) {
    let rules = w.rules_arc();
    let Some(a) = rules.animation(name) else {
        return;
    };
    let q = rules.quirks;
    let (d1, d2) = a.random_delay;
    if d1 != d2 {
        // spec: S-0109 claim 3 (one ranged draw over [d₁′, d₂′] when d₁′ ≠ d₂′)
        let _ = w.rng.ranged(d1, d2, q.ranged_draw_half_span);
    }
    if a.bouncing {
        // spec: S-0202 claim 10 (a bouncing type: three raw values, then three ranged draws R(−65535, 65535))
        for _ in 0..3 {
            let _ = w.rng.raw(); // spec: S-0202 Constants (animation bounce draws: 3 raw values)
        }
        // spec: S-0202 Constants (then 3 ranged draws over −65535 … 65535)
        for _ in 0..3 {
            let _ = w.rng.ranged(-65535, 65535, q.ranged_draw_half_span);
        }
    }
    if a.start_effects {
        // spec: S-0202 claim 10 (a chain reaction on a cell without resource takes no draw and has no effect)
        w.note("S-0109 claim 13: an animation's start-event draws (a chain reaction on a resource cell, S-0401 claim 20; middle-event particles and smudges, request Q-0182) are not taken");
    }
}

/// Sets up a new game on `map` (S-0009, S-0202 claim 8): the early phase
/// of start positions, the initial fill, the overlay type stream and its
/// placement effects, the overlay data stream, the first terrain
/// recalculation pass, the vein rebuild, the map's objects, the second
/// pass, the unused-house cleanup, the late phase, the owners of the map's
/// walls, the starting units, the game-start pass (S-0010 claim 12) and the
/// bridge spans (S-0010 claim 25), with their draws in the order of
/// S-0202's table "Draws during map loading".
pub fn new_game(
    rules: Arc<SimRules>,
    map: &Map,
    catalogue: Arc<TileCatalogue>,
    setup: &GameSetup,
) -> Result<World, LoadError> {
    // spec: S-0412 claims 11 and 13 (setups our engine refuses)
    crate::econ::setup::check(setup, &rules).map_err(LoadError::Setup)?;
    let mut houses = Vec::new();
    for (i, h) in setup.houses.iter().enumerate() {
        let country = rules
            .country_names
            .iter()
            .position(|n| *n == h.country)
            .ok_or_else(|| LoadError::UnknownCountry(h.country.clone()))?
            as u32;
        // spec: S-0412 claims 2, 4 and 5 (credits, difficulty index and row when the houses are created)
        let (state, credits) = crate::econ::setup::house_values(setup, i, &rules);
        let row = crate::econ::setup::row_of(setup, i);
        houses.push(House {
            name: h.name.clone(),
            country,
            allies: Vec::new(),
            human: h.human,
            // spec: S-0412 claim 3 and S-0209 claim 11 (every house of our engine keeps skill level 0: human-controlled, AI, neutral and special houses alike; glossary *house skill level*)
            skill: 0,
            start: None,
            role: h.role,
            removed: false,
            credits,
            pending: Vec::new(),
            combat: crate::combat::fire::house_combat(
                &rules,
                country,
                h.role,
                setup.houses.len(),
                row,
            ),
            econ: crate::econ::HouseEcon {
                setup: state,
                ..Default::default()
            },
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
    // The overlay streams as read; overlays are created one by one below
    // (S-0009 claims 12 and 13).
    let mut grid = map.cells.clone();
    let mut stream = Vec::new();
    let legal: Vec<CellXY> = grid.iter().map(|(c, _)| c).collect();
    for c in legal {
        if let Some(cell) = grid.get_mut(c) {
            stream.push((c, cell.overlay, cell.overlay_data));
            cell.overlay = NO_OVERLAY;
            cell.overlay_data = 0;
        }
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
        cells: Cells {
            // spec: S-0202 table (before the second pass a tile animation starts at a cell's first recalculation)
            anim_first: Some(vec![false; grid.len()]),
            ..Cells::new(grid)
        },
        // spec: S-0201 claim 14 (orders blocked during the cycles of ticks 0 to 3)
        orders_blocked_through: 3,
        projectiles: Default::default(),
        scaled_spawn_delay: Default::default(),
        econ: crate::econ::WorldEcon {
            settings: setup.skirmish.clone(),
            ..Default::default()
        },
        effects: Vec::new(),
        notes: Vec::new(),
        theater: map.theater,
        rules: Some(rules.clone()),
        catalogue: Some(catalogue.clone()),
        regions: Default::default(),
        scratch: Default::default(),
        pathfinder: Default::default(),
    };
    // spec: S-0409 claim 3 (the states at game start, before any object is placed)
    crate::econ::shroud::start(&mut w);
    for h in &mut w.houses {
        // spec: S-0410 claim 1 (the map's free-radar flag, fixed for the game)
        h.econ.radar.free = map.basic.free_radar;
        // The first house update works claim 1 out for every house (S-0410 claim 1 holds from the start).
        h.econ.radar.due = true;
    }
    let q = rules.quirks;
    let order = map.cells.visiting_order(q.narrow_map_partial_pass);
    let players = starts::player_count(&w);
    let late_passive = |w: &World, h: usize| {
        rules
            .countries
            .get(w.houses[h].country as usize)
            .is_none_or(|c| c.passive)
    };

    // Early phase: lists 1 and 2 and the random-position draws (S-0009
    // claims 19, 20 and 33), on blank cells.
    if !setup.sandbox && q.double_start_resolution {
        // spec: S-0009 claim 29 (a header count above 8: pairs beyond the eighth act as start position marks)
        if map.header.count > 8 {
            // S-0009 claim 29 (header pairs beyond the eighth as start position marks) is not implemented.
            w.note("S-0009 claim 29: header pairs beyond the eighth are not applied as start position marks (not implemented)");
        }
        let early_passive = |w: &World, h: usize| {
            rules
                .countries
                .get(w.houses[h].country as usize)
                .is_none_or(|c| {
                    // spec: S-0009 claim 33 (the early phase sees the base rules' country data, quirks.early_phase_stale_country_data)
                    if q.early_phase_stale_country_data {
                        c.passive_base
                    } else {
                        c.passive
                    }
                })
        };
        // spec: S-0009 claim 20 (the early phase sees every legal cell as a blank cell)
        let real = std::mem::replace(&mut w.cells, Cells::new(CellGrid::new(map.footprint)));
        let list1 = starts::build_list(&mut w, map, players);
        let list2 = list1.and_then(|_| starts::build_list(&mut w, map, players));
        w.cells = real;
        let list2 = list2?;
        let n = list2.len();
        // spec: S-0009 claim 19 (players who chose a position get it; passive houses get none)
        let mut order = Vec::new();
        for (h, hs) in setup.houses.iter().enumerate() {
            if early_passive(&w, h) {
                continue;
            }
            w.houses[h].start = hs.start.filter(|&s| (s as usize) < n);
            order.push(h);
        }
        // spec: S-0009 claim 19 (the others in house order, against list 2)
        starts::assign(&mut w, &order, &list2);
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

    if !map.cell_tags.is_empty() {
        // S-0011 claim 34 (the draws of the tags that cell tags create) belongs to the triggers.
        w.note("S-0011 claim 34: tags are not created, so the cell tags' tag draws are not taken");
    }

    // The overlay type stream with its placement effects, then the data
    // stream (S-0009 claims 12 and 13; S-0202 claim 10).
    overlays::place_stream(&mut w, map, &stream);

    // First recalculation pass (S-0010 claim 12).
    recalc_cells(&mut w, &order);

    // The vein rebuild (S-0009 claim 14; S-0202 claim 10).
    overlays::rebuild_veins(&mut w, &order);

    // Terrain objects (no draws), then pre-placed objects kind by kind
    // (S-0009 claim 43; S-0203 claim 3; S-0202 table).
    objects::place_map_objects(&mut w, map, setup.sandbox);

    // Second recalculation pass (S-0010 claim 12): its tile animations
    // replace those started before it (S-0010 claim 19).
    w.cells.anim_first = None;
    recalc_cells(&mut w, &order);

    // Late phase: list 3 and the late assignments (S-0009 claims 20, 28 and 29).
    let mut bases: Vec<Option<CellXY>> = vec![None; w.houses.len()];
    if !setup.sandbox {
        // spec: S-0202 claim 14 and S-0009 claim 28 (the unused-house cleanup follows the second pass and precedes the late phase; its removals recalculate the occupied cells)
        cleanup(&mut w, setup);
        let list3 = starts::build_list(&mut w, map, players)?;
        let n = list3.len();
        let mut order = Vec::new();
        for (h, hs) in setup.houses.iter().enumerate() {
            // spec: S-0009 claim 28 (a removed house takes no position in the late phase)
            if late_passive(&w, h) || w.houses[h].removed {
                continue;
            }
            if !q.double_start_resolution {
                // spec: S-0009 Edge cases (quirks.double_start_resolution off: one list, after the terrain is loaded)
                w.houses[h].start = hs.start.filter(|&s| (s as usize) < n);
            }
            order.push(h);
        }
        // spec: S-0009 claims 28 and 29 (a non-passive house without a position is assigned against list 3, in house order)
        starts::assign(&mut w, &order, &list3);
        for (h, b) in bases.iter_mut().enumerate() {
            // spec: S-0009 claim 20 (the base cell is the position's entry in list 3)
            *b = w.houses[h]
                .start
                .and_then(|s| list3.get(s as usize).copied());
        }
    }

    // spec: S-0210 claim 14 (a) (the owners of the map's walls: once the buildings are placed, before any starting unit)
    crush::assign_wall_owners(&mut w);

    if setup.sandbox {
        // spec: S-0009 claim 20 and Constants (final draw after the starting units: one ranged draw over 0 … 65535 after the last house, its value unused)
        let _ = w.rng.ranged(0, 65535, q.ranged_draw_half_span);
    } else {
        // Starting units (S-0009 claims 45 and 46).
        starting::give(&mut w, &bases, setup);
    }

    if !setup.sandbox {
        // spec: S-0412 claim 14 (the rest of game start after the starting units, all before tick 0)
        // spec: S-0202 claim 14 (the crate placement and the secret-lab choices come before the game-start pass)
        // spec: S-0412 claim 14 (starting units, crates, secret labs, end-of-loading tags, then the game-start tile-animation draws) and S-0407 claim 14
        crate::econ::setup::start_game(&mut w);
    }

    // spec: S-0010 claim 12 (the game-start pass: every legal cell once more, in the visiting order, after the starting units and before the first tick; its tile animations by claim 19)
    // spec: S-0202 claim 14 (its tile-animation draws follow the starting units' final draw, the crate placement draws, the secret-lab choices and the end-of-loading tag draws, and precede the first tick; the tags are not implemented, S-0011)
    recalc_cells(&mut w, &order);

    // spec: S-0010 claim 25 (once at game start, before the first tick: the bridge spans, S-0212's bridge links, and the bridge-body marks of low spans)
    crate::bridges::find_spans(&mut w, &order);

    // Passability categories (S-0211 claim 11) after loading.
    let all: Vec<CellXY> = w.cells.grid.iter().map(|(c, _)| c).collect();
    for c in all {
        let cat = passability::compute_category(&w, c);
        if let Some(i) = w.cells.idx(c) {
            w.cells.category[i] = cat;
        }
    }
    crate::regions::rebuild(&mut w);
    // spec: S-0401 claims 4 and 13 (the map growth switch, and the game-start rebuilding of the lists, which takes no draw and sees only terrain objects)
    crate::econ::resource::game_start(&mut w, map.basic.tiberium_growth.unwrap_or(true));
    Ok(w)
}

/// The unused-house cleanup (S-0009 claim 28). It runs when the game has
/// no computer player, which in our engine means no AI house (S-0412 claim
/// 12), and fewer human slots than the rules' maximum player count. It
/// removes, with their pre-placed objects, the eligible houses (neither
/// human-controlled nor passive in the country data in force) whose
/// predecessor in house order it did not remove
/// (`quirks.cleanup_skips_next_house`; off, every eligible house). The
/// removed house's triggers are not removed: map triggers are not
/// implemented (S-0011).
pub fn cleanup(w: &mut World, setup: &GameSetup) {
    let rules = w.rules_arc();
    // spec: S-0009 claim 28 and S-0412 claim 12 (AI houses take the computer slots: one stops the cleanup)
    if setup.houses.iter().any(|h| h.ai) {
        return;
    }
    // spec: S-0009 claim 28 (human slots, observers included; our engine has no observers)
    let human_slots = setup
        .houses
        .iter()
        .filter(|h| h.role == HouseRole::Player && h.human)
        .count();
    // spec: S-0009 claim 28 (strictly fewer human slots than the maximum player count)
    if i64::try_from(human_slots).unwrap_or(i64::MAX) >= i64::from(rules.max_players) {
        return;
    }
    let mut prev_removed = false;
    for h in 0..w.houses.len() {
        let passive = rules
            .countries
            .get(w.houses[h].country as usize)
            .is_none_or(|c| c.passive);
        // spec: S-0009 claim 28 (eligible: neither human-controlled nor passive, by the country data after every rules layer)
        let eligible = !w.houses[h].human && !passive;
        // spec: S-0009 claim 28 (quirks.cleanup_skips_next_house: the house after a removed one is kept)
        let remove = eligible && !(prev_removed && rules.quirks.cleanup_skips_next_house);
        prev_removed = remove;
        if !remove {
            continue;
        }
        let owner = Some(HouseId(h as u8));
        let ids: Vec<_> = w
            .objects
            .values()
            .filter(|o| o.owner == owner)
            .map(|o| o.id)
            .collect();
        // spec: S-0009 claim 28 (its pre-placed objects go with it)
        for id in ids {
            objects::remove(w, id);
        }
        let house = &mut w.houses[h];
        house.removed = true;
        house.start = None;
    }
    if w.houses.iter().any(|h| h.removed) {
        w.note("S-0009 claim 28: a removed house's triggers are not removed (map triggers are not implemented, S-0011)");
    }
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

/// Our viewer's sandbox setup for a map: one house per owner name that the
/// map's object lists use, in order of first use (vehicles, infantry,
/// aircraft, structures), each with the country of the same name, or the
/// first country when no country has that name. This is a rule of our
/// viewer (no start positions, no players), not of the game;
/// [`GameSetup::sandbox`] is set. Every house is human-controlled except
/// the neutral and special houses (an owner named like the neutral or the
/// special country, which is that country's), which in the original are
/// not human-controlled (S-0009 claim 27; S-0208 Terms). Returns the setup
/// and the owner names whose country was substituted.
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
            // spec: S-0009 Import mapping (the neutral and special houses answer to their country names); S-0208 Terms (they are not human-controlled)
            let role = if name == NEUTRAL && country == NEUTRAL {
                HouseRole::Neutral
            } else if name == SPECIAL && country == SPECIAL {
                HouseRole::Special
            } else {
                HouseRole::Player
            };
            HouseSetup {
                name,
                country,
                human: role == HouseRole::Player,
                start: None,
                role,
                ai: false,
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
            unit_count: 0,
            bases: false,
            skirmish: Default::default(),
        },
        substituted,
    )
}

/// The neutral house's name, which is its country's (S-0009 Import mapping:
/// a fixed name of the original's program).
pub const NEUTRAL: &str = "Neutral"; // spec: S-0009 Import mapping (house names matched by pre-placed objects: Neutral)

/// The special house's name, which is its country's (S-0009 Import mapping).
pub const SPECIAL: &str = "Special"; // spec: S-0009 Import mapping (house names matched by pre-placed objects: Special)
