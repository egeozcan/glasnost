//! Claim tests for S-0520 revision 17 (answers Q-3152), M8 A6c (#284): the
//! pickup of a named crate overlay that frees no place and is no map's own
//! crate leaves it on its cell. Synthetic data of `crate_common`; every name
//! and number is invented. Ledger: `docs/m8-claims/sim-issues.md`,
//! *Batch A6c: #284 and #367*.

#[path = "crate_common/mod.rs"]
mod crate_common;

use crate_common::*;
use glasnost_map::CellXY;
use glasnost_sim::crates;
use glasnost_sim::crush;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{ObjectId, World};

fn overlay(w: &World, c: CellXY) -> Option<u8> {
    w.cells.cell(c).map(|x| x.overlay)
}

fn collector(w: &mut World) -> ObjectId {
    spawn(w, Kind::Vehicle, "ARMED", 0, at(0, 0))
}

/// The keys of the occupied places whose cell is `c`.
fn places_on(w: &World, c: CellXY) -> Vec<u16> {
    w.crates
        .places
        .iter()
        .filter(|(_, p)| p.cell == c)
        .map(|(&k, _)| k)
        .collect()
}

// spec: S-0520 claim 11 (revision 17: a pickup that frees no place removes no crate, whatever its overlay type, except a map's own crate; a named crate overlay on a cell that no occupied place has stays, the pickup still gives its effect and, with the crate setting on, makes its random replacement, and the crate can be collected again)
#[test]
fn a_named_crate_overlay_without_a_place_stays_when_picked_up() {
    let mut w = world_with(true, |_| {});
    let v = collector(&mut w);
    let c = at(1, 0);
    // An overlay placement other than a crate placement: the wooden-crate
    // overlay with the money contents, holding no place.
    crate_at(&mut w, c, WOOD, 0);
    assert!(w.crates.places.is_empty(), "set-up: every place is free");
    let before = credits(&w, 0);
    crates::pickup(&mut w, v, c);
    assert!(credits(&w, 0) > before, "the money effect");
    assert_eq!(overlay(&w, c), Some(WOOD), "the crate stays on its cell");
    assert_eq!(
        w.crates.places.len(),
        1,
        "the random replacement takes a place"
    );
    assert!(places_on(&w, c).is_empty(), "the replacement is elsewhere");
    let before = credits(&w, 0);
    crates::pickup(&mut w, v, c);
    assert!(credits(&w, 0) > before, "collected again");
    assert_eq!(overlay(&w, c), Some(WOOD), "and it stays again");
}

// spec: S-0520 claim 11 and quirks.crate_place_by_cell (revision 17: with the switch off a named crate overlay that no crate placement placed belongs to no place, so its pickup frees none and it stays, even when an occupied place has its cell; on, the pickup frees that place and removes the crate)
#[test]
fn a_named_crate_overlay_no_placement_placed_and_a_place_on_its_cell() {
    for (by_cell, want, left) in [(true, glasnost_map::NO_OVERLAY, 0), (false, WOOD, 1)] {
        let mut w = world_with(false, |r| r.globals.quirks.crate_place_by_cell = by_cell);
        let v = collector(&mut w);
        let c = at(1, 0);
        assert!(crates::place_trigger(&mut w, c, 0), "set-up: a crate");
        // Another rule removes the placed crate; an overlay placement that is
        // no crate placement puts the wooden-crate overlay back.
        crush::remove_overlay(&mut w, c);
        crate_at(&mut w, c, WOOD, 0);
        assert_eq!(places_on(&w, c).len(), 1, "set-up: the place stays");
        let before = credits(&w, 0);
        crates::pickup(&mut w, v, c);
        assert!(credits(&w, 0) > before, "by cell {by_cell}: the effect");
        assert_eq!(overlay(&w, c), Some(want), "by cell {by_cell}: the crate");
        assert_eq!(
            places_on(&w, c).len(),
            left,
            "by cell {by_cell}: the place"
        );
    }
}
