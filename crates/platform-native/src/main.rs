//! Native desktop entry point (winit + wgpu: Vulkan, Metal, DX12 or GL).
//!
//! **Milestone M0**: runs the palette-quad demo. Later milestones add the
//! file-system importer and the game itself; the shell lives in `glasnost-app`
//! and is shared with the web build.
//!
//! Usage: `glasnost [--game | --viewer [--map NAME]] [--game-dir DIR] [--relay URL] [--relay-only] [--ice URL] [--backend auto|gl|vulkan] [--frames N]`.
//! With `--game`, the game client (M5b: menu, skirmish setup, a game against
//! the AI, end screen), reading the install in `DIR`. With `--viewer`, the
//! asset viewer (optionally mounting `DIR` at start, and opening the map
//! viewer on the first map whose name contains `NAME`). Without either, the
//! M0 palette-quad demo. `--relay URL` fills the multiplayer screen's relay
//! address (default `ws://127.0.0.1:8787/`, the relay binary's own).
//! **M9 N7**: network games try direct links between the peers;
//! `--relay-only` keeps them on the relay, and `--ice URL` (repeatable)
//! names a STUN or TURN server (none by default).
//!
//! In a debug build only, `GLASNOST_PERTURB` makes the game client a test
//! client (M9 N4): it names another build, so it plays only with other
//! test clients, and with a value such as `credits:300:0:100` (house 0's
//! credits +100 before tick 300) or `health:300:5:-1` (object 5's health)
//! it changes its own game to force a desync; `none` changes nothing.
//!
//! **M8 step C4**: `--game --bench SPEC` runs the performance benchmark
//! (`glasnost_app::game::bench`) in the game window and closes when it is
//! done; with `--offscreen` it draws without a window. `SPEC` is `default`
//! or words such as `map=2,late=18000,frames=600`. The outputs go to
//! `local-lane/m8/c4/` under the working directory.

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
    let mut relay = None;
    let mut relay_only = false;
    let mut ice: Vec<String> = Vec::new();
    let mut bench = None;
    let mut offscreen = false;
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
            "--relay" => value("--relay").map(|v| relay = Some(v)),
            "--relay-only" => {
                relay_only = true;
                Ok(())
            }
            "--ice" => value("--ice").map(|v| ice.push(v)),
            "--bench" => value("--bench").and_then(|v| {
                glasnost_app::game::bench::BenchOptions::parse(&v).map(|b| bench = Some(b))
            }),
            "--offscreen" => {
                offscreen = true;
                Ok(())
            }
            "-h" | "--help" => {
                println!(
                    "usage: glasnost [--game [--bench SPEC [--offscreen]] | --viewer [--map NAME]] [--game-dir DIR] [--relay URL] [--relay-only] [--ice URL] [--backend auto|gl|vulkan] [--frames N]"
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

    // A test client, in a debug build only (M9 N4; our own netcode).
    #[cfg(debug_assertions)]
    let test = match std::env::var("GLASNOST_PERTURB") {
        Ok(v) if v.trim() == "none" => Some(None),
        Ok(v) => match glasnost_app::Perturb::parse(&v) {
            Ok(p) => Some(Some(p)),
            Err(e) => {
                eprintln!("glasnost: GLASNOST_PERTURB: {e}");
                return ExitCode::from(2);
            }
        },
        Err(_) => None,
    };
    #[cfg(not(debug_assertions))]
    let test = None;

    if game && viewer {
        eprintln!("glasnost: --game and --viewer exclude each other");
        return ExitCode::from(2);
    }
    if (bench.is_some() || offscreen) && !game {
        eprintln!("glasnost: --bench and --offscreen go with --game");
        return ExitCode::from(2);
    }
    if offscreen {
        let (Some(b), Some(dir)) = (bench, game_dir) else {
            eprintln!("glasnost: --offscreen needs --bench SPEC and --game-dir DIR");
            return ExitCode::from(2);
        };
        return match glasnost_app::game::bench::run_offscreen(b, &dir, options.backend) {
            Ok(summary) => {
                println!("{summary}");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("glasnost: benchmark: {e}");
                ExitCode::FAILURE
            }
        };
    }
    let result = if game {
        // The game plays sound on the system's default output device (a
        // build with the default feature `audio`); without it, or without a
        // device, it runs silent and logs why.
        glasnost_audio::output::allow_device();
        glasnost_app::run_game(GameOptions {
            backend: options.backend,
            game_dir,
            exit_after_frames: options.exit_after_frames,
            relay,
            connect: None,
            direct: None,
            relay_only,
            ice: Some(ice.join(" ")).filter(|i| !i.is_empty()),
            report: None,
            test,
            bench,
            probe: false,
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

// The default feature `audio` is what builds the device output into the
// desktop game (checked when this crate compiles); `--no-default-features`
// leaves it out.
#[cfg(all(feature = "audio", not(target_arch = "wasm32")))]
const _: () = assert!(glasnost_audio::output::DEVICE_BUILT);

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    /// Whatever the features, nothing opens a device before the game calls
    /// `allow_device`.
    #[test]
    fn no_device_opens_before_the_game_allows_it() {
        assert!(glasnost_audio::output::known_silent().is_some());
    }
}
