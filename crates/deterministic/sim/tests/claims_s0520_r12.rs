//! Claim tests for S-0520 revision 12 (answers Q-3118), sim issues S21
//! (#284): the named crate overlays, the expiry's overlay test, the place a
//! pickup frees, a map's own crate and `quirks.crate_place_by_cell`.
//! Synthetic data of `crate_common`; every name and number is invented.
//! Ledger: `docs/m8-claims/sim-issues.md`, *Batch S21*.

#[path = "crate_common/mod.rs"]
mod crate_common;

use std::sync::Arc;

use crate_common::*;
use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::crates::{self, Place};
use glasnost_sim::rules::{Kind, SimRules};
use glasnost_sim::world::{ObjectId, World};
use glasnost_sim::{crush, testkit};

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

/// A crate placed on `c` by a trigger action, with a place; returns its key.
fn placed(w: &mut World, c: CellXY) -> u16 {
    let before: Vec<u16> = w.crates.places.keys().copied().collect();
    assert!(crates::place_trigger(w, c, 0), "set-up: a crate on the cell");
    *w.crates
        .places
        .keys()
        .find(|k| !before.contains(k))
        .expect("set-up: a new place")
}

/// Shortens a place's lifetime so that it expires at the next crate update.
fn expire_now(w: &mut World, k: u16) {
    let now = w.tick;
    if let Some(p) = w.crates.places.get_mut(&k) {
        p.start = now;
        p.life = 1;
    }
}

/// Two occupied places on `c`: the older (lower) one's crate removed by
/// another rule, the newer one's crate on the cell.
fn two_places(w: &mut World, c: CellXY) -> (u16, u16) {
    let old = placed(w, c);
    crush::remove_overlay(w, c);
    let new = placed(w, c);
    assert!(old < new);
    assert_eq!(places_on(w, c), vec![old, new]);
    (old, new)
}

fn off(r: &mut Ruleset) {
    r.globals.quirks.crate_place_by_cell = false;
}

// spec: S-0520 claims 4 and 29 (revision 12: an expiry removes the overlay on its place's cell only when it is a named crate overlay; another overlay type with the crate flag stays, and the place is freed and replaced whatever the cell holds)
#[test]
fn issue_284_an_expiry_leaves_a_crate_flagged_overlay_of_another_type() {
    let mut w = world_with(true, |_| {});
    let c = at(2, 2);
    let k = placed(&mut w, c);
    crate_at(&mut w, c, TRIGGER, 0);
    expire_now(&mut w, k);
    let t = w.tick + 2;
    run_to(&mut w, t);
    assert_eq!(overlay(&w, c), Some(TRIGGER), "the overlay stays");
    assert!(places_on(&w, c).is_empty(), "the place is freed");
}

// spec: S-0520 Terms and Import mapping (revision 12: the campaign crate overlay that CrateImg names is a named crate overlay, which an expiry removes; with no CrateImg no overlay is one)
#[test]
fn issue_284_the_campaign_crate_overlay_is_a_named_crate_overlay() {
    for (named, want) in [(true, glasnost_map::NO_OVERLAY), (false, TRIGGER)] {
        let mut w = world_with(true, |r| {
            if named {
                r.sim.general.economy.crate_img = Some("TCRATE".into());
            }
        });
        let c = at(2, 2);
        let k = placed(&mut w, c);
        crate_at(&mut w, c, TRIGGER, 0);
        expire_now(&mut w, k);
        let t = w.tick + 2;
        run_to(&mut w, t);
        assert_eq!(overlay(&w, c), Some(want), "named {named}");
    }
}

// spec: S-0520 claim 11 and Edge cases, unsafe data (revision 12: the pickup of a crate-flagged overlay that is not a named crate overlay frees the place and gives its effect; the overlay stays and can be collected again)
#[test]
fn issue_284_a_pickup_leaves_a_crate_flagged_overlay_of_another_type() {
    let mut w = world_with(false, |_| {});
    let v = collector(&mut w);
    let c = at(1, 0);
    placed(&mut w, c);
    crate_at(&mut w, c, TRIGGER, 0);
    let before = credits(&w, 0);
    crates::pickup(&mut w, v, c);
    assert!(credits(&w, 0) > before, "the money effect");
    assert_eq!(overlay(&w, c), Some(TRIGGER), "the overlay stays");
    assert!(places_on(&w, c).is_empty(), "its place is freed");
    let before = credits(&w, 0);
    crates::pickup(&mut w, v, c);
    assert!(credits(&w, 0) > before, "collected again");
}

// spec: S-0520 claims 11 and 29 (revision 12, quirks.crate_place_by_cell on: a pickup frees the occupied place with the lowest priority number on the crate's cell, here the older place whose crate another rule removed; the newer place stays occupied)
#[test]
fn issue_284_a_pickup_frees_the_lowest_place_on_the_cell() {
    let mut w = world_with(false, |_| {});
    let v = collector(&mut w);
    let c = at(1, 0);
    let (_, new) = two_places(&mut w, c);
    crates::pickup(&mut w, v, c);
    assert_eq!(overlay(&w, c), Some(glasnost_map::NO_OVERLAY));
    assert_eq!(places_on(&w, c), vec![new]);
}

// spec: S-0520 claim 29 and Edge cases (quirks.crate_place_by_cell off: each place belongs to the crate it placed; a pickup frees the place the collected crate belongs to, the newer one, and the older place stays occupied)
#[test]
fn issue_284_switch_off_a_pickup_frees_its_own_place() {
    let mut w = world_with(false, off);
    let v = collector(&mut w);
    let c = at(1, 0);
    let (old, _) = two_places(&mut w, c);
    crates::pickup(&mut w, v, c);
    assert_eq!(overlay(&w, c), Some(glasnost_map::NO_OVERLAY));
    assert_eq!(places_on(&w, c), vec![old]);
}

// spec: S-0520 claim 29 (quirks.crate_place_by_cell on: the older place's expiry removes the newer crate on its cell; off: it removes the overlay only when that is its own crate, so the newer crate stays; both free the older place)
#[test]
fn issue_284_the_older_place_expiry_and_the_newer_crate() {
    for (by_cell, want) in [(true, glasnost_map::NO_OVERLAY), (false, WOOD)] {
        let mut w = world_with(true, |r| r.globals.quirks.crate_place_by_cell = by_cell);
        let c = at(2, 2);
        let (old, new) = two_places(&mut w, c);
        expire_now(&mut w, old);
        let t = w.tick + 2;
        run_to(&mut w, t);
        assert_eq!(overlay(&w, c), Some(want), "by cell {by_cell}");
        assert!(
            !w.crates.places.get(&old).is_some_and(|p| p.cell == c),
            "by cell {by_cell}: the older place is freed"
        );
        assert!(
            w.crates.places.get(&new).is_some_and(|p| p.cell == c),
            "by cell {by_cell}: the newer place stays"
        );
    }
}

/// A game whose map puts `o` on `c` with contents 0 (money), the map's crates
/// kept (quirks.skirmish_drops_map_crates off).
fn map_crate_world(c: CellXY, o: u8, by_cell: bool) -> World {
    let mut r = ruleset();
    r.globals.quirks.skirmish_drops_map_crates = false;
    r.globals.quirks.crate_place_by_cell = by_cell;
    let mut map = testkit::empty_map(SIZE);
    if let Some(cell) = map.cells.get_mut(c) {
        cell.overlay = o;
        cell.overlay_data = 0;
    }
    let rules = Arc::new(SimRules::new(&r, "synthetic-test"));
    let mut w = glasnost_sim::load::new_game(rules, &map, testkit::catalogue(), &testkit::setup(7))
        .expect("synthetic test game loads");
    w.econ.settings.crates = Some(false);
    w.econ.start.bases = true;
    assert_eq!(overlay(&w, c), Some(o), "set-up: the map's crate is kept");
    w
}

// spec: S-0520 claim 11 (revision 12: a map's own crate kept with quirks.skirmish_drops_map_crates off is removed by its pickup whatever its overlay type)
#[test]
fn issue_284_a_map_crate_of_another_type_is_removed_by_its_pickup() {
    let c = at(1, 0);
    let mut w = map_crate_world(c, TRIGGER, true);
    let v = collector(&mut w);
    crates::pickup(&mut w, v, c);
    assert_eq!(overlay(&w, c), Some(glasnost_map::NO_OVERLAY));
}

// spec: S-0520 claim 11 and Edge cases (a map's own crate holds no place; with quirks.crate_place_by_cell on its pickup frees the lowest occupied place on its cell, off it frees none)
#[test]
fn issue_284_a_map_crate_pickup_and_the_places_on_its_cell() {
    for (by_cell, left) in [(true, 0), (false, 1)] {
        let c = at(1, 0);
        let mut w = map_crate_world(c, WOOD, by_cell);
        let v = collector(&mut w);
        let now = w.tick;
        w.crates.places.insert(
            0,
            Place {
                cell: c,
                start: now,
                life: 100_000,
                trigger: true,
            },
        );
        crates::pickup(&mut w, v, c);
        assert_eq!(overlay(&w, c), Some(glasnost_map::NO_OVERLAY));
        assert_eq!(places_on(&w, c).len(), left, "by cell {by_cell}");
    }
}
