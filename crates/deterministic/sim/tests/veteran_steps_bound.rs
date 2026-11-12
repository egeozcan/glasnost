//! A veteran crate's step count does no work past the steps that can
//! change an experience value (#194). Synthetic data only: the
//! `crate_common` test ruleset.

// The watchdog thread and its channel belong to the test harness, not to the
// simulation, which runs on that thread alone.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "crate_common/mod.rs"]
mod crate_common;

use std::sync::mpsc;
use std::time::Duration;

use crate_common::*;
use glasnost_num::{F64, Rounding};
use glasnost_sim::crates;
use glasnost_sim::rules::Kind;

const RZ: Rounding = Rounding::TowardZero;

/// Picks up one veteran crate with argument `arg` next to an unarmed unit
/// of starting experience `start`, on its own thread. Returns the unit's
/// experience afterwards as a number of whole steps, or `None` when the
/// pickup has not returned within ten seconds.
fn pickup(arg: &'static str, start: i64) -> Option<i64> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut w = world_with(false, |r| {
            r.sim.general.economy.powerups.as_mut().unwrap()[14].argument = Some(d(arg));
        });
        let v = spawn(&mut w, Kind::Vehicle, "ARMED", 0, at(0, 0));
        let near = spawn(&mut w, Kind::Vehicle, "UNARMED", 1, at(2, 0));
        w.obj_mut(near).unwrap().combat.experience = F64::from_i64(start, RZ).to_f32(RZ);
        let c = at(1, 0);
        crate_at(&mut w, c, WOOD, 14);
        crates::pickup(&mut w, v, c);
        let e = w.obj(near).unwrap().combat.experience.to_f64();
        let whole = (-1..=3).find(|&n| e.eq_num(F64::from_i64(n, RZ)));
        let _ = tx.send(whole.expect("a whole experience value"));
    });
    rx.recv_timeout(Duration::from_secs(10)).ok()
}

// spec: S-0520 claim 22 and F7 (⌈A⌉ steps; a step maps e < 0 to 0, 0 ≤ e < 1 to 1, 1 ≤ e < 2 to 2, and leaves an elite object unchanged, so every A > 2 gives the result of A = 3)
#[test]
fn a_huge_veteran_argument_gives_the_result_of_three_steps() {
    for start in [-1, 0, 1, 2] {
        let three = pickup("3", start);
        assert_eq!(three, Some(2), "A = 3 from {start}");
        for arg in ["2.5", "1000000000", "1e39"] {
            assert_eq!(
                pickup(arg, start),
                three,
                "A = {arg} from {start}: the pickup did not return, or differs from A = 3"
            );
        }
    }
}

// spec: S-0520 F7 (fewer than three steps still count: A = 1 from −1 gives 0, A = 2 from −1 gives 1)
#[test]
fn small_veteran_arguments_keep_their_counts() {
    assert_eq!(pickup("1", -1), Some(0));
    assert_eq!(pickup("2", -1), Some(1));
    assert_eq!(pickup("0", -1), Some(-1));
}
