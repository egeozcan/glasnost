//! The creators of particle systems of S-0616 claim 1 that belong to other
//! parts of play, and the emitter places of S-0616 Terms.
//!
//! Built here: the effect weapons' fire, spark and railgun emitters with
//! S-0301 claim 5's live-effect gate and the fire emitter's let-go events
//! ([`effect_weapon_emitter`], [`own_effect_alive`],
//! [`other_effect_alive`], [`let_go_fire`]); the barrel system of S-0303
//! claim 19 ([`barrel_particle_system`]); the destruction smoke of S-0312
//! claim 19 f ([`destruction_smoke`]); refinery smoke
//! ([`refinery_smoke`]); natural smoke with S-0516 claim 18's cloak removal
//! and uncloak rule ([`natural_smoke`], [`natural_smoke_cloaked`],
//! [`natural_smoke_yellow`]); the damage sparks of cyborg infantry
//! ([`damage_sparks`]); the sparks of a deactivated vehicle
//! ([`deactivated_sparks`]); the sparks of an electric bolt (S-0316 claim
//! 6, [`electric_bolt_sparks`]); the attached system of a debris piece
//! (S-0317 claim 13, [`voxel_animation_system`]); and the two trigger
//! actions of S-0011 codes 88 and 89 ([`trigger_create`], [`trigger_mark`]). The flame sweep,
//! swing and insertion of a fire emitter are part of the fire kind's turn
//! (`crate::particles::fire_turn`).

use glasnost_num::F64;

use crate::combat::data::{WeaponId, WeaponRules};
use crate::combat::state::FireEmitter;
use crate::gas::Behaviour;
use crate::rules::Kind;
use glasnost_map::CellXY;

use crate::world::{ObjectId, Pos, Target, World};

/// The top of a ranged draw over 0 … 2147483646 (claim 1, damage sparks).
const DRAW_TOP: i32 = 2147483646; // spec: S-0616 claim 1 (one ranged draw R(0, 2147483646) each turn)

/// The damage-sparks depth limit, leptons below the surface (claim 1).
const SPARK_DEPTH: i32 = -10; // spec: S-0616 Constants (damage-sparks depth limit 10 leptons below the surface)

/// The deactivated-vehicle spark period, ticks (claim 1).
const DEACTIVATED_PERIOD: i32 = 30; // spec: S-0616 Constants (deactivated-vehicle spark period 30 ticks)

/// The deactivated-vehicle cadence divisor, leptons (claim 1).
const DEACTIVATED_DIVISOR: i32 = 10; // spec: S-0616 Constants (deactivated-vehicle cadence divisor 10)

/// The weight of the *y* term of the deactivated-vehicle cadence (claim 1).
const DEACTIVATED_WEIGHT: i32 = 65536; // spec: S-0616 Constants (deactivated-vehicle cadence weight 65536)

/// The deactivated-vehicle spark displacement bound, leptons per axis.
const DEACTIVATED_SPREAD: i32 = 100; // spec: S-0616 Constants (damage-spark displacement of a deactivated vehicle −100 … 100 leptons)

/// The destruction smoke height limit, leptons below the terrain.
const DESTRUCTION_DEPTH: i32 = -10; // spec: S-0616 Constants (destruction-smoke depth limit 10 leptons below the surface) and S-0312 claim 19 f

/// The refinery smoke points that count as unset (claim 1).
// spec: S-0616 Constants (refinery smoke unset points (0, 0, 0) and (128, 128, 0))
const UNSET_POINTS: [[i32; 3]; 2] = [[0, 0, 0], [128, 128, 0]];

/// The emitter places of a weapon's effect (S-0616 Terms; S-0301 claim 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    /// The fire emitter place.
    Fire,
    /// The spark emitter place, shared with damage sparks.
    Spark,
    /// The railgun emitter place.
    Railgun,
}

/// The emitter places of a weapon's effect flags, in the order fire,
/// spark, railgun (S-0301 claim 5; S-0616 claim 1). The flags are tested
/// independently, so a weapon may have up to three. A sonic weapon's wave
/// is not modelled (S-0607 claim 21's note), so it has none here.
pub fn places_of(wr: &WeaponRules) -> Vec<Place> {
    // spec: S-0616 claim 1 and S-0301 claim 5 (the fire, spark and railgun flags are tested independently, in the order fire, spark, railgun)
    [
        (wr.fire_particles, Place::Fire),
        (wr.spark, Place::Spark),
        (wr.railgun, Place::Railgun),
    ]
    .into_iter()
    .filter_map(|(on, p)| on.then_some(p))
    .collect()
}

/// Whether any place of `wr`'s effect flags holds a system on `id`.
pub fn any_held(w: &World, id: ObjectId, wr: &WeaponRules) -> bool {
    places_of(wr).into_iter().any(|p| held(w, id, p).is_some())
}

/// The system an object holds in an emitter place.
pub fn held(w: &World, id: ObjectId, place: Place) -> Option<ObjectId> {
    let c = &w.obj(id)?.combat;
    match place {
        Place::Fire => c.fire_emitter.map(|e| e.system),
        Place::Spark => c.spark_emitter,
        Place::Railgun => c.railgun_emitter,
    }
}

/// S-0301 claim 5, first row: an effect of any kind whose flag the asked
/// weapon has is alive on the firer (*rearming*, rank 5).
pub fn own_effect_alive(w: &World, firer: ObjectId, weapon: WeaponId) -> bool {
    let wr = w.rules().combat.weapon(weapon);
    // spec: S-0301 claim 5 (a live effect of any kind whose effect flag the asked weapon has: rearming) and Terms (revision 63, Q-2181: every emitter, matched or not, is alive while its place holds it; claim 5's place-and-kind rule gives when the place is freed)
    any_held(w, firer, wr)
}

/// S-0301 claim 5, second row: an effect of any kind whose flag the asked
/// weapon's other weapon has is alive on the firer (*impossible*, rank 4).
pub fn other_effect_alive(w: &World, firer: ObjectId, slot: usize) -> bool {
    // spec: S-0301 claim 5 (the other weapon of slot 0 is slot 1; of any other slot, slot 0)
    let other = if slot == 0 { 1 } else { 0 };
    let Some(ow) = crate::combat::weapon::slot_weapon(w, firer, other) else {
        return false;
    };
    let wr = w.rules().combat.weapon(ow);
    // spec: S-0301 claim 5 (a live effect of any kind whose effect flag the asked weapon's other weapon has, whatever the asked weapon's own flags: impossible)
    any_held(w, firer, wr)
}

/// Clears every emitter place of `owner` that holds system `sys` (the
/// system leaves play, S-0613 claim 17; S-0616 Terms: an object holds a
/// system in its place until it leaves play).
pub fn release_places(w: &mut World, owner: ObjectId, sys: ObjectId) {
    let Some(ob) = w.obj_mut(owner) else { return };
    let c = &mut ob.combat;
    if c.damage_smoke == Some(sys) {
        c.damage_smoke = None;
    }
    if c.spark_emitter == Some(sys) {
        c.spark_emitter = None;
    }
    if c.railgun_emitter == Some(sys) {
        c.railgun_emitter = None;
    }
    if c.fire_emitter.is_some_and(|e| e.system == sys) {
        c.fire_emitter = None;
    }
}

/// S-0616 claim 1, effect weapons: the emitters of an effect weapon's
/// shot, made after the projectile: for each of its fire, spark and
/// railgun flags, in that order, one emitter of the weapon's system type
/// when the firer's place of that kind is empty. `fp` is the shot's fire
/// point. A railgun flag also runs the railgun shot's own effect (S-0306
/// claim 27, `crate::combat::railgun`) when its place is empty, before its
/// emitter. Creating the emitters takes no draw.
pub fn effect_weapon_emitter(
    w: &mut World,
    firer: ObjectId,
    slot: usize,
    weapon: WeaponId,
    target: Target,
    fp: Pos,
) {
    let rules = w.rules_arc();
    let wr = rules.combat.weapon(weapon);
    let places = places_of(wr);
    if places.is_empty() {
        return;
    }
    let Some(o) = w.obj(firer).cloned() else {
        return;
    };
    // spec: S-0616 claim 1 (the emitter is made at the weapon's fire point, or at the firer's position for a dropping projectile, S-0306 claim 5)
    let dropping = rules
        .combat
        .projectile_of(weapon)
        .is_some_and(|p| p.dropping);
    let at = if dropping { o.pos } else { fp };
    let object = match target {
        Target::Object(x) if w.obj(x).is_some() => Some(x),
        _ => None,
    };
    // spec: S-0616 claim 1 (fire and spark: the target point is the position of the shot's target object at the shot, else the world origin)
    let object_pos = object
        .and_then(|x| w.obj(x))
        .map_or(Pos::default(), |x| x.pos);
    let ti = match wr.attached_system.as_deref() {
        // spec: S-0616 Import mapping (an effect weapon without an attached system: no emitter; the importer warns)
        None => None,
        Some(name) => {
            let ti = rules.gas.system(name);
            if ti.is_none() {
                w.note(format!(
                    "S-0616 claim 1: the attached particle system {name} is not a listed type; no emitter is made"
                ));
            }
            ti
        }
    };
    for place in places {
        // spec: S-0616 Terms and claim 1 (an emitter is made only when the firer's place for that kind is empty)
        if held(w, firer, place).is_some() {
            continue;
        }
        let point = match place {
            Place::Railgun => {
                // spec: S-0616 claim 1 and F14 (railgun: the target point is the beam's end point, computed at the shot from the emitter's position to the target's position) and S-0306 claim 27 (the railgun shot's hits and terrain draws, computed with it, before the emitter)
                let q = crate::process::target_pos(w, target);
                crate::combat::railgun::shot(w, firer, weapon, target, at, q)
            }
            _ => object_pos,
        };
        let Some(ti) = ti else { continue };
        let Some(sys) = crate::particles::create(w, ti, at, o.owner) else {
            continue;
        };
        if let Some(s) = w.gas.systems.get_mut(&sys) {
            // spec: S-0616 claim 1 (owner object: the firer; no first particle) and Terms (a system with an owner object takes the owner's current target as its target object; a railgun turn uses only its fixed target point)
            s.owner_object = Some(firer);
            s.target = point;
            // spec: S-0301 Changes for other specs, S-0616 (c) (revision 63: the place, not the system's kind, sets the creation: the fire or spark place passes the shot's target object, the railgun place passes none)
            s.target_object = if place == Place::Railgun {
                None
            } else {
                object
            };
        }
        let Some(ob) = w.obj_mut(firer) else { return };
        // spec: S-0616 Terms (the emitter is held in the place of its kind)
        match place {
            Place::Fire => {
                ob.combat.fire_emitter = Some(FireEmitter {
                    system: sys,
                    slot: u8::try_from(slot).unwrap_or(u8::MAX),
                    target: Some(target),
                });
            }
            Place::Spark => ob.combat.spark_emitter = Some(sys),
            Place::Railgun => ob.combat.railgun_emitter = Some(sys),
        }
    }
}

/// S-0616 claim 1: the fire emitter of `owner` is marked and let go (a new
/// target not within reach, a move order, death). The system keeps its
/// particles, which go on updating and moving (claim 22).
pub fn let_go_fire(w: &mut World, owner: ObjectId) {
    let Some(e) = w.obj_mut(owner).and_then(|o| o.combat.fire_emitter.take()) else {
        return;
    };
    // spec: S-0616 claim 1 (a fire emitter is marked and let go) and S-0301 claim 5, place-and-kind rule (revision 63: a let-go of the fire place marks the system that place holds, whatever its kind, and the place is freed at the let-go)
    crate::particles::mark(w, e.system);
}

/// S-0301 claim 5's place-and-kind rule for a system held in its owner's
/// fire place whose kind is not fire: the fire place's let-go at the
/// firer's death applies to it as to a fire-kind system (S-0616 claim 1),
/// whose own turn makes that test (`crate::particles::fire_turn`). Called
/// in the system's turn after its kind's own work. The retarget let-go is
/// not made here but at the change of the firer's target
/// ([`retarget_let_go`]).
pub fn fire_place_let_go(w: &mut World, sys: ObjectId) {
    let Some(owner) = w.gas.systems.get(&sys).and_then(|s| s.owner_object) else {
        return;
    };
    if held(w, owner, Place::Fire) != Some(sys) {
        return;
    }
    let alive = w.obj(owner).is_some_and(|o| o.health > 0);
    if !alive {
        // spec: S-0301 claim 5, place-and-kind rule (revision 63: the fire place's let-go events are a retarget to a target not within reach, a move order and the firer's death; a let-go marks the system the place holds, whatever its kind)
        let_go_fire(w, owner);
    }
}

/// S-0616 claim 1 and S-0301 claim 5, *Moment of a retarget's let-go*
/// (revision 67, Q-2246): the retarget let-go of `owner`'s fire place. It is
/// called at every change of `owner`'s own target, in whichever part of the
/// tick makes that change, never in the held system's turn, and it reads
/// nothing of that system's kind. A target that is no longer the one the
/// emitter remembers lets the emitter go when it is not within reach
/// (S-0307 claim 12's range test with the emitter's weapon); a new target
/// within reach, or no target, is remembered. Returns whether the emitter
/// was let go.
pub fn retarget_let_go(w: &mut World, owner: ObjectId) -> bool {
    let Some(o) = w.obj(owner) else { return false };
    let Some(e) = o.combat.fire_emitter else {
        return false;
    };
    // spec: S-0301 claim 5, Moment of a retarget's let-go (revision 67, Q-2246: the test is made at the change of the firer's target, in whichever part of the tick makes that change, and not in the turn of the system the fire place holds; it reads nothing of that system's kind, so it is the same for every kind)
    if o.target == e.target {
        return false;
    }
    let now = o.target;
    let Some(t) = now else {
        if let Some(ob) = w.obj_mut(owner) {
            cleared(ob);
        }
        return false;
    };
    let weapon = crate::combat::weapon::slot_weapon(w, owner, usize::from(e.slot));
    // spec: S-0616 claim 1 (a new target that is not within reach: outside the weapon's range, the test of S-0307 claim 12)
    if crate::combat::range::in_range(w, owner, weapon, Some(t), None) {
        if let Some(ob) = w.obj_mut(owner)
            && let Some(f) = ob.combat.fire_emitter.as_mut()
        {
            f.target = Some(t);
        }
        return false;
    }
    let_go_fire(w, owner);
    true
}

/// The retarget let-go's case of a cleared target ([`retarget_let_go`]),
/// for the clears that hold only the object: a cleared target is not a new
/// target, so the fire emitter stays and remembers no target. Called by
/// `crate::spawn`'s own-target clears at the clear itself.
pub fn cleared(ob: &mut crate::world::Object) {
    if ob.target.is_some() {
        return;
    }
    // spec: S-0301 Terms (a fire emitter is let go on a retarget; a cleared target is not a new target, so the emitter stays) and claim 5, Moment of a retarget's let-go (revision 67, Q-2246: at the change itself)
    if let Some(f) = ob.combat.fire_emitter.as_mut() {
        f.target = None;
    }
}

/// S-0303 claim 19: the barrel particle system at `p` (the 25 % draw has
/// passed). Made as a warhead's system (S-0616 claim 1, exploding overlay
/// row: no owner object, target point P, a first particle when it has a
/// held type), with no owner house (S-0616 claim 1, exploding overlay row).
pub fn barrel_particle_system(w: &mut World, p: Pos) {
    let Some(name) = w.rules().combat.general.barrel_particle.clone() else {
        // spec: S-0303 claim 19 (no barrel particle system named: nothing appears)
        return;
    };
    // spec: S-0303 claim 19 and S-0616 claim 1 (the system appears at P with owner house none; its first particle by S-0613 claim 4)
    crate::gas::warhead_system(w, &name, p, None);
}

/// S-0312 claim 19 f and S-0616 claim 1: the destruction smoke of a
/// destroyed building. Only smoke-kind entries are eligible. "Attached
/// already" means only an earlier destruction-smoke system of this same
/// building still in play; a building makes its destruction smoke only at
/// its destruction, so that never stops it (S-0616 claim 1), and no state
/// is kept for it. The building's damage smoke, natural smoke, effect
/// emitters and damage sparks do not count.
pub fn destruction_smoke(w: &mut World, id: ObjectId) {
    let Some(tc) = crate::combat::veterancy::type_combat(w, id).cloned() else {
        return;
    };
    let rules = w.rules_arc();
    // spec: S-0616 claim 1 and S-0312 claim 19 f (only smoke-kind entries are eligible; entries of every other kind are skipped)
    let eligible: Vec<u32> = tc
        .destroy_systems
        .iter()
        .filter_map(|n| rules.gas.system(n))
        .filter(|&i| rules.gas.systems[i as usize].kind == Some(Behaviour::Smoke))
        .collect();
    let m = eligible.len() as i32;
    // spec: S-0616 claim 1 and S-0312 claim 19 f (with none eligible nothing is made and nothing is drawn; the height above the surface greater than −10 leptons; an earlier destruction-smoke system of this building never exists here, so the attachment condition never stops it)
    if m == 0 || w.height_above_surface(id) <= DESTRUCTION_DEPTH {
        return;
    }
    let half = rules.quirks.ranged_draw_half_span;
    // spec: S-0312 claim 19 f (one ranged draw R(0, m − 1); the entries are counted from the end of the list)
    let k = w.rng.ranged(0, m - 1, half);
    let ti = eligible[(m - 1 - k) as usize];
    let Some(o) = w.obj(id) else { return };
    // spec: S-0312 claim 19 f (placed at its position moved by its type's destruction smoke displacement)
    let pos = offset(o.pos, tc.destroy_offset);
    let Some(sys) = crate::particles::create(w, ti, pos, None) else {
        return;
    };
    if let Some(s) = w.gas.systems.get_mut(&sys) {
        // spec: S-0616 claim 1 (destruction smoke: owner object the building, so it is marked when the building is removed, claim 4; no owner house; target point the world origin, no first particle)
        s.owner_object = Some(id);
        s.owner_offset = tc.destroy_offset;
    }
}

// nocite: vector and matrix dimensions (three axes)
fn offset(p: Pos, d: [i32; 3]) -> Pos {
    Pos::new(
        p.x.wrapping_add(d[0]),
        p.y.wrapping_add(d[1]),
        p.z.wrapping_add(d[2]), // nocite: vector and matrix dimensions (three axes)
    )
}

/// S-0616 claim 1, refinery smoke: a smoke event on `refinery` makes one
/// system of the building type's refinery smoke type at each of its set
/// smoke points, with the game-wide refinery-smoke time as its lifetime.
/// The smoke events: each payment row and the empty-load row of S-0402
/// claim 11 (on the building in the neighbouring cell, before the
/// payment), and each slave delivery of claim 14 that paid something (on
/// the master, after the payments).
pub fn refinery_smoke(w: &mut World, refinery: ObjectId) {
    let Some(o) = w.obj(refinery).cloned() else {
        return;
    };
    // spec: S-0616 claim 1 (a smoke event on a vehicle, infantry or aircraft object makes nothing; a building need not be a refinery, only its type's refinery smoke type is tested)
    if o.kind != Kind::Building {
        return;
    }
    let rules = w.rules_arc();
    let Some(b) = rules.buildings.get(o.type_index as usize) else {
        return;
    };
    let Some(ti) = b
        .raw
        .refinery_smoke_particle_system
        .as_ref()
        .and_then(|n| rules.gas.system(n))
    else {
        return;
    };
    let points = b.raw.refinery_smoke_offsets.clone().unwrap_or_default();
    // spec: S-0616 claim 1 (a point of (0, 0, 0) or (128, 128, 0) counts as unset; up to four points)
    for p in points.iter().take(4) {
        if UNSET_POINTS.contains(p) {
            continue;
        }
        let Some(sys) = crate::particles::create(w, ti, offset(o.pos, *p), None) else {
            continue;
        };
        if let Some(s) = w.gas.systems.get_mut(&sys) {
            // spec: S-0616 claim 1 (owner object the building that makes it; lifetime the game-wide refinery-smoke time, not its type's; no draw)
            s.owner_object = Some(refinery);
            s.owner_offset = *p;
            s.lifetime = rules.gas.refinery_frames;
        }
    }
}

/// The building's natural smoke system, if it is still in play.
fn natural_of(w: &World, id: ObjectId) -> Option<ObjectId> {
    w.obj(id)?
        .combat
        .natural_smoke
        .filter(|s| w.gas.systems.contains_key(s))
}

/// S-0616 claim 1, natural smoke: a building whose type names a natural
/// particle system gets one when it is placed in play, if it has none, at
/// its position plus the natural-smoke displacement. `uncloak`: the
/// S-0516 claim 18 case, which also needs a displacement other than
/// (0, 0, 0).
pub fn natural_smoke(w: &mut World, id: ObjectId, uncloak: bool) {
    let Some(o) = w.obj(id).cloned() else { return };
    if o.kind != Kind::Building {
        return;
    }
    let Some(tc) = crate::combat::veterancy::type_combat(w, id).cloned() else {
        return;
    };
    let Some(ti) = tc
        .natural_system
        .as_ref()
        .and_then(|n| w.rules().gas.system(n))
    else {
        return;
    };
    // spec: S-0616 claim 1 (if it has none) and S-0516 claim 18 (at full uncloaking only when the displacement is not (0, 0, 0))
    if natural_of(w, id).is_some() || (uncloak && tc.natural_offset == [0; 3]) {
        return;
    }
    let Some(sys) = crate::particles::create(w, ti, offset(o.pos, tc.natural_offset), None) else {
        return;
    };
    if let Some(s) = w.gas.systems.get_mut(&sys) {
        // spec: S-0616 claim 1 (natural smoke: no owner object; target point a point of the building, its position, unused)
        s.target = o.pos;
    }
    if let Some(ob) = w.obj_mut(id) {
        ob.combat.natural_smoke = Some(sys);
    }
}

/// S-0516 claim 18 and S-0616 claim 1: a building reaching full cloaking
/// loses its natural smoke system at once, with its particles.
pub fn natural_smoke_cloaked(w: &mut World, id: ObjectId) {
    let Some(sys) = natural_of(w, id) else { return };
    // spec: S-0616 claim 1 and S-0516 claim 18 (full cloaking removes the system at once, with its particles; it is not a mark)
    w.gas.systems.remove(&sys);
    let skip = w.rules().quirks.update_order_removal_skip;
    // spec: S-0203 claim 10 (a removal from the update order, counting during the turn pass)
    w.update.leave(sys, skip);
    if let Some(ob) = w.obj_mut(id) {
        ob.combat.natural_smoke = None;
    }
}

/// S-0616 claim 1: after each hit on the building that reports *now
/// yellow* (S-0302 claim 12), its natural smoke's emission timer is
/// multiplied by 1.5 in single precision. A *now red* hit does not call
/// this. Repair does not undo the factor and each later *now yellow* hit
/// multiplies again, so it compounds.
pub fn natural_smoke_yellow(w: &mut World, id: ObjectId) {
    let Some(sys) = natural_of(w, id) else { return };
    let m = w.rules().mode;
    let Some(s) = w.gas.systems.get_mut(&sys) else {
        return;
    };
    // spec: S-0616 claim 1 and Constants (natural-smoke yellow factor 1.5 on T, in single precision: the product of a single and 1.5 is exact in double, so one narrowing gives the single-precision product; it compounds)
    s.timer = s
        .timer
        .to_f64()
        .mul(crate::rules::nearest_double("1.5"), m)
        .to_f32(m);
}

/// S-0616 claim 1, damage sparks: an infantry object of a cyborg type,
/// strictly below the yellow ratio, higher than 10 leptons below the
/// surface and holding no spark emitter, may make a spark system from the
/// spark-kind entries of its damage list. Called in its turn after its
/// behaviour processing and self-healing, before its radiation hit,
/// movement and infantry firing (S-0616 claim 1, Q-1404 answered).
pub fn damage_sparks(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else { return };
    if o.kind != Kind::Infantry || o.combat.spark_emitter.is_some() {
        return;
    }
    // spec: S-0616 claim 1 (a turn with the object not on the map makes no damage sparks and takes none of their draws)
    if !o.on_map {
        return;
    }
    let rules = w.rules_arc();
    // spec: S-0616 Terms (the damage-sparks flag is the cyborg flag of S-0312 claim 2)
    if !rules.unit(o.kind, o.type_index).is_some_and(|u| u.cyborg) {
        return;
    }
    let m = rules.mode;
    let s = crate::combat::damage::strength(w, id);
    // spec: S-0214 Terms (health ratio: health / strength in floating point)
    let ratio = F64::from_i64(i64::from(o.health), m).div(F64::from_i64(i64::from(s), m), m);
    // spec: S-0616 claim 1 (strictly below the yellow ratio; height above the surface greater than −10 leptons)
    if !ratio.lt(rules.general.condition_yellow) || w.height_above_surface(id) <= SPARK_DEPTH {
        return;
    }
    let Some(tc) = crate::combat::veterancy::type_combat(w, id).cloned() else {
        return;
    };
    let n = tc.spark_systems.len() as i32;
    if n == 0 {
        // spec: S-0616 claim 1 (no spark-kind entry: nothing is made, no draw)
        return;
    }
    let half = rules.quirks.ranged_draw_half_span;
    // spec: S-0616 claim 1 (one ranged draw R(0, 2147483646) each turn, times 4.656612877414201 × 10⁻¹⁰ in double precision)
    let r = w.rng.ranged(0, DRAW_TOP, half);
    let x = F64::from_i64(i64::from(r), m).mul(spark_scale(), m);
    // spec: S-0616 claim 1 (the red probability when the health ratio is below the red ratio, the yellow one otherwise)
    let p = if ratio.lt(rules.general.condition_red) {
        rules.gas.sparking.0
    } else {
        rules.gas.sparking.1
    };
    if !x.lt(p) {
        return;
    }
    // spec: S-0616 claim 1 (one ranged draw R(0, n − 1) picks a spark-kind entry, counted from the start of the list)
    let k = w.rng.ranged(0, n - 1, half);
    let Some(ti) = rules.gas.system(&tc.spark_systems[k as usize]) else {
        return;
    };
    // spec: S-0616 claim 1 (at the object's position plus the type's damage-smoke displacement)
    let Some(sys) = crate::particles::create(w, ti, offset(o.pos, tc.smoke_offset), None) else {
        return;
    };
    if let Some(sy) = w.gas.systems.get_mut(&sys) {
        // spec: S-0616 claim 1 (damage sparks: owner object the object, target point the world origin, no first particle)
        sy.owner_object = Some(id);
        sy.owner_offset = tc.smoke_offset;
    }
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0616 claim 1 (held in the spark emitter place, shared with an effect weapon's spark emitter)
        ob.combat.spark_emitter = Some(sys);
    }
}

/// The spark chance scale of S-0616 claim 1 and F10.
fn spark_scale() -> F64 {
    // spec: S-0616 Constants (spark chance scale 4.656612877414201 × 10⁻¹⁰)
    crate::rules::nearest_double("4.656612877414201e-10")
}

/// S-0616 claim 1, sparks of a deactivated vehicle: on the ticks for which
/// (trunc(x / 10) + trunc(y / 10) × 65536 + t + object number) is a multiple of 30,
/// one system of the game-wide spark type, displaced by two draws
/// R(−100, 100), y first. In our engine a deactivated vehicle is one
/// disabled by S-0405 claim 13 (Terms). Called first in its turn, before
/// every other draw of that turn (S-0616 claim 1, Q-1404 answered).
pub fn deactivated_sparks(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else { return };
    if o.kind != Kind::Vehicle || !crate::econ::power::unit_disabled(w, id) {
        return;
    }
    let t = w.tick;
    // spec: S-0616 claim 1 (the cadence term, integer arithmetic truncating toward zero; the object-number phase is a departure)
    let term = (o.pos.x / DEACTIVATED_DIVISOR)
        .wrapping_add((o.pos.y / DEACTIVATED_DIVISOR).wrapping_mul(DEACTIVATED_WEIGHT))
        .wrapping_add(t)
        .wrapping_add(o.id.0 as i32);
    if glasnost_num::int::rem(term, DEACTIVATED_PERIOD) != 0 {
        return;
    }
    let Some(ti) = w.rules().gas.default_spark else {
        // spec: S-0616 claim 1 (with no game-wide spark type nothing is created and no draw is taken)
        return;
    };
    let half = w.rules().quirks.ranged_draw_half_span;
    // spec: S-0616 claim 1 (two ranged draws R(−100, 100) give the y and then the x displacement)
    let dy = w.rng.ranged(-DEACTIVATED_SPREAD, DEACTIVATED_SPREAD, half);
    let dx = w.rng.ranged(-DEACTIVATED_SPREAD, DEACTIVATED_SPREAD, half);
    // spec: S-0616 claim 1 (deactivated vehicle sparks: no owner object, target point unused, no first particle)
    crate::particles::create(w, ti, offset(o.pos, [dx, dy, 0]), None);
}

/// S-0011 code 88 and S-0616 claim 1: one system of type `ti` (the number
/// slot, a registration index) at the centre of cell `c`, at its ground
/// surface height (no bridge lift), with no owner object, no owner house
/// and no first particle.
pub fn trigger_create(w: &mut World, ti: i32, c: CellXY) {
    let Some(i) = u32::try_from(ti)
        .ok()
        .filter(|&i| (i as usize) < w.rules().gas.systems.len())
    else {
        w.note("S-0616 claim 1: the trigger action names no particle-system type; nothing is made");
        return;
    };
    // spec: S-0616 claim 1 (at the centre of the cell of the action's waypoint, at the ground surface height of that cell, not raised on a bridge)
    let p = crate::superweapons::cell_point(w, c, false);
    crate::particles::create(w, i, p, None);
}

/// S-0011 code 89 and S-0616 claim 1: marks every particle system in play
/// whose position lies in cell `c`, whatever its kind, type, owner or age.
pub fn trigger_mark(w: &mut World, c: CellXY) {
    // spec: S-0616 claim 1 (every particle system in play whose position lies in the waypoint's cell, gas systems and the shared gas system included; nothing is removed at once)
    for s in w.gas.systems.values_mut() {
        if s.pos.cell() == c {
            s.ending = true;
        }
    }
}

/// The sparks of an electric bolt (S-0616 claim 1, electric bolt row;
/// S-0316 claim 6): one system of the game-wide spark type at the bolt
/// end `end`, with no owner object, no owner house and no first particle,
/// made with the bolt whether or not the bolt is ever drawn. Without a
/// listed game-wide spark type nothing is made (our engine's departure, with
/// the importer's warning).
pub fn electric_bolt_sparks(w: &mut World, end: Pos) {
    // spec: S-0616 claim 1 (electric bolt: at the bolt's end, no owner object, target point unused, no first particle) and S-0316 claim 6 (absent spark type: no system, in our engine)
    crate::particles::spark_system(w, end);
}

/// The attached system of a debris piece (S-0616 claim 1, voxel-model
/// animation row; S-0317 claim 13): a system of type `ti` at the piece's
/// creation point `at`, with the piece `piece` as its owner object and no
/// owner house, keeping the displacement from the piece's position `pos`
/// that it has at creation (zero for a falling piece; the whole flight for
/// a meteor piece). It is entered in the update order directly after its
/// piece (S-0317 Open questions: our engine's design). It is marked when
/// the piece leaves play (`crate::pieces`, S-0616 claim 4).
pub fn voxel_animation_system(w: &mut World, ti: u32, piece: ObjectId, at: Pos, pos: Pos) {
    // spec: S-0317 claim 13 (made at the creation point, not the start position, after the piece's own draws; no draw; no owner house)
    let Some(sys) = crate::particles::create(w, ti, at, None) else {
        return;
    };
    if let Some(s) = w.gas.systems.get_mut(&sys) {
        // spec: S-0317 claim 13 and S-0616 claim 5 (the piece is the owner object; the displacement from the piece at creation is kept)
        s.owner_object = Some(piece);
        s.owner_offset = [
            at.x.wrapping_sub(pos.x),
            at.y.wrapping_sub(pos.y),
            at.z.wrapping_sub(pos.z), // nocite: vector and matrix dimensions (three axes)
        ];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glasnost_num::Rounding;

    #[test]
    fn unset_points() {
        assert!(UNSET_POINTS.contains(&[128, 128, 0]));
        assert!(!UNSET_POINTS.contains(&[128, 128, 1]));
    }

    #[test]
    fn spark_scale_maps_the_top_to_one() {
        let m = Rounding::Nearest;
        let x = F64::from_i64(i64::from(DRAW_TOP), m).mul(spark_scale(), m);
        assert!(x.eq_num(F64::from_i64(1, m)));
    }
}
