//! The M7a duel of the gate exercises the psychic weapons and the slave-miner
//! economy, and its replay reproduces the final state.

use glasnost_ai::{fixture, fixture_m7a as kit, game::Game};
use glasnost_sim_gate::{M7A_SEED, M7A_TICKS, m7a};

// spec: S-0610 (berserk hits, the psychic sensor, the psychic reveal), S-0402 claim 14 (slave miners): the gate's M7a duel uses each of them, in the AI's orders and in the simulation's events, and the game it plays is reproduced by its replay (setup and persons' orders; the AI runs again).
#[test]
#[ignore = "long; run in release by `cargo xtask gate`"]
fn every_m7a_ability_is_exercised_and_the_replay_reproduces_the_game() {
    let (hashes, g, ex) = m7a::play(M7A_SEED, M7A_TICKS);
    println!("{ex:#?}");
    println!("{:?}", g.special_uses());
    assert!(
        ex.missing().is_empty(),
        "the M7a duel did not exercise {:?}",
        ex.missing()
    );
    assert_eq!(hashes.len(), usize::try_from(M7A_TICKS).unwrap());
    let replayed = Game::run_replay(
        g.replay(),
        kit::rules(),
        &kit::map(),
        fixture::catalogue(),
        |_| {},
    )
    .expect("the M7a duel's replay runs");
    assert_eq!(replayed.world.tick, g.world.tick);
    assert_eq!(
        replayed.state_hash(),
        g.state_hash(),
        "the replay reproduces the final state"
    );
}

/// Tries seeds for the M7a duel: `S=1,2,3 cargo test --release -p
/// glasnost-sim-gate --test m7a try_seeds -- --ignored --nocapture` prints,
/// per seed, the abilities that a full run did not exercise (a seed with
/// none is a candidate for `M7A_SEED`). Does nothing without `S`.
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
        .map(|&s| std::thread::spawn(move || (s, m7a::play(s, M7A_TICKS))))
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

/// The margins of the M7a duel over a range of seeds
/// (`GLASNOST_SWEEP=1-16 cargo test --release -p glasnost-sim-gate --test m7a m7a_seed_sweep -- --ignored --nocapture`):
/// per counter of `Exercise::counters` the minimum, median and maximum over
/// the seeds, and per seed the counters below 3. Runs eight seeds at a time.
#[test]
#[ignore = "a sweep of whole games; set GLASNOST_SWEEP=<from>-<to>"]
#[allow(clippy::disallowed_methods, clippy::disallowed_macros)] // a local diagnostic: reads the host environment and prints, no simulation
fn m7a_seed_sweep() {
    let range = std::env::var("GLASNOST_SWEEP").unwrap_or_else(|_| "1-16".into());
    let (a, b) = range.split_once('-').expect("GLASNOST_SWEEP=<from>-<to>");
    let (a, b): (u32, u32) = (a.parse().expect("from"), b.parse().expect("to"));
    let seeds: Vec<u32> = (a..=b).collect();
    let mut runs: Vec<(u32, usize, m7a::Exercise)> = Vec::new();
    for chunk in seeds.chunks(8) {
        let hs: Vec<_> = chunk
            .iter()
            .map(|&s| std::thread::spawn(move || (s, m7a::play(s, M7A_TICKS))))
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
