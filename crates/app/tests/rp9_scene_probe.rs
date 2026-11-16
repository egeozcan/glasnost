//! Probes for render perf batch RP9 (#149, #140, never in CI).
//!
//! `shp_header_parse_probe` times `Shp::parse` of synthetic sprite files of
//! several frame counts beside the decode of one frame, the cost a scene
//! build pays again for each frame it decodes the first time.
//!
//! `flying_piece_census` (needs `GLASNOST_GAME_DIR`) starts the frame
//! benchmark's game (map index 9, seed 7, 8 AI houses) without drawing,
//! runs 9 000 ticks, then counts over the bench's 2 641 ticks the debris
//! pieces in play and those whose turn count changed since the tick
//! before (the ones a frame draws again). It prints counts only.
//!
//! Run with `cargo test --release -p glasnost-app --test rp9_scene_probe --
//! --ignored --nocapture`.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;
use std::time::Instant;

use glasnost_fmt::image::IndexedImage;
use glasnost_fmt::shp::{Shp, ShpWriter, WriteFrame};

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
#[ignore = "probe: timings only"]
fn shp_header_parse_probe() {
    for count in [16usize, 128, 512, 2048] {
        let frames = (0..count)
            .map(|i| WriteFrame {
                x: 2,
                y: 3,
                image: IndexedImage::new(40, 30, (0..1200).map(|p| ((p + i) % 7) as u8).collect())
                    .expect("sizes agree"),
                compression: 3,
            })
            .collect();
        let bytes = ShpWriter {
            width: 48,
            height: 40,
            frames,
        }
        .write();
        let (parse, parse_min) = time(500, || {
            std::hint::black_box(Shp::parse(std::hint::black_box(&bytes)).expect("parses"));
        });
        let shp = Shp::parse(&bytes).expect("parses");
        let (decode, decode_min) = time(500, || {
            std::hint::black_box(shp.decode_frame(count / 2).expect("decodes"));
        });
        println!(
            "shp probe: {count} frames, {} bytes: parse median {parse:.2} us (least {parse_min:.2}), one 40 x 30 frame's decode median {decode:.2} us (least {decode_min:.2})",
            bytes.len()
        );
    }
}

#[test]
#[ignore = "probe: needs GLASNOST_GAME_DIR"]
fn flying_piece_census() {
    let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let dir = dir.to_string_lossy().into_owned();
    let (mut client, _) = glasnost_app::game::bench::start_headless(
        glasnost_app::game::bench::BenchOptions::default(),
        &dir,
    )
    .expect("the game starts");
    let session = client.session.as_mut().expect("a session");
    let game = &mut session.play.game;
    for _ in 0..9000 {
        game.step();
    }
    let mut last: BTreeMap<_, i32> = BTreeMap::new();
    let (mut held, mut turned, mut most_held, mut most_turned, mut ticks_with) =
        (0u64, 0u64, 0, 0, 0u64);
    for _ in 0..2641 {
        game.step();
        let now: BTreeMap<_, i32> = game
            .world
            .pieces
            .iter()
            .map(|(id, p)| (*id, p.turns))
            .collect();
        let t = now
            .iter()
            .filter(|&(id, n)| last.get(id) != Some(n))
            .count();
        held += now.len() as u64;
        turned += t as u64;
        most_held = most_held.max(now.len());
        most_turned = most_turned.max(t);
        ticks_with += u64::from(t > 0);
        last = now;
    }
    println!(
        "piece census: over 2641 ticks {held} piece-ticks ({:.2} a tick, most {most_held}), {turned} drawn again ({:.2} a tick, most {most_turned}), {ticks_with} ticks with one or more",
        held as f64 / 2641.0,
        turned as f64 / 2641.0
    );
}
