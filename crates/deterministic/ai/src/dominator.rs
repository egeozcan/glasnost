//! The own AI and the psychic dominator (S-0612; PLAN §4.7).
//!
//! A launch takes over every examined object it reaches that is not a
//! building, not immune, not protected, not hovering like a balloon and not
//! in the air (S-0612 claims 11 to 14), whoever owns it: allied objects
//! included (claim 13). The house's own objects change no owner (claim 14),
//! but the launch's area damage hurts them (claim 9), and every other
//! object in its radius. Buildings are never taken (claim 13); they count
//! only by the damage. A person fires a ready dominator at the best target
//! it sees rather than sitting on it.
//!
//! - The building that gives the dominator is built by the brain's
//!   superweapon step, like every other kind it fires
//!   ([`crate::brain::Brain`], `build_superweapon`).
//! - [`dominate_cell`] picks the target cell. What a launch at a cell is
//!   worth: gain, the value of the seen enemy units the sweep would take,
//!   and the part of their value that the area damage is expected to
//!   destroy of the seen enemy objects it does not take (buildings, which
//!   are never taken, refused units, and takeable units the damage itself
//!   destroys: claims 9, 10 and 13); loss, the allied value the sweep would
//!   take, and the part of the house's and its allies' value the area
//!   damage is expected to destroy (of what the sweep does not take). What the sweep takes is the simulation's own
//!   preview of claims 11 to 13 (`glasnost_sim::domination::preview`: the
//!   swept cells and layers, the objects examined from each cell's start
//!   object, and the takeover tests), cut to the enemy units the house sees:
//!   a person reads the reach off the cursor and sees what stands in it.
//! - Takeovers first (M8 step B3: in the stock lane every launch went to
//!   area damage on buildings while takeable units stood in sight
//!   elsewhere). A takeable unit is one that passes the takeover tests and
//!   stands still: one on the move has left the reach by the sweep. The
//!   aim searches the explored cells from which a sweep reaches a seen
//!   takeable unit, and fires at the one worth most among
//!   those that take at least one enemy unit and gain more than they lose.
//!   Only when no cell takes any does it aim for the damage alone, at the
//!   cells of the seen enemy objects, and only once the record has been
//!   ready for [`HOLD_PERCENT`] of its charge time: a person holds a ready
//!   dominator a while for a group to take, but never so long that a whole
//!   charge goes by for nothing.
//! - There is no smallest group (M7b AI round 3: in the stock lane the
//!   dominator stood ready for up to 4 000 ticks at a time with at most one
//!   dominable enemy unit in sight, and never fired, while the aim waited
//!   for [`MIN_UNITS`]).
//! - A launch while a domination is in force is refused (S-0612 claim 3); a
//!   person sees the domination and does not try.
//!
//! It reads only what a person in the house's seat sees and draws no random
//! value. The numbers are tuning of our own opponent, not game behaviour.

use std::collections::BTreeSet;

use glasnost_map::CellXY;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, Layer, Object, ObjectId, World};

use crate::scripted::Script;
use glasnost_sim::econ::tech;

/// The group size the dominator was once held for; kept as the size the
/// lane reports a group against. The aim itself fires at the best value it
/// sees, a smaller group included.
pub const MIN_UNITS: usize = 3; // nocite: tuning of our own opponent

/// How long a ready dominator is held for a group to take before it is
/// fired for its area damage alone, in percent of the type's charge time.
/// Below 100, so a hold never costs a whole charge.
pub const HOLD_PERCENT: i32 = 50; // nocite: tuning of our own opponent

/// The value a person puts on object `o`: its type's cost, at least 1.
pub(crate) fn value(w: &World, o: &Object) -> i64 {
    // nocite: tuning of our own opponent (a unit's worth is its price; a free unit still counts)
    i64::from(glasnost_sim::econ::production::cost_field(w, tech::type_of(o)).max(1))
}

/// The objects in play inside the reach of a launch at `c` (claim 11's
/// swept cells and layers).
pub(crate) fn reached(w: &World, c: CellXY) -> Vec<ObjectId> {
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

/// Where an area hit lands and the layer of the objects it hits: the point
/// `at`, and `layer`, the ground or a bridge deck (S-0303 claim 4).
#[derive(Clone, Copy)]
pub(crate) struct Blast {
    pub at: glasnost_sim::world::Pos,
    pub layer: Layer,
}

impl Blast {
    /// The dominator's blast at cell `c`: the centre of the cell at its
    /// terrain surface, ground layer.
    pub(crate) fn ground(w: &World, c: CellXY) -> Blast {
        Blast {
            // spec: S-0612 claim 9 (the centre of the target cell at its terrain surface; the ground layer)
            at: glasnost_sim::superweapons::cell_point(w, c, false),
            layer: Layer::Ground,
        }
    }
}

/// The value of the objects of `owners` that an area hit of `amount` with
/// `warhead` at `blast` is expected to destroy (infantry only with
/// `infantry`): each object of the blast's layer within the warhead's spread radius loses the part
/// of its value that the falloff and versus step takes of its strength
/// (S-0303 F1, S-0302 F3), and all of it when that is at least its health. An estimate a person makes
/// from the blast's size and what stands in it; the defences and the
/// height of each object are left out.
pub(crate) fn blast_loss(
    w: &World,
    owners: &[HouseId],
    blast: Blast,
    amount: i32,
    warhead: Option<glasnost_sim::combat::data::WarheadId>,
    infantry: bool,
) -> i64 {
    blast_value(w, blast, amount, warhead, |o| {
        o.owner.is_some_and(|x| owners.contains(&x)) && (infantry || o.kind != Kind::Infantry)
    })
}

/// The value of the objects passing `count` that an area hit of `amount`
/// with `warhead` at `blast` is expected to destroy (see [`blast_loss`]).
pub(crate) fn blast_value(
    w: &World,
    blast: Blast,
    amount: i32,
    warhead: Option<glasnost_sim::combat::data::WarheadId>,
    count: impl Fn(&Object) -> bool,
) -> i64 {
    let Some(hit) = Hit::of(w, amount, warhead) else {
        return 0;
    };
    hit_value(
        w,
        blast,
        hit,
        w.objects
            .values()
            .filter(|o| o.on_map && o.health > 0 && o.kind != Kind::Terrain)
            .filter(|o| count(o)),
    )
}

/// The value of `objects` that `hit` at `blast` is expected to destroy
/// ([`blast_value`]'s sum).
fn hit_value<'o>(
    w: &World,
    blast: Blast,
    hit: Hit,
    objects: impl Iterator<Item = &'o Object>,
) -> i64 {
    objects
        .filter_map(|o| {
            let dmg = hit.on(w, blast, o)?;
            if dmg >= o.health {
                // Destroyed: its whole value.
                return Some(value(w, o));
            }
            let strength = glasnost_sim::combat::damage::strength(w, o.id).max(1);
            Some(value(w, o) * i64::from(dmg.max(0)) / i64::from(strength))
        })
        .sum()
}

/// An area hit of `amount` with `warhead`, with the warhead's spread radius
/// R (S-0303 F1) found once for all the objects it is tried on (design: own
/// AI, #55: finding R for every object was most of the aim's time).
#[derive(Clone, Copy)]
struct Hit {
    amount: i32,
    warhead: glasnost_sim::combat::data::WarheadId,
    r: i32,
}

impl Hit {
    /// The hit, or none with no warhead or no damage (it hurts nothing).
    fn of(
        w: &World,
        amount: i32,
        warhead: Option<glasnost_sim::combat::data::WarheadId>,
    ) -> Option<Hit> {
        let h = warhead?;
        if amount <= 0 {
            return None;
        }
        let (r, _) = glasnost_sim::combat::area::radii(w, h);
        Some(Hit {
            amount,
            warhead: h,
            r,
        })
    }

    /// The damage the hit at `blast` is expected to deal to `o` (see
    /// [`hit_within`]).
    fn on(self, w: &World, blast: Blast, o: &Object) -> Option<i32> {
        hit_within(w, blast, self.amount, self.warhead, self.r, o)
    }
}

/// The damage an area hit of a positive `amount` with warhead `h`, whose
/// spread radius is `r`, at `blast` is expected to deal to `o` (S-0303 F1,
/// S-0302 F3), or `None` when `o` is outside the radius or not in the
/// blast's layer. An estimate a person makes (see [`blast_loss`]).
fn hit_within(
    w: &World,
    blast: Blast,
    amount: i32,
    h: glasnost_sim::combat::data::WarheadId,
    r: i32,
    o: &Object,
) -> Option<i32> {
    // spec: S-0303 claim 4 (one layer only: the deck's on a bridge deck, else the ground's), S-0612 claim 9 (no flying objects)
    // design: own AI (#34): the layer is the blast's, not always the ground
    if o.layer != blast.layer || glasnost_sim::combat::weapon::in_air(w, o.id) {
        return None;
    }
    let p = blast.at;
    // spec: S-0303 claims 7, 8 and 10 (the distance of each hit) and claim 3 (only within R)
    let hits: Vec<i32> = glasnost_sim::combat::area::object_distances(w, p, o.id)
        .into_iter()
        .filter(|&d| d <= r)
        .collect();
    if hits.is_empty() {
        return None;
    }
    // spec: S-0302 claims 5 and 6 (the defended and modified amount of each hit)
    Some(
        hits.into_iter()
            .map(|d| glasnost_sim::combat::damage::hit_estimate(w, o.id, amount, h, d))
            .sum(),
    )
}

/// What a launch at `c` is worth to `house`: the value of the seen enemy
/// units of `takeable` it would take (gain), and the allied value it would
/// take plus the own value its area damage would destroy (loss), and the
/// number of enemy units taken. A takeable unit the area damage is expected
/// to destroy is not taken (S-0612 claim 10): it counts by the damage.
fn worth(
    w: &World,
    aim: &Aim,
    c: CellXY,
    takeable: &BTreeSet<ObjectId>,
    seen: &BTreeSet<ObjectId>,
) -> (i64, i64, usize) {
    let house = aim.house;
    let me = &w.houses[usize::from(house.0)];
    let taken = glasnost_sim::domination::preview(w, house, c);
    let (mut gain, mut loss, mut n) = (0, 0, 0);
    let mut destroyed: BTreeSet<ObjectId> = BTreeSet::new();
    // The allied objects the sweep takes: counted whole, not again by the damage.
    let mut ally_taken: BTreeSet<ObjectId> = BTreeSet::new();
    let blast = Blast::ground(w, c);
    let wrecks = wrecks(w, aim, blast);
    for id in taken {
        let Some(o) = w.obj(id) else { continue };
        if takeable.contains(&id) {
            let hit = aim.hit.and_then(|h| h.on(w, blast, o));
            // spec: S-0303 claim 11 (a death weapon a hit sets off runs in the middle of the detonation's hits)
            let chained = wrecks && hit.is_some();
            if chained || hit.is_some_and(|x| x >= o.health) {
                // spec: S-0612 claim 10 (an object the area damage destroys is not taken over)
                destroyed.insert(id);
                continue;
            }
            gain += value(w, o);
            n += 1;
        } else if o.owner.is_some_and(|x| x != house && me.allied_with(x)) {
            // spec: S-0612 claim 13 (allied objects are taken over)
            loss += value(w, o);
            ally_taken.insert(id);
        }
    }
    // spec: S-0612 claim 14 (own objects change no owner) and claim 9 (the area damage, of the imported amount and warhead)
    // design: own AI (#33): the damage to the allies' objects the sweep does not take is loss too, as the mutator counts it
    loss += aim.value(w, blast, |o| {
        o.owner
            .is_some_and(|x| x == house || (me.allied_with(x) && !ally_taken.contains(&o.id)))
    });
    // spec: S-0612 claims 9 and 10 (the area damage hurts the seen enemy objects that are not taken: buildings, which are never taken (claim 13), units the takeover tests refuse, and takeable units it destroys)
    gain += aim.value(w, blast, |o| {
        seen.contains(&o.id) && (!takeable.contains(&o.id) || destroyed.contains(&o.id))
    });
    (gain, loss, n)
}

/// What the aim of one search reads once for all its cells (design: own
/// AI, #55: three scans of every object a cell, each finding the
/// warhead's spread radius again for every object, were most of the aim's
/// time): the dominator's area hit, and the objects whose damage the aim
/// counts.
struct Aim<'w> {
    house: HouseId,
    /// The dominator's area hit (S-0612 claim 9), none when it hurts
    /// nothing.
    hit: Option<Hit>,
    /// The objects in play, not terrain, on the ground layer of the blast
    /// ([`Blast::ground`]) and not in the air, that are the house's, its
    /// allies' or seen enemies (`seen`): every object [`wrecks`] and
    /// [`worth`]'s damage count can hurt.
    objects: Vec<&'w Object>,
}

impl<'w> Aim<'w> {
    /// The aim of `house`, with `seen` its seen enemy objects.
    fn new(w: &'w World, house: HouseId, seen: &BTreeSet<ObjectId>) -> Aim<'w> {
        let me = &w.houses[usize::from(house.0)];
        let d = &w.rules().superweapons.domination;
        let hit = Hit::of(w, d.damage, d.warhead);
        let objects = if hit.is_some() {
            w.objects
                .values()
                .filter(|o| o.on_map && o.health > 0 && o.kind != Kind::Terrain)
                .filter(|o| {
                    seen.contains(&o.id) || o.owner.is_some_and(|x| x == house || me.allied_with(x))
                })
                // spec: S-0303 claim 4 and S-0612 claim 9 (the dominator's blast hits the ground layer and no flying object)
                .filter(|o| {
                    o.layer == Layer::Ground && !glasnost_sim::combat::weapon::in_air(w, o.id)
                })
                .collect()
        } else {
            Vec::new()
        };
        Aim {
            house,
            hit,
            objects,
        }
    }

    /// [`blast_value`] of the aim's hit at `blast` over its objects that
    /// pass `count`.
    fn value(&self, w: &World, blast: Blast, count: impl Fn(&Object) -> bool) -> i64 {
        self.hit.map_or(0, |hit| {
            hit_value(
                w,
                blast,
                hit,
                self.objects.iter().copied().filter(|o| count(o)),
            )
        })
    }
}

/// Whether the area damage of a launch at `c` is expected to destroy any
/// object the house sees (its own, its allies' or a seen enemy's). A
/// destroyed object's death weapon goes off in the middle of the
/// detonation (S-0303 claim 11), so the aim does not count on taking a
/// unit inside the blast then: in the stock lane (M8 step B3) units the
/// estimate spared were destroyed that way. `blast` is the launch's
/// ([`Blast::ground`] at `c`), and the objects are the aim's.
fn wrecks(w: &World, aim: &Aim, blast: Blast) -> bool {
    aim.hit.is_some_and(|hit| {
        aim.objects
            .iter()
            .any(|o| hit.on(w, blast, o).is_some_and(|x| x >= o.health))
    })
}

/// The seen enemy units a launch could take: not buildings (S-0612
/// claim 13), not `spent`, passing the takeover tests a person reads off
/// them (`glasnost_sim::mind::dominable`), and standing still (see
/// [`standing`]).
fn takeable(script: &Script, w: &World, spent: &[ObjectId]) -> BTreeSet<ObjectId> {
    dominable(script, w, spent)
        .into_iter()
        .filter(|&id| standing(w, id))
        .collect()
}

/// The seen enemy units, not `spent`, that pass the takeover tests: not
/// buildings (S-0612 claim 13) and `glasnost_sim::mind::dominable`.
fn dominable(script: &Script, w: &World, spent: &[ObjectId]) -> BTreeSet<ObjectId> {
    script
        .seen_enemies(w)
        .into_iter()
        .filter_map(|(id, _)| w.obj(id))
        // spec: S-0612 claim 13 (a building is never taken)
        .filter(|o| o.kind != Kind::Building && !spent.contains(&o.id))
        .filter(|o| glasnost_sim::mind::dominable(w, o.id))
        .map(|o| o.id)
        .collect()
}

/// Whether a unit stands still, as a person sees it: it has no movement
/// destination. The sweep comes some time after the launch (S-0612
/// claim 4), and in the stock lane (M8 step B3) units on the move had
/// left the reach by then; a person aims at the units that stand.
fn standing(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.dest.is_none())
}

/// The best cell for a launch by the house of `script` and what it is
/// worth: `(cell, gain, loss, enemy units taken)`. A cell that takes at
/// least one enemy unit beats every cell that takes none (the damage
/// alone).
fn best(script: &Script, w: &World, spent: &[ObjectId]) -> Option<(CellXY, i64, i64, usize)> {
    best_take(script, w, spent).or_else(|| best_damage(script, w, spent))
}

/// The seen enemy objects, less `spent`.
fn seen(script: &Script, w: &World, spent: &[ObjectId]) -> BTreeSet<ObjectId> {
    script
        .seen_enemies(w)
        .into_iter()
        .map(|(id, _)| id)
        .filter(|id| !spent.contains(id))
        .collect()
}

/// The best of `cands` by gain less loss, the earlier cell in scan order on
/// a tie, among those whose gain is larger than their loss and that take
/// at least `min` enemy units.
fn pick(
    script: &Script,
    w: &World,
    cands: Vec<CellXY>,
    takeable: &BTreeSet<ObjectId>,
    seen: &BTreeSet<ObjectId>,
    min: usize,
) -> Option<(CellXY, i64, i64, usize)> {
    if cands.is_empty() {
        return None;
    }
    let aim = Aim::new(w, script.house, seen);
    cands
        .into_iter()
        .map(|c| {
            let (gain, loss, n) = worth(w, &aim, c, takeable, seen);
            (c, gain, loss, n)
        })
        .filter(|&(_, gain, loss, n)| gain > loss && n >= min)
        .max_by_key(|&(c, gain, loss, _)| (gain - loss, std::cmp::Reverse((c.y, c.x))))
}

/// The best cell whose sweep takes at least one seen enemy unit, among the
/// explored cells from which the sweep reaches a seen takeable unit (a
/// person aims anywhere it has seen, and the house's shroud is all it
/// knows of the map).
fn best_take(script: &Script, w: &World, spent: &[ObjectId]) -> Option<(CellXY, i64, i64, usize)> {
    let takeable = takeable(script, w, spent);
    if takeable.is_empty() {
        return None;
    }
    let seen = seen(script, w, spent);
    let mut cands: Vec<CellXY> = takeable
        .iter()
        .filter_map(|&id| w.obj(id))
        // spec: S-0612 claim 11 and S-0303 Terms (the swept cells around the unit's cell; the ring metric m(Δx, Δy) depends only on |Δx| and |Δy|, so these are the cells whose sweep reaches it)
        .flat_map(|o| glasnost_sim::domination::swept(w, o.pos.cell()))
        .map(|(c, _)| c)
        .filter(|&c| glasnost_sim::econ::shroud::explored(w, script.house, c))
        .collect();
    cands.sort_by_key(|c| (c.y, c.x));
    cands.dedup();
    pick(script, w, cands, &takeable, &seen, 1)
}

/// The best cell for the area damage alone, among the cells of the seen
/// enemy objects.
fn best_damage(
    script: &Script,
    w: &World,
    spent: &[ObjectId],
) -> Option<(CellXY, i64, i64, usize)> {
    let takeable = takeable(script, w, spent);
    let seen = seen(script, w, spent);
    let mut cands: Vec<CellXY> = seen
        .iter()
        .filter_map(|&id| w.obj(id))
        .map(|o| o.pos.cell())
        .collect();
    cands.sort_by_key(|c| (c.y, c.x));
    cands.dedup();
    pick(script, w, cands, &takeable, &seen, 0)
}

/// Whether record `sw` of `house` has been ready for less than
/// [`HOLD_PERCENT`] of its type's charge time (S-0512 claim 5: the ready
/// tick; a person sees how long its icon has been ready).
fn holding(w: &World, house: HouseId, sw: u32) -> bool {
    let Some(r) = glasnost_sim::superweapons::charge::get(w, house, sw) else {
        return false;
    };
    let Some(t) = w.rules().superweapons.types.get(sw as usize) else {
        return false;
    };
    // nocite: tuning of our own opponent (the hold, a part of one charge time)
    let hold = i64::from(t.charge.max(0)) * i64::from(HOLD_PERCENT) / 100;
    i64::from(w.tick) - i64::from(r.ready_tick) < hold
}

/// The psychic dominator's target cell for the house of `script` and its
/// record `sw`, if a target worth it is seen and no domination is in
/// force: a cell that takes at least one seen enemy unit, else, once the
/// hold is over ([`HOLD_PERCENT`]), the best cell for the damage alone.
/// `spent` are the units an earlier launch of the same think already
/// reaches; they do not count.
pub(crate) fn dominate_cell(
    script: &Script,
    w: &World,
    sw: u32,
    spent: &[ObjectId],
) -> Option<CellXY> {
    // spec: S-0612 claim 3 (a launch while a domination is in force is refused)
    if glasnost_sim::domination::in_force(w) {
        return None;
    }
    if let Some((c, ..)) = best_take(script, w, spent) {
        return Some(c);
    }
    if holding(w, script.house, sw) {
        return None;
    }
    best_damage(script, w, spent).map(|(c, ..)| c)
}

/// What the lane's dominator probe reports for a house.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Probe {
    /// Seen enemy objects.
    pub seen: usize,
    /// Seen enemy units (not buildings).
    pub units: usize,
    /// Seen enemy units that pass the takeover tests.
    pub dominable: usize,
    /// Of those, the units on the move, which the aim does not count as
    /// takes.
    pub moving: usize,
    /// Why the other seen units are refused, one entry per unit: `air`,
    /// `immune`, `protected`, `balloon`.
    pub refused: Vec<&'static str>,
    /// Takeable units that no launch at their own cell examines (S-0612
    /// claim 12: standing in a building's cells).
    pub not_examined: usize,
    /// The best cell's gain, loss and enemy units taken, if any is worth
    /// firing at.
    pub best: Option<(i64, i64, usize)>,
}

/// A lane diagnostic of the dominator's aim for `house`.
pub fn probe(w: &World, house: HouseId) -> Probe {
    let script = Script::new(house, crate::brain::default_plan());
    let seen: Vec<&Object> = script
        .seen_enemies(w)
        .into_iter()
        .filter_map(|(id, _)| w.obj(id))
        .collect();
    let units: Vec<&&Object> = seen.iter().filter(|o| o.kind != Kind::Building).collect();
    let refused = units
        .iter()
        .filter(|o| !glasnost_sim::mind::dominable(w, o.id))
        .map(|o| {
            if glasnost_sim::combat::weapon::in_air(w, o.id) {
                "air"
            } else if glasnost_sim::combat::veterancy::type_combat(w, o.id)
                .is_some_and(|t| t.immune_psionics)
            {
                "immune"
            } else if glasnost_sim::superweapons::protected(w, o.id) {
                "protected"
            } else {
                "balloon"
            }
        })
        .collect();
    let dominable = dominable(&script, w, &[]);
    let takeable = takeable(&script, w, &[]);
    let not_examined = takeable
        .iter()
        .filter(|&&id| {
            w.obj(id).is_some_and(|o| {
                !glasnost_sim::domination::preview(w, house, o.pos.cell()).contains(&id)
            })
        })
        .count();
    Probe {
        seen: seen.len(),
        units: units.len(),
        dominable: dominable.len(),
        moving: dominable.len() - takeable.len(),
        refused,
        not_examined,
        best: best(&script, w, &[]).map(|(_, g, l, n)| (g, l, n)),
    }
}

#[cfg(test)]
mod aim_tests {
    use super::*;
    use crate::{fixture, game::Game};
    use glasnost_sim::world::HouseId;

    /// The area hit as before #55: the spread radius found for every
    /// object.
    fn blast_hit_as_before(
        w: &World,
        blast: Blast,
        amount: i32,
        warhead: Option<glasnost_sim::combat::data::WarheadId>,
        o: &Object,
    ) -> Option<i32> {
        let h = warhead?;
        if amount <= 0 {
            return None;
        }
        if o.layer != blast.layer || glasnost_sim::combat::weapon::in_air(w, o.id) {
            return None;
        }
        let (r, _) = glasnost_sim::combat::area::radii(w, h);
        let hits: Vec<i32> = glasnost_sim::combat::area::object_distances(w, blast.at, o.id)
            .into_iter()
            .filter(|&d| d <= r)
            .collect();
        if hits.is_empty() {
            return None;
        }
        Some(
            hits.into_iter()
                .map(|d| glasnost_sim::combat::damage::hit_estimate(w, o.id, amount, h, d))
                .sum(),
        )
    }

    /// [`blast_value`] as before #55: every object scanned, the radius
    /// found for each.
    fn blast_value_as_before(
        w: &World,
        blast: Blast,
        amount: i32,
        warhead: Option<glasnost_sim::combat::data::WarheadId>,
        count: impl Fn(&Object) -> bool,
    ) -> i64 {
        w.objects
            .values()
            .filter(|o| o.on_map && o.health > 0 && o.kind != Kind::Terrain)
            .filter(|o| count(o))
            .filter_map(|o| {
                let dmg = blast_hit_as_before(w, blast, amount, warhead, o)?;
                if dmg >= o.health {
                    return Some(value(w, o));
                }
                let strength = glasnost_sim::combat::damage::strength(w, o.id).max(1);
                Some(value(w, o) * i64::from(dmg.max(0)) / i64::from(strength))
            })
            .sum()
    }

    /// [`wrecks`] as before #55.
    fn wrecks_as_before(w: &World, house: HouseId, c: CellXY, seen: &BTreeSet<ObjectId>) -> bool {
        let me = &w.houses[usize::from(house.0)];
        let d = &w.rules().superweapons.domination;
        let blast = Blast::ground(w, c);
        w.objects
            .values()
            .filter(|o| o.on_map && o.health > 0 && o.kind != Kind::Terrain)
            .filter(|o| {
                seen.contains(&o.id) || o.owner.is_some_and(|x| x == house || me.allied_with(x))
            })
            .any(|o| {
                blast_hit_as_before(w, blast, d.damage, d.warhead, o).is_some_and(|x| x >= o.health)
            })
    }

    /// [`worth`] as before #55.
    fn worth_as_before(
        w: &World,
        house: HouseId,
        c: CellXY,
        takeable: &BTreeSet<ObjectId>,
        seen: &BTreeSet<ObjectId>,
    ) -> (i64, i64, usize) {
        let me = &w.houses[usize::from(house.0)];
        let d = &w.rules().superweapons.domination;
        let taken = glasnost_sim::domination::preview(w, house, c);
        let (mut gain, mut loss, mut n) = (0, 0, 0);
        let mut destroyed: BTreeSet<ObjectId> = BTreeSet::new();
        let mut ally_taken: BTreeSet<ObjectId> = BTreeSet::new();
        let wrecks = wrecks_as_before(w, house, c, seen);
        let blast = Blast::ground(w, c);
        for id in taken {
            let Some(o) = w.obj(id) else { continue };
            if takeable.contains(&id) {
                let hit = blast_hit_as_before(w, blast, d.damage, d.warhead, o);
                let chained = wrecks && hit.is_some();
                if chained || hit.is_some_and(|x| x >= o.health) {
                    destroyed.insert(id);
                    continue;
                }
                gain += value(w, o);
                n += 1;
            } else if o.owner.is_some_and(|x| x != house && me.allied_with(x)) {
                loss += value(w, o);
                ally_taken.insert(id);
            }
        }
        loss += blast_value_as_before(w, blast, d.damage, d.warhead, |o| {
            o.owner
                .is_some_and(|x| x == house || (me.allied_with(x) && !ally_taken.contains(&o.id)))
        });
        gain += blast_value_as_before(w, blast, d.damage, d.warhead, |o| {
            seen.contains(&o.id) && (!takeable.contains(&o.id) || destroyed.contains(&o.id))
        });
        (gain, loss, n)
    }

    /// Our tooling for speed (design: own AI, #55): what a launch is worth,
    /// read off the aim's objects with the spread radius found once, is
    /// what every object scanned with the radius found for each gives, and
    /// so is the area damage's value for the mutator; at the cells of the
    /// seen enemies and around the takeable ones, for every house, at many
    /// points of the M6c, M7a and M7b duels.
    #[test]
    fn the_aim_read_once_is_the_aim_of_every_object() {
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
        let (mut compared, mut valued) = (0, 0);
        for (rules, map, map_name, setup) in games {
            let mut g = Game::new(rules, &map, map_name, fixture::catalogue(), setup)
                .expect("the duel loads");
            for _ in 0..40 {
                for _ in 0..225 {
                    g.step();
                }
                let w = &g.world;
                for h in 0..w.houses.len() {
                    let house = HouseId(h as u8);
                    let script = Script::new(house, crate::brain::default_plan());
                    let takeable = takeable(&script, w, &[]);
                    let seen = seen(&script, w, &[]);
                    let aim = Aim::new(w, house, &seen);
                    let mut cells: Vec<CellXY> = seen
                        .iter()
                        .filter_map(|&id| w.obj(id))
                        .map(|o| o.pos.cell())
                        .chain(
                            takeable
                                .iter()
                                .filter_map(|&id| w.obj(id))
                                .flat_map(|o| glasnost_sim::domination::swept(w, o.pos.cell()))
                                .map(|(c, _)| c)
                                .step_by(7),
                        )
                        .collect();
                    cells.sort_by_key(|c| (c.y, c.x));
                    cells.dedup();
                    let d = &w.rules().superweapons.domination;
                    for c in cells {
                        let at = format!("{map_name}, tick {}, house {h}, {c:?}", w.tick);
                        let want = worth_as_before(w, house, c, &takeable, &seen);
                        assert_eq!(worth(w, &aim, c, &takeable, &seen), want, "{at}");
                        let blast = Blast::ground(w, c);
                        let mine = |o: &Object| o.owner == Some(house);
                        assert_eq!(
                            blast_value(w, blast, d.damage, d.warhead, mine),
                            blast_value_as_before(w, blast, d.damage, d.warhead, mine),
                            "{at}"
                        );
                        compared += 1;
                        valued += usize::from(want.0 > 0 || want.1 > 0);
                    }
                }
            }
        }
        assert!(valued > 0 && valued < compared, "{valued} of {compared}");
    }
}
