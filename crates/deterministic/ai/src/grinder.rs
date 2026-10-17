//! The own AI and recycling buildings (S-0603; PLAN §4.7).
//!
//! A recycling building takes in a unit sent to it with the recycle order
//! and pays the building's owner what selling that unit would give its own
//! owner (S-0603 claims 1, 5 and 8). A person who knows this:
//!
//! - **builds one.** Once the house keeps its harvesters and has no
//!   recycling building, it builds the cheapest recycling building type it
//!   can build and place now, with a reserve in hand
//!   ([`crate::brain::Brain`] calls [`build_type`]);
//! - **grinds mind-controlled enemy units.** A controlled unit is consumed
//!   as an object of the controlling house: the house is paid for it and
//!   the controller's slot is freed for a new victim (claim 9). Every idle
//!   controlled enemy vehicle is sent. Controlled infantry go to an
//!   absorbing building while one has room (`absorber`, which also frees the
//!   slot and gives power), and to the grinder only when none has;
//! - **grinds badly damaged units at home.** An idle own unit at home at or
//!   below [`WOUNDED_PERCENT`] of its strength, when the house has no
//!   repair depot to send it to, is turned into money;
//! - **grinds surplus infantry when short of money.** With no absorbing
//!   building to take them, idle own infantry at home beyond
//!   [`crate::absorber::KEEP`] are sent, the cheapest first, while the
//!   house's credits are below [`LOW_MONEY`].
//!
//! It never sends a harvester, a slave miner or a slave, an engineer, a
//! spy, a controller or a unit with passengers, and every order is the
//! click a person gives on the building (the *recycle* cursor, S-0214 rows
//! I26 and n2), so a switched-off, sold or enemy building is never chosen.
//! It reads only the house's own objects and the rules a person may read,
//! and draws no random value. The numbers are tuning of our own opponent,
//! not game behaviour.

use glasnost_map::CellXY;
use glasnost_sim::econ::{TypeRef, production, slaves, tech};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::grinder;
use glasnost_sim::targeting::{self, Action};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{mind, psychic, transport};

use crate::absorber::{HOME_REACH, KEEP};
use crate::scripted::{self, Script, all_types, dist2};
use crate::special::{Special, click_if};

/// An own unit at or below this percentage of its strength counts as badly
/// damaged.
pub const WOUNDED_PERCENT: i64 = 30; // nocite: tuning of our own opponent

/// Credits below which a house grinds its surplus infantry.
pub const LOW_MONEY: i32 = 1000; // nocite: tuning of our own opponent

/// The most units sent in one think.
const PER_THINK: usize = 2; // nocite: tuning of our own opponent

/// The credits kept in hand beyond the grinder's cost, as a multiple of it.
const RESERVE: i32 = 2; // nocite: tuning of our own opponent

/// Whether building type `t` recycles (S-0603 Terms: the grinding flag).
fn grinder_type(w: &World, t: TypeRef) -> bool {
    t.kind == Kind::Building
        && w.rules()
            .buildings
            .get(t.index as usize)
            .is_some_and(|b| b.raw.grinding.unwrap_or(false))
}

/// The house's recycling buildings on the map, live.
pub fn grinders(w: &World, house: HouseId) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|o| {
            o.owner == Some(house) && o.on_map && o.health > 0 && grinder::is_recycler(w, o.id)
        })
        .map(|o| o.id)
        .collect()
}

/// The cheapest recycling building type the house can build and place now,
/// when it has none (nor one on its line) and holds the reserve.
pub(crate) fn build_type(script: &Script, w: &World) -> Option<TypeRef> {
    let house = script.house;
    if script
        .own(w)
        .iter()
        .any(|&id| w.obj(id).is_some_and(|o| grinder_type(w, tech::type_of(o))))
    {
        return None;
    }
    let credits = glasnost_sim::seams::available_credits(w, house);
    all_types(w)
        .into_iter()
        .filter(|&t| grinder_type(w, t))
        .filter(|&t| tech::buildable(w, house, t))
        .filter(|&t| !script.busy(w, t))
        .filter(|&t| credits >= production::cost_field(w, t).saturating_mul(RESERVE))
        .filter(|&t| script.place_cell(w, t).is_some())
        .min_by_key(|&t| (production::cost_field(w, t), t.index))
}

/// Whether unit `id` is bound for a recycling building: *eaten* current or
/// queued.
fn bound(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        o.beh.current == Some(Behaviour::Eaten) || o.beh.queued == Some(Behaviour::Eaten)
    })
}

/// Whether unit `id` may be sent at all: an idle, selectable infantry unit
/// or vehicle, not berserk, not bound already, and none of the kept kinds.
fn free(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if !matches!(o.kind, Kind::Infantry | Kind::Vehicle)
        || !o.on_map
        || psychic::berserk(w, id)
        || bound(w, id)
        || o.contact.is_some()
    {
        return false;
    }
    let t = tech::type_of(o);
    let kept = w.rules().unit(o.kind, o.type_index).is_some_and(|u| {
        u.harvester || u.engineer || u.raw.agent.unwrap_or(false)
    }) || scripted::is_slave_miner(w, t)
        || slaves::is_slave(w, id)
        || mind::controller(w, id).is_some()
        || transport::count(w, id) > 0
        || scripted::deploys_into_yard(w, o.kind, o.type_index);
    !kept && scripted::idle(w, id) && targeting::selectable(w, id)
}

impl Special {
    /// Recycling buildings: idle controlled enemy vehicles (and infantry
    /// when no absorbing building has room), then badly damaged own units
    /// at home, then surplus own infantry at home while money is short,
    /// each to the nearest own recycling building, by the *recycle* click.
    pub(crate) fn grinders(
        &mut self,
        script: &Script,
        own: &[ObjectId],
        w: &World,
        out: &mut Vec<Order>,
    ) {
        let house = script.house;
        let buildings = grinders(w, house);
        if buildings.is_empty() {
            return;
        }
        // Units this think already gave an order to are left alone.
        let ordered: Vec<ObjectId> = out.iter().filter_map(Order::actor).collect();
        let free: Vec<ObjectId> = own
            .iter()
            .copied()
            .filter(|&id| !ordered.contains(&id) && free(w, id))
            .collect();
        let absorber_room = crate::absorber::room_left(w, house);
        let controlled: Vec<ObjectId> = free
            .iter()
            .copied()
            .filter(|&id| mind::mind_controlled(w, id))
            .filter(|&id| w.obj(id).is_some_and(|o| o.kind == Kind::Vehicle) || !absorber_room)
            .collect();
        let home = script.home(w);
        let at_home = |id: ObjectId| {
            home.is_some_and(|h| {
                w.obj(id)
                    .is_some_and(|o| dist2(o.pos.cell(), h) <= HOME_REACH * HOME_REACH)
            })
        };
        let mine: Vec<ObjectId> = free
            .iter()
            .copied()
            .filter(|&id| !mind::mind_controlled(w, id) && at_home(id))
            .collect();
        let depot = own.iter().any(|&id| glasnost_sim::depot::is_depot(w, id));
        let wounded: Vec<ObjectId> = mine
            .iter()
            .copied()
            .filter(|&id| {
                let Some(o) = w.obj(id) else { return false };
                let strength = i64::from(glasnost_sim::combat::damage::strength(w, id));
                // A vehicle goes to a repair depot when the house has one (the brain's repair step).
                !(depot && o.kind == Kind::Vehicle)
                    && i64::from(o.health) * 100 <= strength * WOUNDED_PERCENT
            })
            .collect();
        let mut surplus: Vec<ObjectId> = Vec::new();
        if crate::absorber::absorbers(w, house).is_empty()
            && glasnost_sim::seams::available_credits(w, house) < LOW_MONEY
        {
            let cost = |id: ObjectId| {
                w.obj(id)
                    .map_or(0, |o| production::cost_field(w, tech::type_of(o)))
            };
            let mut spare: Vec<ObjectId> = mine
                .iter()
                .copied()
                .filter(|&id| {
                    w.obj(id).is_some_and(|o| o.kind == Kind::Infantry) && !wounded.contains(&id)
                })
                .collect();
            spare.sort_by_key(|&id| (cost(id), id));
            let n = spare.len().saturating_sub(KEEP);
            spare.truncate(n);
            surplus = spare;
        }
        let mut sent = 0;
        for id in controlled.into_iter().chain(wounded).chain(surplus) {
            if sent >= PER_THINK {
                break;
            }
            let Some(at) = w.obj(id).map(|o| o.pos.cell()) else {
                continue;
            };
            let pick = buildings
                .iter()
                .copied()
                .filter(|&b| self.memory.ok(w, id, Target::Object(b)))
                .filter(|&b| {
                    targeting::cursor(
                        w,
                        house,
                        &[id],
                        targeting::ClickOn::Object(b),
                        targeting::Modifiers::default(),
                    ) == Action::Recycle
                })
                .min_by_key(|&b| (distance(w, b, at), b));
            let Some(b) = pick else { continue };
            if let Some(orders) = click_if(w, house, id, b, &[Action::Recycle]) {
                out.extend(orders);
                sent += 1;
                self.count_grind();
            }
        }
    }
}

fn distance(w: &World, b: ObjectId, at: CellXY) -> i64 {
    w.obj(b).map_or(i64::MAX, |o| dist2(o.pos.cell(), at))
}
