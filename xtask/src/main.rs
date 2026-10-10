//! Repository checks and build helpers: `cargo xtask <command>`.
//!
//! Run `cargo xtask help` for the command list. CI runs `cargo xtask check`.

use std::process::ExitCode;

mod canary;
mod cite;
mod deps;
mod gate;
mod stock_guard;
mod web;
mod workspace;

const HELP: &str = "\
usage: cargo xtask <command>

checks (CI runs `check`, which runs all of them):
  check          every check below
  deps           simulation crates depend on no render/audio/net/fmt/ini/convert/... crate
  determinism    simulation crates share one lint table; no float literals in their code
  cite           behavioural constants in simulation code cite spec:/measure: IDs that exist
  stock-guard    committed files hold no game data, stock values or install paths
  canary         every determinism lint in crates/deterministic/clippy.toml still fires

determinism gate (CI runs it as its own job):
  gate           the scripted replay's per-tick state hashes match the golden file,
                 natively in release and as wasm32 under node (needs node)

web build:
  web [--release] [--serve] [--port N]
                 build the wasm demo into dist/web (needs wasm-bindgen-cli at the
                 version in Cargo.lock) and optionally serve it on 127.0.0.1
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(command) = args.first() else {
        eprint!("{HELP}");
        return ExitCode::from(2);
    };
    let rest = &args[1..];
    let result = match command.as_str() {
        "check" => check_all(),
        "deps" => run_check("deps", deps::run),
        "determinism" => run_check("determinism", cite::run_determinism),
        "cite" => run_check("cite", cite::run),
        "stock-guard" => run_check("stock-guard", stock_guard::run),
        "canary" => run_check("canary", canary::run),
        "gate" => gate::run_gate(),
        "web" => web::run(rest),
        "help" | "-h" | "--help" => {
            print!("{HELP}");
            Ok(())
        }
        other => Err(format!("unknown command {other:?}\n\n{HELP}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

/// A check returns its findings; an empty list means it passed.
type Check = fn() -> Result<Vec<String>, String>;

fn run_check(name: &str, check: Check) -> Result<(), String> {
    let findings = check().map_err(|e| format!("xtask {name}: {e}"))?;
    if findings.is_empty() {
        println!("xtask {name}: ok");
        Ok(())
    } else {
        for f in &findings {
            eprintln!("{f}");
        }
        Err(format!("xtask {name}: {} problem(s)", findings.len()))
    }
}

fn check_all() -> Result<(), String> {
    let checks: [(&str, Check); 5] = [
        ("deps", deps::run),
        ("determinism", cite::run_determinism),
        ("cite", cite::run),
        ("stock-guard", stock_guard::run),
        ("canary", canary::run),
    ];
    let mut failed = Vec::new();
    for (name, check) in checks {
        if let Err(e) = run_check(name, check) {
            eprintln!("{e}");
            failed.push(name);
        }
    }
    if failed.is_empty() {
        Ok(())
    } else {
        Err(format!("xtask check failed: {}", failed.join(", ")))
    }
}
