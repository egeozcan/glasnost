//! Shared builders for the resource and harvest claim tests (S-0401,
//! S-0402): a synthetic ruleset with invented tiberiums, overlays,
//! refineries and harvesters (no game data), and helpers that put resource
//! on cells. `tests/claims_s0401.rs` and `tests/claims_s0402.rs` include
//! this module with `mod harvest_common;`.

#![allow(dead_code)]

use std::sync::Arc;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    BuildingType, Decimal, LandRow, LandType, LocomotionClass, Locomotor, OverlayType, UnitType,
};
use glasnost_rules::sim_econ::TiberiumType;
use glasnost_sim::rules::{Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, World};

/// Map size.
pub const SIZE: i32 = 30;

/// Overlay list position of the first ORE overlay (its run starts here).
pub const ORE: u8 = 0;
/// Overlay list position of the first GEM overlay.
pub const GEM: u8 = 20;
/// Value per bale of ORE (invented).
pub const ORE_VALUE: i32 = 25;
/// Value per bale of GEM (invented).
pub const GEM_VALUE: i32 = 50;
/// Capacity of HARV (invented).
pub const CAPACITY: i32 = 5;

/// Decimal text.
pub fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

fn push(r: &mut Ruleset, reg: &str, n: &str) {
    r.registries.entry(reg.into()).or_default().push(n.into());
}

/// A ground vehicle type.
pub fn wheel() -> UnitType {
    UnitType {
        strength: Some(400),
        speed: Some(7),
        rot: Some(6),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        ..Default::default()
    }
}

/// A harvester type of capacity `c` docking at REFN and REF2.
pub fn harvester(c: i32) -> UnitType {
    let mut u = wheel();
    u.harvester = Some(true);
    u.storage = Some(c);
    u.econ.dock = Some(vec!["REFN".into(), "REF2".into()]);
    u
}

/// A building type of foundation `w` × `h`.
pub fn building(w: i32, h: i32) -> BuildingType {
    BuildingType {
        strength: Some(500),
        foundation: Some([w, h]),
        ..Default::default()
    }
}

/// The resource and harvest ruleset: the testkit's, plus
/// - land types Clear (buildable), Road and Tiberium;
/// - overlays 0 … 39: ORE's run (0, 12, 8) and GEM's run (20, 12, 8), all
///   with the tiberium flag;
/// - tiberiums ORE (value 25) and GEM (value 50), growth and spread
///   fractions 0 (no growth or spread unless a test sets them);
/// - buildings REFN (refinery, 3 × 3, one dock slot), REF2 (refinery,
///   3 × 3, queueing shift (−2, 1)), PURI (ore purifier, 1 × 1), HUT
///   (1 × 1), MINE terrain object handled by the tests that need one;
/// - vehicles HARV (harvester, capacity 5), CHRONO (a chrono miner),
///   EMPTYH (a harvester of capacity 0), SLAVEM (a slave miner).
pub fn ruleset() -> Ruleset {
    let mut r = testkit::ruleset();
    let s = &mut r.sim;
    s.land_types.insert(
        LandType::Clear,
        LandRow {
            buildable: Some(true),
            ..Default::default()
        },
    );
    s.land_types.insert(LandType::Tiberium, LandRow::default());
    for i in 0..40 {
        let n = format!("OV{i:02}");
        push(&mut r, "overlays", &n);
        r.sim.overlays.insert(
            n,
            OverlayType {
                tiberium: Some(true),
                ..Default::default()
            },
        );
    }
    for (n, v, run) in [
        ("ORE", ORE_VALUE, [0, 12, 8]),
        ("GEM", GEM_VALUE, [20, 12, 8]),
    ] {
        push(&mut r, "tiberiums", n);
        r.sim.tiberiums.insert(
            n.into(),
            TiberiumType {
                value: Some(v),
                growth: Some(100),
                growth_percentage: dec("0"),
                spread: Some(100),
                spread_percentage: dec("0"),
                image_run: Some(run),
                ..Default::default()
            },
        );
    }
    let mut refn = building(3, 3);
    refn.econ.refinery = Some(true);
    let mut ref2 = refn.clone();
    ref2.econ.queueing_cell = Some([-2, 1]);
    let mut puri = building(1, 1);
    puri.econ.ore_purifier = Some(true);
    for (n, b) in [
        ("REFN", refn),
        ("REF2", ref2),
        ("PURI", puri),
        ("HUT", building(1, 1)),
    ] {
        push(&mut r, "buildings", n);
        r.sim.buildings.insert(n.into(), b);
    }
    let mut chrono = harvester(CAPACITY);
    chrono.teleporter = Some(true);
    let mut slave = wheel();
    slave.econ.resource_gatherer = Some(true);
    slave.econ.resource_destination = Some(true);
    slave.econ.enslaves = Some("WALKER".into());
    for (n, u) in [
        ("HARV", harvester(CAPACITY)),
        ("CHRONO", chrono),
        ("EMPTYH", harvester(0)),
        ("SLAVEM", slave),
    ] {
        push(&mut r, "vehicles", n);
        r.sim.vehicles.insert(n.into(), u);
    }
    r
}

/// Sets the growth and spread fractions of a tiberium (index 0: ORE).
pub fn fractions(r: &mut Ruleset, tib: &str, growth: &str, spread: &str) {
    let t = r.sim.tiberiums.get_mut(tib).unwrap();
    t.growth_percentage = dec(growth);
    t.spread_percentage = dec(spread);
}

/// A new game on `r` whose map holds resource cells (cell, overlay, stage)
/// from the start; houses 0 (Alpha) and 1 (Beta), both human players.
pub fn game_with_ore(r: &Ruleset, ore: &[(CellXY, u8, u8)], seed: u32) -> World {
    game_with_map(r, ore, seed, |_| {})
}

/// Like [`game_with_ore`], with a further change `f` to the map.
pub fn game_with_map(
    r: &Ruleset,
    ore: &[(CellXY, u8, u8)],
    seed: u32,
    f: impl FnOnce(&mut glasnost_map::Map),
) -> World {
    let mut map = testkit::empty_map(SIZE);
    for &(c, o, d) in ore {
        let cell = map.cells.get_mut(c).expect("cell on the map");
        cell.overlay = o;
        cell.overlay_data = d;
    }
    f(&mut map);
    let rules = Arc::new(SimRules::new(r, "synthetic-test"));
    glasnost_sim::load::new_game(rules, &map, testkit::catalogue(), &testkit::setup(seed))
        .expect("synthetic test game loads")
}

/// A new game on `r` with no resource.
pub fn game(r: &Ruleset) -> World {
    game_with_ore(r, &[], 7)
}

/// The cell (dx, dy) from the middle of the map.
pub fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

/// Puts resource on a cell during play (overlay `o`, stage `d`) and works
/// out its land type again; the growth and spread lists are not touched.
pub fn put(w: &mut World, c: CellXY, o: u8, d: u8) {
    let cell = w.cells.grid.get_mut(c).expect("cell on the map");
    cell.overlay = o;
    cell.overlay_data = d;
    glasnost_sim::load::recalc_cells(w, &[c]);
    glasnost_sim::regions::cell_changed(w, c);
}

/// The stage of a cell and whether it holds an overlay.
pub fn stage(w: &World, c: CellXY) -> Option<u8> {
    glasnost_sim::econ::resource::resource_at(w, c).map(|(_, d)| d)
}

/// Places a building of type `name` for house `h` with its top-left cell
/// at `c`.
pub fn place_building(w: &mut World, name: &str, h: u8, c: CellXY) -> ObjectId {
    testkit::spawn(w, Kind::Building, name, h, c, 0)
}

/// Places a vehicle of type `name` for house `h` at `c`.
pub fn place_vehicle(w: &mut World, name: &str, h: u8, c: CellXY, facing: u16) -> ObjectId {
    testkit::spawn(w, Kind::Vehicle, name, h, c, facing)
}
