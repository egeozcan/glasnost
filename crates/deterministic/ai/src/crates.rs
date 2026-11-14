//! The own AI and crates (S-0520; PLAN §4.7): a person sees a crate on the
//! map and sends the fastest idle unit to run over it. The brain does the
//! same: every crate on a cell the house has explored gets the fastest idle
//! ground unit that is not an engineer, a harvester or a vehicle that deploys
//! into a construction yard, nearest first, one unit per crate. A unit
//! whose destination is the crate is on its way, whatever its behaviour;
//! a unit sent at a crate that did not keep the order is not sent at a
//! crate again for a while ([`Sent`]).
//!
//! It reads only what a person in the house's seat sees (crates on explored
//! cells) and gives its orders through `orders::issue`. It draws no random
//! value. The counter [`Uses::crates`] counts the orders it gave.

use std::collections::BTreeMap;

use glasnost_map::CellXY;
use glasnost_sim::econ::shroud;
use glasnost_sim::orders::Order;
use glasnost_sim::regions::{self, Reach};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting;
use glasnost_sim::world::{ObjectId, Target, World};
use serde::{Deserialize, Serialize};

use crate::scripted::{self, Script, dist2};
use crate::special::Special;

/// How far, in cells, a crate is worth a trip.
const REACH: i64 = 30; // nocite: tuning of our own opponent

/// How long, in ticks, a unit that did not keep a crate order is not sent
/// at a crate again (about a minute of game time).
const RESEND_AFTER: i32 = 900; // nocite: tuning of our own opponent

/// The most crate orders the rule remembers (the oldest go first).
const LIMIT: usize = 256; // nocite: tuning of our own opponent

/// What the crate rule remembers between thinks: for each unit, its last
/// crate order (the crate's cell and the tick), until the crate is gone
/// (picked up by anyone, or expired) or [`RESEND_AFTER`] ticks have passed.
/// A unit that keeps its order is on its way, and the crate is skipped, so
/// a remembered unit that is a candidate again while its crate is still
/// there is one that did not keep the order: it dropped the destination (a
/// deployer infantry that takes a stale guard target again drops it on
/// the next tick) or was given another. It is sent at no crate until then,
/// so that it is neither re-sent at every think nor sent from crate to
/// crate. Saved and hashed with the brain.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Sent {
    last: BTreeMap<ObjectId, (CellXY, i32)>,
}

impl Sent {
    /// Forgets the orders that are old, whose unit is gone or whose crate
    /// is gone.
    fn forget(&mut self, w: &World) {
        let cells: Vec<CellXY> = w.crates.places.values().map(|p| p.cell).collect();
        self.last.retain(|&unit, &mut (cell, tick)| {
            w.tick - tick < RESEND_AFTER && w.obj(unit).is_some() && cells.contains(&cell)
        });
        while self.last.len() > LIMIT {
            let oldest = self
                .last
                .iter()
                .min_by_key(|&(&unit, &(_, tick))| (tick, unit))
                .map(|(&unit, _)| unit)
                .expect("not empty");
            self.last.remove(&oldest);
        }
    }

    /// Whether `unit` was sent at a crate that is still there within the
    /// last [`RESEND_AFTER`] ticks (as of the last [`Sent::forget`]).
    fn lately(&self, w: &World, unit: ObjectId) -> bool {
        self.last
            .get(&unit)
            .is_some_and(|&(_, tick)| w.tick - tick < RESEND_AFTER)
    }
}

/// Whether unit `id` can walk or drive to `c` by the ground it stands on
/// (the crate is in the unit's reach region, S-0212 claim 4), as a person
/// sees from the map: a crate on another island, or on water for a unit
/// that walks or drives, is not worth a click. A unit outside every region
/// (a factory's exit) is not ruled out.
fn same_region(w: &World, id: ObjectId, c: CellXY) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let Some(zone) = w.rules().unit(o.kind, o.type_index).map(|u| u.zone) else {
        return true;
    };
    match (
        regions::reach_in(w, o.pos.cell(), zone),
        regions::reach_in(w, c, zone),
    ) {
        (Reach::Region(a), Reach::Region(b)) => a == b,
        // A goal outside every region of the unit's zone (water for a
        // walker): it never gets there.
        (Reach::Region(_), _) => false,
        _ => true,
    }
}

impl Special {
    /// Crates (S-0520): each seen crate is given to the fastest idle unit
    /// that can walk or drive (the nearest on equal speed), once.
    pub(crate) fn crates(
        &mut self,
        script: &Script,
        own: &[ObjectId],
        w: &World,
        out: &mut Vec<Order>,
    ) {
        let house = script.house;
        self.crate_orders.forget(w);
        let mut taken: Vec<ObjectId> = Vec::new();
        for p in w.crates.places.values() {
            if !shroud::explored(w, house, p.cell) || !glasnost_sim::crates::is_crate(w, p.cell) {
                continue;
            }
            // design: a unit already on its way to the crate keeps it, known by its destination whatever its behaviour (a deployer infantry packing up, S-0215 claim 31, walks there in guard with the destination set); one whose destination was dropped is on its way no more (M8 AI seams)
            if own.iter().any(|&id| {
                w.obj(id)
                    .is_some_and(|o| o.dest == Some(Target::Cell(p.cell)))
            }) {
                continue;
            }
            let best = own
                .iter()
                .copied()
                .filter(|id| !taken.contains(id))
                .filter(|&id| {
                    let Some(o) = w.obj(id) else { return false };
                    matches!(o.kind, Kind::Vehicle | Kind::Infantry)
                        && o.on_map
                        && scripted::idle(w, id)
                        && targeting::selectable(w, id)
                        && w.rules()
                            .unit(o.kind, o.type_index)
                            .is_some_and(|u| !u.harvester && !u.engineer)
                        // nocite: our own opponent's play (a person keeps the vehicle that deploys into a construction yard for the base; sent at a crate, it was drawn back and forth between the crate and its deploy spot and never deployed, M9 rehearsal)
                        && !scripted::deploys_into_yard(w, o.kind, o.type_index)
                        && dist2(o.pos.cell(), p.cell) <= REACH * REACH
                        && same_region(w, id, p.cell)
                        && self.memory.ok(w, id, Target::Cell(p.cell))
                        // design: a unit sent at a crate lately that did not keep the order is sent at no crate while that crate is there, for at most `RESEND_AFTER` (M8 AI seams)
                        && !self.crate_orders.lately(w, id)
                })
                .min_by_key(|&id| {
                    let o = w.obj(id).expect("filtered");
                    let speed = w.rules().unit(o.kind, o.type_index).map_or(0, |u| u.speed);
                    (-i64::from(speed), dist2(o.pos.cell(), p.cell), id)
                });
            let Some(id) = best else { continue };
            taken.push(id);
            self.crate_orders.last.insert(id, (p.cell, w.tick));
            out.push(Order::Behaviour {
                actor: id,
                behaviour: Behaviour::Move,
                attack_move: false,
                target: None,
                dest: Some(Target::Cell(p.cell)),
            });
            self.offer_use(crate::special::Use::Crate, out);
        }
    }
}
