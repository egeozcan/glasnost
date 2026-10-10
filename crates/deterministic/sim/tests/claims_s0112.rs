//! Claim tests for S-0112 claim 7 on synthetic data: wall connection bits
//! in the overlay data value, from the map's overlay list and recomputed by
//! gate and wall tower placements.

mod common;

use common::{Scene, cell};
use glasnost_rules::sim::{BuildingType, OverlayType};

/// A scene whose overlay list has types W0 … W29: positions 0 and 26 are
/// walls, the others plain overlays; and building types GATEA (a first
/// Allied gate, 3 × 1) and TOWER (the wall tower, 1 × 1).
fn wall_scene() -> Scene {
    let mut s = Scene::new();
    for i in 0..30 {
        let name = format!("W{i}");
        s.register("overlays", &name);
        s.ruleset.sim.overlays.insert(
            name,
            OverlayType {
                has_art: Some(true),
                wall: Some(i == 0 || i == 26),
                ..Default::default()
            },
        );
    }
    for (name, f) in [("GATEA", [3, 1]), ("TOWER", [1, 1])] {
        s.building(
            name,
            BuildingType {
                strength: Some(100),
                foundation: Some(f),
                ..Default::default()
            },
        );
    }
    s.ruleset.sim.general.allied_gate_one = Some("GATEA".into());
    s.ruleset.sim.general.wall_tower = Some("TOWER".into());
    s
}

fn data(w: &glasnost_sim::world::World, dx: i32, dy: i32) -> u8 {
    w.cells.cell(cell(dx, dy)).unwrap().overlay_data
}

/// spec: S-0112 claim 7 (bits for neighbours of the same overlay type, bit k for step direction 2k; walls of different types never join; without an overlay data stream the bits of placement stay)
#[test]
fn claim_7_same_type_bits() {
    let mut s = wall_scene();
    s.map.has_overlay_data = false;
    for dx in 0..3 {
        s.cell_mut(cell(dx, 0)).overlay = 0;
    }
    s.cell_mut(cell(1, 1)).overlay = 26;
    s.cell_mut(cell(0, -1)).overlay = 0;
    let w = s.build();
    // (0, 0): north (1) and east (2); (1, 0): east and west (2 + 8), not south (other type); (2, 0): west.
    assert_eq!(data(&w, 0, 0), 1 + 2);
    assert_eq!(data(&w, 1, 0), 2 + 8);
    assert_eq!(data(&w, 2, 0), 8);
    assert_eq!(data(&w, 0, -1), 4);
    assert_eq!(data(&w, 1, 1), 0);
    // With the stream, its values are final (S-0009 claim 13).
    s.map.has_overlay_data = true;
    let w = s.build();
    assert_eq!(data(&w, 1, 0), 0);
}

/// spec: S-0112 claim 7 (a placed first gate recomputes (x − 1, y) and (x + 3, y) with their edge neighbours; a wall tower every cell within two edge steps; the stage is kept)
#[test]
fn claim_7_gate_and_tower() {
    let mut s = wall_scene();
    // Walls at both ends of a gate three cells long at (0, 0) … (2, 0); one of them at stage 1.
    s.cell_mut(cell(-1, 0)).overlay = 0;
    s.cell_mut(cell(3, 0)).overlay = 0;
    s.cell_mut(cell(3, 0)).overlay_data = 16;
    // A wall beside a tower at (10, 0).
    s.cell_mut(cell(10, 1)).overlay = 0;
    s.place_structure(0, "GATEA", cell(0, 0));
    s.place_structure(0, "TOWER", cell(10, 0));
    let w = s.build();
    assert_eq!(data(&w, -1, 0), 2, "east bit toward the gate");
    assert_eq!(data(&w, 3, 0), 16 + 8, "west bit, stage kept");
    assert_eq!(data(&w, 10, 1), 1, "north bit toward the tower");
}

/// spec: S-0112 Edge cases (quirks.gate_tower_fixed_cells: a wall beside a gate one cell long, beyond its fixed cells, gets no bit; off: it does)
#[test]
fn claim_7_fixed_cells_switch() {
    let mut s = wall_scene();
    s.ruleset.sim.buildings.get_mut("GATEA").unwrap().foundation = Some([1, 1]);
    s.cell_mut(cell(1, 0)).overlay = 0;
    s.place_structure(0, "GATEA", cell(0, 0));
    assert_eq!(data(&s.build(), 1, 0), 0);
    s.ruleset.globals.quirks.gate_tower_fixed_cells = false;
    assert_eq!(data(&s.build(), 1, 0), 8);
}
