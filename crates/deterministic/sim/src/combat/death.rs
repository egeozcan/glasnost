//! Death and destruction (S-0312): the death effects of a killed object in
//! their fixed order, with their draws, and when it leaves play (S-0203
//! claim 6); infantry deaths (S-0107 claim 17); terrain objects (S-0302
//! claim 20).

use glasnost_num::F64;
use glasnost_num::trig::{cos_t, sin_t};

use crate::combat::damage::{self, Hit};
use crate::combat::data::WeaponId;
use crate::combat::effects::{self, Effect};
use crate::combat::veterancy::{self, Rank, type_combat};
use crate::infantry::Seq;
use crate::rules::Kind;
use crate::world::{ObjectId, Pos, Target, World};

/// The nearby point N(Q, r, b) of S-0312 F2, `snap` giving the centre of
/// its cell.
// spec: S-0312 F2 (θ = RZ₅₃((f − 16383) × κ))
pub fn nearby_point(w: &World, q: Pos, r: i32, b: u32, snap: bool) -> Pos {
    let mode = w.rules().mode;
    let b = (b & 255) as i64; // spec: S-0312 F2 (the lowest 8 bits b of one raw value)
    // spec: S-0312 F2 (f = 256 b for b < 128, 256 b − 65536 otherwise)
    let f = if b < 128 { 256 * b } else { 256 * b - 65536 };
    // spec: S-0312 F2 (κ = the double nearest −π / 32767)
    let kappa = crate::rules::nearest_double("-0.00009587672516830327");
    let theta = F64::from_i64(f - 16383, mode).mul(kappa, mode);
    let rd = F64::from_i64(i64::from(r), mode);
    // spec: S-0312 F2 (y′ = trunc(RZ₅₃(y − RZ₅₃(r × sin θ))), x′ = trunc(RZ₅₃(RZ₅₃(r × cos θ) + x)))
    let y = F64::from_i64(i64::from(q.y), mode)
        .sub(rd.mul(sin_t(theta, mode), mode), mode)
        .to_i32_common();
    let x = rd
        .mul(cos_t(theta, mode), mode)
        .add(F64::from_i64(i64::from(q.x), mode), mode)
        .to_i32_common();
    let p = Pos::new(x, y, q.z);
    let c = p.cell();
    // spec: S-0312 F2 (a cell coordinate outside 0 to 511: Q itself)
    if !(0..=511).contains(&c.x) || !(0..=511).contains(&c.y) {
        return q;
    }
    if snap { Pos::centre(c, q.z) } else { p }
}

/// Debris (S-0312 claim 4, F1; S-0303 F6 for the dealing).
fn debris(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else { return };
    let Some(tc) = type_combat(w, id).cloned() else {
        return;
    };
    if tc.max_debris <= 0 {
        return;
    }
    let half = w.rules().quirks.ranged_draw_half_span;
    // spec: S-0312 F1 (N = R(Y, X − 1))
    let n = w.rng.ranged(tc.min_debris, tc.max_debris - 1, half);
    let metallic = w.rules().combat.general.metallic_debris.clone();
    // spec: S-0312 F1 (debris animations raised by 20 leptons)
    let raised = Pos::new(o.pos.x, o.pos.y, o.pos.z + 20);
    if !tc.debris_types.is_empty() {
        // spec: S-0312 claim 4 (revision 44: the voxel debris consists of S-0317 pieces, objects of play created at the unraised position with S-0317 claims 3 and 4 draws, owned by the object's house; that house changes neither their ground-impact hits nor their end's area damage, which carry no attacking house, S-0317 claims 1, 7 and 11)
        deal_debris(w, n, &tc.debris_types, &tc.debris_maximums, o.pos, o.owner);
    } else if !tc.debris_anims.is_empty() {
        let c = tc.debris_anims.len() as i32;
        for _ in 0..n.max(0) {
            let k = w.rng.ranged(0, c - 1, half);
            let name = tc.debris_anims[k as usize].clone();
            effects::anim(w, &name, raised);
        }
    } else {
        let c = metallic.len() as i32;
        for _ in 0..n.max(0) {
            // spec: S-0312 Edge cases (an empty metallic list: the draws are taken, nothing is created)
            let k = w.rng.ranged(0, c - 1, half);
            if let Some(name) = metallic.get(k as usize).cloned() {
                effects::anim(w, &name, raised);
            }
        }
    }
}

/// Deals N pieces of debris to the listed types (S-0303 F6): each a piece
/// of S-0317 at `at` with the house `house`.
pub fn deal_debris(
    w: &mut World,
    n: i32,
    types: &[String],
    maxima: &[i32],
    at: Pos,
    house: Option<crate::world::HouseId>,
) {
    let m = types.len();
    // spec: S-0303 F6 (a non-positive total creates no pieces and takes no allocation draws)
    if m == 0 || n <= 0 {
        return;
    }
    let q = w.rules().quirks;
    // spec: S-0303 Edge cases (a missing maximum counts as 0)
    let max_of = |i: usize| maxima.get(i).copied().unwrap_or(0);
    // spec: S-0303 Edge cases (departure: debris that can never be placed: every paired maximum 0 or −2, so every remainder is 0)
    let never = (0..m).all(|i| matches!(max_of(i), 0 | -2));
    let mut left = n;
    let mut j: usize = 0;
    while left > 0 {
        if never && j == m {
            // spec: S-0303 Edge cases (departure: stop after one full pass over the types, having taken its raw values)
            break;
        }
        let i = j % m;
        // spec: S-0303 F6 (bⱼ = wrap(uᵢ + 1): a maximum of 2147483647 gives the divisor −2147483648)
        let b = max_of(i).wrapping_add(1);
        let raw = w.rng.raw() as i32;
        let g = if raw == i32::MIN && !q.random_magnitude_overflow {
            // spec: S-0303 F6 and S-0202 claim 6 (switch off: the exact magnitude 2147483648, its remainder non-negative)
            let a = i64::from(raw).abs();
            let r = if b == 0 { a } else { a % i64::from(b) };
            i32::try_from(r.min(i64::from(left))).unwrap_or(left)
        } else {
            // spec: S-0303 F6 (aⱼ = wrap(|rawⱼ|); gⱼ = min(aⱼ rem bⱼ, ℓⱼ), the remainder truncated toward zero with the sign of aⱼ)
            // spec: S-0303 Edge cases (departure: a zero divisor, maximum −1, gives the remainder aⱼ; −2147483648 rem −1 is 0; S-0205 claim 3)
            glasnost_num::int::rem(raw.wrapping_abs(), b).min(left)
        };
        for _ in 0..g.max(0) {
            // spec: S-0317 Changes for other specs (S-0303 claim 22 and S-0312 claim 4: the debris pieces are objects of play, created with the draws of S-0317 claims 3 and 4)
            crate::pieces::create_named(w, &types[i], at, house);
        }
        // spec: S-0303 F6 (ℓⱼ₊₁ = wrap(ℓⱼ − gⱼ): a negative allocation raises the remaining count)
        left = left.wrapping_sub(g);
        j += 1;
    }
}

/// The weapon an exploding object's death sets off (S-0312 claim 6): its
/// own death weapon, else its current primary weapon, else the game-wide
/// one, with the damage of F3.
fn death_weapon(w: &World, id: ObjectId) -> Option<(WeaponId, i32)> {
    let r = w.rules();
    let mode = r.mode;
    let tc = type_combat(w, id)?;
    let own = tc
        .death_weapon
        .or_else(|| crate::combat::weapon::current_weapon(w, id));
    if let Some(wp) = own {
        // spec: S-0312 F3 (D = trunc(RZ₅₃(w × φ)))
        let dmg = F64::from_i64(i64::from(r.combat.weapon(wp).damage), mode)
            .mul(tc.death_weapon_factor, mode)
            .to_i32_common();
        return Some((wp, dmg));
    }
    let g = r.combat.general.death_weapon?;
    // spec: S-0312 F3 (the game-wide death weapon: D = trunc(RZ₅₃(H × 0.5)))
    let dmg = F64::from_i64(i64::from(damage::strength(w, id)), mode)
        .mul(crate::rules::nearest_double("0.5"), mode)
        .to_i32_common();
    Some((g, dmg))
}

/// A shot-down aircraft's impact (S-0312 claim 16): its death weapon goes
/// off at the impact point `at`.
pub(crate) fn impact_death_weapon(w: &mut World, id: ObjectId, at: crate::world::Pos) {
    if let Some((wp, dmg)) = death_weapon(w, id) {
        let house = w.obj(id).and_then(|o| o.owner);
        // spec: S-0312 claim 16 (shot down: its death weapon goes off at the impact point, claim 6)
        crate::combat::projectile::detonate_weapon(w, wp, dmg, at, Some(id), house, true);
    }
}

/// Whether an object is exploding (S-0312 Terms).
fn exploding(w: &World, id: ObjectId) -> bool {
    let explodes = type_combat(w, id).is_some_and(|tc| tc.explodes);
    let ability = veterancy::has_ability(w, id, crate::combat::data::Ability::Explodes);
    let suicide = crate::combat::weapon::current_weapon(w, id)
        .is_some_and(|x| w.rules().combat.weapon(x).suicide);
    explodes || ability || suicide
}

/// Marks an object killed: projectiles it fired carry no attacker any more
/// (S-0302 claim 1), actors that overrode against it revert (S-0215 claim
/// 10), and objects targeting it lose it; projectiles aimed at it target
/// its cell (S-0306 claim 17).
pub(crate) fn mark_killed(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else { return };
    if let Some(ob) = w.obj_mut(id) {
        ob.combat.killed = true;
        ob.combat.delay_kill_until = None;
    }
    let airborne = crate::combat::weapon::airborne_by_height(w, id);
    for p in w.projectiles.values_mut() {
        if p.attacker == Some(id) {
            // spec: S-0302 claim 1 (a firer killed since the shot: neither attacker nor attacking house)
            p.attacker = None;
            p.house = None;
        }
        if p.target == Some(Target::Object(id)) {
            // spec: S-0306 claim 17 (the target becomes the cell it stood in when not airborne, else nothing)
            p.target = (!airborne).then_some(Target::Cell(o.pos.cell()));
        }
    }
    let ids: Vec<ObjectId> = w.objects.keys().copied().collect();
    for x in ids {
        if !w
            .obj(x)
            .is_some_and(|ob| ob.target == Some(Target::Object(id)))
        {
            continue;
        }
        // spec: S-0501 claim 37 (an aircraft losing its target: the targeting-timer draw comes before the clear)
        crate::air::target_loss_draw(w, x);
        let Some(ob) = w.obj_mut(x) else { continue };
        let mut restored = None;
        if ob.beh.suspended.is_some() {
            // spec: S-0215 claim 10 (the target is destroyed: the revert comes at the killing hit)
            if let Some((t, d)) = ob.beh.revert() {
                crate::spawn::set_own_target(ob, t);
                if ob.kind.is_mobile() {
                    restored = Some(ob.dest);
                    ob.dest = d;
                }
            }
            if ob.target == Some(Target::Object(id)) {
                crate::spawn::clear_own_target(ob);
            }
        } else {
            crate::spawn::clear_own_target(ob);
        }
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
        if let Some(before) = restored {
            // spec: S-0107 claim 15 (a destination brought back is assigned: the destination-driven get-up)
            crate::infantry::destination_assigned(w, x, before);
        }
    }
    w.effects.push(Effect::Killed { id, pos: o.pos });
}

/// Takes a dead object out of play (S-0203 claim 6). A transport's
/// passengers still inside die with no attacker credit (S-0507 claim 16):
/// a crashable vehicle on the ground that cannot crash (S-0312 claim 12,
/// second row) and an aircraft killed on the ground (S-0312 claim 15)
/// among others.
pub fn leave_play(w: &mut World, id: ObjectId) {
    // spec: S-0312 claims 14 and 20 and S-0520 claim 5 (a destroyed crate-carrying vehicle or a building with a crate beneath leaves a crate)
    crate::crates::death_crate(w, id);
    // spec: S-0312 claims 12 and 15 and S-0507 claim 16 (remaining passengers die at removal, without attacker credit)
    crate::transport::remove_passengers(w, id);
    crate::objects::remove(w, id);
}

/// The death of a passenger inside a transport (S-0312 claims 5, 12 and
/// 16): credited to the attacker of the killing hit, its destruction
/// raises fire, and it shows no death effects of its own and takes no
/// draws for them; it is then removed.
pub fn passenger_death(w: &mut World, id: ObjectId, hit: &Hit) {
    if w.obj(id).is_none() {
        return;
    }
    if let Some(ob) = w.obj_mut(id) {
        ob.health = 0;
    }
    mark_killed(w, id);
    // spec: S-0312 claim 5 (each death is credited to the attacker of the killing hit)
    crate::combat::damage::credit(w, id, hit);
    // spec: S-0312 claim 5 (tagged passengers still receive destruction notifications; their actions complete before that passenger is removed)
    destruction_raises(w, id, hit);
    // spec: S-0312 claim 5 (no death effects of their own and no draws for them; the passenger is removed)
    crate::objects::remove(w, id);
}

/// The destruction raises of a killing hit on an infantry unit, vehicle,
/// aircraft or building (S-0312 claim 29), made after health reaches 0 and
/// before the death debris; the actions they fire complete here. A known
/// attacker, or a supplied attacking house, enables the attacker-dependent
/// raises; a supplied house other than the attacker's leaves only the
/// common group. The vehicle's extra group is behind
/// `quirks.vehicle_destruction_extra_raises`; our engine remembers no
/// hijacker type, so its exception never applies.
pub fn destruction_raises(w: &mut World, id: ObjectId, hit: &Hit) {
    let Some(o) = w.obj(id) else { return };
    let kind = o.kind;
    // spec: S-0312 claim 29 (a tagged infantry unit, vehicle, aircraft or building)
    if !matches!(
        kind,
        Kind::Infantry | Kind::Vehicle | Kind::Aircraft | Kind::Building
    ) {
        return;
    }
    let attacker_house = hit.attacker.and_then(|a| w.obj(a)).and_then(|a| a.owner);
    // spec: S-0312 claim 29 (a supplied attacking house other than the known attacker's, or a house without an attacker: only the common group, its three preliminary raises enabled by that house)
    let other_house = hit.house.is_some() && hit.house != attacker_house;
    let known = hit.attacker.is_some() || hit.house.is_some();
    // spec: S-0312 claim 29 and Edge cases (a vehicle's extra earlier group, quirks.vehicle_destruction_extra_raises)
    let extra =
        kind == Kind::Vehicle && !other_house && w.rules().quirks.vehicle_destruction_extra_raises;
    crate::triggers::destruction_raises(w, id, known, extra);
}

/// The death effects of a killing hit (S-0312 claim 1 and its table).
pub fn kill(w: &mut World, id: ObjectId, hit: &Hit) {
    let Some(o) = w.obj(id).cloned() else { return };
    if o.structure.inside.is_some() {
        // spec: S-0508 claim 19 (an occupant killed inside a building by its own house's defeat: individual killing hits, no eviction; our engine then removes the place and repairs the index under claim 23, `garrison::removing`; like a passenger it shows no death effects)
        passenger_death(w, id, hit);
        return;
    }
    let rules = w.rules_arc();
    let half = rules.quirks.ranged_draw_half_span;
    mark_killed(w, id);
    // spec: S-0608 claim 13 (a killing hit on the drainer or the building ends their link at once, before the death effects that follow it)
    crate::drain::killed(w, id);
    // spec: S-0606 claim 14 (a destroyed bunkered vehicle: walls-down effects, link and contact ended, the bunker back at clearing with guard queued, before its other death effects are resolved)
    crate::structures::bunker::release_in_place(w, id);
    if !o.on_map && crate::econ::production::is_item_object(w, id) {
        // spec: S-0414 claim 15 (an item in production takes its hit off the map and stays until its line deletes it; it is not hit again)
        // Q-0193 (interim: no other death effect, no draws; it stays on its line with health 0)
        return;
    }
    // spec: S-0312 claim 29 and table, before step 1 (the destruction raises and the actions they fire, before the debris)
    destruction_raises(w, id, hit);
    // spec: S-0312 claim 30 (a killing hit ends a master's slave group before claims 4 to 7)
    crate::econ::slaves::master_killed(w, id, hit);
    // spec: S-0402 claim 14 (a killed slave is lost to its master: the regeneration delay starts)
    crate::econ::slaves::lost(w, id);
    // spec: S-0506 claims 9 and 13 (a killed slot object is announced as lost; a killed spawner clears its slots)
    crate::spawn::killed(w, id);
    // spec: S-0601 claim 10 (the controller's killing hit releases its victims in the same tick, as part of that hit)
    crate::mind::killed(w, id);
    // spec: S-0312 table, step 1 (debris)
    debris(w, id);
    // spec: S-0312 table, step 2 and claim 5 (an exploding object: passengers die, in the order held, before the death weapon)
    if exploding(w, id) {
        crate::transport::kill_passengers(w, id, hit);
    }
    // spec: S-0312 table, step 2 (an exploding object: passengers die, then the death weapon)
    if exploding(w, id)
        && let Some((wp, dmg)) = death_weapon(w, id)
    {
        let house = o.owner;
        crate::combat::projectile::detonate_weapon(w, wp, dmg, o.pos, Some(id), house, true);
    }
    // spec: S-0312 table step 3 and S-0519 claim 5 (an attached bomb goes off once at the carrier's position, or is removed silently off the map; the carrier's death does not set it off a second time)
    crate::bomb::carrier_killed(w, id);
    match o.kind {
        Kind::Infantry => infantry_death(w, id, hit),
        Kind::Vehicle => {
            let tc = type_combat(w, id).cloned().unwrap_or_default();
            // spec: S-0312 claim 32 (a killing hit on a vessel already sinking, whether its descent began with a killing hit or a cull)
            let already_sinking = crate::naval::sinking(w, id);
            let crashable = rules
                .unit(o.kind, o.type_index)
                .is_some_and(|u| u.vehicle.crashable);
            // spec: S-0312 claim 10 and table rows 4 and 8 (a vessel that sinks shows neither of claim 9's animations and takes neither list pick; its health is restored to 1 health point)
            let sinks = crate::naval::sinks_at_kill(w, id);
            if sinks {
                // spec: S-0312 claim 12 (passengers claim 5 has not killed, before the crew, claim 13)
                passengers_of_destroyed_vehicle(w, id, hit, crashable);
                crew(w, id, hit);
                if let Some(ob) = w.obj_mut(id) {
                    ob.health = 1; // spec: S-0312 claim 10 (its health is restored to 1 health point); claim 32 table row 1 (again at a further killing hit)
                }
                if crashable {
                    // spec: S-0312 claims 10 and 15 (Q-0652: a crashable vessel at or below the water surface cannot start a crash and is removed at the end of the hit's effects, whether sinking started at this killing hit or before it; no crash-start draws, no sinking descent); S-0504 claim 4 (no descent, sinking wakes or wake draws afterwards)
                    leave_play(w, id);
                    return;
                }
                // spec: S-0312 claim 10 and S-0504 claims 4 and 5 (a non-crashable vessel stays in play; the descent starts at its next own update); S-0504 claim 13 and S-0312 claim 32 row 1 (already sinking: the descent goes on from its height, not restarted)
                crate::naval::start_sinking(w, id);
                return;
            }
            // spec: S-0312 claim 9 (one list pick each: explosion list, then destruction list)
            if !tc.explosion.is_empty() {
                let k = w.rng.list_pick(tc.explosion.len() as u32) as usize;
                let unlimited = tc.ammo < 0 || o.combat.ammo != 0;
                let explodes = tc.explodes
                    || veterancy::has_ability(w, id, crate::combat::data::Ability::Explodes);
                // spec: S-0312 claim 9 (an exploding vehicle with ammunition shows the last entry; the draw is still taken)
                let pick = if explodes && unlimited {
                    tc.explosion.len() - 1
                } else {
                    k
                };
                let name = tc.explosion[pick].clone();
                effects::anim(w, &name, o.pos);
            }
            if !tc.destroy_anim.is_empty() {
                let k = w.rng.list_pick(tc.destroy_anim.len() as u32) as usize;
                let name = tc.destroy_anim[k].clone();
                effects::anim(w, &name, o.pos);
            }
            // spec: S-0312 claim 12 (after the vehicle's animations, before the crew, claim 13)
            passengers_of_destroyed_vehicle(w, id, hit, crashable);
            crew(w, id, hit);
            if already_sinking && !crashable {
                // spec: S-0312 claims 15 and 32 table row 2 (a vessel already sinking that fails claim 10's conditions ends at 0 health after claim 9's animations and draws; a type that cannot crash stays in play, still sinking: S-0504 claim 4 goes on, tick::object_turn)
                return;
            }
            // spec: S-0312 claims 15 and 16 (a destroyed vehicle whose type can crash, above the terrain, starts to crash and stays in play until it hits the ground; a crashable vehicle using jumpjet movement uses S-0502 claim 18's motion)
            if crashable && !already_sinking && crate::jump::start_crash(w, id, Some(hit)) {
                return;
            }
            // spec: S-0312 claim 15 (every other destroyed vehicle leaves play at the end of its death effects; a crashable type that was sinking cannot start a crash below the surface and is removed)
            leave_play(w, id);
        }
        Kind::Aircraft => {
            let tc = type_combat(w, id).cloned().unwrap_or_default();
            // spec: S-0312 claim 16 (an empty explosion list: no aircraft-specific animation, no draw and no fallback, at any height)
            if !tc.explosion.is_empty() {
                // spec: S-0312 claim 16 (one list pick from the explosion list)
                let k = w.rng.list_pick(tc.explosion.len() as u32) as usize;
                let name = tc.explosion[k].clone();
                effects::anim(w, &name, o.pos);
            }
            // spec: S-0506 claim 20 (a rocket missile destroyed above the terrain keeps the rocket-flight mode and detonates later; quirks.destroyed_rocket_still_detonates)
            if crate::rocket::keeps_flying_when_destroyed(w, id) {
                return;
            }
            // spec: S-0312 claim 16 (an aircraft above the terrain starts to crash and stays in play until it hits the ground, S-0501 claim 9)
            if crate::air::start_crash_hit(w, id, false, Some(hit)) {
                return;
            }
            // spec: S-0312 claim 16 (an aircraft on the ground leaves play at once) and S-0506 claim 20 (a rocket missile on the ground: removed at once, no detonation)
            leave_play(w, id);
        }
        Kind::Building => building_death(w, id, hit),
        Kind::Terrain => {}
    }
    let _ = half;
}

/// The passengers of a destroyed vehicle that claim 5 has not killed
/// (S-0312 claim 12 and its table).
fn passengers_of_destroyed_vehicle(w: &mut World, id: ObjectId, hit: &Hit, crashable: bool) {
    if crate::transport::count(w, id) == 0 {
        return;
    }
    // spec: S-0312 claim 12 (more than 208 leptons above the terrain: all die, credited to the attacker)
    let high = w.height_above_surface(id) > 208;
    if high || (!crashable && crate::transport::bomb_marked(w, id)) {
        // spec: S-0312 claim 12 (high, or a bomb mark on a type that cannot crash: all die)
        crate::transport::kill_passengers(w, id, hit);
    } else if crashable {
        // spec: S-0312 claim 12 (a type that can crash: they stay inside, and die when it starts to crash or leaves play, death::leave_play)
    } else {
        // spec: S-0312 claim 12 (otherwise each passenger in turn gets out)
        crate::transport::escape(w, id, hit);
    }
}

/// An infantry unit's death (S-0107 claim 17).
fn infantry_death(w: &mut World, id: ObjectId, hit: &Hit) {
    let Some(o) = w.obj(id).cloned() else { return };
    let rules = w.rules_arc();
    let Some(u) = rules.unit(o.kind, o.type_index) else {
        leave_play(w, id);
        return;
    };
    let seq = o.infantry.as_ref().and_then(|s| s.seq);
    let laser_fence = hit.attacker.and_then(|a| w.obj(a)).is_some_and(|a| {
        a.kind == Kind::Building && rules.buildings[a.type_index as usize].building.laser_fence
    });
    // spec: S-0107 claim 17 (death kind: 5 for a laser-fence killer, 3 while parachuting, else the warhead's)
    let kind = if laser_fence {
        5
    } else if seq == Some(Seq::Parachute) {
        3
    } else {
        hit.warhead.map_or(0, |h| rules.combat.warhead(h).inf_death)
    };
    let prone = o.infantry.as_ref().is_some_and(|s| s.prone);
    // Row 1 (the water case) rests on a condition no spec identifies; it is not applied.
    // spec: S-0107 claim 17 row 2 (a prone cyborg or a jumpjet type: a general animation, no sequence)
    if (u.cyborg && prone) || u.jumpjet {
        // spec: S-0107 claim 17 (the rows that show no sequence remove the unit at once, unless its type has the crash flag and a crash begins instead: the jumpjet locomotor's, S-0502 claim 18)
        if u.vehicle.crashable && crate::jump::start_crash(w, id, Some(hit)) {
            return;
        }
        leave_play(w, id);
        return;
    }
    let not_human = u.raw.not_human.unwrap_or(false);
    let virus = |k: i32| k == 8; // spec: S-0107 claim 17 (death kind 8: the virus death of S-0613)
    let warhead_kind = hit.warhead.map_or(0, |h| rules.combat.warhead(h).inf_death);
    // spec: S-0107 claim 17 (a parachutist killed by a kind-8 warhead when rows 1 and 2 do not apply: the extra virus-death animation and particle of S-0613 claim 3, whatever w then is)
    let extra = seq == Some(Seq::Parachute) && virus(warhead_kind);
    // spec: S-0107 claim 17 row 4 (a "not human" type with w = 8, so not parachuting: the extra virus effect follows its die-1 request)
    let extra_after = not_human && virus(kind);
    // spec: S-0613 claim 2 (the animation's owner: the attacker's house, else the hit's attacking house)
    let owner = hit
        .attacker
        .and_then(|a| w.obj(a))
        .map_or(hit.house, |a| a.owner);
    // spec: S-0615 claim 1 (Order with the extra virus effect: a parachutist's virus effect is created first, before its row 8 kind 3 or 5 animation or its row 4 die-1 request; a "not human" parachutist gets this one only)
    if extra {
        crate::gas::virus_death(w, o.pos, owner, true);
    }
    // spec: S-0107 claim 17 row 3 (the type has its own list of death animations with at least one entry: entry w, entry 0 when w is negative or not less than the list's length; no sequence; the importer has left no-type and empty items out) and S-0615 claim 1 (it precedes rows 4 to 8: no die-1 request, no virus effect of a ground "not human" type, no conversion)
    if let Some(list) = u.raw.death_anims.as_ref().filter(|l| !l.is_empty()) {
        let name = usize::try_from(kind)
            .ok()
            .and_then(|i| list.get(i))
            .or_else(|| list.first())
            .cloned();
        if let Some(name) = name {
            death_anim(w, &name, o.pos);
        }
        // spec: S-0107 claim 17 (the rows that show no sequence remove the unit at once, unless its type has the crash flag and a crash begins instead)
        if u.vehicle.crashable && crate::jump::start_crash(w, id, Some(hit)) {
            return;
        }
        leave_play(w, id);
        return;
    }
    // spec: S-0107 claim 17 row 7 and S-0611 claims 1 and 6 (w = 9 reached: the conversion takes place unless claim 6's test refuses it)
    if !not_human
        && kind == crate::mutation::MUTATING_DEATH_KIND
        && crate::mutation::convert(w, id, hit)
    {
        return;
    }
    // spec: S-0107 claim 17 rows 4 to 7 (not human: die-1; w = 1: die-1; w = 2: die-2; w = 9 with the conversion refused: die-2, S-0611 claim 7)
    let request = if not_human {
        Some(Seq::Die1)
    } else {
        match kind {
            1 => Some(Seq::Die1),
            2 | 9 => Some(Seq::Die2),
            _ => None,
        }
    };
    let Some(q) = request else {
        if virus(kind) {
            // spec: S-0107 claim 17 row 8 (w = 8: the virus-death animation and particle delivery of S-0613 claim 2)
            crate::gas::virus_death(w, o.pos, owner, false);
        } else {
            // spec: S-0615 claims 1 and 2 (kinds 3 to 7 and 10: one death animation at the unit's position; S-0615 claim 3: kind 0 and 11 and above, none)
            death_kind_effect(w, kind, o.pos);
        }
        // spec: S-0615 claims 2 and 3 and S-0107 claim 17 (the unit is removed in the same processing, or begins its crash if its type has the crash flag)
        if u.vehicle.crashable && crate::jump::start_crash(w, id, Some(hit)) {
            return;
        }
        // spec: S-0107 claim 17 row 8 (other kinds: no sequence, removed at once)
        leave_play(w, id);
        return;
    };
    let started = crate::infantry::request(w, id, q, true);
    if extra_after {
        // spec: S-0615 claim 1 (a "not human" type killed on the ground with death kind 8 makes its forced die-1 request first, and the extra virus effect comes after it)
        crate::gas::virus_death(w, o.pos, owner, true);
    }
    // spec: S-0107 claim 17 (a cyborg killed by damage that ignores defences is removed at once on a death request)
    if u.cyborg && hit.ignore_defences {
        leave_play(w, id);
        return;
    }
    if !started && !rules.quirks.refused_death_keeps_unit {
        // spec: S-0107 claim 17 (refused_death_keeps_unit off: treated as if the kind that would have resulted, after claim 7's substitutions, had completed on the tick of its death, with the corpse or removal that completion gives)
        if matches!(crate::infantry::substitute(w, id, q), Seq::Die1 | Seq::Die2) {
            corpse(w, id);
        }
        leave_play(w, id);
    }
    // Otherwise the unit stays in play while its death sequence plays (S-0203 claim 6c).
}

/// The animation of death kind `kind` (S-0615 claims 1 to 3): kinds 3, 4,
/// 6, 7 and 10 the game-wide death animation, kind 5 the animation type at
/// registration index 1, each at `pos` with no owner and no attachment.
/// The game-wide burning death animation is a burning walker when its type
/// has the flag (claims 4 to 11). Other kinds, and a kind whose type is not
/// given or not listed, make none. The dispatch takes no draw.
pub fn death_kind_effect(w: &mut World, kind: i32, pos: Pos) {
    let rules = w.rules_arc();
    let g = &rules.combat.general;
    // spec: S-0615 claims 1 and 3 (the death-kind table; kind 0, any negative kind and 11 and above: none)
    let name = match kind {
        3 => g.infantry_explode.clone(),
        4 => g.flaming_infantry.clone(),
        // spec: S-0615 claim 1 and Edge cases (kind 5: the animation type at registration index 1, S-0004 claim 4; no dedicated key)
        5 => rules.animations.get(1).map(|a| a.name.clone()),
        6 => g.infantry_head_pop.clone(),
        7 => g.infantry_nuked.clone(),
        10 => g.infantry_brute.clone(),
        _ => None,
    };
    let Some(name) = name else { return };
    death_anim(w, &name, pos);
}

/// One death animation of type `name` at `pos` (S-0615 claim 2): no owner,
/// no attachment; a burning walker when its type has the flag.
fn death_anim(w: &mut World, name: &str, pos: Pos) {
    let rules = w.rules_arc();
    // spec: S-0615 claim 2 (missing type: when the rules data names none, or names a type that is not listed and so cannot be created, no animation; the unit is still removed)
    if rules.animation(name).is_none() {
        return;
    }
    let walker = rules
        .animations
        .iter()
        .position(|a| a.name.eq_ignore_ascii_case(name))
        .filter(|&i| rules.animations[i].flaming);
    match walker {
        // spec: S-0615 claim 1 (kind 4's type is a burning walker when it has the burning-walker flag; the flag decides, whatever the kind, claims 4 to 11)
        Some(i) => crate::walker::create(w, u32::try_from(i).unwrap_or(u32::MAX), pos),
        // spec: S-0615 claim 2 (no loop delay, repeat factor 1, no house, unattached, at the unit's position; S-0109 playback from then on)
        None => crate::combat::effects::anim(w, name, pos),
    }
}

/// The corpse a completed *die-1* to *die-5* leaves (S-0107 claim 17): from
/// the type's own corpse list when it is not empty, else from the general
/// list unless the type is marked "not human"; one draw picks the entry.
fn corpse(w: &mut World, id: ObjectId) {
    let rules = w.rules_arc();
    let Some(o) = w.obj(id) else { return };
    let pos = o.pos;
    let Some(u) = rules.unit(o.kind, o.type_index) else {
        return;
    };
    let own = u.raw.dead_bodies.as_deref().unwrap_or_default();
    // spec: S-0107 claim 17 (the type's own corpse list when not empty; otherwise the general list, unless the type is marked "not human": no corpse)
    let list: &[String] = if !own.is_empty() {
        own
    } else if u.raw.not_human.unwrap_or(false) {
        &[]
    } else {
        &rules.combat.general.dead_bodies
    };
    let Ok(n) = u64::try_from(list.len()) else {
        return;
    };
    if n == 0 {
        return;
    }
    // spec: S-0107 claim 17 (the pick: one draw from the scenario random stream, read as an unsigned 32-bit number and reduced modulo the list length)
    let r = u64::from(w.rng.raw()) % n;
    let Some(name) = usize::try_from(r).ok().and_then(|i| list.get(i)) else {
        return;
    };
    // spec: S-0107 claim 17 (a corpse animation is left at the unit's position; an unknown name is an animation type the importer created, S-0004 claim 13, which draws nothing without a sprite) and S-0615 claim 2 (a type that cannot be created: nothing, the draw taken)
    if rules.animation(name).is_some() {
        crate::combat::effects::anim(w, name, pos);
    }
}

/// An infantry unit in a completed death sequence leaves play (S-0107
/// claim 17; S-0203 claim 6c): in its own turn.
pub fn infantry_death_turn(w: &mut World, id: ObjectId) {
    let t = w.tick;
    let Some(o) = w.obj(id) else { return };
    if !o.combat.killed {
        return;
    }
    let Some(s) = o.infantry.as_ref() else { return };
    let Some(q) = s.seq else { return };
    if !matches!(q, Seq::Die1 | Seq::Die2 | Seq::WetDie1 | Seq::WetDie2) {
        return;
    }
    let c = w
        .rules()
        .unit(o.kind, o.type_index)
        .and_then(|u| u.sequences.get(q.key()))
        .map_or(0, |e| e.count);
    // spec: S-0107 claim 8 (a sequence is complete once its stage has reached C)
    if s.stage(t) >= c {
        // spec: S-0107 claim 17 (die-1 to die-5 complete: a corpse animation is left and the unit is removed on the same tick; wet-die-1 and wet-die-2: no corpse)
        if matches!(q, Seq::Die1 | Seq::Die2) {
            corpse(w, id);
        }
        leave_play(w, id);
    }
}

/// A destroyed vehicle's crew (S-0312 claim 13).
fn crew(w: &mut World, id: ObjectId, hit: &Hit) {
    let Some(o) = w.obj(id).cloned() else { return };
    let rules = w.rules_arc();
    let Some(u) = rules.unit(o.kind, o.type_index) else {
        return;
    };
    // Hijacker types (special-abilities specs) are not modelled.
    // spec: S-0312 claim 13 (no no-escape marker, the crewed flag, no passenger capacity)
    if hit.no_escape || !u.vehicle.crewed || u.passengers > 0 {
        return;
    }
    // spec: S-0312 claim 27 (the crew escape chance always applies, 0.5 by default)
    let chance = rules.combat.general.crew_escape;
    // spec: S-0312 claim 13 (a chance test "less than" with one unit-interval value)
    let v = w.rng.unit(rules.mode);
    if !v.lt(chance) {
        return;
    }
    let Some(crew_type) = crew_type(w, id) else {
        return;
    };
    // spec: S-0312 claim 13 (a crew that is due takes the creation draw whether or not it can then be placed)
    place_survivor(w, crew_type, o.owner, o.pos, true);
}

/// The crew or survivor type of F4 (S-0312), with its draw.
fn crew_type(w: &mut World, id: ObjectId) -> Option<u32> {
    let rules = w.rules_arc();
    let o = w.obj(id)?.clone();
    let g = &rules.combat.general;
    let _ = &o;
    // Sides are not part of the simulation's rules data yet (S-0004 claim 9): every house counts as without a side, giving the technician of F4's first row.
    // spec: S-0312 F4 (a country that belongs to no side: the game-wide technician, no draw)
    g.technician
}

/// Creates and places a crew or survivor infantry unit (S-0312 claims 13
/// and 18), with its creation and health draws.
// spec: S-0312 claims 13 and 18 (five standing positions of S-0208; cells of 256 leptons; lowest health 5)
fn place_survivor(
    w: &mut World,
    ty: u32,
    owner: Option<crate::world::HouseId>,
    at: Pos,
    crew: bool,
) {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let rules = w.rules_arc();
    let half = rules.quirks.ranged_draw_half_span;
    let id = crate::objects::create(w, Kind::Infantry, ty, owner);
    let c = at.cell();
    let free = (0u8..5).find(|&p| {
        // spec: S-0208 table P (five standing positions)
        w.cells
            .occ(c, crate::world::Layer::Ground)
            .is_some_and(|o| !o.taken(p))
    });
    let Some(p) = free else {
        // spec: S-0312 claims 13 and 18 (one that cannot be placed never enters play and takes no health draw; the creation draw is already taken)
        w.objects.remove(&id);
        return;
    };
    let (u, v) = crate::walk::standing_point(p);
    let z = w.surface_at(256 * c.x + u, 256 * c.y + v);
    crate::objects::place(w, id, Pos::new(256 * c.x + u, 256 * c.y + v, z), 0);
    let s = rules.unit(Kind::Infantry, ty).map_or(0, |u| u.strength);
    // spec: S-0312 claim 13 (crew health R(5, trunc(S / 2))); claim 18 (survivor health R(5, S))
    let top = if crew { s / 2 } else { s };
    let h = w.rng.ranged(5, top, half);
    if let Some(ob) = w.obj_mut(id) {
        ob.health = h;
        ob.combat.expected_health = h;
    }
    // spec: S-0312 claim 12 (a scatter request with no threat point, forced, not move-anywhere)
    crate::scatter::request(
        w,
        id,
        crate::scatter::ScatterRequest {
            threat: None,
            forced: true,
            anywhere: false,
        },
    );
    let tick = w.tick;
    let human = owner.is_none_or(|h| w.houses[usize::from(h.0)].human);
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0312 claim 25 (a player house: a crew on guard, a survivor on move; the neutral or special house: hunt)
        let b = match (human, crew) {
            (true, true) => crate::rules::Behaviour::Guard,
            (true, false) => crate::rules::Behaviour::Move,
            (false, _) => crate::rules::Behaviour::Hunt,
        };
        ob.beh.force(b, tick, stale);
    }
}

/// A destroyed building (S-0312 claims 17 to 21).
// spec: S-0312 Constants (contact kill distance 256 leptons, exclusive)
fn building_death(w: &mut World, id: ObjectId, hit: &Hit) {
    let Some(o) = w.obj(id).cloned() else { return };
    let rules = w.rules_arc();
    let half = rules.quirks.ranged_draw_half_span;
    let tc = type_combat(w, id).cloned().unwrap_or_default();
    // spec: S-0312 claim 17 (contacts within 256 leptons, or any for a helipad, are destroyed by ten times their strength)
    let helipad = rules.buildings[o.type_index as usize].building.helipad;
    let rp = w.reference_point(id);
    // spec: S-0606 claim 13 and S-0312 claim 17 (a destroyed bunker lets its vehicle out within the destroying hit, before the contacts are treated)
    crate::structures::bunker::let_out(w, id);
    // spec: S-0607 claim 12 (row 5: a building lifter that is destroyed lets go, its target cleared)
    crate::lift::building_destroyed(w, id);
    // spec: S-0312 claim 17 (revision 25: the contacts held just before the destroying hit, less
    // the bunkered vehicle, which the let-out above has already released, S-0606 claim 13)
    let contacts: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|x| x.contact == Some(id) && x.id != id)
        .map(|x| x.id)
        .collect();
    for c in contacts {
        let Some(cp) = w.obj(c).map(|x| x.pos) else {
            continue;
        };
        let (dx, dy, dz) = (
            i64::from(rp.x - cp.x),
            i64::from(rp.y - cp.y),
            i64::from(rp.z - cp.z),
        );
        let dist = glasnost_num::geom::root_floor(
            (dx * dx + dy * dy + dz * dz) as u64,
            rules.quirks.table_root_distance,
            rules.mode,
        );
        if dist < 256 || helipad {
            let s = damage::strength(w, c);
            let h = Hit {
                amount: s.wrapping_mul(10), // spec: S-0312 Constants (contact kill damage: 10 times its strength)
                warhead: rules.combat.general.c4_warhead,
                distance: 0,
                attacker: None,
                house: None,
                ignore_defences: true,
                no_escape: true,
            };
            damage::apply(w, c, h);
        } else if crate::depot::link(w, c).is_some_and(|l| l.depot == id) {
            // spec: S-0314 claim 22 (a destroyed depot's surviving contacts: released without a run-away notice)
            crate::depot::depot_destroyed(w, id, c);
        } else if let Some(x) = w.obj_mut(c) {
            x.contact = None;
        }
    }
    // spec: S-0312 claim 17 and S-0508 claim 13 (garrisoned infantry are evicted, without hunting or an inside fallback; with no qualifying exit cell the original removes every occupant without killing it)
    crate::structures::garrison::destroyed(w, id);
    let (fw, fh) = o.foundation;
    // spec: S-0312 claim 19 (a) (a foundation at least 2 × 2: discarded draws, then a percent draw for the smudge)
    if fw >= 2 && fh >= 2 {
        if rules.quirks.building_smudge_discarded_draws {
            if fw > 2 {
                let _ = w.rng.ranged(0, fw - 2, half);
            }
            if fh > 2 {
                let _ = w.rng.ranged(0, fh - 2, half);
            }
        }
        let _scorch = w.rng.percent() < 50; // spec: S-0312 Constants (smudge choice 50 percent)
        // spec: S-0312 claim 26 (the smudge choice, large form: with n fitting candidates one draw R(0, n − 1); n = 0 takes no draw and places nothing)
        w.note("S-0312 claim 26: smudge types, the smudges of cells and the morphable flag of tilesets are not modelled, so no smudge type fits: n = 0, no draw and no smudge");
        // spec: S-0312 Edge cases (quirks.smudge_footprints_beyond_map_edge is not consulted while no smudge type fits; it decides only footprint cells beyond the map edge)
        // spec: S-0312 claim 26 (zero-coordinate exception: with quirks.destruction_smudge_zero_anchor on, an anchor at cell (0, 0) receives no smudge and takes no choice draw; off, it takes the ordinary rules, which here also find n = 0 and take no draw)
        // Either setting therefore takes no smudge-choice draw while no smudge type is modelled.
        let _ = rules.quirks.destruction_smudge_zero_anchor;
    }
    // spec: S-0312 claim 19 (b) (each foundation cell: nearby point raw value, delay R(0, 3), list pick)
    if !tc.explosion.is_empty() {
        for c in damage::foundation_order(o.pos.cell(), o.foundation) {
            let raw = w.rng.raw();
            let p = nearby_point(w, Pos::centre(c, o.pos.z), 64, raw, false);
            let _delay = w.rng.ranged(0, 3, half);
            let k = w.rng.list_pick(tc.explosion.len() as u32) as usize;
            let name = tc.explosion[k].clone();
            effects::anim(w, &name, p);
        }
    }
    // (c) barrel fire animations of an exploding building, (d) spilled resources: their topics are not written.
    // spec: S-0312 claim 19 (e) (one list pick from the destruction list)
    if !tc.destroy_anim.is_empty() {
        let k = w.rng.list_pick(tc.destroy_anim.len() as u32) as usize;
        let name = tc.destroy_anim[k].clone();
        effects::anim(w, &name, o.pos);
    }
    // spec: S-0312 claim 19 (f) and S-0616 claim 1 (one destruction smoke system: smoke-kind entries only, Q-1400 answered; an exploding or sold building keeps it emitting until its removal, any other one marks it at once by claim 4)
    crate::emitters::destruction_smoke(w, id);
    // spec: S-0312 claim 19 (g) and claim 24 (absorbed passengers come out, or die, before the survivors)
    crate::structures::absorb::destroyed(w, id, hit);
    survivors(w, id, hit);
    // spec: S-0312 claim 21 (only the type's explodes flag counts, or the sale's behaviour at the destruction: the building awaits removal with 0 health)
    let selling = o.beh.seen() == Some(crate::rules::Behaviour::Selling);
    if tc.explodes || selling {
        return;
    }
    // spec: S-0312 claim 21 (any other destroyed building leaves play at the end of its death effects; rubble and crate, claim 20, are not modelled)
    leave_play(w, id);
}

/// A building awaiting removal (S-0312 claim 21), after the general part
/// of its own turn: when it is still in play with 0 health (being erased
/// and an electromagnetic pulse, which also make it wait, are not
/// modelled), it leaves play and its cells, then takes the second round of
/// claims 18 and 19 (g) (`quirks.building_removal_second_round`).
pub fn removal_turn(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if o.kind != Kind::Building || !o.combat.killed || o.health > 0 || !o.on_map {
        return;
    }
    // What the second round reads of the building, taken before it leaves play.
    let refund = crate::seams::building_refund(w, id);
    // spec: S-0312 claim 21 (the building leaves play and ceases to occupy its foundation cells before any second-round survivor enters play; it has left the update order before the second round)
    leave_play(w, id);
    if w.rules().quirks.building_removal_second_round {
        // spec: S-0312 claim 21 (a second round of claims 18 and 19 (g), C worked out afresh by F5; absorbed passengers are already out)
        second_round(w, refund);
    }
    // Passengers still inside die (claim 21): claim 24 has already let every absorbed passenger out or killed it in claim 19 (g), so none is left here (`absorb::destroyed`).
    // Rubble and crate of claim 20 are then placed: not modelled.
}

/// The second round of a building's removal (S-0312 claim 21), after it
/// has left play: claim 18's survivors and claim 19 (g)'s smudge steps,
/// which need C > 0. As for the first round, C = 0 until sides exist (see
/// `survivors`), so the round takes no draw.
fn second_round(_w: &mut World, refund: Option<i32>) {
    // spec: S-0312 claim 21 (C afresh by F5; C = 0: nothing in claim 18 happens, and no smudge step of claim 19 (g))
    let _ = refund;
}

/// Survivors of a destroyed building (S-0312 claim 18). The count C of F5
/// needs the owner's side, which the simulation's rules do not carry yet
/// (an owner whose side is not 0, 1 or 2 gives C = 0), and the building's
/// refund K (S-0408 claim 1, `seams::building_refund`); the chance needs
/// the owner's game not to be over (S-0414, `seams::game_over`). Until
/// sides and M5a exist, C = 0 and no survivor is released.
fn survivors(w: &mut World, id: ObjectId, hit: &Hit) {
    let Some(o) = w.obj(id) else { return };
    // spec: S-0312 F5 (C = min(max(trunc(K / q), 1), 5) with K the refund; C = 0 for an owner without a side, so here C = 0)
    let _refund = crate::seams::building_refund(w, id);
    let _over = o.owner.is_some_and(|h| crate::seams::game_over(w, h));
    let _ = hit;
}

/// A destroyed terrain object (S-0302 claim 20).
pub fn terrain_destroyed(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else { return };
    let rules = w.rules_arc();
    let spawns = rules
        .terrain
        .get(o.type_index as usize)
        .is_some_and(|t| t.spawns_resources);
    if spawns && let Some(h) = rules.combat.general.c4_warhead {
        // spec: S-0302 claim 20 (an ore-spawning type: the explosion for 100 with the demolition warhead, and a detonation of 100)
        crate::combat::projectile::detonate_area(w, h, 100, o.pos, None, None, true);
    }
    // spec: S-0302 claim 20 (any other terrain type crumbles and leaves play; its timing is not traced)
    leave_play(w, id);
    let _ = Rank::Rookie;
}
