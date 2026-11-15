//! The cost of `Game::save` (#131): a release probe, ignored by default.
//! Run with
//! `cargo test --release -p glasnost-ai --test save_cost -- --ignored --nocapture`.
//! It plays the synthetic skirmish of `fixture` for a while and prints the
//! median time of a game save beside the median time of the world's own
//! save inside it. Synthetic data only.

use std::time::{Duration, Instant};

use glasnost_ai::fixture;
use glasnost_ai::game::Game;

fn game() -> Game {
    Game::new(
        fixture::rules(),
        &fixture::map(),
        "synthetic-skirmish",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the synthetic skirmish loads")
}

fn median(mut v: Vec<Duration>) -> Duration {
    v.sort();
    v[v.len() / 2]
}

#[test]
#[ignore = "a timing probe for #131; run it in release by hand"]
fn game_save_cost() {
    let mut g = game();
    for _ in 0..3000 {
        g.step();
    }
    const RUNS: usize = 200; // nocite: our own probe's sample count
    let mut whole = Vec::with_capacity(RUNS);
    let mut world = Vec::with_capacity(RUNS);
    let mut len = 0;
    for _ in 0..RUNS {
        let t = Instant::now();
        let b = g.save();
        whole.push(t.elapsed());
        len = b.len();
        let t = Instant::now();
        let w = glasnost_sim::save::save(&g.world);
        world.push(t.elapsed());
        std::hint::black_box(w);
    }
    println!(
        "save_cost: tick {} bytes {len} game save median {:?} world save median {:?}",
        g.world.tick,
        median(whole),
        median(world)
    );
}
