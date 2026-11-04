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
//!   `local-lane/ai-sides.txt`; `GLASNOST_AI_SIDES=a,b` plays only that
//!   pair of sides (`a,a`: the first two open countries of side a).
//! - `GLASNOST_GAME_DIR=<install> glasnost-headless side-audit [side] [map
//!   index] [seed]`: every type the side can build, its flags, and a sandbox
//!   game that offers, builds and places or releases each in turn; kinds,
//!   flags and ticks only, written to the git-ignored `local-lane/side-audit.txt`.
//! - `GLASNOST_GAME_DIR=<install> glasnost-headless triggers [index …]`: the
//!   map trigger event and action codes every map of the install uses
//!   (S-0011), with the listed map indices' own counts; codes and counts
//!   only, written to the git-ignored `local-lane/trigger-codes.txt`.
//! - `glasnost-headless arena [options] "A:TYPE*n,…" "B:TYPE*n,…"`: arena
//!   mode (M7b, `glasnost_tools::arena`): two groups meet on our own flat
//!   map over N seeds; prints win rates, times to kill and survivors. On
//!   the synthetic ruleset, or on the install's reference ruleset when
//!   `GLASNOST_GAME_DIR` is set (then also written to the git-ignored
//!   `local-lane/arena.txt`). `arena --records` runs the fenced-block
//!   arena records of that module's alternative format, if any.
//! - `GLASNOST_GAME_DIR=<install> glasnost-headless arena --record <ID|all>
//!   [options]`: the oracle arena (`glasnost_tools::oracle`): the lead's
//!   arena records (M-0001 to M-0003: front matter plus the sibling CSV)
//!   played in our engine with their setup and compared per pairing;
//!   writes the git-ignored `local-lane/m7b/arena/ours-<ID>.csv` and
//!   `oracle-<ID>.txt`. `--record M-0021` (and `all`, with no other
//!   option) plays M-0021's test pairing on M-0001's ground and judges each
//!   row of its Result (`glasnost_tools::oracle_m0021`); its report, with
//!   the record's local values, goes to `oracle-M-0021.txt`.
//! - `glasnost-headless desync <bundle A> <bundle B>`: two peers' desync
//!   reports (M9 N4, `glasnost_tools::desync`) compared: the first
//!   differing tick and a field-by-field diff of objects, houses, the
//!   game-wide state and our AI's state. On the synthetic ruleset, or on
//!   the install's when `GLASNOST_GAME_DIR` is set. Exit code 0 when the
//!   two agree, 1 when they differ, 2 when they cannot be compared.
//! - `glasnost-headless net-client --relay URL (--host N | --join CODE)
//!   [options]`: a windowless client that takes a row in a room as a
//!   person and plays it with our AI (M9 N6, `glasnost_tools::net_client`),
//!   on the install of `GLASNOST_GAME_DIR` or `--game-dir`, or on the
//!   synthetic one (`--synthetic`).
//! - `glasnost-headless soak --headless BIN [options]`: the local 2v2 of
//!   four net-clients behind bad networks (M9 N6, `glasnost_tools::soak`):
//!   starts the relay (`cargo run --release -p glasnost-relay`), plays, and
//!   compares every client's checkpoint hashes; writes counts and timings
//!   to the git-ignored `local-lane/m9/n6/` (or `--out`). Exit code 0 when
//!   it passed.
//! - `GLASNOST_GAME_DIR=<install> glasnost-headless tick-bench [key=value
//!   …]`: the tick benchmark (M8 C4 part 2, `glasnost_tools::tick_bench`):
//!   the frame benchmark's AI game, ticks timed with nothing drawn, and
//!   with `probe=on` split by the tick's parts (with `searches=on` also
//!   every path computation, M8 C4 part 5); writes a summary, a
//!   per-tick CSV and the state hashes to the git-ignored
//!   `local-lane/m8/c4-2/` (or `out=`). `tick-bench synthetic [ticks]
//!   [probe=on]` times the synthetic scripted replay instead.
//! - `glasnost-headless path-report FILE [--out DIR] [--unit ID]… [--tick
//!   T] [--names] [--any-build]`: a pathing report the client saved (M8
//!   A4, `glasnost_tools::path_report`) replayed with the movement trace
//!   on; writes `report.txt` and `paths.png` (and `state-T.txt` with
//!   `--tick`) to `--out` or a folder named after the file next to it. On
//!   the synthetic ruleset, or on the install's when `GLASNOST_GAME_DIR`
//!   is set. Exit code 0 when the replay reproduced the capture's state
//!   hash, 1 when it did not (the outputs are still written), 2 when the
//!   report cannot be read or does not match this build or the install.

use std::process::ExitCode;

use glasnost_sim::save::Replay;
use glasnost_tools::local_lane::GAME_DIR_VAR;
use glasnost_tools::{
    arena, desync, headless, net_client, oracle, path_report, skirmish_lane, soak, tick_bench,
    trigger_lane,
};

fn arg<T: std::str::FromStr>(args: &[String], i: usize, default: T) -> T {
    args.get(i).and_then(|a| a.parse().ok()).unwrap_or(default)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--version" | "-V") => {
            println!("glasnost-headless {}", glasnost_app::game::client::BUILD);
            ExitCode::SUCCESS
        }
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
        Some("side-audit") => {
            let Some(dir) = std::env::var_os(GAME_DIR_VAR) else {
                eprintln!("glasnost-headless side-audit: set {GAME_DIR_VAR} to an install folder");
                return ExitCode::from(2);
            };
            match glasnost_tools::side_audit::run(
                std::path::Path::new(&dir),
                arg(&args, 1, 2),
                arg(&args, 2, 0),
                arg(&args, 3, 7),
            ) {
                Ok(report) => {
                    print!("{report}");
                    let out = std::path::Path::new("local-lane");
                    if std::fs::create_dir_all(out).is_ok()
                        && std::fs::write(out.join("side-audit.txt"), &report).is_ok()
                    {
                        println!("wrote local-lane/side-audit.txt (git-ignored; never commit it)");
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
        Some("arena") if args[1..].iter().any(|a| a == "--record") => {
            let cli = match oracle::parse_cli(&args[1..]) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!(
                        "glasnost-headless arena: {e}\nusage: glasnost-headless {}",
                        oracle::USAGE
                    );
                    return ExitCode::from(2);
                }
            };
            let dir = std::env::var_os(GAME_DIR_VAR);
            match oracle::run_cli(&cli, dir.as_deref().map(std::path::Path::new)) {
                Ok(report) => {
                    print!("{report}");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("glasnost-headless arena: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        Some("arena") => {
            let cli = match arena::parse_cli(&args[1..]) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!(
                        "glasnost-headless arena: {e}\nusage: glasnost-headless {}",
                        arena::USAGE
                    );
                    return ExitCode::from(2);
                }
            };
            let dir = std::env::var_os(GAME_DIR_VAR).filter(|_| !cli.synthetic);
            let rules = match &dir {
                None => arena::ArenaRules::Synthetic,
                Some(d) => match arena::reference_rules(std::path::Path::new(d)) {
                    Ok(Ok(r)) => arena::ArenaRules::Reference(r),
                    Ok(Err(e)) => {
                        eprintln!("glasnost-headless arena: the rules did not resolve: {e}");
                        return ExitCode::FAILURE;
                    }
                    Err(e) => {
                        eprintln!(
                            "glasnost-headless arena: could not read the install ({:?})",
                            e.kind()
                        );
                        return ExitCode::FAILURE;
                    }
                },
            };
            match arena::run_cli(&cli, &rules, std::path::Path::new("measurements")) {
                Ok(report) => {
                    print!("{report}");
                    let out = std::path::Path::new("local-lane");
                    if dir.is_some()
                        && std::fs::create_dir_all(out).is_ok()
                        && std::fs::write(out.join("arena.txt"), &report).is_ok()
                    {
                        println!("wrote local-lane/arena.txt (git-ignored; never commit it)");
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("glasnost-headless arena: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        Some("desync") => {
            let (Some(a), Some(b)) = (args.get(1), args.get(2)) else {
                eprintln!("usage: glasnost-headless desync <bundle A> <bundle B>");
                return ExitCode::from(2);
            };
            let dir = std::env::var_os(GAME_DIR_VAR);
            match desync::run(
                std::path::Path::new(a),
                std::path::Path::new(b),
                dir.as_deref().map(std::path::Path::new),
            ) {
                Ok((text, differs)) => {
                    print!("{text}");
                    if differs {
                        ExitCode::FAILURE
                    } else {
                        ExitCode::SUCCESS
                    }
                }
                Err(e) => {
                    eprintln!("glasnost-headless desync: {e}");
                    ExitCode::from(2)
                }
            }
        }
        Some("tick-bench") if args.get(1).map(String::as_str) == Some("synthetic") => {
            let probe = args.iter().any(|a| a == "probe=on");
            let (summary, _) = tick_bench::run_synthetic(7, arg(&args, 2, 2000), probe);
            print!("{summary}");
            ExitCode::SUCCESS
        }
        Some("tick-bench") => {
            let o = match tick_bench::Options::parse(&args[1..]) {
                Ok(o) => o,
                Err(e) => {
                    eprintln!(
                        "glasnost-headless tick-bench: {e}\nusage: glasnost-headless {}",
                        tick_bench::USAGE
                    );
                    return ExitCode::from(2);
                }
            };
            let Some(dir) = std::env::var_os(GAME_DIR_VAR) else {
                eprintln!("glasnost-headless tick-bench: set {GAME_DIR_VAR} to an install folder");
                return ExitCode::from(2);
            };
            match tick_bench::run(&o, &dir.to_string_lossy()) {
                Ok(summary) => {
                    print!("{summary}");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("glasnost-headless tick-bench: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        Some("path-report") => {
            let o = match path_report::parse(&args[1..]) {
                Ok(o) => o,
                Err(e) => {
                    eprintln!(
                        "glasnost-headless path-report: {e}\nusage: glasnost-headless {}",
                        path_report::USAGE
                    );
                    return ExitCode::from(2);
                }
            };
            let dir = std::env::var_os(GAME_DIR_VAR);
            match path_report::run(&o, dir.as_deref().map(std::path::Path::new)) {
                Ok((a, out)) => {
                    print!("{}", a.text);
                    println!("wrote report.txt and paths.png to {}", out.display());
                    if a.reproduced {
                        ExitCode::SUCCESS
                    } else {
                        eprintln!(
                            "glasnost-headless path-report: the replay did not reproduce the capture's state hash"
                        );
                        ExitCode::FAILURE
                    }
                }
                Err(e) => {
                    eprintln!("glasnost-headless path-report: {e}");
                    ExitCode::from(2)
                }
            }
        }
        Some("net-client") => {
            let opts = match net_client::parse(&args[1..]) {
                Ok(o) => o,
                Err(e) => {
                    eprintln!(
                        "glasnost-headless net-client: {e}\nusage: glasnost-headless {}",
                        net_client::USAGE
                    );
                    return ExitCode::from(2);
                }
            };
            net_client::init_log();
            match net_client::run(&opts) {
                Ok(_) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("glasnost-headless net-client: {e}");
                    println!("failed {e}");
                    ExitCode::FAILURE
                }
            }
        }
        Some("soak") if args.get(1).map(String::as_str) == Some("compare") => {
            let (dirs, minutes, ticks) = match soak::parse_compare(&args[2..]) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!(
                        "glasnost-headless soak compare: {e}\nusage: glasnost-headless {}",
                        soak::USAGE
                    );
                    return ExitCode::from(2);
                }
            };
            let c = soak::compare(&dirs, minutes, ticks);
            print!("{}", c.text());
            if c.passed() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Some("soak") => {
            let mut s = match soak::parse(&args[1..]) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!(
                        "glasnost-headless soak: {e}\nusage: glasnost-headless {}",
                        soak::USAGE
                    );
                    return ExitCode::from(2);
                }
            };
            if let Err(e) = std::fs::create_dir_all(&s.out) {
                eprintln!(
                    "glasnost-headless soak: cannot make {}: {e}",
                    s.out.display()
                );
                return ExitCode::from(2);
            }
            // A relay of the soak's own, unless it plays on another
            // (`--relay-url`, a split game).
            let _relay = if s.relay_url.is_none() {
                let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
                match soak::RelayProcess::start(root, &s.out.join("relay-log.txt")) {
                    Ok(r) => {
                        s.relay = r.addr;
                        Some(r)
                    }
                    Err(e) => {
                        eprintln!("glasnost-headless soak: {e}");
                        return ExitCode::from(2);
                    }
                }
            } else {
                None
            };
            match soak::run(&s) {
                Ok(run) => {
                    let text = run.text(&s);
                    print!("{text}");
                    let _ = std::fs::write(s.out.join("soak.txt"), &text);
                    if run.passed() {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::FAILURE
                    }
                }
                Err(e) => {
                    eprintln!("glasnost-headless soak: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        _ => {
            eprintln!(
                "usage: glasnost-headless synthetic [ticks] [seed] | replay <file> | smoke [ticks] [seed] [index] | skirmish [ticks] [seed] [index …] | ai [ticks] [seed] [index …] | ai-sides [ticks] [seed] [index …] | triggers [index …] | desync <bundle A> <bundle B> | {} | {} | {} | {}",
                path_report::USAGE,
                arena::USAGE,
                net_client::USAGE,
                soak::USAGE
            );
            ExitCode::from(2)
        }
    }
}
