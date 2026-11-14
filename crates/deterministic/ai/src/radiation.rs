//! The own AI and radiation fields (S-0525; PLAN §4.7): a person sees the
//! green glow of a radiation field on the ground and moves the units that
//! stand in it out of it. The brain does the same with every idle unit that
//! radiation would hurt (an infantry unit, vehicle or aircraft that is not
//! immune and not in the air) and that stands on a cell with radiation: it
//! sends it to the nearest cell of its own reach region that holds none. A
//! radiation-immune deployer needs no order of its own: the deploy rules of
//! `deployers` deploy it near enemies, and it then fires its radiation
//! weapon at its own cell (S-0215 claim 31).
//!
//! It reads the radiation of cells its house has explored, which a person
//! sees as the field's light, and draws no random value. The counter
//! [`Uses::radiation`] counts the orders it gave.

use glasnost_map::CellXY;
use glasnost_sim::combat::veterancy::type_combat;
use glasnost_sim::econ::shroud;
use glasnost_sim::orders::Order;
use glasnost_sim::radiation;
use glasnost_sim::regions::{self, Reach};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting;
use glasnost_sim::world::{ObjectId, Target, World};

use crate::scripted::{self, Script};
use crate::special::Special;

/// How far, in cells, from its place the AI looks for a clean cell.
const ESCAPE_REACH: i32 = 8; // nocite: tuning of our own opponent

/// Whether radiation would hurt the unit where it stands: an infantry
/// unit, vehicle or aircraft that is on the map, not airborne and not
/// immune (S-0525 claim 8).
fn hurt_by_radiation(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    // spec: S-0525 claim 8 (infantry, vehicles and aircraft; on the map, not airborne (Terms: S-0307's height test, the in-the-air answer for aircraft), not immune)
    // design: own AI (#48): the code follows the doc and the spec; a unit hovering above a field got a needless move order
    o.on_map
        && matches!(o.kind, Kind::Infantry | Kind::Vehicle | Kind::Aircraft)
        && !glasnost_sim::combat::weapon::airborne(w, id)
        && type_combat(w, id).is_some_and(|tc| !tc.immune_radiation)
}

impl Special {
    /// Radiation (S-0525): every idle unit that radiation hurts and that
    /// stands on a cell it sees irradiated leaves for the nearest clean
    /// cell of its reach region.
    pub(crate) fn radiation(
        &mut self,
        script: &Script,
        own: &[ObjectId],
        w: &World,
        out: &mut Vec<Order>,
    ) {
        if w.radiation.cells.is_empty() {
            return;
        }
        let house = script.house;
        for &id in own {
            if !hurt_by_radiation(w, id) || !scripted::idle(w, id) || !targeting::selectable(w, id)
            {
                continue;
            }
            let Some(o) = w.obj(id) else { continue };
            let here = o.pos.cell();
            if !shroud::explored(w, house, here) || radiation::exposure(w, here) < 1 {
                continue;
            }
            let Some(zone) = w.rules().unit(o.kind, o.type_index).map(|u| u.zone) else {
                continue;
            };
            let Reach::Region(region) = regions::reach_in(w, here, zone) else {
                continue;
            };
            let clean = |c: CellXY| {
                w.cells.usable(c)
                    && self.memory.ok(w, id, Target::Cell(c))
                    && shroud::explored(w, house, c)
                    && radiation::exposure(w, c) < 1
                    && regions::reach_in(w, c, zone) == Reach::Region(region)
            };
            let Some(dest) = nearest(here, clean) else {
                continue;
            };
            out.push(Order::Behaviour {
                actor: id,
                behaviour: Behaviour::Move,
                attack_move: false,
                target: None,
                dest: Some(Target::Cell(dest)),
            });
            self.offer_use(crate::special::Use::Radiation, out);
        }
    }
}

/// The nearest cell around `from` within [`ESCAPE_REACH`] that `ok` accepts:
/// ring by ring, the first in cell order on equal distance.
fn nearest(from: CellXY, ok: impl Fn(CellXY) -> bool) -> Option<CellXY> {
    for ring in 1..=ESCAPE_REACH {
        let mut best: Option<CellXY> = None;
        for dx in -ring..=ring {
            for dy in -ring..=ring {
                if dx.abs().max(dy.abs()) != ring {
                    continue;
                }
                let c = CellXY::new(from.x + dx, from.y + dy);
                if ok(c) && best.is_none_or(|b| (c.y, c.x) < (b.y, b.x)) {
                    best = Some(c);
                }
            }
        }
        if best.is_some() {
            return best;
        }
    }
    None
}
