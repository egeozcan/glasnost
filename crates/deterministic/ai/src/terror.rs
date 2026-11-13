//! The own AI and the special attacks of S-0519 (PLAN §4.7): bomb planters
//! go for the costliest enemy they may attack that carries no bomb;
//! engineers with a disarm weapon go for an own object that carries a bomb
//! the house sees; suicide units are thrown at the nearest enemy; infantry
//! with explosive charges are sent at the nearest enemy building the cursor
//! offers sabotage on; air-strike designators call their aircraft onto the
//! costliest enemy building. A planter or disarmer passes over a target
//! another own unit of its role is already sent at.
//!
//! It reads only what a person in the house's seat sees (the enemy objects
//! on cells the house has explored, the bombs the house sees) and gives its
//! orders through `orders::issue`. It draws no random value. The counters
//! [`Uses::bombs`], [`Uses::disarms`], [`Uses::suicides`],
//! [`Uses::sabotages`] and [`Uses::strikes`] count the orders it gave.

use glasnost_sim::bomb;
use glasnost_sim::combat::veterancy::actual_cost;
use glasnost_sim::combat::weapon;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::strike;
use glasnost_sim::targeting::{self, Action};
use glasnost_sim::world::{ObjectId, Target, World};

use crate::scripted::{self, Script, dist2};
use crate::special::{Special, click_if};

/// What a unit's primary weapon does that this module cares about.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    Plant,
    Disarm,
    Suicide,
    Designate,
    Charge,
}

fn role(w: &World, id: ObjectId) -> Option<Role> {
    if strike::designator(w, id) {
        return Some(Role::Designate);
    }
    if w.obj(id).is_some_and(|o| o.kind == Kind::Infantry)
        && glasnost_sim::combat::veterancy::type_combat(w, id).is_some()
        && w.rules()
            .unit(Kind::Infantry, w.obj(id)?.type_index)
            .is_some_and(|u| u.raw.c4.unwrap_or(false))
    {
        return Some(Role::Charge);
    }
    let wp = weapon::slot_weapon(w, id, 0)?;
    let r = w.rules();
    if r.combat.weapon(wp).suicide {
        return Some(Role::Suicide);
    }
    let h = r.combat.warhead_of(wp)?;
    if h.ivan_bomb {
        Some(Role::Plant)
    } else if h.bomb_disarm {
        Some(Role::Disarm)
    } else {
        None
    }
}

fn attack(id: ObjectId, e: ObjectId) -> Order {
    Order::Behaviour {
        actor: id,
        behaviour: Behaviour::Attack,
        attack_move: false,
        target: Some(Target::Object(e)),
        dest: None,
    }
}

impl Special {
    /// Special attacks (S-0519): every idle unit with one of the roles
    /// above is given the order a person would give it.
    pub(crate) fn terror(
        &mut self,
        script: &Script,
        own: &[ObjectId],
        w: &World,
        out: &mut Vec<Order>,
    ) {
        let house = script.house;
        let enemies = script.seen_enemies(w);
        // design: own AI (#26): a target another own unit of the same role is already sent at, or is picked for in this think, is left to it: a carrier is no target for a planting weapon once its bomb is planted (S-0519 claim 3), so every idle planter (and every idle disarmer) went for the same one
        let mut taken: Vec<(Role, ObjectId)> = own
            .iter()
            .filter_map(|&id| match w.obj(id)?.target {
                Some(Target::Object(t)) if !scripted::idle(w, id) => {
                    let r = role(w, id)?;
                    matches!(r, Role::Plant | Role::Disarm).then_some((r, t))
                }
                _ => None,
            })
            .collect();
        for &id in own {
            if !scripted::idle(w, id) || !targeting::selectable(w, id) {
                continue;
            }
            let Some(r) = role(w, id) else { continue };
            let Some(at) = w.obj(id).map(|o| o.pos.cell()) else {
                continue;
            };
            match r {
                Role::Disarm => {
                    // The object nearest to the disarmer that carries a bomb the house sees.
                    let best = own
                        .iter()
                        .copied()
                        .filter(|&c| !taken.contains(&(Role::Disarm, c)))
                        .filter(|&c| bomb::visible(w, house, c) && scripted::can_attack(w, id, c))
                        .filter(|&c| self.memory.ok(w, id, Target::Object(c)))
                        .min_by_key(|&c| {
                            (w.obj(c).map_or(i64::MAX, |o| dist2(o.pos.cell(), at)), c)
                        });
                    if let Some(c) = best {
                        out.push(attack(id, c));
                        taken.push((Role::Disarm, c));
                        self.count_disarm();
                    }
                }
                Role::Plant | Role::Designate => {
                    let building_only = r == Role::Designate;
                    let best = enemies
                        .iter()
                        .filter(|&&(e, _)| {
                            (!building_only || w.obj(e).is_some_and(|o| o.kind == Kind::Building))
                                && (building_only || !taken.contains(&(Role::Plant, e)))
                                && scripted::can_attack(w, id, e)
                                && self.memory.ok(w, id, Target::Object(e))
                        })
                        .min_by_key(|&&(e, c)| (-i64::from(actual_cost(w, e)), dist2(c, at), e));
                    if let Some(&(e, _)) = best {
                        out.push(attack(id, e));
                        if building_only {
                            self.count_strike();
                        } else {
                            taken.push((Role::Plant, e));
                            self.count_bomb();
                        }
                    }
                }
                Role::Suicide => {
                    let best = enemies
                        .iter()
                        .filter(|&&(e, _)| {
                            scripted::can_attack(w, id, e)
                                && self.memory.ok(w, id, Target::Object(e))
                        })
                        .min_by_key(|&&(e, c)| (dist2(c, at), e));
                    if let Some(&(e, _)) = best {
                        out.push(attack(id, e));
                        self.count_suicide();
                    }
                }
                Role::Charge => {
                    let mut buildings: Vec<(i64, ObjectId)> = enemies
                        .iter()
                        .filter(|&&(e, _)| {
                            w.obj(e).is_some_and(|o| o.kind == Kind::Building)
                                && self.memory.ok(w, id, Target::Object(e))
                        })
                        .map(|&(e, c)| (dist2(c, at), e))
                        .collect();
                    buildings.sort_unstable();
                    // The first building the cursor offers sabotage on.
                    for (_, b) in buildings {
                        if let Some(orders) = click_if(w, house, id, b, &[Action::Sabotage]) {
                            out.extend(orders);
                            self.count_sabotage();
                            break;
                        }
                    }
                }
            }
        }
    }
}
