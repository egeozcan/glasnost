//! Claim tests for land types and passability (S-0211), on synthetic data.
//!
//! Not covered here because the simulation does not implement them yet:
//! building placement (the buildable flag and the naval range are read into
//! the rules but nothing places buildings), the events that destroy or
//! repair a bridge, laser-fence states and a house's firestorm.
//! Name parsing of land types, zones and locomotion classes, the misspelling
//! departures and the vehicle's crusher-based default class belong to the
//! importer (`crates/convert`), which hands the simulation typed values.

mod common;

use common::{FOOT, Scene, TRACK, WHEEL, cell, code_tile, objects_of, unit_type};
use glasnost_map::terrain::{TileCell, TileInfo, land};
use glasnost_map::{CellGrid, CellXY, Footprint, NO_OVERLAY, Theater};
use glasnost_num::F32;
use glasnost_rules::sim::{
    BuildingType, Decimal, LandRow, LandType, LocomotionClass, MovementZone, OverlayType,
    TerrainObjectType, UnitType,
};
use glasnost_sim::occupancy::{self, Outcome};
use glasnost_sim::passability::{self, Answer, category};
use glasnost_sim::rules::Kind;
use glasnost_sim::world::World;
use glasnost_sim::{load, objects, regions};

/// Tile land codes of the claim's code table used here: 1 ice, 5 tunnel,
/// 6 railroad, 9 water, 10 beach, 11 road, 14 rough.
const ICE: u8 = 1;
const TUNNEL: u8 = 5;
const RAILROAD: u8 = 6;
const WATER: u8 = 9;
const BEACH: u8 = 10;
const ROAD: u8 = 11;
const ROUGH: u8 = 14;

/// IEEE single-precision bit patterns of the invented test values.
const F_ONE: F32 = F32(0x3F80_0000);
const F_HALF: F32 = F32(0x3F00_0000);
const F_MINUS_QUARTER: F32 = F32(0xBE80_0000);

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.to_string()))
}

/// A land row with every class at `all` except the given overrides.
fn row(all: &str) -> LandRow {
    LandRow {
        foot: dec(all),
        track: dec(all),
        wheel: dec(all),
        hover: dec(all),
        float: dec(all),
        amphibious: dec(all),
        float_beach: dec(all),
        buildable: None,
    }
}

/// Registers an overlay type (with artwork, so loading keeps it) and returns
/// its list index.
fn overlay(s: &mut Scene, name: &str, mut t: OverlayType) -> u8 {
    t.has_art = Some(true);
    s.register("overlays", name);
    s.ruleset.sim.overlays.insert(name.into(), t);
    let i = s.ruleset.registries["overlays"]
        .iter()
        .position(|n| n == name)
        .unwrap();
    u8::try_from(i).unwrap()
}

fn put_overlay(s: &mut Scene, c: CellXY, idx: u8) {
    s.cell_mut(c).overlay = idx;
}

fn land_at(w: &World, c: CellXY) -> u8 {
    w.cells.cell(c).unwrap().land
}

fn cat_at(w: &World, c: CellXY) -> u8 {
    w.cells.category(c).unwrap()
}

fn assert_overlay(w: &World, c: CellXY, idx: u8) {
    assert_eq!(
        w.cells.cell(c).unwrap().overlay,
        idx,
        "overlay kept by load"
    );
}

/// Recomputes a cell after a direct change, as the simulation does after an
/// overlay change (land recompute, then the category).
fn recompute(w: &mut World, c: CellXY) {
    load::recalc_cells(w, &[c]);
    regions::cell_changed(w, c);
}

// spec: S-0211 claim 1 (twelve land types and eight classes, none removed by absent sections)
// spec: S-0211 claim 2 (a land type no layer gave has the start-up row)
#[test]
fn land_table_is_fixed_and_absent_rows_are_startup_rows() {
    let mut s = Scene::new();
    s.ruleset.sim.land_types.clear();
    s.ruleset.sim.land_types.insert(LandType::Weeds, row("0.5"));
    let r = s.sim_rules();
    assert_eq!(r.land.len(), 12);
    for (i, l) in r.land.iter().enumerate() {
        assert_eq!(l.multiplier.len(), 8);
        if i == usize::from(land::WEEDS) {
            continue;
        }
        // Start-up row: every class 0 (winged included), not buildable.
        assert!(l.multiplier.iter().all(|m| *m == F32::ZERO), "land {i}");
        assert!(!l.buildable, "land {i}");
    }
    let weeds = &r.land[usize::from(land::WEEDS)];
    assert_eq!(weeds.multiplier[0], F_HALF);
    // A ruleset with no land sections at all still has all twelve rows.
    s.ruleset.sim.land_types.clear();
    assert_eq!(s.sim_rules().land.len(), 12);
}

// spec: S-0211 claim 2 (cap at 1.0, values not above kept, missing key 1.0, winged fixed, buildable default no)
#[test]
fn land_row_from_its_section() {
    let mut s = Scene::new();
    s.ruleset.sim.land_types.insert(
        LandType::Rough,
        LandRow {
            foot: dec("1.5"),
            track: dec("-0.25"),
            wheel: dec("0"),
            hover: None,
            float: dec("0.5"),
            amphibious: dec("150%"),
            float_beach: dec("50%"),
            buildable: None,
        },
    );
    s.ruleset.sim.land_types.insert(
        LandType::Ice,
        LandRow {
            buildable: Some(true),
            ..LandRow::default()
        },
    );
    let r = s.sim_rules();
    use LocomotionClass as C;
    let m = |c| r.multiplier(land::ROUGH, c);
    assert_eq!(m(C::Foot), F_ONE, "capped at 1.0");
    assert_eq!(m(C::Track), F_MINUS_QUARTER, "negative kept");
    assert_eq!(m(C::Wheel), F32::ZERO, "zero kept");
    assert_eq!(m(C::Hover), F_ONE, "missing key gives 1.0");
    assert_eq!(m(C::Winged), F_ONE, "winged fixed at 1.0");
    assert_eq!(m(C::Float), F_HALF);
    assert_eq!(m(C::Amphibious), F_ONE, "percent value capped");
    assert_eq!(m(C::FloatBeach), F_HALF, "percent value");
    assert!(!r.land[usize::from(land::ROUGH)].buildable);
    let ice = &r.land[usize::from(land::ICE)];
    assert!(ice.buildable);
    assert!(ice.multiplier.iter().all(|x| *x == F_ONE));
}

// spec: S-0211 claim 3 (infantry default foot, aircraft default winged, a named class is kept)
#[test]
fn locomotion_class_defaults() {
    let mut s = Scene::new();
    s.infantry("PLAINMAN", UnitType::default());
    s.infantry("ROLLMAN", unit_type(WHEEL, 4, 0));
    s.register("aircraft", "PLAINJET");
    s.ruleset
        .sim
        .aircraft
        .insert("PLAINJET".into(), UnitType::default());
    s.vehicle("TANKY", unit_type(TRACK, 4, 0));
    let r = s.sim_rules();
    let class = |k, n| r.unit(k, r.index_of(k, n).unwrap()).unwrap().class;
    assert_eq!(class(Kind::Infantry, "PLAINMAN"), Some(FOOT));
    assert_eq!(class(Kind::Infantry, "ROLLMAN"), Some(WHEEL));
    assert_eq!(
        class(Kind::Aircraft, "PLAINJET"),
        Some(LocomotionClass::Winged)
    );
    assert_eq!(class(Kind::Vehicle, "TANKY"), Some(TRACK));
}

// spec: S-0211 claim 4 (absent overlay land type reads as clear, ore with clear becomes tiberium, absent restriction is none)
#[test]
fn absent_land_names_and_restriction() {
    let mut s = Scene::new();
    s.ruleset
        .sim
        .land_types
        .insert(LandType::Tiberium, row("1"));
    let plain = overlay(&mut s, "PLAINBITS", OverlayType::default());
    let ore = overlay(
        &mut s,
        "SHINYORE",
        OverlayType {
            tiberium: Some(true),
            ..Default::default()
        },
    );
    let rocky_ore = overlay(
        &mut s,
        "ROCKYORE",
        OverlayType {
            tiberium: Some(true),
            land: Some(LandType::Rock),
            ..Default::default()
        },
    );
    s.vehicle("FREECAR", unit_type(WHEEL, 4, 0));
    let r = s.sim_rules();
    assert_eq!(r.overlays[usize::from(plain)].land, land::CLEAR);
    assert_eq!(r.overlays[usize::from(ore)].land, land::TIBERIUM);
    assert_eq!(r.overlays[usize::from(rocky_ore)].land, land::ROCK);
    let car = r.index_of(Kind::Vehicle, "FREECAR").unwrap();
    assert_eq!(r.unit(Kind::Vehicle, car).unwrap().restricted_to, None);

    // On the map: the ore cell has land tiberium; no restriction allows any land.
    put_overlay(&mut s, cell(0, 0), ore);
    s.set_cell(cell(1, 0), code_tile(WATER), 0);
    let w = s.build();
    assert_overlay(&w, cell(0, 0), ore);
    assert_eq!(land_at(&w, cell(0, 0)), land::TIBERIUM);
    for c in [cell(0, 0), cell(1, 0), cell(2, 0)] {
        assert!(passability::restricted_allows(&w, c, None, false));
    }
}

/// A scene with a water row that only float classes may enter.
fn water_scene() -> Scene {
    let mut s = Scene::new();
    let mut water = row("0");
    water.float = dec("1");
    water.amphibious = dec("1");
    s.ruleset.sim.land_types.insert(LandType::Water, water);
    s
}

// spec: S-0211 claim 9 (a low bridge with the own-land flag gives the overlay's land; removed, the tile's land returns)
#[test]
fn low_bridge_cell_takes_overlay_land() {
    let mut s = water_scene();
    let low = overlay(
        &mut s,
        "PLANKBRIDGE",
        OverlayType {
            land: Some(LandType::Road),
            no_use_tile_land_type: Some(true),
            ..Default::default()
        },
    );
    let c = cell(0, 0);
    s.set_cell(c, code_tile(WATER), 0);
    put_overlay(&mut s, c, low);
    let mut w = s.build();
    assert_overlay(&w, c, low);
    assert_eq!(land_at(&w, c), land::ROAD);
    let r = w.rules_arc();
    assert!(passability::enterable_by_class(
        &r,
        &w,
        c,
        Some(WHEEL),
        MovementZone::Normal,
        false
    ));
    // The bridge overlay goes: the recompute gives the tile's land again.
    w.cells.grid.get_mut(c).unwrap().overlay = NO_OVERLAY;
    recompute(&mut w, c);
    assert_eq!(land_at(&w, c), land::WATER);
    assert_eq!(cat_at(&w, c), category::WATER);
    assert!(!passability::enterable_by_class(
        &r,
        &w,
        c,
        Some(WHEEL),
        MovementZone::Normal,
        false
    ));
}

// spec: S-0211 claim 9 (a high bridge without row-one overlay properties keeps its ground's land; deck objects ignore it)
#[test]
fn high_bridge_cell_keeps_ground_land() {
    let mut s = water_scene();
    let high = overlay(
        &mut s,
        "SKYBRIDGE",
        OverlayType {
            land: Some(LandType::Road),
            ..Default::default()
        },
    );
    let c = cell(0, 0);
    s.set_cell(c, code_tile(WATER), 0);
    put_overlay(&mut s, c, high);
    let w = s.build();
    assert_overlay(&w, c, high);
    assert_eq!(land_at(&w, c), land::WATER);
    let r = w.rules_arc();
    let enter = |class, deck| {
        passability::enterable_by_class(&r, &w, c, Some(class), MovementZone::Normal, deck)
    };
    // Under the deck: only a class that may enter water.
    assert!(enter(LocomotionClass::Float, false));
    assert!(!enter(WHEEL, false));
    // On the deck: the cell's land type does not apply.
    assert!(enter(WHEEL, true));
    assert!(enter(FOOT, true));
}

// spec: S-0211 claim 10 (otherwise case: entry when M is nonzero, tiny and negative values included; winged always)
#[test]
fn entry_by_class_follows_the_multiplier() {
    let mut s = Scene::new();
    let mut rough = row("1");
    rough.wheel = dec("0");
    rough.track = dec("-0.5");
    rough.foot = dec("1e-30");
    rough.hover = dec("0");
    s.ruleset.sim.land_types.insert(LandType::Rough, rough);
    let c = cell(1, 0);
    s.set_cell(c, code_tile(ROUGH), 0);
    // A cell of a land type no rules layer gave (start-up row: all zero).
    let bare = cell(-1, 0);
    s.set_cell(bare, code_tile(RAILROAD), 0);
    s.vehicle("ROLLER", unit_type(WHEEL, 4, 0));
    s.vehicle("CRAWLER", unit_type(TRACK, 4, 0));
    s.place_unit(0, "ROLLER", cell(0, 0), 0);
    s.place_unit(0, "CRAWLER", cell(0, 1), 0);
    let w = s.build();
    assert_eq!(land_at(&w, c), land::ROUGH);
    assert_eq!(land_at(&w, bare), land::RAILROAD);
    let r = w.rules_arc();
    let enter = |class, at| {
        passability::enterable_by_class(&r, &w, at, Some(class), MovementZone::Normal, false)
    };
    assert!(!enter(WHEEL, c), "zero refuses");
    assert!(enter(TRACK, c), "negative allows");
    assert!(enter(FOOT, c), "tiny allows");
    assert!(!enter(LocomotionClass::Hover, c));
    assert!(!enter(TRACK, bare));
    // Winged: always, even where its own multiplier is the start-up 0.
    assert_eq!(
        r.multiplier(land::RAILROAD, LocomotionClass::Winged),
        F32::ZERO
    );
    assert!(enter(LocomotionClass::Winged, bare));

    // The same rule decides a vehicle's entry test.
    let ids = objects_of(&w, Kind::Vehicle);
    let outcome =
        |id, at| occupancy::vehicle_outcome(&w, id, at, occupancy::reference_level(&w, id));
    let (roller, crawler) = (ids[0], ids[1]);
    assert_eq!(outcome(roller, c), Outcome::Blocked);
    assert_ne!(outcome(crawler, c), Outcome::Blocked);
}

// spec: S-0211 claim 10 (wall cells: destroyer zones and crusher-all, crushers only for crushable walls, then as clear; winged first)
#[test]
fn entry_into_wall_cells_by_zone() {
    let mut s = Scene::new();
    let mut clear = row("1");
    clear.hover = dec("0");
    s.ruleset.sim.land_types.insert(LandType::Clear, clear);
    let wall = overlay(
        &mut s,
        "STONEWALL",
        OverlayType {
            land: Some(LandType::Wall),
            wall: Some(true),
            ..Default::default()
        },
    );
    let soft = overlay(
        &mut s,
        "HEDGEWALL",
        OverlayType {
            land: Some(LandType::Wall),
            wall: Some(true),
            crushable: Some(true),
            ..Default::default()
        },
    );
    let (a, b) = (cell(0, 0), cell(2, 0));
    put_overlay(&mut s, a, wall);
    put_overlay(&mut s, b, soft);
    let w = s.build();
    assert_overlay(&w, a, wall);
    assert_overlay(&w, b, soft);
    assert_eq!(land_at(&w, a), land::WALL);
    let r = w.rules_arc();
    let enter =
        |at, class, zone| passability::enterable_by_class(&r, &w, at, Some(class), zone, false);
    use MovementZone as Z;
    for z in [
        Z::Destroyer,
        Z::AmphibiousDestroyer,
        Z::InfantryDestroyer,
        Z::CrusherAll,
    ] {
        assert!(enter(a, TRACK, z), "{z:?} hard wall");
        assert!(enter(b, TRACK, z), "{z:?} crushable wall");
        // Decided as for clear: M(clear, hover) = 0 refuses.
        assert!(!enter(a, LocomotionClass::Hover, z), "{z:?} hover");
    }
    for z in [Z::Crusher, Z::AmphibiousCrusher] {
        assert!(!enter(a, TRACK, z), "{z:?} hard wall");
        assert!(enter(b, TRACK, z), "{z:?} crushable wall");
    }
    for z in [
        Z::Normal,
        Z::Amphibious,
        Z::Subterranean,
        Z::Infantry,
        Z::Fly,
        Z::Water,
        Z::WaterBeach,
    ] {
        assert!(!enter(a, TRACK, z), "{z:?} hard wall");
        assert!(!enter(b, TRACK, z), "{z:?} crushable wall");
    }
    // Winged comes first, whatever the zone.
    assert!(enter(a, LocomotionClass::Winged, Z::Normal));
}

// spec: S-0211 claim 11 (rule 2: crushable, wall, overlay land with zero wheel, rock and rubble, in that order)
#[test]
fn category_rule_two_overlay_flags() {
    let mut s = water_scene();
    let flags = |crushable, wall, rock, rubble, l: Option<LandType>| OverlayType {
        crushable: Some(crushable),
        wall: Some(wall),
        rock: Some(rock),
        rubble: Some(rubble),
        land: l,
        ..Default::default()
    };
    let cases = [
        (flags(true, true, false, false, None), category::CRUSHABLE),
        (flags(false, true, false, false, None), category::WALL_LIKE),
        // Overlay land rock has no row: M(rock, wheel) = 0.
        (
            flags(false, false, false, false, Some(LandType::Rock)),
            category::BLOCKED,
        ),
        (flags(false, false, true, false, None), category::BLOCKED),
        (
            flags(false, false, false, true, Some(LandType::Rock)),
            category::BLOCKED,
        ),
    ];
    let mut placed = Vec::new();
    for (i, (t, want)) in cases.into_iter().enumerate() {
        let idx = overlay(&mut s, &format!("BITS{i}"), t);
        let c = cell(2 * i as i32 - 6, 0);
        put_overlay(&mut s, c, idx);
        placed.push((c, idx, want));
    }
    // Rubble on water: open.
    let rubble = overlay(&mut s, "RUBBLE", flags(false, false, false, true, None));
    let wet = cell(0, 4);
    s.set_cell(wet, code_tile(WATER), 0);
    put_overlay(&mut s, wet, rubble);
    let w = s.build();
    for (c, idx, want) in placed {
        assert_overlay(&w, c, idx);
        assert_eq!(cat_at(&w, c), want, "overlay {idx}");
    }
    // The rock-land overlay without row-one flags leaves the tile's land.
    assert_eq!(land_at(&w, cell(-2, 0)), land::CLEAR);
    assert_overlay(&w, wet, rubble);
    assert_eq!(land_at(&w, wet), land::WATER);
    assert_eq!(cat_at(&w, wet), category::OPEN);
}

// spec: S-0211 claim 11 (rule 3: water, beach, and the wheel column at or below 0.01)
#[test]
fn category_rule_three_land() {
    let mut s = water_scene();
    let mut rough = row("1");
    rough.wheel = dec("0.01");
    s.ruleset.sim.land_types.insert(LandType::Rough, rough);
    let mut ice = row("1");
    ice.wheel = dec("0.0101");
    s.ruleset.sim.land_types.insert(LandType::Ice, ice);
    let mut rail = row("1");
    rail.wheel = dec("0");
    s.ruleset.sim.land_types.insert(LandType::Railroad, rail);
    let cases = [
        (WATER, category::WATER),
        (BEACH, category::BEACH),
        (ROUGH, category::BLOCKED),
        (ICE, category::OPEN),
        (RAILROAD, category::BLOCKED),
        (ROAD, category::OPEN),
    ];
    for (i, (code, _)) in cases.iter().enumerate() {
        s.set_cell(cell(2 * i as i32 - 6, 0), code_tile(*code), 0);
    }
    let w = s.build();
    for (i, (code, want)) in cases.iter().enumerate() {
        assert_eq!(cat_at(&w, cell(2 * i as i32 - 6, 0)), *want, "code {code}");
    }
    // A tracked type may enter railroad (claim 10) although its category is blocked.
    let r = w.rules_arc();
    let rail_cell = cell(2, 0);
    assert!(passability::enterable_by_class(
        &r,
        &w,
        rail_cell,
        Some(TRACK),
        MovementZone::Normal,
        false
    ));
    assert_eq!(
        passability::zone_table(MovementZone::Normal, cat_at(&w, rail_cell)),
        Answer::No
    );
}

// spec: S-0211 claim 11 (`quirks.passability_wheel_column` off: rule 3 blocks only when every ground class is low)
#[test]
fn category_rule_three_every_class_with_quirk_off() {
    let mut s = Scene::new();
    s.ruleset.globals.quirks.passability_wheel_column = false;
    let mut rough = row("1");
    rough.wheel = dec("0");
    s.ruleset.sim.land_types.insert(LandType::Rough, rough);
    s.ruleset.sim.land_types.insert(LandType::Ice, row("0.01"));
    s.set_cell(cell(0, 0), code_tile(ROUGH), 0);
    s.set_cell(cell(2, 0), code_tile(ICE), 0);
    let w = s.build();
    assert_eq!(cat_at(&w, cell(0, 0)), category::OPEN);
    assert_eq!(cat_at(&w, cell(2, 0)), category::BLOCKED);
}

// spec: S-0211 claim 11 (`quirks.passability_wheel_column` off: rule 2 also judges every ground class)
#[test]

fn category_rule_two_every_class_with_quirk_off() {
    let mut s = Scene::new();
    s.ruleset.globals.quirks.passability_wheel_column = false;
    let mut rough = row("1");
    rough.wheel = dec("0");
    s.ruleset.sim.land_types.insert(LandType::Rough, rough);
    let idx = overlay(
        &mut s,
        "ROUGHBITS",
        OverlayType {
            land: Some(LandType::Rough),
            ..Default::default()
        },
    );
    let c = cell(0, 0);
    put_overlay(&mut s, c, idx);
    let w = s.build();
    assert_overlay(&w, c, idx);
    // The cell's own land is clear (row 5), so rules 3 to 5 give open.
    assert_eq!(land_at(&w, c), land::CLEAR);
    assert_eq!(cat_at(&w, c), category::OPEN);
}

fn terrain_type(s: &mut Scene, name: &str, temperate: i32, snow: i32) {
    s.register("terrain_objects", name);
    s.ruleset.sim.terrain_objects.insert(
        name.into(),
        TerrainObjectType {
            temperate_occupation_bits: Some(temperate),
            snow_occupation_bits: Some(snow),
            ..Default::default()
        },
    );
}

fn building_type(s: &mut Scene, name: &str, t: BuildingType) {
    s.register("buildings", name);
    s.ruleset.sim.buildings.insert(
        name.into(),
        BuildingType {
            strength: Some(100),
            foundation: Some([1, 1]),
            ..t
        },
    );
}

/// A scene with terrain objects of pattern 7 and 3 and three kinds of
/// building, each in its own cell.
fn objects_scene(theater: Theater) -> Scene {
    let mut s = Scene::new();
    s.map.theater = theater;
    terrain_type(&mut s, "BIGTREE", 7, 3);
    terrain_type(&mut s, "SHRUB", 3, 7);
    building_type(&mut s, "SHED", BuildingType::default());
    building_type(
        &mut s,
        "ZAPFENCE",
        BuildingType {
            laser_fence: Some(true),
            ..Default::default()
        },
    );
    building_type(
        &mut s,
        "FIREWALL",
        BuildingType {
            firestorm_wall: Some(true),
            ..Default::default()
        },
    );
    s.place_terrain("BIGTREE", cell(-4, 0));
    s.place_terrain("SHRUB", cell(-2, 0));
    s.place_structure(0, "SHED", cell(0, 0));
    s.place_structure(0, "ZAPFENCE", cell(2, 0));
    s.place_structure(1, "FIREWALL", cell(4, 0));
    s
}

// spec: S-0211 claim 11 (rule 4: terrain patterns by theater; buildings, fences and firestorm walls with it off leave open)
#[test]
fn category_rule_four_objects() {
    let w = objects_scene(Theater::Temperate).build();
    assert_eq!(objects_of(&w, Kind::Terrain).len(), 2);
    assert_eq!(objects_of(&w, Kind::Building).len(), 3);
    assert_eq!(cat_at(&w, cell(-4, 0)), category::WALL_LIKE);
    assert_eq!(cat_at(&w, cell(-2, 0)), category::PARTIAL);
    for dx in [0, 2, 4] {
        assert_eq!(cat_at(&w, cell(dx, 0)), category::OPEN, "building at {dx}");
    }
    // The snow theater reads the snow pattern.
    let w = objects_scene(Theater::Snow).build();
    assert_eq!(cat_at(&w, cell(-4, 0)), category::PARTIAL);
    assert_eq!(cat_at(&w, cell(-2, 0)), category::WALL_LIKE);
}

// spec: S-0211 claim 11 (recomputed from the cell after the change; unit moves never change a category)
#[test]
fn category_follows_changes_but_not_units() {
    let mut s = objects_scene(Theater::Temperate);
    s.vehicle("CRAWLER", unit_type(TRACK, 30, 0));
    s.infantry("WALKER", unit_type(FOOT, 4, 0));
    s.place_unit(0, "CRAWLER", cell(-2, 3), 0);
    s.place_infantry(0, "WALKER", cell(0, 6), 1);
    let mut w = s.build();
    let before: Vec<u8> = w.cells.category.clone();
    let was = |w: &World, c: CellXY| before[w.cells.idx(c).unwrap()];
    common::open_orders(&mut w);
    let car = objects_of(&w, Kind::Vehicle)[0];
    let man = objects_of(&w, Kind::Infantry)[0];
    let start = w.obj(car).unwrap().pos.cell();
    common::order_move(&mut w, car, cell(4, 3));
    common::order_move(&mut w, man, cell(3, 6));
    let mut moved = false;
    for _ in 0..300 {
        glasnost_sim::tick::step(&mut w);
        for id in [car, man] {
            let here = w.obj(id).unwrap().pos.cell();
            // The category computed with the unit in the cell is the one
            // the cell had before any unit came near it.
            assert_eq!(
                passability::compute_category(&w, here),
                was(&w, here),
                "tick {}",
                w.tick
            );
        }
        moved |= w.obj(car).unwrap().pos.cell() != start;
    }
    assert!(moved, "the vehicle moved");
    assert_eq!(w.cells.category, before);
    // A terrain object taken out: its cell is recomputed and opens.
    let tree = objects_of(&w, Kind::Terrain)
        .into_iter()
        .find(|&t| w.obj(t).unwrap().pos.cell() == cell(-4, 0))
        .unwrap();
    objects::remove(&mut w, tree);
    assert_eq!(cat_at(&w, cell(-4, 0)), category::OPEN);
}

// spec: S-0211 claim 11 (rule 1: a cell outside the usable area is off-map)
#[test]
fn category_rule_one_off_map() {
    // The same map with an invented smaller local area, so that some legal
    // cells lie outside the usable area.
    let mut s = Scene::new();
    let size = common::SIZE;
    let fp = Footprint::new(size, size, [4, 4, size - 8, size - 8]);
    let mut grid = CellGrid::new(fp);
    let legal: Vec<CellXY> = grid.iter().map(|(c, _)| c).collect();
    for c in legal {
        *grid.get_mut(c).unwrap() = *s.map.cells.get(c).unwrap();
    }
    s.map.footprint = fp;
    s.map.cells = grid;
    let w = s.build();
    let outside: Vec<CellXY> = w
        .cells
        .grid
        .iter()
        .map(|(c, _)| c)
        .filter(|&c| !w.cells.usable(c))
        .collect();
    assert!(
        !outside.is_empty(),
        "the scene has cells outside the usable area"
    );
    for c in outside {
        assert_eq!(cat_at(&w, c), category::OFF_MAP);
        assert_eq!(
            passability::zone_table(MovementZone::Fly, cat_at(&w, c)),
            Answer::Never
        );
    }
    assert_eq!(cat_at(&w, cell(0, 0)), category::OPEN);
}

/// A tile of `w` × `h` cells, every cell with terrain code `code`.
fn block_tile(w: u8, h: u8, code: u8) -> TileInfo {
    TileInfo {
        width: w,
        height: h,
        has_image: true,
        cells: (0..usize::from(w) * usize::from(h))
            .map(|_| {
                Some(TileCell {
                    terrain_code: code,
                    slope: 0,
                })
            })
            .collect(),
        variants: 0,
    }
}

// spec: S-0211 claim 14 (the restricted land itself, tunnel cells by tile size and sub-tile, deck overlays 237 and 238)
#[test]
fn restricted_land_type_cells() {
    let mut s = Scene::new();
    s.ruleset.sim.land_types.insert(LandType::Tunnel, row("1"));
    let base = s.catalogue.tiles.len() as i32;
    for (w, h) in [(3, 4), (3, 5), (4, 3), (5, 3), (2, 2)] {
        s.catalogue.tiles.push(block_tile(w, h, TUNNEL));
    }
    let road = cell(0, 0);
    s.set_cell(road, code_tile(ROAD), 0);
    // (cell, tile, sub-tile, allowed)
    let tunnels = [
        (cell(-8, 4), base, 6, true),
        (cell(-6, 4), base, 5, false),
        (cell(-4, 4), base + 1, 6, true),
        (cell(-2, 4), base + 1, 2, false),
        (cell(0, 4), base + 2, 2, true),
        (cell(2, 4), base + 2, 6, false),
        (cell(4, 4), base + 3, 2, true),
        (cell(6, 4), base + 3, 0, false),
        (cell(8, 4), base + 4, 3, true),
        (cell(8, 6), code_tile(TUNNEL), 0, true),
    ];
    for &(c, tile, sub, _) in &tunnels {
        s.set_cell(c, tile, 0);
        s.cell_mut(c).sub_tile = sub;
    }
    let mut w = s.build();
    let want = Some(land::ROAD);
    assert_eq!(land_at(&w, road), land::ROAD);
    assert!(passability::restricted_allows(&w, road, want, false));
    assert!(!passability::restricted_allows(&w, cell(0, 2), want, false));
    for &(c, tile, sub, ok) in &tunnels {
        assert_eq!(land_at(&w, c), land::TUNNEL, "tile {tile}");
        assert_eq!(
            passability::restricted_allows(&w, c, want, false),
            ok,
            "tile {tile} sub-tile {sub}"
        );
    }
    // Overlay indices 237 and 238: only for a vehicle above the ground.
    let (d1, d2, d3) = (cell(0, -4), cell(2, -4), cell(4, -4));
    for (c, idx) in [(d1, 237), (d2, 238), (d3, 236)] {
        w.cells.grid.get_mut(c).unwrap().overlay = idx;
    }
    assert!(passability::restricted_allows(&w, d1, want, true));
    assert!(passability::restricted_allows(&w, d2, want, true));
    assert!(!passability::restricted_allows(&w, d1, want, false));
    assert!(!passability::restricted_allows(&w, d3, want, true));
    // A deck cell without those overlays does not qualify by default.
    let deck = cell(6, -4);
    let i = w.cells.idx(deck).unwrap();
    w.cells.deck[i] = true;
    assert!(!passability::restricted_allows(&w, deck, want, true));
}

// spec: S-0211 claim 14 (`quirks.restricted_deck_overlay_indices` off: any high-bridge deck qualifies)
#[test]
fn restricted_deck_exception_with_quirk_off() {
    let mut s = Scene::new();
    s.ruleset.globals.quirks.restricted_deck_overlay_indices = false;
    let mut w = s.build();
    let want = Some(land::ROAD);
    let (deck, plain) = (cell(0, 0), cell(2, 0));
    let i = w.cells.idx(deck).unwrap();
    w.cells.deck[i] = true;
    w.cells.grid.get_mut(plain).unwrap().overlay = 237;
    assert!(passability::restricted_allows(&w, deck, want, true));
    assert!(!passability::restricted_allows(&w, deck, want, false));
    assert!(!passability::restricted_allows(&w, plain, want, true));
}

// spec: S-0211 claim 14 (a restricted vehicle counts other land as impassable)
#[test]
fn restricted_vehicle_entry_test() {
    let mut s = Scene::new();
    for dx in -2..=2 {
        s.set_cell(cell(dx, 0), code_tile(ROAD), 0);
    }
    s.vehicle(
        "RAILCART",
        UnitType {
            movement_restricted_to: Some(LandType::Road),
            ..unit_type(WHEEL, 4, 0)
        },
    );
    s.vehicle("FREECAR", unit_type(WHEEL, 4, 0));
    s.place_unit(0, "RAILCART", cell(0, 0), 0);
    s.place_unit(0, "FREECAR", cell(0, -3), 0);
    let w = s.build();
    let ids = objects_of(&w, Kind::Vehicle);
    let (id, free) = (ids[0], ids[1]);
    // Control: an unrestricted vehicle may enter the clear cell.
    assert_eq!(
        occupancy::vehicle_outcome(&w, free, cell(0, 1), occupancy::reference_level(&w, free)),
        Outcome::Clear
    );
    let reference = occupancy::reference_level(&w, id);
    assert_eq!(
        occupancy::vehicle_outcome(&w, id, cell(1, 0), reference),
        Outcome::Clear
    );
    assert_eq!(
        occupancy::vehicle_outcome(&w, id, cell(0, 1), reference),
        Outcome::Blocked
    );
}

// ---------------------------------------------------------------------------
// High-bridge cells (claim 15) and walls under a deck (claim 10)

/// Registers invented overlay types up to list index `last` and returns
/// the scene.
fn many_overlays(last: usize) -> Scene {
    let mut s = Scene::new();
    for i in 0..=last {
        overlay(&mut s, &format!("OV{i}"), OverlayType::default());
    }
    s
}

fn deck_at(w: &World, c: CellXY) -> bool {
    w.cells.deck[w.cells.idx(c).expect("cell")]
}

// spec: S-0211 claim 15 (indices 24 and 25 mark four cells along map north and map west; marking looks at nothing in the marked cells)
// spec: S-0211 Constants (high-bridge overlay indices and marking directions)
#[test]
fn high_bridge_overlays_mark_four_cells() {
    let mut s = many_overlays(25);
    let (a, b) = (cell(0, 0), cell(10, 0));
    put_overlay(&mut s, a, 24);
    put_overlay(&mut s, b, 25);
    let w = s.build();
    for (dx, dy) in [(0, 1), (0, 0), (0, -1), (0, -2)] {
        assert!(deck_at(&w, cell(dx, dy)), "index 24 marks ({dx}, {dy})");
    }
    assert!(!deck_at(&w, cell(0, 2)));
    assert!(!deck_at(&w, cell(0, -3)));
    for (dx, dy) in [(1, 0), (0, 0), (-1, 0), (-2, 0)] {
        assert!(
            deck_at(&w, cell(10 + dx, dy)),
            "index 25 marks ({dx}, {dy})"
        );
    }
    assert!(!deck_at(&w, cell(12, 0)));
    assert!(!deck_at(&w, cell(5, 5)));
}

// spec: S-0211 claim 15 (c) (indices 237 and 238 mark the same cells as 24 and 25)
#[test]
fn other_high_bridge_indices_mark_the_same_cells() {
    let mut w = Scene::new().build();
    let c = cell(0, 0);
    w.cells.mark_high_bridge(c, 237);
    w.cells.mark_high_bridge(cell(10, 0), 238);
    w.cells.mark_high_bridge(cell(20, 0), 23);
    for k in [-1, 0, 1, 2] {
        assert!(deck_at(&w, cell(0, -k)));
        assert!(deck_at(&w, cell(10 - k, 0)));
    }
    assert!(!deck_at(&w, cell(20, 0)), "another index marks nothing");
}

// spec: S-0211 claim 10 (ii) and (iii) (on a deck: any land type, but a wall overlay still refuses the zones (iii) does not list)
// spec: S-0211 Edge cases (a wall overlay under a bridge deck)
#[test]
fn a_wall_under_a_deck_refuses_other_zones() {
    let mut s = Scene::new();
    let wall = overlay(
        &mut s,
        "DECKWALL",
        OverlayType {
            land: Some(LandType::Wall),
            wall: Some(true),
            crushable: Some(true),
            ..Default::default()
        },
    );
    let c = cell(0, 0);
    s.set_cell(c, code_tile(WATER), 0);
    put_overlay(&mut s, c, wall);
    let w = s.build();
    let r = w.rules_arc();
    let enter = |zone, deck| passability::enterable_by_class(&r, &w, c, Some(WHEEL), zone, deck);
    assert!(
        !enter(MovementZone::Normal, true),
        "normal is refused on the deck too"
    );
    assert!(
        enter(MovementZone::Crusher, true),
        "a crusher of a crushable wall enters the deck"
    );
    // A water cell without a wall: the deck admits any class.
    let open = cell(2, 0);
    let mut s2 = Scene::new();
    s2.set_cell(open, code_tile(WATER), 0);
    let w2 = s2.build();
    let r2 = w2.rules_arc();
    assert!(passability::enterable_by_class(
        &r2,
        &w2,
        open,
        Some(WHEEL),
        MovementZone::Normal,
        true
    ));
    assert!(!passability::enterable_by_class(
        &r2,
        &w2,
        open,
        Some(WHEEL),
        MovementZone::Normal,
        false
    ));
}
