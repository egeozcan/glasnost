//! The own AI and deployable units (S-0517; PLAN §4.7): a person deploys a
//! vehicle that fights from the deployed form when the enemy comes near,
//! and packs it up again when the enemy is gone, so that it can move with the
//! army; an infantry unit that fires from its deployed form is deployed
//! when an enemy it can fire at stands in its range, and packed up when none
//! does; a deploy-fire vehicle is told to deploy when an enemy it can fire
//! at is near.
//!
//! The brain reads only what a person in the house's seat sees (the enemy
//! objects on cells the house has explored) and gives its orders through
//! `orders::issue`. It draws no random value and has no limit on how often
//! or how fast it gives these orders. The counter [`Uses::deploys`] counts
//! the orders it gave.

use glasnost_map::CellXY;
use glasnost_sim::econ::{deploy, inplace};
use glasnost_sim::infantry::Seq;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting;
use glasnost_sim::world::{ObjectId, Target, World};

use crate::scripted::{self, Script, dist2};
use crate::special::Special;

/// How near, in cells, an enemy must be for the house to deploy a unit.
const NEAR: i64 = 9; // nocite: tuning of our own opponent

/// How far, in cells, the enemy must be for the house to pack a deployed
/// vehicle up again (wider than `NEAR`, so a unit does not toggle).
const GONE: i64 = 13; // nocite: tuning of our own opponent

/// Whether the vehicle deploys in place (S-0517 claim 1): the in-place flag
/// and none of the rules that come before it.
pub fn in_place_vehicle(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    inplace::is_in_place(w, id)
        && deploy::deploys_into(w, o.kind, o.type_index).is_none()
        && w.rules()
            .unit(o.kind, o.type_index)
            .is_some_and(|u| u.passengers == 0 && !u.harvester)
}

/// Whether the unit is a deploy-fire vehicle (S-0517 claim 2) with a weapon
/// in slot 0.
pub fn deploy_fire_vehicle(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    o.kind == Kind::Vehicle
        && w.rules()
            .unit(o.kind, o.type_index)
            .is_some_and(|u| u.combat.deploy_fire && !u.simple_deployer && u.primary.is_some())
}

/// Whether the infantry unit deploys to fire (S-0517 claim 16) and is not
/// deployed yet.
pub fn firing_infantry(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    o.kind == Kind::Infantry
        && w.rules().unit(o.kind, o.type_index).is_some_and(|u| {
            u.deployer
                && u.raw.deploy_fire.unwrap_or(false)
                && u.raw.undeploy_delay.unwrap_or(-1) < 0
        })
}

/// Whether the infantry unit stands in its deployed form.
pub(crate) fn infantry_deployed(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .and_then(|o| o.infantry.as_ref())
        .and_then(|s| s.seq)
        .is_some_and(Seq::is_deploy)
}

/// Whether the unit stands where it is, as a person sees it: no destination
/// and a resting or attacking behaviour, with or without a target (a unit
/// already firing at the enemy is the one a person deploys).
fn standing(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        o.dest.is_none()
            && matches!(
                o.beh.current,
                None | Some(Behaviour::Guard)
                    | Some(Behaviour::AreaGuard)
                    | Some(Behaviour::Stop)
                    | Some(Behaviour::Attack)
            )
    })
}

/// Whether a seen enemy that `id` can attack stands in the range of the
/// weapon it would use against it from where it stands (S-0307's range
/// test, as the approach step asks it, S-0215 claim 37 b).
fn enemy_in_range(w: &World, id: ObjectId, enemies: &[(ObjectId, CellXY)]) -> bool {
    enemies.iter().any(|&(e, _)| {
        // spec: S-0215 claim 37 (b) (the range test of S-0307 with the weapon chosen for the target)
        glasnost_sim::approach::in_range(w, id, Target::Object(e)) && scripted::can_attack(w, id, e)
    })
}

/// Whether a seen enemy that `id` can attack stands within `r` cells.
fn enemy_within(w: &World, id: ObjectId, enemies: &[(ObjectId, CellXY)], r: i64) -> bool {
    let Some(at) = w.obj(id).map(|o| o.pos.cell()) else {
        return false;
    };
    enemies
        .iter()
        .any(|&(e, c)| dist2(c, at) <= r * r && scripted::can_attack(w, id, e))
}

impl Special {
    /// Deployable units (S-0517): a standing in-place vehicle or a
    /// deploy-fire vehicle that has a seen enemy near deploys; a deployed
    /// in-place vehicle that has none near packs up. A standing firing
    /// infantry unit deploys when a seen enemy it can attack stands in its
    /// range, and a deployed one packs up when none stands in the range of
    /// its deployed weapon. M8 step B7: deployed infantry takes no step
    /// towards a target (S-0517 claim 16) and drops its destination
    /// (S-0215 claim 22), and this part packed none up; in the lane's
    /// 02-0-7 the Allied riflemen, deployed out of range of the enemy's
    /// last buildings and at home, stood so from tick 37 500 to the end,
    /// every wave's order dropped, and the game had no winner. A person
    /// deploys a rifleman when the enemy is in its sights and packs it up
    /// to move it.
    pub(crate) fn deployers(
        &mut self,
        script: &Script,
        own: &[ObjectId],
        w: &World,
        out: &mut Vec<Order>,
    ) {
        let enemies = script.seen_enemies(w);
        for &id in own {
            if !targeting::selectable(w, id) {
                continue;
            }
            let order = if in_place_vehicle(w, id) {
                if inplace::in_transition(w, id) {
                    false
                } else if inplace::deployed(w, id) {
                    // Packing up: no enemy it could fire at is near.
                    !enemy_within(w, id, &enemies, GONE)
                } else {
                    standing(w, id) && enemy_within(w, id, &enemies, NEAR)
                }
            } else if deploy_fire_vehicle(w, id) {
                standing(w, id) && enemy_within(w, id, &enemies, NEAR)
            } else if firing_infantry(w, id) {
                if infantry_deployed(w, id) {
                    !enemy_in_range(w, id, &enemies)
                } else {
                    standing(w, id) && enemy_in_range(w, id, &enemies)
                }
            } else {
                false
            };
            if order {
                out.push(Order::Deploy { actor: id });
                self.count_deploy();
            }
        }
    }
}
