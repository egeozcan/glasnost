//! The own AI and absorbing buildings (S-0605; PLAN §4.7).
//!
//! An infantry absorber takes the house's infantry in, and each passenger
//! adds the building type's extra power to its owner (S-0605 claims 7 and
//! 10, S-0405 claim 1). A person who knows this:
//!
//! - **builds the absorber.** In the stock data the absorber is a side's
//!   power plant, so the plan's power role (`scripted`, the cheapest power
//!   plant the house can build) already builds it; no change is needed
//!   there;
//! - **puts mind-controlled enemy infantry in.** A controlled entrant is
//!   released by its controller on the way in and stays inside as its
//!   original owner's unit, yet its power goes to the building's owner
//!   (claims 7 and 10): a captured enemy foot soldier turns into power and
//!   frees the controller for another victim. Every idle controlled enemy
//!   infantry unit is sent. A permanently controlled one frees no
//!   controller and belongs to the house for good (S-0601 claim 13), so it
//!   counts as the house's own ([`held_by_controller`]);
//! - **puts spare infantry in.** Idle infantry of the house's own at home
//!   are sent while the house is short of power, or while more than
//!   [`KEEP`] idle infantry stand at home, the cheapest first, so the
//!   waves keep their better soldiers. Engineers, spies and mind
//!   controllers are kept for their own work;
//! - **fills one place at a time.** No building gets more entrants than it
//!   has room for, counting the ones already on their way or waiting in
//!   line (claims 3 and 4) and the stalled ones sent again, and the order
//!   is the click a person gives on the building (the *enter* cursor,
//!   S-0214 rows x3 and x4), so a full, switched-off or unfinished building
//!   is never chosen: the unit goes to the nearest one that takes it;
//! - **sends a stalled entrant again.** An entrant that has left *enter*
//!   while it still holds its link place keeps it, and every later entrant
//!   waits for good (S-0605 claim 17). Such a unit gets the *enter* click
//!   on its building again, once per cell it stands on: the new order ends
//!   its contact (S-0213 table T3 (b)) and claim 3 runs afresh.
//!
//! It reads only the house's own objects and the rules a person may read,
//! and draws no random value. The numbers are tuning of our own opponent,
//! not game behaviour.

use std::collections::BTreeMap;

use glasnost_map::CellXY;
use glasnost_sim::econ::slaves;
use glasnost_sim::econ::{production, tech};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::absorb;
use glasnost_sim::targeting::{self, Action};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{mind, psychic};

use crate::scripted::{self, Script, dist2};
use crate::special::{Special, click_if};

/// Cells from the house's home within which idle infantry count as at home.
pub const HOME_REACH: i64 = 16; // nocite: tuning of our own opponent

/// The idle own infantry at home a house keeps out of its absorbers while
/// its power is sufficient.
pub const KEEP: usize = 4; // nocite: tuning of our own opponent

/// The most entrants sent in one think.
const PER_THINK: usize = 3; // nocite: tuning of our own opponent

/// The house's infantry absorbers on the map, live and finished.
pub fn absorbers(w: &World, house: HouseId) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|o| {
            o.owner == Some(house)
                && o.kind == Kind::Building
                && o.on_map
                && o.health > 0
                && absorb::infantry_absorber(w, o.id)
        })
        .map(|o| o.id)
        .collect()
}

fn entering(o: &glasnost_sim::world::Object) -> bool {
    o.beh.current == Some(Behaviour::Enter) || o.beh.queued == Some(Behaviour::Enter)
}

/// The house's units on their way into absorbing building `b`: in contact
/// with it or with it as destination while in *enter*, or waiting in line
/// for it. A stalled entrant ([`stalled`]) does not count: it is sent again.
/// One with the building as destination but no contact and no *enter*
/// (S-0605 claim 5) does not count either.
pub fn en_route(w: &World, b: ObjectId) -> usize {
    w.objects
        .values()
        .filter(|o| {
            o.on_map
                && o.id != b
                && (absorb::waiting_for(w, o.id) == Some(b)
                    || entering(o) && (o.contact == Some(b) || o.dest == Some(Target::Object(b))))
        })
        .count()
}

/// The absorbing building infantry unit `id` has stalled on: it holds the
/// building's contact but has neither *enter* current nor queued (S-0605
/// claim 17).
pub fn stalled(w: &World, id: ObjectId) -> Option<ObjectId> {
    let o = w.obj(id)?;
    if o.kind != Kind::Infantry || !o.on_map || entering(o) {
        return None;
    }
    let b = o.contact?;
    absorb::infantry_absorber(w, b).then_some(b)
}

/// [`en_route`] for each of `buildings`, from one pass over the objects
/// (design: own AI, #64: a pass for each building, and again for the
/// grinder's question). A unit counts once for each building it is bound
/// for, as there.
fn en_route_each(w: &World, buildings: &[ObjectId]) -> BTreeMap<ObjectId, usize> {
    let mut m: BTreeMap<ObjectId, usize> = buildings.iter().map(|&b| (b, 0)).collect();
    for o in w.objects.values().filter(|o| o.on_map) {
        let waits = absorb::waiting_for(w, o.id);
        let (contact, goal) = if entering(o) {
            let goal = match o.dest {
                Some(Target::Object(b)) => Some(b),
                _ => None,
            };
            (o.contact, goal)
        } else {
            (None, None)
        };
        // Each building the unit is bound for once, not the unit itself.
        let bound = [
            waits,
            contact.filter(|&c| Some(c) != waits),
            goal.filter(|&g| Some(g) != waits && Some(g) != contact),
        ];
        for b in bound.into_iter().flatten().filter(|&b| b != o.id) {
            if let Some(n) = m.get_mut(&b) {
                *n += 1;
            }
        }
    }
    m
}

/// The room left in `b` for this think: its capacity less its passengers
/// and the `en_route` ones on their way.
fn room(w: &World, b: ObjectId, en_route: usize) -> usize {
    let cap = w
        .obj(b)
        .and_then(|o| w.rules().buildings.get(o.type_index as usize))
        .map_or(0, |r| r.absorb.capacity);
    let taken = absorb::count(w, b) as usize + en_route;
    usize::try_from(cap).unwrap_or(0).saturating_sub(taken)
}

/// The room left in each of `buildings` for this think ([`room`]).
fn rooms(w: &World, buildings: &[ObjectId]) -> Vec<(ObjectId, usize)> {
    let bound = en_route_each(w, buildings);
    buildings
        .iter()
        .map(|&b| (b, room(w, b, bound.get(&b).copied().unwrap_or(0))))
        .collect()
}

/// Whether one of the house's infantry absorbers has room left (the
/// grinder step sends controlled infantry there first, `grinder`).
pub(crate) fn room_left(w: &World, house: HouseId) -> bool {
    let buildings = absorbers(w, house);
    // No absorber, no pass over the objects.
    !buildings.is_empty() && rooms(w, &buildings).iter().any(|&(_, r)| r > 0)
}

/// Whether the house is short of power, read as a person reads the power
/// bar without a blackout ([`crate::power::short`]).
// design: own AI (#15): the house total read here before is 0 during a timed or drain blackout whatever plants stand (S-0405 claims 5 and 6), so every blackout sent all spare infantry in, the KEEP reserve too
fn short_of_power(w: &World, house: HouseId) -> bool {
    crate::power::short(w, house)
}

/// Whether infantry unit `id` is free to be sent: idle, selectable, not
/// berserk, neither an engineer, a spy nor a mind controller, and not
/// already on its way into a building.
fn free_infantry(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if o.kind != Kind::Infantry || !o.on_map || psychic::berserk(w, id) {
        return false;
    }
    let special = w
        .rules()
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.engineer || u.raw.agent.unwrap_or(false))
        || mind::controller(w, id).is_some();
    // Already on its way: waiting in line or in contact (a new order would
    // end the wait and put it at the back again).
    let going = absorb::waiting_for(w, id).is_some() || o.contact.is_some();
    // design: a slave resting by its master is that master's worker, not
    // spare infantry (the opponent is ours); inside a reactor it would never
    // mine again. The grinder leaves slaves alone too.
    !special
        && !going
        && !slaves::is_slave(w, id)
        && scripted::idle(w, id)
        && targeting::selectable(w, id)
}

impl Special {
    /// Absorbing buildings: idle controlled enemy infantry, then spare own
    /// infantry at home (cheapest first), each to the nearest own infantry
    /// absorber with room, by the *enter* click.
    pub(crate) fn absorbers(
        &mut self,
        script: &Script,
        own: &[ObjectId],
        w: &World,
        out: &mut Vec<Order>,
    ) {
        let house = script.house;
        let buildings = absorbers(w, house);
        if buildings.is_empty() {
            return;
        }
        let mut sent = 0;
        let mut rooms: Vec<(ObjectId, usize)> = rooms(w, &buildings);
        // Stalled entrants first: the enter click on their building again.
        for &id in own {
            if sent >= PER_THINK {
                break;
            }
            let Some(b) = stalled(w, id) else { continue };
            // Once per cell: a unit sent again that still stands where it
            // stood is left until its order has been judged (`memory`).
            let goal = Target::Object(b);
            if psychic::berserk(w, id)
                || !buildings.contains(&b)
                || !self.memory.ok(w, id, goal)
                || self.memory.pending(w, id, goal)
            {
                continue;
            }
            if let Some(orders) = click_if(w, house, id, b, &[Action::Enter]) {
                out.extend(orders);
                // design: own AI (#44): a stalled entrant is left out of
                // `en_route`, so the one sent again takes its room here.
                if let Some(r) = rooms.iter_mut().find(|(x, _)| *x == b) {
                    r.1 = r.1.saturating_sub(1);
                }
                sent += 1;
                self.offer_use(crate::special::Use::Absorb, out);
            }
        }
        if rooms.iter().all(|&(_, r)| r == 0) {
            return;
        }
        let free: Vec<ObjectId> = own
            .iter()
            .copied()
            .filter(|&id| free_infantry(w, id))
            .collect();
        // Controlled enemy infantry anywhere; own infantry only at home.
        let controlled: Vec<ObjectId> = free
            .iter()
            .copied()
            .filter(|&id| held_by_controller(w, id))
            .collect();
        let home = script.home(w);
        let at_home = |id: ObjectId| {
            home.is_some_and(|h| {
                w.obj(id)
                    .is_some_and(|o| dist2(o.pos.cell(), h) <= HOME_REACH * HOME_REACH)
            })
        };
        let mut spare: Vec<ObjectId> = free
            .iter()
            .copied()
            .filter(|&id| !held_by_controller(w, id) && at_home(id))
            .collect();
        let cost = |id: ObjectId| {
            w.obj(id)
                .map_or(0, |o| production::cost_field(w, tech::type_of(o)))
        };
        spare.sort_by_key(|&id| (cost(id), id));
        let surplus = if short_of_power(w, house) {
            spare.len()
        } else {
            spare.len().saturating_sub(KEEP)
        };
        spare.truncate(surplus);
        for id in controlled.into_iter().chain(spare) {
            if sent >= PER_THINK {
                break;
            }
            let Some(at) = w.obj(id).map(|o| o.pos.cell()) else {
                continue;
            };
            let mut near: Vec<(i64, ObjectId)> = rooms
                .iter()
                .filter(|&&(b, r)| r > 0 && self.memory.ok(w, id, Target::Object(b)))
                .map(|&(b, _)| (distance(w, b, at), b))
                .collect();
            near.sort_unstable();
            // design: own AI (#44): the nearest building that takes the
            // click; one that refuses it (switched off, say) does not stop
            // the unit from going to the next.
            let pick = near
                .into_iter()
                .find_map(|(_, b)| Some((b, click_if(w, house, id, b, &[Action::Enter])?)));
            let Some((b, orders)) = pick else { continue };
            out.extend(orders);
            if let Some(r) = rooms.iter_mut().find(|(x, _)| *x == b) {
                r.1 -= 1;
            }
            sent += 1;
            self.offer_use(crate::special::Use::Absorb, out);
        }
    }
}

/// Whether `id` is held by a controller: a mind-controlled enemy whose
/// controller it frees on the way in. A permanently controlled one has no
/// controller to free and belongs to the house for good (S-0601 claim 13),
/// so it counts as the house's own.
// design: own AI (#35)
pub(crate) fn held_by_controller(w: &World, id: ObjectId) -> bool {
    mind::controlled_by(w, id).is_some()
}

fn distance(w: &World, b: ObjectId, at: CellXY) -> i64 {
    w.obj(b).map_or(i64::MAX, |o| dist2(o.pos.cell(), at))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{fixture, game::Game};

    /// [`room`] as before #64: the units on their way read by a pass of
    /// their own.
    fn room_as_before(w: &World, b: ObjectId) -> usize {
        let cap = w
            .obj(b)
            .and_then(|o| w.rules().buildings.get(o.type_index as usize))
            .map_or(0, |r| r.absorb.capacity);
        let taken = absorb::count(w, b) as usize + en_route(w, b);
        usize::try_from(cap).unwrap_or(0).saturating_sub(taken)
    }

    /// Our tooling for speed (design: own AI, #64): the units on their way
    /// counted for every object at once are the units counted object by
    /// object, and the rooms and the grinder's question read from them are
    /// as before, at many points of the M6c, M7a and M7b duels.
    #[test]
    fn the_counts_of_one_pass_are_the_counts_of_a_pass_for_each_object() {
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
        let (mut bound, mut roomy) = (0, 0);
        for (rules, map, map_name, setup) in games {
            let mut g = Game::new(rules, &map, map_name, fixture::catalogue(), setup)
                .expect("the duel loads");
            for _ in 0..120 {
                for _ in 0..75 {
                    g.step();
                }
                let w = &g.world;
                let every: Vec<ObjectId> = w.objects.keys().copied().collect();
                let counts = en_route_each(w, &every);
                for &b in &every {
                    let want = en_route(w, b);
                    assert_eq!(
                        counts.get(&b),
                        Some(&want),
                        "{map_name}, tick {}, {b:?}",
                        w.tick
                    );
                    bound += want;
                }
                for h in 0..w.houses.len() {
                    let house = HouseId(h as u8);
                    let buildings = absorbers(w, house);
                    let want: Vec<(ObjectId, usize)> = buildings
                        .iter()
                        .map(|&b| (b, room_as_before(w, b)))
                        .collect();
                    assert_eq!(rooms(w, &buildings), want, "{map_name}, tick {}", w.tick);
                    let left = want.iter().any(|&(_, r)| r > 0);
                    assert_eq!(room_left(w, house), left, "{map_name}, tick {}", w.tick);
                    roomy += usize::from(left);
                }
            }
        }
        assert!(bound > 0 && roomy > 0, "{bound} bound, {roomy} with room");
    }
}
