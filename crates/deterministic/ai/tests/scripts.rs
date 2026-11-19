//! The pluggable scripts (#449) on the synthetic skirmish: every registry
//! entry at every difficulty sees only its own house's shroud; the
//! difficulty never reaches a house's rules; the choice survives a save,
//! a load, the hash and a replay; a setup without choices plays as before.

use glasnost_ai::scripted::THINK_PERIOD;
use glasnost_ai::scripts::{REGISTRY, Seat};
use glasnost_ai::{fixture, game::Game};
use glasnost_sim::econ::{TypeRef, production, shroud};
use glasnost_sim::load::new_game;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{AiDifficulty, AiSeat, GameSetup, HouseId, World};

/// Every (script, difficulty) the registry offers.
fn every_choice() -> Vec<AiSeat> {
    REGISTRY
        .iter()
        .flat_map(|e| {
            AiDifficulty::ALL.into_iter().map(|d| AiSeat {
                script: e.id,
                difficulty: d,
            })
        })
        .collect()
}

/// The synthetic skirmish with house 0 playing `seat`.
fn setup_with(seed: u32, seat: AiSeat) -> GameSetup {
    let mut s = fixture::setup(seed);
    s.ai = vec![seat, AiSeat::DEFAULT];
    s
}

fn game(setup: GameSetup) -> Game {
    Game::new(
        fixture::rules(),
        &fixture::map(),
        "synthetic-skirmish",
        fixture::catalogue(),
        setup,
    )
    .expect("the synthetic skirmish loads")
}

/// The game stepped until both houses have an army unit (as the AI suite's
/// `armed`), at most `limit` ticks.
fn armed(seat: AiSeat) -> Game {
    let mut g = game(setup_with(5, seat));
    let has = |w: &World, h: u8| {
        w.objects.values().any(|o| {
            o.owner == Some(HouseId(h))
                && o.on_map
                && o.kind == Kind::Vehicle
                && w.rules()
                    .vehicles
                    .get(o.type_index as usize)
                    .map(|t| t.name.as_str())
                    == Some("GUNR")
        })
    };
    // nocite: test data of our own (the AI suite's limit for an armed start)
    for _ in 0..9000 {
        if has(&g.world, 0) && has(&g.world, 1) {
            break;
        }
        g.step();
    }
    g
}

// design: own AI (#449): a script's inputs carry nothing a person in its seat could not see. For every registry entry at every difficulty, two worlds that differ only by an enemy unit on a cell the house has not explored give the same orders, over a whole think period, from the same state.
#[test]
fn every_script_decides_only_on_what_it_sees() {
    let map = fixture::map();
    for seat in every_choice() {
        let mut g = armed(seat);
        let hidden = (0..=2 * fixture::SIZE)
            .flat_map(|y| (0..=2 * fixture::SIZE).map(move |x| glasnost_map::CellXY::new(x, y)))
            .find(|&c| {
                g.world.cells.usable(c)
                    && !shroud::explored(&g.world, HouseId(0), c)
                    && g.world.objects.values().all(|o| o.pos.cell() != c)
            })
            .expect("a cell house 0 has not explored");
        let base = Seat::start(seat, &g.world, &map, HouseId(0));
        assert_eq!(base.choice(), seat);
        let mut thought = false;
        for _ in 0..THINK_PERIOD {
            let mut other = g.world.clone();
            testkit::spawn(&mut other, Kind::Vehicle, "GUNR", 1, hidden, 0);
            assert!(!shroud::explored(&other, HouseId(0), hidden));
            thought |= glasnost_ai::scripted::thinks_at(HouseId(0), g.world.tick);
            let (mut a, mut b) = (base.clone(), base.clone());
            assert_eq!(
                format!("{:?}", a.think(&g.world, None, &mut ())),
                format!("{:?}", b.think(&other, None, &mut ())),
                "{seat:?}: same orders at tick {}",
                g.world.tick
            );
            g.step();
        }
        assert!(thought, "the period held a think tick");
    }
}

// spec: S-0412 claim 5, S-0301 claim 3, S-0302 Terms and claim 3 and the glossary's *difficulty skill block* (every player house, a person's or an AI house, uses the normal row unless its handicap picks another; AI difficulty never sets it): an AI house at every difficulty has the person's house's setup row, difficulty index, combat factors and prices
#[test]
fn the_difficulty_row_is_a_persons_at_every_ai_difficulty() {
    let rules = fixture::rules();
    let map = fixture::map();
    let types: Vec<TypeRef> = (0..rules.buildings.len())
        .map(|i| TypeRef {
            kind: Kind::Building,
            index: i as u32,
        })
        .chain((0..rules.vehicles.len()).map(|i| TypeRef {
            kind: Kind::Vehicle,
            index: i as u32,
        }))
        .collect();
    let mut first: Option<String> = None;
    for level in AiDifficulty::ALL {
        let mut s = fixture::setup(3);
        // House 0 a person's, house 1 an AI house, both of one country.
        s.houses[0].ai = false;
        s.houses[1].country = s.houses[0].country.clone();
        s.ai = vec![
            AiSeat::DEFAULT,
            AiSeat {
                script: AiSeat::DEFAULT_SCRIPT,
                difficulty: level,
            },
        ];
        let w = new_game(rules.clone(), &map, fixture::catalogue(), &s).expect("it loads");
        let (person, ai) = (&w.houses[0], &w.houses[1]);
        assert!(ai.econ.setup.ai && !person.econ.setup.ai);
        assert_eq!(
            ai.econ.setup.difficulty, person.econ.setup.difficulty,
            "{level:?}"
        );
        assert_eq!(
            ai.econ.setup.handicap, person.econ.setup.handicap,
            "{level:?}"
        );
        assert_eq!(ai.combat.rof_factor, person.combat.rof_factor, "{level:?}");
        assert_eq!(ai.combat.firepower, person.combat.firepower, "{level:?}");
        for &t in &types {
            assert_eq!(
                production::price(&w, HouseId(1), t),
                production::price(&w, HouseId(0), t),
                "{level:?} {t:?}"
            );
        }
        // The person's house is the same whatever the AI's difficulty.
        let row = format!(
            "{:?} {:?} {:?}",
            person.econ.setup, person.combat.rof_factor, person.combat.firepower
        );
        assert_eq!(first.get_or_insert_with(|| row.clone()), &row, "{level:?}");
    }
}

// design: own AI (#449): a non-default script or difficulty is kept by a save and a load, and the AI's hash covers it: the loaded game hashes as the running one and plays on alike, and a game at another difficulty hashes otherwise
#[test]
fn the_choice_survives_save_load_and_the_hash() {
    let easy = AiSeat {
        script: AiSeat::DEFAULT_SCRIPT,
        difficulty: AiDifficulty::Easy,
    };
    let hard = AiSeat {
        script: AiSeat::DEFAULT_SCRIPT,
        difficulty: AiDifficulty::Hard,
    };
    let mut s = fixture::setup(5);
    s.ai = vec![easy, hard];
    let mut g = game(s);
    // nocite: test data of our own (a few think periods into the game)
    for _ in 0..300 {
        g.step();
    }
    let mut back = Game::load(&g.save(), fixture::rules(), fixture::catalogue()).expect("loads");
    let choices = |g: &Game| g.seats().iter().map(Seat::choice).collect::<Vec<_>>();
    assert_eq!(choices(&back), vec![easy, hard]);
    assert_eq!(back.state_hash(), g.state_hash());
    // nocite: test data of our own
    for _ in 0..300 {
        g.step();
        back.step();
    }
    assert_eq!(back.state_hash(), g.state_hash());
    // The replay carries the choice: replayed, the game ends alike.
    let again = Game::run_replay(
        g.replay(),
        fixture::rules(),
        &fixture::map(),
        fixture::catalogue(),
        |_| {},
    )
    .expect("replays");
    assert_eq!(choices(&again), vec![easy, hard]);
    assert_eq!(again.state_hash(), g.state_hash());
    // The same game at normal hashes otherwise from the start.
    let normal = game(fixture::setup(5));
    let fresh = {
        let mut s = fixture::setup(5);
        s.ai = vec![easy, hard];
        game(s)
    };
    assert_ne!(normal.state_hash(), fresh.state_hash());
}

// design: own AI (#449): a setup without choices (an old one: no entries, or fewer than its houses) plays the default script at normal, and hashes as a setup that names the default for every house
#[test]
fn a_setup_without_choices_plays_the_default_at_normal() {
    let none = fixture::setup(5);
    assert!(none.ai.is_empty());
    assert_eq!(none.ai_seat(0), AiSeat::DEFAULT);
    let mut short = fixture::setup(5);
    short.ai = vec![AiSeat::DEFAULT];
    let mut named = fixture::setup(5);
    named.ai = vec![AiSeat::DEFAULT; 2];
    let mut games: Vec<Game> = [none, short, named].into_iter().map(game).collect();
    for g in &games {
        assert!(g.seats().iter().all(|s| s.choice() == AiSeat::DEFAULT));
    }
    // nocite: test data of our own
    for _ in 0..450 {
        for g in &mut games {
            g.step();
        }
    }
    assert_eq!(games[0].state_hash(), games[1].state_hash());
    assert_eq!(games[0].state_hash(), games[2].state_hash());
}

// design: own AI (#449): difficulty changes the default script's play: easy keeps one harvester and small waves, hard a third harvester; normal is the default plan
#[test]
fn the_difficulties_change_the_plan_only_away_from_normal() {
    use glasnost_ai::brain::{default_plan, plan_for};
    assert_eq!(plan_for(AiDifficulty::Normal), default_plan());
    let (easy, hard) = (plan_for(AiDifficulty::Easy), plan_for(AiDifficulty::Hard));
    let normal = default_plan();
    assert!(easy.harvesters < normal.harvesters && easy.wave < normal.wave);
    assert!(hard.harvesters > normal.harvesters);
}

/// A probe that keeps the names of the brain's parts it is told of.
struct Parts(Vec<&'static str>);

impl glasnost_sim::probe::Probe for Parts {
    fn mark(&mut self, phase: glasnost_sim::probe::Phase) {
        if let glasnost_sim::probe::Phase::Part(name) = phase {
            self.0.push(name);
        }
    }
}

// design: own AI (#449, B10): at easy the default script acts on one of every three of its house's think ticks, and on the other two gives only what it gives between thinks, so it builds, sends its army and answers a raid later; normal and hard act on every think tick
#[test]
fn easy_acts_on_one_think_tick_in_three() {
    use glasnost_ai::scripts::{AiScript, Schedule, ScriptState, View};
    // nocite: design: own AI (#449, B10): the easy level's rate
    const EASY_EVERY: usize = 3;
    let map = fixture::map();
    let mut got = Vec::new();
    for level in AiDifficulty::ALL {
        let seat = AiSeat {
            script: AiSeat::DEFAULT_SCRIPT,
            difficulty: level,
        };
        let mut g = armed(seat);
        let base = ScriptState::start(seat, &g.world, &map, HouseId(0));
        let (mut acted, mut idle) = (0, 0);
        // nocite: test data of our own (two of easy's think cycles)
        for _ in 0..2 * EASY_EVERY as i32 * THINK_PERIOD {
            let tick = g.world.tick;
            if glasnost_ai::scripted::thinks_at(HouseId(0), tick) {
                // The parts of a think (the economy first, the build and
                // the specials) mark the probe only when the house acts.
                let view = View::new(&g.world, HouseId(0));
                let mut p = Parts(Vec::new());
                base.clone()
                    .think(&view, Schedule { tick, thinks: true }, level, None, &mut p);
                if p.0.contains(&"ai:economy_first") {
                    acted += 1;
                } else {
                    idle += 1;
                }
            }
            g.step();
        }
        got.push((level, acted, idle));
    }
    let n = 2 * EASY_EVERY;
    assert_eq!(
        got,
        vec![
            (AiDifficulty::Easy, 2, n - 2),
            (AiDifficulty::Normal, n, 0),
            (AiDifficulty::Hard, n, 0),
        ]
    );
}
