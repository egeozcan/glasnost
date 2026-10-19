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
//! - **superweapons** (S-0512, S-0513): once the harvesters are kept, it
//!   builds, one after the other, the cheapest building it can that gives a
//!   kind of superweapon it has no building for yet (every Allied and Soviet
//!   kind the tech tree offers), before going on with the tech climb; every
//!   tick, a ready missile or storm superweapon (as the sidebar shows it) is fired with
//!   the order a person gives, at the seen enemy cell with the most seen
//!   enemies around it and none of the house's own objects; a ready
//!   paradrop (S-0522) is fired at a free usable land cell near home, and
//!   the paratroopers that land join its army; a ready iron curtain goes on
//!   a group of its own armed vehicles fighting seen enemies, a force shield
//!   on its buildings with seen enemies at them, a spy plane at the enemy
//!   base, and the teleport takes a group of its armed vehicles (first
//!   stage) to a free block next to the enemy base (second stage), each with
//!   the order a person gives; a ready psychic reveal (S-0610) goes on the
//!   enemy start cell nearest home that the house does not see (then on
//!   the unexplored cell nearest an enemy start); and once
//!   the harvesters are kept it builds a psychic sensor when it has none
//!   (`psychic`);
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
    self, CLIMB_DEPTH, Plan, Role, SPIRAL_RINGS, Script, THINK_PERIOD, all_types, can_attack,
    deploys_into_yard, dist2, idle, is_yard,
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
/// Ticks after which a ferry still unloading with passengers aboard is
/// taken to be stuck (a passenger with no exit blocks the rest, and the
/// transport retries for good, S-0507 claim 10) and is moved on.
const UNLOAD_STUCK: i32 = 450; // nocite: tuning of our own opponent
/// The radius, in cells, around a superweapon's target cell within which
/// seen enemies count for its worth and the house's own objects forbid it.
const STRIKE_RADIUS: i64 = 3; // nocite: tuning of our own opponent
/// The armed vehicles in one effect block that make an iron curtain worth
/// firing at a fight (a pair of tanks about to engage).
const CURTAIN_VEHICLES: usize = 2; // nocite: tuning of our own opponent
/// The fewest of its own armed vehicles in an effect block for a teleport
/// source (a block of three, with none of its infantry in it, was too rare:
/// the teleport stood ready for thousands of ticks in the local lane).
const TELEPORT_MIN: usize = 2; // nocite: tuning of our own opponent
/// The fewest of its own buildings in an effect block for a force shield.
const SHIELD_MIN: usize = 2; // nocite: tuning of our own opponent
/// Cells from a group of vehicles within which a seen enemy counts as a
/// fight for an iron curtain.
const FIGHT: i64 = 10; // nocite: tuning of our own opponent
/// The nearest and farthest ring around the enemy base, in cells, at which
/// the teleport's second stage lands.
const LAND_RINGS: (i32, i32) = (4, 9); // nocite: tuning of our own opponent
/// The nearest and farthest ring around home, in cells, at which a
/// paradrop is aimed.
const DROP_RINGS: (i32, i32) = (3, 8); // nocite: tuning of our own opponent

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
    /// The special abilities (M6c): garrisons, capture, infiltration.
    special: crate::special::Special,
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
        Self {
            script,
            special: Default::default(),
        }
    }

    /// How many orders the brain gave to use each special ability.
    pub fn special_uses(&self) -> crate::special::Uses {
        self.special.uses()
    }

    /// How many (unit, goal) pairs the brain has given up for now: orders
    /// a unit did not take (`memory`).
    pub fn given_up(&self) -> usize {
        self.special.given_up()
    }

    /// The house.
    pub fn house(&self) -> HouseId {
        self.script.house
    }

    /// A counts-only diagnostic of the brain's state (local-lane reports,
    /// see [`Script::diagnose`]).
    pub fn diagnose(&self, w: &World) -> String {
        let mut s = self.script.diagnose(w);
        // The superweapon buildings: giving types whose kind the house has
        // no building for, how many it can build now, place now, or reach by
        // a building that clears their blocker.
        let owned: std::collections::BTreeSet<_> = self
            .script
            .own(w)
            .into_iter()
            .filter_map(|id| w.obj(id).filter(|o| o.kind == Kind::Building))
            .flat_map(|o| Self::giving_kinds(w, o.type_index))
            .collect();
        let giving: Vec<TypeRef> = all_types(w)
            .into_iter()
            .filter(|t| t.kind == Kind::Building)
            .filter(|t| {
                Self::giving_kinds(w, t.index)
                    .iter()
                    .any(|k| !owned.contains(k))
            })
            .collect();
        let buildable: Vec<&TypeRef> = giving
            .iter()
            .filter(|&&t| tech::buildable(w, self.house(), t))
            .collect();
        let placeable = buildable
            .iter()
            .filter(|&&&t| self.script.place_cell(w, t).is_some())
            .count();
        let reachable = giving
            .iter()
            .filter(|&&t| !tech::buildable(w, self.house(), t))
            .filter(|&&t| self.script.unblocker(w, t, CLIMB_DEPTH).is_some())
            .count();
        s.push_str(&format!(
            "; superweapon buildings: kinds owned {}, giving types without an owned kind {}, buildable now {}, placeable {}, blocked but reachable by a clearing building {}, harvesters kept {}, selling {}",
            owned.len(),
            giving.len(),
            buildable.len(),
            placeable,
            reachable,
            self.script.harvesters_kept(w),
            self.script.selling()
        ));
        // The ferry: idle infantry at home, whether its run crosses water,
        // the ferries the house owns and the types it could build for a run.
        if let (Some(home), Some(goal)) = (self.script.home(w), self.script.attack_goal(w)) {
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
            s.push_str(&format!(
                "; ferry: idle infantry at home {}, run crosses water {crossing}, ferries owned {}, type for a land run {}, type for a crossing {}",
                infantry.len(),
                self.ferries(w).len(),
                self.ferry_type(w, false).is_some(),
                self.ferry_type(w, true).is_some()
            ));
        }
        s
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
            let paying = self.economy_first(w, &mut out);
            self.send_for_repair(w, &mut out);
            self.ferry(w, &mut out);
            self.build_superweapon(w, &mut out);
            let sensors = self.special_uses().sensors;
            self.build_sensor(w, &mut out);
            if self.special_uses().sensors == sensors && !self.build_grinder(w, &mut out) {
                let defences = self.special_uses().gattling_defences;
                self.build_gattling_defence(w, &mut out);
                if self.special_uses().gattling_defences == defences && !paying {
                    self.build_bunker(w, &mut out);
                }
            }
            self.build_lifter(w, &mut out);
            self.build_drainer(w, &mut out);
        }
        if w.tick % THINK_PERIOD == 0 {
            self.special.think(&self.script, w, &mut out);
            if !self.script.selling() {
                let plant = self.script.plant_type(w);
                let n = crate::power::plant_first(w, self.house(), plant, &mut out);
                self.special.count_plants_first(n);
            }
        }
        self.fire_superweapons(w, &mut out);
        self.rebuild_yard(w, &mut out);
        self.find_deploy_spot(w, &mut out);
        self.defend(w, &mut out);
        // spec: S-0214 claim 15 and S-0610 claim 6 (a berserk unit takes no orders: a person cannot give it one, so neither does the brain)
        out.retain(|o| {
            o.actor()
                .is_none_or(|a| !glasnost_sim::psychic::berserk(w, a))
        });
        // S-0609 claims 5 and 11: no attack order at another object (from any part of the brain) to a gattling unit firing at its target, so it keeps its spin
        let before = out.len();
        out.retain(|o| match o {
            Order::Behaviour {
                actor,
                behaviour: Behaviour::Attack,
                target,
                ..
            } => {
                !crate::gattling::engaged(w, *actor)
                    || w.obj(*actor).is_some_and(|x| x.target == *target)
            }
            _ => true,
        });
        let kept = before - out.len();
        if kept > 0 {
            self.special.count_spins_kept(kept);
        }
        // S-0607 claims 9 and 12: a lifter takes only the lifter part's orders, and a holding lifter only its drop (`lifter`)
        out.retain(|o| crate::lifter::keeps_order(w, o));
        // S-0608 claims 3 b and 12: a draining drainer takes no order (a move off the foundation ends the link), one on its way to drain only drain orders, and no second drainer goes for a building (`drain`)
        crate::drain::filter_orders(w, &mut out);
        // S-0601 claim 16: no attack order (from any part of the brain) to an unlimited controller one more victim would overload harmfully
        out.retain(|o| match o {
            Order::Behaviour {
                actor,
                behaviour: Behaviour::Attack,
                target: Some(Target::Object(_)),
                ..
            } => {
                glasnost_sim::mind::controller(w, *actor).is_none()
                    || crate::mind::safe_more(w, *actor)
            }
            _ => true,
        });
        out
    }

    /// Economy first on the production lines, as a person who has lost a
    /// harvester or refinery pauses the rest of the sidebar to pay for its
    /// replacement: while the house keeps fewer harvesters than its
    /// economy target (`Script::economy_target`: the plan's, or every slave
    /// miner it wants) or has no refinery, and an item of the harvester or refinery role
    /// runs on a line with less money in hand than its balance owed, every
    /// other running item of the house (power excepted while the house is
    /// short of power) is put on hold with the hold order (S-0404 claim 5).
    /// Once no such item runs, or the economy is whole again, every held
    /// item is resumed with a build order for its type (S-0404 claims 3
    /// and 5), the forced-start hold of claim 10 included, which a person
    /// resumes alike. Found by the M7b AI audit: a slave-miner house that
    /// lost a miner to a raid paid for its replacement in step with a
    /// superweapon building and two army lines, and the miner stood on its
    /// line for 6 000 ticks.
    /// Returns whether the house is paying for its economy (no new
    /// building of its own choosing is started then).
    fn economy_first(&mut self, w: &World, out: &mut Vec<Order>) -> bool {
        use glasnost_sim::econ::production::Hold;
        let h = self.house();
        let economy = |t: TypeRef| {
            matches!(
                scripted::role_of(w, t),
                Some(Role::Harvester | Role::Refinery)
            )
        };
        let short = !self.script.selling()
            && (!self.script.economy_kept(w) || !self.script.has_refinery(w));
        let credits = w.houses[usize::from(h.0)].credits;
        let paying = short
            && w.econ.lines.lines.iter().any(|l| {
                l.house == h
                    && l.item.as_ref().is_some_and(|i| {
                        economy(i.t) && i.hold == Hold::None && !i.done() && credits < i.owed
                    })
            });
        let low_power = crate::power::short(w, h);
        for l in w.econ.lines.lines.iter().filter(|l| l.house == h) {
            let Some(i) = &l.item else { continue };
            if i.done() || economy(i.t) {
                continue;
            }
            if paying {
                let power = scripted::role_of(w, i.t) == Some(Role::Power);
                if i.hold == Hold::None && !(power && low_power) {
                    out.push(Order::Hold { what: i.t });
                    self.special.count_economy_hold();
                }
            } else if i.hold == Hold::Player && tech::buildable(w, h, i.t) {
                out.push(Order::Build { what: i.t });
                self.special.count_resume();
            }
        }
        paying
    }

    /// Builds a psychic sensor (S-0610 claim 9) as a person adds one to the
    /// base: once it keeps its harvesters, is not selling out and has none,
    /// the cheapest sensor type it can build and place now
    /// ([`crate::psychic::build_sensor`]). Like a superweapon building it
    /// replaces a tech-climb building this think ordered, but not a building
    /// the plan places or wants, nor a superweapon building.
    fn build_sensor(&mut self, w: &World, out: &mut Vec<Order>) {
        let climb = |t: &TypeRef| t.kind == Kind::Building && scripted::role_of(w, *t).is_none();
        let giving = |t: &TypeRef| !Self::giving_kinds(w, t.index).is_empty();
        if self.script.selling()
            || !self.script.harvesters_kept(w)
            || out.iter().any(|o| {
                matches!(o, Order::Place { .. })
                    || matches!(o, Order::Build { what } if what.kind == Kind::Building && (!climb(what) || giving(what)))
            })
        {
            return;
        }
        if let Some(t) = crate::psychic::build_sensor(&self.script, w) {
            out.retain(|o| !matches!(o, Order::Build { what } if climb(what)));
            out.push(Order::Build { what: t });
            self.special.count_sensor();
        }
    }

    /// Builds a recycling building (S-0603, [`crate::grinder::build_type`])
    /// as a person adds a grinder to the base, under the same conditions as
    /// the psychic sensor. Returns whether it ordered one.
    fn build_grinder(&mut self, w: &World, out: &mut Vec<Order>) -> bool {
        let climb = |t: &TypeRef| t.kind == Kind::Building && scripted::role_of(w, *t).is_none();
        let giving = |t: &TypeRef| !Self::giving_kinds(w, t.index).is_empty();
        if self.script.selling()
            || !self.script.harvesters_kept(w)
            || out.iter().any(|o| {
                matches!(o, Order::Place { .. })
                    || matches!(o, Order::Build { what } if what.kind == Kind::Building && (!climb(what) || giving(what)))
            })
        {
            return false;
        }
        let Some(t) = crate::grinder::build_type(&self.script, w) else {
            return false;
        };
        out.retain(|o| !matches!(o, Order::Build { what } if climb(what)));
        out.push(Order::Build { what: t });
        true
    }

    /// Builds a gattling defence (S-0609, [`crate::gattling::defence_type`])
    /// as a person adds base defences: once it keeps its harvesters and is
    /// not selling out, when no other building or placement was ordered in
    /// this think (a tech-climb building aside, which it replaces), nor a
    /// superweapon building or a psychic sensor.
    fn build_gattling_defence(&mut self, w: &World, out: &mut Vec<Order>) {
        let climb = |t: &TypeRef| t.kind == Kind::Building && scripted::role_of(w, *t).is_none();
        let giving = |t: &TypeRef| !Self::giving_kinds(w, t.index).is_empty();
        if self.script.selling()
            || !self.script.harvesters_kept(w)
            || out.iter().any(|o| {
                matches!(o, Order::Place { .. })
                    || matches!(o, Order::Build { what } if what.kind == Kind::Building && (!climb(what) || giving(what)))
            })
        {
            return;
        }
        if let Some(t) = crate::gattling::defence_type(&self.script, w) {
            out.retain(|o| !matches!(o, Order::Build { what } if climb(what)));
            out.push(Order::Build { what: t });
            self.special.count_gattling_defence();
        }
    }

    /// Builds a magnetic lifter (S-0607, [`crate::lifter::build_type`]) as a
    /// person adds a few lifters, since every side fields vehicles: not while selling
    /// out, and only when the plan keeps its harvesters. A vehicle order does
    /// not compete with the building orders of this think.
    fn build_lifter(&mut self, w: &World, out: &mut Vec<Order>) {
        if self.script.selling() || !self.script.harvesters_kept(w) {
            return;
        }
        if let Some(t) = crate::lifter::build_type(&self.script, w) {
            out.push(Order::Build { what: t });
            self.special.count_lifter_build();
        }
    }

    /// Builds a drainer (S-0608, [`crate::drain::build_type`]) the way it
    /// builds lifters: not while selling out, and only when the plan keeps
    /// its harvesters.
    fn build_drainer(&mut self, w: &World, out: &mut Vec<Order>) {
        if self.script.selling() || !self.script.harvesters_kept(w) {
            return;
        }
        if let Some(t) = crate::drain::build_type(&self.script, w) {
            out.push(Order::Build { what: t });
            self.special.count_drainer_build();
        }
    }

    /// Builds a tank bunker (S-0606, [`crate::bunker::build_type`]) as a
    /// person adds base defences early: as for the gattling defence (not
    /// while selling out, not in a think that places or orders another
    /// building, and not in a think that ordered a gattling defence), but
    /// without waiting for the harvesters, since the price reserve of
    /// [`crate::bunker::build_type`] already keeps the economy fed.
    fn build_bunker(&mut self, w: &World, out: &mut Vec<Order>) {
        let climb = |t: &TypeRef| t.kind == Kind::Building && scripted::role_of(w, *t).is_none();
        let giving = |t: &TypeRef| !Self::giving_kinds(w, t.index).is_empty();
        if self.script.selling()
            || out.iter().any(|o| {
                matches!(o, Order::Place { .. })
                    || matches!(o, Order::Build { what } if what.kind == Kind::Building && (!climb(what) || giving(what)))
            })
        {
            return;
        }
        if let Some(t) = crate::bunker::build_type(&self.script, w) {
            out.retain(|o| !matches!(o, Order::Build { what } if climb(what)));
            out.push(Order::Build { what: t });
            self.special.count_bunker_build();
        }
    }

    /// The kind of superweapon type `i` when the brain fires it: every kind
    /// of the Allied and Soviet sides (the nuclear strike, the weather
    /// storm, the iron curtain, the teleport's two stages, the paradrops, the
    /// spy plane and the force shield), the psychic reveal (S-0610), the
    /// genetic mutator (S-0611, `mutator`) and the psychic dominator
    /// (S-0612, `dominator`).
    fn fired_kind(w: &World, i: u32) -> Option<glasnost_sim::superweapons::SwKind> {
        use glasnost_sim::superweapons::SwKind;
        let k = w.rules().superweapons.types.get(i as usize)?.kind?;
        matches!(
            k,
            SwKind::Missile
                | SwKind::Storm
                | SwKind::IronCurtain
                | SwKind::Chrono
                | SwKind::ChronoWarp
                | SwKind::Paradrop
                | SwKind::AmerParadrop
                | SwKind::SpyPlane
                | SwKind::ForceShield
                | SwKind::PsychicReveal
                | SwKind::Genetic
                | SwKind::Dominator
        )
        .then_some(k)
    }

    /// The kinds that building type `b` gives and the brain fires (S-0512
    /// Terms: its first and second superweapon fields), in field order.
    fn giving_kinds(w: &World, b: u32) -> Vec<glasnost_sim::superweapons::SwKind> {
        w.rules()
            .superweapons
            .building_sw
            .get(b as usize)
            .map(|f| {
                f.iter()
                    .flatten()
                    .filter_map(|&i| Self::fired_kind(w, i))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Builds the cheapest building the house can build and place now that
    /// gives a kind of superweapon it fires and has no building for yet,
    /// once it keeps its harvesters and is not selling out: so in time one
    /// building of every kind its side's tech tree offers (the kinds come
    /// from the rules' superweapon types that the buildings name, no
    /// building is named here). It comes before the tech climb (it replaces
    /// a climb building this think ordered) but after a building the plan
    /// places or a power plant, refinery or factory it wants.
    fn build_superweapon(&self, w: &World, out: &mut Vec<Order>) {
        let climb = |t: &TypeRef| t.kind == Kind::Building && scripted::role_of(w, *t).is_none();
        if self.script.selling() || !self.script.harvesters_kept(w) || out.iter().any(|o| {
            matches!(o, Order::Place { .. })
                || matches!(o, Order::Build { what } if what.kind == Kind::Building && !climb(what))
        }) {
            return;
        }
        let owned: std::collections::BTreeSet<_> = self
            .script
            .own(w)
            .into_iter()
            .filter_map(|id| w.obj(id).filter(|o| o.kind == Kind::Building))
            .flat_map(|o| Self::giving_kinds(w, o.type_index))
            .collect();
        let giving: Vec<TypeRef> = all_types(w)
            .into_iter()
            .filter(|t| t.kind == Kind::Building)
            .filter(|t| {
                Self::giving_kinds(w, t.index)
                    .iter()
                    .any(|k| !owned.contains(k))
            })
            .collect();
        // A giving building the house can build and place now; for one its
        // tech tree still blocks, the building that clears the blocker (as a
        // person climbs the tree to a superweapon), the tech climb's own
        // rule. The cheapest of these is built next.
        let t = giving
            .iter()
            .copied()
            .filter(|&t| tech::buildable(w, self.house(), t))
            .filter(|&t| self.script.place_cell(w, t).is_some())
            .chain(
                giving
                    .iter()
                    .copied()
                    .filter(|&t| !tech::buildable(w, self.house(), t))
                    .filter_map(|t| self.script.unblocker(w, t, CLIMB_DEPTH)),
            )
            .min_by_key(|&t| (production::cost_field(w, t), t.index));
        if let Some(t) = t
            && !self.script.busy(w, t)
        {
            out.retain(|o| !matches!(o, Order::Build { what } if climb(what)));
            out.push(Order::Build { what: t });
        }
    }

    /// Fires every ready superweapon of the house (granted, ready and not
    /// suspended: what its sidebar icon shows) at the cell its kind's chooser
    /// gives ([`Brain::strike_cell`] for a missile or storm, and so on), with
    /// the fire order a person gives; a storm only while no storm is under
    /// way (a person sees one), a teleport's first stage only while no source
    /// is held.
    fn fire_superweapons(&mut self, w: &World, out: &mut Vec<Order>) {
        use glasnost_sim::superweapons::{StormState, SwKind, charge};
        let h = self.house();
        // The enemy units an earlier launch of this think already reaches:
        // a person does not fire a second superweapon at a group the first
        // one takes (the mutator's victims leave play at once).
        let mut spent: Vec<glasnost_sim::world::ObjectId> = Vec::new();
        for (i, t) in w.rules().superweapons.types.iter().enumerate() {
            let i = i as u32;
            let ready = charge::get(w, h, i).is_some_and(|r| r.granted && r.ready && !r.suspended);
            if !ready {
                continue;
            }
            let cell = match t.kind {
                Some(SwKind::Missile) => self.strike_cell(w),
                Some(SwKind::Storm) if w.sw.storm.state == StormState::Idle => self.strike_cell(w),
                Some(SwKind::Paradrop | SwKind::AmerParadrop) => self.drop_cell(w),
                Some(SwKind::IronCurtain) => self.curtain_cell(w).map(|(c, building)| {
                    if building {
                        self.special.count_curtained_building();
                    }
                    c
                }),
                Some(SwKind::ForceShield) => self.shield_cell(w),
                Some(SwKind::SpyPlane) => self.script.attack_goal(w),
                Some(SwKind::Chrono) if !Self::has_source(w, h) => self.group_cell(w),
                Some(SwKind::ChronoWarp) if Self::has_source(w, h) => self.landing_cell(w),
                Some(SwKind::PsychicReveal) => crate::psychic::reveal_cell(&self.script, w),
                Some(SwKind::Genetic) => crate::mutator::mutate_cell(&self.script, w),
                Some(SwKind::Dominator) => crate::dominator::dominate_cell(&self.script, w, &spent),
                _ => None,
            };
            if let Some(cell) = cell {
                out.push(Order::FireSuperweapon { sw: i, cell });
                if t.kind == Some(SwKind::PsychicReveal) {
                    self.special.count_reveal();
                }
                if t.kind == Some(SwKind::Genetic) {
                    self.special.count_mutator_launch();
                    spent.extend(crate::mutator::reached(w, cell));
                }
                if t.kind == Some(SwKind::Dominator) {
                    self.special.count_dominator_launch();
                    spent.extend(crate::dominator::reached(w, cell));
                }
            }
        }
    }

    /// Whether a first-stage teleport record of the house holds a picked
    /// source, so that a second stage has something to send (S-0513
    /// claim 34).
    fn has_source(w: &World, h: HouseId) -> bool {
        use glasnost_sim::superweapons::{SwKind, charge};
        w.rules()
            .superweapons
            .types
            .iter()
            .enumerate()
            .filter(|(_, t)| t.kind == Some(SwKind::Chrono))
            .any(|(i, _)| charge::get(w, h, i as u32).is_some_and(|r| r.source.is_some()))
    }

    /// Whether `o` is an armed ground vehicle of the plan's army that may
    /// stand in an effect block of an iron curtain or a teleport.
    fn army_vehicle(&self, w: &World, o: &glasnost_sim::world::Object) -> bool {
        o.kind == Kind::Vehicle
            && scripted::role_of(w, tech::type_of(o)) == Some(scripted::Role::Army)
            && !glasnost_sim::combat::veterancy::type_combat(w, o.id).is_some_and(|t| t.organic)
    }

    /// The cell of an effect block (S-0513 claim 4: the cell and its eight
    /// neighbours) holding the most of the house's armed vehicles, at least
    /// `min`, with none of its infantry or organic units (the teleport
    /// kills organic units, claim 35) and no unit of anyone else (it would
    /// send them too); the first in cell order on equal counts. With
    /// `fight`, a seen enemy must be within [`FIGHT`] cells.
    fn vehicle_block(&self, w: &World, fight: bool, min: usize) -> Option<CellXY> {
        let house = self.house();
        let enemies = self.script.seen_enemies(w);
        let mine: Vec<(CellXY, bool)> = w
            .objects
            .values()
            .filter(|o| o.on_map && o.health > 0 && o.kind != Kind::Building)
            .filter(|o| o.owner == Some(house))
            .map(|o| (o.pos.cell(), self.army_vehicle(w, o)))
            .collect();
        let others: Vec<CellXY> = w
            .objects
            .values()
            .filter(|o| o.on_map && o.health > 0 && o.owner != Some(house))
            .map(|o| o.pos.cell())
            .collect();
        let near = |a: CellXY, b: CellXY| (a.x - b.x).abs() <= 1 && (a.y - b.y).abs() <= 1;
        mine.iter()
            .filter(|&&(_, army)| army)
            .map(|&(c, _)| c)
            .filter(|&c| !fight || enemies.iter().any(|&(_, e)| dist2(e, c) <= FIGHT * FIGHT))
            .filter(|&c| !mine.iter().any(|&(m, army)| !army && near(m, c)))
            .filter(|&c| !others.iter().any(|&o| near(o, c)))
            .map(|c| {
                (
                    mine.iter().filter(|&&(m, army)| army && near(m, c)).count(),
                    c,
                )
            })
            .filter(|&(n, _)| n >= min)
            .max_by_key(|&(n, c)| (n, std::cmp::Reverse((c.y, c.x))))
            .map(|(_, c)| c)
    }

    /// An iron curtain's cell (M7b AI round 2), as a person aims it: the
    /// centre of an effect block (S-0513 claim 4, F1: the cell and its eight
    /// neighbours) on the house's own objects the curtain protects (claim
    /// 25: vehicles and buildings), where the protection matters now:
    ///
    /// - a block holding at least [`CURTAIN_VEHICLES`] of the house's armed
    ///   vehicles with a seen enemy within [`FIGHT`] cells (the vehicles
    ///   about to fight), centred on one of them, or
    /// - a block on one of the house's buildings that a seen enemy is
    ///   attacking (its target is the building), centred on the building's
    ///   reference cell.
    ///
    /// No block holds one of the house's or its allies' infantry (the
    /// curtain kills every reached infantry unit, claim 23) or another
    /// house's vehicle or building (it would be protected too, claim 25);
    /// enemy infantry in the block is welcome (it dies, claim 23). Of these
    /// blocks the one whose protected own objects cost the most is fired
    /// at, the first in cell order on equal worth. Returns the cell and
    /// whether the block was chosen for a building under attack.
    ///
    /// The old aim wanted three vehicles in one block with no other house's
    /// unit at all; in the lane the curtain was ready for 7 900 to 51 000
    /// house-ticks and fired once in 7 games.
    fn curtain_cell(&self, w: &World) -> Option<(CellXY, bool)> {
        let house = self.house();
        let me = &w.houses[usize::from(house.0)];
        let friend = |h: Option<HouseId>| h.is_some_and(|h| h == house || me.allies.contains(&h));
        let enemies = self.script.seen_enemies(w);
        // The objects a block reaches, as a person sees them standing in
        // its cells (claim 4: listed in a swept layer of a block cell, so
        // not in flight and not inside); a building once.
        let reached_by = |c: CellXY| {
            use glasnost_sim::superweapons as sw;
            let mut ids: Vec<ObjectId> = sw::block(w, c)
                .into_iter()
                .flat_map(|(_, x)| {
                    sw::swept_layers(w, x)
                        .into_iter()
                        .flat_map(move |l| sw::listed(w, x, l))
                })
                .collect();
            ids.sort();
            ids.dedup();
            ids.into_iter()
                .filter_map(|id| w.obj(id))
                .filter(|o| o.health > 0)
                .collect::<Vec<_>>()
        };
        let attacked: Vec<ObjectId> = enemies
            .iter()
            .filter_map(|&(e, _)| match w.obj(e)?.target {
                Some(Target::Object(t))
                    if w.obj(t)
                        .is_some_and(|b| b.kind == Kind::Building && b.owner == Some(house)) =>
                {
                    Some(t)
                }
                _ => None,
            })
            .collect();
        let mut cands: Vec<(CellXY, bool)> = w
            .objects
            .values()
            .filter(|o| {
                o.on_map && o.health > 0 && o.owner == Some(house) && self.army_vehicle(w, o)
            })
            .map(|o| (o.pos.cell(), false))
            .chain(
                attacked
                    .iter()
                    .filter_map(|&b| w.obj(b))
                    .map(|b| (w.reference_point(b.id).cell(), true)),
            )
            .collect();
        cands.sort_by_key(|&(c, b)| (c.y, c.x, b));
        cands.dedup();
        cands
            .into_iter()
            .filter_map(|(c, building)| {
                let reached = reached_by(c);
                // spec: S-0513 claim 23 (every reached infantry unit dies, of any house)
                let own_infantry = reached
                    .iter()
                    .any(|o| o.kind == Kind::Infantry && friend(o.owner));
                // spec: S-0513 claim 25 (every other reached vehicle, aircraft and building is protected)
                let protects_others = reached
                    .iter()
                    .any(|o| o.kind != Kind::Infantry && !friend(o.owner) && o.owner.is_some());
                if own_infantry || protects_others {
                    return None;
                }
                let vehicles = reached
                    .iter()
                    .filter(|o| o.owner == Some(house) && self.army_vehicle(w, o))
                    .count();
                let fighting = enemies.iter().any(|&(_, e)| dist2(e, c) <= FIGHT * FIGHT);
                let worth_it = if building {
                    reached.iter().any(|o| attacked.contains(&o.id))
                } else {
                    vehicles >= CURTAIN_VEHICLES && fighting
                };
                if !worth_it {
                    return None;
                }
                let worth: i64 = reached
                    .iter()
                    .filter(|o| o.owner == Some(house) && o.kind != Kind::Infantry)
                    .map(|o| i64::from(production::cost_field(w, tech::type_of(o))))
                    .sum();
                Some((worth, c, building))
            })
            .max_by_key(|&(worth, c, _)| (worth, std::cmp::Reverse((c.y, c.x))))
            .map(|(_, c, building)| (c, building))
    }

    /// A teleport source's cell: a block of the house's armed vehicles.
    fn group_cell(&self, w: &World) -> Option<CellXY> {
        self.vehicle_block(w, false, TELEPORT_MIN)
    }

    /// A force shield's cell: the reference cell of one of the house's
    /// buildings with a seen enemy within [`DEFEND`] cells, whose shield
    /// takes the most buildings of the house and its allies, at least
    /// [`SHIELD_MIN`]; the first in cell order on equal counts. The shield
    /// protects buildings of the launching house and its allies whose
    /// reference point is within the shield's radius of the target cell's
    /// centre, and nothing else (S-0513 claim 31, F3: the radial indicator
    /// a person reads off the cursor; the height is left out). The M7b AI
    /// audit found the shield ready in every Yuri game and never fired: the
    /// old aim counted buildings by their cell in a 3 × 3 block, which two
    /// buildings with a gap between them never share.
    fn shield_cell(&self, w: &World) -> Option<CellXY> {
        let house = self.house();
        let me = &w.houses[usize::from(house.0)];
        let enemies = self.script.seen_enemies(w);
        let r = 256 * i64::from(w.rules().superweapons.shield_radius); // spec: S-0513 F3 (256 leptons per cell of the radius R)
        let points: Vec<(bool, CellXY, i64, i64)> = w
            .objects
            .values()
            .filter(|o| o.on_map && o.health > 0 && o.kind == Kind::Building)
            .filter(|o| {
                o.owner
                    .is_some_and(|h| h == house || me.allies.contains(&h))
            })
            .map(|o| {
                let p = w.reference_point(o.id);
                (
                    o.owner == Some(house),
                    p.cell(),
                    i64::from(p.x),
                    i64::from(p.y),
                )
            })
            .collect();
        let mut cands: Vec<CellXY> = points
            .iter()
            .filter(|&&(own, _, _, _)| own)
            .map(|&(_, c, _, _)| c)
            .filter(|&c| enemies.iter().any(|&(_, e)| dist2(e, c) <= DEFEND * DEFEND))
            .collect();
        cands.sort_by_key(|c| (c.y, c.x));
        cands.dedup();
        cands
            .into_iter()
            .map(|c| {
                let (cx, cy) = (i64::from(c.x) * 256 + 128, i64::from(c.y) * 256 + 128); // spec: S-0513 F3 (the target cell centre)
                let n = points
                    .iter()
                    .filter(|&&(_, _, x, y)| (x - cx) * (x - cx) + (y - cy) * (y - cy) < r * r)
                    .count();
                (n, c)
            })
            .filter(|&(n, _)| n >= SHIELD_MIN)
            .max_by_key(|&(n, c)| (n, std::cmp::Reverse((c.y, c.x))))
            .map(|(_, c)| c)
    }

    /// The teleport's second-stage cell: the first cell, ring by ring
    /// around the enemy base from [`LAND_RINGS`]'s nearest ring outwards,
    /// in cell order within a ring, whose whole effect block is usable
    /// land with nothing standing on it.
    fn landing_cell(&self, w: &World) -> Option<CellXY> {
        let goal = self.script.attack_goal(w)?;
        let taken: std::collections::BTreeSet<CellXY> = w
            .objects
            .values()
            .filter(|o| o.on_map)
            .map(|o| o.pos.cell())
            .collect();
        let free = |c: CellXY| {
            w.cells.usable(c)
                && !taken.contains(&c)
                && w.cells
                    .cell(c)
                    .is_some_and(|x| x.land != land::WATER && x.land != land::ROCK)
        };
        (LAND_RINGS.0..=LAND_RINGS.1).find_map(|r| {
            (-r..=r)
                .flat_map(|dy| (-r..=r).map(move |dx| (dx, dy)))
                .filter(|&(dx, dy)| dx.abs().max(dy.abs()) == r)
                .map(|(dx, dy)| CellXY::new(goal.x + dx, goal.y + dy))
                .find(|&c| {
                    (-1..=1).all(|dy| (-1..=1).all(|dx| free(CellXY::new(c.x + dx, c.y + dy))))
                })
        })
    }

    /// A paradrop's cell: the first cell, ring by ring around home from
    /// [`DROP_RINGS`]'s nearest ring outwards, in cell order within a ring,
    /// that is inside the usable area and holds no object.
    fn drop_cell(&self, w: &World) -> Option<CellXY> {
        let home = self.script.home(w)?;
        let taken: std::collections::BTreeSet<CellXY> = w
            .objects
            .values()
            .filter(|o| o.on_map)
            .map(|o| o.pos.cell())
            .collect();
        (DROP_RINGS.0..=DROP_RINGS.1).find_map(|r| {
            (-r..=r)
                .flat_map(|dy| (-r..=r).map(move |dx| (dx, dy)))
                .filter(|&(dx, dy)| dx.abs().max(dy.abs()) == r)
                .map(|(dx, dy)| CellXY::new(home.x + dx, home.y + dy))
                .find(|&c| w.cells.usable(c) && !taken.contains(&c))
        })
    }

    /// The seen enemy's cell with the most seen enemies within
    /// [`STRIKE_RADIUS`] and none of the house's own objects; the first in
    /// cell order on equal counts.
    fn strike_cell(&self, w: &World) -> Option<CellXY> {
        let enemies = self.script.seen_enemies(w);
        let own: Vec<CellXY> = self
            .script
            .own(w)
            .into_iter()
            .filter_map(|id| w.obj(id).map(|o| o.pos.cell()))
            .collect();
        let r2 = STRIKE_RADIUS * STRIKE_RADIUS;
        enemies
            .iter()
            .map(|&(_, c)| c)
            .filter(|&c| !own.iter().any(|&o| dist2(o, c) <= r2))
            .max_by_key(|&c| {
                let n = enemies.iter().filter(|&&(_, e)| dist2(e, c) <= r2).count();
                (n, std::cmp::Reverse((c.y, c.x)))
            })
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
            && !scripted::armed(u)
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
        let (Some(home), Some(goal)) = (self.script.home(w), self.script.base_goal(w)) else {
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
            let at = o.pos.cell();
            let aboard = transport::count(w, f);
            // A passenger that finds no exit blocks those behind it, and the
            // transport keeps retrying until the unload is interrupted
            // (S-0507 claim 10). A person sees the ferry stand there full and
            // moves it on to land nearby, where the unload click is made
            // again.
            if o.beh.current == Some(Behaviour::Unload)
                && aboard > 0
                && o.dest.is_none()
                && w.tick - o.beh.start_tick > UNLOAD_STUCK
            {
                if let Some(c) = self.land_cell(w, f, at, (1, ASHORE), at, None) {
                    out.push(Order::Behaviour {
                        actor: f,
                        behaviour: Behaviour::Move,
                        attack_move: false,
                        target: None,
                        dest: Some(Target::Cell(c)),
                    });
                }
                continue;
            }
            if !idle(w, f) {
                continue;
            }
            let room = transport::capacity(w, f);
            let at_home = dist2(at, home) <= FERRY_HOME * FERRY_HOME;
            let go = |c: CellXY| Order::Behaviour {
                actor: f,
                behaviour: Behaviour::Move,
                attack_move: false,
                target: None,
                dest: Some(Target::Cell(c)),
            };
            // Leave for the goal: land near it from which the passengers'
            // own zone reaches it.
            let to_goal = || {
                let zone = transport::passengers(w, f)
                    .first()
                    .and_then(|&p| w.obj(p))
                    .and_then(|p| w.rules().unit(p.kind, p.type_index))
                    .map(|u| u.zone);
                self.land_cell(w, f, goal, (DROP_NEAR, DROP_FAR), at, zone)
            };
            // A loaded ferry that has drifted just outside the home radius
            // would only drop infantry that are gathered and boarded again:
            // it leaves for the goal instead.
            let near_home = dist2(at, home) <= 4 * FERRY_HOME * FERRY_HOME; // nocite: tuning (the gathering radius above)
            // With no drop cell to go to, the ferry lets them out instead.
            if !at_home
                && aboard > 0
                && near_home
                && let Some(c) = to_goal()
            {
                out.push(go(c));
                continue;
            }
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
                if let Some(c) = to_goal() {
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

    /// The seen enemies a person answers first (M7b AI round 2), each with
    /// its cell: a drainer draining one of the house's power plants, which
    /// blacks out the whole house (S-0405 claim 5) and whose building
    /// cannot be sold (S-0408 claim 3), and an enemy attacking one of the
    /// house's harvesters, slave miners or slaves (its target is the
    /// worker: a person sees the shots). In the lane the Allied side's
    /// waves killed up to 165 slaves of a Yuri house in one game, mostly
    /// with infantry, while the Yuri army stood near by.
    fn raiders(&self, w: &World) -> Vec<(ObjectId, CellXY)> {
        let house = self.house();
        let seen = self.script.seen_enemies(w);
        let drainers: Vec<ObjectId> = crate::power::drained_plants(w, house)
            .into_iter()
            .map(|(_, d)| d)
            .collect();
        let worker = |id: ObjectId| {
            w.obj(id).is_some_and(|o| {
                o.owner == Some(house)
                    && (o.econ.harvest.is_some()
                        || scripted::role_of(w, tech::type_of(o)) == Some(Role::Harvester)
                        || o.econ.slave.is_some()
                        || glasnost_sim::econ::slaves::is_master(w, id))
            })
        };
        seen.into_iter()
            .filter(|&(e, _)| {
                drainers.contains(&e)
                    || w.obj(e)
                        .is_some_and(|o| matches!(o.target, Some(Target::Object(t)) if worker(t)))
            })
            .collect()
    }

    /// Defends the base: every army unit near home that is not already
    /// fighting an intruder attacks the intruder nearest to it. A raider
    /// ([`Brain::raiders`]) comes first, for every army unit that can
    /// attack it and is near home or idle, within [`RECALL`] cells of it.
    fn defend(&mut self, w: &World, out: &mut Vec<Order>) {
        let intruders = self.intruders(w);
        let raiders = self.raiders(w);
        if intruders.is_empty() && raiders.is_empty() {
            return;
        }
        let Some(home) = self.script.home(w) else {
            return;
        };
        for id in self.script.of_role(w, Role::Army) {
            let Some(o) = w.obj(id) else { continue };
            let at = o.pos.cell();
            let near_home = dist2(at, home) <= RECALL * RECALL;
            let busy = matches!(o.target, Some(Target::Object(t))
                if intruders.iter().chain(&raiders).any(|&(e, _)| e == t));
            // A unit on its way into a recycling building (S-0603) is left to go;
            // so is a vehicle on its way into a tank bunker, and a bunkered one
            // is left to the bunker part (`bunker`), which lets it out (S-0606).
            let bound = o.beh.current == Some(Behaviour::Eaten)
                || o.beh.queued == Some(Behaviour::Eaten)
                || (o.beh.current == Some(Behaviour::Enter)
                    && glasnost_sim::structures::bunker::entering_bunker(w, id))
                || glasnost_sim::structures::bunker::bunkered(w, id);
            if busy || bound {
                continue;
            }
            // A unit already sent at the raider from where it stands or
            // lately (`Special::defended_lately`), or one that did not take
            // that order (`memory`), is not sent again: a person does not
            // click the same every second.
            let memory = &self.special.memory;
            let raider = (near_home || idle(w, id))
                .then(|| {
                    raiders
                        .iter()
                        .filter(|&&(e, c)| {
                            dist2(c, at) <= RECALL * RECALL
                                && can_attack(w, id, e)
                                && memory.ok(w, id, Target::Object(e))
                                && !memory.pending(w, id, Target::Object(e))
                                && !self.special.defended_lately(w, id, e)
                        })
                        .min_by_key(|&&(e, c)| (dist2(c, at), e))
                })
                .flatten();
            let pick = raider.copied().or_else(|| {
                near_home
                    .then(|| {
                        intruders
                            .iter()
                            .filter(|&&(e, _)| can_attack(w, id, e))
                            .min_by_key(|&&(e, c)| (dist2(c, at), e))
                            .copied()
                    })
                    .flatten()
            });
            if let Some((e, _)) = pick {
                let order = Order::Behaviour {
                    actor: id,
                    behaviour: Behaviour::Attack,
                    attack_move: false,
                    target: Some(Target::Object(e)),
                    dest: None,
                };
                if raider.is_some() {
                    self.special.count_economy_defence();
                    self.special.memory.note(w, std::slice::from_ref(&order));
                    self.special.note_defended(w, id, e);
                }
                out.push(order);
            }
        }
    }
}
