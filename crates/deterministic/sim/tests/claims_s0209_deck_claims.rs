//! A drive vehicle's vehicle-present claims on and off a bridge deck
//! (S-0209 claim 1; sim issue #181): the flag set when a step begins is
//! the one the vehicle keeps at rest there, in the record of the layer its
//! position is in, and a cell it leaves loses it. Synthetic data: land at
//! level 4 at y = 1, 0, −5, −6 of column x = 0, four high-bridge cells
//! y = −1 … −4 over level-0 ground.

mod common;

use common::*;
use glasnost_sim::occupancy::{self, Outcome};
use glasnost_sim::rules::Kind;
use glasnost_sim::tick;
use glasnost_sim::world::{Layer, World};

fn bridge_scene() -> Scene {
    let mut s = Scene::new();
    for i in 0..=25 {
        let name = format!("OV{i}");
        s.register("overlays", &name);
        s.ruleset.sim.overlays.insert(
            name,
            glasnost_rules::sim::OverlayType {
                has_art: Some(true),
                ..Default::default()
            },
        );
    }
    for y in [0, 1, -5, -6] {
        s.set_cell(cell(0, y), 0, 4);
    }
    for y in -4..=-1 {
        s.cell_mut(cell(0, y)).overlay = 25;
    }
    s
}

/// The (ground, deck) vehicle-present counts of cell (0, y).
fn counts(w: &World, y: i32) -> (u8, u8) {
    let g = w.cells.occ(cell(0, y), Layer::Ground).map_or(0, |o| o.vehicles);
    let d = w.cells.occ(cell(0, y), Layer::Deck).map_or(0, |o| o.vehicles);
    (g, d)
}

// spec: S-0209 claim 1 (a vehicle at rest keeps the flag of its cell, in the cell layer its position is in; a cell it has left has lost the flag) and S-0207 claim 5 (onto and off the deck)
#[test]
fn crossing_a_bridge_leaves_no_claim_behind() {
    let mut s = bridge_scene();
    let mut t = unit_type(WHEEL, 30, 5);
    t.accelerates = Some(false);
    s.vehicle("TESTCAR", t.clone());
    s.vehicle("PROBE", t);
    s.place_unit(0, "TESTCAR", cell(0, 0), 0);
    // A second vehicle on the low ground, far from the bridge: only asked
    // for its entry outcome.
    s.place_unit(0, "PROBE", cell(8, -1), 0);
    let mut w = s.build();
    let ids = objects_of(&w, Kind::Vehicle);
    let id = *ids
        .iter()
        .find(|&&i| w.obj(i).is_some_and(|o| o.pos.cell() == cell(0, 0)))
        .expect("car");
    let probe = *ids.iter().find(|&&i| i != id).expect("probe");
    open_orders(&mut w);
    let under_end = cell(0, -1);
    assert_eq!(
        occupancy::vehicle_outcome(&w, probe, under_end, 0),
        Outcome::Clear,
        "before the crossing the ground under the bridge end is clear"
    );
    order_move(&mut w, id, cell(0, -5));
    tick::step(&mut w);
    w.obj_mut(id).expect("vehicle").path.steps = vec![0; 5];
    let mut on_first_deck_cell = None;
    for _ in 0..400 {
        tick::step(&mut w);
        let o = w.obj(id).expect("v");
        if o.pos.cell() == under_end && o.layer == Layer::Deck && on_first_deck_cell.is_none() {
            on_first_deck_cell = Some(counts(&w, -1));
        }
        if o.pos.cell() == cell(0, -5) && o.drive.as_ref().is_some_and(|d| d.track.is_none()) {
            break;
        }
    }
    let o = w.obj(id).expect("v");
    assert_eq!(o.pos.cell(), cell(0, -5), "the vehicle crossed");
    assert_eq!(o.layer, Layer::Ground);
    assert_eq!(
        on_first_deck_cell,
        Some((0, 1)),
        "on the first deck cell the claim is in the deck record"
    );
    for y in -4..=0 {
        assert_eq!(counts(&w, y), (0, 0), "no claim left on cell y = {y}");
    }
    assert_eq!(
        counts(&w, -5),
        (1, 0),
        "at rest on land the claim is in the ground record"
    );
    assert_eq!(
        occupancy::vehicle_outcome(&w, probe, under_end, 0),
        Outcome::Clear,
        "after the crossing the ground under the bridge end is clear again"
    );
}

// spec: S-0209 claim 1 (a vehicle taken out of a cell's list clears the flags it holds, each in its own record)
#[test]
fn a_vehicle_removed_on_the_deck_clears_its_claims() {
    let mut s = bridge_scene();
    let mut t = unit_type(WHEEL, 30, 5);
    t.accelerates = Some(false);
    s.vehicle("TESTCAR", t);
    s.place_unit(0, "TESTCAR", cell(0, 0), 0);
    let mut w = s.build();
    let id = objects_of(&w, Kind::Vehicle)[0];
    open_orders(&mut w);
    order_move(&mut w, id, cell(0, -5));
    tick::step(&mut w);
    w.obj_mut(id).expect("vehicle").path.steps = vec![0; 5];
    for _ in 0..400 {
        tick::step(&mut w);
        let o = w.obj(id).expect("v");
        if o.pos.cell() == cell(0, -2) && o.layer == Layer::Deck {
            break;
        }
    }
    assert_eq!(w.obj(id).unwrap().pos.cell(), cell(0, -2));
    glasnost_sim::objects::remove(&mut w, id);
    for y in -6..=1 {
        assert_eq!(counts(&w, y), (0, 0), "no claim left on cell y = {y}");
    }
}
