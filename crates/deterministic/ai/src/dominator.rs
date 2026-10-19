//! The own AI and the psychic dominator (S-0612; PLAN §4.7).
//!
//! A launch takes over every object it reaches that is not a building,
//! not immune, not protected, not hovering like a balloon and not in the
//! air (S-0612 claims 11 to 14), whoever owns it: the house's own objects
//! and its allies' included (claim 13), and its area damage hurts them too
//! (claim 9). So a person fires it at a group of enemy units that it sees,
//! where the enemy value it would take is more than the own and allied
//! value inside the reach, and holds it while no group is worth it.
//!
//! - The building that gives the dominator is built by the brain's
//!   superweapon step, like every other kind it fires
//!   ([`crate::brain::Brain`], `build_superweapon`).
//! - [`dominate_cell`] picks the target cell: among the cells of seen enemy
//!   units, the one whose reach holds the most seen enemy value, at least
//!   [`MIN_UNITS`] enemy units that the sweep would take, and more enemy
//!   value than own and allied value (every own or allied unit inside
//!   counts, taken or merely hurt). The reach is what a person reads off
//!   the cursor and the map: the swept cells and layers of claim 11
//!   (`glasnost_sim::domination::swept`). Whether a unit can be taken is
//!   what a person sees of it: its type, a protection glow, a balloon, a
//!   unit in the air (`glasnost_sim::mind::dominable`).
//! - A launch while a domination is in force is refused (S-0612 claim 3); a
//!   person sees the domination and does not try.
//!
//! It reads only what a person in the house's seat sees and draws no random
//! value. The numbers are tuning of our own opponent, not game behaviour.

use std::collections::BTreeSet;

use glasnost_map::CellXY;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{Layer, Object, World};

use crate::scripted::Script;
use glasnost_sim::econ::tech;

/// The fewest seen enemy units the sweep would take for which the
/// dominator is fired.
pub const MIN_UNITS: usize = 3; // nocite: tuning of our own opponent

/// The value a person puts on unit `o`: its type's cost, at least 1.
fn value(w: &World, o: &Object) -> i64 {
    // nocite: tuning of our own opponent (a unit's worth is its price; a free unit still counts)
    i64::from(glasnost_sim::econ::production::cost_field(w, tech::type_of(o)).max(1))
}

/// The objects in play inside the reach of a launch at `c` (claim 11's
/// swept cells and layers).
pub(crate) fn reached(w: &World, c: CellXY) -> Vec<glasnost_sim::world::ObjectId> {
    let reach: BTreeSet<(i32, i32, Layer)> = glasnost_sim::domination::swept(w, c)
        .into_iter()
        .map(|(x, l)| (x.x, x.y, l))
        .collect();
    w.objects
        .values()
        .filter(|o| o.on_map && o.health > 0)
        .filter(|o| reach.contains(&(o.pos.cell().x, o.pos.cell().y, o.layer)))
        .map(|o| o.id)
        .collect()
}

/// The psychic dominator's target cell for the house of `script`, if a
/// group worth it is seen and no domination is in force. `spent` are the
/// units an earlier launch of the same think already reaches; they do not
/// count.
pub(crate) fn dominate_cell(
    script: &Script,
    w: &World,
    spent: &[glasnost_sim::world::ObjectId],
) -> Option<CellXY> {
    if glasnost_sim::domination::in_force(w) {
        return None;
    }
    let house = script.house;
    let me = &w.houses[usize::from(house.0)];
    let enemies: Vec<&Object> = script
        .seen_enemies(w)
        .into_iter()
        .filter_map(|(id, _)| w.obj(id))
        .filter(|o| o.kind != Kind::Building && !spent.contains(&o.id))
        .filter(|o| glasnost_sim::mind::dominable(w, o.id))
        .collect();
    if enemies.len() < MIN_UNITS {
        return None;
    }
    let friends: Vec<&Object> = w
        .objects
        .values()
        .filter(|o| o.on_map && o.health > 0 && o.kind != Kind::Building)
        .filter(|o| {
            o.owner
                .is_some_and(|h| h == house || me.allies.contains(&h))
        })
        .collect();
    let mut cands: Vec<CellXY> = enemies.iter().map(|o| o.pos.cell()).collect();
    cands.sort_by_key(|c| (c.y, c.x));
    cands.dedup();
    cands
        .into_iter()
        .filter_map(|c| {
            let reach: BTreeSet<(i32, i32, Layer)> = glasnost_sim::domination::swept(w, c)
                .into_iter()
                .map(|(x, l)| (x.x, x.y, l))
                .collect();
            let inside = |o: &&&Object| {
                let x = o.pos.cell();
                reach.contains(&(x.x, x.y, o.layer))
            };
            let taken: Vec<&&Object> = enemies.iter().filter(inside).collect();
            if taken.len() < MIN_UNITS {
                return None;
            }
            let gain: i64 = taken.iter().map(|o| value(w, o)).sum();
            let loss: i64 = friends.iter().filter(inside).map(|o| value(w, o)).sum();
            (gain > loss).then_some((gain - loss, c))
        })
        .max_by_key(|&(v, c)| (v, std::cmp::Reverse((c.y, c.x))))
        .map(|(_, c)| c)
}
