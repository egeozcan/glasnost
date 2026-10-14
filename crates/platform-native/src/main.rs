//! Native desktop entry point (winit + wgpu: Vulkan, Metal, DX12 or GL).
//!
//! **Milestone M0**: runs the palette-quad demo. Later milestones add the
//! file-system importer and the game itself; the shell lives in `glasnost-app`
//! and is shared with the web build.
//!
//! Usage: `glasnost [--game | --viewer [--map NAME]] [--game-dir DIR] [--backend auto|gl|vulkan] [--frames N]`.
//! With `--game`, the game client (M5b: menu, skirmish setup, a game against
//! the AI, end screen), reading the install in `DIR`. With `--viewer`, the
//! asset viewer (optionally mounting `DIR` at start, and opening the map
//! viewer on the first map whose name contains `NAME`). Without either, the
//! M0 palette-quad demo.

#[cfg(not(target_arch = "wasm32"))]
fn main() -> std::process::ExitCode {
    use glasnost_app::{DemoOptions, GameOptions, ViewerOptions};
    use std::process::ExitCode;

    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info,wgpu_core=warn,wgpu_hal=warn"),
    )
    .init();

    let mut options = DemoOptions::default();
    let mut viewer = false;
    let mut game = false;
    let mut game_dir = None;
    let mut open_map = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = |name: &str| args.next().ok_or_else(|| format!("{name} needs a value"));
        let parsed = match arg.as_str() {
            "--backend" => value("--backend").and_then(|v| v.parse().map(|b| options.backend = b)),
            "--frames" => value("--frames").and_then(|v| {
                v.parse()
                    .map(|n| options.exit_after_frames = Some(n))
                    .map_err(|e| format!("--frames: {e}"))
            }),
            "--viewer" => {
                viewer = true;
                Ok(())
            }
            "--game" => {
                game = true;
                Ok(())
            }
            "--game-dir" => value("--game-dir").map(|v| game_dir = Some(v)),
            "--map" => value("--map").map(|v| open_map = Some(v)),
            "-h" | "--help" => {
                println!(
                    "usage: glasnost [--game | --viewer [--map NAME]] [--game-dir DIR] [--backend auto|gl|vulkan] [--frames N]"
                );
                return ExitCode::SUCCESS;
            }
            other => Err(format!("unknown argument {other:?}")),
        };
        if let Err(e) = parsed {
            eprintln!("glasnost: {e}");
            return ExitCode::from(2);
        }
    }

    if game && viewer {
        eprintln!("glasnost: --game and --viewer exclude each other");
        return ExitCode::from(2);
    }
    let result = if game {
        glasnost_app::run_game(GameOptions {
            backend: options.backend,
            game_dir,
            exit_after_frames: options.exit_after_frames,
        })
    } else if viewer {
        glasnost_app::run_viewer(ViewerOptions {
            backend: options.backend,
            game_dir,
            exit_after_frames: options.exit_after_frames,
            open_map,
        })
    } else {
        glasnost_app::run_demo(options)
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("glasnost: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {}
