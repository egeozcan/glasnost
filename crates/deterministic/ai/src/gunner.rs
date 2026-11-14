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
//!   nearest infantry unit of the house, idle or at home on the army's
//!   rally (no capture or entry of its own), whose mode gives
//!   that vehicle a healing weapon, by the *enter* click (S-0214); the
//!   click replaces the army's order of the same think for it (the side's
//!   healer is its engineer, an army type the army script rallies), and
//!   the capture step leaves it be;
//! - **builds the passenger.** With such a vehicle waiting and no such
//!   infantry of the house, it queues the cheapest infantry type it can
//!   build whose mode gives the vehicle a healing weapon, without the
//!   credit reserve of the other abilities (the line pays as money comes
//!   in);
//! - **keeps one healer for every [`PER_HEALER`] armed vehicles** of the
//!   house: a person mends a group with one, and every gunner turned into a
//!   healer is one gun less in the fight, so a person stops there.
//!
//! It never sends a mind controller or a berserk unit. It reads only the
//! house's own objects and the rules a person may read, and draws no
//! random value. The numbers are tuning of our own opponent, not game
//! behaviour.

use glasnost_sim::combat::weapon;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind, UnitRules};
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
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

/// The objects a unit is on its way into: of every unit on the map that
/// enters, the object it is in contact with and its destination object,
/// itself left out. One pass over the objects instead of one per vehicle
/// asked (design: own AI, #61).
fn awaited(w: &World) -> std::collections::BTreeSet<ObjectId> {
    w.objects
        .values()
        .filter(|o| o.on_map && entering(o))
        .flat_map(|o| {
            let dest = match o.dest {
                Some(Target::Object(v)) => Some(v),
                _ => None,
            };
            o.contact
                .into_iter()
                .chain(dest)
                .filter(move |&v| v != o.id)
        })
        .collect()
}

/// Whether an order of this think already moves `id`.
fn ordered(out: &[Order], id: ObjectId) -> bool {
    out.iter()
        .any(|o| matches!(o, Order::Behaviour { actor, .. } if *actor == id))
}

/// Whether infantry unit `id` may be sent: selectable, not berserk, no
/// mind controller, not in contact with a building.
fn free_infantry(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    o.kind == Kind::Infantry
        && o.on_map
        && o.contact.is_none()
        && !psychic::berserk(w, id)
        && mind::controller(w, id).is_none()
        && targeting::selectable(w, id)
}

/// Whether unit `id` only moves or stands with the army (the army's
/// rally): no capture, entry or other task of its own that the click
/// would cut short.
fn rallied(w: &World, id: ObjectId) -> bool {
    use Behaviour as B;
    let rally = |b: Option<Behaviour>| {
        matches!(
            b,
            None | Some(B::Move | B::QueuedMove | B::Guard | B::AreaGuard | B::AttackMove)
        )
    };
    w.obj(id)
        .is_some_and(|o| rally(o.beh.current) && rally(o.beh.queued))
}

/// What the gunner step would do now: the (infantry, vehicle) pairs to
/// send in, and the infantry type to build when a vehicle waits and the
/// house has no such infantry.
struct GunnerPlan {
    loads: Vec<(ObjectId, ObjectId)>,
    build: Option<glasnost_sim::econ::TypeRef>,
}

impl Special {
    fn gunner_plan(
        &self,
        script: &Script,
        own: &[ObjectId],
        w: &World,
        out: &[Order],
    ) -> GunnerPlan {
        let mut plan = GunnerPlan {
            loads: Vec::new(),
            build: None,
        };
        let house: HouseId = script.house;
        let Some(home) = script.home(w) else {
            return plan;
        };
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
        // design: own AI (#61): no healer is wanted with too few armed
        // vehicles, and none can be loaded without a switching gunner
        // vehicle (`empty` below takes only those), so the healers are not
        // counted then.
        if armed / PER_HEALER == 0
            || !vehicles
                .iter()
                .any(|&id| unit(id).is_some_and(switching_gunner))
        {
            return plan;
        }
        let awaited = awaited(w);
        let healers = vehicles.iter().filter(|&&id| healer(w, id)).count()
            + vehicles.iter().filter(|&id| awaited.contains(id)).count();
        // A person stops at one healer for every PER_HEALER armed vehicles.
        let mut wanted = (armed / PER_HEALER).saturating_sub(healers);
        if wanted == 0 {
            return plan;
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
                    && !awaited.contains(&id)
                    && !mind::mind_controlled(w, id)
            })
            .collect();
        for v in empty {
            if wanted == 0 {
                break;
            }
            let Some(vr) = unit(v) else { continue };
            let Some(at) = w.obj(v).map(|o| o.pos.cell()) else {
                continue;
            };
            let mut candidates: Vec<(i64, ObjectId)> = own
                .iter()
                .copied()
                .filter(|&id| {
                    !plan.loads.iter().any(|&(p, _)| p == id)
                        && free_infantry(w, id)
                        && (scripted::idle(w, id) || (at_home(id) && rallied(w, id)))
                        && !mind::mind_controlled(w, id)
                        && unit(id).is_some_and(|p| heals_in(w, vr, p))
                        && self.memory.ok(w, id, Target::Object(v))
                })
                .map(|id| (w.obj(id).map_or(i64::MAX, |o| dist2(o.pos.cell(), at)), id))
                .collect();
            candidates.sort_unstable();
            // design: own AI (#25): the nearest unit the enter cursor shows on the vehicle (S-0214; S-0507 claims 1 and 4): a refused nearest unit was picked at every think and nobody was loaded
            let pick = candidates.into_iter().map(|(_, id)| id).find(|&id| {
                targeting::cursor(w, house, &[id], ClickOn::Object(v), Modifiers::default())
                    == Action::Enter
            });
            match pick {
                Some(id) => {
                    plan.loads.push((id, v));
                    wanted -= 1;
                }
                None => {
                    let has = own.iter().any(|&id| {
                        w.obj(id).is_some_and(|o| o.kind == Kind::Infantry)
                            && unit(id).is_some_and(|p| heals_in(w, vr, p))
                    });
                    if !has {
                        plan.build = Self::buildable_infantry(script, w, |p| heals_in(w, vr, p));
                    }
                    break;
                }
            }
        }
        plan
    }

    /// The infantry units the gunner step keeps for its vehicles in this
    /// think: an ability that runs before it (the engineers' captures)
    /// leaves them be, as a person who has picked a unit for a gunner
    /// vehicle does not send it elsewhere.
    pub(crate) fn gunner_reserved(
        &self,
        script: &Script,
        own: &[ObjectId],
        w: &World,
        out: &[Order],
    ) -> Vec<ObjectId> {
        self.gunner_plan(script, own, w, out)
            .loads
            .into_iter()
            .map(|(p, _)| p)
            .collect()
    }

    /// Gunner vehicles: a healer into each idle empty gunner vehicle at
    /// home while the house keeps fewer healers than its armed vehicles
    /// call for, by the *enter* click; the passenger's type is built when
    /// the house has none. `start` is where the special orders begin in
    /// `out`; it moves back by the healer's earlier orders removed before it.
    pub(crate) fn gunners(
        &mut self,
        script: &Script,
        own: &[ObjectId],
        w: &World,
        out: &mut Vec<Order>,
        start: &mut usize,
    ) {
        let plan = self.gunner_plan(script, own, w, out);
        for (id, v) in plan.loads {
            if let Some(orders) = click_if(w, script.house, id, v, &[Action::Enter]) {
                // The healer's click replaces any order of this think for
                // it (the army's rally: an engineer is an army type).
                // design: own AI (#3): the script's and the brain's orders before `start` that go keep the special orders' start in step, so the load itself is noted and the slice never starts past the end
                let before = out[..(*start).min(out.len())]
                    .iter()
                    .filter(|o| o.actor() == Some(id))
                    .count();
                *start -= before;
                out.retain(|o| o.actor() != Some(id));
                out.extend(orders);
                self.count_gunner_load();
            }
        }
        if let Some(t) = plan.build {
            self.queue_unit(script, w, t, out);
        }
    }
}

/// The counts of [`probe`].
#[derive(Clone, Copy, Debug, Default)]
pub struct GunnerProbe {
    /// Armed vehicles that are not healers.
    pub armed: usize,
    /// Switching gunner vehicles.
    pub gunners: usize,
    /// Those carrying anyone.
    pub carrying: usize,
    /// Those empty and idle at home.
    pub empty_home: usize,
    /// Infantry units that would heal in one of them.
    pub healers: usize,
    /// Those at home or idle, free to be sent.
    pub free: usize,
}

/// What [`Special::gunners`] sees for `house` now, for the lane's
/// `GLASNOST_AI_GUNPROBE` lines (a diagnostic, outside the brain): its
/// armed vehicles, its switching gunner vehicles, those carrying anyone,
/// those empty and idle at home, its infantry that would heal in one of
/// them, and those free at home.
pub fn probe(w: &World, house: HouseId) -> GunnerProbe {
    let script = Script::new(house, crate::brain::default_plan());
    let home = script.home(w);
    let at_home = |o: &glasnost_sim::world::Object| {
        home.is_some_and(|h| dist2(o.pos.cell(), h) <= HOME_REACH * HOME_REACH)
    };
    let own: Vec<&glasnost_sim::world::Object> = w
        .objects
        .values()
        .filter(|o| o.owner == Some(house) && o.on_map && o.health > 0)
        .collect();
    let unit = |o: &glasnost_sim::world::Object| w.rules().unit(o.kind, o.type_index);
    let armed = own
        .iter()
        .filter(|o| o.kind == Kind::Vehicle && weapon::armed(w, o.id) && !healer(w, o.id))
        .count();
    let gunners: Vec<_> = own
        .iter()
        .filter(|o| o.kind == Kind::Vehicle && unit(o).is_some_and(switching_gunner))
        .collect();
    let carrying = gunners
        .iter()
        .filter(|o| transport::count(w, o.id) > 0)
        .count();
    let empty_home = gunners
        .iter()
        .filter(|o| transport::count(w, o.id) == 0 && at_home(o) && scripted::idle(w, o.id))
        .count();
    let heals = |p: &glasnost_sim::world::Object| {
        gunners
            .iter()
            .any(|v| unit(v).is_some_and(|vr| unit(p).is_some_and(|pr| heals_in(w, vr, pr))))
    };
    let healers: Vec<_> = own
        .iter()
        .filter(|o| o.kind == Kind::Infantry && heals(o))
        .collect();
    let free = healers
        .iter()
        .filter(|o| {
            free_infantry(w, o.id) && (scripted::idle(w, o.id) || (at_home(o) && rallied(w, o.id)))
        })
        .count();
    GunnerProbe {
        armed,
        gunners: gunners.len(),
        carrying,
        empty_home,
        healers: healers.len(),
        free,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{fixture, game::Game};

    /// Whether a unit is on its way into vehicle `v`, as before #61: one
    /// pass over the objects for each vehicle asked.
    fn awaited_as_before(w: &World, v: ObjectId) -> bool {
        w.objects.values().any(|o| {
            o.on_map
                && o.id != v
                && entering(o)
                && (o.contact == Some(v) || o.dest == Some(Target::Object(v)))
        })
    }

    /// [`Special::gunner_plan`] as before #61: the healers counted, each
    /// awaited vehicle by its own pass, before the wanted count is known.
    fn gunner_plan_as_before(
        s: &Special,
        script: &Script,
        own: &[ObjectId],
        w: &World,
        out: &[Order],
    ) -> GunnerPlan {
        let mut plan = GunnerPlan {
            loads: Vec::new(),
            build: None,
        };
        let house: HouseId = script.house;
        let Some(home) = script.home(w) else {
            return plan;
        };
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
            + vehicles
                .iter()
                .filter(|&&id| awaited_as_before(w, id))
                .count();
        let mut wanted = (armed / PER_HEALER).saturating_sub(healers);
        if wanted == 0 {
            return plan;
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
                    && !awaited_as_before(w, id)
                    && !mind::mind_controlled(w, id)
            })
            .collect();
        for v in empty {
            if wanted == 0 {
                break;
            }
            let Some(vr) = unit(v) else { continue };
            let Some(at) = w.obj(v).map(|o| o.pos.cell()) else {
                continue;
            };
            let mut candidates: Vec<(i64, ObjectId)> = own
                .iter()
                .copied()
                .filter(|&id| {
                    !plan.loads.iter().any(|&(p, _)| p == id)
                        && free_infantry(w, id)
                        && (scripted::idle(w, id) || (at_home(id) && rallied(w, id)))
                        && !mind::mind_controlled(w, id)
                        && unit(id).is_some_and(|p| heals_in(w, vr, p))
                        && s.memory.ok(w, id, Target::Object(v))
                })
                .map(|id| (w.obj(id).map_or(i64::MAX, |o| dist2(o.pos.cell(), at)), id))
                .collect();
            candidates.sort_unstable();
            let pick = candidates.into_iter().map(|(_, id)| id).find(|&id| {
                targeting::cursor(w, house, &[id], ClickOn::Object(v), Modifiers::default())
                    == Action::Enter
            });
            match pick {
                Some(id) => {
                    plan.loads.push((id, v));
                    wanted -= 1;
                }
                None => {
                    let has = own.iter().any(|&id| {
                        w.obj(id).is_some_and(|o| o.kind == Kind::Infantry)
                            && unit(id).is_some_and(|p| heals_in(w, vr, p))
                    });
                    if !has {
                        plan.build = Special::buildable_infantry(script, w, |p| heals_in(w, vr, p));
                    }
                    break;
                }
            }
        }
        plan
    }

    /// Our tooling for speed (design: own AI, #61): the set of awaited
    /// objects from one pass holds exactly the objects a unit is on its way
    /// into, and the gunner plan with its early exits is the plan as before,
    /// for every object and every house at many points of the M6c, M7a and
    /// M7b duels.
    #[test]
    fn the_awaited_set_and_the_early_plan_are_as_before() {
        let games = [
            (
                crate::fixture_m6c::rules(),
                crate::fixture_m6c::map(),
                crate::fixture_m6c::MAP_NAME,
                crate::fixture_m6c::setup(2),
            ),
            (
                crate::fixture_m7a::rules(),
                crate::fixture_m7a::map(),
                crate::fixture_m7a::MAP_NAME,
                crate::fixture_m7a::setup(17),
            ),
            (
                crate::fixture_m7b::rules(),
                crate::fixture_m7b::map(),
                crate::fixture_m7b::MAP_NAME,
                crate::fixture_m7b::setup(11),
            ),
        ];
        let (mut entered, mut plans) = (0, 0);
        for (rules, map, map_name, setup) in games {
            let mut g = Game::new(rules, &map, map_name, fixture::catalogue(), setup)
                .expect("the duel loads");
            for _ in 0..120 {
                for _ in 0..75 {
                    g.step();
                }
                let w = &g.world;
                let set = awaited(w);
                for &v in w.objects.keys() {
                    let want = awaited_as_before(w, v);
                    assert_eq!(set.contains(&v), want, "{map_name}, tick {}, {v:?}", w.tick);
                    entered += usize::from(want);
                }
                for h in 0..w.houses.len() {
                    let script = Script::new(HouseId(h as u8), crate::brain::default_plan());
                    let own = script.own(w);
                    let s = Special::default();
                    let now = s.gunner_plan(&script, &own, w, &[]);
                    let before = gunner_plan_as_before(&s, &script, &own, w, &[]);
                    assert_eq!(now.loads, before.loads, "{map_name}, tick {}", w.tick);
                    assert_eq!(now.build, before.build, "{map_name}, tick {}", w.tick);
                    plans += 1;
                }
            }
        }
        assert!(entered > 0 && plans > 0, "{entered} entered, {plans} plans");
    }
}
