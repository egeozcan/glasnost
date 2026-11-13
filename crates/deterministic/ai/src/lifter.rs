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
//!   [`DROP_HEIGHT`] leptons or more over water the vehicle cannot enter
//!   (not over a bridge deck), or over a cell that holds another vehicle or
//!   a building it sees, of a house it is not allied to, it gives
//!   the lifter a move order to its own cell. That ends the grip and the
//!   vehicle falls there (claims 12 and 17). A vehicle pulled all the way
//!   is dropped unhurt on a free cell beside the lifter (claim 8), so a
//!   vehicle within [`LET_GO`] leptons of its lifter is let go the same way
//!   before that, when it hangs over open land with nothing else on the
//!   cell (claims 16 and 17): it lands by the lifter, inside the base's
//!   guns;
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
        // design: own AI (#45): every vehicle a lifter already aims at is
        // taken before any free lifter picks, whatever the lifters' order.
        let mut chosen: Vec<ObjectId> = lifters
            .iter()
            .filter(|&&l| lift::gripped_unit(w, l).is_none())
            .filter_map(|&l| match w.obj(l)?.target {
                Some(Target::Object(t)) if liftable(w, l, t) => Some(t),
                _ => None,
            })
            .collect();
        for &l in &lifters {
            let Some(lo) = w.obj(l) else { continue };
            let at = lo.pos.cell();
            if let Some(x) = lift::gripped_unit(w, l) {
                if drop_here(w, house, x) || (close(w, l, x) && clear_below(w, x)) {
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
            // A lifter already aiming at a vehicle it can seize keeps it (its
            // target is in `chosen` from the start).
            if matches!(lo.target, Some(Target::Object(t)) if liftable(w, l, t)) {
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

/// Whether held unit `x` hangs over open land with nothing else on the
/// cell, where a vehicle let go lands unhurt and harms nothing: the first
/// other object on the cell decides the fall, an infantry unit there is
/// removed, and a blocked or water cell wrecks or sinks the vehicle
/// (S-0607 claims 16 and 17).
// design: own AI (#24): the early let-go by the lifter read only the distance, and dropped the held vehicle onto the house's own infantry or into water by the lifter
fn clear_below(w: &World, x: ObjectId) -> bool {
    let Some(c) = w.obj(x).map(|o| o.pos.cell()) else {
        return false;
    };
    let Some(i) = w.cells.idx(c) else {
        return false;
    };
    // spec: S-0211 Terms (category 0 open)
    w.cells.category[i] == glasnost_sim::passability::category::OPEN
        && w.cells
            .cell(c)
            .is_some_and(|k| k.land != glasnost_map::terrain::land::WATER)
        && w.cells
            .list(c, glasnost_sim::world::Layer::Ground)
            .iter()
            .all(|&o| o == x)
}

/// Whether held unit `x` is high enough over a cell where its fall kills
/// it: water it cannot enter, away from a bridge deck, or a cell that holds
/// a vehicle or building, seen by `house`, of a house that `house` is not
/// allied to (S-0607 claims 16 and 17).
fn drop_here(w: &World, house: HouseId, x: ObjectId) -> bool {
    let Some(xo) = w.obj(x) else { return false };
    if w.height_above_surface(x) < DROP_HEIGHT {
        return false;
    }
    let c = xo.pos.cell();
    if !glasnost_sim::econ::shroud::explored(w, house, c) {
        return false;
    }
    // spec: S-0607 claim 17 (water counts when the land is water and the unit does not land on a bridge deck; it sinks only where the cell is blocked for it, by S-0209 claim 5's zero speed check (5a) for its class)
    // design: own AI (#46): a held vehicle that may enter water (a hovercraft, say), or one over a bridge deck, is not dropped there
    let water = !w.high_bridge(c)
        && w.cells
            .cell(c)
            .is_some_and(|k| k.land == glasnost_map::terrain::land::WATER)
        && w.rules().unit(xo.kind, xo.type_index).is_some_and(|u| {
            !glasnost_sim::passability::enterable_by_class(w.rules(), w, c, u.class, u.zone, false)
        });
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
                // design: own AI (#46): an enemy the house does not see (a cloaked one it does not sense) is not known to be there
                && crate::in_view(w, house, o)
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

#[cfg(test)]
mod drop_tests {
    use super::*;
    use crate::{fixture, game::Game};
    use glasnost_map::CellXY;
    use glasnost_sim::world::Pos;

    const H: HouseId = HouseId(0);

    /// The synthetic skirmish and a cell south of house 0's start, explored
    /// by it.
    fn world() -> (World, CellXY) {
        let g = Game::new(
            fixture::rules(),
            &fixture::map(),
            "synthetic",
            fixture::catalogue(),
            fixture::setup(5),
        )
        .expect("the skirmish loads");
        let mut w = g.world.clone();
        let home = w
            .objects
            .values()
            .find(|o| o.owner == Some(H) && o.on_map)
            .map(|o| o.pos.cell())
            .expect("house 0 on the map");
        let at = CellXY::new(home.x, home.y + 6);
        glasnost_sim::econ::shroud::area_reveal(&mut w, H, Pos::centre(at, 0), 6);
        (w, at)
    }

    /// An enemy vehicle `name` on cell `c`, lifted 400 leptons.
    fn hanging(w: &mut World, name: &str, c: CellXY) -> ObjectId {
        let id = glasnost_sim::testkit::spawn(w, Kind::Vehicle, name, 1, c, 0);
        w.obj_mut(id).expect("the vehicle").pos.z += 400;
        assert!(w.height_above_surface(id) >= DROP_HEIGHT);
        id
    }

    fn water(w: &mut World, c: CellXY) {
        w.cells.grid.get_mut(c).expect("a legal cell").land = glasnost_map::terrain::land::WATER;
    }

    // S-0607 claim 17 (a dropped unit sinks only when the cell is water, it did not land on a bridge deck, and the cell is blocked for it), design: own AI (#46): a held vehicle that may enter water, or one over a bridge deck, is not dropped there; a land vehicle over open water is.
    #[test]
    fn water_kills_only_what_cannot_stand_on_it() {
        let (mut w, at) = world();
        let land = CellXY::new(at.x, at.y);
        let hover = CellXY::new(at.x + 2, at.y);
        let deck = CellXY::new(at.x + 4, at.y);
        let gunr = hanging(&mut w, "GUNR", land);
        let hovr = hanging(&mut w, "HOVR", hover);
        let over_deck = hanging(&mut w, "GUNR", deck);
        for c in [land, hover, deck] {
            water(&mut w, c);
        }
        let i = w.cells.idx(deck).expect("a legal cell");
        w.cells.deck[i] = true;
        assert!(drop_here(&w, H, gunr), "a land vehicle over water sinks");
        assert!(!drop_here(&w, H, hovr), "a hovercraft lands on water");
        assert!(!drop_here(&w, H, over_deck), "a bridge deck holds it");
    }

    // S-0516 Terms and claim 28 (a cloaked object the house does not sense is hidden from it), design: own AI (#46): an enemy vehicle hidden from the house does not make the cell a killing cell.
    #[test]
    fn a_hidden_enemy_does_not_make_a_killing_cell() {
        let (mut w, at) = world();
        let held = hanging(&mut w, "GUNR", at);
        let below = glasnost_sim::testkit::spawn(&mut w, Kind::Vehicle, "GUNR", 1, at, 0);
        assert_eq!(w.obj(below).map(|o| o.pos.cell()), Some(at));
        assert!(drop_here(&w, H, held), "a seen enemy vehicle below");
        w.obj_mut(below).expect("the vehicle below").cloak.state =
            glasnost_sim::cloak::CloakState::Cloaked;
        assert!(glasnost_sim::cloak::hidden_from(&w, below, Some(H)));
        assert!(!drop_here(&w, H, held), "{:?}", w.cells.list(at, glasnost_sim::world::Layer::Ground));
    }
}
