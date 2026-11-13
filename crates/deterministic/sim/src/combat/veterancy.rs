//! Veterancy (S-0311): experience, rank, abilities and experience from
//! kills.

use glasnost_num::{F32, F64};

use crate::combat::data::{Ability, TypeCombat};
use crate::rules::{Kind, SimRules};
use crate::world::{ObjectId, World};

/// A veterancy rank (S-0311 Terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rank {
    /// Experience below 1, negative included.
    Rookie,
    /// 1 ≤ experience < 2.
    Veteran,
    /// Experience at least 2.
    Elite,
}

/// The rank of an experience value (S-0311 Terms, claim 1): exact
/// comparisons of the single with 1 and 2.
pub fn rank_of(e: F32) -> Rank {
    let e = e.to_f64();
    let rz = glasnost_num::Rounding::TowardZero;
    // spec: S-0311 claim 1 (elite from 2, veteran from 1 up to but not including 2)
    if e.ge(F64::from_i64(2, rz)) {
        Rank::Elite
    } else if e.ge(F64::from_i64(1, rz)) {
        Rank::Veteran
    } else {
        Rank::Rookie
    }
}

/// An object's rank now.
pub fn rank(w: &World, id: ObjectId) -> Rank {
    w.obj(id)
        .map_or(Rank::Rookie, |o| rank_of(o.combat.experience))
}

/// The combat fields of an object's type.
pub fn type_combat(w: &World, id: ObjectId) -> Option<&TypeCombat> {
    let o = w.obj(id)?;
    let r = w.rules.as_deref()?;
    type_combat_in(r, o.kind, o.type_index)
}

/// The combat fields of a type, read from rules `r` (as [`type_combat`]).
/// A caller that holds `w.rules_arc()` borrows from it, so the record
/// outlives later changes to the world without a copy.
pub fn type_combat_in(r: &SimRules, kind: Kind, type_index: u32) -> Option<&TypeCombat> {
    match kind {
        Kind::Building => r.buildings.get(type_index as usize).map(|b| &b.combat),
        Kind::Terrain => None,
        _ => r.unit(kind, type_index).map(|u| &u.combat),
    }
}

/// Whether an object has an ability (S-0311 claim 3): a veteran has its
/// type's veteran list, an elite the veteran and the elite lists.
pub fn has_ability(w: &World, id: ObjectId, a: Ability) -> bool {
    let Some(tc) = type_combat(w, id) else {
        return false;
    };
    // spec: S-0311 claim 3 (veteran: the veteran list; elite: both lists; rookie: none)
    match rank(w, id) {
        Rank::Rookie => false,
        Rank::Veteran => tc.veteran.has(a),
        Rank::Elite => tc.veteran.has(a) || tc.elite.has(a),
    }
}

/// The actual cost of a type for a house (S-0311 F1). The house and
/// country cost factors belong to the production specs (M5); until they
/// are written both are 1.0, so the actual cost is the type's cost.
pub fn actual_cost(w: &World, id: ObjectId) -> i32 {
    crate::combat::acquire::type_cost(w, id)
}

/// A destruction credited to `object` and/or `house` (S-0302 claim 14,
/// S-0311 claims 7 to 10).
pub fn destruction_credited(
    w: &mut World,
    victim: ObjectId,
    object: Option<ObjectId>,
    house: Option<crate::world::HouseId>,
) {
    let rules = w.rules_arc();
    let Some(v) = w.obj(victim).cloned() else {
        return;
    };
    if let Some(vh) = v.owner
        && let Some(h) = w.houses.get_mut(usize::from(vh.0))
    {
        h.combat.losses += 1;
    }
    let credited_house = object
        .and_then(|o| w.obj(o))
        .and_then(|o| o.owner)
        .or(house);
    if let Some(ch) = credited_house
        && let Some(h) = w.houses.get_mut(usize::from(ch.0))
    {
        h.combat.kills += 1;
    }
    // spec: S-0311 claim 7 (an unscored type gives no experience and no score)
    if type_combat(w, victim).is_some_and(|tc| tc.dont_score) {
        return;
    }
    // spec: S-0311 claim 9 (a kill credited to a house alone gives no experience)
    let Some(obj) = object else { return };
    let Some(oo) = w.obj(obj).cloned() else {
        return;
    };
    // spec: S-0311 F2 (V = 0 when the credited object's house is allied with the destroyed object's, the credited house's view)
    let allied = w.allied(oo.owner, v.owner);
    // spec: S-0311 F2 (V = m × A_v, m = 1, 2, 3 for rookie, veteran, elite)
    let mult = match rank_of(v.combat.experience) {
        Rank::Rookie => 1,
        Rank::Veteran => 2,
        Rank::Elite => 3,
    };
    let value = if allied {
        0
    } else {
        mult * actual_cost(w, victim)
    };
    if let Some(ch) = oo.owner
        && let Some(h) = w.houses.get_mut(usize::from(ch.0))
    {
        // spec: S-0311 claim 7 (the credited object's house adds V to its score)
        h.combat.score = h.combat.score.wrapping_add(value);
    }
    // spec: S-0311 claim 8 (case 2: a trainable credited object gains it itself; open-topped passengers and garrisons are seams)
    let trainable = |id: ObjectId| type_combat(w, id).is_some_and(|tc| tc.trainable);
    let obj = if trainable(obj) {
        obj
    } else if crate::spawn::missile_type(w, obj) {
        // spec: S-0311 claim 8 and S-0506 claim 19 (an untrainable missile-spawn object: its launcher, when it exists and is trainable; otherwise nobody)
        match crate::spawn::launcher_of(w, obj) {
            Some(l) if trainable(l) => l,
            _ => return,
        }
    } else if let Some(occupant) = crate::structures::garrison::experience_recipient(w, obj) {
        // spec: S-0311 claim 8 (an untrainable credited object that is not a missile spawn and is a garrisoned building whose occupants fire gives the experience to its firing occupant at destruction time)
        occupant
    } else {
        return;
    };
    if w.obj(obj).is_none() {
        return;
    }
    if value == 0 && !rules.quirks.zero_kill_value_caps {
        // spec: S-0311 Edge cases (zero_kill_value_caps off: a kill value of 0 never changes experience)
        return;
    }
    let ar = actual_cost(w, obj);
    gain(w, obj, value, ar);
}

/// The experience gain of S-0311 F3 for a kill value `value` and the
/// credited object's actual cost `ar` (also S-0614 F5, the cull).
pub fn gain(w: &mut World, obj: ObjectId, value: i32, ar: i32) {
    let rules = w.rules_arc();
    let mode = rules.mode;
    let Some(oo) = w.obj(obj) else { return };
    // spec: S-0311 F3 (s = RZ₅₃(e + RZ₅₃(V / RZ₅₃(A_r × ρ))); e′ = RZ₂₄(c) if c ≤ s or s is NaN, else RZ₂₄(s))
    let e = oo.combat.experience.to_f64();
    let g = &rules.combat.general;
    let s = e.add(
        F64::from_i64(i64::from(value), mode).div(
            F64::from_i64(i64::from(ar), mode).mul(g.veteran_ratio, mode),
            mode,
        ),
        mode,
    );
    let e2 = if s.is_nan() || g.veteran_cap.le(s) {
        g.veteran_cap.to_f32(mode)
    } else {
        s.to_f32(mode)
    };
    if let Some(ob) = w.obj_mut(obj) {
        ob.combat.experience = e2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glasnost_num::Rounding;

    fn single(n: i64, d: i64) -> F32 {
        let rz = Rounding::TowardZero;
        F64::from_i64(n, rz)
            .div(F64::from_i64(d, rz), rz)
            .to_f32(rz)
    }

    // spec: S-0311 claim 1 (rookie below 1, veteran from 1 below 2, elite from 2; negative is rookie)
    #[test]
    fn ranks_from_experience() {
        assert_eq!(rank_of(single(0, 1)), Rank::Rookie);
        assert_eq!(rank_of(single(-1, 4)), Rank::Rookie);
        assert_eq!(rank_of(single(999, 1000)), Rank::Rookie);
        assert_eq!(rank_of(single(1, 1)), Rank::Veteran);
        assert_eq!(rank_of(single(3, 2)), Rank::Veteran);
        assert_eq!(rank_of(single(2, 1)), Rank::Elite);
        assert_eq!(rank_of(single(5, 1)), Rank::Elite);
    }
}
