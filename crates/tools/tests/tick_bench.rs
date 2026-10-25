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

/// `searches=on` (M8 C4 part 5): every path computation is listed, one
/// row of the header's columns each, and the trace behind it changes
/// nothing in the game (the scripted replay of the determinism gate).
#[test]
fn the_search_listing_has_a_row_per_computation_and_leaves_the_game_unchanged() {
    use glasnost_sim::{load, save, testkit, tick};
    let rep = testkit::scripted_replay(7, 1500);
    let new = || {
        load::new_game(
            testkit::rules(),
            &testkit::map(),
            testkit::catalogue(),
            &rep.setup,
        )
        .expect("the synthetic scenario loads")
    };
    let (mut a, mut b) = (new(), new());
    glasnost_sim::trace::switch_on(&mut b);
    let mut csv = String::from(tick_bench::SEARCHES_HEAD);
    while tick::ticks_run(&a) < rep.ticks {
        rep.issue_due(&mut a);
        rep.issue_due(&mut b);
        tick::step(&mut a);
        tick::step(&mut b);
        tick_bench::note_searches(&b, &mut csv);
        assert_eq!(
            save::state_hash(&a),
            save::state_hash(&b),
            "tick {}",
            a.tick
        );
    }
    let columns = tick_bench::SEARCHES_HEAD.trim_end().split(',').count();
    let rows: Vec<&str> = csv.lines().skip(1).collect();
    assert!(!rows.is_empty(), "the replay computes paths");
    for r in &rows {
        assert_eq!(r.split(',').count(), columns, "{r}");
    }
    let opts = tick_bench::Options::parse(&["searches=on".to_owned()]).expect("parses");
    assert!(opts.searches);
}
