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
    /// Gattling stage (S-0308 Terms, S-0609 claims 1 to 4).
    pub gattling_stage: i32,
    /// Gattling spin value (S-0609 Terms and claim 1).
    #[serde(default)]
    pub spin: i32,
    /// The animation-frame counter (S-0105 claim 16; S-0609 claim 14 owns
    /// its advances for gattling-flagged types). Presentation only: no
    /// rule reads it.
    #[serde(default)]
    pub anim_frame: i32,
    /// Firing-frame counter (S-0301 claim 13): `None` idle.
    pub firing_frame: Option<i32>,
    /// A building's pending delayed shot: the tick of its turn (S-0301
    /// claim 14).
    pub delayed_shot: Option<i32>,
    /// The superweapon warp member that a revert last gave back as this
    /// holder's target while the member was off the map (S-0309 claim 27,
    /// S-0513 r108 claim 39a): claim 27's case, so that member is no target
    /// in the firing step and is dropped at the start of the holder's next
    /// turn, whatever temporal beam the holder has attached to it
    /// (`combat::held`). Cleared at that turn's start.
    #[serde(default)]
    pub revert_gave_departed: Option<ObjectId>,
    /// The weapon slot a building's delayed shot was started with: the
    /// check and the shot at the end of the delay use it, not a fresh
    /// choice (S-0301 claim 14, S-0308 claim 14).
    #[serde(default)]
    pub delayed_slot: u8,
    /// A planted bomb the object carries (S-0519 Terms).
    #[serde(default)]
    pub bomb: Option<crate::bomb::Bomb>,
    /// A building's recorded charge planter (S-0519 claim 14).
    #[serde(default)]
    pub charge_planter: Option<ObjectId>,
    /// A prism master's support count *N* (S-0518 Terms).
    #[serde(default)]
    pub support: i32,
    /// A prism supporter's pending beam: the tick it comes and the master's
    /// fire point at recruitment (S-0518 claim 6).
    #[serde(default)]
    pub support_beam: Option<(i32, Pos)>,
    /// An overpowerable building's registered chargers (S-0518 Terms).
    #[serde(default)]
    pub chargers: Vec<ObjectId>,
    /// The number of target changes made so far (S-0519 claim 19): a
    /// designator's strike is cancelled by any change, even one back to the
    /// target it had.
    #[serde(default)]
    pub target_epoch: u32,
    /// A building's acquisition rank (S-0518 claim 3): the number of the
    /// acquisition that last made it one of its house's buildings, from
    /// `World::acquisitions`.
    #[serde(default)]
    pub acquired: u64,
    /// Whether an overpowerable building is overpowered (S-0518 claim 13).
    #[serde(default)]
    pub overpowered: bool,
    /// The charging mark of a unit charging a building by itself (S-0518
    /// claim 14).
    #[serde(default)]
    pub charging: bool,
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
    /// The damage-smoke system held in the damage-smoke emitter place
    /// (S-0302 claim 13, S-0616 Terms and claim 1), until it leaves play.
    pub damage_smoke: Option<crate::world::ObjectId>,
    /// The berserk state and its remaining berserk time in countdown
    /// updates (S-0610 claims 4 and 5, F2): `Some(t)` while berserk, even
    /// when `t` is 0 or less (a hit with *T* ≤ 0 under
    /// `quirks.berserk_zero_duration` stays berserk until the unit's next
    /// countdown update); `None` when not berserk.
    pub berserk: Option<i32>,
    /// The turret was turning at the end of the previous tick (S-0310
    /// claim 3).
    pub turret_was_turning: bool,
    /// The tick of the last aim by the attack processing (S-0215 claim 32).
    pub last_shot_target: Option<Target>,
    /// Hostile-hit mark (S-0314 Terms): a surviving hit by a known attacker
    /// not allied from this object's owner's side. Never cleared.
    #[serde(default)]
    pub hostile_hit: bool,
    /// A building's captured mark (S-0314 Terms): it has changed owner.
    /// Never cleared.
    #[serde(default)]
    pub captured: bool,
    /// A building has no crew (S-0312 F5): its killing damage ignored
    /// defences (claims 18 and 21), the only way the state arises (claim
    /// 18, revision 51: no map field and no other event sets it). Read by
    /// the second survivor round of claim 21.
    #[serde(default)]
    pub no_crew: bool,
    /// A building's computer-repair map flag (S-0314 claim 13, S-0009
    /// claim 40).
    #[serde(default)]
    pub computer_repair: bool,
    /// The fire emitter place (S-0616 Terms and claim 1): the effect
    /// weapon's fire system while the object holds it.
    #[serde(default)]
    pub fire_emitter: Option<FireEmitter>,
    /// The spark emitter place (S-0616 Terms): an effect weapon's spark
    /// system or a damage-sparks system, until it leaves play.
    #[serde(default)]
    pub spark_emitter: Option<crate::world::ObjectId>,
    /// The railgun emitter place (S-0616 Terms): an effect weapon's railgun
    /// system, until it leaves play.
    #[serde(default)]
    pub railgun_emitter: Option<crate::world::ObjectId>,
    /// A building's natural smoke system (S-0616 claim 1); a system that
    /// has left play counts as none.
    #[serde(default)]
    pub natural_smoke: Option<crate::world::ObjectId>,
    /// A building's flash count and air-strike mark sequence (S-0519
    /// claims 25 and 26).
    #[serde(default)]
    pub look: crate::mark::Look,
}

/// A held fire emitter (S-0616 claim 1): the system, the slot of the weapon
/// whose shot made it and the target the firer held at the shot or last
/// took within reach (for the let-go on a retarget).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FireEmitter {
    /// The fire system.
    pub system: crate::world::ObjectId,
    /// The weapon slot of the shot.
    pub slot: u8,
    /// The firer's target as last seen by the emitter.
    pub target: Option<Target>,
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
            // spec: S-0609 claim 1 (created with spin value 0 and stage 0); S-0105 claim 16 (the animation-frame counter is 0 at creation)
            gattling_stage: 0,
            spin: 0,
            anim_frame: 0,
            firing_frame: None,
            delayed_shot: None,
            revert_gave_departed: None,
            delayed_slot: 0,
            bomb: None,
            charge_planter: None,
            support: 0,
            support_beam: None,
            chargers: Vec::new(),
            acquired: 0,
            target_epoch: 0,
            overpowered: false,
            charging: false,
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
            damage_smoke: None,
            berserk: None,
            turret_was_turning: false,
            last_shot_target: None,
            hostile_hit: false,
            captured: false,
            no_crew: false,
            computer_repair: false,
            fire_emitter: None,
            spark_emitter: None,
            railgun_emitter: None,
            natural_smoke: None,
            // spec: S-0519 claims 25 and 26 (no flash, before the first marked turn)
            look: crate::mark::Look::default(),
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
    /// The automatic-repair hold (S-0314 Terms, claims 13 and 15).
    #[serde(default)]
    pub repair_hold: bool,
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
    /// The damage multiplier *M* in 1/256 (S-0518 claim 9): 256 unless a
    /// supported prism shot made it larger.
    #[serde(default = "unscaled")]
    pub multiplier: i32,
    /// Fired by a vehicle from a scalable type: counted in *N* of S-0306
    /// claim 16.
    pub vehicle_scalable: bool,
    /// One of the nine children of an airburst projectile, which carry no
    /// weapon of their own and leave no radiation (S-0525 claim 1).
    #[serde(default)]
    pub airburst_child: bool,
    /// The chain level of S-0318 Terms: 0 for a projectile that is not a
    /// fragment, d + 1 for a fragment launched by a projectile of level d.
    /// A fragment carries no weapon of its own and leaves no radiation
    /// (S-0318 claim 5).
    #[serde(default)]
    pub chain: u8,
    /// An airburst child launched through a link on a loop of airburst
    /// weapons: it launches no children through such a link itself (#203;
    /// [`crate::loops`]).
    #[serde(default)]
    pub looped: bool,
}

/// The unscaled damage multiplier of a projectile, ×1 in 1/256 (S-0518
/// claim 9).
// spec: S-0518 Constants (unscaled damage multiplier 256)
fn unscaled() -> i32 {
    256
}
