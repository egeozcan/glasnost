//! Healing and repair in combat (S-0314): self-healing objects, the
//! house-wide self-healing of infantry and vehicles, and the repair step
//! with its size and cost (claims 3 to 5, 8, F2). What asks for a step
//! (repair depots, hospitals, the repair order of a building) needs
//! docking, entering and the structures and economy topics and is not
//! wired up; money goes through [`crate::seams`]. Healers' shots are hits
//! (S-0302).

use glasnost_num::F64;

use crate::combat::data::Ability;
use crate::combat::veterancy::{has_ability, type_combat};
use crate::rules::Kind;
use crate::world::{HouseId, ObjectId, World};

/// "A multiple of" a period, treating 0 as 1 and a negative period by its
/// magnitude (S-0314 Edge cases, a departure for 0).
pub(crate) fn multiple(t: i32, p: i32) -> bool {
    // spec: S-0314 Edge cases (a period of 0 is treated as 1 tick; a negative one behaves as its absolute value)
    let p = p.unsigned_abs().max(1) as i64;
    i64::from(t).rem_euclid(p) == 0
}

/// One object's self-healing in its turn (S-0314 claims 9 and 10, F3).
pub fn turn(w: &mut World, id: ObjectId) {
    let t = w.tick;
    let Some(o) = w.obj(id) else { return };
    // spec: S-0302 claim 16 (a building in a delayed-kill countdown is healed as claim 11 gives)
    if crate::combat::damage::dead(o) {
        return;
    }
    let (h, kind, owner) = (o.health, o.kind, o.owner);
    let rules = w.rules_arc();
    let g = &rules.combat.general;
    let s = crate::combat::damage::strength(w, id);
    // spec: S-0314 F3 (an object at 0 health or at its strength gains nothing)
    if h <= 0 || h >= s {
        return;
    }
    let self_heal = type_combat(w, id).is_some_and(|tc| tc.self_healing)
        || has_ability(w, id, Ability::SelfHeal);
    // spec: S-0314 claim 9 (1 health at each tick that is a multiple of the building repair period P_b)
    let g9 = i32::from(self_heal && multiple(t, g.repair_period));
    let organic = type_combat(w, id).is_some_and(|tc| tc.organic);
    // spec: S-0314 claim 10 (infantry and organic vehicles use the infantry total; other vehicles the vehicle total; not aircraft or buildings)
    let which = match kind {
        Kind::Infantry => Some(0),
        Kind::Vehicle if organic => Some(0),
        Kind::Vehicle => Some(1),
        _ => None,
    };
    let g10 = which.map_or(0, |i| {
        let total = owner
            .and_then(|hh| w.houses.get(usize::from(hh.0)))
            .map_or(0, |hh| hh.combat.self_heal[i]);
        // spec: S-0314 F3 (a′ = a, or 0 when a is negative, a departure)
        let a = g.self_heal_amount[i].max(0);
        if total > 0 && multiple(t, g.self_heal_frames[i]) {
            a.wrapping_mul(total)
        } else {
            0
        }
    });
    if g9 == 0 && g10 == 0 {
        return;
    }
    // spec: S-0314 F3 (health at the end of the tick = min(H + g₉ + g₁₀, S))
    let h2 = (i64::from(h) + i64::from(g9) + i64::from(g10)).min(i64::from(s)) as i32;
    if let Some(ob) = w.obj_mut(id) {
        ob.health = h2;
    }
    // spec: S-0314 claim 9 and S-0616 claim 1 (after the object's own self-healing gain, its damage smoke takes the healing-mark test)
    let own = g9 > 0;
    // spec: S-0314 claim 10 and S-0616 claim 1 (after house-wide healing only a vehicle that is not organic takes the test; infantry and organic vehicles do not)
    let house = g10 > 0 && kind == Kind::Vehicle && !organic;
    if own || house {
        smoke_heal_mark(w, id, h2, s);
    }
}

/// The damage-smoke depth of the healing mark, leptons (S-0616 claim 1:
/// the mark applies below −10).
const SMOKE_MARK_DEPTH: i32 = -10; // spec: S-0616 Constants (damage-smoke healing-mark depth 10 leptons below the surface)

/// S-0616 claim 1 (with S-0314 claims 9 and 10): after a self-healing gain
/// to health `h` of strength `s`, an existing damage-smoke system is marked
/// when the health ratio is strictly above the yellow ratio or the object
/// is below −10 leptons above the surface. The object keeps it in its
/// place until it leaves play.
fn smoke_heal_mark(w: &mut World, id: ObjectId, h: i32, s: i32) {
    let Some(sys) = w.obj(id).and_then(|o| o.combat.damage_smoke) else {
        return;
    };
    let m = w.rules().mode;
    // spec: S-0214 Terms (health ratio: health / strength in floating point)
    let ratio = F64::from_i64(i64::from(h), m).div(F64::from_i64(i64::from(s), m), m);
    // spec: S-0616 claim 1 and S-0314 claim 9 (strictly above the game-wide yellow ratio, or the height above the surface below −10 leptons)
    if ratio.gt(w.rules().general.condition_yellow) || w.height_above_surface(id) < SMOKE_MARK_DEPTH
    {
        crate::particles::mark(w, sys);
    }
}

/// A building's self-heal counts join or leave its house's totals (S-0314
/// claim 10): added when placed or captured, subtracted (a total below 0
/// set to 0) when removed, sold, destroyed or captured away.
pub fn building_counts(w: &mut World, id: ObjectId, owner: Option<HouseId>, add: bool) {
    let Some(o) = w.obj(id) else { return };
    if o.kind != Kind::Building {
        return;
    }
    let rules = w.rules_arc();
    let b = &rules.buildings[o.type_index as usize].building;
    let neg = rules.quirks.negative_self_heal_counts;
    // spec: S-0314 Edge cases (negative_self_heal_counts off: a negative count is read as 0)
    let count = |c: i32| if neg { c } else { c.max(0) };
    let counts = [count(b.infantry_self_heal), count(b.units_self_heal)];
    let Some(h) = owner.and_then(|h| w.houses.get_mut(usize::from(h.0))) else {
        return;
    };
    for (i, c) in counts.iter().enumerate() {
        let tot = &mut h.combat.self_heal[i];
        if add {
            // spec: S-0314 claim 10 (an addition is never floored)
            *tot = tot.wrapping_add(*c);
        } else {
            // spec: S-0314 claim 10 (a subtraction floors the total at 0)
            *tot = tot.wrapping_sub(*c).max(0);
        }
    }
}

/// The outcome of a repair step (S-0314 claim 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepOutcome {
    /// The object was at full health; nothing changed.
    NotNeeded,
    /// The paying house could not pay; nothing changed.
    CannotPay,
    /// Paid; health rose by the step size and is still below the strength.
    MoreNeeded,
    /// Paid; health is the strength.
    Done,
}

/// The step size of an object (S-0314 claim 4): the game-wide infantry
/// repair step for infantry, the game-wide repair step for every other
/// object, at least 1.
pub fn step_size(w: &World, id: ObjectId) -> i32 {
    let g = &w.rules().combat.general;
    let infantry = w.obj(id).is_some_and(|o| o.kind == Kind::Infantry);
    // spec: S-0314 claim 4 (infantry: the infantry repair step; others: the repair step; a size below 1 counts as 1)
    let s = if infantry {
        g.i_repair_step
    } else {
        g.repair_step
    };
    s.max(1)
}

/// The step cost of F2 (S-0314 claim 5) for cost `c`, strength `s`, the
/// game-wide repair step `rs` and repair percentage `p`, with our engine's
/// departure for a step below 1 or a strength below the step (S-0314 F2,
/// Edge cases).
pub fn f2_step_cost(
    c: i32,
    s: i32,
    rs: i32,
    p: glasnost_num::F64,
    mode: glasnost_num::Rounding,
) -> i32 {
    // spec: S-0314 F2 and Edge cases (our engine: R′ = max(1, R), q = max(1, S ÷ R′))
    let r = rs.max(1);
    let q = (s / r).max(1);
    // spec: S-0314 F2 (c = C ÷ q truncated toward zero; step cost = max(1, trunc(RZ₅₃(c × p))))
    let cq = c / q;
    let v = glasnost_num::F64::from_i64(i64::from(cq), mode)
        .mul(p, mode)
        .to_i32_common();
    v.max(1)
}

/// The step cost of an object (S-0314 claim 5): 0 for infantry, else F2
/// on the type's cost field without house or country factors; for a
/// building type the adjusted cost of S-0408 claim 2. `None` only for
/// objects that have no step (terrain).
pub fn step_cost(w: &World, id: ObjectId) -> Option<i32> {
    let o = w.obj(id)?;
    let r = w.rules();
    let g = &r.combat.general;
    let cost = match o.kind {
        // spec: S-0314 claim 5 (an infantry unit's step costs 0)
        Kind::Infantry => return Some(0),
        // spec: S-0314 claim 5 (a building type: the adjusted cost of S-0408 claim 2)
        Kind::Building => crate::seams::building_adjusted_cost(w, o.type_index)?,
        Kind::Terrain => return None,
        // spec: S-0314 claim 5 (the type's cost field, no house or country cost factor)
        _ => r.unit(o.kind, o.type_index)?.cost,
    };
    let s = crate::combat::damage::strength(w, id);
    Some(f2_step_cost(
        cost,
        s,
        g.repair_step,
        g.repair_percent,
        r.mode,
    ))
}

/// One repair step on an object (S-0314 claim 3), paid by its owner, the
/// paying house. `None` when the step cost is not known (a building before
/// M5a) or the object has no owner.
pub fn repair_step(w: &mut World, id: ObjectId) -> Option<StepOutcome> {
    let o = w.obj(id)?;
    let house = o.owner?;
    let h = o.health;
    let s = crate::combat::damage::strength(w, id);
    // spec: S-0314 claim 3 (the first matching row: full health reports not needed, before the money test)
    if h >= s {
        return Some(StepOutcome::NotNeeded);
    }
    let cost = step_cost(w, id)?;
    // spec: S-0314 claim 3 (the paying house's available credits below the step cost: cannot pay; a cost of 0 is not refused at non-negative credits, but the comparison still applies at a negative balance)
    if crate::seams::available_credits(w, house) < cost {
        return Some(StepOutcome::CannotPay);
    }
    let size = step_size(w, id);
    crate::seams::pay(w, house, cost);
    // spec: S-0314 claim 3 and S-0614 claim 12 (a repair step that goes ahead makes the parasite leave)
    crate::parasite::repair_step(w, id);
    // spec: S-0314 claim 3 (health plus the step size below the strength: more needed; otherwise health becomes the strength: done)
    let (health, out) = if i64::from(h) + i64::from(size) < i64::from(s) {
        (h + size, StepOutcome::MoreNeeded)
    } else {
        (s, StepOutcome::Done)
    };
    if let Some(ob) = w.obj_mut(id) {
        ob.health = health;
    }
    Some(out)
}

/// One step of a building's repair (S-0314 claim 8), paid by its owner:
/// returns whether repair stays on, or `None` when the object has no
/// owner or no step cost. There is no full-health
/// test: a building at full health pays one step and repair turns off.
pub fn building_repair_step(w: &mut World, id: ObjectId) -> Option<bool> {
    let o = w.obj(id)?;
    let house = o.owner?;
    let h = o.health;
    let s = crate::combat::damage::strength(w, id);
    let cost = step_cost(w, id)?;
    // spec: S-0314 claim 8 (available credits below the step cost: nothing paid, health unchanged, repair turns off)
    if crate::seams::available_credits(w, house) < cost {
        return Some(false);
    }
    // spec: S-0314 claim 8 and Edge cases (the game-wide repair step as it stands; our engine raises a value below 1 to 1)
    let step = w.rules().combat.general.repair_step.max(1);
    crate::seams::pay(w, house, cost);
    // spec: S-0314 claim 8 (health plus the step below the strength: it rises and repair stays on; otherwise health becomes the strength and repair turns off)
    let (health, on) = if i64::from(h) + i64::from(step) < i64::from(s) {
        (h + step, true)
    } else {
        (s, false)
    };
    if let Some(ob) = w.obj_mut(id) {
        ob.health = health;
    }
    Some(on)
}
