//! The special abilities of the own AI (M6c; PLAN §4.7): garrisoning
//! buildings (S-0508), engineers capturing buildings (S-0509, tech buildings
//! first: S-0515, `tech_buildings`) and spies infiltrating them (S-0510).
//!
//! Like everything the brain does, it uses an ability the way a person in
//! its seat could, with no limit of attention or speed: it clicks (through
//! `targeting::click`, so the cursor rules of S-0214 decide what the click
//! means) on objects of cells its house has explored, and issues the
//! resulting orders. It reads no hidden state and draws no random value.
//! A person also builds the units it needs: when the house has none and a
//! target is in sight, it orders one from its sidebar.
//!
//! The counters in [`Uses`] are the orders it gave for each ability, kept
//! with the brain (saved and hashed with it) and printed by the headless
//! runner.

use glasnost_map::CellXY;
use glasnost_sim::combat::effects::InfiltrationRow;
use glasnost_sim::econ::{TypeRef, production, tech};
use glasnost_sim::orders::Order;
use glasnost_sim::probe::{Phase, Probe};
use glasnost_sim::rules::Kind;
use glasnost_sim::structures::{engineer, garrison, spy};
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use serde::{Deserialize, Serialize};

use crate::memory::Memory;
use crate::scripted::{self, Script, all_types, dist2};
use crate::tech_buildings;

/// How far, in cells, from the house's home a garrison building is looked
/// for.
const GARRISON_REACH: i64 = 24; // nocite: tuning of our own opponent

/// The most occupants the AI keeps in garrisons at once: the rest of its
/// infantry fights.
const GARRISON_LIMIT: i32 = 6; // nocite: tuning of our own opponent

/// The credits a house keeps in hand, as a multiple of a unit's price, before
/// it orders an engineer or a spy.
const SPECIAL_RESERVE: i32 = 2; // nocite: tuning of our own opponent

/// The most engineers a house keeps for the tech buildings that pay in its
/// sight ([`Special::engineers`]).
const TECH_ENGINEERS: usize = 2; // nocite: tuning of our own opponent

/// How far, in cells, from an engineer bound for a tech building its
/// escort is looked for ([`Special::escort`]).
const ESCORT_REACH: i64 = 15; // nocite: tuning of our own opponent

/// How many orders the AI gave to use each special ability.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Uses {
    /// Infantry sent to garrison a building (S-0508).
    pub garrisons: u32,
    /// Engineers sent to capture or repair a building (S-0509).
    pub engineers: u32,
    /// Spies sent to infiltrate a building (S-0510).
    pub spies: u32,
    /// Engineers sent to take over a tech building held by a passive house
    /// (S-0515; a subset of `engineers`).
    pub tech: u32,
    /// Jumpjet units sent against an enemy a walking unit cannot reach
    /// (S-0502).
    pub jumps: u32,
    /// Deploy orders given to deployable units (S-0517): in-place vehicles
    /// deployed and packed up, deploy-fire vehicles and firing infantry
    /// deployed.
    pub deploys: u32,
    /// Units sent out of a radiation field (S-0525).
    pub radiation: u32,
    /// Erasing units sent against an enemy (S-0514).
    pub erasures: u32,
    /// Prism buildings ordered (S-0518).
    pub prisms: u32,
    /// Chargers sent at an own overpowerable building (S-0518).
    pub charges: u32,
    /// Overpowerable buildings (other than the prism type) ordered for the
    /// house's chargers (S-0518).
    pub overpowerables: u32,
    /// Bomb planters sent at an enemy (S-0519).
    pub bombs: u32,
    /// Disarmers sent at an own bombed object (S-0519).
    pub disarms: u32,
    /// Suicide units sent at an enemy (S-0519).
    pub suicides: u32,
    /// Infantry with explosive charges sent at an enemy building (S-0519).
    pub sabotages: u32,
    /// Air-strike designators sent against an enemy building (S-0519).
    pub strikes: u32,
    /// Units sent to run over a crate (S-0520).
    pub crates: u32,
    /// Units with a berserk weapon sent at an enemy group (S-0610).
    pub berserks: u32,
    /// Psychic sensors ordered (S-0610 claim 9).
    pub sensors: u32,
    /// Psychic reveals fired (S-0610 claims 13 to 15).
    pub reveals: u32,
    /// Mind controllers sent to take over an enemy (S-0601).
    #[serde(default)]
    pub takeovers: u32,
    /// Gattling defence buildings ordered (S-0609).
    #[serde(default)]
    pub gattling_defences: u32,
    /// Attack orders not given to a gattling unit firing at its target,
    /// which keeps its spin (S-0609).
    #[serde(default)]
    pub spins_kept: u32,
    /// Infantry sent into an own absorbing building (S-0605).
    #[serde(default)]
    pub absorbs: u32,
    /// Units sent into an own recycling building (S-0603).
    #[serde(default)]
    pub grinds: u32,
    /// Tank bunkers ordered (S-0606, `bunker`).
    #[serde(default)]
    pub bunker_builds: u32,
    /// Vehicles sent into an own tank bunker (S-0606).
    #[serde(default)]
    pub bunker_entries: u32,
    /// Deploy clicks given to bunkered vehicles to let them out (S-0606
    /// claim 12).
    #[serde(default)]
    pub bunker_exits: u32,
    /// Magnetic lifters ordered (S-0607, `lifter`).
    #[serde(default)]
    pub lifter_builds: u32,
    /// Lifters sent at an enemy vehicle (S-0607, `lifter`).
    #[serde(default)]
    pub lift_orders: u32,
    /// Move orders given to a holding lifter over a killing cell, which
    /// drop the held vehicle there (S-0607 claims 12 and 17).
    #[serde(default)]
    pub lift_drops: u32,
    /// Drainers ordered (S-0608, `drain`).
    #[serde(default)]
    pub drainer_builds: u32,
    /// Drainers sent at an enemy building to drain it (S-0608, `drain`).
    #[serde(default)]
    pub drain_orders: u32,
    /// Genetic mutator launches ordered (S-0611, `mutator`).
    #[serde(default)]
    pub mutator_launches: u32,
    /// Psychic dominator launches ordered (S-0612, `dominator`).
    #[serde(default)]
    pub dominator_launches: u32,
    /// Hold orders given to production lines while the house pays for a
    /// harvester or refinery it lacks (S-0404 claim 5, `Brain::economy_first`).
    #[serde(default)]
    pub economy_holds: u32,
    /// Build orders that resumed a held line (S-0404 claims 3 and 5).
    #[serde(default)]
    pub resumes: u32,
    /// Infantry sent into an empty gunner vehicle to give it a healing
    /// weapon (S-0308 claims 5 and 6, `gunner`).
    #[serde(default)]
    pub gunner_loads: u32,
    /// Building orders turned into a power plant order because the
    /// building's drain would have left the house short of power
    /// (`power::plant_first`).
    #[serde(default)]
    pub plants_first: u32,
    /// Army units sent at an enemy draining one of the house's power
    /// plants (S-0608, S-0405 claim 5) or attacking one of its harvesters,
    /// slave miners or slaves (`Brain::defend`).
    #[serde(default)]
    pub economy_defence: u32,
    /// Iron curtains fired on one of the house's buildings under attack
    /// rather than on a group of vehicles (`Brain::curtain_cell`).
    #[serde(default)]
    pub curtained_buildings: u32,
    /// Hold orders given to army units while the house saves for a
    /// superweapon building (S-0404 claim 5, `Brain::economy_first`).
    #[serde(default)]
    pub superweapon_holds: u32,
    /// Construction-yard vehicles built to expand the base toward a far
    /// resource field (S-0415, S-0406 claim 4, `expand`).
    #[serde(default)]
    pub expansion_builds: u32,
    /// Such vehicles sent to the field to deploy there (`expand`).
    #[serde(default)]
    pub expansion_moves: u32,
    /// Buildings ordered to clear the yard vehicle's blocker on the tech
    /// tree for an expansion (S-0407 claim 8, `expand`).
    #[serde(default)]
    pub expansion_unblocks: u32,
    /// Such vehicles cancelled on the line because the house could not pay
    /// for them (S-0404 claim 6, `Brain::drop_expansion`).
    #[serde(default)]
    pub expansion_cancels: u32,
    /// Units or defences built because nothing of the house could hit a
    /// seen airborne drainer (S-0306 claim 23, `antiair`).
    #[serde(default)]
    pub anti_air_builds: u32,
    /// Engineers and expansion yard vehicles cancelled on their line while
    /// a superweapon building waits for money and the house is not safe
    /// (S-0404 claim 6, `Brain::economy_first`, M8 step B7).
    #[serde(default)]
    pub non_fighter_cancels: u32,
    /// Army units sent along with an engineer bound for a tech building
    /// (`Special::escort`, M8 step A4 round 8).
    #[serde(default)]
    pub escorts: u32,
}

/// What an order the brain gives is for: the counter of [`Uses`] it adds
/// one to ([`Special::offer_use`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Use {
    Garrison,
    Engineer,
    Spy,
    Tech,
    Jump,
    Deploy,
    Radiation,
    Erasure,
    Prism,
    Charge,
    Overpowerable,
    Bomb,
    Disarm,
    Suicide,
    Sabotage,
    Strike,
    Crate,
    Berserk,
    Sensor,
    Reveal,
    Takeover,
    GattlingDefence,
    Absorb,
    Grind,
    BunkerBuild,
    BunkerEntry,
    BunkerExit,
    LifterBuild,
    LiftOrder,
    LiftDrop,
    DrainerBuild,
    DrainOrder,
    MutatorLaunch,
    DominatorLaunch,
    EconomyHold,
    Resume,
    GunnerLoad,
    EconomyDefence,
    CurtainedBuilding,
    SuperweaponHold,
    ExpansionBuild,
    ExpansionMove,
    ExpansionUnblock,
    ExpansionCancel,
    AntiAirBuild,
    NonFighterCancel,
    Escort,
}

impl Uses {
    /// Counts one order given for `u`.
    fn add(&mut self, u: Use) {
        let c = match u {
            Use::Garrison => &mut self.garrisons,
            Use::Engineer => &mut self.engineers,
            Use::Spy => &mut self.spies,
            Use::Tech => &mut self.tech,
            Use::Jump => &mut self.jumps,
            Use::Deploy => &mut self.deploys,
            Use::Radiation => &mut self.radiation,
            Use::Erasure => &mut self.erasures,
            Use::Prism => &mut self.prisms,
            Use::Charge => &mut self.charges,
            Use::Overpowerable => &mut self.overpowerables,
            Use::Bomb => &mut self.bombs,
            Use::Disarm => &mut self.disarms,
            Use::Suicide => &mut self.suicides,
            Use::Sabotage => &mut self.sabotages,
            Use::Strike => &mut self.strikes,
            Use::Crate => &mut self.crates,
            Use::Berserk => &mut self.berserks,
            Use::Sensor => &mut self.sensors,
            Use::Reveal => &mut self.reveals,
            Use::Takeover => &mut self.takeovers,
            Use::GattlingDefence => &mut self.gattling_defences,
            Use::Absorb => &mut self.absorbs,
            Use::Grind => &mut self.grinds,
            Use::BunkerBuild => &mut self.bunker_builds,
            Use::BunkerEntry => &mut self.bunker_entries,
            Use::BunkerExit => &mut self.bunker_exits,
            Use::LifterBuild => &mut self.lifter_builds,
            Use::LiftOrder => &mut self.lift_orders,
            Use::LiftDrop => &mut self.lift_drops,
            Use::DrainerBuild => &mut self.drainer_builds,
            Use::DrainOrder => &mut self.drain_orders,
            Use::MutatorLaunch => &mut self.mutator_launches,
            Use::DominatorLaunch => &mut self.dominator_launches,
            Use::EconomyHold => &mut self.economy_holds,
            Use::Resume => &mut self.resumes,
            Use::GunnerLoad => &mut self.gunner_loads,
            Use::EconomyDefence => &mut self.economy_defence,
            Use::CurtainedBuilding => &mut self.curtained_buildings,
            Use::SuperweaponHold => &mut self.superweapon_holds,
            Use::ExpansionBuild => &mut self.expansion_builds,
            Use::ExpansionMove => &mut self.expansion_moves,
            Use::ExpansionUnblock => &mut self.expansion_unblocks,
            Use::ExpansionCancel => &mut self.expansion_cancels,
            Use::AntiAirBuild => &mut self.anti_air_builds,
            Use::NonFighterCancel => &mut self.non_fighter_cancels,
            Use::Escort => &mut self.escorts,
        };
        *c = c.saturating_add(1);
    }
}

/// The special-ability part of one AI house's brain.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Special {
    uses: Uses,
    /// The orders given and the ones that did not take (`memory`).
    pub(crate) memory: Memory,
    /// What the tank bunkers remember between thinks (`bunker`).
    #[serde(default)]
    pub(crate) bunker: crate::bunker::Watch,
    /// The crate orders given lately (`crates`).
    #[serde(default)]
    pub(crate) crate_orders: crate::crates::Sent,
}

/// The click a person would give `unit` on `target`, as orders, when the
/// cursor over it shows one of `want`.
pub(crate) fn click_if(
    w: &World,
    house: HouseId,
    unit: ObjectId,
    target: ObjectId,
    want: &[Action],
) -> Option<Vec<Order>> {
    let mods = Modifiers::default();
    let on = ClickOn::Object(target);
    want.contains(&targeting::cursor(w, house, &[unit], on, mods))
        .then(|| targeting::click(w, house, &[unit], on, mods).orders)
        .filter(|o| !o.is_empty())
}

/// The house's infantry en route to enter each building, by building
/// ([`Special::en_route`]).
#[derive(Debug, PartialEq, Eq)]
struct EnRoute(std::collections::BTreeMap<ObjectId, i32>);

impl EnRoute {
    /// The infantry en route to enter building `b`.
    fn get(&self, b: ObjectId) -> i32 {
        self.0.get(&b).copied().unwrap_or(0)
    }
}

impl Special {
    /// The orders given so far.
    pub fn uses(&self) -> Uses {
        self.uses
    }

    /// Thinks once per think period: every ability in turn. Alone (not in
    /// a brain), every order it gives is noted as given.
    pub fn think(&mut self, script: &Script, w: &World, out: &mut Vec<Order>) {
        self.think_with(script, w, out, &mut ());
        self.memory.note_given(w, out, usize::MAX);
    }

    /// [`Special::think`], telling `p` which ability runs (the tools'
    /// benchmark, `glasnost_sim::probe`); `p` reads nothing and changes
    /// nothing. Its orders are offered to the memory, and noted when the
    /// caller tells it which it gave ([`Memory::note_given`]).
    pub fn think_with<P: Probe>(
        &mut self,
        script: &Script,
        w: &World,
        out: &mut Vec<Order>,
        p: &mut P,
    ) {
        let on = p.on();
        let mut mark = |part: &'static str| {
            if on {
                p.mark(Phase::Part(part));
            }
        };
        let house = script.house;
        let mut own = script.own(w);
        let home = script.home(w);
        self.memory.review(w);
        let mut start = out.len();
        // design: own AI (#47): each module sees only the house's units no earlier module of this think gave an order, so a unit gets one job (the later order would win, and the earlier module's counters and bookkeeping would be wrong)
        let free = |own: &mut Vec<ObjectId>, out: &[Order]| {
            let given: std::collections::BTreeSet<ObjectId> = out
                .get(start..)
                .unwrap_or_default()
                .iter()
                .filter_map(Order::actor)
                .collect();
            own.retain(|id| !given.contains(id));
        };
        mark("ai:special:garrison");
        self.garrison(house, &own, home, w, out);
        free(&mut own, out);
        mark("ai:special:engineers");
        self.engineers(script, &own, w, out);
        free(&mut own, out);
        mark("ai:special:spies");
        self.spies(script, &own, w, out);
        free(&mut own, out);
        mark("ai:special:jumpjets");
        self.jumpjets(script, &own, w, out);
        free(&mut own, out);
        mark("ai:special:deployers");
        self.deployers(script, &own, w, out);
        free(&mut own, out);
        mark("ai:special:radiation");
        self.radiation(script, &own, w, out);
        free(&mut own, out);
        mark("ai:special:chrono");
        self.chrono(script, &own, w, out);
        free(&mut own, out);
        mark("ai:special:linked");
        self.linked(script, &own, w, out);
        free(&mut own, out);
        mark("ai:special:terror");
        self.terror(script, &own, w, out);
        free(&mut own, out);
        mark("ai:special:crates");
        self.crates(script, &own, w, out);
        free(&mut own, out);
        mark("ai:special:berserk");
        self.berserk(script, &own, w, out);
        free(&mut own, out);
        mark("ai:special:mind_control");
        self.mind_control(script, &own, w, out);
        free(&mut own, out);
        mark("ai:special:absorbers");
        self.absorbers(script, &own, w, out);
        free(&mut own, out);
        mark("ai:special:grinders");
        self.grinders(script, &own, w, out);
        free(&mut own, out);
        mark("ai:special:bunkers");
        self.bunkers(script, &own, w, out);
        free(&mut own, out);
        mark("ai:special:lifters");
        self.lifters(script, &own, w, out);
        free(&mut own, out);
        mark("ai:special:drainers");
        self.drainers(script, &own, w, out);
        free(&mut own, out);
        mark("ai:special:gunners");
        self.gunners(script, &own, w, out, &mut start);
        self.memory.offer(out.get(start..).unwrap_or_default());
    }

    /// How many (unit, goal) pairs the brain has given up for now (an order
    /// the unit did not take, see `memory`).
    pub fn given_up(&self) -> usize {
        self.memory.blocked()
    }

    /// Offers the order just given, the last of `out`, as a use of `u`
    /// ([`Uses`]): one order for each ability use, the click's last order
    /// when a click gives several.
    pub(crate) fn offer_use(&mut self, u: Use, out: &[Order]) {
        debug_assert!(!out.is_empty(), "an order is given before it is offered");
        self.uses.add(u);
    }

    /// Counts building orders turned into power plant orders.
    pub(crate) fn count_plants_first(&mut self, n: usize) {
        self.uses.plants_first += n as u32;
    }

    /// Counts attack orders dropped for gattling units that keep firing
    /// at their targets (S-0609, `gattling`).
    pub(crate) fn count_spins_kept(&mut self, n: usize) {
        self.uses.spins_kept = self.uses.spins_kept.saturating_add(n as u32);
    }

    /// Whether the house sees object `b` ([`crate::in_view`]: an explored
    /// cell and no cloak hiding it; what it can see, so what it may click).
    fn seen(w: &World, house: HouseId, b: ObjectId) -> bool {
        crate::in_view(w, house, b)
    }

    /// Infantry en route to enter each building: own units in *capture*
    /// with the building as destination, counted by building from one pass
    /// over the objects (design: own AI, #62: one pass for every building
    /// asked, in garrison once for every own object, was quadratic). The
    /// world does not change during a think, so the counts hold for it.
    fn en_route(w: &World, house: HouseId) -> EnRoute {
        let mut m = std::collections::BTreeMap::new();
        for o in w.objects.values().filter(|o| {
            o.owner == Some(house)
                && o.on_map
                && o.beh.current == Some(glasnost_sim::rules::Behaviour::Capture)
        }) {
            if let Some(Target::Object(b)) = o.dest {
                let n: &mut i32 = m.entry(b).or_insert(0);
                *n = n.saturating_add(1);
            }
        }
        EnRoute(m)
    }

    /// Garrisoning (S-0508): every idle infantry unit of an occupier type
    /// is sent to the nearest building it may garrison (S-0508 claim 1, the
    /// cursor of S-0214 row I8) on an explored cell near home and with a
    /// free place, civilian garrison buildings before the house's own, up to
    /// [`GARRISON_LIMIT`] occupants in all.
    fn garrison(
        &mut self,
        house: HouseId,
        own: &[ObjectId],
        home: Option<CellXY>,
        w: &World,
        out: &mut Vec<Order>,
    ) {
        let Some(home) = home else { return };
        let buildings: Vec<ObjectId> = w
            .objects
            .values()
            .filter(|o| {
                o.kind == Kind::Building
                    && o.on_map
                    && o.health > 0
                    && garrison::garrisonable(w, o.id)
                    && dist2(o.pos.cell(), home) <= GARRISON_REACH * GARRISON_REACH
            })
            .map(|o| o.id)
            .filter(|&b| Self::seen(w, house, b))
            .collect();
        if buildings.is_empty() {
            return;
        }
        // The house's occupants so far, and the places the orders of this think already filled.
        let inside: i32 = w
            .objects
            .values()
            .filter(|o| o.owner == Some(house) && o.structure.inside.is_some())
            .count()
            .try_into()
            .unwrap_or(i32::MAX);
        let en_route = Self::en_route(w, house);
        // The occupants bound for the buildings do not change during the think.
        let bound: i32 = buildings.iter().map(|&b| en_route.get(b)).sum();
        let mut sent: Vec<(ObjectId, i32)> = Vec::new();
        for &id in own {
            let Some(o) = w.obj(id) else { continue };
            let total = inside + i32::try_from(sent.len()).unwrap_or(i32::MAX) + bound;
            if total >= GARRISON_LIMIT {
                break;
            }
            if o.kind != Kind::Infantry
                || !garrison::occupier(w, id)
                || !scripted::idle(w, id)
                || !targeting::selectable(w, id)
            {
                continue;
            }
            let at: CellXY = o.pos.cell();
            let free = |b: ObjectId, sent: &[(ObjectId, i32)]| {
                let cap = garrison::capacity(w, b);
                let placed = garrison::count(w, b)
                    + en_route.get(b)
                    + sent
                        .iter()
                        .filter(|&&(x, _)| x == b)
                        .map(|&(_, n)| n)
                        .sum::<i32>();
                cap < 0 || placed < cap
            };
            let pick = buildings
                .iter()
                .copied()
                .filter(|&b| {
                    garrison::may_enter(w, id, b)
                        && free(b, &sent)
                        && self.memory.ok(w, id, Target::Object(b))
                })
                .min_by_key(|&b| {
                    (
                        // A civilian garrison building first, then the nearest.
                        !garrison::civilian_building(w, b),
                        w.obj(b).map_or(i64::MAX, |bo| dist2(bo.pos.cell(), at)),
                        b,
                    )
                });
            let Some(b) = pick else { continue };
            if let Some(orders) = click_if(w, house, id, b, &[Action::Capture]) {
                out.extend(orders);
                sent.push((b, 1));
                self.offer_use(Use::Garrison, out);
            }
        }
    }

    /// The cheapest type of infantry the house can build now that
    /// satisfies `f` (the script's kept buildability; design: own AI, #60).
    pub(crate) fn buildable_infantry(
        script: &Script,
        w: &World,
        f: impl Fn(&glasnost_sim::rules::UnitRules) -> bool,
    ) -> Option<TypeRef> {
        all_types(w)
            .into_iter()
            .filter(|t| t.kind == Kind::Infantry)
            .filter(|&t| w.rules().unit(t.kind, t.index).is_some_and(&f))
            .filter(|&t| script.buildable(w, t))
            .min_by_key(|&t| (production::cost_field(w, t), t.index))
    }

    /// Whether the house has, or is already making, an object that
    /// satisfies `f`: on the map, off it, in a line.
    fn has_or_makes(w: &World, house: HouseId, f: impl Fn(ObjectId) -> bool) -> bool {
        w.objects
            .values()
            .any(|o| o.owner == Some(house) && o.kind == Kind::Infantry && f(o.id))
    }

    /// Orders one unit of type `t` without the credit reserve of
    /// [`Self::order_unit`]: the line pays for it as money comes in, as a
    /// person queues a unit the army is waiting for (the gunner's
    /// passenger: the M7b lane found the reserve never met, every credit
    /// spent on the army).
    pub(crate) fn queue_unit(&self, script: &Script, w: &World, t: TypeRef, out: &mut Vec<Order>) {
        if script.selling() || Self::queued(w, script.house, t) {
            return;
        }
        out.push(Order::Build { what: t });
    }

    /// Whether the line of `t` already holds one in its queue (a queued copy
    /// has no object yet).
    fn queued(w: &World, house: HouseId, t: TypeRef) -> bool {
        production::line_of(w, house, t)
            .is_some_and(|l| l.item.as_ref().is_some_and(|i| i.t == t) || l.queue.contains(&t))
    }

    /// Orders one unit of `t` from the sidebar when the house keeps enough
    /// credits in hand and the type is not already in its line.
    pub(crate) fn order_unit(&self, script: &Script, w: &World, t: TypeRef, out: &mut Vec<Order>) {
        let price = production::cost_field(w, t);
        if script.selling()
            || Self::queued(w, script.house, t)
            || glasnost_sim::seams::available_credits(w, script.house) < price * SPECIAL_RESERVE
        {
            return;
        }
        out.push(Order::Build { what: t });
    }

    /// The enemy buildings an engineer may enter, as the cursor shows: the
    /// house's seen enemy buildings of capturable types, and its own
    /// damaged ones to repair (S-0214 rows I4 to I7).
    /// `seen` is [`Special::seen_buildings`], which does not hang on the
    /// unit and is read once for every engineer of the think (design: own
    /// AI, #62: it was read again for each idle engineer).
    fn engineer_targets(
        w: &World,
        house: HouseId,
        unit: ObjectId,
        seen: &[ObjectId],
    ) -> Vec<(ObjectId, Action)> {
        let mods = Modifiers::default();
        seen.iter()
            .copied()
            .filter_map(|b| {
                let a = targeting::cursor(w, house, &[unit], ClickOn::Object(b), mods);
                (matches!(a, Action::Capture | Action::DamagingEntry)
                    || (a == Action::RepairBuilding && Self::worth_repair(w, house, b)))
                .then_some((b, a))
            })
            .collect()
    }

    /// The live buildings on the map the house sees, by id: the ones the
    /// cursor of an engineer is asked over ([`Special::engineer_targets`]).
    fn seen_buildings(w: &World, house: HouseId) -> Vec<ObjectId> {
        w.objects
            .values()
            .filter(|o| o.kind == Kind::Building && o.on_map && o.health > 0)
            .map(|o| o.id)
            .filter(|&b| Self::seen(w, house, b))
            .collect()
    }

    /// Whether an own damaged building is worth an engineer, who is spent
    /// by the repair (S-0509): one the house cannot do without (its
    /// construction yard, a factory, a refinery, or a tech building that
    /// pays, S-0515 claims 3 and 5), at half its strength or below. Our own
    /// design (M8 step A4 round 8, the lead's note 51: engineers went into
    /// any damaged building, walls and barely scratched ones too).
    fn worth_repair(w: &World, house: HouseId, b: ObjectId) -> bool {
        let Some(o) = w.obj(b) else { return false };
        // design: own AI (#22): the house's own building only; the cursor offers the repair on an ally's building too (S-0214 rows I4 to I7), and an engineer was spent on it
        if o.owner != Some(house) {
            return false;
        }
        let role = scripted::role_of(w, tech::type_of(o));
        let matters = matches!(
            role,
            Some(scripted::Role::Yard | scripted::Role::Refinery | scripted::Role::Factory(_))
        ) || glasnost_sim::structures::tech::cash_producing(w, b);
        let strength = glasnost_sim::combat::damage::strength(w, b);
        // design: half strength or below, the point at which a person spends an engineer on a building
        matters && o.health * 2 <= strength // nocite: tuning of our own opponent (half strength)
    }

    /// Engineers (S-0509): every idle engineer is sent to the nearest
    /// enemy building it may enter (capture), or, with none, to the nearest
    /// own building that is damaged (repair); the house orders an engineer
    /// when it has none and sees a building to capture.
    fn engineers(&mut self, script: &Script, own: &[ObjectId], w: &World, out: &mut Vec<Order>) {
        let house = script.house;
        let mut taken: Vec<ObjectId> = Vec::new();
        let mut any = false;
        // An engineer the gunner step loads into a gunner vehicle stays.
        let kept = self.gunner_reserved(script, own, w, out);
        let en_route = Self::en_route(w, house);
        let mut seen: Option<Vec<ObjectId>> = None;
        for &id in own {
            if !engineer::is_engineer(w, id) {
                continue;
            }
            any = true;
            if kept.contains(&id) {
                continue;
            }
            if !scripted::idle(w, id) || !targeting::selectable(w, id) {
                continue;
            }
            let at = w.obj(id).map_or(CellXY::new(0, 0), |o| o.pos.cell());
            // Read once, for the first idle engineer: a house with none skips the pass.
            let seen = seen.get_or_insert_with(|| Self::seen_buildings(w, house));
            let targets = Self::engineer_targets(w, house, id, seen);
            let rank = |&(b, a): &(ObjectId, Action)| {
                (
                    // Capturing before repairing, a tech building that pays before the rest (S-0515), then the nearest; the filter below already leaves out a building another engineer is bound for.
                    a == Action::RepairBuilding,
                    tech_buildings::rank(w, b),
                    w.obj(b).map_or(i64::MAX, |o| dist2(o.pos.cell(), at)),
                    b,
                )
            };
            let Some(&(b, a)) = targets
                .iter()
                .filter(|&&(b, _)| {
                    !taken.contains(&b)
                        && en_route.get(b) == 0
                        && self.memory.ok(w, id, Target::Object(b))
                })
                .min_by_key(|t| rank(t))
            else {
                continue;
            };
            if let Some(orders) = click_if(w, house, id, b, &[a]) {
                out.extend(orders);
                taken.push(b);
                self.offer_use(Use::Engineer, out);
                if tech_buildings::passive_tech(w, b) {
                    self.offer_use(Use::Tech, out);
                    if let Some(order) = Self::escort(script, w, id, b, out) {
                        out.push(order);
                        self.offer_use(Use::Escort, out);
                    }
                }
            }
        }
        let Some(t) = Self::buildable_infantry(script, w, |u| u.engineer) else {
            return;
        };
        // design: every tech building that pays, in sight and with no
        // engineer bound for it, wants an engineer of its own, up to
        // TECH_ENGINEERS at once, and the house queues it at once, paying as
        // money comes in (a person buys the engineer for an oil derrick
        // first thing: M8 step A4 round 8, the lead's note 48, where an
        // engineer was ordered only with twice its price in hand and only
        // when the house had none)
        let open = tech_buildings::paying_in_sight(w, house)
            .into_iter()
            .filter(|&b| en_route.get(b) == 0)
            .count();
        let have = Self::count_of(w, house, |id| engineer::is_engineer(w, id));
        if open > 0 && have < open.min(TECH_ENGINEERS) {
            self.queue_unit(script, w, t, out);
        } else if !any
            && have == 0
            && (Self::captureable_in_sight(w, house) || tech_buildings::in_sight(w, house))
        {
            self.order_unit(script, w, t, out);
        }
    }

    /// The house's infantry, on the map, off it or in a line, that
    /// satisfies `f`.
    fn count_of(w: &World, house: HouseId, f: impl Fn(ObjectId) -> bool) -> usize {
        w.objects
            .values()
            .filter(|o| o.owner == Some(house) && o.kind == Kind::Infantry && f(o.id))
            .count()
    }

    /// An escort for engineer `eng` bound for tech building `b`: the idle
    /// army unit nearest the engineer, within [`ESCORT_REACH`] cells, that
    /// walks or drives and has no order in this think yet, attack-moved to
    /// the building. Our own design (M8 step A4 round 8, the lead's note
    /// 48): a person sends a tank along with an engineer when one is near.
    fn escort(
        script: &Script,
        w: &World,
        eng: ObjectId,
        b: ObjectId,
        out: &[Order],
    ) -> Option<Order> {
        let at = w.obj(eng)?.pos.cell();
        let goal = w.obj(b)?.pos.cell();
        let ordered = |id: ObjectId| out.iter().any(|o| o.actor() == Some(id));
        let unit = script
            .of_role(w, scripted::Role::Army)
            .into_iter()
            .filter(|&id| {
                w.obj(id)
                    .is_some_and(|o| o.kind == Kind::Vehicle || o.kind == Kind::Infantry)
                    && scripted::idle(w, id)
                    && !ordered(id)
            })
            .filter_map(|id| w.obj(id).map(|o| (dist2(o.pos.cell(), at), id)))
            .filter(|&(d, _)| d <= ESCORT_REACH * ESCORT_REACH)
            .min()?
            .1;
        Some(Order::Behaviour {
            actor: unit,
            behaviour: glasnost_sim::rules::Behaviour::Move,
            attack_move: true,
            target: None,
            dest: Some(Target::Cell(goal)),
        })
    }

    /// Whether the house sees an enemy building an engineer would capture.
    fn captureable_in_sight(w: &World, house: HouseId) -> bool {
        let me = &w.houses[usize::from(house.0)];
        w.objects.values().any(|o| {
            o.kind == Kind::Building
                && o.on_map
                && o.health > 0
                && o.owner != Some(house)
                && o.owner.is_some_and(|h| {
                    !me.allied_with(h)
                        && w.houses
                            .get(usize::from(h.0))
                            .is_some_and(|x| x.role == glasnost_sim::world::HouseRole::Player)
                })
                && w.rules()
                    .buildings
                    .get(o.type_index as usize)
                    .is_some_and(|b| b.raw.capturable.unwrap_or(false))
                && Self::seen(w, house, o.id)
        })
    }
}

impl Special {
    /// How worth infiltrating a building is to `house`, lower first: the
    /// first of its effect rows (S-0510 claim 3) ranked by what a person
    /// wants: stolen technology, a factory flag it does not have yet, the
    /// victim's money, its power, a superweapon, its radar. The victim's
    /// credits are not asked: the house cannot see them (#36).
    fn infiltration_rank(w: &World, house: HouseId, b: ObjectId) -> Option<u8> {
        let me = w.houses.get(usize::from(house.0))?;
        w.obj(b)?.owner?;
        spy::rows(w, b).first().and_then(|row| match row {
            InfiltrationRow::Tech => Some(0),
            InfiltrationRow::VehicleFactory if !me.infiltrated_vehicle_factory => Some(1),
            InfiltrationRow::InfantryFactory if !me.infiltrated_infantry_factory => Some(1),
            // design: own AI (#36): the cash row whatever the victim holds, which the house does not see
            InfiltrationRow::Cash => Some(2), // nocite: our opponent's preference order
            InfiltrationRow::Power => Some(3), // nocite: our opponent's preference order
            InfiltrationRow::Superweapon => Some(4), // nocite: our opponent's preference order
            InfiltrationRow::Radar => Some(5), // nocite: our opponent's preference order
            _ => None,
        })
    }

    /// The enemy buildings a spy may enter, as the cursor shows (S-0214 row
    /// I24: capture) with the rank of what its entry would do.
    /// `ranked` is [`Special::spy_buildings`], which does not hang on the
    /// unit and is read once for every spy of the think (design: own AI,
    /// #62: it was read again for each idle spy).
    fn spy_targets(
        w: &World,
        house: HouseId,
        unit: ObjectId,
        ranked: &[(ObjectId, u8)],
    ) -> Vec<(ObjectId, u8)> {
        let mods = Modifiers::default();
        ranked
            .iter()
            .copied()
            .filter(|&(b, _)| {
                targeting::cursor(w, house, &[unit], ClickOn::Object(b), mods) == Action::Capture
            })
            .collect()
    }

    /// The live enemy player buildings on the map the house sees and would
    /// infiltrate, by id, with the rank of what an entry would do: the ones
    /// the cursor of a spy is asked over ([`Special::spy_targets`]).
    fn spy_buildings(w: &World, house: HouseId) -> Vec<(ObjectId, u8)> {
        w.objects
            .values()
            .filter(|o| o.kind == Kind::Building && o.on_map && o.health > 0)
            // design: own AI (#39): an enemy player's building, as the purchase test asks
            .filter(|o| Self::enemy_player(w, house, o.owner))
            .map(|o| o.id)
            .filter(|&b| Self::seen(w, house, b))
            .filter_map(|b| Some((b, Self::infiltration_rank(w, house, b)?)))
            .collect()
    }

    /// Spies (S-0510): every idle spy is sent to the best seen enemy
    /// building it may infiltrate, never to one another spy of the house is
    /// bound for; a house with no spy (on the map, off it or in its line)
    /// that sees such a building orders one.
    fn spies(&mut self, script: &Script, own: &[ObjectId], w: &World, out: &mut Vec<Order>) {
        let house = script.house;
        let mut taken: Vec<ObjectId> = Vec::new();
        let mut any = false;
        let mut en_route: Option<EnRoute> = None;
        let mut ranked: Option<Vec<(ObjectId, u8)>> = None;
        for &id in own {
            if !spy::is_spy(w, id) || engineer::is_engineer(w, id) {
                continue;
            }
            any = true;
            if !scripted::idle(w, id) || !targeting::selectable(w, id) {
                continue;
            }
            let at = w.obj(id).map_or(CellXY::new(0, 0), |o| o.pos.cell());
            // Counted and read once, for the first idle spy: a house with none skips the passes.
            let en_route = en_route.get_or_insert_with(|| Self::en_route(w, house));
            let ranked = ranked.get_or_insert_with(|| Self::spy_buildings(w, house));
            let Some((b, _)) = Self::spy_targets(w, house, id, ranked)
                .into_iter()
                .filter(|&(b, _)| {
                    !taken.contains(&b)
                        && en_route.get(b) == 0
                        && self.memory.ok(w, id, Target::Object(b))
                })
                .min_by_key(|&(b, rank)| {
                    (
                        rank,
                        w.obj(b).map_or(i64::MAX, |o| dist2(o.pos.cell(), at)),
                        b,
                    )
                })
            else {
                continue;
            };
            if let Some(orders) = click_if(w, house, id, b, &[Action::Capture]) {
                out.extend(orders);
                taken.push(b);
                self.offer_use(Use::Spy, out);
            }
        }
        if !any
            && let Some(t) =
                Self::buildable_infantry(script, w, |u| u.raw.agent.unwrap_or(false) && !u.engineer)
            && !Self::has_or_makes(w, house, |id| spy::is_spy(w, id))
            && Self::infiltratable_in_sight(w, house)
        {
            self.order_unit(script, w, t, out);
        }
    }

    /// Whether the house sees an enemy building with a worthwhile
    /// infiltration effect.
    fn infiltratable_in_sight(w: &World, house: HouseId) -> bool {
        w.objects.values().any(|o| {
            o.kind == Kind::Building
                && o.on_map
                && o.health > 0
                && Self::enemy_player(w, house, o.owner)
                && Self::seen(w, house, o.id)
                && Self::infiltration_rank(w, house, o.id).is_some()
        })
    }

    /// Whether `owner` is a player house that is neither `house` nor one
    /// of its allies: the only owner whose building a spy enters (#39).
    fn enemy_player(w: &World, house: HouseId, owner: Option<HouseId>) -> bool {
        let me = &w.houses[usize::from(house.0)];
        owner.is_some_and(|h| {
            h != house
                && !me.allied_with(h)
                && w.houses
                    .get(usize::from(h.0))
                    .is_some_and(|x| x.role == glasnost_sim::world::HouseRole::Player)
        })
    }
}

#[cfg(test)]
mod en_route_tests {
    use super::*;
    use crate::{fixture, game::Game};

    /// The infantry en route to enter building `b`, as before #62: one pass
    /// over the objects for each building asked.
    fn en_route_as_before(w: &World, house: HouseId, b: ObjectId) -> i32 {
        let n = w
            .objects
            .values()
            .filter(|o| {
                o.owner == Some(house)
                    && o.on_map
                    && o.beh.current == Some(glasnost_sim::rules::Behaviour::Capture)
                    && o.dest == Some(Target::Object(b))
            })
            .count();
        i32::try_from(n).unwrap_or(i32::MAX)
    }

    /// The buildings an engineer may enter, as before #62: the seen
    /// buildings read again for each engineer asked.
    fn engineer_targets_as_before(
        w: &World,
        house: HouseId,
        unit: ObjectId,
    ) -> Vec<(ObjectId, Action)> {
        let mods = Modifiers::default();
        w.objects
            .values()
            .filter(|o| o.kind == Kind::Building && o.on_map && o.health > 0)
            .map(|o| o.id)
            .filter(|&b| Special::seen(w, house, b))
            .filter_map(|b| {
                let a = targeting::cursor(w, house, &[unit], ClickOn::Object(b), mods);
                (matches!(a, Action::Capture | Action::DamagingEntry)
                    || (a == Action::RepairBuilding && Special::worth_repair(w, house, b)))
                .then_some((b, a))
            })
            .collect()
    }

    /// The buildings a spy may enter, as before #62: the seen enemy
    /// buildings and their ranks read again for each spy asked.
    fn spy_targets_as_before(w: &World, house: HouseId, unit: ObjectId) -> Vec<(ObjectId, u8)> {
        let mods = Modifiers::default();
        w.objects
            .values()
            .filter(|o| o.kind == Kind::Building && o.on_map && o.health > 0)
            .filter(|o| Special::enemy_player(w, house, o.owner))
            .map(|o| o.id)
            .filter(|&b| Special::seen(w, house, b))
            .filter_map(|b| {
                let rank = Special::infiltration_rank(w, house, b)?;
                (targeting::cursor(w, house, &[unit], ClickOn::Object(b), mods) == Action::Capture)
                    .then_some((b, rank))
            })
            .collect()
    }

    /// [`Special::engineers`] as before #62: the infantry en route counted
    /// for each building asked, and the rank still holding the component
    /// the filter makes always false.
    fn engineers_as_before(
        s: &mut Special,
        script: &Script,
        own: &[ObjectId],
        w: &World,
        out: &mut Vec<Order>,
    ) {
        let house = script.house;
        let mut taken: Vec<ObjectId> = Vec::new();
        let mut any = false;
        let kept = s.gunner_reserved(script, own, w, out);
        for &id in own {
            if !engineer::is_engineer(w, id) {
                continue;
            }
            any = true;
            if kept.contains(&id) {
                continue;
            }
            if !scripted::idle(w, id) || !targeting::selectable(w, id) {
                continue;
            }
            let at = w.obj(id).map_or(CellXY::new(0, 0), |o| o.pos.cell());
            let targets = engineer_targets_as_before(w, house, id);
            let rank = |&(b, a): &(ObjectId, Action)| {
                (
                    a == Action::RepairBuilding,
                    tech_buildings::rank(w, b),
                    taken.contains(&b) || en_route_as_before(w, house, b) > 0,
                    w.obj(b).map_or(i64::MAX, |o| dist2(o.pos.cell(), at)),
                    b,
                )
            };
            let Some(&(b, a)) = targets
                .iter()
                .filter(|&&(b, _)| {
                    !taken.contains(&b)
                        && en_route_as_before(w, house, b) == 0
                        && s.memory.ok(w, id, Target::Object(b))
                })
                .min_by_key(|t| rank(t))
            else {
                continue;
            };
            if let Some(orders) = click_if(w, house, id, b, &[a]) {
                out.extend(orders);
                taken.push(b);
                s.offer_use(Use::Engineer, out);
                if tech_buildings::passive_tech(w, b) {
                    s.offer_use(Use::Tech, out);
                    if let Some(order) = Special::escort(script, w, id, b, out) {
                        out.push(order);
                        s.offer_use(Use::Escort, out);
                    }
                }
            }
        }
        let Some(t) = Special::buildable_infantry(script, w, |u| u.engineer) else {
            return;
        };
        let open = tech_buildings::paying_in_sight(w, house)
            .into_iter()
            .filter(|&b| en_route_as_before(w, house, b) == 0)
            .count();
        let have = Special::count_of(w, house, |id| engineer::is_engineer(w, id));
        if open > 0 && have < open.min(TECH_ENGINEERS) {
            s.queue_unit(script, w, t, out);
        } else if !any
            && have == 0
            && (Special::captureable_in_sight(w, house) || tech_buildings::in_sight(w, house))
        {
            s.order_unit(script, w, t, out);
        }
    }

    /// Our tooling for speed (design: own AI, #62): the counts from one
    /// pass are the counts of a pass for each building, for every object
    /// and every house; the engineer and spy targets from the buildings
    /// read once are the targets read for each unit, for every own
    /// infantry unit; and the engineers step with them and without the
    /// always-false rank component gives the orders and counts as before,
    /// at many points of the M6c, M7a and M7b duels.
    #[test]
    fn the_counts_of_one_pass_are_the_counts_of_a_pass_for_each_building() {
        let games = [
            (
                crate::fixture_m6c::rules(),
                crate::fixture_m6c::map(),
                crate::fixture_m6c::MAP_NAME,
                crate::fixture_m6c::setup(2),
            ),
            (
                crate::fixture_m7a::rules(),
                crate::fixture_m7a::map(),
                crate::fixture_m7a::MAP_NAME,
                crate::fixture_m7a::setup(17),
            ),
            (
                crate::fixture_m7b::rules(),
                crate::fixture_m7b::map(),
                crate::fixture_m7b::MAP_NAME,
                crate::fixture_m7b::setup(11),
            ),
        ];
        let (mut bound, mut targets, mut ordered) = (0, 0, 0);
        for (rules, map, map_name, setup) in games {
            let mut g = Game::new(rules, &map, map_name, fixture::catalogue(), setup)
                .expect("the duel loads");
            // The engineers step at every tick (an engineer is idle only
            // until the next think), the counts at every 25th.
            for step in 0..3000 {
                g.step();
                let w = &g.world;
                for h in 0..w.houses.len() {
                    let house = HouseId(h as u8);
                    if step % 25 == 0 {
                        let counts = Special::en_route(w, house);
                        for &b in w.objects.keys() {
                            let want = en_route_as_before(w, house, b);
                            assert_eq!(counts.get(b), want, "{map_name}, tick {}, {b:?}", w.tick);
                            bound += want;
                        }
                        let seen = Special::seen_buildings(w, house);
                        let ranked = Special::spy_buildings(w, house);
                        for o in w.objects.values() {
                            if o.owner != Some(house) || o.kind != Kind::Infantry {
                                continue;
                            }
                            let now = Special::engineer_targets(w, house, o.id, &seen);
                            let before = engineer_targets_as_before(w, house, o.id);
                            assert_eq!(now, before, "{map_name}, tick {}, {:?}", w.tick, o.id);
                            targets += now.len();
                            let now = Special::spy_targets(w, house, o.id, &ranked);
                            let before = spy_targets_as_before(w, house, o.id);
                            assert_eq!(now, before, "{map_name}, tick {}, {:?}", w.tick, o.id);
                            targets += now.len();
                        }
                    }
                    let script = Script::new(house, crate::brain::default_plan());
                    let own = script.own(w);
                    let (mut now, mut before) = (Special::default(), Special::default());
                    let (mut now_out, mut before_out) = (Vec::new(), Vec::new());
                    now.engineers(&script, &own, w, &mut now_out);
                    engineers_as_before(&mut before, &script, &own, w, &mut before_out);
                    assert_eq!(now_out, before_out, "{map_name}, tick {}", w.tick);
                    assert_eq!(
                        format!("{now:?}"),
                        format!("{before:?}"),
                        "{map_name}, tick {}",
                        w.tick
                    );
                    ordered += usize::from(!now_out.is_empty());
                }
            }
        }
        assert!(
            bound > 0 && targets > 0 && ordered > 0,
            "{bound} bound, {targets} targets, {ordered} ordered"
        );
    }
}
