//! `cargo xtask gate`: the determinism gate. The synthetic replays
//! (`glasnost_sim::testkit`: scripted movement and combat; `glasnost_ai`:
//! the scripted skirmish) must give the
//! committed per-tick state hashes (`crates/sim-gate/golden/*.txt`) in a
//! native release build and as a wasm32 module under node.

use std::process::Command;

use crate::workspace;

const PACKAGE: &str = "glasnost-sim-gate";
const TARGET: &str = "wasm32-unknown-unknown";

fn run(cmd: &mut Command) -> Result<(), String> {
    let status = cmd.status().map_err(|e| format!("{cmd:?}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{cmd:?} failed ({status})"))
    }
}

/// Runs the gate.
pub fn run_gate() -> Result<(), String> {
    let root = workspace::root();
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    println!("xtask gate: native, release");
    run(Command::new(&cargo).current_dir(&root).args([
        "test",
        "--release",
        "--locked",
        "-p",
        PACKAGE,
        "--",
        "--include-ignored",
    ]))?;
    println!("xtask gate: wasm32 under node");
    run(Command::new(&cargo).current_dir(&root).args([
        "build",
        "--release",
        "--locked",
        "--target",
        TARGET,
        "-p",
        PACKAGE,
        "--lib",
    ]))?;
    let target_dir =
        std::env::var("CARGO_TARGET_DIR").map_or_else(|_| root.join("target"), Into::into);
    let wasm = target_dir
        .join(TARGET)
        .join("release")
        .join("glasnost_sim_gate.wasm");
    // The scenario indices of `glasnost_sim_gate::Scenario::ALL`.
    for (i, name) in ["scripted", "combat", "skirmish"].iter().enumerate() {
        run(Command::new("node")
            .current_dir(&root)
            .arg("xtask/gate.mjs")
            .arg(&wasm)
            .arg(i.to_string())
            .arg(format!("crates/sim-gate/golden/{name}.txt")))?;
    }
    println!("xtask gate: ok");
    Ok(())
}
