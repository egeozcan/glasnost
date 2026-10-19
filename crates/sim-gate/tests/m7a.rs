//! The M7a duel of the gate exercises the psychic weapons and the slave-miner
//! economy, and its replay reproduces the final state.

use glasnost_ai::{fixture, fixture_m7a as kit, game::Game};
use glasnost_sim_gate::{M7A_SEED, M7A_TICKS, m7a};

// spec: S-0610 (berserk hits, the psychic sensor, the psychic reveal), S-0402 claim 14 (slave miners), S-0609 claim 2 (gattling stages), S-0601 claim 6 (takeovers), S-0613 claims 2 to 4 and 13 (gas clouds and their hits), S-0605 claims 1 and 7 (infantry absorbed), S-0606 claims 3, 7 and 12 (bunker entries, bunkered vehicles, exits), S-0607 claims 5 and 13 (lifter builds, lift orders, seized and dropped vehicles): the gate's M7a duel uses each of them, in the AI's orders and in the simulation's events, and the game it plays is reproduced by its replay (setup and persons' orders; the AI runs again).
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
        let (s, (hashes, g, ex)) = h.join().expect("the run finished");
        println!(
            "seed {s}: {} ticks, outcome {:?}, missing {:?}",
            hashes.len(),
            g.outcome(),
            ex.missing()
        );
    }
}

/// The margins of the M7a duel over a range of seeds
/// (`GLASNOST_SWEEP=1-16 cargo test --release -p glasnost-sim-gate --test m7a m7a_seed_sweep -- --ignored --nocapture`):
/// per counter of `Exercise::counters` the minimum, median and maximum over
/// the seeds, and per seed whether the game ended before its last tick (a
/// side won early), every counter and the counters below 3. Runs eight
/// seeds at a time.
#[test]
#[ignore = "a sweep of whole games; set GLASNOST_SWEEP=<from>-<to>"]
#[allow(clippy::disallowed_methods, clippy::disallowed_macros)] // a local diagnostic: reads the host environment and prints, no simulation
fn m7a_seed_sweep() {
    let range = std::env::var("GLASNOST_SWEEP").unwrap_or_else(|_| "1-16".into());
    let (a, b) = range.split_once('-').expect("GLASNOST_SWEEP=<from>-<to>");
    let (a, b): (u32, u32) = (a.parse().expect("from"), b.parse().expect("to"));
    let seeds: Vec<u32> = (a..=b).collect();
    let mut runs: Vec<(u32, usize, m7a::Exercise, String)> = Vec::new();
    for chunk in seeds.chunks(8) {
        let hs: Vec<_> = chunk
            .iter()
            .map(|&s| std::thread::spawn(move || (s, m7a::play(s, M7A_TICKS))))
            .collect();
        for h in hs {
            let (s, (hashes, g, ex)) = h.join().expect("the run finished");
            let won = g
                .outcome()
                .map_or("none".into(), |o| format!("{:?}", o.results));
            runs.push((s, hashes.len(), ex, won));
        }
    }
    for (s, n, ex, won) in &runs {
        let low: Vec<String> = ex
            .counters()
            .iter()
            .filter(|&&(_, v)| v < 3)
            .map(|&(name, v)| format!("{name} = {v}"))
            .collect();
        let least = ex.counters().iter().map(|&(_, v)| v).min().unwrap_or(0);
        let early = if *n < usize::try_from(M7A_TICKS).unwrap_or(usize::MAX) {
            "yes"
        } else {
            "no"
        };
        println!(
            "seed {s}: {n} ticks, ended early {early}, results {won}, smallest counter {least}, below 3: {low:?}"
        );
        println!("seed {s} counters: {:?}", ex.counters().map(|(_, v)| v));
    }
    let early = runs
        .iter()
        .filter(|(_, n, _, _)| *n < usize::try_from(M7A_TICKS).unwrap_or(usize::MAX))
        .map(|(s, n, _, _)| format!("{s} (tick {n})"))
        .collect::<Vec<_>>();
    println!(
        "ended early: {} of {} seeds {early:?}",
        early.len(),
        runs.len()
    );
    let names = runs[0].2.counters().map(|(name, _)| name);
    let mut worst = u32::MAX;
    for (i, name) in names.iter().enumerate() {
        let mut v: Vec<u32> = runs
            .iter()
            .map(|(_, _, ex, _)| ex.counters()[i].1)
            .collect();
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
