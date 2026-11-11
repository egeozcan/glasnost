//! Superweapon fire orders with cells far off the map (#219): refused at
//! issue outside the engine's cell window, and carried through every kind's
//! effect code inside it, off the map as S-0513 claims 1 and 3 and S-0512
//! claim 10 ask. Synthetic data only: the `sw_common` rules.

mod sw_common;

use glasnost_map::CellXY;
use glasnost_sim::orders::{self, Order};
use sw_common::*;

/// The window's corners: the farthest cells an order may carry.
fn corners() -> [CellXY; 4] {
    [
        CellXY::new(-512, -512),
        CellXY::new(1023, 1023),
        CellXY::new(-512, 1023),
        CellXY::new(1023, -512),
    ]
}

// nocite: our own input validation (#219): a second-stage order from a house without superweapons, at the smallest x whose cell centre leaves i32
#[test]
fn a_second_stage_order_with_a_far_cell_is_refused() {
    let r = ruleset();
    let mut w = game(&r);
    let taken = orders::issue(
        &mut w,
        BETA,
        Order::FireSuperweapon {
            sw: WARP,
            cell: CellXY::new(8_388_608, 0),
        },
    );
    step(&mut w);
    assert!(!taken);
}

// nocite: our own input validation (#219): a ready curtain at the largest cell
#[test]
fn a_ready_curtain_order_with_the_largest_cell_is_refused() {
    let r = ruleset();
    let mut w = game(&r);
    spawn(&mut w, "IRONB", ALPHA, 0, 0);
    ready_tick(&mut w, ALPHA, CURTAIN, 200);
    let taken = orders::issue(
        &mut w,
        ALPHA,
        Order::FireSuperweapon {
            sw: CURTAIN,
            cell: CellXY::new(i32::MAX, i32::MAX),
        },
    );
    step(&mut w);
    assert!(!taken);
    assert!(rec(&w, ALPHA, CURTAIN).ready, "the charge is not spent");
}

// spec: S-0513 claims 1 and 3 (a fire order is not checked against the cursor's offers again, and no kind checks its target cell at launch) and S-0512 claim 10 (the warp uses a coordinate outside the playable area); nocite: our own input validation (#219): the window's corners are taken and carried through without an overflow
#[test]
fn every_kind_fires_at_the_window_corners_without_a_panic() {
    let r = ruleset();
    for (building, kinds) in [
        ("SILO", &[STRIKE][..]),
        ("IRONB", &[CURTAIN][..]),
        ("WEATHERB", &[STORM][..]),
        ("CHRONOB", &[SOURCE, WARP][..]),
        ("AIRB", &[DROP, SPY][..]),
        ("SHIELDB", &[SHIELD][..]),
    ] {
        for c in corners() {
            let mut w = game(&r);
            spawn(&mut w, building, ALPHA, -6, -6);
            ready_tick(&mut w, ALPHA, kinds[0], 400);
            for &k in kinds {
                fire_now(&mut w, ALPHA, k, c);
            }
            run(&mut w, 150);
        }
    }
    // The second stage from a house without superweapons, at each corner.
    for c in corners() {
        let mut w = game(&r);
        fire_now(&mut w, BETA, WARP, c);
        run(&mut w, 150);
    }
}
