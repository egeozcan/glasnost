//! S-0513 revision 126 (Q-3160, #364): a warp landing cell that holds no
//! tile is tested for the wet tile with the no-tile marker 65535 as its tile
//! number, compared with each role range as it stands. It is wet only when a
//! range reaches 65535; an unset role's range, counted from −1, never does.
//! Synthetic rules and tiles (`tests/sw_common`); every number other than the
//! marker is invented.

mod sw_common;

use glasnost_map::terrain::{Roles, UNSET, land};
use glasnost_map::{CellXY, NO_TILE};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{LandType, OverlayType};
use glasnost_sim::world::{ObjectId, World};
use glasnost_sim::{naval, testkit};
use sw_common::*;

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
    // An ordinary overlay of the road land type: on a cell that holds no
    // tile it gives the cell its land type (S-0211 claim 5, row 2).
    r.sim.overlays.insert(
        "PAVE".into(),
        OverlayType {
            land: Some(LandType::Road),
            has_art: Some(true),
            ..Default::default()
        },
    );
    let l = r.registries.entry("overlays".into()).or_default();
    if !l.iter().any(|n| n == "PAVE") {
        l.push("PAVE".into());
    }
    r.globals.quirks.warp_naval_road_cell = road_quirk;
    r.globals.quirks.unset_role_range = unset_range;
    r
}

fn pave(r: &Ruleset) -> u8 {
    let i = r.registries["overlays"]
        .iter()
        .position(|n| n == "PAVE")
        .unwrap();
    u8::try_from(i).unwrap()
}

/// The testkit catalogue with the roles `roles` gives.
fn catalogue(w: &mut World, roles: impl FnOnce(&mut Roles)) {
    let mut cat = (*testkit::catalogue()).clone();
    roles(&mut cat.roles);
    w.catalogue = Some(std::sync::Arc::new(cat));
}

/// Set roles whose ranges all lie far below the no-tile marker.
fn low_roles(r: &mut Roles) {
    r.water = 100;
    r.shore = 200;
    r.waterfalls = [300, 304, 308, 312];
}

/// Gives `c` no tile and the road overlay, so the cell has the overlay's
/// road land type and still holds no tile (S-0211 claim 5, row 2).
fn no_tile_road(w: &mut World, c: CellXY, overlay: u8) {
    let cell = w.cells.grid.get_mut(c).unwrap();
    cell.tile = NO_TILE;
    cell.overlay = overlay;
    glasnost_sim::load::recalc_cells(w, &[c]);
    let cell = w.cells.cell(c).unwrap();
    assert_eq!(cell.land, land::ROAD, "a road cell");
    assert_eq!(cell.tile, NO_TILE, "it holds no tile");
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

#[derive(Debug, PartialEq, Eq)]
enum Fate {
    Survives,
    SinksNoCredit,
    KilledNoCredit,
}

/// The fate of BOAT warped onto a road cell that holds no tile.
fn fate(road_quirk: bool, unset_range: bool, roles: impl FnOnce(&mut Roles)) -> Fate {
    let r = rules(road_quirk, unset_range);
    let mut w = game(&r);
    catalogue(&mut w, roles);
    no_tile_road(&mut w, at(5, 5), pave(&r));
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

// spec: S-0513 claim 38 and Terms (revision 126: a cell that holds no tile has a wet tile only when a range contains the no-tile marker; when none does, a naval member on such a cell given the road land type by an overlay meets the road test under either setting and is killed with no credit)
#[test]
fn no_range_reaches_the_marker_a_no_tile_road_cell_is_a_road_cell_under_both_settings() {
    for quirk in [true, false] {
        assert_eq!(
            fate(quirk, true, low_roles),
            Fate::KilledNoCredit,
            "quirk {quirk}"
        );
    }
}

// spec: S-0513 Terms (revision 126: the no-tile marker 65535 compared with each range as it stands; a role of range count n starting from 65536 − n up to 65535 contains it) and claim 38 (on such a wet tile a non-infantry member sinks with no credit; the road cell is counted with quirks.warp_naval_road_cell on and not with it off)
#[test]
fn a_waterfall_range_reaching_the_marker_makes_the_no_tile_cell_wet() {
    // A waterfall role, range count 4: the first start that reaches 65535 is 65532.
    let reaches = |r: &mut Roles| {
        low_roles(r);
        r.waterfalls[2] = 65532;
    };
    assert_eq!(fate(true, true, reaches), Fate::SinksNoCredit);
    assert_eq!(fate(false, true, reaches), Fate::Survives);
    let short = |r: &mut Roles| {
        low_roles(r);
        r.waterfalls[2] = 65531;
    };
    assert_eq!(fate(true, true, short), Fate::KilledNoCredit);
    assert_eq!(fate(false, true, short), Fate::KilledNoCredit);
}

// spec: S-0513 Terms (revision 126: the water role, range count 14, contains the marker from the start 65522 on, not from 65521)
#[test]
fn the_water_range_reaches_the_marker_from_its_last_start() {
    let reaches = |r: &mut Roles| {
        low_roles(r);
        r.water = 65522;
    };
    assert_eq!(fate(true, true, reaches), Fate::SinksNoCredit);
    let short = |r: &mut Roles| {
        low_roles(r);
        r.water = 65521;
    };
    assert_eq!(fate(true, true, short), Fate::KilledNoCredit);
}

// spec: S-0513 Terms (revision 126: an unset role's range, counted from −1 under quirks.unset_role_range, does not contain the no-tile marker) and S-0010 claim 5
#[test]
fn an_unset_role_range_does_not_reach_the_marker() {
    let unset = |r: &mut Roles| {
        low_roles(r);
        r.shore = UNSET;
        r.water = UNSET;
        r.waterfalls = [UNSET; 4];
    };
    for range in [true, false] {
        assert_eq!(
            fate(true, range, unset),
            Fate::KilledNoCredit,
            "unset_role_range {range}"
        );
        assert_eq!(
            fate(false, range, unset),
            Fate::KilledNoCredit,
            "unset_role_range {range}, road switch off"
        );
    }
}
