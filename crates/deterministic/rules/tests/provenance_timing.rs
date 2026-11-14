//! Load time of a large synthetic layer (#397). Ignored: it prints a figure
//! for the report and asserts nothing. Run with
//! `cargo test --release -p glasnost-rules --test provenance_timing -- --ignored --nocapture`.

// A host-side measurement in a test that prints its figure, never in the
// simulation.
#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use std::time::Instant;

use glasnost_rules::resolve::{Resolver, Source};

/// A layer of `n` objects of four leaves each, one leaf per line.
fn layer(n: usize) -> String {
    let mut s = String::from("{\n");
    for i in 0..n {
        s.push_str(&format!(
            "  \"k{i}\": {{\n    \"a\": {i},\n    \"b\": \"t{i}\",\n    \"c\": true,\n    \"d\": [{i}, 1]\n  }},\n"
        ));
    }
    s.push_str("  \"end\": 0\n}\n");
    s
}

// nocite: host load-time figure for our own rules format, not game behaviour
#[test]
#[ignore]
fn large_layer_load_time() {
    for n in [2_000usize, 20_000] {
        let text = layer(n);
        let mut best = u128::MAX;
        for _ in 0..5 {
            let t0 = Instant::now();
            let mut r = Resolver::default();
            r.apply(&Source {
                layer: "mod 1: timing",
                file: "timing.jsonc",
                text: &text,
            })
            .unwrap();
            let (_, provenance) = r.finish().unwrap();
            std::hint::black_box(&provenance);
            best = best.min(t0.elapsed().as_micros());
        }
        println!(
            "layer of {n} objects ({} bytes, {} lines): best of 5 = {best} us",
            text.len(),
            text.lines().count()
        );
    }
}
