//! The brain of one AI house (PLAN §4.7, M5b): what it remembers between
//! ticks and how it thinks. Its state is saved with the game
//! ([`crate::game::Game`]).
//!
//! It plays in two layers:
//!
//! - **the plan** ([`Script`], the M5a scripted builder and attack waves),
//!   every [`THINK_PERIOD`] ticks: build-up, harvesters, army, waves; it
//!   rebuilds what the house lost of its power, refineries, factories and
//!   harvesters, because it asks for them whenever the house has none (or
//!   fewer harvesters than the plan keeps);
//! - **the reactions**, every tick (no reaction delay): place a finished
//!   building at once, defend the base against seen enemies near its
//!   buildings, and rebuild a lost construction yard (build a unit that
//!   deploys into one, then deploy it);
//! - **selling out**: a house with no income for a long while and too few
//!   credits for an army unit sells its buildings one by one for their
//!   refunds and builds army units with them, and builds no more buildings;
//!   a house with nothing left to fight with sells out, so a game between
//!   two broke houses still ends.
//!
//! It sees only what its own house sees: its own objects and enemy objects
//! on cells its house has explored (fog of war is off in a skirmish, S-0409
//! claim 19, so a person sees what stands there), and the map's start
//! waypoints (lobby knowledge). It issues only orders a human could, through
//! the same `orders::issue`; a refused order is simply not applied, and the
//! next think tries again. It draws nothing from any random stream.
//!
//! The numbers here are tuning of our own opponent, not game behaviour.

use glasnost_map::{CellXY, Map};
use glasnost_sim::econ::TypeRef;
use glasnost_sim::econ::{deploy, placement, production, tech};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use serde::{Deserialize, Serialize};

use crate::scripted::{
    Plan, Role, SPIRAL_RINGS, Script, THINK_PERIOD, all_types, deploys_into_yard, dist2, idle,
    is_yard,
};
use crate::skirmish::start_cells;

/// Cells from one of the house's buildings within which a seen enemy counts
/// as attacking the base.
const DEFEND: i64 = 12; // nocite: tuning of our own opponent

/// Cells from the house's home within which army units are called back to
/// defend the base; a wave further out goes on.
const RECALL: i64 = 30; // nocite: tuning of our own opponent

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
        Self::with_starts(w, &start_cells(map), house, plan)
    }

    /// A brain for house `house` playing `plan`, given the map's start
    /// cells.
    pub fn with_starts(w: &World, starts: &[CellXY], house: HouseId, plan: Plan) -> Self {
        let mut script = Script::new(house, plan);
        script.set_starts(w, starts);
        Self { script }
    }

    /// The house.
    pub fn house(&self) -> HouseId {
        self.script.house
    }

    /// Thinks at the start of the world's current tick; returns the orders
    /// to issue, in the order to issue them (a later order to the same unit
    /// replaces an earlier one, so the reactions come last).
    pub fn think(&mut self, w: &World) -> Vec<Order> {
        let mut out = if w.tick % THINK_PERIOD == 0 {
            self.script.think(w)
        } else {
            let mut v = Vec::new();
            self.place_done(w, &mut v);
            v
        };
        self.rebuild_yard(w, &mut out);
        self.find_deploy_spot(w, &mut out);
        self.defend(w, &mut out);
        out
    }

    /// Places a finished building of the house.
    fn place_done(&self, w: &World, out: &mut Vec<Order>) {
        let house = self.house();
        let done = w.econ.lines.lines.iter().find_map(|l| {
            let item = l.item.as_ref()?;
            (l.house == house && item.t.kind == Kind::Building && item.done_tick.is_some())
                .then_some(item.t)
        });
        if let Some(t) = done
            && let Some(cell) = self.script.place_cell(w, t)
        {
            out.push(Order::Place {
                what: t,
                cell: Some(cell),
            });
        }
    }

    /// The vehicle types the house can build now that deploy into a
    /// construction yard, cheapest first.
    fn yard_units(&self, w: &World) -> Vec<TypeRef> {
        let r = w.rules();
        let mut v: Vec<TypeRef> = all_types(w)
            .into_iter()
            .filter(|t| t.kind == Kind::Vehicle)
            .filter(|t| {
                deploy::deploys_into(w, t.kind, t.index).is_some_and(|b| {
                    r.buildings.get(b as usize).is_some_and(|b| {
                        b.factory == Some(glasnost_sim::rules::FactoryKind::Building)
                    })
                })
            })
            .filter(|&t| tech::buildable(w, self.house(), t))
            .collect();
        v.sort_by_key(|&t| (production::cost_field(w, t), t.index));
        v
    }

    /// Rebuilds a lost construction yard: while the house has neither a
    /// yard nor a unit that deploys into one, it builds such a unit, queued
    /// behind what its line holds (the plan deploys it).
    fn rebuild_yard(&self, w: &World, out: &mut Vec<Order>) {
        let own = self.script.own(w);
        let has_yard_or_unit = own.iter().any(|&id| {
            w.obj(id).is_some_and(|o| {
                is_yard(w, o)
                    || (o.kind == Kind::Vehicle && deploys_into_yard(w, o.kind, o.type_index))
            })
        });
        if has_yard_or_unit || self.script.selling() {
            return;
        }
        // Queued behind what the line holds, once.
        if let Some(&t) = self.yard_units(w).first() {
            let held = production::line_of(w, self.house(), t)
                .is_some_and(|l| l.item.as_ref().is_some_and(|i| i.t == t) || l.queue.contains(&t));
            if !held
                && !out
                    .iter()
                    .any(|o| matches!(o, Order::Build { what } if *what == t))
            {
                out.push(Order::Build { what: t });
            }
        }
    }

    /// Whether a unit `id` that deploys into building type `b` could deploy
    /// standing in cell `c`: the deploy's cell test (S-0415 claim 3, the
    /// answer a person's deploy cursor gives) and the gap the plan keeps to
    /// the house's buildings.
    fn deploys_at(&self, w: &World, id: ObjectId, b: u32, c: CellXY) -> bool {
        let size = placement::place_type(w, b).size;
        let at = deploy::anchor(c, size);
        let test = placement::Test {
            ignore: Some(id),
            ..placement::Test::order(self.house())
        };
        let t = TypeRef {
            kind: Kind::Building,
            index: b,
        };
        placement::cell_test(w, b, at, &test) && self.script.free_but(w, t, at, Some(id))
    }

    /// While the house has no construction yard, sends an idle unit that
    /// deploys into one, which could not deploy where it stands, to the
    /// nearest cell of a square spiral around it where it could; the plan
    /// deploys it there.
    fn find_deploy_spot(&self, w: &World, out: &mut Vec<Order>) {
        let own = self.script.own(w);
        if own
            .iter()
            .any(|&id| w.obj(id).is_some_and(|o| is_yard(w, o)))
        {
            return;
        }
        for &id in &own {
            let Some(o) = w.obj(id) else { continue };
            if o.kind != Kind::Vehicle || !idle(w, id) {
                continue;
            }
            let Some(b) = deploy::deploys_into(w, o.kind, o.type_index)
                .filter(|_| deploys_into_yard(w, o.kind, o.type_index))
            else {
                continue;
            };
            let here = o.pos.cell();
            if self.deploys_at(w, id, b, here) {
                continue;
            }
            let spot = (1..=SPIRAL_RINGS).find_map(|ring| {
                (-ring..=ring)
                    .flat_map(|dy| (-ring..=ring).map(move |dx| (dx, dy)))
                    .filter(|&(dx, dy)| dx.abs() == ring || dy.abs() == ring)
                    .map(|(dx, dy)| CellXY::new(here.x + dx, here.y + dy))
                    .find(|&c| w.cells.usable(c) && self.deploys_at(w, id, b, c))
            });
            if let Some(c) = spot {
                out.push(Order::Behaviour {
                    actor: id,
                    behaviour: Behaviour::Move,
                    attack_move: false,
                    target: None,
                    dest: Some(Target::Cell(c)),
                });
            }
        }
    }

    /// Seen enemies near the house's buildings.
    fn intruders(&self, w: &World) -> Vec<(ObjectId, CellXY)> {
        let own = self.script.own(w);
        let buildings: Vec<CellXY> = own
            .iter()
            .filter_map(|&id| w.obj(id))
            .filter(|o| o.kind == Kind::Building)
            .map(|o| o.pos.cell())
            .collect();
        self.script
            .seen_enemies(w)
            .into_iter()
            .filter(|&(_, c)| buildings.iter().any(|&b| dist2(b, c) <= DEFEND * DEFEND))
            .collect()
    }

    /// Defends the base: every army unit near home that is not already
    /// fighting an intruder attacks the intruder nearest to it.
    fn defend(&self, w: &World, out: &mut Vec<Order>) {
        let intruders = self.intruders(w);
        if intruders.is_empty() {
            return;
        }
        let Some(home) = self.script.home(w) else {
            return;
        };
        for id in self.script.of_role(w, Role::Army) {
            let Some(o) = w.obj(id) else { continue };
            let at = o.pos.cell();
            if dist2(at, home) > RECALL * RECALL {
                continue;
            }
            let busy = matches!(o.target, Some(Target::Object(t))
                if intruders.iter().any(|&(e, _)| e == t));
            if busy {
                continue;
            }
            if let Some(&(e, _)) = intruders.iter().min_by_key(|&&(e, c)| (dist2(c, at), e)) {
                out.push(Order::Behaviour {
                    actor: id,
                    behaviour: Behaviour::Attack,
                    attack_move: false,
                    target: Some(Target::Object(e)),
                    dest: None,
                });
            }
        }
    }
}
