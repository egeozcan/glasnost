//! Probe for render perf batch RP11 (#137, never in CI): the session's radar
//! picture tick by tick in a played game's late state. With
//! `GLASNOST_GAME_DIR` set, it starts the frame benchmark's game (map index
//! 9, seed 7, 8 AI houses) without drawing, runs 9 000 ticks, then times
//! `Radar::rgba` for the local house after each of 300 more ticks (each
//! observed first, as the session does), and 200 times more with no tick
//! between. It prints times and a hash of the pictures only, so a run
//! before a change and one after it show the same pictures.
//!
//! Run with `cargo test --release -p glasnost-app --test rp11_radar_probe --
//! --ignored --nocapture`.

#![cfg(not(target_arch = "wasm32"))]

use std::hash::{Hash, Hasher};
use std::time::Instant;

use glasnost_app::game::bench::{BenchOptions, start_headless};

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
    for _ in 0..300 {
        game.step();
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
}
