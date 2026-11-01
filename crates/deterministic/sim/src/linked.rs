//! Linked weapons (S-0518): prism support beams that make a master's one
//! shot stronger, and the charging of overpowerable buildings by infantry
//! with an electric-charge weapon.
//!
//! No site takes a draw. The support count, the pending beams and the
//! charger lists are per-object state (`combat::state::CombatState`).

use glasnost_map::CellXY;
use glasnost_num::geom::isqrt;

use crate::combat::check::{Answer, check};
use crate::combat::effects::Effect;
use crate::rules::{Behaviour, Kind};
use crate::world::{ObjectId, Target, World};

/// Whether an object is a building of the prism type (S-0518 Terms).
pub fn prism(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        o.kind == Kind::Building
            && w.rules()
                .combat
                .general
                .prism
                .building
                .is_some_and(|t| t == o.type_index)
    })
}

/// Whether the ordinary building-attack path of claims 1 to 10 applies: a
/// prism building without the SAM flag and without installed upgrades.
pub fn master_path(w: &World, id: ObjectId) -> bool {
    prism(w, id)
        && w.obj(id).is_some_and(|o| {
            !w.rules().buildings[o.type_index as usize].building.sam
                && crate::combat::behave::upgrade_gate(w, id).is_none()
        })
}

/// The support distance F2 between two buildings, in leptons.
fn distance(w: &World, a: ObjectId, b: ObjectId) -> i64 {
    let (Some(a), Some(b)) = (w.obj(a), w.obj(b)) else {
        return i64::MAX;
    };
    let sq = |d: i32| i128::from(d) * i128::from(d);
    // spec: S-0518 F2 (signed 32-bit differences; the sum formed as (Δx² + Δz²) + Δy², then the square root truncated)
    let n = (sq(a.pos.x.wrapping_sub(b.pos.x)) + sq(a.pos.z.wrapping_sub(b.pos.z)))
        + sq(a.pos.y.wrapping_sub(b.pos.y));
    isqrt(n as u128) as i64
}

/// A building joins its house's buildings (S-0518 claim 3): by placement into
/// play or by an ownership change to that house. It takes the next
/// acquisition number, which ranks it after every building the house already
/// held; acquisitions within one tick rank in the order they take effect.
/// An unchanged-owner assignment is no acquisition (the callers do not call).
pub fn acquire(w: &mut World, b: ObjectId) {
    // spec: S-0518 claim 3 (a captured or transferred building ranks after every building the new owner already held, even if created earlier; losing and reacquiring gives a new later rank; acquisitions within one tick rank in the order their events take effect, not by object number)
    w.acquisitions = w.acquisitions.wrapping_add(1);
    let n = w.acquisitions;
    if let Some(ob) = w.obj_mut(b) {
        ob.combat.acquired = n;
    }
}

/// The delayed-fire field *d* of the prism type (S-0518 claims 6 and 7).
fn delay_field(w: &World) -> i32 {
    w.rules().combat.general.prism.building.map_or(0, |t| {
        w.rules().buildings[t as usize].building.delayed_fire_delay
    })
}

/// Whether building `b` is eligible to support `master` (claim 2).
fn eligible(w: &World, master: ObjectId, b: ObjectId, reach: i32) -> bool {
    let (Some(m), Some(o)) = (w.obj(master), w.obj(b)) else {
        return false;
    };
    let q = w.rules().quirks;
    let t = w.tick;
    // spec: S-0518 claim 2 (the master's own house, the prism type, not the master, alive)
    if b == master || o.owner != m.owner || !prism(w, b) || o.health <= 0 {
        return false;
    }
    // spec: S-0518 claim 2 (the rearm countdown has run out)
    if t < o.combat.rearm_until {
        return false;
    }
    // spec: S-0518 claim 2 (the delayed-fire countdown is 0: a pending beam keeps a building out, except one whose countdown was set to 0, quirks.prism_zero_delay_double_count)
    if o.combat.support_beam.is_some() && (delay_field(w) > 0 || !q.prism_zero_delay_double_count) {
        return false;
    }
    // spec: S-0518 claim 2 (not being drained, S-0608) and (current behaviour not attack)
    if crate::drain::being_drained(w, b) || o.beh.current == Some(Behaviour::Attack) {
        return false;
    }
    // spec: S-0518 claim 2 and Edge cases (quirks.prism_support_ignores_operation off: a building not in operation is not eligible)
    if !q.prism_support_ignores_operation && !crate::econ::power::in_operation(w, b) {
        return false;
    }
    // spec: S-0518 claim 2 (the distance F2 is at most the master's slot-1 range)
    distance(w, master, b) <= i64::from(reach)
}

/// The master gets *can fire* (claims 2 to 5): recruits the nearest eligible
/// supporter, returning true (no shot in this processing), or false when the
/// master's own delayed shot starts.
pub fn recruit(w: &mut World, master: ObjectId) -> bool {
    let n = w.obj(master).map_or(0, |o| o.combat.support);
    let g = &w.rules().combat.general.prism;
    // spec: S-0518 claim 2 (only while the support count is below the cap)
    if n >= g.max {
        return false;
    }
    // spec: S-0518 Terms (support reach: the master's slot-1 weapon range, 0 without one)
    let reach = crate::combat::range::firer_weapon_range(w, master, 1);
    // spec: S-0518 claim 3 (the nearest by F2; a tie goes to the one the house acquired first, S-0404 claim 7, Q-0985 answered by revision 3: the acquisition rank, not the object number; the number only completes the ordering, no two buildings share a rank)
    let best = w
        .objects
        .keys()
        .copied()
        .filter(|&b| eligible(w, master, b, reach))
        .min_by_key(|&b| {
            (
                distance(w, master, b),
                w.obj(b).map_or(0, |o| o.combat.acquired),
                b,
            )
        });
    let Some(s) = best else { return false };
    let to = crate::combat::position::fire_point(w, master, 0, 0);
    let due = w.tick.wrapping_add((delay_field(w) - 1).max(0));
    if let Some(ob) = w.obj_mut(master) {
        // spec: S-0518 claim 4 (the support count rises by 1; no shot in this processing)
        ob.combat.support = ob.combat.support.wrapping_add(1);
    }
    if let Some(ob) = w.obj_mut(s) {
        // spec: S-0518 claim 6 (the supporter's beam becomes pending, drawn max(d − 1, 0) ticks later, to the master's fire point at recruitment)
        ob.combat.support_beam = Some((due, to));
    }
    true
}

/// The master's own delayed shot starts (claim 7): always, whatever the
/// type's animation flag; the countdown is the delayed-fire field.
pub fn start_shot(w: &mut World, master: ObjectId) {
    let due = w.tick.wrapping_add((delay_field(w) - 1).max(0));
    if let Some(ob) = w.obj_mut(master) {
        ob.combat.delayed_shot = Some(due);
        // spec: S-0518 claim 7 (the master's shot and its final check use slot 0)
        ob.combat.delayed_slot = 0;
    }
}

/// The master's delayed shot comes (claims 7 to 10, F1): slot 0 whatever
/// weapon choice gave; a projectile it launches carries the support
/// multiplier and resets the count; a dropped or projectile-less shot keeps
/// the count (`quirks.prism_count_survives_dropped_shot`).
pub fn master_shot(w: &mut World, master: ObjectId) {
    let Some(o) = w.obj(master).cloned() else {
        return;
    };
    let keep = w.rules().quirks.prism_count_survives_dropped_shot;
    let mut launched = false;
    if let Some(t) = o.target
        // spec: S-0518 claim 7 (the firing check again, with range, for slot 0; S-0301 claim 14)
        && check(w, master, Some(t), 0, true) == Answer::CanFire
    {
        let first = w.next_id;
        crate::combat::fire::shot(w, master, 0, t);
        let n = o.combat.support;
        let m = multiplier(w, n);
        let ids: Vec<ObjectId> = w
            .projectiles
            .iter()
            .filter(|(id, p)| id.0 >= first && p.attacker == Some(master))
            .map(|(&id, _)| id)
            .collect();
        launched = !ids.is_empty();
        if launched && n > 0 {
            for id in ids {
                if let Some(p) = w.projectiles.get_mut(&id) {
                    // spec: S-0518 claim 9 (a projectile launched with N > 0 carries M, F1)
                    p.multiplier = m;
                }
            }
        }
    }
    if let Some(ob) = w.obj_mut(master) {
        if launched {
            // spec: S-0518 claims 9 and 10 (a launched projectile resets the count)
            ob.combat.support = 0;
        } else if !keep {
            // spec: S-0518 Edge cases (quirks.prism_count_survives_dropped_shot off: the count becomes 0 whenever the delayed shot comes)
            ob.combat.support = 0;
        }
    }
}

/// *M* of F1 for the support count `n`.
pub fn multiplier(w: &World, n: i32) -> i32 {
    let s = w.rules().combat.general.prism.modifier;
    // spec: S-0518 F1 (U = ((s × N + 100) × 256) modulo 2³², an unsigned integer; M = ⌊U / 100⌋)
    let u = (s.wrapping_mul(n).wrapping_add(100)).wrapping_mul(256) as u32;
    (u / 100) as i32 // spec: S-0518 F1 (unsigned division by 100)
}

/// The damage supplied to area damage for a carried damage and multiplier
/// (claim 9, F1): ⌊*P* / 256⌋ with *P* the 32-bit product.
pub fn scaled_damage(damage: i32, m: i32) -> i32 {
    // spec: S-0518 F1 (P = the signed 32-bit interpretation of D_t × M; D = ⌊P / 256⌋ toward −∞)
    damage.wrapping_mul(m).div_euclid(256)
}

/// The master's support count is reset by a missing or dropped target
/// (claim 10).
pub fn reset_count(w: &mut World, master: ObjectId) {
    if let Some(ob) = w.obj_mut(master) {
        ob.combat.support = 0;
    }
}

/// A supporter's pending beam in its turn (claim 6): drawn, the rearm
/// countdown starts at exactly the support rearm, its own count is 0.
pub fn supporter_turn(w: &mut World, id: ObjectId) {
    let t = w.tick;
    let Some((due, to)) = w.obj(id).and_then(|o| o.combat.support_beam) else {
        return;
    };
    if t < due {
        return;
    }
    let g = w.rules().combat.general.prism.clone();
    let from = crate::combat::position::fire_point(w, id, 0, 0);
    let house = w.obj(id).and_then(|o| o.owner);
    w.effects.push(Effect::SupportBeam {
        from,
        to,
        duration: g.duration,
        house,
    });
    if let Some(ob) = w.obj_mut(id) {
        ob.combat.support_beam = None;
        // spec: S-0518 claim 6 (the rearm countdown starts with exactly R_s ticks; no rate-of-fire factor, no draw; its own support count becomes 0)
        ob.combat.rearm_until = t.wrapping_add(g.rearm);
        ob.combat.rearm_delay = g.rearm;
        ob.combat.support = 0;
    }
}

// ---------------------------------------------------------------------------
// Charging

/// An electric-charge detonation's effect (claim 11): an infantry firer
/// that hit a building is put on its charger list, once.
pub fn charge_hit(w: &mut World, firer: Option<ObjectId>, target: Option<Target>) {
    let (Some(f), Some(Target::Object(b))) = (firer, target) else {
        return;
    };
    // spec: S-0518 claim 11 (a firer that is not infantry registers nothing; any building, whatever its type or owner)
    if w.obj(f).is_none_or(|o| o.kind != Kind::Infantry) {
        return;
    }
    if let Some(ob) = w.obj_mut(b)
        && ob.kind == Kind::Building
        && !ob.combat.chargers.contains(&f)
    {
        ob.combat.chargers.push(f);
    }
}

/// *n*, the registered chargers of a building (S-0405 Terms).
pub fn chargers(w: &World, b: ObjectId) -> i32 {
    w.obj(b).map_or(0, |o| {
        i32::try_from(o.combat.chargers.len()).unwrap_or(i32::MAX)
    })
}

/// Whether a building's type is overpowerable.
pub fn overpowerable(w: &World, b: ObjectId) -> bool {
    w.obj(b).is_some_and(|o| {
        o.kind == Kind::Building
            && w.rules().buildings[o.type_index as usize]
                .building
                .overpowerable
    })
}

/// A building's own turn (claims 12 and 13): chargers that no longer aim at
/// it are removed (always for an overpowerable type, for another type only
/// with `quirks.stale_chargers_kept` off), then the overpowered state is
/// set for an overpowerable type.
pub fn building_turn(w: &mut World, b: ObjectId) {
    let Some(o) = w.obj(b) else { return };
    if o.combat.chargers.is_empty() && !o.combat.overpowered {
        return;
    }
    let over = overpowerable(w, b);
    let prune = over || !w.rules().quirks.stale_chargers_kept;
    if prune {
        let keep: Vec<ObjectId> = w
            .obj(b)
            .map(|o| o.combat.chargers.clone())
            .unwrap_or_default()
            .into_iter()
            .filter(|&c| {
                w.obj(c)
                    .is_some_and(|u| u.target == Some(Target::Object(b)))
            })
            .collect();
        if let Some(ob) = w.obj_mut(b) {
            // spec: S-0518 claim 12 (every charger whose current target is not the building is removed in its turn)
            ob.combat.chargers = keep;
        }
    }
    let n = chargers(w, b);
    let Some(o) = w.obj(b) else { return };
    let on = !o.econ.building.switched_off;
    let full = o.owner.is_some_and(|h| {
        crate::econ::power::fraction(w, h).eq_num(glasnost_num::F64::from_i64(1, w.rules().mode))
    });
    // spec: S-0518 claim 13 and F3 (overpowered when n ≥ 3, or n ≥ 1 with the building switched on and the owner at full power; another type never)
    let state = over && (n >= 3 || (n >= 1 && on && full)); // spec: S-0518 Constants (3 chargers overpower, 1 when on at full power)
    if let Some(ob) = w.obj_mut(b) {
        ob.combat.overpowered = state;
    }
}

/// An object leaves play (claim 12): it leaves every charger list at once.
pub fn leaving(w: &mut World, id: ObjectId) {
    for ob in w.objects.values_mut() {
        if !ob.combat.chargers.is_empty() {
            ob.combat.chargers.retain(|&c| c != id);
        }
    }
}

/// Whether an infantry unit or vehicle's slot-1 weapon has an electric-charge
/// warhead (claim 14).
pub fn charger_type(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    matches!(o.kind, Kind::Infantry | Kind::Vehicle)
        && crate::combat::weapon::slot_weapon(w, id, 1)
            .and_then(|x| w.rules().combat.warhead_of(x))
            .is_some_and(|h| h.electric_assault)
}

/// A charger without a target looks at the eight cells next to it, in step
/// direction order, for a building of an overpowerable type of its own
/// house (claim 14). When it finds one it takes it as its target, sets its
/// charging mark and gets *attack*; returns whether it did.
pub fn start_charging(w: &mut World, id: ObjectId) -> bool {
    if !charger_type(w, id) || w.obj(id).is_none_or(|o| o.target.is_some()) {
        return false;
    }
    let Some((cell, owner)) = w.obj(id).map(|o| (o.pos.cell(), o.owner)) else {
        return false;
    };
    // spec: S-0518 claim 14 (the eight cells next to its own, in step-direction order 0 to 7; the first holding an own-house overpowerable building; an ally's is never chosen)
    let found = crate::regions::STEPS.iter().find_map(|&(dx, dy)| {
        let c = CellXY::new(cell.x + dx, cell.y + dy);
        w.cells
            .list(c, crate::world::Layer::Ground)
            .iter()
            .copied()
            .find(|&b| {
                overpowerable(w, b)
                    && w.obj(b)
                        .is_some_and(|o| o.owner == owner && o.owner.is_some())
            })
    });
    let Some(b) = found else { return false };
    if let Some(ob) = w.obj_mut(id) {
        crate::spawn::set_own_target(ob, Some(Target::Object(b)));
        ob.combat.charging = true;
        // spec: S-0518 claim 14 (attack is queued without starting immediately; the processing keeps its ordinary delay, Q-0985 answered by revision 3)
        ob.beh.queue(Some(Behaviour::Attack));
    }
    // spec: S-0301 claim 5, Moment of a retarget's let-go (revision 67, Q-2246: at the change of the firer's target)
    crate::emitters::retarget_let_go(w, id);
    true
}

/// An infantry unit's *attack* processing while its charging mark is set
/// (claim 15): an automatic range search without the timer gate; a unit
/// that finds a target takes it and the mark clears. Returns whether the
/// mark was set (the search ran).
pub fn break_off(w: &mut World, id: ObjectId) {
    // spec: S-0518 claim 15 and Edge cases (the break-off search belongs to infantry; quirks.vehicle_charge_no_break_off off: vehicles break off too)
    let no_vehicles = w.rules().quirks.vehicle_charge_no_break_off;
    let marked = w.obj(id).is_some_and(|o| {
        o.combat.charging && (o.kind == Kind::Infantry || (!no_vehicles && o.kind == Kind::Vehicle))
    });
    if !marked {
        return;
    }
    // spec: S-0518 claim 15 (revision 3, Q-0985 answered: the range search of S-0309 claim 1 centred on its position, run before the ordinary approach, with no targeting-timer gate or reset draw; the held building target is not tested, protected or cleared; passive-acquisition permissions and the opportunity gate do not limit it, the no-auto-fire restriction does)
    let found = crate::combat::acquire::search(
        w,
        id,
        crate::combat::acquire::SearchKind::Range,
        crate::combat::acquire::SearchContext::BreakOff,
    );
    if found.is_some()
        && let Some(ob) = w.obj_mut(id)
    {
        // spec: S-0518 claim 15 (a nonempty result becomes the target and clears the mark, even when it equals the held building; nothing found: the held target and the mark stay unchanged)
        ob.combat.charging = false;
    }
}
