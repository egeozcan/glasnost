//! Probe for render perf batch RP9 (#137, never in CI): the radar
//! composition of a played game's late state. With `GLASNOST_GAME_DIR`
//! set, it starts the frame benchmark's game (map index 9, seed 7, 8 AI
//! houses) without drawing, runs 9 000 ticks and times
//! `minimap::compose_kept` for house 0 against a fixed synthetic terrain
//! picture, with the per-pixel shroud test alone beside it. It prints
//! times, counts and a hash of the composed pixels only.
//!
//! Run with `cargo test --release -p glasnost-app --test rp9_radar_probe --
//! --ignored --nocapture`.

#![cfg(not(target_arch = "wasm32"))]

use std::hash::{Hash, Hasher};
use std::time::Instant;

use glasnost_app::game::bench::{BenchOptions, start_headless};
use glasnost_sim::econ::minimap::{self, Arrivals, Bits, Rgb, View};
use glasnost_sim::world::{HouseId, ObjectId};

/// The median and the least of `runs` timings of `f`, in µs.
fn time(runs: usize, mut f: impl FnMut()) -> (f64, f64) {
    let mut t: Vec<f64> = (0..runs)
        .map(|_| {
            let s = Instant::now();
            f();
            s.elapsed().as_secs_f64() * 1e6
        })
        .collect();
    t.sort_by(f64::total_cmp);
    (t[t.len() / 2], t[0])
}

#[test]
#[ignore = "probe: needs GLASNOST_GAME_DIR"]
fn radar_composition_probe() {
    let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let dir = dir.to_string_lossy().into_owned();
    let (mut client, _) = start_headless(BenchOptions::default(), &dir).expect("the game starts");
    let session = client.session.as_mut().expect("a session");
    let game = &mut session.play.game;
    for _ in 0..9000 {
        game.step();
    }
    let w = &game.world;
    let g = minimap::grid(w).expect("usable cells");
    let img = minimap::image(&g);
    let shift = w.rules().quirks.radar_pixel_cell_shift;
    let cells = minimap::pixel_cells(&g, &img, shift);
    let terrain: Vec<Rgb> = (0..img.width * img.height)
        .map(|i| [(i % 251) as u8, (i % 113) as u8, (i % 37) as u8])
        .collect();
    let h = HouseId(0);
    let mut arr = Arrivals::default();
    arr.observe(w, h, &g, &img);
    let colour = |h: HouseId| [40 + 20 * h.0, 200, 10];
    let flash = |_: ObjectId| 0;
    let view = View {
        house: h,
        house_colour: &colour,
        neutral: [7, 7, 7],
        flash: &flash,
        flash_frame_time: 1,
        bits: Bits([5, 6, 5]),
        on_flat_grid: true,
    };
    let pixels = minimap::compose_kept(w, &view, &arr, &g, &img, &terrain, &cells);
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    pixels.hash(&mut hasher);
    let listed: std::collections::BTreeSet<(i32, i32)> = arr.at.keys().map(|&(_, p)| p).collect();
    println!(
        "radar probe: tick {}, image {} x {}, {} arrivals at {} pixels, composition hash {:016x}",
        w.tick,
        img.width,
        img.height,
        arr.at.len(),
        listed.len(),
        hasher.finish()
    );
    for round in 0..2 {
        let (med, min) = time(200, || {
            std::hint::black_box(minimap::compose_kept(
                w, &view, &arr, &g, &img, &terrain, &cells,
            ));
        });
        println!("radar probe {round}: compose_kept median {med:.1} us, least {min:.1} us");
        let (med, min) = time(200, || {
            let mut n = 0usize;
            for &c in &cells {
                n += usize::from(minimap::cell_shrouded_on(w, h, c, true));
            }
            std::hint::black_box(n);
        });
        println!("radar probe {round}: shroud tests alone median {med:.1} us, least {min:.1} us");
        let none = Arrivals::default();
        let (med, min) = time(200, || {
            std::hint::black_box(minimap::compose_kept(
                w, &view, &none, &g, &img, &terrain, &cells,
            ));
        });
        println!(
            "radar probe {round}: compose_kept with no arrival median {med:.1} us, least {min:.1} us"
        );
        let (med, _) = time(200, || {
            let mut n = 0usize;
            for &c in &cells {
                n += usize::from(w.cells.usable(c));
            }
            std::hint::black_box(n);
        });
        let (med2, _) = time(200, || {
            let mut n = 0i64;
            for &c in &cells {
                n += i64::from(w.surface_at(c.x * 256 + 128, c.y * 256 + 128));
            }
            std::hint::black_box(n);
        });
        let (med3, _) = time(200, || {
            let mut n = 0usize;
            for &c in &cells {
                let p = glasnost_sim::world::Pos::new(c.x * 256 + 128, c.y * 256 + 128, 0);
                n += usize::from(glasnost_sim::econ::shroud::point_shrouded_on(w, h, p, true));
            }
            std::hint::black_box(n);
        });
        println!(
            "radar probe {round}: parts median: usable {med:.1} us, surface {med2:.1} us, point test at height 0 {med3:.1} us"
        );
    }
}
