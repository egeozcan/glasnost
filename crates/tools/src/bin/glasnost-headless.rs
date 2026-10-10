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

use std::process::ExitCode;

use glasnost_sim::save::Replay;
use glasnost_tools::headless;
use glasnost_tools::local_lane::GAME_DIR_VAR;

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
        _ => {
            eprintln!(
                "usage: glasnost-headless synthetic [ticks] [seed] | replay <file> | smoke [ticks] [seed] [index]"
            );
            ExitCode::from(2)
        }
    }
}
