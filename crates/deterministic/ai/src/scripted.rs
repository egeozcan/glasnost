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
//!    to harvest, and harvesters stalled in *unload* with no refinery
//!    contact ([`stalled_unload`]); a slave miner counts as a harvester (and, deployed, its
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
//!    A smaller wave (at least half the size) goes too once units have
//!    waited long enough, so a house that cannot afford a full wave still
//!    attacks. No wave goes before the opening is over, a guard (a share of
//!    the army, the waiting units nearest home) stays home, each wave asks
//!    for more units than the last, a wave takes on the enemy whose
//!    building stands nearest home first, and while it sees no enemy it
//!    goes to the nearest start that may still hold an enemy base. The
//!    fastest waiting unit scouts the house's own side of the map (the
//!    unexplored cell nearest home, nearer home than any enemy start), one
//!    at a time. A unit whose goal lies
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

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use glasnost_map::CellXY;
use glasnost_rules::sim::MovementZone;
use glasnost_sim::combat::{check, range, weapon};
use glasnost_sim::econ::TypeRef;
use glasnost_sim::econ::{
    deploy, harvest, placement, production, resource, sell, shroud, slaves, tech,
};
use glasnost_sim::orders::Order;
use glasnost_sim::probe::{Phase, Probe};
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

/// The tick within each [`THINK_PERIOD`] at which a house's brain thinks,
/// by house number (houses beyond the table share its phases in turn).
/// The brains of a game think in different ticks, so that no tick runs more
/// than one think in a game of up to eight AI houses (M8 step B7: with every
/// brain in the same tick, each think tick of an eight-house game took a
/// median 57 ms, against a 16.7 ms frame). House 0 thinks where every house
/// thought before. The phases leave out 4, 9 and 14, the ticks of the
/// period on which the vehicle movement's cost peaked in the same game (M8
/// C4 part 2), and spread a duel's two houses half a period apart. A phase
/// is a function of the house number alone, so it is the same on every
/// peer, needs no saving, and stays put when a house leaves the game.
const THINK_PHASES: [i32; 8] = [0, 7, 11, 3, 13, 5, 10, 2]; // nocite: our own AI's schedule (M8 step B7)

/// The phase of `house`'s thinks: it thinks in the ticks `t` with
/// `t ≡ phase (mod THINK_PERIOD)`.
pub fn think_phase(house: HouseId) -> i32 {
    THINK_PHASES[usize::from(house.0) % THINK_PHASES.len()]
}

/// Whether `house`'s brain thinks in tick `tick` ([`think_phase`]).
pub fn thinks_at(house: HouseId, tick: i32) -> bool {
    (tick - think_phase(house)).rem_euclid(THINK_PERIOD) == 0
}

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

/// Ticks a done building waits on its line for a cell before the house
/// cancels it (design: own AI, #40).
const PLACE_PATIENCE: i32 = 900; // nocite: tuning of our own opponent

/// The tick before which no wave goes: the house builds up and defends
/// first (M8 step A4 round 8, the lead's note 52; in the quick lane the
/// first pushes came from about tick 6600 before, so this stops only a rush
/// in the first minutes).
pub const OPENING: i32 = 4500; // nocite: tuning of our own opponent

/// The share of the house's army (one in this many) that stays home as its
/// guard when a wave goes.
const GUARD_SHARE: usize = 3; // nocite: tuning of our own opponent
/// The fewest units of the guard.
pub const GUARD_MIN: usize = 2; // nocite: tuning of our own opponent
/// The most units of the guard: a large army does not keep a third of
/// itself at home (in the quick lane an uncapped guard left one game of 36
/// without a winner after 120000 ticks).
const GUARD_MAX: usize = 10; // nocite: tuning of our own opponent

/// How many more units each wave the house has sent asks for, and the
/// largest wave asked for.
const WAVE_GROWTH: usize = 2; // nocite: tuning of our own opponent
const WAVE_MAX: usize = 14; // nocite: tuning of our own opponent

/// How many times the worth of what the house sees of the enemy that fights
/// (its army units and armed buildings) the house's own army is worth at
/// least for the house to be clearly winning ([`Script::finishing`]).
/// design: an enemy whose seen fighting worth is a third of the house's
/// army cannot stop it; a person stops waiting for bigger waves, and sends
/// what they have once nothing of the enemy builds (M8 step A4 round 9b).
const FINISH_SHARE: i64 = 3; // nocite: tuning of our own opponent

/// Resource cells around a refinery, slave miner or home that keep one
/// harvester at work ([`Script::harvesters_wanted`]).
const CELLS_PER_HARVESTER: usize = 20; // nocite: tuning of our own opponent

/// How far around home, in cells, [`Script::harvester_reach`] looks for
/// where a harvester would leave the base when the house has none.
const HOME_RING: i32 = 6; // nocite: tuning of our own opponent

/// How near a start cell an ally's building stands for that start to be
/// the ally's base ([`Script::ally_base_at`]), in cells.
const ALLY_BASE: i32 = 10; // nocite: tuning of our own opponent

/// Cells around a refinery, slave miner or home within which resource
/// counts for [`Script::harvesters_wanted`].
pub(crate) const FIELD_REACH: i32 = 14; // nocite: tuning of our own opponent

/// Empty cells between two resource cells of one field
/// ([`Script::far_field`]).
const FIELD_GAP: i32 = 2; // nocite: tuning of our own opponent

/// What the far-field search of [`Script::far_field`] sees: the explored
/// resource cells, those of fields the house works, those of unworked
/// fields on its own half of the map, and the far field's cell.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Fields {
    pub explored: usize,
    pub worked: usize,
    pub own_half: usize,
    pub far: Option<CellXY>,
}

/// Spacing of the search grid that waves sweep once every enemy start
/// position has been visited.
const SEARCH_STEP: i32 = 8; // nocite: tuning of our own opponent

/// Ticks without income after which a house that cannot afford an army
/// unit counts as broke and sells a building.
const BROKE_PATIENCE: i32 = 1500; // nocite: tuning of our own opponent

/// Ticks after a sale within which a rise of credits is taken as its refund,
/// not as income.
const SALE_WINDOW: i32 = 300; // nocite: tuning of our own opponent

/// The ticks over which the economy watch sums the house's income
/// ([`Script::income`]): every so many ticks it reads the credits earned
/// since the last reading, as a person reads his income off the credits
/// counter over the last few minutes.
pub const INCOME_WINDOW: i32 = 3000; // nocite: tuning of our own opponent

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

/// Whether `t` is a building type whose objects keep slaves (S-0402
/// claim 19: its type names a slave type), such as a slave miner's
/// deployed form.
pub(crate) fn keeps_slaves(w: &World, t: TypeRef) -> bool {
    t.kind == Kind::Building
        && w.rules()
            .econ
            .harvest
            .buildings
            .get(t.index as usize)
            .is_some_and(|b| b.group.is_some())
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
            // design: a move toward an object that is no longer in play
            // goes nowhere, and a person sees the unit standing and gives
            // it an order (M8 step A4 round 9: a drainer bound for a
            // building destroyed under it stood for the rest of the game)
            Some(Target::Object(k)) => o.beh.current == Some(Behaviour::Move) && w.obj(k).is_none(),
        };
        // A deployed vehicle or one changing form takes no destination (S-0517 claim 9), so it is not free for a wave.
        // A bunkered vehicle refuses a destination (S-0606; `bunker` lets it out first).
        // Deployed infantry drops a destination (S-0215 claim 22; `deployers` packs it up first).
        arrived
            && !glasnost_sim::structures::bunker::bunkered(w, id)
            && !crate::deployers::infantry_deployed(w, id)
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

/// How many ticks a harvester's *unload* must have lasted before
/// [`stalled_unload`] counts it as stalled.
// design: (M8 AI stalled harvester) a stalled harvester stays in *unload*
// for good (S-0402 claim 11), so the wait only has to pass the few ticks
// between a lost contact and the next processing of *unload*, in which a
// harvester whose contact ended otherwise (or under the play-fixes layer)
// moves on by itself. The clock a person reads is the harvester standing
// still where no refinery is: ten think periods is about what a person
// takes to notice it and click it, and costs a house one short pause of
// one harvester's income, not the tens of thousands of ticks the quick
// lane's stalled harvester stood (intake 113 ledger). The behaviour's start tick is the
// only clock the harvester shows, so the wait counts from the start of
// *unload*, the docking, which is never later than the lost contact.
const UNLOAD_STALL: i32 = 150; // nocite: tuning of our own opponent

/// Whether harvester `id` stands stalled in *unload*: *unload* current for
/// more than [`UNLOAD_STALL`] ticks, out of the unloading state, with no
/// contact, no destination and nothing queued. That is S-0402 claim 11's
/// *Contact lost otherwise while unloading* (its refinery destroyed under
/// it, `quirks.unload_without_contact_stalls`): the harvester keeps its
/// load and stays put until an order gives it a behaviour. A person sees
/// a harvester standing still with no refinery beside it and sends it to
/// harvest; so does the script. A harvester that really unloads holds its
/// refinery's contact (every other row of claim 11 needs one), and an
/// ordered one has a destination or a queued behaviour, so neither counts.
pub fn stalled_unload(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        // spec: S-0402 claim 11 (row 1, not in contact with anything, and *Contact lost otherwise while unloading*: nothing queued, it stays in unload, out of the unloading state, processed every tick; quirks.unload_without_contact_stalls)
        o.beh.current == Some(Behaviour::Unload)
            && o.contact.is_none()
            && o.dest.is_none()
            && o.beh.queued.is_none()
            && !harvest::in_unloading_state(w, id)
            && w.tick - o.beh.start_tick > UNLOAD_STALL
    })
}

/// Whether a unit resting in guard holds a target it cannot shoot at: guard
/// keeps an acquired target while it is out of the weapon's range, and the
/// unit stays where it is (guard never pursues, S-0215 claim 24). A person
/// sees such a unit standing still and gives it an order; so does the
/// script, which would otherwise leave a home guard idle while the enemy
/// destroys the base. Whether the target is stale is asked whatever the
/// moment (M8 AI seams): a unit that is rearming would otherwise answer
/// *rearming* ahead of the target tests and the range test, and keep a
/// target its weapon may not hurt, or one out of its range.
fn stale_guard_target(w: &World, o: &Object) -> bool {
    let resting = matches!(
        o.beh.current,
        Some(Behaviour::Guard) | Some(Behaviour::AreaGuard) | Some(Behaviour::Stop)
    );
    let Some(t) = o.target else { return false };
    if !resting {
        return false;
    }
    let t = Some(t);
    let (slot, wp) = weapon::would_use(w, o.id, t);
    // spec: S-0301 claim 9 (the own AI's lasting test, our design, PLAN §4.7: the firing check without the range test and without the reasons of the moment of ranks 5 to 8, so that rank 5's rearming hides neither rank 9's target tests, a seeker's out-of-range answer among them, nor the range answer, which is tested on its own below)
    wp.is_none()
        || matches!(
            check::check_lasting(w, o.id, t, slot),
            check::Answer::OutOfRange | check::Answer::Illegal | check::Answer::Impossible
        )
        // spec: S-0307 claims 1 to 11 (the range test of the weapon it would use)
        || !range::in_range(w, o.id, wp, t, None)
}

/// Whether unit `id` can attack enemy `e` at all: the weapon it would use
/// against it (S-0308 claim 3) passes the firing check without the range
/// test, short of an illegal target or an impossible shot (S-0301 claim
/// 9). An aircraft out of a ground weapon's reach is not worth a unit's
/// attention. Reloading does not count: a unit whose countdown runs is
/// asked as if it had none, so that the rearming answer does not hide an
/// illegal target. This lasting test is the own AI's (our design, PLAN
/// §4.7); the attack cursor of S-0214 claim 6 (e) (r78) counts the reasons
/// of the moment ranked above the target tests instead.
pub(crate) fn can_attack(w: &World, id: ObjectId, e: ObjectId) -> bool {
    let t = Some(Target::Object(e));
    let (slot, weapon) = weapon::would_use(w, id, t);
    // spec: S-0301 claim 9 (the check without the reasons of the moment of ranks 5 to 8: the own AI's lasting test, our design, PLAN §4.7)
    weapon.is_some()
        && !matches!(
            check::check_lasting(w, id, t, slot),
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

/// An object the free-gap test looks at ([`Script::known_objects`]).
#[derive(Clone, Debug)]
struct Known {
    /// The object.
    id: ObjectId,
    /// Its position's cell (a building's top-left cell).
    top_left: CellXY,
    /// A building's foundation; none for another object.
    foundation: Option<(i32, i32)>,
}

/// [`free_among`]'s answers over the map's coordinate square, for every gap
/// up to [`GAP`] and no object left out (design: own AI, #52 and #57: the
/// placement searches asked the free-gap test of every known object for
/// every cell they tried). A building blocks cell `k` at gap `g` exactly
/// when `k` lies within its foundation's rectangle widened by `g` cells on
/// each side, that is when the larger of `k`'s distances outside the
/// rectangle across and down is at most `g`; any other object blocks the
/// cell it stands on.
#[derive(Clone, Debug)]
struct Blocked {
    /// The coordinate square's side.
    side: i32,
    /// Per cell (row-major), the smallest such distance to a known
    /// building, capped at `GAP + 1`.
    near: Vec<u8>,
    /// Per cell, whether a known object other than a building stands on it.
    occupied: Vec<bool>,
}

impl Blocked {
    /// The map of `known` on a coordinate square of `len` cells.
    fn new(known: &[Known], len: usize) -> Self {
        let side = len.isqrt();
        let cap = u8::try_from(GAP + 1).unwrap_or(u8::MAX);
        let mut b = Self {
            side: i32::try_from(side).unwrap_or(0),
            near: vec![cap; side * side],
            occupied: vec![false; side * side],
        };
        for o in known {
            let tl = o.top_left;
            match o.foundation {
                Some((fw, fh)) => {
                    let (x0, x1, y0, y1) = (tl.x, tl.x + fw - 1, tl.y, tl.y + fh - 1);
                    for y in (y0 - GAP)..=(y1 + GAP) {
                        for x in (x0 - GAP)..=(x1 + GAP) {
                            let Some(i) = b.index(CellXY::new(x, y)) else {
                                continue;
                            };
                            let d = (x0 - x).max(x - x1).max(y0 - y).max(y - y1).max(0);
                            let d = u8::try_from(d).unwrap_or(cap).min(cap);
                            b.near[i] = b.near[i].min(d);
                        }
                    }
                }
                None => {
                    if let Some(i) = b.index(tl) {
                        b.occupied[i] = true;
                    }
                }
            }
        }
        b
    }

    /// The index of `c` in the square, if it lies there.
    fn index(&self, c: CellXY) -> Option<usize> {
        let ok = (0..self.side).contains(&c.x) && (0..self.side).contains(&c.y);
        ok.then(|| (c.y * self.side + c.x) as usize)
    }

    /// [`free_among`] with no object left out, when every cell lies in the
    /// square; none when one does not (the caller asks [`free_among`]).
    fn free(&self, cells: &[CellXY], gap: i32) -> Option<bool> {
        if !(0..=GAP).contains(&gap) {
            return None;
        }
        let mut free = true;
        for &k in cells {
            let i = self.index(k)?;
            free &= !self.occupied[i] && i32::from(self.near[i]) > gap;
        }
        Some(free)
    }
}

/// Whether foundation cells `cells` hold none of `known` but object `but`
/// and keep a gap of `gap` cells to each of its buildings
/// ([`Script::free_gap`]).
fn free_among(known: &[Known], cells: &[CellXY], gap: i32, but: Option<ObjectId>) -> bool {
    known.iter().filter(|o| Some(o.id) != but).all(|o| {
        let tl = o.top_left;
        match o.foundation {
            Some((fw, fh)) => cells.iter().all(|k| {
                k.x < tl.x - gap
                    || k.x > tl.x + fw - 1 + gap
                    || k.y < tl.y - gap
                    || k.y > tl.y + fh - 1 + gap
            }),
            None => !cells.contains(&tl),
        }
    })
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
    /// The waves the house has sent (each asks for [`WAVE_GROWTH`] more
    /// units than the last, M8 step A4 round 8).
    #[serde(default)]
    waves: usize,
    /// The army unit sent to explore the house's own side of the map
    /// ([`Script::scout_cell`], M8 step A4 round 8).
    #[serde(default)]
    scout: Option<ObjectId>,
    /// The economy watch: credits and the harvesters' summed loads at the
    /// last think, the tick since which neither showed income, the last
    /// sale's tick, and whether the house is selling out (it then builds no
    /// more buildings or harvesters, only army units).
    watch: Watch,
    /// The house's home as of its last think ([`Script::home`]): kept, so
    /// that it stays where the house's construction yard stood (design:
    /// own AI, #12).
    #[serde(default)]
    home_cell: Option<CellXY>,
    /// The placement cells and clearing buildings found during the current
    /// think (derived data: never saved, hashed or compared).
    #[serde(skip)]
    placed: PlaceMemo,
}

/// [`Script::place_cell`]'s, [`Script::unblocker`]'s and
/// [`Script::buildable`]'s answers during one think, and the lists of
/// objects every part of the brain asks for ([`Script::own`],
/// [`Script::seen_enemies`], [`Script::known_objects`], [`Script::home`];
/// own AI issues batch 2). A think here is one call of the brain, on a
/// think tick or between thinks. The world does not change while a brain thinks (it reads
/// `&World`), and the answers depend only on the world and the house, so
/// an answer found once holds for the rest of that think. Outside a think
/// (between [`Script::begin_think`] and [`Script::end_think`]) nothing is
/// kept, and every call searches. Our own tooling for speed (M8 C4 part
/// 2): the same orders, found once instead of many times.
#[derive(Clone, Debug, Default)]
struct PlaceMemo(std::cell::RefCell<Option<Kept>>);

/// The answers [`PlaceMemo`] keeps.
#[derive(Clone, Debug, Default)]
struct Kept {
    /// [`Script::place_cell`] by type.
    cells: BTreeMap<TypeRef, Option<CellXY>>,
    /// [`Script::unblocker`] by type and depth.
    unblockers: BTreeMap<(TypeRef, u32), Option<TypeRef>>,
    /// [`Script::buildable`] by type.
    buildable: BTreeMap<TypeRef, bool>,
    /// [`Script::at_limit`] by type.
    limited: BTreeMap<TypeRef, bool>,
    /// The tech-climb building the script ordered this think
    /// ([`Script::climbed`]).
    climbed: Option<TypeRef>,
    /// [`Script::own`] (design: own AI, #53 and #59: one object scan a
    /// think instead of one per question).
    own: Option<Arc<[ObjectId]>>,
    /// [`Script::seen_enemies`] (design: own AI, #53 and #59).
    seen: Option<Arc<[(ObjectId, CellXY)]>>,
    /// [`Script::known_objects`] with no object left out (design: own AI,
    /// #52 and #57: one object scan a think instead of one per cell
    /// tested).
    known: Option<Arc<[Known]>>,
    /// The free-gap test's map of [`Script::known_objects`] (design: own
    /// AI, #52 and #57).
    blocked: Option<Arc<Blocked>>,
    /// [`Script::home`] (design: own AI, #59).
    home: Option<Option<CellXY>>,
    /// [`Script::of_role`] by role (design: own AI, #53 and #59).
    roles: BTreeMap<Role, Arc<[ObjectId]>>,
    /// [`Script::candidates`] by role (design: own AI, #59 and #60).
    candidates: BTreeMap<Role, Arc<[TypeRef]>>,
}

impl PlaceMemo {
    fn get(&self, t: TypeRef) -> Option<Option<CellXY>> {
        self.0
            .borrow()
            .as_ref()
            .and_then(|m| m.cells.get(&t).copied())
    }

    fn put(&self, t: TypeRef, found: Option<CellXY>) {
        if let Some(m) = self.0.borrow_mut().as_mut() {
            m.cells.insert(t, found);
        }
    }

    fn unblocker(&self, t: TypeRef, depth: u32) -> Option<Option<TypeRef>> {
        self.0
            .borrow()
            .as_ref()
            .and_then(|m| m.unblockers.get(&(t, depth)).copied())
    }

    fn put_unblocker(&self, t: TypeRef, depth: u32, found: Option<TypeRef>) {
        if let Some(m) = self.0.borrow_mut().as_mut() {
            m.unblockers.insert((t, depth), found);
        }
    }

    fn buildable(&self, t: TypeRef) -> Option<bool> {
        self.0
            .borrow()
            .as_ref()
            .and_then(|m| m.buildable.get(&t).copied())
    }

    fn put_buildable(&self, t: TypeRef, yes: bool) {
        if let Some(m) = self.0.borrow_mut().as_mut() {
            m.buildable.insert(t, yes);
        }
    }

    fn limited(&self, t: TypeRef) -> Option<bool> {
        self.0
            .borrow()
            .as_ref()
            .and_then(|m| m.limited.get(&t).copied())
    }

    fn put_limited(&self, t: TypeRef, yes: bool) {
        if let Some(m) = self.0.borrow_mut().as_mut() {
            m.limited.insert(t, yes);
        }
    }

    fn set_climbed(&self, t: TypeRef) {
        if let Some(m) = self.0.borrow_mut().as_mut() {
            m.climbed = Some(t);
        }
    }

    /// Whether a think runs (answers are kept).
    fn thinking(&self) -> bool {
        self.0.borrow().is_some()
    }

    /// The answer `field` keeps for the current think, else `make`'s, kept
    /// for the rest of the think when one runs. No borrow is held while
    /// `make` runs, so it may ask the memo itself.
    fn keep<T: Clone>(
        &self,
        field: fn(&mut Kept) -> &mut Option<T>,
        make: impl FnOnce() -> T,
    ) -> T {
        let kept = self.0.borrow_mut().as_mut().and_then(|m| field(m).clone());
        if let Some(v) = kept {
            return v;
        }
        let v = make();
        if let Some(m) = self.0.borrow_mut().as_mut() {
            *field(m) = Some(v.clone());
        }
        v
    }

    /// [`PlaceMemo::keep`] for an answer kept by key in map `field`.
    fn keep_by<K: Ord + Copy, T: Clone>(
        &self,
        field: fn(&mut Kept) -> &mut BTreeMap<K, T>,
        key: K,
        make: impl FnOnce() -> T,
    ) -> T {
        let kept = self
            .0
            .borrow_mut()
            .as_mut()
            .and_then(|m| field(m).get(&key).cloned());
        if let Some(v) = kept {
            return v;
        }
        let v = make();
        if let Some(m) = self.0.borrow_mut().as_mut() {
            field(m).insert(key, v.clone());
        }
        v
    }
}

/// What [`Script`] remembers of the house's income between thinks.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Watch {
    credits: i32,
    loads: i64,
    stall_since: Option<i32>,
    last_sale: Option<i32>,
    selling: bool,
    /// Whether the house has known a resource cell on the map (explored),
    /// as a person remembers having seen a field (M7b AI round 2).
    #[serde(default)]
    knew_field: bool,
    /// The balance owed and the start price of each item on the house's
    /// lines at the last think, by the item's object (M8 step B5).
    #[serde(default)]
    owed: BTreeMap<ObjectId, (i32, i32)>,
    /// The credits the house has earned since the watch began
    /// ([`Script::watch_income`]).
    #[serde(default)]
    earned: i64,
    /// `earned` at the start of the current window, once one has begun.
    #[serde(default)]
    mark: Option<i64>,
    /// The credits earned over the last whole window of
    /// [`INCOME_WINDOW`] ticks, once one has passed.
    #[serde(default)]
    income: Option<i64>,
    /// The world's next object id at the last think: objects made since
    /// have ids from it on (#50).
    #[serde(default)]
    next_id: Option<u32>,
    /// The house's buildings being sold at the last think, with the refund
    /// each pays when it goes (#50).
    #[serde(default)]
    sales: BTreeMap<ObjectId, i32>,
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
            waves: 0,
            scout: None,
            watch: Watch::default(),
            home_cell: None,
            placed: PlaceMemo::default(),
        }
    }

    /// A think starts: [`Script::place_cell`], [`Script::unblocker`] and
    /// [`Script::buildable`] keep their answers until [`Script::end_think`].
    pub(crate) fn begin_think(&self) {
        *self.placed.0.borrow_mut() = Some(Kept::default());
    }

    /// The think is over: the kept answers go.
    pub(crate) fn end_think(&self) {
        *self.placed.0.borrow_mut() = None;
    }

    /// Where the house places its done building of type `t`, on a think
    /// tick or between thinks alike: an anti-air defence ordered against a
    /// drainer beside it (M8 step B6, `antiair::place_goal`), a refinery by
    /// its field, and while the naval yard has no cell, another building
    /// toward the water; else the spiral's cell.
    pub(crate) fn done_cell(&self, w: &World, t: TypeRef) -> Option<CellXY> {
        let aa = crate::antiair::place_goal(self, w, t).and_then(|g| self.place_near(w, t, g));
        let toward = || {
            (!naval_building(w, t))
                .then(|| self.water_goal(w))
                .flatten()
                .and_then(|g| self.place_toward(w, t, g))
        };
        aa.or_else(|| self.place_refinery(w, t))
            .or_else(toward)
            .or_else(|| self.place_cell(w, t))
    }

    /// Places a done building of the house, on a think tick or between
    /// thinks alike: the first, line by line, that [`Script::done_cell`]
    /// finds a cell for, so that a done defence on its own line is placed
    /// while a building no cell takes waits on another. With `give_up` (a
    /// think tick), a done building that has had no cell for
    /// [`PLACE_PATIENCE`] ticks is cancelled, its line emptied of the type
    /// and its cost refunded (S-0404 claim 6), as a person cancels a
    /// building that fits nowhere and builds something else. Returns
    /// whether the house has a done building.
    // design: own AI (#40): only the first done building was looked at, and one with no cell waited on its line for good, so that nothing else was built
    pub(crate) fn place_done(&self, w: &World, give_up: bool, out: &mut Vec<Order>) -> bool {
        let done: Vec<(TypeRef, i32)> = w
            .econ
            .lines
            .lines
            .iter()
            .filter(|l| l.house == self.house)
            .filter_map(|l| {
                let item = l.item.as_ref()?;
                (item.t.kind == Kind::Building).then_some((item.t, item.done_tick?))
            })
            .collect();
        let mut placed = false;
        for &(t, at) in &done {
            let cell = if placed { None } else { self.done_cell(w, t) };
            if let Some(cell) = cell {
                // One a think: two placed at once may ask for the same cell.
                placed = true;
                out.push(Order::Place {
                    what: t,
                    cell: Some(cell),
                });
            } else if give_up && !placed && w.tick - at >= PLACE_PATIENCE {
                // spec: S-0404 claim 6 (a done item is cancelled and refunded; the cancel-all order also empties a queued copy)
                out.push(Order::CancelAll { what: t });
            }
        }
        !done.is_empty()
    }

    /// The tech-climb building the script ordered in the current think, if
    /// any: the one Build order a special building of the brain replaces.
    /// Another role-less building's Build order (a held item's resume, a
    /// sensor or a superweapon building) is not one.
    pub(crate) fn climbed(&self) -> Option<TypeRef> {
        self.placed.0.borrow().as_ref().and_then(|m| m.climbed)
    }

    /// Sets the enemy start cells to visit: every start cell but the one
    /// nearest the house's first objects.
    pub fn set_starts(&mut self, w: &World, starts: &[CellXY]) {
        let home = self.home(w);
        // design: own AI (#27): the start nearest the house's first objects is its own, however far they stand from it (bases off, or the base unit placed off its waypoint); one within five cells only was left out, and the own start could be the first target
        let own = home.and_then(|h| {
            starts
                .iter()
                .copied()
                .min_by_key(|&c| (dist2(c, h), c.y, c.x))
        });
        self.targets = starts.iter().copied().filter(|&c| Some(c) != own).collect();
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

    /// The types the house can build now for `role`, cheapest first: not
    /// one at its build limit, whose icon is dark and whose build order
    /// the sim refuses.
    /// The list is kept for the rest of a think (design: own AI, #59 and
    /// #60).
    pub(crate) fn candidates(&self, w: &World, role: Role) -> Vec<TypeRef> {
        self.placed
            .keep_by(
                |m| &mut m.candidates,
                role,
                || self.find_candidates(w, role).into(),
            )
            .to_vec()
    }

    /// [`Script::candidates`]'s search.
    fn find_candidates(&self, w: &World, role: Role) -> Vec<TypeRef> {
        let mut v: Vec<TypeRef> = all_types(w)
            .into_iter()
            // design: own AI (#6): a type at its limit is no candidate, so the next one is tried; spec: S-0407 claims 9 and 13 (at limit: refused when the order is issued)
            .filter(|&t| self.allowed(w, t, role) && self.buildable(w, t) && !self.at_limit(w, t))
            .collect();
        v.sort_by_key(|&t| (production::cost_field(w, t), t.kind as u8, t.index));
        v
    }

    /// The cost of the cheapest army type the house can build now (0 with
    /// none).
    fn cheapest_army(&self, w: &World) -> i32 {
        self.candidates(w, Role::Army)
            .first()
            .map_or(0, |&t| production::cost_field(w, t))
    }

    /// The house's living objects on the map, in object order; one scan a
    /// think, the list kept for the rest of it (design: own AI, #53 and
    /// #59: the world does not change while a brain thinks).
    pub(crate) fn own(&self, w: &World) -> Vec<ObjectId> {
        self.placed
            .keep(|m| &mut m.own, || self.scan_own(w).into())
            .to_vec()
    }

    /// [`Script::own`]'s scan.
    fn scan_own(&self, w: &World) -> Vec<ObjectId> {
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
    /// The list is kept for the rest of a think (design: own AI, #53 and
    /// #59).
    pub(crate) fn of_role(&self, w: &World, role: Role) -> Vec<ObjectId> {
        self.placed
            .keep_by(|m| &mut m.roles, role, || self.find_of_role(w, role).into())
            .to_vec()
    }

    /// [`Script::of_role`]'s search.
    fn find_of_role(&self, w: &World, role: Role) -> Vec<ObjectId> {
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

    /// The house's home: the cell of its construction yard. Once known it
    /// is kept from think to think: it stays while a yard stands on it,
    /// and while the house has no yard (a lost yard does not move it to
    /// the house's oldest unit, wherever that stands); when the yard there
    /// is gone and the house has another, it moves to the yard nearest it
    /// (a captured enemy yard with a lower object number does not move it
    /// into the enemy base while the house's own yard stands). Before it
    /// is known: the house's first yard, else its first building, else its
    /// first object.
    pub(crate) fn home(&self, w: &World) -> Option<CellXY> {
        self.placed.keep(|m| &mut m.home, || self.find_home(w))
    }

    /// [`Script::home`]'s search.
    // design: own AI (#12): home was the house's first yard by object number, else its first object
    fn find_home(&self, w: &World) -> Option<CellXY> {
        let own = self.own(w);
        let yards: Vec<CellXY> = own
            .iter()
            .filter_map(|&id| w.obj(id))
            .filter(|o| is_yard(w, o))
            .map(|o| o.pos.cell())
            .collect();
        if let Some(h) = self.home_cell {
            if yards.is_empty() || yards.contains(&h) {
                return Some(h);
            }
            return yards
                .iter()
                .copied()
                .min_by_key(|&c| (dist2(c, h), c.y, c.x));
        }
        yards.first().copied().or_else(|| {
            own.iter()
                .filter_map(|&id| w.obj(id))
                .find(|o| o.kind == Kind::Building)
                .or_else(|| own.first().and_then(|&id| w.obj(id)))
                .map(|o| o.pos.cell())
        })
    }

    /// Keeps the house's home for the next thinks ([`Script::home`]).
    fn keep_home(&mut self, w: &World) {
        self.home_cell = self.home(w);
    }

    /// The enemy start cells (the map's start waypoints of the other
    /// players: lobby knowledge), in visiting order, less those where an
    /// ally's base stands ([`Script::ally_base_at`]). Read at every think,
    /// so an alliance made during the game counts from then on.
    pub(crate) fn enemy_starts(&self, w: &World) -> Vec<CellXY> {
        let me = &w.houses[usize::from(self.house.0)];
        if !me.allies.iter().any(|&h| h != self.house) {
            return self.targets.clone();
        }
        self.targets
            .iter()
            .copied()
            .filter(|&c| !self.ally_base_at(w, c))
            .collect()
    }

    /// Whether a building of a player house that this house allies (not
    /// itself) stands within [`ALLY_BASE`] cells of start cell `c`: that
    /// start is an ally's base, and a person sends no wave there. Allies
    /// share their sight (S-0409 claim 2), so the house sees the building;
    /// the alliance is this house's view of it (S-0414 claim 12: an
    /// alliance is one-way), as the sim's targeting reads it (S-0309
    /// claim 18).
    pub(crate) fn ally_base_at(&self, w: &World, c: CellXY) -> bool {
        let me = &w.houses[usize::from(self.house.0)];
        let r = i64::from(ALLY_BASE) * i64::from(ALLY_BASE);
        w.objects.values().any(|o| {
            o.on_map
                && o.kind == Kind::Building
                && o.owner.is_some_and(|h| {
                    h != self.house
                        && me.allied_with(h)
                        && w.houses
                            .get(usize::from(h.0))
                            .is_some_and(|x| x.role == glasnost_sim::world::HouseRole::Player)
                })
                && dist2(o.pos.cell(), c) <= r
                && shroud::explored(w, self.house, o.pos.cell())
        })
    }

    /// Where the next wave would go, without choosing it: the seen enemy
    /// nearest home, else the next enemy start cell.
    pub(crate) fn attack_goal(&self, w: &World) -> Option<CellXY> {
        let home = self.home(w)?;
        let seen = self.seen_enemies(w);
        seen.iter()
            .copied()
            .min_by_key(|&(e, c)| (dist2(c, home), e))
            .map(|(_, c)| c)
            .or_else(|| self.next_start(w, &seen))
    }

    /// The enemy start cell waves go to while they see no enemy: the
    /// nearest start that may still hold an enemy base, one the house has
    /// not explored or where it sees an enemy building within
    /// [`ALLY_BASE`] cells; with none, the start of the visiting order.
    /// A person takes on one enemy base at a time, the nearest, and moves
    /// on once it is gone (M8 step A4 round 8, the lead's note 52: each
    /// wave went to the next start in turn, so in a game of three or more
    /// the waves were spread over every enemy).
    fn next_start(&self, w: &World, seen: &[(ObjectId, CellXY)]) -> Option<CellXY> {
        let targets = self.enemy_starts(w);
        targets
            .iter()
            .copied()
            .find(|&c| !self.cleared(w, c, seen))
            .or_else(|| (!targets.is_empty()).then(|| targets[self.next_target % targets.len()]))
    }

    /// Whether start cell `c` holds no enemy base the house knows of: it has
    /// explored the cell and sees no enemy building within [`ALLY_BASE`]
    /// cells of it.
    fn cleared(&self, w: &World, c: CellXY, seen: &[(ObjectId, CellXY)]) -> bool {
        let r = i64::from(ALLY_BASE) * i64::from(ALLY_BASE);
        shroud::explored(w, self.house, c)
            && !seen.iter().any(|&(e, at)| {
                dist2(at, c) <= r && w.obj(e).is_some_and(|o| o.kind == Kind::Building)
            })
    }

    /// Where a ferry would go, without choosing it: the seen enemy
    /// building nearest home, else the next enemy start cell. A person
    /// ferries infantry to the enemy's base, not to a unit that flies or
    /// stands where no one walks.
    pub(crate) fn base_goal(&self, w: &World) -> Option<CellXY> {
        self.base_goal_for(w, None)
    }

    /// [`Script::base_goal`] for units of movement zone `zone`: only a
    /// building whose cell lies in a reach region of that zone (S-0212
    /// claim 4) counts, so that a naval yard on the water, which no one
    /// walks to, does not make every infantry unit seem to need a ferry.
    pub(crate) fn base_goal_for(&self, w: &World, zone: Option<MovementZone>) -> Option<CellXY> {
        let home = self.home(w)?;
        let seen = self.seen_enemies(w);
        seen.iter()
            .copied()
            .filter(|&(e, _)| w.obj(e).is_some_and(|o| o.kind == Kind::Building))
            // design: own AI (#9): the goal is a building the zone's units can stand by
            .filter(|&(_, c)| {
                zone.is_none_or(|z| matches!(regions::reach_in(w, c, z), Reach::Region(_)))
            })
            .min_by_key(|&(e, c)| (dist2(c, home), e))
            .map(|(_, c)| c)
            .or_else(|| self.next_start(w, &seen))
    }

    /// Whether the line of `t` holds an item or a queue.
    pub(crate) fn busy(&self, w: &World, t: TypeRef) -> bool {
        production::line_of(w, self.house, t)
            .is_some_and(|l| l.item.is_some() || !l.queue.is_empty())
    }

    /// Whether the line of `t` holds `t`, as its item or in its queue.
    pub(crate) fn in_line(&self, w: &World, t: TypeRef) -> bool {
        production::line_of(w, self.house, t)
            .is_some_and(|l| l.item.as_ref().is_some_and(|i| i.t == t) || l.queue.contains(&t))
    }

    /// Whether a build order for `t` would start it or join its line's
    /// queue: the line is idle, or its queue is below the limit (S-0404
    /// claim 3, the queue limit counts queued types).
    pub(crate) fn queue_room(&self, w: &World, t: TypeRef) -> bool {
        // spec: S-0404 claim 3 (the queue limit counts queued types)
        let max = w.rules().econ.production.max_queued;
        production::line_of(w, self.house, t)
            .is_none_or(|l| l.item.is_none() || (l.queue.len() as i32) < max)
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
        if let Some(found) = self.placed.unblocker(t, depth) {
            return found;
        }
        let found = self.search_unblocker(w, t, depth);
        self.placed.put_unblocker(t, depth, found);
        found
    }

    /// Whether the house can build `t` now ([`tech::buildable`]), the
    /// answer kept for the rest of a think.
    pub(crate) fn buildable(&self, w: &World, t: TypeRef) -> bool {
        if let Some(yes) = self.placed.buildable(t) {
            return yes;
        }
        let yes = tech::buildable(w, self.house, t);
        self.placed.put_buildable(t, yes);
        yes
    }

    /// Whether a build order for `t` would be refused at issue now by its
    /// build limit or by claim 16's test, a full pad total for a dock-bound
    /// aircraft among them ([`tech::refused_at_issue`]), the answer kept for
    /// the rest of a think.
    fn at_limit(&self, w: &World, t: TypeRef) -> bool {
        if let Some(yes) = self.placed.limited(t) {
            return yes;
        }
        // spec: S-0407 claims 13 and 16 *At issue* (design: own AI, #426: a candidate the sidebar would refuse at issue is not chosen)
        let yes = tech::refused_at_issue(w, self.house, t);
        self.placed.put_limited(t, yes);
        yes
    }

    /// [`Script::unblocker`]'s search.
    fn search_unblocker(&self, w: &World, t: TypeRef, depth: u32) -> Option<TypeRef> {
        if self.buildable(w, t) {
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
                if self.buildable(w, b) {
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
            .filter(|&t| self.allowed(w, t, Role::Army) && !self.buildable(w, t))
            .filter_map(|t| self.unblocker(w, t, CLIMB_DEPTH))
            .map(|b| (production::cost_field(w, b), b.index, b))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|(_, _, b)| b)
            // design: own AI (#40): the cheapest with a cell now, so that a building no cell takes is not ordered again after its cancel
            .find(|&b| self.placeable(w, b))
    }

    /// The credits the house earned over the last whole window of
    /// [`INCOME_WINDOW`] ticks; none before one has passed since the watch
    /// began.
    pub(crate) fn income(&self) -> Option<i64> {
        self.watch.income
    }

    /// Adds what the house earned since the last think to the watch, as a
    /// person reads it off the credits counter and the sidebar: the change
    /// of its credits plus what its lines paid meanwhile (S-0404 claim 4:
    /// each step is paid from the credits). A line item's payment is the
    /// fall of its balance owed since the last think (its start price less
    /// the balance for an item started since); an item gone from the lines
    /// paid the rest of its balance when its object is still there (done
    /// and released or placed), and was cancelled, its payments refunded
    /// (S-0404 claim 6, at an unchanged price), when not. An own object made
    /// since the last think and no longer on a line was started and done in
    /// between, and paid its whole start price. The refund of a building
    /// seen in its sale at the last think and gone now is no income
    /// (#50). A fall of the
    /// credits that no payment explains (a drain, S-0608) is no income.
    /// Every [`INCOME_WINDOW`] ticks the window's sum becomes
    /// [`Script::income`]. M8 step B5: a house that kept every harvester it
    /// wanted bought a yard vehicle on a mined-out map with an income of
    /// 275 to 600 credits per 1 500 ticks, and it stood on its line to the
    /// end of the game.
    fn watch_income(&mut self, w: &World) {
        let credits = w.houses[usize::from(self.house.0)].credits;
        let now: BTreeMap<ObjectId, (i32, i32)> = w
            .econ
            .lines
            .lines
            .iter()
            .filter(|l| l.house == self.house)
            .filter_map(|l| l.item.as_ref())
            .map(|i| (i.object, (i.owed, i.start_price)))
            .collect();
        let mut paid: i64 = now
            .iter()
            .map(|(id, &(owed, start))| {
                let before = self.watch.owed.get(id).map_or(start, |&(b, _)| b);
                i64::from(before) - i64::from(owed)
            })
            .sum();
        for (id, &(owed, start)) in &self.watch.owed {
            if now.contains_key(id) {
                continue;
            }
            paid += if w.obj(*id).is_some() {
                i64::from(owed)
            } else {
                -(i64::from(start) - i64::from(owed))
            };
        }
        // design: own AI (#50): an item started and done between two thinks
        // (an own object made since the last think, with a start price and
        // no longer on a line) paid its whole start price meanwhile (S-0404
        // claims 2 and 4).
        if let Some(first) = self.watch.next_id {
            paid += w
                .objects
                .range(ObjectId(first)..)
                .filter(|(id, o)| o.owner == Some(self.house) && !now.contains_key(id))
                .filter_map(|(_, o)| o.econ.start_price)
                .map(i64::from)
                .sum::<i64>();
        }
        // design: own AI (#50): the refund of a building the house sold
        // (S-0408 claim 1, paid when the build-down ends and the building
        // goes, S-0408 claims 6 and 17) is no income. A building seen in
        // its sale at a think and gone at the next paid it; one sold
        // whole between two thinks, or an upgrade sold off (claim 5), is
        // not seen.
        let refunds: i64 = self
            .watch
            .sales
            .iter()
            .filter(|&(id, _)| w.obj(*id).is_none_or(|o| o.owner != Some(self.house)))
            .map(|(_, &r)| i64::from(r))
            .sum();
        self.watch.sales = w
            .objects
            .values()
            .filter(|o| o.owner == Some(self.house) && o.kind == Kind::Building)
            .filter(|o| o.econ.sale.as_ref().is_some_and(|s| !s.packs))
            .map(|o| (o.id, sell::refund(w, self.house, tech::type_of(o))))
            .collect();
        self.watch.next_id = Some(w.next_id);
        let gained = i64::from(credits) - i64::from(self.watch.credits) + paid - refunds;
        self.watch.earned += gained.max(0);
        self.watch.owed = now;
        // A window ends at each multiple of INCOME_WINDOW; the watch reads it at the first think at or after it (thinks come every THINK_PERIOD ticks, at the house's phase).
        if (w.tick - THINK_PERIOD).div_euclid(INCOME_WINDOW) != w.tick.div_euclid(INCOME_WINDOW) {
            if let Some(mark) = self.watch.mark {
                self.watch.income = Some(self.watch.earned - mark);
            }
            self.watch.mark = Some(self.watch.earned);
        }
    }

    /// Updates the economy watch and, when the house is broke (no income:
    /// no credits earned and no change in what its harvesters and its slaves
    /// on the map carry, for [`BROKE_PATIENCE`] ticks; and too few credits
    /// for its cheapest army unit), sells one building, as a person in its seat would, the
    /// costliest first (S-0408 claims 1 and 3: the refund goes to its
    /// credits; only a sellable building is chosen). It keeps one factory
    /// per kind while it has anything else to sell; a house with nothing
    /// left to fight with and nothing else to sell sells those too, which
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
        // design: own AI (#60): asked last, as the patience and the refund
        // decide alone most thinks (the test reads every type's tech).
        let affordable = || {
            all_types(w)
                .into_iter()
                .filter(|&a| self.buildable(w, a))
                .any(|a| credits >= production::cost_field(w, a))
        };
        if t - since < BROKE_PATIENCE || refund || affordable() {
            return;
        }
        self.watch.selling = true;
        let army = !self.of_role(w, Role::Army).is_empty();
        // design: while its harvesters wait for a lost refinery the house
        // keeps the construction yard that builds one, and sells the rest
        // first to pay for it (M8 step B8: in the quick lane a house sold
        // its yard first, the costliest building, and its two miners stood
        // for the rest of the game).
        let rebuilding = self.refinery_lost(w).is_some();
        // design: one factory per kind is kept while anything else is left
        // to sell, army or none: a house with no army and no money sells
        // its other buildings first to fund one, and sells out (which ends
        // its part in the game) only once the kept buildings are all it
        // has and it still cannot pay for a unit (M8 step A4 round 9b: a
        // house with no army sold its costliest building first, often its
        // war factory, and so the means to build the army its refunds paid
        // for). With an army the kept buildings are never sold.
        let keep = |id: ObjectId| {
            let Some(o) = w.obj(id) else { return true };
            (rebuilding && is_yard(w, o))
                || FACTORIES.iter().any(|&k| {
                    self.allowed(w, tech::type_of(o), Role::Factory(k))
                        && self.of_role(w, Role::Factory(k)).first() == Some(&id)
                })
        };
        let sellable: Vec<ObjectId> = self
            .own(w)
            .into_iter()
            .filter(|&id| w.obj(id).is_some_and(|o| o.econ.sale.is_none()) && sell::sellable(w, id))
            .collect();
        let costliest = |ids: &mut dyn Iterator<Item = ObjectId>| {
            ids.max_by_key(|&id| {
                let o = w.obj(id).expect("own object");
                (
                    production::cost_field(w, tech::type_of(o)),
                    std::cmp::Reverse(id),
                )
            })
        };
        let yard_kept = |id: ObjectId| rebuilding && w.obj(id).is_some_and(|o| is_yard(w, o));
        let sale = costliest(&mut sellable.iter().copied().filter(|&id| !keep(id))).or_else(|| {
            (!army)
                .then(|| costliest(&mut sellable.iter().copied().filter(|&id| !yard_kept(id))))
                .flatten()
        });
        if let Some(actor) = sale {
            // design: own AI (#50): its refund is no income at the think after it goes
            if let Some(o) = w.obj(actor) {
                let r = sell::refund(w, self.house, tech::type_of(o));
                self.watch.sales.insert(actor, r);
            }
            out.push(Order::Sell { actor });
            self.watch.last_sale = Some(t);
            self.watch.stall_since = Some(t);
        }
    }

    /// The harvesters the house keeps: the plan's count, and more while
    /// the explored resource around its refineries, slave miners,
    /// construction yards and home would keep them at work, one for every [`CELLS_PER_HARVESTER`]
    /// resource cells within [`FIELD_REACH`] cells, as a person adds
    /// harvesters to a large field. A person stops at twice the plan's
    /// count: more harvesters than that crowd the same fields and docks
    /// (M7b AI audit: two slave miners earned about a third of what two
    /// harvesters of the other sides earned on the same stock maps, and the
    /// house starved).
    pub(crate) fn harvesters_wanted(&self, w: &World) -> usize {
        let mut centres = self.field_centres(w);
        centres.extend(self.home(w));
        let mut cells = std::collections::BTreeSet::new();
        for c in centres {
            for dy in -FIELD_REACH..=FIELD_REACH {
                for dx in -FIELD_REACH..=FIELD_REACH {
                    let x = CellXY::new(c.x + dx, c.y + dy);
                    if dx * dx + dy * dy <= FIELD_REACH * FIELD_REACH
                        && resource::resource_at(w, x).is_some()
                        && shroud::explored(w, self.house, x)
                    {
                        cells.insert((x.y, x.x));
                    }
                }
            }
        }
        let n = self.plan.harvesters;
        if cells.is_empty() && self.field_spent(w) {
            return 0;
        }
        let want = (cells.len() / CELLS_PER_HARVESTER).max(n);
        // A slave miner is its slaves' own refinery (S-0402 claim 14), so
        // nothing queues: a person buys one for every field its slaves can
        // work (M7b AI round 2: a Yuri house with two slave miners earned a
        // quarter of what its enemy's two harvesters did, beside fields
        // nobody worked).
        if self.slave_miners(w) {
            return want;
        }
        // A person stops at twice the plan's harvesters: more queue at the
        // refineries.
        want.min(2 * n) // nocite: tuning of our own opponent
    }

    /// The cells of the house's construction yards, refineries and slave
    /// miners: the centres of the fields it works.
    fn field_centres(&self, w: &World) -> Vec<CellXY> {
        self.own(w)
            .into_iter()
            .filter_map(|id| w.obj(id))
            .filter(|o| {
                slaves::is_master(w, o.id)
                    || is_yard(w, o)
                    || (o.kind == Kind::Building
                        && self.allowed(w, tech::type_of(o), Role::Refinery))
            })
            .map(|o| o.pos.cell())
            .collect()
    }

    /// The explored resource cell nearest home of a field the house does
    /// not work, nearer home than every enemy start cell, the first in
    /// cell order on equal distance: the far field a person sends a second
    /// yard to (a field in the enemy's half of the map is not one). A field
    /// is the explored resource cells joined by gaps of at most
    /// [`FIELD_GAP`] cells; the house works it when one of its cells lies
    /// within [`FIELD_REACH`] cells of one of the house's construction
    /// yards, refineries or slave miners (the field's edge grows away from
    /// them as the resource spreads).
    pub(crate) fn far_field(&self, w: &World) -> Option<CellXY> {
        self.fields(w).far
    }

    /// The far-field search of [`Self::far_field`] with its counts (the
    /// lane's expansion probe reads them).
    pub(crate) fn fields(&self, w: &World) -> Fields {
        let Some(home) = self.home(w) else {
            return Fields::default();
        };
        let centres = self.field_centres(w);
        let reach = i64::from(FIELD_REACH) * i64::from(FIELD_REACH);
        let ore: std::collections::BTreeSet<(i32, i32)> = w
            .cells
            .grid
            .iter()
            .map(|(c, _)| c)
            .filter(|&c| {
                w.cells.usable(c)
                    && resource::resource_at(w, c).is_some()
                    && shroud::explored(w, self.house, c)
            })
            .map(|c| (c.y, c.x))
            .collect();
        let mut worked: std::collections::BTreeSet<(i32, i32)> = ore
            .iter()
            .copied()
            .filter(|&(y, x)| {
                centres
                    .iter()
                    .any(|&m| dist2(m, CellXY::new(x, y)) <= reach)
            })
            .collect();
        let mut todo: Vec<(i32, i32)> = worked.iter().copied().collect();
        while let Some((y, x)) = todo.pop() {
            for dy in -FIELD_GAP - 1..=FIELD_GAP + 1 {
                for dx in -FIELD_GAP - 1..=FIELD_GAP + 1 {
                    let n = (y + dy, x + dx);
                    if ore.contains(&n) && worked.insert(n) {
                        todo.push(n);
                    }
                }
            }
        }
        let enemy_starts = self.enemy_starts(w);
        let own_half: Vec<CellXY> = ore
            .difference(&worked)
            .map(|&(y, x)| CellXY::new(x, y))
            .filter(|&c| enemy_starts.iter().all(|&t| dist2(c, home) < dist2(c, t)))
            .collect();
        Fields {
            explored: ore.len(),
            worked: worked.len(),
            own_half: own_half.len(),
            far: own_half
                .into_iter()
                .min_by_key(|&c| (dist2(c, home), c.y, c.x)),
        }
    }

    /// Whether an explored cell of the map holds a resource for the house.
    fn knows_field(&self, w: &World) -> bool {
        w.cells.grid.iter().any(|(c, _)| {
            resource::resource_at(w, c).is_some() && shroud::explored(w, self.house, c)
        })
    }

    /// Whether the fields the house has known are all mined out: it has
    /// known a resource cell (as of its last think) and no explored cell
    /// holds one now. A person does not buy harvesters for a map whose
    /// fields are all mined out, and then spends on the army (M7b AI round
    /// 2: a house kept wanting a lost slave miner on a map with no resource
    /// left, held its army for it and ended the game with 53 605 credits
    /// unspent at the tick limit). A house that has never seen a field is
    /// not told by this that there is none.
    pub(crate) fn field_spent(&self, w: &World) -> bool {
        self.watch.knew_field && !self.knows_field(w)
    }

    /// Whether the house's harvesters are slave miners (its first harvester
    /// candidate is one).
    pub(crate) fn slave_miners(&self, w: &World) -> bool {
        self.candidates(w, Role::Harvester)
            .first()
            .is_some_and(|&t| is_slave_miner(w, t))
    }

    /// The harvester count the house pays for before its army and before
    /// the other items on its lines: the plan's, or with slave miners
    /// every one it wants ([`Self::harvesters_wanted`]). M7b AI round 2: a
    /// Yuri house kept two slave miners, earned a quarter of its Allied
    /// enemy's income and bought the army while its third miner waited on
    /// its line for money.
    pub(crate) fn economy_target(&self, w: &World) -> usize {
        if self.slave_miners(w) {
            self.harvesters_wanted(w)
        } else {
            self.plan.harvesters
        }
    }

    /// Whether the house keeps the plan's harvester count (deployed slave
    /// miners count), or wants none (a mined-out map, [`Self::field_spent`]):
    /// the condition for the extras of the base (superweapon buildings,
    /// sensors, defences of the brain's own).
    pub(crate) fn harvesters_kept(&self, w: &World) -> bool {
        self.of_role(w, Role::Harvester).len() + self.slave_buildings(w).len()
            >= self.plan.harvesters
            || self.field_spent(w)
    }

    /// Whether the house keeps its [`Self::economy_target`], or wants no
    /// harvester (a mined-out map): while it does not, a harvester waiting
    /// on its line for money is paid first (`Brain::economy_first`).
    pub(crate) fn economy_kept(&self, w: &World) -> bool {
        self.of_role(w, Role::Harvester).len() + self.slave_buildings(w).len()
            >= self.economy_target(w)
            || self.field_spent(w)
    }

    /// The base role the plan builds next: the first missing role the
    /// house can build a type for now (a role it cannot build yet, such as
    /// a refinery that needs a factory first, does not hold up the others).
    /// Power is read off the plants without a blackout (`power::short`): a
    /// blackout is not answered with power plants.
    pub(crate) fn wanted_role(&self, w: &World) -> Option<Role> {
        self.base_role(w)
            .or_else(|| self.field_refinery(w).then_some(Role::Refinery))
    }

    /// Whether the role the plan builds next is the refinery by a new field
    /// ([`Script::field_refinery`]): no other base role is wanted (the
    /// lane's economy probe counts these spells, M8 step B9).
    pub(crate) fn wants_field_refinery(&self, w: &World) -> bool {
        self.base_role(w).is_none() && self.field_refinery(w)
    }

    /// [`Script::wanted_role`] without the refinery by a new field.
    fn base_role(&self, w: &World) -> Option<Role> {
        // design: a lost refinery its harvesters wait for comes before
        // power (M8 step B8): with no refinery the house has no income.
        if self.refinery_lost(w).is_some() {
            return Some(Role::Refinery);
        }
        let low_power =
            crate::power::short(w, self.house) || self.of_role(w, Role::Power).is_empty();
        low_power
            .then_some(Role::Power)
            .into_iter()
            .chain((!self.has_refinery(w)).then_some(Role::Refinery))
            .chain(
                FACTORIES
                    .into_iter()
                    .map(Role::Factory)
                    .filter(|&f| self.of_role(w, f).is_empty()),
            )
            // design: own AI (#40): a role none of whose types has a cell now does not hold up the roles after it (`build` orders only a type with a cell)
            .find(|&r| {
                self.candidates(w, r)
                    .into_iter()
                    .any(|t| self.placeable(w, t))
            })
    }

    /// The refinery type the house rebuilds before anything else, when it
    /// keeps harvesters (not slave miners, which are their slaves' own
    /// refinery, S-0402 claim 14) and owns no refinery (it lost them all,
    /// or, starting with a harvester, has built none yet): the cheapest
    /// refinery type it can build and place now. A harvester whose owner
    /// owns no refinery queues *guard* and stands (S-0402 claim 13), so the
    /// house earns nothing until one stands again; a person rebuilds it
    /// first, clearing the building line and holding the rest of the
    /// sidebar for it (`Brain::economy_first`), and keeps the construction
    /// yard while selling out (M8 step B8: in the quick lane a lost
    /// refinery waited behind a power plant on the building line at
    /// credits near 0, and the yard was sold). A house with no harvester
    /// left that sells out counts too, for a refinery type that comes with
    /// a free harvester (S-0404 claim 2, S-0402 claim 18): that refinery
    /// is its way back to an income.
    pub(crate) fn refinery_lost(&self, w: &World) -> Option<TypeRef> {
        if self.has_refinery(w) || self.slave_miners(w) {
            return None;
        }
        let harvesters = !self.of_role(w, Role::Harvester).is_empty();
        // design: a house selling out whose refinery and harvesters are
        // all gone rebuilds the refinery that brings a harvester, and keeps
        // the yard for it, as a person does (M8 step B10). Selling out
        // latches until the house earns again (`Script::sell_when_broke`),
        // and with no harvester that never came: such a house ordered
        // nothing and sold its yard first. A house that is not selling out
        // (the opening among them) keeps the plan's order.
        let brings_harvester = |t: TypeRef| {
            production::free_unit(w, t.index)
                .is_some_and(|f| role_of(w, f) == Some(Role::Harvester))
        };
        if !harvesters && !self.watch.selling {
            return None;
        }
        self.candidates(w, Role::Refinery).into_iter().find(|&t| {
            !keeps_slaves(w, t) && (harvesters || brings_harvester(t)) && self.placeable(w, t)
        })
    }

    /// The refinery type the plan orders for a new field
    /// ([`Script::field_refinery`]) when that is what it builds next: the
    /// cheapest one it can build and place now (`Script::build`'s pick).
    pub(crate) fn field_refinery_type(&self, w: &World) -> Option<TypeRef> {
        if !self.wants_field_refinery(w) {
            return None;
        }
        self.candidates(w, Role::Refinery)
            .into_iter()
            .find(|&t| !keeps_slaves(w, t) && self.placeable(w, t))
    }

    /// Whether the house builds a refinery beside a new field (M8 step
    /// B8): it keeps harvesters (not slave miners, S-0402 claim 14) and has
    /// a refinery, the explored resource within [`FIELD_REACH`] cells of
    /// its refineries would not keep one harvester at work (fewer than
    /// [`CELLS_PER_HARVESTER`] cells), the field of [`Script::ore_goal`]
    /// lies beyond that reach of every refinery, no refinery is on its
    /// lines, and a refinery type it can build both has a cell in the
    /// build spiral (`Script::placeable`) and fits with its foundation
    /// within that reach of the field. A person builds a refinery at the field his harvesters
    /// drive to once the one by his refinery runs dry (M8 step B8: in the
    /// quick lane on maps 2 and 8, with the refinery beside the home field,
    /// the Allied miners loaded a median of 15 to 34 cells from their only
    /// refinery once that field was mined down); one per field, since
    /// the new refinery puts that field within reach. A building that
    /// comes with a free unit gives the house a harvester with it (S-0402
    /// claim 18).
    pub(crate) fn field_refinery(&self, w: &World) -> bool {
        if self.slave_miners(w) || self.of_role(w, Role::Harvester).is_empty() {
            return false;
        }
        let refineries: Vec<CellXY> = self
            .of_role(w, Role::Refinery)
            .iter()
            .filter_map(|&id| w.obj(id))
            .map(|o| o.pos.cell())
            .collect();
        if refineries.is_empty() {
            return false;
        }
        let reach = i64::from(FIELD_REACH) * i64::from(FIELD_REACH);
        let near = w
            .cells
            .grid
            .iter()
            .map(|(c, _)| c)
            .filter(|&c| {
                refineries.iter().any(|&r| dist2(r, c) <= reach)
                    && w.cells.usable(c)
                    && resource::resource_at(w, c).is_some()
                    && shroud::explored(w, self.house, c)
            })
            .count();
        if near >= CELLS_PER_HARVESTER {
            return false;
        }
        let Some(goal) = self.ore_goal(w) else {
            return false;
        };
        let types: Vec<TypeRef> = self
            .candidates(w, Role::Refinery)
            .into_iter()
            .filter(|&t| !keeps_slaves(w, t))
            .collect();
        // design: the want holds only while the refinery could be ordered
        // and placed now (M8 step B9): `Script::build` orders a type only
        // when `Script::placeable` finds it a cell, so a refinery that fits
        // by the field but not in the build spiral was wanted with nothing
        // ever ordered, and the tech climb behind it waited. Read afresh at
        // every think, the want comes back when a cell opens, a new field
        // is explored or a harvester reaches one (`Script::ore_goal`).
        !refineries.iter().any(|&r| dist2(r, goal) <= reach)
            && !types.iter().any(|&t| self.in_line(w, t))
            && types.iter().any(|&t| {
                self.placeable(w, t) && self.refinery_site(w, t, goal, Some(reach)).is_some()
            })
    }

    /// The power plant the plan builds next: the cheapest power plant type
    /// the house can build and place now.
    pub(crate) fn plant_type(&self, w: &World) -> Option<TypeRef> {
        self.candidates(w, Role::Power)
            .into_iter()
            .find(|&t| self.placeable(w, t))
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
        self.think_with(w, &mut ())
    }

    /// [`Script::think`], telling `p` which part of the plan runs (the
    /// tools' benchmark, `glasnost_sim::probe`, M8 C4 part 6); `p` reads
    /// nothing and changes nothing.
    pub fn think_with<P: Probe>(&mut self, w: &World, p: &mut P) -> Vec<Order> {
        let on = p.on();
        let mut mark = |part: &'static str| {
            if on {
                p.mark(Phase::Part(part));
            }
        };
        mark("ai:script:watch");
        self.keep_home(w);
        let mut out = Vec::new();
        let house = self.house;
        let own = self.own(w);
        if !self.watch.knew_field {
            self.watch.knew_field = self.knows_field(w);
        }
        self.watch_income(w);
        mark("ai:script:watch:sell");
        self.sell_when_broke(w, &mut out);
        let selling = self.watch.selling;

        mark("ai:script:deploy");
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
        // cell beyond it like a person's (S-0213 claim 20 (a)), so that its
        // infantry gather toward the attack goal (design). Without one a
        // released unit is only sent off the barracks to a cell next to it
        // (S-0417 claims 10 and 17).
        self.rally_points(w, &own, &mut out);

        mark("ai:script:buildings");
        // 2 and 3. Buildings: a done building is placed first; otherwise the
        // next wanted one is ordered.
        mark("ai:script:buildings:done");
        let placing = self.place_done(w, true, &mut out);
        if !placing && (!selling || self.refinery_lost(w).is_some()) {
            // design: a house selling out still rebuilds a lost refinery
            // its harvesters wait for (`Script::refinery_lost`, M8 step B8).
            mark("ai:script:buildings:wanted");
            let wanted = self.wanted_role(w);
            mark("ai:script:buildings:order");
            let walk = || {
                // The cheapest power plant with a cell toward the water.
                let goal = self.water_goal(w)?;
                self.candidates(w, Role::Power)
                    .into_iter()
                    .find(|&t| self.place_toward(w, t, goal).is_some())
            };
            if let Some(role) = wanted {
                self.build(w, role, &mut out);
            } else if !selling && self.harvesters_kept(w) {
                mark("ai:script:buildings:walk");
                let walked = walk();
                mark("ai:script:buildings:climb");
                if let Some(t) = walked.or_else(|| self.climb(w))
                    && self.order_build(w, t, &mut out).is_some()
                    && role_of(w, t).is_none()
                {
                    self.placed.set_climbed(t);
                }
            }
        }

        mark("ai:script:harvesters");
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
            } else if !refineries.is_empty() && (resting || idle(w, h) || stalled_unload(w, h)) {
                // A harvester that rests (whatever destination it kept), is
                // idle or stands stalled in *unload* is sent to harvest.
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
        let wanted = self.harvesters_wanted(w);
        if can_harvest && economy < wanted && !selling {
            self.build(w, Role::Harvester, &mut out);
        }

        mark("ai:script:army_types");
        // The army: one type round-robin per kind of factory, once the
        // plan's harvesters are kept (while a harvester can still be had:
        // a harvester type to build now, or a vehicle factory to build
        // while the house has none). Beyond the plan's count, up to every
        // slave miner the house wants (`economy_target`), the army waits
        // for the miner only while the house cannot pay for both beyond
        // what its running items still owe: a person with money builds the
        // army beside the extra miner.
        let vehicle_factory = Role::Factory(FactoryKind::Vehicle);
        let credits = w.houses[usize::from(house.0)].credits;
        let owed: i32 = w
            .econ
            .lines
            .lines
            .iter()
            .filter(|l| l.house == house)
            .filter_map(|l| l.item.as_ref())
            .filter(|i| !i.done() && i.hold == production::Hold::None)
            .map(|i| i.owed)
            .fold(0i32, i32::saturating_add);
        let harvester = self.candidates(w, Role::Harvester).first().copied();
        let rich = harvester.is_some_and(|t| {
            credits.saturating_sub(owed)
                >= production::cost_field(w, t).saturating_add(self.cheapest_army(w))
        });
        let short = economy < self.plan.harvesters.min(wanted)
            || (economy < self.economy_target(w).min(wanted) && !rich);
        let economy_first = !selling
            && short
            && (harvester.is_some()
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

        mark("ai:script:army_units");
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
        // The ferry's goal by the unit's movement zone (`base_goal_for`).
        let mut ahead: BTreeMap<Option<u8>, Option<CellXY>> = BTreeMap::new();
        // The nearest region cells to home (`reachable`), for the units
        // that gather there for a ferry.
        let mut gather: BTreeMap<(u8, u32), CellXY> = BTreeMap::new();
        for id in army.iter().copied().filter(|&id| idle(w, id)) {
            let Some(at) = w.obj(id).map(|o| o.pos.cell()) else {
                continue;
            };
            // design: own AI (#28): only an enemy the unit can fire at from where it stands, or reaches (its reach region, or its weapon's range from a cell of it, S-0212 claim 4) as the wave asks; an attack it could not carry out pinned it where it stood
            let mut region: Option<Option<u32>> = None;
            let near = enemies
                .iter()
                .filter(|&&(e, c)| {
                    dist2(c, at) <= ENGAGE * ENGAGE
                        && can_attack(w, id, e)
                        // spec: S-0215 claim 37 (b) (the range test of S-0307 with the weapon chosen for the target)
                        && (glasnost_sim::approach::in_range(w, id, Target::Object(e))
                            || self.reaches(
                                w,
                                id,
                                *region.get_or_insert_with(|| self.reach_region(w, id)),
                                e,
                                c,
                            ))
                })
                .min_by_key(|&&(_, c)| dist2(c, at));
            if let Some(&(e, _)) = near {
                out.push(Order::Behaviour {
                    actor: id,
                    behaviour: Behaviour::Attack,
                    attack_move: false,
                    target: Some(Target::Object(e)),
                    dest: None,
                });
            } else if let Some(h) = home
                && let Some(g) = {
                    let zone = w
                        .obj(id)
                        .and_then(|o| w.rules().unit(o.kind, o.type_index))
                        .map(|u| u.zone);
                    *ahead
                        .entry(zone.map(|z| z as u8))
                        .or_insert_with(|| self.base_goal_for(w, zone))
                }
                && self.needs_ferry(w, id, g)
            {
                // Infantry that cannot walk to the goal waits at home for
                // a ferry (the brain's ferry takes idle infantry from
                // there) instead of standing at the nearest shore.
                // design: own AI (#29): the nearest region cells to home are kept for the think (one search a region, not one a unit), and a unit that already stands at the cell nearest home it can reach (stranded on an islet) is not sent there again at every think
                let dest = (dist2(at, h) > FERRY_GATHER * FERRY_GATHER)
                    .then(|| self.reachable(w, id, h, &mut gather))
                    .filter(|&d| dist2(d, at) > ARRIVED * ARRIVED);
                if let Some(dest) = dest {
                    out.push(Order::Behaviour {
                        actor: id,
                        behaviour: Behaviour::Move,
                        attack_move: false,
                        target: None,
                        dest: Some(Target::Cell(dest)),
                    });
                }
            } else if !crate::lifter::is_lifter(w, id) {
                // design: own AI (#13): a lifter keeps to its own part (`lifter::keeps_order` drops a wave's order to it), so it does not count toward a wave
                waiting.push(id);
            }
        }
        mark("ai:script:waves");
        let since = *self.waiting_since.get_or_insert(w.tick);
        if waiting.is_empty() {
            self.waiting_since = None;
        }
        // design: a guard stays home when a wave goes, a share of the
        // army between GUARD_MIN and GUARD_MAX: its units nearest home, whether they wait or fight
        // there; no wave goes before the opening is over; each wave asks
        // for more units than the last, and a wave that goes for patience
        // alone is at least half the size asked for (M8 step A4 round 8,
        // the lead's note 52: every unit went as soon as six waited, from
        // the start of the game)
        // design: the end-game (M8 step A4 round 9b): a house clearly
        // winning (`finishing`: it has looked at the enemy base and what it
        // sees fight there is worth a third of its army or less) sends its
        // waves at the plan's first size, not grown, while the enemy can
        // still build, its guard kept; once nothing it sees of the enemy
        // builds, a wave goes with whatever waits, and with one enemy house
        // left the guard goes too (in the synthetic skirmish three tanks
        // stood at home, two as the guard, before an enemy left with a lab
        // and a harvester, for good)
        let winning = if w.tick >= OPENING {
            self.finishing(w, &enemies)
        } else {
            None
        };
        let finish = winning == Some(false);
        let guard_n = if finish && self.enemy_houses_left(w) <= 1 {
            0
        } else {
            (army.len() / GUARD_SHARE).clamp(GUARD_MIN, GUARD_MAX)
        };
        if let Some(h) = home {
            let mut by_home: Vec<(i64, ObjectId)> = army
                .iter()
                .filter_map(|&id| w.obj(id).map(|o| (dist2(o.pos.cell(), h), id)))
                .collect();
            by_home.sort_unstable();
            let guard: BTreeSet<ObjectId> = by_home
                .into_iter()
                .take(guard_n)
                .map(|(_, id)| id)
                .collect();
            waiting.retain(|id| !guard.contains(id));
        }
        let size = match winning {
            Some(false) => 1,
            Some(true) => self.plan.wave.max(1),
            None => (self.plan.wave + WAVE_GROWTH * self.waves)
                .min(WAVE_MAX.max(self.plan.wave))
                .max(1),
        };
        let goes = |n: usize| {
            n >= size || (n >= size.div_ceil(2) && w.tick - since >= WAVE_PATIENCE) // nocite: tuning of our own opponent (half a wave)
        };
        // The units the scout is chosen from when no wave goes.
        let mut scouting = waiting.clone();
        if w.tick >= OPENING && goes(waiting.len()) {
            // Each unit goes to the seen enemy nearest home that it can
            // reach and attack (a person does not send riflemen to stand at
            // the shore by a ship), else all go to the next enemy start cell.
            // design: the seen enemy of the house whose building stands
            // nearest home comes first, so that in a game of three or more
            // a wave takes on one enemy (M8 step A4 round 8, note 52)
            let home = self.home(w);
            let from = w.obj(waiting[0]).map(|o| o.pos.cell());
            let mut by_home: Vec<(ObjectId, CellXY)> = enemies.clone();
            if let Some(h) = home {
                let focus = enemies
                    .iter()
                    .filter_map(|&(e, c)| {
                        let o = w.obj(e)?;
                        (o.kind == Kind::Building).then_some((dist2(c, h), e, o.owner))
                    })
                    .min()
                    .and_then(|(_, _, owner)| owner);
                let other =
                    |e: ObjectId| focus.is_some_and(|f| w.obj(e).and_then(|o| o.owner) != Some(f));
                by_home.sort_by_key(|&(e, c)| (other(e), dist2(c, h), e));
            }
            let mut picks: BTreeMap<(Kind, u32, Option<u32>), Option<CellXY>> = BTreeMap::new();
            let mut fallback: Option<Option<CellXY>> = None;
            // Whether the fallback took the next start in turn.
            let mut rotated = false;
            // The nearest region cells, kept per goal (`reachable`).
            let mut near: BTreeMap<CellXY, BTreeMap<(u8, u32), CellXY>> = BTreeMap::new();
            // The units the wave would send, with their destinations.
            let mut sends: Vec<(ObjectId, CellXY)> = Vec::new();
            for &id in &waiting {
                let Some(o) = w.obj(id) else { continue };
                let at = o.pos.cell();
                let region = self.reach_region(w, id);
                let key = (o.kind, o.type_index, region);
                let pick = *picks.entry(key).or_insert_with(|| {
                    by_home
                        .iter()
                        .find(|&&(e, c)| can_attack(w, id, e) && self.reaches(w, id, region, e, c))
                        .map(|&(_, c)| c)
                });
                let goal = pick.or_else(|| {
                    *fallback.get_or_insert_with(|| {
                        let targets = self.enemy_starts(w);
                        // design: the nearest start that may still hold an enemy base (`next_start`), M8 step A4 round 8
                        targets
                            .iter()
                            .copied()
                            .find(|&c| !self.cleared(w, c, &enemies))
                            .or_else(|| from.and_then(|f| self.unseen_cell(w, f, id)))
                            .or_else(|| {
                                (!targets.is_empty()).then(|| {
                                    rotated = true;
                                    targets[self.next_target % targets.len()]
                                })
                            })
                    })
                });
                if let Some(c) = goal {
                    let dest = self.reachable(w, id, c, near.entry(c).or_default());
                    // design: own AI (#11): a unit whose destination is where it stands (a ship off an inland enemy, at the water nearest it; a vehicle stranded on an island) is parked: it does not count toward the wave and is not sent again
                    if dist2(dest, at) > ARRIVED * ARRIVED {
                        sends.push((id, dest));
                    }
                }
            }
            if goes(sends.len()) {
                self.waiting_since = None;
                // design: the end-game's waves do not make the next battle's wave larger
                if winning.is_none() {
                    self.waves += 1;
                }
                if rotated {
                    self.next_target += 1;
                }
                for (id, dest) in sends {
                    out.push(Order::Behaviour {
                        actor: id,
                        behaviour: Behaviour::Move,
                        attack_move: true,
                        target: None,
                        dest: Some(Target::Cell(dest)),
                    });
                }
                return out;
            }
            // Parked units do not keep the patience running.
            if sends.is_empty() {
                self.waiting_since = None;
            }
            scouting = sends.into_iter().map(|(id, _)| id).collect();
        }
        if let Some(h) = home {
            self.send_scout(w, h, &scouting, &mut out);
        }
        out
    }

    /// design: while no wave goes, one army unit at a time, the fastest of
    /// those waiting (not the guard), explores the house's own side of the
    /// map ([`Script::scout_cell`]), so the house finds the tech buildings
    /// near it while its army builds up; a person scouts for oil derricks
    /// first thing (M8 step A4 round 8, the lead's note 48: with no early
    /// wave the house saw them late). The scout is kept until it is idle
    /// again or gone.
    fn send_scout(&mut self, w: &World, h: CellXY, waiting: &[ObjectId], out: &mut Vec<Order>) {
        self.scout = self
            .scout
            .filter(|&s| w.obj(s).is_some_and(|o| o.on_map && o.health > 0) && !idle(w, s));
        if self.scout.is_some() {
            return;
        }
        let speed = |id: ObjectId| {
            w.obj(id)
                .and_then(|o| w.rules().unit(o.kind, o.type_index))
                .map_or(0, |u| u.speed)
        };
        let Some(s) = waiting
            .iter()
            .copied()
            .filter(|&id| self.reach_region(w, id).is_some())
            .max_by_key(|&id| (speed(id), std::cmp::Reverse(id)))
        else {
            return;
        };
        if let Some(c) = self.scout_cell(w, s, h) {
            out.push(Order::Behaviour {
                actor: s,
                behaviour: Behaviour::Move,
                attack_move: false,
                target: None,
                dest: Some(Target::Cell(c)),
            });
            self.scout = Some(s);
        }
    }

    /// The reach region unit `id` stands in for its movement zone (S-0212
    /// claim 4); `None` for a unit that flies (S-0502), has no zone or
    /// stands outside every region.
    pub(crate) fn reach_region(&self, w: &World, id: ObjectId) -> Option<u32> {
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
    /// goes for enemy `e` at cell `c`: `c` lies in that region, or the unit
    /// has none, or it is a ship (it fires from the water nearest the
    /// enemy, as a person sends ships to the water off an enemy base), or
    /// `e` stands on the land of another region and the unit's weapon
    /// reaches it from a cell of its own region (a person drives
    /// a long-range gun to the shore facing an enemy across the water;
    /// [`Script::reachable`] then sends it to the region's cell nearest
    /// `c`, which is in range if any is).
    pub(crate) fn reaches(
        &self,
        w: &World,
        id: ObjectId,
        region: Option<u32>,
        e: ObjectId,
        c: CellXY,
    ) -> bool {
        let Some(r) = region else { return true };
        let Some(zone) = w
            .obj(id)
            .and_then(|o| w.rules().unit(o.kind, o.type_index))
            .map(|u| u.zone)
        else {
            return true;
        };
        if zone == glasnost_rules::sim::MovementZone::Water
            || regions::reach_in(w, c, zone) == Reach::Region(r)
        {
            return true;
        }
        // design: a ground unit whose weapon reaches an enemy on the land of another region from a usable cell of its own region goes for it, so that a long-range gun is not left at home on an island map (M8 AI seams: the launcher of the island tech climb sat 22 cells from enemies on the far shore while a shore cell had them at 10); an enemy on the water is not one (a person does not send a tank to stand at the shore by a ship)
        if !matches!(regions::reach_in(w, c, zone), Reach::Region(_)) {
            return false;
        }
        let (slot, wp) = weapon::would_use(w, id, Some(Target::Object(e)));
        if wp.is_none() {
            return false;
        }
        // spec: S-0307 claim 13 (the firer's weapon range, leptons)
        let range = i64::from(range::firer_weapon_range(w, id, slot));
        // spec: S-0204 claim 2 (a cell is 256 leptons)
        let lepton = 256i64;
        let cells = (range + lepton - 1) / lepton;
        let span = i32::try_from(cells).unwrap_or(i32::MAX);
        (-span..=span).any(|dy| {
            (-span..=span).any(|dx| {
                let k = CellXY::new(c.x + dx, c.y + dy);
                dist2(k, c) * lepton * lepton <= range * range
                    && w.cells.usable(k)
                    && regions::reach_in(w, k, zone) == Reach::Region(r)
            })
        })
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
    /// usable area, that the house does not see now and that unit `id` can
    /// go to: in its reach region (S-0212 claim 4) when it has one
    /// ([`Script::reach_region`]); the first in cell order on equal
    /// distance.
    fn unseen_cell(&self, w: &World, from: CellXY, id: ObjectId) -> Option<CellXY> {
        let region = self.reach_region(w, id);
        let zone = w
            .obj(id)
            .and_then(|o| w.rules().unit(o.kind, o.type_index))
            .map(|u| u.zone);
        // design: own AI (#10): a search cell the wave cannot reach (the middle of a lake) stays unseen when the wave stands on the nearest shore, and it was picked again on every think; only cells of the unit's own region are searched
        let goes = |c: CellXY| match (region, zone) {
            (Some(r), Some(z)) => regions::reach_in(w, c, z) == Reach::Region(r),
            _ => true,
        };
        w.cells
            .grid
            .iter()
            .map(|(c, _)| c)
            .filter(|c| c.x % SEARCH_STEP == 0 && c.y % SEARCH_STEP == 0)
            .filter(|&c| w.cells.usable(c) && !shroud::seen(w, self.house, c) && goes(c))
            .min_by_key(|&c| (dist2(c, from), c.y, c.x))
    }

    /// Where scout `s` goes from home `h`: the cell of the search grid
    /// nearest `h` that the house has not explored (S-0409 Terms), in the
    /// usable area and in the scout's reach region (S-0212 claim 4), and
    /// nearer `h` than every enemy start; the first in cell order on equal
    /// distance. `None` when the house's side is explored.
    fn scout_cell(&self, w: &World, s: ObjectId, h: CellXY) -> Option<CellXY> {
        let o = w.obj(s)?;
        let zone = w.rules().unit(o.kind, o.type_index)?.zone;
        let Reach::Region(r) = regions::reach_in(w, o.pos.cell(), zone) else {
            return None;
        };
        let starts = self.enemy_starts(w);
        w.cells
            .grid
            .iter()
            .map(|(c, _)| c)
            .filter(|c| c.x % SEARCH_STEP == 0 && c.y % SEARCH_STEP == 0)
            .filter(|&c| {
                let d = dist2(c, h);
                w.cells.usable(c)
                    && !shroud::explored(w, self.house, c)
                    && starts.iter().all(|&e| d < dist2(c, e))
                    && regions::reach_in(w, c, zone) == Reach::Region(r)
            })
            .min_by_key(|&c| (dist2(c, h), c.y, c.x))
    }

    /// Whether the house is clearly winning, as a person reads it off the
    /// map, and if so whether the enemy can still build: `None` when it is
    /// not; `Some(builds)` when it knows where the enemy is left (it sees an
    /// enemy building and has explored the enemy start cell nearest each
    /// one it sees: it has looked at the enemy base, not only at a building
    /// a scout came by; or it sees no enemy building and every enemy start
    /// is cleared, so what is left of the enemy is to be searched for) and
    /// what it sees of the enemy that fights (army units, and armed
    /// buildings: S-0214 Terms, a weapon in the slot it uses) is worth, by
    /// cost, at most a [`FINISH_SHARE`]th of its own army. `builds` is
    /// whether an enemy object it sees can build (a factory or yard
    /// building, or a vehicle that deploys into a yard). design: such an
    /// enemy cannot stop the house's army; one that can build may yet raise
    /// a raid (M8 step A4 round 9b; in the synthetic skirmish the enemy's
    /// last harvester stood on cells the house had not explored, and the
    /// house's tanks stood at the cleared enemy base for good).
    pub(crate) fn finishing(&self, w: &World, enemies: &[(ObjectId, CellXY)]) -> Option<bool> {
        let worth = |o: &Object| i64::from(production::cost_field(w, tech::type_of(o)));
        let starts = self.enemy_starts(w);
        let mut building = false;
        let mut builds = false;
        let mut theirs: i64 = 0;
        for (o, c) in enemies.iter().filter_map(|&(e, c)| Some((w.obj(e)?, c))) {
            if o.kind == Kind::Building {
                building = true;
                let base = starts
                    .iter()
                    .copied()
                    .min_by_key(|&s| (dist2(s, c), s.y, s.x));
                if base.is_some_and(|s| !shroud::explored(w, self.house, s)) {
                    return None;
                }
                builds |= w
                    .rules()
                    .buildings
                    .get(o.type_index as usize)
                    .is_some_and(|b| b.factory.is_some());
                if glasnost_sim::combat::weapon::armed(w, o.id) {
                    theirs += worth(o);
                }
            } else if deploys_into_yard(w, o.kind, o.type_index) {
                builds = true;
            } else if role_of(w, tech::type_of(o)) == Some(Role::Army) {
                theirs += worth(o);
            }
        }
        let ours: i64 = self
            .of_role(w, Role::Army)
            .iter()
            .filter_map(|&id| w.obj(id))
            .map(worth)
            .sum();
        let known =
            building || (!starts.is_empty() && starts.iter().all(|&s| self.cleared(w, s, enemies)));
        (known && theirs * FINISH_SHARE <= ours).then_some(builds)
    }

    /// The enemy player houses still in the game: those that are not
    /// defeated, which a person is told of (S-0414 claim 5: a message names
    /// the defeated house), in the house's view of the alliances.
    pub(crate) fn enemy_houses_left(&self, w: &World) -> usize {
        let me = &w.houses[usize::from(self.house.0)];
        (0..w.houses.len())
            .map(|i| HouseId(i as u8))
            .filter(|&h| {
                let x = &w.houses[usize::from(h.0)];
                h != self.house
                    && !me.allied_with(h)
                    && x.role == glasnost_sim::world::HouseRole::Player
                    && !x.removed
                    && !glasnost_sim::econ::victory::defeated(w, h)
            })
            .count()
    }

    /// Enemy objects on cells the house has explored, with their cells: fog
    /// of war is off in a skirmish (S-0409 claim 19), so a person in the
    /// house's seat sees what stands on every explored cell, except a
    /// cloaked object the house does not sense (S-0516 claim 28) and a
    /// unit it is shown as its disguise (claims 35 and 37); a building is
    /// seen by any explored cell of its foundation (#37).
    /// One scan a think, the list kept for the rest of it (design: own AI,
    /// #53 and #59).
    pub(crate) fn seen_enemies(&self, w: &World) -> Vec<(ObjectId, CellXY)> {
        self.placed
            .keep(|m| &mut m.seen, || self.scan_seen_enemies(w).into())
            .to_vec()
    }

    /// [`Script::seen_enemies`]'s scan. The cloak is asked only of an
    /// enemy (it only reads the world, so the answer is the same as asking
    /// it of every object first; design: own AI, #53).
    fn scan_seen_enemies(&self, w: &World) -> Vec<(ObjectId, CellXY)> {
        let me = &w.houses[usize::from(self.house.0)];
        w.objects
            .values()
            .filter(|o| o.on_map && o.health > 0)
            .filter_map(|o| {
                let owner = o.owner?;
                let h = w.houses.get(usize::from(owner.0))?;
                // The house's view of the alliance, as the sim's targeting reads it (S-0309 claim 18).
                let enemy = owner != self.house
                    && !me.allied_with(owner)
                    && h.role == glasnost_sim::world::HouseRole::Player;
                let c = o.pos.cell();
                // A cloaked enemy the house does not sense is not seen (S-0516 claim 28: not drawn for it); a building by any explored cell of it (`crate::in_view`, #37).
                let seen = || crate::in_view(w, self.house, o.id);
                // design: own AI (#37): an enemy unit the house is shown as its disguise is taken for what it shows (S-0516 claims 35 and 37)
                let disguised = || crate::shown_as_disguise(w, self.house, o.id);
                (enemy && seen() && !disguised()).then_some((o.id, c))
            })
            .collect()
    }

    /// The first cell of a square spiral around the construction yard where
    /// a place order for `t` is legal, the cell test passes and the
    /// foundation is free. A naval building is looked for in the wider
    /// spiral, as a person looks for the water near the base.
    pub fn place_cell(&self, w: &World, t: TypeRef) -> Option<CellXY> {
        if let Some(found) = self.placed.get(t) {
            return found;
        }
        let found = self.search_place_cell(w, t);
        self.placed.put(t, found);
        found
    }

    /// [`Script::place_cell`]'s search. The gap narrows pass by pass, and
    /// only the free-gap test depends on the gap: the order's legality and
    /// the cell test of each spiral cell are asked once, on its first pass,
    /// and remembered for the later ones (the same answer as asking every
    /// pass, in the same order, with less work). The legality and the cell
    /// test are asked through one `placement::PlaceSearch` for the type,
    /// the cell test first (both only read the world, so either order gives
    /// the same answer, and a naval yard's wide spiral is mostly land,
    /// where the cell test fails at once), and the free-gap test reads the
    /// objects the house knows of, listed once for the search
    /// ([`Script::known_objects`]).
    fn search_place_cell(&self, w: &World, t: TypeRef) -> Option<CellXY> {
        let rings = if naval_building(w, t) {
            WIDE_RINGS
        } else {
            SPIRAL_RINGS
        };
        let cells: Vec<CellXY> = self.spiral(w, rings).collect();
        let mut legal: Vec<Option<bool>> = vec![None; cells.len()];
        let search = placement::PlaceSearch::new(w, self.house, t, &cells);
        let p = search.place_type();
        let free = self.free_test(w);
        // A cramped base (a start in a small clearing) has no cell with the
        // usual gap: the gap narrows, as a person squeezes a building in
        // when the room is short, down to none.
        (0..=GAP).rev().find_map(|gap| {
            cells.iter().zip(legal.iter_mut()).find_map(|(&c, l)| {
                let ok = *l.get_or_insert_with(|| {
                    // The cell test of `cell_fits` (an upgrade needs only the place order's conditions).
                    (p.upgrade_of.is_some() || search.cell_test(w, c)) && search.place_legal(w, c)
                });
                (ok && free(&placement::foundation_cells_of(p, c), gap)).then_some(c)
            })
        })
    }

    /// The first cell of a square spiral around `goal`, ring by ring up to
    /// [`SPIRAL_RINGS`], where a place order for `t` is legal, the cell
    /// test passes and the foundation is free, the gap narrowing as in
    /// [`Script::place_cell`]: a building placed where it is needed, as a
    /// person drops a defence beside the building under attack (M8 step
    /// B6, `antiair::place_goal`).
    pub(crate) fn place_near(&self, w: &World, t: TypeRef, goal: CellXY) -> Option<CellXY> {
        let ring = |r: i32| {
            (-r..=r).flat_map(move |dy| {
                (-r..=r)
                    .filter(move |dx| dx.abs() == r || dy.abs() == r)
                    .map(move |dx| CellXY::new(goal.x + dx, goal.y + dy))
            })
        };
        (0..=GAP).rev().find_map(|gap| {
            (0..=SPIRAL_RINGS)
                .flat_map(ring)
                .find(|&c| self.fits_gap(w, t, c, gap))
        })
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
        self.order_fits(w, t, c) && self.free_gap(w, t, c, None, gap)
    }

    /// The part of [`Script::fits_gap`] that does not depend on the gap.
    fn order_fits(&self, w: &World, t: TypeRef, c: CellXY) -> bool {
        // The place order's conditions and the cell test (S-0406 claims 3 to 6, 8 and 12).
        placement::place_legal(w, self.house, t, c) && self.cell_fits(w, t, c)
    }

    /// The cell test of [`Script::order_fits`] (S-0406 claims 3 and 8); an
    /// upgrade needs only the place order's conditions.
    fn cell_fits(&self, w: &World, t: TypeRef, c: CellXY) -> bool {
        placement::place_type(w, t.index).upgrade_of.is_some()
            || placement::cell_test(w, t.index, c, &placement::Test::order(self.house))
    }

    /// The water the base grows toward: when a naval factory type that the
    /// house has none of is buildable but has no cell to stand on, the
    /// nearest explored site within the wide spiral where its foundation
    /// would stand but for the distance to the base (the cursor shows the
    /// foundation fits, S-0406 claims 3 and 5, and only the adjacency of
    /// claim 4 fails), the first in cell order on equal distance. With no
    /// such site the base does not grow: a person does not lay buildings
    /// toward water too small or too far for the yard (M7b AI audit: on a
    /// map with a narrow strip of water the base walked nine power plants
    /// toward it and starved the economy).
    pub(crate) fn water_goal(&self, w: &World) -> Option<CellXY> {
        let home = self.home(w)?;
        let stuck: Vec<u32> = w
            .rules()
            .buildings
            .iter()
            .enumerate()
            .filter(|(i, b)| {
                let t = TypeRef {
                    kind: Kind::Building,
                    index: *i as u32,
                };
                b.naval
                    && b.factory.is_some()
                    && tech::present_count(w, self.house, t) == 0
                    && self.buildable(w, t)
                    && !self.placeable(w, t)
            })
            .map(|(i, _)| i as u32)
            .collect();
        if stuck.is_empty() {
            return None;
        }
        let test = placement::Test::order(self.house);
        let r = WIDE_RINGS;
        (-r..=r)
            .flat_map(|dy| (-r..=r).map(move |dx| CellXY::new(home.x + dx, home.y + dy)))
            .filter(|&c| {
                w.cells
                    .grid
                    .get(c)
                    .is_some_and(|cell| cell.land == glasnost_map::terrain::land::WATER)
                    && stuck.iter().any(|&t| {
                        placement::explored(w, self.house, t, c)
                            && placement::cell_test(w, t, c, &test)
                    })
            })
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

    /// The explored resource cell a refinery goes beside: the one nearest
    /// home (first in cell order on equal distance) with at least
    /// [`CELLS_PER_HARVESTER`] explored resource cells within
    /// [`FIELD_REACH`] cells, a field that keeps a harvester at work; with
    /// none, the explored resource cell nearest home.
    fn ore_goal(&self, w: &World) -> Option<CellXY> {
        let home = self.home(w)?;
        let reach = self.harvester_reach(w, home);
        let mut ore: Vec<CellXY> = w
            .cells
            .grid
            .iter()
            .map(|(c, _)| c)
            .filter(|&c| {
                w.cells.usable(c)
                    && resource::resource_at(w, c).is_some()
                    && shroud::explored(w, self.house, c)
                    && (reach.is_empty()
                        || reach
                            .iter()
                            .any(|&(z, r)| regions::reach_in(w, c, z) == Reach::Region(r)))
            })
            .collect();
        ore.sort_by_key(|&c| (dist2(c, home), c.y, c.x));
        let reach = i64::from(FIELD_REACH) * i64::from(FIELD_REACH);
        let rich = |c: CellXY| {
            ore.iter().filter(|&&o| dist2(o, c) <= reach).count() >= CELLS_PER_HARVESTER
        };
        ore.iter()
            .copied()
            .find(|&c| rich(c))
            .or_else(|| ore.first().copied())
    }

    /// The reach regions (S-0212 claim 4) the house's harvesters drive
    /// in, each with its movement zone: those its harvesters stand in
    /// (not slave miners, which go by their own rules, S-0402 claim 14,
    /// nor one that flies), or, with none on the map, those of the cells
    /// within [`HOME_RING`] cells of `home` for the zone of each harvester
    /// type of its plan, where one would leave the base. A person sends a
    /// harvester only to a field its order can reach (M8 step B9: the
    /// pathing query a person's order gives); empty when nothing is known,
    /// and then no field is left out.
    fn harvester_reach(&self, w: &World, home: CellXY) -> Vec<(MovementZone, u32)> {
        let mut out: Vec<(MovementZone, u32)> = Vec::new();
        let mut add = |z: MovementZone, c: CellXY| {
            if let Reach::Region(r) = regions::reach_in(w, c, z)
                && !out.contains(&(z, r))
            {
                out.push((z, r));
            }
        };
        let mut any = false;
        for id in self.of_role(w, Role::Harvester) {
            let Some(o) = w.obj(id) else { continue };
            if is_slave_miner(w, tech::type_of(o)) || crate::jumpjets::flies(w, id) {
                continue;
            }
            if let Some(u) = w.rules().unit(o.kind, o.type_index) {
                any = true;
                add(u.zone, o.pos.cell());
            }
        }
        if !any {
            // Every harvester type of the plan, buildable now or not: a
            // house with no harvester yet has no war factory either, and
            // its first harvester may come later or with a refinery
            // (S-0404 claim 2).
            let types = all_types(w)
                .into_iter()
                .filter(|&t| self.allowed(w, t, Role::Harvester));
            for t in types {
                if is_slave_miner(w, t) {
                    continue;
                }
                let Some(u) = w.rules().unit(t.kind, t.index) else {
                    continue;
                };
                for dy in -HOME_RING..=HOME_RING {
                    for dx in -HOME_RING..=HOME_RING {
                        add(u.zone, CellXY::new(home.x + dx, home.y + dy));
                    }
                }
            }
        }
        out
    }

    /// Where a person puts a refinery (M8 step B8): in the wider spiral
    /// around the construction yard, the legal cell whose foundation is
    /// nearest the field of [`Script::ore_goal`], with no resource under
    /// it, the gap narrowing as in [`Script::place_cell`]. A harvester
    /// leaves its refinery for the field after every load, so a refinery
    /// beside the field shortens every trip; the spiral's first legal cell
    /// lies wherever the spiral starts (M8 step B8: in the quick lane the
    /// Allied miners loaded 10 to 16 cells from their refinery and spent a
    /// third of each cycle on the way out). None for a house whose
    /// refineries are slave miners (S-0402 claim 14), for a type that keeps
    /// slaves ([`keeps_slaves`]: a slave miner's building goes where the
    /// slave-miner rules send it; before its first war factory a Yuri-style
    /// house builds no slave-miner vehicle, M8 step B8), or for another
    /// role.
    pub(crate) fn place_refinery(&self, w: &World, t: TypeRef) -> Option<CellXY> {
        if role_of(w, t) != Some(Role::Refinery) || self.slave_miners(w) || keeps_slaves(w, t) {
            return None;
        }
        let goal = self.ore_goal(w)?;
        self.refinery_site(w, t, goal, None)
    }

    /// [`Script::place_refinery`]'s search toward `goal`, among the cells
    /// whose foundation centre lies within `within` (a squared distance)
    /// of it when given.
    fn refinery_site(
        &self,
        w: &World,
        t: TypeRef,
        goal: CellXY,
        within: Option<i64>,
    ) -> Option<CellXY> {
        let search = {
            let cells: Vec<CellXY> = self.spiral(w, WIDE_RINGS).collect();
            placement::PlaceSearch::new(w, self.house, t, &cells)
        };
        let p = search.place_type();
        let (fw, fh) = p.size;
        let centre = |c: CellXY| CellXY::new(c.x + fw / 2, c.y + fh / 2); // nocite: the middle cell of the foundation
        let mut cells: Vec<CellXY> = self
            .spiral(w, WIDE_RINGS)
            .filter(|&c| within.is_none_or(|r| dist2(centre(c), goal) <= r))
            .collect();
        cells.sort_by_key(|&c| (dist2(centre(c), goal), c.y, c.x));
        let mut legal: Vec<Option<bool>> = vec![None; cells.len()];
        let free = self.free_test(w);
        (0..=GAP).rev().find_map(|gap| {
            cells.iter().zip(legal.iter_mut()).find_map(|(&c, l)| {
                let ok = *l.get_or_insert_with(|| {
                    search.cell_test(w, c)
                        && search.place_legal(w, c)
                        && placement::foundation_cells_of(p, c)
                            .iter()
                            .all(|&f| resource::resource_at(w, f).is_none())
                });
                (ok && free(&placement::foundation_cells_of(p, c), gap)).then_some(c)
            })
        })
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
        self.free_cells(w, &cells, gap, but)
    }

    /// Whether foundation cells `cells` hold none of the objects the house
    /// knows but `but` and keep a gap of `gap` cells to its buildings
    /// ([`free_among`]); during a think, with no object left out, read off
    /// the think's [`Blocked`] map (the same answer).
    fn free_cells(&self, w: &World, cells: &[CellXY], gap: i32, but: Option<ObjectId>) -> bool {
        if but.is_none()
            && self.placed.thinking()
            && let Some(free) = self.blocked(w).free(cells, gap)
        {
            return free;
        }
        free_among(&self.known_objects(w), cells, gap, but)
    }

    /// The free-gap test with no object left out, for a search that asks it
    /// of many cells: [`Blocked`] during a think, else [`free_among`] over
    /// the objects listed once (the same answers).
    fn free_test(&self, w: &World) -> impl Fn(&[CellXY], i32) -> bool {
        let blocked = self.placed.thinking().then(|| self.blocked(w));
        let known = self.known_objects(w);
        move |cells, gap| {
            blocked
                .as_ref()
                .and_then(|b| b.free(cells, gap))
                .unwrap_or_else(|| free_among(&known, cells, gap, None))
        }
    }

    /// The think's [`Blocked`] map of the objects the house knows.
    fn blocked(&self, w: &World) -> Arc<Blocked> {
        self.placed.keep(
            |m| &mut m.blocked,
            || Arc::new(Blocked::new(&self.known_objects(w), w.cells.grid.len())),
        )
    }

    /// The objects the free-gap test looks at, in object order: only what
    /// the house knows, its own objects and those on cells it has explored
    /// (the AI sees only its own house's shroud; fog is off in a skirmish,
    /// S-0409 claim 19), on the map. One scan a think, the list kept for
    /// the rest of it (design: own AI, #52 and #57: the scan was made for
    /// every cell a spiral tested); [`free_among`] leaves out the object a
    /// test does not count.
    fn known_objects(&self, w: &World) -> Arc<[Known]> {
        self.placed
            .keep(|m| &mut m.known, || self.scan_known_objects(w).into())
    }

    /// [`Script::known_objects`]'s scan.
    fn scan_known_objects(&self, w: &World) -> Vec<Known> {
        w.objects
            .values()
            .filter(|o| {
                o.on_map
                    && (o.owner == Some(self.house)
                        || shroud::explored(w, self.house, o.pos.cell()))
            })
            .map(|o| Known {
                id: o.id,
                // A building's position is its top-left cell's centre.
                top_left: o.pos.cell(),
                foundation: (o.kind == Kind::Building).then_some(o.foundation),
            })
            .collect()
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

#[cfg(test)]
mod place_cell_tests {
    use super::*;
    use crate::{fixture, game::Game};

    /// The placement search as it was before M8 C4 part 2: every pass asks
    /// every spiral cell the whole test, the place order's conditions first
    /// and the free gap as it was before M8 C4 part 6.
    fn three_passes(s: &Script, w: &World, t: TypeRef) -> Option<CellXY> {
        let rings = if naval_building(w, t) {
            WIDE_RINGS
        } else {
            SPIRAL_RINGS
        };
        (0..=GAP).rev().find_map(|gap| {
            s.spiral(w, rings)
                .find(|&c| s.order_fits(w, t, c) && free_gap_as_before(s, w, t, c, gap))
        })
    }

    /// The free-gap test as it was before M8 C4 part 6: each object the
    /// house knows asked in turn.
    fn free_gap_as_before(s: &Script, w: &World, t: TypeRef, c: CellXY, gap: i32) -> bool {
        let cells = placement::foundation_cells(w, t.index, c);
        w.objects
            .values()
            .filter(|o| {
                o.on_map && (o.owner == Some(s.house) || shroud::explored(w, s.house, o.pos.cell()))
            })
            .all(|o| {
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

    /// The skirmishes the placement tests play: a roomy base, a cramped one
    /// (where the gap narrows), a sea along one side and a strait (where a
    /// naval yard's wide spiral is mostly land and the cell test and the
    /// adjacency disagree).
    fn placement_maps() -> Vec<(&'static str, glasnost_map::Map)> {
        vec![
            ("synthetic", fixture::map()),
            ("synthetic-cramped", fixture::map_cramped(5)),
            ("synthetic-naval", fixture::map_naval()),
            ("synthetic-strait", fixture::map_strait()),
        ]
    }

    /// Our tooling for speed (M8 C4 part 6): a placement search for a house
    /// and a building type answers every cell of the map, inside its window
    /// and outside it, as the place order's conditions (S-0406 claims 3 to
    /// 6 and 12) and the cell test (claims 3 and 8) answer it asked afresh,
    /// at several points of a game, for every house.
    #[test]
    fn the_placement_search_answers_every_cell_as_the_place_order_and_the_cell_test() {
        let mut compared = 0;
        let mut legal = 0;
        let mut tested = 0;
        for (name, map) in placement_maps() {
            let mut g = Game::new(
                fixture::rules(),
                &map,
                name,
                fixture::catalogue(),
                fixture::setup(3),
            )
            .expect("the skirmish loads");
            for _ in 0..3 {
                for _ in 0..800 {
                    g.step();
                }
                let w = &g.world;
                let all: Vec<CellXY> = w.cells.grid.iter().map(|(c, _)| c).collect();
                for h in 0..w.houses.len() {
                    let house = HouseId(h as u8);
                    let s = Script::new(house, Plan::default());
                    for t in all_types(w)
                        .into_iter()
                        .filter(|t| t.kind == Kind::Building)
                    {
                        let rings = if naval_building(w, t) {
                            WIDE_RINGS
                        } else {
                            SPIRAL_RINGS
                        };
                        let window: Vec<CellXY> = s.spiral(w, rings).collect();
                        let search = placement::PlaceSearch::new(w, house, t, &window);
                        let test = placement::Test::order(house);
                        for &c in &all {
                            let want = placement::place_legal(w, house, t, c);
                            assert_eq!(
                                search.place_legal(w, c),
                                want,
                                "{name}, house {h}, {t:?}, {c:?}"
                            );
                            let want_cell = placement::cell_test(w, t.index, c, &test);
                            assert_eq!(
                                search.cell_test(w, c),
                                want_cell,
                                "{name}, house {h}, {t:?}, {c:?}"
                            );
                            compared += 1;
                            legal += usize::from(want);
                            tested += usize::from(want_cell);
                        }
                    }
                }
            }
        }
        assert!(legal > 0 && legal < compared, "{legal} of {compared}");
        assert!(tested > 0 && tested < compared, "{tested} of {compared}");
    }

    /// Our tooling for speed (M8 C4 parts 2 and 6): the search that asks
    /// each cell's legality once, through one placement search and the
    /// objects listed once, and the answers kept for a think, give every
    /// house the cell the three passes give, for every building type, at
    /// several points of a game, on each of the placement maps.
    #[test]
    fn the_kept_and_once_asked_placement_answers_match_the_three_passes() {
        let maps = placement_maps();
        let mut compared = 0;
        let mut found = 0;
        let mut unblocked = 0;
        for (name, map) in maps {
            let mut g = Game::new(
                fixture::rules(),
                &map,
                name,
                fixture::catalogue(),
                fixture::setup(3),
            )
            .expect("the skirmish loads");
            for _ in 0..4 {
                for _ in 0..600 {
                    g.step();
                }
                let w = &g.world;
                let houses = w.houses.len();
                for h in 0..houses {
                    let s = Script::new(HouseId(h as u8), Plan::default());
                    s.begin_think();
                    for t in all_types(w)
                        .into_iter()
                        .filter(|t| t.kind == Kind::Building)
                    {
                        let want = three_passes(&s, w, t);
                        assert_eq!(s.place_cell(w, t), want, "house {h}, type {t:?}");
                        assert_eq!(s.place_cell(w, t), want, "kept: house {h}, type {t:?}");
                        compared += 1;
                        found += usize::from(want.is_some());
                    }
                    // The clearing buildings and the tech climb, kept and
                    // asked afresh (a script outside a think keeps nothing).
                    let plain = Script::new(HouseId(h as u8), Plan::default());
                    for t in all_types(w) {
                        let can = tech::buildable(w, HouseId(h as u8), t);
                        assert_eq!(s.buildable(w, t), can, "house {h}, {t:?}");
                        assert_eq!(s.buildable(w, t), can, "kept: house {h}, {t:?}");
                        for depth in 0..=CLIMB_DEPTH {
                            let want = plain.unblocker(w, t, depth);
                            assert_eq!(s.unblocker(w, t, depth), want, "house {h}, {t:?}, {depth}");
                            unblocked += usize::from(want.is_some());
                        }
                    }
                    assert_eq!(s.climb(w), plain.climb(w), "house {h}");
                    s.end_think();
                    assert!(s.placed.0.borrow().is_none());
                    assert!(plain.placed.0.borrow().is_none());
                }
            }
        }
        assert!(
            compared > 0 && found > 0 && found < compared,
            "{found} of {compared}"
        );
        assert!(unblocked > 0);
    }

    /// The house's objects as [`Script::own`] listed them before batch 2.
    fn own_as_before(s: &Script, w: &World) -> Vec<ObjectId> {
        w.objects
            .values()
            .filter(|o| o.owner == Some(s.house) && o.on_map && o.health > 0)
            .map(|o| o.id)
            .collect()
    }

    /// [`Script::seen_enemies`] as before batch 2: the cloak asked of every
    /// object.
    fn seen_as_before(s: &Script, w: &World) -> Vec<(ObjectId, CellXY)> {
        let me = &w.houses[usize::from(s.house.0)];
        w.objects
            .values()
            .filter(|o| o.on_map && o.health > 0)
            .filter_map(|o| {
                let owner = o.owner?;
                let h = w.houses.get(usize::from(owner.0))?;
                let enemy = owner != s.house
                    && !me.allied_with(owner)
                    && h.role == glasnost_sim::world::HouseRole::Player;
                let c = o.pos.cell();
                // The view rule of #37: any explored cell of a building, no disguise shown.
                let seen =
                    crate::in_view(w, s.house, o.id) && !crate::shown_as_disguise(w, s.house, o.id);
                (enemy && seen).then_some((o.id, c))
            })
            .collect()
    }

    /// [`Script::home`] as before batch 2, for a script with no home kept
    /// yet, with the first building before the first object (design: own
    /// AI, #12).
    fn home_as_before(s: &Script, w: &World) -> Option<CellXY> {
        let own = own_as_before(s, w);
        own.iter()
            .find(|&&id| w.obj(id).is_some_and(|o| is_yard(w, o)))
            .or_else(|| {
                own.iter()
                    .find(|&&id| w.obj(id).is_some_and(|o| o.kind == Kind::Building))
            })
            .or(own.first())
            .and_then(|&id| w.obj(id))
            .map(|o| o.pos.cell())
    }

    /// The free-gap test as before batch 2: the house's known objects
    /// listed afresh, without `but`, for each test.
    fn free_but_as_before(
        s: &Script,
        w: &World,
        t: TypeRef,
        c: CellXY,
        but: Option<ObjectId>,
        gap: i32,
    ) -> bool {
        let cells = placement::foundation_cells(w, t.index, c);
        w.objects
            .values()
            .filter(|o| {
                o.on_map
                    && Some(o.id) != but
                    && (o.owner == Some(s.house) || shroud::explored(w, s.house, o.pos.cell()))
            })
            .all(|o| {
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

    /// The fixture games the kept-list test plays: the placement maps, the
    /// islands, and the M6c, M7a and M7b duels (cloaked units, drainers,
    /// superweapons).
    fn kept_games() -> Vec<Game> {
        let mut games: Vec<Game> = placement_maps()
            .into_iter()
            .chain([("synthetic-islands", fixture::map_islands())])
            .map(|(name, map)| {
                Game::new(
                    fixture::rules(),
                    &map,
                    name,
                    fixture::catalogue(),
                    fixture::setup(3),
                )
                .expect("the skirmish loads")
            })
            .collect();
        games.push(
            Game::new(
                crate::fixture_m6c::rules(),
                &crate::fixture_m6c::map(),
                crate::fixture_m6c::MAP_NAME,
                fixture::catalogue(),
                crate::fixture_m6c::setup(2),
            )
            .expect("the M6c duel loads"),
        );
        games.push(
            Game::new(
                crate::fixture_m7a::rules(),
                &crate::fixture_m7a::map(),
                crate::fixture_m7a::MAP_NAME,
                fixture::catalogue(),
                crate::fixture_m7a::setup(17),
            )
            .expect("the M7a duel loads"),
        );
        games.push(
            Game::new(
                crate::fixture_m7b::rules(),
                &crate::fixture_m7b::map(),
                crate::fixture_m7b::MAP_NAME,
                fixture::catalogue(),
                crate::fixture_m7b::setup(11),
            )
            .expect("the M7b duel loads"),
        );
        games
    }

    /// Our tooling for speed (design: own AI, #52, #53, #57, #59 and #60):
    /// the house's objects, those of each role, the types it can build for
    /// each role, the enemies it sees, its home and the free-gap test
    /// (for every building type at every cell, with and without an object
    /// left out), kept for a think, answer as they did asked afresh, at
    /// several points of each fixture game, for every house, during a think
    /// and outside one.
    #[test]
    fn the_lists_kept_for_a_think_answer_as_asked_afresh() {
        let mut seen = 0;
        let mut free = 0;
        let mut tested = 0;
        let mut roles_found = 0;
        let mut candidates_found = 0;
        for mut g in kept_games() {
            for _ in 0..3 {
                for _ in 0..900 {
                    g.step();
                }
                let w = &g.world;
                let all: Vec<CellXY> = w.cells.grid.iter().map(|(c, _)| c).collect();
                for h in 0..w.houses.len() {
                    let house = HouseId(h as u8);
                    let s = Script::new(house, Plan::default());
                    for thinking in [true, false] {
                        if thinking {
                            s.begin_think();
                        }
                        let own = own_as_before(&s, w);
                        let enemies = seen_as_before(&s, w);
                        for _ in 0..2 {
                            assert_eq!(s.own(w), own, "house {h}");
                            assert_eq!(s.seen_enemies(w), enemies, "house {h}");
                            assert_eq!(s.home(w), home_as_before(&s, w), "house {h}");
                        }
                        seen += enemies.len();
                        let plain = Script::new(house, Plan::default());
                        let roles = [
                            Role::Yard,
                            Role::Power,
                            Role::Refinery,
                            Role::Factory(FactoryKind::Infantry),
                            Role::Factory(FactoryKind::Vehicle),
                            Role::Harvester,
                            Role::Army,
                        ];
                        for _ in 0..2 {
                            for role in roles {
                                let units = plain.of_role(w, role);
                                assert_eq!(s.of_role(w, role), units, "house {h}, {role:?}");
                                roles_found += units.len();
                                let types = plain.candidates(w, role);
                                assert_eq!(s.candidates(w, role), types, "house {h}, {role:?}");
                                candidates_found += types.len();
                            }
                        }
                        let buts = [None, own.first().copied(), own.last().copied()];
                        for t in all_types(w)
                            .into_iter()
                            .filter(|t| t.kind == Kind::Building)
                            .take(3)
                        {
                            for &c in all.iter().step_by(5) {
                                for but in buts {
                                    for gap in 0..=GAP {
                                        let want = free_but_as_before(&s, w, t, c, but, gap);
                                        assert_eq!(
                                            s.free_gap(w, t, c, but, gap),
                                            want,
                                            "house {h}, {t:?}, {c:?}, {but:?}, gap {gap}"
                                        );
                                        tested += 1;
                                        free += usize::from(want);
                                    }
                                }
                            }
                        }
                        if thinking {
                            s.end_think();
                        }
                    }
                }
            }
        }
        assert!(seen > 0, "no enemy seen");
        assert!(free > 0 && free < tested, "{free} of {tested}");
        assert!(roles_found > 0 && candidates_found > 0);
    }
}

#[cfg(test)]
mod can_attack_tests {
    use super::*;
    use glasnost_sim::testkit::{self, mid, spawn};

    const SIZE: i32 = 32;

    // spec: S-0301 claim 9 (the own AI's lasting test, our design, PLAN §4.7: the firing rules must not rule the target illegal; reasons of the moment, reloading among them, do not count), S-0304 claim 1 (a versus value of exactly zero: illegal, S-0301 claim 9 rank 9) and S-0304 claim 2 (rearming comes before illegal; without force-fire the attack cursor does not appear on such an object)
    #[test]
    fn a_reloading_unit_cannot_attack_a_target_its_weapon_may_not_hurt() {
        let mut r = testkit::combat_ruleset();
        // The gun's warhead does nothing to class light (index 3, the cart's).
        r.sim
            .warheads
            .get_mut("HE")
            .unwrap()
            .verses
            .as_mut()
            .unwrap()[3] = glasnost_rules::sim::Decimal("0".into());
        let mut w = testkit::game_with(&r, SIZE, 7);
        let tank = spawn(&mut w, Kind::Vehicle, "CRUSHER", 0, mid(SIZE, 0, 0), 0);
        let cart = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, 3, 0), 0);
        let heavy = spawn(&mut w, Kind::Vehicle, "CRUSHER", 1, mid(SIZE, 0, 3), 0);
        assert!(!can_attack(&w, tank, cart));
        assert!(can_attack(&w, tank, heavy));
        let tick = w.tick;
        w.obj_mut(tank).unwrap().combat.rearm_until = tick + 50;
        assert!(!can_attack(&w, tank, cart));
        assert!(can_attack(&w, tank, heavy));
    }
}

#[cfg(test)]
mod unseen_cell_tests {
    use super::*;
    use crate::{fixture, game::Game};
    use glasnost_sim::testkit;

    // design: own AI (#10) and S-0212 claim 4 (reach regions): the wave's search cell, once the enemy starts are cleared, is one its lead unit can go to; a cell in the middle of a lake (or on an island in it) stays unseen from the shore and was picked again on every think
    #[test]
    fn the_search_cell_is_one_the_wave_can_reach() {
        let centre = fixture::at(-fixture::START_DX, -12);
        let mut m = fixture::map();
        let cells: Vec<CellXY> = m.cells.iter().map(|(c, _)| c).collect();
        for c in cells {
            let d = (c.x - centre.x).abs().max((c.y - centre.y).abs());
            if (2..=9).contains(&d)
                && let Some(cell) = m.cells.get_mut(c)
            {
                cell.tile = fixture::WATER_TILE;
                cell.overlay = glasnost_map::NO_OVERLAY;
            }
        }
        let g = Game::new(
            fixture::rules(),
            &m,
            "synthetic-lake",
            fixture::catalogue(),
            fixture::setup(5),
        )
        .expect("the lake skirmish loads");
        let mut w = g.world.clone();
        let at = fixture::at(-fixture::START_DX + 4, 4);
        let tank = testkit::spawn(&mut w, Kind::Vehicle, "GUNR", 0, at, 0);
        let zone = w
            .rules()
            .unit(Kind::Vehicle, w.obj(tank).unwrap().type_index)
            .unwrap()
            .zone;
        let home = regions::reach_in(&w, at, zone);
        assert!(matches!(home, Reach::Region(_)));
        let s = Script::new(HouseId(0), fixture::plan());
        // The scene: the nearest unseen search cell is one the tank cannot reach.
        let nearest = w
            .cells
            .grid
            .iter()
            .map(|(c, _)| c)
            .filter(|c| c.x % SEARCH_STEP == 0 && c.y % SEARCH_STEP == 0)
            .filter(|&c| w.cells.usable(c) && !shroud::seen(&w, HouseId(0), c))
            .min_by_key(|&c| (dist2(c, centre), c.y, c.x))
            .expect("an unseen cell");
        assert_ne!(regions::reach_in(&w, nearest, zone), home, "{nearest:?}");
        let c = s.unseen_cell(&w, centre, tank).expect("a search cell");
        assert_eq!(regions::reach_in(&w, c, zone), home, "{c:?}");
    }
}

#[cfg(test)]
mod start_tests {
    use super::*;
    use crate::{fixture, game::Game};
    use glasnost_sim::testkit;

    // design: own AI (#27): the house's own start is no wave target when its base unit stands more than five cells off it (bases off, or the unit placed off its waypoint): it was the first target
    #[test]
    fn the_own_start_is_no_target_when_the_base_unit_stands_off_it() {
        let map = fixture::map();
        let g = Game::new(
            fixture::rules(),
            &map,
            "synthetic-skirmish",
            fixture::catalogue(),
            fixture::setup(5),
        )
        .expect("the skirmish loads");
        let mut w = g.world.clone();
        let starts = crate::skirmish::start_cells(&map);
        let own: Vec<ObjectId> = w
            .objects
            .values()
            .filter(|o| o.owner == Some(HouseId(0)) && o.on_map)
            .map(|o| o.id)
            .collect();
        let first = w.obj(own[0]).expect("house 0's first object").pos.cell();
        let mine = *starts
            .iter()
            .min_by_key(|&&s| dist2(s, first))
            .expect("a start");
        for id in own {
            glasnost_sim::objects::remove(&mut w, id);
        }
        testkit::spawn(
            &mut w,
            Kind::Vehicle,
            "MCV",
            0,
            CellXY::new(mine.x + 8, mine.y),
            0,
        );
        let mut s = Script::new(HouseId(0), Plan::default());
        s.set_starts(&w, &starts);
        let targets = s.enemy_starts(&w);
        assert!(!targets.contains(&mine), "{targets:?} holds {mine:?}");
        assert_eq!(targets.len(), starts.len() - 1, "{targets:?}");
    }
}

#[cfg(test)]
mod income_tests {
    use super::*;
    use crate::{fixture, game::Game};
    use glasnost_sim::testkit;

    /// The fixture skirmish's world and a cell by house 0's first object.
    fn world() -> (World, CellXY) {
        let g = Game::new(
            fixture::rules(),
            &fixture::map(),
            "synthetic-skirmish",
            fixture::catalogue(),
            fixture::setup(5),
        )
        .expect("the skirmish loads");
        let w = g.world.clone();
        let home = w
            .objects
            .values()
            .find(|o| o.owner == Some(HouseId(0)) && o.on_map)
            .map(|o| o.pos.cell())
            .expect("house 0 has an object");
        (w, CellXY::new(home.x + 4, home.y + 4))
    }

    // design: own AI (#50): an item started and done between two thinks was paid for (S-0404 claim 4), so its price is no lost income
    #[test]
    fn an_item_made_between_two_thinks_is_no_lost_income() {
        let (mut w, at) = world();
        let house = HouseId(0);
        let mut s = Script::new(house, Plan::default());
        let c0 = w.houses[0].credits;
        s.watch.credits = c0;
        s.watch_income(&w);
        assert_eq!(s.watch.earned, 0);
        // Before the next think: a load pays 300 and a tank is bought and
        // made for 700, never seen on its line.
        let tank = testkit::spawn(&mut w, Kind::Vehicle, "GUNR", 0, at, 0);
        w.obj_mut(tank).unwrap().econ.start_price = Some(700);
        w.houses[0].credits = c0 + 300 - 700;
        w.tick += THINK_PERIOD;
        s.watch_income(&w);
        assert_eq!(s.watch.earned, 300);
    }

    // design: own AI (#50): the refund of a building the house sold (S-0408 claim 1) is no income
    #[test]
    fn a_sale_refund_is_no_income() {
        let (mut w, at) = world();
        let house = HouseId(0);
        let b = testkit::spawn(&mut w, Kind::Building, "PROC", 0, at, 0);
        w.obj_mut(b).unwrap().econ.sale = Some(sell::Sale::default());
        let mut s = Script::new(house, Plan::default());
        let c0 = w.houses[0].credits;
        s.watch.credits = c0;
        s.watch_income(&w);
        let refund = sell::refund(&w, house, tech::type_of(w.obj(b).unwrap()));
        assert!(refund > 0);
        glasnost_sim::objects::remove(&mut w, b);
        w.houses[0].credits = c0 + refund;
        w.tick += THINK_PERIOD;
        s.watch_income(&w);
        assert_eq!(s.watch.earned, 0);
    }
}

#[cfg(test)]
mod unblocker_tests {
    use super::*;
    use crate::{fixture, game::Game};

    /// Gives infantry type `t` the prerequisite list `prereq`.
    fn needing(w: &mut World, t: u32, prereq: Vec<tech::Prereq>) {
        let mut r = (*w.rules_arc()).clone();
        r.econ.tech.infantry[t as usize].prereq = prereq;
        w.rules = Some(std::sync::Arc::new(r));
    }

    // design: own AI (#49): a type with a blocker that no building clears (an upgrade or an unknown name, S-0407 claim 8 (d)) is never unblocked, so no building is climbed for it
    #[test]
    fn a_type_with_a_blocker_nothing_clears_has_no_unblocker() {
        let mut g = Game::new(
            fixture::rules(),
            &fixture::map(),
            "synthetic-skirmish",
            fixture::catalogue(),
            fixture::setup(5),
        )
        .expect("the skirmish loads");
        for _ in 0..900 {
            g.step();
        }
        let mut w = g.world.clone();
        let house = HouseId(0);
        let s = Script::new(house, Plan::default());
        // The scene: a building the house lacks and can build and place now.
        let b = (0..w.rules().buildings.len() as u32)
            .map(|index| TypeRef {
                kind: Kind::Building,
                index,
            })
            .find(|&b| {
                tech::present_count(&w, house, b) == 0 && s.buildable(&w, b) && s.placeable(&w, b)
            })
            .expect("a building the house can add");
        let t = (0..w.rules().infantry.len() as u32)
            .find(|&index| {
                let t = TypeRef {
                    kind: Kind::Infantry,
                    index,
                };
                tech::unmet_prerequisites(&w, house, t).is_some()
            })
            .expect("an infantry type no other condition refuses");
        let infantry = TypeRef {
            kind: Kind::Infantry,
            index: t,
        };
        needing(&mut w, t, vec![tech::Prereq::Building(b.index)]);
        let s = Script::new(house, Plan::default());
        assert_eq!(s.unblocker(&w, infantry, CLIMB_DEPTH), Some(b));
        needing(
            &mut w,
            t,
            vec![tech::Prereq::Building(b.index), tech::Prereq::Unmet],
        );
        let s = Script::new(house, Plan::default());
        assert_eq!(s.unblocker(&w, infantry, CLIMB_DEPTH), None);
    }
}
