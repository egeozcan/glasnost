//! The own AI and jumpjets (S-0502; PLAN §4.7): a jumpjet flies over what
//! stops a ground unit, so the brain uses it the way a person does, to reach
//! the enemy a walking army cannot, and builds one when it sees such an enemy
//! and has none.
//!
//! Like everything the brain does, it reads only what a person in the
//! house's seat sees (the enemy objects on cells the house has explored, the
//! reach regions of the map's ground, which a person sees as water and cliffs
//! on the screen) and gives its orders through `orders::issue`. It draws no
//! random value. The counter [`Uses::jumps`] counts the orders it gave.

use glasnost_map::CellXY;
use glasnost_rules::sim::MovementZone;
use glasnost_sim::econ::{TypeRef, production};
use glasnost_sim::jump;
use glasnost_sim::orders::Order;
use glasnost_sim::regions::{self, Reach};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting;
use glasnost_sim::world::{ObjectId, Target, World};

use crate::scripted::{self, Script, all_types, dist2};
use crate::special::Special;

/// The credits a house keeps in hand, as a multiple of a unit's price, before
/// it orders a jumpjet unit for an enemy it cannot walk to.
const RESERVE: i32 = 2; // nocite: tuning of our own opponent

/// Whether a jumpjet unit flies: it moves with the jumpjet locomotor
/// (S-0502 Terms), so its goals need no reach region of the ground.
pub fn flies(w: &World, id: ObjectId) -> bool {
    jump::is_jumpjet(w, id)
}

/// Whether a walking unit at `from` could not walk to `to`: the two cells lie
/// in different reach regions of the ground zone, or one is in none (water
/// and cliffs a person sees on the map).
pub fn across_obstacle(w: &World, from: CellXY, to: CellXY) -> bool {
    let a = regions::reach_in(w, from, MovementZone::Normal);
    let b = regions::reach_in(w, to, MovementZone::Normal);
    match (a, b) {
        (Reach::Region(x), Reach::Region(y)) => x != y,
        _ => true,
    }
}

impl Special {
    /// Jumpjets (S-0502): every idle armed jumpjet unit is sent against the
    /// nearest seen enemy that a walking unit of the house's cannot reach and
    /// that the jumpjet may attack; a house with no jumpjet unit (on the map
    /// or in its line) that sees such an enemy orders the cheapest armed
    /// jumpjet type its sidebar offers, once, under the credit reserve.
    pub(crate) fn jumpjets(
        &mut self,
        script: &Script,
        own: &[ObjectId],
        w: &World,
        out: &mut Vec<Order>,
    ) {
        let house = script.house;
        let enemies = script.seen_enemies(w);
        let fliers: Vec<ObjectId> = own.iter().copied().filter(|&id| flies(w, id)).collect();
        let idle: Vec<ObjectId> = fliers
            .iter()
            .copied()
            .filter(|&id| {
                scripted::idle(w, id)
                    && targeting::selectable(w, id)
                    && w.obj(id)
                        .and_then(|o| w.rules().unit(o.kind, o.type_index))
                        .is_some_and(scripted::armed)
            })
            .collect();
        for &id in &idle {
            let Some(at) = w.obj(id).map(|o| o.pos.cell()) else {
                continue;
            };
            let Some(&(e, _)) = enemies
                .iter()
                .filter(|&&(e, c)| {
                    across_obstacle(w, at, c)
                        && scripted::can_attack(w, id, e)
                        && self.memory.ok(w, id, Target::Object(e))
                })
                .min_by_key(|&&(e, c)| (dist2(c, at), e))
            else {
                continue;
            };
            out.push(Order::Behaviour {
                actor: id,
                behaviour: Behaviour::Attack,
                attack_move: false,
                target: Some(Target::Object(e)),
                dest: None,
            });
            self.offer_use(crate::special::Use::Jump, out);
        }
        if fliers.is_empty()
            && let Some(t) = cheapest_armed_jumpjet(w, script)
            && !has_or_makes(w, house, t)
            && sees_unreachable_enemy(w, script, &enemies)
        {
            self.order_jumpjet(script, w, t, out);
        }
    }

    /// Orders one unit of `t` from the sidebar when the house keeps enough
    /// credits in hand and does not sell out.
    fn order_jumpjet(&self, script: &Script, w: &World, t: TypeRef, out: &mut Vec<Order>) {
        let price = production::cost_field(w, t);
        if script.selling()
            || glasnost_sim::seams::available_credits(w, script.house) < price * RESERVE
        {
            return;
        }
        out.push(Order::Build { what: t });
    }
}

/// Whether the house sees an enemy that no walking unit of it can reach from
/// its base.
fn sees_unreachable_enemy(w: &World, script: &Script, enemies: &[(ObjectId, CellXY)]) -> bool {
    let Some(home) = script.home(w) else {
        return false;
    };
    enemies.iter().any(|&(_, c)| across_obstacle(w, home, c))
}

/// The cheapest armed jumpjet type of infantry or vehicles the house can
/// build now (the script's kept buildability; design: own AI, #60).
fn cheapest_armed_jumpjet(w: &World, script: &Script) -> Option<TypeRef> {
    all_types(w)
        .into_iter()
        .filter(|t| matches!(t.kind, Kind::Infantry | Kind::Vehicle))
        .filter(|t| {
            w.rules()
                .unit(t.kind, t.index)
                .is_some_and(|u| u.jumpjet_locomotor && scripted::armed(u) && !u.harvester)
        })
        .filter(|&t| script.buildable(w, t))
        .min_by_key(|&t| (production::cost_field(w, t), t.index))
}

/// Whether the house has a unit of type `t`, or one in its production
/// line.
fn has_or_makes(w: &World, house: glasnost_sim::world::HouseId, t: TypeRef) -> bool {
    let on_map = w
        .objects
        .values()
        .any(|o| o.owner == Some(house) && o.kind == t.kind && o.type_index == t.index);
    let queued = production::line_of(w, house, t)
        .is_some_and(|l| l.item.as_ref().is_some_and(|i| i.t == t) || l.queue.contains(&t));
    on_map || queued
}
