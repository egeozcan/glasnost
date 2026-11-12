//! The brain of one AI house (PLAN §4.7, M5b): what it remembers between
//! ticks and how it thinks. Its state is saved with the game
//! ([`crate::game::Game`]).
//!
//! It plays in two layers:
//!
//! - **the plan** ([`Script`], the M5a scripted builder and attack waves),
//!   every [`THINK_PERIOD`](scripted::THINK_PERIOD) ticks at its house's
//!   phase ([`scripted::think_phase`]): build-up, harvesters, army, waves; it
//!   rebuilds what the house lost of its power, refineries, factories and
//!   harvesters, because it asks for them whenever the house has none (or
//!   fewer harvesters than the plan keeps);
//! - **the reactions**, every tick (no reaction delay): place a finished
//!   building at once, defend the base against seen enemies near its
//!   buildings, and rebuild a lost construction yard (build a unit that
//!   deploys into one, then deploy it);
//! - **the ferry** (S-0507), with the plan: one unarmed
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
use glasnost_sim::probe::{Phase, Probe};
use glasnost_sim::regions;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::transport;
use glasnost_sim::world::{HouseId, Layer, ObjectId, Target, World};
use serde::{Deserialize, Serialize};

use crate::scripted::{
    self, CLIMB_DEPTH, Plan, Role, SPIRAL_RINGS, Script, all_types, can_attack, deploys_into_yard,
    dist2, idle, is_yard,
};
use crate::skirmish::start_cells;

/// Cells from one of the house's buildings within which a seen enemy counts
/// as attacking the base.
const DEFEND: i64 = 12; // nocite: tuning of our own opponent

/// Cells from the house's home within which army units are called back to
/// defend the base; a wave further out goes on.
pub(crate) const RECALL: i64 = 30; // nocite: tuning of our own opponent

/// An idle army vehicle with health at or below this share (in percent)
/// of its strength is sent to a repair depot.
const REPAIR_BELOW: i64 = 50; // nocite: tuning of our own opponent
/// The scale of [`REPAIR_BELOW`].
const PERCENT: i64 = 100; // nocite: percent scale

/// Construction yards, and units that deploy into one, beyond which the
/// house builds no yard vehicle to expand (`Brain::expand`).
const EXPANSION_YARDS: usize = 2; // nocite: tuning of our own opponent

/// Multiples of a yard vehicle's cost the house has in hand, beyond what
/// its superweapon building still owes, before it builds one to expand.
const EXPANSION_RICH: i64 = 2; // nocite: tuning of our own opponent

/// Multiples of a yard vehicle's cost the house earns per income window
/// (`scripted::INCOME_WINDOW`) before it buys one with less money in hand
/// (`Brain::rich`): it then pays for itself within a window.
const EXPANSION_EARNING: i64 = 1; // nocite: tuning of our own opponent

/// A house that earns less than a yard vehicle's cost divided by this per
/// income window is broke for the expansion (`Brain::drop_expansion`);
/// below [`EXPANSION_EARNING`], so that a house whose income hovers at the
/// threshold does not buy and cancel in turn.
const EXPANSION_BROKE: i64 = 2; // nocite: tuning of our own opponent

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
                "; ferry: idle infantry at home {}, run crosses water {crossing}, ferries owned {}, type for a land run {}, type for a crossing {}, its line busy {}",
                infantry.len(),
                self.ferries(w).len(),
                self.ferry_type(w, false).is_some(),
                self.ferry_type(w, true).is_some(),
                self.ferry_type(w, crossing).is_some_and(|t| self.script.busy(w, t))
            ));
        }
        s
    }

    /// Whether the brain's plan and the parts that go with it think in tick
    /// `tick`: every [`THINK_PERIOD`](scripted::THINK_PERIOD) ticks, at the
    /// house's phase ([`scripted::think_phase`]). The reactions run in every
    /// tick.
    pub fn thinks_at(&self, tick: i32) -> bool {
        scripted::thinks_at(self.house(), tick)
    }

    /// Thinks at the start of the world's current tick; returns the orders
    /// to issue, in the order to issue them (a later order to the same unit
    /// replaces an earlier one, so the reactions come last).
    pub fn think(&mut self, w: &World) -> Vec<Order> {
        self.think_with(w, &mut ())
    }

    /// [`Brain::think`], telling `p` which part of the brain runs (the
    /// tools' benchmark, `glasnost_sim::probe`); `p` reads nothing and
    /// changes nothing.
    pub fn think_with<P: Probe>(&mut self, w: &World, p: &mut P) -> Vec<Order> {
        let on = p.on();
        let mark = |p: &mut P, part: &'static str| {
            if on {
                p.mark(Phase::Part(part));
            }
        };
        // The world does not change during the think: placement answers are kept for it.
        self.script.begin_think();
        mark(p, "ai:script");
        let thinks = self.thinks_at(w.tick);
        let mut out = if thinks {
            self.script.think_with(w, p)
        } else {
            let mut v = Vec::new();
            self.place_done(w, &mut v);
            v
        };
        let mut short = false;
        if thinks {
            mark(p, "ai:economy_first");
            let paying;
            (paying, short) = self.economy_first(w, &mut out);
            mark(p, "ai:drop_expansion");
            self.drop_expansion(w, &mut out);
            mark(p, "ai:send_for_repair");
            self.send_for_repair(w, &mut out);
            mark(p, "ai:ferry");
            self.ferry(w, &mut out);
            mark(p, "ai:build_superweapon");
            self.build_superweapon(w, &mut out);
            mark(p, "ai:build_specials");
            let sensors = self.special_uses().sensors;
            self.build_sensor(w, &mut out);
            if self.special_uses().sensors == sensors && !self.build_grinder(w, &mut out) {
                let defences = self.special_uses().gattling_defences;
                self.build_gattling_defence(w, &mut out);
                if self.special_uses().gattling_defences == defences && !paying {
                    self.build_bunker(w, &mut out);
                }
            }
            mark(p, "ai:build_lifter");
            self.build_lifter(w, &mut out);
            mark(p, "ai:build_drainer");
            self.build_drainer(w, &mut out);
            mark(p, "ai:build_anti_air");
            self.build_anti_air(w, &mut out);
            mark(p, "ai:expand");
            if !paying {
                self.expand(w, &mut out, p);
            }
        }
        if thinks {
            mark(p, "ai:special");
            self.special.think_with(&self.script, w, &mut out, p);
            if short {
                // No new non-fighter while the superweapon building waits
                // for money: `economy_first` would cancel it (M8 step B7).
                let has_yard = self.has_yard(w);
                out.retain(
                    |o| !matches!(o, Order::Build { what } if non_fighter(w, *what, has_yard)),
                );
            }
            mark(p, "ai:plant_first");
            if !self.script.selling() {
                let plant = self.script.plant_type(w);
                // design: a lost refinery is not put behind a power plant
                // (M8 step B8: in the quick lane the plant that replaced it
                // was cancelled for it at the next think, over and over).
                // design: nor is a refinery by a new field while the house
                // cannot pay for both (M8 step B10): with no money the plant
                // would take what the refinery needs for its income.
                let credits = i64::from(w.houses[usize::from(self.house().0)].credits);
                let both = |t: TypeRef| {
                    plant.map_or(0, |p| i64::from(production::price(w, self.house(), p)))
                        + i64::from(production::price(w, self.house(), t))
                };
                let lost = self.script.refinery_lost(w).or_else(|| {
                    self.script
                        .field_refinery_type(w)
                        .filter(|&t| credits < both(t))
                });
                let n = crate::power::plant_first(w, self.house(), plant, lost, &mut out);
                self.special.count_plants_first(n);
            }
        }
        mark(p, "ai:fire_superweapons");
        self.fire_superweapons(w, &mut out);
        mark(p, "ai:rebuild_yard");
        self.rebuild_yard(w, &mut out);
        mark(p, "ai:find_deploy_spot");
        self.find_deploy_spot(w, &mut out);
        mark(p, "ai:defend");
        self.defend(w, &mut out);
        mark(p, "ai:filters");
        // spec: S-0214 claim 15 and S-0610 claim 6 (a berserk unit takes no orders: a person cannot give it one, so neither does the brain)
        out.retain(|o| {
            o.actor()
                .is_none_or(|a| !glasnost_sim::psychic::berserk(w, a))
        });
        // design: no attack order and no attack move (from any part of the brain) to an aircraft with no ammunition left (S-0501 T1). The order is offered (S-0214 claim 6 (e)), and the aircraft takes off for it (S-0501 claim 8), ends its run without a shot (T1) and flies back to its pad, losing the service time; a person waits for it to rearm (M8 step A4 round 8, the lead's note 50: `defend` sent spent aircraft from their pads at intruders)
        out.retain(|o| match o {
            Order::Behaviour {
                actor,
                behaviour,
                attack_move,
                ..
            } if *behaviour == Behaviour::Attack || *attack_move => {
                w.obj(*actor).is_none_or(|x| x.kind != Kind::Aircraft)
                    || glasnost_sim::air::ammo_left(w, *actor)
            }
            _ => true,
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
        // S-0213 claims 3 and 10: the deploy order queues unload, and a
        // behaviour order of the same tick for the same actor determines
        // that state whatever the issue order; a unit given the deploy order
        // (`deployers`) takes no behaviour order in the same think, as a
        // person who packs a unit up does not click it elsewhere in the same
        // moment (M8 step B7).
        let deploying: std::collections::BTreeSet<ObjectId> = out
            .iter()
            .filter_map(|o| match o {
                Order::Deploy { actor } => Some(*actor),
                _ => None,
            })
            .collect();
        if !deploying.is_empty() {
            out.retain(
                |o| !matches!(o, Order::Behaviour { actor, .. } if deploying.contains(actor)),
            );
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
        // S-0601 claims 2, 7 and 9: a capacity-1 controller that holds a victim may still fire its takeover weapon at a new target, and that takeover releases the victim back to its owner; no part of the brain sends one at capacity at another object with that weapon, as a person keeps the victim rather than swapping (our own design, PLAN §4.7; M8 seam trace 4: a controller swapped a plant and a tank back and forth while the army's attack on a near enemy sent it at whichever was the enemy's)
        out.retain(|o| match o {
            Order::Behaviour {
                actor,
                behaviour: Behaviour::Attack,
                target: Some(t @ Target::Object(_)),
                ..
            } => !crate::mind::swaps_victim(w, *actor, *t),
            _ => true,
        });
        self.script.end_think();
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
    /// building of its own choosing is started then), and whether a
    /// superweapon building waits for money (no non-fighter is started
    /// then).
    fn economy_first(&mut self, w: &World, out: &mut Vec<Order>) -> (bool, bool) {
        use glasnost_sim::econ::production::Hold;
        let h = self.house();
        let economy = |t: TypeRef| is_economy(w, t);
        let paying = self.paying(w);
        let low_power = crate::power::short(w, h);
        // Saving for a superweapon (M7b AI round 3, narrowed in M8 step
        // B2): while a building that gives a superweapon the house fires
        // runs on its line owing more than the money in hand, and the house
        // is safe (`Self::safe`), the army's running unit items that still
        // owe money are put on hold, as a person stops training units to
        // finish the superweapon. With less than the balance in hand, every
        // credit the army spends is one the building still needs, so holding
        // the army shortens the time to the superweapon (S-0404 claim 4: a
        // step is paid from the money in hand or waits). In the stock lane
        // the second superweapon building of the Yuri side stood on its line
        // for 9 000 to more than 28 000 ticks with the house's credits near 0
        // while every army line drew on them, and was granted in only 2 of 6
        // full games. M8 step B2 tried a hold only while the money in hand
        // did not pay the building's next step: the army spent what the
        // later steps needed, the house held and resumed on most thinks
        // (up to 442 holds in a quick game, against 60 before) and every
        // superweapon came later (the first force shield by a median 1 936
        // ticks). What B2 kept: no hold while the house loses a fight
        // (`Self::losing`), and only an item that still owes money is held.
        // A yard vehicle bought to expand (`Self::expand`, M8 step B1) also
        // draws on that money and is held with the army; one bought while
        // the house has no yard is its way back and runs on. M8 step B5
        // narrows B2's rule: while the house is not safe, the building
        // still short, only the army that can join the fight resumes; a
        // yard vehicle and an engineer, which do not fight, stay held. In
        // the lane a house losing a fight for 300 ticks put 23 steps into
        // an engineer and 8 into a tank while the mutator building waited
        // for money. Held, the engineer's share went to the tank instead
        // (the earlier line is paid first, S-0404 claim 1), and that game's
        // mutator did not come sooner. M8 step B7: a held item keeps its
        // line and the queue does not advance (S-0404 claim 5), so the held
        // engineer kept the infantry line from the fighters for thousands of
        // ticks (B6's 12-2-13: 8 400 to 19 666), the base was raided and the
        // dominator waited for money. While the house is short and not
        // safe, a non-fighter is now cancelled (the refund at an unchanged
        // price is everything paid, S-0404 claim 6), as a person clears the
        // barracks for the soldiers the base needs, and none is started
        // while the building is short.
        let has_yard = self.has_yard(w);
        let lost = self.script.refinery_lost(w);
        let refund_for = self.refund_for_refinery(w, lost, paying);
        let army = |t: TypeRef| army_item(w, t, has_yard);
        let short = !paying && !self.script.selling() && self.superweapon_short(w);
        let saving = short && self.safe(w);
        for l in w.econ.lines.lines.iter().filter(|l| l.house == h) {
            let Some(i) = &l.item else { continue };
            if i.done() || economy(i.t) {
                continue;
            }
            if paying && lost.is_some_and(|t| production::line_kind(w, t) == Some(l.kind)) {
                // design: the item on the line a lost refinery needs is
                // cancelled, not held: a held item keeps its line (S-0404
                // claim 5), and the refinery could not start (M8 step B8).
                // spec: S-0404 claim 6 (the cancel-all order; at an unchanged price the refund is everything paid)
                out.push(Order::CancelAll { what: i.t });
            } else if refund_for == Some(i.object) {
                // spec: S-0404 claim 6 (the cancel-all order; the refund P′ − B)
                out.push(Order::CancelAll { what: i.t });
            } else if paying {
                let power = scripted::role_of(w, i.t) == Some(Role::Power);
                if i.hold == Hold::None && !(power && low_power) {
                    out.push(Order::Hold { what: i.t });
                    self.special.count_economy_hold();
                }
            } else if army(i.t) && saving {
                // Only an item that still owes money competes for it.
                if i.hold == Hold::None && i.owed > 0 {
                    // spec: S-0404 claim 5 (the hold order)
                    out.push(Order::Hold { what: i.t });
                    self.special.count_superweapon_hold();
                }
            } else if short && non_fighter(w, i.t, has_yard) {
                // A non-fighter is cancelled, not held: a held item keeps
                // its line (S-0404 claim 5), and the fighters the house
                // needs now come from that line.
                if i.owed > 0 {
                    // spec: S-0404 claim 6 (the cancel-all order; at an unchanged price the refund is everything paid)
                    out.push(Order::CancelAll { what: i.t });
                    self.special.count_non_fighter_cancel();
                }
            // design: own AI (#60): the tech tree's answer kept for the think
            } else if i.hold == Hold::Player && self.script.buildable(w, i.t) {
                out.push(Order::Build { what: i.t });
                self.special.count_resume();
            }
        }
        (paying, short)
    }

    /// The item the house cancels at this think to pay for a lost
    /// refinery (M8 step B9): while it pays for its economy first
    /// ([`Self::paying`]) because its harvesters wait for a lost refinery
    /// (`Script::refinery_lost`), and the money in hand is less than the
    /// refinery still owes (its whole price before it is on a line), the
    /// item not of the economy, not done and not on the refinery's line
    /// whose cancel refunds the most (S-0404 claim 6: *P*′ − *B*, or the
    /// price at the start with `quirks.refund_at_current_price` off), when
    /// that refund is above 0; the earliest line on a tie. One a think, so
    /// that no more is cancelled than the refinery needs. A house with no
    /// refinery has no income: held, those items kept what was paid into
    /// them while the refinery waited for money (M8 step B9: in the quick
    /// lane a broke house held a defence building with about 840 credits
    /// paid and an infantry item while its refinery stood at 11 to 21 of 54
    /// for more than 5 000 ticks), and a person cancels them to finish the
    /// refinery.
    fn refund_for_refinery(
        &self,
        w: &World,
        lost: Option<TypeRef>,
        paying: bool,
    ) -> Option<ObjectId> {
        let t = lost.filter(|_| paying)?;
        let h = self.house();
        let mine = || w.econ.lines.lines.iter().filter(|l| l.house == h);
        let need = mine()
            .filter_map(|l| l.item.as_ref())
            .find(|i| i.t == t)
            .map_or_else(|| production::price(w, h, t), |i| i.owed);
        if w.houses[usize::from(h.0)].credits >= need {
            return None;
        }
        let refund = |i: &production::Item| {
            // spec: S-0404 claim 6 and F4 (refund = P′ − B; quirks.refund_at_current_price off: P − B)
            let p = if w.rules().quirks.refund_at_current_price {
                production::price(w, h, i.t)
            } else {
                i.start_price
            };
            p - i.owed
        };
        let refinery_line = production::line_kind(w, t);
        let mut best: Option<(i32, ObjectId)> = None;
        for l in mine().filter(|l| Some(l.kind) != refinery_line) {
            let Some(i) = &l.item else { continue };
            if i.done() || is_economy(w, i.t) {
                continue;
            }
            let r = refund(i);
            if r > 0 && best.is_none_or(|(b, _)| r > b) {
                best = Some((r, i.object));
            }
        }
        best.map(|(_, id)| id)
    }

    /// Whether the house owns a construction yard.
    fn has_yard(&self, w: &World) -> bool {
        self.script
            .own(w)
            .iter()
            .any(|&id| w.obj(id).is_some_and(|o| is_yard(w, o)))
    }

    /// Whether the house is safe enough to save for a superweapon
    /// building: not selling out, no seen enemy near its base, its army not
    /// outnumbered by the enemy units it sees, and no fight it is losing
    /// ([`Self::losing`]). M8 step B7 tried a fight lost far off (more than
    /// [`RECALL`] cells from home) as no end to the saving, the army lines
    /// then waiting while an attack abroad went badly; the lane played
    /// worse (wins 29 to 24, the dominator ready in 26 games to 19), so a
    /// lost fight anywhere still ends it. The probe keeps the split
    /// ([`Self::losing_at_home`]).
    fn safe(&self, w: &World) -> bool {
        !self.script.selling()
            && self.intruders(w).is_empty()
            && !self.outnumbered(w)
            && !self.losing(w)
    }

    /// Whether the house is losing a fight ([`Self::losing`]) within
    /// [`RECALL`] cells of home (the expansion probe's view, M8 step B7).
    fn losing_at_home(&self, w: &World) -> bool {
        self.script
            .home(w)
            .is_some_and(|h| self.losing_where(w, |c| dist2(c, h) <= RECALL * RECALL))
    }

    /// The balance owed (S-0404 Terms) of the running item of a building
    /// that gives a superweapon the house fires, if one is on its lines.
    fn superweapon_owed(&self, w: &World) -> Option<i32> {
        use glasnost_sim::econ::production::Hold;
        let h = self.house();
        w.econ
            .lines
            .lines
            .iter()
            .filter(|l| l.house == h)
            .filter_map(|l| l.item.as_ref())
            .find(|i| {
                i.t.kind == Kind::Building
                    && !Self::giving_kinds(w, i.t.index).is_empty()
                    && i.hold == Hold::None
                    && !i.done()
            })
            .map(|i| i.owed)
    }

    /// Whether the house saves for a superweapon building now
    /// ([`Self::economy_first`]): safe, and such a building runs on its
    /// line owing more than the money in hand.
    fn saving(&self, w: &World) -> bool {
        self.safe(w) && self.superweapon_short(w)
    }

    /// Whether a superweapon building runs on the house's lines owing more
    /// than the money in hand.
    fn superweapon_short(&self, w: &World) -> bool {
        let credits = w.houses[usize::from(self.house().0)].credits;
        self.superweapon_owed(w).is_some_and(|owed| credits < owed)
    }

    /// Whether the house is losing a fight, as a person reads it off the
    /// map: seen enemy army units shoot at its objects, its own army units
    /// fight them (some stand within [`FIGHT`] cells of the shooters), and
    /// the seen enemy army units there are worth at least as much (by cost)
    /// as its own. Shots with no army of the house near them are a raid,
    /// which the base defence and the economy defence answer; counted as a
    /// lost fight, they ended the saving in about two of every five lane
    /// samples.
    fn losing(&self, w: &World) -> bool {
        self.losing_where(w, |_| true)
    }

    /// [`Self::losing`] for the shooters standing where `at` holds.
    fn losing_where(&self, w: &World, at: impl Fn(CellXY) -> bool) -> bool {
        let house = self.house();
        let army = |o: &glasnost_sim::world::Object| {
            o.kind != Kind::Building && scripted::role_of(w, tech::type_of(o)) == Some(Role::Army)
        };
        let seen: Vec<&glasnost_sim::world::Object> = self
            .script
            .seen_enemies(w)
            .iter()
            .filter_map(|&(id, _)| w.obj(id))
            .filter(|o| army(o))
            .collect();
        let shooters: Vec<CellXY> = seen
            .iter()
            .filter(|o| {
                matches!(o.target, Some(Target::Object(v))
                    if w.obj(v).is_some_and(|v| v.owner == Some(house)))
            })
            .map(|o| o.pos.cell())
            .filter(|&c| at(c))
            .collect();
        if shooters.is_empty() {
            return false;
        }
        let near = |c: CellXY| shooters.iter().any(|&s| dist2(s, c) <= FIGHT * FIGHT);
        let worth = |o: &glasnost_sim::world::Object| {
            i64::from(production::cost_field(w, tech::type_of(o)))
        };
        let enemy: i64 = seen
            .iter()
            .filter(|o| near(o.pos.cell()))
            .map(|o| worth(o))
            .sum();
        let own: i64 = self
            .script
            .own(w)
            .iter()
            .filter_map(|&id| w.obj(id))
            .filter(|o| army(o) && near(o.pos.cell()))
            .map(worth)
            .sum();
        own > 0 && enemy >= own
    }

    /// Whether the house pays for its economy first ([`Self::economy_first`]):
    /// it is short of its economy target or of a refinery, and an item of
    /// the harvester or refinery role runs on a line with less money in hand
    /// than its balance owed; or its harvesters wait for a lost refinery
    /// (`Script::refinery_lost`) that is not on its lines yet, selling out
    /// or not (M8 step B8).
    fn paying(&self, w: &World) -> bool {
        use glasnost_sim::econ::production::Hold;
        let h = self.house();
        let lost = self.script.refinery_lost(w);
        if lost.is_some_and(|t| !self.script.in_line(w, t)) {
            return true;
        }
        if self.field_refinery_short(w) {
            return true;
        }
        let short = (!self.script.selling() || lost.is_some())
            && (!self.script.economy_kept(w) || !self.script.has_refinery(w));
        let credits = w.houses[usize::from(h.0)].credits;
        short
            && w.econ.lines.lines.iter().any(|l| {
                l.house == h
                    && l.item.as_ref().is_some_and(|i| {
                        is_economy(w, i.t) && i.hold == Hold::None && !i.done() && credits < i.owed
                    })
            })
    }

    /// Whether a refinery the house builds while it has one (the refinery
    /// by a new field, `Script::field_refinery`) runs on its lines owing
    /// more than the money in hand, the house is safe ([`Self::safe`]),
    /// not selling out, and not a slave-miner house (S-0402 claim 14).
    /// The house then pays for its economy first ([`Self::paying`]).
    fn field_refinery_short(&self, w: &World) -> bool {
        use glasnost_sim::econ::production::Hold;
        let s = &self.script;
        if s.selling() || s.slave_miners(w) || !s.has_refinery(w) {
            return false;
        }
        let h = self.house();
        let credits = w.houses[usize::from(h.0)].credits;
        // design: a refinery by a new field that waits for money is paid
        // first, as a person pauses the superweapon and the army lines to
        // get his income back (M8 step B10). In the quick lane on main
        // e375cf6, 116 spells of over 3 000 ticks (669 000 ticks in 60
        // games) had a house broke with that refinery waiting: in 77 a
        // superweapon building on another line drew the money first (the
        // earlier line is paid first, S-0404 claim 1), in the rest the
        // army lines did. A test of the income over the last window
        // (`Script::income`) beside the credits was tried and never fired
        // in those spells: the house earned, and the earlier line took it
        // first. A house that is not safe keeps training, as the
        // superweapon saving does.
        let short = w.econ.lines.lines.iter().any(|l| {
            l.house == h
                && l.item.as_ref().is_some_and(|i| {
                    scripted::role_of(w, i.t) == Some(Role::Refinery)
                        && !scripted::keeps_slaves(w, i.t)
                        && i.hold == Hold::None
                        && !i.done()
                        && credits < i.owed
                })
        });
        short && self.safe(w)
    }

    /// Takes back the script's tech-climb Build order of this think
    /// ([`Script::climbed`]) for a building the brain orders instead.
    fn drop_climb(&self, out: &mut Vec<Order>) {
        // design: own AI (#8): only the script's own climb order goes; a held item's resume (`economy_first`) and the brain's other building orders stay
        if let Some(t) = self.script.climbed() {
            out.retain(|o| !matches!(o, Order::Build { what } if *what == t));
        }
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
            self.drop_climb(out);
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
        self.drop_climb(out);
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
            self.drop_climb(out);
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

    /// Builds a unit or defence able to hit the enemy drainers it sees in
    /// the air ([`crate::antiair::build_type`]), as a person answers a
    /// drain he cannot shoot at: not while selling out. A defence replaces
    /// a tech-climb building this think ordered, as a superweapon building
    /// does.
    fn build_anti_air(&mut self, w: &World, out: &mut Vec<Order>) {
        if self.script.selling() {
            return;
        }
        let Some(t) = crate::antiair::build_type(&self.script, w) else {
            return;
        };
        if t.kind == Kind::Building {
            if out.iter().any(|o| matches!(o, Order::Place { .. })) {
                return;
            }
            self.drop_climb(out);
        }
        out.push(Order::Build { what: t });
        self.special.count_anti_air_build();
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
            self.drop_climb(out);
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
        if let Some(t) = self.superweapon_next(w)
            && !self.script.busy(w, t)
        {
            self.drop_climb(out);
            out.push(Order::Build { what: t });
        }
    }

    /// The building [`Self::build_superweapon`] goes for next: the
    /// cheapest building the house can build and place now that gives a
    /// kind of superweapon it fires and has no building for yet, or one
    /// that clears the blocker of such a building on the tech tree; none
    /// once the house has a building of every kind it can climb to. A
    /// building of it on a line, done or not, still counts (its kind is
    /// the house's only once it stands).
    fn superweapon_next(&self, w: &World) -> Option<TypeRef> {
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
        giving
            .iter()
            .copied()
            // design: own AI (#60): the tech tree's answer kept for the think
            .filter(|&t| self.script.buildable(w, t))
            .filter(|&t| self.script.place_cell(w, t).is_some())
            .chain(
                giving
                    .iter()
                    .copied()
                    .filter(|&t| !self.script.buildable(w, t))
                    .filter_map(|t| self.script.unblocker(w, t, CLIMB_DEPTH)),
            )
            .min_by_key(|&t| (production::cost_field(w, t), t.index))
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
                Some(SwKind::Chrono) => self.teleport_cells(w, h, false),
                Some(SwKind::ChronoWarp) => self.teleport_cells(w, h, true),
                Some(SwKind::PsychicReveal) => crate::psychic::reveal_cell(&self.script, w),
                Some(SwKind::Genetic) => crate::mutator::mutate_cell(&self.script, w),
                Some(SwKind::Dominator) => {
                    crate::dominator::dominate_cell(&self.script, w, i, &spent)
                }
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

    /// The source a first-stage teleport record of the house holds picked
    /// and not yet sent, so that a second stage has something to send
    /// (S-0513 claim 34). A source the second stage has sent is not one: a
    /// person picks again (S-0512 claim 10, the unconsumed source selection).
    fn fresh_source(w: &World, h: HouseId) -> Option<CellXY> {
        use glasnost_sim::superweapons::{SwKind, charge};
        w.rules()
            .superweapons
            .types
            .iter()
            .enumerate()
            .filter(|(_, t)| t.kind == Some(SwKind::Chrono))
            // spec: S-0512 claim 10 (the second stage needs an unconsumed source selection; sending it consumes it)
            .find_map(|(i, _)| {
                charge::get(w, h, i as u32)
                    .filter(|r| r.source_fresh)
                    .and_then(|r| r.source)
            })
    }

    /// The teleport's stages as a person clicks them: the first stage picks
    /// a block of the house's vehicles ([`Brain::group_cell`]) while no
    /// fresh source is held, or picks again when the vehicles have left
    /// the picked block (S-0512 claim 10: the player may pick another
    /// source); the second stage sends a fresh source that still holds the
    /// vehicles. Returns the cell of the second stage with `second`, else
    /// of the first.
    fn teleport_cells(&self, w: &World, h: HouseId, second: bool) -> Option<CellXY> {
        let source = Self::fresh_source(w, h);
        // design: own AI (#2): a picked block the vehicles have left is not sent; the first stage picks the block they now stand in
        let holds = source.is_some_and(|s| {
            self.vehicle_block(w, false, TELEPORT_MIN, Some(s))
                .is_some()
        });
        match (second, holds) {
            (false, false) => self.group_cell(w),
            (true, true) => self.landing_cell(w),
            _ => None,
        }
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
    /// `fight`, a seen enemy must be within [`FIGHT`] cells. With `only`,
    /// that cell is the one block asked about.
    fn vehicle_block(
        &self,
        w: &World,
        fight: bool,
        min: usize,
        only: Option<CellXY>,
    ) -> Option<CellXY> {
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
        let cells: Vec<CellXY> = match only {
            Some(c) => vec![c],
            None => mine
                .iter()
                .filter(|&&(_, army)| army)
                .map(|&(c, _)| c)
                .collect(),
        };
        cells
            .into_iter()
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
        let friend = |h: Option<HouseId>| h.is_some_and(|h| h == house || me.allied_with(h));
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
        self.vehicle_block(w, false, TELEPORT_MIN, None)
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
            .filter(|o| o.owner.is_some_and(|h| h == house || me.allied_with(h)))
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

    /// Places a finished building of the house: an anti-air defence near
    /// the drainer in the air it answers (`antiair::place_goal`, M8 step
    /// B6), a refinery beside the field (`Script::place_refinery`, M8 step
    /// B8), any other in the spiral around the yard.
    fn place_done(&self, w: &World, out: &mut Vec<Order>) {
        let house = self.house();
        let done = w.econ.lines.lines.iter().find_map(|l| {
            let item = l.item.as_ref()?;
            (l.house == house && item.t.kind == Kind::Building && item.done_tick.is_some())
                .then_some(item.t)
        });
        // design: own AI (#7): the same cell a think tick would choose, toward the water too (a plant the base walk ordered landed in the spiral when it was done between thinks)
        let cell = done.and_then(|t| self.script.done_cell(w, t));
        if let Some(t) = done
            && let Some(cell) = cell
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
        yard_units(w, self.house())
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

    /// Expands the base toward a far resource field with a second
    /// construction yard (M7b AI round 3). Deploying a yard vehicle gives a
    /// construction yard wherever the deploy's cell test passes, with no
    /// adjacency to the base (S-0415 claim 3, S-0406 claim 4); a second
    /// yard makes every building and defence faster when the game-wide
    /// multiple-factory factor is above zero (S-0403 claim 6), keeps the
    /// base building when one yard falls, and puts the far field within
    /// the reach the house counts harvesters for. A person builds one when
    /// rich, with the economy kept, no enemy at the base and an explored
    /// field beyond the reach of every yard on his half of the map
    /// ([`Script::far_field`]); he drives it to the free cell nearest that
    /// field where the deploy cursor shows it fits and deploys it there.
    /// Before the round the Yuri side's yard vehicle was built only to
    /// replace a lost yard.
    fn expand<P: Probe>(&mut self, w: &World, out: &mut Vec<Order>, p: &mut P) {
        let own = self.script.own(w);
        let yards = own
            .iter()
            .filter(|&&id| w.obj(id).is_some_and(|o| is_yard(w, o)))
            .count();
        // With no yard, `rebuild_yard` and the plan's deploy act instead.
        if yards == 0 || self.script.selling() {
            return;
        }
        // design: own AI (#58): the far field (a scan of the whole map) is
        // looked for only when an idle yard vehicle or the tests below ask
        // for it; it reads only the world and the plan, so when it is
        // looked for does not change it.
        let far = std::cell::OnceCell::new();
        if p.on() {
            p.mark(Phase::Part("ai:expand:units"));
        }
        let units: Vec<ObjectId> = own
            .iter()
            .copied()
            .filter(|&id| {
                w.obj(id).is_some_and(|o| {
                    o.kind == Kind::Vehicle && deploys_into_yard(w, o.kind, o.type_index)
                })
            })
            .collect();
        for &id in &units {
            let Some(o) = w.obj(id) else { continue };
            let Some(b) = deploy::deploys_into(w, o.kind, o.type_index) else {
                continue;
            };
            if !idle(w, id) {
                continue;
            }
            let field: Option<CellXY> = *far.get_or_init(|| self.script.far_field(w));
            let here = o.pos.cell();
            let reach = i64::from(scripted::FIELD_REACH) * i64::from(scripted::FIELD_REACH);
            // With no far field left (another yard took it), the unit deploys
            // where it is, as a spare yard.
            let near = field.is_none_or(|f| dist2(here, f) <= reach);
            let fits = |c: CellXY| {
                w.cells.usable(c)
                    && glasnost_sim::econ::resource::resource_at(w, c).is_none()
                    && self.deploys_at(w, id, b, c)
            };
            if near && fits(here) {
                // spec: S-0415 claims 1 to 3 (the deploy order)
                out.push(Order::Deploy { actor: id });
                continue;
            }
            let goes = reach_test(w, id);
            // design: own AI (#5): a unit still driving to a spot that still takes it is left to arrive (re-planning in its last cells cleared its path every think)
            if matches!(o.dest, Some(Target::Cell(c)) if goes(c) && fits(c)) {
                continue;
            }
            let around = |centre: CellXY| {
                (1..=SPIRAL_RINGS).find_map(|ring| {
                    (-ring..=ring)
                        .flat_map(|dy| (-ring..=ring).map(move |dx| (dx, dy)))
                        .filter(|&(dx, dy)| dx.abs() == ring || dy.abs() == ring)
                        .map(|(dx, dy)| CellXY::new(centre.x + dx, centre.y + dy))
                        .filter(|&c| c != here)
                        // design: own AI (#5) and S-0212 claim 4: only a cell the unit can drive to (in its reach region); one across water was picked again on every think and never reached
                        .find(|&c| goes(c) && fits(c))
                })
            };
            let mut spot = around(field.unwrap_or(here));
            if spot.is_none() && field.is_some() {
                // design: own AI (#5): with no cell it can reach by the far field (an island's), the unit deploys as a spare yard where it stands, or near it
                if fits(here) {
                    // spec: S-0415 claims 1 to 3 (the deploy order)
                    out.push(Order::Deploy { actor: id });
                    continue;
                }
                spot = around(here);
            }
            if let Some(c) = spot {
                out.push(Order::Behaviour {
                    actor: id,
                    behaviour: Behaviour::Move,
                    attack_move: false,
                    target: None,
                    dest: Some(Target::Cell(c)),
                });
                self.special.count_expansion_move();
            }
        }
        if p.on() {
            p.mark(Phase::Part("ai:expand:build"));
        }
        let types = yard_types(w);
        let Some(&cheapest) = types.first() else {
            return;
        };
        let t = self.yard_units(w).first().copied();
        let h = self.house();
        let lined = w.econ.lines.lines.iter().any(|l| {
            l.house == h
                && types
                    .iter()
                    .any(|&x| l.item.as_ref().is_some_and(|i| i.t == x) || l.queue.contains(&x))
        });
        let credits = i64::from(w.houses[usize::from(h.0)].credits);
        let cost = i64::from(production::cost_field(w, t.unwrap_or(cheapest)));
        if p.on() {
            p.mark(Phase::Part("ai:expand:tests"));
        }
        // A person keeps at most one yard beyond the first: a third adds
        // little speed for its price and leaves the army short.
        if yards + units.len() + usize::from(lined) >= EXPANSION_YARDS
            || !self.rich(w, credits, cost)
            || far.get_or_init(|| self.script.far_field(w)).is_none()
            || !self.script.harvesters_kept(w)
            || !self.intruders(w).is_empty()
            || self.outnumbered(w)
            || self.losing(w)
        {
            return;
        }
        if let Some(t) = t {
            if !out
                .iter()
                .any(|o| matches!(o, Order::Build { what } if *what == t))
            {
                out.push(Order::Build { what: t });
                self.special.count_expansion_build();
            }
            return;
        }
        // M8 step B1: the yard vehicle waits on the tech tree. A person who
        // wants to expand builds what clears its blocker (S-0407 claim 8),
        // as the tech climb does for an army type (`Script::unblocker`); in
        // the stock lane no other step built it, and the yard vehicle stayed
        // unbuildable for most of each game. Like a superweapon building it
        // replaces a tech-climb building this think ordered, but no other
        // building order of this think (the plan's, a superweapon building,
        // the brain's other extras).
        let climbing = self.script.climb(w);
        if out.iter().any(|o| {
            matches!(o, Order::Place { .. })
                || matches!(o, Order::Build { what } if what.kind == Kind::Building && Some(*what) != climbing)
        }) {
            return;
        }
        let Some(b) = types
            .iter()
            .find_map(|&x| self.script.unblocker(w, x, CLIMB_DEPTH))
        else {
            return;
        };
        if self.script.busy(w, b) {
            return;
        }
        out.retain(|o| !matches!(o, Order::Build { what } if Some(*what) == climbing));
        out.push(Order::Build { what: b });
        self.special.count_expansion_unblock();
    }

    /// Whether the house is rich enough to expand (M8 step B1): credits in
    /// hand of [`EXPANSION_RICH`] times the yard vehicle's cost, or its
    /// economy whole (every harvester it wants, `Script::economy_kept`) and
    /// no superweapon building on its lines: a person finishes the
    /// superweapon first, then expands. The stock lane showed the first test
    /// alone never passing once the yard vehicle could be built: every line
    /// drew the credits down as they came in (the probe read them in the
    /// low hundreds or below in most samples), so a house with nothing more
    /// urgent to pay for buys the yard vehicle step by step like any item
    /// (S-0404 claim 4: no money is needed to start one). M8 step B5: the
    /// money in hand counts beyond the balance the superweapon building
    /// owes, and the second test also wants an income of
    /// [`EXPANSION_EARNING`] times the cost per income window
    /// (`Script::income`) and no superweapon building left to go for
    /// ([`Self::superweapon_next`]), not only none on the lines: in the
    /// think in which the next one is ordered it is not on them yet
    /// (orders are applied after the think), and the yard vehicle, and
    /// before it the building that clears its blocker, went in front of the
    /// superweapons (in one game every later superweapon of the house came
    /// 1 000 ticks or more after round 3's).
    fn rich(&self, w: &World, credits: i64, cost: i64) -> bool {
        let owed = self.superweapon_owed(w).map_or(0, i64::from);
        rich_by(
            credits - owed,
            cost,
            self.script.economy_kept(w),
            self.script.income(),
            || self.superweapon_wanted(w),
        )
    }

    /// Whether a superweapon building is on the house's lines or left to
    /// go for ([`Self::superweapon_next`]).
    fn superweapon_wanted(&self, w: &World) -> bool {
        self.superweapon_owed(w).is_some() || self.superweapon_next(w).is_some()
    }

    /// Cancels a yard vehicle bought to expand (the house has a yard) when
    /// the house is broke for it, with the cancel-all order a person gives
    /// on the sidebar (S-0404 claim 6: at an unchanged price the refund is
    /// everything paid): it earns less than the yard vehicle's cost divided
    /// by [`EXPANSION_BROKE`] per income window (`Script::income`), and its
    /// credits do not cover what the yard vehicle and a superweapon
    /// building on its lines still owe. A yard vehicle bought with no yard
    /// (`Self::rebuild_yard`) runs on. Found in M8 step B5: a house that
    /// kept every harvester it wanted on a nearly mined-out map earned 275
    /// to 600 credits per 1 500 ticks, and its yard vehicle stood on the
    /// line from tick 43 500 to the end of the game (57 798), never past
    /// step 23 of 54, the army vehicles queued behind it (an order to a
    /// busy line joins its queue, S-0404 claim 3).
    fn drop_expansion(&mut self, w: &World, out: &mut Vec<Order>) {
        let Some(income) = self.script.income() else {
            return;
        };
        let h = self.house();
        let has_yard = self
            .script
            .own(w)
            .iter()
            .any(|&id| w.obj(id).is_some_and(|o| is_yard(w, o)));
        if !has_yard {
            return;
        }
        let types = yard_types(w);
        let mut lined: Vec<TypeRef> = Vec::new();
        let mut owed: i64 = 0;
        for l in w.econ.lines.lines.iter().filter(|l| l.house == h) {
            if let Some(i) = l.item.as_ref().filter(|i| types.contains(&i.t)) {
                owed += i64::from(i.owed);
                if !lined.contains(&i.t) {
                    lined.push(i.t);
                }
            }
            for &q in l.queue.iter().filter(|q| types.contains(q)) {
                if !lined.contains(&q) {
                    lined.push(q);
                }
            }
        }
        let Some(&first) = lined.first() else {
            return;
        };
        let credits = i64::from(w.houses[usize::from(h.0)].credits);
        let superweapon = self.superweapon_owed(w).map_or(0, i64::from);
        let cost = i64::from(production::cost_field(w, first));
        if income * EXPANSION_BROKE >= cost || credits >= superweapon + owed {
            return;
        }
        for t in lined {
            out.retain(|o| !matches!(o, Order::Build { what } if *what == t));
            // spec: S-0404 claim 6 (the cancel-all order and its refund)
            out.push(Order::CancelAll { what: t });
            self.special.count_expansion_cancel();
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
            let goes = reach_test(w, id);
            // design: own AI (#5): a unit still driving to a spot that still takes it is left to arrive (re-planning in its last cells cleared its path every tick)
            if matches!(o.dest, Some(Target::Cell(c)) if goes(c) && w.cells.usable(c) && self.deploys_at(w, id, b, c))
            {
                continue;
            }
            let spot = (1..=SPIRAL_RINGS).find_map(|ring| {
                (-ring..=ring)
                    .flat_map(|dy| (-ring..=ring).map(move |dx| (dx, dy)))
                    .filter(|&(dx, dy)| dx.abs() == ring || dy.abs() == ring)
                    .map(|(dx, dy)| CellXY::new(here.x + dx, here.y + dy))
                    // design: own AI (#5) and S-0212 claim 4: only a cell the unit can drive to (in its reach region); one across water was picked again on every tick and never reached
                    .find(|&c| w.cells.usable(c) && goes(c) && self.deploys_at(w, id, b, c))
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
            // design: own AI (#60): the answer kept for the think
            .filter(|&t| self.script.buildable(w, t))
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
    ///   to the goal, else the cheapest; when its line is making something
    ///   else, the ferry joins the line's queue (S-0404 claim 3), as a
    ///   person queues it on the sidebar rather than waiting for the line
    ///   to stand idle (M8 step B4: a house short of money kept its
    ///   vehicle line busy at every think and never had a ferry);
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
        let ferries = self.ferries(w);
        // design: own AI (#9): the goal by the movement zone of the infantry and of the passengers (`Script::base_goal_for`): a naval yard on the water made every infantry unit seem to need a ferry, and gave a full ferry no drop cell
        let zone_of = |id: ObjectId| {
            w.obj(id)
                .and_then(|o| w.rules().unit(o.kind, o.type_index))
                .map(|u| u.zone)
        };
        let aboard_all: Vec<ObjectId> = ferries
            .iter()
            .flat_map(|&f| transport::passengers(w, f))
            .collect();
        let mut goals: std::collections::BTreeMap<u8, CellXY> = std::collections::BTreeMap::new();
        for &p in infantry.iter().chain(&aboard_all) {
            if let Some(z) = zone_of(p)
                && !goals.contains_key(&(z as u8))
                && let Some(c) = self.script.base_goal_for(w, Some(z))
            {
                goals.insert(z as u8, c);
            }
        }
        let goal_for = |z: Option<glasnost_rules::sim::MovementZone>| {
            z.and_then(|z| goals.get(&(z as u8)).copied())
                .unwrap_or(goal)
        };
        let crossing = infantry
            .iter()
            .any(|&p| Self::crossing(w, p, goal_for(zone_of(p))));
        if !ferries.iter().any(|&f| Self::ferry_suits(w, f, crossing))
            && !infantry.is_empty()
            && !self.script.selling()
            && self.script.harvesters_kept(w)
            && let Some(t) = self.ferry_type(w, crossing)
            && !self.script.in_line(w, t)
            && self.script.queue_room(w, t)
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
                    .and_then(|&p| zone_of(p));
                self.land_cell(w, f, goal_for(zone), (DROP_NEAR, DROP_FAR), at, zone)
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
                } else if targeting::cursor(w, house, &[f], ClickOn::Object(f), mods)
                    == Action::Deploy
                {
                    // design: own AI (#9): with no drop cell near the goal a full ferry at home lets its passengers out (they stayed aboard for as long as the goal stood), as it does away from home
                    out.extend(targeting::click(w, house, &[f], ClickOn::Object(f), mods).orders);
                }
                continue;
            }
            // Only a ferry that suits the run takes infantry aboard.
            if !Self::ferry_suits(w, f, crossing) {
                continue;
            }
            // design: own AI (#9): and only while it has a drop cell for them near the goal, so that it does not take them aboard only to let them out
            let has_drop = free.first().is_none_or(|&p| {
                let zone = zone_of(p);
                self.land_cell(w, f, goal_for(zone), (DROP_NEAR, DROP_FAR), at, zone)
                    .is_some()
            });
            if !has_drop {
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

    /// Whether the house's army units are fewer than the enemy units it
    /// sees (buildings and workers not counted on either side).
    fn outnumbered(&self, w: &World) -> bool {
        let unit = |o: &glasnost_sim::world::Object| {
            o.kind != Kind::Building
                && scripted::role_of(
                    w,
                    TypeRef {
                        kind: o.kind,
                        index: o.type_index,
                    },
                ) == Some(Role::Army)
        };
        let own = self
            .script
            .own(w)
            .iter()
            .filter_map(|&id| w.obj(id))
            .filter(|o| unit(o))
            .count();
        let seen = self
            .script
            .seen_enemies(w)
            .iter()
            .filter_map(|&(id, _)| w.obj(id))
            .filter(|o| unit(o))
            .count();
        own < seen
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
        let house = self.house();
        for id in self.script.of_role(w, Role::Army) {
            let Some(o) = w.obj(id) else { continue };
            let at = o.pos.cell();
            let near_home = dist2(at, home) <= RECALL * RECALL;
            // A threat: an intruder, a raider, or an enemy that is shooting
            // at one of the house's own (a raider the unit was sent at turns
            // on the unit and is no raider any more; the fight goes on).
            let threat = |t: Option<Target>| {
                matches!(t, Some(Target::Object(t))
                if intruders.iter().chain(&raiders).any(|&(e, _)| e == t)
                    || w.obj(t).is_some_and(|x| {
                        x.owner.is_some_and(|h| h != house)
                            && matches!(x.target, Some(Target::Object(v))
                                if w.obj(v).is_some_and(|v| v.owner == Some(house)))
                    }))
            };
            // A unit shooting at a threat, or on its way to one under an
            // attack order (the order's destination, S-0213 table T3 (f),
            // stays when the unit has no target in the meantime), is about
            // it already: a person does not click it again on every cell.
            let attacking = matches!(o.beh.current, Some(Behaviour::Attack))
                || matches!(o.beh.queued, Some(Behaviour::Attack));
            let busy = threat(o.target) || (attacking && threat(o.dest));
            // A unit sent from where it stands at a raider that still raids
            // waits for that order to be judged (`memory`): it is not sent
            // at another raider each tick in the meantime.
            let sent = threat(self.special.memory.sent_here(w, id));
            // A unit on its way into a recycling building (S-0603) is left to go;
            // so is a vehicle on its way into a tank bunker, and a bunkered one
            // is left to the bunker part (`bunker`), which lets it out (S-0606).
            let bound = o.beh.current == Some(Behaviour::Eaten)
                || o.beh.queued == Some(Behaviour::Eaten)
                || (o.beh.current == Some(Behaviour::Enter)
                    && glasnost_sim::structures::bunker::entering_bunker(w, id))
                || glasnost_sim::structures::bunker::bunkered(w, id);
            // spec: S-0214 claim 15 and S-0610 claim 6 (a berserk unit takes no orders: the brain gives it none, and counts none)
            let berserk = glasnost_sim::psychic::berserk(w, id);
            // design: an aircraft with no ammunition left rearms first (the think's filter, M8 step A4 round 8)
            let spent = o.kind == Kind::Aircraft && !glasnost_sim::air::ammo_left(w, id);
            if busy || sent || bound || berserk || spent {
                continue;
            }
            // A unit already sent at the raider from where it stands, or
            // one that did not take that order (`memory`), is not sent
            // again: a person does not click the same every second.
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
                        })
                        .min_by_key(|&&(e, c)| (dist2(c, at), e))
                })
                .flatten();
            // design: own AI (#4): an intruder the unit cannot go for (out of its reach region and of its weapon's range from there, S-0212 claim 4) is not attacked, and one it was sent at is judged like a raider (`memory`): an attack the sim drops for want of an approach (S-0307 claim 17) was given again on every tick
            let region = self.script.reach_region(w, id);
            let pick = raider.copied().or_else(|| {
                near_home
                    .then(|| {
                        intruders
                            .iter()
                            .filter(|&&(e, c)| {
                                can_attack(w, id, e)
                                    && self.script.reaches(w, id, region, e, c)
                                    && memory.ok(w, id, Target::Object(e))
                                    && !memory.pending(w, id, Target::Object(e))
                            })
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
                }
                self.special.memory.note(w, std::slice::from_ref(&order));
                out.push(order);
            }
        }
    }
}

/// Whether unit `id` can drive to a cell: it lies in the reach region the
/// unit stands in for its movement zone (S-0212 claim 4). Every cell passes
/// for a unit with no zone or standing in no region.
fn reach_test(w: &World, id: ObjectId) -> impl Fn(CellXY) -> bool + '_ {
    let at = w.obj(id).and_then(|o| {
        let zone = w.rules().unit(o.kind, o.type_index)?.zone;
        match regions::reach_in(w, o.pos.cell(), zone) {
            regions::Reach::Region(r) => Some((zone, r)),
            _ => None,
        }
    });
    move |c| at.is_none_or(|(zone, r)| regions::reach_in(w, c, zone) == regions::Reach::Region(r))
}

/// Whether a line item of type `t` belongs to the army that
/// `Brain::economy_first` holds while the house saves: a unit of the army
/// role, and a yard vehicle bought to expand (the house has a yard); one
/// bought with no yard is the house's way back and runs on.
fn army_item(w: &World, t: TypeRef, has_yard: bool) -> bool {
    (t.kind != Kind::Building && scripted::role_of(w, t) == Some(Role::Army))
        || (has_yard && deploys_into_yard(w, t.kind, t.index))
}

/// Whether such an item does not fight: an engineer, or a yard vehicle
/// bought to expand (M8 steps B5 and B7).
fn non_fighter(w: &World, t: TypeRef, has_yard: bool) -> bool {
    army_item(w, t, has_yard)
        && (deploys_into_yard(w, t.kind, t.index)
            || w.rules().unit(t.kind, t.index).is_none_or(|u| u.engineer))
}

/// The vehicle types house `h` can build now that deploy into a
/// construction yard, cheapest first.
pub fn yard_units(w: &World, h: HouseId) -> Vec<TypeRef> {
    yard_types(w)
        .into_iter()
        .filter(|&t| tech::buildable(w, h, t))
        .collect()
}

/// The wealth test of [`Brain::rich`], shared with the probe: `spare`
/// credits (in hand, less what the superweapon building owes) of
/// [`EXPANSION_RICH`] times the yard vehicle's `cost`, or the economy kept,
/// an `income` per window of [`EXPANSION_EARNING`] times the cost, and no
/// superweapon building `wanted` (read last: it searches the tech tree).
fn rich_by(
    spare: i64,
    cost: i64,
    economy_kept: bool,
    income: Option<i64>,
    wanted: impl FnOnce() -> bool,
) -> bool {
    spare >= EXPANSION_RICH * cost
        || (economy_kept && income.is_some_and(|i| i >= EXPANSION_EARNING * cost) && !wanted())
}

/// Every vehicle type that deploys into a construction yard, cheapest
/// first.
fn yard_types(w: &World) -> Vec<TypeRef> {
    let r = w.rules();
    let mut v: Vec<TypeRef> = all_types(w)
        .into_iter()
        .filter(|t| t.kind == Kind::Vehicle)
        .filter(|t| {
            deploy::deploys_into(w, t.kind, t.index).is_some_and(|b| {
                r.buildings
                    .get(b as usize)
                    .is_some_and(|b| b.factory == Some(glasnost_sim::rules::FactoryKind::Building))
            })
        })
        .collect();
    v.sort_by_key(|&t| (production::cost_field(w, t), t.index));
    v
}

/// Whether type `t` has the harvester or refinery role: what the house pays
/// for first ([`Brain::economy_first`]).
fn is_economy(w: &World, t: TypeRef) -> bool {
    matches!(
        scripted::role_of(w, t),
        Some(Role::Harvester | Role::Refinery)
    )
}

/// What the expansion step ([`Brain::expand`]) sees for one house, counts
/// only (the lane's expansion probe, `GLASNOST_AI_EXPROBE`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExpansionProbe {
    /// Construction yards the house owns.
    pub yards: usize,
    /// Its vehicles that deploy into a construction yard.
    pub units: usize,
    /// Of those, the idle ones.
    pub idle_units: usize,
    /// Whether a yard vehicle runs or waits on one of its lines.
    pub lined: bool,
    /// Whether it can build a yard vehicle now.
    pub buildable: bool,
    /// Vehicle types that deploy into a construction yard, in the rules.
    pub yard_types: usize,
    /// The fewest unmet prerequisites of one of them, when no other
    /// condition refuses it (S-0407 claim 8).
    pub unmet: Option<usize>,
    /// The building that would clear a yard vehicle's blocker, that the
    /// house can build and place now (`Script::unblocker`).
    pub unblock: Option<u32>,
    /// The cheapest yard vehicle's cost (buildable now, or else of any).
    pub cost: i64,
    /// Its credits.
    pub credits: i64,
    /// Explored resource cells.
    pub explored: usize,
    /// Of those, the cells of fields it works.
    pub worked: usize,
    /// The cells of unworked fields on its own half of the map.
    pub own_half: usize,
    /// Whether a far field was found.
    pub field: bool,
    /// Whether it keeps the plan's harvesters.
    pub harvesters_kept: bool,
    /// Seen enemies near its buildings.
    pub intruders: usize,
    /// Those by kind: army units, workers (harvesters, slave miners,
    /// slaves), buildings, other units (M8 step B7).
    pub intruders_by: std::collections::BTreeMap<&'static str, usize>,
    /// Whether it sells out.
    pub selling: bool,
    /// Whether it pays for its economy first.
    pub paying: bool,
    /// Whether it keeps every harvester it wants.
    pub economy_kept: bool,
    /// Whether its army is outnumbered by the enemy units it sees.
    pub outnumbered: bool,
    /// Whether it is losing a fight.
    pub losing: bool,
    /// Whether it is losing one near home (M8 step B7).
    pub losing_at_home: bool,
    /// Whether it saves for a superweapon building.
    pub saving: bool,
    /// Whether a building that gives a superweapon it fires runs on its
    /// lines.
    pub superweapon_building: bool,
    /// The balance that building owes.
    pub superweapon_owed: i64,
    /// Whether such a building is on its lines or left to go for
    /// (M8 step B5).
    pub superweapon_wanted: bool,
    /// Its income over the last income window (`Script::income`).
    pub income: Option<i64>,
}

impl ExpansionProbe {
    /// The conditions of the yard vehicle's build that fail, by name.
    pub fn blocked_by(&self) -> Vec<&'static str> {
        let mut v = Vec::new();
        if self.yards == 0 {
            v.push("no yard");
        }
        if self.selling {
            v.push("selling");
        }
        if self.paying {
            v.push("paying");
        }
        if !self.buildable {
            v.push(match (self.unmet, self.unblock) {
                (_, Some(_)) => "not buildable, unblockable now",
                (Some(_), None) => "not buildable, prerequisite",
                (None, None) => "not buildable, refused",
            });
        }
        if self.yards + self.units + usize::from(self.lined) >= EXPANSION_YARDS {
            v.push("yards enough");
        }
        if !rich_by(
            self.credits - self.superweapon_owed,
            self.cost,
            self.economy_kept,
            self.income,
            || self.superweapon_wanted,
        ) {
            v.push("not rich");
        }
        if self.outnumbered {
            v.push("outnumbered");
        }
        if self.losing {
            v.push("losing");
        }
        if !self.field {
            v.push("no field");
        }
        if !self.harvesters_kept {
            v.push("harvesters short");
        }
        if self.intruders > 0 {
            v.push("intruders");
        }
        v
    }
}

/// The economy item a house waits for ([`BrokeProbe`], M8 step B10).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EconomyWait {
    /// A lost refinery its harvesters wait for (`Script::refinery_lost`).
    LostRefinery,
    /// A refinery by a new field: wanted (`Script::field_refinery`) or on
    /// its lines while it has a refinery.
    FieldRefinery,
    /// A harvester on its lines while it is short of its economy target.
    Harvester,
}

/// What a broke house waits behind, one sample (the lane's economy probe,
/// `GLASNOST_AI_ECOPROBE`, M8 step B10); counts only.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BrokeProbe {
    /// The economy item it waits for.
    pub wait: Option<EconomyWait>,
    /// Whether that item is on its lines.
    pub lined: bool,
    /// Whether its credits are below what the item still owes (its whole
    /// price before it is on a line).
    pub broke: bool,
    /// Whether a power plant runs on its lines.
    pub plant_lined: bool,
    /// Whether a building that gives a superweapon it fires runs, not
    /// held, on its lines.
    pub superweapon_lined: bool,
    /// Whether it saves for that building (holds its army).
    pub saving: bool,
    /// Whether it pays for its economy first (`Brain::paying`).
    pub paying: bool,
    /// Whether it sells out.
    pub selling: bool,
    /// Its income over the last window (`Script::income`).
    pub income: Option<i64>,
    /// Its credits.
    pub credits: i64,
    /// Whether it owns a construction yard and no other building.
    pub yard_only: bool,
    /// Whether no item of it is on any line.
    pub idle: bool,
    /// Its harvesters on the map.
    pub harvesters: usize,
    /// Its army units on the map.
    pub army: usize,
    /// The cheapest refinery price it could build now (none: 0).
    pub refinery_price: i64,
    /// The cheapest power plant price it could build now (none: 0).
    pub plant_price: i64,
    /// The role the plan builds next.
    pub wanted: Option<Role>,
}

impl Brain {
    /// What a broke house waits behind now ([`BrokeProbe`], M8 step B10).
    pub fn broke_probe(&self, w: &World) -> BrokeProbe {
        let h = self.house();
        let s = &self.script;
        let credits = w.houses[usize::from(h.0)].credits;
        let items: Vec<&production::Item> = w
            .econ
            .lines
            .lines
            .iter()
            .filter(|l| l.house == h)
            .filter_map(|l| l.item.as_ref())
            .collect();
        let refinery_item = items.iter().find(|i| {
            scripted::role_of(w, i.t) == Some(Role::Refinery) && !scripted::keeps_slaves(w, i.t)
        });
        let harvester_item = items
            .iter()
            .find(|i| scripted::role_of(w, i.t) == Some(Role::Harvester));
        let lost = s.refinery_lost(w);
        let (wait, item) = if lost.is_some() {
            (Some(EconomyWait::LostRefinery), refinery_item)
        } else if s.has_refinery(w) && (s.wants_field_refinery(w) || refinery_item.is_some()) {
            (Some(EconomyWait::FieldRefinery), refinery_item)
        } else if !s.economy_kept(w) && harvester_item.is_some() {
            (Some(EconomyWait::Harvester), harvester_item)
        } else {
            (None, None)
        };
        let price = |role: Role| {
            s.candidates(w, role)
                .into_iter()
                .filter(|&t| !scripted::keeps_slaves(w, t))
                .map(|t| i64::from(production::price(w, h, t)))
                .min()
                .unwrap_or(0)
        };
        let refinery_price = price(Role::Refinery);
        let need = item.map_or_else(
            || match wait {
                Some(EconomyWait::Harvester) | None => 0,
                _ => lost.map_or(refinery_price, |t| i64::from(production::price(w, h, t))),
            },
            |i| i64::from(i.owed),
        );
        let own = s.own(w);
        let buildings: Vec<&glasnost_sim::world::Object> = own
            .iter()
            .filter_map(|&id| w.obj(id))
            .filter(|o| o.kind == Kind::Building)
            .collect();
        BrokeProbe {
            wait,
            lined: item.is_some(),
            broke: wait.is_some() && i64::from(credits) < need,
            plant_lined: items
                .iter()
                .any(|i| scripted::role_of(w, i.t) == Some(Role::Power) && !i.done()),
            superweapon_lined: self.superweapon_owed(w).is_some(),
            saving: self.saving(w),
            paying: self.paying(w),
            selling: s.selling(),
            income: s.income(),
            credits: i64::from(credits),
            yard_only: !buildings.is_empty() && buildings.iter().all(|o| is_yard(w, o)),
            idle: items.is_empty(),
            harvesters: s.of_role(w, Role::Harvester).len(),
            army: s.of_role(w, Role::Army).len(),
            refinery_price,
            plant_price: price(Role::Power),
            wanted: s.wanted_role(w),
        }
    }

    /// What the anti-air step sees now ([`crate::antiair::Probe`]).
    pub fn antiair_probe(&self, w: &World) -> crate::antiair::Probe {
        crate::antiair::probe(&self.script, w)
    }

    /// Whether the plan builds the refinery by a new field next
    /// (`Script::field_refinery`; the lane's economy probe counts these
    /// spells, M8 step B9).
    pub fn wants_field_refinery(&self, w: &World) -> bool {
        self.script.wants_field_refinery(w)
    }

    /// What the plan's economy sees now, one line, counts and cells only
    /// (the lane's economy probe, `GLASNOST_AI_ECOPROBE`, M8 step B8): the
    /// house's refineries with the explored resource cells within
    /// `scripted::FIELD_REACH` of each and the distance to the nearest;
    /// each harvester's cell, behaviour, phase, load and distances to the
    /// nearest refinery and the nearest explored resource cell; the role
    /// the plan builds next, the refinery types it can build and place, the
    /// items on its lines (line kind, role, progress, hold), and the
    /// economy-first and selling states.
    pub fn economy_probe(&self, w: &World) -> String {
        use glasnost_sim::econ::{resource, shroud};
        use std::fmt::Write as _;
        let h = self.house();
        let s = &self.script;
        let cells = |a: CellXY, b: CellXY| dist2(a, b).isqrt();
        let ore: Vec<CellXY> = w
            .cells
            .grid
            .iter()
            .map(|(c, _)| c)
            .filter(|&c| {
                w.cells.usable(c)
                    && resource::resource_at(w, c).is_some()
                    && shroud::explored(w, h, c)
            })
            .collect();
        let nearest_ore = |c: CellXY| ore.iter().map(|&o| cells(c, o)).min();
        let reach = i64::from(scripted::FIELD_REACH);
        let refineries: Vec<CellXY> = s
            .of_role(w, Role::Refinery)
            .iter()
            .filter_map(|&id| w.obj(id))
            .map(|o| o.pos.cell())
            .collect();
        let mut out = String::new();
        let _ = write!(
            out,
            "ecoprobe tick {} house {}: credits {} yards {} selling {} paying {} no-field {} explored ore {} refineries {}",
            w.tick,
            h.0,
            w.houses[usize::from(h.0)].credits,
            usize::from(self.has_yard(w)),
            s.selling(),
            self.paying(w),
            w.houses[usize::from(h.0)].econ.harvest.no_field,
            ore.len(),
            refineries.len(),
        );
        for &r in &refineries {
            let near = ore.iter().filter(|&&o| cells(r, o) <= reach).count();
            let _ = write!(
                out,
                " [{},{} ore {near} nearest {:?}]",
                r.x,
                r.y,
                nearest_ore(r)
            );
        }
        let harvesters = s.of_role(w, Role::Harvester);
        let _ = write!(
            out,
            "; harvesters {} (target {}, wanted {})",
            harvesters.len(),
            s.economy_target(w),
            s.harvesters_wanted(w)
        );
        for &id in &harvesters {
            let Some(o) = w.obj(id) else { continue };
            let c = o.pos.cell();
            let (phase, load) = o
                .econ
                .harvest
                .as_ref()
                .map_or((None, 0), |x| (Some(x.phase), x.total()));
            let _ = write!(
                out,
                " [{} {},{} {:?} {:?} load {load} refinery {:?} ore {:?}]",
                id.0,
                c.x,
                c.y,
                o.beh.current,
                phase,
                refineries.iter().map(|&r| cells(c, r)).min(),
                nearest_ore(c),
            );
        }
        let types = s.candidates(w, Role::Refinery);
        let placeable = types
            .iter()
            .filter(|&&t| s.place_cell(w, t).is_some())
            .count();
        let lined: Vec<String> =
            w.econ
                .lines
                .lines
                .iter()
                .filter(|l| l.house == h)
                .flat_map(|l| {
                    let item = l.item.iter().map(|i| {
                        format!(
                            "{:?}: {:?} {}/54 owed {} hold {:?}{}",
                            l.kind,
                            scripted::role_of(w, i.t),
                            i.progress,
                            i.owed,
                            i.hold,
                            if i.done() { " done" } else { "" }
                        )
                    });
                    let queued = l.queue.iter().map(move |&t| {
                        format!("{:?}: {:?} queued", l.kind, scripted::role_of(w, t))
                    });
                    item.chain(queued).collect::<Vec<_>>()
                })
                .collect();
        let _ = write!(
            out,
            "; plan wants {:?}; refinery types {} placeable {}; line items {lined:?}",
            s.wanted_role(w),
            types.len(),
            placeable,
        );
        out
    }

    /// What the expansion step sees now ([`ExpansionProbe`]).
    pub fn expansion_probe(&self, w: &World) -> ExpansionProbe {
        let h = self.house();
        let own = self.script.own(w);
        let units: Vec<ObjectId> = own
            .iter()
            .copied()
            .filter(|&id| {
                w.obj(id).is_some_and(|o| {
                    o.kind == Kind::Vehicle && deploys_into_yard(w, o.kind, o.type_index)
                })
            })
            .collect();
        let t = self.yard_units(w).first().copied();
        let types = yard_types(w);
        let fields = self.script.fields(w);
        ExpansionProbe {
            yards: own
                .iter()
                .filter(|&&id| w.obj(id).is_some_and(|o| is_yard(w, o)))
                .count(),
            units: units.len(),
            idle_units: units.iter().filter(|&&id| idle(w, id)).count(),
            lined: t.is_some_and(|t| {
                w.econ.lines.lines.iter().any(|l| {
                    l.house == h
                        && (l.item.as_ref().is_some_and(|i| i.t == t) || l.queue.contains(&t))
                })
            }),
            buildable: t.is_some(),
            yard_types: types.len(),
            unmet: types
                .iter()
                .filter_map(|&x| tech::unmet_prerequisites(w, h, x).map(|v| v.len()))
                .min(),
            unblock: types
                .iter()
                .find_map(|&x| self.script.unblocker(w, x, CLIMB_DEPTH))
                .map(|b| b.index),
            cost: t
                .or(types.first().copied())
                .map_or(0, |t| i64::from(production::cost_field(w, t))),
            credits: i64::from(w.houses[usize::from(h.0)].credits),
            explored: fields.explored,
            worked: fields.worked,
            own_half: fields.own_half,
            field: fields.far.is_some(),
            harvesters_kept: self.script.harvesters_kept(w),
            intruders: self.intruders(w).len(),
            intruders_by: self.intruders(w).iter().fold(
                std::collections::BTreeMap::new(),
                |mut m, &(id, _)| {
                    let kind = w.obj(id).map_or("other", |o| {
                        let t = tech::type_of(o);
                        if o.kind == Kind::Building {
                            "building"
                        } else if o.econ.slave.is_some()
                            || o.econ.harvest.is_some()
                            || scripted::role_of(w, t) == Some(Role::Harvester)
                        {
                            "worker"
                        } else if scripted::role_of(w, t) == Some(Role::Army) {
                            "army"
                        } else {
                            "other"
                        }
                    });
                    *m.entry(kind).or_default() += 1;
                    m
                },
            ),
            selling: self.script.selling(),
            paying: self.paying(w),
            economy_kept: self.script.economy_kept(w),
            outnumbered: self.outnumbered(w),
            losing: self.losing(w),
            losing_at_home: self.losing_at_home(w),
            saving: self.saving(w),
            superweapon_building: self.superweapon_owed(w).is_some(),
            superweapon_owed: self.superweapon_owed(w).map_or(0, i64::from),
            superweapon_wanted: self.superweapon_wanted(w),
            income: self.script.income(),
        }
    }
}
