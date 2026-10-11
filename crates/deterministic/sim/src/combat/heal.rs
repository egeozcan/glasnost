//! Healing and repair in combat (S-0314): self-healing objects and the
//! house-wide self-healing of infantry and vehicles. Repair depots,
//! hospitals and building repair need docking, entering and the repair
//! order's money rules (structures and economy topics); healers' shots are
//! hits (S-0302).

use crate::combat::data::Ability;
use crate::combat::veterancy::{has_ability, type_combat};
use crate::rules::Kind;
use crate::world::{HouseId, ObjectId, World};

/// "A multiple of" a period, treating 0 as 1 and a negative period by its
/// magnitude (S-0314 Edge cases, a departure for 0).
fn multiple(t: i32, p: i32) -> bool {
    // spec: S-0314 Edge cases (a period of 0 is treated as 1 tick; a negative one behaves as its absolute value)
    let p = p.unsigned_abs().max(1) as i64;
    i64::from(t).rem_euclid(p) == 0
}

/// One object's self-healing in its turn (S-0314 claims 9 and 10, F3).
pub fn turn(w: &mut World, id: ObjectId) {
    let t = w.tick;
    let Some(o) = w.obj(id).cloned() else { return };
    if o.combat.killed {
        return;
    }
    let rules = w.rules_arc();
    let g = &rules.combat.general;
    let s = crate::combat::damage::strength(w, id);
    let h = o.health;
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
    let which = match o.kind {
        Kind::Infantry => Some(0),
        Kind::Vehicle if organic => Some(0),
        Kind::Vehicle => Some(1),
        _ => None,
    };
    let g10 = which.map_or(0, |i| {
        let total = o
            .owner
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
