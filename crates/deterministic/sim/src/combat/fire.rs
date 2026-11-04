//! Making a shot (S-0301): the delay between shots and how it is chosen,
//! bursts, ammunition and reloading, and the house factors set when the
//! houses are set up.

use glasnost_num::F64;

use crate::combat::state::HouseCombat;
use crate::rules::{Kind, SimRules};
use crate::world::{ObjectId, Target, World};

/// A new object's ammunition a₀ (S-0301 claim 10, Formula F6): the
/// maximum A when the initial field J is −1, otherwise J.
pub fn initial_ammo(r: &SimRules, kind: Kind, type_index: u32) -> i32 {
    let (max, initial) = match kind {
        Kind::Building => r
            .buildings
            .get(type_index as usize)
            .map_or((-1, -1), |b| (b.combat.ammo, b.combat.initial_ammo)),
        Kind::Terrain => (-1, -1),
        _ => r
            .unit(kind, type_index)
            .map_or((-1, -1), |u| (u.combat.ammo, u.combat.initial_ammo)),
    };
    starting_ammo(max, initial, r.quirks.initial_ammo_above_capacity)
}

/// Formula F6 of S-0301: a₀ = A when J = −1, otherwise J; with `above`
/// off (`quirks.initial_ammo_above_capacity`), a J above a nonnegative A is
/// capped at A.
pub fn starting_ammo(max: i32, initial: i32, above: bool) -> i32 {
    // spec: S-0301 claim 10, Formulas F6 (a₀ = A when J = −1, otherwise J)
    if initial == -1 {
        return max;
    }
    // spec: S-0301 Edge cases (quirks.initial_ammo_above_capacity off: capped at a nonnegative A)
    if !above && max >= 0 {
        return initial.min(max);
    }
    initial
}

/// A house's combat factors, fixed when the houses are set up: every
/// player house uses the normal difficulty row (S-0301 claim 3, S-0302
/// claim 3; glossary *difficulty skill block*); the neutral and special
/// houses have no row and no country factor, so both factors are exactly
/// 1.0 (S-0412 claim 5).
pub fn house_combat(
    r: &SimRules,
    country: u32,
    role: crate::world::HouseRole,
    houses: usize,
    row: crate::econ::setup::Row,
) -> HouseCombat {
    let mode = r.mode;
    let g = &r.combat.general;
    let one = F64::from_i64(1, mode);
    let (rof, fp) = r
        .combat
        .countries
        .get(country as usize)
        .map_or((one, one), |c| (c.rof, c.firepower));
    let player = role == crate::world::HouseRole::Player;
    // spec: S-0412 claim 5 (the normal row, or the row the player's handicap picks)
    let ri = crate::econ::setup::row_index(row);
    HouseCombat {
        // spec: S-0301 F2 (h = RZ₅₃(ℓ × κ) for a player house in a skirmish; h = 1.0 for the neutral and special houses, S-0412 claim 5)
        rof_factor: if player {
            g.rows_rof[ri].mul(rof, mode)
        } else {
            one
        },
        // spec: S-0302 claim 3 and F1 (F_h = RZ₅₃(φᵣ × φ_c) for a player house; exactly 1.0 for the neutral and special houses, S-0412 claim 5)
        firepower: if player {
            g.rows_firepower[ri].mul(fp, mode)
        } else {
            one
        },
        anger: vec![0; houses],
        enemy: None,
        score: 0,
        kills: 0,
        losses: 0,
        self_heal: [0, 0],
        // spec: S-0314 Terms (the automatic-repair hold is initially clear)
        repair_hold: false,
    }
}

/// The reload delay of S-0301 F3 for ammunition `a`.
pub fn reload_delay(tc: &crate::combat::data::TypeCombat, a: i32) -> i32 {
    // spec: S-0301 F3 (E when a = 0 and E ≠ −1)
    if a == 0 && tc.empty_reload != -1 {
        return tc.empty_reload;
    }
    // spec: S-0301 F3 (q = 1 if W = 0, else trunc(a / W); delay = L + I × q², 32-bit)
    let q = if tc.pip_wrap == 0 {
        1
    } else {
        glasnost_num::int::div(a, tc.pip_wrap)
    };
    tc.reload
        .wrapping_add(tc.reload_increment.wrapping_mul(q.wrapping_mul(q)))
}

/// Makes a shot with the weapon in `slot` at `target` (S-0301 claims 1 to
/// 18; S-0313 for the fire point; S-0306 for the launch). Returns whether a
/// shot was made (a void shot is none, claim 15).
pub fn shot(w: &mut World, firer: ObjectId, slot: usize, target: Target) -> bool {
    let rules = w.rules_arc();
    let mode = rules.mode;
    let q = rules.quirks;
    let half = q.ranged_draw_half_span;
    let Some(weapon) = crate::combat::weapon::slot_weapon(w, firer, slot) else {
        return false;
    };
    let Some(o) = w.obj(firer).cloned() else {
        return false;
    };
    // spec: S-0608 claim 16 (Q-1442: a weapon that is both suicide and draining makes the suicide hit instead of the drain attempt: no attachment, no drain-specific target clearing, no ordinary shot)
    if rules.combat.weapon(weapon).suicide {
        // spec: S-0519 claim 11 (a suicide shot creates no projectile, plays no report, starts no rearm countdown and uses no ammunition: the firer takes one hit of its type's strength instead)
        crate::bomb::suicide(w, firer);
        return true;
    }
    if rules.combat.weapon(weapon).drain {
        // spec: S-0608 claim 1 (a shot with a draining weapon is a drain attempt: no projectile, no shot damage, no warhead effect, no ordinary-shot draws; no rearm countdown, no burst advance, no ordinary per-shot ammunition)
        // spec: S-0608 claim 16 (Q-1442: the attempt records no new most recent shot, makes no ordinary firing reveal to the target's house and does not itself uncloak the firer, whether or not a link forms; an aircraft's pass bookkeeping is `air`'s, S-0501 claims 12, 15 and 16)
        crate::drain::attempt(w, firer, target);
        return false;
    }
    // spec: S-0301 claim 13 (a shot made in the tick the counter equals the sync frame is a sync shot, whether or not the countdown runs)
    let was_sync = crate::combat::check::sync_tick(w, firer, slot, weapon);
    // spec: S-0501 claim 20 and S-0522 claim 11 (a shot of an aircraft that carries passengers is a drop attempt, exactly as S-0522 claim 9, with no projectile and no ordinary shot effects: no delay and no delay draw, no burst-counter advance, no weapon report, muzzle animation or spray draw, no projectile; a refused drop leaves the rearm countdown unchanged, a successful one expires it now; the ammunition and pass counter changes are the attempt's; it is not a void shot under S-0301 claim 15)
    if crate::air::is_aircraft(w, firer) && crate::paradrop::shot_drops(w, firer) {
        let dropped = crate::paradrop::attempt(w, firer);
        let t = w.tick;
        if dropped && let Some(ob) = w.obj_mut(firer) {
            ob.combat.rearm_until = t;
        }
        // A firing pass that reaches the attempt still makes a round pending (S-0501 claim 15), even if the drop was refused: the shot counts as made.
        return true;
    }
    let tc = crate::combat::veterancy::type_combat(w, firer)
        .cloned()
        .unwrap_or_default();
    let wr = rules.combat.weapon(weapon).clone();
    let k = o.combat.burst;
    let b = wr.burst;
    let spray = o.kind != Kind::Building
        && rules
            .unit(o.kind, o.type_index)
            .is_some_and(|u| u.raw.spray_attack.unwrap_or(false));
    // spec: S-0306 claim 3 a (an area-fire weapon: the firer's cell; a spray-attack firer: the cell of S-0313 claim 9)
    let mut shot_target = target;
    if spray {
        let s = if k == 0 {
            // spec: S-0313 claim 9 (the first shot of a burst draws R(0, 7))
            w.rng.ranged(0, 7, half)
        } else {
            // spec: S-0313 claim 9 (later shots advance by trunc(8 / B) mod 8)
            (o.combat.spray_shift + 8 / b).rem_euclid(8)
        };
        if let Some(ob) = w.obj_mut(firer) {
            ob.combat.spray_shift = s;
        }
        let (dx, dy) = crate::combat::position::SPRAY[s as usize];
        shot_target =
            Target::Cell(crate::world::Pos::new(o.pos.x + dx, o.pos.y + dy, o.pos.z).cell());
    } else if wr.area_fire {
        shot_target = Target::Cell(o.pos.cell());
    }
    // spec: S-0302 claim 2 (the shot damage is fixed at the moment of firing)
    let damage = crate::combat::damage::shot_damage(w, firer, weapon);
    let fp = crate::combat::position::fire_point(w, firer, slot, k);
    if wr.spawner {
        // spec: S-0301 Terms (spawner weapon) and S-0506 claim 3 (a spawner shot launches no projectile; it hands its target to the spawn controller)
        crate::spawn::shot(w, firer, shot_target);
    }
    let launched = if wr.spawner {
        None
    } else if wr.disk_laser {
        // spec: S-0615 claim 12 (a disc-laser weapon's shot makes a disc laser instead of a projectile, holding D_s)
        crate::disc::shot(w, firer, weapon, shot_target, damage);
        None
    } else {
        crate::combat::projectile::launch(w, firer, weapon, shot_target, fp, damage)
    };
    // spec: S-0301 claim 15 (a void shot: the countdown, burst counter and ammunition stay as they were; a drop shot, S-0522 claim 11, returned above and is never void)
    if launched.is_none() && wr.projectile.is_some() && !wr.spawner && !wr.disk_laser {
        return false;
    }
    // spec: S-0316 claim 2 (the electric-bolt flag wins over the magnetic-beam flag: the bolt and no magnetic wave, even with no live wave; the shot itself is fired)
    if wr.mag_beam
        && !wr.electric_bolt
        && let Some(pid) = launched
    {
        // spec: S-0607 claim 21 and F3 (a magnetic-beam weapon's shot may create a magnetic wave, from the firing point to the impact point; claim 21 Its points: F3 is evaluated once, at creation, from the firing and impact points as computed when the shot is fired, the impact point being the projectile's stored target point)
        let k = w
            .projectiles
            .get(&pid)
            .map(|p| p.stored)
            .unwrap_or_else(|| crate::process::target_pos(w, shot_target));
        crate::lift::shot(w, firer, shot_target, fp, k);
    }
    // spec: S-0616 claim 1 (an effect weapon's emitter is made at the shot, after the projectile) and S-0301 claim 18
    crate::emitters::effect_weapon_emitter(w, firer, slot, weapon, shot_target, fp);
    // spec: S-0409 claim 16 (a shot from a reveal-on-fire weapon reveals the firer to the target's owner)
    crate::econ::shroud::fired(w, firer, &wr.name, target);
    let t = w.tick;
    let j = k + 1;
    let berserk = o.combat.berserk.is_some();
    // spec: S-0301 claim 2 (the order of the delay kinds: 1-tick building, bare rate, in-burst, full)
    let d = if o.kind == Kind::Building && o.combat.ammo > 1 && q.building_ammo_fast_rearm {
        1 // spec: S-0301 claim 10 (a building holding more than 1 round: 1 tick, no draw)
    } else if wr.effect() {
        // spec: S-0301 claim 5 (an effect weapon: its bare rate, no draw)
        wr.rof
    } else if j < b {
        // spec: S-0301 claim 4 (the in-burst delay: a vehicle's burst delay j for j ≤ 4 and not −1, else R(3, 5))
        let field = if o.kind == Kind::Vehicle && (1..=4).contains(&j) {
            rules
                .unit(o.kind, o.type_index)
                .map_or(-1, |u| u.vehicle.burst_delay[(j - 1) as usize])
        } else {
            -1
        };
        if field != -1 {
            field
        } else {
            w.rng.ranged(3, 5, half) // spec: S-0301 claim 4 (R(3, 5))
        }
    } else {
        full_delay(w, firer, &wr)
    };
    // spec: S-0301 claim 8 (a berserk firer halves the delay, truncated; a disc laser shot is never halved)
    let d = if berserk && !wr.disk_laser { d / 2 } else { d };
    // spec: S-0301 claim 23 (a spawner weapon's shot and a disc laser's firing make no shot animation; suicide, drain, drop and void shots returned above)
    if !wr.spawner && !wr.disk_laser {
        // spec: S-0301 Terms (the launch point: the fire point, or the firer's own position for a dropping projectile, S-0306 claim 5)
        let launch = if rules
            .combat
            .projectile_of(weapon)
            .is_some_and(|p| p.dropping)
        {
            o.pos
        } else {
            fp
        };
        // spec: S-0301 claims 18 and 22 and its ordering consequences (after the projectile is placed and after the delay draw, before a sonic wave or laser)
        crate::combat::shotanim::create(w, firer, weapon, launch);
        // spec: S-0301 claim 24 a (every shot that reaches the shot-animation choice starts the
        // target line of a vehicle whose type has the target-laser flag; only vehicles draw it)
        if o.kind == Kind::Vehicle
            && rules
                .unit(o.kind, o.type_index)
                .is_some_and(|u| u.vehicle.target_laser)
        {
            w.effects.push(crate::combat::effects::Effect::TargetLine {
                firer,
                house: o.owner,
            });
        }
    }
    let sync_frames = o.kind == Kind::Vehicle
        && rules
            .unit(o.kind, o.type_index)
            .is_some_and(|u| u.vehicle.firing_frames > 0);
    let firing_frames = rules
        .unit(o.kind, o.type_index)
        .map_or(0, |u| u.vehicle.firing_frames);
    let expected = match target {
        Target::Object(x)
            if !rules
                .combat
                .projectile_of(weapon)
                .is_some_and(|p| p.inaccurate) =>
        {
            Some((
                x,
                crate::combat::damage::expected_damage(w, firer, weapon, x),
            ))
        }
        _ => None,
    };
    let flyer = crate::air::is_aircraft(w, firer);
    let Some(ob) = w.obj_mut(firer) else {
        return false;
    };
    // spec: S-0301 claim 1 (the rearm countdown is set to the shot's delay D)
    ob.combat.rearm_until = t.wrapping_add(d);
    ob.combat.rearm_delay = d;
    // spec: S-0301 claim 4 (the counter is left at j mod B)
    ob.combat.burst = j.rem_euclid(b);
    // spec: S-0206 claim 9 (the tick of the last shot of any weapon)
    ob.last_fired = t;
    // spec: S-0301 claim 10 (a shot uses one round when a > 0; a disc laser shot uses none)
    // spec: S-0501 claim 15 (a flying aircraft spends one round per run instead, not per shot)
    if ob.combat.ammo > 0 && !wr.disk_laser && !flyer {
        ob.combat.ammo -= 1;
    }
    // spec: S-0301 claim 11 (a vehicle shot that launches a projectile with a < A restarts the reload, unless manual reload or A = 0)
    if ob.kind == Kind::Vehicle
        && launched.is_some()
        && !wr.disk_laser
        && tc.ammo > 0
        && !tc.manual_reload
        && ob.combat.ammo < tc.ammo
    {
        ob.combat.reload_left = Some(reload_delay(&tc, ob.combat.ammo));
    }
    // spec: S-0301 claim 13 (a shot that is not a sync shot runs the firing-frame counter at 2F − 1)
    if sync_frames && !was_sync {
        ob.combat.firing_frame = Some(2 * firing_frames - 1);
    }
    let _ = mode;
    // spec: S-0615 claim 12 (a disc laser shot plays no report at the shot; claim 16: the report comes with the damage)
    if !wr.disk_laser {
        crate::combat::projectile::shot_effect(w, firer, weapon, fp, shot_target);
        crate::combat::effects::line(w, firer, weapon, fp, shot_target);
    }
    // spec: S-0316 claims 1 and 2 (a shot that put its projectile on the map makes one bolt and its sparks in the tick of the shot, after the report; a firing that launches no projectile, such as a spawner's or a disc laser's, and a void shot make neither; each shot of a burst its own)
    if launched.is_some() && crate::combat::effects::makes_bolt(&wr) {
        crate::combat::effects::bolt(w, firer, weapon, slot, k, fp, shot_target);
    }
    // spec: S-0309 claim 15 and S-0302 claim 18 (a shot lowers the target's expected remaining health)
    if let Some((x, e)) = expected
        && let Some(tob) = w.obj_mut(x)
    {
        tob.combat.expected_health = tob.combat.expected_health.wrapping_sub(e);
    }
    after_shot(w, firer, weapon);
    if o.kind == Kind::Building {
        // spec: S-0313 claim 6 and S-0308 claim 13 (after every shot a garrisoned building makes, the shared index becomes (k + 1) mod n; a disc laser shot leaves it)
        crate::structures::garrison::shot_launched(w, firer, wr.disk_laser);
    }
    if launched.is_some() {
        // spec: S-0516 claim 39 (a still-disguising vehicle that fires and creates a projectile starts its fake blink)
        crate::cloak::disguise::fired(w, firer, weapon);
    }
    // spec: S-0614 claim 1 (a latching weapon: the firer leaves the map and travels inside the projectile)
    if let Some(carrier) = launched
        && crate::parasite::latching(w, weapon)
    {
        crate::parasite::fired(w, firer, carrier, target);
    }
    true
}

/// The rearm delay of a charge planter's arrival (S-0519 claim 13, Q-0986
/// answered by revision 3), in ticks. Arrival is not a shot: the delay is
/// chosen with the walker's present burst counter *k*, not *k* + 1, which
/// arrival neither advances nor wraps; no ammunition is used and nothing is
/// fired; berserk halving does not apply. The first applicable row:
/// - no second weapon: 1, no draw;
/// - a sonic weapon, or a spark-particle, fire-particle or railgun weapon
///   with a live effect of a kind whose flag it has on the walker (that
///   emitter place holds a system, S-0616 Terms): its rate-of-fire field,
///   no draw;
/// - *k* below the weapon's burst count: one draw R(3, 5);
/// - otherwise the full delay of S-0301 claim 2 with its draw R(0, 2).
pub fn arrival_delay(w: &mut World, walker: ObjectId) -> i32 {
    let Some(weapon) = crate::combat::weapon::slot_weapon(w, walker, 1) else {
        // spec: S-0519 claim 13 (no second weapon: 1 tick, no draw)
        return 1;
    };
    let rules = w.rules_arc();
    let half = rules.quirks.ranged_draw_half_span;
    let k = w.obj(walker).map_or(0, |o| o.combat.burst);
    let wr = rules.combat.weapon(weapon).clone();
    // spec: S-0301 claim 5 (revision 24: the effect flags are independent; any held effect of a kind whose flag the weapon has counts)
    let live = crate::emitters::any_held(w, walker, &wr);
    if wr.sonic || live {
        // spec: S-0519 claim 13 (a sonic second weapon, or a spark-particle, fire-particle or railgun one with an effect of its kind alive on the walker: its rate-of-fire field unchanged, no draw)
        wr.rof
    } else if k < wr.burst {
        // spec: S-0519 claim 13 (the counter below the burst count, a fresh walker with a burst count of 1 included: an inclusive ranged draw 3 through 5)
        w.rng.ranged(3, 5, half)
    } else {
        // spec: S-0519 claim 13 (otherwise the full delay of S-0301 claim 2, with the factors of claims 3, 6 and 7 and the draw R(0, 2); no berserk halving)
        full_delay(w, walker, &wr)
    }
}

/// The full delay of S-0301 F1, with its draw R(0, 2).
// spec: S-0301 claim 2 (the full delay draw R(0, 2))
fn full_delay(w: &mut World, firer: ObjectId, wr: &crate::combat::data::WeaponRules) -> i32 {
    let rules = w.rules_arc();
    let mode = rules.mode;
    let half = rules.quirks.ranged_draw_half_span;
    let h = w
        .obj(firer)
        .and_then(|o| o.owner)
        .and_then(|x| w.houses.get(usize::from(x.0)))
        .map_or(F64::from_i64(1, mode), |x| x.combat.rof_factor);
    let u = w.rng.ranged(0, 2, half);
    // spec: S-0301 F1 (D₁ = trunc(RZ₅₃(RZ₅₃(R × h) + u)))
    let d1 = F64::from_i64(i64::from(wr.rof), mode)
        .mul(h, mode)
        .add(F64::from_i64(i64::from(u), mode), mode)
        .to_i32_common();
    let rank = crate::combat::veterancy::rank(w, firer);
    let tc = crate::combat::veterancy::type_combat(w, firer);
    let vet_rof = tc.is_some_and(|tc| tc.veteran.has(crate::combat::data::Ability::Rof));
    let elite_rof = tc.is_some_and(|tc| tc.elite.has(crate::combat::data::Ability::Rof));
    // spec: S-0301 claim 6 (veteran with the veteran ability, or elite with either: × v once)
    let vet = match rank {
        crate::combat::veterancy::Rank::Veteran => vet_rof,
        crate::combat::veterancy::Rank::Elite => vet_rof || elite_rof,
        crate::combat::veterancy::Rank::Rookie => false,
    };
    // spec: S-0301 F1 (D₂ = trunc(RZ₅₃(D₁ × v)))
    let d2 = if vet {
        F64::from_i64(i64::from(d1), mode)
            .mul(rules.combat.general.veteran_rof, mode)
            .to_i32_common()
    } else {
        d1
    };
    // spec: S-0301 claim 7 and F1 (D₃ = trunc(D₂ / n) for a garrisoned building, n the occupant count)
    let d3 = if crate::structures::garrison::garrisoned(w, firer) {
        glasnost_num::int::div(d2, crate::structures::garrison::count(w, firer))
    } else {
        d2
    };
    // spec: S-0301 F1 (D₄ = trunc(RZ₅₃(D₃ / m)) for a garrisoned building when m is above 0)
    let m = rules.combat.general.occupy_rof;
    let d4 = if crate::structures::garrison::garrisoned(w, firer) && m.gt(F64::ZERO) {
        F64::from_i64(i64::from(d3), mode)
            .div(m, mode)
            .to_i32_common()
    } else {
        d3
    };
    // spec: S-0301 claim 7 and F1 (D₅ = trunc(RZ₅₃(D₄ / b)) for a bunkered vehicle when b ≠ 0, else D₄)
    let b = rules.combat.general.bunker_rof;
    if crate::structures::bunker::bunkered(w, firer) && !b.eq_num(F64::ZERO) {
        F64::from_i64(i64::from(d4), mode)
            .div(b, mode)
            .to_i32_common()
    } else {
        d4
    }
}

/// The rules after a shot: distributed fire (S-0309 claim 20), fire-once
/// weapons (claim 21) and the fraidycat's last round (S-0315 claim 2).
fn after_shot(w: &mut World, firer: ObjectId, weapon: crate::combat::data::WeaponId) {
    let rules = w.rules_arc();
    let Some(o) = w.obj(firer).cloned() else {
        return;
    };
    let tc = crate::combat::veterancy::type_combat(w, firer).cloned();
    let distributed = match o.kind {
        Kind::Building => false,
        _ => rules
            .unit(o.kind, o.type_index)
            .is_some_and(|u| u.raw.distributed_fire.unwrap_or(false)),
    };
    // spec: S-0309 claim 20 (a distributed-fire object firing outside attack (a building: in any behaviour) adds the target to its list and drops it)
    if distributed
        && (o.kind == Kind::Building || o.beh.seen() != Some(crate::rules::Behaviour::Attack))
        && let Some(Target::Object(x)) = o.target
        && let Some(ob) = w.obj_mut(firer)
    {
        ob.combat.fired_at.push(x);
        crate::spawn::clear_own_target(ob);
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
    }
    // spec: S-0309 claim 21 (a fire-once weapon drops its target after the shot)
    if rules.combat.weapon(weapon).fire_once
        && let Some(ob) = w.obj_mut(firer)
    {
        crate::spawn::clear_own_target(ob);
        ob.combat.burst = 0;
    }
    let _ = tc;
    if o.kind == Kind::Infantry {
        crate::combat::fear::last_round(w, firer);
    }
}

/// A vehicle's reload in its turn (S-0301 claim 11).
pub fn reload_turn(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else { return };
    if o.kind != Kind::Vehicle {
        return;
    }
    let Some(tc) = crate::combat::veterancy::type_combat(w, id).cloned() else {
        return;
    };
    if tc.ammo < 0 {
        return;
    }
    let moving = crate::occupancy::in_progress(w, id);
    let Some(ob) = w.obj_mut(id) else { return };
    // spec: S-0301 claim 11 (while a vehicle without the mobile-fire flag is moved, its reload countdown does not advance)
    if let Some(left) = ob.combat.reload_left.as_mut()
        && (tc.mobile_fire || !moving)
    {
        *left -= 1;
    }
    let over = ob.combat.reload_left.is_none_or(|l| l <= 0);
    // spec: S-0301 claim 11 (countdown over and a < A: a grows by 1 and the countdown restarts; one round per turn at most)
    if over && ob.combat.ammo < tc.ammo {
        ob.combat.ammo += 1;
        ob.combat.reload_left = Some(reload_delay(&tc, ob.combat.ammo));
    }
}

/// A building's refill in its turn (S-0301 claim 10): after its own firing
/// in that tick, a building at 0 rounds is set to its type's maximum, unless
/// the type has the hospital or armory flag. No countdown, no reload field.
pub fn building_refill(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    // spec: S-0301 claim 10 (only a building at 0 rounds is refilled; −1 is never changed)
    if o.kind != Kind::Building || o.combat.ammo != 0 {
        return;
    }
    let Some(bt) = w
        .rules
        .as_deref()
        .and_then(|r| r.buildings.get(o.type_index as usize))
    else {
        return;
    };
    // spec: S-0301 claim 10 and Import mapping (no refill for a type with the hospital or the armory flag; absent no)
    if bt.raw.hospital.unwrap_or(false) || bt.raw.armory.unwrap_or(false) {
        return;
    }
    let max = bt.combat.ammo;
    // spec: S-0301 claim 10 (set to the type's maximum A: −1 gives unlimited, 0 stays 0; the reload fields play no part)
    if let Some(ob) = w.obj_mut(id) {
        ob.combat.ammo = max;
    }
}

/// The reload countdown's elapsed time in a warp turn that excludes the
/// ordinary reload work (S-0203 claim 4, S-0301 claim 11): the countdown
/// advances one, nothing is granted and nothing is restarted. Not the
/// moving-vehicle freeze.
pub fn reload_elapsed(w: &mut World, id: ObjectId) {
    let Some(ob) = w.obj_mut(id) else { return };
    if ob.kind != Kind::Vehicle {
        return;
    }
    // spec: S-0301 claim 11 (warp exclusion: elapsed time on an existing countdown still passes; a never-started countdown stays unstarted)
    if let Some(left) = ob.combat.reload_left.as_mut() {
        *left -= 1;
    }
}

/// A vehicle's firing-frame counter in its turn (S-0301 claim 13).
pub fn firing_frame_turn(w: &mut World, id: ObjectId) {
    let Some(ob) = w.obj_mut(id) else { return };
    // spec: S-0301 claim 13 (running above 0: one less; at 0: idle; tunnels are not modelled)
    ob.combat.firing_frame = match ob.combat.firing_frame {
        Some(v) if v > 0 => Some(v - 1),
        _ => None,
    };
}
