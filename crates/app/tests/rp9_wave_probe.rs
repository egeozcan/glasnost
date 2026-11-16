//! Probe for render perf batch RP9 (#141, never in CI): the time of
//! `beams::wave_spots` for three synthetic magnetic waves of about 2 500
//! spots each and one long one, as render perf batch RP5 timed them. It
//! prints times, spot counts and a hash of the spots only.
//!
//! Run with `cargo test --release -p glasnost-app --test rp9_wave_probe --
//! --ignored --nocapture`.

use std::hash::{Hash, Hasher};
use std::time::Instant;

use glasnost_app::mapview::beams::wave_spots;
use glasnost_num::soft::Rounding;
use glasnost_sim::world::Pos;

#[test]
#[ignore = "probe: timings only"]
fn wave_spots_probe() {
    let waves = [
        (Pos::new(20000, 20000, 0), Pos::new(22000, 18000, 120)),
        (Pos::new(20000, 20000, 104), Pos::new(21800, 21900, 0)),
        (Pos::new(20000, 20000, 0), Pos::new(18200, 19400, 200)),
        (Pos::new(30000, 30000, 0), Pos::new(39000, 20500, 300)),
    ];
    for round in 0..2 {
        for (n, (e1, e2)) in waves.iter().enumerate() {
            let spots = wave_spots(*e1, *e2, (1, 0), 63, Rounding::TowardZero);
            let mut h = std::collections::hash_map::DefaultHasher::new();
            format!("{spots:?}").hash(&mut h);
            let mut t: Vec<f64> = (0..200)
                .map(|_| {
                    let s = Instant::now();
                    std::hint::black_box(wave_spots(*e1, *e2, (1, 0), 63, Rounding::TowardZero));
                    s.elapsed().as_secs_f64() * 1e6
                })
                .collect();
            t.sort_by(f64::total_cmp);
            println!(
                "wave probe {round}: wave {n}, {} spots, hash {:016x}, median {:.1} us, least {:.1} us",
                spots.len(),
                h.finish(),
                t[t.len() / 2],
                t[0]
            );
        }
    }
}
