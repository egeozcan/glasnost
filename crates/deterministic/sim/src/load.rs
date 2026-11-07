//! Loading a map into a new game (S-0009 claims 6, 12 to 14, 17 to 20, 27, 28,
//! 33 and 45 to 46; S-0010 claims 12 and 25; S-0202 claims 8 and 10 and its table
//! "Draws during map loading"; S-0009 claims 34 to 43 for the pre-placed
//! objects).
//!
//! The groups of draws run in the table's order. The groups that belong to
//! specs not implemented here (the tags of S-0011 claim 34, the smudge
//! selection of S-0312 claim 26 and the resource chain reaction of S-0401
//! claim 20 in an animation's start event) take no draw and are recorded in
//! the world's notes when a map reaches them.

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
        if let Some(a) = anim {
            let level = w.cells.cell(c).map_or(0, |x| i32::from(x.level));
            let at = tile_anim_position(c, level, a.dx, a.dy);
            let role = if w.cells.anim_first.is_some() {
                // spec: S-0010 claim 19 (a tile animation started before the second pass does not survive loading)
                crate::anims::Role::FirstPassTile
            } else {
                crate::anims::Role::Ordinary
            };
            let spawn = crate::anims::Spawn {
                role,
                ..Default::default()
            };
            let pos = crate::world::Pos::new(at.0, at.1, at.2);
            crate::anims::spawn(w, &a.name, Some(pos), spawn);
        }
    }
}

/// The ground displacement of S-0101 F6 for one sum *u*:
/// trunc₀(4473959 · *u* / 1048576), with a 64-bit product.
fn f6_term(u: i32) -> i32 {
    // spec: S-0101 F6 (a = 4473959 / 1048576; trunc₀ of the exact product, a 64-bit product)
    let v = i64::from(u) * 4_473_959 / 1_048_576;
    i32::try_from(v).unwrap_or(if v < 0 { i32::MIN } else { i32::MAX })
}

/// The world position of a tile animation of a cell at `level` whose tile
/// has the displacement (`dx`, `dy`) in pixels (S-0010 Formulas, *Tile
/// animation position*; S-0101 F6). F6's exact form is its value with
/// `quirks.chop_rounding` on; claim 19's view-size case is a departure
/// that our engine does not reproduce.
pub fn tile_anim_position(c: CellXY, level: i32, dx: i32, dy: i32) -> (i32, i32, i32) {
    // spec: S-0010 Formulas (X = 256x + 128 + trunc₀(a(dx + 2dy)), Y = 256y + 128 + trunc₀(a(2dy − dx)), z = 104ℓ)
    let x = 256 * c.x + 128 + f6_term(dx.wrapping_add(dy.wrapping_mul(2)));
    let y = 256 * c.y + 128 + f6_term(dy.wrapping_mul(2).wrapping_sub(dx)); // spec: S-0010 Formulas (Y)
    (x, y, 104 * level) // spec: S-0010 Formulas (z = 104ℓ)
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
fn tile_anim_due(
    w: &mut World,
    cat: &TileCatalogue,
    c: CellXY,
) -> Option<glasnost_map::terrain::TileAnim> {
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
    Some(anim.clone())
}

/// The creation draws of S-0202 claim 10 before the start event: the stage
/// delay the ranged draw gave, when one was taken, and a bouncing type's
/// three raw values in the order taken (S-0319 claims 2 and 3 form its
/// first velocity from them).
pub(crate) fn creation_draws(
    w: &mut World,
    a: &crate::rules::AnimRules,
) -> (Option<i32>, Option<crate::bounce::Raws>) {
    let q = w.rules().quirks;
    let (d1, d2) = a.random_delay;
    let mut delay = None;
    if d1 != d2 {
        // spec: S-0109 claim 3 (one ranged draw over [d₁′, d₂′] when d₁′ ≠ d₂′)
        delay = Some(w.rng.ranged(d1, d2, q.ranged_draw_half_span));
    }
    if a.bouncing {
        // spec: S-0202 claim 10 (a bouncing type of a sprite animation of S-0109: three raw values, then three ranged draws R(−65535, 65535); revision 36: not a voxel-model debris piece, whose draws are S-0317's)
        // spec: S-0202 Constants (animation bounce draws: 3 raw values) and S-0319 claims 2 and 3 (r₁, r₂, r₃, or q₁, q₂, q₃, in the order taken)
        let raws = [w.rng.raw(), w.rng.raw(), w.rng.raw()];
        // spec: S-0202 Constants (then 3 ranged draws over −65535 … 65535) and S-0319 claims 2 and 3 (their values have no effect)
        for _ in 0..3 {
            let _ = w.rng.ranged(-65535, 65535, q.ranged_draw_half_span);
        }
        return (delay, Some(raws));
    }
    (delay, None)
}

/// The creation draws of an animation created with no loop delay at the
/// world position `at`, during loading or during play alike (S-0202 claims
/// 10 and 15: a weapon, teleport, death, debris, trigger or terrain creator
/// does not change the rule; *Animation creation draws*; `None` for a
/// resource glint, S-0202 claim 14, whose position no spec gives, so its
/// middle event takes its particle draws and notes the scorch-or-crater
/// part): the
/// stage-delay draw of S-0109 claim 3 when its range holds more than one
/// value, then a bouncing type's three raw values and three ranged draws
/// over −65535 … 65535, then the start event (S-0109 claim 13): the
/// immediate middle event when μ = 0 (S-0109 claim 14, S-0202 claim 15,
/// [`middle_event`]), then the resource chain reaction. A chain reaction on
/// a cell holding no resource takes no draw (S-0202 claim 10); one on a
/// resource cell is S-0401 claim 20's, the economy topic's seam, and is not
/// taken. An animation type the rules do not list has no fields (S-0004
/// claim 3) and takes no draw.
///
/// Returns the stage delay the creation's ranged draw gave, when one was
/// taken, for the drawing's playback (presentation only, never read by the
/// simulation).
pub fn anim_created_at(w: &mut World, name: &str, at: Option<(i32, i32, i32)>) -> Option<i32> {
    anim_created(w, name, at, false)
}

/// [`anim_created_at`], with `played` set when the caller models the
/// animation's later playback itself or removes it at once (the virus-death
/// animation of S-0613 claims 2 and 3, a storm cloud of S-0513 claim 46):
/// only its creation draws and start event are taken here. Otherwise the
/// animation is an ordinary one in play (`crate::anims`), appended to the
/// update order (S-0109 claim 29).
pub(crate) fn anim_created(
    w: &mut World,
    name: &str,
    at: Option<(i32, i32, i32)>,
    played: bool,
) -> Option<i32> {
    if !played {
        let pos = at.map(|(x, y, z)| crate::world::Pos::new(x, y, z));
        return crate::anims::spawn(w, name, pos, crate::anims::Spawn::default()).delay;
    }
    let rules = w.rules_arc();
    let a = rules.animation(name)?;
    // A caller that models the playback itself (a storm cloud, a virus death) does not move the animation: S-0319 moves the ordinary animations in play.
    let (delay, _) = creation_draws(w, a);
    start_event(w, a, at, played);
    delay
}

/// The start event of an animation of type `a` at `at` (S-0109 claim 13):
/// the immediate middle event when μ = 0, then the resource chain
/// reaction. It comes at creation, and at a follow-up switch after the new
/// type's delay choice (S-0109 claim 28). `played`: the caller models the
/// later playback, so a later middle event is not noted as missing.
pub(crate) fn start_event(
    w: &mut World,
    a: &crate::rules::AnimRules,
    at: Option<(i32, i32, i32)>,
    played: bool,
) {
    let effects = a.particles.is_some_and(|(_, n)| n > 0) || a.scorch || a.crater;
    match a.mu {
        // spec: S-0109 claim 13 (the start event produces the middle event at once when μ = 0)
        Some(0) => middle_event(w, a, at),
        Some(_) if played => {}
        // spec: S-0202 claim 15 and S-0109 claim 14 (with μ > 0 the same composition comes at the middle event later in playback)
        Some(_) if effects => w.note("S-0109 claim 14: an animation's middle event later in playback (μ > 0) is not modelled (animations have no playback clock in the simulation), so its particle, scorch-or-crater and crater draws are not taken"),
        None if effects => w.note("S-0109 claim 25: an animation type's μ is unknown (the import found no art for it), so its start event's middle-event draws are not taken"),
        _ => {}
    }
    // spec: S-0109 claim 13 (the middle event's effects and draws precede the chain reaction)
    if a.chain_reaction {
        let resource = at.is_none_or(|(x, y, _)| {
            // spec: S-0202 claim 10 (a chain reaction on a cell without resource takes no draw and has no effect)
            crate::econ::resource::resource_at(w, cell_of(x, y)).is_some()
        });
        if resource {
            w.note("S-0401 claim 20: an animation's start-event chain reaction on a resource cell (the economy topic's seam) takes no draw and has no effect");
        }
    }
}

/// The cell containing a ground position (S-0204 claim 2).
fn cell_of(x: i32, y: i32) -> CellXY {
    // spec: S-0204 claim 2 (the containing cell)
    CellXY::new(x / 256, y / 256)
}

/// The height limit of the middle event's scorch marks and craters, in
/// leptons (S-0109 claim 14).
const SMUDGE_HEIGHT_LIMIT: i32 = 30; // spec: S-0109 claim 14 (attempted only below 30 leptons above the terrain)

/// The resource levels a crater request removes (S-0109 claim 14, S-0401
/// claim 16 (c)).
const CRATER_BALES: i32 = 6; // spec: S-0401 Constants (bales a crater takes: 6)

/// The middle event of an animation at `at` (S-0109 claim 14; S-0202 claim
/// 15 for its composition at creation): the particle groups, then, when the
/// height test passes, the scorch-or-crater choice, a crater's removal of 6
/// resource levels from the containing cell (with any recalculation it
/// causes), then the smudge selection of S-0312 claim 26, which is not
/// modelled: smudge types and the smudges of cells are not carried, so no
/// type fits and it takes no draw, as for destructions.
pub(crate) fn middle_event(
    w: &mut World,
    a: &crate::rules::AnimRules,
    at: Option<(i32, i32, i32)>,
) {
    let q = w.rules().quirks;
    if let Some((p, n)) = a.particles {
        // spec: S-0109 claim 14 (particles requested when a type is given and the count is positive; not gated by height; created in the shared gas system of S-0613 claim 1)
        for _ in 0..n.max(0) {
            crate::gas::middle_event_particle(w, p, at);
        }
    }
    if !a.scorch && !a.crater {
        return;
    }
    let Some((x, y, z)) = at else {
        w.note("S-0109 claim 14: the middle event of an animation whose position is not carried takes no scorch-or-crater choice and removes no resource");
        return;
    };
    // spec: S-0109 claim 14 (attempted only when Z − H(X, Y) < 30; no lower bound)
    if z - w.surface_at(x, y) >= SMUDGE_HEIGHT_LIMIT {
        return;
    }
    let crater = if a.scorch && a.crater {
        // spec: S-0109 claim 14 (both flags: one ranged draw over [0, 2147483646] with S-0202 claim 4's range)
        let r = w.rng.ranged(0, 2147483646, q.ranged_draw_half_span);
        // spec: S-0109 claim 14 (r times the nearest binary64 to 1 / 2147483646 compared with 0.5 in the selected rounding; with quirks.chop_rounding on, a scorch mark exactly when r < 1073741824)
        let mode = q.rounding();
        // spec: S-0109 claim 14 (compared with 0.5)
        let half =
            glasnost_num::F64::from_i64(1, mode).div(glasnost_num::F64::from_i64(2, mode), mode);
        !crate::rng::unit_value(r, mode).lt(half)
    } else {
        a.crater
    };
    if crater {
        // spec: S-0109 claim 14 and S-0401 claim 16 (c) (a crater request takes 6 bales from the containing cell before placement, by claim 3's rule)
        crate::econ::resource::take_bales(w, cell_of(x, y), CRATER_BALES);
    }
    // spec: S-0202 claim 15 and S-0312 claim 26 (the smudge-type draw comes last: R(0, n − 1), no draw for n ≤ 1)
    w.note("S-0312 claim 26: smudge types, the smudges of cells and the morphable flag of tilesets are not modelled, so no smudge type fits a middle event's scorch mark or crater: n = 0, no draw and no smudge");
}

/// The resource-glint setup of S-0202 claim 14. With a glint animation
/// type configured, each cell in `order` whose contained resource value
/// (value per bale times bales, S-0401 claim 2, a wrapping signed 32-bit
/// product) is nonzero takes the choice R(0, f − 1); a result of 0 creates
/// the glint with no loop delay, its creation draws taken before the next
/// cell's choice. The glint is cosmetic in our engine: only its draws are
/// kept.
pub fn resource_glints(w: &mut World, order: &[CellXY]) {
    let rules = w.rules_arc();
    // spec: S-0202 claim 14 (with no configured animation the group takes no draw)
    let Some(glint) = rules.ore_twinkle.clone() else {
        return;
    };
    let q = rules.quirks;
    // spec: S-0202 claim 14 (f − 1 wraps in signed 32-bit arithmetic)
    let top = rules.general.ore_twinkle_chance.wrapping_sub(1);
    for &c in order {
        let Some((t, d)) = crate::econ::resource::resource_at(w, c) else {
            continue;
        };
        let v = rules.econ.resource.tiberiums[t].value;
        // spec: S-0202 claim 14 (contained value: value per bale times the bale count d + 1, S-0401 claim 2, a wrapping signed 32-bit product; zero-value cells take no draw)
        if v.wrapping_mul(i32::from(d) + 1) == 0 {
            continue;
        }
        // spec: S-0202 claim 14 (the choice R(0, f − 1); claims 4 and 5 for its bounds)
        if w.rng.ranged(0, top, q.ranged_draw_half_span) == 0 {
            // spec: S-0202 claim 14 (a zero result creates the glint at once, with claim 10's creation draws)
            // spec: S-0202 claim 14 (the glint takes all applicable creation and start-event draws of claim 10)
            anim_created_at(w, &glint, None);
        }
    }
}

/// One round of house creation (S-0202 claim 14): one ranged draw
/// R(450, 1800) per house of the game, in house order, its result unused.
fn house_creation_round(w: &mut World, half_span: bool) {
    for _ in 0..w.houses.len() {
        // spec: S-0202 claim 14 (the bounds 450 and 1800, the same for every house, raw values by claim 4's ranged-draw rule; no rule uses the result)
        let _ = w.rng.ranged(450, 1800, half_span);
    }
}

/// Sets up a new game on `map` (S-0009, S-0202 claim 8): the first round of
/// house-creation draws (S-0202 claim 14), the early phase of start
/// positions, the second round of house-creation draws, the initial fill,
/// the overlay type stream and its
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
            // spec: S-0215 claim 38 (a house begins with no recorded base centre or base-start cell and a radius of 0)
            base: Default::default(),
            computed_edge: 0, // spec: S-0522 claim 5 (the computed edge starts as north, 0)
            lost_to_capture: false, // spec: S-0509 claim 11 (the mark is clear at the start of a game)
            infiltrated_vehicle_factory: false, // spec: S-0510 Terms (both flags are clear at the start of a game)
            infiltrated_infantry_factory: false, // spec: S-0510 Terms (both flags are clear at the start of a game)
            idle_test_house: false, // spec: S-0216 claim 8 (Removal before the groups: only the arena's human player house is idle)
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
        // spec: S-0202 claim 13 (seeded with 0 at session start, independently of the game seed; an ordinary-map game neither reseeds it nor draws R(0, 65535) for it). Seam: our engine starts each game as a fresh session, so the state an earlier game of the same session left is not carried over.
        map_rng: Some(Rng::new(0)),
        game_speed: setup.game_speed,
        houses,
        objects: Default::default(),
        // Map objects start at object number 1.
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
        flying: Default::default(),
        jump_slots: Default::default(),
        threat: Default::default(),
        leftovers: Default::default(),
        cloak: Default::default(),
        sw: Default::default(),
        radiation: Default::default(),
        gas: Default::default(),
        waves: Default::default(),
        pieces: Default::default(),
        discs: Vec::new(),
        walkers: Default::default(),
        mutation: Default::default(),
        drain: Default::default(),
        routes: Default::default(),
        domination: Default::default(),
        depot_anims: Default::default(),
        anims: Default::default(),
        // spec: S-0104 claim 14 and Import mapping (the lighting sets are read once with the map)
        lighting: crate::lighting::start(&map.lighting),
        temporal: Default::default(),
        mind: Default::default(),
        acquisitions: 0,
        strike: Default::default(),
        crates: Default::default(),
        spawned: Default::default(),
        triggers: Default::default(),
        econ: crate::econ::WorldEcon {
            settings: setup.skirmish.clone(),
            start: crate::econ::setup::StartState {
                bases: setup.bases,
                ..Default::default()
            },
            ..Default::default()
        },
        effects: Vec::new(),
        notes: Vec::new(),
        theater: map.theater,
        // spec: S-0303 Import mapping (the map's special-flags section: Inert, HarvesterImmune)
        map_flags: crate::world::MapFlags {
            inert: map.basic.inert,
            harvester_immune: map.basic.harvester_immune,
        },
        rules: Some(rules.clone()),
        catalogue: Some(catalogue.clone()),
        regions: Default::default(),
        scratch: Default::default(),
        locomotion_under_way: Vec::new(),
        trace: Default::default(),
        pathfinder: Default::default(),
    };
    let q = rules.quirks;
    if !setup.sandbox {
        // spec: S-0202 claim 14 (house creation, first round: one ranged draw R(450, 1800) per house of S-0009 claim 19's list, in its order, before the early phase; the result unused)
        house_creation_round(&mut w, q.ranged_draw_half_span);
    }
    // spec: S-0409 claim 3 (the states at game start, before any object is placed)
    crate::econ::shroud::start(&mut w);
    // spec: S-0011 claim 4 and S-0009 claim 32 (the trigger definitions, the difficulty position and the variables)
    crate::triggers::begin(&mut w, map);
    for h in &mut w.houses {
        // spec: S-0410 claim 1 (the map's free-radar flag, fixed for the game)
        h.econ.radar.free = map.basic.free_radar;
        // The first house update works claim 1 out for every house (S-0410 claim 1 holds from the start).
        h.econ.radar.due = true;
    }
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

    if !setup.sandbox {
        // spec: S-0202 claim 14 (house creation, second round: the same list created again, one draw R(450, 1800) per house in the same order, after the early phase and before the initial fill)
        house_creation_round(&mut w, q.ranged_draw_half_span);
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

    // spec: S-0011 claim 34 (the tags that cell tags create, in entry order, after the initial fill's draws)
    crate::triggers::cell_tags(&mut w);

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
    // spec: S-0010 claim 19 (none of the tile animations the first pass started survives loading)
    crate::anims::drop_first_pass_tiles(&mut w);
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
            if let Some(c) = *b {
                // spec: S-0215 claim 38 (a start assignment supplies a base-start cell)
                crate::base::start_assigned(&mut w, HouseId(h as u8), c);
            }
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

    // spec: S-0011 claims 6, 34 and 35 (the end of loading: the remaining tags in registration order, polling, house assignment; after the secret labs, before the game-start pass)
    crate::triggers::end_of_loading(&mut w);

    // spec: S-0010 claim 12 (the game-start pass: every legal cell once more, in the visiting order, after the starting units and before the first tick; its tile animations by claim 19)
    // spec: S-0202 claim 14 (its tile-animation draws follow the starting units' final draw, the crate placement draws, the secret-lab choices and the end-of-loading tag draws, and precede the first tick)
    recalc_cells(&mut w, &order);

    // spec: S-0524 claim 9 (before the first tick, after tag setup and the game-start terrain pass: the starting-building record)
    crate::mp_triggers::record_starting_buildings(&mut w);

    // spec: S-0613 claim 1 (the shared gas system is the last object of the game-start step to enter play: after the map-placed objects, the starting units and every tile or cell animation started while loading, before only the ore-twinkle animations; appended to the update order, S-0203 claim 3)
    crate::gas::create_shared(&mut w);

    // spec: S-0202 claim 14 and table *Load-time draw order* (resource-glint setup after the game-start recalculation and before tick 0)
    resource_glints(&mut w, &order);

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
        // spec: S-0009 claim 28 (its triggers: the existing instances whose definition's owner country is the house's)
        let country = w.houses[h].country;
        crate::triggers::remove_country(w, country);
        let house = &mut w.houses[h];
        house.removed = true;
        house.start = None;
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
