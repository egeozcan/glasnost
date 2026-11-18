//! A release probe of `linked::recruit`'s cost (#339's rest): the fix of
//! S-0518 claim 6 scans the update order twice per recruitment, to tell
//! whether the supporter takes its turns before the master. The probe times
//! a recruitment, those two scans alone, and a whole tick, with `n` filler
//! vehicles ahead of both prism buildings in the update order (the scans'
//! worst case). Synthetic data: the `linked_common` fixture (prism type
//! PRISM, target type VICT). Timings only; it checks no behaviour.
//!
//! `cargo test --release -p glasnost-sim --test perf_linked_recruit -- --ignored --nocapture`

#[path = "linked_common/mod.rs"]
mod linked_common;

use std::hint::black_box;
use std::time::Instant;

use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{ObjectId, World};
use linked_common::*;

/// Recruitments timed per size.
const RECRUITS: u32 = 20_000;
/// Ticks timed per size.
const TICKS: u32 = 20;

fn quiet(w: &mut World, dx: i32, dy: i32) -> ObjectId {
    let b = spawn(w, Kind::Building, "PRISM", 0, at(dx, dy));
    let tick = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    if let Some(o) = w.obj_mut(b) {
        o.beh.force(Behaviour::Sleep, tick, stale);
    }
    b
}

/// Undoes a recruitment, so that the next one finds the same supporter.
fn reset(w: &mut World, m: ObjectId, s: ObjectId) {
    w.obj_mut(m).unwrap().combat.support = 0;
    let o = w.obj_mut(s).unwrap();
    o.combat.support_beam = None;
    o.combat.rearm_until = 0;
}

#[test]
#[ignore = "release timing probe for #339; prints figures only"]
fn issue_339_recruit_cost() {
    for n in [0usize, 1000, 3000] {
        let mut w = world(|_| {});
        let mut k = 0;
        for dy in -30..=30 {
            for dx in -30..=30 {
                if k < n && (dx < -3 || dx > 3 || dy < -3 || dy > 3) {
                    spawn(&mut w, Kind::Vehicle, "VICT", 1, at(dx, dy));
                    k += 1;
                }
            }
        }
        assert_eq!(k, n);
        let m = quiet(&mut w, 0, 0);
        let s = quiet(&mut w, -1, 0);
        run_to(&mut w, 5);
        let order = w.update.order.len();

        let t = Instant::now();
        for _ in 0..RECRUITS {
            reset(&mut w, m, s);
            assert!(glasnost_sim::linked::recruit(&mut w, black_box(m)));
        }
        let recruit = t.elapsed() / RECRUITS;
        reset(&mut w, m, s);

        let t = Instant::now();
        let mut sum = 0usize;
        for _ in 0..RECRUITS {
            let o = &w.update.order;
            sum += o.iter().position(|&x| x == black_box(s)).unwrap_or(0);
            sum += o.iter().position(|&x| x == black_box(m)).unwrap_or(0);
        }
        black_box(sum);
        let scans = t.elapsed() / RECRUITS;

        let t = Instant::now();
        for _ in 0..TICKS {
            glasnost_sim::tick::step(&mut w);
        }
        let tick = t.elapsed() / TICKS;

        println!(
            "filler {n}: update order {order} objects; recruit {recruit:?}, of which the two order scans {scans:?}; one tick {tick:?}"
        );
    }
}
