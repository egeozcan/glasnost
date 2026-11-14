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
        // The line before the placement search, whose answer a busy line throws away (design: own AI, #64).
        .filter(|&t| !script.busy(w, t))
        .filter(|&t| script.place_cell(w, t).is_some())
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

/// The seen enemy units within [`GROUP`] cells of enemy `e` at `c`, of
/// `enemies`: its group, which a berserk weapon is aimed at.
fn group_of(enemies: &[(ObjectId, CellXY)], e: ObjectId, c: CellXY) -> usize {
    enemies
        .iter()
        .filter(|&&(o, oc)| o != e && dist2(oc, c) <= GROUP * GROUP)
        .count()
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
        // Each enemy's group, counted once for the call when first asked: it
        // does not hang on the unit (design: own AI, #64: it was counted
        // again for every unit and enemy).
        let mut groups: Vec<Option<usize>> = vec![None; enemies.len()];
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
                .enumerate()
                .filter(|&(_, &(e, c))| {
                    dist2(c, at) <= REACH * REACH
                        && !psychic::berserk(w, e)
                        && scripted::can_attack(w, id, e)
                        && self.memory.ok(w, id, Target::Object(e))
                })
                .map(|(i, &(e, c))| {
                    let group = *groups[i].get_or_insert_with(|| group_of(&enemies, e, c));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{fixture, game::Game};

    /// [`build_sensor`] as before #64: the placement searched before the
    /// line is asked.
    fn build_sensor_as_before(script: &Script, w: &World) -> Option<TypeRef> {
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

    /// The berserk targets as before #64: each enemy's group counted again
    /// for every unit and enemy.
    fn berserk_as_before(
        s: &mut Special,
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
                        && s.memory.ok(w, id, Target::Object(e))
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
            s.count_berserk();
        }
    }

    /// Our tooling for speed (design: own AI, #64): the sensor with the
    /// line asked first is the sensor as before, and the berserk orders
    /// with each group counted once are the orders as before, for every
    /// house at many points of the M6c, M7a and M7b duels.
    #[test]
    fn the_sensor_and_the_berserk_targets_are_as_before() {
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
        let (mut sensors, mut sent) = (0, 0);
        for (rules, map, map_name, setup) in games {
            let mut g = Game::new(rules, &map, map_name, fixture::catalogue(), setup)
                .expect("the duel loads");
            // At every tick: a berserk weapon is idle, or in guard on its own, only until the next think.
            for step in 0..9000 {
                g.step();
                let w = &g.world;
                for h in 0..w.houses.len() {
                    let house = glasnost_sim::world::HouseId(h as u8);
                    let script = Script::new(house, crate::brain::default_plan());
                    let at = format!("{map_name}, tick {}, {house:?}", w.tick);
                    if step % 25 == 0 {
                        let want = build_sensor_as_before(&script, w);
                        assert_eq!(build_sensor(&script, w), want, "{at}");
                        sensors += usize::from(want.is_some());
                    }
                    let own = script.own(w);
                    let (mut now, mut before) = (Special::default(), Special::default());
                    let (mut now_out, mut before_out) = (Vec::new(), Vec::new());
                    now.berserk(&script, &own, w, &mut now_out);
                    berserk_as_before(&mut before, &script, &own, w, &mut before_out);
                    assert_eq!(now_out, before_out, "{at}");
                    assert_eq!(format!("{now:?}"), format!("{before:?}"), "{at}");
                    sent += now_out.len();
                }
            }
        }
        assert!(sensors > 0 && sent > 0, "{sensors} sensors, {sent} sent");
    }
}
