//! The brain of one AI house: what it remembers between ticks and how it
//! thinks. Its state is saved with the game ([`crate::game::Game`]).

use glasnost_map::Map;
use glasnost_sim::orders::Order;
use glasnost_sim::world::{HouseId, World};
use serde::{Deserialize, Serialize};

use crate::scripted::{Plan, Script, THINK_PERIOD};
use crate::skirmish::start_cells;

/// The default plan of an AI house: types by role, two harvesters, waves of
/// six (tuning of our own opponent).
pub fn default_plan() -> Plan {
    Plan {
        names: Default::default(),
        harvesters: 2, // nocite: tuning of our own opponent
        wave: 6,       // nocite: tuning of our own opponent
    }
}

/// One AI house's brain.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Brain {
    script: Script,
}

impl Brain {
    /// A brain for house `house` with the default plan.
    pub fn new(w: &World, map: &Map, house: HouseId) -> Self {
        Self::with_plan(w, map, house, default_plan())
    }

    /// A brain for house `house` playing `plan`.
    pub fn with_plan(w: &World, map: &Map, house: HouseId, plan: Plan) -> Self {
        let mut script = Script::new(house, plan);
        script.set_starts(w, &start_cells(map));
        Self { script }
    }

    /// The house.
    pub fn house(&self) -> HouseId {
        self.script.house
    }

    /// Thinks at the start of the world's current tick; returns the orders
    /// to issue.
    pub fn think(&mut self, w: &mut World) -> Vec<Order> {
        if w.tick % THINK_PERIOD != 0 {
            return Vec::new();
        }
        self.script.think(w)
    }
}
