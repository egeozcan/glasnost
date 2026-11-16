//! Claim tests for S-0520 revision 16 (answers Q-3140), sim issues S23
//! (#349): a crate whose overlay S-0009 claim 12's slope condition refuses,
//! and `quirks.crate_phantom_place`. Synthetic data of `crate_common`;
//! every name and number is invented. Ledger:
//! `docs/m8-claims/sim-issues.md`, *Batch S23*.

#[path = "crate_common/mod.rs"]
mod crate_common;

use crate_common::*;
use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::crates;
use glasnost_sim::world::World;

fn overlay(w: &World, c: CellXY) -> Option<u8> {
    w.cells.cell(c).map(|x| x.overlay)
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

/// Gives cell `c` slope shape `s`.
fn slope(w: &mut World, c: CellXY, s: u8) {
    w.cells.grid.get_mut(c).unwrap().slope = s;
}

fn off(r: &mut Ruleset) {
    r.globals.quirks.crate_phantom_place = false;
}

// spec: S-0520 claim 5 and claim 3 (revision 16, Q-3140: when S-0009 claim 12's slope condition refuses a trigger crate's overlay at its selected cell, the place is still taken and the lifetime request made, and no crate appears; quirks.crate_phantom_place on)
#[test]
fn issue_349_a_trigger_crate_refused_by_the_slope_takes_a_place_and_shows_nothing() {
    let mut w = world_with(true, |_| {});
    let c = at(2, 2);
    let none = overlay(&w, c);
    slope(&mut w, c, 5);
    let before = w.rng.clone();
    assert!(
        crates::place_trigger(&mut w, c, 0),
        "the attempt counts as the successful one"
    );
    assert_eq!(
        overlay(&w, c),
        none,
        "no crate appears on a slope shape above 4"
    );
    assert_eq!(places_on(&w, c).len(), 1, "the place is taken");
    assert!(w.rng != before, "the lifetime request is made");
}

// spec: S-0520 claim 5 (revision 16: a death crate refused by the slope condition takes its place and lifetime request too; quirks.crate_phantom_place on)
#[test]
fn issue_349_a_death_crate_refused_by_the_slope_takes_a_place() {
    let mut w = world_with(true, |_| {});
    let c = at(-3, 4);
    let none = overlay(&w, c);
    slope(&mut w, c, 7);
    assert!(crates::place_death(&mut w, c, true));
    assert_eq!(overlay(&w, c), none);
    assert_eq!(places_on(&w, c).len(), 1);
}

// spec: S-0520 Edge cases (quirks.crate_phantom_place off: the overlay's appearance is one of claim 3's placement tests; a trigger crate places nothing, with no place and no lifetime draw)
#[test]
fn issue_349_quirk_off_a_refused_trigger_crate_places_nothing() {
    let mut w = world_with(true, off);
    let c = at(2, 2);
    let none = overlay(&w, c);
    slope(&mut w, c, 5);
    let before = w.rng.clone();
    let places = w.crates.places.len();
    assert!(!crates::place_trigger(&mut w, c, 0));
    assert_eq!(overlay(&w, c), none);
    assert_eq!(w.crates.places.len(), places, "no place");
    assert!(w.rng == before, "no lifetime draw");
}

// spec: S-0009 claim 12 (a slope shape of at most 4 takes the overlay) and S-0520 claim 3 (the crate appears)
#[test]
fn issue_349_a_slope_of_four_still_takes_the_crate() {
    for on in [true, false] {
        let mut w = world_with(true, |r| r.globals.quirks.crate_phantom_place = on);
        let c = at(2, 2);
        slope(&mut w, c, 4);
        assert!(crates::place_trigger(&mut w, c, 0));
        assert_eq!(overlay(&w, c), Some(WOOD), "quirk {on}");
        assert_eq!(places_on(&w, c).len(), 1);
    }
}

// spec: S-0520 Edge cases (quirks.crate_phantom_place: the place holds no crate and stays occupied until its expiry, which makes its random placement as any expiry does) and claim 4
#[test]
fn issue_349_a_phantom_place_expires_and_frees_its_place() {
    let mut w = world_with(true, |_| {});
    let c = at(2, 2);
    slope(&mut w, c, 5);
    assert!(crates::place_trigger(&mut w, c, 0));
    let k = places_on(&w, c)[0];
    let now = w.tick;
    if let Some(p) = w.crates.places.get_mut(&k) {
        p.start = now;
        p.life = 1;
    }
    let t = w.tick + 2;
    run_to(&mut w, t);
    assert!(places_on(&w, c).is_empty(), "the expiry freed the place");
}
