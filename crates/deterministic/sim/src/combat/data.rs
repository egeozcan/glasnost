//! The combat part of the rules: weapons, projectiles and warheads indexed
//! by name, each type's combat fields, and the game-wide combat values, every
//! field resolved with the engine default that its spec states.

use std::collections::BTreeMap;

use glasnost_num::{F64, Rounding};
use glasnost_rules::sim::{
    self as rs, ArmourClass, CombatFields, Decimal, DifficultyRow, Flh, ProjectileType,
    WarheadType, WeaponType,
};

/// Index of a weapon in [`CombatRules::weapons`].
pub type WeaponId = u32;
/// Index of a projectile type in [`CombatRules::projectiles`].
pub type ProjectileId = u32;
/// Index of a warhead in [`CombatRules::warheads`].
pub type WarheadId = u32;

/// The number of weapon slots of a type (S-0308 Terms).
// spec: S-0308 Constants (18 weapon slots, numbered 0 to 17)
pub const SLOTS: usize = 18;

/// A double from the exact value of a decimal field.
fn dec(d: &Option<Decimal>, mode: Rounding) -> Option<F64> {
    d.as_ref().map(|d| d.value(mode))
}

/// A single-precision field widened to a double.
fn single(d: &Option<Decimal>, mode: Rounding) -> Option<F64> {
    d.as_ref().map(|d| d.single_value(mode))
}

/// The double 1.0.
pub fn one(mode: Rounding) -> F64 {
    F64::from_i64(1, mode)
}

/// The lepton form of a range field (S-0307 claim 1): the single-precision
/// decimal × 256 in double precision, truncated toward zero.
pub fn leptons(d: &Decimal, mode: Rounding) -> i32 {
    // spec: S-0307 claim 1 (R₀ = trunc(RZ₅₃(x × 256)), x read as a single)
    d.single_value(mode)
        .mul(F64::from_i64(256, mode), mode)
        .to_i32_common()
}

/// A weapon's resolved fields.
#[derive(Clone, Debug, Default)]
pub struct WeaponRules {
    /// Name.
    pub name: String,
    /// Damage field D.
    pub damage: i32,
    /// Ambient-damage field.
    pub ambient: i32,
    /// Projectile type.
    pub projectile: Option<ProjectileId>,
    /// Warhead.
    pub warhead: Option<WarheadId>,
    /// Rate-of-fire field R, ticks.
    pub rof: i32,
    /// Burst count B (below 1 read as 1, a departure of S-0301 Edge cases).
    pub burst: i32,
    /// Stored speed, leptons per tick (S-0306 Import mapping).
    pub speed: i32,
    /// Weapon range R₀, leptons (S-0307 claim 1).
    pub range: i32,
    /// Minimum range, leptons.
    pub min_range: i32,
    /// Cell rangefinding.
    pub cell_rangefinding: bool,
    /// Sonic-wave weapon.
    pub sonic: bool,
    /// Spark-particle weapon.
    pub spark: bool,
    /// Fire-particle weapon.
    pub fire_particles: bool,
    /// Railgun weapon.
    pub railgun: bool,
    /// Magnetic-beam weapon.
    pub mag_beam: bool,
    /// Temporal weapon.
    pub temporal: bool,
    /// Spawner weapon.
    pub spawner: bool,
    /// Disc laser weapon.
    pub disk_laser: bool,
    /// Fire-only-visible flag.
    pub decloak_to_fire: bool,
    /// Lobber flag.
    pub lobber: bool,
    /// Never-use flag.
    pub never_use: bool,
    /// Draining weapon.
    pub drain: bool,
    /// Fire-while-moving flag.
    pub fire_while_moving: bool,
    /// Suicide flag.
    pub suicide: bool,
    /// Suppress flag.
    pub supress: bool,
    /// Fire-once flag.
    pub fire_once: bool,
    /// Omni-fire weapon.
    pub omni_fire: bool,
    /// Area-fire flag.
    pub area_fire: bool,
}

impl WeaponRules {
    /// An effect weapon of S-0301 claim 5: sonic, spark-particle,
    /// fire-particle or railgun.
    pub fn effect(&self) -> bool {
        self.sonic || self.spark || self.fire_particles || self.railgun
    }
}

/// A projectile type's resolved fields.
#[derive(Clone, Debug, Default)]
pub struct ProjectileRules {
    /// Name.
    pub name: String,
    /// Turn-rate field ρ.
    pub rot: i32,
    /// Anti-air.
    pub aa: bool,
    /// Anti-ground.
    pub ag: bool,
    /// Arcing flag.
    pub arcing: bool,
    /// Vertical flag.
    pub vertical: bool,
    /// Invisible flag.
    pub inviso: bool,
    /// Inaccurate flag.
    pub inaccurate: bool,
    /// Flak-scattering flag.
    pub flak_scatter: bool,
    /// Dropping flag.
    pub dropping: bool,
    /// Airburst flag.
    pub airburst: bool,
    /// Very-high flag.
    pub very_high: bool,
    /// Level flag.
    pub level: bool,
    /// Floater flag.
    pub floater: bool,
    /// Subject to cliffs.
    pub cliffs: bool,
    /// Subject to walls.
    pub walls: bool,
    /// Subject to elevation (S-0307 claim 6).
    pub elevation: bool,
    /// Ranged flag.
    pub ranged: bool,
    /// Degenerating flag.
    pub degenerates: bool,
    /// Scalable flag.
    pub scalable: bool,
    /// Acceleration α.
    pub acceleration: i32,
    /// Course lock duration κ.
    pub course_lock: i32,
    /// Arm time, ticks.
    pub arm: i32,
    /// Detonation altitude, leptons of world height.
    pub detonation_altitude: i32,
    /// Cluster count n_c.
    pub cluster: i32,
    /// The airburst weapon.
    pub airburst_weapon: Option<WeaponId>,
    /// Trailer animation name.
    pub trailer: Option<String>,
    /// Spawn delay field, ticks.
    pub spawn_delay: i32,
    /// Drawn as a voxel model.
    pub voxel: bool,
}

impl ProjectileRules {
    /// Steering (ρ > 0, S-0306 Terms).
    pub fn steering(&self) -> bool {
        self.rot > 0
    }

    /// The kind, first that fits (S-0306 Terms).
    pub fn kind(&self) -> ProjectileKind {
        if self.steering() {
            ProjectileKind::Steering
        } else if self.vertical {
            ProjectileKind::Vertical
        } else if self.arcing {
            ProjectileKind::Arcing
        } else {
            ProjectileKind::Straight
        }
    }
}

/// A projectile type's flight kind (S-0306 Terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectileKind {
    /// ρ > 0.
    Steering,
    /// The vertical flag.
    Vertical,
    /// The arcing flag.
    Arcing,
    /// Every other type.
    Straight,
}

/// A warhead's resolved fields.
#[derive(Clone, Debug, Default)]
pub struct WarheadRules {
    /// Name.
    pub name: String,
    /// Versus values by class index (doubles).
    pub verses: [F64; 11], // spec: S-0305 claim 1 (eleven classes)
    /// Spread S, cells (a single, widened).
    pub spread: F64,
    /// Edge fraction p (a single, widened).
    pub percent_at_max: F64,
    /// Prone factor P (a double).
    pub prone: F64,
    /// Infantry-death kind.
    pub inf_death: i32,
    /// Wall flag.
    pub wall: bool,
    /// Wood flag.
    pub wood: bool,
    /// Absolute wall-destroyer flag.
    pub wall_absolute: bool,
    /// Resource flag.
    pub tiberium: bool,
    /// Sonic warhead.
    pub sonic: bool,
    /// Electric-charge warhead.
    pub electric_assault: bool,
    /// Penetrates bunkers.
    pub penetrates_bunker: bool,
    /// Radiation warhead.
    pub radiation: bool,
    /// Psychic damage.
    pub psychic: bool,
    /// Poison.
    pub poison: bool,
    /// Affects allies.
    pub affects_allies: bool,
    /// Berserk warhead.
    pub psychedelic: bool,
    /// Sets building fires.
    pub sparky: bool,
    /// Causes delayed kills.
    pub causes_delay_kill: bool,
    /// Delay-kill frames F.
    pub delay_kill_frames: i32,
    /// Delay at the edge α (a single, widened).
    pub delay_kill_at_max: F64,
    /// Rocking flag.
    pub rocker: bool,
    /// Particle system name.
    pub particle: Option<String>,
    /// Conventional flag.
    pub conventional: bool,
    /// Electromagnetic flag.
    pub em: bool,
    /// Animation list.
    pub anim_list: Vec<String>,
    /// Maximum debris X.
    pub max_debris: i32,
    /// Minimum debris Y.
    pub min_debris: i32,
    /// Debris types.
    pub debris_types: Vec<String>,
    /// Debris maxima.
    pub debris_maximums: Vec<i32>,
    /// Airstrike warhead.
    pub airstrike: bool,
    /// Locomotor warhead.
    pub is_locomotor: bool,
    /// Registered under the nuclear warhead name (S-0306 claim 25).
    pub nuke: bool,
}

impl WarheadRules {
    /// The versus value for an armour class.
    pub fn versus(&self, class: usize) -> F64 {
        self.verses.get(class).copied().unwrap_or_default()
    }

    /// Infantry-only (S-0305 claim 6): the versus values for classes 4
    /// and 6 are both exactly 0.
    pub fn infantry_only(&self) -> bool {
        // spec: S-0305 claim 6 (classes 4 and 6 both exactly 0)
        self.versus(4).eq_num(F64::ZERO) && self.versus(6).eq_num(F64::ZERO)
    }
}

/// The eighteen veterancy abilities in the order of their import names
/// (S-0311 Terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Ability {
    Faster,
    Stronger,
    Firepower,
    Scatter,
    Rof,
    Sight,
    Cloak,
    TiberiumProof,
    VeinProof,
    SelfHeal,
    Explodes,
    RadarInvisible,
    Sensors,
    Fearless,
    C4,
    TiberiumHeal,
    GuardArea,
    Crusher,
}

impl Ability {
    /// The import names (S-0311 Import mapping), compared ignoring letter
    /// case and without trimming.
    // spec: S-0311 Import mapping (the eighteen ability names)
    pub const NAMES: [(&'static str, Ability); 18] = [
        ("faster", Ability::Faster),
        ("stronger", Ability::Stronger),
        ("firepower", Ability::Firepower),
        ("scatter", Ability::Scatter),
        ("rof", Ability::Rof),
        ("sight", Ability::Sight),
        ("cloak", Ability::Cloak),
        ("tiberium_proof", Ability::TiberiumProof),
        ("vein_proof", Ability::VeinProof),
        ("self_heal", Ability::SelfHeal),
        ("explodes", Ability::Explodes),
        ("radar_invisible", Ability::RadarInvisible),
        ("sensors", Ability::Sensors),
        ("fearless", Ability::Fearless),
        ("c4", Ability::C4),
        ("tiberium_heal", Ability::TiberiumHeal),
        ("guard_area", Ability::GuardArea),
        ("crusher", Ability::Crusher),
    ];

    fn bit(self) -> u32 {
        Self::NAMES
            .iter()
            .position(|(_, a)| *a == self)
            .map_or(0, |i| 1 << i)
    }
}

/// A set of veterancy abilities.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Abilities(u32);

impl Abilities {
    /// The abilities a list names (S-0311 Import mapping: items compared
    /// ignoring letter case, not trimmed; unknown items name nothing).
    pub fn from_list(items: &[String]) -> Self {
        let mut bits = 0;
        for it in items {
            if let Some((_, a)) = Ability::NAMES
                .iter()
                .find(|(n, _)| it.eq_ignore_ascii_case(n))
            {
                bits |= a.bit();
            }
        }
        Abilities(bits)
    }

    /// Whether the set holds an ability.
    pub fn has(self, a: Ability) -> bool {
        self.0 & a.bit() != 0
    }
}

/// The target-health scan mode (S-0309 Terms).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScanMode {
    /// *none*.
    #[default]
    None,
    /// *normal*.
    Normal,
    /// *strong*.
    Strong,
}

/// The combat fields of an infantry, vehicle, aircraft or building type.
#[derive(Clone, Debug, Default)]
pub struct TypeCombat {
    /// Armour class index (S-0305).
    pub armour: usize,
    /// Weapons in slots 0 … 17 (S-0308 claim 1).
    pub slots: [Option<WeaponId>; SLOTS],
    /// Elite slots.
    pub elite_slots: [Option<WeaponId>; SLOTS],
    /// Turret count above 0: a multi-weapon type (S-0308 Terms).
    pub multi_weapon: bool,
    /// Turret count N.
    pub turret_count: i32,
    /// Gattling type.
    pub gattling: bool,
    /// Charge-turret type.
    pub charge_turret: bool,
    /// Gunner type.
    pub gunner: bool,
    /// Mode index.
    pub ifv_mode: i32,
    /// Passenger weapon index (−1: choose normally).
    pub open_transport_weapon: i32,
    /// Deploy weapon index.
    pub deploy_fire_weapon: i32,
    /// Deploy-fire flag.
    pub deploy_fire: bool,
    /// Occupying weapon.
    pub occupy_weapon: Option<WeaponId>,
    /// Elite occupying weapon.
    pub elite_occupy_weapon: Option<WeaponId>,
    /// Land-targeting mode.
    pub land_targeting: i32,
    /// Naval-targeting mode.
    pub naval_targeting: i32,
    /// Underwater.
    pub underwater: bool,
    /// Organic.
    pub organic: bool,
    /// Unnatural.
    pub unnatural: bool,
    /// Naval.
    pub naval: bool,
    /// Maximum ammunition A.
    pub ammo: i32,
    /// Initial ammunition.
    pub initial_ammo: i32,
    /// Reload L.
    pub reload: i32,
    /// Empty reload E.
    pub empty_reload: i32,
    /// Reload increment I.
    pub reload_increment: i32,
    /// Pip wrap W.
    pub pip_wrap: i32,
    /// Manual reload.
    pub manual_reload: bool,
    /// Mobile-fire flag.
    pub mobile_fire: bool,
    /// Damage reduces readiness.
    pub readiness: bool,
    /// Readiness reduction factor κ (a single, widened).
    pub readiness_factor: F64,
    /// Air range bonus, leptons.
    pub air_range_bonus: i32,
    /// Radial fire segments.
    pub radial_segments: i32,
    /// Immune.
    pub immune: bool,
    /// Type-immune.
    pub type_immune: bool,
    /// Immune to radiation.
    pub immune_radiation: bool,
    /// Immune to psionic weapons.
    pub immune_psionic_weapons: bool,
    /// Immune to poison.
    pub immune_poison: bool,
    /// Immune to psionics.
    pub immune_psionics: bool,
    /// Self-damage flag.
    pub damage_self: bool,
    /// Eligible damage particle systems (smoke), in list order.
    pub smoke_systems: Vec<String>,
    /// Maximum debris X.
    pub max_debris: i32,
    /// Minimum debris Y.
    pub min_debris: i32,
    /// Voxel debris types.
    pub debris_types: Vec<String>,
    /// Their maxima.
    pub debris_maximums: Vec<i32>,
    /// Debris animations.
    pub debris_anims: Vec<String>,
    /// Explodes flag.
    pub explodes: bool,
    /// Own death weapon.
    pub death_weapon: Option<WeaponId>,
    /// Death-weapon damage factor φ (a single, widened).
    pub death_weapon_factor: F64,
    /// Explosion list.
    pub explosion: Vec<String>,
    /// Destruction-animation list.
    pub destroy_anim: Vec<String>,
    /// Trainable.
    pub trainable: bool,
    /// Unscored.
    pub dont_score: bool,
    /// Self-healing type.
    pub self_healing: bool,
    /// Passive-acquire flag.
    pub passive_acquire: bool,
    /// Retaliate flag.
    pub retaliate: bool,
    /// Detects disguises.
    pub detect_disguise: bool,
    /// No-auto-fire flag.
    pub no_auto_fire: bool,
    /// Target-health scan mode.
    pub scan_mode: ScanMode,
    /// Special threat value s_T (a double).
    pub special_threat: F64,
    /// Threat coefficients c_me, c_te, c_st, c_sg, c_d (doubles).
    pub coefficients: [F64; 5], // spec: S-0309 F4 (five coefficients)
    /// Insignificant.
    pub insignificant: bool,
    /// Threat posed.
    pub threat_posed: i32,
    /// Airstrike team.
    pub airstrike_team: i32,
    /// Drainable.
    pub drainable: bool,
    /// Muzzle positions by slot, ordinary and elite (S-0313).
    pub flh: [Flh; SLOTS],
    /// Elite muzzle positions.
    pub elite_flh: [Flh; SLOTS],
    /// Turret displacement δ.
    pub turret_offset: i32,
    /// Veteran abilities.
    pub veteran: Abilities,
    /// Elite abilities.
    pub elite: Abilities,
}

/// The vehicle-only combat fields.
#[derive(Clone, Debug, Default)]
pub struct VehicleCombat {
    /// Burst delays 1 … 4 (−1: none).
    pub burst_delay: [i32; 4], // spec: S-0301 Constants (four burst-delay fields)
    /// Firing frames F.
    pub firing_frames: i32,
    /// Firing sync frames for k mod B = 0 and 1 (−1: none).
    pub sync_frames: [i32; 2], // spec: S-0301 Constants (two firing sync values)
    /// Seeker type.
    pub hunter_seeker: bool,
    /// Deploy-to-fire flag.
    pub deploy_to_fire: bool,
    /// Jumpjet-turn flag.
    pub jump_jet_turn: bool,
    /// Crewed flag.
    pub crewed: bool,
    /// Carries a crate.
    pub carries_crate: bool,
    /// Can crash.
    pub crashable: bool,
    /// Weight (a double).
    pub weight: Option<F64>,
    /// Suppression threshold.
    pub suppression_threshold: i32,
    /// Missile spawn.
    pub missile_spawn: bool,
}

/// The building-only combat fields.
#[derive(Clone, Debug, Default)]
pub struct BuildingCombat {
    /// Defence building (build category *combat*).
    pub defense: bool,
    /// Can be demolished (the charges flag; S-0302 claim 7).
    pub can_c4: bool,
    /// Eligible for delayed kills.
    pub delay_kill: bool,
    /// Leaves rubble.
    pub leave_rubble: bool,
    /// Helipad.
    pub helipad: bool,
    /// Artillery flag.
    pub artillary: bool,
    /// Can be occupied.
    pub can_be_occupied: bool,
    /// Occupants may fire.
    pub can_occupy_fire: bool,
    /// Maximum occupants.
    pub max_occupants: i32,
    /// Infantry self-heal count.
    pub infantry_self_heal: i32,
    /// Vehicle self-heal count.
    pub units_self_heal: i32,
    /// Aim shift, leptons.
    pub aim_shift: [i32; 3], // nocite: a lepton triple (x, y, z), our shape
    /// Art height H_b.
    pub art_height: i32,
    /// Voxel turret.
    pub voxel_turret: bool,
    /// Voxel barrel.
    pub voxel_barrel: bool,
    /// Primary fire pixel shift.
    pub fire_pixel: Option<[i32; 2]>, // nocite: a pixel pair, our shape
    /// Dual-shift flag.
    pub dual_offset: bool,
    /// Turret anchor pixel shift.
    pub turret_anim: [i32; 2], // nocite: a pixel pair, our shape
    /// Animation-delayed fire.
    pub anim_delayed_fire: bool,
    /// Delayed-fire delay d.
    pub delayed_fire_delay: i32,
    /// Laser-fence section.
    pub laser_fence: bool,
    /// Bridge repair hut.
    pub bridge_repair_hut: bool,
    /// Weapons factory.
    pub weapons_factory: bool,
    /// SAM flag.
    pub sam: bool,
    /// Electromagnetic-pulse cannon.
    pub emp_cannon: bool,
    /// Stupid guard mode.
    pub stupid_guard: bool,
    /// Unit repair building.
    pub unit_repair: bool,
    /// Capturable.
    pub capturable: bool,
}

/// A country's combat fields.
#[derive(Clone, Debug)]
pub struct CountryCombat {
    /// φ_c (double).
    pub firepower: F64,
    /// κ (double).
    pub rof: F64,
    /// House armour factors by kind: infantry, vehicles, aircraft, other
    /// buildings, defence buildings (singles, widened).
    pub armour: [F64; 5], // spec: S-0302 F2 (five per-kind armour factors)
    /// Veteran lists: infantry, vehicle and aircraft registration indices.
    pub veteran: [Vec<u32>; 3], // spec: S-0311 claim 11 (three country veteran lists)
}

/// Game-wide combat values.
#[derive(Clone, Debug)]
pub struct CombatGeneral {
    /// Maximum damage.
    pub max_damage: i32,
    /// Occupy and bunker rate-of-fire factors m, b (singles, widened).
    pub occupy_rof: F64,
    /// Bunker rate-of-fire factor.
    pub bunker_rof: F64,
    /// Garrison, bunker, open-topped damage multipliers (singles).
    pub damage_mult: [F64; 3], // spec: S-0302 F1 (M_g, M_b, M_t)
    /// Demolition warhead.
    pub c4_warhead: Option<WarheadId>,
    /// Crushing warhead.
    pub crush_warhead: Option<WarheadId>,
    /// Ion-cannon warhead.
    pub ion_cannon_warhead: Option<WarheadId>,
    /// Bridge strength (no absent-key default stated: request Q-0060).
    pub bridge_strength: Option<i32>,
    /// Ammunition-crate damage (no absent-key default stated: request Q-0060).
    pub ammo_crate_damage: Option<i32>,
    /// Splash list.
    pub splash_list: Vec<String>,
    /// Ballistic scatter B, leptons.
    pub ballistic_scatter: i32,
    /// Occupied-weapon range G, cells.
    pub occupy_weapon_range: i32,
    /// Bunker range bonus, cells.
    pub bunker_range_bonus: i32,
    /// Open-topped range bonus, cells.
    pub open_topped_range_bonus: i32,
    /// Fire-suppression distance L, leptons.
    pub fire_supress: i32,
    /// Game-wide death weapon.
    pub death_weapon: Option<WeaponId>,
    /// Veteran rate-of-fire factor v.
    pub veteran_rof: F64,
    /// Veteran combat factor V_c.
    pub veteran_combat: F64,
    /// Veteran armour factor V_a.
    pub veteran_armor: F64,
    /// Veteran sight factor.
    pub veteran_sight: F64,
    /// Veteran ratio ρ.
    pub veteran_ratio: F64,
    /// Veteran cap c.
    pub veteran_cap: F64,
    /// Damage-fire animations.
    pub damage_fire_types: Vec<String>,
    /// Infantry-explosion animation.
    pub infantry_explode: Option<String>,
    /// Missile turn variation m.
    pub missile_rot_var: F64,
    /// Missile safety altitude.
    pub missile_safety_altitude: i32,
    /// Harvester types (vehicle registration indices).
    pub harvester_units: Vec<u32>,
    /// Barrel explosion animation.
    pub barrel_explode: Option<String>,
    /// Barrel debris list.
    pub barrel_debris: Vec<String>,
    /// Barrel particle system.
    pub barrel_particle: Option<String>,
    /// Metallic debris list.
    pub metallic_debris: Vec<String>,
    /// Nullify animation.
    pub nullify_anim: Option<String>,
    /// Lightning warhead.
    pub lightning_warhead: Option<WarheadId>,
    /// Lightning-bolt explosion.
    pub bolt_explosion: Option<String>,
    /// Crew escape chance (no absent-key default stated: request Q-0060).
    pub crew_escape: Option<F64>,
    /// Crew types by side: allied, soviet, third (infantry indices).
    pub side_crew: [Option<u32>; 3], // spec: S-0312 Terms (three side crew types)
    /// Technician type.
    pub technician: Option<u32>,
    /// Engineer type.
    pub engineer: Option<u32>,
    /// Survivor divisors by side (no absent-key default stated: request Q-0060).
    pub survivor_divisor: [Option<i32>; 3], // spec: S-0312 F5 (three survivor divisors)
    /// Sinking weight (no absent-key default stated: request Q-0060).
    pub sinking_weight: Option<F64>,
    /// Building repair period P_b, ticks.
    pub repair_period: i32,
    /// Vehicle repair count K_u.
    pub repair_count_vehicle: i32,
    /// Infantry repair count K_i.
    pub repair_count_infantry: i32,
    /// Repair step.
    pub repair_step: i32,
    /// Infantry repair step.
    pub i_repair_step: i32,
    /// Repair percentage p.
    pub repair_percent: F64,
    /// Self-heal intervals: infantry, vehicles.
    pub self_heal_frames: [i32; 2], // spec: S-0314 claim 10 (two intervals)
    /// Self-heal amounts: infantry, vehicles.
    pub self_heal_amount: [i32; 2], // spec: S-0314 claim 10 (two amounts)
    /// Threat per occupant.
    pub threat_per_occupant: i32,
    /// Enemy-house bonus B.
    pub enemy_house_bonus: F64,
    /// Disguise-detection list.
    pub disguise_detection: Vec<i32>,
    /// Gravity g.
    pub gravity: i32,
    /// Allied wall transparency.
    pub allied_wall_transparency: bool,
    /// Elevation increment I.
    pub elevation_increment: i32,
    /// Elevation increment bonus b.
    pub elevation_bonus: F64,
    /// Elevation bonus cap c.
    pub elevation_cap: F64,
    /// The repair-and-sell threshold.
    pub iq_repair_sell: i32,
    /// The normal row's rate-of-fire factor ℓ.
    pub row_rof: F64,
    /// The normal row's firepower factor φᵣ.
    pub row_firepower: F64,
    /// The normal row's destroy-walls flag.
    pub row_destroy_walls: bool,
}

/// The combat rules of one game.
#[derive(Clone, Debug)]
pub struct CombatRules {
    /// Weapons in name order.
    pub weapons: Vec<WeaponRules>,
    /// Projectile types in name order.
    pub projectiles: Vec<ProjectileRules>,
    /// Warheads in name order.
    pub warheads: Vec<WarheadRules>,
    /// Weapon index by name.
    pub weapon_index: BTreeMap<String, WeaponId>,
    /// Game-wide values.
    pub general: CombatGeneral,
    /// Countries by registration index.
    pub countries: Vec<CountryCombat>,
}

impl CombatRules {
    /// A weapon.
    pub fn weapon(&self, w: WeaponId) -> &WeaponRules {
        &self.weapons[w as usize]
    }

    /// A weapon's projectile type.
    pub fn projectile_of(&self, w: WeaponId) -> Option<&ProjectileRules> {
        self.weapon(w)
            .projectile
            .map(|p| &self.projectiles[p as usize])
    }

    /// A weapon's warhead.
    pub fn warhead_of(&self, w: WeaponId) -> Option<&WarheadRules> {
        self.weapon(w).warhead.map(|h| &self.warheads[h as usize])
    }

    /// A warhead.
    pub fn warhead(&self, h: WarheadId) -> &WarheadRules {
        &self.warheads[h as usize]
    }
}

/// Builds the weapon, projectile and warhead tables. Weapons, projectiles
/// and warheads are indexed in name order (our own order: no spec ties a
/// behaviour to it).
pub fn tables(
    weapons: &BTreeMap<String, WeaponType>,
    projectiles: &BTreeMap<String, ProjectileType>,
    warheads: &BTreeMap<String, WarheadType>,
    gravity: i32,
    mode: Rounding,
) -> (
    Vec<WeaponRules>,
    Vec<ProjectileRules>,
    Vec<WarheadRules>,
    BTreeMap<String, WeaponId>,
) {
    let p_index: BTreeMap<&String, u32> = projectiles
        .keys()
        .enumerate()
        .map(|(i, k)| (k, i as u32))
        .collect();
    let h_index: BTreeMap<&String, u32> = warheads
        .keys()
        .enumerate()
        .map(|(i, k)| (k, i as u32))
        .collect();
    let w_index: BTreeMap<String, u32> = weapons
        .keys()
        .enumerate()
        .map(|(i, k)| (k.clone(), i as u32))
        .collect();
    let b = |x: Option<bool>| x.unwrap_or(false);
    let one = one(mode);
    let pr: Vec<ProjectileRules> = projectiles
        .iter()
        .map(|(n, p)| ProjectileRules {
            name: n.clone(),
            // spec: S-0306 Constants (projectile defaults: turn rate 0, anti-ground yes, anti-air no)
            rot: p.rot.unwrap_or(0),
            aa: b(p.aa),
            ag: p.ag.unwrap_or(true),
            arcing: b(p.arcing),
            vertical: b(p.vertical),
            inviso: b(p.inviso),
            inaccurate: b(p.inaccurate),
            flak_scatter: b(p.flak_scatter),
            dropping: b(p.dropping),
            airburst: b(p.airburst),
            very_high: b(p.very_high),
            level: b(p.level),
            floater: b(p.floater),
            cliffs: b(p.subject_to_cliffs),
            walls: b(p.subject_to_walls),
            elevation: b(p.subject_to_elevation),
            ranged: b(p.ranged),
            degenerates: b(p.degenerates),
            scalable: b(p.scalable),
            // spec: S-0306 Constants (acceleration 3, spawn delay 3, cluster 1, arm 0, detonation altitude 0, course lock 0)
            acceleration: p.acceleration.unwrap_or(3),
            course_lock: p.course_lock_duration.unwrap_or(0),
            arm: p.arm.unwrap_or(0),
            detonation_altitude: p.detonation_altitude.unwrap_or(0),
            cluster: p.cluster.unwrap_or(1),
            airburst_weapon: p
                .airburst_weapon
                .as_ref()
                .and_then(|w| w_index.get(w).copied()),
            trailer: p.trailer.clone(),
            spawn_delay: p.spawn_delay.unwrap_or(3), // spec: S-0306 Constants (spawn delay 3)
            voxel: b(p.voxel),
        })
        .collect();
    let hr: Vec<WarheadRules> = warheads
        .iter()
        .map(|(n, h)| {
            // spec: S-0003 claim 13 and S-0302 Constants (versus value absent: 1.0 each)
            let mut verses = [one; 11];
            if let Some(v) = &h.verses {
                // spec: S-0305 claim 1 (eleven versus items)
                for (i, d) in v.iter().take(11).enumerate() {
                    verses[i] = d.value(mode);
                }
            }
            WarheadRules {
                name: n.clone(),
                verses,
                // spec: S-0303 Import mapping (CellSpread absent 0, PercentAtMax absent 1.0, both singles)
                spread: single(&h.cell_spread, mode).unwrap_or(F64::ZERO),
                percent_at_max: single(&h.percent_at_max, mode).unwrap_or(one),
                // spec: S-0302 Constants (prone factor absent: 1.0, a double)
                prone: dec(&h.prone_damage, mode).unwrap_or(one),
                // spec: S-0302 Import mapping (InfDeath absent: 0)
                inf_death: h.inf_death.unwrap_or(0),
                wall: b(h.wall),
                wood: b(h.wood),
                wall_absolute: b(h.wall_absolute_destroyer),
                tiberium: b(h.tiberium),
                sonic: b(h.sonic),
                electric_assault: b(h.electric_assault),
                penetrates_bunker: b(h.penetrates_bunker),
                radiation: b(h.radiation),
                psychic: b(h.psychic_damage),
                poison: b(h.poison),
                // spec: S-0302 Import mapping (AffectsAllies absent: yes)
                affects_allies: h.affects_allies.unwrap_or(true),
                psychedelic: b(h.psychedelic),
                sparky: b(h.sparky),
                causes_delay_kill: b(h.causes_delay_kill),
                // spec: S-0302 Import mapping (DelayKillFrames absent 5 ticks, DelayKillAtMax absent 1.0, a single)
                delay_kill_frames: h.delay_kill_frames.unwrap_or(5),
                delay_kill_at_max: single(&h.delay_kill_at_max, mode).unwrap_or(one),
                rocker: b(h.rocker),
                particle: h.particle.clone(),
                conventional: b(h.conventional),
                em: b(h.em_effect),
                anim_list: h.anim_list.clone().unwrap_or_default(),
                // spec: S-0303 Import mapping (MaxDebris, MinDebris absent: 0, 0)
                max_debris: h.max_debris.unwrap_or(0),
                min_debris: h.min_debris.unwrap_or(0),
                debris_types: h.debris_types.clone().unwrap_or_default(),
                debris_maximums: h.debris_maximums.clone().unwrap_or_default(),
                airstrike: b(h.airstrike),
                is_locomotor: b(h.is_locomotor),
                // spec: S-0306 claim 25 and Import mapping (the nuclear warhead name, compared without regard to case)
                nuke: n.eq_ignore_ascii_case("NUKE"),
            }
        })
        .collect();
    let wr: Vec<WeaponRules> = weapons
        .iter()
        .map(|(n, w)| {
            let projectile = w.projectile.as_ref().and_then(|p| p_index.get(p).copied());
            let range = w.range.as_ref().map_or(0, |d| leptons(d, mode));
            let rho0 = projectile.is_some_and(|p| pr[p as usize].rot == 0);
            let floater = projectile.is_some_and(|p| pr[p as usize].floater);
            // spec: S-0306 Import mapping (stored speed = min(trunc(256 × min(v, 100) / 100), 255); v ≤ 0 gives 0)
            let imported = crate::rules::type_speed(w.speed.unwrap_or(0));
            // spec: S-0306 claim 1 (for ρ = 0 the weapon speed is v(R₀, g), g_f for a floater)
            let speed = if rho0 {
                crate::combat::ballistics::arc_speed(range, gravity, floater, mode)
            } else {
                imported
            };
            WeaponRules {
                name: n.clone(),
                // spec: S-0302 Import mapping (Damage absent: 0); S-0308 Import mapping (AmbientDamage absent: 0)
                damage: w.damage.unwrap_or(0),
                ambient: w.ambient_damage.unwrap_or(0),
                projectile,
                warhead: w.warhead.as_ref().and_then(|h| h_index.get(h).copied()),
                // spec: S-0301 Constants (rate-of-fire field absent: 0 ticks)
                rof: w.rof.unwrap_or(0),
                // spec: S-0301 Edge cases (a burst count below 1 is treated as 1, a departure; absent: 1)
                burst: w.burst.unwrap_or(1).max(1),
                speed,
                range,
                min_range: w.minimum_range.as_ref().map_or(0, |d| leptons(d, mode)),
                cell_rangefinding: b(w.cell_rangefinding),
                sonic: b(w.is_sonic),
                spark: b(w.use_spark_particles),
                fire_particles: b(w.use_fire_particles),
                railgun: b(w.is_railgun),
                mag_beam: b(w.is_mag_beam),
                temporal: b(w.temporal),
                spawner: b(w.spawner),
                disk_laser: b(w.disk_laser),
                // spec: S-0301 Import mapping (DecloakToFire absent: yes)
                decloak_to_fire: w.decloak_to_fire.unwrap_or(true),
                lobber: b(w.lobber),
                never_use: b(w.never_use),
                drain: b(w.drain_weapon),
                // spec: S-0310 Import mapping (FireWhileMoving absent: yes)
                fire_while_moving: w.fire_while_moving.unwrap_or(true),
                suicide: b(w.suicide),
                supress: b(w.supress),
                fire_once: b(w.fire_once),
                omni_fire: b(w.omni_fire),
                area_fire: b(w.area_fire),
            }
        })
        .collect();
    (wr, pr, hr, w_index)
}

/// Resolves a type's combat fields. `primary`/`secondary` and their elite
/// forms give slots 0 and 1 when the data gives no slot list (S-0308 claim
/// 1, a type whose turret count is 0 or less).
#[allow(clippy::too_many_arguments)]
pub fn type_combat(
    c: &CombatFields,
    primary: [Option<&String>; 4], // nocite: four names (primary, secondary and their elite forms), our shape
    turret_count: i32,
    infantry: bool,
    abilities: (&[String], &[String]),
    general_coefficients: [F64; 5], // spec: S-0309 F4 (five coefficients)
    weapons: &BTreeMap<String, WeaponId>,
    mode: Rounding,
) -> TypeCombat {
    let b = |x: Option<bool>| x.unwrap_or(false);
    let one = one(mode);
    let w = |n: Option<&String>| n.and_then(|n| weapons.get(n).copied());
    let mut slots = [None; SLOTS];
    let mut elite_slots = [None; SLOTS];
    match &c.weapon_slots {
        Some(v) => {
            for (i, n) in v.iter().take(SLOTS).enumerate() {
                slots[i] = (!n.is_empty()).then(|| w(Some(n))).flatten();
            }
        }
        None => {
            slots[0] = w(primary[0]);
            slots[1] = w(primary[1]);
        }
    }
    match &c.elite_weapon_slots {
        Some(v) => {
            for (i, n) in v.iter().take(SLOTS).enumerate() {
                elite_slots[i] = (!n.is_empty()).then(|| w(Some(n))).flatten();
            }
        }
        None => {
            elite_slots[0] = w(primary[2]); // nocite: index into the four names above
            elite_slots[1] = w(primary[3]); // nocite: index into the four names above
        }
    }
    let mut flh = [[0; 3]; SLOTS]; // spec: S-0313 Import mapping (muzzle positions absent: 0, 0, 0)
    let mut elite_flh = [[0; 3]; SLOTS]; // spec: S-0313 Import mapping (muzzle positions absent: 0, 0, 0)
    // spec: S-0313 Import mapping (muzzle positions absent: 0, 0, 0)
    if let Some(v) = &c.weapon_flh {
        for (i, f) in v.iter().take(SLOTS).enumerate() {
            flh[i] = *f;
        }
    }
    if let Some(v) = &c.elite_weapon_flh {
        for (i, f) in v.iter().take(SLOTS).enumerate() {
            elite_flh[i] = *f;
        }
    }
    if c.weapon_flh.is_none() {
        flh[0] = c.primary_fire_flh.unwrap_or([0; 3]); // spec: S-0313 Import mapping (muzzle positions absent: 0, 0, 0)
        flh[1] = c.secondary_fire_flh.unwrap_or([0; 3]); // spec: S-0313 Import mapping (muzzle positions absent: 0, 0, 0)
    }
    if c.elite_weapon_flh.is_none() {
        elite_flh[0] = c.elite_primary_fire_flh.unwrap_or(flh[0]);
        elite_flh[1] = c.elite_secondary_fire_flh.unwrap_or(flh[1]);
    }
    let coefficients = match &c.coefficients {
        Some(v) => {
            let mut out = general_coefficients;
            for (i, d) in v.iter().enumerate() {
                out[i] = d.value(mode);
            }
            out
        }
        // spec: S-0309 Import mapping (a coefficient no layer gives: the game-wide default)
        None => general_coefficients,
    };
    TypeCombat {
        armour: c.armor.unwrap_or(ArmourClass::None).index(),
        slots,
        elite_slots,
        multi_weapon: turret_count > 0,
        turret_count,
        gattling: b(c.is_gattling),
        charge_turret: b(c.is_charge_turret),
        gunner: false,
        ifv_mode: 0,
        // spec: S-0308 Constants (passenger weapon index absent: −1)
        open_transport_weapon: -1,
        // spec: S-0308 Constants (deploy weapon index absent: 1)
        deploy_fire_weapon: 1,
        deploy_fire: false,
        occupy_weapon: None,
        elite_occupy_weapon: None,
        // spec: S-0308 Constants (land- and naval-targeting modes absent: 0)
        land_targeting: c.land_targeting.unwrap_or(0),
        naval_targeting: c.naval_targeting.unwrap_or(0),
        underwater: b(c.underwater),
        // spec: S-0314 Import mapping (Organic absent: yes for infantry types, no otherwise)
        organic: c.organic.unwrap_or(infantry),
        unnatural: b(c.unnatural),
        naval: b(c.naval),
        // spec: S-0301 Constants (maximum and initial ammunition absent: −1, unlimited)
        ammo: c.ammo.unwrap_or(-1),
        initial_ammo: c.initial_ammo.unwrap_or(-1),
        // spec: S-0301 Constants (reload 0, empty reload −1, reload increment 0, pip wrap 0)
        reload: c.reload.unwrap_or(0),
        empty_reload: c.empty_reload.unwrap_or(-1),
        reload_increment: c.reload_increment.unwrap_or(0),
        pip_wrap: c.pip_wrap.unwrap_or(0),
        manual_reload: b(c.manual_reload),
        mobile_fire: true,
        readiness: b(c.damage_reduces_readiness),
        // spec: S-0302 Import mapping (ReadinessReductionMultiplier absent: 0.0, a single)
        readiness_factor: single(&c.readiness_reduction_multiplier, mode).unwrap_or(F64::ZERO),
        air_range_bonus: c.air_range_bonus.as_ref().map_or(0, |d| leptons(d, mode)),
        // spec: S-0306 Import mapping (RadialFireSegments absent: 0)
        radial_segments: c.radial_fire_segments.unwrap_or(0),
        immune: false,
        type_immune: b(c.type_immune),
        immune_radiation: false,
        immune_psionic_weapons: b(c.immune_to_psionic_weapons),
        immune_poison: b(c.immune_to_poison),
        immune_psionics: b(c.immune_to_psionics),
        damage_self: b(c.damage_self),
        smoke_systems: c.damage_smoke_systems.clone().unwrap_or_default(),
        // spec: S-0312 Import mapping (MaxDebris, MinDebris absent: 0, 0)
        max_debris: c.max_debris.unwrap_or(0),
        min_debris: c.min_debris.unwrap_or(0),
        debris_types: c.debris_types.clone().unwrap_or_default(),
        debris_maximums: c.debris_maximums.clone().unwrap_or_default(),
        debris_anims: c.debris_anims.clone().unwrap_or_default(),
        explodes: b(c.explodes),
        death_weapon: w(c.death_weapon.as_ref()),
        // spec: S-0312 Import mapping (DeathWeaponDamageModifier absent: 1.0, a single)
        death_weapon_factor: single(&c.death_weapon_damage_modifier, mode).unwrap_or(one),
        explosion: c.explosion.clone().unwrap_or_default(),
        destroy_anim: c.destroy_anim.clone().unwrap_or_default(),
        // spec: S-0311 Import mapping (Trainable absent: yes)
        trainable: c.trainable.unwrap_or(true),
        dont_score: b(c.dont_score),
        self_healing: b(c.self_healing),
        // spec: S-0309 Constants (passive-acquire and retaliate flags absent: yes)
        passive_acquire: c.can_passive_aquire.unwrap_or(true),
        retaliate: c.can_retaliate.unwrap_or(true),
        detect_disguise: b(c.detect_disguise),
        no_auto_fire: b(c.no_auto_fire),
        // spec: S-0309 Import mapping (VHPScan: None, Normal, Strong; absent none)
        scan_mode: match c
            .vhp_scan
            .as_deref()
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("normal") => ScanMode::Normal,
            Some("strong") => ScanMode::Strong,
            _ => ScanMode::None,
        },
        // spec: S-0309 Import mapping (SpecialThreatValue absent: 0)
        special_threat: dec(&c.special_threat_value, mode).unwrap_or(F64::ZERO),
        coefficients,
        insignificant: b(c.insignificant),
        // spec: S-0309 Import mapping (ThreatPosed absent: 0)
        threat_posed: c.threat_posed.unwrap_or(0),
        airstrike_team: c.airstrike_team.unwrap_or(0),
        drainable: b(c.drainable),
        flh,
        elite_flh,
        // spec: S-0313 Import mapping (TurretOffset absent: 0)
        turret_offset: c.turret_offset.unwrap_or(0),
        veteran: Abilities::from_list(abilities.0),
        elite: Abilities::from_list(abilities.1),
    }
}

/// Resolves the vehicle-only fields.
pub fn vehicle_combat(u: &rs::UnitType, mode: Rounding) -> VehicleCombat {
    let b = |x: Option<bool>| x.unwrap_or(false);
    VehicleCombat {
        // spec: S-0301 Import mapping (BurstDelay absent: −1)
        burst_delay: u.burst_delay.unwrap_or([-1; 4]),
        // spec: S-0301 Import mapping (FiringFrames absent: 0, FiringSyncFrame absent: −1)
        firing_frames: u.firing_frames.unwrap_or(0),
        sync_frames: u.firing_sync_frames.unwrap_or([-1, -1]),
        hunter_seeker: b(u.hunter_seeker),
        deploy_to_fire: b(u.deploy_to_fire),
        jump_jet_turn: b(u.jump_jet_turn),
        crewed: b(u.crewed),
        carries_crate: b(u.carries_crate),
        crashable: b(u.crashable),
        weight: dec(&u.weight, mode),
        // spec: S-0315 Constants (suppression threshold absent: 0)
        suppression_threshold: u.suppression_threshold.unwrap_or(0),
        missile_spawn: b(u.missile_spawn),
    }
}

/// Resolves a country's combat fields.
pub fn country_combat(
    c: &rs::Country,
    names: &BTreeMap<(crate::rules::Kind, String), u32>,
    mode: Rounding,
) -> CountryCombat {
    use crate::rules::Kind;
    let one = one(mode);
    let list = |kind: Kind, v: &Option<Vec<String>>| -> Vec<u32> {
        v.iter()
            .flatten()
            .filter_map(|n| names.get(&(kind, n.clone())).copied())
            .collect()
    };
    CountryCombat {
        // spec: S-0302 Import mapping (country Firepower absent: 1.0, a double); S-0301 Import mapping (ROF absent: 1.0)
        firepower: dec(&c.firepower, mode).unwrap_or(one),
        rof: dec(&c.rof, mode).unwrap_or(one),
        // spec: S-0302 Import mapping (per-kind armour factors absent: 1.0, singles)
        armour: [
            single(&c.armor_infantry_mult, mode).unwrap_or(one),
            single(&c.armor_units_mult, mode).unwrap_or(one),
            single(&c.armor_aircraft_mult, mode).unwrap_or(one),
            single(&c.armor_buildings_mult, mode).unwrap_or(one),
            single(&c.armor_defenses_mult, mode).unwrap_or(one),
        ],
        veteran: [
            list(Kind::Infantry, &c.veteran_infantry),
            list(Kind::Vehicle, &c.veteran_units),
            list(Kind::Aircraft, &c.veteran_aircraft),
        ],
    }
}

/// The normal difficulty row (glossary *difficulty skill block*: every
/// house uses it in our engine).
pub fn normal_row(rows: &BTreeMap<String, DifficultyRow>) -> DifficultyRow {
    rows.get("normal").cloned().unwrap_or_default()
}

/// F1 of S-0314: the building repair period and a repair count.
pub fn repair_period(r: F64, mode: Rounding) -> i32 {
    // spec: S-0314 F1 (P_b = trunc(RZ₅₃(r_b × 900)))
    r.mul(F64::from_i64(900, mode), mode).to_i32_common()
}

/// The repair count K of S-0314 F1: the smallest positive integer at least
/// RZ₅₃(r × 900).
pub fn repair_count(r: F64, mode: Rounding) -> i32 {
    // spec: S-0314 F1 (K = the smallest positive integer with RZ₅₃(r × 900) ≤ K)
    let p = r.mul(F64::from_i64(900, mode), mode);
    let t = p.to_i32_common();
    let k = if F64::from_i64(i64::from(t), mode).lt(p) {
        t + 1
    } else {
        t
    };
    k.max(1)
}

/// The four tables of [`tables`].
pub type Tables = (
    Vec<WeaponRules>,
    Vec<ProjectileRules>,
    Vec<WarheadRules>,
    BTreeMap<String, WeaponId>,
);

/// Builds the combat rules from the typed data.
pub fn combat_rules(
    s: &rs::SimData,
    tables: Tables,
    gravity: i32,
    names: &BTreeMap<(crate::rules::Kind, String), u32>,
    country_names: &[String],
    mode: Rounding,
) -> CombatRules {
    use crate::rules::{Kind, nearest_double};
    let (weapons, projectiles, warheads, weapon_index) = tables;
    let g = &s.general;
    let one = one(mode);
    let warhead = |n: &Option<String>| -> Option<WarheadId> {
        n.as_ref()
            .and_then(|n| warheads.iter().position(|h| &h.name == n))
            .map(|i| i as u32)
    };
    let infantry = |n: &Option<String>| -> Option<u32> {
        n.as_ref()
            .and_then(|n| names.get(&(Kind::Infantry, n.clone())).copied())
    };
    let row = normal_row(&s.difficulty);
    // spec: S-0314 Constants (repair rates absent: the double nearest 0.016)
    let rate = |d: &Option<Decimal>| dec(d, mode).unwrap_or_else(|| nearest_double("0.016"));
    let general = CombatGeneral {
        // spec: S-0302 Constants (maximum damage absent: 1000)
        max_damage: g.max_damage.unwrap_or(1000),
        // spec: S-0301 Constants (occupy and bunker factors absent: 1.0, singles)
        occupy_rof: single(&g.occupy_rof_multiplier, mode).unwrap_or(one),
        bunker_rof: single(&g.bunker_rof_multiplier, mode).unwrap_or(one),
        // spec: S-0302 Import mapping (the three damage multipliers absent: 1.0 each, singles)
        damage_mult: [
            single(&g.occupy_damage_multiplier, mode).unwrap_or(one),
            single(&g.bunker_damage_multiplier, mode).unwrap_or(one),
            single(&g.open_topped_damage_multiplier, mode).unwrap_or(one),
        ],
        c4_warhead: warhead(&g.c4_warhead),
        crush_warhead: warhead(&g.crush_warhead),
        ion_cannon_warhead: warhead(&g.ion_cannon_warhead),
        bridge_strength: g.bridge_strength,
        ammo_crate_damage: g.ammo_crate_damage,
        splash_list: g.splash_list.clone().unwrap_or_default(),
        // spec: S-0306 Constants (ballistic scatter absent: 256 leptons)
        ballistic_scatter: g
            .ballistic_scatter
            .as_ref()
            .map_or(256, |d| leptons(d, mode)),
        // spec: S-0307 Constants (occupied-weapon range 5, bunker bonus 2, open-topped bonus 2 cells)
        occupy_weapon_range: g.occupy_weapon_range.unwrap_or(5),
        bunker_range_bonus: g.bunker_weapon_range_bonus.unwrap_or(2), // spec: S-0307 Constants (bunker range bonus absent: 2 cells)
        open_topped_range_bonus: g.open_topped_range_bonus.unwrap_or(2), // spec: S-0307 Constants (open-topped range bonus absent: 2 cells)
        // spec: S-0309 Constants (fire-suppression distance absent: 256 leptons)
        fire_supress: g.fire_supress.as_ref().map_or(256, |d| leptons(d, mode)),
        death_weapon: g
            .death_weapon
            .as_ref()
            .and_then(|n| weapon_index.get(n).copied()),
        // spec: S-0311 Constants (veteran factors absent: 1.0; ratio 10.0; cap 1.0)
        veteran_rof: dec(&g.veteran_rof, mode).unwrap_or(one),
        veteran_combat: dec(&g.veteran_combat, mode).unwrap_or(one),
        veteran_armor: dec(&g.veteran_armor, mode).unwrap_or(one),
        veteran_sight: dec(&g.veteran_sight, mode).unwrap_or(one),
        veteran_ratio: dec(&g.veteran_ratio, mode).unwrap_or_else(|| F64::from_i64(10, mode)), // spec: S-0311 Constants (veteran ratio absent: 10.0)
        veteran_cap: dec(&g.veteran_cap, mode).unwrap_or(one),
        damage_fire_types: g.damage_fire_types.clone().unwrap_or_default(),
        infantry_explode: g.infantry_explode.clone(),
        // spec: S-0306 Constants (missile turn variation absent: 0.25; safety altitude 500 leptons)
        missile_rot_var: dec(&g.missile_rot_var, mode).unwrap_or_else(|| nearest_double("0.25")),
        missile_safety_altitude: g.missile_safety_altitude.unwrap_or(500), // spec: S-0306 Constants (missile safety altitude absent: 500 leptons)
        harvester_units: g
            .harvester_unit
            .iter()
            .flatten()
            .filter_map(|n| names.get(&(Kind::Vehicle, n.clone())).copied())
            .collect(),
        barrel_explode: g.barrel_explode.clone(),
        barrel_debris: g.barrel_debris.clone().unwrap_or_default(),
        barrel_particle: g.barrel_particle.clone(),
        metallic_debris: g.metallic_debris.clone().unwrap_or_default(),
        nullify_anim: g.weapon_nullify_anim.clone(),
        lightning_warhead: warhead(&g.lightning_warhead),
        bolt_explosion: g.weather_con_bolt_explosion.clone(),
        crew_escape: dec(&g.crew_escape, mode),
        side_crew: [
            infantry(&g.allied_crew),
            infantry(&g.soviet_crew),
            infantry(&g.third_crew),
        ],
        technician: infantry(&g.technician),
        engineer: infantry(&g.engineer),
        survivor_divisor: [
            g.allied_survivor_divisor,
            g.soviet_survivor_divisor,
            g.third_survivor_divisor,
        ],
        sinking_weight: dec(&g.ship_sinking_weight, mode),
        repair_period: repair_period(rate(&g.repair_rate), mode),
        repair_count_vehicle: repair_count(rate(&g.u_repair_rate), mode),
        repair_count_infantry: repair_count(rate(&g.i_repair_rate), mode),
        // spec: S-0314 Constants (repair step 5, infantry repair step 1, repair percentage 0.25)
        repair_step: g.repair_step.unwrap_or(5),
        i_repair_step: g.i_repair_step.unwrap_or(1),
        repair_percent: dec(&g.repair_percent, mode).unwrap_or_else(|| nearest_double("0.25")),
        // spec: S-0314 Constants (self-heal intervals 1000 ticks, amounts 1)
        self_heal_frames: [
            g.self_heal_infantry_frames.unwrap_or(1000),
            g.self_heal_unit_frames.unwrap_or(1000),
        ],
        self_heal_amount: [
            g.self_heal_infantry_amount.unwrap_or(1),
            g.self_heal_unit_amount.unwrap_or(1),
        ],
        // spec: S-0309 Import mapping (ThreatPerOccupant absent 0, EnemyHouseThreatBonus absent 0)
        threat_per_occupant: g.threat_per_occupant.unwrap_or(0),
        enemy_house_bonus: dec(&g.enemy_house_threat_bonus, mode).unwrap_or(F64::ZERO),
        disguise_detection: g.disguise_detection_percent.clone().unwrap_or_default(),
        gravity,
        // spec: S-0306 Import mapping (AlliedWallTransparency absent: no)
        allied_wall_transparency: g.allied_wall_transparency.unwrap_or(false),
        // spec: S-0307 Constants (elevation increment 0, increment bonus 1.0, bonus cap 0.0)
        elevation_increment: g.elevation_increment.unwrap_or(0),
        elevation_bonus: dec(&g.elevation_increment_bonus, mode).unwrap_or(one),
        elevation_cap: dec(&g.elevation_bonus_cap, mode).unwrap_or(F64::ZERO),
        // spec: S-0314 Constants (repair-and-sell threshold absent: 3)
        iq_repair_sell: g.iq_repair_sell.unwrap_or(3),
        // spec: S-0301 Import mapping (row ROF absent: 1.0); S-0302 Import mapping (row FirePower absent: 1.0)
        row_rof: dec(&row.rof, mode).unwrap_or(one),
        row_firepower: dec(&row.firepower, mode).unwrap_or(one),
        // spec: S-0309 Import mapping (DestroyWalls absent: yes)
        row_destroy_walls: row.destroy_walls.unwrap_or(true),
    };
    let countries = country_names
        .iter()
        .map(|n| {
            country_combat(
                &s.countries.get(n).cloned().unwrap_or_default(),
                names,
                mode,
            )
        })
        .collect();
    CombatRules {
        weapons,
        projectiles,
        warheads,
        weapon_index,
        general,
        countries,
    }
}
