//! The synthetic skirmish the game tests play over the relay: invented
//! rules and map (`glasnost_ai::fixture`) with eight starts, the setup
//! rules a client offers, each client's game from the room's `Begin`, and
//! the scripted orders that stand in for a person.

#![allow(dead_code)]

use glasnost_ai::fixture;
use glasnost_ai::game::Game;
use glasnost_map::Map;
use glasnost_net::lobby::Begin;
use glasnost_net::simnet::SplitMix;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{GameSetup, HouseId, Target};
use glasnost_ui::setup::{self as ui_setup, MapEntry, SetupRules};

/// The synthetic skirmish map with six more start waypoints.
pub fn map8() -> Map {
    let mut m = fixture::map();
    for (k, dx, dy) in [
        (2u16, -14, -18),
        (3, 14, 18),
        (4, -14, 18),
        (5, 14, -18),
        (6, 0, -24),
        (7, 0, 24),
    ] {
        let c = fixture::at(dx, dy);
        m.waypoints.insert(k, (c.x as i16, c.y as i16));
    }
    m
}

/// The setup rules every client offers.
pub fn setup_rules() -> SetupRules {
    SetupRules {
        money: 2000,
        money_range: (1000, 5000, 100),
        units: 3,
        units_range: (1, 9),
        bases: true,
        flags: [false, true, true, true, true],
        game_speed: 4,
        countries: vec!["Alphaland".into(), "Betaland".into()],
        colours: vec![[200, 0, 0, 255]; 8],
    }
}

/// Each client's game from the room's `Begin`, as a client builds it: the
/// lobby of the setup, the game setup from it, the session's start values.
pub fn build(begin: &Begin, map: &Map) -> (Game, glasnost_net::StartInfo, GameSetup) {
    let rules = setup_rules();
    let maps = [MapEntry {
        label: "synthetic-8".into(),
        starts: 8,
    }];
    let lobby = begin
        .local_setup(0)
        .lobby(&rules, &maps)
        .expect("the room's setup is sound");
    let gs = ui_setup::game_setup(
        &lobby,
        &fixture::rules(),
        &rules.countries,
        begin.seed,
        begin.seed,
    )
    .expect("a game setup");
    let info = begin.start_info(&gs).expect("the room's slots and seed");
    (game(map, &gs), info, gs)
}

pub fn game(map: &Map, gs: &GameSetup) -> Game {
    Game::new(
        fixture::rules(),
        map,
        "synthetic-8",
        fixture::catalogue(),
        gs.clone(),
    )
    .expect("the synthetic skirmish loads")
}

/// A person's scripted orders: deploy the first vehicle early, then now and
/// then send one of the house's units somewhere.
pub fn scripted(g: &Game, house: HouseId, tick: u32, rng: &mut SplitMix) -> Vec<Order> {
    let units: Vec<_> = g
        .world
        .objects
        .iter()
        .filter(|(_, o)| {
            o.owner == Some(house)
                && o.on_map
                && o.health > 0
                && matches!(o.kind, Kind::Vehicle | Kind::Infantry)
        })
        .map(|(&id, o)| (id, o.kind))
        .collect();
    let mut out = Vec::new();
    if tick == 3
        && let Some(&(id, _)) = units.iter().find(|(_, k)| *k == Kind::Vehicle)
    {
        out.push(Order::Deploy { actor: id });
    }
    if tick.is_multiple_of(7) && !units.is_empty() {
        let (id, _) = units[rng.below(units.len() as u64) as usize];
        let dx = rng.below(41) as i32 - 20;
        let dy = rng.below(41) as i32 - 20;
        out.push(Order::Behaviour {
            actor: id,
            behaviour: Behaviour::Move,
            attack_move: rng.below(2) == 0,
            target: None,
            dest: Some(Target::Cell(fixture::at(dx, dy))),
        });
    }
    out
}
