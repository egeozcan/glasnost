//! Claim tests of S-0516 claim 20 as revised for Q-0796 (revision 8): the
//! generator's phase transitions on changes in operation, the idle initial
//! state, the manual power-off, and the immediate collapse of a generator of
//! type radius 0. Synthetic rules only (see `cloak_common`).

mod cloak_common;

use std::collections::BTreeSet;

use cloak_common::*;
use glasnost_sim::cloak::{self, FieldPhase};
use glasnost_sim::objects;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, ObjectId, Pos, World};

fn radius(w: &World, g: ObjectId) -> i32 {
    w.obj(g).unwrap().cloak.field.map_or(0, |f| f.radius)
}

fn phase(w: &World, g: ObjectId) -> FieldPhase {
    w.obj(g).unwrap().cloak.field.unwrap().phase
}

fn field_offsets(w: &World, h: HouseId) -> BTreeSet<(i32, i32)> {
    let o = at(0, 0);
    w.cloak
        .fields
        .iter()
        .filter(|(hh, _)| *hh == h.0)
        .map(|(_, c)| (c.x - o.x, c.y - o.y))
        .collect()
}

/// Places a building switched off, so that it is not in operation at its
/// first placement.
fn place_off(w: &mut World, name: &str, h: HouseId, dx: i32, dy: i32) -> ObjectId {
    let ti = w.rules().index_of(Kind::Building, name).unwrap();
    let id = objects::create(w, Kind::Building, ti, Some(h));
    w.obj_mut(id).unwrap().econ.building.switched_off = true;
    let p = Pos::centre(at(dx, dy), 0);
    let z = w.surface_at(p.x, p.y);
    objects::place(w, id, Pos::new(p.x, p.y, z), 0);
    id
}

fn switch_on(w: &mut World, id: ObjectId) {
    w.obj_mut(id).unwrap().econ.building.switched_off = false;
}

/// The rules with GENNY0, a generator of type radius 0.
fn rules_with_zero() -> glasnost_rules::schema::Ruleset {
    let mut r = ruleset();
    let mut zero = r.sim.buildings["GENNY"].clone();
    zero.cloak_radius_in_cells = Some(0);
    r.sim.buildings.insert("GENNY0".into(), zero);
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("GENNY0".into());
    r
}

// spec: S-0516 claim 20 (a newly created generator is idle at radius 0; out of operation at placement it stays so indefinitely, and the first turn that detects being in operation makes growth pending and grows radius 1 in that turn; a powered placement grows radius 1 in its first turn)
#[test]
fn claim_20_new_generator_is_idle_at_radius_zero() {
    let r = ruleset();
    let mut w = game(&r);
    let g = place_off(&mut w, "GENNY", ALPHA, 0, 0);
    assert_eq!((radius(&w, g), phase(&w, g)), (0, FieldPhase::Idle));
    run(&mut w, 6);
    assert_eq!((radius(&w, g), phase(&w, g)), (0, FieldPhase::Idle));
    assert!(field_offsets(&w, ALPHA).is_empty());
    switch_on(&mut w, g);
    run(&mut w, 1);
    assert_eq!(radius(&w, g), 1, "growth in the turn that detects the change");
    assert_eq!(phase(&w, g), FieldPhase::Growing);
    run(&mut w, 2);
    assert_eq!(radius(&w, g), GEN_RADIUS);
    // A powered placement.
    let mut w = game(&r);
    let g = spawn(&mut w, "GENNY", ALPHA, 0, 0);
    run(&mut w, 1);
    assert_eq!(radius(&w, g), 1);
}

// spec: S-0516 claim 20 (idle or shrinking below the type radius, a change to being in operation: growing from the current radius, no wait for the shrink to reach 0; a growing generator is not idle and receives no restart)
#[test]
fn claim_20_partial_shrink_reverses_on_restored_operation() {
    let r = ruleset();
    let mut w = game(&r);
    let g = spawn(&mut w, "GENNY", ALPHA, 0, 0);
    run(&mut w, 6);
    assert_eq!((radius(&w, g), phase(&w, g)), (GEN_RADIUS, FieldPhase::Idle));
    switch_off(&mut w, g);
    run(&mut w, 2);
    assert_eq!(radius(&w, g), 1, "two shrink steps, still positive");
    switch_on(&mut w, g);
    run(&mut w, 1);
    assert_eq!(phase(&w, g), FieldPhase::Growing, "growing from radius 1");
    assert_eq!(radius(&w, g), 2, "no collapse to 0 first");
    // Another source sold during the pending growth does not restart it.
    let n = spawn(&mut w, "GENNY", ALPHA, 0, 3);
    objects::remove(&mut w, n);
    assert_eq!(radius(&w, g), 2, "growing is not idle");
    run(&mut w, 2);
    assert_eq!(radius(&w, g), GEN_RADIUS);
}

// spec: S-0516 claim 20 (shrinking at the type radius stays shrinking on a change to being in operation; growing stays growing; staying in operation requests nothing, so an idle field below its full radius stays idle)
#[test]
fn claim_20_rows_without_a_phase_change() {
    let r = ruleset();
    // Shrinking at the type radius, a change to being in operation.
    let mut w = game(&r);
    let g = spawn(&mut w, "GENNY", ALPHA, 0, 0);
    run(&mut w, 6);
    {
        let f = w.obj_mut(g).unwrap().cloak.field.as_mut().unwrap();
        f.phase = FieldPhase::Shrinking;
        f.was_on = false;
    }
    run(&mut w, 1);
    assert_eq!(phase(&w, g), FieldPhase::Shrinking);
    assert_eq!(radius(&w, g), GEN_RADIUS - 1, "the shrink goes on");
    // An idle field below its full radius, operation unchanged: no growth.
    let mut w = game(&r);
    let g = spawn(&mut w, "GENNY", ALPHA, 0, 0);
    run(&mut w, 6);
    w.obj_mut(g).unwrap().cloak.field.as_mut().unwrap().radius = 1;
    run(&mut w, 5);
    assert_eq!((radius(&w, g), phase(&w, g)), (1, FieldPhase::Idle));
}

// spec: S-0516 claim 20 (at current radius 0 a change to being out of operation leaves every phase as it is: pending growth still grows, with no further power gate; idle stays idle; shrinking ends in idle)
#[test]
fn claim_20_radius_zero_loss_of_operation_changes_no_phase() {
    let r = ruleset();
    for (start, after) in [
        (FieldPhase::Growing, (1, FieldPhase::Growing)),
        (FieldPhase::Idle, (0, FieldPhase::Idle)),
        (FieldPhase::Shrinking, (0, FieldPhase::Idle)),
    ] {
        let mut w = game(&r);
        let g = spawn(&mut w, "GENNY", ALPHA, 0, 0);
        run(&mut w, 6);
        {
            // Radius 0 with the given phase, last seen in operation.
            let f = w.obj_mut(g).unwrap().cloak.field.as_mut().unwrap();
            f.radius = 0;
            f.phase = start;
            f.was_on = true;
        }
        switch_off(&mut w, g);
        run(&mut w, 1);
        assert_eq!((radius(&w, g), phase(&w, g)), after, "{start:?}");
    }
}

// spec: S-0516 claim 20 (an accepted power-off then power-on with no building turn between leaves shrinking pending, as unchanged operation asks for nothing; after the collapse and its stop update the generator stays idle at radius 0 while in operation, until a further change or a neighbour restart)
#[test]
fn claim_20_no_automatic_recovery() {
    let r = ruleset();
    let mut w = game(&r);
    let g = spawn(&mut w, "GENNY", ALPHA, 0, 0);
    run(&mut w, 6);
    cloak::field::switched_off(&mut w, g);
    assert_eq!(phase(&w, g), FieldPhase::Shrinking);
    switch_on(&mut w, g);
    run(&mut w, GEN_RADIUS);
    assert_eq!(radius(&w, g), 0, "the collapse was not reversed");
    run(&mut w, 1);
    assert_eq!((radius(&w, g), phase(&w, g)), (0, FieldPhase::Idle));
    run(&mut w, 10);
    assert_eq!((radius(&w, g), phase(&w, g)), (0, FieldPhase::Idle));
    // A further operation transition restores it.
    switch_off(&mut w, g);
    run(&mut w, 1);
    switch_on(&mut w, g);
    run(&mut w, 1);
    assert_eq!(radius(&w, g), 1);
}

// spec: S-0516 claim 20 and quirks.cloak_generator_zero_radius_shutdown (an accepted manual power-off at an idle radius 0 makes shrinking pending and sets the radius to the type radius, adding no coverage; supply loss alone leaves it idle at radius 0; off, nothing is requested at radius 0, and a zero type radius is never given a positive radius)
#[test]
fn claim_20_manual_power_off_at_radius_zero() {
    for on in [true, false] {
        let mut r = rules_with_zero();
        r.globals.quirks.cloak_generator_zero_radius_shutdown = on;
        let mut w = game(&r);
        let g = place_off(&mut w, "GENNY", ALPHA, 0, 0);
        run(&mut w, 2);
        assert_eq!((radius(&w, g), phase(&w, g)), (0, FieldPhase::Idle));
        cloak::field::switched_off(&mut w, g);
        if on {
            assert_eq!((radius(&w, g), phase(&w, g)), (GEN_RADIUS, FieldPhase::Shrinking));
            assert!(field_offsets(&w, ALPHA).is_empty(), "no coverage added");
            run(&mut w, GEN_RADIUS + 1);
            assert_eq!((radius(&w, g), phase(&w, g)), (0, FieldPhase::Idle));
        } else {
            assert_eq!((radius(&w, g), phase(&w, g)), (0, FieldPhase::Idle));
        }
        // A zero type radius is never invented a positive radius.
        let z = place_off(&mut w, "GENNY0", ALPHA, 6, 0);
        cloak::field::switched_off(&mut w, z);
        assert_eq!(radius(&w, z), 0);
    }
}

// spec: S-0516 claim 20 (an immediate collapse (removal, sale, destruction, capture) performs the neighbour-restart test even when current and type radius are 0: the strict reach is ΔX² + ΔY² < 16; ordinary shrinking already at radius 0 performs none)
#[test]
fn claim_20_immediate_collapse_at_type_radius_zero() {
    let r = rules_with_zero();
    for capture in [false, true] {
        let mut w = game(&r);
        let z = spawn(&mut w, "GENNY0", ALPHA, 0, 0);
        let near = spawn(&mut w, "GENNY", BETA, 3, 0);
        let edge = spawn(&mut w, "GENNY", BETA, 0, 4);
        run(&mut w, 6);
        assert_eq!(radius(&w, near), GEN_RADIUS);
        if capture {
            change_owner(&mut w, z, BETA);
        } else {
            objects::remove(&mut w, z);
        }
        assert_eq!(
            (radius(&w, near), phase(&w, near)),
            (0, FieldPhase::Growing),
            "3 cells: inside the reach, capture {capture}"
        );
        assert_eq!(radius(&w, edge), GEN_RADIUS, "4 cells: on the boundary");
    }
    // Ordinary shrinking at radius 0: no test.
    let mut w = game(&r);
    let z = spawn(&mut w, "GENNY0", ALPHA, 0, 0);
    let near = spawn(&mut w, "GENNY", BETA, 3, 0);
    run(&mut w, 6);
    switch_off(&mut w, z);
    run(&mut w, 4);
    assert_eq!(radius(&w, near), GEN_RADIUS);
}

// spec: S-0516 claim 20 (capture: the old field collapses with its restart test, then the generator grows again for the new owner only if it is in operation; an unpowered captured generator stays idle at radius 0 until a change in operation)
#[test]
fn claim_20_capture_of_an_unpowered_generator() {
    let r = ruleset();
    let mut w = game(&r);
    let g = spawn(&mut w, "GENNY", ALPHA, 0, 0);
    run(&mut w, 6);
    switch_off(&mut w, g);
    run(&mut w, 1);
    change_owner(&mut w, g, BETA);
    assert_eq!((radius(&w, g), phase(&w, g)), (0, FieldPhase::Idle));
    run(&mut w, 8);
    assert_eq!((radius(&w, g), phase(&w, g)), (0, FieldPhase::Idle));
    switch_on(&mut w, g);
    run(&mut w, 6);
    assert_eq!(radius(&w, g), GEN_RADIUS);
    // Powered: grows again for the new owner.
    let mut w = game(&r);
    let g = spawn(&mut w, "GENNY", ALPHA, 0, 0);
    run(&mut w, 6);
    change_owner(&mut w, g, BETA);
    assert_eq!(phase(&w, g), FieldPhase::Growing);
    run(&mut w, 4);
    assert_eq!(radius(&w, g), GEN_RADIUS);
}
