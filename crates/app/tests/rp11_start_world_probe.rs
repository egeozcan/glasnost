//! Probe for render perf batch RP11 (#149, never in CI): the memory of the
//! start world the session's live drawing keeps beside the game. With
//! `GLASNOST_GAME_DIR` set, it starts the frame benchmark's game (map index
//! 9, seed 7, 8 AI houses) without drawing and measures, by the resident
//! set before and after a copy is made, the memory a copy of the kept start
//! world takes, of its largest parts, and of the game's world beside it
//! (every copy is kept until the end, so none reuses another's pages). It
//! prints sizes only.
//!
//! Run with `cargo test --release -p glasnost-app --test rp11_start_world_probe
//! -- --ignored --nocapture`.

#![cfg(target_os = "linux")]

use glasnost_app::game::bench::{BenchOptions, start_headless};

/// A line of this process's status, in kB.
fn status_kb(key: &str) -> i64 {
    let s = std::fs::read_to_string("/proc/self/status").expect("the process status");
    s.lines()
        .find_map(|l| l.strip_prefix(key))
        .and_then(|v| v.trim().trim_end_matches("kB").trim().parse().ok())
        .expect("a size")
}

/// The growth of the resident set while `f` makes a copy, which is kept.
fn grows<T>(kept: &mut Vec<Box<dyn std::any::Any>>, f: impl FnOnce() -> T) -> i64
where
    T: 'static,
{
    let before = status_kb("VmRSS:");
    let copy = f();
    let after = status_kb("VmRSS:");
    kept.push(Box::new(copy));
    after - before
}

#[test]
#[ignore = "probe: needs GLASNOST_GAME_DIR"]
fn start_world_probe() {
    let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let dir = dir.to_string_lossy().into_owned();
    let (client, _) = start_headless(BenchOptions::default(), &dir).expect("the game starts");
    let session = client.session.as_ref().expect("a session");
    let start = &session.loaded.live.as_ref().expect("a live drawing").world;
    let game = &session.play.game.world;
    let started = status_kb("VmRSS:");
    let mut kept = Vec::new();
    let whole = grows(&mut kept, || start.clone());
    let objects = grows(&mut kept, || start.objects.clone());
    let cells = grows(&mut kept, || start.cells.clone());
    let econ = grows(&mut kept, || start.econ.clone());
    let regions = grows(&mut kept, || start.regions.clone());
    let game_whole = grows(&mut kept, || game.clone());
    println!(
        "start world probe: resident set after the start {started} kB, peak {} kB",
        status_kb("VmHWM:")
    );
    println!(
        "start world probe: kept start world: {} objects, {} cell indices; a copy {whole} kB (objects {objects} kB, cells {cells} kB, economy {econ} kB, regions {regions} kB)",
        start.objects.len(),
        start.cells.grid.len()
    );
    println!(
        "start world probe: the game's world: {} objects; a copy {game_whole} kB",
        game.objects.len()
    );
    drop(kept);
}
