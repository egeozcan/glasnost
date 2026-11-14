//! The M6c duel of the gate exercises every special ability, and its replay
//! reproduces the final state.

use glasnost_ai::{fixture, fixture_m6c as kit, game::Game};
use glasnost_rules::layers::Layers;
use glasnost_sim_gate::{M6C_SEED, M6C_TICKS, m6c};

// spec: S-0508 (garrisons), S-0509 (engineers and capture), S-0510 (spies and infiltration), S-0515 (tech buildings), S-0502 (jumpjets), S-0517 (deployable units), S-0525 (radiation fields), S-0514 (chrono weapons), S-0518 (linked weapons), S-0519 (terror and strikes), S-0520 (crates): the gate's M6c duel uses every one of them, in the AI's orders and in the simulation's events, and the game it plays is reproduced by its replay (setup and persons' orders; the AI runs again).
#[test]
#[ignore = "long; run in release by `cargo xtask gate`"]
fn every_m6c_ability_is_exercised_and_the_replay_reproduces_the_game() {
    let (hashes, g, ex) = m6c::play(M6C_SEED, M6C_TICKS);
    println!("{ex:#?}");
    println!("{:?}", g.special_uses());
    assert!(
        ex.missing().is_empty(),
        "the M6c duel did not exercise {:?}",
        ex.missing()
    );
    assert_eq!(hashes.len(), usize::try_from(M6C_TICKS).unwrap());
    let replayed = Game::run_replay(
        g.replay(),
        kit::rules(),
        &kit::map(),
        fixture::catalogue(),
        |_| {},
    )
    .expect("the M6c duel's replay runs");
    assert_eq!(replayed.world.tick, g.world.tick);
    assert_eq!(
        replayed.state_hash(),
        g.state_hash(),
        "the replay reproduces the final state"
    );
}

// spec: S-0510 (spies and infiltration), design: own AI (batch 6): since a spy goes only into an enemy player's building (#39), the duel's spies must reach the enemy base; in the seeds where the sweep on main 5dcbc5c counted no infiltration, a spy infiltrates.
#[test]
#[ignore = "long; run in release (`cargo test --release -p glasnost-sim-gate -- --include-ignored`)"]
fn a_spy_infiltrates_in_the_seeds_main_found_none() {
    let seeds = [5u32, 7, 9, 10, 11, 13, 14, 15];
    let hs: Vec<_> = seeds
        .iter()
        .map(|&s| std::thread::spawn(move || (s, m6c::play(s, M6C_TICKS).2.infiltrations)))
        .collect();
    let none: Vec<u32> = hs
        .into_iter()
        .map(|h| h.join().expect("the run finished"))
        .filter(|&(_, n)| n == 0)
        .map(|(s, _)| s)
        .collect();
    assert!(none.is_empty(), "no infiltration in seeds {none:?}");
}

/// Tries seeds for the M6c duel: `S=1,2,3 cargo test --release -p
/// glasnost-sim-gate --test m6c try_seeds -- --ignored --nocapture` prints,
/// per seed, the abilities that a full run did not exercise (a seed with
/// none is a candidate for `M6C_SEED`). Does nothing without `S`.
#[test]
#[ignore = "a tool for choosing the seed"]
fn try_seeds() {
    let seeds: Vec<u32> = std::env::var("S")
        .unwrap_or_default()
        .split(',')
        .filter_map(|v| v.parse().ok())
        .collect();
    let hs: Vec<_> = seeds
        .iter()
        .map(|&s| std::thread::spawn(move || (s, m6c::play(s, M6C_TICKS))))
        .collect();
    for h in hs {
        let (s, (hashes, _, ex)) = h.join().expect("the run finished");
        println!(
            "seed {s}: {} ticks, missing {:?}",
            hashes.len(),
            ex.missing()
        );
    }
}

/// The margins of the M6c duel over a range of seeds
/// (`GLASNOST_SWEEP=1-16 cargo test --release -p glasnost-sim-gate --test m6c m6c_seed_sweep -- --ignored --nocapture`):
/// per counter of `Exercise::counters` the minimum, median and maximum over
/// the seeds, and per seed the counters below 3. Runs eight seeds at a time.
#[test]
#[ignore = "a sweep of whole games; set GLASNOST_SWEEP=<from>-<to>"]
#[allow(clippy::disallowed_methods, clippy::disallowed_macros)] // a local diagnostic: reads the host environment and prints, no simulation
fn m6c_seed_sweep() {
    let range = std::env::var("GLASNOST_SWEEP").unwrap_or_else(|_| "1-16".into());
    let (a, b) = range.split_once('-').expect("GLASNOST_SWEEP=<from>-<to>");
    let (a, b): (u32, u32) = (a.parse().expect("from"), b.parse().expect("to"));
    let seeds: Vec<u32> = (a..=b).collect();
    // `GLASNOST_PLAY_FIXES=1`: our own fixes layer (M8 step C7); unset, the
    // scenario's reference rules, as the gate plays them.
    let layers = Layers::from_flag(std::env::var("GLASNOST_PLAY_FIXES").ok().as_deref());
    if layers != Layers::REFERENCE {
        println!("rules layers: {} (GLASNOST_PLAY_FIXES)", layers.describe());
    }
    let rules = glasnost_sim_gate::layered(kit::ruleset(), kit::MAP_NAME, layers);
    let mut runs: Vec<(u32, usize, m6c::Exercise)> = Vec::new();
    for chunk in seeds.chunks(8) {
        let hs: Vec<_> = chunk
            .iter()
            .map(|&s| {
                let rules = rules.clone();
                std::thread::spawn(move || (s, m6c::play_with(rules, s, M6C_TICKS)))
            })
            .collect();
        for h in hs {
            let (s, (hashes, _, ex)) = h.join().expect("the run finished");
            runs.push((s, hashes.len(), ex));
        }
    }
    for (s, n, ex) in &runs {
        let low: Vec<String> = ex
            .counters()
            .iter()
            .filter(|&&(_, v)| v < 3)
            .map(|&(name, v)| format!("{name} = {v}"))
            .collect();
        let least = ex.counters().iter().map(|&(_, v)| v).min().unwrap_or(0);
        println!("seed {s}: {n} ticks, smallest counter {least}, below 3: {low:?}");
    }
    let names = runs[0].2.counters().map(|(name, _)| name);
    let mut worst = u32::MAX;
    for (i, name) in names.iter().enumerate() {
        let mut v: Vec<u32> = runs.iter().map(|(_, _, ex)| ex.counters()[i].1).collect();
        v.sort_unstable();
        worst = worst.min(v[0]);
        println!(
            "{name}: min {} median {} max {}",
            v[0],
            v[v.len() / 2],
            v[v.len() - 1]
        );
    }
    println!("smallest minimum over {} seeds: {worst}", runs.len());
}
