//! The game wrapper: the AI's state is saved and hashed with the world, so
//! a save in mid-game, loaded and continued, plays on alike, and a replay
//! (setup and the person's orders only) runs the AI again to the same state.

use glasnost_ai::{fixture, game::Game, scripted, siege};
use glasnost_sim::orders::Order;
use glasnost_sim::probe::{Phase, Probe};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::tick;
use glasnost_sim::world::{AiDifficulty, AiSeat, HouseId, Target};

const FIRST: i32 = 400;
const MORE: i32 = 200;

fn start() -> Game {
    start_with(fixture::setup(5).ai)
}

/// The synthetic skirmish (seed 5) with the AI seats `ai`.
fn start_with(ai: Vec<AiSeat>) -> Game {
    let mut setup = fixture::setup(5);
    setup.ai = ai;
    Game::new(
        fixture::rules(),
        &fixture::map(),
        "synthetic-skirmish",
        fixture::catalogue(),
        setup,
    )
    .expect("the synthetic skirmish loads")
}

/// The siege script at hard.
fn siege_hard() -> AiSeat {
    AiSeat {
        script: siege::ID,
        difficulty: AiDifficulty::Hard,
    }
}

/// The tick house 1 surrenders at in [`surrendered`] (#462's case).
// nocite: test data of our own (the issue's regression case)
const SURRENDER_AT: i32 = 500;

/// #462's case: both seats siege at hard, house 1 surrenders at
/// [`SURRENDER_AT`] through [`Game::issue`], and the game is stepped until
/// it is over (S-0414 claim 11, then the end delay of claim 8). `each` is
/// called after every tick.
fn surrendered(mut each: impl FnMut(&mut Game)) -> Game {
    let mut g = start_with(vec![siege_hard(); 2]);
    // nocite: test data of our own (a bound on the end delay's ticks)
    for _ in 0..SURRENDER_AT + 2000 {
        if g.outcome().is_some() {
            break;
        }
        if g.world.tick == SURRENDER_AT {
            assert!(
                g.issue(HouseId(1), Order::Surrender),
                "the surrender is taken"
            );
        }
        g.step();
        each(&mut g);
    }
    assert!(g.outcome().is_some(), "the game is over");
    g
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

/// Records which houses' brains ran their plan in a step: the probe is
/// told `Think(house)` before each brain, and the plan's parts only in a
/// think that runs them (`ai:economy_first` follows the plan).
#[derive(Default)]
struct Planners {
    thinking: Option<u8>,
    planned: Vec<u8>,
}

impl Probe for Planners {
    fn mark(&mut self, phase: Phase) {
        match phase {
            Phase::Think(h) => self.thinking = Some(h),
            Phase::Part("ai:economy_first") => self.planned.extend(self.thinking),
            _ => {}
        }
    }
}

// PLAN §4.7 (M8 step B7): the brains of a game run their plans in different ticks, each every THINK_PERIOD ticks at its house's phase, and no tick runs two.
#[test]
fn the_brains_plan_in_different_ticks() {
    let mut g = start();
    let period = scripted::THINK_PERIOD;
    let mut by_house: std::collections::BTreeMap<u8, Vec<i32>> = Default::default();
    for _ in 0..4 * period {
        let tick = g.world.tick;
        let mut p = Planners::default();
        g.step_with(&mut p);
        assert!(p.planned.len() <= 1, "tick {tick}: {:?} planned", p.planned);
        for h in p.planned {
            by_house.entry(h).or_default().push(tick);
        }
    }
    assert_eq!(by_house.len(), 2, "both houses planned: {by_house:?}");
    for (h, ticks) in &by_house {
        let phase = scripted::think_phase(HouseId(*h));
        let want: Vec<i32> = (0..4 * period).filter(|t| t % period == phase).collect();
        assert_eq!(ticks, &want, "house {h} plans at its phase");
    }
    // Every house number of an eight-player game has a phase of its own.
    let phases: std::collections::BTreeSet<i32> =
        (0..8).map(|h| scripted::think_phase(HouseId(h))).collect();
    assert_eq!(phases.len(), 8);
    assert!(phases.iter().all(|&p| (0..period).contains(&p)));
}

// design: own AI (#51): once the game is over (S-0414) a step changes nothing, so the state stays the one its replay ends on
#[test]
fn a_step_after_the_game_is_over_changes_nothing() {
    let mut g = start();
    let theirs: Vec<_> = g
        .world
        .objects
        .values()
        .filter(|o| o.owner == Some(HouseId(1)))
        .map(|o| o.id)
        .collect();
    for id in theirs {
        glasnost_sim::objects::remove(&mut g.world, id);
    }
    for _ in 0..2000 {
        if g.outcome().is_some() {
            break;
        }
        g.step();
    }
    assert!(g.outcome().is_some(), "the game is over");
    // A tick run again would apply an order waiting for the winner's unit.
    let mover = g
        .world
        .objects
        .values()
        .find(|o| o.owner == Some(HouseId(0)) && o.on_map && o.kind != Kind::Building)
        .map(|o| o.id)
        .expect("a unit of the winner");
    let at = g.world.obj(mover).unwrap().pos.cell();
    assert!(glasnost_sim::orders::issue(
        &mut g.world,
        HouseId(0),
        glasnost_sim::orders::Order::Behaviour {
            actor: mover,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(glasnost_map::CellXY::new(at.x + 10, at.y))),
        },
    ));
    let (tick, hash) = (g.world.tick, g.state_hash());
    for _ in 0..2 * scripted::THINK_PERIOD {
        g.step();
    }
    assert_eq!(g.world.tick, tick);
    assert_eq!(g.state_hash(), hash);
}

// PLAN §4.7 (M8 step B7): a game saved between the two houses' thinks of a period (house 0 has thought, house 1 not yet) plays on alike after a load.
#[test]
fn a_save_between_the_houses_thinks_plays_on_alike() {
    let mut g = start();
    let period = scripted::THINK_PERIOD;
    let beta = scripted::think_phase(HouseId(1));
    assert!(beta > scripted::think_phase(HouseId(0)));
    while !(g.world.tick >= FIRST && g.world.tick % period == beta - 1) {
        g.step();
    }
    let bytes = g.save();
    let mut loaded = Game::load(&bytes, fixture::rules(), fixture::catalogue()).expect("loads");
    for _ in 0..MORE {
        g.step();
        loaded.step();
        assert_eq!(loaded.state_hash(), g.state_hash(), "tick {}", g.world.tick);
    }
}

// design: own AI (#462); spec: S-0414 claim 8 (no tick runs after the final one, so an order given to a finished game would never apply): a finished game refuses a person's order, and its pending orders, its replay and its hash stay as they were, so the replay still ends on the live state.
#[test]
fn a_finished_game_refuses_input() {
    let mut g = surrendered(|_| {});
    let pending: Vec<String> = g
        .world
        .houses
        .iter()
        .map(|h| format!("{:?}", h.pending))
        .collect();
    let replay = postcard::to_allocvec(g.replay()).expect("the replay serializes");
    let hash = g.state_hash();
    assert!(
        !g.issue(HouseId(0), Order::Surrender),
        "a finished game takes no order"
    );
    let after: Vec<String> = g
        .world
        .houses
        .iter()
        .map(|h| format!("{:?}", h.pending))
        .collect();
    assert_eq!(after, pending, "no order is left pending");
    assert!(
        postcard::to_allocvec(g.replay()).expect("the replay serializes") == replay,
        "the replay is unchanged"
    );
    assert_eq!(g.state_hash(), hash, "the state is unchanged");
    let again = Game::run_replay(
        g.replay(),
        fixture::rules(),
        &fixture::map(),
        fixture::catalogue(),
        |_| {},
    )
    .expect("replays");
    assert_eq!(
        again.state_hash(),
        hash,
        "the replay ends on the live state"
    );
}

// spec: S-0414 claim 8 and S-0201 claim 1 (#463: no tick runs after the final one, which leaves the tick number at itself): a replay whose recorded length runs past the game's end stops at the final tick, and its callback is called once per tick actually run.
#[test]
fn a_replay_stops_at_the_final_tick() {
    let g = surrendered(|_| {});
    let ran = tick::ticks_run(&g.world);
    let mut longer = g.replay().clone();
    // nocite: test data of our own (a recorded length past the game's end, as a rerun that ends sooner)
    longer.ticks = ran + 100;
    let mut seen = Vec::new();
    let again = Game::run_replay(
        &longer,
        fixture::rules(),
        &fixture::map(),
        fixture::catalogue(),
        |r| {
            seen.push(tick::ticks_run(&r.world));
            // Before #463 the loop never ends: stop it here.
            assert!(seen.len() <= longer.ticks as usize, "calls past the end");
        },
    )
    .expect("replays");
    assert_eq!(seen, (1..=ran).collect::<Vec<_>>(), "one call per tick run");
    assert!(again.outcome().is_some(), "the replayed game is over");
    assert_eq!(again.state_hash(), g.state_hash());
}

// design: own AI (#464), as `Replay::run_on` (#398): each tick's effects (presentation only, neither saved nor hashed) reach the replay's callback once, in the tick that made them, and are cleared after it; the final hash is the live game's.
#[test]
fn a_replay_hands_each_ticks_effects_once() {
    let mut live = Vec::new();
    let g = surrendered(|g| live.push(std::mem::take(&mut g.world.effects)));
    assert!(
        live.iter().filter(|e| !e.is_empty()).count() > 1,
        "several ticks make effects"
    );
    let mut replayed = Vec::new();
    let again = Game::run_replay(
        g.replay(),
        fixture::rules(),
        &fixture::map(),
        fixture::catalogue(),
        |r| replayed.push(r.world.effects.clone()),
    )
    .expect("replays");
    assert_eq!(replayed.len(), live.len(), "one call per tick");
    for (i, (r, l)) in replayed.iter().zip(&live).enumerate() {
        assert_eq!(r, l, "call {i}: that tick's effects only");
    }
    assert!(again.world.effects.is_empty(), "none kept after the last");
    assert_eq!(again.state_hash(), g.state_hash());
}

/// The houses of each per-house diagnostic of `g`, in its order.
fn diagnostic_houses(g: &Game) -> Vec<Vec<HouseId>> {
    vec![
        g.diagnose().into_iter().map(|(h, _)| h).collect(),
        g.special_uses().into_iter().map(|(h, _)| h).collect(),
        g.antiair_probes().into_iter().map(|(h, _)| h).collect(),
        g.expansion_probes().into_iter().map(|(h, _)| h).collect(),
        g.broke_probes().into_iter().map(|(h, _)| h).collect(),
    ]
}

// design: own AI (#465): every AI seat has its row in each per-house diagnostic, the siege's from the brain that runs its base, with the house numbers in seat order; in a mixed game and in an all-siege game.
#[test]
fn every_seat_has_its_diagnostic_rows() {
    for ai in [
        vec![siege_hard(), AiSeat::DEFAULT],
        vec![AiSeat::DEFAULT, siege_hard()],
        vec![siege_hard(); 2],
    ] {
        let mut g = start_with(ai.clone());
        // nocite: test data of our own (a few think periods into the game)
        for _ in 0..3 * scripted::THINK_PERIOD {
            g.step();
        }
        let houses = g.ai_houses();
        assert_eq!(houses, vec![HouseId(0), HouseId(1)], "{ai:?}");
        for (i, rows) in diagnostic_houses(&g).into_iter().enumerate() {
            assert_eq!(rows, houses, "{ai:?}: diagnostic {i}");
        }
        assert_eq!(g.economy_probes().len(), houses.len(), "{ai:?}");
    }
}
