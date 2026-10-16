//! The M6c duel of the gate exercises every special ability, and its replay
//! reproduces the final state.

use glasnost_ai::{fixture, fixture_m6c as kit, game::Game};
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
