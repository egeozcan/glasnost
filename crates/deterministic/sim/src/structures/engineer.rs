//! Engineers and capture (S-0509): what an engineer does when it reaches a
//! building (the outcome table of claim 3), the capture and its ordering
//! (claims 8 and 9), the used-up engineer (claims 10 and 11) and the takeover
//! of a vehicle-like building (claim 17).

use glasnost_num::F64;

use crate::combat::damage::Hit;
use crate::combat::effects::{Effect, Sound};
use crate::econ::TypeRef;
use crate::rules::{Behaviour, Kind};
use crate::scatter::{self, ScatterRequest};
use crate::structures::ownership::{self, Cause};
use crate::world::{HouseId, Layer, ObjectId, Target, World};

/// Whether the multi-engineer rule is on (S-0509 Terms): a game setting
/// that a skirmish never uses.
pub fn multi_engineer(w: &World) -> bool {
    let _ = w;
    // spec: S-0509 Terms and S-0412 claim 1 (in a skirmish the rule is always off)
    false
}

/// An engineer type (S-0509 Import mapping, `Engineer`).
pub fn is_engineer(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .filter(|o| o.kind == Kind::Infantry)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        // spec: S-0509 Import mapping (Engineer absent: no)
        .is_some_and(|u| u.engineer)
}

fn capturable(w: &World, b: ObjectId) -> bool {
    w.obj(b)
        .and_then(|o| w.rules().buildings.get(o.type_index as usize))
        // spec: S-0509 Import mapping (Capturable absent: no)
        .is_some_and(|t| t.raw.capturable.unwrap_or(false))
}

/// A vehicle-like building (S-0214 Terms): its type undeploys into a
/// vehicle and has a 1 × 1 foundation.
pub fn vehicle_like(w: &World, b: ObjectId) -> bool {
    w.obj(b)
        .and_then(|o| w.rules().buildings.get(o.type_index as usize))
        // spec: S-0214 Terms (a building whose type undeploys into a vehicle and has a 1 × 1 foundation)
        .is_some_and(|t| t.raw.undeploys_into.is_some() && t.foundation == (1, 1))
}

/// An arrival-friendly building for an entering engineer (S-0509 Terms):
/// owned by the engineer's house, owned by a house the engineer's house
/// counts as allied, or garrisonable and owned by a passive house.
pub fn arrival_friendly(w: &World, engineer: ObjectId, b: ObjectId) -> bool {
    let (Some(e), Some(o)) = (w.obj(engineer), w.obj(b)) else {
        return false;
    };
    // spec: S-0509 Terms (the engineer's own house, or a house its house counts as allied: the one-way alliance read from the engineer's house)
    if e.owner.is_some() && (e.owner == o.owner || w.allied(e.owner, o.owner)) {
        return true;
    }
    // spec: S-0509 Terms (of a garrisonable type and owned by a passive neutral house)
    crate::structures::garrison::garrisonable(w, b)
        && o.owner.is_some_and(|h| crate::econ::setup::passive(w, h))
}

/// Whether `b` is being sold (S-0408) or erased (S-0214 Terms).
fn being_sold(w: &World, b: ObjectId) -> bool {
    w.obj(b)
        .is_some_and(|o| matches!(o.beh.seen(), Some(Behaviour::Selling)) || o.econ.sale.is_some())
}

/// The unit and the building still stand after an entry raise's actions.
fn still_here(w: &World, unit: ObjectId, b: ObjectId) -> bool {
    w.obj(unit).is_some_and(|o| o.on_map) && w.obj(b).is_some_and(|o| o.on_map)
}

/// The used-up engineer (claim 10): its own tag gets one
/// destroyed-by-anything raise, then it is removed from play, not killed.
fn used_up(w: &mut World, engineer: ObjectId) {
    // spec: S-0509 claim 10 (the engineer's tag receives one destroyed-by-anything raise with the engineer as causing object, then it is removed: no death sequence, no death effect, no kill or loss counted, no experience)
    crate::triggers::destroyed_by_anything(w, engineer);
    crate::objects::remove(w, engineer);
}

/// An engineer's building entry after the general entry raise (claims 3 to
/// 11). Returns whether the engineer left the map.
pub fn entry(w: &mut World, engineer: ObjectId, b: ObjectId) -> bool {
    if !still_here(w, engineer, b) {
        return w.obj(engineer).is_none_or(|o| !o.on_map);
    }
    let q = w.rules().quirks;
    let friendly = arrival_friendly(w, engineer, b);
    let (health, strength) = w.obj(b).map_or((0, 0), |o| {
        (o.health, crate::combat::damage::strength(w, b))
    });
    let sold = being_sold(w, b);
    let erased = crate::econ::power::being_erased(w, b);
    if friendly {
        // spec: S-0509 Edge cases (quirks.engineer_repairs_during_sale off: a sale-time entry is refused, whatever the health, after the general raise)
        if sold && !q.engineer_repairs_during_sale {
            crate::structures::entry::refuse(w, engineer, b);
            return false;
        }
        // spec: S-0509 claim 3 rows 2 and 3 (arrival-friendly: refused at full health, repaired below it; the repairable flag and a sale are not tested)
        if health >= strength {
            crate::structures::entry::refuse(w, engineer, b);
            return false;
        }
        repair(w, engineer, b);
        return true;
    }
    // spec: S-0509 claim 3 row 4 (not arrival-friendly, the multi-engineer rule on and the health ratio above the condition-red fraction: a damaging hit)
    if multi_engineer(w) && ratio_above_red(w, health, strength) {
        damaging_hit(w, engineer, b);
        return true;
    }
    // spec: S-0509 claim 3 row 5 (not capturable: nothing happens to the building; quirks.engineer_lost_on_uncapturable on: the engineer is used up, off: refused)
    if !capturable(w, b) {
        if q.engineer_lost_on_uncapturable {
            used_up(w, engineer);
            return true;
        }
        crate::structures::entry::refuse(w, engineer, b);
        return false;
    }
    // spec: S-0509 claim 3 row 6 (being sold or being erased: refused, the engineer stays)
    if sold || erased {
        crate::structures::entry::refuse(w, engineer, b);
        return false;
    }
    // spec: S-0509 claim 3 row 7 and claim 7 (any other: captured, at any health, full health included)
    capture(w, engineer, b)
}

/// Whether the health ratio is above the condition-red fraction (F1): the
/// floating-point quotient compared exactly with the double *c*, strictly.
fn ratio_above_red(w: &World, health: i32, strength: i32) -> bool {
    let mode = w.rules().mode;
    if strength <= 0 {
        return false;
    }
    // spec: S-0509 F1 (r = H / S in floating point, rounded once; row 4 applies when r > c, strictly)
    let r =
        F64::from_i64(i64::from(health), mode).div(F64::from_i64(i64::from(strength), mode), mode);
    r.gt(w.rules().general.condition_red)
}

/// The damaging hit of the multi-engineer rule (claim 6, F2).
pub fn damaging_hit(w: &mut World, engineer: ObjectId, b: ObjectId) {
    let mode = w.rules().mode;
    let (h, s) = w.obj(b).map_or((0, 0), |o| {
        (o.health, crate::combat::damage::strength(w, b))
    });
    let c = w.rules().general.condition_red;
    let f = |n: i64| F64::from_i64(n, mode);
    let half = F64::from_i64(1, mode).div(f(2), mode); // spec: S-0509 Constants (damaging-hit factor 0.5, F2)
    // spec: S-0509 F2 (D = H − trunc(S × c × 0.5), the multiplications in that order)
    let d = i64::from(h) - i64::from(f(i64::from(s)).mul(c, mode).mul(half, mode).to_i32_common());
    // spec: S-0509 F2 (Y = ((1 − c × 0.5) × 0.5) × S)
    let y = f(1)
        .sub(c.mul(half, mode), mode)
        .mul(half, mode)
        .mul(f(i64::from(s)), mode);
    // spec: S-0509 F2 (A = trunc(Y) if Y < D, else D; the comparison exact, D converted to floating point)
    let a = if y.lt(f(d)) {
        y.to_i32_common()
    } else {
        d as i32
    };
    let hit = Hit {
        amount: a,
        warhead: w.rules().combat.general.c4_warhead,
        // spec: S-0509 claim 6 (distance 0, the engineer the attacker, no attacking house, defence bonuses ignored, not preventing passenger escape)
        distance: 0,
        attacker: Some(engineer),
        house: None,
        ignore_defences: true,
        no_escape: false,
    };
    crate::combat::damage::apply(w, b, hit);
    // spec: S-0509 claim 6 (the building is not captured, even if it survives; the engineer is used up)
    used_up(w, engineer);
}

/// Repair (claim 5): health becomes strength, the building's own repair
/// switch is switched off, the building-repaired sound plays at its centre
/// and the engineer is used up.
fn repair(w: &mut World, engineer: ObjectId, b: ObjectId) {
    let strength = crate::combat::damage::strength(w, b);
    let centre = w.reference_point(b);
    let owner = w.obj(b).and_then(|o| o.owner);
    if let Some(ob) = w.obj_mut(b) {
        // spec: S-0509 claim 5 (the building's health becomes its strength)
        ob.health = strength;
        ob.combat.expected_health = strength;
        // spec: S-0509 claim 5 (if the building's own repair was switched on, it is switched off)
        ob.econ.building.repairing = false;
    }
    // spec: S-0509 claim 5 (the game-wide building-repaired sound plays at the building's centre)
    w.effects.push(Effect::Sound {
        sound: Sound::BuildingRepaired,
        of: w.obj(b).map(|o| TypeRef {
            kind: o.kind,
            index: o.type_index,
        }),
        house: owner,
        pos: Some(centre),
    });
    used_up(w, engineer);
}

/// A capture (claims 8 and 9): a second entry raise, the lost-to-capture
/// mark of the house that owns the building at that moment, the tag
/// carry-over, the building ownership change to the engineer's house, the
/// scatter of the infantry standing on the building, and the used-up
/// engineer. Returns whether the engineer left the map.
fn capture(w: &mut World, engineer: ObjectId, b: ObjectId) -> bool {
    // spec: S-0509 claim 8 (the capture's second entry raise, on top of the general one, before the capture's own ownership and tag changes)
    crate::mp_triggers::entry_raise(w, b, engineer);
    if !still_here(w, engineer, b) {
        return w.obj(engineer).is_none_or(|o| !o.on_map);
    }
    let new = w.obj(engineer).and_then(|o| o.owner);
    let old = w.obj(b).and_then(|o| o.owner);
    // spec: S-0509 claim 11 (the mark is set on the house that owns the building at that moment; nothing else sets it)
    if let Some(h) = old
        && let Some(house) = w.houses.get_mut(usize::from(h.0))
    {
        house.lost_to_capture = true;
    }
    // spec: S-0524 claim 10 and S-0509 claim 8 (the engineer's tag passes on when its chain has the carry-over flag; the relative order of the carry-over and the ownership change is not established: the carry-over is done first)
    crate::mp_triggers::carry_over(w, engineer, b);
    ownership::change(w, b, new, Cause::Capture);
    scatter_standing(w, b);
    used_up(w, engineer);
    true
}

/// The infantry standing on a captured building (claim 9): every infantry
/// unit on the map in a cell of the building whose destination is none or
/// the building receives a scatter request with no threat point, forced and
/// move-anywhere; units of every house count.
fn scatter_standing(w: &mut World, b: ObjectId) {
    let Some(o) = w.obj(b).cloned() else { return };
    let cells = crate::econ::placement::foundation_cells(w, o.type_index, o.pos.cell());
    let mut list: Vec<ObjectId> = Vec::new();
    for c in &cells {
        for &id in w.cells.list(*c, Layer::Ground) {
            if list.contains(&id) {
                continue;
            }
            if let Some(x) = w.obj(id)
                && x.kind == Kind::Infantry
                && x.on_map
                && (x.dest.is_none() || x.dest == Some(Target::Object(b)))
            {
                list.push(id);
            }
        }
    }
    for id in list {
        // spec: S-0509 claim 9 (a scatter request with no threat point, forced and move-anywhere, S-0208 claims 11 to 13)
        scatter::request(
            w,
            id,
            ScatterRequest {
                threat: None,
                forced: true,
                anywhere: true,
            },
        );
    }
}

/// The takeover of a vehicle-like building by an infantry unit (claim 17):
/// one entry raise, the tag carry-over, the ownership change to the unit's
/// house, the scatter of claim 9 and the used-up unit; the lost-to-capture
/// mark is not set.
pub fn takeover(w: &mut World, unit: ObjectId, b: ObjectId) -> bool {
    // spec: S-0509 claim 17 (the building's tag receives one entry raise with the unit as causing object, which sees the old owner)
    crate::mp_triggers::entry_raise(w, b, unit);
    if !still_here(w, unit, b) {
        return w.obj(unit).is_none_or(|o| !o.on_map);
    }
    // spec: S-0509 claim 17 (neither alliance nor ownership is tested; the unit's type, its spy flag included, plays no part)
    let new = w.obj(unit).and_then(|o| o.owner);
    crate::mp_triggers::carry_over(w, unit, b);
    ownership::change(w, b, new, Cause::Takeover);
    scatter_standing(w, b);
    used_up(w, unit);
    true
}

/// Whether the lost-to-capture mark of `house` is set (S-0509 claim 11).
pub fn lost_to_capture(w: &World, house: HouseId) -> bool {
    w.houses
        .get(usize::from(house.0))
        .is_some_and(|h| h.lost_to_capture)
}
