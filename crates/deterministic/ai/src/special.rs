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
use glasnost_sim::econ::{TypeRef, production, shroud, tech};
use glasnost_sim::orders::Order;
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

impl Special {
    /// The orders given so far.
    pub fn uses(&self) -> Uses {
        self.uses
    }

    /// Thinks once per think period: every ability in turn.
    pub fn think(&mut self, script: &Script, w: &World, out: &mut Vec<Order>) {
        let house = script.house;
        let own = script.own(w);
        let home = script.home(w);
        self.memory.review(w);
        let start = out.len();
        self.garrison(house, &own, home, w, out);
        self.engineers(script, &own, w, out);
        self.spies(script, &own, w, out);
        self.jumpjets(script, &own, w, out);
        self.deployers(script, &own, w, out);
        self.radiation(script, &own, w, out);
        self.chrono(script, &own, w, out);
        self.linked(script, &own, w, out);
        self.terror(script, &own, w, out);
        self.crates(script, &own, w, out);
        self.berserk(script, &own, w, out);
        self.mind_control(script, &own, w, out);
        self.absorbers(script, &own, w, out);
        self.grinders(script, &own, w, out);
        self.bunkers(script, &own, w, out);
        self.lifters(script, &own, w, out);
        self.drainers(script, &own, w, out);
        self.memory.note(w, &out[start..]);
    }

    /// How many (unit, goal) pairs the brain has given up for now (an order
    /// the unit did not take, see `memory`).
    pub fn given_up(&self) -> usize {
        self.memory.blocked()
    }

    /// Counts one deploy order given to a deployable unit (S-0517,
    /// `deployers`).
    pub(crate) fn count_deploy(&mut self) {
        self.uses.deploys += 1;
    }

    /// Counts one prism building ordered (S-0518, `linked`).
    pub(crate) fn count_prism(&mut self) {
        self.uses.prisms += 1;
    }

    /// Counts one overpowerable building ordered (S-0518, `linked`).
    pub(crate) fn count_overpowerable(&mut self) {
        self.uses.overpowerables += 1;
    }

    /// Counts one charger sent at a building (S-0518, `linked`).
    pub(crate) fn count_charge(&mut self) {
        self.uses.charges += 1;
    }

    /// Counts one berserk-weapon unit sent at an enemy (S-0610, `psychic`).
    pub(crate) fn count_berserk(&mut self) {
        self.uses.berserks += 1;
    }

    /// Counts one controller sent to take over an enemy (S-0601, `mind`).
    pub(crate) fn count_takeover(&mut self) {
        self.uses.takeovers += 1;
    }

    /// Counts one infantry unit sent into an absorbing building (S-0605,
    /// `absorber`).
    pub(crate) fn count_absorb(&mut self) {
        self.uses.absorbs += 1;
    }

    /// Counts one unit sent into a recycling building (S-0603, `grinder`).
    pub(crate) fn count_grind(&mut self) {
        self.uses.grinds += 1;
    }

    /// Counts one tank bunker ordered (S-0606, `bunker`).
    pub(crate) fn count_bunker_build(&mut self) {
        self.uses.bunker_builds += 1;
    }

    /// Counts one vehicle sent into a tank bunker (S-0606, `bunker`).
    pub(crate) fn count_bunker_entry(&mut self) {
        self.uses.bunker_entries += 1;
    }

    /// Counts one bunkered vehicle let out (S-0606, `bunker`).
    pub(crate) fn count_bunker_exit(&mut self) {
        self.uses.bunker_exits += 1;
    }

    /// Counts one lifter ordered (S-0607, `lifter`).
    pub(crate) fn count_lifter_build(&mut self) {
        self.uses.lifter_builds += 1;
    }

    /// Counts one lifter sent at an enemy vehicle (S-0607, `lifter`).
    pub(crate) fn count_lift_order(&mut self) {
        self.uses.lift_orders += 1;
    }

    /// Counts one drop order given to a holding lifter (S-0607, `lifter`).
    pub(crate) fn count_lift_drop(&mut self) {
        self.uses.lift_drops += 1;
    }

    /// Counts one drainer ordered (S-0608, `drain`).
    pub(crate) fn count_drainer_build(&mut self) {
        self.uses.drainer_builds += 1;
    }

    /// Counts one drainer sent at an enemy building (S-0608, `drain`).
    pub(crate) fn count_drain_order(&mut self) {
        self.uses.drain_orders += 1;
    }

    /// Counts one gattling defence building ordered (S-0609, `gattling`).
    pub(crate) fn count_gattling_defence(&mut self) {
        self.uses.gattling_defences += 1;
    }

    /// Counts attack orders dropped for gattling units that keep firing
    /// at their targets (S-0609, `gattling`).
    pub(crate) fn count_spins_kept(&mut self, n: usize) {
        self.uses.spins_kept = self.uses.spins_kept.saturating_add(n as u32);
    }

    /// Counts one psychic sensor ordered (S-0610, `psychic`).
    pub(crate) fn count_sensor(&mut self) {
        self.uses.sensors += 1;
    }

    /// Counts one psychic reveal fired (S-0610, `psychic`).
    pub(crate) fn count_reveal(&mut self) {
        self.uses.reveals += 1;
    }

    /// Counts one unit sent to a crate (S-0520, `crates`).
    pub(crate) fn count_crate(&mut self) {
        self.uses.crates += 1;
    }

    /// Counts one bomb planter sent at an enemy (S-0519, `terror`).
    pub(crate) fn count_bomb(&mut self) {
        self.uses.bombs += 1;
    }

    /// Counts one disarmer sent at a bombed object (S-0519, `terror`).
    pub(crate) fn count_disarm(&mut self) {
        self.uses.disarms += 1;
    }

    /// Counts one suicide unit sent at an enemy (S-0519, `terror`).
    pub(crate) fn count_suicide(&mut self) {
        self.uses.suicides += 1;
    }

    /// Counts one unit with charges sent at a building (S-0519, `terror`).
    pub(crate) fn count_sabotage(&mut self) {
        self.uses.sabotages += 1;
    }

    /// Counts one designator sent at a building (S-0519, `terror`).
    pub(crate) fn count_strike(&mut self) {
        self.uses.strikes += 1;
    }

    /// Counts one erasing unit sent against an enemy (S-0514, `chrono`).
    pub(crate) fn count_erasure(&mut self) {
        self.uses.erasures += 1;
    }

    /// Counts one unit sent out of a radiation field (S-0525, `radiation`).
    pub(crate) fn count_radiation(&mut self) {
        self.uses.radiation += 1;
    }

    /// Counts one order given to a jumpjet unit (S-0502, `jumpjets`).
    pub(crate) fn count_jump(&mut self) {
        self.uses.jumps += 1;
    }

    /// Whether the house's object `b` stands on a cell the house has
    /// explored (what it can see, so what it may click).
    fn seen(w: &World, house: HouseId, b: ObjectId) -> bool {
        w.obj(b)
            .is_some_and(|o| o.on_map && shroud::explored(w, house, o.pos.cell()))
    }

    /// Infantry en route to enter building `b`: own units in *capture* with
    /// the building as destination.
    fn en_route(w: &World, house: HouseId, b: ObjectId) -> i32 {
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
        let mut sent: Vec<(ObjectId, i32)> = Vec::new();
        for &id in own {
            let Some(o) = w.obj(id) else { continue };
            let total = inside
                + i32::try_from(sent.len()).unwrap_or(i32::MAX)
                + buildings
                    .iter()
                    .map(|&b| Self::en_route(w, house, b))
                    .sum::<i32>();
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
                    + Self::en_route(w, house, b)
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
                self.uses.garrisons += 1;
            }
        }
    }

    /// The cheapest type of infantry the house can build now that
    /// satisfies `f`.
    fn buildable_infantry(
        w: &World,
        house: HouseId,
        f: impl Fn(&glasnost_sim::rules::UnitRules) -> bool,
    ) -> Option<TypeRef> {
        all_types(w)
            .into_iter()
            .filter(|t| t.kind == Kind::Infantry)
            .filter(|&t| w.rules().unit(t.kind, t.index).is_some_and(&f))
            .filter(|&t| tech::buildable(w, house, t))
            .min_by_key(|&t| (production::cost_field(w, t), t.index))
    }

    /// Whether the house has, or is already making, an object that
    /// satisfies `f`: on the map, off it, in a line.
    fn has_or_makes(w: &World, house: HouseId, f: impl Fn(ObjectId) -> bool) -> bool {
        w.objects
            .values()
            .any(|o| o.owner == Some(house) && o.kind == Kind::Infantry && f(o.id))
    }

    /// Whether the line of `t` already holds one in its queue (a queued copy
    /// has no object yet).
    fn queued(w: &World, house: HouseId, t: TypeRef) -> bool {
        production::line_of(w, house, t)
            .is_some_and(|l| l.item.as_ref().is_some_and(|i| i.t == t) || l.queue.contains(&t))
    }

    /// Orders one unit of `t` from the sidebar when the house keeps enough
    /// credits in hand and the type is not already in its line.
    fn order_unit(&self, script: &Script, w: &World, t: TypeRef, out: &mut Vec<Order>) {
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
    fn engineer_targets(w: &World, house: HouseId, unit: ObjectId) -> Vec<(ObjectId, Action)> {
        let mods = Modifiers::default();
        w.objects
            .values()
            .filter(|o| o.kind == Kind::Building && o.on_map && o.health > 0)
            .map(|o| o.id)
            .filter(|&b| Self::seen(w, house, b))
            .filter_map(|b| {
                let a = targeting::cursor(w, house, &[unit], ClickOn::Object(b), mods);
                matches!(
                    a,
                    Action::Capture | Action::DamagingEntry | Action::RepairBuilding
                )
                .then_some((b, a))
            })
            .collect()
    }

    /// Engineers (S-0509): every idle engineer is sent to the nearest
    /// enemy building it may enter (capture), or, with none, to the nearest
    /// own building that is damaged (repair); the house orders an engineer
    /// when it has none and sees a building to capture.
    fn engineers(&mut self, script: &Script, own: &[ObjectId], w: &World, out: &mut Vec<Order>) {
        let house = script.house;
        let mut taken: Vec<ObjectId> = Vec::new();
        let mut any = false;
        for &id in own {
            if !engineer::is_engineer(w, id) {
                continue;
            }
            any = true;
            if !scripted::idle(w, id) || !targeting::selectable(w, id) {
                continue;
            }
            let at = w.obj(id).map_or(CellXY::new(0, 0), |o| o.pos.cell());
            let targets = Self::engineer_targets(w, house, id);
            let rank = |&(b, a): &(ObjectId, Action)| {
                (
                    // Capturing before repairing, a tech building that pays before the rest (S-0515), then the nearest, never a building another engineer is bound for.
                    a == Action::RepairBuilding,
                    tech_buildings::rank(w, b),
                    taken.contains(&b) || Self::en_route(w, house, b) > 0,
                    w.obj(b).map_or(i64::MAX, |o| dist2(o.pos.cell(), at)),
                    b,
                )
            };
            let Some(&(b, a)) = targets
                .iter()
                .filter(|&&(b, _)| {
                    !taken.contains(&b)
                        && Self::en_route(w, house, b) == 0
                        && self.memory.ok(w, id, Target::Object(b))
                })
                .min_by_key(|t| rank(t))
            else {
                continue;
            };
            if let Some(orders) = click_if(w, house, id, b, &[a]) {
                out.extend(orders);
                taken.push(b);
                self.uses.engineers += 1;
                if tech_buildings::passive_tech(w, b) {
                    self.uses.tech += 1;
                }
            }
        }
        if !any
            && let Some(t) = Self::buildable_infantry(w, house, |u| u.engineer)
            && !Self::has_or_makes(w, house, |id| engineer::is_engineer(w, id))
            && (Self::captureable_in_sight(w, house) || tech_buildings::in_sight(w, house))
        {
            self.order_unit(script, w, t, out);
        }
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
                    !me.allies.contains(&h)
                        && w.houses
                            .get(usize::from(h.0))
                            .is_some_and(|x| x.role == glasnost_sim::world::HouseRole::Player)
                })
                && w.rules()
                    .buildings
                    .get(o.type_index as usize)
                    .is_some_and(|b| b.raw.capturable.unwrap_or(false))
                && shroud::explored(w, house, o.pos.cell())
        })
    }
}

impl Special {
    /// How worth infiltrating a building is to `house`, lower first: the
    /// first of its effect rows (S-0510 claim 3) ranked by what a person
    /// wants: stolen technology, a factory flag it does not have yet, the
    /// victim's money, its power, a superweapon, its radar.
    fn infiltration_rank(w: &World, house: HouseId, b: ObjectId) -> Option<u8> {
        let me = w.houses.get(usize::from(house.0))?;
        let owner = w.obj(b)?.owner?;
        let credits = w.houses.get(usize::from(owner.0)).map_or(0, |h| h.credits);
        spy::rows(w, b).first().and_then(|row| match row {
            InfiltrationRow::Tech => Some(0),
            InfiltrationRow::VehicleFactory if !me.infiltrated_vehicle_factory => Some(1),
            InfiltrationRow::InfantryFactory if !me.infiltrated_infantry_factory => Some(1),
            InfiltrationRow::Cash if credits > 0 => Some(2), // nocite: our opponent's preference order
            InfiltrationRow::Power => Some(3), // nocite: our opponent's preference order
            InfiltrationRow::Superweapon => Some(4), // nocite: our opponent's preference order
            InfiltrationRow::Radar => Some(5), // nocite: our opponent's preference order
            _ => None,
        })
    }

    /// The enemy buildings a spy may enter, as the cursor shows (S-0214 row
    /// I24: capture) with the rank of what its entry would do.
    fn spy_targets(w: &World, house: HouseId, unit: ObjectId) -> Vec<(ObjectId, u8)> {
        let mods = Modifiers::default();
        w.objects
            .values()
            .filter(|o| o.kind == Kind::Building && o.on_map && o.health > 0)
            .map(|o| o.id)
            .filter(|&b| Self::seen(w, house, b))
            .filter_map(|b| {
                let rank = Self::infiltration_rank(w, house, b)?;
                (targeting::cursor(w, house, &[unit], ClickOn::Object(b), mods) == Action::Capture)
                    .then_some((b, rank))
            })
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
        for &id in own {
            if !spy::is_spy(w, id) || engineer::is_engineer(w, id) {
                continue;
            }
            any = true;
            if !scripted::idle(w, id) || !targeting::selectable(w, id) {
                continue;
            }
            let at = w.obj(id).map_or(CellXY::new(0, 0), |o| o.pos.cell());
            let Some((b, _)) = Self::spy_targets(w, house, id)
                .into_iter()
                .filter(|&(b, _)| {
                    !taken.contains(&b)
                        && Self::en_route(w, house, b) == 0
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
                self.uses.spies += 1;
            }
        }
        if !any
            && let Some(t) =
                Self::buildable_infantry(w, house, |u| u.raw.agent.unwrap_or(false) && !u.engineer)
            && !Self::has_or_makes(w, house, |id| spy::is_spy(w, id))
            && Self::infiltratable_in_sight(w, house)
        {
            self.order_unit(script, w, t, out);
        }
    }

    /// Whether the house sees an enemy building with a worthwhile
    /// infiltration effect.
    fn infiltratable_in_sight(w: &World, house: HouseId) -> bool {
        let me = &w.houses[usize::from(house.0)];
        w.objects.values().any(|o| {
            o.kind == Kind::Building
                && o.on_map
                && o.health > 0
                && o.owner.is_some_and(|h| {
                    h != house
                        && !me.allies.contains(&h)
                        && w.houses
                            .get(usize::from(h.0))
                            .is_some_and(|x| x.role == glasnost_sim::world::HouseRole::Player)
                })
                && shroud::explored(w, house, o.pos.cell())
                && Self::infiltration_rank(w, house, o.id).is_some()
        })
    }
}
