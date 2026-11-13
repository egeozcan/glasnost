//! The game wrapper: the AI's state is saved and hashed with the world, so
//! a save in mid-game, loaded and continued, plays on alike, and a replay
//! (setup and the person's orders only) runs the AI again to the same state.

use glasnost_ai::{fixture, game::Game, scripted};
use glasnost_sim::probe::{Phase, Probe};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{HouseId, Target};

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
