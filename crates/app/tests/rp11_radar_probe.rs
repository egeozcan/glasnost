//! Probe for render perf batch RP11 (#137, never in CI): the session's radar
//! picture tick by tick in a played game's late state. With
//! `GLASNOST_GAME_DIR` set, it starts the frame benchmark's game (map index
//! 9, seed 7, 8 AI houses) without drawing, runs 9 000 ticks, then times
//! `Radar::rgba` for the local house after each of 300 more ticks (each
//! observed first, as the session does), and 200 times more with no tick
//! between. It prints times and a hash of the pictures only, so a run
//! before a change and one after it show the same pictures; then how many
//! shroud marks and explored states change a tick, and the times of the
//! parts a kept picture checks every time.
//!
//! Run with `cargo test --release -p glasnost-app --test rp11_radar_probe --
//! --ignored --nocapture`.

#![cfg(not(target_arch = "wasm32"))]

use std::hash::{Hash, Hasher};
use std::time::Instant;

use glasnost_app::game::bench::{BenchOptions, start_headless};
use glasnost_sim::econ::minimap;

/// The median, the least and the mean of timings in µs.
fn summary(mut t: Vec<f64>) -> (f64, f64, f64) {
    let mean = t.iter().sum::<f64>() / t.len() as f64;
    t.sort_by(f64::total_cmp);
    (t[t.len() / 2], t[0], mean)
}

#[test]
#[ignore = "probe: needs GLASNOST_GAME_DIR"]
fn radar_picture_probe() {
    let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let dir = dir.to_string_lossy().into_owned();
    let (mut client, _) = start_headless(BenchOptions::default(), &dir).expect("the game starts");
    let session = client.session.as_mut().expect("a session");
    for _ in 0..9000 {
        session.play.game.step();
    }
    let house = session.play.local;
    let radar = session.radar.as_mut().expect("a radar");
    let game = &mut session.play.game;
    radar.observe(&game.world, house);
    let available = house.is_some_and(|h| glasnost_sim::econ::radar::available(&game.world, h));
    let first = radar.rgba(&game.world, house);
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    let mut ticked = Vec::new();
    let marks = |w: &glasnost_sim::world::World| -> Vec<u8> {
        house
            .and_then(|h| w.econ.shroud.houses.get(usize::from(h.0)).cloned().flatten())
            .map(|s| s.marks)
            .unwrap_or_default()
    };
    let (mut changed, mut turned) = (0usize, 0usize);
    for _ in 0..300 {
        let before = marks(&game.world);
        let explored_before: Vec<bool> = cells_of(&game.world)
            .iter()
            .map(|&c| explored(&game.world, house, c))
            .collect();
        game.step();
        let after = marks(&game.world);
        changed += before.iter().zip(&after).filter(|(a, b)| a != b).count();
        turned += cells_of(&game.world)
            .iter()
            .zip(&explored_before)
            .filter(|&(&c, &e)| explored(&game.world, house, c) != e)
            .count();
        radar.observe(&game.world, house);
        let s = Instant::now();
        let px = radar.rgba(&game.world, house);
        ticked.push(s.elapsed().as_secs_f64() * 1e6);
        px.hash(&mut hasher);
    }
    let mut still = Vec::new();
    for _ in 0..200 {
        let s = Instant::now();
        let px = std::hint::black_box(radar.rgba(&game.world, house));
        still.push(s.elapsed().as_secs_f64() * 1e6);
        px.hash(&mut hasher);
    }
    println!(
        "radar picture probe: house {house:?}, radar {available}, from tick {}, {} bytes a picture, picture hash {:016x}",
        game.world.tick - 300,
        first.len(),
        hasher.finish()
    );
    let (med, min, mean) = summary(ticked);
    println!(
        "radar picture probe: after a tick median {med:.1} us, least {min:.1} us, mean {mean:.1} us"
    );
    let (med, min, mean) = summary(still);
    println!(
        "radar picture probe: with no tick median {med:.1} us, least {min:.1} us, mean {mean:.1} us"
    );
    println!(
        "radar picture probe: per tick {:.1} shroud marks changed, {:.1} cells' explored state turned",
        changed as f64 / 300.0,
        turned as f64 / 300.0
    );
    // The parts of a picture with no tick, over the cells the pixels show.
    let w = &game.world;
    let g = minimap::grid(w).expect("usable cells");
    let img = minimap::image(&g);
    let cells = minimap::pixel_cells(&g, &img, w.rules().quirks.radar_pixel_cell_shift);
    let kept: Vec<Option<(i8, u8)>> = cells
        .iter()
        .map(|&c| w.cells.cell(c).map(|x| (x.level, x.slope)))
        .collect();
    let (ground, _, _) = summary(
        (0..200)
            .map(|_| {
                let s = Instant::now();
                let n = cells
                    .iter()
                    .zip(&kept)
                    .filter(|&(&c, k)| w.cells.cell(c).map(|x| (x.level, x.slope)) != *k)
                    .count();
                std::hint::black_box(n);
                s.elapsed().as_secs_f64() * 1e6
            })
            .collect(),
    );
    let (gaps, _, _) = summary(
        (0..200)
            .map(|_| {
                let s = Instant::now();
                let n = w
                    .objects
                    .values()
                    .filter(|o| o.econ.sight.gap.is_some())
                    .count();
                std::hint::black_box(n);
                s.elapsed().as_secs_f64() * 1e6
            })
            .collect(),
    );
    let now = marks(w);
    let old = now.clone();
    let (compare, _, _) = summary(
        (0..200)
            .map(|_| {
                let s = Instant::now();
                std::hint::black_box(old == now);
                s.elapsed().as_secs_f64() * 1e6
            })
            .collect(),
    );
    println!(
        "radar picture probe: parts median: ground check of {} pixels {ground:.1} us, gap scan of {} objects {gaps:.1} us, {} marks compared {compare:.1} us",
        cells.len(),
        w.objects.len(),
        now.len()
    );
}

/// Every legal cell of the world.
fn cells_of(w: &glasnost_sim::world::World) -> Vec<glasnost_map::CellXY> {
    w.cells.grid.iter().map(|(c, _)| c).collect()
}

fn explored(
    w: &glasnost_sim::world::World,
    house: Option<glasnost_sim::world::HouseId>,
    c: glasnost_map::CellXY,
) -> bool {
    house.is_none_or(|h| glasnost_sim::econ::shroud::explored(w, h, c))
}
