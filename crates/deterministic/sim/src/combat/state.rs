//! The combat state of objects, houses and projectiles: plain serializable
//! data, saved and hashed with the rest of the world.

use glasnost_map::CellXY;
use glasnost_num::{F32, F64};
use serde::{Deserialize, Serialize};

use crate::combat::data::WeaponId;
use crate::world::{HouseId, ObjectId, Pos, Target};

/// One object's combat state (infantry, vehicles, aircraft, buildings).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatState {
    /// The tick from which the rearm countdown is over (S-0301 claim 1).
    pub rearm_until: i32,
    /// The delay D last chosen (S-0301 claim 16).
    pub rearm_delay: i32,
    /// Burst counter k (S-0301 claim 4).
    pub burst: i32,
    /// Ammunition a (S-0301 claim 10; −1 unlimited).
    pub ammo: i32,
    /// Reload countdown: ticks left, `None` when never started (S-0301
    /// claim 11, which counts it as over).
    pub reload_left: Option<i32>,
    /// Experience, a single (S-0311 claim 1).
    pub experience: F32,
    /// Expected remaining health (S-0309 claim 15).
    pub expected_health: i32,
    /// Weapon number (S-0308 claim 5).
    pub weapon_number: i32,
    /// Gattling stage (S-0308 Terms; no spec raises it yet).
    pub gattling_stage: i32,
    /// Firing-frame counter (S-0301 claim 13): `None` idle.
    pub firing_frame: Option<i32>,
    /// A building's pending delayed shot: the tick of its turn (S-0301
    /// claim 14).
    pub delayed_shot: Option<i32>,
    /// Radial fire segment index of the next shot (S-0306 claim 7).
    pub radial_index: i32,
    /// Spray shift number s (S-0313 claim 9).
    pub spray_shift: i32,
    /// Object firepower multiplier F_o (S-0302 claim 4), a double.
    pub firepower_mult: F64,
    /// Object armour multiplier A_o (S-0302 claim 4), a double.
    pub armour_mult: F64,
    /// The target is an opportunity target (S-0309 claim 24).
    pub opportunity: bool,
    /// Engaged mark (S-0309 claim 23).
    pub engaged: bool,
    /// Limited-scan mark (S-0309 claim 22).
    pub limited_scan: bool,
    /// Fired-at list of a distributed-fire object (S-0309 claim 20).
    pub fired_at: Vec<ObjectId>,
    /// Killed: the hit that took its last health has landed (S-0302 Terms).
    pub killed: bool,
    /// The tick at which a delayed kill ends (S-0302 claim 16).
    pub delay_kill_until: Option<i32>,
    /// Has a damage particle effect (S-0302 claim 13).
    pub damage_effect: bool,
    /// Berserk until this tick (seam: no berserk warhead's start rule).
    pub berserk_until: Option<i32>,
    /// The turret was turning at the end of the previous tick (S-0310
    /// claim 3).
    pub turret_was_turning: bool,
    /// The tick of the last aim by the attack processing (S-0215 claim 32).
    pub last_shot_target: Option<Target>,
}

impl CombatState {
    /// The state of a new object.
    pub fn new(health: i32, ammo: i32, one: F64) -> Self {
        Self {
            rearm_until: i32::MIN,
            rearm_delay: 0,
            burst: 0,
            ammo,
            reload_left: None,
            experience: F32::ZERO,
            expected_health: health,
            weapon_number: 0,
            gattling_stage: 0,
            firing_frame: None,
            delayed_shot: None,
            radial_index: 0,
            spray_shift: 0,
            firepower_mult: one,
            armour_mult: one,
            opportunity: false,
            engaged: false,
            limited_scan: false,
            fired_at: Vec::new(),
            killed: false,
            delay_kill_until: None,
            damage_effect: false,
            berserk_until: None,
            turret_was_turning: false,
            last_shot_target: None,
        }
    }
}

/// A house's combat state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HouseCombat {
    /// Rate-of-fire factor h (S-0301 claim 3, F2), a double.
    pub rof_factor: F64,
    /// Firepower multiplier F_h (S-0302 claim 3), a double.
    pub firepower: F64,
    /// Anger towards each house, by house index (S-0309 claim 16).
    pub anger: Vec<i32>,
    /// Enemy house (S-0309 claim 16).
    pub enemy: Option<HouseId>,
    /// Score total from kill values (S-0311 claim 7).
    pub score: i32,
    /// Destroyed-object count, credited (S-0302 claim 16).
    pub kills: i32,
    /// Objects lost.
    pub losses: i32,
    /// Infantry and vehicle self-heal totals (S-0314 claim 10).
    pub self_heal: [i32; 2], // spec: S-0314 claim 10 (two running totals)
}

/// A projectile in flight (S-0306), an object of the update order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Projectile {
    /// Its object number (S-0306 Terms).
    pub id: ObjectId,
    /// The weapon that fired it.
    pub weapon: WeaponId,
    /// The attacker its detonation carries (S-0302 claim 1): cleared when
    /// the firer is killed or removed.
    pub attacker: Option<ObjectId>,
    /// The attacking house.
    pub house: Option<HouseId>,
    /// The firer as fired (for the house tests of the flight rows).
    pub firer: Option<ObjectId>,
    /// The firer's house at the shot (allied tests of T1).
    pub firer_house: Option<HouseId>,
    /// Position, leptons.
    pub pos: Pos,
    /// Velocity, three doubles, leptons per tick.
    pub vel: [F64; 3], // nocite: three components x, y, z
    /// The projectile's target (S-0306 claim 17).
    pub target: Option<Target>,
    /// The stored target point K (S-0306 claim 9).
    pub stored: Pos,
    /// The placement position (for the pass-by row's first value).
    pub placed_at: Pos,
    /// Arm time, ticks.
    pub arm: i32,
    /// Tick of placement.
    pub placed: i32,
    /// Damage amount carried (S-0302 claim 2, less degeneration).
    pub damage: i32,
    /// The recorded value Ω of the pass-by row (S-0306 T1).
    pub omega: i32,
    /// The top speed S (steering, vertical).
    pub top_speed: i32,
    /// Turns taken.
    pub turns: i32,
    /// Unlocked turns counted (steering, S-0306 T1 closing average).
    pub unlocked: i32,
    /// Course lock still on.
    pub locked: bool,
    /// The closing average C (a double).
    pub closing: F64,
    /// The cell at the end of its previous turn (line-of-fire in flight).
    pub prev_cell: CellXY,
    /// The fire point (line-of-fire in flight, X₁).
    pub fire_point: Pos,
    /// Waiting for the nuclear fireball until this tick (S-0306 claim 25).
    pub nuke_wait: Option<i32>,
    /// Fired by a vehicle from a scalable type: counted in *N* of S-0306
    /// claim 16.
    pub vehicle_scalable: bool,
}
