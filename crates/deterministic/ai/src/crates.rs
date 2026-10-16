//! The own AI and crates (S-0520; PLAN §4.7): a person sees a crate on the
//! map and sends the fastest idle unit to run over it. The brain does the
//! same: every crate on a cell the house has explored gets the fastest idle
//! ground unit that is not an engineer or a harvester, nearest first, one
//! unit per crate.
//!
//! It reads only what a person in the house's seat sees (crates on explored
//! cells) and gives its orders through `orders::issue`. It draws no random
//! value. The counter [`Uses::crates`] counts the orders it gave.

use glasnost_map::CellXY;
use glasnost_sim::econ::shroud;
use glasnost_sim::orders::Order;
use glasnost_sim::regions::{self, Reach};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting;
use glasnost_sim::world::{ObjectId, Target, World};

use crate::scripted::{self, Script, dist2};
use crate::special::Special;

/// How far, in cells, a crate is worth a trip.
const REACH: i64 = 30; // nocite: tuning of our own opponent

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
        let mut taken: Vec<ObjectId> = Vec::new();
        for p in w.crates.places.values() {
            if !shroud::explored(w, house, p.cell) || !glasnost_sim::crates::is_crate(w, p.cell) {
                continue;
            }
            // A unit already on its way to the crate keeps it.
            if own.iter().any(|&id| {
                w.obj(id).is_some_and(|o| {
                    o.dest == Some(Target::Cell(p.cell)) && o.beh.current == Some(Behaviour::Move)
                })
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
                        && dist2(o.pos.cell(), p.cell) <= REACH * REACH
                        && same_region(w, id, p.cell)
                        && self.memory.ok(w, id, Target::Cell(p.cell))
                })
                .min_by_key(|&id| {
                    let o = w.obj(id).expect("filtered");
                    let speed = w.rules().unit(o.kind, o.type_index).map_or(0, |u| u.speed);
                    (-i64::from(speed), dist2(o.pos.cell(), p.cell), id)
                });
            let Some(id) = best else { continue };
            taken.push(id);
            out.push(Order::Behaviour {
                actor: id,
                behaviour: Behaviour::Move,
                attack_move: false,
                target: None,
                dest: Some(Target::Cell(p.cell)),
            });
            self.count_crate();
        }
    }
}
