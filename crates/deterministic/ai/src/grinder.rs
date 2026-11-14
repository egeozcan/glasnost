//! The own AI and recycling buildings (S-0603; PLAN §4.7).
//!
//! A recycling building takes in a unit sent to it with the recycle order
//! and pays the building's owner what selling that unit would give its own
//! owner (S-0603 claims 1, 5 and 8). A person who knows this:
//!
//! - **builds one when it has a use for it.** Once the house keeps its
//!   harvesters, has no recycling building (nor one on its line) and holds
//!   a unit it would send (an idle controlled enemy vehicle or a badly
//!   damaged own unit), it orders the cheapest recycling building type it
//!   can build and place now ([`crate::brain::Brain`] calls
//!   [`build_type`]). It queues it behind a building already on the line
//!   and pays as it goes (S-0404), as a person does, rather than waiting
//!   for a free line and money in hand: a house short of money keeps its
//!   building line busy all game;
//! - **grinds mind-controlled enemy units.** A controlled unit is consumed
//!   as an object of the controlling house: the house is paid for it and
//!   the controller's slot is freed for a new victim (claim 9). Every idle
//!   controlled enemy vehicle is sent, whatever else this think told it
//!   (the recycle order comes later and replaces it). Controlled infantry go to an
//!   absorbing building while one has room (`absorber`, which also frees the
//!   slot and gives power), and to the grinder only when none has. A
//!   permanently controlled unit frees no slot and belongs to the house for
//!   good (S-0601 claim 13): it counts as the house's own;
//! - **grinds badly damaged units.** An idle own unit at or
//!   below [`WOUNDED_PERCENT`] of its strength, when the house has no
//!   repair depot to send it to, is turned into money;
//! - **grinds surplus infantry when short of money.** With no absorbing
//!   building that has room for them, idle own infantry at home beyond
//!   [`crate::absorber::KEEP`] are sent, the cheapest first, while the
//!   house's credits are below [`LOW_MONEY`].
//!
//! A unit far from the grinder is first moved to its side with a click on
//! the ground, then sent in from there. A direct recycle click from afar
//! can fail at once when a strip of ground the unit cannot enter or a
//! ledge stands across the route (S-0603 claim 3, S-0212 Edge cases "short
//! strip" and "ledge": the original's behaviour with
//! `quirks.corridor_retry_no_fallback` and
//! `quirks.corridor_free_part_ignores_height` on, their defaults; since
//! S-0212 r82 a short line of buildings no longer makes it fail), and a
//! person would then click a nearer cell first. When the
//! unit does not take that click either (it stands idle on the same cell:
//! the order memory), the next think clicks another side of the grinder,
//! the nearest one not yet failed from there, rather than the same cell
//! again (M8 B4: M7a seed 4's vehicles stood all game beside a grinder). The brain's base defence
//! leaves a unit on its way in alone.
//!
//! It never sends an own harvester, slave miner or slave, engineer or spy
//! (a controlled enemy one standing idle is sent), nor a controller or a
//! unit with passengers, and every order is the
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

// design: own AI (#35): a permanently controlled unit counts as the house's own here too
use crate::absorber::{HOME_REACH, KEEP, held_by_controller};
use crate::scripted::{self, Script, all_types, dist2};
use crate::special::{Special, click_if};

/// An own unit at or below this percentage of its strength counts as badly
/// damaged.
pub const WOUNDED_PERCENT: i64 = 30; // nocite: tuning of our own opponent

/// Credits below which a house grinds its surplus infantry.
pub const LOW_MONEY: i32 = 1000; // nocite: tuning of our own opponent

/// The most units sent in one think.
const PER_THINK: usize = 2; // nocite: tuning of our own opponent

/// A unit farther than this many cells from the grinder is first moved to
/// its side, [`SIDE`] cells off, and sent in from there.
const NEAR: i64 = 12; // nocite: tuning of our own opponent

/// How many cells off the grinder a far unit is first moved to.
const SIDE: i32 = 4; // nocite: tuning of our own opponent

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
/// when it has none (nor one on its line) and holds a unit it would send.
pub(crate) fn build_type(script: &Script, w: &World) -> Option<TypeRef> {
    let house = script.house;
    let own = script.own(w);
    if own
        .iter()
        .any(|&id| w.obj(id).is_some_and(|o| grinder_type(w, tech::type_of(o))))
    {
        return None;
    }
    let depot = own.iter().any(|&id| glasnost_sim::depot::is_depot(w, id));
    let useful = own.iter().any(|&id| {
        free(w, id)
            && if held_by_controller(w, id) {
                w.obj(id).is_some_and(|o| o.kind == Kind::Vehicle)
            } else {
                wounded(w, id, depot)
            }
    });
    if !useful {
        return None;
    }
    let on_line = |t: TypeRef| {
        production::line_of(w, house, t)
            .is_some_and(|l| l.item.as_ref().is_some_and(|i| i.t == t) || l.queue.contains(&t))
    };
    all_types(w)
        .into_iter()
        .filter(|&t| grinder_type(w, t))
        .filter(|&t| script.startable(w, t))
        .filter(|&t| !on_line(t))
        // design: own AI (#8): only while its line is free, as for the other special buildings (S-0411 claim 6: a line with an item takes no new build order)
        .filter(|&t| !script.busy(w, t))
        .filter(|&t| script.place_cell(w, t).is_some())
        .min_by_key(|&t| (production::cost_field(w, t), t.index))
}

/// Whether own unit `id` is badly damaged and goes to a recycling building:
/// at or below [`WOUNDED_PERCENT`] of its strength, and not a vehicle when
/// the house has a repair depot (the brain's repair step sends it there).
fn wounded(w: &World, id: ObjectId, depot: bool) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let strength = i64::from(glasnost_sim::combat::damage::strength(w, id));
    !(depot && o.kind == Kind::Vehicle) && i64::from(o.health) * 100 <= strength * WOUNDED_PERCENT // nocite: a percentage, tuning of our own opponent
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
    // A controlled enemy harvester, engineer or the like does no work for
    // the house while it stands idle, so only the house's own are kept.
    let own_kind_kept = !held_by_controller(w, id)
        && (w
            .rules()
            .unit(o.kind, o.type_index)
            .is_some_and(|u| u.harvester || u.engineer || u.raw.agent.unwrap_or(false))
            || scripted::is_slave_miner(w, t)
            || slaves::is_slave(w, id)
            || scripted::deploys_into_yard(w, o.kind, o.type_index));
    let kept = own_kind_kept || mind::controller(w, id).is_some() || transport::count(w, id) > 0;
    !kept && scripted::idle(w, id) && targeting::selectable(w, id)
}

impl Special {
    /// Recycling buildings: idle controlled enemy vehicles (and infantry
    /// when no absorbing building has room), then badly damaged own units
    /// then surplus own infantry at home while money is short,
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
        // Units this think already gave an order to are left alone, but for
        // an idle controlled unit: the recycle order (later, so it replaces
        // the other) turns it into money and frees its controller's slot.
        let ordered: Vec<ObjectId> = out.iter().filter_map(Order::actor).collect();
        let free: Vec<ObjectId> = own
            .iter()
            .copied()
            .filter(|&id| (!ordered.contains(&id) || held_by_controller(w, id)) && free(w, id))
            .collect();
        let absorber_room = crate::absorber::room_left(w, house);
        let controlled: Vec<ObjectId> = free
            .iter()
            .copied()
            .filter(|&id| held_by_controller(w, id))
            .filter(|&id| w.obj(id).is_some_and(|o| o.kind == Kind::Vehicle) || !absorber_room)
            .collect();
        let mine: Vec<ObjectId> = free
            .iter()
            .copied()
            .filter(|&id| !held_by_controller(w, id))
            .collect();
        let depot = own.iter().any(|&id| glasnost_sim::depot::is_depot(w, id));
        let wounded: Vec<ObjectId> = mine
            .iter()
            .copied()
            .filter(|&id| wounded(w, id, depot))
            .collect();
        let mut surplus: Vec<ObjectId> = Vec::new();
        if !absorber_room && glasnost_sim::seams::available_credits(w, house) < LOW_MONEY {
            let cost = |id: ObjectId| {
                w.obj(id)
                    .map_or(0, |o| production::cost_field(w, tech::type_of(o)))
            };
            // Only at home: infantry gathering elsewhere are a wave forming.
            let home = script.home(w);
            let at_home = |id: ObjectId| {
                home.is_some_and(|h| {
                    w.obj(id)
                        .is_some_and(|o| dist2(o.pos.cell(), h) <= HOME_REACH * HOME_REACH)
                })
            };
            let mut spare: Vec<ObjectId> = mine
                .iter()
                .copied()
                .filter(|&id| {
                    w.obj(id).is_some_and(|o| o.kind == Kind::Infantry)
                        && !wounded.contains(&id)
                        && at_home(id)
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
            if let Some(g) = w.obj(b).map(|o| o.pos.cell())
                && dist2(g, at) > NEAR * NEAR
            {
                // Far off, it is first walked or driven to the grinder's
                // side by a click on the ground, and sent in once there:
                // the side facing it, or, once a side failed from where the
                // unit stands, the next nearest one.
                for side in sides(g, at) {
                    if !self.memory.ok(w, id, Target::Cell(side)) {
                        continue;
                    }
                    let r = targeting::click(
                        w,
                        house,
                        &[id],
                        targeting::ClickOn::Cell(side),
                        targeting::Modifiers::default(),
                    );
                    if !r.orders.is_empty() {
                        // design: own AI (#23): a failure of the order counts for the side clicked, whatever cell the click gave (S-0214 claim 11)
                        self.memory.ask(&r.orders, Target::Cell(side));
                        out.extend(r.orders);
                        sent += 1;
                        break;
                    }
                }
                continue;
            }
            if let Some(orders) = click_if(w, house, id, b, &[Action::Recycle]) {
                out.extend(orders);
                sent += 1;
                self.offer_use(crate::special::Use::Grind, out);
            }
        }
    }
}

/// The cells [`SIDE`] cells off grinder cell `g` a far unit at `at` may be
/// moved to first: the one on the line towards the unit, then the eight
/// around the grinder (straight and diagonal), nearest to the unit first.
fn sides(g: CellXY, at: CellXY) -> Vec<CellXY> {
    let (dx, dy) = (at.x - g.x, at.y - g.y);
    let k = dx.abs().max(dy.abs()).max(1);
    let mut out = vec![CellXY::new(g.x + dx * SIDE / k, g.y + dy * SIDE / k)];
    let mut ring: Vec<CellXY> = [-SIDE, 0, SIDE]
        .into_iter()
        .flat_map(|x| [-SIDE, 0, SIDE].into_iter().map(move |y| (x, y)))
        .filter(|&(x, y)| (x, y) != (0, 0))
        .map(|(x, y)| CellXY::new(g.x + x, g.y + y))
        .collect();
    ring.sort_by_key(|&c| (dist2(c, at), c.y, c.x));
    for c in ring {
        if !out.contains(&c) {
            out.push(c);
        }
    }
    out
}

fn distance(w: &World, b: ObjectId, at: CellXY) -> i64 {
    w.obj(b).map_or(i64::MAX, |o| dist2(o.pos.cell(), at))
}
