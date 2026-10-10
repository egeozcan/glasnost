//! Orders (S-0213): the single command interface for human players and our
//! AI houses, the pending lists, and how orders apply at a tick boundary.

use glasnost_map::CellXY;
use glasnost_num::F64;
use glasnost_num::geom::{root_floor, table_root};
use serde::{Deserialize, Serialize};

use crate::occupancy::{self, Outcome};
use crate::regions::{self, Reach};
use crate::rules::{Behaviour, Kind};
use crate::scatter::{self, ScatterRequest};
use crate::world::{HouseId, Layer, ObjectId, Pos, Target, World};

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
}

impl Order {
    /// The actor the order names.
    pub fn actor(&self) -> ObjectId {
        match *self {
            Order::Behaviour { actor, .. }
            | Order::Stop { actor }
            | Order::Scatter { actor }
            | Order::Deploy { actor }
            | Order::GuardSpot { actor, .. } => actor,
        }
    }
}

/// The smallest run of orders that is spread (S-0213 claim 7).
// spec: S-0213 Constants (smallest spread group: 2 orders)
const MIN_RUN: usize = 2;

/// Pending-list capacity (S-0213 claim 2).
// spec: S-0213 claim 2 (at most 128 pending orders per house)
pub const PENDING_LIMIT: usize = 128;

/// Issues an order in a house's name during the current tick. Returns
/// whether it was taken (S-0213 claims 1 and 2).
pub fn issue(w: &mut World, house: HouseId, order: Order) -> bool {
    // spec: S-0213 claim 1 (an order block refuses input)
    if w.tick <= w.orders_blocked_through {
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
            apply_other(w, o);
        }
        // spec: S-0213 claim 3 (then the behaviour orders, in issue order)
        let behaviour: Vec<Order> = orders
            .into_iter()
            .filter(|o| matches!(o, Order::Behaviour { .. }))
            .collect();
        let dests = group_destinations(w, &behaviour);
        for (o, d) in behaviour.iter().zip(dests) {
            apply_behaviour(w, o, d);
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

fn apply_other(w: &mut World, o: &Order) {
    match *o {
        Order::Stop { actor } => {
            // spec: S-0213 claim 8 (live and on the map; tethering, construction and tunnels do not arise yet)
            if !live_on_map(w, actor) {
                return;
            }
            let t = w.tick;
            let harvester = w
                .obj(actor)
                .and_then(|ob| w.rules().unit(ob.kind, ob.type_index))
                .is_some_and(|u| u.harvester);
            let kind = w.obj(actor).map(|ob| ob.kind);
            if kind == Some(Kind::Vehicle) {
                crate::drive::stop(w, actor);
            }
            if let Some(ob) = w.obj_mut(actor) {
                if matches!(
                    ob.beh.current,
                    Some(Behaviour::Construction | Behaviour::Selling)
                ) {
                    // spec: S-0215 claim 12 (the stop order does nothing in construction or selling)
                    return;
                }
                // spec: S-0213 claim 8 (target and destination become nothing; contact broken off)
                ob.target = None;
                ob.dest = None;
                ob.contact = None;
                if ob.kind.is_mobile() {
                    // spec: S-0213 claim 8 (a mobile actor has no queued destinations left)
                    ob.queued_dests.clear();
                    ob.path.steps.clear();
                }
                if harvester
                    && matches!(ob.beh.current, Some(Behaviour::Harvest | Behaviour::Return))
                {
                    // spec: S-0213 claim 8 (a harvester in harvest or return gets guard at once)
                    ob.beh.queue_and_start_at_once(Behaviour::Guard, t);
                }
            }
        }
        Order::Scatter { actor } => {
            // spec: S-0213 claim 9 (a mobile actor, live and on the map: no threat point, forced, not move-anywhere)
            if !live_on_map(w, actor) || !w.obj(actor).is_some_and(|ob| ob.kind.is_mobile()) {
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
            // spec: S-0213 claim 10 (live, on the map, not an aircraft)
            if !live_on_map(w, actor) || w.obj(actor).is_some_and(|ob| ob.kind == Kind::Aircraft) {
                return;
            }
            if on_vehicle_factory(w, actor) {
                // spec: S-0213 claim 10 (an actor on a vehicle-factory cell ignores the order)
                return;
            }
            if let Some(ob) = w.obj_mut(actor) {
                // spec: S-0213 claim 10 (contact broken off, target and destination nothing, unload queued)
                ob.contact = None;
                ob.target = None;
                ob.dest = None;
                ob.beh.queue(Some(Behaviour::Unload));
            }
        }
        Order::GuardSpot { actor, spot } => {
            // spec: S-0213 claim 20 (needs only a live actor; sets its guard spot)
            if let Some(ob) = w.obj_mut(actor) {
                ob.guard_spot = Some(spot);
            }
        }
        Order::Behaviour { .. } => {}
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
                (Some(t), None) => (Behaviour::Attack, Some(Some(t))),
                (_, Some(d)) => (Behaviour::Move, Some(Some(d))),
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
        && ob.beh.current == Some(Behaviour::Enter)
        && let Some(Target::Object(d)) = dest
        && w.obj(d).is_some_and(|x| x.kind == Kind::Building)
        && ob.contact == Some(d)
    {
        return;
    }
    let Some(m) = w.obj_mut(actor) else { return };
    // spec: S-0213 table T3 (b) (it breaks off any contact)
    m.contact = None;
    // spec: S-0213 table T3 (d) (the resulting behaviour is queued, S-0215 claim 12)
    m.beh.queue(Some(b));
    // spec: S-0215 claim 12 (every such order forgets what an override put aside)
    m.beh.aside = None;
    if let Some(mem) = memory {
        m.attack_move = mem;
    }
    if b == Behaviour::AreaGuard && mobile {
        // spec: S-0213 table T3 (e) (area guard: target nothing; destination and guard spot the order's target)
        m.target = None;
        m.dest = target;
        m.guard_spot = target;
    } else {
        // spec: S-0213 table T3 (f) (target and destination from the order)
        m.target = target;
        m.dest = dest;
        if mobile {
            // spec: S-0213 table T3 (g) (a mobile actor forgets its guard spot and queued destinations)
            m.guard_spot = None;
            m.queued_dests.clear();
        }
    }
    if mobile {
        // A new destination starts a new path search (S-0212 claim 20).
        m.path.steps.clear();
        m.path.retry_at = 0;
    }
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
            let run: Vec<(usize, ObjectId)> = (i..j).map(|n| (n, orders[n].actor())).collect();
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
        let half = glasnost_num::decimal::parse("0.5", glasnost_num::decimal::Field::Decimal, mode)
            .unwrap_or_default();
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
