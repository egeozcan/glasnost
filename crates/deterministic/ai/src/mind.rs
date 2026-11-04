//! The own AI and mind control (S-0601; PLAN §4.7).
//!
//! A mind controller takes an enemy object over instead of damaging it
//! (S-0601 claims 5 and 6), so a person aims it at the costliest enemy it
//! may take, and keeps a limited controller from wasting its shots when it
//! is full. The brain:
//!
//! - sends every idle (or guarding) controller of the house that is not at
//!   capacity (claim 2) against the costliest seen enemy within [`REACH`]
//!   cells that its firing check allows, with the attack order a person
//!   gives. Only objects the takeover test accepts are picked (claim 3:
//!   not psionics-immune or protected, not controlled already, not a
//!   building under construction or being sold); the firing check refuses
//!   the others (claim 4) only once they are in range, so the brain reads
//!   the test itself. No two controllers are sent at the same
//!   object in one think;
//! - does not send an unlimited controller (claim 16) when one more victim
//!   would bring it to an overload level that damages it: a person reads
//!   the overload rules and stops before the pulses start ([`safe_more`]).
//!   The brain drops any attack order at an object that another part of it
//!   (the waves, the base defence) gives such a controller. The
//!   controller's own target search when it guards is the game's, not the
//!   brain's, and is left alone;
//! - leaves a capacity-1 controller that holds a victim alone (it is at
//!   capacity, claim 2), as a person keeps the victim rather than swapping.
//!   The brain drops any attack order on an object that another part of it
//!   (the army's attack on a near enemy, the base defence) gives such a
//!   controller with its takeover weapon ([`swaps_victim`]): that takeover
//!   would release the victim (claims 7 and 9), and the next order would
//!   send it back at the released one (M8 seam trace 4);
//! - uses the objects it controls like any other of its own: they belong to
//!   the house (claim 6), so the waves take the armed ones along
//!   (`scripted::Script::of_role`).
//!
//! The controller types are built like any other army type (they are armed
//! units), so the production side needs no change.
//!
//! It reads only what a person in the house's seat sees (`seen_enemies`:
//! explored cells, cloaked enemies it does not sense left out) and the
//! rules a person may read, and draws no random value. The numbers are
//! tuning of our own opponent, not game behaviour.

use glasnost_map::CellXY;
use glasnost_sim::econ::{production, tech};
use glasnost_sim::mind;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Behaviour;
use glasnost_sim::world::{ObjectId, Target, World};
use glasnost_sim::{psychic, targeting};

use crate::scripted::{self, Script, dist2};
use crate::special::Special;

/// Cells from a controller within which it is sent at an enemy.
const REACH: i64 = 10; // nocite: tuning of our own opponent

/// Whether controller `c` may take one more victim without reaching an
/// overload level whose pulses damage it (S-0601 claim 16, F2). A limited
/// controller never pulses.
pub fn safe_more(w: &World, c: ObjectId) -> bool {
    let Some(rec) = mind::controller(w, c) else {
        return false;
    };
    if !rec.unlimited {
        return true;
    }
    let m = &w.rules().combat.general.mind;
    let n = rec.victims.len() as i64 + 1;
    match mind::overload_level(&m.overload_count, n) {
        None => true,
        Some(i) => m.overload_damage.get(i).copied().unwrap_or(0) < 1,
    }
}

/// Whether the house's controller `c` is free to be sent at a new victim:
/// it has a record, is not at capacity and, if unlimited, may take one more
/// safely.
pub fn free_controller(w: &World, c: ObjectId) -> bool {
    mind::controller(w, c).is_some() && !mind::at_capacity(w, c) && safe_more(w, c)
}

/// Whether an attack order on `t` would make controller `c` swap its
/// victim: `c` is at capacity (claim 2), which for a limited controller
/// that may still fire means capacity 1 (claim 7), and the weapon it would
/// use against `t` is a takeover weapon, whose landing releases the victim
/// it holds (claims 7 and 9). A weapon without the takeover warhead may
/// still be used.
pub fn swaps_victim(w: &World, c: ObjectId, t: Target) -> bool {
    if !mind::at_capacity(w, c) {
        return false;
    }
    let (_, weapon) = glasnost_sim::combat::weapon::would_use(w, c, Some(t));
    weapon.is_some_and(|wp| mind::takeover_weapon(w, wp))
}

impl Special {
    /// Mind control: every idle or guarding controller that is free is sent
    /// at the costliest seen enemy within [`REACH`] cells it may take (then
    /// the nearest), one controller per enemy per think.
    pub(crate) fn mind_control(
        &mut self,
        script: &Script,
        own: &[ObjectId],
        w: &World,
        out: &mut Vec<Order>,
    ) {
        let enemies = script.seen_enemies(w);
        if enemies.is_empty() {
            return;
        }
        let mut aimed: Vec<ObjectId> = Vec::new();
        for &id in own {
            let guarding = w.obj(id).is_some_and(|o| {
                o.dest.is_none()
                    && matches!(o.beh.current, Some(Behaviour::Guard | Behaviour::AreaGuard))
            });
            if !free_controller(w, id)
                || psychic::berserk(w, id)
                || !(scripted::idle(w, id) || guarding)
                || !targeting::selectable(w, id)
            {
                continue;
            }
            let Some(at) = w.obj(id).map(|o| o.pos.cell()) else {
                continue;
            };
            let cost = |e: ObjectId| {
                w.obj(e)
                    .map_or(0, |o| production::cost_field(w, tech::type_of(o)))
            };
            let best = enemies
                .iter()
                .filter(|&&(e, c): &&(ObjectId, CellXY)| {
                    dist2(c, at) <= REACH * REACH
                        && !aimed.contains(&e)
                        && mind::can_take(w, id, e)
                        && scripted::can_attack(w, id, e)
                        && self.memory.ok(w, id, Target::Object(e))
                })
                .map(|&(e, c)| (std::cmp::Reverse(cost(e)), dist2(c, at), e))
                .min();
            let Some((_, _, e)) = best else { continue };
            aimed.push(e);
            if w.obj(id)
                .is_some_and(|o| o.target == Some(Target::Object(e)))
            {
                continue;
            }
            out.push(Order::Behaviour {
                actor: id,
                behaviour: Behaviour::Attack,
                attack_move: false,
                target: Some(Target::Object(e)),
                dest: None,
            });
            self.count_takeover();
        }
    }
}
