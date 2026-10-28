//! Our own rules patch layer *Pathing and other fixes* (M8 step C7) in a
//! whole game: a game played with it saves and loads only under its own
//! rules, never under the reference ruleset, and its replay reproduces it.
//! Saves, replays and network starts carry the layer by the rules hash, so
//! loading never resolves the rules again with another setting.

use glasnost_ai::{fixture, game::Game};
use glasnost_rules::layers::Layers;

/// The ticks played (our own test length: the AI has built and harvests).
// nocite: our own test length
const TICKS: i32 = 600;

// spec: S-0212 claims 13 and 13a and Edge cases (both corridor switches off) and S-0314 Edge cases (quirks.depot_loss_strands_hover off), as the play-fixes layer sets them: a game with the layer is a game of its own rules, saved, loaded and replayed under them
#[test]
fn a_game_with_the_layer_saves_loads_and_replays_under_its_own_rules() {
    let name = "synthetic-skirmish";
    let fixed = glasnost_sim_gate::layered(fixture::ruleset(), name, Layers::PLAY_FIXES);
    let plain = glasnost_sim_gate::layered(fixture::ruleset(), name, Layers::REFERENCE);
    assert_eq!(
        plain.hash,
        fixture::rules().hash,
        "no layer: the gate's rules"
    );
    assert_ne!(plain.hash, fixed.hash);
    assert!(!fixed.quirks.corridor_retry_no_fallback && !fixed.quirks.corridor_split_probe);
    assert!(!fixed.quirks.depot_loss_strands_hover);
    let catalogue = fixture::catalogue();
    let mut g = Game::new(
        fixed.clone(),
        &fixture::map(),
        name,
        catalogue.clone(),
        fixture::setup(3),
    )
    .expect("the synthetic game loads");
    while g.world.tick < TICKS / 2 {
        g.step();
    }
    let bytes = g.save();
    assert!(
        Game::load(&bytes, plain.clone(), catalogue.clone()).is_err(),
        "a save with the layer does not load under the reference ruleset"
    );
    let mut loaded = Game::load(&bytes, fixed.clone(), catalogue.clone()).expect("it loads");
    assert_eq!(loaded.state_hash(), g.state_hash());
    while g.world.tick < TICKS {
        g.step();
        loaded.step();
    }
    assert_eq!(
        loaded.state_hash(),
        g.state_hash(),
        "the loaded game plays on alike"
    );
    assert_eq!(g.replay().rules_hash, fixed.hash);
    assert!(
        Game::run_replay(
            g.replay(),
            plain,
            &fixture::map(),
            catalogue.clone(),
            |_| ()
        )
        .is_err(),
        "its replay does not run under the reference ruleset"
    );
    let again = Game::run_replay(g.replay(), fixed, &fixture::map(), catalogue, |_| ())
        .expect("its replay runs under its own rules");
    assert_eq!(
        again.state_hash(),
        g.state_hash(),
        "the replay reproduces the game"
    );
}
