//! S-0207 claim 15: a vehicle whose next cell holds an allied closed gate
//! waits without turning until the gate is open, also after the search of
//! T5's obstacle row (#266). Synthetic types and values.

mod common;

use common::{Scene, TRACK, cell, object_in, open_orders, order_move, unit_type};
use glasnost_rules::sim::{BuildingType, UnitType, WarheadType, WeaponType};
use glasnost_sim::{process, tick};

fn scene() -> Scene {
    let mut s = Scene::new();
    s.vehicle("Cart", unit_type(TRACK, 6, 10));
    s.vehicle(
        "Ram",
        UnitType {
            primary: Some("Pea".into()),
            ..unit_type(TRACK, 6, 10)
        },
    );
    s.ruleset.sim.weapons.insert(
        "Pea".into(),
        WeaponType {
            damage: Some(7),
            warhead: Some("Soft".into()),
            ..Default::default()
        },
    );
    s.ruleset
        .sim
        .warheads
        .insert("Soft".into(), WarheadType::default());
    s.register("buildings", "Door");
    s.ruleset.sim.buildings.insert(
        "Door".into(),
        BuildingType {
            strength: Some(300),
            foundation: Some([1, 1]),
            gate: Some(true),
            ..Default::default()
        },
    );
    // A corridor along y = 0, walled by cells four levels up, with one
    // opening north of the mover at (0, -1).
    for x in -4..=9 {
        s.set_cell(cell(x, -1), 0, 4);
        s.set_cell(cell(x, 1), 0, 4);
    }
    s.set_cell(cell(-4, 0), 0, 4);
    s.set_cell(cell(9, 0), 0, 4);
    s.set_cell(cell(0, -1), 0, 0);
    // The mover faces north; an allied gate is east of it; an enemy
    // vehicle stands in the second cell of the curve N, NE.
    s.place_unit(0, "Ram", cell(0, 0), 0);
    s.place_unit(1, "Cart", cell(1, -2), 0);
    s.place_structure(0, "Door", cell(1, 0));
    s
}

// spec: S-0207 claim 15 (a vehicle whose next cell holds an allied closed gate waits without turning on every tick until the gate is open): the ordinary way, a fresh search with no stored path
#[test]
fn issue_266_control_no_turn_towards_a_closed_gate() {
    let s = scene();
    let mut w = s.build();
    let m = object_in(&w, cell(0, 0)).expect("m");
    open_orders(&mut w);
    order_move(&mut w, m, cell(5, 0));
    let mut waited = 0;
    for _ in 0..6 {
        tick::step(&mut w);
        let o = w.obj(m).expect("m");
        let closed = process::closed_friendly_gate(&w, m, cell(1, 0)).is_some();
        if closed && o.path.steps.first() == Some(&2) {
            waited += 1;
            assert_eq!(o.body.target, 0, "no turn in front of the gate");
        }
    }
    assert!(waited > 0, "it waited in front of the closed gate");
}

// spec: S-0207 claim 15 (a vehicle whose next cell holds an allied closed gate waits without turning on every tick until the gate is open, whether or not its body already faces the step; After T5's obstacle row)
#[test]
fn issue_266_no_turn_towards_a_closed_gate_after_t5_obstacle_search() {
    let s = scene();
    let mut w = s.build();
    let m = object_in(&w, cell(0, 0)).expect("m");
    open_orders(&mut w);
    order_move(&mut w, m, cell(5, 0));
    tick::step(&mut w);
    assert!(
        w.obj(m)
            .and_then(|o| o.drive.as_ref())
            .is_some_and(|d| d.track.is_none())
    );
    // Its stored path is made to curve north, then north-east into the
    // enemy's cell (a two-cell curve whose second cell gives an obstacle).
    {
        let o = w.obj_mut(m).expect("m");
        o.path.steps = vec![0, 1, 2, 2, 2, 2];
        o.path.retry_at = 0;
    }
    tick::step(&mut w);
    let o = w.obj(m).expect("m");
    assert!(o.drive.as_ref().is_some_and(|d| d.track.is_none()));
    assert_eq!(
        o.path.steps.first().copied(),
        Some(2),
        "the fresh path's first step leads east, into the gate's cell"
    );
    assert!(
        process::closed_friendly_gate(&w, m, cell(1, 0)).is_some(),
        "the gate is still closed"
    );
    assert_eq!(
        o.body.target, 0,
        "the vehicle started turning towards the step while the gate is closed"
    );
}
