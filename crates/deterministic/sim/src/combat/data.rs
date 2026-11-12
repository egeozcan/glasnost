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

/// A range field in leptons after `quirks.negative_range_fields` (S-0307
/// Edge cases): kept as it is when the switch is on, otherwise max(0, v),
/// except that a weapon range of −512 (`unlimited`) stays unlimited.
pub fn range_field(v: i32, keep_negative: bool, weapon_range: bool) -> i32 {
    // spec: S-0307 Edge cases (quirks.negative_range_fields off: max(0, value); a weapon range of −512 remains unlimited)
    if keep_negative || (weapon_range && v == crate::combat::range::UNLIMITED) {
        v
    } else {
        v.max(0)
    }
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
    /// The electric-bolt flag (S-0316 claims 1 to 13).
    pub electric_bolt: bool,
    /// The alternate-colour flag: a bolt's side strands in entry 5 (S-0316
    /// claim 10).
    pub alt_colour: bool,
    /// The laser flag (S-0316 claim 2, S-0514 claim 23).
    pub laser: bool,
    /// The radiation-beam flag (S-0316 claim 2, S-0514 claim 23).
    pub rad_beam: bool,
    /// Spawner weapon.
    pub spawner: bool,
    /// Disc laser weapon.
    pub disk_laser: bool,
    /// The final beam's duration in ticks (S-0615 claim 16).
    pub laser_duration: i32,
    /// The inner, outer and spread laser colours (S-0615 claim 15).
    pub laser_colours: [[i32; 3]; 3], // spec: S-0615 Import mapping (three integer colour channels, S-0003)
    /// The house-colour flag (S-0615 claim 15).
    pub house_colour: bool,
    /// Fire-only-visible flag.
    pub decloak_to_fire: bool,
    /// Lobber flag.
    pub lobber: bool,
    /// The radiation level *L*<sub>w</sub> (S-0525 claim 1; S-0513 claim 12).
    pub rad_level: i32,
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
    /// Launch-inside flag (S-0614 Terms).
    pub limbo_launch: bool,
    /// The animation of an assault's occupant removal (S-0508 claim 14).
    pub occupant_anim: Option<String>,
    /// Animation list: the per-direction delivery animation (S-0614 claim 9)
    /// and the shot animation list (S-0301 claim 20 a).
    pub anim: Vec<String>,
    /// The open-topped animation (S-0301 claim 20 c).
    pub open_topped_anim: Option<String>,
    /// The assault animation; ordinary shots never use it (S-0301 Terms).
    pub assault_anim: Option<String>,
    /// Fake-blink time in ticks (S-0516 claim 39).
    pub disguise_fake_blink: i32,
    /// The unlimited-control flag (S-0601 claim 1).
    pub infinite_mind_control: bool,
    /// The effect weapon's emitter type, a particle-system name (S-0616
    /// claim 1).
    pub attached_system: Option<String>,
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
    /// The fragment weapon (S-0318 Terms).
    pub shrapnel_weapon: Option<WeaponId>,
    /// The fragment count c (S-0318 claim 2).
    pub shrapnel_count: i32,
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
    /// Fire flag: excludes destroyable-cliff cells from the attackable
    /// cells (S-0214 Import mapping).
    pub fire: bool,
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
    /// Bomb-planting warhead (S-0519 Import mapping, `IvanBomb`).
    pub ivan_bomb: bool,
    /// Disarm warhead (S-0519 Import mapping, `BombDisarm`).
    pub bomb_disarm: bool,
    /// Locomotor warhead.
    pub is_locomotor: bool,
    /// What the seize installs: the locomotor the seizing warhead names, by
    /// its class in S-0607 claim 22.
    pub seize_locomotor: SeizeLocomotor,
    /// Registered under the nuclear warhead name (S-0306 claim 25).
    pub nuke: bool,
    /// Parasite flag (S-0614 Terms).
    pub parasite: bool,
    /// Paralysis length P, ticks (S-0614 claims 6, 7).
    pub paralyzes: i32,
    /// Cull flag (S-0614 claim 11).
    pub culling: bool,
    /// Temporal flag (S-0301 Terms, S-0514 Terms): a weapon with this
    /// warhead is a temporal weapon.
    pub temporal: bool,
    /// Disguise flag (S-0516 claim 31).
    pub makes_disguise: bool,
    /// The missile-maker flag (S-0513 claim 8).
    pub nuke_maker: bool,
    /// The takeover flag, `MindControl` (S-0514 claim 20, row 1).
    pub mind_control: bool,
    /// The direct-rocking flag, `directrocker` (S-0514 claim 20, row 8).
    pub direct_rocker: bool,
    /// The bright flag (S-0524 claim 6).
    pub bright: bool,
    /// The flash-size fraction, single precision widened (S-0524 claim 6).
    pub light_size: F64,
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
/// The class of the locomotor a seizing warhead names (S-0607 claims 5 and
/// 22).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeizeLocomotor {
    /// The jumpjet locomotor: the lift of claims 5 to 21.
    Jumpjet,
    /// One that can stand in temporarily (driving, walking, ship,
    /// teleporting or drop-pod movement): the target is seized with it as
    /// its stand-in (claims 22 to 24).
    StandIn(crate::lift::StandIn),
    /// One that cannot stand in temporarily (hovering, mechanical walking,
    /// flying, rocket or tunnelling movement), or an unknown identifier:
    /// claim 5's other effects, no seize.
    NoStandIn,
}

impl Default for SeizeLocomotor {
    fn default() -> Self {
        // spec: S-0607 claim 22 and Import mapping (no locomotor named and no earlier layer: the teleporting locomotor)
        Self::StandIn(crate::lift::StandIn::Teleport)
    }
}

impl SeizeLocomotor {
    /// The class of the imported locomotor name (`convert` writes the
    /// names of S-0207 table T3, and `unknown` for an identifier that table
    /// does not list).
    pub fn of(name: Option<&str>) -> Self {
        use crate::lift::StandIn;
        match name {
            // spec: S-0607 claim 5 (the jumpjet locomotor)
            Some("jumpjet") => Self::Jumpjet,
            // spec: S-0607 claim 22 and Import mapping (an absent key, or an empty or malformed value ignored as absent, with no earlier layer: the teleporting locomotor, which can stand in)
            None | Some("teleport") => Self::StandIn(StandIn::Teleport),
            // spec: S-0607 claim 22 (driving, walking, ship, teleporting or drop-pod movement can stand in temporarily)
            Some("drive") => Self::StandIn(StandIn::Drive),
            Some("walk") => Self::StandIn(StandIn::Walk),
            Some("ship") => Self::StandIn(StandIn::Ship),
            Some("drop_pod") => Self::StandIn(StandIn::DropPod),
            // spec: S-0607 claim 22 (hovering, mechanical walking, flying, rocket or tunnelling movement cannot stand in temporarily; an unknown identifier, design: treated as one that cannot)
            Some(_) => Self::NoStandIn,
        }
    }
}

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
    /// Gattling flag (S-0609 Terms: a gattling-flagged type).
    pub gattling: bool,
    /// The gattling spin settings (S-0609 Terms and Import mapping).
    pub spin: crate::gattling::SpinType,
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
    /// The type's own cloak flag (S-0504 Terms; S-0516 claim 2 a).
    pub cloakable: bool,
    /// Cloak-stop (S-0516 claim 3).
    pub cloak_stop: bool,
    /// Cloaking speed S, ticks per stage (S-0516 claim 1).
    pub cloaking_speed: i32,
    /// Sensors flag (S-0516 claims 13 and 18).
    pub sensors: bool,
    /// Sensors-sight R in cells (S-0516 claims 22 and 23).
    pub sensors_sight: i32,
    /// Can disguise (S-0516 claims 30 to 34).
    pub can_disguise: bool,
    /// Disguises when still (S-0516 claim 33).
    pub disguise_when_still: bool,
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
    /// May be erased by a temporal weapon (S-0514 Import mapping,
    /// `Warpable`).
    pub warpable: bool,
    /// Immune to psionic weapons.
    pub immune_psionic_weapons: bool,
    /// Immune to poison.
    pub immune_poison: bool,
    /// Immune to psionics.
    pub immune_psionics: bool,
    /// Berserk-friendly: a berserk unit may not fire at objects of this
    /// type (S-0610 claim 6).
    pub berserk_friendly: bool,
    /// Self-damage flag.
    pub damage_self: bool,
    /// Eligible damage particle systems (smoke), in list order.
    pub smoke_systems: Vec<String>,
    /// The damage-smoke displacement, leptons (S-0616 claim 1).
    pub smoke_offset: [i32; 3], // nocite: vector and matrix dimensions (three axes)
    /// The spark-kind damage particle systems, in list order (S-0616
    /// claim 1, damage sparks).
    pub spark_systems: Vec<String>,
    /// The natural particle system (S-0616 claim 1).
    pub natural_system: Option<String>,
    /// The natural-smoke displacement, leptons (S-0616 claim 1).
    pub natural_offset: [i32; 3], // nocite: vector and matrix dimensions (three axes)
    /// The destruction particle-system list (S-0312 claim 19 f).
    pub destroy_systems: Vec<String>,
    /// The destruction smoke displacement, leptons (S-0312 claim 19 f).
    pub destroy_offset: [i32; 3], // nocite: vector and matrix dimensions (three axes)
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
    /// The elite strike size (S-0519 Import mapping: absent, the normal).
    pub elite_airstrike_team: i32,
    /// The strike aircraft type name, normal and elite (S-0519 Import
    /// mapping: a missing elite type takes the normal type).
    pub airstrike_team_type: [Option<String>; 2], // spec: S-0519 Terms (normal and elite)
    /// The recharge delay in ticks, normal and elite (S-0519 Import mapping).
    pub airstrike_recharge: [i32; 2], // spec: S-0519 Terms (normal and elite)
    /// The bomb-sight range in cells (S-0519 Import mapping).
    pub bomb_sight: i32,
    /// Drainable.
    pub drainable: bool,
    /// Muzzle positions by slot, ordinary and elite (S-0313).
    pub flh: [Flh; SLOTS],
    /// Elite muzzle positions.
    pub elite_flh: [Flh; SLOTS],
    /// Turret displacement δ.
    pub turret_offset: i32,
    /// The five alternate link anchors (S-0601 claim 17, F5).
    pub alternate_flh: [Flh; crate::mind::ANCHORS],
    /// The type's release sound (S-0601 claim 9).
    pub mind_cleared_sound: Option<String>,
    /// The link-line end height, leptons (S-0601 claim 17).
    pub link_height: i32,
    /// The mobile control-ring height, leptons (S-0601 claim 18).
    pub ring_height: i32,
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
    /// Target-line flag (S-0301 claim 24): drawing only.
    pub target_laser: bool,
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
    /// Can carry a parasite (S-0614 claim 2).
    pub parasiteable: bool,
    /// Re-select flag (S-0614 claim 19, presentation).
    pub reselect_if_limboed: bool,
}

/// The building-only combat fields.
#[derive(Clone, Debug, Default)]
pub struct BuildingCombat {
    /// Defence building (build category *combat*).
    pub defense: bool,
    /// Can be demolished (the charges flag; S-0302 claim 7).
    pub can_c4: bool,
    /// Both a resource gatherer and a resource destination (S-0308 table P
    /// row 6: no airstrike target).
    pub gatherer_destination: bool,
    /// Eligible for delayed kills.
    pub delay_kill: bool,
    /// Leaves rubble.
    pub leave_rubble: bool,
    /// An overpowerable type (S-0518 Import mapping).
    pub overpowerable: bool,
    /// Helipad.
    pub helipad: bool,
    /// Crewed flag (S-0312 claim 18 and F5).
    pub crewed: bool,
    /// Artillery flag.
    pub artillary: bool,
    /// Can be occupied.
    pub can_be_occupied: bool,
    /// Occupants may fire.
    pub can_occupy_fire: bool,
    /// Maximum occupants.
    pub max_occupants: i32,
    /// Garrison muzzle points, pixel shifts (S-0313 claim 6).
    pub garrison_muzzles: Vec<[i32; 2]>, // nocite: pixel pairs, our shape
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

/// The game-wide radiation settings (S-0525 Import mapping).
#[derive(Clone, Debug)]
pub struct RadRules {
    /// The duration multiple *M* (F2).
    pub duration_multiple: i32,
    /// The application delay *A*, ticks (claim 9).
    pub application_delay: i32,
    /// The level cap *C*<sub>max</sub> (F6).
    pub level_max: i32,
    /// The level delay *D*<sub>L</sub>, ticks (claim 5).
    pub level_delay: i32,
    /// The light delay *D*<sub>I</sub>, ticks (claim 14).
    pub light_delay: i32,
    /// The level factor *k*<sub>L</sub> (F6).
    pub level_factor: F64,
    /// The light factor *k*<sub>I</sub> (F7).
    pub light_factor: F64,
    /// The tint factor *k*<sub>T</sub> (F7).
    pub tint_factor: F64,
    /// The radiation colour (F7).
    pub colour: [i32; 3], // spec: S-0525 Import mapping (RadColor: a byte triple)
    /// The radiation-site warhead (claim 10).
    pub site_warhead: Option<WarheadId>,
}

/// The game-wide planted-bomb and charge settings (S-0519 Import mapping).
#[derive(Clone, Debug)]
pub struct IvanRules {
    /// The bomb warhead.
    pub warhead: Option<WarheadId>,
    /// The bomb damage, health points.
    pub damage: i32,
    /// The timed delay *D*, ticks.
    pub delay: i32,
    /// The icon flicker rate *F*, ticks.
    pub flicker: i32,
    /// The timed-bomb detonate permission (S-0214 row G3).
    pub can_detonate_time_bomb: bool,
    /// The charge delay *c* in minutes, a double (F4).
    pub c4_delay: F64,
}

/// The game-wide drain settings (S-0608 Import mapping).
#[derive(Clone, Debug, Default)]
pub struct DrainRules {
    /// The payout period *P* in ticks (F1).
    pub period: i32,
    /// The payout amount *A* in credits (F2).
    pub amount: i32,
    /// The drain animation (claim 5).
    pub anim: Option<String>,
}

/// The game-wide prism support settings (S-0518 Import mapping).
#[derive(Clone, Debug)]
pub struct PrismRules {
    /// The prism type: a building type index.
    pub building: Option<u32>,
    /// The held support modifier *s*, a whole-number percentage (F4).
    pub modifier: i32,
    /// The support cap *N*ₘₐₓ.
    pub max: i32,
    /// The support rearm *R*ₛ, ticks.
    pub rearm: i32,
    /// The beam duration *L*ₛ, ticks.
    pub duration: i32,
}

/// The mind-control settings (S-0601 Import mapping).
#[derive(Clone, Debug, Default)]
pub struct MindRules {
    /// The rules-wide release sound (claim 9).
    pub cleared_sound: Option<String>,
    /// The take-over sound (claim 5).
    pub takeover_sound: Option<String>,
    /// The overload sound (claim 16).
    pub overload_sound: Option<String>,
    /// The control ring animation (claim 18).
    pub ring: Option<String>,
    /// The permanent control ring animation (claim 18).
    pub perma_ring: Option<String>,
    /// The link-line timer, ticks (claim 17).
    pub line_frames: i32,
    /// *count* of F2.
    pub overload_count: Vec<i32>,
    /// *damage* of F2.
    pub overload_damage: Vec<i32>,
    /// *frames* of F2.
    pub overload_frames: Vec<i32>,
    /// The fate tables of claim 8: normal, wounded, low power, low money;
    /// each the weights of team, grinder, bio reactor, hunt.
    pub fate: [Vec<i32>; 4], // spec: S-0601 claim 8 (four fate tables)
    /// The low-money mark, credits (claim 8; absent: none, never selecting its table).
    pub low_money_mark: Option<i32>,
    /// The wounded mark, a single widened (claim 8; absent: none, never selecting its table).
    pub wounded_mark: Option<F64>,
}

/// Game-wide combat values.
#[derive(Clone, Debug)]
pub struct CombatGeneral {
    /// The mind-control settings (S-0601).
    pub mind: MindRules,
    /// The planted-bomb and charge settings (S-0519).
    pub ivan: IvanRules,
    /// The prism support settings (S-0518).
    pub prism: PrismRules,
    /// The completion animation of an erasure (S-0514 Import mapping).
    pub warp_away: Option<String>,
    /// The sparkle animation of an erased or phasing-in unit (S-0514 claim
    /// 18).
    pub chrono_sparkle: Option<String>,
    /// The open-topped warp distance *W*, cells (S-0514 F4).
    pub warp_distance: i32,
    /// The drain settings (S-0608).
    pub drain: DrainRules,
    /// The radiation settings (S-0525).
    pub radiation: RadRules,
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
    /// The falling-damage multiplier *m*, a single widened (S-0607 F2).
    pub falling_damage_multiplier: F64,
    /// The current-strength switch (S-0607 F2).
    pub current_strength_damage: bool,
    /// Ion-cannon warhead.
    pub ion_cannon_warhead: Option<WarheadId>,
    /// Bridge strength, bridge-strength points (S-0303 claims 18 and 24).
    pub bridge_strength: i32,
    /// Ammunition-crate damage, damage points (S-0303 claims 19 and 24).
    pub ammo_crate_damage: i32,
    /// The railgun hit distance, leptons (S-0306 claim 27).
    pub railgun_radius: i32,
    /// The railgun cliff-collapse chance, percentage points (S-0306 claim 27).
    pub collapse_chance: i32,
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
    /// Infantry-explosion animation (S-0302 claim 15; death kind 3 of
    /// S-0615 claim 1).
    pub infantry_explode: Option<String>,
    /// The general corpse list (S-0107 claim 17).
    pub dead_bodies: Vec<String>,
    /// The burning death animation, death kind 4 (S-0615 claim 1).
    pub flaming_infantry: Option<String>,
    /// The head-pop death animation, death kind 6 (S-0615 claim 1).
    pub infantry_head_pop: Option<String>,
    /// The nuked death animation, death kind 7 (S-0615 claim 1).
    pub infantry_nuked: Option<String>,
    /// The crushing-blow death animation, death kind 10 (S-0615 claim 1).
    pub infantry_brute: Option<String>,
    /// The warhead of an animation's damage over time (S-0109 claim 17,
    /// `FlameDamage2`).
    pub flame_damage2: Option<WarheadId>,
    /// The disc laser's charge-up sound (S-0615 claim 15).
    pub disk_laser_charge_up: Option<String>,
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
    /// The bridge-explosion list (S-0511 claim 9, F2).
    pub bridge_explosions: Vec<String>,
    /// *v*_min of S-0511 F3, the no-parachute maximum fall rate, leptons
    /// per update.
    pub no_parachute_max_fall_rate: i32,
    /// The parachute maximum fall rate of S-0511 F3, leptons per update.
    pub parachute_max_fall_rate: i32,
    /// The repair sound (S-0511 claim 17).
    pub repair_bridge_sound: Option<String>,
    /// Wake animation (S-0614 claim 10; S-0312).
    pub wake: Option<String>,
    /// Default spark system (S-0614 claim 9).
    pub default_spark_system: Option<String>,
    /// Nullify animation.
    pub nullify_anim: Option<String>,
    /// Lightning warhead.
    pub lightning_warhead: Option<WarheadId>,
    /// Lightning-bolt explosion.
    pub bolt_explosion: Option<String>,
    /// Crew escape chance (S-0312 claims 13 and 27).
    pub crew_escape: F64,
    /// Crew types by side: allied, soviet, third (infantry indices).
    pub side_crew: [Option<u32>; 3], // spec: S-0312 Terms (three side crew types)
    /// Technician type.
    pub technician: Option<u32>,
    /// Engineer type.
    pub engineer: Option<u32>,
    /// Survivor divisors q by side 0, 1 and 2 (S-0312 F5).
    pub survivor_divisor: [i32; 3], // spec: S-0312 F5 (three survivor divisors)
    /// Sinking weight (S-0312 claims 10 and 27).
    pub sinking_weight: F64,
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
    /// The per-occupant threat of the threat records (S-0212 claim 16a),
    /// whose absent-key default differs from S-0309's.
    pub record_threat_per_occupant: i32,
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
    /// The repair reserve, in credits (S-0314 claim 12).
    pub credit_reserve: i32,
    /// The sale threshold (S-0314 claim 14), compared as unsigned.
    pub iq_sell_back: i32,
    /// The normal row's rate-of-fire factor ℓ.
    pub row_rof: F64,
    /// The normal row's firepower factor φᵣ.
    pub row_firepower: F64,
    /// The normal row's destroy-walls flag.
    pub row_destroy_walls: bool,
    /// The rate-of-fire factors ℓ of the easy, normal and difficult rows,
    /// for the handicap (S-0412 claim 5).
    pub rows_rof: [F64; 3], // spec: S-0412 claim 5 (the three rows: easy, normal, difficult)
    /// The firepower factors φᵣ of the easy, normal and difficult rows.
    pub rows_firepower: [F64; 3], // spec: S-0412 claim 5 (the three rows: easy, normal, difficult)
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
    negative_range_fields: bool,
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
            // spec: S-0318 Import mapping (ShrapnelWeapon absent: no fragments)
            shrapnel_weapon: p
                .shrapnel_weapon
                .as_ref()
                .and_then(|w| w_index.get(w).copied()),
            shrapnel_count: p.shrapnel_count.unwrap_or(0), // spec: S-0318 Import mapping (ShrapnelCount absent: 0)
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
                    // The importer writes each versus value as the exact decimal of its double (S-0003 claim 13), read back at double precision.
                    verses[i] = glasnost_num::decimal::nearest_double(&d.0);
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
                // spec: S-0214 Import mapping (warhead Fire absent: no)
                fire: b(h.fire),
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
                ivan_bomb: b(h.ivan_bomb),
                bomb_disarm: b(h.bomb_disarm),
                is_locomotor: b(h.is_locomotor),
                seize_locomotor: SeizeLocomotor::of(h.locomotor.as_deref()),
                // spec: S-0306 claim 25 and Import mapping (the nuclear warhead name, compared without regard to case)
                nuke: n.eq_ignore_ascii_case("NUKE"),
                // spec: S-0614 Import mapping (Parasite absent: no; Paralyzes absent: 0; Culling absent: no)
                parasite: b(h.parasite),
                paralyzes: h.paralyzes.unwrap_or(0),
                culling: b(h.culling),
                // spec: S-0301 Import mapping and S-0514 Import mapping (Temporal, a warhead key; absent: no)
                temporal: b(h.temporal),
                // spec: S-0516 Import mapping (MakesDisguise absent: no)
                makes_disguise: b(h.makes_disguise),
                // spec: S-0513 Import mapping (NukeMaker absent: no)
                nuke_maker: b(h.nuke_maker),
                // spec: S-0514 Import mapping (MindControl and directrocker, the flags of claim 20's selection; absent: no)
                mind_control: b(h.mind_control),
                direct_rocker: b(h.direct_rocker),
                // spec: S-0524 I2 (Bright absent: false)
                bright: b(h.bright),
                // spec: S-0524 I2 (CombatLightSize: narrowed to single precision; absent 0)
                light_size: single(&h.combat_light_size, mode).unwrap_or(F64::ZERO),
            }
        })
        .collect();
    let wr: Vec<WeaponRules> = weapons
        .iter()
        .map(|(n, w)| {
            let projectile = w.projectile.as_ref().and_then(|p| p_index.get(p).copied());
            // spec: S-0307 Edge cases (quirks.negative_range_fields; the replaced range also feeds the speed below)
            let range = range_field(
                w.range.as_ref().map_or(0, |d| leptons(d, mode)),
                negative_range_fields,
                true,
            );
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
                min_range: range_field(
                    w.minimum_range.as_ref().map_or(0, |d| leptons(d, mode)),
                    negative_range_fields,
                    false,
                ),
                cell_rangefinding: b(w.cell_rangefinding),
                sonic: b(w.is_sonic),
                spark: b(w.use_spark_particles),
                fire_particles: b(w.use_fire_particles),
                railgun: b(w.is_railgun),
                mag_beam: b(w.is_mag_beam),
                // spec: S-0316 Import mapping (IsElectricBolt, IsAlternateColor, IsLaser, IsRadBeam: booleans, absent no)
                electric_bolt: b(w.is_electric_bolt),
                alt_colour: b(w.is_alternate_color),
                laser: b(w.is_laser),
                rad_beam: b(w.is_rad_beam),
                spawner: b(w.spawner),
                disk_laser: b(w.disk_laser),
                // spec: S-0615 Import mapping (LaserDuration absent: 10 ticks)
                laser_duration: w.laser_duration.unwrap_or(10),
                // spec: S-0615 Import mapping (LaserInnerColor, LaserOuterColor, LaserOuterSpread: each absent (0, 0, 0))
                laser_colours: [
                    w.laser_inner_color.unwrap_or([0; 3]),
                    w.laser_outer_color.unwrap_or([0; 3]),
                    w.laser_outer_spread.unwrap_or([0; 3]),
                ],
                // spec: S-0615 Import mapping (IsHouseColor absent: no)
                house_colour: b(w.is_house_color),
                // spec: S-0301 Import mapping (DecloakToFire absent: yes)
                decloak_to_fire: w.decloak_to_fire.unwrap_or(true),
                lobber: b(w.lobber),
                // spec: S-0513 Import mapping and S-0525 Import mapping (RadLevel absent: 0 radiation-level units)
                rad_level: w.rad_level.unwrap_or(0),
                never_use: b(w.never_use),
                drain: b(w.drain_weapon),
                // spec: S-0310 Import mapping (FireWhileMoving absent: yes)
                fire_while_moving: w.fire_while_moving.unwrap_or(true),
                suicide: b(w.suicide),
                supress: b(w.supress),
                fire_once: b(w.fire_once),
                omni_fire: b(w.omni_fire),
                area_fire: b(w.area_fire),
                // spec: S-0614 Import mapping (LimboLaunch absent: no; Anim absent: empty)
                limbo_launch: b(w.limbo_launch),
                occupant_anim: w.occupant_anim.clone(),
                anim: w.anim.clone().unwrap_or_default(),
                // spec: S-0301 Import mapping (OpenToppedAnim and AssaultAnim absent: none)
                open_topped_anim: w.open_topped_anim.clone(),
                assault_anim: w.assault_anim.clone(),
                // spec: S-0516 Import mapping (DisguiseFakeBlinkTime absent: 0 ticks)
                disguise_fake_blink: w.disguise_fake_blink_time.unwrap_or(0),
                // spec: S-0601 Import mapping (InfiniteMindControl absent: no)
                infinite_mind_control: b(w.infinite_mind_control),
                // spec: S-0616 Import mapping (AttachedParticleSystem absent: none)
                attached_system: w.attached_particle_system.clone(),
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
    // spec: S-0313 claim 1a and Import mapping (revision 37: an absent non-elite muzzle position is 0, 0, 0)
    let mut flh = [[0; 3]; SLOTS];
    // spec: S-0313 claim 1b (revision 42: the turret count decides which family a reading takes; above 0 the numbered fields only, the primary and secondary fields and their elite forms not read)
    let numbered = turret_count > 0;
    if numbered {
        if let Some(v) = &c.weapon_flh {
            for (i, f) in v.iter().take(SLOTS).enumerate() {
                flh[i] = *f;
            }
        }
    } else {
        // spec: S-0313 claim 1b (turret count 0 or less: the primary and secondary fields give slots 0 and 1; the numbered fields not read)
        flh[0] = c.primary_fire_flh.unwrap_or([0; 3]); // spec: S-0313 claim 1a (an absent primary muzzle position is 0, 0, 0; nothing from the other slot)
        flh[1] = c.secondary_fire_flh.unwrap_or([0; 3]); // spec: S-0313 claim 1a (an absent secondary muzzle position is 0, 0, 0; nothing from the other slot)
    }
    // spec: S-0313 claims 1a and 1b and Import mapping (the elite version of a slot with no elite muzzle position takes the same slot's non-elite one, as imported; for numbered weapons, elite weapon n takes weapon n's; the elite primary field plays no part for a turret count above 0)
    let mut elite_flh = flh;
    if numbered {
        if let Some(v) = &c.elite_weapon_flh {
            for (i, f) in v.iter().take(SLOTS).enumerate() {
                elite_flh[i] = *f;
            }
        }
    } else {
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
        spin: crate::gattling::SpinType::from_fields(c),
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
        // spec: S-0504 Import mapping (Cloakable absent: no)
        cloakable: b(c.cloakable),
        // spec: S-0516 Import mapping (CloakStop absent: no)
        cloak_stop: b(c.cloak_stop),
        // spec: S-0516 Import mapping (CloakingSpeed absent: 7 ticks per stage)
        cloaking_speed: c.cloaking_speed.unwrap_or(7),
        // spec: S-0516 Import mapping (Sensors absent: no)
        sensors: b(c.sensors),
        // spec: S-0516 Import mapping (SensorsSight absent: 0 cells)
        sensors_sight: c.sensors_sight.unwrap_or(0),
        // spec: S-0516 Import mapping (CanDisguise absent: no)
        can_disguise: b(c.can_disguise),
        // spec: S-0516 Import mapping (DisguiseWhenStill absent: no)
        disguise_when_still: b(c.disguise_when_still),
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
        warpable: true, // spec: S-0514 Import mapping (Warpable absent: yes)
        // spec: S-0610 claim 1 (both psychic immunities absent: no here; building types take yes in `building_rules`)
        immune_psionic_weapons: b(c.immune_to_psionic_weapons),
        immune_poison: b(c.immune_to_poison),
        immune_psionics: b(c.immune_to_psionics),
        // spec: S-0610 Import mapping (BerserkFriendly absent: no)
        berserk_friendly: b(c.berserk_friendly),
        damage_self: b(c.damage_self),
        smoke_systems: c.damage_smoke_systems.clone().unwrap_or_default(),
        // spec: S-0616 Import mapping (DamageSmokeOffset absent 0, 0, 0)
        smoke_offset: c.damage_smoke_offset.unwrap_or([0; 3]),
        spark_systems: c.damage_spark_systems.clone().unwrap_or_default(),
        // spec: S-0616 Import mapping (NaturalParticleSystem absent: none; NaturalParticleLocation absent 0, 0, 0)
        natural_system: c.natural_particle_system.clone(),
        natural_offset: c.natural_particle_location.unwrap_or([0; 3]), // nocite: vector and matrix dimensions (three axes)
        // spec: S-0312 Import mapping (DestroyParticleSystems absent: empty; DestroySmokeOffset absent 0, 0, 0)
        destroy_systems: c.destroy_particle_systems.clone().unwrap_or_default(),
        destroy_offset: c.destroy_smoke_offset.unwrap_or([0; 3]), // nocite: vector and matrix dimensions (three axes)
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
        // spec: S-0519 Import mapping (the elite values default to the normal ones; recharge 0; bomb sight 0)
        elite_airstrike_team: c
            .elite_airstrike_team
            .unwrap_or_else(|| c.airstrike_team.unwrap_or(0)),
        airstrike_team_type: [
            c.airstrike_team_type.clone(),
            c.elite_airstrike_team_type
                .clone()
                .or_else(|| c.airstrike_team_type.clone()),
        ],
        airstrike_recharge: [
            c.airstrike_recharge_time.unwrap_or(0),
            c.elite_airstrike_recharge_time
                .unwrap_or_else(|| c.airstrike_recharge_time.unwrap_or(0)),
        ],
        bomb_sight: c.bomb_sight.unwrap_or(0),
        drainable: b(c.drainable),
        flh,
        elite_flh,
        // spec: S-0313 Import mapping (TurretOffset absent: 0)
        turret_offset: c.turret_offset.unwrap_or(0),
        alternate_flh: {
            // spec: S-0601 Import mapping (AlternateFLH0 through AlternateFLH4 absent: the zero triple)
            let mut a = [[0; 3]; crate::mind::ANCHORS];
            for (i, f) in c
                .alternate_flh
                .iter()
                .flatten()
                .take(crate::mind::ANCHORS)
                .enumerate()
            {
                a[i] = *f;
            }
            a
        },
        // spec: S-0601 Import mapping (MindClearedSound of the type absent: none)
        mind_cleared_sound: c.mind_cleared_sound.clone(),
        // spec: S-0601 Import mapping and Constants (LeptonMindControlOffset absent: 70 leptons)
        link_height: c.lepton_mind_control_offset.unwrap_or(70),
        // spec: S-0601 Import mapping and Constants (MindControlRingOffset absent: 140 leptons)
        ring_height: c.mind_control_ring_offset.unwrap_or(140),
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
        // spec: S-0301 Import mapping (TargetLaser absent: no)
        target_laser: b(u.target_laser),
        deploy_to_fire: b(u.deploy_to_fire),
        jump_jet_turn: b(u.jump_jet_turn),
        crewed: b(u.crewed),
        carries_crate: b(u.carries_crate),
        crashable: b(u.crashable),
        weight: dec(&u.weight, mode),
        // spec: S-0315 Constants (suppression threshold absent: 0)
        suppression_threshold: u.suppression_threshold.unwrap_or(0),
        missile_spawn: b(u.missile_spawn),
        // spec: S-0614 Import mapping (Parasiteable absent: yes for all three type kinds)
        parasiteable: u.parasiteable.unwrap_or(true),
        // spec: S-0614 Import mapping (ReselectIfLimboed absent: no)
        reselect_if_limboed: b(u.reselect_if_limboed),
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
        mind: MindRules {
            // spec: S-0601 Import mapping (the three sounds and the two ring animations absent: none)
            cleared_sound: g.mind_cleared_sound.clone(),
            takeover_sound: g.yuri_mind_control_sound.clone(),
            overload_sound: g.master_mind_overload_death_sound.clone(),
            ring: g.controlled_animation_type.clone(),
            perma_ring: g.perma_controlled_animation_type.clone(),
            // spec: S-0601 Import mapping (MindControlAttackLineFrames absent: 15 ticks)
            line_frames: g.mind_control_attack_line_frames.unwrap_or(15),
            // spec: S-0601 Import mapping (OverloadCount, OverloadDamage, OverloadFrames absent: empty)
            overload_count: g.overload_count.clone().unwrap_or_default(),
            overload_damage: g.overload_damage.clone().unwrap_or_default(),
            overload_frames: g.overload_frames.clone().unwrap_or_default(),
            // spec: S-0601 Import mapping (the four fate tables absent: empty, in the order normal, wounded, low power, low money)
            fate: [
                g.ai_capture_normal.clone().unwrap_or_default(),
                g.ai_capture_wounded.clone().unwrap_or_default(),
                g.ai_capture_low_power.clone().unwrap_or_default(),
                g.ai_capture_low_money.clone().unwrap_or_default(),
            ],
            // spec: S-0601 claim 8 and Import mapping (revision 9: an absent mark is none and never selects its table, our engine's design rule)
            low_money_mark: g.ai_capture_low_money_mark,
            // spec: S-0601 claim 8 (the wounded mark is a binary32 threshold, compared without narrowing the quotient)
            wounded_mark: single(&g.ai_capture_wounded_mark, mode),
        },
        ivan: IvanRules {
            warhead: warhead(&g.ivan_warhead),
            // spec: S-0519 Import mapping (IvanDamage absent: 100; IvanTimedDelay 450; IvanIconFlickerRate 8; both permissions no)
            damage: g.ivan_damage.unwrap_or(100),
            delay: g.ivan_timed_delay.unwrap_or(450), // spec: S-0519 Constants (timed delay default 450)
            flicker: g.ivan_icon_flicker_rate.unwrap_or(8), // spec: S-0519 Constants (flicker rate default 8)
            can_detonate_time_bomb: g.can_detonate_time_bomb.unwrap_or(false),
            // spec: S-0519 Import mapping and F4 (C4Delay absent: the double nearest 0.03, never through single precision; a given value is a single widened)
            c4_delay: g
                .c4_delay
                .as_ref()
                .map_or_else(|| nearest_double("0.03"), |d| d.single_value(mode)),
        },
        prism: PrismRules {
            building: g
                .prism_type
                .as_ref()
                .and_then(|n| names.get(&(Kind::Building, n.clone())).copied()),
            // spec: S-0518 Import mapping (the engine's starting held modifier is 100; support cap 8, rearm 100, duration 15)
            modifier: g.prism_support_modifier.unwrap_or(100), // spec: S-0518 Import mapping (starting held modifier 100)
            max: g.prism_support_max.unwrap_or(8), // spec: S-0518 Import mapping (support cap 8)
            rearm: g.prism_support_delay.unwrap_or(100), // spec: S-0518 Import mapping (support rearm 100)
            duration: g.prism_support_duration.unwrap_or(15), // spec: S-0518 Import mapping (beam duration 15)
        },
        warp_away: g.warp_away.clone(),
        chrono_sparkle: g.chrono_sparkle.clone(),
        // spec: S-0514 Constants (warp distance default 5 cells) and Import mapping
        warp_distance: g.open_topped_warp_distance.unwrap_or(5),
        drain: DrainRules {
            // spec: S-0608 Import mapping and Constants (payout period absent: 100 ticks)
            period: g.drain_money_frame_delay.unwrap_or(100),
            // spec: S-0608 Import mapping and Constants (payout amount absent: 0 credits)
            amount: g.drain_money_amount.unwrap_or(0),
            // spec: S-0608 Import mapping (DrainAnimationType absent: no animation)
            anim: g.drain_animation_type.clone(),
        },
        // spec: S-0525 Import mapping (every [Radiation] value absent: the value in force, 0 or none at a fresh start)
        radiation: RadRules {
            duration_multiple: g.rad_duration_multiple.unwrap_or(0),
            application_delay: g.rad_application_delay.unwrap_or(0),
            level_max: g.rad_level_max.unwrap_or(0),
            level_delay: g.rad_level_delay.unwrap_or(0),
            light_delay: g.rad_light_delay.unwrap_or(0),
            level_factor: dec(&g.rad_level_factor, mode).unwrap_or(F64::ZERO),
            light_factor: dec(&g.rad_light_factor, mode).unwrap_or(F64::ZERO),
            tint_factor: dec(&g.rad_tint_factor, mode).unwrap_or(F64::ZERO),
            colour: g.rad_color.unwrap_or([0, 0, 0]),
            site_warhead: warhead(&g.rad_site_warhead),
        },
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
        // spec: S-0607 F2 and Constants (FallingDamageMultiplier absent: 1.0, a single)
        falling_damage_multiplier: single(&g.falling_damage_multiplier, mode).unwrap_or(one),
        // spec: S-0607 F2 and Constants (CurrentStrengthDamage absent: on)
        current_strength_damage: g.current_strength_damage.unwrap_or(true),
        ion_cannon_warhead: warhead(&g.ion_cannon_warhead),
        // spec: S-0303 claim 24 (at a fresh program start the bridge strength is 1000 and the ammunition-crate damage 100; an omission keeps the value in force)
        bridge_strength: g.bridge_strength.unwrap_or(1000),
        ammo_crate_damage: g.ammo_crate_damage.unwrap_or(100), // spec: S-0303 claim 24 (ammunition-crate damage 100)
        railgun_radius: g.railgun_damage_radius.unwrap_or(128), // spec: S-0306 Import mapping (railgundamageradius absent: 128 leptons)
        collapse_chance: g.collapse_chance.unwrap_or(100), // spec: S-0306 Import mapping (CollapseChance absent: 100)
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
        // spec: S-0107 Import mapping ([General] DeadBodies, absent empty)
        dead_bodies: g.dead_bodies.clone().unwrap_or_default(),
        // spec: S-0615 Import mapping (the game-wide death animations of kinds 4, 6, 7 and 10: absent none; the charge-up sound: absent silent)
        flaming_infantry: g.flaming_infantry.clone(),
        infantry_head_pop: g.infantry_head_pop.clone(),
        infantry_nuked: g.infantry_nuked.clone(),
        infantry_brute: g.infantry_brute.clone(),
        disk_laser_charge_up: g.disk_laser_charge_up.clone(),
        // spec: S-0109 Import mapping (FlameDamage2 absent: none, no damage is done)
        flame_damage2: warhead(&g.flame_damage2),
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
        // spec: S-0511 Import mapping (BridgeExplosions absent: empty)
        bridge_explosions: g.bridge_explosions.clone().unwrap_or_default(),
        no_parachute_max_fall_rate: g.no_parachute_max_fall_rate.unwrap_or(-100), // spec: S-0511 Import mapping (NoParachuteMaxFallRate absent: −100)
        parachute_max_fall_rate: g.parachute_max_fall_rate.unwrap_or(-3), // spec: S-0511 Import mapping (ParachuteMaxFallRate absent: −3)
        // spec: S-0511 Import mapping (RepairBridgeSound absent: no sound)
        repair_bridge_sound: g.repair_bridge_sound.clone(),
        // spec: S-0614 Import mapping (Wake as S-0312; DefaultSparkSystem default not established: none)
        wake: g.wake.clone(),
        default_spark_system: g.default_spark_system.clone(),
        nullify_anim: g.weapon_nullify_anim.clone(),
        lightning_warhead: warhead(&g.lightning_warhead),
        bolt_explosion: g.weather_con_bolt_explosion.clone(),
        // spec: S-0312 claim 27 (at a fresh program start the crew escape chance is 0.5; an omission keeps the value in force)
        crew_escape: dec(&g.crew_escape, mode).unwrap_or_else(|| nearest_double("0.5")),
        side_crew: [
            infantry(&g.allied_crew),
            infantry(&g.soviet_crew),
            infantry(&g.third_crew),
        ],
        technician: infantry(&g.technician),
        engineer: infantry(&g.engineer),
        // spec: S-0312 claim 27 and Import mapping (survivor divisors absent from every layer: 200, 100 and 300, engine constants)
        survivor_divisor: [
            g.allied_survivor_divisor.unwrap_or(200),
            g.soviet_survivor_divisor.unwrap_or(100),
            g.third_survivor_divisor.unwrap_or(300),
        ],
        // spec: S-0312 claim 27 (at a fresh program start the sinking weight is 3.0)
        sinking_weight: dec(&g.ship_sinking_weight, mode).unwrap_or_else(|| F64::from_i64(3, mode)),
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
        // spec: S-0212 Import mapping ([General] ThreatPerOccupant absent: 5, the per-occupant threat of claim 16a)
        record_threat_per_occupant: g.threat_per_occupant.unwrap_or(5),
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
        // spec: S-0314 Constants (repair reserve absent: 1000 credits)
        credit_reserve: g.credit_reserve.unwrap_or(1000),
        // spec: S-0314 Constants (sale threshold absent: 2)
        iq_sell_back: g.iq_sell_back.unwrap_or(2),
        // spec: S-0301 Import mapping (row ROF absent: 1.0); S-0302 Import mapping (row FirePower absent: 1.0)
        row_rof: dec(&row.rof, mode).unwrap_or(one),
        row_firepower: dec(&row.firepower, mode).unwrap_or(one),
        // spec: S-0309 Import mapping (DestroyWalls absent: yes)
        row_destroy_walls: row.destroy_walls.unwrap_or(true),
        // spec: S-0412 claim 5 (the handicap's rows by our names easy, normal, difficult); S-0301 and S-0302 Import mapping (absent: 1.0)
        rows_rof: ["easy", "normal", "difficult"].map(|n| {
            dec(&s.difficulty.get(n).cloned().unwrap_or_default().rof, mode).unwrap_or(one)
        }),
        rows_firepower: ["easy", "normal", "difficult"].map(|n| {
            dec(
                &s.difficulty.get(n).cloned().unwrap_or_default().firepower,
                mode,
            )
            .unwrap_or(one)
        }),
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
