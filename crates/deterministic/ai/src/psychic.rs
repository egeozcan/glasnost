//! The own AI and psychic weapons (S-0610; PLAN §4.7).
//!
//! - **Berserk weapons**: a unit hit by a berserk-causing warhead attacks
//!   whatever is near it, its own side included (S-0610 claims 4 and 6), so
//!   a person aims a berserk weapon at an enemy standing among other
//!   enemies. The brain sends every idle unit whose weapon has a
//!   berserk-causing warhead against the seen enemy unit within [`REACH`]
//!   cells that has the most other seen enemy units around it, with the
//!   attack order a person gives. It skips targets that are already berserk
//!   (a person sees the tint, claim 7) and the ones its weapon may not fire
//!   at (the firing check refuses psionics-immune targets, claim 3).
//! - **The psychic sensor** (claims 9 to 11): once the house keeps its
//!   harvesters and has no sensor, it builds the cheapest building type with
//!   a psychic-detection radius it can build and place now, as a person
//!   adds the sensor to a base ([`crate::brain::Brain`] calls
//!   [`build_sensor`]).
//! - **The psychic reveal** (claims 13 to 15): fired by the brain's
//!   superweapon step at the enemy start cell nearest home that the house
//!   does not see now ([`reveal_cell`]).
//!
//! The deployed psychic infantry pulse (S-0517 claim 16) is already the
//! deployers' (`deployers`, firing infantry near an enemy). No order goes to
//! a berserk unit (S-0214 claim 15: it takes none); the brain drops such
//! orders before it issues them.
//!
//! It reads only what a person in the house's seat sees and draws no random
//! value. The numbers are tuning of our own opponent, not game behaviour.

use glasnost_map::CellXY;
use glasnost_sim::combat::weapon;
use glasnost_sim::econ::{TypeRef, production, tech};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{ObjectId, Target, World};
use glasnost_sim::{econ::shroud, psychic, targeting};

use crate::scripted::{self, Script, all_types, dist2};
use crate::special::Special;

/// Cells from a berserk-weapon unit within which it is sent at an enemy.
const REACH: i64 = 12; // nocite: tuning of our own opponent
/// Cells around a target within which other enemy units count as its group.
const GROUP: i64 = 3; // nocite: tuning of our own opponent

/// Whether one of the unit's two weapon slots has a berserk-causing warhead
/// (S-0610 Terms: a berserk weapon).
pub fn berserk_weapon(w: &World, id: ObjectId) -> bool {
    let r = w.rules();
    // nocite: the primary and secondary slots our own AI looks at
    (0..2).any(|i| {
        weapon::slot_weapon(w, id, i)
            .and_then(|wp| r.combat.warhead_of(wp))
            .is_some_and(|h| h.psychedelic)
    })
}

/// Whether building type `t` is a psychic sensor (a positive
/// psychic-detection radius, S-0610 claim 9).
fn sensor_type(w: &World, t: TypeRef) -> bool {
    t.kind == Kind::Building
        && w.rules()
            .buildings
            .get(t.index as usize)
            .is_some_and(|b| b.raw.econ.psychic_detection_radius.unwrap_or(0) > 0)
}

/// The cheapest psychic sensor type the house can build and place now, when
/// it has none (nor one on its line).
pub(crate) fn build_sensor(script: &Script, w: &World) -> Option<TypeRef> {
    let house = script.house;
    if !psychic::sensors(w, house).is_empty()
        || script
            .own(w)
            .iter()
            .any(|&id| w.obj(id).is_some_and(|o| sensor_type(w, tech::type_of(o))))
    {
        return None;
    }
    all_types(w)
        .into_iter()
        .filter(|&t| sensor_type(w, t))
        .filter(|&t| script.startable(w, t))
        .filter(|&t| script.place_cell(w, t).is_some())
        .filter(|&t| !script.busy(w, t))
        .min_by_key(|&t| (production::cost_field(w, t), t.index))
}

/// The psychic reveal's cell: the enemy start cell nearest home that the
/// house does not see now; when it sees them all, the unexplored cell (on a
/// lattice of [`LATTICE`] cells) nearest an enemy start, as a person
/// scouts what the map still hides; none when nothing is unexplored (it
/// keeps the reveal for later).
pub(crate) fn reveal_cell(script: &Script, w: &World) -> Option<CellXY> {
    let home = script.home(w)?;
    let starts = script.enemy_starts(w);
    if let Some(c) = starts
        .iter()
        .copied()
        .filter(|&c| !shroud::seen(w, script.house, c))
        .min_by_key(|&c| (dist2(c, home), c.y, c.x))
    {
        return Some(c);
    }
    let near = |c: CellXY| {
        starts
            .iter()
            .map(|&s| dist2(s, c))
            .min()
            .unwrap_or_else(|| dist2(home, c))
    };
    w.cells
        .grid
        .iter()
        .map(|(c, _)| c)
        .filter(|c| c.x % LATTICE == 0 && c.y % LATTICE == 0)
        .filter(|&c| w.cells.usable(c) && !shroud::explored(w, script.house, c))
        .min_by_key(|&c| (near(c), c.y, c.x))
}

/// The spacing of the cells the psychic reveal's scouting looks at.
const LATTICE: i32 = 4; // nocite: tuning of our own opponent

impl Special {
    /// Berserk weapons: every idle unit with a berserk weapon is sent at the
    /// seen enemy unit within [`REACH`] cells with the most seen enemy units
    /// within [`GROUP`] cells of it (then the nearest), that it may attack
    /// and that is not berserk already.
    pub(crate) fn berserk(
        &mut self,
        script: &Script,
        own: &[ObjectId],
        w: &World,
        out: &mut Vec<Order>,
    ) {
        let enemies: Vec<(ObjectId, CellXY)> = script
            .seen_enemies(w)
            .into_iter()
            .filter(|&(e, _)| w.obj(e).is_some_and(|o| o.kind != Kind::Building))
            .collect();
        if enemies.is_empty() {
            return;
        }
        for &id in own {
            // An idle unit, or one in guard that took the target nearest it
            // by itself: a person re-aims it at the group.
            let guarding = w.obj(id).is_some_and(|o| {
                o.dest.is_none()
                    && matches!(o.beh.current, Some(Behaviour::Guard | Behaviour::AreaGuard))
            });
            if !berserk_weapon(w, id)
                || psychic::berserk(w, id)
                || !(scripted::idle(w, id) || guarding)
                || !targeting::selectable(w, id)
            {
                continue;
            }
            let Some(at) = w.obj(id).map(|o| o.pos.cell()) else {
                continue;
            };
            let best = enemies
                .iter()
                .filter(|&&(e, c)| {
                    dist2(c, at) <= REACH * REACH
                        && !psychic::berserk(w, e)
                        && scripted::can_attack(w, id, e)
                        && self.memory.ok(w, id, Target::Object(e))
                })
                .map(|&(e, c)| {
                    let group = enemies
                        .iter()
                        .filter(|&&(o, oc)| o != e && dist2(oc, c) <= GROUP * GROUP)
                        .count();
                    (std::cmp::Reverse(group), dist2(c, at), e)
                })
                .min();
            let Some((_, _, e)) = best else { continue };
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
            self.count_berserk();
        }
    }
}
