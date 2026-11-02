//! The own AI and drainers (S-0608; PLAN §4.7).
//!
//! A drainer's draining weapon forms a link with an enemy building of a
//! drainable type when it is fired from over the building's foundation
//! (S-0608 claims 1 and 2). While the link lasts, a building of a
//! resource-destination type pays its house's credits to the drainer's
//! house (claims 9 and 10), and a power plant's house is blacked out
//! (S-0405 claims 5 and 7). The link ends when the drainer leaves the
//! foundation or either side goes (claims 12 to 16). A person who knows
//! this:
//!
//! - **sends idle drainers at the enemy's economy.** An idle drainer that
//!   drains nothing is sent by the attack click at a seen enemy building
//!   it could drain now: of a drainable type, not drained already and not
//!   refused by the firing check with the weapon the drainer would choose
//!   against it, which is then the draining one (S-0308 table P row 8). A
//!   building that pays credits (a resource destination) comes first, then
//!   a power plant, then any other; the nearest first within each. No two
//!   drainers are sent at one building;
//! - **leaves a drainer alone while it drains or flies to drain.** No
//!   other part of the brain gives a draining drainer an order, since any
//!   move off the foundation ends the link, and a drainer on its way to a
//!   building takes no wave order. No other part sends a second drainer
//!   at a building one drains or flies to ([`filter_orders`]);
//! - **builds a few drainers.** It keeps up to [`WANTED`] drainers, the
//!   cheapest type it can build, the way it keeps lifters
//!   ([`build_type`]).
//!
//! Every order is one a person gives with the mouse. It reads only the
//! house's own objects, the enemies it has seen and the rules a person may
//! read. It draws no random value. The numbers are tuning of our own
//! opponent, not game behaviour. A ruleset without a drainer type leaves
//! the brain as it was.

use glasnost_sim::combat::check::{self, Answer};
use glasnost_sim::combat::weapon;
use glasnost_sim::drain;
use glasnost_sim::econ::{TypeRef, production, tech};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::Action;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

use crate::scripted::{Script, all_types, dist2};
use crate::special::{Special, click_if};

/// How many drainers a house wants.
pub const WANTED: usize = 3; // nocite: tuning of our own opponent

/// The credits kept in hand, as a multiple of the drainer's price, before
/// one is ordered.
const RESERVE: i32 = 2; // nocite: tuning of our own opponent

/// Whether unit type `t` is a drainer: one of its weapons is a draining
/// weapon (S-0608 Terms).
pub fn drainer_type(w: &World, t: TypeRef) -> bool {
    if !matches!(t.kind, Kind::Vehicle | Kind::Infantry | Kind::Aircraft) {
        return false;
    }
    let r = w.rules();
    r.unit(t.kind, t.index).is_some_and(|u| {
        u.combat
            .slots
            .iter()
            .flatten()
            .any(|&wp| r.combat.weapon(wp).drain)
    })
}

/// Whether object `id` is a drainer.
pub fn is_drainer(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| drainer_type(w, tech::type_of(o)))
}

/// How many drainers the house has or has queued.
fn count(w: &World, house: HouseId) -> usize {
    let on_map = w
        .objects
        .values()
        .filter(|o| o.owner == Some(house) && o.health > 0 && drainer_type(w, tech::type_of(o)))
        .count();
    let queued: usize = all_types(w)
        .into_iter()
        .filter(|&t| drainer_type(w, t))
        .map(|t| {
            production::line_of(w, house, t).map_or(0, |l| {
                usize::from(l.item.as_ref().is_some_and(|i| i.t == t))
                    + l.queue.iter().filter(|&&q| q == t).count()
            })
        })
        .sum();
    on_map + queued
}

/// The cheapest drainer type the house can build now, while it has fewer
/// than [`WANTED`] and can pay [`RESERVE`] times the price. As for lifters,
/// the first is queued behind whatever the factory builds; a further one
/// waits for a free line and the reserve.
pub(crate) fn build_type(script: &Script, w: &World) -> Option<TypeRef> {
    let house = script.house;
    let have = count(w, house);
    if have >= WANTED {
        return None;
    }
    let credits = glasnost_sim::seams::available_credits(w, house);
    let first = have == 0;
    all_types(w)
        .into_iter()
        .filter(|&t| drainer_type(w, t))
        .filter(|&t| crate::startable(w, house, t))
        .filter(|&t| first || !script.busy(w, t))
        .filter(|&t| first || credits >= production::cost_field(w, t).saturating_mul(RESERVE))
        .min_by_key(|&t| (production::cost_field(w, t), t.index))
}

/// Whether drainer `d` could drain building `b` now: an enemy building on
/// the map, of a drainable type, not drained already, against which `d`
/// would choose its draining weapon (S-0308 table P row 8), and which the
/// firing check does not refuse it (S-0608 claim 3).
pub fn drain_target(w: &World, d: ObjectId, b: ObjectId) -> bool {
    let (Some(dobj), Some(bo)) = (w.obj(d), w.obj(b)) else {
        return false;
    };
    if bo.kind != Kind::Building || !bo.on_map || bo.health <= 0 {
        return false;
    }
    if w.allied(dobj.owner, bo.owner) || !drain::drainable(w, b) || drain::being_drained(w, b) {
        return false;
    }
    let target = Some(Target::Object(b));
    let slot = weapon::choose_slot(w, d, target);
    let Some(t) = w.rules().unit(dobj.kind, dobj.type_index) else {
        return false;
    };
    let draining = t.combat.slots[slot].is_some_and(|wp| w.rules().combat.weapon(wp).drain);
    // spec: S-0214 claim 6 (e) and S-0301 claim 9 (the check without the reasons of the moment of ranks 5 to 8, so that reloading does not hide an illegal target)
    draining && check::check_lasting(w, d, target, slot) != Answer::Illegal
}

/// Whether drainer `d` is on its way to drain: it attacks a building it
/// could drain. A drainer that only keeps such a target (in guard, say)
/// is not on its way: it does not go there, so it is sent again.
fn on_the_way(w: &World, d: ObjectId) -> bool {
    w.obj(d).is_some_and(|o| {
        o.beh.current == Some(Behaviour::Attack)
            && matches!(o.target, Some(Target::Object(b)) if drain_target(w, d, b))
    })
}

/// The order of preference among buildings to drain: one that pays
/// credits, then a power plant, then any other.
fn rank(w: &World, b: ObjectId) -> u8 {
    if drain::resource_destination(w, b) {
        return 0;
    }
    let power = w.obj(b).and_then(|o| {
        w.rules()
            .buildings
            .get(o.type_index as usize)
            .and_then(|t| t.raw.econ.power)
    });
    if power.unwrap_or(0) > 0 { 1 } else { 2 } // nocite: tuning of our own opponent (an order of preference)
}

impl Special {
    /// Drainers: idle ones that drain nothing are sent at a seen enemy
    /// building they could drain.
    pub(crate) fn drainers(
        &mut self,
        script: &Script,
        own: &[ObjectId],
        w: &World,
        out: &mut Vec<Order>,
    ) {
        let house = script.house;
        let drainers: Vec<ObjectId> = own
            .iter()
            .copied()
            .filter(|&id| is_drainer(w, id) && glasnost_sim::targeting::selectable(w, id))
            .collect();
        if drainers.is_empty() {
            return;
        }
        let enemies = script.seen_enemies(w);
        // Buildings an own drainer drains or flies to.
        let mut chosen: Vec<ObjectId> = drainers.iter().filter_map(|&d| claim(w, d)).collect();
        for &d in &drainers {
            if drain::draining(w, d) || on_the_way(w, d) {
                continue;
            }
            let Some(at) = w.obj(d).map(|o| o.pos.cell()) else {
                continue;
            };
            let pick = enemies
                .iter()
                .filter(|&&(b, _)| !chosen.contains(&b))
                .filter(|&&(b, _)| drain_target(w, d, b))
                .filter(|&&(b, _)| self.memory.ok(w, d, Target::Object(b)))
                .map(|&(b, c)| (rank(w, b), dist2(c, at), b))
                .min()
                .map(|(_, _, b)| b);
            if let Some(b) = pick
                && let Some(orders) = click_if(w, house, d, b, &[Action::Attack])
            {
                out.extend(orders);
                chosen.push(b);
                self.count_drain_order();
            }
        }
    }
}

/// The building drainer `d` drains or flies to drain, if any.
fn claim(w: &World, d: ObjectId) -> Option<ObjectId> {
    drain::partner(w, d).or_else(|| match w.obj(d)?.target {
        Some(Target::Object(b)) if on_the_way(w, d) => Some(b),
        _ => None,
    })
}

/// Leaves out the orders the brain does not issue to drainers. A draining
/// drainer takes no order (a move off the foundation would end the link,
/// S-0608 claim 12). An attack on a building it could drain is kept unless
/// another drainer of the house drains that building, flies to it, or got
/// an earlier such order in `out`: a second drainer would only have its
/// attempt refused (claim 3 b). A drainer on its way to drain, or given a
/// drain order earlier in `out`, takes no other order; any other drainer
/// takes every order.
pub(crate) fn filter_orders(w: &World, out: &mut Vec<Order>) {
    let mut claimed: std::collections::BTreeMap<ObjectId, ObjectId> = w
        .objects
        .values()
        .filter(|o| o.on_map && is_drainer(w, o.id))
        .filter_map(|o| Some((claim(w, o.id)?, o.id)))
        .collect();
    // Drainers given a drain order earlier in `out`: a later order of
    // another part would replace it.
    let mut sent: std::collections::BTreeSet<ObjectId> = std::collections::BTreeSet::new();
    out.retain(|order| {
        let Some(actor) = order.actor() else {
            return true;
        };
        if !is_drainer(w, actor) {
            return true;
        }
        if drain::draining(w, actor) {
            return false;
        }
        match order {
            Order::Behaviour {
                behaviour: Behaviour::Attack,
                target: Some(Target::Object(b)),
                ..
            } if drain_target(w, actor, *b) => {
                let owner = w.obj(actor).and_then(|o| o.owner);
                match claimed.get(b) {
                    Some(&other)
                        if other != actor && w.obj(other).and_then(|o| o.owner) == owner =>
                    {
                        false
                    }
                    _ => {
                        claimed.insert(*b, actor);
                        sent.insert(actor);
                        true
                    }
                }
            }
            _ => !on_the_way(w, actor) && !sent.contains(&actor),
        }
    });
}
