//! The own AI and chrono weapons (S-0514; PLAN §4.7): a temporal beam
//! erases a target over time, the faster the more attackers join and the
//! more the target is worth, so a person sends every erasing unit against
//! the costliest enemy it can erase, joining a lock already made. The brain
//! does the same with every idle unit whose own type supplies a beam and
//! that still has it. A gunner vehicle that holds a beam only through its
//! passenger is not sent by this step (#48: the doc follows the code).
//!
//! It reads only what a person in the house's seat sees (the enemy objects
//! on cells the house has explored) and gives its orders through
//! `orders::issue`. It draws no random value. The counter
//! [`Uses::erasures`] counts the orders it gave.

use glasnost_sim::combat::veterancy::{actual_cost, type_combat};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting;
use glasnost_sim::temporal;
use glasnost_sim::world::{ObjectId, Target, World};

use crate::scripted::{self, Script, dist2};
use crate::special::Special;

/// Whether a person could erase the enemy object: an infantry unit,
/// vehicle, aircraft or building of an erasable type that no curtain or
/// shield protects and that is in the ground layer (S-0514 claims 1 and 3).
fn erasable(w: &World, e: ObjectId) -> bool {
    w.obj(e).is_some_and(|o| {
        matches!(
            o.kind,
            Kind::Infantry | Kind::Vehicle | Kind::Aircraft | Kind::Building
        )
    }) && type_combat(w, e).is_some_and(|tc| tc.warpable)
        && !glasnost_sim::superweapons::protected(w, e)
        && temporal::ground_layer(w, e)
}

impl Special {
    /// Chrono weapons (S-0514): every idle unit whose type supplies a
    /// temporal beam, that has it and holds no lock, is sent against the
    /// costliest seen enemy it may attack
    /// and that can be erased (the nearest on equal cost), with the attack
    /// order a person gives.
    pub(crate) fn chrono(
        &mut self,
        script: &Script,
        own: &[ObjectId],
        w: &World,
        out: &mut Vec<Order>,
    ) {
        let enemies = script.seen_enemies(w);
        for &id in own {
            if !temporal::type_beam(w, id)
                || !temporal::has_beam(w, id)
                || w.temporal.beams.contains_key(&id)
                || !scripted::idle(w, id)
                || !targeting::selectable(w, id)
            {
                continue;
            }
            let Some(at) = w.obj(id).map(|o| o.pos.cell()) else {
                continue;
            };
            let best = enemies
                .iter()
                .filter(|&&(e, _)| {
                    erasable(w, e)
                        && scripted::can_attack(w, id, e)
                        && self.memory.ok(w, id, Target::Object(e))
                })
                .min_by_key(|&&(e, c)| (-i64::from(actual_cost(w, e)), dist2(c, at), e));
            let Some(&(e, _)) = best else { continue };
            out.push(Order::Behaviour {
                actor: id,
                behaviour: Behaviour::Attack,
                attack_move: false,
                target: Some(Target::Object(e)),
                dest: None,
            });
            self.offer_use(crate::special::Use::Erasure, out);
        }
    }
}
