//! Route mode (S-0213 claims 16 to 17, with S-0215 claim 13's route row).
//!
//! A house's route points are places on the map ([`RoutePoint`]); each
//! route holder's planned route is a list of steps ([`Step`]), each a point
//! with the action the click gave that holder, optionally ending in a loop.
//! The route-mode orders ([`Order::RoutePoint`], [`Order::RouteJoin`],
//! [`Order::RouteCommit`] and [`Order::RouteDelete`]) build, commit and
//! delete them at the order boundary; every other order naming an actor
//! ends its route ([`order_ends`]), except the orders a route issues for its
//! points ([`Order::Routed`]). The route row of the fall-back to rest issues
//! the next committed point ([`fall_back`]).
//!
//! The click side ([`click`]) makes the refusals S-0213 makes at the click:
//! the kinds that cannot take part, the same-route-state rule, the
//! route-holder limit and the point-click test; the refusals made when a
//! point add is applied are in [`apply_point`] and announced through
//! [`Effect::Route`]. State is world-level ([`RouteWorld`]), saved and
//! hashed. The look of points and lines is the client's.

use std::collections::BTreeMap;

use glasnost_map::CellXY;
use serde::{Deserialize, Serialize};

use crate::combat::effects::Effect;
use crate::orders::{self, Order};
use crate::rules::{Behaviour, Kind};
use crate::targeting::{self, Action, ClickOn, Modifiers};
use crate::world::{HouseId, ObjectId, Target, World};

/// The route-holder limit per house: count plus new objects (S-0213
/// Constants).
pub const HOLDER_LIMIT: u32 = 128; // spec: S-0213 Constants (route-holder limit per house: 128, count plus new objects)

/// The most points a point add takes a route to (S-0213 claim 16b).
pub const MAX_POINTS: usize = 16; // spec: S-0213 claim 16b (a point add is refused when the route already holds 16 points)

/// The smallest loop input can make (S-0213 claim 16c), and the size below
/// which a deletion cancels a loop (claim 16e).
pub const MIN_LOOP: usize = 3; // spec: S-0213 claims 16c and 16e (a loop made by input holds at least 3 points; one left with fewer than 3 is cancelled)

/// A route point: a place on the map that belongs to a house (S-0213
/// claim 16a).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePoint {
    /// The house it belongs to.
    pub house: HouseId,
    /// Its place: the target of the order that made it, or its destination.
    pub place: Target,
    /// The cell of the place when the point was made, for a place object
    /// that is gone.
    pub cell: CellXY,
    /// Committed (claim 16c): only committed points are followed.
    pub committed: bool,
}

/// One point of a holder's route, with the action the click gave that
/// holder (S-0213 claim 16b).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    /// The route point.
    pub point: u32,
    /// The behaviour of the click's order.
    pub behaviour: Behaviour,
    /// The click's order was an attack-move.
    pub attack_move: bool,
    /// Its target.
    pub target: Option<Target>,
    /// Its destination.
    pub dest: Option<Target>,
}

/// An object's route and route record (S-0213 claims 16a to 16c). The
/// record is the entry itself: kept, even once the route is empty, until
/// the object is removed from play.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Route {
    /// The house whose orders built it.
    pub house: HouseId,
    /// The points still to issue, in order.
    pub steps: Vec<Step>,
    /// The loop's first step, when the route has a loop: the steps from it
    /// to the end repeat.
    pub loop_from: Option<usize>,
    /// Within a loop whose earlier points are all issued, the step to issue
    /// next.
    pub next: usize,
}

/// The route state of the world (S-0213 claims 16 to 16f).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteWorld {
    /// Every route point, by number.
    pub points: BTreeMap<u32, RoutePoint>,
    /// The next point number.
    pub next_point: u32,
    /// Every route record, by object.
    pub routes: BTreeMap<ObjectId, Route>,
    /// The route-holder count of each house under
    /// `quirks.route_holder_count_drift` (S-0213 Terms).
    pub counts: BTreeMap<HouseId, u32>,
}

/// What a route-mode click does (S-0213 claim 16).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteClick {
    /// The orders to issue: point adds or point joins.
    Orders(Vec<Order>),
    /// Refused, with the string-table label of the message; no order.
    Refused(&'static str),
    /// With nothing selected, a click on a point selects it (claim 16e).
    SelectPoint(u32),
    /// The ordinary click: the selection changes as outside route mode,
    /// no order is given.
    Ordinary,
    /// A click whose ordinary orders are none of them behaviour orders (a
    /// deploy, a detonate): no order, no point, no message and no selection
    /// change.
    Ignored,
}

/// What a route order did, for the local player's messages and sound
/// (S-0213 claim 16b).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteNote {
    /// A point add was accepted; `info` names the message shown as
    /// information when the added point is terminating, continual or the
    /// 16th.
    Added {
        /// The information message's label, if any.
        info: Option<&'static str>,
    },
    /// A point add was refused, with the message's label.
    Refused(&'static str),
    /// A loop was closed.
    Looped,
}

/// The step's place.
fn place(s: &Step) -> Option<Target> {
    s.target.or(s.dest)
}

/// The cell of a place.
pub fn place_cell(w: &World, t: Target) -> Option<CellXY> {
    match t {
        Target::Cell(c) => Some(c),
        Target::Object(x) => w.obj(x).map(|o| o.pos.cell()),
    }
}

/// The cell a point's marker stands in: its place's, or the cell it was
/// made at when the place object is gone.
pub fn point_cell(w: &World, p: &RoutePoint) -> CellXY {
    place_cell(w, p.place).unwrap_or(p.cell)
}

/// A terminating action (S-0213 claim 16b): *enter*, *capture* or *eaten*.
fn terminating(s: &Step) -> bool {
    // spec: S-0213 claim 16b (terminating: its action is enter, capture or eaten)
    !s.attack_move
        && matches!(
            s.behaviour,
            Behaviour::Enter | Behaviour::Capture | Behaviour::Eaten
        )
}

/// A continual action (S-0213 claim 16b): *guard*, *attack* on a cell, or
/// *area guard* on a cell or with no target.
fn continual(s: &Step) -> bool {
    if s.attack_move {
        return false;
    }
    // spec: S-0213 claim 16b (continual: guard, attack on a cell, or area guard on a cell or with no target)
    match s.behaviour {
        Behaviour::Guard => true,
        Behaviour::Attack => matches!(s.target, Some(Target::Cell(_))),
        Behaviour::AreaGuard => !matches!(s.target, Some(Target::Object(_))),
        _ => false,
    }
}

/// Whether the object's route holds at least one point: a route holder
/// (S-0213 Terms).
pub fn holds(w: &World, id: ObjectId) -> bool {
    w.routes
        .routes
        .get(&id)
        .is_some_and(|r| !r.steps.is_empty())
}

/// Whether the object has a route record (S-0213 claim 16b): from its
/// first route point until it is removed from play. Under
/// `quirks.spent_route_blocks_join` off, an empty route counts as none.
pub fn has_record(w: &World, id: ObjectId) -> bool {
    // spec: S-0213 claim 16b and Edge cases (quirks.spent_route_blocks_join: on, the record is kept once the route is empty; off, an empty route counts as no route record)
    w.routes
        .routes
        .get(&id)
        .is_some_and(|r| w.rules().quirks.spent_route_blocks_join || !r.steps.is_empty())
}

/// The house's route-holder count (S-0213 Terms). Under
/// `quirks.route_holder_count_drift` off it is the number of its route
/// holders.
pub fn count(w: &World, house: HouseId) -> u32 {
    // spec: S-0213 claim 16f and Edge cases (quirks.route_holder_count_drift: on, the count kept by its own rules; off, always the number of the house's route holders)
    if w.rules().quirks.route_holder_count_drift {
        w.routes.counts.get(&house).copied().unwrap_or(0)
    } else {
        let n = w
            .routes
            .routes
            .values()
            .filter(|r| r.house == house && !r.steps.is_empty())
            .count();
        u32::try_from(n).unwrap_or(u32::MAX)
    }
}

fn lower(w: &mut World, house: HouseId) {
    // spec: S-0213 Terms (lowered by 1, never below 0)
    let c = w.routes.counts.entry(house).or_insert(0);
    *c = c.saturating_sub(1);
}

/// The house's point whose marker stands in `cell`, if any; the lowest
/// numbered one when several do.
pub fn point_in(w: &World, house: HouseId, cell: CellXY) -> Option<u32> {
    // design: a click hits a point's marker when it is in the cell the marker stands in (the markers' look is not studied, S-0213 claim 16g)
    w.routes
        .points
        .iter()
        .find(|(_, p)| p.house == house && point_cell(w, p) == cell)
        .map(|(&n, _)| n)
}

/// Deletes every point that no route holds any more.
fn drop_orphans(w: &mut World) {
    let held: std::collections::BTreeSet<u32> = w
        .routes
        .routes
        .values()
        .flat_map(|r| r.steps.iter().map(|s| s.point))
        .collect();
    // spec: S-0213 claim 16f (a point that no route holds any more is deleted); design (Q-2136): after any end of holding, not only destruction
    w.routes.points.retain(|n, _| held.contains(n));
}

/// Ends the object's route (S-0213 claims 16f and 17): a holder stops
/// holding and lowers its house's count at once. The record stays.
fn end(w: &mut World, id: ObjectId) {
    let Some(r) = w.routes.routes.get_mut(&id) else {
        return;
    };
    let house = r.house;
    let held = !r.steps.is_empty();
    r.steps.clear();
    r.loop_from = None;
    r.next = 0;
    if held {
        // spec: S-0213 claim 16f (an object whose route still holds a point stops being a route holder and lowers the count by 1 at once)
        lower(w, house);
        drop_orphans(w);
    }
}

/// Whether an order is one of route mode's own, which do not end a route
/// (S-0213 claim 17), or one a route issued for its point.
pub fn route_order(o: &Order) -> bool {
    matches!(
        o,
        Order::RoutePoint { .. }
            | Order::RouteJoin { .. }
            | Order::RouteCommit
            | Order::RouteDelete { .. }
            | Order::Routed(_)
    )
}

/// An order naming `actor` is applied (S-0213 claim 17): unless it is a
/// route order, it removes the actor's planned route, for any actor that
/// still exists, whether or not the order then passes its checks.
pub fn order_ends(w: &mut World, o: &Order) {
    if route_order(o) {
        return;
    }
    if let Some(a) = o.actor()
        && w.obj(a).is_some()
    {
        // spec: S-0213 claim 17 (any order that names an actor, apart from the route-mode orders and those a route issues, removes that actor's planned route when it is applied)
        end(w, a);
    }
}

/// Objects removed from play leave their routes (S-0213 claim 16f): a
/// holder lowers its house's count, it leaves every point it held, and
/// points no route holds are deleted; its record ends. Made at the order
/// boundary for every object gone since the last.
pub fn prune(w: &mut World) {
    let gone: Vec<ObjectId> = w
        .routes
        .routes
        .keys()
        .copied()
        .filter(|&id| w.obj(id).is_none())
        .collect();
    for id in gone {
        // spec: S-0213 claim 16f (destroyed and removed from play: it stops holding, lowering the count, and leaves every point it held)
        end(w, id);
        w.routes.routes.remove(&id);
    }
    drop_orphans(w);
}

fn note(w: &mut World, house: HouseId, note: RouteNote) {
    w.effects.push(Effect::Route { house, note });
}

/// Applies a point add (S-0213 claims 16a and 16b): the refusals are made
/// now, and an accepted point is added at the order's place to the end of
/// the actor's route.
pub fn apply_point(w: &mut World, house: HouseId, actor: ObjectId, step: Step) {
    let Some(o) = w.obj(actor) else { return };
    // spec: S-0213 claim 16d (only vehicles and infantry can hold a route)
    if !matches!(o.kind, Kind::Vehicle | Kind::Infantry) {
        return;
    }
    let Some(at) = place(&step) else { return };
    let cell = place_cell(w, at).unwrap_or(CellXY::new(0, 0));
    if let Some(r) = w.routes.routes.get(&actor) {
        // spec: S-0213 claim 16b (a point add is refused, when applied, after a terminating or continual last point, in a loop, or at 16 points; each with its own message)
        let refusal = if r.loop_from.is_some() {
            Some("MSG:PostLoop")
        } else if r.steps.len() >= MAX_POINTS {
            Some("MSG:NodeMaximum")
        } else if r.steps.last().is_some_and(terminating) {
            Some("MSG:PostTerminatingCommand")
        } else if r.steps.last().is_some_and(continual) {
            Some("MSG:PostContinualCommand")
        } else {
            None
        };
        if let Some(m) = refusal {
            note(w, house, RouteNote::Refused(m));
            return;
        }
    }
    // spec: S-0213 claim 16a (a later point add at a point's place reuses it instead of making a second point there)
    let existing = w
        .routes
        .points
        .iter()
        .find(|(_, p)| p.house == house && p.place == at)
        .map(|(&n, _)| n);
    let point = match existing {
        Some(n) => n,
        None => {
            let n = w.routes.next_point;
            w.routes.next_point = n.wrapping_add(1);
            w.routes.points.insert(
                n,
                RoutePoint {
                    house,
                    place: at,
                    cell,
                    // spec: S-0213 claim 16c (points added since the last commit are not followed)
                    committed: false,
                },
            );
            n
        }
    };
    let r = w.routes.routes.entry(actor).or_insert(Route {
        house,
        steps: Vec::new(),
        loop_from: None,
        next: 0,
    });
    let first = r.steps.is_empty();
    r.house = house;
    let info = if terminating(&step) {
        Some("MSG:PostTerminatingCommand")
    } else if continual(&step) {
        Some("MSG:PostContinualCommand")
    } else if r.steps.len() + 1 == MAX_POINTS {
        Some("MSG:NodeMaximum")
    } else {
        None
    };
    r.steps.push(Step { point, ..step });
    if first {
        // spec: S-0213 Terms (applying a point add to an object whose route holds no point raises the count by 1)
        *w.routes.counts.entry(house).or_insert(0) += 1;
    }
    // spec: S-0213 claim 16b (an accepted point plays the route-point sound; an added terminating, continual or 16th point shows that case's message as information)
    note(w, house, RouteNote::Added { info });
}

/// Applies a point join (S-0213 claims 16a to 16c): it closes a loop when
/// the point is in the actor's route, and otherwise gives the actor, whose
/// route is empty, a copy of a holder's route from the point to its end.
pub fn apply_join(w: &mut World, house: HouseId, actor: ObjectId, point: u32) {
    if w.obj(actor).is_none() || !w.routes.points.contains_key(&point) {
        return;
    }
    if let Some(r) = w.routes.routes.get_mut(&actor)
        && let Some(i) = r.steps.iter().position(|s| s.point == point)
    {
        // spec: S-0213 claims 16b and 16c (a click on a point already in the route closes a loop from it to the route's end; refused for a route with a loop and for its last two points)
        if r.loop_from.is_none() && i + MIN_LOOP <= r.steps.len() {
            r.loop_from = Some(i);
            r.next = 0;
            // spec: S-0213 claim 16c (closing a loop shows an information message)
            note(w, house, RouteNote::Looped);
        }
        return;
    }
    if holds(w, actor) {
        return;
    }
    // design (Q-2137): the copied route is that of the point's holder of the house with the lowest object number, other than the actor
    let Some((_, src)) = w.routes.routes.iter().find(|(id, r)| {
        **id != actor && r.house == house && r.steps.iter().any(|s| s.point == point)
    }) else {
        return;
    };
    let i = src.steps.iter().position(|s| s.point == point).unwrap_or(0);
    let steps: Vec<Step> = src.steps[i..].to_vec();
    // spec: S-0213 claim 16b (the copy keeps that route's loop if it has one); design (Q-2137): a loop starting before the joined point loops the whole copy
    let loop_from = src.loop_from.map(|l| l.saturating_sub(i));
    let copy = Route {
        house,
        steps,
        loop_from,
        next: 0,
    };
    w.routes.routes.insert(actor, copy);
    // spec: S-0213 claim 16f (a point join that gives an object its first points does not raise the count, under quirks.route_holder_count_drift)
}

/// Applies a commit (S-0213 claim 16c): every point of the house becomes
/// committed, and a holder that had no committed point left to follow
/// starts at once if its current behaviour is *guard* or continual.
pub fn apply_commit(w: &mut World, house: HouseId) {
    let waiting: Vec<ObjectId> = w
        .routes
        .routes
        .iter()
        .filter(|(_, r)| r.house == house)
        .filter(|(_, r)| {
            next_step(r)
                .and_then(|s| w.routes.points.get(&s.point))
                .is_some_and(|p| !p.committed)
        })
        .map(|(&id, _)| id)
        .collect();
    // spec: S-0213 claim 16c (when a commit is applied, every point of the house's routes becomes committed)
    for p in w.routes.points.values_mut() {
        if p.house == house {
            p.committed = true;
        }
    }
    for id in waiting {
        let Some(o) = w.obj(id) else { continue };
        let current = Step {
            point: 0,
            behaviour: o.beh.current.unwrap_or(Behaviour::Sleep),
            attack_move: false,
            target: o.target,
            dest: o.dest,
        };
        // spec: S-0213 claim 16c (a holder whose route could not run before the commit starts at once if its current behaviour is guard or continual: it gives up that behaviour and issues its next point, which takes effect at the next order boundary; any other holder is not interrupted)
        if o.beh.current.is_some() && continual(&current) {
            issue_next(w, id);
        }
    }
}

/// Applies a point delete (S-0213 claim 16e): the point leaves every route
/// that holds it, and a loop left with fewer than 3 points is cancelled.
pub fn apply_delete(w: &mut World, house: HouseId, point: u32) {
    if w.routes.points.get(&point).is_none_or(|p| p.house != house) {
        return;
    }
    for r in w.routes.routes.values_mut() {
        let before = r.steps.len();
        let mut k = 0;
        let mut kept = Vec::with_capacity(before);
        let mut loop_from = r.loop_from;
        let mut next = r.next;
        let _ = before;
        for (i, s) in r.steps.drain(..).enumerate() {
            if s.point == point {
                if let Some(l) = loop_from
                    && i < l
                {
                    loop_from = Some(l - 1);
                }
                if next > k {
                    next -= 1;
                }
                continue;
            }
            kept.push(s);
            k += 1;
        }
        r.steps = kept;
        if let Some(l) = loop_from
            && r.steps.len().saturating_sub(l) < MIN_LOOP
        {
            // spec: S-0213 claim 16e (a loop left with fewer than 3 points is cancelled; the remaining points form an ordinary route)
            loop_from = None;
        }
        r.loop_from = loop_from;
        r.next = if r.loop_from.is_some() && next < r.steps.len() {
            next
        } else {
            0
        };
    }
    // spec: S-0213 claim 16f (deleting a route's last remaining point ends holding but does not lower the count, under quirks.route_holder_count_drift; off, the count is the holders')
    w.routes.points.remove(&point);
}

/// The step a route issues next, if any.
fn next_step(r: &Route) -> Option<&Step> {
    match r.loop_from {
        Some(0) => r.steps.get(r.next),
        _ => r.steps.first(),
    }
}

/// Whether the object follows a route whose next point is committed
/// (S-0215 claim 13's route row).
pub fn follows(w: &World, id: ObjectId) -> bool {
    w.routes
        .routes
        .get(&id)
        .and_then(next_step)
        .and_then(|s| w.routes.points.get(&s.point))
        .is_some_and(|p| p.committed)
}

/// The route row of the fall-back to rest (S-0215 claim 13): a holder
/// whose next point is committed issues its action as an order.
pub fn fall_back(w: &mut World, id: ObjectId) {
    // spec: S-0215 claim 13 (it follows a route and is not in a patrol that has already begun: it issues the action of the next route point as an order; not an immediate destination assignment) and S-0213 claim 16c (uncommitted points are not followed: the holder waits for a commit)
    if follows(w, id) {
        issue_next(w, id);
    }
}

/// Issues the next point of the object's route (S-0213 claim 16c): a point
/// outside a loop is taken off the front, so a route without a loop is used
/// up when its last point is issued; a loop's points are issued in turn.
fn issue_next(w: &mut World, id: ObjectId) {
    let Some(owner) = w.obj(id).and_then(|o| o.owner) else {
        return;
    };
    let Some(r) = w.routes.routes.get_mut(&id) else {
        return;
    };
    let house = r.house;
    let step = match r.loop_from {
        Some(0) => {
            let Some(s) = r.steps.get(r.next).cloned() else {
                return;
            };
            // spec: S-0213 claim 16c (the loop's points are issued in turn, the first loop point again after the last)
            r.next = (r.next + 1) % r.steps.len();
            s
        }
        l => {
            if r.steps.is_empty() {
                return;
            }
            // spec: S-0213 claim 16c (issuing a point takes it off the front of the route; points before the loop are issued once)
            r.loop_from = l.map(|l| l - 1);
            r.steps.remove(0)
        }
    };
    let used_up = r.steps.is_empty();
    if used_up {
        // spec: S-0213 claim 16f (a route without a loop is used up when its last point is issued: the holder stops holding and lowers the count then)
        lower(w, house);
    }
    let order = point_order(w, owner, id, &step);
    if used_up {
        drop_orphans(w);
    }
    if let Some(o) = order {
        // spec: S-0215 claim 13 (the route point issues an order through the ordinary order timing of S-0213 claim 1) and S-0213 claim 17 (orders a route issues for its points do not end it)
        orders::issue(w, owner, Order::Routed(Box::new(o)));
    }
}

/// The order a holder issues for a route point (S-0215 claim 13's
/// route-point selection rule): an object point is classified anew as an
/// ordinary object click with the actor owner's house acting, under the
/// service and alliance guards, else the cell action; a cell point issues
/// the action its click recorded.
fn point_order(w: &World, owner: HouseId, id: ObjectId, step: &Step) -> Option<Order> {
    let cell = w
        .routes
        .points
        .get(&step.point)
        .map(|p| point_cell(w, p))
        .or_else(|| place(step).and_then(|t| place_cell(w, t)));
    let cell_action = |c: CellXY| {
        // spec: S-0215 claim 13 (the cell-action fallback: the actor's action on the point's cell)
        targeting::click(w, owner, &[id], ClickOn::Cell(c), Modifiers::default())
            .orders
            .into_iter()
            .next()
    };
    let Some(Target::Object(t)) = step.target.filter(|_| !step.attack_move) else {
        // spec: S-0215 claim 13 (a point with no object uses the cell action): the cell action the click gave the holder
        return Some(Order::Behaviour {
            actor: id,
            behaviour: step.behaviour,
            attack_move: step.attack_move,
            target: step.target,
            dest: step.dest,
        });
    };
    let Some(to) = w.obj(t) else {
        return cell.and_then(cell_action);
    };
    let on = ClickOn::Object(t);
    let action = targeting::cursor(w, owner, &[id], on, Modifiers::default());
    let allied = w.allied(Some(owner), to.owner);
    let object_ok = if allied {
        // spec: S-0215 claim 13 (for an object the owner counts as allied: only a service building, a unit repair building for a damaged non-infantry actor or a unit reload building for an actor short of ammunition, whose owner counts the actor's owner allied too, and not select or toggle select)
        let service = targeting::service_building_for(w, t, id);
        service
            && w.allied(to.owner, Some(owner))
            && !matches!(action, Action::Select | Action::ToggleSelect)
    } else {
        // spec: S-0215 claim 13 (for an object the owner does not count as allied, select, toggle select, move and no-move use the cell fallback)
        !matches!(
            action,
            Action::Select | Action::ToggleSelect | Action::Move | Action::NoMove
        )
    };
    if object_ok {
        let orders = targeting::click(w, owner, &[id], on, Modifiers::default()).orders;
        if let Some(o) = orders.into_iter().next() {
            return Some(o);
        }
    }
    cell.and_then(cell_action)
}

/// A route-mode click by `house` with `selection` (S-0213 claims 16 to
/// 16e): the refusals made at the click, a point selection, or the point
/// adds or joins to issue.
pub fn click(
    w: &World,
    house: HouseId,
    selection: &[ObjectId],
    on: ClickOn,
    cell: CellXY,
    mods: Modifiers,
) -> RouteClick {
    let marker = point_in(w, house, cell);
    if selection.is_empty() {
        // spec: S-0213 claim 16e (with no object selected, a click on an existing route point selects it and shows an information message)
        return marker.map_or(RouteClick::Ordinary, RouteClick::SelectPoint);
    }
    // The ordinary selecting clicks stay what they are in route mode.
    let plain = targeting::click(w, house, selection, on, mods);
    if marker.is_none() && plain.orders.is_empty() {
        return RouteClick::Ordinary;
    }
    // spec: S-0213 claim 16d (a selected aircraft, building or object of any other kind refuses the click, each with its message)
    for &id in selection {
        match w.obj(id).map(|o| o.kind) {
            Some(Kind::Vehicle | Kind::Infantry) => {}
            Some(Kind::Aircraft) => return RouteClick::Refused("MSG:PlanningModeNoAircraft"),
            Some(Kind::Building) => return RouteClick::Refused("MSG:PlanningModeNoBuildings"),
            _ => return RouteClick::Refused("MSG:PlanningModeNoUnknowns"),
        }
    }
    // spec: S-0213 claim 16d (the selected objects share one route state: none holds a route, or all hold routes that end at the same point)
    let ends: Vec<Option<u32>> = selection
        .iter()
        .map(|&id| {
            w.routes
                .routes
                .get(&id)
                .and_then(|r| r.steps.last())
                .map(|s| s.point)
        })
        .collect();
    let same =
        ends.iter().all(|e| e.is_none()) || ends.iter().all(|e| e.is_some() && *e == ends[0]);
    // spec: S-0213 claim 16 (refused when the count plus the selected objects whose routes hold no point would be more than 128)
    let new = selection.iter().filter(|&&id| !holds(w, id)).count();
    let over = u64::from(count(w, house)) + new as u64 > u64::from(HOLDER_LIMIT);
    // spec: S-0213 claim 16d (when the selection both mixes route states and would pass the holder limit, only the incompatible-selection message is given)
    if !same {
        return RouteClick::Refused("MSG:PlanningModeHeteroSel");
    }
    if over {
        return RouteClick::Refused("MSG:PlannerMaximum");
    }
    if let Some(p) = marker {
        // spec: S-0213 claim 16b (clicks on existing points: refused with the invalid-point message when any selected object with a route record has a loop, lacks the point, or has it as its last point or the one before)
        for &id in selection {
            if !has_record(w, id) {
                continue;
            }
            let Some(r) = w.routes.routes.get(&id) else {
                continue;
            };
            let at = r.steps.iter().position(|s| s.point == p);
            let bad = r.loop_from.is_some() || at.is_none_or(|i| i + 2 >= r.steps.len());
            if bad {
                return RouteClick::Refused("MSG:PlanningModeInvalidNodeX");
            }
        }
        // spec: S-0213 claims 16a and 16b (otherwise the click closes a loop for the objects whose routes hold the point and joins the point for the others: point-join orders)
        let orders = selection
            .iter()
            .map(|&actor| Order::RouteJoin { actor, point: p })
            .collect();
        return RouteClick::Orders(orders);
    }
    // spec: S-0213 claim 16b and S-0214 table VD row VD1, table I rows v2 and I22 (the deploy action is refused in route mode: no-deploy, no order)
    let deploying: Vec<ObjectId> = selection
        .iter()
        .copied()
        .filter(|&id| targeting::action_of(w, house, selection, id, on, mods) == Action::Deploy)
        .collect();
    let mut out = Vec::new();
    for o in plain.orders {
        if o.actor().is_some_and(|a| deploying.contains(&a)) {
            continue;
        }
        // spec: S-0213 claim 16a (a point-add order is the ordinary behaviour order of claim 5 marked as a route point, one per eligible selected object, in selection order); design (Q-2138): orders a click gives that are not behaviour orders, a detonate among them, add no point
        if let Order::Behaviour {
            actor,
            behaviour,
            attack_move,
            target,
            dest,
        } = o
        {
            out.push(Order::RoutePoint {
                actor,
                behaviour,
                attack_move,
                target,
                dest,
            });
        }
    }
    if out.is_empty() {
        // spec: S-0213 claim 16b (the deploy action is refused in route mode: the click gives no order); design (Q-2138): no message, and a detonate likewise
        return RouteClick::Ignored;
    }
    RouteClick::Orders(out)
}

/// The routes drawn for a viewer (S-0213 claim 16g): each holder of the
/// viewer's house or a house allied with it, with the cells of its points
/// in order.
pub fn drawn(w: &World, viewer: HouseId) -> Vec<(ObjectId, Vec<CellXY>)> {
    w.routes
        .routes
        .iter()
        // spec: S-0213 claim 16g (drawn for the viewer's own house and for houses allied with it, never for other houses)
        .filter(|(_, r)| r.house == viewer || w.allied(Some(viewer), Some(r.house)))
        .filter(|(_, r)| !r.steps.is_empty())
        .map(|(&id, r)| {
            let cells = r
                .steps
                .iter()
                .filter_map(|s| w.routes.points.get(&s.point))
                .map(|p| point_cell(w, p))
                .collect();
            (id, cells)
        })
        .collect()
}
