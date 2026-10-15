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
//! - **the ferry** (S-0507), every [`THINK_PERIOD`] ticks: one unarmed
//!   hovering transport carries idle infantry from home to land near the
//!   next wave's goal, across water when they cannot walk there, loading
//!   and unloading with the clicks a person gives;
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

use glasnost_map::terrain::land;
use glasnost_map::{CellXY, Map};
use glasnost_sim::econ::TypeRef;
use glasnost_sim::econ::{deploy, placement, production, tech};
use glasnost_sim::orders::Order;
use glasnost_sim::regions;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::transport;
use glasnost_sim::world::{HouseId, Layer, ObjectId, Target, World};
use serde::{Deserialize, Serialize};

use crate::scripted::{
    self, Plan, Role, SPIRAL_RINGS, Script, THINK_PERIOD, all_types, can_attack, deploys_into_yard,
    dist2, idle, is_yard,
};
use crate::skirmish::start_cells;

/// Cells from one of the house's buildings within which a seen enemy counts
/// as attacking the base.
const DEFEND: i64 = 12; // nocite: tuning of our own opponent

/// Cells from the house's home within which army units are called back to
/// defend the base; a wave further out goes on.
const RECALL: i64 = 30; // nocite: tuning of our own opponent

/// An idle army vehicle with health at or below this share (in percent)
/// of its strength is sent to a repair depot.
const REPAIR_BELOW: i64 = 50; // nocite: tuning of our own opponent
/// The scale of [`REPAIR_BELOW`].
const PERCENT: i64 = 100; // nocite: percent scale

/// Cells from home within which a ferry counts as at home, and idle
/// infantry is sent aboard it.
const FERRY_HOME: i64 = 10; // nocite: tuning of our own opponent
/// Passengers a ferry waits for before it leaves (at most its capacity).
const FERRY_LOAD: i32 = 3; // nocite: tuning of our own opponent
/// The ring around the goal, in cells, where a ferry lets its passengers
/// out: no nearer than the first bound, no farther than the second.
const DROP_NEAR: i64 = 7; // nocite: tuning of our own opponent
const DROP_FAR: i64 = 12; // nocite: tuning of our own opponent
/// The ring around home where an empty ferry waits.
const WAIT_NEAR: i64 = 3; // nocite: tuning of our own opponent
const WAIT_FAR: i64 = 6; // nocite: tuning of our own opponent
/// The ring around a ferry, in cells, searched for land when it cannot
/// let its passengers out where it stopped.
const ASHORE: i64 = 4; // nocite: tuning of our own opponent

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
        if w.tick % THINK_PERIOD == 0 {
            self.send_for_repair(w, &mut out);
            self.ferry(w, &mut out);
        }
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

    /// Sends damaged idle army vehicles to a repair depot of the house that
    /// accepts them, by the enter order a person gives with a click (S-0314
    /// claim 24: our AI uses ordinary enter orders only, which follow
    /// claims 16 to 23).
    fn send_for_repair(&self, w: &World, out: &mut Vec<Order>) {
        let own = self.script.own(w);
        let depots: Vec<ObjectId> = own
            .iter()
            .copied()
            .filter(|&id| glasnost_sim::depot::is_depot(w, id))
            .collect();
        if depots.is_empty() {
            return;
        }
        for id in self.script.of_role(w, Role::Army) {
            let Some(o) = w.obj(id) else { continue };
            if o.kind != Kind::Vehicle || !idle(w, id) || o.contact.is_some() {
                continue;
            }
            let strength = i64::from(glasnost_sim::combat::damage::strength(w, id));
            if i64::from(o.health) * PERCENT > strength * REPAIR_BELOW {
                continue;
            }
            let at = o.pos.cell();
            if let Some(&d) = depots
                .iter()
                .filter(|&&d| glasnost_sim::depot::accepts(w, d, id, true))
                .min_by_key(|&&d| (w.obj(d).map_or(i64::MAX, |b| dist2(b.pos.cell(), at)), d))
            {
                out.push(Order::Behaviour {
                    actor: id,
                    behaviour: Behaviour::Enter,
                    attack_move: false,
                    target: None,
                    dest: Some(Target::Object(d)),
                });
            }
        }
    }

    /// The house's ferries: its unarmed vehicle transports (S-0507 Terms).
    fn ferries(&self, w: &World) -> Vec<ObjectId> {
        self.script
            .own(w)
            .into_iter()
            .filter(|&id| {
                w.obj(id).is_some_and(|o| {
                    transport::vehicle_transport(w, id)
                        && scripted::role_of(w, tech::type_of(o)).is_none()
                        && targeting::selectable(w, id)
                })
            })
            .collect()
    }

    /// Whether a ferry of type rules `u` suits a run: a transport that
    /// moves on land and, when the run crosses water, hovers over water
    /// too (S-0503).
    fn suits(u: &glasnost_sim::rules::UnitRules, crossing: bool) -> bool {
        use glasnost_rules::sim::MovementZone as Z;
        u.passengers > 0
            && u.primary.is_none()
            && u.locomotor != glasnost_sim::rules::Locomotor::None
            && u.zone != Z::Water
            && (!crossing
                || (u.zone == Z::Amphibious
                    && u.locomotor == glasnost_sim::rules::Locomotor::Hover))
    }

    /// Whether ferry `f` suits a run.
    fn ferry_suits(w: &World, f: ObjectId, crossing: bool) -> bool {
        w.obj(f)
            .and_then(|o| w.rules().unit(o.kind, o.type_index))
            .is_some_and(|u| Self::suits(u, crossing))
    }

    /// The cheapest unarmed vehicle transport type the house can build now
    /// that suits a run.
    fn ferry_type(&self, w: &World, crossing: bool) -> Option<TypeRef> {
        let r = w.rules();
        let mut v: Vec<TypeRef> = all_types(w)
            .into_iter()
            .filter(|t| t.kind == Kind::Vehicle)
            .filter(|&t| {
                r.vehicles
                    .get(t.index as usize)
                    .is_some_and(|u| Self::suits(u, crossing))
            })
            .filter(|&t| tech::buildable(w, self.house(), t))
            .collect();
        v.sort_by_key(|&t| (production::cost_field(w, t), t.index));
        v.first().copied()
    }

    /// Whether infantry `p` cannot walk to `goal` (another reach region of
    /// its movement zone, S-0212 claim 4): its run crosses water.
    fn crossing(w: &World, p: ObjectId, goal: CellXY) -> bool {
        let Some(o) = w.obj(p) else { return false };
        let Some(zone) = w.rules().unit(o.kind, o.type_index).map(|u| u.zone) else {
            return false;
        };
        let from = regions::reach_in(w, o.pos.cell(), zone);
        matches!(from, regions::Reach::Region(_)) && regions::reach_in(w, goal, zone) != from
    }

    /// The free land cell nearest to `from` (the first in cell order on
    /// equal distance) within the ring of `near` to `far` cells around
    /// `center` that ferry `f` can reach and, when `zone` is given, from
    /// which that zone reaches `center`.
    fn land_cell(
        &self,
        w: &World,
        f: ObjectId,
        center: CellXY,
        (near, far): (i64, i64),
        from: CellXY,
        zone: Option<glasnost_rules::sim::MovementZone>,
    ) -> Option<CellXY> {
        let fz = w
            .obj(f)
            .and_then(|o| w.rules().unit(o.kind, o.type_index))?
            .zone;
        let mine = regions::reach_in(w, from, fz);
        let goal = zone.map(|z| (z, regions::reach_in(w, center, z)));
        w.cells
            .grid
            .iter()
            .filter(|(c, cell)| {
                let d = dist2(*c, center);
                d >= near * near
                    && d <= far * far
                    && cell.land != land::WATER
                    && w.cells.usable(*c)
                    && w.cells.occ(*c, Layer::Ground).is_some_and(|o| o.empty())
            })
            .map(|(c, _)| c)
            // From a cell outside every region (a factory's exit) any
            // reachable cell will do.
            .filter(|&c| {
                let r = regions::reach_in(w, c, fz);
                r == mine
                    || (!matches!(mine, regions::Reach::Region(_))
                        && matches!(r, regions::Reach::Region(_)))
            })
            .filter(|&c| goal.is_none_or(|(z, r)| regions::reach_in(w, c, z) == r))
            .min_by_key(|&c| (dist2(c, from), c.y, c.x))
    }

    /// Ferries infantry (PLAN §4.7, S-0507), by the orders a person gives:
    ///
    /// - builds a ferry ([`Brain::ferry_type`]) once the harvesters are
    ///   kept, while the house has idle infantry and no ferry that suits
    ///   the run: one that hovers over water when the infantry cannot walk
    ///   to the goal, else the cheapest;
    /// - at home, sends idle infantry aboard with a click on the ferry
    ///   (the enter cursor, claim 5);
    /// - with [`FERRY_LOAD`] aboard (or every boarder in and no more idle
    ///   infantry), sends it to a free land cell near the next wave's goal
    ///   from which its passengers can walk there (across water when they
    ///   cannot walk from home);
    /// - there, unloads with a click on the ferry itself (the deploy
    ///   cursor, claims 7 and 8), or moves to land nearby when the cursor
    ///   refuses;
    /// - empty and away, goes home.
    fn ferry(&self, w: &World, out: &mut Vec<Order>) {
        let house = self.house();
        let (Some(home), Some(goal)) = (self.script.home(w), self.script.attack_goal(w)) else {
            return;
        };
        let mods = Modifiers::default();
        let infantry: Vec<ObjectId> = self
            .script
            .of_role(w, Role::Army)
            .into_iter()
            .filter(|&id| {
                w.obj(id).is_some_and(|o| {
                    o.kind == Kind::Infantry
                        && dist2(o.pos.cell(), home) <= 4 * FERRY_HOME * FERRY_HOME // nocite: tuning (twice the home radius)
                }) && idle(w, id)
            })
            .collect();
        let crossing = infantry.iter().any(|&p| Self::crossing(w, p, goal));
        let ferries = self.ferries(w);
        if !ferries.iter().any(|&f| Self::ferry_suits(w, f, crossing))
            && !infantry.is_empty()
            && !self.script.selling()
            && self.script.harvesters_kept(w)
            && let Some(t) = self.ferry_type(w, crossing)
            && !self.script.busy(w, t)
            && !out
                .iter()
                .any(|o| matches!(o, Order::Build { what } if *what == t))
        {
            out.push(Order::Build { what: t });
        }
        let mut taken: Vec<ObjectId> = Vec::new();
        for f in ferries {
            let Some(o) = w.obj(f) else { continue };
            if !idle(w, f) {
                continue;
            }
            let at = o.pos.cell();
            let aboard = transport::count(w, f);
            let room = transport::capacity(w, f);
            let at_home = dist2(at, home) <= FERRY_HOME * FERRY_HOME;
            let go = |c: CellXY| Order::Behaviour {
                actor: f,
                behaviour: Behaviour::Move,
                attack_move: false,
                target: None,
                dest: Some(Target::Cell(c)),
            };
            if !at_home {
                if aboard == 0 {
                    if let Some(c) = self.land_cell(w, f, home, (WAIT_NEAR, WAIT_FAR), at, None) {
                        out.push(go(c));
                    }
                } else if targeting::cursor(w, house, &[f], ClickOn::Object(f), mods)
                    == Action::Deploy
                {
                    out.extend(targeting::click(w, house, &[f], ClickOn::Object(f), mods).orders);
                } else if o.dest.is_none()
                    && let Some(c) = self.land_cell(w, f, at, (1, ASHORE), at, None)
                {
                    out.push(go(c));
                }
                continue;
            }
            // A ferry that stands where a building does (a factory's exit)
            // takes nobody aboard (S-0507 claim 4): it moves off first.
            if w.cells
                .occ(at, Layer::Ground)
                .is_some_and(|c| c.buildings > 0)
            {
                if let Some(c) = self.land_cell(w, f, home, (WAIT_NEAR, WAIT_FAR), at, None) {
                    out.push(go(c));
                }
                continue;
            }
            let entering: Vec<ObjectId> = self
                .script
                .own(w)
                .into_iter()
                .filter(|&id| w.obj(id).is_some_and(|p| p.dest == Some(Target::Object(f))))
                .collect();
            let boarders = aboard + entering.len() as i32;
            let free: Vec<ObjectId> = infantry
                .iter()
                .copied()
                .filter(|id| !taken.contains(id) && !entering.contains(id))
                .collect();
            let full = aboard >= FERRY_LOAD.min(room);
            if full || (aboard > 0 && entering.is_empty() && free.is_empty()) {
                // Leave for the goal: land near it from which the
                // passengers' own zone reaches it.
                let zone = transport::passengers(w, f)
                    .first()
                    .and_then(|&p| w.obj(p))
                    .and_then(|p| w.rules().unit(p.kind, p.type_index))
                    .map(|u| u.zone);
                if let Some(c) = self.land_cell(w, f, goal, (DROP_NEAR, DROP_FAR), at, zone) {
                    out.push(go(c));
                }
                continue;
            }
            // Only a ferry that suits the run takes infantry aboard.
            if !Self::ferry_suits(w, f, crossing) {
                continue;
            }
            for p in free
                .into_iter()
                .take(usize::try_from(room - boarders).unwrap_or(0))
            {
                if targeting::cursor(w, house, &[p], ClickOn::Object(f), mods) == Action::Enter {
                    out.extend(targeting::click(w, house, &[p], ClickOn::Object(f), mods).orders);
                    taken.push(p);
                }
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
            if let Some(&(e, _)) = intruders
                .iter()
                .filter(|&&(e, _)| can_attack(w, id, e))
                .min_by_key(|&&(e, c)| (dist2(c, at), e))
            {
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
