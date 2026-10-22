//! The own AI and linked weapons (S-0518; PLAN §4.7): a person who can
//! build the prism type builds several, close together, because each one
//! that is ready adds to the next one's shot; a person who has chargers
//! builds an overpowerable building for them if the sidebar offers one;
//! and sends charging infantry at its own overpowerable buildings, which
//! then fire their stronger weapon. The brain does all three with no limit
//! of attention.
//!
//! It reads only its own objects and its sidebar, and gives its orders
//! through `orders::issue`. It draws no random value. The counters
//! [`Uses::prisms`], [`Uses::overpowerables`] and [`Uses::charges`] count
//! the orders it gave.

use glasnost_sim::econ::{TypeRef, production, tech};
use glasnost_sim::linked;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

use crate::scripted::{self, Script, dist2};
use crate::special::Special;

/// How many prism buildings the house wants: a master and two supporters.
const PRISMS_WANTED: usize = 3; // nocite: tuning of our own opponent

/// The credits kept in hand, as a multiple of the building's price, before
/// a prism building is ordered.
const RESERVE: i32 = 2; // nocite: tuning of our own opponent

/// How many overpowerable buildings (other than the prism type) a house
/// with a charger wants.
const OVERPOWERABLE_WANTED: usize = 1; // nocite: tuning of our own opponent

/// How near, in cells, a charger looks for an own overpowerable building:
/// across the base, as a person sends one from the barracks door (M8 B4:
/// at 12, chargers stood idle at the barracks 15 to 16 cells from the
/// prisms and none was sent all game, in M6c seed 18 with
/// `quirks.path_no_reopen` flipped).
const CHARGE_REACH: i64 = 24; // nocite: tuning of our own opponent

/// The prism type as a buildable type of the house.
fn prism_type(w: &World, house: HouseId) -> Option<TypeRef> {
    let b = w.rules().combat.general.prism.building?;
    let t = TypeRef {
        kind: Kind::Building,
        index: b,
    };
    tech::buildable(w, house, t).then_some(t)
}

/// The cheapest overpowerable building type other than the prism type
/// that the house can build (the first in the rules on equal price).
fn overpowerable_type(w: &World, house: HouseId) -> Option<TypeRef> {
    let r = w.rules();
    let prism = r.combat.general.prism.building;
    (0..r.buildings.len())
        .filter_map(|i| u32::try_from(i).ok())
        .filter(|&i| Some(i) != prism && r.buildings[i as usize].building.overpowerable)
        .map(|index| TypeRef {
            kind: Kind::Building,
            index,
        })
        .filter(|&t| tech::buildable(w, house, t))
        .min_by_key(|&t| (production::cost_field(w, t), t.index))
}

/// How many buildings of the type the house has or has queued.
fn count(w: &World, house: HouseId, t: TypeRef) -> usize {
    let on_map = w
        .objects
        .values()
        .filter(|o| o.owner == Some(house) && o.kind == t.kind && o.type_index == t.index)
        .count();
    let queued = production::line_of(w, house, t).map_or(0, |l| {
        usize::from(l.item.as_ref().is_some_and(|i| i.t == t))
            + l.queue.iter().filter(|&&q| q == t).count()
    });
    on_map + queued
}

impl Special {
    /// Linked weapons (S-0518): orders a prism building while the house
    /// has fewer than [`PRISMS_WANTED`], its building line is free and it
    /// can pay twice its price; sends
    /// every idle charger at the nearest own overpowerable building that
    /// has fewer than three chargers aimed at it.
    pub(crate) fn linked(
        &mut self,
        script: &Script,
        own: &[ObjectId],
        w: &World,
        out: &mut Vec<Order>,
    ) {
        let house = script.house;
        let affords = |t: TypeRef| {
            !script.busy(w, t)
                && !script.selling()
                && glasnost_sim::seams::available_credits(w, house)
                    >= production::cost_field(w, t).saturating_mul(RESERVE)
        };
        let mut ordered = false;
        if let Some(t) = prism_type(w, house)
            && count(w, house, t) < PRISMS_WANTED
            && affords(t)
        {
            out.push(Order::Build { what: t });
            self.count_prism();
            ordered = true;
        }
        // A house with a charger and no overpowerable building of its own
        // orders one, as a person who built the charger would.
        if !ordered
            && own.iter().any(|&id| linked::charger_type(w, id))
            && let Some(t) = overpowerable_type(w, house)
        {
            let have: usize = (0..w.rules().buildings.len())
                .filter_map(|i| u32::try_from(i).ok())
                .filter(|&i| w.rules().buildings[i as usize].building.overpowerable)
                .map(|index| {
                    count(
                        w,
                        house,
                        TypeRef {
                            kind: Kind::Building,
                            index,
                        },
                    )
                })
                .sum();
            if have < OVERPOWERABLE_WANTED && affords(t) {
                out.push(Order::Build { what: t });
                self.count_overpowerable();
            }
        }
        for &id in own {
            if !linked::charger_type(w, id)
                || !scripted::idle(w, id)
                || !targeting::selectable(w, id)
            {
                continue;
            }
            let Some(at) = w.obj(id).map(|o| o.pos.cell()) else {
                continue;
            };
            let best = own
                .iter()
                .copied()
                .filter(|&b| {
                    linked::overpowerable(w, b)
                        && self.memory.ok(w, id, Target::Object(b))
                        && w.obj(b).is_some_and(|o| {
                            o.on_map
                                && o.health > 0
                                && dist2(o.pos.cell(), at) <= CHARGE_REACH * CHARGE_REACH
                        })
                        && linked::chargers(w, b) < 3 // spec: S-0518 Constants (3 chargers overpower whatever the power)
                })
                .min_by_key(|&b| (w.obj(b).map_or(i64::MAX, |o| dist2(o.pos.cell(), at)), b));
            let Some(b) = best else { continue };
            out.push(Order::Behaviour {
                actor: id,
                behaviour: Behaviour::Attack,
                attack_move: false,
                target: Some(Target::Object(b)),
                dest: None,
            });
            self.count_charge();
        }
    }
}
