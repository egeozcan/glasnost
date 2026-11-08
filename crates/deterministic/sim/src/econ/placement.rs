//! Building placement (S-0406): foundation cells, the cell test, adjacency,
//! explored ground, walls and upgrades; the construction state a placed
//! building begins (S-0108 claim 5).
//!
//! One core, [`put_on_map`], puts an existing building object on the map
//! for a place order ([`place_building`]), for a deploying vehicle
//! (`crate::econ::deploy`) and for a filling building of the wall fill.
//! The conditions judged when a place order is issued (claims 4 to 6) are
//! [`place_legal`]; the input rules of placement mode (claim 7, the cursor
//! shroud result of claim 6) are [`click_cell`] and [`cursor_shroud_result`].

use glasnost_map::{CellXY, NO_OVERLAY, Theater};
use glasnost_num::{F64, Rounding};
use glasnost_rules::schema::Ruleset;
use serde::{Deserialize, Serialize};

use super::TypeRef;
use crate::rules::{Behaviour, Kind};
use crate::world::{HouseId, HouseRole, Layer, ObjectId, Pos, World};

/// The game-wide values the structures topics read (S-0406, S-0408,
/// S-0415, S-0108).
#[derive(Clone, Debug, Default)]
pub struct Rules {
    /// The default of the build-off-ally option (S-0406 claim 4; S-0412
    /// owns the lobby setting).
    pub build_off_ally: bool,
    /// The default of the redeploy setting (S-0415 Terms; S-0412 owns the
    /// lobby setting).
    pub mcv_redeploys: bool,
    /// The build-up time *B* in game minutes, a double (S-0108 claim 5).
    pub buildup_time: F64,
    /// The refund percentage *r*, a double, narrowed when used (S-0408
    /// claim 1).
    pub refund_percent: F64,
}

impl Rules {
    /// The structures rules of a resolved ruleset.
    pub fn new(r: &Ruleset, mode: Rounding) -> Self {
        let g = &r.sim.general.economy;
        Self {
            // spec: S-0406 Constants (build-off-ally option absent-key default: on)
            build_off_ally: g.build_off_ally.unwrap_or(true),
            // spec: S-0412 Import mapping via S-0415 Terms (MCVRedeploys absent: yes)
            mcv_redeploys: g.mcv_redeploys.unwrap_or(true),
            // spec: S-0108 claim 5 and Import mapping (BuildupTime absent: the double nearest 0.05)
            buildup_time: g
                .buildup_time
                .as_ref()
                .map_or_else(|| crate::rules::nearest_double("0.05"), |d| d.value(mode)),
            // spec: S-0408 Constants (refund percentage absent-key default 0.5)
            refund_percent: g
                .refund_percent
                .as_ref()
                .map_or_else(|| crate::rules::nearest_double("0.5"), |d| d.value(mode)),
        }
    }
}

/// A building's construction state (S-0108 claim 5): set while it builds
/// up after a placement or a deploy, cleared when the construction has
/// completed. A building the map places is complete (S-0009 claim 40), so
/// the default is "complete".
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Construction {
    /// The construction has not completed yet (S-0408 claim 3 (a)).
    pub building_up: bool,
    /// The tick *C* of the construction behaviour's first processing
    /// (S-0406 claim 14 and F6); none before it.
    #[serde(default)]
    pub first: Option<i32>,
    /// The entry tick, from which the construction state's stage timing
    /// counts until *C* (S-0406 claim 17); none for a building that did
    /// not enter play under construction.
    #[serde(default)]
    pub entry: Option<i32>,
    /// The building has had its grand opening at an owner change during
    /// its build-up (S-0309 claim 31): the construction completes at its
    /// next processing with no second opening.
    #[serde(default)]
    pub opened: bool,
}

/// The placement fields of a building type (S-0406 Import mapping), with
/// their absent-key defaults.
#[derive(Clone, Debug, Default)]
pub struct PlaceType {
    /// Foundation rectangle *W* × *H* (S-0108 vocabulary).
    pub size: (i32, i32),
    /// The 8-cell 3 × 3 word (claim 2).
    pub refinery: bool,
    /// Place-anywhere flag (claim 3).
    pub anywhere: bool,
    /// Has a to-tile field (claims 3 and 5).
    pub to_tile: bool,
    /// Water-bound (claim 3 d).
    pub water_bound: bool,
    /// Naval flag (claim 3 d).
    pub naval: bool,
    /// The wall overlay, by overlay registration index (Terms).
    pub wall_overlay: Option<u8>,
    /// A wall type that has a wall overlay (claim 10; without one the wall
    /// flag is ignored).
    pub wall: bool,
    /// Gate flag (claims 3 b and 9).
    pub gate: bool,
    /// Laser-fence flag (claim 3 b, c).
    pub laser_fence: bool,
    /// Laser-fence-post flag (claim 3 b).
    pub post: bool,
    /// Firestorm-wall flag (claim 11).
    pub firestorm: bool,
    /// Adjacency *A* (claim 4).
    pub adjacent: i32,
    /// Base-normal flag (claim 4).
    pub base_normal: bool,
    /// Ally-eligible flag (claim 4).
    pub ally_eligible: bool,
    /// Guard range *G* in leptons (claim 11).
    pub guard_range: i32,
    /// The target type of an upgrade type (claim 12).
    pub upgrade_of: Option<u32>,
    /// The upgrade level field (claim 12).
    pub level: i32,
    /// The upgrade count *U* of the type (claim 12).
    pub upgrades: i32,
}

/// The placement fields of building type `t`.
pub fn place_type(w: &World, t: u32) -> PlaceType {
    let r = w.rules();
    let Some(b) = r.buildings.get(t as usize) else {
        return PlaceType::default();
    };
    let e = &b.raw.econ;
    let mode = r.mode;
    let wall_overlay = e.to_overlay.as_deref().and_then(|n| overlay_index(w, n));
    let support = support_flags(w, t);
    PlaceType {
        size: b.foundation,
        // spec: S-0406 claim 2 (the 8-cell 3 × 3 word; absent: no)
        refinery: b.raw.foundation_refinery.unwrap_or(false),
        // spec: S-0406 Constants (place-anywhere absent-key default: no)
        anywhere: e.place_anywhere.unwrap_or(false),
        // spec: S-0406 Import mapping (ToTile absent: none)
        to_tile: e.to_tile.is_some(),
        // spec: S-0406 Import mapping (WaterBound absent: no)
        water_bound: e.water_bound.unwrap_or(false),
        naval: b.naval,
        wall_overlay,
        // spec: S-0406 claim 10 (a wall flag without a wall overlay is ignored, departure)
        wall: e.wall.unwrap_or(false) && wall_overlay.is_some(),
        gate: b.gate,
        // spec: S-0406 Import mapping (LaserFence, LaserFencePost, FirestormWall absent: no)
        laser_fence: b.raw.laser_fence.unwrap_or(false),
        post: e.laser_fence_post.unwrap_or(false),
        firestorm: b.raw.firestorm_wall.unwrap_or(false),
        // spec: S-0406 Constants (adjacency absent-key default 3 cells)
        adjacent: e.adjacent.unwrap_or(3),
        base_normal: support.0,
        ally_eligible: support.1,
        // spec: S-0406 Import mapping (GuardRange: cells converted to leptons as S-0215 states, trunc(256 × c); absent 0)
        guard_range: e.guard_range.as_ref().map_or(0, |d| {
            d.value(mode)
                .mul(F64::from_i64(256, mode), mode)
                .to_i32_common()
        }),
        upgrade_of: e
            .powers_up_building
            .as_deref()
            .and_then(|n| r.index_of(Kind::Building, n)),
        // spec: S-0406 Import mapping (PowersUpToLevel absent: −1)
        level: e.powers_up_to_level.unwrap_or(-1),
        // spec: S-0406 Import mapping (Upgrades absent: 0)
        upgrades: e.upgrades.unwrap_or(0),
    }
}

/// The base-normal and ally-eligible flags of building type `t` (claim 4),
/// as [`place_type`] gives them, without the rest of its fields: the
/// adjacency scan asks them of every building it meets.
fn support_flags(w: &World, t: u32) -> (bool, bool) {
    let Some(b) = w.rules().buildings.get(t as usize) else {
        // As `PlaceType::default()`.
        return (false, false);
    };
    let e = &b.raw.econ;
    (
        // spec: S-0406 Constants (base-normal absent-key default yes)
        e.base_normal.unwrap_or(true),
        // spec: S-0406 Constants (ally-eligible absent-key default no)
        e.eligible_for_ally_building.unwrap_or(false),
    )
}

/// The overlay registration index of an overlay type name.
fn overlay_index(w: &World, name: &str) -> Option<u8> {
    w.rules()
        .overlays
        .iter()
        .position(|o| o.name.eq_ignore_ascii_case(name))
        .and_then(|i| u8::try_from(i).ok())
        .filter(|&i| i != NO_OVERLAY)
}

/// The builder of a wall overlay (S-0406 Terms): the first building type
/// in the registry whose wall overlay is `overlay`.
pub fn wall_builder(w: &World, overlay: u8) -> Option<u32> {
    let r = w.rules();
    (0..r.buildings.len() as u32).find(|&t| {
        r.buildings[t as usize]
            .raw
            .econ
            .to_overlay
            .as_deref()
            .and_then(|n| overlay_index(w, n))
            == Some(overlay)
    })
}

/// The foundation cells of a building of type `t` with top-left cell
/// `top_left` (S-0406 claim 2): the foundation rectangle, less the cell
/// (2, 1) of the 8-cell 3 × 3 word.
pub fn foundation_cells(w: &World, t: u32, top_left: CellXY) -> Vec<CellXY> {
    let p = place_type(w, t);
    cells_of(&p, top_left)
}

/// [`foundation_cells`] for placement fields `p`.
pub fn foundation_cells_of(p: &PlaceType, top_left: CellXY) -> Vec<CellXY> {
    cells_of(p, top_left)
}

fn cells_of(p: &PlaceType, top_left: CellXY) -> Vec<CellXY> {
    let mut v = crate::objects::foundation_cells(top_left, p.size);
    if p.refinery {
        // spec: S-0406 claim 2 and Constants (the 8-cell 3 × 3 word lacks the cell (2, 1))
        v.retain(|c| (c.x - top_left.x, c.y - top_left.y) != (2, 1));
    }
    v
}

/// Whose walls the wall exceptions of claim 8 accept.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WallRule {
    /// The placing house's walls (S-0406 claim 8).
    Owner(HouseId),
    /// Only walls that have no owner (S-0415 claim 3,
    /// `quirks.deploy_refuses_own_walls`).
    Unowned,
}

/// How a cell test is asked.
#[derive(Clone, Copy, Debug)]
pub struct Test {
    /// The house placing.
    pub house: HouseId,
    /// Whose walls the wall exceptions accept.
    pub walls: WallRule,
    /// The place-anywhere flag counts (not for a fill cell, claim 11).
    pub anywhere: bool,
    /// An object that does not count as being in the way (the deploying
    /// vehicle, S-0415 claim 3).
    pub ignore: Option<ObjectId>,
}

impl Test {
    /// The test of a place order by `house`.
    pub fn order(house: HouseId) -> Self {
        Self {
            house,
            walls: WallRule::Owner(house),
            anywhere: true,
            ignore: None,
        }
    }
}

/// Whether the cell `c` meets a wall exception of claim 8 for type `t`.
fn wall_exception(w: &World, t: u32, p: &PlaceType, c: CellXY, rule: WallRule) -> bool {
    let Some(i) = w.cells.idx(c) else {
        return false;
    };
    let Some(cell) = w.cells.grid.get(c).copied() else {
        return false;
    };
    if cell.overlay == NO_OVERLAY {
        return false;
    }
    let owner = w.cells.wall_owner[i];
    let owned = match rule {
        WallRule::Owner(h) => owner == Some(h),
        WallRule::Unowned => owner.is_none(),
    };
    if !owned {
        return false;
    }
    let g = w.rules().gates;
    let t = Some(t);
    // spec: S-0406 Constants (damaged wall threshold: overlay data value 16)
    let damaged = cell.overlay_data >= 16;
    let own_kind = p.wall_overlay == Some(cell.overlay) && damaged;
    match cell.overlay {
        // spec: S-0406 claim 8 (indices 0 and 2: own type's damaged wall, the wall tower, the first gate pair)
        0 | 2 => own_kind || t == g.tower || t == g.allied_one || t == g.allied_two,
        // spec: S-0406 claim 8 (index 26: own type's damaged wall, the second gate pair)
        26 => own_kind || t == g.soviet_one || t == g.soviet_two,
        _ => false,
    }
}

/// Whether the tile number lies in the naval placement range (S-0406
/// claim 3 d).
fn naval_tile(w: &World, tile: i32) -> bool {
    // spec: S-0406 claim 3 d (the theater with index 5 forces the water role unset)
    let start = if w.theater == Theater::Lunar {
        -1
    } else {
        w.catalogue.as_deref().map_or(-1, |c| c.roles.water)
    };
    if start < 0 {
        // spec: S-0406 Constants (water role unset: tile numbers 0 to 12, quirks.unset_role_range)
        return w.rules().quirks.unset_role_range && (0..=12).contains(&tile);
    }
    // spec: S-0406 Constants (naval placement range: 14 tiles from the water role's start)
    (start..start + 14).contains(&tile)
}

/// Whether a cell is marked as part of a bridge (S-0406 claim 3 d).
fn bridge_marked(w: &World, c: CellXY) -> bool {
    w.cells
        .idx(c)
        .is_some_and(|i| w.cells.deck[i] || w.cells.bridge_body[i])
}

/// Condition (d), ground (S-0406 claim 3 d).
fn ground_ok(w: &World, p: &PlaceType, c: CellXY) -> bool {
    let Some(cell) = w.cells.cell(c) else {
        return false;
    };
    if p.water_bound {
        // spec: S-0406 claim 3 d (water-bound: the float multiplier of the land type is not 0)
        let m = w
            .rules()
            .multiplier(cell.land, glasnost_rules::sim::LocomotionClass::Float);
        return m != glasnost_num::F32::ZERO;
    }
    // spec: S-0406 claim 3 d (not bridge-marked and slope shape 0)
    if bridge_marked(w, c) || cell.slope != 0 {
        return false;
    }
    if p.naval {
        // spec: S-0406 claim 3 d (naval: the tile in the water role range, B not asked)
        naval_tile(w, cell.tile)
    } else {
        // spec: S-0406 claim 3 d (the buildable flag B of the land type)
        w.rules()
            .land
            .get(usize::from(cell.land))
            .is_some_and(|l| l.buildable)
    }
}

/// The vehicle-present claims the ignored object holds in `c`.
fn ignored_claims(w: &World, ignore: Option<ObjectId>, c: CellXY) -> u8 {
    let Some(o) = ignore.and_then(|id| w.obj(id)) else {
        return 0;
    };
    let drive = o
        .drive
        .as_ref()
        .map_or(0, |d| u8::from(d.claims.contains(&c)));
    // spec: S-0415 claim 3 (the deploying vehicle does not count as being in the way) and S-0503 claim 19 (a hover's deploy attempt takes the cell test at the cell where it stopped: its own vehicle-present flags there, S-0503 claim 13, do not count)
    let hover = o.hover.as_ref().map_or(0, |h| {
        h.claims
            .iter()
            .filter(|&&(x, l)| x == c && l == Layer::Ground)
            .count()
    });
    drive.saturating_add(u8::try_from(hover).unwrap_or(u8::MAX))
}

/// Condition (b), nothing in the way (S-0406 claim 3 b), with the object
/// conditions of gate, post and laser-fence types.
fn nothing_in_way(w: &World, p: &PlaceType, c: CellXY, test: &Test) -> bool {
    for &id in w.cells.list(c, Layer::Ground) {
        if Some(id) == test.ignore {
            // spec: S-0415 claim 3 (the deploying vehicle does not count as being in the way)
            continue;
        }
        let Some(o) = w.obj(id) else { continue };
        if p.laser_fence {
            // spec: S-0406 claim 3 b (a laser-fence type: only a building or a terrain object refuses)
            if matches!(o.kind, Kind::Building | Kind::Terrain) {
                return false;
            }
            continue;
        }
        if (p.gate || p.post)
            && o.kind == Kind::Building
            && o.owner == Some(test.house)
            && w.rules().buildings[o.type_index as usize]
                .raw
                .laser_fence
                .unwrap_or(false)
        {
            // spec: S-0406 claim 3 b (a gate or post type: an own laser-fence building does not refuse)
            continue;
        }
        // spec: S-0406 claim 3 b (an aircraft, infantry unit, vehicle, building or terrain object refuses)
        return false;
    }
    if p.laser_fence {
        return true;
    }
    let Some(rec) = w.cells.occ(c, Layer::Ground) else {
        return false;
    };
    // spec: S-0406 claim 3 b (the occupancy record: no standing position taken and no vehicle present)
    let taken = rec.marks.iter().any(Option::is_some);
    let vehicles = rec
        .vehicles
        .saturating_sub(ignored_claims(w, test.ignore, c));
    !taken && vehicles == 0
}

/// Condition (c) with its exceptions: `Some(true)` when the cell is exempt
/// from (c) and (d), `Some(false)` when it passes (c) and must pass (d),
/// `None` when (c) refuses.
fn overlay_ok(w: &World, t: u32, p: &PlaceType, c: CellXY, test: &Test) -> Option<bool> {
    let cell = w.cells.cell(c)?;
    if cell.overlay == NO_OVERLAY {
        return Some(false);
    }
    if wall_exception(w, t, p, c, test.walls) {
        // spec: S-0406 claim 8 (a wall exception: exempt from (c) and (d))
        let ground = !w.rules().quirks.wall_exception_skips_ground;
        // spec: S-0406 Edge cases (quirks.wall_exception_skips_ground off: (d) still asked)
        return Some(!ground);
    }
    if p.laser_fence {
        let ov = w.rules().overlays.get(usize::from(cell.overlay));
        // spec: S-0406 claim 3 c (a laser-fence type on ore, gems or overlay index 126, not bridge-marked, slope 0)
        let fence_overlay = cell.overlay == 126 || ov.is_some_and(|o| o.tiberium);
        if fence_overlay && !bridge_marked(w, c) && cell.slope == 0 {
            return Some(true);
        }
    }
    // spec: S-0406 claim 3 c (any other overlay refuses)
    None
}

/// The cell test of one cell for type `t` (S-0406 claims 3 and 8), the
/// to-tile rule's tileset condition aside.
fn cell_passes(w: &World, t: u32, p: &PlaceType, c: CellXY, test: &Test, to_tile: bool) -> bool {
    // spec: S-0406 claim 3 a (a usable cell, height-aware)
    if !w.cells.usable(c) {
        return false;
    }
    let way = if to_tile && !(p.laser_fence || p.gate || p.post) {
        // spec: S-0406 claim 3 (to-tile rule: no building in place of (b), and the tileset test)
        let no_building = !w.cells.list(c, Layer::Ground).iter().any(|&id| {
            Some(id) != test.ignore && w.obj(id).is_some_and(|o| o.kind == Kind::Building)
        });
        no_building && tileset_passes(w, c)
    } else {
        nothing_in_way(w, p, c, test)
    };
    if !way {
        return false;
    }
    match overlay_ok(w, t, p, c, test) {
        None => false,
        Some(true) => true,
        Some(false) => ground_ok(w, p, c),
    }
}

/// The tileset test of the to-tile rule (S-0406 claim 3; S-0010 claim
/// 15): a tile of a morphable tileset, or a tile number outside the tile
/// range. Seam: the tile catalogue does not carry the morphable flag yet,
/// so only a tile number outside the range passes.
fn tileset_passes(w: &World, c: CellXY) -> bool {
    let tile = w.cells.cell(c).map_or(-1, |x| x.tile);
    let count = w.catalogue.as_deref().map_or(0, |cat| cat.tile_count());
    // spec: S-0406 claim 3 (a tile number outside the tile range always passes)
    !(0..count).contains(&tile)
}

/// The cell test (S-0406 claims 3 and 8) of type `t` at top-left cell
/// `top_left`.
pub fn cell_test(w: &World, t: u32, top_left: CellXY, test: &Test) -> bool {
    cell_test_with(w, t, &place_type(w, t), top_left, test)
}

/// [`cell_test`] with type `t`'s placement fields `p` already read.
pub fn cell_test_with(w: &World, t: u32, p: &PlaceType, top_left: CellXY, test: &Test) -> bool {
    if test.anywhere && p.anywhere {
        // spec: S-0406 claim 3 (place-anywhere: none of (a) to (d) is asked; it takes precedence over a to-tile field)
        return true;
    }
    // The foundation cells of `cells_of`, in its order, without a list.
    let (fw, fh) = p.size;
    let mut cells = (0..fh.max(0)).flat_map(|dy| {
        // spec: S-0406 claim 2 and Constants (the 8-cell 3 × 3 word lacks the cell (2, 1))
        let lacks = move |dx: i32| p.refinery && (dx, dy) == (2, 1);
        (0..fw.max(0))
            .filter(move |&dx| !lacks(dx))
            .map(move |dx| CellXY::new(top_left.x + dx, top_left.y + dy))
    });
    if p.to_tile {
        // spec: S-0406 claim 3 (to-tile rule: one foundation cell passing is enough; no cells passes)
        let mut any = false;
        let mut none = true;
        for c in cells {
            none = false;
            if cell_passes(w, t, p, c, test, true) {
                any = true;
                break;
            }
        }
        return none || any;
    }
    // spec: S-0406 claim 3 (every foundation cell passes; no cells passes every condition, claim 2)
    cells.all(|c| cell_passes(w, t, p, c, test, false))
}

/// Whether the build-off-ally option is on (S-0406 claim 4; the setup
/// value of S-0412 claim 1, defaulting to the rules' default).
pub fn build_off_ally(w: &World) -> bool {
    crate::econ::setup::build_off_ally(w)
}

/// The adjacency condition (S-0406 claim 4, F1).
pub fn adjacent(w: &World, house: HouseId, t: u32, top_left: CellXY) -> bool {
    let ally_option = build_off_ally(w);
    adjacent_by(&place_type(w, t), top_left, |c| {
        supports(w, house, ally_option, c)
    })
}

/// The adjacency rectangle's margin *d* of F1 and the foundation's
/// width and height of placement fields `p`.
fn adjacency_reach(p: &PlaceType) -> (i32, i32, i32) {
    // spec: S-0406 F1 (d = A + 1; the zero-size word's rectangle is 0 × 0)
    let d = p.adjacent.saturating_add(1);
    (d, p.size.0.max(0), p.size.1.max(0))
}

/// [`adjacent`] for placement fields `p`, with the cell test `support`
/// (whether a cell holds a building the house may build next to).
fn adjacent_by(p: &PlaceType, top_left: CellXY, mut support: impl FnMut(CellXY) -> bool) -> bool {
    let (d, fw, fh) = adjacency_reach(p);
    let (x0, y0) = (top_left.x, top_left.y);
    for y in (y0 - d)..=(y0 + fh - 1 + d) {
        for x in (x0 - d)..=(x0 + fw - 1 + d) {
            // spec: S-0406 F1 (the foundation rectangle itself is not looked at)
            if (x0..x0 + fw).contains(&x) && (y0..y0 + fh).contains(&y) {
                continue;
            }
            if support(CellXY::new(x, y)) {
                return true;
            }
        }
    }
    false
}

/// Whether cell `c` holds a building that lets `house` build next to it
/// (S-0406 claim 4), the build-off-ally option being `ally_option`.
fn supports(w: &World, house: HouseId, ally_option: bool, c: CellXY) -> bool {
    w.cells.list(c, Layer::Ground).iter().any(|&id| {
        let Some(o) = w.obj(id) else { return false };
        if o.kind != Kind::Building {
            return false;
        }
        let Some(owner) = o.owner else { return false };
        let (base_normal, ally_eligible) = support_flags(w, o.type_index);
        // spec: S-0406 claim 4 (own base-normal building, any state)
        if owner == house && base_normal {
            return true;
        }
        // spec: S-0406 claim 4 (build-off-ally: the owner allies the placing house, an ally-eligible building; a house allies itself)
        let allies = owner == house
            || w.houses
                .get(usize::from(owner.0))
                .is_some_and(|h| h.allied_with(house));
        ally_option && allies && ally_eligible
    })
}

/// The place order's conditions ([`place_legal`]) and the cell test of a
/// place order ([`cell_test`] with [`Test::order`]) of one building type
/// for one house on an unchanged world, asked of many top-left cells (our
/// tooling for speed, M8 C4 parts 2 and 6; nothing here is game state).
/// The type's placement fields are read once, and the adjacency condition
/// of each top-left cell in the window of the cells the search was made for
/// is read off running sums of the cells that hold a building the house
/// may build next to, each such cell asked once: the same answers as
/// asking the functions themselves, cell by cell.
pub struct PlaceSearch {
    house: HouseId,
    t: TypeRef,
    p: PlaceType,
    table: Option<SupportTable>,
}

/// The running sums behind [`PlaceSearch`]'s adjacency answers.
struct SupportTable {
    /// The window of top-left cells the table answers for: least and
    /// greatest x and y.
    window: (i32, i32, i32, i32),
    /// The area's first cell (the window's least corner less the margin).
    x0: i32,
    y0: i32,
    /// The area's width.
    aw: i32,
    /// `sums[(y + 1) × (aw + 1) + x + 1]`: the supporting cells of the
    /// area with offsets at most (x, y).
    sums: Vec<u32>,
}

/// The largest adjacency margin for which [`PlaceSearch`] keeps a table
/// (a wider one is answered cell by cell).
const TABLE_MARGIN: i32 = 64; // nocite: our tooling's bound (M8 C4 part 6), not game behaviour

impl SupportTable {
    fn new(w: &World, house: HouseId, p: &PlaceType, cells: &[CellXY]) -> Option<Self> {
        let (d, fw, fh) = adjacency_reach(p);
        if !(1..=TABLE_MARGIN).contains(&d) || cells.is_empty() {
            return None;
        }
        let lx = cells.iter().map(|c| c.x).min()?;
        let hx = cells.iter().map(|c| c.x).max()?;
        let ly = cells.iter().map(|c| c.y).min()?;
        let hy = cells.iter().map(|c| c.y).max()?;
        let (x0, y0) = (lx - d, ly - d);
        let aw = hx + fw - 1 + d - x0 + 1;
        let ah = hy + fh - 1 + d - y0 + 1;
        let row = usize::try_from(aw + 1).ok()?;
        let mut sums = vec![0u32; row * usize::try_from(ah + 1).ok()?];
        let ally_option = build_off_ally(w);
        for y in 0..ah {
            let mut run = 0u32;
            for x in 0..aw {
                let c = CellXY::new(x0 + x, y0 + y);
                // Off the grid a cell's list is empty, so it supports nothing.
                if w.cells.idx(c).is_some() && supports(w, house, ally_option, c) {
                    run += 1;
                }
                let (ux, uy) = (x as usize, y as usize);
                sums[(uy + 1) * row + ux + 1] = sums[uy * row + ux + 1] + run;
            }
        }
        Some(Self {
            window: (lx, ly, hx, hy),
            x0,
            y0,
            aw,
            sums,
        })
    }

    /// The supporting cells in the rectangle from (`ax`, `ay`) to (`bx`,
    /// `by`), inclusive, which lies inside the area; 0 when it is empty.
    fn count(&self, ax: i32, ay: i32, bx: i32, by: i32) -> u32 {
        if ax > bx || ay > by {
            return 0;
        }
        let row = (self.aw + 1) as usize;
        let at = |x: i32, y: i32| self.sums[(y - self.y0) as usize * row + (x - self.x0) as usize];
        at(bx + 1, by + 1) + at(ax, ay) - at(ax, by + 1) - at(bx + 1, ay)
    }

    /// [`adjacent_by`]'s answer for top-left cell `c`, when `c` lies in the
    /// window.
    fn adjacent(&self, p: &PlaceType, c: CellXY) -> Option<bool> {
        let (lx, ly, hx, hy) = self.window;
        if c.x < lx || c.x > hx || c.y < ly || c.y > hy {
            return None;
        }
        let (d, fw, fh) = adjacency_reach(p);
        // spec: S-0406 F1 (the rectangle around the foundation, less the foundation rectangle itself)
        let around = self.count(c.x - d, c.y - d, c.x + fw - 1 + d, c.y + fh - 1 + d);
        let inside = self.count(c.x, c.y, c.x + fw - 1, c.y + fh - 1);
        Some(around > inside)
    }
}

impl PlaceSearch {
    /// A search for type `t` of `house` on `w`, whose adjacency table
    /// covers the top-left cells `cells` (other cells are answered without
    /// it).
    pub fn new(w: &World, house: HouseId, t: TypeRef, cells: &[CellXY]) -> Self {
        let p = if t.kind == Kind::Building {
            place_type(w, t.index)
        } else {
            PlaceType::default()
        };
        let table = (t.kind == Kind::Building && p.upgrade_of.is_none())
            .then(|| SupportTable::new(w, house, &p, cells))
            .flatten();
        Self { house, t, p, table }
    }

    /// The type's placement fields.
    pub fn place_type(&self) -> &PlaceType {
        &self.p
    }

    /// [`place_legal`] of the search's house and type at `cell`.
    pub fn place_legal(&self, w: &World, cell: CellXY) -> bool {
        place_legal_by(w, self.house, self.t, &self.p, cell, |c| {
            self.table
                .as_ref()
                .and_then(|k| k.adjacent(&self.p, c))
                .unwrap_or_else(|| {
                    let ally_option = build_off_ally(w);
                    adjacent_by(&self.p, c, |k| supports(w, self.house, ally_option, k))
                })
        })
    }

    /// [`cell_test`] of the search's type at `cell`, as a place order of
    /// its house asks it ([`Test::order`]).
    pub fn cell_test(&self, w: &World, cell: CellXY) -> bool {
        cell_test_with(w, self.t.index, &self.p, cell, &Test::order(self.house))
    }
}

/// The explored-ground condition (S-0406 claim 5): every cell of the
/// foundation rectangle is explored by `house`; a to-tile type is exempt.
pub fn explored(w: &World, house: HouseId, t: u32, top_left: CellXY) -> bool {
    explored_with(w, house, &place_type(w, t), top_left)
}

/// [`explored`] for placement fields `p`.
fn explored_with(w: &World, house: HouseId, p: &PlaceType, top_left: CellXY) -> bool {
    if p.to_tile {
        // spec: S-0406 claim 5 (a type with a to-tile field is exempt)
        return true;
    }
    // spec: S-0406 claim 5 (every cell of the foundation rectangle, 9 cells for the 8-cell word)
    (0..p.size.1).all(|dy| {
        (0..p.size.0).all(|dx| {
            crate::econ::shroud::explored(w, house, CellXY::new(top_left.x + dx, top_left.y + dy))
        })
    })
}

/// The conditions judged when a place order is issued (S-0406 claims 4 to
/// 6 and 12): adjacency, or for an upgrade type a building it can go on,
/// and explored ground. Applying the order does not judge them again.
pub fn place_legal(w: &World, house: HouseId, t: TypeRef, cell: CellXY) -> bool {
    if t.kind != Kind::Building {
        return false;
    }
    let p = place_type(w, t.index);
    place_legal_by(w, house, t, &p, cell, |c| adjacent(w, house, t.index, c))
}

/// [`place_legal`] for type `t` with placement fields `p` and the
/// adjacency test `adjacent` of a top-left cell.
fn place_legal_by(
    w: &World,
    house: HouseId,
    t: TypeRef,
    p: &PlaceType,
    cell: CellXY,
    adjacent: impl FnOnce(CellXY) -> bool,
) -> bool {
    if t.kind != Kind::Building {
        return false;
    }
    let support = if p.upgrade_of.is_some() {
        // spec: S-0406 claim 12 (an upgrade order that names such a building passes claim 4)
        upgrade_target(w, house, t.index, cell).is_some()
    } else {
        adjacent(cell)
    };
    // spec: S-0406 claims 5 and 12 (the shroud condition is asked of an upgrade type's rectangle too)
    support && explored_with(w, house, p, cell)
}

/// The cell a click on `click` names in placement mode (S-0406 claim 7,
/// F2).
pub fn click_cell(w: &World, t: u32, click: CellXY) -> CellXY {
    let p = place_type(w, t);
    let cells = cells_of(&p, CellXY::new(0, 0));
    // spec: S-0406 F2 (the bounding rectangle of the foundation cells; 1 × 1 with no cell)
    let (bw, bh) = if cells.is_empty() {
        (1, 1)
    } else {
        let mx = cells.iter().map(|c| c.x).max().unwrap_or(0);
        let my = cells.iter().map(|c| c.y).max().unwrap_or(0);
        let nx = cells.iter().map(|c| c.x).min().unwrap_or(0);
        let ny = cells.iter().map(|c| c.y).min().unwrap_or(0);
        (mx - nx + 1, my - ny + 1)
    };
    // spec: S-0406 F2 (top-left = (x − ⌊(W − 1)/2⌋, y − ⌊(H − 1)/2⌋))
    CellXY::new(click.x - (bw - 1) / 2, click.y - (bh - 1) / 2)
}

/// The cursor shroud result (S-0406 Terms, claim 6): claim 5 for the
/// foundation rectangle that a click on `cursor` would name. Placement mode
/// recomputes it whenever the cell under the cursor changes
/// (`quirks.placement_shroud_at_cursor`).
pub fn cursor_shroud_result(w: &World, house: HouseId, t: u32, cursor: CellXY) -> bool {
    explored(w, house, t, click_cell(w, t, cursor))
}

/// Whether a factory building of `house` could make building type `t`
/// without the tech-tree rule (S-0406 claim 1, S-0404 Terms): on the map,
/// neither being sold nor queued to be sold, a factory of buildings, sharing an owner country with
/// the type.
pub fn factory_could_make(w: &World, house: HouseId, t: u32) -> bool {
    let r = w.rules();
    let Some(item) = r.buildings.get(t as usize) else {
        return false;
    };
    let item_owners = item.raw.owner.clone().unwrap_or_default();
    w.objects.values().any(|o| {
        o.kind == Kind::Building
            && o.owner == Some(house)
            && o.on_map
            && o.health > 0
            // spec: S-0404 Terms via S-0406 claim 1 (a building being sold or queued to be sold cannot make)
            && !crate::econ::tech::being_sold(o)
            && !crate::econ::tech::queued_to_be_sold(o)
            && r.buildings[o.type_index as usize].factory
                == Some(crate::rules::FactoryKind::Building)
            // spec: S-0404 Terms (its type's owner countries share one with the item type's)
            && r.buildings[o.type_index as usize]
                .raw
                .owner
                .iter()
                .flatten()
                .any(|c| item_owners.contains(c))
    })
}

/// The building object a place order places: the done item's object
/// (S-0404 claim 7), which the placement puts on the map without creating
/// one. A caller with no done item of the type (a direct engine call, as in
/// the claim tests; no order reaches here without one, since S-0404 claim 7
/// asks for the done item first) gets a new object with its creation draw.
fn item_object(w: &mut World, house: HouseId, t: TypeRef) -> ObjectId {
    crate::econ::production::item_object(w, house, t)
        .filter(|&id| w.obj(id).is_some())
        .unwrap_or_else(|| crate::objects::create(w, Kind::Building, t.index, Some(house)))
}

/// The cue of a placement refused as it is applied: the placement refusal
/// announcement for the house's own player, no game-wide refusal sound.
fn refused_cue(w: &mut World, house: HouseId, t: TypeRef) {
    // spec: S-0404 claim 7; S-0406 claims 1 and 12; S-0701 claim 24 (placement refused as it is applied: the cannot-deploy-here announcement for the house's own player; a missing factory is silent)
    w.effects.push(crate::combat::effects::Effect::Sound {
        sound: crate::combat::effects::Sound::CannotDeploy,
        of: Some(t),
        house: Some(house),
        pos: None,
    });
}

/// Applies the placement of a done building (S-0406 claims 1, 3, 8 to
/// 12): the item's object is put on the map at `cell`, or added as an
/// upgrade to the building there. Returns whether it was placed; on
/// failure nothing is placed (a cell-test refusal requests the cannot-deploy
/// announcement, S-0404 claim 7; placement mode is the
/// input's, S-0411). The factory condition of claim 1 is asked first.
pub fn place_building(w: &mut World, house: HouseId, t: TypeRef, cell: CellXY) -> bool {
    if t.kind != Kind::Building {
        return false;
    }
    // spec: S-0406 claim 1 (without a factory building that could make the type the order does nothing)
    if !factory_could_make(w, house, t.index) {
        return false;
    }
    let p = place_type(w, t.index);
    if p.upgrade_of.is_some() {
        // spec: S-0406 claim 12 (an upgrade type is not placed by the cell test)
        // spec: S-0406 claim 12 (the target's owner, type and room are checked again when the order is applied)
        let Some(target) = upgrade_target(w, house, t.index, cell) else {
            refused_cue(w, house, t);
            return false;
        };
        add_upgrade(w, target, t.index);
        return true;
    }
    if !cell_test(w, t.index, cell, &Test::order(house)) {
        refused_cue(w, house, t);
        return false;
    }
    let id = item_object(w, house, t);
    put_on_map(w, id, cell, true);
    true
}

/// Puts the building `id` on the map with top-left cell `cell`, after its
/// cell test has passed (S-0406 claims 1 and 9 to 11): walls underneath
/// removed, a wall type turned into its overlay, otherwise the building put
/// at the centre of the cell at the surface height in its construction
/// state; then the wall fill for a place order (`by_order`). The facing is
/// 0 unless the caller sets one.
pub fn put_on_map(w: &mut World, id: ObjectId, cell: CellXY, by_order: bool) {
    // spec: S-0406 claim 16 and Constants (a placed building faces 0, map north)
    put_on_map_facing(w, id, cell, by_order, 0);
}

/// [`put_on_map`] with a body facing (a deploy, S-0415 claim 6).
pub fn put_on_map_facing(
    w: &mut World,
    id: ObjectId,
    cell: CellXY,
    by_order: bool,
    facing: glasnost_num::geom::Facing,
) {
    let Some(o) = w.obj(id) else { return };
    let (t, house) = (o.type_index, o.owner);
    let p = place_type(w, t);
    if let Some(h) = house {
        remove_underneath(w, t, &p, h, cell);
    }
    if p.wall {
        // spec: S-0406 claim 10 (a wall type: its wall overlay in the cell, the placing house its owner; no building remains)
        w.objects.remove(&id);
        // spec: S-0406 claim 10 (pending resolution our engine retains a damaged wall that claim 9 could not remove, with its owner and damage state unchanged)
        let kept = w.cells.cell(cell).is_some_and(|x| x.overlay != NO_OVERLAY);
        if let Some(ov) = p.wall_overlay
            && !kept
        {
            put_wall(w, cell, ov, house);
        }
    } else {
        // spec: S-0406 claim 1 and F4 (the centre of the cell, (256x + 128, 256y + 128), at the surface height)
        let z = w.surface_at(256 * cell.x + 128, 256 * cell.y + 128);
        // A place order or a wall fill is placed; any other insertion is a deploy's (S-0415 claim 6).
        begin_construction(w, id, !by_order);
        crate::objects::place_with(w, id, Pos::centre(cell, z), facing, by_order);
    }
    if by_order
        && let (Some(h), Some(ov)) = (house, p.wall_overlay)
        && crate::walls::is_wall(w, ov)
        && !p.firestorm
    {
        wall_fill(w, t, &p, h, cell, ov);
    }
}

/// Starts a building's construction state at entry (S-0108 claim 5,
/// S-0406 claims 1 and 17, S-0415 claim 6): no current behaviour and
/// *construction* queued; a deploy (`deployed`) also sets the ready mark.
pub fn begin_construction(w: &mut World, id: ObjectId, deployed: bool) {
    let t = w.tick;
    // spec: S-0406 Edge cases (quirks.placed_buildup_twice off: a place order sets the ready mark at entry, as a deploy does)
    let mark = deployed || !w.rules().quirks.placed_buildup_twice;
    // spec: S-0108 claim 6 (entering construction leaves the idle body)
    crate::econ::body::leave_idle(w, id);
    if let Some(o) = w.obj_mut(id) {
        // spec: S-0406 claim 17 (at entry: no current behaviour and construction queued; stage 0, timed from the entry tick)
        o.beh.current = None;
        o.beh.queued = Some(Behaviour::Construction);
        o.econ.construction.building_up = true;
        o.econ.construction.first = None;
        o.econ.construction.entry = Some(t);
        o.econ.construction.opened = false;
        if mark {
            // spec: S-0406 claim 17 (a deploy sets the ready mark when the building enters play)
            o.beh.ready_mark = true;
        }
    }
}

/// The behaviour a rule sees when it asks for a building's behaviour
/// (S-0406 claim 17, S-0215 claim 1): the current one, or the queued one
/// (such as *construction*) while none is current.
pub fn seen_behaviour(o: &crate::world::Object) -> Option<Behaviour> {
    // spec: S-0406 claim 17 and S-0215 claim 1 (a rule that asks for its behaviour while none is current sees the queued one)
    o.beh.seen()
}

/// Whether the first per-tick start of a building's turn is withheld
/// (S-0406 claim 17): while its construction state lasts.
pub fn first_start_withheld(w: &World, id: ObjectId) -> bool {
    // spec: S-0406 claim 17 (the first start of a tick does not apply while the construction state lasts)
    w.obj(id)
        .is_some_and(|o| o.kind == Kind::Building && o.econ.construction.building_up)
}

/// The construction state's part of a building turn before its first
/// per-tick start (S-0406 claims 17 and 18): until *construction* is
/// current, the ready mark is set on the turn the build-up run from entry
/// reports its last stage, or on any turn of a zero-delay range.
pub fn construction_turn(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let c = &o.econ.construction;
    if !c.building_up || c.first.is_some() || o.beh.current == Some(Behaviour::Construction) {
        return;
    }
    let Some(entry) = c.entry else { return };
    let never = w.rules().quirks.single_stage_buildup_never_completes;
    let range = crate::econ::sell::buildup_range(w, o.type_index);
    // spec: S-0406 claim 17 (a) and (b) (the mark on the turn the stage lands on the last stage, R = T + (n − 1) × d_b, or on any turn of a zero-delay range); claim 18 (never for a positive-delay range of one stage or fewer)
    let due = range
        .report_after(never)
        .is_some_and(|a| w.tick.wrapping_sub(entry) >= a);
    if due && let Some(ob) = w.obj_mut(id) {
        ob.beh.ready_mark = true;
    }
}

/// The end of a building turn for a building that had its grand opening at
/// an owner change during its build-up (S-0309 claim 31): once
/// *construction* is neither current nor queued and the idle body has been
/// requested, the idle body takes the build-up's place at the end of the
/// turn, and the construction state ends with it.
pub fn opened_turn_end(w: &mut World, id: ObjectId) {
    let Some(ob) = w.obj_mut(id) else { return };
    let c = &ob.econ.construction;
    let construction = ob.beh.current == Some(Behaviour::Construction)
        || ob.beh.queued == Some(Behaviour::Construction);
    // spec: S-0309 claim 31 (a building being built up: the idle body follows by S-0108 claim 6 from the idle-body request of the change, or from the one that guard's first processing makes, at the end of that turn; the first start of the turn makes nothing current while the construction state lasts)
    if !c.building_up || !c.opened || construction || ob.econ.idle_body.is_none() {
        return;
    }
    ob.econ.construction = Construction::default();
}

/// Puts a wall overlay in a cell for its owner (S-0406 claim 10; S-0112
/// claim 7: a new wall starts with data value 0).
pub(crate) fn put_wall(w: &mut World, c: CellXY, overlay: u8, owner: Option<HouseId>) {
    let Some(i) = w.cells.idx(c) else { return };
    if let Some(cell) = w.cells.grid.get_mut(c) {
        cell.overlay = overlay;
        // spec: S-0112 claim 7 (a newly placed wall starts with data value 0)
        cell.overlay_data = 0;
    }
    w.cells.wall_owner[i] = owner;
    // spec: S-0112 claim 7 (a wall placed in c: c and its four edge neighbours)
    crate::walls::wall_placed(w, c);
    crate::load::recalc_cells(w, &[c]);
    crate::regions::cell_changed(w, c);
}

/// What a placement removes underneath (S-0406 claim 9).
fn remove_underneath(w: &mut World, t: u32, p: &PlaceType, house: HouseId, cell: CellXY) {
    let g = w.rules().gates;
    let overlay_at = |w: &World, c: CellXY| w.cells.cell(c).map_or(NO_OVERLAY, |x| x.overlay);
    let owned = |w: &World, c: CellXY| {
        w.cells
            .idx(c)
            .is_some_and(|i| w.cells.wall_owner[i] == Some(house))
    };
    if Some(t) == g.tower {
        // spec: S-0406 claim 9 (the wall tower: its top-left cell's own wall of index 0 or 2)
        if owned(w, cell) && matches!(overlay_at(w, cell), 0 | 2) {
            crate::econ::sell::wall_sale(w, house, cell, true);
        }
    } else if let Some(ov) = p.wall_overlay {
        // spec: S-0406 claim 9 (a type with a wall overlay: its top-left cell's own wall of that overlay)
        if owned(w, cell) && overlay_at(w, cell) == ov {
            crate::econ::sell::wall_sale(w, house, cell, true);
        }
    } else if p.gate {
        // spec: S-0406 claim 9 (a gate-flag type: each foundation cell's own wall of index 0, 2 or 26, and own laser-fence buildings)
        for c in cells_of(p, cell) {
            if owned(w, c) && matches!(overlay_at(w, c), 0 | 2 | 26) {
                crate::econ::sell::wall_sale(w, house, c, true);
            }
            let fences: Vec<ObjectId> = w
                .cells
                .list(c, Layer::Ground)
                .iter()
                .copied()
                .filter(|&x| {
                    w.obj(x).is_some_and(|o| {
                        o.kind == Kind::Building
                            && o.owner == Some(house)
                            && w.rules().buildings[o.type_index as usize]
                                .raw
                                .laser_fence
                                .unwrap_or(false)
                    })
                })
                .collect();
            for f in fences {
                // spec: S-0406 claim 9 (a laser-fence building removed from play without refund)
                crate::objects::remove(w, f);
            }
        }
    }
}

/// The wall fill (S-0406 claim 11, F3).
fn wall_fill(w: &mut World, t: u32, p: &PlaceType, house: HouseId, cell: CellXY, ov: u8) {
    // spec: S-0406 F3 (L = ⌊G / 256⌋, toward −∞)
    let l = p.guard_range.div_euclid(256);
    let test = Test {
        house,
        walls: WallRule::Owner(house),
        // spec: S-0406 claim 11 (the fill's cell test has no place-anywhere exemption)
        anywhere: false,
        ignore: None,
    };
    // spec: S-0406 claim 11 (four map directions; they do not interact, any order)
    for (dx, dy) in crate::walls::EDGE_STEPS {
        let mut gap = Vec::new();
        let mut found = false;
        for k in 1..=l {
            let c = CellXY::new(cell.x + k * dx, cell.y + k * dy);
            let holds = w.cells.idx(c).is_some_and(|i| {
                w.cells.wall_owner[i] == Some(house)
                    && w.cells.cell(c).is_some_and(|x| x.overlay == ov)
            });
            if holds {
                // spec: S-0406 claim 11 (the nearest own wall of the type, damaged or not, ends the direction)
                found = true;
                break;
            }
            // spec: S-0406 claim 11 (every cell before it passes the cell test)
            if !cell_test(w, t, c, &test) {
                break;
            }
            gap.push(c);
        }
        if !found {
            continue;
        }
        for c in gap {
            // spec: S-0406 claim 11 (each filling wall is a building created under S-0202 claim 10: one creation draw)
            let id = crate::objects::create(w, Kind::Building, t, Some(house));
            // spec: S-0406 claim 11 (a filling building passes no further test; it does not fill further)
            put_on_map(w, id, c, false);
        }
    }
}

/// The building an upgrade of type `u` placed at `cell` by `house` goes
/// on (S-0406 claim 12), if there is one with room.
pub fn upgrade_target(w: &World, house: HouseId, u: u32, cell: CellXY) -> Option<ObjectId> {
    let p = place_type(w, u);
    let target = p.upgrade_of?;
    let tower = w.rules().gates.tower == Some(target);
    w.cells
        .list(cell, Layer::Ground)
        .iter()
        .copied()
        .find(|&id| {
            let Some(o) = w.obj(id) else { return false };
            if o.kind != Kind::Building || o.owner != Some(house) || o.type_index != target {
                return false;
            }
            let c = o.econ.building.upgrade_count;
            let cap = place_type(w, target).upgrades;
            match p.level {
                // spec: S-0406 claim 12 (level field absent: the count below the type's upgrade count)
                -1 => c < cap,
                // spec: S-0406 claim 12 (level 1 to 3: the count is 0)
                1..=3 => {
                    // spec: S-0406 Edge cases (quirks.upgrade_level_without_room off: refused when U = 0 and not the wall tower)
                    let no_room =
                        cap == 0 && !tower && !w.rules().quirks.upgrade_level_without_room;
                    c == 0 && !no_room
                }
                // spec: S-0406 claim 12 (any other level value refuses)
                _ => false,
            }
        })
}

/// Adds an upgrade of type `u` to building `b` (S-0406 claim 12, F5).
pub fn add_upgrade(w: &mut World, b: ObjectId, u: u32) {
    let p = place_type(w, u);
    let Some(o) = w.obj(b) else { return };
    let target = o.type_index;
    let tower = w.rules().gates.tower == Some(target);
    let cap = place_type(w, target).upgrades;
    let strength = crate::combat::damage::strength(w, b);
    let owner = o.owner;
    let c = o.econ.building.upgrade_count;
    // spec: S-0406 F5 (i = 1 for an absent level field, else n)
    let i = if p.level == -1 { 1 } else { p.level };
    // spec: S-0406 F5 (c′ = min(c + i, U); uncapped on the wall tower type)
    let c2 = if tower { c + i } else { (c + i).min(cap) };
    if let Some(ob) = w.obj_mut(b) {
        ob.econ.building.upgrade_count = c2;
        // spec: S-0406 claim 12 (full health: the type's strength)
        ob.health = strength;
        // spec: S-0406 F5 (recorded in position c′ when 1 ≤ c′ ≤ 3; otherwise nothing, departure)
        if (1..=3).contains(&c2) {
            ob.econ.building.upgrades[(c2 - 1) as usize] = Some(u);
        }
    }
    if let Some(h) = owner {
        // spec: S-0406 claim 12 (power and the offer take the upgrade into account, S-0405 claim 7, S-0407 claim 12)
        crate::econ::power::make_due(w, h);
        // spec: S-0406 claim 12 and S-0410 claim 2 (a successful upgrade placement is a radar recheck event, whatever it does to power)
        crate::econ::radar::mark_due(w, h);
    }
    // spec: S-0512 claim 2 (c) (a building gaining an upgrade: the grant check at once)
    crate::superweapons::charge::upgrade_gained(w, b);
}

/// Whether a house is a player house (glossary *AI house*: every player
/// house counts as human-controlled in our engine).
pub fn player_house(w: &World, h: HouseId) -> bool {
    w.houses
        .get(usize::from(h.0))
        .is_some_and(|x| x.role == HouseRole::Player && x.human)
}
