//! The own AI and gunner vehicles (S-0308 claims 5 and 6; PLAN §4.7).
//!
//! A gunner vehicle takes its weapon from its first passenger: when an
//! object enters it and is its only passenger, the vehicle's weapon number
//! becomes the passenger type's mode index (S-0308 claim 6), and a vehicle
//! of a switching type fires the weapon in that slot (claim 5). A person
//! who reads this off the units puts into an empty gunner vehicle an
//! infantry unit whose mode gives it a healing weapon (negative damage,
//! S-0214 Terms), so that the army takes a vehicle along that mends its
//! vehicles. A person who knows this:
//!
//! - **loads a healer.** Each idle, empty gunner vehicle of the house at
//!   home ([`HOME_REACH`] cells), with no unit on its way in, gets the
//!   nearest free infantry unit of the house at home whose mode gives that
//!   vehicle a healing weapon, by the *enter* click (S-0214);
//! - **builds the passenger.** With such a vehicle waiting and no such
//!   infantry of the house, it orders the cheapest infantry type it can
//!   build whose mode gives the vehicle a healing weapon;
//! - **keeps one healer for every [`PER_HEALER`] armed vehicles** of the
//!   house: a person mends a group with one, and every gunner turned into a
//!   healer is one gun less in the fight, so a person stops there.
//!
//! It never sends a mind controller, a berserk unit or an infantry unit
//! another ability gave an order in the same think. It reads only the
//! house's own objects and the rules a person may read, and draws no
//! random value. The numbers are tuning of our own opponent, not game
//! behaviour.

use glasnost_sim::combat::weapon;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind, UnitRules};
use glasnost_sim::targeting::{self, Action};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{mind, psychic, transport};

use crate::scripted::{self, Script, dist2};
use crate::special::{Special, click_if};

/// Cells from home within which an empty gunner vehicle is loaded, and an
/// infantry unit is sent to it.
pub const HOME_REACH: i64 = 16; // nocite: tuning of our own opponent

/// The armed vehicles of the house for each healing gunner it keeps.
pub const PER_HEALER: usize = 5; // nocite: tuning of our own opponent

/// Whether vehicle type `v` is a gunner type that changes its weapon with
/// its passenger: the gunner flag and a switching type (S-0308 claims 5
/// and 6: a charge-turret type keeps weapon number 0).
fn switching_gunner(v: &UnitRules) -> bool {
    let c = &v.combat;
    c.gunner && c.multi_weapon && !c.gattling && !c.charge_turret
}

/// Whether infantry type `p`, as the only passenger of a vehicle of type
/// `v`, gives it a healing weapon: the vehicle's slot at the passenger's
/// mode index holds a weapon of negative damage (S-0308 claims 5 and 6,
/// S-0214 Terms).
pub fn heals_in(w: &World, v: &UnitRules, p: &UnitRules) -> bool {
    if !switching_gunner(v) {
        return false;
    }
    let Ok(m) = usize::try_from(p.combat.ifv_mode) else {
        return false;
    };
    v.combat
        .slots
        .get(m)
        .copied()
        .flatten()
        .is_some_and(|x| w.rules().combat.weapon(x).damage < 0)
}

/// Whether own vehicle `id` is a healing gunner now: a switching gunner
/// whose only passenger gives it a healing weapon.
fn healer(w: &World, id: ObjectId) -> bool {
    let Some(v) = w.obj(id).and_then(|o| w.rules().unit(o.kind, o.type_index)) else {
        return false;
    };
    let ps = transport::passengers(w, id);
    ps.len() == 1
        && w.obj(ps[0])
            .and_then(|o| w.rules().unit(o.kind, o.type_index))
            .is_some_and(|p| heals_in(w, v, p))
}

fn entering(o: &glasnost_sim::world::Object) -> bool {
    o.beh.current == Some(Behaviour::Enter) || o.beh.queued == Some(Behaviour::Enter)
}

/// Whether a unit is on its way into vehicle `v`.
fn awaited(w: &World, v: ObjectId) -> bool {
    w.objects.values().any(|o| {
        o.on_map
            && o.id != v
            && entering(o)
            && (o.contact == Some(v) || o.dest == Some(Target::Object(v)))
    })
}

/// Whether an order of this think already moves `id`.
fn ordered(out: &[Order], id: ObjectId) -> bool {
    out.iter()
        .any(|o| matches!(o, Order::Behaviour { actor, .. } if *actor == id))
}

/// Whether infantry unit `id` is free to be sent: idle, selectable, not
/// berserk, no mind controller, not in contact with a building.
fn free_infantry(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    o.kind == Kind::Infantry
        && o.on_map
        && o.contact.is_none()
        && !psychic::berserk(w, id)
        && mind::controller(w, id).is_none()
        && scripted::idle(w, id)
        && targeting::selectable(w, id)
}

impl Special {
    /// Gunner vehicles: a healer into each idle empty gunner vehicle at
    /// home while the house keeps fewer healers than its armed vehicles
    /// call for, by the *enter* click; the passenger's type is built when
    /// the house has none.
    pub(crate) fn gunners(
        &mut self,
        script: &Script,
        own: &[ObjectId],
        w: &World,
        out: &mut Vec<Order>,
    ) {
        let house: HouseId = script.house;
        let Some(home) = script.home(w) else { return };
        let at_home = |id: ObjectId| {
            w.obj(id)
                .is_some_and(|o| dist2(o.pos.cell(), home) <= HOME_REACH * HOME_REACH)
        };
        let unit = |id: ObjectId| w.obj(id).and_then(|o| w.rules().unit(o.kind, o.type_index));
        let vehicles: Vec<ObjectId> = own
            .iter()
            .copied()
            .filter(|&id| {
                w.obj(id)
                    .is_some_and(|o| o.kind == Kind::Vehicle && o.on_map && o.health > 0)
            })
            .collect();
        let armed = vehicles
            .iter()
            .filter(|&&id| weapon::armed(w, id) && !healer(w, id))
            .count();
        let healers = vehicles.iter().filter(|&&id| healer(w, id)).count()
            + vehicles.iter().filter(|&&id| awaited(w, id)).count();
        // A person stops at one healer for every PER_HEALER armed vehicles.
        let mut wanted = (armed / PER_HEALER).saturating_sub(healers);
        if wanted == 0 {
            return;
        }
        let empty: Vec<ObjectId> = vehicles
            .iter()
            .copied()
            .filter(|&id| {
                transport::count(w, id) == 0
                    && unit(id).is_some_and(switching_gunner)
                    && at_home(id)
                    && scripted::idle(w, id)
                    && !ordered(out, id)
                    && !awaited(w, id)
                    && !mind::mind_controlled(w, id)
            })
            .collect();
        let mut taken: Vec<ObjectId> = Vec::new();
        for v in empty {
            if wanted == 0 {
                break;
            }
            let Some(vr) = unit(v) else { continue };
            let Some(at) = w.obj(v).map(|o| o.pos.cell()) else {
                continue;
            };
            let pick = own
                .iter()
                .copied()
                .filter(|&id| {
                    !taken.contains(&id)
                        && free_infantry(w, id)
                        && at_home(id)
                        && !ordered(out, id)
                        && !mind::mind_controlled(w, id)
                        && unit(id).is_some_and(|p| heals_in(w, vr, p))
                        && self.memory.ok(w, id, Target::Object(v))
                })
                .min_by_key(|&id| (w.obj(id).map_or(i64::MAX, |o| dist2(o.pos.cell(), at)), id));
            match pick {
                Some(id) => {
                    if let Some(orders) = click_if(w, house, id, v, &[Action::Enter]) {
                        out.extend(orders);
                        taken.push(id);
                        wanted -= 1;
                        self.count_gunner_load();
                    }
                }
                None => {
                    let has = own.iter().any(|&id| {
                        w.obj(id).is_some_and(|o| o.kind == Kind::Infantry)
                            && unit(id).is_some_and(|p| heals_in(w, vr, p))
                    });
                    if !has
                        && let Some(t) = Self::buildable_infantry(w, house, |p| heals_in(w, vr, p))
                    {
                        self.order_unit(script, w, t, out);
                    }
                    break;
                }
            }
        }
    }
}
