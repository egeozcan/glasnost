//! The headless simulation runner.
//!
//! Usage:
//! - `glasnost-headless synthetic [ticks] [seed]`: per-tick state hashes of
//!   the synthetic scripted replay.
//! - `glasnost-headless replay <file>`: per-tick hashes of a replay file over
//!   the synthetic scenario.
//! - `GLASNOST_GAME_DIR=<install> glasnost-headless smoke [ticks] [seed]`:
//!   the local lane; runs every map of the install in the sandbox setup and
//!   writes counts and timings to the git-ignored `local-lane/sim-smoke.txt`.
//!   `smoke [ticks] [seed] [index]` runs only the map with that index.
//! - `GLASNOST_GAME_DIR=<install> glasnost-headless skirmish [ticks] [seed]
//!   [index …]`: the local lane's scripted skirmish (M5a) on the first two
//!   stock maps with two start positions (or the listed map indices), played
//!   to the last side; writes counts and timings to the git-ignored
//!   `local-lane/skirmish.txt`.
//! - `GLASNOST_GAME_DIR=<install> glasnost-headless ai [ticks] [seed]
//!   [index …]`: the same games played by our own AI (M5b,
//!   `glasnost_ai::game::Game`); writes `local-lane/ai-duel.txt`.
//!   `ai-sides [ticks] [seed] [index …]` plays one such game per pair of
//!   sides (each player the first open country of its side) and writes
//!   `local-lane/ai-sides.txt`.
//! - `GLASNOST_GAME_DIR=<install> glasnost-headless triggers [index …]`: the
//!   map trigger event and action codes every map of the install uses
//!   (S-0011), with the listed map indices' own counts; codes and counts
//!   only, written to the git-ignored `local-lane/trigger-codes.txt`.

use std::process::ExitCode;

use glasnost_sim::save::Replay;
use glasnost_tools::local_lane::GAME_DIR_VAR;
use glasnost_tools::{headless, skirmish_lane, trigger_lane};

fn arg<T: std::str::FromStr>(args: &[String], i: usize, default: T) -> T {
    args.get(i).and_then(|a| a.parse().ok()).unwrap_or(default)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("synthetic") => {
            for (t, h) in headless::run_synthetic(arg(&args, 2, 7), arg(&args, 1, 1000))
                .iter()
                .enumerate()
            {
                println!("{t} {h}");
            }
            ExitCode::SUCCESS
        }
        Some("replay") => {
            let Some(path) = args.get(1) else {
                eprintln!("glasnost-headless replay <file>");
                return ExitCode::from(2);
            };
            let rep = std::fs::read(path)
                .map_err(|e| e.to_string())
                .and_then(|b| Replay::from_bytes(&b).map_err(|e| e.to_string()))
                .and_then(|r| headless::replay(&r));
            match rep {
                Ok(hashes) => {
                    for (t, h) in hashes.iter().enumerate() {
                        println!("{t} {h}");
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("glasnost-headless: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        Some("smoke") => {
            let Some(dir) = std::env::var_os(GAME_DIR_VAR) else {
                eprintln!("glasnost-headless smoke: set {GAME_DIR_VAR} to an install folder");
                return ExitCode::from(2);
            };
            let only = args.get(3).and_then(|a| a.parse().ok());
            match headless::smoke(
                std::path::Path::new(&dir),
                arg(&args, 1, 1000),
                arg(&args, 2, 7),
                only,
            ) {
                Ok(report) => {
                    print!("{report}");
                    let out = std::path::Path::new("local-lane");
                    if std::fs::create_dir_all(out).is_ok()
                        && std::fs::write(out.join("sim-smoke.txt"), &report).is_ok()
                    {
                        println!("wrote local-lane/sim-smoke.txt (git-ignored; never commit it)");
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!(
                        "glasnost-headless: could not read the install ({:?})",
                        e.kind()
                    );
                    ExitCode::FAILURE
                }
            }
        }
        Some(cmd @ ("skirmish" | "ai" | "ai-sides")) => {
            let Some(dir) = std::env::var_os(GAME_DIR_VAR) else {
                eprintln!("glasnost-headless {cmd}: set {GAME_DIR_VAR} to an install folder");
                return ExitCode::from(2);
            };
            let (players, file) = match cmd {
                "ai" => (skirmish_lane::Players::Ai, "ai-duel.txt"),
                "ai-sides" => (skirmish_lane::Players::AiSides, "ai-sides.txt"),
                _ => (skirmish_lane::Players::Scripted, "skirmish.txt"),
            };
            let only: Vec<usize> = args.iter().skip(3).filter_map(|a| a.parse().ok()).collect();
            match skirmish_lane::run(
                std::path::Path::new(&dir),
                2,
                &only,
                arg(&args, 1, 120_000),
                arg(&args, 2, 7),
                players,
            ) {
                Ok(report) => {
                    print!("{report}");
                    let out = std::path::Path::new("local-lane");
                    if std::fs::create_dir_all(out).is_ok()
                        && std::fs::write(out.join(file), &report).is_ok()
                    {
                        println!("wrote local-lane/{file} (git-ignored; never commit it)");
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!(
                        "glasnost-headless: could not read the install ({:?})",
                        e.kind()
                    );
                    ExitCode::FAILURE
                }
            }
        }
        Some("triggers") => {
            let Some(dir) = std::env::var_os(GAME_DIR_VAR) else {
                eprintln!("glasnost-headless triggers: set {GAME_DIR_VAR} to an install folder");
                return ExitCode::from(2);
            };
            let detail: Vec<usize> = args.iter().skip(1).filter_map(|a| a.parse().ok()).collect();
            match trigger_lane::run(std::path::Path::new(&dir), &detail) {
                Ok(report) => {
                    print!("{report}");
                    let out = std::path::Path::new("local-lane");
                    if std::fs::create_dir_all(out).is_ok()
                        && std::fs::write(out.join("trigger-codes.txt"), &report).is_ok()
                    {
                        println!(
                            "wrote local-lane/trigger-codes.txt (git-ignored; never commit it)"
                        );
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!(
                        "glasnost-headless: could not read the install ({:?})",
                        e.kind()
                    );
                    ExitCode::FAILURE
                }
            }
        }
        _ => {
            eprintln!(
                "usage: glasnost-headless synthetic [ticks] [seed] | replay <file> | smoke [ticks] [seed] [index] | skirmish [ticks] [seed] [index …] | ai [ticks] [seed] [index …] | ai-sides [ticks] [seed] [index …] | triggers [index …]"
            );
            ExitCode::from(2)
        }
    }
}
