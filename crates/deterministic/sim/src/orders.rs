//! Orders (S-0213): the single command interface for human players and our
//! AI houses, the pending lists, and how orders apply at a tick boundary.

use glasnost_map::CellXY;
use glasnost_num::F64;
use glasnost_num::geom::{root_floor, table_root};
use serde::{Deserialize, Serialize};

use crate::econ::TypeRef;
use crate::occupancy::{self, Outcome};
use crate::regions::{self, Reach};
use crate::rules::{Behaviour, Kind};
use crate::scatter::{self, ScatterRequest};
use crate::world::{AttackMoveMemory, HouseId, Layer, ObjectId, Pos, Target, World};

/// An order (S-0213 table T1). House orders of other topics (production,
/// superweapons, alliances, cheering) are added with those topics.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Order {
    /// A behaviour order (S-0213 claim 5).
    Behaviour {
        /// The actor.
        actor: ObjectId,
        /// The behaviour.
        behaviour: Behaviour,
        /// An attack-move order (S-0213 claim 6); `behaviour` is then
        /// ignored.
        attack_move: bool,
        /// Target.
        target: Option<Target>,
        /// Destination.
        dest: Option<Target>,
    },
    /// Stop (S-0213 claim 8).
    Stop {
        /// The actor.
        actor: ObjectId,
    },
    /// Scatter (S-0213 claim 9).
    Scatter {
        /// The actor.
        actor: ObjectId,
    },
    /// Deploy (S-0213 claim 10).
    Deploy {
        /// The actor.
        actor: ObjectId,
    },
    /// Guard spot (S-0213 claim 20).
    GuardSpot {
        /// The actor.
        actor: ObjectId,
        /// The guard spot.
        spot: Target,
    },
    /// Cheer: all of the issuing house's infantry cheer (S-0213 table T1,
    /// a house order with no actor).
    Cheer,
    /// Build: start or queue an item of a type (S-0213 claim 14, S-0404
    /// claim 3).
    Build {
        /// The type.
        what: TypeRef,
    },
    /// Hold the line of a type (S-0404 claim 5).
    Hold {
        /// The type.
        what: TypeRef,
    },
    /// Cancel one item or queued copy of a type (S-0404 claim 6).
    Cancel {
        /// The type.
        what: TypeRef,
    },
    /// Cancel every item and queued copy of a type (S-0404 claim 6).
    CancelAll {
        /// The type.
        what: TypeRef,
    },
    /// Place a done item (S-0404 claim 7): a building at a cell (S-0406),
    /// a unit without one (its release).
    Place {
        /// The type.
        what: TypeRef,
        /// The cell a building's placement names.
        cell: Option<CellXY>,
    },
    /// Sell (S-0213 claim 12).
    Sell {
        /// The actor.
        actor: ObjectId,
    },
    /// Sell the wall in a cell (S-0213 claim 12, S-0408 claim 12).
    SellWall {
        /// The cell.
        cell: CellXY,
    },
    /// Repair switch (S-0213 claim 13).
    Repair {
        /// The actor.
        actor: ObjectId,
    },
    /// Power off (S-0213 claim 13, S-0405 claim 10).
    PowerOff {
        /// The actor.
        actor: ObjectId,
    },
    /// Power on (S-0213 claim 13, S-0405 claim 10).
    PowerOn {
        /// The actor.
        actor: ObjectId,
    },
    /// Primary factory switch (S-0213 claim 13).
    Primary {
        /// The actor.
        actor: ObjectId,
    },
    /// Surrender (S-0414 claim 11).
    Surrender,
    /// The alliance order towards a house (S-0414 claims 12 to 14).
    Alliance {
        /// The other house.
        house: HouseId,
    },
    /// Detonate (S-0213 claim 11): a planted bomb the actor carries goes off.
    Detonate {
        /// The actor.
        actor: ObjectId,
    },
    /// A superweapon fire order (S-0213 orders table; S-0512 claim 8,
    /// S-0513 claim 2): the type by its position in the superweapon type
    /// list, and the target cell.
    FireSuperweapon {
        /// The superweapon type, by list position.
        sw: u32,
        /// The target cell.
        cell: CellXY,
    },
}

impl Order {
    /// The actor the order names; house orders have none.
    pub fn actor(&self) -> Option<ObjectId> {
        match *self {
            Order::Behaviour { actor, .. }
            | Order::Stop { actor }
            | Order::Scatter { actor }
            | Order::Deploy { actor }
            | Order::GuardSpot { actor, .. }
            | Order::Sell { actor }
            | Order::Repair { actor }
            | Order::PowerOff { actor }
            | Order::PowerOn { actor }
            | Order::Primary { actor }
            | Order::Detonate { actor } => Some(actor),
            Order::Cheer
            | Order::Build { .. }
            | Order::Hold { .. }
            | Order::Cancel { .. }
            | Order::CancelAll { .. }
            | Order::Place { .. }
            | Order::SellWall { .. }
            | Order::Surrender
            | Order::FireSuperweapon { .. }
            | Order::Alliance { .. } => None,
        }
    }
}

/// The smallest run of orders that is spread (S-0213 claim 7).
// spec: S-0213 Constants (smallest spread group: 2 orders)
const MIN_RUN: usize = 2;

/// Pending-list capacity (S-0213 claim 2).
// spec: S-0213 claim 2 (at most 128 pending orders per house)
pub const PENDING_LIMIT: usize = 128;

/// Whether orders are blocked in the cycle of the current tick (S-0213
/// claim 1, S-0201 claim 14).
pub fn blocked(w: &World) -> bool {
    // spec: S-0213 claim 1 and S-0201 claim 14 (the block covers the cycles of ticks up to its last blocked tick)
    w.tick <= w.orders_blocked_through
}

/// Issues an order in a house's name during the current tick. Returns
/// whether it was taken (S-0213 claims 1 and 2).
pub fn issue(w: &mut World, house: HouseId, order: Order) -> bool {
    // spec: S-0213 claim 11 and quirks.detonate_in_planter_name (the detonate order is issued in the name of the planter's house; off: in the issuing house's name)
    let house = match order {
        Order::Detonate { actor } if w.rules().quirks.detonate_in_planter_name => w
            .obj(actor)
            .and_then(|o| o.combat.bomb)
            .and_then(|b| b.house)
            .unwrap_or(house),
        _ => house,
    };
    // spec: S-0213 claim 1 (an order block refuses input)
    if blocked(w) {
        return false;
    }
    // spec: S-0213 claim 3 and S-0404 claim 10 (a production order is judged when it is issued)
    if !crate::econ::sidebar::legal(w, house, &order) {
        return false;
    }
    let limit = w.rules().quirks.pending_order_limit;
    let Some(h) = w.houses.get_mut(usize::from(house.0)) else {
        return false;
    };
    // spec: S-0213 claim 2 (an order issued while the list is full is dropped; quirks.pending_order_limit)
    if limit && h.pending.len() >= PENDING_LIMIT {
        return false;
    }
    h.pending.push(order);
    true
}

/// Applies every house's pending orders (S-0213 claims 1 and 3): houses
/// in house order; per house the orders that are not behaviour orders in
/// issue order, then the behaviour orders in issue order.
pub fn apply_pending(w: &mut World) {
    for hi in 0..w.houses.len() {
        let orders = std::mem::take(&mut w.houses[hi].pending);
        // spec: S-0213 claim 3 (non-behaviour orders first, in issue order)
        for o in orders
            .iter()
            .filter(|o| !matches!(o, Order::Behaviour { .. }))
        {
            apply_other(w, HouseId(hi as u8), o);
            // spec: S-0519 claim 19 (revision 3: a designator's change of target cancels its strike when the change applies, in the tick of the order)
            crate::strike::sweep(w);
        }
        // spec: S-0213 claim 3 (then the behaviour orders, in issue order)
        let behaviour: Vec<Order> = orders
            .into_iter()
            .filter(|o| matches!(o, Order::Behaviour { .. }))
            .collect();
        let dests = group_destinations(w, &behaviour);
        for (o, d) in behaviour.iter().zip(dests) {
            apply_behaviour(w, o, d);
            // spec: S-0519 claim 19 (revision 3: cancellation when the change applies)
            crate::strike::sweep(w);
        }
    }
}

/// Live and on the map (S-0213 Terms).
fn live_on_map(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.on_map)
}

/// Valid: live, on the map, more than zero hit points (S-0213 Terms).
fn valid(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.on_map && o.health > 0)
}

/// Claim 4: a named target or destination that is a unit or building must
/// be valid; cells and other objects are not checked.
fn target_ok(w: &World, t: Option<Target>) -> bool {
    match t {
        Some(Target::Object(x)) => match w.obj(x) {
            Some(o) if o.kind != Kind::Terrain => valid(w, x),
            Some(_) => true,
            // A deleted object fails (it is no longer live).
            None => false,
        },
        _ => true,
    }
}

fn apply_other(w: &mut World, house: HouseId, o: &Order) {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    match *o {
        Order::Detonate { actor } => {
            // spec: S-0213 claim 11 (if the actor exists and carries a planted bomb, it is resolved at once: no other check is made)
            if w.obj(actor).is_some_and(|x| x.combat.bomb.is_some()) {
                crate::bomb::detonate_order(w, actor);
            }
        }
        Order::Stop { actor } => {
            // spec: S-0213 claim 8 (live, on the map and not tethered; construction and tunnels do not arise yet)
            if !live_on_map(w, actor) || crate::econ::tethered(w, actor) {
                return;
            }
            // spec: S-0314 claims 20 and 23 (an approaching vehicle given another order loses its depot contact; any order clears a queued entry)
            crate::depot::stop_order(w, actor);
            let t = w.tick;
            let harvester = w
                .obj(actor)
                .and_then(|ob| w.rules().unit(ob.kind, ob.type_index))
                .is_some_and(|u| u.harvester);
            let kind = w.obj(actor).map(|ob| ob.kind);
            if kind == Some(Kind::Vehicle) {
                crate::hover::stop_order(w, actor);
                // spec: S-0503 claim 16 (a hover whose D equals its heading point keeps its stored path)
                if !crate::hover::keeps_path(w, actor) {
                    crate::drive::stop(w, actor);
                }
                if crate::econ::slaves::is_master(w, actor)
                    && !w.obj(actor).is_some_and(|ob| {
                        matches!(
                            crate::econ::placement::seen_behaviour(ob),
                            Some(Behaviour::Construction | Behaviour::Selling)
                        )
                    })
                {
                    // spec: S-0213 claim 8 (a vehicle that controls slave units, whatever its behaviour: its slaves are sent back to guard it; the slave-recall row applies only to vehicles, not to a deployed slave-miner building)
                    crate::econ::slaves::recall(w, actor);
                }
            }
            if w.obj(actor).is_some_and(|ob| {
                matches!(
                    crate::econ::placement::seen_behaviour(ob),
                    Some(Behaviour::Construction | Behaviour::Selling)
                )
            }) {
                // spec: S-0215 claim 12 (the stop order does nothing in construction or selling)
                return;
            }
            // spec: S-0213 claim 8 and S-0502 claim 28 (stop requests the destination clear while the target is still present, so a balloon-hover vehicle can reject it; the target is cleared after)
            let first_rejected = crate::jump::clear_rejected(w, actor);
            if let Some(ob) = w.obj_mut(actor) {
                // spec: S-0213 claim 8 (target and destination become nothing; contact broken off)
                // spec: S-0215 claim 12 and S-0506 claim 25 (an order's own-target clear obeys the spawner pending-target rule)
                crate::spawn::explicit_clear_own_target(ob);
                ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
                if !first_rejected {
                    ob.dest = None;
                }
                ob.contact = None;
                // spec: S-0213 claim 8 (an open-topped transport: its passengers lose their targets too)
                let passengers = ob.transport.passengers.clone();
                if ob.kind.is_mobile() {
                    // spec: S-0213 claim 8 (a mobile actor has no queued destinations left)
                    ob.queued_dests.clear();
                    // spec: S-0503 claim 16 (a hover's stored path is left when its D equals its heading point)
                    if !ob.hover.as_ref().is_some_and(|h| h.retained.is_some()) {
                        ob.path.steps.clear();
                    }
                }
                if harvester
                    && matches!(ob.beh.seen(), Some(Behaviour::Harvest | Behaviour::Return))
                {
                    // spec: S-0213 claim 8 (a harvester in harvest or return gets guard at once)
                    ob.beh.queue_and_start_at_once(Behaviour::Guard, t, stale);
                }
                if !passengers.is_empty() {
                    crate::transport::clear_passenger_targets(w, actor);
                }
            }
            // spec: S-0213 claim 8 and S-0502 claims 17 and 24 (a jumpjet with a movement commitment is rerouted by the stop; the order-level destination stays empty)
            if !first_rejected {
                crate::jump::stop_ordered(w, actor);
            }
            // spec: S-0213 claim 8 and S-0502 claim 28 (a balloon-hover type requests the clear again after the target is cleared: that request cannot be rejected, so stop ends with an empty destination; after an effective first clear a vehicle's second, unchanged empty assignment is suppressed, an infantry unit's reroutes again; the rule is per assignment, not one reroute per stop)
            if crate::jump::balloon_hover_type(w, actor) {
                if let Some(ob) = w.obj_mut(actor) {
                    ob.dest = None;
                }
                crate::jump::stop_ordered(w, actor);
            }
        }
        Order::Scatter { actor } => {
            // spec: S-0213 claim 9 (a mobile actor, live, on the map and not tethered: no threat point, forced, not move-anywhere)
            if !live_on_map(w, actor)
                || !w.obj(actor).is_some_and(|ob| ob.kind.is_mobile())
                || crate::econ::tethered(w, actor)
            {
                return;
            }
            scatter::request(
                w,
                actor,
                ScatterRequest {
                    threat: None,
                    forced: true,
                    anywhere: false,
                },
            );
        }
        Order::Deploy { actor } => {
            // spec: S-0213 claim 10 (live, on the map, not tethered, not an aircraft)
            if !live_on_map(w, actor)
                || crate::econ::tethered(w, actor)
                || w.obj(actor).is_some_and(|ob| ob.kind == Kind::Aircraft)
            {
                return;
            }
            if on_vehicle_factory(w, actor) {
                // spec: S-0213 claim 10 (an actor on a vehicle-factory cell ignores the order)
                return;
            }
            // spec: S-0213 claim 10 and S-0502 claim 28 (deploy makes one destination-clear request, while the target is still present, and no retry after the target is cleared: a balloon-hover vehicle can keep a nonempty destination and end with no target)
            let rejected = crate::jump::clear_rejected(w, actor);
            if let Some(ob) = w.obj_mut(actor) {
                // spec: S-0213 claim 10 (contact broken off, target and destination nothing, unload queued)
                ob.contact = None;
                // spec: S-0215 claim 12 and S-0506 claim 25 (an order's own-target clear obeys the spawner pending-target rule)
                crate::spawn::explicit_clear_own_target(ob);
                ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
                if !rejected {
                    ob.dest = None;
                }
                ob.beh.queue(Some(Behaviour::Unload));
                // spec: S-0415 claim 2 (each deploy order begins a new deploy: its first attempt is still to come)
                // The in-place form (S-0517) is a state of the vehicle, not of the order: a deploy order to a deployed vehicle packs it up (claim 3).
                let form = ob.econ.deploy.form;
                ob.econ.deploy = crate::econ::deploy::DeployState::default();
                ob.econ.deploy.form = form;
            }
            // spec: S-0213 claim 10 and S-0502 claim 24 (the deploy order's destination clear reroutes a jumpjet that retains a movement commitment, as the stop order's does; a suppressed unchanged vehicle assignment keeps the preceding point and commitment)
            if !rejected {
                crate::jump::destination_cleared(w, actor);
            }
        }
        Order::GuardSpot { actor, spot } => {
            // spec: S-0213 claim 20 (needs only a live actor; sets its guard spot)
            if let Some(ob) = w.obj_mut(actor) {
                ob.guard_spot = Some(spot);
            }
        }
        Order::Cheer => {
            // spec: S-0213 table T1 (cheer: all of the issuer's infantry cheer, S-0107's cheer sequence)
            let infantry: Vec<ObjectId> = w
                .objects
                .values()
                .filter(|o| o.kind == Kind::Infantry && o.owner == Some(house) && o.on_map)
                .map(|o| o.id)
                .collect();
            for id in infantry {
                crate::infantry::request(w, id, crate::infantry::Seq::Cheer, false);
            }
        }
        Order::Build { what } => crate::econ::production::build(w, house, what),
        Order::Hold { what } => crate::econ::production::hold(w, house, what),
        Order::Cancel { what } => crate::econ::production::cancel(w, house, what, false),
        Order::CancelAll { what } => crate::econ::production::cancel(w, house, what, true),
        Order::Place { what, cell } => crate::econ::production::place(w, house, what, cell),
        Order::Sell { actor } => crate::econ::sell::sell(w, house, actor),
        Order::SellWall { cell } => crate::econ::sell::sell_wall(w, house, cell),
        Order::Repair { actor } => crate::econ::sell::repair(w, actor),
        Order::PowerOff { actor } => crate::econ::power::switch(w, actor, false),
        Order::PowerOn { actor } => crate::econ::power::switch(w, actor, true),
        Order::Primary { actor } => primary(w, actor),
        Order::Surrender => crate::econ::victory::surrender(w, house),
        Order::Alliance { house: other } => crate::econ::victory::alliance(w, house, other),
        Order::FireSuperweapon { sw, cell } => {
            crate::superweapons::launch::fire(w, house, sw, cell)
        }
        Order::Behaviour { .. } => {}
    }
}

/// The primary order (S-0213 claim 13): a live building with a factory
/// category (producing or not), on the map or not, toggles its primary
/// mark; becoming primary takes the mark from every other building of its
/// house on the map with exactly the same category and of the same naval
/// or land kind. No owner check (claim 15).
fn primary(w: &mut World, actor: ObjectId) {
    let Some(o) = w.obj(actor) else { return };
    if o.kind != Kind::Building || o.health <= 0 {
        return;
    }
    let r = w.rules_arc();
    // spec: S-0213 claim 13 (the imported factory category, the four producing kinds and the other recognised categories; the no-factory value is not changed)
    let Some(factory) = r.buildings[o.type_index as usize].factory_category else {
        return;
    };
    let naval = r.buildings[o.type_index as usize].naval;
    let owner = o.owner;
    if o.econ.building.primary {
        // spec: S-0213 claim 13 (already primary: it stops being primary)
        if let Some(ob) = w.obj_mut(actor) {
            ob.econ.building.primary = false;
        }
        return;
    }
    // spec: S-0213 claim 13 (every other on-map building of the house with exactly the same category and naval kind stops being primary)
    let others: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|b| {
            b.id != actor
                && b.kind == Kind::Building
                && b.on_map
                && b.owner == owner
                && r.buildings[b.type_index as usize].factory_category == Some(factory)
                && r.buildings[b.type_index as usize].naval == naval
        })
        .map(|b| b.id)
        .collect();
    for b in others {
        if let Some(ob) = w.obj_mut(b) {
            ob.econ.building.primary = false;
        }
    }
    if let Some(ob) = w.obj_mut(actor) {
        ob.econ.building.primary = true;
    }
    // spec: S-0522 claim 5 (the computed edge is computed again each time a building factory of the house is marked primary)
    if factory == crate::rules::FactoryCategory::Produces(crate::rules::FactoryKind::Building) {
        crate::air::recompute_edge(w, actor);
    }
}

/// Whether the actor stands on a cell occupied by a vehicle-factory
/// building (S-0213 claim 10).
fn on_vehicle_factory(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let c = o.pos.cell();
    w.cells.list(c, Layer::Ground).iter().any(|&x| {
        w.obj(x).is_some_and(|b| {
            b.kind == Kind::Building
                && w.rules().buildings[b.type_index as usize]
                    .raw
                    .weapons_factory
                    .unwrap_or(false)
        })
    })
}

/// Attack-move eligibility (S-0214 claim 5): infantry or a vehicle with a
/// weapon in its first slot and without the prevent-attack-move flag.
fn eligible(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if !matches!(o.kind, Kind::Infantry | Kind::Vehicle) {
        return false;
    }
    // spec: S-0214 claim 5 (a weapon in its first slot, no prevent-attack-move flag)
    w.rules()
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.primary.is_some() && !u.raw.prevent_attack_move.unwrap_or(false))
}

fn apply_behaviour(w: &mut World, o: &Order, dest_override: Option<Target>) {
    let Order::Behaviour {
        actor,
        behaviour,
        attack_move,
        target,
        dest,
    } = *o
    else {
        return;
    };
    let dest = dest_override.or(dest);
    // spec: S-0213 claim 4 (actor valid; a named unit or building target or destination valid)
    if !valid(w, actor) || !target_ok(w, target) || !target_ok(w, dest) {
        return;
    }
    let Some(ob) = w.obj(actor).cloned() else {
        return;
    };
    let mobile = ob.kind.is_mobile();
    // spec: S-0213 claim 6 (the attack-move translation, decided at application)
    let (b, memory) = if attack_move {
        if !mobile {
            (Behaviour::AttackMove, None)
        } else if eligible(w, actor) {
            match (target, dest) {
                (Some(t), None) => (Behaviour::Attack, Some(Some(AttackMoveMemory::Target(t)))),
                (_, Some(d)) => (Behaviour::Move, Some(Some(AttackMoveMemory::Dest(d)))),
                (None, None) => (Behaviour::AttackMove, Some(None)),
            }
        } else {
            match target {
                Some(_) => (Behaviour::Attack, None),
                None => (Behaviour::Move, None),
            }
        }
    } else {
        // spec: S-0213 claim 6 (any other behaviour order removes the attack-move memory of a mobile actor)
        (behaviour, if mobile { Some(None) } else { None })
    };
    // spec: S-0213 table T3 (a) (enter while entering, towards a building it is in contact with: ignored)
    if b == Behaviour::Enter
        && mobile
        && ob.beh.seen() == Some(Behaviour::Enter)
        && let Some(Target::Object(d)) = dest
        && w.obj(d).is_some_and(|x| x.kind == Kind::Building)
        && ob.contact == Some(d)
    {
        return;
    }
    // spec: S-0314 claims 20 and 23 (an order to a vehicle in depot contact, or an enter order on a depot that holds a contact)
    if crate::depot::order_applied(w, actor, b, dest) {
        return;
    }
    // spec: S-0213 table T3 (b) (a tethered actor keeps its link, unless it is in contact with a building of the unloading kind)
    let keeps = crate::econ::tethered(w, actor)
        && !w
            .obj(actor)
            .and_then(|m| m.contact)
            .and_then(|c| w.obj(c))
            .filter(|c| c.kind == Kind::Building)
            .and_then(|c| w.rules().buildings.get(c.type_index as usize))
            .is_some_and(|b| b.raw.econ.dock_unload.unwrap_or(false));
    if b != Behaviour::Attack
        && ob.kind == Kind::Vehicle
        && crate::econ::slaves::is_master(w, actor)
    {
        // spec: S-0213 table T3 (h) and claim 5 (a mobile actor controlling slave units, resulting behaviour not attack: its slaves are sent back to guard it; buildings, a deployed slave miner included, are excluded; the mobile masters modelled are vehicles)
        crate::econ::slaves::recall(w, actor);
    }
    // spec: S-0213 table T3 (b) and S-0605 claim 4 (the actor forgets any building it was waiting in line to enter)
    crate::structures::absorb::order_applied(w, actor);
    let mut retained = false;
    let Some(m) = w.obj_mut(actor) else { return };
    if !keeps {
        // spec: S-0213 table T3 (b) (otherwise it breaks off any contact)
        m.contact = None;
    }
    // spec: S-0213 table T3 (d) (the resulting behaviour is queued, S-0215 claim 12)
    m.beh.queue(Some(b));
    // spec: S-0215 claim 12 (every such order forgets what an override put aside)
    m.beh.aside = None;
    if let Some(mem) = memory {
        m.attack_move = mem;
        // spec: S-0309 claim 23 (c) (a new attack-move order, or the memory removed, ends the engaged mark)
        m.combat.engaged = false;
    }
    if b == Behaviour::AreaGuard && mobile {
        // spec: S-0213 table T3 (e) (area guard: target nothing; destination and guard spot the order's target)
        // spec: S-0215 claim 12 and S-0506 claim 25 (an order's own-target clear obeys the spawner pending-target rule)
        crate::spawn::explicit_clear_own_target(m);
        m.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
        m.dest = target;
        m.guard_spot = target;
    } else {
        // spec: S-0213 table T3 (f) (target and destination from the order)
        match target {
            Some(_) => m.target = target,
            // spec: S-0215 claim 12 and S-0506 claim 25 (an order setting the target to nothing obeys the spawner pending-target rule)
            None => crate::spawn::explicit_clear_own_target(m),
        }
        // spec: S-0213 claim 21 and S-0502 claim 28 (an order installs its target before it requests the empty destination, so a balloon-hover vehicle with a nonempty destination can reject that clear and keep its destination, point, commitment and reservation)
        retained = mobile && dest.is_none() && crate::jump::clear_rejected(w, actor);
        let Some(m) = w.obj_mut(actor) else { return };
        if !retained {
            m.dest = dest;
        }
        if mobile {
            // spec: S-0213 table T3 (g) (a mobile actor forgets its guard spot and queued destinations)
            m.guard_spot = None;
            m.queued_dests.clear();
        }
    }
    if mobile && !retained {
        // A new destination starts a new path search (S-0212 claim 20).
        let Some(m) = w.obj_mut(actor) else { return };
        m.path.steps.clear();
        m.path.retry_at = 0;
    }
    // spec: S-0213 claim 21 (the destination assignment of the applied order decides the prone get-up, S-0107 claim 15)
    crate::infantry::destination_assigned(w, actor, ob.dest);
    // spec: S-0314 claim 23 (an enter order on a depot that holds no contact asks for entry at once)
    crate::depot::order_entered(w, actor, b, dest);
    // spec: S-0605 claim 3 (the destination assignment of an enter order on an absorbing building, after the order's other effects: contact, or waiting in line)
    crate::structures::absorb::order_entered(w, actor, b, dest);
    // spec: S-0213 table T3 (f) (an open-topped transport gives the same target to its passengers when the target is not nothing)
    crate::transport::hand_down_target(w, actor);
}

/// Group destinations (S-0213 claim 7). Returns, per behaviour order, the
/// destination that replaces the order's own (`None`: keep it).
fn group_destinations(w: &mut World, orders: &[Order]) -> Vec<Option<Target>> {
    let mut out = vec![None; orders.len()];
    let key = |o: &Order| match *o {
        Order::Behaviour {
            behaviour,
            attack_move,
            target,
            dest,
            ..
        } => Some((behaviour, attack_move, target, dest)),
        _ => None,
    };
    let mut i = 0;
    while i < orders.len() {
        let k = key(&orders[i]);
        let mut j = i + 1;
        // spec: S-0213 claim 7 (a run: consecutive orders with the same behaviour, target and destination, as issued)
        while j < orders.len() && key(&orders[j]) == k {
            j += 1;
        }
        if let Some((_, _, _, Some(Target::Cell(common)))) = k
            && j - i >= MIN_RUN
        {
            let run: Vec<(usize, ObjectId)> = (i..j)
                .filter_map(|n| orders[n].actor().map(|a| (n, a)))
                .collect();
            for (n, c) in spread(w, &run, common) {
                out[n] = Some(Target::Cell(c));
            }
        }
        i = j;
    }
    out
}

/// The distance d(a, b) of S-0213 Formulas (table root, truncated).
fn dist(w: &World, a: Pos, b: Pos) -> u64 {
    let dx = i64::from(a.x - b.x);
    let dy = i64::from(a.y - b.y);
    let dz = i64::from(a.z - b.z);
    // spec: S-0213 Formulas (d = trunc(√̃(Δx² + Δy² + Δz²)), the approximate root of S-0204)
    root_floor(
        (dx * dx + dy * dy + dz * dz) as u64,
        w.rules().quirks.table_root_distance,
        w.rules().mode,
    )
}

/// Spreads one run over cells (S-0213 claim 7 with our engine's interim
/// rules). Returns (order index, cell) for every member placed.
fn spread(w: &mut World, run: &[(usize, ObjectId)], common: CellXY) -> Vec<(usize, CellXY)> {
    let mode = w.rules().mode;
    // spec: S-0213 claim 7 (members: the mobile actors of the run; the spreading checks no validity)
    let members: Vec<(usize, ObjectId, Pos)> = run
        .iter()
        .filter_map(|&(n, id)| {
            w.obj(id)
                .filter(|o| o.kind.is_mobile())
                .map(|o| (n, id, o.pos))
        })
        .collect();
    if members.is_empty() {
        return Vec::new();
    }
    let cnt = members.len() as i64;
    let sum = |f: fn(&Pos) -> i32| members.iter().map(|m| i64::from(f(&m.2))).sum::<i64>();
    // spec: S-0213 Formulas (centre: per-axis integer sums divided by n, truncated toward zero)
    let centre = Pos::new(
        (sum(|p| p.x) / cnt) as i32,
        (sum(|p| p.y) / cnt) as i32,
        (sum(|p| p.z) / cnt) as i32,
    );
    // spec: S-0213 claim 7 (leader: smallest distance to the centre, the latest in the run on ties)
    let mut leader = 0;
    for (k, m) in members.iter().enumerate() {
        if dist(w, m.2, centre) <= dist(w, members[leader].2, centre) {
            leader = k;
        }
    }
    let lpos = members[leader].2;
    let lcell = lpos.cell();
    // spec: S-0213 claim 7 (others placed by increasing distance from the leader, ties in run order)
    let mut others: Vec<usize> = (0..members.len()).filter(|&k| k != leader).collect();
    others.sort_by_key(|&k| (dist(w, members[k].2, lpos), k));
    let mut given: Vec<CellXY> = Vec::new();
    let mut done: Vec<bool> = vec![false; members.len()];
    let mut out = Vec::new();
    let common_level = level_of(w, common);
    for &k in &others {
        if done[k] {
            continue;
        }
        let (n, id, p) = members[k];
        let mcell = p.cell();
        let zone = w
            .obj(id)
            .and_then(|o| w.rules().unit(o.kind, o.type_index))
            .map(|u| u.zone);
        let common_reach = zone.map(|z| regions::reach(w, common, z));
        let vx = mcell.x - lcell.x;
        let vy = mcell.y - lcell.y;
        // spec: S-0213 Formulas (u = v / √̃(vx² + vy²); u = (0, 0) when v = 0, interim rule)
        let (ux, uy) = if (vx, vy) == (0, 0) {
            (F64::ZERO, F64::ZERO)
        } else {
            let r = table_root(
                (i64::from(vx) * i64::from(vx) + i64::from(vy) * i64::from(vy)) as u64,
                mode,
            )
            .to_f64();
            (
                F64::from_i64(i64::from(vx), mode).div(r, mode),
                F64::from_i64(i64::from(vy), mode).div(r, mode),
            )
        };
        let half = crate::consts::HALF;
        let reference = occupancy::reference_level(w, id);
        let mut fallback: Option<CellXY> = None;
        let mut result: Option<CellXY> = None;
        let mut spoiled = false;
        // spec: S-0213 claim 7 (interim: j = 1 to 6, each step one cell length from the common cell's centre)
        for j in 1..=6 {
            let jj = F64::from_i64(j, mode);
            let qx = F64::from_i64(i64::from(common.x), mode)
                .add(half, mode)
                .add(jj.mul(ux, mode), mode);
            let qy = F64::from_i64(i64::from(common.y), mode)
                .add(half, mode)
                .add(jj.mul(uy, mode), mode);
            // spec: S-0213 Formulas (each point's cell by truncation toward zero, S-0205 claim 7)
            let c = CellXY::new(qx.to_i32_common(), qy.to_i32_common());
            // spec: S-0213 claim 7 (the classes of a look cell, the first that fits)
            let class = if !w.cells.usable(c) {
                Class::OffArea
            } else if zone.is_some_and(|z| Some(regions::reach(w, c, z)) != common_reach)
                || matches!(common_reach, Some(Reach::OffMap | Reach::None))
            {
                Class::OtherRegion
            } else if given.contains(&c) {
                Class::Taken
            } else if (level_of(w, c) - common_level).abs() >= 3 {
                Class::TooSteep
            } else if obstructed(w, id, c, reference) {
                Class::Obstructed
            } else {
                Class::Open
            };
            match class {
                // spec: S-0213 claim 7 (an off-area or too steep end gives the fallback cell)
                Class::OffArea | Class::TooSteep => break,
                Class::Open => {
                    if !spoiled {
                        result = Some(c);
                    }
                    break;
                }
                Class::Taken => fallback = Some(c),
                Class::OtherRegion | Class::Obstructed => spoiled = true,
            }
        }
        // spec: S-0213 claim 7 (a look that is not spoiled and ends open: the end cell; otherwise the fallback cell)
        let cell = result.unwrap_or(fallback.unwrap_or(common));
        given.push(cell);
        done[k] = true;
        out.push((n, cell));
        // spec: S-0213 claim 7 (shared cells: unplaced members in the same cell get the same destination)
        for &k2 in &others {
            if !done[k2] && members[k2].2.cell() == mcell {
                done[k2] = true;
                out.push((members[k2].0, cell));
            }
        }
    }
    out
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Class {
    OffArea,
    OtherRegion,
    Taken,
    TooSteep,
    Obstructed,
    Open,
}

/// A cell's level, a bridge deck counted at its height (S-0213 claim 7).
fn level_of(w: &World, c: CellXY) -> i32 {
    let deck = w.cells.idx(c).is_some_and(|i| w.cells.deck[i]);
    // spec: S-0213 claim 7 and S-0207 claim 9 (a bridge deck counted at its height, 4 levels above its cell)
    w.cells.cell(c).map_or(0, |x| i32::from(x.level)) + if deck { 4 } else { 0 }
}

fn obstructed(w: &World, id: ObjectId, c: CellXY, reference: i32) -> bool {
    let kind = w.obj(id).map(|o| o.kind);
    let out = match kind {
        Some(Kind::Infantry) => occupancy::infantry_outcome(w, id, c, reference),
        _ => occupancy::vehicle_outcome(w, id, c, reference),
    };
    // spec: S-0213 claim 7 (obstructed: friendly obstacle, enemy obstacle or blocked)
    matches!(
        out,
        Outcome::FriendlyObstacle | Outcome::EnemyObstacle | Outcome::Blocked
    )
}
