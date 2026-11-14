//! S-0009 claim 12 with S-0207 claim 7: a cell whose overlay value is 255
//! holds no overlay, however many overlay types the rules register (#264).
//! Synthetic types on the flat test map of `tests/common`.

mod common;

use common::{Scene, TRACK, cell, open_orders, order_move, run, unit_type};
use glasnost_map::NO_OVERLAY;
use glasnost_rules::sim::{MovementZone, OverlayType};
use glasnost_sim::rules::Kind;

/// Drives a crusher-all vehicle over cells that hold no overlay, with `n`
/// registered overlay types of which the last is crushable; returns whether
/// the vehicle ever entered the crush-slowdown state.
fn slowdown_on_empty_cells(n: usize) -> bool {
    let mut s = Scene::new();
    let mut t = unit_type(TRACK, 6, 8);
    t.movement_zone = Some(MovementZone::CrusherAll);
    s.vehicle("ROLLER", t);
    let names: Vec<String> = (0..n).map(|i| format!("OV{i}")).collect();
    for (i, name) in names.iter().enumerate() {
        s.ruleset.sim.overlays.insert(
            name.clone(),
            OverlayType {
                has_art: Some(true),
                crushable: Some(i + 1 == n),
                ..Default::default()
            },
        );
    }
    s.ruleset.registries.insert("overlays".into(), names);
    s.place_unit(0, "ROLLER", cell(0, 0), 64);
    let mut w = s.build();
    for x in 0..=8 {
        assert_eq!(w.cells.cell(cell(x, 0)).unwrap().overlay, NO_OVERLAY);
    }
    let id = common::objects_of(&w, Kind::Vehicle)[0];
    open_orders(&mut w);
    order_move(&mut w, id, cell(8, 0));
    let mut entered = false;
    for _ in 0..600 {
        run(&mut w, 1);
        entered |= w.obj(id).unwrap().crush_slowdown;
    }
    assert_eq!(w.obj(id).unwrap().pos.cell(), cell(8, 0));
    entered
}

// spec: S-0009 claim 12 (the value 255 means "no overlay"); S-0207 claim 7 (crush mode: the first cell holds a crushable overlay); S-0210 claim 9
#[test]
fn issue_264_control_255_types_no_slowdown_on_empty_cells() {
    assert!(!slowdown_on_empty_cells(255));
}

// spec: S-0009 claim 12 (the value 255 means "no overlay"); S-0207 claim 7 (crush mode: the first cell holds a crushable overlay); S-0210 claim 9
#[test]
fn issue_264_with_256_types_empty_cells_are_not_overlay_255() {
    assert!(
        !slowdown_on_empty_cells(256),
        "a cell with no overlay was read as overlay type 255"
    );
}
