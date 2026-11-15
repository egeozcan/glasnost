//! The cost of taking objects out of play (#402): a removal and a killing
//! hit, each against a table of many objects. A measurement only, on
//! synthetic data, our own tooling rather than game behaviour: its only
//! assertions are the object counts, so it is ignored by default. Run it
//! in a release build:
//!
//! ```sh
//! cargo test --release -p glasnost-sim --test removal_cost -- --ignored --nocapture
//! ```

// The clock, the microsecond figures and the printing belong to the bench,
// not to the simulation, which only runs the removals timed.
#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::float_arithmetic,
    clippy::cast_precision_loss
)]

use std::time::Instant;

use glasnost_sim::combat::damage::Hit;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, World};
use glasnost_sim::{combat, objects};

/// A world of `side` × `side` infantry units of two houses, run a few
/// ticks; returns it with the units' IDs.
fn world(side: i32) -> (World, Vec<ObjectId>) {
    let size = 100;
    let mut w = testkit::game_with(&testkit::ruleset(), size, 1);
    let mut ids = Vec::new();
    for dx in 0..side {
        for dy in 0..side {
            let c = testkit::mid(size, dx - side / 2, dy - side / 2);
            let house = u8::from((dx + dy) % 2 == 1);
            ids.push(testkit::spawn(
                &mut w,
                Kind::Infantry,
                "WALKER",
                house,
                c,
                0,
            ));
        }
    }
    testkit::run(&mut w, 5);
    (w, ids)
}

/// Every `ids.len() / n`th of `ids`, `n` of them, spread over the table.
fn victims(ids: &[ObjectId], n: usize) -> Vec<ObjectId> {
    ids.iter().copied().step_by(ids.len() / n).take(n).collect()
}

/// A hit no unit survives.
fn killing_hit() -> Hit {
    Hit {
        amount: 1_000_000,
        warhead: None,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: true,
        no_escape: false,
    }
}

/// Mean microseconds of one removal and of one killing hit (a synthetic
/// infantry unit leaves play inside its killing hit, so this includes its
/// removal), `n` victims each, in fresh worlds of the same size.
fn measure(side: i32, n: usize) -> (usize, f64, f64) {
    let (mut w, ids) = world(side);
    let before = w.objects.len();
    let vs = victims(&ids, n);
    let t = Instant::now();
    for &v in &vs {
        objects::remove(&mut w, v);
    }
    let removal = t.elapsed().as_secs_f64() * 1e6 / vs.len() as f64;
    assert_eq!(w.objects.len(), before - vs.len());

    let (mut w, ids) = world(side);
    let vs = victims(&ids, n);
    let hit = killing_hit();
    let t = Instant::now();
    for &v in &vs {
        combat::death::kill(&mut w, v, &hit);
    }
    let kill = t.elapsed().as_secs_f64() * 1e6 / vs.len() as f64;
    assert!(vs.iter().all(|&v| w.obj(v).is_none()));
    (before, removal, kill)
}

/// The smallest and the median of `xs`.
fn min_median(mut xs: Vec<f64>) -> (f64, f64) {
    xs.sort_by(f64::total_cmp);
    (xs[0], xs[xs.len() / 2])
}

#[test]
#[ignore = "a measurement: run in a release build with --ignored --nocapture"]
fn removal_cost_by_object_count() {
    // Warm-up, not reported.
    let _ = measure(10, 10);
    for side in [22, 45, 64] {
        let mut objects = 0;
        let (mut rs, mut ks) = (Vec::new(), Vec::new());
        for _ in 0..5 {
            let (n, r, k) = measure(side, 64);
            objects = n;
            rs.push(r);
            ks.push(k);
        }
        let (r0, r1) = min_median(rs);
        let (k0, k1) = min_median(ks);
        println!(
            "objects {objects:5}: one removal min {r0:7.1} median {r1:7.1} us; one killing hit with its removal min {k0:7.1} median {k1:7.1} us"
        );
    }
}
