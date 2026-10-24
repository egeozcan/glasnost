//! The tick benchmark's probe (M8 C4 part 2) on synthetic data: the
//! benchmark runs, and the probe changes nothing in the game (the state
//! hashes with it on and off are the same tick for tick). No timing is
//! asserted.

use glasnost_ai::{fixture, game::Game};
use glasnost_tools::tick_bench::{self, Clock};

#[test]
fn the_synthetic_tick_benchmark_runs_alike_with_and_without_the_probe() {
    let (off, hash_off) = tick_bench::run_synthetic(7, 400, false);
    let (on, hash_on) = tick_bench::run_synthetic(7, 400, true);
    assert_eq!(hash_off, hash_on);
    assert!(off.contains("ticks: 400"), "{off}");
    assert!(on.contains("attribution"), "{on}");
    assert!(on.contains("TurnPass"), "{on}");
}

#[test]
fn the_probe_leaves_an_ai_game_unchanged() {
    let new = || {
        Game::new(
            fixture::rules(),
            &fixture::map(),
            "synthetic",
            fixture::catalogue(),
            fixture::setup(5),
        )
        .expect("the synthetic skirmish loads")
    };
    let (mut a, mut b) = (new(), new());
    let mut clock = Clock::default();
    for t in 0..600 {
        a.step();
        b.step_with(&mut clock);
        if t % 50 == 0 {
            assert_eq!(a.state_hash(), b.state_hash(), "tick {t}");
        }
    }
    assert_eq!(a.state_hash(), b.state_hash());
    assert!(!clock.tick.is_empty(), "the probe was told of the parts");
}
