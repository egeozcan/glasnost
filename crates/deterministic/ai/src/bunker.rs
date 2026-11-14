//! The own AI and tank bunkers (S-0606; PLAN §4.7).
//!
//! A tank bunker takes one turreted vehicle in; while it is bunkered the
//! vehicle fires faster and further from behind the bunker's walls, and
//! hits on it may land on the bunker instead (S-0606 claim 9 and the specs
//! it cites). A person who knows this:
//!
//! - **builds bunkers near the base.** Early, without waiting for its
//!   harvesters, it builds up to [`WANTED`] bunkers, the cheapest type it can build,
//!   when it holds [`RESERVE`] times the price ([`build_type`]); the brain
//!   places them by its ordinary placement, in the spiral around the
//!   construction yard, so they stand in the base as defences;
//! - **sends suitable vehicles in.** An empty bunker with no vehicle on its
//!   way gets the nearest idle vehicle of the house within [`HOME_REACH`]
//!   cells of it whose *enter* cursor the bunker shows (S-0214; the bunker's
//!   permission of S-0606 claim 2 decides it, so only bunker-eligible
//!   vehicles are chosen). Harvesters, slave miners, construction vehicles,
//!   transports carrying passengers, mind controllers and magnetic lifters
//!   (S-0607, kept at home by `lifter`) are never sent. Nor is a vehicle
//!   sent where it would be let out again at once: not into a bunker down
//!   to [`RETREAT_PERCENT`], and not into one under threat (see *attack*);
//! - **lets them out with a deploy click** (the bunkered vehicle's deploy
//!   cursor, S-0214 table VD row VD5, which gives it *unload*: S-0606
//!   claim 12) to:
//!   - *repair*: the vehicle is down to [`REPAIR_PERCENT`] of its strength,
//!     so it can drive to a repair depot;
//!   - *retreat*: the bunker is down to [`RETREAT_PERCENT`] of its strength,
//!     so the vehicle is not caught when it falls;
//!   - *attack*: the bunker is under threat, a seen enemy within
//!     [`ATTACK_REACH`] cells of it or one whose target is the bunker, and
//!     the vehicle has no target (it cannot reach it from where it stands).
//!     An enemy near another building of the house does not count: the
//!     bunker is most useful as an enemy comes near it (#427);
//!   - *join*: the vehicle has sat in its bunker for [`JOIN_AFTER`] ticks,
//!     has nothing to shoot and its bunker is under no threat: the bunker guards
//!     nothing, so the vehicle comes out to go with the next wave, and an
//!     idle vehicle may take its place. A vehicle let out is not sent back
//!     in for [`REENTRY_PAUSE`] ticks.
//!
//! The brain's base defence leaves a vehicle on its way into a bunker, or
//! in one, to this part.
//!
//! Every order is one a person gives with the mouse. It reads only the
//! house's own objects, the enemies it has seen and the rules a person may
//! read, and draws no random value. The numbers are tuning of our own
//! opponent, not game behaviour.

use std::collections::BTreeMap;

use glasnost_map::CellXY;
use glasnost_sim::econ::{TypeRef, production, tech};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::bunker;
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{mind, psychic, transport};

use serde::{Deserialize, Serialize};

use crate::scripted::{self, Script, all_types, dist2};
use crate::special::{Special, click_if};

/// How many tank bunkers a house wants.
pub const WANTED: usize = 2; // nocite: tuning of our own opponent

/// The credits kept in hand, as a multiple of the bunker's price, before
/// one is ordered.
const RESERVE: i32 = 2; // nocite: tuning of our own opponent

/// Cells from a bunker within which an idle vehicle is sent into it.
pub const HOME_REACH: i64 = 14; // nocite: tuning of our own opponent

/// A bunkered vehicle at or below this percentage of its strength is let
/// out to be repaired.
pub const REPAIR_PERCENT: i32 = 40; // nocite: tuning of our own opponent

/// A bunkered vehicle whose bunker is at or below this percentage of its
/// strength is let out.
pub const RETREAT_PERCENT: i32 = 25; // nocite: tuning of our own opponent

/// Cells from a bunker within which a seen enemy the vehicle has no target
/// on lets it out to attack.
pub const ATTACK_REACH: i64 = 10; // nocite: tuning of our own opponent

/// Ticks a vehicle sits in its bunker with nothing to do before it is let
/// out to join the army.
pub const JOIN_AFTER: i32 = 1200; // nocite: tuning of our own opponent

/// Ticks after being let out before a vehicle is sent into a bunker again.
pub const REENTRY_PAUSE: i32 = 1500; // nocite: tuning of our own opponent

/// What the bunker part of the brain remembers between thinks.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Watch {
    /// The bunkered vehicles and the tick each was first seen inside.
    inside_since: BTreeMap<ObjectId, i32>,
    /// The vehicles let out and the tick of the deploy click.
    let_out: BTreeMap<ObjectId, i32>,
}

/// Whether building type `t` is a tank bunker.
pub fn bunker_type(w: &World, t: TypeRef) -> bool {
    t.kind == Kind::Building
        && w.rules()
            .buildings
            .get(t.index as usize)
            .is_some_and(|b| b.raw.bunker.unwrap_or(false))
}

/// The house's tank bunkers on the map, live.
pub fn bunkers(w: &World, house: HouseId) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|o| o.owner == Some(house) && o.on_map && o.health > 0)
        .filter(|o| bunker::is_bunker(w, o.id))
        .map(|o| o.id)
        .collect()
}

/// How many tank bunkers the house has or has queued.
fn count(w: &World, house: HouseId) -> usize {
    let on_map = w
        .objects
        .values()
        .filter(|o| o.owner == Some(house) && bunker_type(w, tech::type_of(o)))
        .count();
    let queued: usize = all_types(w)
        .into_iter()
        .filter(|&t| bunker_type(w, t))
        .map(|t| {
            production::line_of(w, house, t).map_or(0, |l| {
                usize::from(l.item.as_ref().is_some_and(|i| i.t == t))
                    + l.queue.iter().filter(|&&q| q == t).count()
            })
        })
        .sum();
    on_map + queued
}

/// The cheapest tank bunker type the house can build and place now, while
/// it has fewer than [`WANTED`] and can pay [`RESERVE`] times the price.
pub(crate) fn build_type(script: &Script, w: &World) -> Option<TypeRef> {
    let house = script.house;
    if count(w, house) >= WANTED {
        return None;
    }
    let credits = glasnost_sim::seams::available_credits(w, house);
    all_types(w)
        .into_iter()
        .filter(|&t| bunker_type(w, t))
        .filter(|&t| script.startable(w, t))
        .filter(|&t| !script.busy(w, t))
        .filter(|&t| credits >= production::cost_field(w, t).saturating_mul(RESERVE))
        .filter(|&t| script.place_cell(w, t).is_some())
        .min_by_key(|&t| (production::cost_field(w, t), t.index))
}

/// Whether bunker `b` is taken: a vehicle is bunkered in it, or one is in
/// contact with it or on its way (in *enter* with it as destination).
pub fn taken(w: &World, b: ObjectId) -> bool {
    bunker::occupied(w, b)
        || w.objects.values().any(|o| {
            o.id != b
                && (o.contact == Some(b)
                    || (o.dest == Some(Target::Object(b))
                        && (o.beh.current == Some(Behaviour::Enter)
                            || o.beh.queued == Some(Behaviour::Enter))))
        })
}

/// Whether vehicle `id` may be sent into a bunker: an idle, selectable
/// vehicle above [`REPAIR_PERCENT`] of its strength, not berserk, in no
/// contact, and none of the kept kinds
/// (harvesters, slave miners, construction vehicles, mind controllers,
/// transports carrying passengers, magnetic lifters).
fn free(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if o.kind != Kind::Vehicle || !o.on_map || psychic::berserk(w, id) || o.contact.is_some() {
        return false;
    }
    let t = tech::type_of(o);
    let kept = w
        .rules()
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.harvester)
        || scripted::is_slave_miner(w, t)
        || scripted::deploys_into_yard(w, o.kind, o.type_index)
        || mind::controller(w, id).is_some()
        || transport::count(w, id) > 0
        || crate::lifter::is_lifter(w, id);
    // A vehicle due for repair is not sent: it would be let out again at once.
    !kept
        && percent(w, id) > REPAIR_PERCENT
        && scripted::idle(w, id)
        && targeting::selectable(w, id)
}

/// A percentage of an object's strength left.
fn percent(w: &World, id: ObjectId) -> i32 {
    let s = glasnost_sim::combat::damage::strength(w, id);
    w.obj(id).map_or(0, |o| {
        if s > 0 {
            o.health.saturating_mul(100) / s // nocite: a percentage for our own tuning
        } else {
            100 // nocite: a percentage for our own tuning
        }
    })
}

/// Whether bunker `b` is under threat: a seen enemy stands within
/// [`ATTACK_REACH`] cells of one of its cells, or a seen enemy's target is
/// the bunker (a person sees the shots come in).
fn threatened(w: &World, b: ObjectId, enemies: &[(ObjectId, CellXY)]) -> bool {
    let Some(bo) = w.obj(b) else { return false };
    let cells = crate::cells_of(w, bo);
    enemies.iter().any(|&(e, c)| {
        // design: own AI (#427): the threat near the bunker itself, not near any building of the house
        cells
            .iter()
            .any(|&x| dist2(c, x) <= ATTACK_REACH * ATTACK_REACH)
            || w.obj(e)
                .is_some_and(|o| o.target == Some(Target::Object(b)))
    })
}

/// Why bunkered vehicle `v` in bunker `b` is let out, if it is, with
/// `attacked` the bunker test of [`threatened`].
fn exit_reason(
    w: &World,
    v: ObjectId,
    b: ObjectId,
    joins: bool,
    attacked: bool,
) -> Option<&'static str> {
    if percent(w, v) <= REPAIR_PERCENT {
        return Some("repair");
    }
    if percent(w, b) <= RETREAT_PERCENT {
        return Some("retreat");
    }
    if w.obj(v)?.target.is_some() {
        return None;
    }
    // An enemy near the bunker or aiming at it, out of the vehicle's reach.
    if attacked {
        return Some("attack");
    }
    joins.then_some("join")
}

impl Special {
    /// Tank bunkers: bunkered vehicles let out first (repair, retreat,
    /// attack, join the army), then each empty bunker gets the nearest free
    /// vehicle by the *enter* click.
    pub(crate) fn bunkers(
        &mut self,
        script: &Script,
        own: &[ObjectId],
        w: &World,
        out: &mut Vec<Order>,
    ) {
        let house = script.house;
        let buildings = bunkers(w, house);
        let now = w.tick;
        self.bunker
            .let_out
            .retain(|_, &mut t| now - t < REENTRY_PAUSE);
        let held: Vec<ObjectId> = buildings
            .iter()
            .filter_map(|&b| bunker::occupant(w, b))
            .collect();
        self.bunker.inside_since.retain(|v, _| held.contains(v));
        for &v in &held {
            self.bunker.inside_since.entry(v).or_insert(now);
        }
        if buildings.is_empty() {
            return;
        }
        let mods = Modifiers::default();
        // Asked once a bunker for the think: the exits and the entries both read it.
        let enemies = script.seen_enemies(w);
        let attacked: BTreeMap<ObjectId, bool> = buildings
            .iter()
            .map(|&b| (b, threatened(w, b, &enemies)))
            .collect();
        let attacked = |b: ObjectId| attacked.get(&b).copied().unwrap_or(false);
        for &b in &buildings {
            let Some(v) = bunker::occupant(w, b) else {
                continue;
            };
            // The house's own vehicle only: a captured bunker's occupant
            // stays its own house's (S-0606 claim 15).
            if w.obj(v).and_then(|o| o.owner) != Some(house) || !targeting::selectable(w, v) {
                continue;
            }
            let joins = self
                .bunker
                .inside_since
                .get(&v)
                .is_some_and(|&t| now - t >= JOIN_AFTER);
            if exit_reason(w, v, b, joins, attacked(b)).is_some()
                && targeting::cursor(w, house, &[v], ClickOn::Object(v), mods) == Action::Deploy
            {
                out.push(Order::Deploy { actor: v });
                self.bunker.let_out.insert(v, now);
                self.offer_use(crate::special::Use::BunkerExit, out);
            }
        }
        // design: own AI (#18): no vehicle is sent in while it would be let out at the next think: not into a bunker down to RETREAT_PERCENT, and not into one under threat (the vehicle would come out to fight); every idle vehicle near the base went in and out in turn
        // design: own AI (#427): the same bunker test as the exit's, so the threat near that bunker
        let mut sent: Vec<ObjectId> = Vec::new();
        for &b in &buildings {
            if taken(w, b)
                || bunker::occupant(w, b).is_some()
                || percent(w, b) <= RETREAT_PERCENT
                || attacked(b)
            {
                continue;
            }
            let Some(at) = w.obj(b).map(|o| o.pos.cell()) else {
                continue;
            };
            let pick = own
                .iter()
                .copied()
                .filter(|&id| {
                    !sent.contains(&id) && !self.bunker.let_out.contains_key(&id) && free(w, id)
                })
                .filter(|&id| self.memory.ok(w, id, Target::Object(b)))
                .filter_map(|id| {
                    let d = dist2(w.obj(id)?.pos.cell(), at);
                    (d <= HOME_REACH * HOME_REACH).then_some((d, id))
                })
                .filter(|&(_, id)| {
                    targeting::cursor(w, house, &[id], ClickOn::Object(b), mods) == Action::Enter
                })
                .min()
                .map(|(_, id)| id);
            let Some(id) = pick else { continue };
            if let Some(orders) = click_if(w, house, id, b, &[Action::Enter]) {
                out.extend(orders);
                sent.push(id);
                self.offer_use(crate::special::Use::BunkerEntry, out);
            }
        }
    }
}
