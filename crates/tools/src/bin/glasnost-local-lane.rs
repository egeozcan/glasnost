//! Runs the local-only lane against the install named by `GLASNOST_GAME_DIR`
//! and prints counts (never names, bytes or paths). Writes the install's
//! `coverage.csv` (key names and outcomes, no values) to the git-ignored
//! `local-lane/` folder of the current directory.
//!
//! Usage: `GLASNOST_GAME_DIR=<install folder> glasnost-local-lane`

use std::process::ExitCode;

use glasnost_tools::local_lane::{self, GAME_DIR_VAR};

fn main() -> ExitCode {
    let Some(dir) = std::env::var_os(GAME_DIR_VAR) else {
        eprintln!("glasnost-local-lane: set {GAME_DIR_VAR} to an install folder");
        return ExitCode::from(2);
    };
    match local_lane::run(std::path::Path::new(&dir)) {
        Ok(report) => {
            print!("{report}");
            // Names only, into the git-ignored local-lane folder.
            let out = std::path::Path::new("local-lane");
            if std::fs::create_dir_all(out).is_ok()
                && std::fs::write(out.join("coverage.csv"), &report.coverage_csv).is_ok()
            {
                println!("wrote local-lane/coverage.csv (git-ignored; never commit it)");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            // The error kind only: io messages can include the path.
            eprintln!(
                "glasnost-local-lane: could not read the install ({:?})",
                e.kind()
            );
            ExitCode::FAILURE
        }
    }
}
