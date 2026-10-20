//! Shot animations (S-0301 claims 20 to 23): which animation an ordinary
//! shot makes, the shot facing that picks it from an eight-entry list,
//! where it is created and what it follows.
//!
//! The simulation takes the animation's creation draws (S-0202 claim 10,
//! S-0109) and reports it as a presentation event; the drawing plays it.

use glasnost_num::geom::{Facing, eight_direction, n_step};

use crate::combat::data::WeaponId;
use crate::combat::effects::Effect;
use crate::rules::Kind;
use crate::world::{ObjectId, Pos, World};

/// The animation list length chosen by facing (S-0301 claim 20 a).
pub const FACING_LIST: usize = 8; // spec: S-0301 Constants (facing-chosen animation list length: 8 entries)

/// The entry shift of an eight-entry list (S-0301 F7).
const ENTRY_SHIFT: u32 = 1; // spec: S-0301 Constants (entry shift of an eight-entry list: 1)

/// An occupied building's shot-animation depth adjustment (S-0301 F8).
const OCCUPIED_DEPTH: i32 = -200; // spec: S-0301 Constants (occupied building's shot-animation depth adjustment: −200)

/// The divisor of a building's shot-animation depth adjustment (S-0301 F8).
const DEPTH_DIVISOR: i32 = 4; // spec: S-0301 Constants (divisor of a building's shot-animation depth adjustment: 4)

/// A building's facing rounding before the 8-direction (S-0301 claim 21).
const BUILDING_IDLE_STEPS: u32 = 256; // spec: S-0301 claim 21 (a building without a target: its turret facing rounded to the 256-step facing)
const SPRITE_TURRET_STEPS: u32 = 32; // spec: S-0301 claim 21 (a sprite-turreted building with a target: rounded to the 32-step facing)

/// The firer's shot facing (S-0301 claim 21) and, for a building row that
/// rounds first, the step count *n* of that first rounding (F7).
pub fn shot_facing(w: &World, firer: ObjectId) -> (Facing, Option<u32>) {
    let t = w.tick;
    let Some(o) = w.obj(firer) else {
        return (0, None);
    };
    let r = w.rules();
    match o.kind {
        Kind::Vehicle => {
            let turret = r.unit(o.kind, o.type_index).is_some_and(|u| u.turret);
            // spec: S-0301 claim 21 (a vehicle whose own type has the turret flag: its turret facing; any other vehicle: its body facing)
            if turret {
                (o.turret.value(t), None)
            } else {
                (o.body.value(t), None)
            }
        }
        // spec: S-0301 claim 21 (infantry, a passenger firing out included: its body facing)
        Kind::Infantry => (o.body.value(t), None),
        // spec: S-0301 claim 21 (an aircraft: its turret facing, the one S-0310 claim 5 compares)
        Kind::Aircraft => (o.turret.value(t), None),
        Kind::Building => {
            let Some(target) = o.target else {
                // spec: S-0301 claim 21 (a building without a target: its turret facing rounded to 256 steps)
                return (o.turret.value(t), Some(BUILDING_IDLE_STEPS));
            };
            let b = &r.buildings[o.type_index as usize];
            // spec: S-0301 claim 21 and S-0310 Terms (a turreted building whose voxel-turret flag is not set has a sprite turret)
            if b.turret && !b.building.voxel_turret {
                return (o.turret.value(t), Some(SPRITE_TURRET_STEPS));
            }
            // spec: S-0301 claim 21 (any other building with a target: the direction of the target from its aiming point, S-0310 claim 4)
            let from = crate::combat::check::building_aim_point(w, firer);
            let to = crate::combat::range::target_point(w, target);
            (crate::combat::check::direction_to(w, from, to), None)
        }
        Kind::Terrain => (0, None),
    }
}

/// Formula F7 of S-0301: the entry of an eight-entry list for shot facing
/// `f`, rounded first to `pre` steps when a building's row says so.
pub fn eight_entry(f: Facing, pre: Option<u32>) -> usize {
    // spec: S-0301 F7 (f′ = (65536 / n) × (⌊f × n / 65536 + ½⌋ mod n) with n = 256 or 32)
    let f = match pre {
        Some(n) => ((65536 / n) * n_step(f, n)) as Facing,
        None => f,
    };
    // spec: S-0301 F7 (d = ⌊f / 8192 + ½⌋ mod 8, S-0204 F4 with n = 8; e = (d + 1) mod 8)
    let d = u32::from(eight_direction(f));
    ((d + ENTRY_SHIFT) % FACING_LIST as u32) as usize
}

/// The shot animation type a shot of `weapon` by `firer` makes (S-0301
/// claim 20), if any. Choosing takes no draw.
pub fn choose(w: &World, firer: ObjectId, weapon: WeaponId) -> Option<String> {
    let r = w.rules();
    let wr = r.combat.weapon(weapon);
    let o = w.obj(firer)?;
    if o.kind == Kind::Building && crate::structures::garrison::garrisoned(w, firer) {
        // spec: S-0301 claim 20 b (a garrisoned building: the occupant animation replaces the list's choice; none without one)
        return wr.occupant_anim.clone();
    }
    let n = wr.anim.len();
    let listed = match n {
        // spec: S-0301 claim 20 a (n = 0: no type)
        0 => None,
        // spec: S-0301 claim 20 a (n = 8: entry (d + 1) mod 8 by the shot facing, F7)
        FACING_LIST => {
            let (f, pre) = shot_facing(w, firer);
            wr.anim.get(eight_entry(f, pre)).cloned()
        }
        // spec: S-0301 claim 20 a (any other n ≥ 1: entry 0, whatever the facing)
        _ => wr.anim.first().cloned(),
    };
    if listed.is_some() {
        return listed;
    }
    // spec: S-0301 claim 20 c (no type chosen and a passenger firing out: the weapon's open-topped animation)
    let firing_out = o.kind == Kind::Infantry && crate::transport::open_topped_passenger(w, firer);
    if firing_out {
        return wr.open_topped_anim.clone();
    }
    None
}

/// Formula F8 of S-0301: a building's shot-animation depth adjustment for
/// a launch point at map y `yl`.
pub fn building_depth(w: &World, building: ObjectId, yl: i32) -> i32 {
    // spec: S-0301 F8 (z_a = −200 when the building has at least one occupant)
    if crate::structures::garrison::count(w, building) > 0 {
        return OCCUPIED_DEPTH;
    }
    // spec: S-0301 F8 (y₀ = 256 × the smallest cell y of the foundation)
    let y0 = w.obj(building).map_or(0, |o| 256 * o.pos.cell().y);
    // spec: S-0301 F8 (z_a = min(0, −trunc((y_l − y₀) / 4)), signed 32-bit, truncated toward zero)
    0.min(glasnost_num::int::div(yl.wrapping_sub(y0), DEPTH_DIVISOR).wrapping_neg())
}

/// Creates the shot animation of a shot of `weapon` by `firer` launched
/// from `launch` (S-0301 claims 20 and 22), if one is chosen: its creation
/// draws in the firer's turn (S-0109 claim 29, S-0202 claim 10), then the
/// presentation event. The caller places it after the projectile and the
/// delay draw, and leaves out the shots of claim 23.
pub fn create(w: &mut World, firer: ObjectId, weapon: WeaponId, launch: Pos) {
    // spec: S-0301 claim 23 (a shot for which claim 20 chooses no type makes no shot animation and gets no placeholder)
    let Some(name) = choose(w, firer, weapon) else {
        return;
    };
    let Some(o) = w.obj(firer) else { return };
    let building = o.kind == Kind::Building;
    let at = o.pos;
    // spec: S-0301 claim 22 (created with no initial loop delay, repeat factor 1 and no reverse request: the creation draws of S-0202 claim 10)
    let delay = crate::load::anim_created_at(w, &name, Some((launch.x, launch.y, launch.z)));
    let (attached, z_adjust) = if building {
        // spec: S-0301 claim 22 (a building's shot animation is not attached; its creator replaces its depth adjustment with z_a of F8)
        (None, Some(building_depth(w, firer, launch.y)))
    } else {
        // spec: S-0301 claim 22 and S-0109 claim 22 (attached to a firer that is not a building, at its displacement from the firer, height included)
        let offset = Pos::new(launch.x - at.x, launch.y - at.y, launch.z - at.z);
        (Some((firer, offset)), None)
    };
    w.effects.push(Effect::ShotAnim {
        name,
        pos: launch,
        delay,
        attached,
        z_adjust,
    });
}
