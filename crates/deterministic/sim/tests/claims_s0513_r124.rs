//! S-0513 revision 124 (Q-3141, #364): the arrival fate's road cell is read
//! by the landing cell's land type and its sink test by the landing cell's
//! tile (the wet tile of S-0513 Terms); `quirks.warp_naval_road_cell` off
//! stops a road cell on a wet tile from counting as a road cell. Synthetic
//! rules and tiles (`tests/sw_common`); every number is invented.

mod sw_common;

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{LandType, OverlayType};
use glasnost_sim::world::{ObjectId, World};
use glasnost_sim::{naval, testkit};
use sw_common::*;

/// The synthetic water tile appended to the catalogue.
const WET: i32 = 5;

fn rules(road_quirk: bool, unset_range: bool) -> Ruleset {
    let mut r = ruleset();
    r.sim.general.superweapons.chrono_delay = Some(7);
    // A naval type whose locomotion class may enter road land.
    let mut boat = r.sim.vehicles["BUGGY"].clone();
    boat.combat.naval = Some(true);
    r.sim.vehicles.insert("BOAT".into(), boat);
    let l = r.registries.entry("vehicles".into()).or_default();
    if !l.iter().any(|n| n == "BOAT") {
        l.push("BOAT".into());
    }
    // An overlay that gives its cell the road land type while the cell keeps its tile (S-0211 claim 5, row 1).
    r.sim.overlays.insert(
        "ROADOV".into(),
        OverlayType {
            land: Some(LandType::Road),
            no_use_tile_land_type: Some(true),
            has_art: Some(true),
            ..Default::default()
        },
    );
    let l = r.registries.entry("overlays".into()).or_default();
    if !l.iter().any(|n| n == "ROADOV") {
        l.push("ROADOV".into());
    }
    r.globals.quirks.warp_naval_road_cell = road_quirk;
    r.globals.quirks.unset_role_range = unset_range;
    r
}

/// The list index of the road overlay.
fn road_overlay(r: &Ruleset) -> u8 {
    let i = r.registries["overlays"]
        .iter()
        .position(|n| n == "ROADOV")
        .unwrap();
    u8::try_from(i).unwrap()
}

/// The catalogue with a water tile appended (tile 5) and the roles `roles`
/// gives.
fn catalogue(w: &mut World, roles: impl FnOnce(&mut glasnost_map::terrain::Roles)) {
    let mut cat = (*testkit::catalogue()).clone();
    let mut water = cat.tiles[0].clone();
    if let Some(Some(c)) = water.cells.first_mut() {
        c.terrain_code = 9; // the catalogue's water terrain code (as in claims_s0614)
    }
    assert_eq!(cat.tiles.len() as i32, WET);
    cat.tiles.push(water);
    cat.allow_tiberium.push(false);
    roles(&mut cat.roles);
    w.catalogue = Some(std::sync::Arc::new(cat));
}

/// Set roles: the water role starts at the water tile; the shore and the
/// four waterfall roles lie beyond every tile.
fn set_roles(r: &mut glasnost_map::terrain::Roles) {
    r.water = WET;
    r.shore = 100;
    r.waterfalls = [200, 204, 208, 212];
}

/// Gives `c` tile `tile` and the road overlay, which gives it the road land
/// type while the cell keeps its tile (row one of S-0211 claim 5).
fn road_on(w: &mut World, c: CellXY, tile: i32, overlay: u8) {
    let cell = w.cells.grid.get_mut(c).unwrap();
    cell.tile = tile;
    cell.overlay = overlay;
    glasnost_sim::load::recalc_cells(w, &[c]);
    assert_eq!(w.cells.cell(c).unwrap().land, land::ROAD, "a road cell");
}

/// Warps BOAT of BETA from the centre to `at(5, 5)` and runs through its
/// arrival effects; the boat and ALPHA's kills before the warp.
fn warp_boat(w: &mut World) -> (ObjectId, i32) {
    spawn(w, "CHRONOB", ALPHA, -9, -9);
    ready_tick(w, ALPHA, SOURCE, 300);
    let b = spawn(w, "BOAT", BETA, 0, 0);
    let kills = w.houses[0].combat.kills;
    fire_now(w, ALPHA, SOURCE, at(0, 0));
    fire(w, ALPHA, WARP, at(5, 5));
    let tl = w.tick;
    while w.tick <= tl + 64 {
        step(w);
    }
    (b, kills)
}

/// The fate of BOAT warped onto a road cell of tile `tile`.
#[derive(Debug, PartialEq, Eq)]
enum Fate {
    Survives,
    SinksNoCredit,
    KilledNoCredit,
}

fn fate(
    road_quirk: bool,
    unset_range: bool,
    tile: i32,
    roles: impl FnOnce(&mut glasnost_map::terrain::Roles),
) -> Fate {
    let r = rules(road_quirk, unset_range);
    let mut w = game(&r);
    catalogue(&mut w, roles);
    road_on(&mut w, at(5, 5), tile, road_overlay(&r));
    let (b, kills) = warp_boat(&mut w);
    assert_eq!(w.houses[0].combat.kills, kills, "never credited");
    let dead = w.obj(b).is_none_or(|o| o.health <= 0);
    if naval::sinking(&w, b) {
        Fate::SinksNoCredit
    } else if dead {
        Fate::KilledNoCredit
    } else {
        Fate::Survives
    }
}

// spec: S-0513 claim 38 (revision 124: quirks.warp_naval_road_cell on: a road cell is one whose land type is road whatever its tile, so a ship landing on a road cell whose tile is a wet tile sinks with no credit)
#[test]
fn quirk_on_a_ship_on_a_wet_road_cell_sinks_with_no_credit() {
    assert_eq!(fate(true, true, WET, set_roles), Fate::SinksNoCredit);
}

// spec: S-0513 claim 38 and Edge cases (revision 124: quirks.warp_naval_road_cell off: a road cell whose tile is a wet tile is not a road cell here; a ship that can enter it is not killed by the road test)
#[test]
fn quirk_off_a_ship_on_a_wet_road_cell_survives() {
    assert_eq!(fate(false, true, WET, set_roles), Fate::Survives);
}

// spec: S-0513 claim 38 (revision 124: a road cell on a dry tile is a road cell under both settings, and the ship is killed with no credit there)
#[test]
fn both_positions_a_ship_on_a_dry_road_cell_is_killed_with_no_credit() {
    for quirk in [true, false] {
        assert_eq!(
            fate(quirk, true, 0, set_roles),
            Fate::KilledNoCredit,
            "quirk {quirk}"
        );
    }
}

// spec: S-0513 Terms (wet tile: a tile in the range of a waterfall role, 4 tile numbers from its start) and claim 38 (the sink test reads the tile)
#[test]
fn a_waterfall_tile_is_wet() {
    let east = |r: &mut glasnost_map::terrain::Roles| {
        set_roles(r);
        r.waterfalls[0] = 1;
    };
    assert_eq!(fate(true, true, 4, east), Fate::SinksNoCredit);
    let past = |r: &mut glasnost_map::terrain::Roles| {
        set_roles(r);
        r.waterfalls[3] = 0;
    };
    assert_eq!(fate(true, true, 4, past), Fate::KilledNoCredit);
}

// spec: S-0513 Terms (wet tile: an unset shore role takes part counted from the unset start, quirks.unset_role_range) and S-0010 claim 5 (an unchecked role's range from the unset start −1: the shore role covers tiles 0 to 40; off: an unset role passes no tile)
#[test]
fn an_unset_shore_role_counts_from_the_unset_start() {
    let unset = |r: &mut glasnost_map::terrain::Roles| {
        set_roles(r);
        r.shore = glasnost_map::terrain::UNSET;
    };
    assert_eq!(fate(true, true, 0, unset), Fate::SinksNoCredit);
    assert_eq!(fate(true, false, 0, unset), Fate::KilledNoCredit);
}
