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
//! The input side follows S-0213 claims 16b and 16d: at a press, the kind
//! test ([`press_kind_refusal`]) and what the points whose click areas hold
//! the cursor do ([`press`]: take one, refuse, or leave an ordinary click);
//! a press that takes a point sends [`press_orders`]; at an ordinary
//! click's release, the same-route-state rule and the route-holder limit
//! ([`click`]). The refusals made when a point add is applied are in
//! [`apply_point`] and announced through [`Effect::Route`]. State is world-level ([`RouteWorld`]), saved and
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

/// The most steps any route holds, a point counting once for each time the
/// route holds it (S-0213 claim 16b, our engine's own bound: the original
/// has none, Q-2971). A point add to a route of this length is refused
/// with no message, and a join whose copy would take a route past it does
/// nothing.
pub const MAX_STEPS: usize = 1024; // spec: S-0213 claim 16b and Constants (longest route in our engine: 1024 steps)

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
    /// The tick of the point add that made it: only point adds applied in
    /// that tick share it (claim 16a).
    pub made: i32,
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
    /// Committed (claim 16c): being committed belongs to each route, and
    /// only committed points are followed.
    pub committed: bool,
    /// When the route was given the point: the application stamp of the
    /// point add or join that gave it (claim 16b's longest holder).
    pub since: u64,
}

/// An object's route and route record (S-0213 claims 16a to 16c). The
/// record is the entry itself: kept, even once the route is empty, until
/// the object is removed from play.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Route {
    /// The house whose orders built it.
    pub house: HouseId,
    /// The points still to issue, in the order they will next be issued
    /// (claim 16c).
    pub steps: Vec<Step>,
    /// The loop's first step, when the route has a loop: once the steps
    /// before it have been issued, every step from it to the route's end
    /// repeats (claim 16c, *A loop that ends early*).
    pub loop_from: Option<usize>,
    /// The number of points the loop was given, when the route has a loop
    /// (claim 16c): the route's steps from the loop's first, unless the
    /// loop ends early (claim 16b), when it is fewer. It plays no part in
    /// the order of issue; it counts for a point delete (claim 16e) and a
    /// join's copy (claim 16b, *Joins*).
    #[serde(default)]
    pub loop_len: Option<usize>,
}

impl Route {
    /// Whether the route's loop ends before the route's last point (S-0213
    /// claim 16c, *A loop that ends early*): its number of points is less
    /// than the route's steps from its first.
    pub fn loop_ends_early(&self) -> bool {
        match (self.loop_from, self.loop_len) {
            (Some(l), Some(n)) => n < self.steps.len().saturating_sub(l),
            _ => false,
        }
    }

    /// Sets the route's loop: its first step and its number of points.
    fn set_loop(&mut self, lp: Option<LoopSpan>) {
        self.loop_from = lp.map(|(f, _)| f);
        self.loop_len = lp.map(|(_, n)| n);
    }

    /// The point's first place in the route (S-0213 claim 16b, *A point
    /// that a route holds twice*).
    pub fn first_place(&self, point: u32) -> Option<usize> {
        self.steps.iter().position(|s| s.point == point)
    }

    /// Whether the route holds the point.
    pub fn holds_point(&self, point: u32) -> bool {
        self.first_place(point).is_some()
    }

    /// The route's last point, if any.
    pub fn last_point(&self) -> Option<u32> {
        self.steps.last().map(|s| s.point)
    }

    /// The step recorded for the holder at a point: the one at the point's
    /// earliest place still in the route (claim 16b).
    fn action_at(&self, point: u32) -> Option<&Step> {
        self.steps.iter().find(|s| s.point == point)
    }

    /// When the route was given the point: the earliest stamp of its
    /// places (claim 16b, the longest holder).
    fn given(&self, point: u32) -> Option<u64> {
        self.steps
            .iter()
            .filter(|s| s.point == point)
            .map(|s| s.since)
            .min()
    }
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
    /// The next application stamp of a point add or join, in application
    /// order (claim 16b: the longest holder).
    pub stamp: u64,
}

/// What an ordinary route-mode click does at its release (S-0213 claims
/// 16b and 16d): one that no point's click area takes or refuses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteClick {
    /// The orders to issue: point adds, and a detonate or a building's rally
    /// point on a cell sent as outside route mode (claim 16b).
    Orders(Vec<Order>),
    /// Refused, with the string-table label of the message; no order.
    Refused(&'static str),
    /// The ordinary click: one that selects, toggles the selection or does
    /// nothing outside route mode; the selection changes as there, and no
    /// order is given.
    Ordinary,
    /// A click whose orders route mode gives none of (a deploy, or an order
    /// other than a behaviour order, a detonate or a rally point on a cell): no
    /// order, no point, no message and no selection change.
    Ignored,
}

/// What a route-mode press does with the points whose click areas hold the
/// cursor (S-0213 claim 16b, *Click areas*).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Press {
    /// The press takes the point: with objects selected it sends
    /// [`press_orders`], with nothing selected it selects the point (claim
    /// 16e).
    Take(u32),
    /// Refused with the invalid-point message.
    Refused,
    /// An ordinary route-mode click (claim 16d): [`click`] at the release,
    /// or a tool's click.
    Ordinary,
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
    /// A point delete was applied, and this point, the one after the
    /// deleted point in its longest holder's route, becomes the selected
    /// point (claim 16e).
    Selected(u32),
}

/// The place of a behaviour order a route-mode click `on` gave (S-0213
/// claim 16b): its target's; for a *move* order, which names no target,
/// the clicked cell, or for a click on an object the cell that object
/// stands on at the click; for any other order that names no target, its
/// destination's. An order with neither a target nor a destination has no
/// place. An attack-move click has the place that the same click gives
/// without attack-move entry, so the order's attack-move mark plays no
/// part.
pub fn order_place(
    w: &World,
    on: ClickOn,
    behaviour: Behaviour,
    target: Option<Target>,
    dest: Option<Target>,
) -> Option<Target> {
    if let Some(t) = target {
        // spec: S-0213 claim 16b (revision 84: the place of the target that the click gave the order; revision 89, Q-2230: an attack-move click on an object whose order names a target puts the point at the target's place)
        return Some(t);
    }
    // spec: S-0213 claim 16b (revision 89, Q-2230: an attack-move click has the place that the same click gives without attack-move entry; entry changes only the order's kind, not the target, destination or cell the click gave it, so an attack move that would be a *move* without entry has the move's place)
    if behaviour == Behaviour::Move {
        // spec: S-0213 claim 16b (revision 84: a move order's place is the clicked cell's, or for a click on an object that of the cell the object stands on at the click, whatever the move's destination, also one the nearby-cell search found; revision 89: the same for an attack-move click on a cell or on an object)
        return match on {
            ClickOn::Cell(c) => Some(Target::Cell(c)),
            ClickOn::Object(x) => w.obj(x).map(|o| Target::Cell(o.pos.cell())),
        };
    }
    // spec: S-0213 claim 16b (revision 84: any other order that names no target has its destination's place; one with neither a target nor a destination has no place)
    dest
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

/// The house's earliest-made point whose place's cell is `cell`, if any (a
/// lookup for tools and tests; a press finds its point by click area,
/// [`press`]).
pub fn point_in(w: &World, house: HouseId, cell: CellXY) -> Option<u32> {
    w.routes
        .points
        .iter()
        .find(|(_, p)| p.house == house && point_cell(w, p) == cell)
        .map(|(&n, _)| n)
}

/// A point's click area holds a screen pixel `(dx, dy)` away from the
/// screen position of the point's place (S-0213 claim 16b, *Click areas*):
/// a square of 16 by 16 screen pixels, from 8 left of and above that
/// position to 7 right of and below it.
pub fn in_click_area(dx: i32, dy: i32) -> bool {
    // spec: S-0213 claim 16b, Click areas (from 8 pixels left of and above the place's screen position to 7 pixels right of and below it; the point's cell plays no part)
    (-8..=7).contains(&dx) && (-8..=7).contains(&dy)
}

/// Whether the point is drawn for `viewer` (S-0213 claim 16g): one of its
/// own house's or of a house allied with it.
pub fn drawn_for(w: &World, viewer: HouseId, p: &RoutePoint) -> bool {
    // spec: S-0213 claim 16g (drawn for the viewer's own house and for houses allied with it, never for other houses)
    p.house == viewer || w.allied(Some(viewer), Some(p.house))
}

/// The point-click test (S-0213 claim 16b, *Clicks on existing points*):
/// whether the selection can join or close a loop at the point. It refuses
/// when a selected object with a route record has a loop, lacks the point,
/// or has it (at its first place) as its route's last point or the one
/// before it. With nothing selected, or only objects with no route record,
/// it accepts every point.
pub fn accepts(w: &World, selection: &[ObjectId], point: u32) -> bool {
    selection.iter().all(|&id| {
        if !has_record(w, id) {
            return true;
        }
        let Some(r) = w.routes.routes.get(&id) else {
            return true;
        };
        // spec: S-0213 claim 16b (refused when a selected object with a route record has a loop, lacks the point, or has it as its route's last point or the one before it, at its first place in the route)
        r.loop_from.is_none() && r.first_place(point).is_some_and(|i| i + 2 < r.steps.len())
    })
}

/// The point taken from an ordered list of points whose click areas hold
/// the cursor (earliest-made first): the earliest-made, or, under
/// `quirks.stacked_point_takes_earliest` off with objects selected, the
/// earliest-made one at the first's place that a selected route holds.
fn pick(w: &World, selection: &[ObjectId], list: &[u32]) -> Option<u32> {
    let &first = list.first()?;
    // spec: S-0213 claim 16b, Two points at one place (the earliest-made point is the candidate, whatever the selected routes hold) and Edge cases (quirks.stacked_point_takes_earliest)
    if w.rules().quirks.stacked_point_takes_earliest || selection.is_empty() {
        return Some(first);
    }
    let Some(f) = w.routes.points.get(&first) else {
        return Some(first);
    };
    // spec: S-0213 claim 16b, Two points at one place, quirk off (with objects selected, the earliest-made point of the house at the place that a selected object's route holds, else the earliest-made one)
    let held = |n: u32| {
        selection
            .iter()
            .any(|id| w.routes.routes.get(id).is_some_and(|r| r.holds_point(n)))
    };
    list.iter()
        .copied()
        .find(|n| {
            w.routes
                .points
                .get(n)
                .is_some_and(|p| p.house == f.house && p.place == f.place)
                && held(*n)
        })
        .or(Some(first))
}

/// The points of `under` that play a part in a press by `house`: all of
/// them, or under `quirks.hidden_point_refuses_click` off only those drawn
/// for the player.
fn tested<'a>(w: &'a World, house: HouseId, under: &'a [u32]) -> impl Iterator<Item = u32> + 'a {
    // spec: S-0213 claim 16b, Click areas (a press that takes no highlighted point is tested against the earliest-made point of any house, drawn or not) and Edge cases (quirks.hidden_point_refuses_click off: only points drawn for the player)
    let all = w.rules().quirks.hidden_point_refuses_click;
    under.iter().copied().filter(move |n| {
        w.routes
            .points
            .get(n)
            .is_some_and(|p| all || drawn_for(w, house, p))
    })
}

/// What a route-mode press by `house` with `selection` does, with `under`
/// the points of any house whose click areas hold the cursor, in any order
/// (S-0213 claim 16b, *Click areas* and *Two points at one place*).
pub fn press(w: &World, house: HouseId, selection: &[ObjectId], under: &[u32]) -> Press {
    let mut under: Vec<u32> = under
        .iter()
        .copied()
        .filter(|n| w.routes.points.contains_key(n))
        .collect();
    // spec: S-0213 claim 16b, Click areas (the earliest-made point is the candidate; point numbers are given in the order the points were made)
    under.sort_unstable();
    under.dedup();
    let drawn: Vec<u32> = under
        .iter()
        .copied()
        .filter(|n| drawn_for(w, house, &w.routes.points[n]))
        .collect();
    // spec: S-0213 claim 16b, Click areas (of the points drawn for the player whose click areas hold the cursor, the earliest-made one is the candidate; it is highlighted, and a press takes it, only when it is a point of the player's own house that the point-click test accepts; otherwise no later-made point under it is considered)
    if let Some(c) = pick(w, selection, &drawn)
        && w.routes.points[&c].house == house
        && accepts(w, selection, c)
    {
        return Press::Take(c);
    }
    let rest: Vec<u32> = tested(w, house, &under).collect();
    // spec: S-0213 claim 16b, Click areas (when the point-click test refuses that point, the press is refused with the invalid-point message; otherwise, and when no click area holds the cursor, the press is an ordinary route-mode click)
    match pick(w, selection, &rest) {
        Some(t) if !accepts(w, selection, t) => Press::Refused,
        _ => Press::Ordinary,
    }
}

/// The highlighted point (S-0213 claim 16b): the one a press would take,
/// if any.
pub fn highlighted(
    w: &World,
    house: HouseId,
    selection: &[ObjectId],
    under: &[u32],
) -> Option<u32> {
    // spec: S-0213 claim 16b (while the cursor is over a point that the point-click test would refuse, the point is not highlighted as a click target)
    match press(w, house, selection, under) {
        Press::Take(p) => Some(p),
        _ => None,
    }
}

/// The kind refusal (S-0213 claim 16d): the message of the first selected
/// object, in selection order, that is not a vehicle or infantry unit.
pub fn kind_refusal(w: &World, selection: &[ObjectId]) -> Option<&'static str> {
    // spec: S-0213 claim 16d (a selected aircraft, building or object of any other kind refuses the press, each with its message; when several fail, the message is that of the first of them in S-0214's selection order)
    selection
        .iter()
        .find_map(|&id| match w.obj(id).map(|o| o.kind) {
            Some(Kind::Vehicle | Kind::Infantry) => None,
            Some(Kind::Aircraft) => Some("MSG:PlanningModeNoAircraft"),
            Some(Kind::Building) => Some("MSG:PlanningModeNoBuildings"),
            _ => Some("MSG:PlanningModeNoUnknowns"),
        })
}

/// The kind test of a route-mode battlefield press (S-0213 claim 16d,
/// *When the tests are made*): made when the button is pressed, whatever
/// the click would do, for a press with the cursor outside every point's
/// click area; under `quirks.click_area_skips_kind_test` off for every
/// press with objects selected. A refusal ends the click.
pub fn press_kind_refusal(
    w: &World,
    house: HouseId,
    selection: &[ObjectId],
    under: &[u32],
) -> Option<&'static str> {
    if selection.is_empty() {
        // spec: S-0213 claim 16d (with nothing selected, no object is tested)
        return None;
    }
    // spec: S-0213 claim 16d, When the tests are made (revision 84, Q-2193: the click areas here are those of every point of every house, drawn for the player or not; with quirks.hidden_point_refuses_click off, only those drawn for the player) and Edge cases (with that switch off, a press inside only the click areas of points not drawn for the player counts as one outside every click area)
    let inside = tested(w, house, under).next().is_some();
    // spec: S-0213 claim 16d, When the tests are made (a press inside a click area is not tested for kind) and Edge cases (quirks.click_area_skips_kind_test off: every route-mode battlefield press with objects selected meets the kind test first)
    if inside && w.rules().quirks.click_area_skips_kind_test {
        return None;
    }
    kind_refusal(w, selection)
}

/// The objects whose routes hold the point, in the order in which they were
/// given it (S-0213 claim 16b), of `house`.
fn holders_of(w: &World, house: HouseId, point: u32) -> Vec<ObjectId> {
    let mut v: Vec<(u64, ObjectId)> = w
        .routes
        .routes
        .iter()
        .filter(|(_, r)| r.house == house)
        .filter_map(|(&id, r)| r.given(point).map(|g| (g, id)))
        .collect();
    v.sort_unstable();
    v.into_iter().map(|(_, id)| id).collect()
}

/// The objects a press on a point sends orders to (S-0213 claim 16b, *Who
/// gets a press's orders*): the selection in S-0214's selection order, a
/// selected object with no route record or an empty route itself, and for
/// one whose route holds points every object whose route holds that
/// route's last point, in the order in which they were given it, selected
/// or not; each object once.
pub fn press_list(w: &World, house: HouseId, selection: &[ObjectId]) -> Vec<ObjectId> {
    let mut list: Vec<ObjectId> = Vec::new();
    let put = |list: &mut Vec<ObjectId>, id: ObjectId| {
        // spec: S-0213 claim 16b, Who gets a press's orders (an object already listed is not listed again)
        if !list.contains(&id) {
            list.push(id);
        }
    };
    for &id in selection {
        let last = w.routes.routes.get(&id).and_then(Route::last_point);
        match last {
            // spec: S-0213 claim 16b, Who gets a press's orders (a selected object with no route record, or with an empty route, is listed itself)
            None => put(&mut list, id),
            // spec: S-0213 claim 16b, Who gets a press's orders (for a selected object whose route holds points, every object whose route holds that route's last point is listed, the selected object among them, in the order in which they were given that point, whether they are selected or not)
            Some(p) => {
                for h in holders_of(w, house, p) {
                    put(&mut list, h);
                }
            }
        }
    }
    list
}

/// The orders of a press that takes `point` with objects selected (S-0213
/// claims 2 and 16b): one point join for each object of [`press_list`], in
/// its order, naming the point by its place.
pub fn press_orders(w: &World, house: HouseId, selection: &[ObjectId], point: u32) -> Vec<Order> {
    let Some(p) = w.routes.points.get(&point) else {
        return Vec::new();
    };
    let place = p.place;
    // spec: S-0213 claims 2 and 16b (the orders are issued in the list's order, so the pending-list limit drops those after the first 128 in the list; they name the point by its place)
    press_list(w, house, selection)
        .into_iter()
        .map(|actor| Order::RouteJoin {
            actor,
            point,
            place,
        })
        .collect()
}

/// The objects that selecting `id` while the route-mode key is held also
/// selects (S-0213 claim 16h): every other object whose route holds the
/// last point of `id`'s route, in the order in which they were given it.
pub fn also_selected(w: &World, id: ObjectId) -> Vec<ObjectId> {
    let Some(r) = w.routes.routes.get(&id) else {
        return Vec::new();
    };
    let Some(last) = r.last_point() else {
        return Vec::new();
    };
    // spec: S-0213 claim 16h (selecting, while the key is held, an object whose route holds a point also selects every other object whose route holds that route's last point, in the order in which they were given that point)
    holders_of(w, r.house, last)
        .into_iter()
        .filter(|&h| h != id)
        .collect()
}

/// Deletes every point that no route holds any more.
fn drop_orphans(w: &mut World) {
    let held: std::collections::BTreeSet<u32> = w
        .routes
        .routes
        .values()
        .flat_map(|r| r.steps.iter().map(|s| s.point))
        .collect();
    // spec: S-0213 claim 16f (a route point that no route holds any more is deleted at once, whichever way its last holder left it: an order removing the route, issuing it from a route without a loop or before a loop's first point, a point delete, or the holder's destruction)
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
    r.set_loop(None);
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
/// the actor's route, recording the order for the actor.
pub fn apply_point(
    w: &mut World,
    house: HouseId,
    actor: ObjectId,
    step: Step,
    place: Option<Target>,
) {
    // spec: S-0213 claim 16d (revision 84, Q-2192: an aircraft or a building is a route holder like any other object; only claim 16b's four refusals apply to its point adds)
    if w.obj(actor).is_none() {
        return;
    }
    // spec: S-0213 claim 16b (revision 84: an order that names neither a target nor a destination has no place: when applied it adds no point, with no message and no sound, and has no other effect)
    let Some(at) = place else { return };
    let cell = place_cell(w, at).unwrap_or(CellXY::new(0, 0));
    if let Some(r) = w.routes.routes.get(&actor) {
        // spec: S-0213 claim 16b (our engine's bound: a point add to a route of 1024 steps or more is refused with no message and no sound)
        if r.steps.len() >= MAX_STEPS {
            return;
        }
        // spec: S-0213 claim 16b, A point that a route holds twice (the action read at the last point is the one recorded at that point's earliest place still in the route)
        let last = r.last_point().and_then(|p| r.action_at(p));
        // spec: S-0213 claim 16b (a point add is refused, when applied, after a terminating or continual last point, in a loop, or at 16 points; each with its own message) and Constants (refused only at exactly 16: a longer route, from a press's copy, is not refused for its length)
        let refusal = if r.loop_from.is_some() {
            Some("MSG:PostLoop")
        } else if r.steps.len() == MAX_POINTS {
            Some("MSG:NodeMaximum")
        } else if last.is_some_and(terminating) {
            Some("MSG:PostTerminatingCommand")
        } else if last.is_some_and(continual) {
            Some("MSG:PostContinualCommand")
        } else {
            None
        };
        if let Some(m) = refusal {
            note(w, house, RouteNote::Refused(m));
            return;
        }
    }
    let tick = w.tick;
    // spec: S-0213 claim 16a (point adds share a point only when they are applied in the same tick: a later point add makes a new point, even at an existing point's place)
    let existing = w
        .routes
        .points
        .iter()
        .find(|(_, p)| p.house == house && p.place == at && p.made == tick)
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
                    made: tick,
                },
            );
            n
        }
    };
    let since = w.routes.stamp;
    w.routes.stamp = since.wrapping_add(1);
    let r = w.routes.routes.entry(actor).or_insert(Route {
        house,
        steps: Vec::new(),
        loop_from: None,
        loop_len: None,
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
    r.steps.push(Step {
        point,
        // spec: S-0213 claim 16c (points added since the last commit are not followed)
        committed: false,
        since,
        ..step
    });
    if first {
        // spec: S-0213 Terms (applying a point add to an object whose route holds no point raises the count by 1)
        *w.routes.counts.entry(house).or_insert(0) += 1;
    }
    // spec: S-0213 claim 16b (an accepted point plays the route-point sound; an added terminating, continual or 16th point shows that case's message as information)
    note(w, house, RouteNote::Added { info });
}

/// The point a press's join order acts on when applied (S-0213 claim 16b,
/// *Click areas*): the house's earliest-made point at the order's place;
/// under `quirks.stacked_point_takes_earliest` off the point the order
/// names. `None` when there is none: the order does nothing.
fn join_point(w: &World, house: HouseId, point: u32, place: Target) -> Option<u32> {
    if w.rules().quirks.stacked_point_takes_earliest {
        // spec: S-0213 claim 16b, Click areas (the orders a press sends name the point by its place; when applied they act on the house's earliest-made point at that place, and do nothing when no point of the house is there)
        w.routes
            .points
            .iter()
            .find(|(_, p)| p.house == house && p.place == place)
            .map(|(&n, _)| n)
    } else {
        // spec: S-0213 claim 16b, Two points at one place, quirk off (the press's orders name the point itself: when applied they act on it, and an order whose point has gone does nothing)
        w.routes
            .points
            .get(&point)
            .is_some_and(|p| p.house == house)
            .then_some(point)
    }
}

/// A loop as its first step and its number of points ([`Route::loop_from`],
/// [`Route::loop_len`]).
type LoopSpan = (usize, usize);

/// The copy a join gives (S-0213 claim 16b, *Joins*): the steps of the
/// point's longest holder's route from the point's first place to its end,
/// and the copy's loop as a step range within it.
fn join_copy(
    w: &World,
    house: HouseId,
    actor: ObjectId,
    point: u32,
) -> Option<(Vec<Step>, Option<LoopSpan>)> {
    // spec: S-0213 claim 16b (the join copies the route of the point's longest holder: of the routes holding the point when the join is applied, the one given it earliest by a point add or join; of holders given it in the same tick, the one whose order was applied first)
    let src = holders_of(w, house, point)
        .into_iter()
        .find(|&id| id != actor)?;
    let src = &w.routes.routes[&src];
    let i = src.first_place(point)?;
    // spec: S-0213 claim 16b (the copy runs from the joined point's first place in that route to the end of that route as it then stands, with the actions recorded there for that holder)
    let mut steps: Vec<Step> = src.steps[i..].to_vec();
    let lp = match (src.loop_from, src.loop_len) {
        // spec: S-0213 claim 16b, Joins (its loop's first point is the joined point or a later one: the copy has the same loop, from the same first point) and revision 84 (Q-2191: the copy's loop is given as many points as that route's loop has, also when that loop ends early, so when the joined point comes before the loop's first point the copy's loop begins at that point's place in the copy and ends early as the source loop does)
        (Some(l), Some(n)) if l >= i => Some((l - i, n)),
        // spec: S-0213 claim 16b, Joins (revision 84, Q-2191: its loop's first point comes before the joined point: the copy goes on, after that route's last point, with that route's points from its loop's first point up to the one before the joined point; the copy's loop begins at the joined point and is given the source loop's number of points, so it is the whole copy unless the source loop ends early, when it ends early too)
        (Some(l), Some(n)) => {
            steps.extend_from_slice(&src.steps[l..i]);
            Some((0, n))
        }
        // spec: S-0213 claim 16b (that route has no loop: the copy has none)
        _ => None,
    };
    Some((steps, lp))
}

/// Applies a point join (S-0213 claims 16a to 16c), deciding when it is
/// applied: it closes a loop when the actor's route then holds the point
/// the order acts on, and otherwise adds a copy of the point's longest
/// holder's route after the actor's own points. Neither is tested for the
/// point-click test's refusals.
pub fn apply_join(w: &mut World, house: HouseId, actor: ObjectId, point: u32, place: Target) {
    if w.obj(actor).is_none() {
        return;
    }
    let Some(point) = join_point(w, house, point, place) else {
        return;
    };
    if let Some(r) = w.routes.routes.get_mut(&actor)
        && let Some(i) = r.first_place(point)
    {
        // spec: S-0213 claims 16b and 16c (it closes a loop when, at the order's application, the point is in the actor's route: from the point's first place to the route's last point, with no test, replacing any loop the route already has)
        let n = r.steps.len() - i;
        r.set_loop(Some((i, n)));
        loop_attack_moves(w, house, actor);
        // spec: S-0213 claim 16c (closing a loop shows an information message)
        note(w, house, RouteNote::Looped);
        return;
    }
    let Some((mut copy, lp)) = join_copy(w, house, actor, point) else {
        return;
    };
    let own: Vec<Step> = w
        .routes
        .routes
        .get(&actor)
        .map(|r| r.steps.clone())
        .unwrap_or_default();
    let n = own.len();
    let ignores = w.rules().quirks.press_copy_ignores_route_state;
    // spec: S-0213 claim 16b, Who gets a press's orders, and Edge cases (quirks.press_copy_ignores_route_state off: a join does nothing when the copy would take its route past 16 points)
    if !ignores && n + copy.len() > MAX_POINTS {
        return;
    }
    // spec: S-0213 claim 16b (our engine's bound: a join whose copy would take the route past 1024 steps does nothing, adding no part of its copy, with the switch on or off)
    if n + copy.len() > MAX_STEPS {
        return;
    }
    let since = w.routes.stamp;
    w.routes.stamp = since.wrapping_add(1);
    for s in &mut copy {
        // spec: S-0213 claim 16b (the copied points are uncommitted until the next commit)
        s.committed = false;
        s.since = since;
    }
    let mut steps = own;
    // spec: S-0213 claim 16b (the copy is added after the actor's own points, whatever their number; a join is not tested for the point add refusals)
    steps.extend(copy);
    let lp = match lp {
        // spec: S-0213 claim 16b (its route then has the copy's loop in place of any loop it had, or no loop when the copy has none)
        None => None,
        Some((cf, cn)) if !ignores => {
            // spec: S-0213 Edge cases (quirks.press_copy_ignores_route_state off: the copy's loop always begins in the copy, at the copied loop's first point) and claim 16b, Joins (revision 84: with the copied loop's number of points)
            Some((n + cf, cn))
        }
        Some((cf, cn)) => {
            let first = steps[n + cf].point;
            // spec: S-0213 claim 16b, A point that a route holds twice (the copy's loop begins at the first place, in the joining holder's route, of the copied loop's first point; when that is one of its own points, the loop has as many points as the copied loop, so it ends before its route's last point) and Joins (revision 84: the copy's loop is given the copied loop's number of points in every case)
            let j = steps
                .iter()
                .position(|s| s.point == first)
                .unwrap_or(n + cf);
            Some((j, cn))
        }
    };
    let r = w.routes.routes.entry(actor).or_insert(Route {
        house,
        steps: Vec::new(),
        loop_from: None,
        loop_len: None,
    });
    r.house = house;
    r.steps = steps;
    r.set_loop(lp);
    // spec: S-0213 claim 16f (a point join that gives an object its first points does not raise the count, under quirks.route_holder_count_drift)
}

/// The attack moves of a loop just closed on `actor`'s route (S-0213 claim
/// 16c, *Attack moves in a loop*): when any holder of the loop's first
/// point is of a type eligible for attack moves (claim 6), every point from
/// the loop's first point to the route's end whose recorded order for a
/// holder is a *move* becomes an attack-move order with the same
/// destination, for each such holder of that point.
fn loop_attack_moves(w: &mut World, house: HouseId, actor: ObjectId) {
    let Some(r) = w.routes.routes.get(&actor) else {
        return;
    };
    let Some(l) = r.loop_from else { return };
    let Some(first) = r.steps.get(l).map(|s| s.point) else {
        return;
    };
    // spec: S-0213 claim 16c (revision 84: from the loop's first point to the route's end, a loop that ends early included; points before the loop's first point keep their orders)
    let points: std::collections::BTreeSet<u32> = r.steps[l..].iter().map(|s| s.point).collect();
    // spec: S-0213 claim 16c (revision 84: whether a type is eligible is read from the holders of the loop's first point only, when the loop-close order is applied) and claim 6 (S-0214 claim 5's eligibility)
    if !holders_of(w, house, first)
        .into_iter()
        .any(|h| orders::attack_move_eligible(w, h))
    {
        return;
    }
    for (_, hr) in w.routes.routes.iter_mut().filter(|(_, r)| r.house == house) {
        for s in hr.steps.iter_mut().filter(|s| points.contains(&s.point)) {
            // spec: S-0213 claim 16c (revision 84: the test is made per holder: each holder whose recorded order at the point is a move gets an attack-move order with the same destination; each whose order there is not a move keeps it)
            if s.behaviour == Behaviour::Move && !s.attack_move {
                s.attack_move = true;
            }
        }
    }
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
        .filter(|(_, r)| next_step(r).is_some_and(|s| !s.committed))
        .map(|(&id, _)| id)
        .collect();
    // spec: S-0213 claim 16c (when a commit is applied, every point of the house's routes becomes committed; being committed belongs to each route, not to the place)
    for r in w.routes.routes.values_mut() {
        if r.house == house {
            for s in &mut r.steps {
                s.committed = true;
            }
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
            committed: false,
            since: 0,
        };
        // spec: S-0213 claim 16c (a holder whose route could not run before the commit starts at once if its current behaviour is guard or continual: it gives up that behaviour and waits, and issues its next point, which takes effect at the next order boundary; any other holder is not interrupted)
        if o.beh.current.is_some() && continual(&current) {
            wait(w, id);
            issue_next(w, id);
        }
    }
}

/// Gives up the holder's behaviour before a route point's issue (S-0213
/// claim 16c): a queue of *wait* with an immediate start. The target and
/// destination are left as they are.
fn wait(w: &mut World, id: ObjectId) {
    let t = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let ready = crate::process::ready(w, id);
    if let Some(o) = w.obj_mut(id) {
        // spec: S-0213 claim 16c (giving up is a queue of wait with an immediate start, S-0215 claim 7, at the commit and before each fall-back's issue; the commit does not change the target or destination)
        o.beh
            .queue_and_start(Some(Behaviour::Wait), ready, t, stale);
    }
}

/// Applies a point delete (S-0213 claim 16e): the point leaves every route
/// that holds it, and a loop left with fewer than 3 points is cancelled.
pub fn apply_delete(w: &mut World, house: HouseId, point: u32) {
    if w.routes.points.get(&point).is_none_or(|p| p.house != house) {
        return;
    }
    let next = next_after(w, house, point);
    for r in w.routes.routes.values_mut() {
        let mut kept = Vec::with_capacity(r.steps.len());
        let (mut from, mut n) = (r.loop_from, r.loop_len);
        let l0 = r.loop_from;
        // spec: S-0213 claim 16b, A point that a route holds twice (a point delete removes the point from every place in every route that holds it)
        for (i, s) in r.steps.drain(..).enumerate() {
            if s.point == point {
                match l0 {
                    // spec: S-0213 claim 16e (a point before the loop's first point: the loop keeps its points)
                    Some(l) if i < l => from = from.map(|f| f - 1),
                    // spec: S-0213 claim 16e (revision 84: deleting any point at or after the loop's first point lowers the loop's number of points by 1, a point after a loop that ends early included)
                    Some(_) => n = n.map(|n| n.saturating_sub(1)),
                    None => {}
                }
                continue;
            }
            kept.push(s);
        }
        r.steps = kept;
        // spec: S-0213 claim 16e (a loop left with fewer than 3 points is cancelled; the remaining points form an ordinary route) and revision 84 (on a loop that ends early, once its number of points is below 3)
        let lp = from.zip(n).filter(|&(_, n)| n >= MIN_LOOP);
        r.set_loop(lp);
    }
    // spec: S-0213 claim 16f (deleting a route's last remaining point ends holding but does not lower the count, under quirks.route_holder_count_drift; off, the count is the holders')
    w.routes.points.remove(&point);
    if let Some(n) = next.filter(|n| w.routes.points.contains_key(n)) {
        // spec: S-0213 claim 16e, The delete key (when a point-delete order of the local player's house is applied, the point after the deleted one in the route of its longest holder, going round a loop, becomes the selected point, if there is one)
        note(w, house, RouteNote::Selected(n));
    }
}

/// The point after `point`'s first place in the route of its longest
/// holder, going round a loop (S-0213 claim 16e and claim 16b, *A point
/// that a route holds twice*). It is found in that route as it stands
/// before the delete.
fn next_after(w: &World, house: HouseId, point: u32) -> Option<u32> {
    let h = holders_of(w, house, point).into_iter().next()?;
    let r = &w.routes.routes[&h];
    // spec: S-0213 claim 16e (revision 89, Q-2226 question 2: where the route holds the deleted point at two places, the point after its first place is taken; that first place is never the route's last point, so the count round the loop does not apply)
    let i = r.first_place(point)?;
    let last = i + 1 == r.steps.len();
    let j = match (r.loop_from, r.loop_len) {
        // spec: S-0213 claim 16e (revision 89, Q-2226 question 1: when the deleted point is the route's last point, the count goes on round the loop's points only: with n the points from the loop's first to the route's last, both included, and C the loop's number of points, the selected point is the one n mod C places after the loop's first point; on a loop that runs to the route's end n = C, so it is the loop's first point)
        (Some(l), c) if last => {
            let n = r.steps.len() - l;
            l + n % c.unwrap_or(n).max(1)
        }
        // spec: S-0213 claim 16e (revision 89: a route without a loop gives no point when its last point is deleted; otherwise the point after the deleted one)
        _ => i + 1,
    };
    r.steps.get(j).map(|s| s.point).filter(|&n| n != point)
}

/// The step a route issues next, if any: its first (S-0213 claim 16c, the
/// route holds its points in the order they will next be issued).
fn next_step(r: &Route) -> Option<&Step> {
    r.steps.first()
}

/// Whether the object follows a route whose next point is committed
/// (S-0215 claim 13's route row).
pub fn follows(w: &World, id: ObjectId) -> bool {
    w.routes
        .routes
        .get(&id)
        .and_then(next_step)
        .is_some_and(|s| s.committed)
}

/// The route row of the fall-back to rest (S-0215 claim 13): a holder
/// whose next point is committed gives up its behaviour and issues the
/// point's action as an order.
pub fn fall_back(w: &mut World, id: ObjectId) {
    // spec: S-0213 claim 16d (revision 84, Q-2192: an aircraft issues no route point when it falls back to rest; a building's moments are its own) and Changes for other specs (S-0215 claim 13's route row: infantry units and vehicles only)
    if !w
        .obj(id)
        .is_some_and(|o| matches!(o.kind, Kind::Infantry | Kind::Vehicle))
    {
        return;
    }
    // spec: S-0215 claim 13 (it follows a route and is not in a patrol that has already begun: it issues the action of the next route point as an order; not an immediate destination assignment) and S-0213 claim 16c (uncommitted points are not followed: the holder waits for a commit)
    issue_later(w, id);
}

/// A holder's issue of its next route point at one of the moments of S-0213
/// claims 16c and 16d (an infantry unit's or vehicle's fall-back to rest,
/// an aircraft's flight end, a building's *attack* without a target): when
/// its next point is committed, it queues *wait* with an immediate start
/// and issues the point. Returns whether it issued one.
pub fn issue_later(w: &mut World, id: ObjectId) -> bool {
    if !follows(w, id) {
        return false;
    }
    // spec: S-0213 claim 16c and Changes for other specs (S-0215 claim 13: each issue at a fall-back is preceded by the queue of wait with an immediate start, so a ready holder's rest table finds wait current and queues no guard) and revision 84 (an aircraft or building holder issues its later points at the moments of claim 16d instead; each such issue is preceded by the same queue of wait)
    wait(w, id);
    issue_next(w, id);
    true
}

/// Issues the next point of the object's route (S-0213 claim 16c): a point
/// outside a loop is taken off the front, so a route without a loop is used
/// up when its last point is issued; a loop's point moves to the end.
fn issue_next(w: &mut World, id: ObjectId) {
    let Some(owner) = w.obj(id).and_then(|o| o.owner) else {
        return;
    };
    let Some(r) = w.routes.routes.get_mut(&id) else {
        return;
    };
    if r.steps.is_empty() {
        return;
    }
    let house = r.house;
    let step = r.steps.remove(0);
    match r.loop_from {
        // spec: S-0213 claim 16c (issuing a point of a loop moves it to the end of the route, so a looping holder's route holds its points in the order it will next issue them) and revision 84 (Q-2191, A loop that ends early: once the points before the loop's first point have all been issued, every issued point, of the loop or after it, moves to the end of the route; the loop's number of points plays no part in the order of issue)
        Some(0) => r.steps.push(step.clone()),
        // spec: S-0213 claim 16c (issuing a point takes it off the front of the route; points before the loop's first point are issued once)
        Some(l) => r.loop_from = Some(l - 1),
        None => {}
    }
    let used_up = r.steps.is_empty();
    if used_up {
        // spec: S-0213 claim 16f (a route without a loop is used up when its last point is issued: the holder stops holding and lowers the count then)
        lower(w, house);
    }
    // spec: S-0213 claim 16b (revision 84: the order a holder issues for a point is the one recorded for it at the add, or copied to it at a join, as changed by a loop's attack-move change; it is not classified anew when it is issued) and Changes for other specs (S-0215 claim 13: the route row's object classification, service and alliance guards and cell-action fallback do not apply to these points)
    let order = orders::RoutedOrder {
        actor: id,
        behaviour: step.behaviour,
        attack_move: step.attack_move,
        target: step.target,
        dest: step.dest,
    };
    // spec: S-0213 claim 16f (a point issued from a route without a loop or before a loop's first point that no route holds any more is deleted at once)
    drop_orphans(w);
    // spec: S-0215 claim 13 (the route point issues an order through the ordinary order timing of S-0213 claim 1) and S-0213 claim 17 (orders a route issues for its points do not end it)
    orders::issue_routed(w, owner, order);
}

/// An ordinary route-mode click by `house` with `selection`, at its release
/// (S-0213 claims 16b and 16d): one that no point's click area takes or
/// refuses ([`press`]), past the kind test made at the press
/// ([`press_kind_refusal`]). A click that selects, toggles the selection or
/// does nothing is the ordinary click and meets no test here. Any other
/// meets the same-route-state and holder-limit tests, and then gives its
/// behaviour orders as point adds and its detonate as it is.
pub fn click(
    w: &World,
    house: HouseId,
    selection: &[ObjectId],
    on: ClickOn,
    mods: Modifiers,
) -> RouteClick {
    let plain = targeting::click(w, house, selection, on, mods);
    // spec: S-0213 claims 16b and 16d (a click that selects or toggles the selection gives the unit no action and adds no point; it is not tested for route state or the holder limit)
    if selection.is_empty() || plain.orders.is_empty() {
        return RouteClick::Ordinary;
    }
    // spec: S-0213 claim 16b and S-0214 table VD row VD1, table I rows v2 and I22 (for a vehicle or infantry unit the deploy action is refused in route mode: no-deploy, no order) and revision 84 (an aircraft's deploy is not refused for kind: its unload order has no place and adds no point when applied)
    let deploying: Vec<ObjectId> = selection
        .iter()
        .copied()
        .filter(|&id| {
            w.obj(id)
                .is_some_and(|o| matches!(o.kind, Kind::Vehicle | Kind::Infantry))
                && targeting::action_of(w, house, selection, id, on, mods) == Action::Deploy
        })
        .collect();
    let mut out = Vec::new();
    for o in plain.orders {
        if o.actor().is_some_and(|a| deploying.contains(&a)) {
            continue;
        }
        match o {
            // spec: S-0213 claims 16a and 16b (a point-add order is the ordinary behaviour order of claim 5 marked as a route point, one per eligible selected object, in selection order: every click action of a vehicle or infantry unit that orders something, except the detonate, gives a behaviour order)
            Order::Behaviour {
                actor,
                behaviour,
                attack_move,
                target,
                dest,
            } => out.push(Order::RoutePoint {
                actor,
                behaviour,
                attack_move,
                target,
                dest,
                place: order_place(w, on, behaviour, target, dest),
            }),
            // spec: S-0213 claim 16b (a route-mode click that would order a detonate adds no point: when claim 16d's tests pass, the detonate order is sent at the release, as outside route mode, with no route-mode message)
            Order::Detonate { .. } => out.push(o),
            // spec: S-0213 claim 16b (revision 84, Q-2192 question 3: a click on a cell that sets a factory's rally point sends that order directly, adds no point and takes effect when applied; revision 87: so does a building's *move* on an object, the guard-spot order of S-0214 table M part 3, which then acts as claim 20 says; each meets claim 16d's release-time tests and is sent at the release)
            Order::GuardSpot { actor, .. }
                if w.obj(actor).is_some_and(|x| x.kind == Kind::Building) =>
            {
                out.push(o)
            }
            // spec: S-0213 claim 16b (revision 87: a building's deploy action is not refused in route mode; its primary order, claim 13, when the building is factory-marked, is sent directly with no point, at the release after claim 16d's tests)
            Order::Primary { actor } if w.obj(actor).is_some_and(|x| x.kind == Kind::Building) => {
                out.push(o)
            }
            // spec: S-0213 claim 16b (revision 87: a building is given its deploy action only by a click on the building itself, and each behaviour order *unload* that the action gives names the building itself as its target, so it has the building's own place and is a point add there; its points are issued as claim 16d says) and Changes for other specs (S-0214 table M part 3, deploy row)
            Order::Deploy { actor } if w.obj(actor).is_some_and(|x| x.kind == Kind::Building) => {
                let target = Some(Target::Object(actor));
                out.push(Order::RoutePoint {
                    actor,
                    behaviour: Behaviour::Unload,
                    attack_move: false,
                    target,
                    dest: None,
                    place: target,
                })
            }
            // spec: S-0213 claim 16b (an action that S-0214 gives no order for gives none here either and adds no point; the clicks of S-0214 give no other order kinds)
            _ => {}
        }
    }
    if out.is_empty() {
        // spec: S-0213 claim 16b (the deploy action is refused in route mode: the click gives no order, so it does nothing and meets no test)
        return RouteClick::Ignored;
    }
    // spec: S-0213 claim 16d (the selected objects share one route state: none holds a route, or all hold routes that end at the same point)
    let ends: Vec<Option<u32>> = selection
        .iter()
        .map(|&id| w.routes.routes.get(&id).and_then(Route::last_point))
        .collect();
    let same =
        ends.iter().all(|e| e.is_none()) || ends.iter().all(|e| e.is_some() && *e == ends[0]);
    // spec: S-0213 claim 16 and Formulas (refused when the count plus the selected objects whose routes hold no point would be more than 128)
    let new = selection.iter().filter(|&&id| !holds(w, id)).count();
    let over = u64::from(count(w, house)) + new as u64 > u64::from(HOLDER_LIMIT);
    // spec: S-0213 claim 16d (the same-route-state and holder-limit tests are made at the release; when the selection both mixes route states and would pass the holder limit, only the incompatible-selection message is given)
    if !same {
        return RouteClick::Refused("MSG:PlanningModeHeteroSel");
    }
    if over {
        return RouteClick::Refused("MSG:PlannerMaximum");
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
