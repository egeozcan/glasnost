//! The own AI and magnetic lifters (S-0607; PLAN §4.7).
//!
//! A lifter's beam does no damage. It seizes a ground vehicle whose type's
//! size is below the shot's damage and pulls it through the air towards
//! the lifter (S-0607 claims 1 to 8). When the grip ends in the air the
//! vehicle falls. It survives on clear land, sinks over water and is
//! destroyed on a blocked cell (claims 13 to 18). A grip ends when the
//! lifter accepts a move order (claim 12). A person who knows this:
//!
//! - **builds a few lifters, since every side fields vehicles.** It keeps
//!   up to [`WANTED`] lifters, the cheapest type it can build. A lifter is
//!   any vehicle type whose primary weapon's warhead is a locomotor warhead
//!   ([`lifter_type`]). The first is queued behind whatever the war factory
//!   builds and paid as the money comes; a further one waits for a free
//!   line and [`RESERVE`] times the price in hand ([`build_type`]);
//! - **keeps them at home, by its own defences.** A lifter is not sent with
//!   the waves. It goes back towards the construction yard when it is more
//!   than [`HOME_REACH`] cells away with nothing to lift, so its pulls drag
//!   enemy vehicles into the base's guns;
//! - **lifts enemy vehicles, the most dangerous first.** An idle lifter, or
//!   one aiming at something it cannot lift, is sent by the attack click
//!   at a seen enemy vehicle it can seize within [`HUNT_REACH`] cells of
//!   home or [`LIFT_REACH`] cells of itself (a lifter that went along with
//!   a wave lifts what it meets there). That vehicle stands on the ground,
//!   is not already held, has a type size below the beam's damage and is
//!   not refused by the firing check. The dearest type goes first, then the
//!   nearest;
//! - **drops them where the fall kills.** While a lifter holds a vehicle at
//!   [`DROP_HEIGHT`] leptons or more over water, or over a cell that holds
//!   another vehicle or a building of a house it is not allied to, it gives
//!   the lifter a move order to its own cell. That ends the grip and the
//!   vehicle falls there (claims 12 and 17). A vehicle pulled all the way
//!   is dropped unhurt on a free cell beside the lifter (claim 8), so a
//!   vehicle within [`LET_GO`] leptons of its lifter is let go the same way
//!   before that: it lands by the lifter, inside the base's guns;
//! - **leaves a holding lifter alone.** No other part of the brain gives a
//!   lifter an order while it grips a unit, because any accepted
//!   destination would let the vehicle go over clear ground
//!   ([`holding`]).
//!
//! Every order is one a person gives with the mouse. It reads only the
//! house's own objects, the enemies it has seen, the cells it has explored
//! and the rules a person may read. It draws no random value. The numbers
//! are tuning of our own opponent, not game behaviour.

use glasnost_sim::combat::check::{self, Answer};
use glasnost_sim::econ::{TypeRef, production, tech};
use glasnost_sim::lift;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::Action;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

use crate::scripted::{self, Script, all_types, dist2};
use crate::special::{Special, click_if};

/// How many lifters a house wants.
pub const WANTED: usize = 2; // nocite: tuning of our own opponent

/// The credits kept in hand, as a multiple of the lifter's price, before
/// one is ordered.
const RESERVE: i32 = 2; // nocite: tuning of our own opponent

/// Cells from home within which a lifter goes for an enemy vehicle.
pub const HUNT_REACH: i64 = 22; // nocite: tuning of our own opponent

/// Cells from the lifter within which it goes for an enemy vehicle
/// wherever it is.
pub const LIFT_REACH: i64 = 8; // nocite: tuning of our own opponent

/// Cells from home beyond which an idle lifter goes back.
pub const HOME_REACH: i64 = 8; // nocite: tuning of our own opponent

/// The least height above the surface, in leptons, at which a held
/// vehicle is let go over a killing cell.
pub const DROP_HEIGHT: i32 = 200; // nocite: tuning of our own opponent

/// The horizontal distance in leptons from its lifter within which a held
/// vehicle is let go wherever it is.
pub const LET_GO: i64 = 512; // nocite: tuning of our own opponent

/// The seizing weapon of unit type `t`: its primary weapon, when that
/// weapon's warhead is a locomotor warhead.
fn seizing_weapon(w: &World, t: TypeRef) -> Option<glasnost_sim::combat::data::WeaponId> {
    if t.kind != Kind::Vehicle {
        return None;
    }
    let r = w.rules();
    let u = r.unit(t.kind, t.index)?;
    let wp = u.combat.slots[0]?;
    let h = r.combat.weapon(wp).warhead?;
    r.combat.warhead(h).is_locomotor.then_some(wp)
}

/// Whether vehicle type `t` is a lifter: its primary weapon's warhead is a
/// locomotor warhead.
pub fn lifter_type(w: &World, t: TypeRef) -> bool {
    seizing_weapon(w, t).is_some()
}

/// Whether object `id` is a lifter.
pub fn is_lifter(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| lifter_type(w, tech::type_of(o)))
}

/// Whether lifter `id` holds a unit in its grip: other parts of the brain
/// give it no order then.
pub fn holding(w: &World, id: ObjectId) -> bool {
    lift::gripped_unit(w, id).is_some()
}

/// How many lifters the house has or has queued.
fn count(w: &World, house: HouseId) -> usize {
    let on_map = w
        .objects
        .values()
        .filter(|o| o.owner == Some(house) && o.health > 0 && lifter_type(w, tech::type_of(o)))
        .count();
    let queued: usize = all_types(w)
        .into_iter()
        .filter(|&t| lifter_type(w, t))
        .map(|t| {
            production::line_of(w, house, t).map_or(0, |l| {
                usize::from(l.item.as_ref().is_some_and(|i| i.t == t))
                    + l.queue.iter().filter(|&&q| q == t).count()
            })
        })
        .sum();
    on_map + queued
}

/// The cheapest lifter type the house can build now, while it has fewer
/// than [`WANTED`] and can pay [`RESERVE`] times the price.
pub(crate) fn build_type(script: &Script, w: &World) -> Option<TypeRef> {
    let house = script.house;
    let have = count(w, house);
    if have >= WANTED {
        return None;
    }
    let credits = glasnost_sim::seams::available_credits(w, house);
    // The first lifter is queued behind whatever the war factory builds,
    // paid as the money comes; a further one waits for a free line and the
    // reserve.
    let first = have == 0;
    all_types(w)
        .into_iter()
        .filter(|&t| lifter_type(w, t))
        .filter(|&t| crate::startable(w, house, t))
        .filter(|&t| first || !script.busy(w, t))
        .filter(|&t| first || credits >= production::cost_field(w, t).saturating_mul(RESERVE))
        .min_by_key(|&t| (production::cost_field(w, t), t.index))
}

/// Whether lifter `l` can seize enemy `e`: a vehicle on the ground that
/// nobody holds, of a type whose size is below the beam's damage, and the
/// firing check does not refuse it.
fn liftable(w: &World, l: ObjectId, e: ObjectId) -> bool {
    let (Some(lo), Some(eo)) = (w.obj(l), w.obj(e)) else {
        return false;
    };
    if eo.kind != Kind::Vehicle || !eo.on_map || eo.health <= 0 || lift::seized(w, e) {
        return false;
    }
    if w.height_above_surface(e) != 0 {
        return false;
    }
    let Some(wp) = seizing_weapon(w, tech::type_of(lo)) else {
        return false;
    };
    let damage = w.rules().combat.weapon(wp).damage;
    // spec: S-0301 claim 9 (the own AI's lasting test, our design, PLAN §4.7: the check without the reasons of the moment of ranks 5 to 8, so that reloading does not hide an illegal target)
    lift::eligible(w, e, damage)
        && check::check_lasting(w, l, Some(Target::Object(e)), 0) != Answer::Illegal
}

impl Special {
    /// Magnetic lifters: drops over killing cells first, then lift orders
    /// at enemy vehicles, then idle lifters back home. Returns the drop
    /// orders apart: the brain issues them after its filter of orders to
    /// holding lifters.
    pub(crate) fn lifters(
        &mut self,
        script: &Script,
        own: &[ObjectId],
        w: &World,
        out: &mut Vec<Order>,
    ) {
        let house = script.house;
        let lifters: Vec<ObjectId> = own
            .iter()
            .copied()
            .filter(|&id| is_lifter(w, id) && glasnost_sim::targeting::selectable(w, id))
            .collect();
        if lifters.is_empty() {
            return;
        }
        let home = script.home(w);
        let enemies = script.seen_enemies(w);
        let mut chosen: Vec<ObjectId> = Vec::new();
        for &l in &lifters {
            let Some(lo) = w.obj(l) else { continue };
            let at = lo.pos.cell();
            if let Some(x) = lift::gripped_unit(w, l) {
                if drop_here(w, house, x) || close(w, l, x) {
                    out.push(Order::Behaviour {
                        actor: l,
                        behaviour: Behaviour::Move,
                        attack_move: false,
                        target: None,
                        dest: Some(Target::Cell(at)),
                    });
                    self.count_lift_drop();
                }
                continue;
            }
            // A lifter already aiming at a vehicle it can seize keeps it.
            let aiming = matches!(lo.target, Some(Target::Object(t)) if liftable(w, l, t));
            if aiming {
                if let Some(Target::Object(t)) = lo.target {
                    chosen.push(t);
                }
                continue;
            }
            let near_home = |c| {
                home.is_none_or(|h| dist2(c, h) <= HUNT_REACH * HUNT_REACH)
                    || dist2(c, at) <= LIFT_REACH * LIFT_REACH
            };
            let pick = enemies
                .iter()
                .filter(|&&(e, c)| !chosen.contains(&e) && near_home(c))
                .filter(|&&(e, _)| liftable(w, l, e))
                .filter(|&&(e, _)| self.memory.ok(w, l, Target::Object(e)))
                .map(|&(e, c)| {
                    let price = w
                        .obj(e)
                        .map_or(0, |o| production::cost_field(w, tech::type_of(o)));
                    (std::cmp::Reverse(price), dist2(c, at), e)
                })
                .min()
                .map(|(_, _, e)| e);
            if let Some(e) = pick
                && let Some(orders) = click_if(w, house, l, e, &[Action::Attack])
            {
                out.extend(orders);
                chosen.push(e);
                self.count_lift_order();
                continue;
            }
            // Nothing to lift: back towards home, by its own defences.
            if let Some(h) = home
                && scripted::idle(w, l)
                && dist2(at, h) > HOME_REACH * HOME_REACH
            {
                out.push(Order::Behaviour {
                    actor: l,
                    behaviour: Behaviour::Move,
                    attack_move: false,
                    target: None,
                    dest: Some(Target::Cell(h)),
                });
            }
        }
    }
}

/// Whether held unit `x` is within [`LET_GO`] leptons of lifter `l`,
/// horizontally.
fn close(w: &World, l: ObjectId, x: ObjectId) -> bool {
    let (Some(lo), Some(xo)) = (w.obj(l), w.obj(x)) else {
        return false;
    };
    let (dx, dy) = (
        i64::from(xo.pos.x - lo.pos.x),
        i64::from(xo.pos.y - lo.pos.y),
    );
    dx * dx + dy * dy <= LET_GO * LET_GO
}

/// Whether held unit `x` is high enough over a cell where its fall kills
/// it: water, or a cell that holds a vehicle or building of a house that
/// `house` is not allied to (S-0607 claims 16 and 17).
fn drop_here(w: &World, house: HouseId, x: ObjectId) -> bool {
    let Some(xo) = w.obj(x) else { return false };
    if w.height_above_surface(x) < DROP_HEIGHT {
        return false;
    }
    let c = xo.pos.cell();
    if !glasnost_sim::econ::shroud::explored(w, house, c) {
        return false;
    }
    let water = w
        .cells
        .cell(c)
        .is_some_and(|k| k.land == glasnost_map::terrain::land::WATER);
    let occupants = w.cells.list(c, glasnost_sim::world::Layer::Ground);
    let own_there = occupants
        .iter()
        .any(|&o| o != x && w.obj(o).is_some_and(|p| w.allied(Some(house), p.owner)));
    if own_there {
        return false;
    }
    water
        || occupants.iter().any(|&o| {
            o != x
                && w.obj(o).is_some_and(|p| {
                    matches!(p.kind, Kind::Vehicle | Kind::Building)
                        && !w.allied(Some(house), p.owner)
                })
        })
}

/// Whether the brain issues `order`. A lifter takes only its own part's
/// orders: an attack on an enemy vehicle it can seize, and a plain move
/// (back home, or the drop). A holding lifter takes only the drop: a move
/// to its own cell. Wave and defence orders to lifters are left out.
pub(crate) fn keeps_order(w: &World, order: &Order) -> bool {
    let Some(actor) = order.actor() else {
        return true;
    };
    if !is_lifter(w, actor) {
        return true;
    }
    let Some(at) = w.obj(actor).map(|o| o.pos.cell()) else {
        return true;
    };
    match order {
        Order::Behaviour {
            behaviour: Behaviour::Move,
            attack_move: false,
            dest: Some(Target::Cell(c)),
            ..
        } => !holding(w, actor) || *c == at,
        Order::Behaviour {
            behaviour: Behaviour::Attack,
            target: Some(Target::Object(e)),
            ..
        } => !holding(w, actor) && liftable(w, actor, *e),
        // design: own AI (#13): a wave's attack-move is not the lifter's part
        Order::Behaviour {
            attack_move: true, ..
        } => false,
        _ => !holding(w, actor),
    }
}
