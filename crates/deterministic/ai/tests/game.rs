//! The game wrapper: the AI's state is saved and hashed with the world, so
//! a save in mid-game, loaded and continued, plays on alike, and a replay
//! (setup and the person's orders only) runs the AI again to the same state.

use glasnost_ai::{fixture, game::Game};

const FIRST: i32 = 400;
const MORE: i32 = 200;

fn start() -> Game {
    Game::new(
        fixture::rules(),
        &fixture::map(),
        "synthetic-skirmish",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the synthetic skirmish loads")
}

#[test]
fn save_load_and_replay_keep_the_state() {
    let mut g = start();
    assert_eq!(g.ai_houses().len(), 2, "both houses are AI houses");
    while g.world.tick < FIRST {
        g.step();
    }
    let bytes = g.save();
    let mut loaded = Game::load(&bytes, fixture::rules(), fixture::catalogue()).expect("loads");
    assert_eq!(loaded.state_hash(), g.state_hash());
    for _ in 0..MORE {
        g.step();
        loaded.step();
    }
    assert_eq!(
        loaded.state_hash(),
        g.state_hash(),
        "loaded game plays on alike"
    );
    let replayed = Game::run_replay(
        g.replay(),
        fixture::rules(),
        &fixture::map(),
        fixture::catalogue(),
        |_| {},
    )
    .expect("replays");
    assert_eq!(replayed.world.tick, g.world.tick);
    assert_eq!(
        replayed.state_hash(),
        g.state_hash(),
        "replay runs the AI again"
    );
}
