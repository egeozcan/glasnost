//! A scripted house: a simple build plan played through orders a human
//! could issue, for the headless skirmish and the determinism gate.
//!
//! Each think looks only at the house's own objects, its production lines,
//! what its sidebar offers and what stands on the cells its shroud shows
//! (every explored cell: fog of war is off in a skirmish, S-0409 claim 19),
//! and returns orders:
//!
//! 1. deploy the MCV (any vehicle that deploys into a building) while the
//!    house has no construction yard;
//! 2. build a power plant, a refinery, then a factory of infantry and one of
//!    vehicles (skipping, for now, a role the house cannot build a type for
//!    yet), and place each one when it is done, on the first cell of a
//!    square spiral around the construction yard that passes the place
//!    order's conditions and the cell test;
//! 3. build another power plant whenever the drain reaches the output;
//!    once the base stands and the harvesters are kept, climb the tech
//!    tree: for an army type the house cannot build yet, build the
//!    cheapest building that clears one of its blockers (an unmet
//!    prerequisite, read off the tech tree as a player reads the sidebar's
//!    tooltips, or a missing factory of its kind: an aircraft's pad, a
//!    ship's naval yard), climbing further when
//!    that building is itself blocked; a building is only ordered when the
//!    spiral has a cell for it (a naval yard needs water, looked for in a
//!    wider spiral). While a naval yard can be built but finds no water
//!    next to the base, the house builds power plants on the legal cells
//!    nearest the explored water, so that the base grows toward it;
//! 4. keep the plan's harvester count, and send idle or resting harvesters
//!    to harvest; a slave miner counts as a harvester (and, deployed, its
//!    building too), needs no refinery and stands for one (so a house
//!    whose slave miners have all packed up to move on orders no new
//!    refinery building), and is sent to harvest at the
//!    nearest explored resource cell, the order a person gives it;
//! 5. once the harvesters are kept, build army units (one type per kind:
//!    vehicles, infantry, aircraft; the type it has fewest of, in turn on
//!    a tie), and when
//!    a wave is ready attack-move it to the nearest seen enemy or the next enemy start position (lobby
//!    knowledge: the map's start waypoints), and once every start position
//!    has been visited, to the nearest cell of a coarse search grid that
//!    the house does not see now; idle units attack seen enemies near them.
//!    A smaller wave goes too once units have waited long enough, so a house
//!    that cannot afford a full wave still attacks. A unit whose goal lies
//!    outside its reach region (a ship, for a goal on land) goes to the
//!    nearest cell of its region instead. An idle aircraft with ammunition
//!    left attacks the nearest seen enemy and flies back to rearm by
//!    itself.
//!
//! Types are found by role from the rules (a building with power output, the
//! refinery flag, a factory kind; a harvester; a unit with a primary weapon)
//! among those the house can build now, the cheapest first; a plan can name
//! the types to use instead.
//!
//! The numbers here (think period, spiral size, engage distance, wave patience) are tuning
//! of our own opponent, not game behaviour.

use std::collections::BTreeMap;

use glasnost_map::CellXY;
use glasnost_sim::combat::{check, weapon};
use glasnost_sim::econ::TypeRef;
use glasnost_sim::econ::{deploy, placement, production, resource, sell, shroud, slaves, tech};
use glasnost_sim::orders::Order;
use glasnost_sim::regions::{self, Reach};
use glasnost_sim::rules::{Behaviour, FactoryKind, Kind, Locomotor};
use glasnost_sim::targeting;
use glasnost_sim::world::{HouseId, Object, ObjectId, Target, World};
use serde::{Deserialize, Serialize};

/// A build plan.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    /// Type names to use instead of finding types by role; an empty list
    /// leaves that role to the rules.
    pub names: Names,
    /// Harvesters to keep.
    pub harvesters: usize,
    /// Idle army units that make a wave.
    pub wave: usize,
}

/// Type names of a plan, by role.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Names {
    /// Power plants.
    pub power: Vec<String>,
    /// Refineries.
    pub refinery: Vec<String>,
    /// Factories.
    pub factories: Vec<String>,
    /// Harvesters.
    pub harvester: Vec<String>,
    /// Army units.
    pub army: Vec<String>,
}

/// The kinds of army the round-robin builds from, one counter each.
const ARMY_KINDS: [Kind; 3] = [Kind::Vehicle, Kind::Infantry, Kind::Aircraft]; // nocite: the three kinds of army unit

/// Ticks between two thinks.
pub const THINK_PERIOD: i32 = 15; // nocite: tuning of our own opponent

/// Cells within which an idle army unit attacks a seen enemy.
const ENGAGE: i64 = 10; // nocite: tuning of our own opponent

/// Cells from home within which infantry waiting for a ferry stays (the
/// brain's ferry takes idle infantry from within twice its home radius).
const FERRY_GATHER: i64 = 15; // nocite: tuning of our own opponent

/// Cells beyond a barracks, toward the attack goal, that the rally point
/// click is made on.
const RALLY_AHEAD: i32 = 3; // nocite: tuning of our own opponent

/// Ticks after which idle army units go as a wave even when fewer than the
/// plan's wave size.
const WAVE_PATIENCE: i32 = 3000; // nocite: tuning of our own opponent

/// Spacing of the search grid that waves sweep once every enemy start
/// position has been visited.
const SEARCH_STEP: i32 = 8; // nocite: tuning of our own opponent

/// Ticks without income after which a house that cannot afford an army
/// unit counts as broke and sells a building.
const BROKE_PATIENCE: i32 = 1500; // nocite: tuning of our own opponent

/// Ticks after a sale within which a rise of credits is taken as its refund,
/// not as income.
const SALE_WINDOW: i32 = 300; // nocite: tuning of our own opponent

/// Free cells kept between the house's buildings.
const GAP: i32 = 2; // nocite: tuning of our own opponent

/// The largest ring of the placement spiral around the construction yard.
pub(crate) const SPIRAL_RINGS: i32 = 12; // nocite: tuning of our own opponent

/// The largest ring of the wider search around the construction yard: for
/// a naval building, and for a building placed toward the water.
pub(crate) const WIDE_RINGS: i32 = 32; // nocite: tuning of our own opponent

/// How many buildings deep the tech climb looks for one it can build now.
pub(crate) const CLIMB_DEPTH: u32 = 4; // nocite: tuning of our own opponent

/// What a type is for in the plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    /// A construction yard (a factory of buildings).
    Yard,
    /// A power plant.
    Power,
    /// A refinery.
    Refinery,
    /// A factory of infantry or vehicles.
    Factory(FactoryKind),
    /// A harvester.
    Harvester,
    /// An armed ground unit.
    Army,
}

/// The factories the plan builds, in order.
const FACTORIES: [FactoryKind; 2] = [FactoryKind::Infantry, FactoryKind::Vehicle]; // nocite: the script's own build order, not game behaviour

/// Every building, vehicle, infantry and aircraft type.
pub fn all_types(w: &World) -> Vec<TypeRef> {
    let r = w.rules();
    let n = |k: Kind| match k {
        Kind::Building => r.buildings.len(),
        Kind::Vehicle => r.vehicles.len(),
        Kind::Aircraft => r.aircraft.len(),
        _ => r.infantry.len(),
    };
    [
        Kind::Building,
        Kind::Vehicle,
        Kind::Infantry,
        Kind::Aircraft,
    ]
    .into_iter()
    .flat_map(|kind| (0..n(kind) as u32).map(move |index| TypeRef { kind, index }))
    .collect()
}

/// A type's name.
fn name_of(w: &World, t: TypeRef) -> &str {
    let r = w.rules();
    match t.kind {
        Kind::Building => r.buildings.get(t.index as usize).map(|b| b.name.as_str()),
        Kind::Vehicle => r.vehicles.get(t.index as usize).map(|u| u.name.as_str()),
        Kind::Infantry => r.infantry.get(t.index as usize).map(|u| u.name.as_str()),
        Kind::Aircraft => r.aircraft.get(t.index as usize).map(|u| u.name.as_str()),
        _ => None,
    }
    .unwrap_or("")
}

/// The role type `t` plays in the plan, if any (the first that fits).
pub fn role_of(w: &World, t: TypeRef) -> Option<Role> {
    let roles = [Role::Yard, Role::Power, Role::Refinery]
        .into_iter()
        .chain(FACTORIES.map(Role::Factory))
        .chain([Role::Harvester, Role::Army]);
    roles.into_iter().find(|&r| plays(w, t, r))
}

/// Whether type `t` plays `role`.
fn plays(w: &World, t: TypeRef, role: Role) -> bool {
    let r = w.rules();
    match t.kind {
        Kind::Building => {
            let Some(b) = r.buildings.get(t.index as usize) else {
                return false;
            };
            match role {
                Role::Power => b.factory.is_none() && b.raw.econ.power.unwrap_or(0) > 0,
                // The refinery flag, or a member of the refinery group
                // (S-0407 Import mapping, PROC).
                Role::Refinery => {
                    b.raw.econ.refinery == Some(true)
                        || r.econ.tech.groups[tech::Group::Proc as usize].contains(&t.index)
                }
                Role::Factory(k) => b.factory == Some(k) && !b.naval,
                Role::Yard => b.factory == Some(FactoryKind::Building),
                _ => false,
            }
        }
        Kind::Aircraft => {
            let Some(u) = r.aircraft.get(t.index as usize) else {
                return false;
            };
            // An armed flying aircraft that is not a spawner's slot type
            // (those come with their spawner, S-0506 claim 1).
            let slot = u.raw.spawned == Some(true)
                || u.raw.missile_spawn == Some(true)
                || r.missiles.spawners.values().any(|s| s.spawned == t.index);
            role == Role::Army && armed(u) && u.locomotor == Locomotor::Fly && !slot
        }
        Kind::Vehicle | Kind::Infantry => {
            let list = if t.kind == Kind::Vehicle {
                &r.vehicles
            } else {
                &r.infantry
            };
            let Some(u) = list.get(t.index as usize) else {
                return false;
            };
            let ground = matches!(t.kind, Kind::Infantry)
                || deploy::deploys_into(w, t.kind, t.index).is_none();
            // Only types the simulation moves: driving, walking,
            // teleporting, ship and hovering vehicles and infantry.
            let moves = u.locomotor != Locomotor::None;
            match role {
                Role::Harvester => (u.harvester || is_slave_miner(w, t)) && moves,
                Role::Army => armed(u) && !u.harvester && !is_slave_miner(w, t) && ground && moves,
                _ => false,
            }
        }
        _ => false,
    }
}

/// Whether unit type `u` is armed: it has a primary weapon, or it is a
/// gattling type with a weapon in its stage slots (S-0609 Terms: its
/// weapons are the slot list, often with no primary weapon set).
pub(crate) fn armed(u: &glasnost_sim::rules::UnitRules) -> bool {
    u.primary.is_some() || (u.combat.gattling && u.combat.slots[0].is_some())
}

/// Whether type `t` is listed in a paradrop payload (S-0522 claim 2): the
/// paratroopers a paradrop lands join the army whatever the plan's names.
fn paratrooper(w: &World, t: TypeRef) -> bool {
    t.kind == Kind::Infantry
        && w.rules()
            .superweapons
            .payloads
            .iter()
            .flatten()
            .flatten()
            .any(|&(i, _)| i == t.index)
}

/// Whether type `t` is a slave miner: a vehicle that gathers with slaves and
/// takes their loads itself (S-0402 claim 14 and Terms).
pub(crate) fn is_slave_miner(w: &World, t: TypeRef) -> bool {
    t.kind == Kind::Vehicle
        && w.rules()
            .econ
            .harvest
            .vehicles
            .get(t.index as usize)
            .is_some_and(|u| u.slave_miner)
}

/// Whether type `kind`/`index` is a vehicle that deploys into a
/// construction yard (a slave miner deploys too, but into its own building).
pub fn deploys_into_yard(w: &World, kind: Kind, index: u32) -> bool {
    deploy::deploys_into(w, kind, index).is_some_and(|b| {
        w.rules()
            .buildings
            .get(b as usize)
            .is_some_and(|b| b.factory == Some(FactoryKind::Building))
    })
}

/// Whether an object is a construction yard (a factory of buildings).
pub fn is_yard(w: &World, o: &glasnost_sim::world::Object) -> bool {
    o.kind == Kind::Building
        && w.rules()
            .buildings
            .get(o.type_index as usize)
            .is_some_and(|b| b.factory == Some(FactoryKind::Building))
}

/// Cells within which a unit moving to a cell counts as arrived, so that a
/// crowd jammed around its goal is free for the next wave.
const ARRIVED: i64 = 2; // nocite: tuning of our own opponent

/// Whether a unit has nothing to do: no destination (or a cell destination
/// it has as good as reached), no target, and a resting behaviour (or a
/// move that has arrived).
pub fn idle(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        let arrived = match o.dest {
            None => true,
            Some(Target::Cell(c)) => {
                o.beh.current == Some(Behaviour::Move)
                    && dist2(c, o.pos.cell()) <= ARRIVED * ARRIVED
            }
            Some(_) => false,
        };
        // A deployed vehicle or one changing form takes no destination (S-0517 claim 9), so it is not free for a wave.
        // A bunkered vehicle refuses a destination (S-0606; `bunker` lets it out first).
        arrived
            && !glasnost_sim::structures::bunker::bunkered(w, id)
            && !glasnost_sim::econ::inplace::blocks_destination(w, id)
            && (o.target.is_none() || stale_guard_target(w, o))
            && matches!(
                o.beh.current,
                None | Some(Behaviour::Guard)
                    | Some(Behaviour::AreaGuard)
                    | Some(Behaviour::Stop)
                    | Some(Behaviour::Move)
            )
    })
}

/// Whether a unit resting in guard holds a target it cannot shoot at: guard
/// keeps an acquired target while it is out of the weapon's range, and the
/// unit stays where it is (guard never pursues, S-0215 claim 24). A person
/// sees such a unit standing still and gives it an order; so does the
/// script, which would otherwise leave a home guard idle while the enemy
/// destroys the base.
fn stale_guard_target(w: &World, o: &Object) -> bool {
    let resting = matches!(
        o.beh.current,
        Some(Behaviour::Guard) | Some(Behaviour::AreaGuard) | Some(Behaviour::Stop)
    );
    let Some(t) = o.target else { return false };
    if !resting {
        return false;
    }
    let (slot, weapon) = weapon::would_use(w, o.id, Some(t));
    weapon.is_none()
        || matches!(
            check::check(w, o.id, Some(t), slot, true),
            check::Answer::OutOfRange | check::Answer::Illegal | check::Answer::Impossible
        )
}

/// Whether unit `id` can attack enemy `e` at all, as its cursor shows: the
/// weapon it would use against it (S-0308 claim 3) passes the firing check
/// without the range test, short of an illegal target or an impossible
/// shot (S-0301 claim 9). An aircraft out of a ground weapon's reach is
/// not worth a unit's attention.
pub(crate) fn can_attack(w: &World, id: ObjectId, e: ObjectId) -> bool {
    let t = Some(Target::Object(e));
    let (slot, weapon) = weapon::would_use(w, id, t);
    weapon.is_some()
        && !matches!(
            check::check(w, id, t, slot, false),
            check::Answer::Illegal | check::Answer::Impossible
        )
}

/// The squared distance between two cells.
/// Whether `t` is a naval building (S-0406 claim 3 d: it stands on water).
fn naval_building(w: &World, t: TypeRef) -> bool {
    t.kind == Kind::Building
        && w.rules()
            .buildings
            .get(t.index as usize)
            .is_some_and(|b| b.naval)
}

pub(crate) fn dist2(a: CellXY, b: CellXY) -> i64 {
    let dx = i64::from(a.x - b.x);
    let dy = i64::from(a.y - b.y);
    dx * dx + dy * dy
}

/// A scripted house.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Script {
    /// The house.
    pub house: HouseId,
    plan: Plan,
    /// Enemy start cells, in visiting order.
    targets: Vec<CellXY>,
    /// The next target's index.
    next_target: usize,
    /// Where the next tie among army types starts, one counter per kind of
    /// army (vehicle, infantry, aircraft): a counter shared by the kinds
    /// would step past the same types every round and leave some never
    /// built.
    next_army: [usize; ARMY_KINDS.len()],
    /// The tick since which idle army units have been waiting for a wave.
    waiting_since: Option<i32>,
    /// The economy watch: credits and the harvesters' summed loads at the
    /// last think, the tick since which neither showed income, the last
    /// sale's tick, and whether the house is selling out (it then builds no
    /// more buildings or harvesters, only army units).
    watch: Watch,
}

/// What [`Script`] remembers of the house's income between thinks.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Watch {
    credits: i32,
    loads: i64,
    stall_since: Option<i32>,
    last_sale: Option<i32>,
    selling: bool,
}

impl Script {
    /// A script for `house` playing `plan`; [`Script::set_starts`] gives
    /// it the cells to attack.
    pub fn new(house: HouseId, plan: Plan) -> Self {
        Self {
            house,
            plan,
            targets: Vec::new(),
            next_target: 0,
            next_army: [0; ARMY_KINDS.len()],
            waiting_since: None,
            watch: Watch::default(),
        }
    }

    /// Sets the enemy start cells to visit: every start cell but the one
    /// nearest the house's first objects.
    pub fn set_starts(&mut self, w: &World, starts: &[CellXY]) {
        let home = self.home(w);
        self.targets = starts
            .iter()
            .copied()
            .filter(|&c| home.is_none_or(|h| dist2(c, h) > 25)) // nocite: tuning (own start within 5 cells)
            .collect();
        if let Some(h) = home {
            self.targets.sort_by_key(|&c| dist2(c, h));
        }
    }

    /// Whether `t` may be used for `role` under the plan's names.
    fn allowed(&self, w: &World, t: TypeRef, role: Role) -> bool {
        let n = &self.plan.names;
        let list = match role {
            Role::Power => &n.power,
            Role::Refinery => &n.refinery,
            Role::Yard | Role::Factory(_) => &n.factories,
            Role::Harvester => &n.harvester,
            Role::Army => &n.army,
        };
        plays(w, t, role) && (list.is_empty() || list.iter().any(|x| x == name_of(w, t)))
    }

    /// The types the house can build now for `role`, cheapest first.
    pub(crate) fn candidates(&self, w: &World, role: Role) -> Vec<TypeRef> {
        let mut v: Vec<TypeRef> = all_types(w)
            .into_iter()
            .filter(|&t| self.allowed(w, t, role) && tech::buildable(w, self.house, t))
            .collect();
        v.sort_by_key(|&t| (production::cost_field(w, t), t.kind as u8, t.index));
        v
    }

    pub(crate) fn own(&self, w: &World) -> Vec<ObjectId> {
        w.objects
            .values()
            .filter(|o| o.owner == Some(self.house) && o.on_map && o.health > 0)
            .map(|o| o.id)
            .collect()
    }

    /// The house's deployed slave miners and other buildings with slaves
    /// (S-0402 claim 19): economy, like its harvesters.
    pub(crate) fn slave_buildings(&self, w: &World) -> Vec<ObjectId> {
        self.own(w)
            .into_iter()
            .filter(|&id| {
                w.obj(id).is_some_and(|o| o.kind == Kind::Building) && slaves::is_master(w, id)
            })
            .collect()
    }

    /// Whether the house has a refinery for the plan: a refinery building,
    /// or a slave miner on the move, which acts as its slaves' refinery
    /// whether it is a vehicle or deployed (S-0402 claim 14). A person
    /// whose slave miners have packed up to move on does not build another
    /// refinery for that.
    pub(crate) fn has_refinery(&self, w: &World) -> bool {
        // spec: S-0402 claim 14 (a slave miner acts as its slaves' refinery as a vehicle too)
        !self.of_role(w, Role::Refinery).is_empty()
            || self.own(w).into_iter().any(|id| {
                w.obj(id)
                    .is_some_and(|o| is_slave_miner(w, tech::type_of(o)))
            })
    }

    /// The nearest explored cell to `from` that holds a resource, the first
    /// in cell order on equal distance: where a person sends a slave miner.
    fn field_cell(&self, w: &World, from: CellXY) -> Option<CellXY> {
        w.cells
            .grid
            .iter()
            .map(|(c, _)| c)
            .filter(|&c| {
                w.cells.usable(c)
                    && shroud::explored(w, self.house, c)
                    && resource::resource_at(w, c).is_some()
            })
            .min_by_key(|&c| (dist2(c, from), c.y, c.x))
    }

    /// The house's objects on the map that play `role`. A slave with a
    /// master (S-0402 claim 14) works for its master and is never army, even
    /// when its type is armed.
    pub(crate) fn of_role(&self, w: &World, role: Role) -> Vec<ObjectId> {
        self.own(w)
            .into_iter()
            .filter(|&id| {
                w.obj(id).is_some_and(|o| {
                    let t = tech::type_of(o);
                    self.allowed(w, t, role)
                        || (role == Role::Army && plays(w, t, role) && paratrooper(w, t))
                })
                    && !(role == Role::Army && slaves::is_slave(w, id))
                    // A person orders only what they can select.
                    && (matches!(role, Role::Yard | Role::Power | Role::Refinery | Role::Factory(_))
                        || targeting::selectable(w, id))
            })
            .collect()
    }

    /// The house's construction yard or, before one exists, its first
    /// object's cell.
    pub(crate) fn home(&self, w: &World) -> Option<CellXY> {
        let own = self.own(w);
        own.iter()
            .find(|&&id| w.obj(id).is_some_and(|o| is_yard(w, o)))
            .or(own.first())
            .and_then(|&id| w.obj(id))
            .map(|o| o.pos.cell())
    }

    /// The enemy start cells (the map's start waypoints of the other
    /// players: lobby knowledge).
    pub(crate) fn enemy_starts(&self) -> &[CellXY] {
        &self.targets
    }

    /// Where the next wave would go, without choosing it: the seen enemy
    /// nearest home, else the next enemy start cell.
    pub(crate) fn attack_goal(&self, w: &World) -> Option<CellXY> {
        let home = self.home(w)?;
        self.seen_enemies(w)
            .into_iter()
            .min_by_key(|&(e, c)| (dist2(c, home), e))
            .map(|(_, c)| c)
            .or_else(|| {
                (!self.targets.is_empty())
                    .then(|| self.targets[self.next_target % self.targets.len()])
            })
    }

    /// Where a ferry would go, without choosing it: the seen enemy
    /// building nearest home, else the next enemy start cell. A person
    /// ferries infantry to the enemy's base, not to a unit that flies or
    /// stands where no one walks.
    pub(crate) fn base_goal(&self, w: &World) -> Option<CellXY> {
        let home = self.home(w)?;
        self.seen_enemies(w)
            .into_iter()
            .filter(|&(e, _)| w.obj(e).is_some_and(|o| o.kind == Kind::Building))
            .min_by_key(|&(e, c)| (dist2(c, home), e))
            .map(|(_, c)| c)
            .or_else(|| {
                (!self.targets.is_empty())
                    .then(|| self.targets[self.next_target % self.targets.len()])
            })
    }

    /// Whether the line of `t` holds an item or a queue.
    pub(crate) fn busy(&self, w: &World, t: TypeRef) -> bool {
        production::line_of(w, self.house, t)
            .is_some_and(|l| l.item.is_some() || !l.queue.is_empty())
    }

    /// Orders a build of the cheapest type the house can build for `role`
    /// when its line is free; returns the type ordered.
    fn build(&self, w: &World, role: Role, out: &mut Vec<Order>) -> Option<TypeRef> {
        let t = self
            .candidates(w, role)
            .into_iter()
            .find(|&t| self.placeable(w, t))?;
        self.order_build(w, t, out)
    }

    /// Orders a build of `t` when its line is free.
    fn order_build(&self, w: &World, t: TypeRef, out: &mut Vec<Order>) -> Option<TypeRef> {
        if self.busy(w, t) {
            return None;
        }
        out.push(Order::Build { what: t });
        Some(t)
    }

    /// Whether `t` is not a building or the spiral has a cell for it now,
    /// so that a finished building never waits on its line for a cell.
    fn placeable(&self, w: &World, t: TypeRef) -> bool {
        t.kind != Kind::Building || self.place_cell(w, t).is_some()
    }

    /// The building types that meet prerequisite entry `p` (S-0407
    /// claim 8): the building itself or a member of the group.
    fn meeting(w: &World, p: tech::Prereq) -> Vec<u32> {
        match p {
            tech::Prereq::Building(i) => vec![i],
            tech::Prereq::Group(g) => w.rules().econ.tech.groups[g as usize].clone(),
            tech::Prereq::Upgrade(_) | tech::Prereq::Unmet => Vec::new(),
        }
    }

    /// The factory building types that could make `t`: of its kind, naval
    /// for a naval vehicle (S-0404 Terms).
    fn factories_for(w: &World, t: TypeRef) -> Vec<u32> {
        let naval = t.kind == Kind::Vehicle && production::naval(w, t);
        w.rules()
            .buildings
            .iter()
            .enumerate()
            .filter(|(_, b)| b.factory.is_some_and(|f| f.kind() == t.kind) && b.naval == naval)
            .map(|(i, _)| i as u32)
            .collect()
    }

    /// A building the house can build and place now that clears a blocker
    /// of `t`, looking `depth` buildings deeper when the one that clears it
    /// is blocked itself; `None` when `t` is not blocked, is refused for
    /// another reason (tech level, owner, side) or nothing clears it.
    pub(crate) fn unblocker(&self, w: &World, t: TypeRef, depth: u32) -> Option<TypeRef> {
        if tech::buildable(w, self.house, t) {
            return None;
        }
        let blockers = tech::unmet_prerequisites(w, self.house, t)?;
        let mut options: Vec<Vec<u32>> =
            blockers.into_iter().map(|p| Self::meeting(w, p)).collect();
        if options.is_empty() {
            // Every prerequisite is met: a factory of its kind may be
            // missing. A factory type the house already has does not help
            // (the type is refused for another reason, such as its build
            // limit or the factory's owner countries), so only the others
            // are tried.
            let missing: Vec<u32> = Self::factories_for(w, t)
                .into_iter()
                .filter(|&index| {
                    let b = TypeRef {
                        kind: Kind::Building,
                        index,
                    };
                    tech::present_count(w, self.house, b) == 0
                })
                .collect();
            options.push(missing);
        }
        for list in options {
            let mut list: Vec<TypeRef> = list
                .into_iter()
                .map(|index| TypeRef {
                    kind: Kind::Building,
                    index,
                })
                .collect();
            list.sort_by_key(|&b| (production::cost_field(w, b), b.index));
            for b in list {
                if tech::buildable(w, self.house, b) {
                    if self.placeable(w, b) {
                        return Some(b);
                    }
                } else if depth > 0
                    && let Some(x) = self.unblocker(w, b, depth - 1)
                {
                    return Some(x);
                }
            }
        }
        None
    }

    /// The next building of the tech climb: the cheapest building that
    /// clears a blocker of an army type the house cannot build yet.
    pub(crate) fn climb(&self, w: &World) -> Option<TypeRef> {
        all_types(w)
            .into_iter()
            .filter(|&t| self.allowed(w, t, Role::Army) && !tech::buildable(w, self.house, t))
            .filter_map(|t| self.unblocker(w, t, CLIMB_DEPTH))
            .min_by_key(|&b| (production::cost_field(w, b), b.index))
    }

    /// Updates the economy watch and, when the house is broke (no income:
    /// no credits earned and no change in what its harvesters and its slaves
    /// on the map carry, for [`BROKE_PATIENCE`] ticks; and too few credits
    /// for its cheapest army unit), sells one building, as a person in its seat would, the
    /// costliest first. It keeps one factory per kind while the house has
    /// army units; a house with nothing left to fight with sells out, which
    /// ends its part in the game.
    fn sell_when_broke(&mut self, w: &World, out: &mut Vec<Order>) {
        let t = w.tick;
        let credits = w.houses[usize::from(self.house.0)].credits;
        let loads: i64 = self
            .of_role(w, Role::Harvester)
            .iter()
            .filter_map(|&id| w.obj(id)?.econ.harvest.as_ref())
            .map(|h| h.load.iter().map(|&b| i64::from(b)).sum::<i64>())
            .sum::<i64>()
            + self
                .own(w)
                .iter()
                .filter_map(|&id| w.obj(id)?.econ.slave.as_ref())
                .map(|s| s.load.iter().map(|&b| i64::from(b)).sum::<i64>())
                .sum::<i64>();
        let refund = self.watch.last_sale.is_some_and(|s| t - s < SALE_WINDOW);
        let income = loads != self.watch.loads || (credits > self.watch.credits && !refund);
        self.watch.credits = credits;
        self.watch.loads = loads;
        if income {
            self.watch.stall_since = None;
            self.watch.selling = false;
            return;
        }
        let since = *self.watch.stall_since.get_or_insert(t);
        // Broke: too few credits for anything the house can build now.
        let affordable = all_types(w)
            .into_iter()
            .filter(|&a| tech::buildable(w, self.house, a))
            .any(|a| credits >= production::cost_field(w, a));
        if t - since < BROKE_PATIENCE || affordable || refund {
            return;
        }
        self.watch.selling = true;
        let army = !self.of_role(w, Role::Army).is_empty();
        let keep = |id: ObjectId| {
            let Some(o) = w.obj(id) else { return true };
            FACTORIES.iter().any(|&k| {
                self.allowed(w, tech::type_of(o), Role::Factory(k))
                    && self.of_role(w, Role::Factory(k)).first() == Some(&id)
            }) && army
        };
        let sale = self
            .own(w)
            .into_iter()
            .filter(|&id| w.obj(id).is_some_and(|o| o.econ.sale.is_none()) && sell::sellable(w, id))
            .filter(|&id| !keep(id))
            .max_by_key(|&id| {
                let o = w.obj(id).expect("own object");
                (
                    production::cost_field(w, tech::type_of(o)),
                    std::cmp::Reverse(id),
                )
            });
        if let Some(actor) = sale {
            out.push(Order::Sell { actor });
            self.watch.last_sale = Some(t);
            self.watch.stall_since = Some(t);
        }
    }

    /// Whether the house keeps the plan's harvester count (deployed slave
    /// miners count).
    pub(crate) fn harvesters_kept(&self, w: &World) -> bool {
        self.of_role(w, Role::Harvester).len() + self.slave_buildings(w).len()
            >= self.plan.harvesters
    }

    /// Rally-point clicks for the house's infantry factories that have
    /// none: on the cell `RALLY_AHEAD` cells beyond the foundation toward
    /// the attack goal (the click's own search picks the cell, S-0212
    /// table T5).
    fn rally_points(&self, w: &World, own: &[ObjectId], out: &mut Vec<Order>) {
        let goal = self.attack_goal(w);
        for &id in own {
            let Some(o) = w.obj(id) else { continue };
            let Some(b) = w.rules().buildings.get(o.type_index as usize) else {
                continue;
            };
            if o.kind != Kind::Building
                || o.guard_spot.is_some()
                || b.factory != Some(FactoryKind::Infantry)
            {
                continue;
            }
            let c = o.pos.cell();
            let (fw, fh) = o.foundation;
            // Per axis: beyond the side the goal lies on, the middle when the goal is level with the building (and beyond the far side with no goal).
            let toward = |lo: i32, n: i32, g: Option<i32>| match g {
                Some(g) if g < lo => lo - 1 - RALLY_AHEAD,
                Some(g) if g < lo + n => lo + n / 2, // nocite: the middle cell of the foundation side
                _ => lo + n + RALLY_AHEAD,
            };
            let at = CellXY::new(
                toward(c.x, fw, goal.map(|g| g.x)),
                toward(c.y, fh, goal.map(|g| g.y)),
            );
            let click = targeting::click(
                w,
                self.house,
                &[id],
                targeting::ClickOn::Cell(at),
                targeting::Modifiers::default(),
            );
            out.extend(
                click
                    .orders
                    .into_iter()
                    .filter(|o| matches!(o, Order::GuardSpot { .. })),
            );
        }
    }

    /// Whether the house is selling out (see [`Script::think`]).
    pub(crate) fn selling(&self) -> bool {
        self.watch.selling
    }

    /// The orders of one think.
    pub fn think(&mut self, w: &World) -> Vec<Order> {
        let mut out = Vec::new();
        let house = self.house;
        let own = self.own(w);
        self.sell_when_broke(w, &mut out);
        let selling = self.watch.selling;

        // 1. Deploy the MCV.
        let has_yard = own
            .iter()
            .any(|&id| w.obj(id).is_some_and(|o| is_yard(w, o)));
        for &id in &own {
            let Some(o) = w.obj(id) else { continue };
            if o.kind == Kind::Vehicle
                && deploys_into_yard(w, o.kind, o.type_index)
                && !has_yard
                && idle(w, id)
            {
                out.push(Order::Deploy { actor: id });
            }
        }

        // 1b. A barracks without a rally point gets one, by a click on a
        // cell beyond it like a person's (S-0213 claim 20 (a)): without
        // one a released infantry unit is given no destination and stays
        // in contact at the barracks (S-0417 claim 10, S-0404 claim 13), so
        // the next unit's release finds no free place and is cancelled
        // (S-0404 claim 7, S-0417 claim 14).
        self.rally_points(w, &own, &mut out);

        // 2 and 3. Buildings: a done building is placed first; otherwise the
        // next wanted one is ordered.
        let done = w.econ.lines.lines.iter().find_map(|l| {
            let item = l.item.as_ref()?;
            (l.house == house && item.t.kind == Kind::Building && item.done_tick.is_some())
                .then_some(item.t)
        });
        if let Some(t) = done {
            // While the naval yard has no cell, other buildings go toward
            // the water.
            let toward = (!naval_building(w, t))
                .then(|| self.water_goal(w))
                .flatten()
                .and_then(|g| self.place_toward(w, t, g));
            if let Some(cell) = toward.or_else(|| self.place_cell(w, t)) {
                out.push(Order::Place {
                    what: t,
                    cell: Some(cell),
                });
            }
        } else if !selling {
            let p = &w.houses[usize::from(house.0)].econ.power;
            // The first missing role the house can build a type for now
            // (a role it cannot build yet, such as a refinery that needs a
            // factory first, does not hold up the others).
            let low_power = p.output <= p.drain || self.of_role(w, Role::Power).is_empty();
            let wanted = low_power
                .then_some(Role::Power)
                .into_iter()
                .chain((!self.has_refinery(w)).then_some(Role::Refinery))
                .chain(
                    FACTORIES
                        .into_iter()
                        .map(Role::Factory)
                        .filter(|&f| self.of_role(w, f).is_empty()),
                )
                .find(|&r| !self.candidates(w, r).is_empty());
            let walk = || {
                // The cheapest power plant with a cell toward the water.
                let goal = self.water_goal(w)?;
                self.candidates(w, Role::Power)
                    .into_iter()
                    .find(|&t| self.place_toward(w, t, goal).is_some())
            };
            if let Some(role) = wanted {
                self.build(w, role, &mut out);
            } else if self.harvesters_kept(w)
                && let Some(t) = walk().or_else(|| self.climb(w))
            {
                self.order_build(w, t, &mut out);
            }
        }

        // 4. Harvesters.
        let harvesters = self.of_role(w, Role::Harvester);
        let refineries = self.of_role(w, Role::Refinery);
        // Deployed slave miners count towards the harvesters kept.
        let economy = harvesters.len() + self.slave_buildings(w).len();
        for &h in &harvesters {
            let Some(o) = w.obj(h) else { continue };
            let resting = matches!(
                o.beh.current,
                None | Some(Behaviour::Guard) | Some(Behaviour::AreaGuard) | Some(Behaviour::Stop)
            );
            if is_slave_miner(w, tech::type_of(o)) {
                // A resting slave miner whose slaves are home is sent to
                // harvest at a field cell; it finds its site and deploys
                // there itself (S-0402 claim 22).
                if resting
                    && slaves::at_rest(w, h)
                    && let Some(c) = self.field_cell(w, o.pos.cell())
                {
                    out.push(Order::Behaviour {
                        actor: h,
                        behaviour: Behaviour::Harvest,
                        attack_move: false,
                        target: None,
                        dest: Some(Target::Cell(c)),
                    });
                }
            } else if !refineries.is_empty() && (resting || idle(w, h)) {
                // A harvester that rests (whatever destination it kept) or
                // is idle is sent to harvest.
                out.push(Order::Behaviour {
                    actor: h,
                    behaviour: Behaviour::Harvest,
                    attack_move: false,
                    target: None,
                    dest: None,
                });
            }
        }
        // A harvester is built once a refinery stands; a slave miner needs
        // none.
        let can_harvest = !refineries.is_empty()
            || self
                .candidates(w, Role::Harvester)
                .first()
                .is_some_and(|&t| is_slave_miner(w, t));
        if can_harvest && economy < self.plan.harvesters && !selling {
            self.build(w, Role::Harvester, &mut out);
        }

        // The army: one type round-robin per kind of factory, once the
        // harvesters are kept (while a harvester can still be had: a
        // harvester type to build now, or a vehicle factory to build while
        // the house has none).
        let vehicle_factory = Role::Factory(FactoryKind::Vehicle);
        let economy_first = !selling
            && economy < self.plan.harvesters
            && (!self.candidates(w, Role::Harvester).is_empty()
                || (self.of_role(w, vehicle_factory).is_empty()
                    && !self.candidates(w, vehicle_factory).is_empty()));
        // Lifters are built by their own rule (`lifter::build_type`), not
        // in the round of army types.
        let army_types: Vec<TypeRef> = if economy_first {
            Vec::new()
        } else {
            self.candidates(w, Role::Army)
                .into_iter()
                .filter(|&t| !crate::lifter::lifter_type(w, t))
                .collect()
        };
        // Of the types whose line is free (a naval yard's and a war
        // factory's vehicles do not wait on each other), the one the house
        // has fewest of on the map, then the one it has built fewest of
        // (S-0407 claim 10's ever-built count), then the first from the
        // kind's counter on. A plain turn through a list that grows with
        // the tech tree reaches its later types too seldom in a short game,
        // and the two vehicle lines' picks moved one shared counter, so
        // some types were never built.
        let own = self.own(w);
        let living = |t: TypeRef| {
            own.iter()
                .filter(|&&id| w.obj(id).is_some_and(|o| tech::type_of(o) == t))
                .count()
        };
        for (k, kind) in ARMY_KINDS.into_iter().enumerate() {
            let of_kind: Vec<TypeRef> = army_types
                .iter()
                .copied()
                .filter(|t| t.kind == kind)
                .collect();
            let n = of_kind.len();
            let pick = (0..n)
                .filter(|&i| !self.busy(w, of_kind[i]))
                .min_by_key(|&i| {
                    let t = of_kind[i];
                    let built = tech::ever_built(w, self.house, t);
                    (living(t), built, (i + n - self.next_army[k] % n) % n)
                });
            if let Some(i) = pick {
                out.push(Order::Build { what: of_kind[i] });
                self.next_army[k] = i + 1;
            }
        }

        // 5. Army units. An idle aircraft with ammunition left (on its pad,
        // rearmed) attacks the nearest seen enemy; it flies back to its pad
        // by itself (S-0501 claims 21 to 30).
        let (air, army): (Vec<ObjectId>, Vec<ObjectId>) = self
            .of_role(w, Role::Army)
            .into_iter()
            .partition(|&id| w.obj(id).is_some_and(|o| o.kind == Kind::Aircraft));
        let enemies = self.seen_enemies(w);
        for id in air
            .into_iter()
            .filter(|&id| idle(w, id) && glasnost_sim::air::ammo_left(w, id))
        {
            let Some(at) = w.obj(id).map(|o| o.pos.cell()) else {
                continue;
            };
            if let Some(&(e, _)) = enemies
                .iter()
                .filter(|&&(e, _)| can_attack(w, id, e))
                .min_by_key(|&&(_, c)| (dist2(c, at), c.y, c.x))
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
        let mut waiting: Vec<ObjectId> = Vec::new();
        let home = self.home(w);
        let ahead = self.base_goal(w);
        for id in army.iter().copied().filter(|&id| idle(w, id)) {
            let Some(at) = w.obj(id).map(|o| o.pos.cell()) else {
                continue;
            };
            let near = enemies
                .iter()
                .filter(|&&(e, c)| dist2(c, at) <= ENGAGE * ENGAGE && can_attack(w, id, e))
                .min_by_key(|&&(_, c)| dist2(c, at));
            if let Some(&(e, _)) = near {
                out.push(Order::Behaviour {
                    actor: id,
                    behaviour: Behaviour::Attack,
                    attack_move: false,
                    target: Some(Target::Object(e)),
                    dest: None,
                });
            } else if let (Some(h), Some(g)) = (home, ahead)
                && self.needs_ferry(w, id, g)
            {
                // Infantry that cannot walk to the goal waits at home for
                // a ferry (the brain's ferry takes idle infantry from
                // there) instead of standing at the nearest shore.
                if dist2(at, h) > FERRY_GATHER * FERRY_GATHER {
                    let mut near = BTreeMap::new();
                    let dest = self.reachable(w, id, h, &mut near);
                    out.push(Order::Behaviour {
                        actor: id,
                        behaviour: Behaviour::Move,
                        attack_move: false,
                        target: None,
                        dest: Some(Target::Cell(dest)),
                    });
                }
            } else {
                waiting.push(id);
            }
        }
        let since = *self.waiting_since.get_or_insert(w.tick);
        if waiting.is_empty() {
            self.waiting_since = None;
        }
        let patient = !waiting.is_empty() && w.tick - since >= WAVE_PATIENCE;
        if waiting.len() >= self.plan.wave.max(1) || patient {
            self.waiting_since = None;
            // Each unit goes to the seen enemy nearest home that it can
            // reach and attack (a person does not send riflemen to stand at
            // the shore by a ship), else all go to the next enemy start cell.
            let home = self.home(w);
            let from = w.obj(waiting[0]).map(|o| o.pos.cell());
            let mut by_home: Vec<(ObjectId, CellXY)> = enemies.clone();
            if let Some(h) = home {
                by_home.sort_by_key(|&(e, c)| (dist2(c, h), e));
            }
            let mut picks: BTreeMap<(Kind, u32, Option<u32>), Option<CellXY>> = BTreeMap::new();
            let mut fallback: Option<Option<CellXY>> = None;
            // The nearest region cells, kept per goal (`reachable`).
            let mut near: BTreeMap<CellXY, BTreeMap<(u8, u32), CellXY>> = BTreeMap::new();
            for &id in &waiting {
                let Some(o) = w.obj(id) else { continue };
                let region = self.reach_region(w, id);
                let key = (o.kind, o.type_index, region);
                let pick = *picks.entry(key).or_insert_with(|| {
                    by_home
                        .iter()
                        .find(|&&(e, c)| self.reaches(w, id, region, c) && can_attack(w, id, e))
                        .map(|&(_, c)| c)
                });
                let goal = pick.or_else(|| {
                    *fallback.get_or_insert_with(|| {
                        (self.next_target < self.targets.len())
                            .then(|| {
                                let c = self.targets[self.next_target];
                                self.next_target += 1;
                                c
                            })
                            .or_else(|| from.and_then(|f| self.unseen_cell(w, f)))
                            .or_else(|| {
                                (!self.targets.is_empty()).then(|| {
                                    let c = self.targets[self.next_target % self.targets.len()];
                                    self.next_target += 1;
                                    c
                                })
                            })
                    })
                });
                if let Some(c) = goal {
                    let dest = self.reachable(w, id, c, near.entry(c).or_default());
                    out.push(Order::Behaviour {
                        actor: id,
                        behaviour: Behaviour::Move,
                        attack_move: true,
                        target: None,
                        dest: Some(Target::Cell(dest)),
                    });
                }
            }
        }
        out
    }

    /// The reach region unit `id` stands in for its movement zone (S-0212
    /// claim 4); `None` for a unit that flies (S-0502), has no zone or
    /// stands outside every region.
    fn reach_region(&self, w: &World, id: ObjectId) -> Option<u32> {
        let o = w.obj(id)?;
        if crate::jumpjets::flies(w, id) {
            return None;
        }
        let zone = w.rules().unit(o.kind, o.type_index)?.zone;
        match regions::reach_in(w, o.pos.cell(), zone) {
            Reach::Region(r) => Some(r),
            _ => None,
        }
    }

    /// Whether unit `id`, in reach region `region` ([`Script::reach_region`]),
    /// goes for an enemy at cell `c`: `c` lies in that region, or the unit
    /// has none, or it is a ship (it fires from the water nearest the
    /// enemy, as a person sends ships to the water off an enemy base).
    fn reaches(&self, w: &World, id: ObjectId, region: Option<u32>, c: CellXY) -> bool {
        let Some(r) = region else { return true };
        let Some(zone) = w
            .obj(id)
            .and_then(|o| w.rules().unit(o.kind, o.type_index))
            .map(|u| u.zone)
        else {
            return true;
        };
        zone == glasnost_rules::sim::MovementZone::Water
            || regions::reach_in(w, c, zone) == Reach::Region(r)
    }

    /// Whether unit `id` is infantry that cannot walk to `goal`: it stands
    /// in a reach region of its movement zone (S-0212 claim 4) that does
    /// not hold the goal, and it does not fly (S-0502).
    pub(crate) fn needs_ferry(&self, w: &World, id: ObjectId, goal: CellXY) -> bool {
        let Some(o) = w.obj(id) else { return false };
        if o.kind != Kind::Infantry || crate::jumpjets::flies(w, id) {
            return false;
        }
        let Some(zone) = w.rules().unit(o.kind, o.type_index).map(|u| u.zone) else {
            return false;
        };
        let from = regions::reach_in(w, o.pos.cell(), zone);
        matches!(from, Reach::Region(_)) && regions::reach_in(w, goal, zone) != from
    }

    /// Where unit `id` goes for goal `c`: the goal itself when it lies in
    /// the unit's reach region (S-0212 claim 4), else the nearest cell of
    /// that region to the goal, the first in cell order on equal distance,
    /// as a person sends ships to the water off an enemy base. `near`
    /// keeps the answers of one wave by zone and region.
    fn reachable(
        &self,
        w: &World,
        id: ObjectId,
        c: CellXY,
        near: &mut BTreeMap<(u8, u32), CellXY>,
    ) -> CellXY {
        let Some(o) = w.obj(id) else { return c };
        // A jumpjet flies over what stops the ground (S-0502): its goal needs no reach region.
        if crate::jumpjets::flies(w, id) {
            return c;
        }
        let Some(zone) = w.rules().unit(o.kind, o.type_index).map(|u| u.zone) else {
            return c;
        };
        let Reach::Region(r) = regions::reach_in(w, o.pos.cell(), zone) else {
            return c;
        };
        if regions::reach_in(w, c, zone) == Reach::Region(r) {
            return c;
        }
        *near.entry((zone as u8, r)).or_insert_with(|| {
            w.cells
                .grid
                .iter()
                .map(|(k, _)| k)
                .filter(|&k| w.cells.usable(k) && regions::reach_in(w, k, zone) == Reach::Region(r))
                .min_by_key(|&k| (dist2(k, c), k.y, k.x))
                .unwrap_or(c)
        })
    }

    /// The nearest cell of the search grid, from `from`, that is in the
    /// usable area and that the house does not see now; the first in cell
    /// order on equal distance.
    fn unseen_cell(&self, w: &World, from: CellXY) -> Option<CellXY> {
        w.cells
            .grid
            .iter()
            .map(|(c, _)| c)
            .filter(|c| c.x % SEARCH_STEP == 0 && c.y % SEARCH_STEP == 0)
            .filter(|&c| w.cells.usable(c) && !shroud::seen(w, self.house, c))
            .min_by_key(|&c| (dist2(c, from), c.y, c.x))
    }

    /// Enemy objects on cells the house has explored, with their cells: fog
    /// of war is off in a skirmish (S-0409 claim 19), so a person in the
    /// house's seat sees what stands on every explored cell, except a
    /// cloaked object the house does not sense (S-0516 claim 28).
    pub(crate) fn seen_enemies(&self, w: &World) -> Vec<(ObjectId, CellXY)> {
        let me = &w.houses[usize::from(self.house.0)];
        w.objects
            .values()
            .filter(|o| o.on_map && o.health > 0)
            .filter_map(|o| {
                let owner = o.owner?;
                let h = w.houses.get(usize::from(owner.0))?;
                let enemy = owner != self.house
                    && !me.allies.contains(&owner)
                    && h.role == glasnost_sim::world::HouseRole::Player;
                let c = o.pos.cell();
                // A cloaked enemy the house does not sense is not seen (S-0516 claim 28: not drawn for it).
                let hidden = glasnost_sim::cloak::hidden_from(w, o.id, Some(self.house));
                (enemy && !hidden && shroud::explored(w, self.house, c)).then_some((o.id, c))
            })
            .collect()
    }

    /// The first cell of a square spiral around the construction yard where
    /// a place order for `t` is legal, the cell test passes and the
    /// foundation is free. A naval building is looked for in the wider
    /// spiral, as a person looks for the water near the base.
    pub fn place_cell(&self, w: &World, t: TypeRef) -> Option<CellXY> {
        let rings = if naval_building(w, t) {
            WIDE_RINGS
        } else {
            SPIRAL_RINGS
        };
        // A cramped base (a start in a small clearing) has no cell with the
        // usual gap: the gap narrows, as a person squeezes a building in
        // when the room is short, down to none.
        (0..=GAP)
            .rev()
            .find_map(|gap| self.spiral(w, rings).find(|&c| self.fits_gap(w, t, c, gap)))
    }

    /// The cells of the square spiral around the construction yard, ring by
    /// ring up to `rings`.
    fn spiral(&self, w: &World, rings: i32) -> impl Iterator<Item = CellXY> {
        let home = self.home(w);
        (1..=rings).flat_map(move |ring| {
            (-ring..=ring).flat_map(move |dy| {
                (-ring..=ring)
                    .filter(move |dx| dx.abs() == ring || dy.abs() == ring)
                    .filter_map(move |dx| home.map(|h| CellXY::new(h.x + dx, h.y + dy)))
            })
        })
    }

    /// Whether a place order for `t` at `c` is legal, the cell test passes
    /// and the foundation is free.
    fn fits(&self, w: &World, t: TypeRef, c: CellXY) -> bool {
        self.fits_gap(w, t, c, GAP)
    }

    /// [`Script::fits`] with a gap of `gap` cells to other buildings.
    fn fits_gap(&self, w: &World, t: TypeRef, c: CellXY, gap: i32) -> bool {
        // The place order's conditions and the cell test (S-0406 claims 3 to 6, 8 and 12); an upgrade needs only the former.
        let upgrade = placement::place_type(w, t.index).upgrade_of.is_some();
        placement::place_legal(w, self.house, t, c)
            && (upgrade || placement::cell_test(w, t.index, c, &placement::Test::order(self.house)))
            && self.free_gap(w, t, c, None, gap)
    }

    /// The water the base grows toward: when a naval factory type that the
    /// house has none of is buildable but has no cell to stand on, the
    /// explored water cell nearest the construction yard (the first in cell
    /// order on equal distance).
    pub(crate) fn water_goal(&self, w: &World) -> Option<CellXY> {
        let home = self.home(w)?;
        let stuck = w.rules().buildings.iter().enumerate().any(|(i, b)| {
            let t = TypeRef {
                kind: Kind::Building,
                index: i as u32,
            };
            b.naval
                && b.factory.is_some()
                && tech::present_count(w, self.house, t) == 0
                && tech::buildable(w, self.house, t)
                && !self.placeable(w, t)
        });
        if !stuck {
            return None;
        }
        w.cells
            .grid
            .iter()
            .filter(|(c, cell)| {
                cell.land == glasnost_map::terrain::land::WATER
                    && shroud::explored(w, self.house, *c)
            })
            .map(|(c, _)| c)
            .min_by_key(|&c| (dist2(c, home), c.y, c.x))
    }

    /// The legal cell for `t` in the wider spiral nearest to `goal`, when it
    /// is nearer than every building of the house: a step of the base
    /// toward the water.
    fn place_toward(&self, w: &World, t: TypeRef, goal: CellXY) -> Option<CellXY> {
        let reach = self
            .own(w)
            .iter()
            .filter_map(|&id| w.obj(id))
            .filter(|o| o.kind == Kind::Building)
            .map(|o| dist2(o.pos.cell(), goal))
            .min()?;
        self.spiral(w, WIDE_RINGS)
            .filter(|&c| dist2(c, goal) < reach)
            .filter(|&c| self.fits(w, t, c))
            .min_by_key(|&c| (dist2(c, goal), c.y, c.x))
    }

    /// Whether the foundation's cells hold no object and keep a gap of
    /// [`GAP`] cells to other buildings, so that units can leave the
    /// factories and reach the docks.
    pub(crate) fn free(&self, w: &World, t: TypeRef, c: CellXY) -> bool {
        self.free_but(w, t, c, None)
    }

    /// [`Script::free`], not counting object `but`.
    pub(crate) fn free_but(&self, w: &World, t: TypeRef, c: CellXY, but: Option<ObjectId>) -> bool {
        self.free_gap(w, t, c, but, GAP)
    }

    /// [`Script::free_but`] with a gap of `gap` cells.
    fn free_gap(&self, w: &World, t: TypeRef, c: CellXY, but: Option<ObjectId>, gap: i32) -> bool {
        let cells = placement::foundation_cells(w, t.index, c);
        // Only what the house knows: its own objects and those on cells it
        // has explored (the AI sees only its own house's shroud; fog is off
        // in a skirmish, S-0409 claim 19).
        w.objects
            .values()
            .filter(|o| {
                o.on_map
                    && Some(o.id) != but
                    && (o.owner == Some(self.house)
                        || shroud::explored(w, self.house, o.pos.cell()))
            })
            .all(|o| {
                // A building's position is its top-left cell's centre.
                let tl = o.pos.cell();
                if o.kind == Kind::Building {
                    let (fw, fh) = o.foundation;
                    cells.iter().all(|k| {
                        k.x < tl.x - gap
                            || k.x > tl.x + fw - 1 + gap
                            || k.y < tl.y - gap
                            || k.y > tl.y + fh - 1 + gap
                    })
                } else {
                    !cells.contains(&tl)
                }
            })
    }

    /// A diagnostic of the house's plan, counts only (local-lane reports):
    /// whether it keeps its harvesters, for every role it lacks the cells
    /// that the placement spiral turns down by stage, the enemies it has
    /// seen and whether its army reaches them by the movement zone.
    pub fn diagnose(&self, w: &World) -> String {
        use std::fmt::Write as _;
        let mut s = String::new();
        let _ = write!(
            s,
            "selling {}, harvesters {}/{} (slave buildings {}), waiting since {:?}",
            self.watch.selling,
            self.of_role(w, Role::Harvester).len(),
            self.plan.harvesters,
            self.slave_buildings(w).len(),
            self.waiting_since
        );
        let wanted = [Role::Power, Role::Refinery]
            .into_iter()
            .chain(FACTORIES.into_iter().map(Role::Factory));
        for role in wanted {
            let have = self.of_role(w, role).len();
            let cands = self.candidates(w, role);
            let _ = write!(
                s,
                "; {role:?}: have {have}, buildable types {}",
                cands.len()
            );
            if have == 0
                && let Some(&t) = cands.first()
            {
                let (mut cells, mut legal, mut test, mut free, mut bare) = (0, 0, 0, 0, 0);
                for c in self.spiral(w, SPIRAL_RINGS) {
                    cells += 1;
                    let ok = placement::place_legal(w, self.house, t, c);
                    legal += usize::from(ok);
                    let t2 = ok
                        && placement::cell_test(w, t.index, c, &placement::Test::order(self.house));
                    test += usize::from(t2);
                    free += usize::from(t2 && self.free(w, t, c));
                    bare += usize::from(t2 && self.free_gap(w, t, c, None, 0));
                }
                let _ = write!(
                    s,
                    " (first type: of {cells} spiral cells {legal} legal, {test} pass the cell test, {free} also keep the gap, {bare} are free of objects)"
                );
            }
        }
        let enemies = self.seen_enemies(w);
        let ebuild = enemies
            .iter()
            .filter(|&&(e, _)| w.obj(e).is_some_and(|o| o.kind == Kind::Building))
            .count();
        let _ = write!(
            s,
            "; seen enemies {} (buildings {ebuild}); army {}",
            enemies.len(),
            self.of_role(w, Role::Army).len()
        );
        let mut reach: BTreeMap<String, (usize, usize)> = BTreeMap::new();
        if let Some(home) = self.home(w) {
            let mut probe = w.clone();
            regions::ensure(&mut probe);
            if let Some(&(_, goal)) = enemies.iter().min_by_key(|&&(_, c)| dist2(c, home)) {
                for id in self.of_role(w, Role::Army) {
                    let Some(o) = w.obj(id) else { continue };
                    let Some(zone) = w.rules().unit(o.kind, o.type_index).map(|u| u.zone) else {
                        continue;
                    };
                    let mine = regions::reach_in(&probe, o.pos.cell(), zone);
                    let e = reach.entry(format!("{zone:?}")).or_default();
                    e.0 += 1;
                    e.1 += usize::from(
                        matches!(mine, Reach::Region(_))
                            && mine == regions::reach_in(&probe, goal, zone),
                    );
                }
            }
        }
        let _ = write!(
            s,
            "; army reaching the nearest seen enemy, by zone (units, reaching): {reach:?}"
        );
        s
    }
}
