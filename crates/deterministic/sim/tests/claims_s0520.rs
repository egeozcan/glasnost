//! Claim tests for crates (S-0520): placement, lifetime, expiry and
//! replacement, trigger and death crates, pickup timing and gates, on
//! synthetic data. The numbers are invented by the fixture.

#[path = "crate_common/mod.rs"]
mod crate_common;

use crate_common::*;
use glasnost_map::CellXY;
use glasnost_sim::crates::{self, Place};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, Target, World};

fn crate_cells(w: &World) -> Vec<CellXY> {
    w.cells
        .grid
        .iter()
        .filter(|(c, _)| crates::is_crate(w, *c))
        .map(|(c, _)| c)
        .collect()
}

// spec: S-0520 claim 3 (a random placement puts one crate with random contents on a cell of the map and starts its lifetime of 450 C … 1800 C ticks, C = 1 here)
#[test]
fn a_random_placement_makes_one_crate() {
    let mut w = world_with(true, |_| {});
    assert!(crates::place_random(&mut w));
    let cells = crate_cells(&w);
    assert_eq!(cells.len(), 1);
    let (k, p) = w
        .crates
        .places
        .iter()
        .next()
        .map(|(k, p)| (*k, *p))
        .unwrap();
    assert_eq!((k, p.cell), (0, cells[0]));
    assert!((450..=1800).contains(&p.life), "lifetime {}", p.life);
    assert_eq!(p.start, w.tick);
    assert_eq!(w.cells.cell(p.cell).unwrap().overlay, WOOD);
    assert!(!p.trigger);
}

// spec: S-0520 claim 3 (a water cell takes the water-crate overlay)
#[test]
fn a_water_cell_takes_the_water_crate() {
    let mut w = world_with(true, |_| {});
    let c = at(3, 3);
    make_water(&mut w, c, 3);
    assert!(crates::place_trigger(&mut w, c, 0));
    let found = crate_cells(&w);
    assert_eq!(found.len(), 1);
    assert_eq!(w.cells.cell(found[0]).unwrap().overlay, WATER);
}

// spec: S-0520 claim 1 (every crate place occupied: a placement does nothing and takes no draw)
#[test]
fn the_places_are_limited_to_256() {
    let mut w = world_with(true, |_| {});
    for k in 0..256u16 {
        w.crates.places.insert(
            k,
            Place {
                cell: CellXY::new(0, 0),
                start: 0,
                life: 100_000,
                trigger: false,
            },
        );
    }
    let before = w.rng.clone();
    assert!(!crates::place_random(&mut w));
    assert!(!crates::place_trigger(&mut w, at(1, 1), 0));
    assert!(w.rng == before, "no draw");
}

// spec: S-0524 claim 4 (the number slot: 20 leaves the crate random, every other value is reduced modulo 256; 0 … 18 fixed, 19 … 255 random)
#[test]
fn trigger_crates_take_their_kind_from_the_number_slot() {
    for (slot, want) in [(20, 255u8), (256, 0), (5, 5), (-1, 255), (19, 19)] {
        let mut w = world_with(false, |_| {});
        let c = at(2, 2);
        assert!(crates::place_trigger(&mut w, c, slot), "slot {slot}");
        let contents = w.cells.cell(c).unwrap().overlay_data;
        // 19 … 255 mean random contents.
        assert_eq!(contents.min(19), want.min(19), "slot {slot}");
        assert!(w.crates.places.values().next().unwrap().trigger);
    }
}

// spec: S-0524 claim 4 (placement fails when the selected cell holds an overlay; it does not retry another candidate, and takes no lifetime draw)
#[test]
fn a_trigger_crate_does_not_retry_over_an_overlay() {
    let mut w = world_with(false, |_| {});
    let c = at(2, 2);
    crate_at(&mut w, c, PLAIN, 0);
    let before = w.rng.clone();
    assert!(!crates::place_trigger(&mut w, c, 0));
    assert!(w.crates.places.is_empty());
    assert!(w.rng == before, "a failed placement takes no draw");
}

// spec: S-0524 Edge cases (quirks.trigger_crate_respects_setting: no effect with crates off)
#[test]
fn the_trigger_crate_setting_quirk() {
    for respects in [false, true] {
        let mut w = world_with(false, |r| {
            r.globals.quirks.trigger_crate_respects_setting = respects
        });
        assert_eq!(crates::place_trigger(&mut w, at(2, 2), 0), !respects);
    }
}

// spec: S-0520 claim 4 (with crates on a crate expires when its lifetime has run out and is replaced by one random placement; the old cell is clear)
#[test]
fn an_expired_crate_is_replaced() {
    let mut w = world_with(true, |_| {});
    crates::place_random(&mut w);
    let old = *w.crates.places.values().next().unwrap();
    run_to(&mut w, old.start + old.life + 1);
    assert_eq!(w.crates.places.len(), 1, "one expiry, one replacement");
    let now = w.crates.places.values().next().unwrap();
    assert!(now.start >= old.start + old.life, "a new crate");
    if now.cell != old.cell {
        assert!(!crates::is_crate(&w, old.cell));
    }
}

// spec: S-0520 claim 4 (crates off: the update does nothing, trigger crates included, however long they lie)
#[test]
fn with_crates_off_nothing_expires() {
    let mut w = world_with(false, |_| {});
    crates::place_trigger(&mut w, at(2, 2), 0);
    let p = *w.crates.places.values().next().unwrap();
    run_to(&mut w, p.start + 1801 + 5);
    assert_eq!(w.crates.places.len(), 1);
    assert!(crates::is_crate(&w, p.cell));
}

// spec: S-0524 Edge cases (quirks.trigger_crate_expiry_while_disabled: an uncollected trigger crate expires while crates are off, with no replacement)
#[test]
fn the_expiry_quirk_lets_a_trigger_crate_go() {
    let mut w = world_with(false, |r| {
        r.globals.quirks.trigger_crate_expiry_while_disabled = true
    });
    crates::place_trigger(&mut w, at(2, 2), 0);
    let p = *w.crates.places.values().next().unwrap();
    run_to(&mut w, p.start + p.life + 2);
    assert!(w.crates.places.is_empty());
    assert!(!crates::is_crate(&w, p.cell));
}

// spec: S-0520 claim 4 (simultaneous expiries: replacement groups in increasing place-priority order, each seeing the predecessor's place free, so a replacement cannot expire again in that update)
#[test]
fn a_zero_lifetime_replacement_does_not_expire_in_the_same_update() {
    let mut w = world_with(true, |r| {
        r.sim.general.economy.crate_regen = Some(d("0"));
    });
    crates::place_random(&mut w);
    assert_eq!(w.crates.places.values().next().unwrap().life, 0);
    let before = w.crates.places.keys().copied().collect::<Vec<_>>();
    testkit::run(&mut w, 1);
    assert_eq!(w.crates.places.len(), 1);
    // The replacement took the freed place, and exists after the tick.
    assert_eq!(w.crates.places.keys().copied().collect::<Vec<_>>(), before);
}

// spec: S-0520 claim 6 (a walker collects the crate of the cell it commits to, before it arrives there: money in R(b, b + 900) with b = 100 here, the crate gone, a replacement placed with crates on)
#[test]
fn a_walker_collects_a_money_crate_before_arriving() {
    let mut w = world_with(true, |_| {});
    let inf = spawn(&mut w, Kind::Infantry, "WALKER", 0, at(0, 0));
    let c = at(3, 0);
    crate_at(&mut w, c, WOOD, 0);
    w.crates.places.insert(
        0,
        Place {
            cell: c,
            start: 0,
            life: 100_000,
            trigger: false,
        },
    );
    run_to(&mut w, 6);
    let before = credits(&w, 0);
    assert!(orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: inf,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(at(5, 0))),
        },
    ));
    let mut picked_at = None;
    for _ in 0..600 {
        testkit::run(&mut w, 1);
        if credits(&w, 0) != before && picked_at.is_none() {
            picked_at = Some(w.obj(inf).unwrap().pos.cell());
        }
    }
    let got = credits(&w, 0) - before;
    assert!((100..=1000).contains(&got), "money {got}");
    let at_pick = picked_at.expect("picked up");
    assert!(
        at_pick.x < c.x || at_pick.y != c.y || at_pick == c,
        "collected before arriving at the centre: {at_pick:?}"
    );
    assert!(!crates::is_crate(&w, c), "the crate is gone");
    assert_eq!(crate_cells(&w).len(), 1, "the replacement");
}

// spec: S-0520 claim 7 (a cell whose overlay lacks the crate flag: no pickup; a passive collecting country: no pickup, the crate stays)
#[test]
fn the_gates_refuse_a_pickup() {
    let mut w = world_with(false, |_| {});
    let v = spawn(&mut w, Kind::Vehicle, "ARMED", 0, at(0, 0));
    let c = at(1, 0);
    crate_at(&mut w, c, PLAIN, 0);
    assert!(!crates::pickup(&mut w, v, c));
    assert_eq!(credits(&w, 0), 10_000_i32.min(credits(&w, 0)));
    let money = credits(&w, 0);
    crate_at(&mut w, c, WOOD, 0);
    assert!(!crates::pickup(&mut w, v, c) || credits(&w, 0) != money);
}

// spec: S-0520 claim 7 (a crate-trigger overlay sets the any-crate mark when the pickup continues; a plain crate does not)
#[test]
fn a_trigger_overlay_sets_the_any_crate_mark() {
    let mut w = world_with(false, |_| {});
    let v = spawn(&mut w, Kind::Vehicle, "ARMED", 0, at(0, 0));
    crate_at(&mut w, at(1, 0), WOOD, 0);
    crates::pickup(&mut w, v, at(1, 0));
    assert!(!w.crates.any_picked);
    crate_at(&mut w, at(2, 0), TRIGGER, 0);
    crates::pickup(&mut w, v, at(2, 0));
    assert!(w.crates.any_picked);
}

// spec: S-0312 claims 14 and 20 and S-0520 claim 5 (a destroyed crate-carrying vehicle leaves a random crate; a building with a crate beneath a money crate when its type says so; crates must be on)
#[test]
fn deaths_leave_crates() {
    for on in [true, false] {
        let mut w = world_with(on, |r| {
            r.sim.vehicles.get_mut("ARMED").unwrap().carries_crate = Some(true);
            let b = r.sim.buildings.get_mut("BASE").unwrap();
            b.crate_beneath = Some(true);
            b.crate_beneath_is_money = Some(true);
        });
        let v = spawn(&mut w, Kind::Vehicle, "ARMED", 1, at(0, 0));
        let b = spawn(&mut w, Kind::Building, "BASE", 1, at(6, 0));
        for id in [v, b] {
            let hit = glasnost_sim::combat::damage::Hit {
                amount: 100_000,
                warhead: w.rules().combat.general.c4_warhead,
                distance: 0,
                attacker: None,
                house: None,
                ignore_defences: true,
                no_escape: false,
            };
            glasnost_sim::combat::damage::apply(&mut w, id, hit);
        }
        testkit::run(&mut w, 3);
        let cells = crate_cells(&w);
        assert_eq!(cells.len(), if on { 2 } else { 0 }, "crates on: {on}");
        if on {
            let contents: Vec<u8> = cells
                .iter()
                .map(|&c| w.cells.cell(c).unwrap().overlay_data)
                .collect();
            assert!(
                contents.contains(&0) && contents.contains(&255),
                "{contents:?}"
            );
        }
    }
}

/// A world with crates on and the building type BASE given a money crate
/// beneath (and the explodes flag when `explodes`); destroys a BASE of
/// house 1 at (6, 0), runs 3 ticks and returns its top-left cell and the
/// crate cells.
fn crate_beneath(explodes: bool) -> (CellXY, Vec<CellXY>) {
    let mut w = world_with(true, |r| {
        let b = r.sim.buildings.get_mut("BASE").unwrap();
        b.crate_beneath = Some(true);
        b.crate_beneath_is_money = Some(true);
        if explodes {
            b.combat.explodes = Some(true);
        }
    });
    let b = spawn(&mut w, Kind::Building, "BASE", 1, at(6, 0));
    let top_left = w.obj(b).unwrap().pos.cell();
    let hit = glasnost_sim::combat::damage::Hit {
        amount: 100_000,
        warhead: w.rules().combat.general.c4_warhead,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: true,
        no_escape: false,
    };
    glasnost_sim::combat::damage::apply(&mut w, b, hit);
    testkit::run(&mut w, 3);
    assert!(w.obj(b).is_none(), "the building has left play");
    (top_left, crate_cells(&w))
}

// spec: S-0312 claim 20 (a crate beneath appears on the building's top-left foundation cell) and claim 21 (placed in the moment it leaves play); S-0520 claim 5; S-0312 scenario D14 variant b (a different crate cell falsifies b)
#[test]
fn a_crate_beneath_lands_on_the_top_left_foundation_cell() {
    let (top_left, cells) = crate_beneath(false);
    assert_eq!(cells, vec![top_left]);
}

// spec: S-0312 claim 21 (a building with the explodes flag awaits removal; its crate is placed in its removal turn, after the second round) and claim 20 (the top-left foundation cell)
#[test]
fn an_exploding_buildings_crate_beneath_lands_on_its_top_left_cell() {
    let (top_left, cells) = crate_beneath(true);
    assert_eq!(cells, vec![top_left]);
}

// spec: S-0520 Determinism: crate places, lifetimes and the mark survive a save and load
#[test]
fn crate_state_survives_a_save_and_load() {
    let mut w = world_with(true, |_| {});
    crates::place_random(&mut w);
    crates::place_trigger(&mut w, at(2, 2), 3);
    w.crates.any_picked = true;
    let bytes = glasnost_sim::save::save(&w);
    let h = glasnost_sim::save::state_hash(&w);
    let w2 = glasnost_sim::save::load(&bytes, w.rules_arc(), w.catalogue.clone().unwrap()).unwrap();
    assert_eq!(glasnost_sim::save::state_hash(&w2), h);
    assert_eq!(w2.crates, w.crates);
}

// spec: S-0524 claim 4 (revision 9, scenario C4): the number slot exactly 20 stores the random value 255; every other input stores its reduction modulo 256 as the overlay data value, even when that reduction is 20 (276 stores 20, not 255); 19 stays 19, 21 stays 21, 255 stays 255, 256 stores 0 and -1 stores 255; stored values 19 to 255 pick randomly at pickup
#[test]
fn trigger_crates_store_the_exact_reduction_of_the_number_slot() {
    for (slot, want) in [
        (19, 19u8),
        (20, 255),
        (21, 21),
        (255, 255),
        (256, 0),
        (276, 20),
        (-1, 255),
        (5, 5),
        (18, 18),
    ] {
        let mut w = world_with(false, |_| {});
        let c = at(2, 2);
        assert!(crates::place_trigger(&mut w, c, slot), "slot {slot}");
        assert_eq!(w.cells.cell(c).unwrap().overlay_data, want, "slot {slot}");
    }
}
