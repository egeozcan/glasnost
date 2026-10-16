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
}

/// The special-ability part of one AI house's brain.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Special {
    uses: Uses,
}

/// The click a person would give `unit` on `target`, as orders, when the
/// cursor over it shows one of `want`.
fn click_if(
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
        self.garrison(house, &own, home, w, out);
        self.engineers(script, &own, w, out);
        self.spies(script, &own, w, out);
        self.jumpjets(script, &own, w, out);
        self.deployers(script, &own, w, out);
        self.radiation(script, &own, w, out);
    }

    /// Counts one deploy order given to a deployable unit (S-0517,
    /// `deployers`).
    pub(crate) fn count_deploy(&mut self) {
        self.uses.deploys += 1;
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
                .filter(|&b| garrison::may_enter(w, id, b) && free(b, &sent))
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
                .filter(|&&(b, _)| !taken.contains(&b) && Self::en_route(w, house, b) == 0)
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
                .filter(|&(b, _)| !taken.contains(&b) && Self::en_route(w, house, b) == 0)
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
