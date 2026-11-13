//! The nearby-cell search's class test on a bridge cell outside deck mode
//! (S-0212 claim 25; sim issue #199): a bridge cell is judged as entered on
//! its deck in every use, so the land under it is not consulted. Deck mode
//! changes only the reach read, the centre's level and the ring cut-off.
//! Synthetic data: a level-4 rock centre cell, beside it one high-bridge
//! cell over level-0 rock, everything else flat level-0 clear ground.

mod common;

use common::*;
use glasnost_rules::sim::{LocomotionClass, MovementZone};
use glasnost_sim::path::{self, NearbySettings, NearbyUse};
use glasnost_sim::world::World;

fn world() -> World {
    let mut s = Scene::new();
    // spec: S-0211 claim 5 (terrain code 15: rock)
    let rock = code_tile(15);
    s.set_cell(cell(0, 0), rock, 4);
    s.set_cell(cell(0, -1), rock, 0);
    let mut w = s.build();
    let i = w.cells.idx(cell(0, -1)).unwrap();
    w.cells.deck[i] = true;
    w.cells.deck_entry[i] = true;
    w.cells.bridge_axis[i] = glasnost_sim::cells::Axis::Y;
    w.regions.dirty = true;
    w
}

fn settings(deck: bool) -> NearbySettings {
    NearbySettings {
        class: Some(LocomotionClass::Wheel),
        zone: MovementZone::Normal,
        reach: None,
        deck,
        height: true,
        reference: None,
        no_bridges: false,
        no_overlay: false,
    }
}

// spec: S-0212 claim 25 (enterable by class, a bridge cell being judged as entered on its deck, so that its land type is not consulted; the height condition with the centre's level not raised outside deck mode)
#[test]
fn a_bridge_cell_over_rock_qualifies_outside_deck_mode() {
    let w = world();
    assert_eq!(
        path::nearby(&w, cell(0, 0), NearbyUse::Settings(settings(false))),
        Some(cell(0, -1)),
        "the deck cell (level 0 + 4) is within 1 level of the level-4 centre and is entered on its deck"
    );
}

// spec: S-0212 claim 25 (the same cell in deck mode: the class test does not change)
#[test]
fn a_bridge_cell_over_rock_qualifies_in_deck_mode() {
    let w = world();
    assert_eq!(
        path::nearby(&w, cell(0, 0), NearbyUse::Settings(settings(true))),
        Some(cell(0, -1))
    );
}
