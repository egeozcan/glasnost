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

/// Destroys building `id`: health 0, then removal from the map.
fn destroy(w: &mut glasnost_sim::world::World, id: glasnost_sim::world::ObjectId) {
    w.obj_mut(id).unwrap().health = 0;
    glasnost_sim::objects::remove(w, id);
}

fn tower(w: &glasnost_sim::world::World) -> glasnost_sim::world::ObjectId {
    let r = w.rules_arc();
    w.objects
        .values()
        .find(|o| r.buildings[o.type_index as usize].name == "TOWER")
        .map(|o| o.id)
        .expect("TOWER")
}

/// spec: S-0112 claim 7 (a destroyed wall tower: walls of any type in the edge neighbours of its top-left cell, at stage 0, take a wall hit of 200, north, east, south, west; a damaged one takes none)
/// spec: S-0303 F4 (the hit: a draw over 0 … T when 200 < T; none when T ≤ 200; a link-less two-stage wall reaching stage 1 is removed)
#[test]
fn claim_7_destroyed_tower_hits_its_walls() {
    let mut s = wall_scene();
    s.map.has_overlay_data = true;
    let w0 = s.ruleset.sim.overlays.get_mut("W0").unwrap();
    w0.strength = Some(300);
    w0.damage_levels = Some(4);
    let w26 = s.ruleset.sim.overlays.get_mut("W26").unwrap();
    w26.strength = Some(100);
    w26.damage_levels = Some(2);
    // North: W0 at stage 0. East: W0 at stage 1. South: W26 at stage 0. West: nothing.
    s.cell_mut(cell(0, -1)).overlay = 0;
    s.cell_mut(cell(1, 0)).overlay = 0;
    s.cell_mut(cell(1, 0)).overlay_data = 16;
    s.cell_mut(cell(0, 1)).overlay = 26;
    s.place_structure(0, "TOWER", cell(0, 0));
    let mut w = s.build();
    assert_eq!(data(&w, 0, -1), 4, "joined to the tower before");
    let id = tower(&w);
    let mut expect = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    let r = expect.ranged(0, 300, half);
    destroy(&mut w, id);
    assert_eq!(w.rng, expect, "one draw: the north wall's");
    // North: stage 1 when 200 > r, its bits recomputed without the tower.
    assert_eq!(data(&w, 0, -1), if 200 > r { 16 } else { 0 });
    // East: damaged, not hit; its bits no longer join the tower.
    assert_eq!(data(&w, 1, 0), 16);
    // South: two stages, strength 100: the hit succeeds and removes the link-less wall.
    assert_eq!(
        w.cells.cell(cell(0, 1)).unwrap().overlay,
        glasnost_map::NO_OVERLAY
    );
}

/// spec: S-0112 claim 7 (the departure rows are for a destroyed tower: one leaving with health above 0 deals no hits)
#[test]
fn claim_7_tower_removed_alive_deals_no_hits() {
    let mut s = wall_scene();
    let w0 = s.ruleset.sim.overlays.get_mut("W0").unwrap();
    w0.damage_levels = Some(4);
    s.cell_mut(cell(0, -1)).overlay = 0;
    s.place_structure(0, "TOWER", cell(0, 0));
    let mut w = s.build();
    let id = tower(&w);
    glasnost_sim::objects::remove(&mut w, id);
    assert_eq!(data(&w, 0, -1) / 16, 0);
}

/// spec: S-0112 claim 7 (a wall building of the map's building list is placed as a wall built in play: its overlay at stage 0, joined to its neighbours; no building remains)
/// spec: S-0210 claim 14 (a) (the initial owner from the wall building is replaced before play by the nearest qualifying building's; Q-0684 answered)
#[test]
fn claim_7_map_wall_building_becomes_its_overlay() {
    let mut s = wall_scene();
    s.map.has_overlay_data = true;
    let mut wall = glasnost_rules::sim::BuildingType {
        strength: Some(100),
        foundation: Some([1, 1]),
        ..Default::default()
    };
    wall.econ.wall = Some(true);
    wall.econ.to_overlay = Some("W0".into());
    s.building("WALLB", wall);
    // An overlay-list wall east of it (stream value 0: no bits of its own).
    s.cell_mut(cell(1, 0)).overlay = 0;
    s.place_structure(0, "WALLB", cell(0, 0));
    s.place_structure(1, "TOWER", cell(5, 5));
    let w = s.build();
    let r = w.rules_arc();
    assert!(
        w.objects
            .values()
            .all(|o| o.kind != glasnost_sim::rules::Kind::Building
                || r.buildings[o.type_index as usize].name != "WALLB"),
        "no wall building remains"
    );
    let c = w.cells.cell(cell(0, 0)).unwrap();
    assert_eq!(c.overlay, 0);
    assert_eq!(c.overlay_data, 2, "stage 0, joined east");
    assert_eq!(data(&w, 1, 0), 8, "the neighbour recomputed, joined west");
    let i = w.cells.idx(cell(0, 0)).unwrap();
    assert_eq!(
        w.cells.wall_owner[i],
        Some(glasnost_sim::world::HouseId(1)),
        "claim 14 (a) runs after and gives the nearest building's owner"
    );
}
