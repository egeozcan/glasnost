//! The special abilities of the own AI (M6c; PLAN §4.7): garrisoning
//! buildings (S-0508), engineers capturing buildings (S-0509) and spies
//! infiltrating them (S-0510).
//!
//! Like everything the brain does, it uses an ability the way a person in
//! its seat could, with no limit of attention or speed: it clicks (through
//! `targeting::click`, so the cursor rules of S-0214 decide what the click
//! means) on objects of cells its house has explored, and issues the
//! resulting orders. It reads no hidden state and draws no random value.
//!
//! The counters in [`Uses`] are the orders it gave for each ability, kept
//! with the brain (saved and hashed with it) and printed by the headless
//! runner.

use glasnost_map::CellXY;
use glasnost_sim::econ::shroud;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Kind;
use glasnost_sim::structures::garrison;
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use serde::{Deserialize, Serialize};

use crate::scripted::{self, dist2};

/// How far, in cells, from an idle occupier a garrison building is looked
/// for.
const GARRISON_REACH: i64 = 24; // nocite: tuning of our own opponent

/// How many orders the AI gave to use each special ability.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Uses {
    /// Infantry sent to garrison a building (S-0508).
    pub garrisons: u32,
    /// Engineers sent to capture or repair a building (S-0509).
    pub engineers: u32,
    /// Spies sent to infiltrate a building (S-0510).
    pub spies: u32,
}

/// The special-ability part of one AI house's brain.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Special {
    uses: Uses,
}

/// The click a person would give `unit` on `target`, as orders, when the
/// cursor over it shows `want`.
fn click_if(
    w: &World,
    house: HouseId,
    unit: ObjectId,
    target: ObjectId,
    want: Action,
) -> Option<Vec<Order>> {
    let mods = Modifiers::default();
    let on = ClickOn::Object(target);
    (targeting::cursor(w, house, &[unit], on, mods) == want)
        .then(|| targeting::click(w, house, &[unit], on, mods).orders)
}

impl Special {
    /// The orders given so far.
    pub fn uses(&self) -> Uses {
        self.uses
    }

    /// Thinks once per think period: every ability in turn.
    pub fn think(&mut self, house: HouseId, own: &[ObjectId], w: &World, out: &mut Vec<Order>) {
        self.garrison(house, own, w, out);
    }

    /// Whether the house's object `b` stands on a cell the house has
    /// explored (what it can see, so what it may click).
    fn seen(w: &World, house: HouseId, b: ObjectId) -> bool {
        w.obj(b)
            .is_some_and(|o| o.on_map && shroud::explored(w, house, o.pos.cell()))
    }

    /// Infantry en route to garrison building `b`: own units in *capture*
    /// with the building as destination.
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
    /// cursor of S-0214 row I8) on an explored cell within reach and with a
    /// free place, civilian garrison buildings before the house's own.
    fn garrison(&mut self, house: HouseId, own: &[ObjectId], w: &World, out: &mut Vec<Order>) {
        let buildings: Vec<ObjectId> = w
            .objects
            .values()
            .filter(|o| {
                o.kind == Kind::Building
                    && o.on_map
                    && o.health > 0
                    && garrison::garrisonable(w, o.id)
            })
            .map(|o| o.id)
            .filter(|&b| Self::seen(w, house, b))
            .collect();
        if buildings.is_empty() {
            return;
        }
        // Places the orders of this think already filled.
        let mut sent: Vec<(ObjectId, i32)> = Vec::new();
        for &id in own {
            let Some(o) = w.obj(id) else { continue };
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
                    w.obj(b).is_some_and(|bo| {
                        dist2(bo.pos.cell(), at) <= GARRISON_REACH * GARRISON_REACH
                    }) && garrison::may_enter(w, id, b)
                        && free(b, &sent)
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
            if let Some(orders) = click_if(w, house, id, b, Action::Capture) {
                if orders.is_empty() {
                    continue;
                }
                out.extend(orders);
                sent.push((b, 1));
                self.uses.garrisons += 1;
            }
        }
    }
}
