//! The headless skirmish runner: starts a game from lobby settings, lets a
//! [`Script`] play each player house through orders, and records every order
//! it issued in a [`Replay`], so the game can be replayed without the
//! scripts (the determinism gate) and checked across a save and load.

use std::sync::Arc;

use glasnost_map::terrain::TileCatalogue;
use glasnost_map::{CellXY, Map};
use glasnost_sim::econ::victory;
use glasnost_sim::load::{LoadError, new_game};
use glasnost_sim::orders;
use glasnost_sim::rules::SimRules;
use glasnost_sim::save::Replay;
use glasnost_sim::tick;
use glasnost_sim::world::{GameSetup, HouseId, HouseRole, World};

use crate::scripted::{Plan, Script, THINK_PERIOD};

/// Start waypoints a map can define (0 … 7).
const START_WAYPOINTS: u16 = 8; // nocite: the runner reads waypoints 0 … 7 as start cells, a lobby view of the map

/// A finished run.
pub struct Run {
    /// The recorded replay; `ticks` is the tick the run stopped at.
    pub replay: Replay,
    /// The world at the end.
    pub world: World,
    /// The game's result once it is over (S-0414).
    pub over: bool,
    /// The winning house, when one won.
    pub winner: Option<HouseId>,
}

/// The map's start cells (its waypoints 0 … 7 that are defined).
pub fn start_cells(map: &Map) -> Vec<CellXY> {
    (0..START_WAYPOINTS)
        .filter_map(|k| map.waypoints.get(&k))
        .map(|&(x, y)| CellXY::new(i32::from(x), i32::from(y)))
        .collect()
}

/// The player houses still in the game.
pub fn players_left(w: &World) -> Vec<HouseId> {
    (0..w.houses.len())
        .map(|i| HouseId(i as u8))
        .filter(|&h| {
            let house = &w.houses[usize::from(h.0)];
            house.role == HouseRole::Player && !house.removed && !victory::defeated(w, h)
        })
        .collect()
}

/// Runs a skirmish for at most `max_ticks` ticks: the player houses play
/// `plan`, and the run stops after the tick in which the game is over
/// (S-0414, play to the last side).
pub fn run(
    rules: Arc<SimRules>,
    map: &Map,
    map_name: &str,
    catalogue: Arc<TileCatalogue>,
    setup: GameSetup,
    plan: &Plan,
    max_ticks: i32,
) -> Result<Run, LoadError> {
    let mut w = new_game(rules.clone(), map, catalogue, &setup)?;
    let mut replay = Replay::new(&rules, map_name, setup);
    let starts = start_cells(map);
    let mut scripts: Vec<Script> = players_left(&w)
        .into_iter()
        .map(|h| {
            let mut s = Script::new(h, plan.clone());
            s.set_starts(&w, &starts);
            s
        })
        .collect();
    let mut winner = None;
    let mut over = false;
    while w.tick < max_ticks {
        if w.tick % THINK_PERIOD == 0 {
            for s in &mut scripts {
                if !players_left(&w).contains(&s.house) {
                    continue;
                }
                for o in s.think(&w) {
                    if orders::issue(&mut w, s.house, o.clone()) {
                        replay.record(w.tick, s.house, o);
                    }
                }
            }
        }
        tick::step(&mut w);
        if let Some(o) = victory::outcome(&w) {
            over = true;
            winner = o
                .results
                .iter()
                .find(|r| r.1 == victory::Verdict::Win)
                .map(|r| r.0);
            break;
        }
    }
    replay.ticks = tick::ticks_run(&w);
    Ok(Run {
        replay,
        world: w,
        over,
        winner,
    })
}
