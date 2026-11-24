//! The siege script (#449, registry id 2): our own second AI script,
//! beside the default. Nothing here is game behaviour; every number is
//! tuning of our own opponent (`design: own AI (#449)`).
//!
//! **What it does.** The default script's brain ([`Brain`]) runs the base:
//! the build plan, the harvesters, the defence of the base and the special
//! abilities. Its plan here holds its waves ([`HELD`]): the brain never
//! sends one. At a level without superweapons ([`Tuning::superweapons`])
//! the brain's orders for superweapon buildings are dropped and its
//! tech-climb building ordered instead ([`Siege::no_superweapons`]), so the
//! money goes to the army. The army is the siege's own ([`Army`]): it
//! gathers at home until it is large enough and, at hard, worth more than
//! the enemy army the house remembers having seen ([`Tuning::edge`]); then
//! it goes out as one strike, attack-moving to the enemy's seen harvesters
//! and refineries ([`Tuning::economy`]), then from building to building of
//! the enemy house nearest home; new units join it in groups, and a strike
//! that is outmatched comes home ([`Tuning::recall`]). Once the house has
//! seen every enemy start and no enemy building is left in sight, the
//! strike hunts: each unit goes to the nearest seen enemy, else the nearest
//! cell the house does not see, so that the enemy's last units are found
//! and the game ends by victory (S-0414 claim 3: a house is defeated once it
//! has no building, vehicle, infantry or aircraft left).
//!
//! **Fair play.** The script reads only the house's view: its own objects,
//! the enemy objects it sees ([`Script::seen_enemies`], the shroud filter of
//! the default script) and what it remembers of them, the cells it has
//! explored or sees now, and the map's start cells (lobby knowledge, as the
//! default script). It gives only orders a person could give (move and
//! attack-move orders to its own units, and the build orders of the
//! sidebar), on its think ticks, and its house has a person's rules and
//! money (the levels change only the play: S-0302 claim 3, S-0412 claim 5).

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use glasnost_map::{CellXY, Map};
use glasnost_rules::sim::MovementZone;
use glasnost_sim::econ::{TypeRef, production, shroud, tech};
use glasnost_sim::orders::Order;
use glasnost_sim::probe::Probe;
use glasnost_sim::regions::{self, Reach};
use glasnost_sim::rules::{Behaviour, FactoryKind, Kind};
use glasnost_sim::world::{AiDifficulty, HouseId, ObjectId, Target, World};
use serde::{Deserialize, Serialize};

use crate::brain::Brain;
use crate::scripted::{self, Plan, Role, Script, dist2, idle};
use crate::scripts::{AiScript, Schedule, View};

/// The siege script's registry id.
pub const ID: u8 = 2; // nocite: design: own AI (#449): the registry's next id

/// The brain's wave size: more idle units than a house ever has, so that
/// the brain sends no wave of its own and the army is the siege's.
pub const HELD: usize = 1 << 20; // nocite: design: own AI (#449)

/// Cells within which a unit counts as at its goal (the default script's
/// `ARRIVED`).
const ARRIVED: i64 = 2; // nocite: design: own AI (#449)

/// Cells within which a goal moved on is the same goal for a unit already
/// sent: a person does not click a unit again for a few cells.
const SAME_GOAL: i64 = 4; // nocite: design: own AI (#449)

/// Cells around an enemy start within which an enemy building means the
/// start still holds an enemy base.
const SITE: i64 = 12; // nocite: design: own AI (#449)

/// The step of the grid of cells a hunt searches.
const SEARCH_STEP: i32 = 6; // nocite: design: own AI (#449)

/// How one level plays (design: own AI, #449).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tuning {
    /// Harvesters the brain's plan keeps.
    pub harvesters: usize,
    /// Whether the house builds superweapon buildings (the brain's own
    /// choice); without, the brain's orders for them are dropped.
    pub superweapons: bool,
    /// Idle army units at home that make the first strike, and a new one
    /// after a strike is lost.
    pub strike: usize,
    /// Idle units at home that go out together to join a strike.
    pub join: usize,
    /// Army units that stay home when a strike goes.
    pub guard: usize,
    /// The army acts on one of every this many of the house's think ticks.
    pub every: i32,
    /// Whether the strike hunts the enemy's last units over the map once no
    /// enemy base is left in sight.
    pub hunts: bool,
    /// The strike goes only while the units that would go are worth at least
    /// this many percent of the enemy army the house remembers ([`Army::seen`]);
    /// 0: on the count alone.
    pub edge: i64,
    /// A strike worth less than this many percent of the remembered enemy
    /// army is called home; 0: never.
    pub recall: i64,
    /// Whether the strike goes for the enemy's economy first: the seen
    /// enemy harvester or refinery nearest home before other buildings.
    pub economy: bool,
    /// Ticks without a strike after which the edge is judged at 80 percent;
    /// 0: never. A person who has waited long for an edge takes a near-even
    /// fight rather than none.
    pub patience: i32,
    /// Credits in hand from which the siege adds a factory of the army's
    /// kinds ([`Siege::more_factories`]); 0: never.
    pub rich: i32,
    /// Whether the house trains soldiers while the brain's army round waits
    /// for the plan's harvesters, with the money the next harvester leaves
    /// over ([`Siege::beside`]).
    pub beside: bool,
    /// Credits the units that would go must be worth at least for a new
    /// strike, beside the edge (which holds for any strike while the house
    /// remembers no enemy army); 0: none.
    pub floor: i64,
    /// Whether the house builds a navy while every enemy start is reached by
    /// land ([`Siege::no_navy`]); without, the brain's orders for naval
    /// buildings and ships are dropped then.
    pub navy: bool,
    /// Refineries the house builds up to, beyond the brain's own rule, once
    /// the plan's harvesters stand ([`Siege::more_refineries`]); 0: none.
    pub refineries: usize,
    /// Refineries the house owns, with the plan's harvesters standing, before
    /// the money in hand counts as rich ([`Siege::economy_stands`]); 0: from
    /// the start.
    pub rich_after: usize,
    /// Whether the army lines run while a refinery by a new field waits
    /// for money ([`Siege::train_while_paying`]); without, the brain holds
    /// them to pay for that refinery first.
    pub train_while_paying: bool,
}

/// The tuning of level `level`.
pub fn tuning(level: AiDifficulty) -> Tuning {
    match level {
        // nocite: design: own AI (#449): easy earns from one harvester, strikes with few and joins with as few, so its army goes out in small groups, and acts at a third of the rate, as the default's easy (whose brain builds no superweapon building)
        AiDifficulty::Easy => Tuning {
            harvesters: 1,
            superweapons: true,
            strike: 6,
            join: 6,
            guard: 1,
            every: 3,
            hunts: false,
            edge: 0,
            recall: 0,
            economy: false,
            patience: 0,
            rich: 0,
            beside: false,
            floor: 0,
            navy: true,
            refineries: 0,
            rich_after: 0,
            train_while_paying: false,
        },
        // nocite: design: own AI (#449): normal earns from three harvesters, puts no money into superweapons (saving for one starves its army), strikes on the count alone and joins its strike in large groups
        AiDifficulty::Normal => Tuning {
            harvesters: 3,
            superweapons: false,
            strike: 12,
            join: 6,
            guard: 3,
            every: 1,
            hunts: false,
            edge: 0,
            recall: 0,
            economy: false,
            patience: 0,
            rich: 0,
            beside: false,
            floor: 0,
            navy: true,
            refineries: 0,
            rich_after: 0,
            train_while_paying: false,
        },
        // nocite: design: own AI (#449): hard earns from three harvesters, puts no money into superweapons, strikes once ten wait and they are worth a fifth more than the enemy army it remembers (four fifths of it after 3000 ticks without a strike), calls a strike worth less than half of it home, goes for the enemy's harvesters and refineries first, joins in small groups and hunts the last units
        AiDifficulty::Hard => Tuning {
            harvesters: 3,
            superweapons: false,
            strike: 10,
            join: 3,
            guard: 2,
            every: 1,
            hunts: true,
            edge: 120,
            recall: 50,
            economy: true,
            patience: 3000,
            // nocite: design: own AI (#449): three thousand credits in hand, money the two army lines leave unspent only while the income outruns them
            rich: 3000,
            // nocite: design: own AI (#449): hard trains soldiers with the money the plan's next harvester leaves over, as a person keeps the barracks busy while the war factory builds harvesters
            beside: true,
            // nocite: design: own AI (#449): a new strike's units worth five thousand credits at least, so the soldiers trained beside the harvesters wait at home for the war factory's units rather than go out alone while the house remembers no enemy army
            floor: 5000,
            // nocite: design: own AI (#449): hard spends nothing on ships while its tanks reach every enemy start, as a person on a land map leaves the shipyard unbuilt
            navy: false,
            // nocite: design: own AI (#449): hard builds up to four refineries once its harvesters stand, as a person whose credits run out with fields unworked builds the next refinery early
            refineries: 4,
            // nocite: design: own AI (#449): the starting money is not income; hard counts as rich only on two refineries with its harvesters standing, as a person builds the second refinery before the second war factory
            rich_after: 2,
            // nocite: design: own AI (#449): hard keeps training while a refinery by a new field waits for money, as a person who builds a refinery on credit keeps the barracks and the war factory busy and lets the refinery wait
            train_while_paying: true,
        },
    }
}

/// The brain's plan at level `level`: the default plan's types by role,
/// the level's harvesters, and the waves held.
pub fn plan_for(level: AiDifficulty) -> Plan {
    Plan {
        harvesters: tuning(level).harvesters,
        wave: HELD,
        ..crate::brain::default_plan()
    }
}

/// The siege script's state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Siege {
    /// The base: the default script's brain on the siege's plan.
    brain: Brain,
    /// The house's view, asked for its objects, the enemies it sees and the
    /// enemy starts (it never thinks; derived answers only).
    eyes: Script,
    /// The army.
    army: Army,
}

/// The siege's army: the units out on the strike.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Army {
    /// The units sent out, until they are gone.
    out: BTreeSet<ObjectId>,
    /// Whether a strike is under way.
    striking: bool,
    /// The strikes launched.
    strikes: u32,
    /// The enemy army units the house has seen: by object, its worth, the
    /// cell and the tick it was last seen at. One is forgotten when the
    /// house sees its last cell without it, or [`MEMORY`] ticks after it was
    /// last seen; a person keeps such a count of the enemy army in mind.
    seen: BTreeMap<ObjectId, (i64, CellXY, i32)>,
    /// The strikes called home.
    recalls: u32,
    /// The tick the last strike ended at (called home or lost), 0 before
    /// the first; the army's patience ([`Tuning::patience`]) runs from it.
    ended: i32,
}

/// The factories of each army kind (infantry, vehicles) the siege adds up
/// to ([`Siege::more_factories`]).
const MOST_FACTORIES: usize = 3; // nocite: design: own AI (#449)

/// Ticks after which an enemy army unit not seen again is forgotten.
const MEMORY: i32 = 4500; // nocite: design: own AI (#449)

impl Army {
    /// Notes the enemy army units seen now (`enemies`, the shroud-filtered
    /// view), forgets those whose last cell the house sees without them or
    /// that have not been seen for [`MEMORY`] ticks, and returns the worth
    /// of those remembered.
    fn remember(&mut self, w: &World, house: HouseId, enemies: &[(ObjectId, CellXY)]) -> i64 {
        let now: BTreeSet<ObjectId> = enemies.iter().map(|&(e, _)| e).collect();
        self.seen.retain(|e, &mut (_, c, at)| {
            now.contains(e) || (w.tick - at < MEMORY && !shroud::seen(w, house, c))
        });
        for &(e, c) in enemies {
            let Some(o) = w.obj(e) else { continue };
            if o.kind != Kind::Building
                && scripted::role_of(w, tech::type_of(o)) == Some(Role::Army)
            {
                self.seen.insert(e, (worth(w, e), c, w.tick));
            }
        }
        self.seen.values().map(|&(v, _, _)| v).sum()
    }
}

impl Siege {
    /// The state house `house` starts a game with at level `level`.
    pub fn start(w: &World, map: &Map, house: HouseId, level: AiDifficulty) -> Self {
        let plan = plan_for(level);
        let starts = crate::skirmish::start_cells(map);
        let mut eyes = Script::new(house, plan.clone());
        eyes.set_starts(w, &starts);
        Self {
            brain: Brain::with_starts(w, &starts, house, plan),
            eyes,
            army: Army::default(),
        }
    }

    /// The strikes launched so far.
    pub fn strikes(&self) -> u32 {
        self.army.strikes
    }

    /// The strikes called home so far.
    pub fn recalls(&self) -> u32 {
        self.army.recalls
    }

    /// The worth of the enemy army units the house remembers.
    pub fn remembered(&self) -> i64 {
        self.army.seen.values().map(|&(v, _, _)| v).sum()
    }

    /// The units out on the strike.
    pub fn out(&self) -> &BTreeSet<ObjectId> {
        &self.army.out
    }

    /// Drops the brain's orders for buildings that give a superweapon
    /// ([`gives_superweapon`]), so that the level's money goes to the army
    /// and the brain never holds the army lines to save for one. The brain
    /// orders such a building in place of its tech-climb building of the
    /// think (`Script::climb`, the cheapest building that opens an army type
    /// it cannot build yet), so with one dropped the climb building is
    /// ordered instead, when its line is free and it gives no superweapon
    /// itself: the house still climbs to its better army types.
    fn no_superweapons(&self, w: &World, out: &mut Vec<Order>) {
        let before = out.len();
        // design: own AI (#449): a level without superweapons builds no building that gives one
        out.retain(|o| !matches!(o, Order::Build { what } if gives_superweapon(w, *what)));
        if out.len() == before {
            return;
        }
        let building = out
            .iter()
            .any(|o| matches!(o, Order::Build { what } if what.kind == Kind::Building));
        if let Some(c) = self.eyes.climb(w)
            && !building
            && !gives_superweapon(w, c)
            && !self.eyes.busy(w, c)
        {
            out.push(Order::Build { what: c });
        }
    }

    /// Whether every enemy start lies in the reach region (S-0212 claim 4)
    /// of the house's first ground vehicle by object number: its army
    /// reaches every enemy by land. False with no such vehicle, or no
    /// enemy start.
    fn by_land(&self, w: &World) -> bool {
        let eyes = &self.eyes;
        let ground = eyes.own(w).iter().find_map(|&id| {
            let o = w.obj(id)?;
            if o.kind != Kind::Vehicle {
                return None;
            }
            let zone = w.rules().unit(o.kind, o.type_index)?.zone;
            if zone == MovementZone::Water {
                return None;
            }
            eyes.reach_region(w, id).map(|r| (zone, r))
        });
        let Some((zone, r)) = ground else {
            return false;
        };
        let starts = eyes.enemy_starts(w);
        !starts.is_empty()
            && starts
                .iter()
                .all(|&c| regions::reach_in(w, c, zone) == Reach::Region(r))
    }

    /// No navy while the enemy is reached by land (design: own AI, #449):
    /// while [`Self::by_land`], the brain's orders for a naval building
    /// (S-0406 claim 3 d) and for a unit that moves on water are dropped,
    /// and a dropped building gives way to the climb building as in
    /// [`Self::no_superweapons`].
    fn no_navy(&self, w: &World, out: &mut Vec<Order>) {
        let ship_order = |o: &Order| matches!(o, Order::Build { what } if naval(w, *what));
        if !out.iter().any(ship_order) || !self.by_land(w) {
            return;
        }
        let building = out.iter().any(|o| {
            matches!(o, Order::Build { what } if what.kind == Kind::Building && naval(w, *what))
        });
        out.retain(|o| !ship_order(o));
        if !building
            || out
                .iter()
                .any(|o| matches!(o, Order::Build { what } if what.kind == Kind::Building))
        {
            return;
        }
        if let Some(c) = self.eyes.climb(w)
            && !naval(w, c)
            && !gives_superweapon(w, c)
            && !self.eyes.busy(w, c)
        {
            out.push(Order::Build { what: c });
        }
    }

    /// Whether the house's income stands (design: own AI, #449): with
    /// `refineries` 0 always; else once the plan's harvesters stand
    /// (`Script::harvesters_kept`) and the house owns at least `refineries`
    /// refineries, or is a slave miner's house (its miners are their own
    /// refinery, S-0402 claim 14). Money in hand before then is the starting
    /// money, which a person puts into refineries first.
    fn economy_stands(&self, w: &World, refineries: usize) -> bool {
        let eyes = &self.eyes;
        refineries == 0
            || (eyes.harvesters_kept(w)
                && (eyes.slave_miners(w) || eyes.of_role(w, Role::Refinery).len() >= refineries))
    }

    /// More factories when rich (design: own AI, #449): with at least
    /// `rich` credits in hand and no base building ordered this think
    /// ([`base_ordered`]: the brain's, or a refinery of
    /// [`Siege::more_refineries`], which goes first), in place of the brain's
    /// other building order (a rich brain orders the next building the think
    /// its line frees), the cheapest factory the house can build, place and
    /// pay for, of the army
    /// kind (infantry or vehicles) it owns fewer of, vehicles on a tie, once
    /// it owns one of that kind and while it owns fewer than
    /// [`MOST_FACTORIES`], when the building line is free; the plan's next
    /// power plant (`Script::plant_type`) in its place when the factory
    /// would leave the house short of power. Each more factory of a
    /// kind shortens the build times of that kind (S-0403 claim 6), as a
    /// person with money to spare builds a second war factory; the first of
    /// each kind stays the brain's plan.
    fn more_factories(&self, w: &World, rich: i32, out: &mut Vec<Order>) {
        let eyes = &self.eyes;
        let house = eyes.house;
        let credits = w.houses[usize::from(house.0)].credits;
        if credits < rich || base_ordered(w, out) {
            return;
        }
        let owned = |k: FactoryKind| eyes.of_role(w, Role::Factory(k)).len();
        let (inf, veh) = (owned(FactoryKind::Infantry), owned(FactoryKind::Vehicle));
        let kinds = if veh <= inf {
            [(FactoryKind::Vehicle, veh), (FactoryKind::Infantry, inf)]
        } else {
            [(FactoryKind::Infantry, inf), (FactoryKind::Vehicle, veh)]
        };
        for (k, n) in kinds {
            if n == 0 || n >= MOST_FACTORIES {
                continue;
            }
            let pick = eyes.candidates(w, Role::Factory(k)).into_iter().find(|&t| {
                production::price(w, house, t) <= credits && eyes.place_cell(w, t).is_some()
            });
            let Some(t) = pick else { continue };
            // A plant first when the factory would leave the house short.
            let t = if crate::power::short_after(w, house, t) {
                match eyes.plant_type(w) {
                    Some(p) => p,
                    None => return,
                }
            } else {
                t
            };
            if !eyes.busy(w, t) {
                out.retain(|o| !matches!(o, Order::Build { what } if what.kind == Kind::Building));
                out.push(Order::Build { what: t });
            }
            return;
        }
    }

    /// More refineries (design: own AI, #449): once the plan's harvesters
    /// stand (`Script::harvesters_kept`), while the house owns at least one
    /// refinery and fewer than `most`, the brain orders no base building this
    /// think (yard, power, refinery or factory) and no refinery is on its
    /// line, the cheapest refinery type it can place (`Script::place_refinery`:
    /// beside the field of `Script::ore_goal`), on the free building line, in
    /// place of the brain's other building order (its tech climb, a defence or
    /// a sensor: a rich brain orders the next building the think its line
    /// frees); the plan's next power plant in its place when the refinery
    /// would leave the house short of power. Each refinery
    /// comes with a harvester (S-0404 claim 2, S-0402 claim 18). The brain
    /// builds one by a new field only once the field by its refineries runs
    /// low (`Script::field_refinery`); a person whose credits run out with
    /// fields unworked builds the next refinery early. Not for slave miners,
    /// which are their slaves' own refinery (S-0402 claim 14).
    fn more_refineries(&self, w: &World, most: usize, out: &mut Vec<Order>) {
        let eyes = &self.eyes;
        let house = eyes.house;
        if eyes.selling() || eyes.slave_miners(w) || !eyes.harvesters_kept(w) {
            return;
        }
        let owned = eyes.of_role(w, Role::Refinery).len();
        if owned == 0 || owned >= most {
            return;
        }
        // The base's own buildings come first; the refinery takes the place
        // of the brain's other building orders (its tech climb, its
        // defences, its sensors).
        if base_ordered(w, out) {
            return;
        }
        let types: Vec<TypeRef> = eyes
            .candidates(w, Role::Refinery)
            .into_iter()
            .filter(|&t| !scripted::keeps_slaves(w, t))
            .collect();
        if types.iter().any(|&t| eyes.busy(w, t)) {
            return;
        }
        let Some(t) = types
            .into_iter()
            .find(|&t| eyes.place_refinery(w, t).is_some())
        else {
            return;
        };
        // A plant first when the refinery would leave the house short.
        let t = if crate::power::short_after(w, house, t) {
            match eyes.plant_type(w) {
                Some(p) => p,
                None => return,
            }
        } else {
            t
        };
        if !eyes.busy(w, t) {
            out.retain(|o| !matches!(o, Order::Build { what } if what.kind == Kind::Building));
            out.push(Order::Build { what: t });
        }
    }

    /// The army trains while a new refinery waits for money (design: own
    /// AI, #449): the brain pays for its economy first (`Brain::paying`)
    /// when it lacks a refinery or its economy target, or when a refinery
    /// by a new field owes more than the money in hand while the house is
    /// safe, and holds every other item for it (S-0404 claim 5). While the
    /// house owns a refinery, has lost none (`Script::refinery_lost`) and
    /// keeps its economy target (`Script::economy_kept`), that refinery is
    /// the only reason left; then the brain's hold orders for army items
    /// are dropped, and the army items it held are resumed with a build
    /// order for their type (S-0404 claims 3 and 5), as the brain itself
    /// resumes them. The refinery is paid beside them. Not while selling
    /// out, nor for slave miners (their own refinery, S-0402 claim 14).
    fn train_while_paying(&self, w: &World, out: &mut Vec<Order>) {
        let eyes = &self.eyes;
        if eyes.selling()
            || eyes.slave_miners(w)
            || !eyes.has_refinery(w)
            || eyes.refinery_lost(w).is_some()
            || !eyes.economy_kept(w)
        {
            return;
        }
        let army =
            |t: TypeRef| t.kind != Kind::Building && scripted::role_of(w, t) == Some(Role::Army);
        out.retain(|o| !matches!(o, Order::Hold { what } if army(*what)));
        let house = eyes.house;
        for l in w.econ.lines.lines.iter().filter(|l| l.house == house) {
            let Some(i) = &l.item else { continue };
            let resume = Order::Build { what: i.t };
            if !i.done()
                && i.hold == production::Hold::Player
                && army(i.t)
                && eyes.buildable(w, i.t)
                && !out.contains(&resume)
            {
                out.push(resume);
            }
        }
    }

    /// Soldiers beside the harvesters (design: own AI, #449): while the
    /// house keeps fewer harvesters than its plan (`Script::harvesters_kept`),
    /// the brain's army round waits for them, whatever the money in hand.
    /// A person with money to spare trains soldiers at the barracks while
    /// the war factory builds the harvesters. So with no infantry ordered
    /// this think, one infantry army type on its free line: the one the
    /// house has fewest of on the map, then has built fewest of, then the
    /// cheapest, never one it sends alone (`scripted::spent_alone`), when
    /// the money in hand, beyond what the running items owe and what this
    /// think's build orders cost, pays for the next harvester and the
    /// soldier both (the brain's own wealth test for a harvester beyond the
    /// plan, `rich` in its army round). With no harvester type to build yet,
    /// the cheapest war factory stands in for it. The vehicle line is left
    /// to the harvesters.
    fn beside(&self, w: &World, out: &mut Vec<Order>) {
        let eyes = &self.eyes;
        let house = eyes.house;
        if eyes.selling() || eyes.harvesters_kept(w) {
            return;
        }
        if out
            .iter()
            .any(|o| matches!(o, Order::Build { what } if what.kind == Kind::Infantry))
        {
            return;
        }
        let price = |t: TypeRef| i64::from(production::price(w, house, t));
        let reserve = eyes
            .candidates(w, Role::Harvester)
            .first()
            .copied()
            .or_else(|| {
                eyes.candidates(w, Role::Factory(FactoryKind::Vehicle))
                    .first()
                    .copied()
            })
            .map_or(0, price);
        let owed: i64 = w
            .econ
            .lines
            .lines
            .iter()
            .filter(|l| l.house == house)
            .filter_map(|l| l.item.as_ref())
            .filter(|i| !i.done() && i.hold == production::Hold::None)
            .map(|i| i64::from(i.owed))
            .sum();
        let ordered: i64 = out
            .iter()
            .filter_map(|o| match o {
                Order::Build { what } => Some(price(*what)),
                _ => None,
            })
            .sum();
        let spare = i64::from(w.houses[usize::from(house.0)].credits) - owed - ordered;
        let own = eyes.own(w);
        let living = |t: TypeRef| {
            own.iter()
                .filter(|&&id| w.obj(id).is_some_and(|o| tech::type_of(o) == t))
                .count()
        };
        let pick = eyes
            .candidates(w, Role::Army)
            .into_iter()
            .filter(|&t| {
                t.kind == Kind::Infantry
                    && !scripted::spent_alone(w, t)
                    && !crate::lifter::lifter_type(w, t)
                    && !eyes.busy(w, t)
            })
            .min_by_key(|&t| {
                (
                    living(t),
                    tech::ever_built(w, house, t),
                    production::cost_field(w, t),
                    t.index,
                )
            });
        if let Some(t) = pick
            && spare >= reserve + price(t)
        {
            out.push(Order::Build { what: t });
        }
    }

    /// The army's orders for this think, for units the brain gave none.
    fn command(
        &mut self,
        w: &World,
        level: AiDifficulty,
        busy: &BTreeSet<ObjectId>,
        out: &mut Vec<Order>,
    ) {
        let t = tuning(level);
        let eyes = &self.eyes;
        let Some(home) = eyes.home(w) else { return };
        let army: Vec<ObjectId> = eyes
            .of_role(w, Role::Army)
            .into_iter()
            .filter(|&id| free(w, id))
            .collect();
        let alive: BTreeSet<ObjectId> = army.iter().copied().collect();
        self.army.out.retain(|id| alive.contains(id));
        if self.army.striking && self.army.out.is_empty() {
            // The strike is lost: the army gathers again.
            self.army.striking = false;
            self.army.ended = w.tick;
        }
        let enemies = eyes.seen_enemies(w);
        let enemy_worth = self.army.remember(w, eyes.house, &enemies);
        let out_worth: i64 = self.army.out.iter().map(|&id| worth(w, id)).sum();
        // nocite: design: own AI (#449): the level's recall is a percent of the remembered enemy army's worth
        if self.army.striking && t.recall > 0 && out_worth * 100 < t.recall * enemy_worth {
            // The strike is outmatched: it comes home to fight beside the
            // base and the new units, and the army gathers again.
            for &id in &self.army.out {
                if !busy.contains(&id) {
                    out.push(Order::Behaviour {
                        actor: id,
                        behaviour: Behaviour::Move,
                        attack_move: false,
                        target: None,
                        dest: Some(Target::Cell(home)),
                    });
                }
            }
            self.army.out.clear();
            self.army.striking = false;
            self.army.recalls += 1;
            self.army.ended = w.tick;
            return;
        }
        // Idle units at home, nearest home first: the guard is the first.
        let mut waiting: Vec<(i64, ObjectId)> = army
            .iter()
            .copied()
            .filter(|id| !self.army.out.contains(id) && idle(w, *id) && !busy.contains(id))
            .filter_map(|id| w.obj(id).map(|o| (dist2(o.pos.cell(), home), id)))
            .collect();
        waiting.sort_unstable();
        let guard = t.guard.min(waiting.len());
        // The edge is judged on the units that would go, so that a strike
        // that goes is not outmatched (and called home) at once.
        let goers_worth: i64 = waiting
            .iter()
            .skip(guard)
            .map(|&(_, id)| worth(w, id))
            .sum();
        let waited = t.patience > 0 && w.tick - self.army.ended >= t.patience;
        // nocite: design: own AI (#449): after the level's patience, goers worth 80 percent of the remembered enemy army are edge enough (with the guard and the base's defence behind them)
        let want = if waited { t.edge.min(80) } else { t.edge };
        // nocite: design: own AI (#449): the edge is a percent of the remembered enemy army's worth
        let edge = want == 0 || goers_worth * 100 >= want * enemy_worth;
        // design: own AI (#449): the level's floor holds a new strike whose units are worth less, whatever the edge
        let edge = edge && goers_worth >= t.floor;
        let free_n = army.len().saturating_sub(self.army.out.len());
        let goes = if self.army.striking {
            waiting.len() > guard && waiting.len() - guard >= t.join
        } else {
            waiting.len() >= t.strike && free_n >= t.strike && edge
        };
        let mut goals = Goals::new(eyes, w, home, &enemies, t.hunts, t.economy);
        let mut send: Vec<ObjectId> = Vec::new();
        if goes {
            send.extend(waiting.iter().skip(guard).map(|&(_, id)| id));
        }
        // The units already out that stand idle, or go for a goal that has
        // moved on, are sent at the goal again.
        for &id in &self.army.out {
            if busy.contains(&id) {
                continue;
            }
            send.push(id);
        }
        let mut sent_any = false;
        for id in send {
            let Some(o) = w.obj(id) else { continue };
            let at = o.pos.cell();
            let Some(goal) = goals.for_unit(w, id) else {
                continue;
            };
            let out_already = self.army.out.contains(&id);
            if out_already {
                let fighting = o.target.is_some();
                let toward = match o.dest {
                    Some(Target::Cell(c)) => dist2(c, goal) <= SAME_GOAL * SAME_GOAL,
                    _ => false,
                };
                if fighting || (toward && !idle(w, id)) {
                    continue;
                }
            }
            if dist2(goal, at) <= ARRIVED * ARRIVED {
                continue;
            }
            out.push(Order::Behaviour {
                actor: id,
                behaviour: Behaviour::Move,
                attack_move: true,
                target: None,
                dest: Some(Target::Cell(goal)),
            });
            if !out_already {
                self.army.out.insert(id);
                sent_any = true;
            }
        }
        if sent_any && !self.army.striking {
            self.army.striking = true;
            self.army.strikes += 1;
        }
    }
}

/// Whether `t` is a building that gives a superweapon (S-0512 Terms: a
/// building type's first and second superweapon fields).
pub fn gives_superweapon(w: &World, t: TypeRef) -> bool {
    t.kind == Kind::Building
        && w.rules()
            .superweapons
            .building_sw
            .get(t.index as usize)
            .is_some_and(|f| f.iter().any(Option::is_some))
}

/// Whether `t` belongs to a navy: a naval building (S-0406 claim 3 d: it
/// stands on water) or a unit whose movement zone is water.
pub fn naval(w: &World, t: TypeRef) -> bool {
    match t.kind {
        Kind::Building => w
            .rules()
            .buildings
            .get(t.index as usize)
            .is_some_and(|b| b.naval),
        k => w
            .rules()
            .unit(k, t.index)
            .is_some_and(|u| u.zone == MovementZone::Water),
    }
}

/// Whether `out` orders a base building (design: own AI, #449): a
/// construction yard, power plant, refinery or factory. The siege's more
/// refineries and more factories wait behind these and take the place of
/// the brain's other building orders.
fn base_ordered(w: &World, out: &[Order]) -> bool {
    out.iter().any(|o| {
        matches!(o, Order::Build { what } if what.kind == Kind::Building
        && matches!(
            scripted::role_of(w, *what),
            Some(Role::Yard | Role::Power | Role::Refinery | Role::Factory(_))
        ))
    })
}

/// Whether army unit `id` is free for the strike: not bound into a building
/// or bunker, not berserk, not a lifter (it keeps to its own part), not
/// deployed, and not an aircraft (the brain's own part flies them).
fn free(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let bound = o.beh.current == Some(Behaviour::Eaten)
        || o.beh.queued == Some(Behaviour::Eaten)
        || (o.beh.current == Some(Behaviour::Enter)
            && glasnost_sim::structures::bunker::entering_bunker(w, id))
        || glasnost_sim::structures::bunker::bunkered(w, id);
    // spec: S-0214 claim 15 and S-0610 claim 6 (a berserk unit takes no orders)
    let berserk = glasnost_sim::psychic::berserk(w, id);
    o.kind != Kind::Aircraft
        && !bound
        && !berserk
        && !crate::lifter::is_lifter(w, id)
        && !crate::deployers::infantry_deployed(w, id)
        && !glasnost_sim::econ::inplace::blocks_destination(w, id)
}

/// The goals of one think, kept by the unit's movement zone and reach
/// region (S-0212 claim 4).
struct Goals<'a> {
    eyes: &'a Script,
    home: CellXY,
    enemies: &'a [(ObjectId, CellXY)],
    hunts: bool,
    /// Whether a seen enemy harvester or refinery comes before the other
    /// buildings.
    economy: bool,
    /// The enemy house the strike takes on: the owner of the seen enemy
    /// building nearest home.
    focus: Option<HouseId>,
    /// The building goal by (zone, region); `None` key for a unit with no
    /// region (a jumpjet in the air).
    kept: BTreeMap<Option<(u8, u32)>, Option<CellXY>>,
    /// The start cells that may still hold an enemy base.
    open_starts: Vec<CellXY>,
}

impl<'a> Goals<'a> {
    fn new(
        eyes: &'a Script,
        w: &World,
        home: CellXY,
        enemies: &'a [(ObjectId, CellXY)],
        hunts: bool,
        economy: bool,
    ) -> Self {
        let focus = enemies
            .iter()
            .filter_map(|&(e, c)| {
                let o = w.obj(e)?;
                (o.kind == Kind::Building).then_some((dist2(c, home), e, o.owner))
            })
            .min()
            .and_then(|(_, _, owner)| owner);
        let house = eyes.house;
        let open_starts = eyes
            .enemy_starts(w)
            .into_iter()
            .filter(|&s| {
                !shroud::explored(w, house, s)
                    || enemies.iter().any(|&(e, c)| {
                        dist2(c, s) <= SITE * SITE
                            && w.obj(e).is_some_and(|o| o.kind == Kind::Building)
                    })
            })
            .collect();
        Self {
            eyes,
            home,
            enemies,
            hunts,
            economy,
            focus,
            kept: BTreeMap::new(),
            open_starts,
        }
    }

    /// Where unit `id` goes: the seen enemy building of the focus house
    /// nearest home that it can reach (its reach region holds the building's
    /// cell), else any seen enemy building so, else the nearest start that
    /// may still hold an enemy base; when hunting, else the seen enemy
    /// nearest the unit, else the nearest cell the house does not see.
    fn for_unit(&mut self, w: &World, id: ObjectId) -> Option<CellXY> {
        let o = w.obj(id)?;
        let zone = w.rules().unit(o.kind, o.type_index).map(|u| u.zone);
        let key = match (zone, self.eyes.reach_region(w, id)) {
            (Some(z), Some(r)) => Some((z as u8, r)),
            _ => None,
        };
        let in_region = |c: CellXY| match (zone, key) {
            (Some(z), Some((_, r))) => regions::reach_in(w, c, z) == Reach::Region(r),
            _ => true,
        };
        let home = self.home;
        let focus = self.focus;
        let enemies = self.enemies;
        let open_starts = &self.open_starts;
        let economy_first = self.economy;
        let base = *self.kept.entry(key).or_insert_with(|| {
            let role = |e: ObjectId| {
                w.obj(e)
                    .and_then(|x| scripted::role_of(w, tech::type_of(x)))
            };
            let economy = || {
                enemies
                    .iter()
                    .copied()
                    .filter(|&(e, c)| {
                        matches!(role(e), Some(Role::Harvester | Role::Refinery)) && in_region(c)
                    })
                    .min_by_key(|&(e, c)| (dist2(c, home), e))
                    .map(|(_, c)| c)
            };
            let building = |any: bool| {
                enemies
                    .iter()
                    .copied()
                    .filter(|&(e, c)| {
                        w.obj(e)
                            .is_some_and(|x| x.kind == Kind::Building && (any || x.owner == focus))
                            && in_region(c)
                    })
                    .min_by_key(|&(e, c)| (dist2(c, home), e))
                    .map(|(_, c)| c)
            };
            economy()
                .filter(|_| economy_first)
                .or_else(|| building(false))
                .or_else(|| building(true))
                .or_else(|| {
                    open_starts
                        .iter()
                        .copied()
                        .filter(|&s| in_region(s))
                        .min_by_key(|&s| (dist2(s, home), s.y, s.x))
                })
        });
        if base.is_some() || !self.hunts {
            return base;
        }
        let at = o.pos.cell();
        let house = self.eyes.house;
        enemies
            .iter()
            .copied()
            .filter(|&(e, c)| in_region(c) && scripted::can_attack(w, id, e))
            .min_by_key(|&(e, c)| (dist2(c, at), e))
            .map(|(_, c)| c)
            .or_else(|| {
                w.cells
                    .grid
                    .iter()
                    .map(|(c, _)| c)
                    .filter(|c| c.x % SEARCH_STEP == 0 && c.y % SEARCH_STEP == 0)
                    .filter(|&c| w.cells.usable(c) && !shroud::seen(w, house, c) && in_region(c))
                    .min_by_key(|&c| (dist2(c, at), c.y, c.x))
            })
    }
}

/// The worth of an object by cost (the cost field, S-0403 claim 2), for
/// the tools' reports of the script.
pub fn worth(w: &World, id: ObjectId) -> i64 {
    w.obj(id).map_or(0, |o| {
        i64::from(production::cost_field(w, tech::type_of(o)))
    })
}

impl AiScript for Siege {
    fn house(&self) -> HouseId {
        self.brain.house()
    }

    fn think<P: Probe>(
        &mut self,
        view: &View<'_>,
        schedule: Schedule,
        level: AiDifficulty,
        own: Option<Arc<[ObjectId]>>,
        p: &mut P,
    ) -> Vec<Order> {
        let w = view.world();
        let mut out = AiScript::think(&mut self.brain, view, schedule, level, own.clone(), p);
        if !schedule.thinks {
            return out;
        }
        let t = tuning(level);
        // design: own AI (#449): the army acts on the house's think ticks only, and at a level with `every` above one on one of every `every` of them, read from the tick alone
        let acts = w
            .tick
            .div_euclid(scripted::THINK_PERIOD)
            .rem_euclid(t.every)
            == 0;
        let busy: BTreeSet<ObjectId> = out.iter().filter_map(Order::actor).collect();
        match own {
            Some(own) => self.eyes.begin_think_with_own(own),
            None => self.eyes.begin_think(),
        }
        if !t.superweapons {
            self.no_superweapons(w, &mut out);
        }
        if !t.navy {
            self.no_navy(w, &mut out);
        }
        if t.train_while_paying {
            self.train_while_paying(w, &mut out);
        }
        if t.refineries > 0 {
            self.more_refineries(w, t.refineries, &mut out);
        }
        if t.rich > 0 && self.economy_stands(w, t.rich_after) {
            self.more_factories(w, t.rich, &mut out);
        }
        if t.beside {
            self.beside(w, &mut out);
        }
        if acts {
            self.command(w, level, &busy, &mut out);
        }
        self.eyes.end_think();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;
    use glasnost_sim::load::new_game;
    use glasnost_sim::testkit;
    use glasnost_sim::world::Pos;

    fn world() -> World {
        new_game(
            fixture::rules(),
            &fixture::map(),
            fixture::catalogue(),
            &fixture::setup(5),
        )
        .expect("the synthetic skirmish loads")
    }

    fn building(w: &World, name: &str) -> TypeRef {
        TypeRef {
            kind: Kind::Building,
            index: w.rules().index_of(Kind::Building, name).expect(name),
        }
    }

    // design: own AI (#449): the house remembers an enemy army unit it has seen until it sees the unit's last cell without it, or for MEMORY ticks; it never learns of one it has not seen
    #[test]
    fn the_memory_forgets_a_unit_once_its_cell_is_seen_empty_or_after_a_while() {
        let mut w = world();
        let house = HouseId(0);
        // nocite: test data of our own (two cells by the enemy start, unexplored)
        let (a, b) = (fixture::at(20, 6), fixture::at(20, -6));
        assert!(!shroud::seen(&w, house, a) && !shroud::seen(&w, house, b));
        let ea = testkit::spawn(&mut w, Kind::Vehicle, "GUNR", 1, a, 0);
        let eb = testkit::spawn(&mut w, Kind::Vehicle, "GUNR", 1, b, 0);
        let one = worth(&w, ea);
        assert!(one > 0);
        let mut army = Army::default();
        assert_eq!(army.remember(&w, house, &[]), 0, "none seen, none known");
        assert_eq!(army.remember(&w, house, &[(ea, a), (eb, b)]), 2 * one);
        // Out of sight, both are still in mind.
        assert_eq!(army.remember(&w, house, &[]), 2 * one);
        // The house sees a's cell without it: a is forgotten, b kept.
        shroud::area_reveal(&mut w, house, Pos::centre(a, 0), 2);
        assert!(shroud::seen(&w, house, a));
        assert_eq!(army.remember(&w, house, &[]), one);
        // After MEMORY ticks unseen, b is forgotten too.
        w.tick += MEMORY;
        assert_eq!(army.remember(&w, house, &[]), 0);
    }

    // design: own AI (#449): at hard, while the house's first ground vehicle reaches every enemy start by land, the brain's orders for a naval building and a ship are dropped and its other orders kept; the other levels keep their navy
    #[test]
    fn hard_builds_no_navy_while_the_enemy_is_reached_by_land() {
        assert!(!tuning(AiDifficulty::Hard).navy);
        assert!(tuning(AiDifficulty::Easy).navy && tuning(AiDifficulty::Normal).navy);
        let mut w = world();
        let navy = building(&w, "NAVY");
        let vehicle = |w: &World, name: &str| TypeRef {
            kind: Kind::Vehicle,
            index: w.rules().index_of(Kind::Vehicle, name).expect(name),
        };
        let (boat, gunr) = (vehicle(&w, "BOAT"), vehicle(&w, "GUNR"));
        // spec: S-0406 claim 3 d (a naval building); S-0212 claim 4 (a ship's zone is water)
        assert!(naval(&w, navy) && naval(&w, boat));
        assert!(!naval(&w, gunr) && !naval(&w, building(&w, "POWR")));
        // A tank of house 0 by its start: the skirmish's enemy start is reached by land.
        // nocite: test data of our own (a usable free cell a few cells from the start)
        let c = (2..8)
            .flat_map(|d| [fixture::at(d, 3), fixture::at(-d, 3), fixture::at(d, -3)])
            .find(|&c| w.cells.usable(c) && w.objects.values().all(|o| o.pos.cell() != c))
            .expect("a free cell by the start");
        testkit::spawn(&mut w, Kind::Vehicle, "GUNR", 0, c, 0);
        let s = Siege::start(&w, &fixture::map(), HouseId(0), AiDifficulty::Hard);
        s.eyes.begin_think();
        assert!(s.by_land(&w));
        let mut out = vec![
            Order::Build { what: navy },
            Order::Build { what: boat },
            Order::Build { what: gunr },
        ];
        s.no_navy(&w, &mut out);
        s.eyes.end_think();
        assert!(
            !out.iter()
                .any(|o| matches!(o, Order::Build { what } if naval(&w, *what))),
            "{out:?}"
        );
        assert!(
            out.iter()
                .any(|o| matches!(o, Order::Build { what } if *what == gunr)),
            "{out:?}"
        );
    }

    // design: own AI (#449): at a level without superweapons the brain's order for a superweapon building is dropped, and its other orders are kept
    #[test]
    fn a_level_without_superweapons_orders_no_superweapon_building() {
        let w = world();
        let (silo, powr) = (building(&w, "SILO"), building(&w, "POWR"));
        // spec: S-0512 Terms (a building's superweapon fields)
        assert!(gives_superweapon(&w, silo) && !gives_superweapon(&w, powr));
        let s = Siege::start(&w, &fixture::map(), HouseId(0), AiDifficulty::Hard);
        s.eyes.begin_think();
        let mut out = vec![Order::Build { what: silo }, Order::Build { what: powr }];
        s.no_superweapons(&w, &mut out);
        assert_eq!(
            format!("{out:?}"),
            format!("{:?}", vec![Order::Build { what: powr }])
        );
        let mut out = vec![Order::Build { what: silo }];
        s.no_superweapons(&w, &mut out);
        s.eyes.end_think();
        assert!(
            !out.iter()
                .any(|o| matches!(o, Order::Build { what } if gives_superweapon(&w, *what))),
            "{out:?}"
        );
    }
}
