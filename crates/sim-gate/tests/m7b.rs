//! The M7b duel of the gate exercises the drain, the genetic mutator and
//! the psychic dominator, and its replay reproduces the final state.

use glasnost_ai::{fixture, fixture_m7b as kit, game::Game};
use glasnost_rules::layers::Layers;
use glasnost_sim_gate::{M7B_SEED, M7B_TICKS, m7b};

// spec: S-0608 claims 1, 2, 5, 7 to 10 and 14 (drain links formed, credits moved by payouts, drained buildings, links ended), S-0611 claims 2 and 4 (mutator launches and cell-mode conversions) and S-0612 claims 1, 4 and 14 (dominator launches, sweeps and takeovers): the gate's M7b duel uses the drain, the mutator and the dominator, in the AI's orders and in the simulation's events, and the game it plays is reproduced by its replay (setup and persons' orders; the AI runs again).
#[test]
#[ignore = "long; run in release by `cargo xtask gate`"]
fn the_drain_is_exercised_and_the_replay_reproduces_the_game() {
    let (hashes, g, ex) = m7b::play(M7B_SEED, M7B_TICKS);
    println!("{ex:#?}");
    println!("{:?}", g.special_uses());
    assert!(
        ex.missing().is_empty(),
        "the M7b duel did not exercise {:?} enough",
        ex.missing()
    );
    assert_eq!(hashes.len(), usize::try_from(M7B_TICKS).unwrap());
    let replayed = Game::run_replay(
        g.replay(),
        kit::rules(),
        &kit::map(),
        fixture::catalogue(),
        |_| {},
    )
    .expect("the M7b duel's replay runs");
    assert_eq!(replayed.world.tick, g.world.tick);
    assert_eq!(
        replayed.state_hash(),
        g.state_hash(),
        "the replay reproduces the final state"
    );
}

/// The margins of the M7b duel over a range of seeds
/// (`GLASNOST_SWEEP=1-16 cargo test --release -p glasnost-sim-gate --test m7b m7b_seed_sweep -- --ignored --nocapture`):
/// per drain counter of `Exercise::counters` (and the M7a counters) the
/// minimum, median and maximum over the seeds, and per seed the ticks
/// played, the result, the first tick with a link and the drain counters
/// below the margin. Runs eight seeds at a time.
#[test]
#[ignore = "a sweep of whole games; set GLASNOST_SWEEP=<from>-<to>"]
#[allow(clippy::disallowed_methods, clippy::disallowed_macros)] // a local diagnostic: reads the host environment and prints, no simulation
fn m7b_seed_sweep() {
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
    let mut runs: Vec<(u32, usize, m7b::Exercise, String)> = Vec::new();
    for chunk in seeds.chunks(8) {
        let hs: Vec<_> = chunk
            .iter()
            .map(|&s| {
                let rules = rules.clone();
                std::thread::spawn(move || (s, m7b::play_with(rules, s, M7B_TICKS)))
            })
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
        println!(
            "seed {s}: {n} ticks, results {won}, first link {:?}, drain counters {:?}, drainer builds {}, links ended {}, below {}: {:?}, M7a counters below 3: {:?}",
            ex.first_link,
            ex.counters().map(|(_, v)| v),
            ex.m7a.uses.drainer_builds,
            ex.ends,
            m7b::MARGIN,
            ex.missing(),
            ex.m7a
                .counters()
                .iter()
                .filter(|&&(_, v)| v < 3)
                .map(|&(name, v)| format!("{name} = {v}"))
                .collect::<Vec<_>>()
        );
    }
    let table = |name: &str, v: &mut Vec<u32>| {
        v.sort_unstable();
        println!(
            "{name}: min {} median {} max {}",
            v[0],
            v[v.len() / 2],
            v[v.len() - 1]
        );
        v[0]
    };
    let mut worst = u32::MAX;
    for i in 0..runs[0].2.counters().len() {
        let name = runs[0].2.counters()[i].0;
        let mut v: Vec<u32> = runs.iter().map(|r| r.2.counters()[i].1).collect();
        worst = worst.min(table(name, &mut v));
    }
    for i in 0..runs[0].2.m7a.counters().len() {
        let name = runs[0].2.m7a.counters()[i].0;
        let mut v: Vec<u32> = runs.iter().map(|r| r.2.m7a.counters()[i].1).collect();
        table(name, &mut v);
    }
    println!("smallest drain minimum over {} seeds: {worst}", runs.len());
}
