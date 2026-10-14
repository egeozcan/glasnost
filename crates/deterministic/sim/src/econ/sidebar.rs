//! Sidebar behaviour (S-0411) that is a rule for every player house in our
//! engine: the orders the interface gives by itself (claim 9, the automatic
//! release; claim 2 with S-0404 claim 9, the automatic cancels), the left
//! and right click tables (claims 4 and 5) as pure functions that both the
//! viewer and our AI use, the icon states (claims 3, 7 and 8) and the
//! legality of production orders (claims 4 and 6, S-0404 claim 10, S-0407
//! claim 13), which `crate::orders::issue` applies. The drawing is the
//! interface's.

use super::TypeRef;
use super::production::{self, Hold, Item, LineKind};
use super::tech::{self, Answer};
use crate::orders::Order;
use crate::rules::Kind;
use crate::world::{HouseId, World};

/// A mouse button on a build icon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Click {
    /// Left click (claim 4).
    Left,
    /// Right click (claim 5).
    Right,
}

/// What a click on a build icon does (claims 4 and 5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClickOutcome {
    /// The order to issue.
    Order(Order),
    /// No order: placement mode starts (claim 4, row 4).
    PlacementMode,
    /// No order: the busy-line refusal (claim 4, row 6; a refusal sound).
    Refused,
    /// No order and nothing else.
    Nothing,
}

/// The icon's item: the line's item when it is of the icon's type (Terms).
fn icon_item(w: &World, house: HouseId, t: TypeRef) -> Option<Item> {
    production::line_of(w, house, t)
        .and_then(|l| l.item.clone())
        .filter(|i| i.t == t)
}

fn is_building_line(w: &World, t: TypeRef) -> bool {
    matches!(
        production::line_kind(w, t),
        Some(LineKind::Building | LineKind::Defence)
    )
}

fn line_has_item(w: &World, house: HouseId, t: TypeRef) -> bool {
    production::line_of(w, house, t).is_some_and(|l| l.item.is_some())
}

/// Whether some factory building of the house could make the type without
/// the tech-tree rule, switched on or not (claim 4, rows 1 and 3); for an
/// aircraft with `quirks.done_aircraft_click_cancels`, only buildings with
/// a free dock count.
fn any_factory(w: &World, house: HouseId, t: TypeRef) -> bool {
    let docks = t.kind == Kind::Aircraft && w.rules().quirks.done_aircraft_click_cancels;
    w.objects.values().any(|b| {
        b.owner == Some(house)
            && tech::could_make(w, b, t)
            // spec: S-0411 Edge cases (quirks.done_aircraft_click_cancels: an aircraft counts only buildings with a free dock)
            && (!docks || production::dock_free(w, b.id))
    })
}

/// The order a click on the icon of type `t` gives (claims 4 and 5). The
/// extra hold order of claim 5 after a pending new-build click in the same
/// cycle is the interface's to add, since the simulation does not see
/// pending clicks.
pub fn click(w: &World, house: HouseId, t: TypeRef, button: Click, shift: bool) -> ClickOutcome {
    let item = icon_item(w, house, t);
    let unit = !is_building_line(w, t);
    match button {
        Click::Left => {
            if let Some(i) = &item {
                if i.done() {
                    // spec: S-0411 claim 4 rows 1 to 4 (a done item: cancel without a factory, else place or placement mode)
                    return if !any_factory(w, house, t) {
                        ClickOutcome::Order(Order::Cancel { what: t })
                    } else if unit {
                        ClickOutcome::Order(Order::Place {
                            what: t,
                            cell: None,
                        })
                    } else {
                        ClickOutcome::PlacementMode
                    };
                }
                if i.hold != Hold::None {
                    // spec: S-0411 claim 4 row 5 (a held item: a build order that resumes it)
                    return ClickOutcome::Order(Order::Build { what: t });
                }
            }
            if !unit && line_has_item(w, house, t) {
                // spec: S-0411 claim 4 row 6 (a building or defence whose line has an item: refused)
                return ClickOutcome::Refused;
            }
            if tech::at_limit(w, house, t) {
                // spec: S-0411 claim 4 row 7 (at its build limit: no order)
                return ClickOutcome::Nothing;
            }
            // spec: S-0411 claim 4 last row (a build order that starts or queues)
            ClickOutcome::Order(Order::Build { what: t })
        }
        Click::Right => {
            let queued = production::line_of(w, house, t).is_some_and(|l| l.queue.contains(&t));
            let cancel = if shift {
                Order::CancelAll { what: t }
            } else {
                Order::Cancel { what: t }
            };
            match &item {
                // spec: S-0411 claim 5 (running or waiting for money: a hold order)
                Some(i) if i.running() => ClickOutcome::Order(Order::Hold { what: t }),
                // spec: S-0411 claim 5 (on hold or done: cancel; with Shift, cancel-all)
                Some(_) => ClickOutcome::Order(cancel),
                // spec: S-0411 claim 5 (only queued copies: cancel; with Shift, cancel-all)
                None if queued => ClickOutcome::Order(cancel),
                None => ClickOutcome::Nothing,
            }
        }
    }
}

/// The legality of a production order when it is issued (S-0404 claim 10,
/// S-0411 claims 4 and 6, S-0407 claim 13), for every house alike
/// (S-0213 claim 3). Orders other than build orders and place orders with a
/// cell have no legality rule of their own.
pub fn legal(w: &World, house: HouseId, order: &Order) -> bool {
    match *order {
        Order::Build { what } => {
            // spec: S-0407 claim 13 (a build order names a type offered to the house)
            if !tech::offered(w, house, what) {
                return false;
            }
            let answer = tech::answer(w, house, what);
            // spec: S-0407 Edge cases (quirks.stale_offer_orderable off: refused when the answer is refused)
            if !w.rules().quirks.stale_offer_orderable && answer == Answer::Refused {
                return false;
            }
            // spec: S-0411 claim 4 Legality (the resume of the held item from its own icon is exempt from the limit and busy-line tests)
            if icon_item(w, house, what).is_some_and(|i| i.hold != Hold::None && !i.done()) {
                return true;
            }
            // spec: S-0407 claim 13 (not at limit when issued)
            if tech::at_limit(w, house, what) {
                return false;
            }
            // spec: S-0411 claim 6 (a building or defence line with an item takes no starting or queuing build order)
            !(is_building_line(w, what) && line_has_item(w, house, what))
        }
        Order::Place {
            what,
            cell: Some(c),
        } => {
            // spec: S-0404 claim 10 (a place order with a cell: adjacency and explored ground, S-0406 claim 6)
            super::placement::place_legal(w, house, what, c)
        }
        _ => true,
    }
}

/// S-0411 claim 2 with S-0404 claim 9, in the house update of a recheck of
/// every player house (`crate::econ::tech::house_update`): for each type the
/// recheck removed from the taken-up set (its icon goes with it), a cancel
/// order when it is its line's item, then a cancel-all order when the house
/// has a line of its kind. The orders are applied at the tick's order
/// boundary (S-0203 claim 7). The types come in tab order and, within a
/// tab, in icon order (S-0411 claim 13).
pub fn removal_orders(w: &mut World, house: HouseId, removed: &[TypeRef]) {
    // spec: S-0404 claim 9 (for every player house; the non-player houses never build)
    if !production::player(w, house) {
        return;
    }
    // spec: S-0411 claim 13 (the types in the order the recheck gives: tab, then icon order; each type's orders before the next type's)
    for &t in removed {
        let line = production::line_of(w, house, t);
        let item = line.is_some_and(|l| l.item.as_ref().is_some_and(|i| i.t == t));
        let exists = line.is_some();
        // spec: S-0411 claim 2 (the cancel order first, then the cancel-all order)
        if item {
            crate::orders::issue(w, house, Order::Cancel { what: t });
        }
        if exists {
            crate::orders::issue(w, house, Order::CancelAll { what: t });
        }
    }
}

/// S-0411 claim 9: the automatic place order of a unit item that became
/// done, issued in the tick after the done tick, for every player house
/// (`quirks.release_order_once`; off: in every tick while it waits).
pub fn release_orders(w: &mut World, house: HouseId) {
    if !production::player(w, house) {
        return;
    }
    let tick = w.tick;
    let once = w.rules().quirks.release_order_once;
    let due: Vec<TypeRef> = w
        .econ
        .lines
        .lines
        .iter()
        .filter(|l| l.house == house && l.kind.unit())
        .filter_map(|l| l.item.as_ref())
        .filter(|i| {
            // spec: S-0411 claim 9 (in tick n + 1 for an item done in tick n; quirks.release_order_once)
            i.done_tick
                .is_some_and(|d| if once { tick == d + 1 } else { tick > d })
        })
        .map(|i| i.t)
        .collect();
    for t in due {
        crate::orders::issue(
            w,
            house,
            Order::Place {
                what: t,
                cell: None,
            },
        );
    }
}

/// The queue count an icon shows (claim 7): the copies of its type in its
/// line, shown only while at least one copy is queued.
pub fn queue_count(w: &World, house: HouseId, t: TypeRef) -> Option<i32> {
    let l = production::line_of(w, house, t)?;
    let q = l.queue.iter().filter(|&&x| x == t).count() as i32;
    // spec: S-0411 Formulas (shown count = q + i, shown exactly when q ≥ 1)
    (q >= 1).then(|| q + i32::from(l.item.as_ref().is_some_and(|i| i.t == t)))
}

/// The state display of the icon that holds its line's item (claim 8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IconState {
    /// The clock stage, progress + 1 (1 to 55).
    pub stage: i32,
    /// The *ready* label.
    pub ready: bool,
    /// The *on hold* label.
    pub on_hold: bool,
}

/// The state display of an icon, when its line's item is of its type
/// (claim 8).
pub fn icon_state(w: &World, house: HouseId, t: TypeRef) -> Option<IconState> {
    let i = icon_item(w, house, t)?;
    Some(IconState {
        // spec: S-0411 Formulas (clock stage = progress + 1)
        stage: i.progress + 1,
        ready: i.done(),
        on_hold: i.hold != Hold::None && !i.done(),
    })
}

/// Whether an icon is drawn darkened (claim 3): never the icon of its
/// line's item; otherwise a building or defence whose line has an item, a
/// type no switched-on factory building could make, or a type at limit.
pub fn darkened(w: &World, house: HouseId, t: TypeRef) -> bool {
    if icon_item(w, house, t).is_some() {
        return false;
    }
    // spec: S-0411 claim 3 (no switched-on factory building could make the type with the full tech-tree condition left out, the build-limit test retained)
    let switched_on = w.objects.values().any(|b| {
        b.owner == Some(house)
            && !b.econ.building.switched_off
            && tech::could_make_limit_only(w, b, t)
    });
    // spec: S-0411 claim 3 (the three causes of a darkened icon)
    (is_building_line(w, t) && line_has_item(w, house, t))
        || !switched_on
        || tech::at_limit(w, house, t)
}

/// The house's icons (claim 2): its taken-up set, the types offered at the
/// last recheck (S-0407 Terms), in tab order and then icon order (S-0004
/// claim 11).
pub fn icons(w: &World, house: HouseId) -> Vec<TypeRef> {
    w.houses
        .get(usize::from(house.0))
        .map(|h| h.econ.tech.icons.iter().flatten().map(|i| i.t).collect())
        .unwrap_or_default()
}
