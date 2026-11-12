//! The combat fields of the typed `sim` section (M4): weapons, projectiles,
//! warheads, the combat fields of object types, countries, the difficulty
//! rows and the game-wide combat values, each read as the Import mapping of
//! the combat spec named on its row says (S-0301 to S-0315), with the
//! importer warnings those specs ask for.

use glasnost_ini::binfloat::{Binary, Format};
use glasnost_rules::sim::ArmourClass;

use super::*;
use crate::art::int_as_of;

/// The largest gattling stage count our engine reads (S-0609 Constants).
const SPIN_STAGES: usize = 6; // spec: S-0609 Constants (largest stage count supported: 6 stages)

/// A table row: our field name, the INI key, the value form and the spec
/// whose Import mapping maps the key.
type Row = (&'static str, &'static str, Form, &'static str);

/// Combat fields shared by infantry, vehicle, aircraft and building types
/// (`glasnost_rules::sim::CombatFields`), written into `combat`.
pub(super) const COMBAT_FIELDS: &[Row] = &[
    ("is_gattling", "IsGattling", Bool, "S-0308, S-0609"),
    // spec: S-0609 Import mapping (WeaponStages, RateUp, RateDown: integers, held value, absent 0)
    ("weapon_stages", "WeaponStages", Int, "S-0609"),
    ("rate_up", "RateUp", Int, "S-0609"),
    ("rate_down", "RateDown", Int, "S-0609"),
    ("is_charge_turret", "IsChargeTurret", Bool, "S-0301, S-0308"),
    ("land_targeting", "LandTargeting", Int, "S-0306, S-0308"),
    // spec: S-0616 Import mapping (object type NaturalParticleSystem: a particle-system name; absent none)
    (
        "natural_particle_system",
        "NaturalParticleSystem",
        Ref(Kind::ParticleSystems),
        "S-0616",
    ),
    ("naval_targeting", "NavalTargeting", Int, "S-0306, S-0308"),
    ("underwater", "Underwater", Bool, "S-0308, S-0504"),
    ("cloakable", "Cloakable", Bool, "S-0504, S-0516"),
    ("cloak_stop", "CloakStop", Bool, "S-0516"),
    ("cloaking_speed", "CloakingSpeed", Int, "S-0516"),
    ("sensors", "Sensors", Bool, "S-0516"),
    ("sensors_sight", "SensorsSight", Int, "S-0516"),
    ("can_disguise", "CanDisguise", Bool, "S-0516"),
    ("disguise_when_still", "DisguiseWhenStill", Bool, "S-0516"),
    ("organic", "Organic", Bool, "S-0308, S-0314"),
    ("unnatural", "Unnatural", Bool, "S-0308"),
    ("naval", "Naval", Bool, "S-0308"),
    ("ammo", "Ammo", Int, "S-0301"),
    ("initial_ammo", "InitialAmmo", Int, "S-0301"),
    ("reload", "Reload", Int, "S-0301"),
    ("empty_reload", "EmptyReload", Int, "S-0301"),
    ("reload_increment", "ReloadIncrement", Int, "S-0301"),
    ("pip_wrap", "PipWrap", Int, "S-0301"),
    ("manual_reload", "ManualReload", Bool, "S-0301"),
    (
        "damage_reduces_readiness",
        "DamageReducesReadiness",
        Bool,
        "S-0301, S-0302",
    ),
    (
        "readiness_reduction_multiplier",
        "ReadinessReductionMultiplier",
        Dec,
        "S-0302",
    ),
    // The lepton form of S-0307 claim 1: the decimal text (−1 is a "not given" value).
    ("air_range_bonus", "AirRangeBonus", Dec, "S-0307"),
    ("radial_fire_segments", "RadialFireSegments", Int, "S-0306"),
    ("type_immune", "TypeImmune", Bool, "S-0302"),
    (
        "immune_to_psionic_weapons",
        "ImmuneToPsionicWeapons",
        Bool,
        "S-0302, S-0610",
    ),
    ("immune_to_poison", "ImmuneToPoison", Bool, "S-0302"),
    (
        "immune_to_psionics",
        "ImmuneToPsionics",
        Bool,
        "S-0302, S-0610",
    ),
    ("berserk_friendly", "BerserkFriendly", Bool, "S-0610"),
    ("mind_cleared_sound", "MindClearedSound", Text, "S-0601"),
    (
        "lepton_mind_control_offset",
        "LeptonMindControlOffset",
        Int,
        "S-0601",
    ),
    (
        "mind_control_ring_offset",
        "MindControlRingOffset",
        Int,
        "S-0601",
    ),
    ("damage_self", "DamageSelf", Bool, "S-0302, S-0303"),
    ("max_debris", "MaxDebris", Int, "S-0312"),
    ("min_debris", "MinDebris", Int, "S-0312"),
    (
        "debris_types",
        "DebrisTypes",
        Refs(Kind::VoxelAnimations),
        "S-0312",
    ),
    ("debris_maximums", "DebrisMaximums", Maxima, "S-0312"),
    (
        "debris_anims",
        "DebrisAnims",
        Refs(Kind::Animations),
        "S-0312",
    ),
    ("explodes", "Explodes", Bool, "S-0312"),
    ("death_weapon", "DeathWeapon", Ref(Kind::Weapons), "S-0312"),
    (
        "death_weapon_damage_modifier",
        "DeathWeaponDamageModifier",
        Dec,
        "S-0312",
    ),
    ("explosion", "Explosion", Refs(Kind::Animations), "S-0312"),
    (
        "destroy_anim",
        "DestroyAnim",
        Refs(Kind::Animations),
        "S-0312",
    ),
    (
        "destroy_particle_systems",
        "DestroyParticleSystems",
        Refs(Kind::ParticleSystems),
        "S-0003, S-0312",
    ),
    ("trainable", "Trainable", Bool, "S-0311"),
    ("dont_score", "DontScore", Bool, "S-0311"),
    ("self_healing", "SelfHealing", Bool, "S-0314"),
    ("can_passive_aquire", "CanPassiveAquire", Bool, "S-0309"),
    ("can_retaliate", "CanRetaliate", Bool, "S-0309"),
    ("detect_disguise", "DetectDisguise", Bool, "S-0309"),
    ("no_auto_fire", "NoAutoFire", Bool, "S-0309"),
    ("vhp_scan", "VHPScan", Text, "S-0309"),
    ("special_threat_value", "SpecialThreatValue", Dec, "S-0309"),
    ("insignificant", "Insignificant", Bool, "S-0309"),
    ("threat_posed", "ThreatPosed", Int, "S-0309"),
    ("airstrike_team", "AirstrikeTeam", Int, "S-0309, S-0519"),
    ("elite_airstrike_team", "EliteAirstrikeTeam", Int, "S-0519"),
    (
        "airstrike_team_type",
        "AirstrikeTeamType",
        Ref(Kind::Aircraft),
        "S-0519",
    ),
    (
        "elite_airstrike_team_type",
        "EliteAirstrikeTeamType",
        Ref(Kind::Aircraft),
        "S-0519",
    ),
    (
        "airstrike_recharge_time",
        "AirstrikeRechargeTime",
        Int,
        "S-0519",
    ),
    (
        "elite_airstrike_recharge_time",
        "EliteAirstrikeRechargeTime",
        Int,
        "S-0519",
    ),
    ("bomb_sight", "BombSight", Int, "S-0519"),
    ("drainable", "Drainable", Bool, "S-0308, S-0608"),
    ("spawns", "Spawns", Ref(Kind::Aircraft), "S-0506"),
    ("spawns_number", "SpawnsNumber", Int, "S-0506"),
    ("spawn_regen_rate", "SpawnRegenRate", Int, "S-0506"),
    ("spawn_reload_rate", "SpawnReloadRate", Int, "S-0506"),
];

/// Combat fields of infantry, vehicle and aircraft types (`UnitType`).
pub(super) const UNIT_COMBAT: &[Row] = &[
    ("mobile_fire", "MobileFire", Bool, "S-0301, S-0310"),
    ("ifv_mode", "IFVMode", Int, "S-0308"),
    (
        "open_transport_weapon",
        "OpenTransportWeapon",
        Int,
        "S-0308",
    ),
    ("deploy_fire_weapon", "DeployFireWeapon", Int, "S-0308"),
    ("crewed", "Crewed", Bool, "S-0312"),
    ("carries_crate", "CarriesCrate", Bool, "S-0312"),
    ("crate_goodie", "CrateGoodie", Bool, "S-0520"),
    ("crashable", "Crashable", Bool, "S-0312"),
    ("weight", "Weight", Dec, "S-0312"),
    (
        "suppression_threshold",
        "SuppressionThreshold",
        Int,
        "S-0315",
    ),
    ("missile_spawn", "MissileSpawn", Bool, "S-0311"),
    ("parasiteable", "Parasiteable", Bool, "S-0614"),
    ("reselect_if_limboed", "ReselectIfLimboed", Bool, "S-0614"),
];

/// Combat fields the Import mappings give for vehicle types only.
const VEHICLE_COMBAT: &[Row] = &[
    ("hunter_seeker", "HunterSeeker", Bool, "S-0301"),
    // spec: S-0301 Import mapping (TargetLaser: a boolean, absent no; only a vehicle shows the line)
    ("target_laser", "TargetLaser", Bool, "S-0301"),
    ("gunner", "Gunner", Bool, "S-0308"),
    ("deploy_to_fire", "DeployToFire", Bool, "S-0309, S-0310"),
];

/// Combat fields the Import mappings give for infantry types only.
const INFANTRY_COMBAT: &[Row] = &[
    ("jump_jet_turn", "JumpJetTurn", Bool, "S-0310"),
    (
        "occupy_weapon",
        "OccupyWeapon",
        Ref(Kind::Weapons),
        "S-0308",
    ),
    (
        "elite_occupy_weapon",
        "EliteOccupyWeapon",
        Ref(Kind::Weapons),
        "S-0308",
    ),
];

/// Combat fields of building types (`BuildingType`).
const BUILDING_COMBAT: &[Row] = &[
    ("crate_beneath", "CrateBeneath", Bool, "S-0312"),
    (
        "crate_beneath_is_money",
        "CrateBeneathIsMoney",
        Bool,
        "S-0312",
    ),
    ("cloak_generator", "CloakGenerator", Bool, "S-0516"),
    ("cloak_radius_in_cells", "CloakRadiusInCells", Int, "S-0516"),
    ("sensor_array", "SensorArray", Bool, "S-0516"),
    (
        "detect_disguise_range",
        "DetectDisguiseRange",
        Int,
        "S-0516",
    ),
    (
        "eligible_for_delay_kill",
        "EligibleForDelayKill",
        Bool,
        "S-0302",
    ),
    ("leave_rubble", "LeaveRubble", Bool, "S-0312"),
    ("overpowerable", "Overpowerable", Bool, "S-0518, S-0308"),
    ("helipad", "Helipad", Bool, "S-0312"),
    ("crewed", "Crewed", Bool, "S-0312, S-0408"),
    ("artillary", "Artillary", Bool, "S-0309"),
    ("max_number_occupants", "MaxNumberOccupants", Int, "S-0313"),
    (
        "infantry_gain_self_heal",
        "InfantryGainSelfHeal",
        Int,
        "S-0314",
    ),
    ("units_gain_self_heal", "UnitsGainSelfHeal", Int, "S-0314"),
    // spec: S-0313 claim 10 and Import mapping, S-0310 Import mapping (the voxel-turret and voxel-barrel flags: rules, the section named by the building type; art has no effect)
    (
        "turret_anim_is_voxel",
        "TurretAnimIsVoxel",
        Bool,
        "S-0313, S-0310",
    ),
    ("barrel_anim_is_voxel", "BarrelAnimIsVoxel", Bool, "S-0313"),
    // spec: S-0616 Import mapping (building type RefinerySmokeParticleSystem: a particle-system name; absent none)
    (
        "refinery_smoke_particle_system",
        "RefinerySmokeParticleSystem",
        Ref(Kind::ParticleSystems),
        "S-0616",
    ),
];

/// The refinery smoke point keys, in order (S-0616 Import mapping).
const REFINERY_SMOKE_POINTS: [&str; 4] = [
    "RefinerySmokeOffsetOne",
    "RefinerySmokeOffsetTwo",
    "RefinerySmokeOffsetThree",
    "RefinerySmokeOffsetFour",
]; // spec: S-0616 Import mapping (RefinerySmokeOffsetOne … RefinerySmokeOffsetFour)

/// Combat fields of overlay types.
const OVERLAY_COMBAT: &[Row] = &[
    ("chain_reaction", "ChainReaction", Bool, "S-0303"),
    ("explodes", "Explodes", Bool, "S-0303"),
];

/// Combat fields of terrain-object types.
const TERRAIN_COMBAT: &[Row] = &[
    ("immune", "Immune", Bool, "S-0302"),
    // spec: S-0214 Import mapping (object type LegalTarget and Bombable, for every object type)
    ("legal_target", "LegalTarget", Bool, "S-0214"),
    ("bombable", "Bombable", Bool, "S-0214"),
];

/// The game-wide tree strength's absent-key default (S-0102 claim 23 and
/// Import mapping).
const TREE_STRENGTH_DEFAULT: i32 = 25; // spec: S-0102 Constants (game-wide tree strength default 25)

/// The strength value that unsets a terrain type's strength (S-0102 claim
/// 23).
const UNSET_STRENGTH: i32 = -1; // spec: S-0102 claim 23 (writing −1 unsets the strength)

/// Combat fields of weapons.
const WEAPON_COMBAT: &[Row] = &[
    (
        "infinite_mind_control",
        "InfiniteMindControl",
        Bool,
        "S-0601",
    ),
    ("rof", "ROF", Int, "S-0301"),
    ("burst", "Burst", Int, "S-0301"),
    // The imported speed field as read (−1 is a "not given" value, S-0306 Import mapping).
    ("speed", "Speed", Int, "S-0306"),
    // The lepton form of S-0307 claim 1: the decimal text.
    ("range", "Range", Dec, "S-0307"),
    ("minimum_range", "MinimumRange", Dec, "S-0307"),
    ("cell_rangefinding", "CellRangefinding", Bool, "S-0307"),
    ("is_sonic", "IsSonic", Bool, "S-0301, S-0302"),
    ("use_spark_particles", "UseSparkParticles", Bool, "S-0301"),
    (
        "use_fire_particles",
        "UseFireParticles",
        Bool,
        "S-0301, S-0302",
    ),
    ("is_railgun", "IsRailgun", Bool, "S-0301"),
    // spec: S-0616 Import mapping (weapon AttachedParticleSystem: the effect weapon's emitter type, a particle-system name; absent none)
    (
        "attached_particle_system",
        "AttachedParticleSystem",
        Ref(Kind::ParticleSystems),
        "S-0616",
    ),
    ("is_mag_beam", "IsMagBeam", Bool, "S-0301"),
    // spec: S-0316 Import mapping (IsElectricBolt, IsAlternateColor, IsLaser, IsRadBeam: booleans, absent no)
    ("is_electric_bolt", "IsElectricBolt", Bool, "S-0316"),
    ("is_alternate_color", "IsAlternateColor", Bool, "S-0316"),
    ("is_laser", "IsLaser", Bool, "S-0316, S-0514"),
    ("is_rad_beam", "IsRadBeam", Bool, "S-0316, S-0514"),
    ("spawner", "Spawner", Bool, "S-0301"),
    ("camera", "Camera", Bool, "S-0501"),
    ("disk_laser", "DiskLaser", Bool, "S-0301, S-0615"),
    // spec: S-0615 Import mapping (LaserDuration: integer ticks, absent 10; IsHouseColor: a boolean, absent no)
    ("laser_duration", "LaserDuration", Int, "S-0615"),
    ("is_house_color", "IsHouseColor", Bool, "S-0615"),
    ("decloak_to_fire", "DecloakToFire", Bool, "S-0301"),
    (
        "disguise_fake_blink_time",
        "DisguiseFakeBlinkTime",
        Int,
        "S-0516",
    ),
    ("lobber", "Lobber", Bool, "S-0306"),
    ("never_use", "NeverUse", Bool, "S-0308"),
    ("drain_weapon", "DrainWeapon", Bool, "S-0308, S-0608"),
    ("fire_while_moving", "FireWhileMoving", Bool, "S-0310"),
    ("suicide", "Suicide", Bool, "S-0312"),
    (
        "mig_attack_cursor",
        "MigAttackCursor",
        Bool,
        "S-0519, S-0214",
    ),
    ("supress", "Supress", Bool, "S-0309"),
    ("fire_once", "FireOnce", Bool, "S-0309"),
    ("reveal_on_fire", "RevealOnFire", Bool, "S-0409"),
    ("limbo_launch", "LimboLaunch", Bool, "S-0614"),
    // spec: S-0301 Import mapping (Anim: the animation list, its entries in the order written; absent empty)
    ("anim", "Anim", Refs(Kind::Animations), "S-0301, S-0614"),
    // spec: S-0508 Import mapping (OccupantAnim: an animation name of the weapon, none when absent), S-0301 Import mapping (the occupant animation of claim 20 b)
    (
        "occupant_anim",
        "OccupantAnim",
        Ref(Kind::Animations),
        "S-0301, S-0508",
    ),
    // spec: S-0301 Import mapping (OpenToppedAnim and AssaultAnim: animation type names, none when absent)
    (
        "open_topped_anim",
        "OpenToppedAnim",
        Ref(Kind::Animations),
        "S-0301",
    ),
    (
        "assault_anim",
        "AssaultAnim",
        Ref(Kind::Animations),
        "S-0301",
    ),
];

/// Combat fields of projectiles.
const PROJECTILE_COMBAT: &[Row] = &[
    ("aa", "AA", Bool, "S-0306"),
    ("arcing", "Arcing", Bool, "S-0306"),
    ("vertical", "Vertical", Bool, "S-0306"),
    ("inviso", "Inviso", Bool, "S-0306"),
    ("inaccurate", "Inaccurate", Bool, "S-0306"),
    ("flak_scatter", "FlakScatter", Bool, "S-0306"),
    ("dropping", "Dropping", Bool, "S-0306"),
    ("airburst", "Airburst", Bool, "S-0306"),
    ("very_high", "VeryHigh", Bool, "S-0306"),
    ("level", "Level", Bool, "S-0306"),
    ("floater", "Floater", Bool, "S-0306"),
    ("subject_to_cliffs", "SubjectToCliffs", Bool, "S-0306"),
    ("subject_to_walls", "SubjectToWalls", Bool, "S-0306"),
    ("subject_to_elevation", "SubjectToElevation", Bool, "S-0307"),
    ("ranged", "Ranged", Bool, "S-0306"),
    ("degenerates", "Degenerates", Bool, "S-0306"),
    ("scalable", "Scalable", Bool, "S-0306"),
    ("acceleration", "Acceleration", Int, "S-0306"),
    ("course_lock_duration", "CourseLockDuration", Int, "S-0306"),
    ("arm", "Arm", Int, "S-0306"),
    ("detonation_altitude", "DetonationAltitude", Int, "S-0306"),
    ("cluster", "Cluster", Int, "S-0306"),
    (
        "airburst_weapon",
        "AirburstWeapon",
        Ref(Kind::Weapons),
        "S-0306",
    ),
    (
        "shrapnel_weapon",
        "ShrapnelWeapon",
        Ref(Kind::Weapons),
        "S-0318",
    ),
    ("shrapnel_count", "ShrapnelCount", Int, "S-0318"),
];

/// Combat fields of warheads (`Verses` has its own reading).
const WARHEAD_COMBAT: &[Row] = &[
    ("makes_disguise", "MakesDisguise", Bool, "S-0516"),
    ("nuke_maker", "NukeMaker", Bool, "S-0513"),
    ("bright", "Bright", Bool, "S-0524"),
    ("combat_light_size", "CombatLightSize", Dec, "S-0524"),
    ("cell_spread", "CellSpread", Dec, "S-0302, S-0303"),
    ("percent_at_max", "PercentAtMax", Dec, "S-0302, S-0303"),
    ("prone_damage", "ProneDamage", Dec, "S-0302"),
    ("penetrates_bunker", "PenetratesBunker", Bool, "S-0302"),
    ("radiation", "Radiation", Bool, "S-0302"),
    ("psychic_damage", "PsychicDamage", Bool, "S-0302, S-0610"),
    ("poison", "Poison", Bool, "S-0302"),
    ("affects_allies", "AffectsAllies", Bool, "S-0302, S-0303"),
    ("psychedelic", "Psychedelic", Bool, "S-0302, S-0610"),
    ("sparky", "Sparky", Bool, "S-0302"),
    ("causes_delay_kill", "CausesDelayKill", Bool, "S-0302"),
    ("delay_kill_frames", "DelayKillFrames", Int, "S-0302"),
    ("delay_kill_at_max", "DelayKillAtMax", Dec, "S-0302"),
    (
        "wall_absolute_destroyer",
        "WallAbsoluteDestroyer",
        Bool,
        "S-0303",
    ),
    ("tiberium", "Tiberium", Bool, "S-0303"),
    ("rocker", "Rocker", Bool, "S-0303"),
    ("particle", "Particle", Ref(Kind::ParticleSystems), "S-0303"),
    ("conventional", "Conventional", Bool, "S-0303"),
    ("em_effect", "EMEffect", Bool, "S-0303, S-0306"),
    ("anim_list", "AnimList", Refs(Kind::Animations), "S-0303"),
    ("max_debris", "MaxDebris", Int, "S-0303"),
    ("min_debris", "MinDebris", Int, "S-0303"),
    (
        "debris_types",
        "DebrisTypes",
        Refs(Kind::VoxelAnimations),
        "S-0303",
    ),
    ("debris_maximums", "DebrisMaximums", Maxima, "S-0303"),
    ("airstrike", "Airstrike", Bool, "S-0308"),
    ("is_locomotor", "IsLocomotor", Bool, "S-0308"),
    ("parasite", "Parasite", Bool, "S-0614"),
    ("paralyzes", "Paralyzes", Int, "S-0614"),
    ("culling", "Culling", Bool, "S-0614"),
    ("temporal", "Temporal", Bool, "S-0301, S-0514"),
    // spec: S-0514 Import mapping (the flags of claim 20's selection; each a boolean, absent: no)
    ("mind_control", "MindControl", Bool, "S-0514"),
    ("direct_rocker", "directrocker", Bool, "S-0514"),
];

/// Combat fields of countries.
const COUNTRY_COMBAT: &[Row] = &[
    ("firepower", "Firepower", Dec, "S-0302"),
    ("rof", "ROF", Dec, "S-0301"),
    ("armor_infantry_mult", "ArmorInfantryMult", Dec, "S-0302"),
    ("armor_units_mult", "ArmorUnitsMult", Dec, "S-0302"),
    ("armor_aircraft_mult", "ArmorAircraftMult", Dec, "S-0302"),
    ("armor_buildings_mult", "ArmorBuildingsMult", Dec, "S-0302"),
    ("armor_defenses_mult", "ArmorDefensesMult", Dec, "S-0302"),
];

/// The general settings section.
const GENERAL: &[u8] = b"General"; // spec: S-0004 Import mapping (the general settings section)
/// The audio-visual settings section.
const AUDIO_VISUAL: &[u8] = b"AudioVisual"; // spec: S-0306 Import mapping (`[AudioVisual]`)
/// The wall model section.
const WALL_MODEL: &[u8] = b"WallModel"; // spec: S-0306 Import mapping (`[WallModel]`)
/// The elevation model section.
const ELEVATION_MODEL: &[u8] = b"ElevationModel"; // spec: S-0307 Import mapping (`[ElevationModel]`)
/// The AI intelligence levels section.
const IQ: &[u8] = b"IQ"; // spec: S-0314 Import mapping (`[IQ]`)
const AI: &[u8] = b"AI"; // spec: S-0314 Import mapping (`[AI]`)
/// The radiation settings section.
const RADIATION: &[u8] = b"Radiation"; // spec: S-0525 Import mapping (`[Radiation]`)

/// Game-wide combat fields: our name, section, key, form, spec.
const GENERAL_COMBAT: &[(&str, &[u8], &str, Form, &str)] = &[
    ("cloaking_stages", GENERAL, "CloakingStages", Int, "S-0516"),
    ("cloak_delay", GENERAL, "CloakDelay", Dec, "S-0516"),
    ("cloak_sound", AUDIO_VISUAL, "CloakSound", Text, "S-0516"),
    (
        "allied_disguise",
        GENERAL,
        "AlliedDisguise",
        Ref(Kind::Infantry),
        "S-0516",
    ),
    (
        "soviet_disguise",
        GENERAL,
        "SovietDisguise",
        Ref(Kind::Infantry),
        "S-0516",
    ),
    (
        "third_disguise",
        GENERAL,
        "ThirdDisguise",
        Ref(Kind::Infantry),
        "S-0516",
    ),
    (
        "default_mirage_disguises",
        GENERAL,
        "DefaultMirageDisguises",
        Refs(Kind::TerrainObjects),
        "S-0516",
    ),
    (
        "infantry_blink_disguise_time",
        GENERAL,
        "InfantryBlinkDisguiseTime",
        Int,
        "S-0516",
    ),
    (
        "attack_cursor_on_disguise",
        GENERAL,
        "AttackCursorOnDisguise",
        Bool,
        "S-0516",
    ),
    (
        "v3_rocket_type",
        GENERAL,
        "V3RocketType",
        Ref(Kind::Aircraft),
        "S-0506",
    ),
    (
        "v3_rocket_pause_frames",
        GENERAL,
        "V3RocketPauseFrames",
        Int,
        "S-0506",
    ),
    (
        "v3_rocket_tilt_frames",
        GENERAL,
        "V3RocketTiltFrames",
        Int,
        "S-0506",
    ),
    (
        "v3_rocket_pitch_initial",
        GENERAL,
        "V3RocketPitchInitial",
        Dec,
        "S-0506",
    ),
    (
        "v3_rocket_pitch_final",
        GENERAL,
        "V3RocketPitchFinal",
        Dec,
        "S-0506",
    ),
    (
        "v3_rocket_turn_rate",
        GENERAL,
        "V3RocketTurnRate",
        Dec,
        "S-0506",
    ),
    (
        "v3_rocket_raise_rate",
        GENERAL,
        "V3RocketRaiseRate",
        Dec,
        "S-0506",
    ),
    (
        "v3_rocket_acceleration",
        GENERAL,
        "V3RocketAcceleration",
        Dec,
        "S-0506",
    ),
    (
        "v3_rocket_altitude",
        GENERAL,
        "V3RocketAltitude",
        Int,
        "S-0506",
    ),
    ("v3_rocket_damage", GENERAL, "V3RocketDamage", Int, "S-0506"),
    (
        "v3_rocket_elite_damage",
        GENERAL,
        "V3RocketEliteDamage",
        Int,
        "S-0506",
    ),
    (
        "v3_rocket_body_length",
        GENERAL,
        "V3RocketBodyLength",
        Int,
        "S-0506",
    ),
    (
        "v3_rocket_lazy_curve",
        GENERAL,
        "V3RocketLazyCurve",
        Bool,
        "S-0506",
    ),
    (
        "d_misl_type",
        GENERAL,
        "DMislType",
        Ref(Kind::Aircraft),
        "S-0506",
    ),
    (
        "d_misl_pause_frames",
        GENERAL,
        "DMislPauseFrames",
        Int,
        "S-0506",
    ),
    (
        "d_misl_tilt_frames",
        GENERAL,
        "DMislTiltFrames",
        Int,
        "S-0506",
    ),
    (
        "d_misl_pitch_initial",
        GENERAL,
        "DMislPitchInitial",
        Dec,
        "S-0506",
    ),
    (
        "d_misl_pitch_final",
        GENERAL,
        "DMislPitchFinal",
        Dec,
        "S-0506",
    ),
    ("d_misl_turn_rate", GENERAL, "DMislTurnRate", Dec, "S-0506"),
    (
        "d_misl_raise_rate",
        GENERAL,
        "DMislRaiseRate",
        Dec,
        "S-0506",
    ),
    (
        "d_misl_acceleration",
        GENERAL,
        "DMislAcceleration",
        Dec,
        "S-0506",
    ),
    ("d_misl_altitude", GENERAL, "DMislAltitude", Int, "S-0506"),
    ("d_misl_damage", GENERAL, "DMislDamage", Int, "S-0506"),
    (
        "d_misl_elite_damage",
        GENERAL,
        "DMislEliteDamage",
        Int,
        "S-0506",
    ),
    (
        "d_misl_body_length",
        GENERAL,
        "DMislBodyLength",
        Int,
        "S-0506",
    ),
    (
        "d_misl_lazy_curve",
        GENERAL,
        "DMislLazyCurve",
        Bool,
        "S-0506",
    ),
    (
        "c_misl_type",
        GENERAL,
        "CMislType",
        Ref(Kind::Aircraft),
        "S-0506",
    ),
    (
        "c_misl_pause_frames",
        GENERAL,
        "CMislPauseFrames",
        Int,
        "S-0506",
    ),
    (
        "c_misl_tilt_frames",
        GENERAL,
        "CMislTiltFrames",
        Int,
        "S-0506",
    ),
    (
        "c_misl_pitch_initial",
        GENERAL,
        "CMislPitchInitial",
        Dec,
        "S-0506",
    ),
    (
        "c_misl_pitch_final",
        GENERAL,
        "CMislPitchFinal",
        Dec,
        "S-0506",
    ),
    ("c_misl_turn_rate", GENERAL, "CMislTurnRate", Dec, "S-0506"),
    (
        "c_misl_raise_rate",
        GENERAL,
        "CMislRaiseRate",
        Dec,
        "S-0506",
    ),
    (
        "c_misl_acceleration",
        GENERAL,
        "CMislAcceleration",
        Dec,
        "S-0506",
    ),
    ("c_misl_altitude", GENERAL, "CMislAltitude", Int, "S-0506"),
    ("c_misl_damage", GENERAL, "CMislDamage", Int, "S-0506"),
    (
        "c_misl_elite_damage",
        GENERAL,
        "CMislEliteDamage",
        Int,
        "S-0506",
    ),
    (
        "c_misl_body_length",
        GENERAL,
        "CMislBodyLength",
        Int,
        "S-0506",
    ),
    (
        "c_misl_lazy_curve",
        GENERAL,
        "CMislLazyCurve",
        Bool,
        "S-0506",
    ),
    (
        "v3_warhead",
        COMBAT_DAMAGE,
        "V3Warhead",
        Ref(Kind::Warheads),
        "S-0506",
    ),
    (
        "v3_elite_warhead",
        COMBAT_DAMAGE,
        "V3EliteWarhead",
        Ref(Kind::Warheads),
        "S-0506",
    ),
    (
        "d_misl_warhead",
        COMBAT_DAMAGE,
        "DMislWarhead",
        Ref(Kind::Warheads),
        "S-0506",
    ),
    (
        "d_misl_elite_warhead",
        COMBAT_DAMAGE,
        "DMislEliteWarhead",
        Ref(Kind::Warheads),
        "S-0506",
    ),
    (
        "c_misl_warhead",
        COMBAT_DAMAGE,
        "CMislWarhead",
        Ref(Kind::Warheads),
        "S-0506",
    ),
    (
        "c_misl_elite_warhead",
        COMBAT_DAMAGE,
        "CMislEliteWarhead",
        Ref(Kind::Warheads),
        "S-0506",
    ),
    ("max_damage", COMBAT_DAMAGE, "MaxDamage", Int, "S-0302"),
    (
        "occupy_rof_multiplier",
        COMBAT_DAMAGE,
        "OccupyROFMultiplier",
        Dec,
        "S-0301",
    ),
    (
        "bunker_rof_multiplier",
        COMBAT_DAMAGE,
        "BunkerROFMultiplier",
        Dec,
        "S-0301",
    ),
    (
        "occupy_damage_multiplier",
        COMBAT_DAMAGE,
        "OccupyDamageMultiplier",
        Dec,
        "S-0302",
    ),
    (
        "bunker_damage_multiplier",
        COMBAT_DAMAGE,
        "BunkerDamageMultiplier",
        Dec,
        "S-0302",
    ),
    (
        "open_topped_damage_multiplier",
        COMBAT_DAMAGE,
        "OpenToppedDamageMultiplier",
        Dec,
        "S-0302",
    ),
    // spec: S-0109 Import mapping ([CombatDamage] C4Warhead: also the warhead of the built-in invisibility-animation type's damage over time, absent none) and S-0501 Import mapping (r134: also the demolition warhead of claim 7's landing-check hit where no free cell is found)
    (
        "c4_warhead",
        COMBAT_DAMAGE,
        "C4Warhead",
        Ref(Kind::Warheads),
        "S-0302, S-0303, S-0109, S-0501",
    ),
    // spec: S-0109 Import mapping ([CombatDamage] FlameDamage2: the warhead of an animation's damage over time, absent none)
    (
        "flame_damage2",
        COMBAT_DAMAGE,
        "FlameDamage2",
        Ref(Kind::Warheads),
        "S-0109, S-0615",
    ),
    (
        "crush_warhead",
        COMBAT_DAMAGE,
        "CrushWarhead",
        Ref(Kind::Warheads),
        "S-0303",
    ),
    (
        "ion_cannon_warhead",
        COMBAT_DAMAGE,
        "IonCannonWarhead",
        Ref(Kind::Warheads),
        "S-0303",
    ),
    (
        "bridge_strength",
        COMBAT_DAMAGE,
        "BridgeStrength",
        Int,
        "S-0303",
    ),
    (
        "ammo_crate_damage",
        COMBAT_DAMAGE,
        "AmmoCrateDamage",
        Int,
        "S-0303",
    ),
    // spec: S-0306 Import mapping (railgundamageradius, spelled so: the railgun hit distance, integer leptons, not a cells-form value)
    (
        "railgun_damage_radius",
        COMBAT_DAMAGE,
        "railgundamageradius",
        Int,
        "S-0306",
    ),
    // spec: S-0306 Import mapping (CollapseChance: integer percentage points, compared directly with R(0, 99))
    (
        "collapse_chance",
        COMBAT_DAMAGE,
        "CollapseChance",
        Int,
        "S-0306",
    ),
    (
        "default_spark_system",
        COMBAT_DAMAGE,
        "DefaultSparkSystem",
        Ref(Kind::ParticleSystems),
        "S-0614",
    ),
    (
        "mind_cleared_sound",
        AUDIO_VISUAL,
        "MindClearedSound",
        Text,
        "S-0601",
    ),
    (
        "yuri_mind_control_sound",
        AUDIO_VISUAL,
        "YuriMindControlSound",
        Text,
        "S-0601",
    ),
    (
        "master_mind_overload_death_sound",
        AUDIO_VISUAL,
        "MasterMindOverloadDeathSound",
        Text,
        "S-0601",
    ),
    (
        "controlled_animation_type",
        COMBAT_DAMAGE,
        "ControlledAnimationType",
        Ref(Kind::Animations),
        "S-0601",
    ),
    (
        "perma_controlled_animation_type",
        COMBAT_DAMAGE,
        "PermaControlledAnimationType",
        Ref(Kind::Animations),
        "S-0601",
    ),
    (
        "mind_control_attack_line_frames",
        COMBAT_DAMAGE,
        "MindControlAttackLineFrames",
        Int,
        "S-0601",
    ),
    (
        "overload_count",
        COMBAT_DAMAGE,
        "OverloadCount",
        Maxima,
        "S-0601",
    ),
    (
        "overload_damage",
        COMBAT_DAMAGE,
        "OverloadDamage",
        Maxima,
        "S-0601",
    ),
    (
        "overload_frames",
        COMBAT_DAMAGE,
        "OverloadFrames",
        Maxima,
        "S-0601",
    ),
    (
        "ai_capture_normal",
        GENERAL,
        "AICaptureNormal",
        Maxima,
        "S-0601",
    ),
    (
        "ai_capture_wounded",
        GENERAL,
        "AICaptureWounded",
        Maxima,
        "S-0601",
    ),
    (
        "ai_capture_low_power",
        GENERAL,
        "AICaptureLowPower",
        Maxima,
        "S-0601",
    ),
    (
        "ai_capture_low_money",
        GENERAL,
        "AICaptureLowMoney",
        Maxima,
        "S-0601",
    ),
    (
        "ai_capture_low_money_mark",
        GENERAL,
        "AICaptureLowMoneyMark",
        Int,
        "S-0601",
    ),
    (
        "ai_capture_wounded_mark",
        GENERAL,
        "AICaptureWoundedMark",
        Dec,
        "S-0601",
    ),
    (
        "wake",
        GENERAL,
        "Wake",
        Ref(Kind::Animations),
        "S-0614, S-0207, S-0312, S-0504, S-0503",
    ),
    (
        "splash_list",
        COMBAT_DAMAGE,
        "SplashList",
        Refs(Kind::Animations),
        "S-0303",
    ),
    (
        "ballistic_scatter",
        COMBAT_DAMAGE,
        "BallisticScatter",
        Dec,
        "S-0306",
    ),
    (
        "occupy_weapon_range",
        COMBAT_DAMAGE,
        "OccupyWeaponRange",
        Int,
        "S-0307",
    ),
    (
        "bunker_weapon_range_bonus",
        COMBAT_DAMAGE,
        "BunkerWeaponRangeBonus",
        Int,
        "S-0307",
    ),
    (
        "open_topped_range_bonus",
        COMBAT_DAMAGE,
        "OpenToppedRangeBonus",
        Int,
        "S-0307",
    ),
    ("fire_supress", COMBAT_DAMAGE, "FireSupress", Dec, "S-0309"),
    (
        "death_weapon",
        COMBAT_DAMAGE,
        "DeathWeapon",
        Ref(Kind::Weapons),
        "S-0312",
    ),
    ("veteran_rof", GENERAL, "VeteranROF", Dec, "S-0301"),
    ("veteran_combat", GENERAL, "VeteranCombat", Dec, "S-0302"),
    ("veteran_armor", GENERAL, "VeteranArmor", Dec, "S-0302"),
    ("veteran_sight", GENERAL, "VeteranSight", Dec, "S-0311"),
    ("veteran_ratio", GENERAL, "VeteranRatio", Dec, "S-0311"),
    ("veteran_cap", GENERAL, "VeteranCap", Dec, "S-0311"),
    (
        "damage_fire_types",
        GENERAL,
        "DamageFireTypes",
        Refs(Kind::Animations),
        "S-0302",
    ),
    (
        "infantry_explode",
        GENERAL,
        "InfantryExplode",
        Ref(Kind::Animations),
        "S-0302, S-0615",
    ),
    // spec: S-0107 Import mapping ([General] DeadBodies: the general corpse list, animation names, absent empty)
    (
        "dead_bodies",
        GENERAL,
        "DeadBodies",
        Refs(Kind::Animations),
        "S-0107",
    ),
    // spec: S-0615 Import mapping ([General] FlamingInfantry, InfantryHeadPop, InfantryNuked, InfantryBrute: animation type names, absent none; [AudioVisual] DiskLaserChargeUp: a sound name, absent silent)
    (
        "flaming_infantry",
        GENERAL,
        "FlamingInfantry",
        Ref(Kind::Animations),
        "S-0615",
    ),
    (
        "infantry_head_pop",
        GENERAL,
        "InfantryHeadPop",
        Ref(Kind::Animations),
        "S-0615",
    ),
    (
        "infantry_nuked",
        GENERAL,
        "InfantryNuked",
        Ref(Kind::Animations),
        "S-0615",
    ),
    (
        "infantry_brute",
        GENERAL,
        "InfantryBrute",
        Ref(Kind::Animations),
        "S-0615",
    ),
    (
        "disk_laser_charge_up",
        AUDIO_VISUAL,
        "DiskLaserChargeUp",
        Text,
        "S-0615",
    ),
    ("missile_rot_var", GENERAL, "MissileROTVar", Dec, "S-0306"),
    (
        "missile_safety_altitude",
        GENERAL,
        "MissileSafetyAltitude",
        Int,
        "S-0306",
    ),
    (
        "barrel_explode",
        GENERAL,
        "BarrelExplode",
        Ref(Kind::Animations),
        "S-0303",
    ),
    (
        "barrel_debris",
        GENERAL,
        "BarrelDebris",
        Refs(Kind::VoxelAnimations),
        "S-0303",
    ),
    (
        "barrel_particle",
        GENERAL,
        "BarrelParticle",
        Ref(Kind::ParticleSystems),
        "S-0303",
    ),
    // spec: S-0616 Import mapping ([General] RefinerySmokeFrames: an integer, ticks; absent 25)
    (
        "refinery_smoke_frames",
        GENERAL,
        "RefinerySmokeFrames",
        Int,
        "S-0616",
    ),
    // spec: S-0616 Import mapping ([General] ConditionRedSparkingProbability and ConditionYellowSparkingProbability: decimals held as doubles; absent 0.02 and 0.01)
    (
        "condition_red_sparking_probability",
        GENERAL,
        "ConditionRedSparkingProbability",
        Dec,
        "S-0616",
    ),
    (
        "condition_yellow_sparking_probability",
        GENERAL,
        "ConditionYellowSparkingProbability",
        Dec,
        "S-0616",
    ),
    (
        "metallic_debris",
        GENERAL,
        "MetallicDebris",
        Refs(Kind::Animations),
        "S-0303, S-0312",
    ),
    (
        "bridge_explosions",
        GENERAL,
        "BridgeExplosions",
        Refs(Kind::Animations),
        "S-0511",
    ),
    (
        "no_parachute_max_fall_rate",
        GENERAL,
        "NoParachuteMaxFallRate",
        Int,
        "S-0511",
    ),
    (
        "parachute_max_fall_rate",
        GENERAL,
        "ParachuteMaxFallRate",
        Int,
        "S-0511",
    ),
    (
        "repair_bridge_sound",
        AUDIO_VISUAL,
        "RepairBridgeSound",
        Text,
        "S-0511",
    ),
    (
        "weapon_nullify_anim",
        GENERAL,
        "WeaponNullifyAnim",
        Ref(Kind::Animations),
        "S-0303",
    ),
    (
        "lightning_warhead",
        GENERAL,
        "LightningWarhead",
        Ref(Kind::Warheads),
        "S-0303",
    ),
    (
        "weather_con_bolt_explosion",
        GENERAL,
        "WeatherConBoltExplosion",
        Ref(Kind::Animations),
        "S-0303",
    ),
    ("crew_escape", GENERAL, "CrewEscape", Dec, "S-0312"),
    (
        "allied_survivor_divisor",
        GENERAL,
        "AlliedSurvivorDivisor",
        Int,
        "S-0312",
    ),
    (
        "soviet_survivor_divisor",
        GENERAL,
        "SovietSurvivorDivisor",
        Int,
        "S-0312",
    ),
    (
        "third_survivor_divisor",
        GENERAL,
        "ThirdSurvivorDivisor",
        Int,
        "S-0312",
    ),
    (
        "ship_sinking_weight",
        GENERAL,
        "ShipSinkingWeight",
        Dec,
        "S-0312",
    ),
    ("repair_rate", GENERAL, "RepairRate", Dec, "S-0314"),
    ("u_repair_rate", GENERAL, "URepairRate", Dec, "S-0314"),
    ("i_repair_rate", GENERAL, "IRepairRate", Dec, "S-0314"),
    ("repair_step", GENERAL, "RepairStep", Int, "S-0314"),
    ("i_repair_step", GENERAL, "IRepairStep", Int, "S-0314"),
    ("repair_percent", GENERAL, "RepairPercent", Dec, "S-0314"),
    (
        "self_heal_infantry_frames",
        GENERAL,
        "SelfHealInfantryFrames",
        Int,
        "S-0314",
    ),
    (
        "self_heal_unit_frames",
        GENERAL,
        "SelfHealUnitFrames",
        Int,
        "S-0314",
    ),
    (
        "self_heal_infantry_amount",
        GENERAL,
        "SelfHealInfantryAmount",
        Int,
        "S-0314",
    ),
    (
        "self_heal_unit_amount",
        GENERAL,
        "SelfHealUnitAmount",
        Int,
        "S-0314",
    ),
    (
        "threat_per_occupant",
        GENERAL,
        "ThreatPerOccupant",
        Int,
        "S-0309",
    ),
    (
        "enemy_house_threat_bonus",
        GENERAL,
        "EnemyHouseThreatBonus",
        Dec,
        "S-0309",
    ),
    (
        "gravity",
        AUDIO_VISUAL,
        "Gravity",
        Int,
        "S-0306, S-0503, S-0613",
    ),
    // spec: S-0613 Import mapping ([General] InfantryVirus an animation name; WindDirection an integer table index, absent −1)
    (
        "infantry_virus",
        GENERAL,
        "InfantryVirus",
        Ref(Kind::Animations),
        "S-0613",
    ),
    ("wind_direction", GENERAL, "WindDirection", Int, "S-0613"),
    (
        "allied_wall_transparency",
        WALL_MODEL,
        "AlliedWallTransparency",
        Bool,
        "S-0306",
    ),
    (
        "elevation_increment",
        ELEVATION_MODEL,
        "ElevationIncrement",
        Int,
        "S-0307",
    ),
    (
        "elevation_increment_bonus",
        ELEVATION_MODEL,
        "ElevationIncrementBonus",
        Dec,
        "S-0307",
    ),
    (
        "elevation_bonus_cap",
        ELEVATION_MODEL,
        "ElevationBonusCap",
        Dec,
        "S-0307",
    ),
    ("iq_repair_sell", IQ, "RepairSell", Int, "S-0314"),
    ("credit_reserve", AI, "CreditReserve", Int, "S-0314"),
    ("iq_sell_back", IQ, "SellBack", Int, "S-0314"),
    // spec: S-0503 Import mapping (HoverHeight an integer; the other hover keys decimals, read as singles)
    ("hover_height", GENERAL, "HoverHeight", Int, "S-0503"),
    ("hover_bob", GENERAL, "HoverBob", Dec, "S-0503"),
    ("hover_boost", GENERAL, "HoverBoost", Dec, "S-0503"),
    (
        "hover_acceleration",
        GENERAL,
        "HoverAcceleration",
        Dec,
        "S-0503",
    ),
    ("hover_brake", GENERAL, "HoverBrake", Dec, "S-0503"),
    ("hover_dampen", GENERAL, "HoverDampen", Dec, "S-0503"),
    // spec: S-0519 Import mapping (the bomb and charge settings of the combat-damage section)
    (
        "ivan_warhead",
        COMBAT_DAMAGE,
        "IvanWarhead",
        Ref(Kind::Warheads),
        "S-0519",
    ),
    ("ivan_damage", COMBAT_DAMAGE, "IvanDamage", Int, "S-0519"),
    (
        "ivan_timed_delay",
        COMBAT_DAMAGE,
        "IvanTimedDelay",
        Int,
        "S-0519",
    ),
    (
        "ivan_icon_flicker_rate",
        COMBAT_DAMAGE,
        "IvanIconFlickerRate",
        Int,
        "S-0519",
    ),
    (
        "can_detonate_time_bomb",
        COMBAT_DAMAGE,
        "CanDetonateTimeBomb",
        Bool,
        "S-0519, S-0214",
    ),
    (
        "can_detonate_death_bomb",
        COMBAT_DAMAGE,
        "CanDetonateDeathBomb",
        Bool,
        "S-0519",
    ),
    ("c4_delay", COMBAT_DAMAGE, "C4Delay", Dec, "S-0519"),
    // spec: S-0607 Import mapping (FallingDamageMultiplier a decimal held as a single; CurrentStrengthDamage a boolean)
    (
        "falling_damage_multiplier",
        COMBAT_DAMAGE,
        "FallingDamageMultiplier",
        Dec,
        "S-0607",
    ),
    (
        "current_strength_damage",
        COMBAT_DAMAGE,
        "CurrentStrengthDamage",
        Bool,
        "S-0607",
    ),
    // spec: S-0518 Import mapping (PrismType a building type name; the other prism keys integers; the modifier is read apart, F4)
    (
        "prism_type",
        GENERAL,
        "PrismType",
        Ref(Kind::Buildings),
        "S-0518",
    ),
    (
        "prism_support_max",
        GENERAL,
        "PrismSupportMax",
        Int,
        "S-0518",
    ),
    (
        "prism_support_delay",
        GENERAL,
        "PrismSupportDelay",
        Int,
        "S-0518",
    ),
    (
        "prism_support_duration",
        GENERAL,
        "PrismSupportDuration",
        Int,
        "S-0518",
    ),
    // spec: S-0514 Import mapping (WarpAway and ChronoSparkle1 animation names; OpenToppedWarpDistance an integer of cells)
    (
        "warp_away",
        GENERAL,
        "WarpAway",
        Ref(Kind::Animations),
        "S-0514",
    ),
    (
        "chrono_sparkle",
        GENERAL,
        "ChronoSparkle1",
        Ref(Kind::Animations),
        "S-0514",
    ),
    (
        "open_topped_warp_distance",
        COMBAT_DAMAGE,
        "OpenToppedWarpDistance",
        Int,
        "S-0514",
    ),
    // spec: S-0608 Import mapping (DrainMoneyFrameDelay and DrainMoneyAmount integers of ticks and credits; DrainAnimationType an animation name)
    (
        "drain_money_frame_delay",
        COMBAT_DAMAGE,
        "DrainMoneyFrameDelay",
        Int,
        "S-0608",
    ),
    (
        "drain_money_amount",
        COMBAT_DAMAGE,
        "DrainMoneyAmount",
        Int,
        "S-0608",
    ),
    (
        "drain_animation_type",
        COMBAT_DAMAGE,
        "DrainAnimationType",
        Ref(Kind::Animations),
        "S-0608",
    ),
    // spec: S-0525 Import mapping ([Radiation]: integers, three decimals held as doubles, the warhead name; RadColor is read apart)
    (
        "rad_duration_multiple",
        RADIATION,
        "RadDurationMultiple",
        Int,
        "S-0525",
    ),
    (
        "rad_application_delay",
        RADIATION,
        "RadApplicationDelay",
        Int,
        "S-0525",
    ),
    ("rad_level_max", RADIATION, "RadLevelMax", Int, "S-0525"),
    ("rad_level_delay", RADIATION, "RadLevelDelay", Int, "S-0525"),
    ("rad_light_delay", RADIATION, "RadLightDelay", Int, "S-0525"),
    // spec: S-0104 Import mapping (AmbientChangeRate decimal game minutes, absent 0.2; AmbientChangeStep decimal ambient factor, absent 0.1)
    (
        "ambient_change_rate",
        AUDIO_VISUAL,
        "AmbientChangeRate",
        Dec,
        "S-0104",
    ),
    (
        "ambient_change_step",
        AUDIO_VISUAL,
        "AmbientChangeStep",
        Dec,
        "S-0104",
    ),
    (
        "rad_level_factor",
        RADIATION,
        "RadLevelFactor",
        Dec,
        "S-0525",
    ),
    (
        "rad_light_factor",
        RADIATION,
        "RadLightFactor",
        Dec,
        "S-0525",
    ),
    ("rad_tint_factor", RADIATION, "RadTintFactor", Dec, "S-0525"),
    (
        "rad_site_warhead",
        RADIATION,
        "RadSiteWarhead",
        Ref(Kind::Warheads),
        "S-0525",
    ),
];

/// The crew types of `[General]` and our field names (S-0312 Import
/// mapping): infantry type keys.
// spec: S-0312 Import mapping (AlliedCrew, SovietCrew, ThirdCrew, Technician, Engineer: infantry names)
const CREWS: [(&str, &str); 5] = [
    ("allied_crew", "AlliedCrew"),
    ("soviet_crew", "SovietCrew"),
    ("third_crew", "ThirdCrew"),
    ("technician", "Technician"),
    ("engineer", "Engineer"),
];

/// The side crew types, the survivor divisor of each side and the
/// divisor's engine default (S-0312 F4, F5, Import mapping).
// spec: S-0312 F4, F5 and Import mapping (the three side crews, their survivor divisors, absent: 200, 100 and 300)
const SIDE_CREWS: [(&str, &str, i32); 3] = [
    ("AlliedCrew", "AlliedSurvivorDivisor", 200),
    ("SovietCrew", "SovietSurvivorDivisor", 100),
    ("ThirdCrew", "ThirdSurvivorDivisor", 300),
];

/// The five threat coefficients of a type and the matching game-wide
/// defaults, in the order own-effectiveness, target-effectiveness,
/// special-threat, strength, distance (S-0309 Import mapping).
// spec: S-0309 Import mapping (the five coefficient keys and their defaults)
const COEFFICIENTS: [(&str, &str); 5] = [
    (
        "MyEffectivenessCoefficient",
        "MyEffectivenessCoefficientDefault",
    ),
    (
        "TargetEffectivenessCoefficient",
        "TargetEffectivenessCoefficientDefault",
    ),
    (
        "TargetSpecialThreatCoefficient",
        "TargetSpecialThreatCoefficientDefault",
    ),
    (
        "TargetStrengthCoefficient",
        "TargetStrengthCoefficientDefault",
    ),
    (
        "TargetDistanceCoefficient",
        "TargetDistanceCoefficientDefault",
    ),
];

/// The text limit of a building's category word (S-0302 Import mapping).
const BUILD_CAT_LIMIT: usize = 31; // spec: S-0302 Import mapping (BuildCat: the first 31 bytes)

/// The coefficient magnitude above which the importer warns (S-0309 Edge
/// cases: 10⁶).
const COEFFICIENT_WARN: i128 = 1_000_000; // spec: S-0309 Edge cases (a coefficient's magnitude above 10^6)

/// The difficulty rows: our name and section (S-0302 Terms; S-0006 Import
/// mapping).
// spec: S-0301 Import mapping, S-0302 Import mapping (the three difficulty-row sections Easy, Normal, Difficult)
const DIFFICULTY_ROWS: [(&str, &str); 3] = [
    ("easy", "Easy"),
    ("normal", "Normal"),
    ("difficult", "Difficult"),
];

/// The ability names of an ability list (S-0311 Import mapping), compared
/// ignoring letter case.
// spec: S-0311 Import mapping (the eighteen ability names)
const ABILITIES: [&str; 18] = [
    "faster",
    "stronger",
    "firepower",
    "scatter",
    "rof",
    "sight",
    "cloak",
    "tiberium_proof",
    "vein_proof",
    "self_heal",
    "explodes",
    "radar_invisible",
    "sensors",
    "fearless",
    "c4",
    "tiberium_heal",
    "guard_area",
    "crusher",
];

/// Weapon slots (S-0308 claim 1: `Weapon1` … `Weapon18`).
const SLOTS: usize = 18; // spec: S-0308 claim 1, Import mapping (Weapon1 … Weapon18)

/// Muzzle-position keys of the primary and secondary slots and their
/// fields (S-0313 Import mapping).
// spec: S-0313 Import mapping (PrimaryFireFLH, SecondaryFireFLH and the elite forms)
const FLH_KEYS: [(&str, &str); 4] = [
    ("primary_fire_flh", "PrimaryFireFLH"),
    ("secondary_fire_flh", "SecondaryFireFLH"),
    ("elite_primary_fire_flh", "ElitePrimaryFireFLH"),
    ("elite_secondary_fire_flh", "EliteSecondaryFireFLH"),
];

/// A muzzle position no layer gives (S-0313 Import mapping: 0, 0, 0).
const FLH_ABSENT: [i32; 3] = [0, 0, 0]; // spec: S-0313 Import mapping and claim 1a (non-elite muzzle positions absent: 0, 0, 0)

/// The alternate link anchors read (S-0601 Import mapping, F5).
const ALTERNATE_FLH: usize = 5; // spec: S-0601 Import mapping (AlternateFLH0 through AlternateFLH4)

/// The burst-delay and firing-sync value no layer gives (S-0301 Import
/// mapping: −1).
const NO_DELAY: i32 = -1; // spec: S-0301 Import mapping (BurstDelay0 … 3 and FiringSyncFrame0, 1 absent: −1)

/// The armour-modifier list's item count (S-0003 Constants, S-0305 claim 1).
const VERSES: usize = value::ARMOUR_CLASS_COUNT; // spec: S-0003 Constants (armour_class_count), S-0305 claim 1

/// Damage magnitudes above this change when narrowed to single precision
/// (S-0302 Edge cases: 2²⁴).
const DAMAGE_WARN: i32 = 1 << 24; // spec: S-0302 Edge cases (a damage field above 2^24 in magnitude)

/// The largest garrison muzzle-point count the original supports (S-0313
/// Edge cases).
const COVERED_MUZZLE_POINTS: usize = 32; // nocite: how many MuzzleFlash keys the coverage table lists, our own bound
const MUZZLE_POINTS_READ: i32 = 256; // nocite: a bound on the points read (S-0313 Edge cases: every point up to the capacity), our own safety limit
const MAX_OCCUPANTS_QUIET: i32 = 10; // spec: S-0313 Edge cases (more than ten garrison muzzle points)

/// The scan radius range without a warning (S-0303 Edge cases).
const SPREAD_QUIET: (i128, i128) = (0, 11); // spec: S-0303 Edge cases (a spread outside 0 to 11)

/// The weapon index range without a warning (S-0308 Edge cases).
const INDEX_MAX: i32 = 17; // spec: S-0308 Edge cases (indices outside 0 … 17)

/// The elevation cap magnitude from which the importer warns (S-0307 Edge
/// cases).
const CAP_WARN: i128 = 148; // spec: S-0307 Edge cases (|trunc(c)| ≥ 148)

/// The range a lepton field of −2 cells gives: unlimited (S-0307 claim 2).
const UNLIMITED: i128 = 512; // spec: S-0307 claim 2 (R0 of exactly −512 leptons)

/// The building repair period of a repair rate, in ticks per game minute
/// (S-0314 Edge cases: a rate below 1/900 gives a period of 0).
const TICKS_PER_MINUTE: i64 = 900; // spec: S-0314 Edge cases (a rate whose size is below 1/900 of a game minute)

/// The game-wide repair step no layer gives (S-0314 Import mapping: 5).
const REPAIR_STEP_ABSENT: i32 = 5; // spec: S-0314 Import mapping (RepairStep absent: 5)

/// The damage-fire animations the list needs (S-0302 claim 13: three).
const FIRE_TYPES: usize = 3; // spec: S-0302 Edge cases (a damage-fire list with fewer than three entries)

/// The barrel fire animation, a name built into the original (S-0312
/// Import mapping).
const BARREL_FIRE: &[u8] = b"FIRE3"; // spec: S-0312 Import mapping (the animation type named FIRE3)

/// A decimal's sign, integer part and whether a fraction follows, from its
/// exact expansion; infinity has a saturated integer part.
#[derive(Clone, Copy, Debug)]
struct Exact {
    neg: bool,
    int: i128,
    frac: bool,
}

impl Exact {
    fn of(b: &Binary) -> Self {
        let s = b.to_exact_decimal();
        let neg = s.starts_with('-');
        let m = s.trim_start_matches('-');
        if !m.bytes().next().is_some_and(|c| c.is_ascii_digit()) {
            // Infinity (no imported value is not a number).
            return Self {
                neg,
                int: i128::MAX / 4,
                frac: false,
            };
        }
        let (i, f) = m.split_once('.').unwrap_or((m, ""));
        let int = i.bytes().fold(0i128, |a, d| {
            a.saturating_mul(10)
                .saturating_add(i128::from(d - b'0'))
                .min(i128::MAX / 4)
        });
        Self {
            neg,
            int,
            frac: f.bytes().any(|d| d != b'0'),
        }
    }

    fn is_zero(self) -> bool {
        self.int == 0 && !self.frac
    }

    fn negative(self) -> bool {
        self.neg && !self.is_zero()
    }

    /// The value is below `n`.
    fn lt(self, n: i128) -> bool {
        if self.negative() {
            self.int > -n || (self.int == -n && self.frac)
        } else {
            self.int < n
        }
    }

    /// The value is above `n`.
    fn gt(self, n: i128) -> bool {
        if self.negative() {
            self.int < -n
        } else {
            self.int > n || (self.int == n && self.frac)
        }
    }

    /// The value is exactly 1.
    fn is_one(self) -> bool {
        !self.neg && self.int == 1 && !self.frac
    }
}

/// The double S-0003 gives a stored decimal (claims 6 to 8), or `None`
/// without a number.
fn decimal_of(v: &[u8]) -> Option<Binary> {
    match value::decimal(v, DecimalField::Double) {
        IniDecimal::Value { value, .. } => Some(value.convert_rz(Format::Double)),
        IniDecimal::NoNumber => None,
    }
}

/// The lepton form of S-0307 claim 1: trunc(RZ₅₃(x × 256)).
fn leptons(x: &Binary) -> Exact {
    // spec: S-0307 claim 1 (× 256 in double precision, truncated toward zero)
    let p = x.mul_rz(&Binary::from_i64(Format::Double, 256), Format::Double);
    Exact {
        frac: false,
        ..Exact::of(&p)
    }
}

/// The text of a double in our decimal grammar: its exact decimal
/// expansion; an infinity is written as a decimal that S-0003 also reads as
/// infinity (claim 6: overflow to infinity).
pub(super) fn exact_text(b: &Binary) -> String {
    let s = b.to_exact_decimal();
    match s.as_str() {
        // spec: S-0003 claims 6 and 13 (a magnitude beyond the formats' range converts to infinity)
        "inf" => "1e400".into(),
        "-inf" => "-1e400".into(),
        _ => s,
    }
}

/// The layer and line of the last value a layer gave the key.
pub(super) fn last_set(state: Option<&KeyState>) -> (usize, u32) {
    state
        .and_then(|s| {
            s.history.iter().rev().find_map(|e| match e.what {
                What::Set(_) => Some((e.layer, e.line)),
                _ => None,
            })
        })
        .unwrap_or((0, 0))
}

/// The stored values a layer gave the key, with their layer and line.
fn sets(state: Option<&KeyState>) -> Vec<(Vec<u8>, usize, u32)> {
    state
        .into_iter()
        .flat_map(|s| &s.history)
        .filter_map(|e| match &e.what {
            What::Set(v) => Some((v.clone(), e.layer, e.line)),
            _ => None,
        })
        .collect()
}

pub(super) fn int_of(state: Option<&KeyState>) -> Option<i32> {
    fold(state, None, |v, _| value::integer(v))
}

pub(super) fn bool_of(state: Option<&KeyState>) -> Option<bool> {
    fold(state, None, |v, _| value::boolean(v))
}

fn dec_of(state: Option<&KeyState>) -> Option<Exact> {
    fold(state, None, |v, _| decimal_of(v)).map(|b| Exact::of(&b))
}

fn text_of(state: Option<&KeyState>) -> Option<Vec<u8>> {
    fold(state, None, |v, _| {
        Some(value::text(v, value::COMMON_TEXT_LIMIT).to_vec())
    })
}

fn list_of(state: Option<&KeyState>, untrimmed: bool) -> Option<Vec<Vec<u8>>> {
    fold(state, None, |v, _| {
        Some(
            // spec: S-0003 claims 12 and 19 (quirks.list_items_untrimmed off: name items trimmed before the no-type test)
            value::name_items(v, value::COMMON_TEXT_LIMIT, untrimmed)
                .into_iter()
                .filter(|i| !value::is_no_type(i))
                .map(<[u8]>::to_vec)
                .collect(),
        )
    })
}

pub(super) fn key<'a>(t: &'a TypeRec, k: &str) -> Option<&'a KeyState> {
    t.keys.get(k.as_bytes())
}

/// The type of `kind` that `name` names (S-0003 claim 14).
fn find_in<'o>(outcome: &'o Outcome, kind: Kind, name: &[u8]) -> Option<&'o TypeRec> {
    outcome
        .registries
        .get(&kind)?
        .iter()
        .find(|t| value::type_name_eq(&t.name, name))
}

/// The disc laser's colour keys (S-0615 Import mapping): our field, the
/// weapon key.
const LASER_COLOURS: [(&str, &str); 3] = [
    ("laser_inner_color", "LaserInnerColor"),
    ("laser_outer_color", "LaserOuterColor"),
    ("laser_outer_spread", "LaserOuterSpread"),
];

/// Every (category, key, spec) this module converts, for `coverage.csv`:
/// rules keys by kind or `global:Section`, art keys under `art`.
pub fn converted_keys() -> Vec<(&'static str, String, &'static str)> {
    let mut out = Vec::new();
    let mut add = |cats: &[&'static str], rows: &[Row]| {
        for c in cats {
            for r in rows {
                out.push((*c, r.1.to_owned(), r.3));
            }
        }
    };
    const TYPES: [&str; 4] = ["infantry", "vehicles", "aircraft", "buildings"];
    const UNITS: [&str; 3] = ["infantry", "vehicles", "aircraft"];
    add(&TYPES, COMBAT_FIELDS);
    add(&UNITS, UNIT_COMBAT);
    add(&["vehicles"], VEHICLE_COMBAT);
    add(&["infantry"], INFANTRY_COMBAT);
    add(&["buildings"], BUILDING_COMBAT);
    add(&["overlays"], OVERLAY_COMBAT);
    add(&["terrain_objects"], TERRAIN_COMBAT);
    add(&["weapons"], WEAPON_COMBAT);
    add(&["projectiles"], PROJECTILE_COMBAT);
    add(&["warheads"], WARHEAD_COMBAT);
    add(&["countries"], COUNTRY_COMBAT);
    for (_, section, k, _, spec) in GENERAL_COMBAT {
        let cat = match *section {
            s if s == COMBAT_DAMAGE => "global:CombatDamage",
            s if s == AUDIO_VISUAL => "global:AudioVisual",
            s if s == WALL_MODEL => "global:WallModel",
            s if s == ELEVATION_MODEL => "global:ElevationModel",
            s if s == IQ => "global:IQ",
            s if s == RADIATION => "global:Radiation",
            _ => "global:General",
        };
        out.push((cat, (*k).to_owned(), *spec));
    }
    // spec: S-0525 Import mapping (RadColor, a byte triple)
    out.push(("global:Radiation", "RadColor".to_owned(), "S-0525"));
    // spec: S-0518 Import mapping (the support modifier, read through the layers' F4)
    out.push((
        "global:General",
        "PrismSupportModifier".to_owned(),
        "S-0518",
    ));
    let mut keyed = |cats: &[&'static str], keys: &[String], spec: &'static str| {
        for c in cats {
            for k in keys {
                out.push((*c, k.clone(), spec));
            }
        }
    };
    let s = |k: &[&str]| k.iter().map(|k| (*k).to_owned()).collect::<Vec<_>>();
    keyed(&TYPES, &s(&["Armor"]), "S-0305");
    keyed(&["terrain_objects", "overlays"], &s(&["Armor"]), "S-0305");
    keyed(
        &["terrain_objects"],
        &s(&["Strength"]),
        "S-0102, S-0009, S-0302",
    );
    keyed(&["global:General"], &s(&["TreeStrength"]), "S-0102, S-0009");
    let mut weapons = s(&[
        "Primary",
        "Secondary",
        "ElitePrimary",
        "EliteSecondary",
        "ClearAllWeapons",
        "TurretCount",
        "WeaponCount",
    ]);
    for n in 1..=SLOTS {
        weapons.push(format!("Weapon{n}"));
        weapons.push(format!("EliteWeapon{n}"));
    }
    keyed(&TYPES, &weapons, "S-0308");
    let stages: Vec<String> = (1..=SPIN_STAGES)
        .flat_map(|k| [format!("Stage{k}"), format!("EliteStage{k}")])
        .collect();
    keyed(&TYPES, &stages, "S-0609");
    let coefficients: Vec<String> = COEFFICIENTS.iter().map(|c| c.0.to_owned()).collect();
    keyed(&TYPES, &coefficients, "S-0309");
    let defaults: Vec<String> = COEFFICIENTS.iter().map(|c| c.1.to_owned()).collect();
    keyed(&["global:General"], &defaults, "S-0309");
    keyed(
        &["global:General"],
        &s(&["DisabledDisguiseDetectionPercent"]),
        "S-0309",
    );
    keyed(&["global:General"], &s(&["HarvesterUnit"]), "S-0303");
    // spec: S-0607 Import mapping (the locomotor a seizing warhead installs)
    keyed(&["warheads"], &s(&["Locomotor"]), "S-0607");
    // spec: S-0613 Import mapping (the gas-cloud keys of particle and particle-system types)
    keyed(
        &["particles"],
        &s(&[
            "MaxEC",
            "MaxDC",
            "Damage",
            "Warhead",
            "StartFrame",
            "StartStateAI",
            "EndStateAI",
            "StateAIAdvance",
            "DeleteOnStateLimit",
            "NextParticle",
            "NextParticleOffset",
            "WindEffect",
            "Translucency",
        ]),
        "S-0202, S-0613",
    );
    keyed(&["particle_systems"], &s(&["Lifetime"]), "S-0613");
    // spec: S-0616 Import mapping (the keys of the smoke, fire, spark and railgun kinds that change play)
    keyed(
        &["particles"],
        &s(&[
            "Velocity",
            "Deacc",
            "FinalDamageState",
            "Normalized",
            "Radius",
            "XVelocity",
            "YVelocity",
            "MinZVelocity",
            "ZVelocityRange",
        ]),
        "S-0616",
    );
    keyed(
        &["particle_systems"],
        &s(&[
            "SpawnFrames",
            "Slowdown",
            "SpawnCutoff",
            "SpawnRadius",
            "ParticleCap",
            "SparkSpawnFrames",
            "SpawnSparkPercentage",
            "SpawnDirection",
            "ParticlesPerCoord",
        ]),
        "S-0616",
    );
    keyed(
        &UNITS,
        &s(&["DamageSmokeOffset", "DamSmkOffScrnRel"]),
        "S-0616",
    );
    keyed(&TYPES, &s(&["NaturalParticleLocation"]), "S-0616");
    // spec: S-0317 Import mapping (the voxel-animation keys of a debris piece)
    keyed(&["voxel_animations"], &super::piece_keys(), "S-0317");
    keyed(&TYPES, &s(&["DestroySmokeOffset"]), "S-0312");
    keyed(&["buildings"], &s(&REFINERY_SMOKE_POINTS), "S-0616");
    let crews: Vec<String> = CREWS.iter().map(|c| c.1.to_owned()).collect();
    keyed(&["global:General"], &crews, "S-0312");
    keyed(&UNITS, &s(&["DamageParticleSystems"]), "S-0302");
    keyed(&["buildings"], &s(&["BuildCat"]), "S-0302");
    keyed(&["buildings"], &s(&["Cost"]), "S-0311, S-0314");
    keyed(
        &["vehicles"],
        &s(&["BurstDelay0", "BurstDelay1", "BurstDelay2", "BurstDelay3"]),
        "S-0301",
    );
    keyed(
        &UNITS,
        &s(&["VeteranAbilities", "EliteAbilities"]),
        "S-0311",
    );
    keyed(
        &["countries"],
        &s(&["VeteranInfantry", "VeteranUnits", "VeteranAircraft"]),
        "S-0311",
    );
    // Art keys of the type's image section.
    let mut flh = s(&[
        "PrimaryFireFLH",
        "SecondaryFireFLH",
        "ElitePrimaryFireFLH",
        "EliteSecondaryFireFLH",
        "TurretOffset",
        "PrimaryFirePixelOffset",
        "PrimaryFireDualOffset",
    ]);
    for n in 1..=SLOTS {
        flh.push(format!("Weapon{n}FLH"));
        flh.push(format!("EliteWeapon{n}FLH"));
    }
    // spec: S-0313 Import mapping (MuzzleFlash followed by 0, 1, …: the garrison muzzle points)
    for n in 0..COVERED_MUZZLE_POINTS {
        flh.push(format!("MuzzleFlash{n}"));
    }
    keyed(&["art"], &flh, "S-0313");
    keyed(
        &["art"],
        &s(&[
            "AlternateFLH0",
            "AlternateFLH1",
            "AlternateFLH2",
            "AlternateFLH3",
            "AlternateFLH4",
        ]),
        "S-0601",
    );
    keyed(
        &["art"],
        &s(&[
            "FiringFrames",
            "FiringSyncFrame0",
            "FiringSyncFrame1",
            "IsAnimDelayedFire",
            "DelayedFireDelay",
        ]),
        "S-0301",
    );
    keyed(&["weapons"], &s(&LASER_COLOURS.map(|(_, k)| k)), "S-0615");
    // spec: S-0615 Import mapping (IsFlamingGuy and RunningFrames: animation type keys of the art data); S-0109 Import mapping (Damage, δ)
    keyed(&["art"], &s(&["IsFlamingGuy", "RunningFrames"]), "S-0615");
    keyed(&["art"], &s(&["Damage"]), "S-0109, S-0615, S-0319");
    // spec: S-0319 Import mapping (the bouncing-animation and trailer keys of an animation type's own art section)
    keyed(&["art"], &s(&["Bouncer", "IsMeteor"]), "S-0109, S-0319");
    keyed(
        &["art"],
        &s(&[
            "MaxXYVel",
            "MinZVel",
            "MaxZVel",
            "Elasticity",
            "DamageRadius",
            "Warhead",
            "BounceAnim",
            "ExpireAnim",
            "TrailerAnim",
            "TrailerSeperation",
            "Spawns",
            "SpawnCount",
            "IsTiberium",
            "TiberiumSpawnType",
            "TiberiumSpreadRadius",
        ]),
        "S-0319",
    );
    keyed(&["art"], &s(&["TargetCoordOffset", "Height"]), "S-0306");
    keyed(&["art"], &s(&["Trailer", "SpawnDelay"]), "S-0306, S-0007");
    // spec: S-0313 claim 10 (the turret anchor pair: rules keys of the building type)
    keyed(
        &["buildings"],
        &s(&["TurretAnimX", "TurretAnimY"]),
        "S-0313, S-0310",
    );
    out
}

impl Builder<'_> {
    /// The fields of a combat table that the type's layered keys give.
    fn cfields(&mut self, rec: &mut Rec, t: &TypeRec, kind: Kind, table: &[Row]) {
        for &(field, k, form, _) in table {
            let what = format!("{} {} {k}", kind.key(), show(&t.name));
            if let Some(n) = self.field(key(t, k), form, &what) {
                rec.put(field, n);
            }
        }
    }

    /// Warns once for every value a layer gave that `bad` flags.
    fn each(
        &mut self,
        state: Option<&KeyState>,
        bad: impl Fn(&[u8]) -> bool,
        message: impl Fn(&str) -> String,
    ) {
        for (v, layer, line) in sets(state) {
            if bad(&v) {
                self.warn(layer, line, message(&show(&v)));
            }
        }
    }

    /// Warns for an integer key given a value that `bad` flags.
    fn each_int(
        &mut self,
        state: Option<&KeyState>,
        bad: impl Fn(i32) -> bool,
        message: impl Fn(&str) -> String,
    ) {
        self.each(state, |v| value::integer(v).is_some_and(&bad), message);
    }

    /// Warns for a decimal key given a value that `bad` flags.
    fn each_dec(
        &mut self,
        state: Option<&KeyState>,
        bad: impl Fn(Exact) -> bool,
        message: impl Fn(&str) -> String,
    ) {
        self.each(
            state,
            |v| decimal_of(v).is_some_and(|b| bad(Exact::of(&b))),
            message,
        );
    }

    pub(super) fn warn_at(&mut self, state: Option<&KeyState>, message: String) {
        let (layer, line) = last_set(state);
        self.warn(layer, line, message);
    }

    fn global(&self, section: &[u8], k: &str) -> Option<KeyState> {
        self.i
            .outcome
            .globals
            .get(section)
            .and_then(|s| s.get(k.as_bytes()))
            .cloned()
    }

    /// The type of `kind` that `name` names (S-0003 claim 14).
    fn find_rec(&self, kind: Kind, name: &[u8]) -> Option<&TypeRec> {
        find_in(self.i.outcome, kind, name)
    }

    /// A type's armour class (S-0305 claims 3 and 4): each given word
    /// replaces the class in force; a word that matches none of the eleven
    /// gives class *none*, with a warning. Returns the class's word.
    fn armour(&mut self, kind: Kind, t: &TypeRec) -> Option<&'static str> {
        let state = key(t, "Armor");
        let mut notes = Vec::new();
        let class = fold(state, None, |v, e| {
            // spec: S-0305 claims 3 and 4 (the whole value, A to Z folded; no match gives none)
            let (c, matched) = ArmourClass::from_word(v);
            if !matched {
                notes.push((e.layer, e.line, show(v)));
            }
            Some(c)
        });
        for (layer, line, v) in notes {
            // spec: S-0305 claim 4 (the importer warns for every value that matches no word)
            self.warn(layer, line, format!(
                "{} {} Armor {v:?} is not an armour word; the type is given class none (S-0305 claim 4)",
                kind.key(),
                show(&t.name)
            ));
        }
        class.map(|c| ArmourClass::WORDS[c.index()])
    }

    /// The stored value of an art key in the type's image section after
    /// each type read (S-0007 claim 1), in read order.
    pub(super) fn art_values(
        &self,
        kind: Kind,
        index: usize,
        t: &TypeRec,
        k: &str,
    ) -> Vec<(Vec<u8>, u32)> {
        self.images(kind, index, t)
            .iter()
            .filter_map(|image| {
                self.art_section(image)
                    .and_then(|s| s.get(k.as_bytes()).map(|e| (e.value.clone(), e.line)))
            })
            .collect()
    }

    /// An integer art key: a value the image section gives replaces the
    /// held one; an absent key keeps it (S-0003 claim 3).
    pub(super) fn art_int(&self, kind: Kind, index: usize, t: &TypeRec, k: &str) -> Option<i32> {
        self.art_values(kind, index, t, k)
            .iter()
            .fold(None, |held, (v, _)| value::integer(v).or(held))
    }

    /// A boolean art key (S-0003 claim 2).
    pub(super) fn art_bool(&self, kind: Kind, index: usize, t: &TypeRec, k: &str) -> Option<bool> {
        self.art_values(kind, index, t, k)
            .iter()
            .fold(None, |held, (v, _)| value::boolean(v).or(held))
    }

    /// An integer pair or triple of the art data (S-0003 claim 9): unread
    /// components keep the held value's (0 before any), with a warning.
    pub(super) fn art_tuple<const N: usize>(
        &mut self,
        kind: Kind,
        index: usize,
        t: &TypeRec,
        k: &str,
    ) -> Option<[i32; N]> {
        self.art_tuple_from(kind, index, t, k, [0; N])
    }

    /// [`Self::art_tuple`] with `initial` as the value held before any
    /// data is read.
    pub(super) fn art_tuple_from<const N: usize>(
        &mut self,
        kind: Kind,
        index: usize,
        t: &TypeRec,
        k: &str,
        initial: [i32; N],
    ) -> Option<[i32; N]> {
        let mut held: Option<[i32; N]> = None;
        for (v, line) in self.art_values(kind, index, t, k) {
            // spec: S-0003 claim 9 (unread components take the value in force's, our engine; the importer warns)
            let (out, unread) = value::int_tuple_or::<N>(&v, held.unwrap_or(initial));
            if unread {
                self.art_warn(line, format!(
                    "{} {} art {k} {:?} has fewer than {N} readable integers; the others keep the value in force (S-0003 claim 9)",
                    kind.key(),
                    show(&t.name),
                    show(&v)
                ));
            }
            held = Some(out);
        }
        held
    }

    fn put_tuple<const N: usize>(rec: &mut Rec, field: &str, v: [i32; N]) {
        rec.put(
            field,
            jsonc::array(v.iter().map(|&x| int_node(x)).collect()),
        );
    }

    /// Writes a list of muzzle positions (three integers each).
    fn put_flh_list(rec: &mut Rec, field: &str, list: &[[i32; 3]]) {
        rec.put(
            field,
            jsonc::array(
                list.iter()
                    .map(|f| jsonc::array(f.iter().map(|&x| int_node(x)).collect()))
                    .collect(),
            ),
        );
    }

    /// An integer triple of one reading's art section, its unread
    /// components taking `held`'s (S-0003 claim 9, with a warning).
    fn art_triple_at(
        &mut self,
        kind: Kind,
        t: &TypeRec,
        image: &[u8],
        k: &str,
        held: [i32; 3],
    ) -> Option<[i32; 3]> {
        let e = self.art_section(image)?.get(k.as_bytes()).cloned()?;
        // spec: S-0003 claim 9 (unread components take the value in force's, our engine; the importer warns)
        let (out, unread) = value::int_tuple_or::<3>(&e.value, held);
        if unread {
            self.art_warn(e.line, format!(
                "{} {} art {k} {:?} has fewer than 3 readable integers; the others keep the value in force (S-0003 claim 9)",
                kind.key(),
                show(&t.name),
                show(&e.value)
            ));
        }
        Some(out)
    }

    /// The muzzle positions of the weapon slots and their elite versions
    /// (S-0313 claims 1a and 1b, Import mapping), read at every type read:
    /// the turret count after that read's rules section decides the family
    /// the reading takes; written for the family of the last reading.
    fn muzzle_positions(&mut self, combat: &mut Rec, kind: Kind, index: usize, t: &TypeRec) {
        // spec: S-0313 Import mapping and claim 1a (muzzle positions: three integers of the type's image section; 0, 0, 0 from the type's creation)
        let mut flh = [FLH_ABSENT; SLOTS];
        let mut elite = [FLH_ABSENT; SLOTS];
        let mut given = [false; SLOTS];
        let mut tc = 0;
        let images = self.images(kind, index, t);
        for (&r, image) in t.reads.iter().zip(&images) {
            // spec: S-0313 claim 1b (the turret count after the same type read's rules section decides the family that reading takes) and S-0308 Import mapping (engine default 0)
            tc = int_as_of(t, "TurretCount", r, 0);
            let keys: Vec<(usize, String, String)> = if tc > 0 {
                // spec: S-0313 claim 1b (turret count above 0: only the numbered fields and their elite forms, up to the weapon count; a weapon count of 0 reads neither family)
                let n = int_as_of(t, "WeaponCount", r, 0).clamp(0, SLOTS as i32) as usize;
                (0..n)
                    .map(|i| {
                        (
                            i,
                            format!("Weapon{}FLH", i + 1),
                            format!("EliteWeapon{}FLH", i + 1),
                        )
                    })
                    .collect()
            } else {
                // spec: S-0313 claim 1b (turret count 0 or less: only the primary and secondary fields and their elite forms, for slots 0 and 1)
                FLH_KEYS
                    .iter()
                    .take(2)
                    .zip(FLH_KEYS.iter().skip(2))
                    .enumerate()
                    .map(|(i, ((_, k), (_, ek)))| (i, (*k).to_owned(), (*ek).to_owned()))
                    .collect()
            };
            for (i, k, ek) in keys {
                // spec: S-0313 claim 1b (an absent non-elite field keeps the position the slot already holds)
                if let Some(v) = self.art_triple_at(kind, t, image, &k, flh[i]) {
                    flh[i] = v;
                    given[i] = true;
                }
                // spec: S-0313 claim 1b (an absent elite field gives the elite version the slot's non-elite position as it stands after this reading, not a value it held before)
                match self.art_triple_at(kind, t, image, &ek, elite[i]) {
                    Some(v) => {
                        elite[i] = v;
                        given[i] = true;
                    }
                    None => elite[i] = flh[i],
                }
            }
            // spec: S-0313 claim 1b (the family that is not read leaves its slots as they are)
        }
        let Some(last) = given.iter().rposition(|&g| g) else {
            return;
        };
        if tc > 0 {
            // spec: S-0313 claim 1b (a turret count above 0: the numbered weapons' positions)
            Self::put_flh_list(combat, "weapon_flh", &flh[..=last]);
            Self::put_flh_list(combat, "elite_weapon_flh", &elite[..=last]);
        } else {
            // spec: S-0313 claim 1b (a turret count of 0 or less: slots 0 and 1 through the primary and secondary fields)
            for (i, (field, _)) in FLH_KEYS.iter().take(2).enumerate() {
                if given[i] {
                    Self::put_tuple(combat, field, flh[i]);
                }
            }
            for (i, (field, _)) in FLH_KEYS.iter().skip(2).enumerate() {
                if given[i] {
                    Self::put_tuple(combat, field, elite[i]);
                }
            }
        }
    }

    /// The combat fields every object type with weapons shares (S-0301 to
    /// S-0315), written into `combat`.
    fn shared_combat(&mut self, combat: &mut Rec, kind: Kind, index: usize, t: &TypeRec) {
        self.cfields(combat, t, kind, COMBAT_FIELDS);
        // spec: S-0616 Import mapping (NaturalParticleLocation: three integers, leptons; absent 0, 0, 0) and S-0312 Import mapping (DestroySmokeOffset: three integers in leptons; absent 0, 0, 0)
        for (field, k) in [
            ("natural_particle_location", "NaturalParticleLocation"),
            ("destroy_smoke_offset", "DestroySmokeOffset"),
        ] {
            if let Some(v) = int_triple(t, k) {
                Self::put_tuple(combat, field, v);
            }
        }
        self.spin_settings(combat, kind, t);
        if let Some(word) = self.armour(kind, t) {
            combat.put("armor", jsonc::string(word));
        }
        self.coefficients(combat, kind, t);
        self.muzzle_positions(combat, kind, index, t);
        // spec: S-0601 Import mapping (AlternateFLH0 to AlternateFLH4: integer triples of the type's art section; absent: the zero triple)
        let alt: Vec<Option<[i32; 3]>> = (0..ALTERNATE_FLH)
            .map(|n| self.art_tuple::<3>(kind, index, t, &format!("AlternateFLH{n}")))
            .collect();
        if let Some(last) = alt.iter().rposition(Option::is_some) {
            combat.put(
                "alternate_flh",
                jsonc::array(
                    alt[..=last]
                        .iter()
                        .map(|f| {
                            let f = f.unwrap_or(FLH_ABSENT);
                            jsonc::array(f.iter().map(|&x| int_node(x)).collect())
                        })
                        .collect(),
                ),
            );
        }
        // spec: S-0313 Import mapping (TurretOffset: an integer of the type's image section)
        if let Some(v) = self.art_int(kind, index, t, "TurretOffset") {
            combat.put("turret_offset", int_node(v));
        }
        self.shared_warnings(kind, t);
    }

    fn shared_warnings(&mut self, kind: Kind, t: &TypeRec) {
        let name = format!("{} {}", kind.key(), show(&t.name));
        if bool_of(key(t, "DamageReducesReadiness")) == Some(true)
            && int_of(key(t, "Strength")).unwrap_or(0) == 0
        {
            // spec: S-0302 Edge cases (readiness loss with a strength of 0: reproduced, with a warning)
            self.warn_at(key(t, "DamageReducesReadiness"), format!(
                "{name} has DamageReducesReadiness and a strength of 0: any hit takes all its ammunition (S-0302 claim 10, F6)"
            ));
        }
        // spec: S-0314 Edge cases (the importer warns for a negative cost field)
        self.each_int(key(t, "Cost"), |c| c < 0, |v| {
            format!("{name} Cost {v} is negative; its repair step still costs at least 1 credit (S-0314 Edge cases)")
        });
        self.debris_warnings(&name, t);
    }

    /// The debris cases of S-0303 and S-0312 Edge cases, for a warhead or
    /// an object type.
    fn debris_warnings(&mut self, name: &str, t: &TypeRec) {
        let max = int_of(key(t, "MaxDebris")).unwrap_or(0);
        if max <= 0 {
            return;
        }
        let types = list_of(
            key(t, "DebrisTypes"),
            self.i.ini_quirks.list_items_untrimmed,
        )
        .unwrap_or_default();
        // spec: S-0003 claim 20 (the maxima list: decimal prefixes)
        let maxima = fold(key(t, "DebrisMaximums"), None, |v, _| {
            Some(value::decimal_prefix_list(v))
        })
        .unwrap_or_default();
        let metallic = self.global(GENERAL, "MetallicDebris");
        let metallic_empty = list_of(metallic.as_ref(), self.i.ini_quirks.list_items_untrimmed)
            .unwrap_or_default()
            .is_empty();
        let at = key(t, "MaxDebris");
        if types.is_empty() && metallic_empty {
            // spec: S-0303 Edge cases, S-0312 Edge cases (a positive maximum debris count with nothing to throw)
            self.warn_at(at, format!(
                "{name} has MaxDebris {max} but no DebrisTypes and the game-wide MetallicDebris list is empty: no debris is created (S-0303 F6, a departure)"
            ));
        }
        if !types.is_empty() && maxima.len() < types.len() {
            // spec: S-0303 Edge cases, S-0312 Edge cases (a maxima list shorter than the types list: missing maxima count as 0)
            self.warn_at(key(t, "DebrisMaximums"), format!(
                "{name} DebrisMaximums has fewer items than DebrisTypes; the missing maxima count as 0 (S-0303 F6)"
            ));
        }
        let paired = |i: usize| maxima.get(i).copied().unwrap_or(0);
        if !types.is_empty() && (0..types.len()).all(|i| matches!(paired(i), 0 | -2)) {
            // spec: S-0303 Edge cases, S-0312 Edge cases (every paired maximum 0 or −2 with a positive count: one pass that places nothing)
            self.warn_at(key(t, "DebrisMaximums"), format!(
                "{name} has MaxDebris {max} but every debris maximum is 0 or -2: our engine stops after one pass that places nothing (S-0303 F6, a departure)"
            ));
        }
        if (0..types.len()).any(|i| paired(i) == -1) {
            // spec: S-0303 Edge cases (departure: a zero debris-allocation divisor; the importer warns about a retained maximum of −1)
            self.warn_at(key(t, "DebrisMaximums"), format!(
                "{name} DebrisMaximums keeps a maximum of -1: the original stops with an error when it deals that type; our engine gives it min(|value|, pieces left) (S-0303 Edge cases, a departure)"
            ));
        }
    }

    /// The five threat coefficients (S-0309 Import mapping, Edge cases): at
    /// each reading of the type's section a given key replaces the held
    /// value; an absent key keeps it, except that a held 0 takes the
    /// matching game-wide default as it stands in that layer (the general
    /// settings are read before the types, S-0006 Terms) while
    /// `quirks.coefficient_zero_replaced` is on. Written only when the
    /// result differs from the game-wide defaults in force at the end, which
    /// the simulation uses for an absent field.
    fn coefficients(&mut self, combat: &mut Rec, kind: Kind, t: &TypeRec) {
        let name = format!("{} {}", kind.key(), show(&t.name));
        let mut out = Vec::new();
        let mut differs = false;
        for (k, default_key) in COEFFICIENTS {
            let state = key(t, k);
            // spec: S-0309 Edge cases (the importer warns for a type coefficient written as 0, in either setting)
            self.each(state, is_zero, |v| {
                format!("{name} {k} is written as {v:?}: it holds only until a later layer reads the section without the key (S-0309 Edge cases, quirks.coefficient_zero_replaced)")
            });
            // spec: S-0309 Edge cases (the importer warns for a coefficient magnitude above 10^6)
            self.each_dec(
                state,
                |x| x.gt(COEFFICIENT_WARN) || x.lt(-COEFFICIENT_WARN),
                |v| format!("{name} {k} {v} is above 10^6 in magnitude: threat scores can wrap (S-0309 Edge cases)"),
            );
            // spec: S-0309 claim 26 (each of a newly registered type's coefficients starts at double-precision 0)
            let mut held = "0".to_owned();
            for &r in &t.reads {
                let given = state.and_then(|s| {
                    s.history.iter().rev().find_map(|e| match &e.what {
                        What::Set(v) if e.layer == r => Some(v.clone()),
                        _ => None,
                    })
                });
                match given {
                    // spec: S-0003 claim 8 (no number: our engine keeps the value in force; the decimal reading warns)
                    Some(v) => {
                        if let Some(text) = decimal_text(&v) {
                            held = text;
                        }
                    }
                    // spec: S-0309 Edge cases (a held 0 takes the game-wide default as it stands when the type is read)
                    None if self.i.sim_quirks.coefficient_zero_replaced
                        && is_zero(held.as_bytes()) =>
                    {
                        held = self.coefficient_default(default_key, Some(r));
                    }
                    // spec: S-0309 Edge cases (switch off: an absent key always keeps the held value)
                    None => {}
                }
            }
            let what = format!("{name} {k}");
            let _ = self.decimal(state, None, &what);
            let end = self.coefficient_default(default_key, None);
            differs |= held != end;
            out.push(jsonc::string(held));
        }
        if differs {
            combat.put("coefficients", jsonc::array(out));
        }
    }

    /// A game-wide default coefficient after layer `upto` (S-0309 Import
    /// mapping: engine default 0).
    fn coefficient_default(&mut self, k: &str, upto: Option<usize>) -> String {
        let state = self.global(GENERAL, k);
        let what = format!("[General] {k}");
        self.decimal(state.as_ref(), upto, &what)
            // spec: S-0309 Import mapping (the game-wide default coefficients: engine default 0)
            .unwrap_or_else(|| "0".to_owned())
    }

    /// The weapon slots (S-0308 claim 1). At each reading of the type's
    /// section a type whose turret count (as that layer leaves it) is above
    /// 0 fills slots 0 … N − 1 from `Weapon1` … `WeaponN` (N its weapon
    /// count, at most 18), and any other type fills slots 0 and 1 from
    /// `Primary` and `Secondary` and the elite slots from the elite keys;
    /// keys the layer omits keep the slot. The clear-weapons flag, as that
    /// layer leaves it, then empties slots 0 and 1 and elite slots 0 and 1
    /// (`quirks.clear_weapons_any_turret_count`; off: only with turret
    /// count 0 or less).
    ///
    /// `primary` and `secondary` (and the elite pair of mobile types) are
    /// written from slots 0 and 1; `combat.weapon_slots` is written for a
    /// type whose turret count is above 0 or that holds a weapon beyond slot
    /// 1, likewise the elite slots.
    fn weapon_slots(&mut self, rec: &mut Rec, combat: &mut Rec, kind: Kind, t: &TypeRec) {
        let mut slots = vec![String::new(); SLOTS];
        let mut elite = vec![String::new(); SLOTS];
        let weapon = |b: &Self, k: &str, r: usize| {
            given_at(t, k, r).map(|v| b.ref_name(Kind::Weapons, v).unwrap_or_default())
        };
        for &r in &t.reads {
            // spec: S-0308 Import mapping (turret count and weapon count: engine default 0)
            let tc = int_as_of(t, "TurretCount", r, 0);
            if tc > 0 {
                let n = int_as_of(t, "WeaponCount", r, 0).clamp(0, SLOTS as i32) as usize;
                for k in 1..=n {
                    if let Some(w) = weapon(self, &format!("Weapon{k}"), r) {
                        slots[k - 1] = w;
                    }
                    if let Some(w) = weapon(self, &format!("EliteWeapon{k}"), r) {
                        elite[k - 1] = w;
                    }
                }
            } else {
                // spec: S-0308 claim 1 (turret count 0 or less: Primary, Secondary and the elite keys)
                for (i, (k, ek)) in [("Primary", "ElitePrimary"), ("Secondary", "EliteSecondary")]
                    .into_iter()
                    .enumerate()
                {
                    if let Some(w) = weapon(self, k, r) {
                        slots[i] = w;
                    }
                    if let Some(w) = weapon(self, ek, r) {
                        elite[i] = w;
                    }
                }
            }
            if bool_as_of(t, "ClearAllWeapons", r, false)
                && (self.i.sim_quirks.clear_weapons_any_turret_count || tc <= 0)
            {
                // spec: S-0308 claim 1, Edge cases (the clear-weapons flag empties slots 0 and 1 and elite slots 0 and 1)
                for s in [&mut slots, &mut elite] {
                    s[0].clear();
                    s[1].clear();
                }
            }
        }
        self.empty_stage_slots(kind, t, &slots);
        let name = format!("{} {}", kind.key(), show(&t.name));
        // spec: S-0308 claim 16 and Edge cases (a weapon count above 18: our engine fills only the first 18 slots and their elite slots, the higher references still register weapons; the importer warns)
        self.each_int(key(t, "WeaponCount"), |n| n > SLOTS as i32, |v| {
            format!("{name} WeaponCount {v} is above 18; only Weapon1 … Weapon18 fill firing slots, the higher numbered weapons are registered but never fired (S-0308 claim 16, a departure)")
        });
        let tc = int_of(key(t, "TurretCount")).unwrap_or(0);
        let unit = kind != Kind::Buildings;
        for (field, slot) in [("primary", &slots[0]), ("secondary", &slots[1])] {
            if !slot.is_empty() {
                rec.put(field, jsonc::string(slot.clone()));
            }
        }
        if unit {
            for (field, slot) in [("elite_primary", &elite[0]), ("elite_secondary", &elite[1])] {
                if !slot.is_empty() {
                    rec.put(field, jsonc::string(slot.clone()));
                }
            }
        }
        let trimmed = |s: &[String]| {
            let end = s.iter().rposition(|w| !w.is_empty()).map_or(0, |i| i + 1);
            jsonc::array(s[..end].iter().cloned().map(jsonc::string).collect())
        };
        if tc > 0 || slots[2..].iter().any(|w| !w.is_empty()) {
            combat.put("weapon_slots", trimmed(&slots));
        }
        if tc > 0
            || elite[2..].iter().any(|w| !w.is_empty())
            || (!unit && elite[..2].iter().any(|w| !w.is_empty()))
        {
            combat.put("elite_weapon_slots", trimmed(&elite));
        }
    }

    /// The numbered muzzle positions belong to the numbered weapon slots
    /// (S-0313 Terms: one muzzle position per weapon slot; S-0308 claim 1:
    /// a type uses either the numbered slots or the primary and secondary
    /// pair). They are kept only for a type written with numbered slots, so
    /// that a single-weapon type keeps its primary and secondary positions.
    fn numbered_flh_only_with_slots(combat: &mut Rec) {
        if !combat.0.iter().any(|(f, _)| f == "weapon_slots") {
            combat
                .0
                .retain(|(f, _)| f != "weapon_flh" && f != "elite_weapon_flh");
        }
    }

    /// `DamageSmokeOffset` and `DamSmkOffScrnRel` (S-0616 claim 1 and
    /// Import mapping): the damage-smoke displacement, three integers in
    /// leptons; the screen-relative case is not specified, so the importer
    /// warns and the displacement is used as given.
    fn damage_smoke_offset(&mut self, combat: &mut Rec, kind: Kind, t: &TypeRec) {
        // spec: S-0616 Import mapping (DamageSmokeOffset: three integers, leptons; absent 0, 0, 0)
        if let Some(o) = int_triple(t, "DamageSmokeOffset") {
            Self::put_tuple(combat, "damage_smoke_offset", o);
        }
        let rel = fold(t.keys.get(&b"DamSmkOffScrnRel"[..]), None, |v, _| {
            value::boolean(v)
        });
        if rel == Some(true) {
            // spec: S-0616 claim 1 and Open questions (the screen-relative flag on: not specified; the importer warns and uses the displacement as given)
            let line = t.reads.last().copied().unwrap_or(t.created_in);
            self.warn(line, 0, format!(
                "{} {}: DamSmkOffScrnRel is on; the screen-relative damage-smoke displacement is not specified, so it is used as given (S-0616 claim 1)",
                kind.key(),
                show(&t.name)
            ));
        }
    }

    /// `DamageParticleSystems` (S-0302 Import mapping): the listed particle
    /// systems whose `BehavesLike` is `Smoke` (letter case ignored), in list
    /// order, as type keys.
    fn smoke_systems(&mut self, combat: &mut Rec, t: &TypeRec) {
        let Some(list) = list_of(
            key(t, "DamageParticleSystems"),
            self.i.ini_quirks.list_items_untrimmed,
        ) else {
            return;
        };
        let Some(reg) = self.i.outcome.registries.get(&Kind::ParticleSystems) else {
            combat.put("damage_smoke_systems", jsonc::array(Vec::new()));
            return;
        };
        let keys = type_keys(reg);
        let mut out = Vec::new();
        let mut sparks = Vec::new();
        for item in list {
            let Some(i) = reg.iter().position(|p| value::type_name_eq(&p.name, &item)) else {
                continue;
            };
            let kind = text_of(key(&reg[i], "BehavesLike"));
            // spec: S-0302 Import mapping (eligible when BehavesLike is Smoke, letter case ignored; other and unknown words are not)
            let smoke = kind
                .as_ref()
                .is_some_and(|b| b.eq_ignore_ascii_case(b"Smoke"));
            // spec: S-0616 claim 1 (damage sparks: the spark-kind entries of the same list, in list order)
            let spark = kind.is_some_and(|b| b.eq_ignore_ascii_case(b"Spark"));
            if smoke {
                out.push(jsonc::string(keys[i].clone()));
            }
            if spark {
                sparks.push(jsonc::string(keys[i].clone()));
            }
        }
        combat.put("damage_smoke_systems", jsonc::array(out));
        if !sparks.is_empty() {
            combat.put("damage_spark_systems", jsonc::array(sparks));
        }
    }

    /// The gattling thresholds and the warnings for the spin settings
    /// (S-0609 Import mapping and Edge cases). At each reading of the
    /// type's section, when the gattling flag and the stage count as that
    /// layer leaves them are set and at least 2, the layer's `Stage`*k* and
    /// `EliteStage`*k* for *k* = 1 … *N* (at most 6) replace the held
    /// thresholds; any other layer keeps them (initially 0). The two rows
    /// are written as six integers once a layer has populated them.
    fn spin_settings(&mut self, combat: &mut Rec, kind: Kind, t: &TypeRec) {
        let mut rows = [[0i32; SPIN_STAGES]; 2];
        let mut given = [false; 2];
        let mut populated = false;
        // The layer and line of the WeaponStages that took a populated type below 2 stages, while the count stays below 2.
        let mut reduced = None;
        let mut last = (false, 0);
        for &r in &t.reads {
            // spec: S-0609 Import mapping (the gattling flag and the stage count as that layer leaves them; absent: no and 0)
            let flag = bool_as_of(t, "IsGattling", r, false);
            let n = int_as_of(t, "WeaponStages", r, 0);
            last = (flag, n);
            if !(flag && n >= 2) {
                if populated && flag && n < 2 && reduced.is_none() {
                    // The WeaponStages in force at this layer, which may come from an earlier layer that also cleared the flag.
                    reduced = sets(key(t, "WeaponStages"))
                        .into_iter()
                        .rfind(|e| e.1 <= r)
                        .map(|e| (e.1, e.2));
                }
                continue;
            }
            reduced = None;
            // spec: S-0609 Import mapping and Edge cases (k = 1 … N, at most 6 in our engine)
            let top = n.min(SPIN_STAGES as i32) as usize;
            for (row, prefix) in [(0, "Stage"), (1, "EliteStage")] {
                for k in 1..=top {
                    if let Some(v) =
                        given_at(t, &format!("{prefix}{k}"), r).and_then(value::integer)
                    {
                        // spec: S-0609 Import mapping (integer each; a later layer's value replaces the held one)
                        rows[row][k - 1] = v;
                        given[row] = true;
                    }
                }
            }
            populated = true;
        }
        for (row, field) in [(0, "stage_thresholds"), (1, "elite_stage_thresholds")] {
            if given[row] {
                combat.put(
                    field,
                    jsonc::array(rows[row].iter().map(|&v| int_node(v)).collect()),
                );
            }
        }
        let name = format!("{} {}", kind.key(), show(&t.name));
        // spec: S-0609 Edge cases (revision 5, answering Q-1151: a type whose stage count, as the layers finally leave it, is below 2 after an earlier layer populated its thresholds uses no thresholds, and the importer warns about the layer that reduced the count) and Import mapping
        if let (Some((layer, line)), (true, n)) = (reduced, last)
            && n < 2
        {
            self.warn(layer, line, format!(
                "{name}: WeaponStages falls below 2 after an earlier layer set the stage thresholds; our engine uses no thresholds for a type with fewer than 2 stages (S-0609 Edge cases)"
            ));
        }
        // spec: S-0609 Edge cases (more than 6 stages: our engine reads at most six, a larger count is used as 6, and the importer warns)
        self.each_int(key(t, "WeaponStages"), |n| n > SPIN_STAGES as i32, |v| {
            format!("{name} WeaponStages {v} is above 6; our engine uses 6 stages and reads only Stage1 … Stage6 and EliteStage1 … EliteStage6 (S-0609 Edge cases)")
        });
        // spec: S-0609 Edge cases (a negative stage count, spin-up or spin-down amount or threshold is used as 0, and the importer warns for each such imported value)
        let mut negative: Vec<String> = ["WeaponStages", "RateUp", "RateDown"]
            .iter()
            .map(|k| (*k).to_owned())
            .collect();
        for k in 1..=SPIN_STAGES {
            negative.push(format!("Stage{k}"));
            negative.push(format!("EliteStage{k}"));
        }
        for k in negative {
            self.each_int(
                key(t, &k),
                |n| n < 0,
                |v| format!("{name} {k} {v} is negative; our engine uses 0 (S-0609 Edge cases)"),
            );
        }
    }

    /// Warns for a gattling-flagged type whose ground slot 2*s* is empty
    /// for a stage *s* it can reach (S-0609 Edge cases: the spin sound
    /// reads that slot's report list; our engine plays nothing).
    fn empty_stage_slots(&mut self, kind: Kind, t: &TypeRec, slots: &[String]) {
        if !bool_of(key(t, "IsGattling")).unwrap_or(false) {
            return;
        }
        // spec: S-0609 Edge cases (stage count 0 or 1: the stage stays 0 and the sound comes from slot 0; above 6: 6)
        let n = int_of(key(t, "WeaponStages"))
            .unwrap_or(0)
            .clamp(1, SPIN_STAGES as i32) as usize;
        let empty: Vec<String> = (0..n)
            .filter(|s| slots.get(2 * s).is_none_or(String::is_empty))
            .map(|s| (2 * s).to_string())
            .collect();
        if empty.is_empty() {
            return;
        }
        let Some((_, layer, line)) = sets(key(t, "IsGattling")).into_iter().last() else {
            return;
        };
        // spec: S-0609 Edge cases (an empty slot 2s: our engine treats it as an empty report list, no sound and no draw, with an importer warning)
        self.warn(layer, line, format!(
            "{} {}: a gattling type with an empty weapon slot {} for a stage it can reach; its spin sound there plays nothing (S-0609 Edge cases)",
            kind.key(),
            show(&t.name),
            empty.join(", ")
        ));
    }

    /// The turret table pairs of a vehicle type that reads them (S-0105
    /// claim 43, Import mapping, *Turret table pairs*): the fixed type name,
    /// or every vehicle type with `quirks.turret_table_fixed_type_only` off.
    /// For the fixed type name, the charge-turret flag a pair with no weapon
    /// number gives (`quirks.turret_pair_no_weapon_sets_charge_flag`),
    /// written over the held `is_charge_turret`; for every type that reads
    /// them, the importer warnings for its pairs. The table itself is
    /// [`crate::Import::turret_tables`].
    fn turret_pairs(&mut self, combat: &mut Rec, t: &TypeRec) {
        use crate::turrets::{STEMS, TABLE_LEN, charge_flag, is_fixed_type, pairs_in};
        let fixed = is_fixed_type(&t.name);
        if !fixed && self.i.sim_quirks.turret_table_fixed_type_only {
            // spec: S-0105 claim 43 and Edge cases (quirks.turret_table_fixed_type_only: no other type reads its pairs)
            return;
        }
        // spec: S-0105 Edge cases (the charge-turret rule stays with the fixed type name whatever quirks.turret_table_fixed_type_only holds)
        if fixed
            && self.i.sim_quirks.turret_pair_no_weapon_sets_charge_flag
            && let Some(on) = charge_flag(t)
        {
            // spec: S-0105 Edge cases (quirks.turret_pair_no_weapon_sets_charge_flag: the fixed type name has charge-turret behaviour exactly as its pairs with no weapon number give it; S-0308 claim 5 and S-0301 claim 16 follow the flag)
            combat.0.retain(|(f, _)| f != "is_charge_turret");
            combat.put("is_charge_turret", jsonc::boolean(on));
        }
        // spec: S-0007 claim 17 (our engine treats a turret count above 18 as 18)
        let n = int_of(key(t, "TurretCount"))
            .unwrap_or(0)
            .min(TABLE_LEN as i32);
        let name = show(&t.name);
        for layer in t.reads.clone() {
            for p in pairs_in(t, layer) {
                let stem = STEMS[p.stem].0;
                if !p.no_weapon() && !p.sets_entry() {
                    // spec: S-0105 Import mapping (a weapon number outside 0 … 17 other than −1: our engine ignores the pair and the importer warns, design)
                    let (l, line) = p.weapon_at.unwrap_or((layer, 0));
                    self.warn(
                        l,
                        line,
                        format!(
                            "vehicles {name} {stem}TurretWeapon {} is outside 0 … 17; the pair is ignored (S-0105 Import mapping)",
                            p.weapon
                        ),
                    );
                } else if p.sets_entry() && !(-1..n).contains(&p.index) {
                    // spec: S-0105 Import mapping and claim 12 (a turret index outside −1 … N − 1 is kept and draws nothing; the importer warns)
                    let (l, line) = p.index_at.or(p.weapon_at).unwrap_or((layer, 0));
                    self.warn(
                        l,
                        line,
                        format!(
                            "vehicles {name} {stem}TurretIndex {} is outside −1 … {}; kept, it draws no turret (S-0105 claim 12, Import mapping)",
                            p.index,
                            n - 1
                        ),
                    );
                }
            }
        }
    }

    /// The combat fields of an infantry, vehicle or aircraft type.
    pub(super) fn unit_combat(&mut self, rec: &mut Rec, kind: Kind, index: usize, t: &TypeRec) {
        let mut combat = Rec::default();
        self.shared_combat(&mut combat, kind, index, t);
        self.weapon_slots(rec, &mut combat, kind, t);
        Self::numbered_flh_only_with_slots(&mut combat);
        self.smoke_systems(&mut combat, t);
        self.damage_smoke_offset(&mut combat, kind, t);
        if kind == Kind::Vehicles {
            self.turret_pairs(&mut combat, t);
        }
        if !combat.0.is_empty() {
            rec.put("combat", combat.node());
        }
        self.cfields(rec, t, kind, UNIT_COMBAT);
        if kind == Kind::Aircraft {
            // spec: S-0004 claim 14 and S-0506 claim 21 (an aircraft's trailer animation: the art key of its image section; each read's value replaces the held one, none names no type)
            let trailer = self
                .art_values(kind, index, t, "Trailer")
                .iter()
                .fold(None, |_, (v, _)| Some(self.ref_name(Kind::Animations, v)));
            if let Some(Some(n)) = trailer {
                rec.put("trailer", jsonc::string(n));
            }
        }
        let name = format!("{} {}", kind.key(), show(&t.name));
        match kind {
            Kind::Vehicles => {
                self.cfields(rec, t, kind, VEHICLE_COMBAT);
                // spec: S-0301 Import mapping (BurstDelay0 … 3: integers, ticks; absent −1)
                let delays: Vec<Option<i32>> = (0..4)
                    .map(|j| int_of(key(t, &format!("BurstDelay{j}"))))
                    .collect();
                if delays.iter().any(Option::is_some) {
                    rec.put(
                        "burst_delay",
                        jsonc::array(
                            delays
                                .iter()
                                .map(|d| int_node(d.unwrap_or(NO_DELAY)))
                                .collect(),
                        ),
                    );
                }
                for j in 0..4 {
                    // spec: S-0301 Edge cases (the importer warns for a burst-delay value below −1)
                    self.each_int(key(t, &format!("BurstDelay{j}")), |d| d < NO_DELAY, |v| {
                        format!("{name} BurstDelay{j} {v} is below −1: the delay can be 0 or less (S-0301 Edge cases)")
                    });
                }
                // spec: S-0301 Import mapping (FiringFrames, FiringSyncFrame0 and 1: integers of the vehicle image; sync frames absent −1)
                if let Some(f) = self.art_int(kind, index, t, "FiringFrames") {
                    rec.put("firing_frames", int_node(f));
                }
                let sync =
                    [0, 1].map(|j| self.art_int(kind, index, t, &format!("FiringSyncFrame{j}")));
                if sync.iter().any(Option::is_some) {
                    Self::put_tuple(
                        rec,
                        "firing_sync_frames",
                        sync.map(|s| s.unwrap_or(NO_DELAY)),
                    );
                }
            }
            Kind::Infantry => self.cfields(rec, t, kind, INFANTRY_COMBAT),
            _ => {}
        }
        // spec: S-0308 Edge cases (a mode index outside 0 … 17 gives weapon 0; the importer warns)
        self.each_int(key(t, "IFVMode"), |m| !(0..=INDEX_MAX).contains(&m), |v| {
            format!("{name} IFVMode {v} is outside 0 … 17: it gives weapon number 0 (S-0308 claim 6)")
        });
        // spec: S-0308 Edge cases (a passenger weapon index outside −1 … 17: treated as 0, a departure)
        self.each_int(key(t, "OpenTransportWeapon"), |m| !(-1..=INDEX_MAX).contains(&m), |v| {
            format!("{name} OpenTransportWeapon {v} is outside −1 … 17: our engine uses 0 (S-0308 Edge cases, a departure)")
        });
        if bool_of(key(t, "DeployFire")) == Some(true) {
            // spec: S-0308 Edge cases (a deploy weapon index outside 0 … 17 on a deploy-fire type: treated as 0, a departure)
            self.each_int(key(t, "DeployFireWeapon"), |m| !(0..=INDEX_MAX).contains(&m), |v| {
                format!("{name} DeployFireWeapon {v} is outside 0 … 17: our engine uses 0 (S-0308 Edge cases, a departure)")
            });
        }
        // spec: S-0315 Edge cases (a suppression threshold below 0: reproduced; the importer warns)
        self.each_int(key(t, "SuppressionThreshold"), |x| x < 0, |v| {
            format!("{name} SuppressionThreshold {v} is below 0: every non-negative hit suppresses the parasite (S-0315 claim 6)")
        });
        let untrimmed = self.i.ini_quirks.list_items_untrimmed;
        for k in ["VeteranAbilities", "EliteAbilities"] {
            for (v, layer, line) in sets(key(t, k)) {
                for item in value::name_items(&v, value::COMMON_TEXT_LIMIT, untrimmed) {
                    if untrimmed && value::edge_whitespace(item) {
                        // spec: S-0311 Edge cases (quirks.list_items_untrimmed on: the importer warns for surrounding whitespace)
                        self.warn(layer, line, format!(
                            "{name} {k}: a list item with whitespace at its start or end names a different type (S-0003 claim 12, quirks.list_items_untrimmed)"
                        ));
                    }
                    if !ABILITIES
                        .iter()
                        .any(|a| item.eq_ignore_ascii_case(a.as_bytes()))
                    {
                        // spec: S-0311 Edge cases (the importer warns for each non-empty item that matches no ability, in either setting of quirks.list_items_untrimmed)
                        self.warn(layer, line, format!(
                            "{name} {k}: {:?} names no ability and is ignored (S-0311 Import mapping)",
                            show(item)
                        ));
                    }
                }
            }
        }
    }

    /// The combat fields of a building type.
    pub(super) fn building_combat(&mut self, rec: &mut Rec, index: usize, t: &TypeRec) {
        let kind = Kind::Buildings;
        let mut combat = Rec::default();
        self.shared_combat(&mut combat, kind, index, t);
        self.weapon_slots(rec, &mut combat, kind, t);
        Self::numbered_flh_only_with_slots(&mut combat);
        if !combat.0.is_empty() {
            rec.put("combat", combat.node());
        }
        self.cfields(rec, t, kind, BUILDING_COMBAT);
        // spec: S-0616 Import mapping (RefinerySmokeOffsetOne … RefinerySmokeOffsetFour: three integers each, leptons; absent unset)
        let points: Vec<Option<[i32; 3]>> = REFINERY_SMOKE_POINTS
            .iter()
            .map(|k| int_triple(t, k))
            .collect();
        if points.iter().any(Option::is_some) {
            rec.put(
                "refinery_smoke_offsets",
                jsonc::array(
                    points
                        .iter()
                        // spec: S-0616 claim 1 (a point of (0, 0, 0) counts as unset, as a point not given does)
                        .map(|p| {
                            jsonc::array(p.unwrap_or([0; 3]).iter().map(|&x| int_node(x)).collect())
                        })
                        .collect(),
                ),
            );
        }
        // spec: S-0004 claim 11 (the sidebar tab's build category uses this same rule)
        // spec: S-0302 Import mapping (BuildCat: the first 31 bytes, surrounding whitespace trimmed, a whole word compared ignoring A–Z case; a defence building only for Combat, any other nonempty word, unknown ones included, is not; missing or empty keeps the held category, initially not a defence)
        if let Some(d) = fold(key(t, "BuildCat"), None, |v, _| {
            let word = value::text(v, BUILD_CAT_LIMIT).trim_ascii();
            (!word.is_empty()).then(|| word.eq_ignore_ascii_case(b"Combat"))
        }) {
            rec.put("defense", jsonc::boolean(d));
        }
        // spec: S-0306 Import mapping (TargetCoordOffset: an integer triple of the building image; Height: an integer)
        if let Some(v) = self.art_tuple::<3>(kind, index, t, "TargetCoordOffset") {
            Self::put_tuple(rec, "target_coord_offset", v);
        }
        if let Some(v) = self.art_int(kind, index, t, "Height") {
            rec.put("art_height", int_node(v));
        }
        // spec: S-0310 Import mapping, S-0313 Import mapping, S-0301 Import mapping (building image flags)
        for (field, k) in [
            ("primary_fire_dual_offset", "PrimaryFireDualOffset"),
            ("is_anim_delayed_fire", "IsAnimDelayedFire"),
        ] {
            if let Some(b) = self.art_bool(kind, index, t, k) {
                rec.put(field, jsonc::boolean(b));
            }
        }
        // spec: S-0313 Import mapping (PrimaryFirePixelOffset: two integers, pixels)
        if let Some(v) = self.art_tuple::<2>(kind, index, t, "PrimaryFirePixelOffset") {
            Self::put_tuple(rec, "primary_fire_pixel_offset", v);
        }
        // spec: S-0313 Import mapping (MuzzleFlash followed by 0, 1, …: two integers, pixels, of the building image, one for each place of the occupant capacity; a point not given is (0, 0))
        if let Some(n) = int_of(key(t, "MaxNumberOccupants")).filter(|&n| n > 0) {
            let mut pts: Vec<[i32; 2]> = Vec::new();
            let mut any = false;
            for i in 0..n.min(MUZZLE_POINTS_READ) {
                let p = self.art_tuple::<2>(kind, index, t, &format!("MuzzleFlash{i}"));
                any |= p.is_some();
                pts.push(p.unwrap_or([0; 2]));
            }
            if any {
                rec.put(
                    "garrison_muzzles",
                    jsonc::array(
                        pts.iter()
                            .map(|p| jsonc::array(p.iter().map(|&x| int_node(x)).collect()))
                            .collect(),
                    ),
                );
            }
        }
        // spec: S-0313 claim 10 and Import mapping (TurretAnimX, TurretAnimY: integers, pixels, from the building type's rules section through its layers, a missing key keeping the held value; absent 0; art has no effect)
        let xy =
            ["TurretAnimX", "TurretAnimY"].map(|k| fold(key(t, k), None, |v, _| value::integer(v)));
        if xy.iter().any(Option::is_some) {
            Self::put_tuple(rec, "turret_anim_xy", xy.map(|v| v.unwrap_or(0)));
        }
        // spec: S-0301 Import mapping (DelayedFireDelay: an integer of the building image, ticks)
        if let Some(v) = self.art_int(kind, index, t, "DelayedFireDelay") {
            rec.put("delayed_fire_delay", int_node(v));
        }
        let name = format!("buildings {}", show(&t.name));
        // spec: S-0313 Edge cases (a maximum occupant count above ten: the importer warns)
        self.each_int(key(t, "MaxNumberOccupants"), |n| n > MAX_OCCUPANTS_QUIET, |v| {
            format!("{name} MaxNumberOccupants {v} is above 10; the original supports at most ten garrison muzzle points (S-0313 Edge cases, a departure)")
        });
        for k in ["InfantryGainSelfHeal", "UnitsGainSelfHeal"] {
            // spec: S-0314 Edge cases (the importer warns for a negative self-heal count, in either setting of quirks.negative_self_heal_counts)
            self.each_int(key(t, k), |n| n < 0, |v| {
                format!("{name} {k} {v} is negative (S-0314 claim 10, quirks.negative_self_heal_counts)")
            });
        }
        // spec: S-0516 claim 20 and Import mapping (a negative effective cloak radius, kept as signed 8-bit, is invalid: the generator grows no field; the importer warns)
        self.each_int(key(t, "CloakRadiusInCells"), |n| (n as i8) < 0, |v| {
            format!("{name} CloakRadiusInCells {v} is negative as a signed 8-bit value: our engine grows no cloak field (S-0516 claim 20, a departure)")
        });
        let step = self.global(GENERAL, "RepairStep");
        let step = int_of(step.as_ref()).unwrap_or(REPAIR_STEP_ABSENT);
        if let Some(s) = int_of(key(t, "Strength"))
            && s < step
            && step >= 1
        {
            // spec: S-0314 Edge cases (strength below the repair step: our engine takes one step; the importer warns)
            self.warn_at(key(t, "Strength"), format!(
                "{name} Strength {s} is below the repair step {step}: our engine counts one step (S-0314 F2, a departure)"
            ));
        }
    }

    /// Overlay combat fields and the armour checks of S-0305 (the overlay's
    /// armour stays as written; S-0305 claim 5's forced class is the
    /// simulation's).
    pub(super) fn overlay_combat(&mut self, rec: &mut Rec, t: &TypeRec) {
        self.cfields(rec, t, Kind::Overlays, OVERLAY_COMBAT);
        let name = format!("overlays {}", show(&t.name));
        for (v, layer, line) in sets(key(t, "Armor")) {
            let (class, matched) = ArmourClass::from_word(&v);
            if bool_as_of(t, "Tiberium", layer, false) {
                if class != ArmourClass::Wood || !matched {
                    // spec: S-0305 claim 5 (a resource overlay's armour word other than wood is ignored; the importer warns)
                    self.warn(layer, line, format!(
                        "{name} is a resource overlay: its Armor {:?} is ignored and its class is wood (S-0305 claim 5)",
                        show(&v)
                    ));
                }
            } else if !matched {
                // spec: S-0305 claim 4 (the importer warns for every value that matches no word)
                self.warn(layer, line, format!(
                    "{name} Armor {:?} is not an armour word; the type is given class none (S-0305 claim 4)",
                    show(&v)
                ));
            }
        }
    }

    /// Terrain-object combat fields.
    pub(super) fn terrain_combat(&mut self, rec: &mut Rec, t: &TypeRec) {
        self.cfields(rec, t, Kind::TerrainObjects, TERRAIN_COMBAT);
        if let Some(v) = self.terrain_strength(t) {
            rec.put("strength", int_node(v));
        }
        if let Some(word) = self.armour(Kind::TerrainObjects, t) {
            rec.put("armor", jsonc::string(word));
        }
    }

    /// A terrain type's strength (S-0102 claim 23, S-0009 claim 42): unset
    /// until a layer gives it; after each layer that reads the type's
    /// section, a strength still unset takes `[General] TreeStrength` as it
    /// stands in that layer (25 when no layer gives it), and −1 unsets it
    /// again. With `quirks.tree_strength_fixed_at_first_section` off, a type
    /// left unset by its own key takes the tree strength in force after the
    /// last layer. `None`: unset (no layer read the section), which the
    /// simulation reads as −1.
    fn terrain_strength(&mut self, t: &TypeRec) -> Option<i32> {
        let state = key(t, "Strength");
        let tree = self.global(GENERAL, "TreeStrength");
        let tree_at = |upto: Option<usize>| {
            // spec: S-0102 Import mapping (TreeStrength absent: 25)
            fold(tree.as_ref(), upto, |v, _| value::integer(v)).unwrap_or(TREE_STRENGTH_DEFAULT)
        };
        let unset = |v: Option<i32>| v.filter(|&x| x != UNSET_STRENGTH);
        if t.reads.is_empty() {
            // spec: S-0102 claim 23 (a type whose section no layer holds keeps its strength unset)
            return None;
        }
        if !self.i.sim_quirks.tree_strength_fixed_at_first_section {
            // spec: S-0102 Edge cases (switch off: the tree strength in force after the last layer)
            let own = unset(fold(state, None, |v, _| value::integer(v)));
            return Some(own.unwrap_or_else(|| tree_at(None)));
        }
        let mut held: Option<i32> = None;
        for &r in &t.reads {
            for e in state.into_iter().flat_map(|s| &s.history) {
                if e.layer != r {
                    continue;
                }
                match &e.what {
                    // spec: S-0003 claims 3 and 8 (an unreadable integer keeps the value in force); S-0102 claim 23 (−1 unsets)
                    What::Set(v) => {
                        if let Some(x) = value::integer(v) {
                            held = unset(Some(x));
                        }
                    }
                    What::Reset => held = None,
                    _ => {}
                }
            }
            if held.is_none() {
                // spec: S-0102 claim 23, S-0006 claim 5 (after a layer holding the section, an unset strength takes the tree strength as it stands then; an explicit −1 refills)
                held = Some(tree_at(Some(r)));
            }
        }
        held
    }

    /// Weapon combat fields and their warnings.
    pub(super) fn weapon_combat(&mut self, rec: &mut Rec, t: &TypeRec) {
        self.cfields(rec, t, Kind::Weapons, WEAPON_COMBAT);
        let name = format!("weapons {}", show(&t.name));
        for (field, k) in LASER_COLOURS {
            // spec: S-0615 Import mapping (LaserInnerColor, LaserOuterColor, LaserOuterSpread: three integer colour channels, S-0003; each absent (0, 0, 0); unread components keep the value in force)
            let mut rgb = [0u8; 3];
            let mut given = false;
            for e in key(t, k).into_iter().flat_map(|s| &s.history) {
                match &e.what {
                    What::Set(v) => {
                        let (next, unread) = value::byte_triple(v, rgb);
                        rgb = next;
                        given = true;
                        if unread {
                            self.warn(e.layer, e.line, format!(
                                "{name} {k} {:?} has unreadable components; they keep the value in force (S-0003 claim 11, a departure)",
                                show(v)
                            ));
                        }
                    }
                    What::Reset => {
                        rgb = [0; 3];
                        given = false;
                    }
                    _ => {}
                }
            }
            if given {
                Self::put_tuple(rec, field, rgb.map(i32::from));
            }
        }
        let effect = ["UseSparkParticles", "UseFireParticles", "IsRailgun"]
            .iter()
            .any(|k| bool_of(key(t, k)) == Some(true));
        if effect && !rec.0.iter().any(|(f, _)| f == "attached_particle_system") {
            // spec: S-0616 Import mapping (an effect weapon without AttachedParticleSystem: the original fails; our engine makes no emitter, with an importer warning)
            let line = t.reads.last().copied().unwrap_or(t.created_in);
            self.warn(line, 0, format!(
                "{name} is a spark, fire or railgun weapon without an AttachedParticleSystem: our engine makes no emitter at its shots (S-0616 Import mapping, a departure)"
            ));
        }
        // spec: S-0519 claim 23 and Import mapping (a suicide weapon with no game-wide demolition warhead: the self-hit is still delivered, our engine completes the death deterministically; the importer warns)
        if bool_of(key(t, "Suicide")) == Some(true)
            && self.global(COMBAT_DAMAGE, "C4Warhead").is_none()
        {
            self.warn_at(key(t, "Suicide"), format!(
                "{name} is a suicide weapon but no rules layer gives [CombatDamage] C4Warhead: the self-hit is delivered without a warhead and the death completes with the ordinary presentation and no warhead-selected special death effect (S-0519 claim 23, a design departure)"
            ));
        }
        // spec: S-0301 Edge cases (a burst count below 1 is treated as 1; the importer warns)
        self.each_int(key(t, "Burst"), |b| b < 1, |v| {
            format!("{name} Burst {v} is below 1; our engine treats it as 1 (S-0301 Edge cases, a departure)")
        });
        // spec: S-0301 Edge cases (the importer warns for a negative rate-of-fire field)
        self.each_int(
            key(t, "ROF"),
            |r| r < 0,
            |v| {
                format!("{name} ROF {v} is negative: the delay is over at once (S-0301 Edge cases)")
            },
        );
        // spec: S-0302 Edge cases (a damage magnitude above 2^24 changes when narrowed to single precision)
        self.each_int(key(t, "Damage"), |d| d.unsigned_abs() > DAMAGE_WARN.unsigned_abs(), |v| {
            format!("{name} Damage {v} is above 2^24 in magnitude: single precision changes it (S-0302 Edge cases)")
        });
        for k in ["Range", "MinimumRange"] {
            // spec: S-0307 Edge cases (negative range fields other than −1 and the unlimited −2: reproduced, the importer warns)
            self.each(
                key(t, k),
                // −1 is a "not given" value and never reaches here (S-0307 claim 1).
                |v| {
                    decimal_of(v).is_some_and(|x| {
                        let r = leptons(&x);
                        r.negative() && r.int != UNLIMITED
                    })
                },
                |v| format!("{name} {k} {v} is negative: targets are out of range unless other terms reach them (S-0307 Edge cases)"),
            );
        }
    }

    /// Projectile combat fields, art `Voxel` and their warnings.
    pub(super) fn projectile_combat(&mut self, rec: &mut Rec, index: usize, t: &TypeRec) {
        let kind = Kind::Projectiles;
        self.cfields(rec, t, kind, PROJECTILE_COMBAT);
        // spec: S-0306 Terms and Import mapping (art Voxel of the projectile image: a boolean; the
        // image of S-0007 claims 26 and 27, the same voxel-drawn property the drawing uses)
        let read = self.art_bool(kind, index, t, "Voxel");
        let voxel = match self
            .i
            .arts
            .and_then(|a| a.get(&(kind, index)))
            .and_then(|a| a.projectile.as_ref())
        {
            Some(look) => (look.voxel || read.is_some()).then_some(look.voxel),
            None => read,
        };
        if let Some(b) = voxel {
            rec.put("voxel", jsonc::boolean(b));
        }
        let name = format!("projectiles {}", show(&t.name));
        // spec: S-0306 Edge cases (a negative turn-rate field: reproduced, the importer warns)
        self.each_int(key(t, "ROT"), |r| r < 0, |v| {
            format!("{name} ROT {v} is negative: the projectile flies by its other flags but homes (S-0306 Edge cases)")
        });
        // spec: S-0306 Edge cases (a negative course lock duration: reproduced, the importer warns)
        self.each_int(key(t, "CourseLockDuration"), |r| r < 0, |v| {
            format!("{name} CourseLockDuration {v} is negative: the lock never ends (S-0306 Edge cases)")
        });
        // spec: S-0306 Edge cases (a cluster count of 0 or less: reproduced, the importer warns)
        self.each_int(key(t, "Cluster"), |r| r <= 0, |v| {
            format!("{name} Cluster {v} is not above 0: the projectile leaves play without detonating (S-0306 Edge cases)")
        });
        let vertical = bool_of(key(t, "Vertical")) == Some(true);
        if vertical && int_of(key(t, "DetonationAltitude")).unwrap_or(0) == 0 {
            // spec: S-0306 Edge cases (detonation altitude 0 on a vertical projectile: reproduced, the importer warns)
            self.warn_at(key(t, "Vertical"), format!(
                "{name} is vertical with a detonation altitude of 0: it detonates in its first turn (S-0306 Edge cases)"
            ));
        }
        let steering = int_of(key(t, "ROT")).unwrap_or(0) > 0;
        if bool_of(key(t, "Arcing")) == Some(true) && (steering || vertical) {
            // spec: S-0306 Edge cases (the arcing flag on a steering or vertical type: reproduced, the importer warns)
            self.warn_at(key(t, "Arcing"), format!(
                "{name} is arcing and also steering or vertical: its shots are void in almost every case (S-0306 Edge cases)"
            ));
        }
        let (trailer, delay) = self.projectile_trailer(t);
        if let Some(Some(n)) = &trailer {
            rec.put("trailer", jsonc::string(n.clone()));
        }
        if let Some((d, line)) = delay {
            rec.put("spawn_delay", int_node(d));
            if d == 0 && matches!(trailer, Some(Some(_))) {
                // spec: S-0306 Edge cases (a spawn delay of 0 with a trailer: no trailer animation, a departure; the importer warns)
                self.art_warn(line, format!(
                    "{name} has a trailer and SpawnDelay 0: our engine creates no trailer animation (S-0306 Edge cases, a departure)"
                ));
            }
        }
    }

    /// A projectile's trailer animation and trailer delay (S-0007 claim 33,
    /// S-0306 Import mapping): read at each type read from the art section
    /// that the read's non-empty rules image names (S-0004 claim 14), never
    /// from the rules section. The trailer is `None` while no read gave
    /// one, `Some(None)` once a no-type name cleared it; the delay comes
    /// with the art line that gave it.
    fn projectile_trailer(&self, t: &TypeRec) -> (Option<Option<String>>, Option<(i32, u32)>) {
        let mut trailer = None;
        let mut delay = None;
        for &r in &t.reads {
            // spec: S-0007 claim 33, S-0004 claim 14 (no image value, or an empty one: neither setting is read)
            let Some(image) = crate::layers::projectile_trailer_image(t, r) else {
                continue;
            };
            let Some(sec) = self.i.art.section(image) else {
                continue;
            };
            if let Some(v) = sec.value(b"Trailer") {
                let n = value::text(v, value::COMMON_TEXT_LIMIT);
                // spec: S-0007 claim 33 (an empty trailer keeps the held one; a no-type name clears it)
                if value::is_no_type(n) {
                    trailer = Some(None);
                } else if !n.is_empty() {
                    trailer = Some(self.ref_name(Kind::Animations, n));
                }
            }
            if let Some(e) = sec.get(b"SpawnDelay") {
                // spec: S-0007 claim 33, S-0003 claim 3 (an integer; missing or empty keeps the held delay)
                if let Some(d) = value::integer(&e.value) {
                    delay = Some((d, e.line));
                }
            }
        }
        (trailer, delay)
    }

    /// Warhead combat fields, the versus list and their warnings.
    pub(super) fn warhead_combat(&mut self, rec: &mut Rec, t: &TypeRec) {
        self.cfields(rec, t, Kind::Warheads, WARHEAD_COMBAT);
        if let Some(v) = self.verses(t) {
            rec.put("verses", v);
        }
        // spec: S-0607 Import mapping and claim 22 (warhead Locomotor: a locomotor identifier read as S-0207 claim 14 reads a type's locomotor field, blanks trimmed; a value without the form of a class identifier, an empty one included, is ignored as if the key were absent, so the value in force from an earlier layer stays; an identifier table T3 does not list is kept as unknown, design)
        let loco = fold(key(t, "Locomotor"), None, |v, _| {
            let id = class_identifier(v)?;
            Some(
                LOCOMOTORS
                    .iter()
                    .find(|l| l.0 == id)
                    .map_or("unknown", |l| l.1),
            )
        });
        if let Some(name) = loco {
            rec.put("locomotor", jsonc::string(name));
        }
        let name = format!("warheads {}", show(&t.name));
        // spec: S-0615 claim 3 and Import mapping (a negative InfDeath acts as kind 0 in the ordinary dispatch; bad data: the importer warns)
        self.each_int(
            key(t, "InfDeath"),
            |v| v < 0,
            |v| format!("{name} InfDeath {v} is negative: it acts as death kind 0 (no animation, removed at once), an own death animation list uses entry 0 and a \"not human\" type makes its die-1 request (S-0615 claim 3)"),
        );
        // spec: S-0607 claim 22 (an unknown identifier, design: the importer warns, and a hit is treated as with a locomotor that cannot stand in temporarily)
        self.each(
            key(t, "Locomotor"),
            |v| class_identifier(v).is_some_and(|id| !LOCOMOTORS.iter().any(|l| l.0 == id)),
            |v| format!("{name} Locomotor {v} names no known locomotor: the original stops with a fatal error when such a warhead seizes; our engine treats its hit as one with a locomotor that cannot stand in temporarily: the target stops and is not seized (S-0607 claim 22, a design departure)"),
        );
        if bool_of(key(t, "IsLocomotor")) == Some(true) {
            // spec: S-0607 claim 22 (revision 7: a malformed non-empty value on a seizing warhead is ignored and gives an importer warning; on a warhead without the seizing flag the value is unused and does not warn; an empty value makes no entry, S-0001 claim 8, so it reads as absent and gives no warning)
            self.each(
                key(t, "Locomotor"),
                |v| class_identifier(v).is_none(),
                |v| format!("{name} Locomotor \"{v}\" is not a class identifier: it is ignored as if the key were absent, so this seizing warhead installs the locomotor in force from an earlier rules layer, else the teleporting locomotor (S-0607 claim 22)"),
            );
        }
        // spec: S-0303 Edge cases (a spread outside 0 to 11: clamped by our engine, the importer warns)
        self.each_dec(
            key(t, "CellSpread"),
            |x| x.lt(SPREAD_QUIET.0) || x.gt(SPREAD_QUIET.1),
            |v| format!("{name} CellSpread {v} is outside 0 to 11 cells: our engine clamps the scan radius (S-0303 Edge cases, a departure)"),
        );
        if bool_of(key(t, "CausesDelayKill")) == Some(true)
            && dec_of(key(t, "CellSpread")).is_none_or(|s| s.lt(1))
        {
            // spec: S-0312 Edge cases (a delay-kill warhead with a spread below 1, while quirks.delay_kill_whole_cell_spread is on)
            self.warn_at(key(t, "CausesDelayKill"), format!(
                "{name} causes delayed kills with a spread below one cell: with quirks.delay_kill_whole_cell_spread on the delay has no effect (S-0312 F6)"
            ));
        }
        self.debris_warnings(&name, t);
    }

    /// `Verses` (S-0003 claim 13, S-0305 claim 1): eleven doubles in class
    /// order, each written as its exact decimal expansion. A layer that has
    /// the section but not the key resets it (S-0006 claim 5).
    fn verses(&mut self, t: &TypeRec) -> Option<Node> {
        let integer_prefix = self.i.ini_quirks.percent_list_integer_prefix;
        // spec: S-0003 Formulas (Q₅₃ follows quirks.chop_rounding)
        let chop = self.i.sim_quirks.chop_rounding;
        let mut notes = Vec::new();
        let list = fold(key(t, "Verses"), None, |v, e| {
            let (vals, n) = value::percent_list(v, VERSES, integer_prefix, chop);
            notes.push((e.layer, e.line, show(v), n));
            Some(vals)
        })?;
        let name = format!("warheads {} Verses", show(&t.name));
        for (layer, line, v, n) in notes {
            if n.missing > 0 {
                // spec: S-0003 claim 13 (fewer than eleven items: our engine fills 100 %, a departure; the importer warns)
                self.warn(layer, line, format!(
                    "{name} {v:?} has {} of 11 items; the last {} are 100 % (the original stops loading; S-0003 claim 13, a departure)",
                    VERSES - n.missing,
                    n.missing
                ));
            }
            if n.extra > 0 {
                // spec: S-0003 claim 13 (items beyond eleven are ignored; the importer warns)
                self.warn(
                    layer,
                    line,
                    format!(
                        "{name} {v:?} has {} items beyond 11; they are ignored (S-0003 claim 13)",
                        n.extra
                    ),
                );
            }
            if n.no_number > 0 {
                // spec: S-0003 claim 13 (an item with no number gives 0; the importer warns)
                self.warn(
                    layer,
                    line,
                    format!(
                        "{name} {v:?}: {} items have no number and give 0 (S-0003 claim 13)",
                        n.no_number
                    ),
                );
            }
            if n.prefix_differs > 0 {
                // spec: S-0003 claim 13 (quirks.percent_list_integer_prefix on: the importer warns for every `%` item whose decimal differs from its integer prefix)
                self.warn(layer, line, format!(
                    "{name} {v:?}: {} items with `%` use only their integer part (S-0003 claim 13, quirks.percent_list_integer_prefix)",
                    n.prefix_differs
                ));
            }
            if n.above_one > 0 {
                // spec: S-0003 claim 13 and claim 18 (an item without `%` above 1 in magnitude is taken as it is)
                self.warn(layer, line, format!(
                    "{name} {v:?}: {} items without `%` are above 1 in magnitude; they are taken as factors, not percentages (S-0003 claim 18)",
                    n.above_one
                ));
            }
            if n.uncovered {
                // spec: S-0003 Formulas (importer warning test for claim 6's uncovered cases)
                self.warn(layer, line, format!(
                    "{name} {v:?}: the original's result for an item is not characterised exactly (S-0003 claim 13)"
                ));
            }
        }
        Some(jsonc::array(
            list.iter().map(|b| jsonc::string(exact_text(b))).collect(),
        ))
    }

    /// Country combat fields, the veteran lists and their warnings.
    pub(super) fn country_combat(&mut self, rec: &mut Rec, t: &TypeRec) {
        self.cfields(rec, t, Kind::Countries, COUNTRY_COMBAT);
        // spec: S-0311 Import mapping (VeteranInfantry, VeteranUnits, VeteranAircraft: lists of type names)
        for (field, k, kind) in [
            ("veteran_infantry", "VeteranInfantry", Kind::Infantry),
            ("veteran_units", "VeteranUnits", Kind::Vehicles),
            ("veteran_aircraft", "VeteranAircraft", Kind::Aircraft),
        ] {
            let what = format!("countries {} {k}", show(&t.name));
            if let Some(list) = self.name_list(key(t, k), kind, &what) {
                rec.put(field, list);
            }
        }
        let name = format!("countries {}", show(&t.name));
        // spec: S-0302 Edge cases (the country's general armour factor has no effect; the importer warns for a value other than 1.0)
        self.each_dec(key(t, "Armor"), |x| !x.is_one(), |v| {
            format!("{name} Armor {v}: the original ignores the country's armour factor (S-0302 claim 3)")
        });
        // spec: S-0301 Edge cases (the importer warns for a negative factor)
        self.each_dec(key(t, "ROF"), Exact::negative, |v| {
            format!("{name} ROF {v} is negative: delays can be 0 or less (S-0301 Edge cases)")
        });
    }

    /// The difficulty rows (S-0301 claim 3, S-0302 claim 3, S-0309 claim 25
    /// part i). Their keys reset on override (S-0006 claim 5).
    pub(super) fn difficulty(&mut self) -> Node {
        let mut rows = Vec::new();
        for (name, section) in DIFFICULTY_ROWS {
            let s = section.as_bytes();
            let mut rec = Rec::default();
            for (field, k, form) in [
                ("rof", "ROF", Dec),
                ("firepower", "FirePower", Dec),
                ("destroy_walls", "DestroyWalls", Bool),
            ] {
                let state = self.global(s, k);
                let what = format!("[{section}] {k}");
                if let Some(n) = self.field(state.as_ref(), form, &what) {
                    rec.put(field, n);
                }
            }
            let armor = self.global(s, "Armor");
            // spec: S-0302 Edge cases (the difficulty row's armour factor has no effect; the importer warns for a value other than 1.0)
            self.each_dec(armor.as_ref(), |x| !x.is_one(), |v| {
                format!("[{section}] Armor {v}: the original ignores the difficulty row's armour factor (S-0302 claim 3)")
            });
            let rof = self.global(s, "ROF");
            // spec: S-0301 Edge cases (the importer warns for a negative factor)
            self.each_dec(rof.as_ref(), Exact::negative, |v| {
                format!(
                    "[{section}] ROF {v} is negative: delays can be 0 or less (S-0301 Edge cases)"
                )
            });
            if !rec.0.is_empty() {
                rows.push((name.to_owned(), rec.node()));
            }
        }
        jsonc::object(rows)
    }

    /// The game-wide combat fields.
    pub(super) fn general_combat(&mut self, rec: &mut Rec) {
        for &(field, section, k, form, _) in GENERAL_COMBAT {
            let state = self.global(section, k);
            let what = format!("[{}] {k}", show(section));
            let got = self.field(state.as_ref(), form, &what);
            if field == "bridge_explosions"
                && got.as_ref().is_none_or(
                    |n| matches!(&n.value, glasnost_rules::jsonc::Value::Array(v) if v.is_empty()),
                )
            {
                // spec: S-0511 Edge cases (an empty bridge-explosion list: the importer warns)
                self.warn(0, 0, format!(
                    "{what} is empty or given by no rules layer: a lost bridge section makes no explosion draws, and hut demolition takes its draws and creates no explosion (S-0511 claim 9, F2)"
                ));
            }
            if let Some(n) = got {
                rec.put(field, n);
            } else if field == "wake" {
                // spec: S-0504 claim 4 and Import mapping (no wake animation: ships make no wakes and a sinking vessel takes its draws and makes none; the importer warns)
                self.warn(0, 0, format!(
                    "{what} is given by no rules layer: ships make no wakes, and a sinking vessel takes its wake draws and makes no wake (S-0504 claim 4, a design rule)"
                ));
            }
        }
        self.radiation_import(rec);
        // spec: S-0518 Import mapping and F4 (the held support modifier after every layer's read)
        if let Some(d) = self
            .global(GENERAL, "PrismSupportModifier")
            .and_then(|s| s.derived)
        {
            rec.put("prism_support_modifier", int_node(d as i32));
        }
        // spec: S-0303 Import mapping, S-0309 Import mapping (HarvesterUnit: a vehicle-type list)
        let harvesters = self.global(GENERAL, "HarvesterUnit");
        if let Some(list) = self.name_list(
            harvesters.as_ref(),
            Kind::Vehicles,
            "[General] HarvesterUnit",
        ) {
            rec.put("harvester_unit", list);
        }
        for (field, k) in CREWS {
            let state = self.global(GENERAL, k);
            let name = fold(state.as_ref(), None, |v, _| {
                Some(value::text(v, value::COMMON_TEXT_LIMIT).to_vec())
            });
            if let Some(key) = name.and_then(|n| self.type_key(Kind::Infantry, &n)) {
                rec.put(field, jsonc::string(key));
            }
        }
        // spec: S-0309 Import mapping (the five game-wide default coefficients, doubles; engine default 0)
        let defaults: Vec<Option<String>> = COEFFICIENTS
            .iter()
            .map(|(_, k)| {
                let state = self.global(GENERAL, k);
                self.decimal(state.as_ref(), None, &format!("[General] {k}"))
            })
            .collect();
        if defaults.iter().any(Option::is_some) {
            rec.put(
                "coefficient_defaults",
                jsonc::array(
                    defaults
                        .into_iter()
                        .map(|d| jsonc::string(d.unwrap_or_else(|| "0".to_owned())))
                        .collect(),
                ),
            );
        }
        // spec: S-0309 Import mapping (DisabledDisguiseDetectionPercent: comma-separated plain decimal integers)
        let disguise = self.global(GENERAL, "DisabledDisguiseDetectionPercent");
        if let Some(list) = fold(disguise.as_ref(), None, |v, _| {
            Some(
                value::list(v, value::COMMON_TEXT_LIMIT)
                    .into_iter()
                    .map(|i| int_node(value::decimal_integer(i)))
                    .collect::<Vec<_>>(),
            )
        }) {
            rec.put("disguise_detection_percent", jsonc::array(list));
        }
        self.general_warnings();
    }

    /// The radiation colour and the warnings of the `[Radiation]` settings
    /// (S-0525 Import mapping, claim 16 and Edge cases).
    fn radiation_import(&mut self, rec: &mut Rec) {
        let colour = self.global(RADIATION, "RadColor");
        let mut rgb = [0u8; 3]; // spec: S-0525 Import mapping (RadColor absent: the value in force, (0, 0, 0) at a fresh start)
        let mut given = false;
        if let Some(st) = &colour {
            for e in &st.history {
                let What::Set(v) = &e.what else { continue };
                // spec: S-0525 Import mapping (RadColor: a byte triple, S-0003 C7; unread components keep the value in force)
                let (next, unread) = value::byte_triple(v, rgb);
                rgb = next;
                given = true;
                if unread {
                    self.warn(e.layer, e.line, format!(
                        "[Radiation] RadColor {:?} has unreadable components; they keep the value in force (S-0003 claim 11, a departure)",
                        show(v)
                    ));
                }
            }
        }
        if given {
            Self::put_tuple(rec, "rad_color", rgb.map(i32::from));
        }
        for k in ["RadApplicationDelay", "RadLevelDelay", "RadLightDelay"] {
            let state = self.global(RADIATION, k);
            // spec: S-0525 claim 16 (an application, level or light delay of 0 or less: the original stops or acts oddly; the importer warns and our engine takes the safe results)
            self.each_int(state.as_ref(), |n| n <= 0, |v| {
                format!("[Radiation] {k} {v} is 0 or below: the original stops at the first use for 0 (a program error); our engine takes the safe results of S-0525 claim 16 (a departure)")
            });
        }
        let multiple = self.global(RADIATION, "RadDurationMultiple");
        // spec: S-0525 claim 16 (a duration multiple of 0 or less: a field ends at its first update)
        self.each_int(multiple.as_ref(), |n| n <= 0, |v| {
            format!("[Radiation] RadDurationMultiple {v} is 0 or below: a radiation field ends at its first update (S-0525 claim 16)")
        });
        let site = self.global(RADIATION, "RadSiteWarhead");
        let site_given = site
            .as_ref()
            .is_some_and(|s| s.history.iter().any(|e| matches!(e.what, What::Set(_))));
        if !site_given {
            // spec: S-0525 Edge cases (radiation-site warhead absent: the original's behaviour is unknown; our engine makes fields deal no hits, and the importer warns)
            self.warn(0, 0, "[Radiation] RadSiteWarhead is given by no rules layer: radiation fields deal no hits (S-0525 Edge cases, a departure)".to_owned());
        }
    }

    fn general_warnings(&mut self) {
        let cd = |k: &str| format!("[CombatDamage] {k}");
        // spec: S-0519 F3 (a timed delay below 6 ticks or a flicker rate of 0: the original divides by zero while drawing the bomb icon; our engine takes the fixed form, a departure)
        let delay = self.global(COMBAT_DAMAGE, "IvanTimedDelay");
        self.each_int(delay.as_ref(), |n| (0..6).contains(&n), |v| {
            format!("[CombatDamage] IvanTimedDelay {v} is below 6: the bomb icon's frame divisor is 0 in the original, which divides by zero while drawing; our engine substitutes 1 (S-0519 F3, a departure)")
        });
        let flicker = self.global(COMBAT_DAMAGE, "IvanIconFlickerRate");
        self.each_int(flicker.as_ref(), |n| n == 0, |v| {
            format!("[CombatDamage] IvanIconFlickerRate {v} is 0: the original divides by zero while drawing the bomb icon; our engine uses frame bit 0 (S-0519 F3, a departure)")
        });
        let period = self.global(COMBAT_DAMAGE, "DrainMoneyFrameDelay");
        // spec: S-0608 Questions for the project lead (a payout period of exactly 0: the original fails with a division fault; the proposed departure, pending the lead's choice: the importer warns and payouts never happen)
        self.each_int(period.as_ref(), |n| n == 0, |v| {
            format!("[CombatDamage] DrainMoneyFrameDelay {v} is 0: the original fails with a division fault when a drained resource-destination building pays; our engine makes no payouts (S-0608 F1, a proposed departure)")
        });
        let min = self.global(COMBAT_DAMAGE, "MinDamage");
        // spec: S-0302 Edge cases (the minimum-damage field has no effect; the importer warns when it is given)
        self.each(
            min.as_ref(),
            |_| true,
            |v| {
                format!(
                    "{} {v}: the original ignores the minimum damage (S-0302 claim 6)",
                    cd("MinDamage")
                )
            },
        );
        let max = self.global(COMBAT_DAMAGE, "MaxDamage");
        // spec: S-0302 Edge cases (the importer warns for a maximum damage below 1)
        self.each_int(
            max.as_ref(),
            |m| m < 1,
            |v| {
                format!(
                    "{} {v} is below 1: every capped hit gives the maximum itself (S-0302 F3)",
                    cd("MaxDamage")
                )
            },
        );
        for (section, k) in [
            (GENERAL, "VeteranROF"),
            (COMBAT_DAMAGE, "OccupyROFMultiplier"),
            (COMBAT_DAMAGE, "BunkerROFMultiplier"),
        ] {
            let state = self.global(section, k);
            // spec: S-0301 Edge cases (the importer warns for a negative factor)
            self.each_dec(state.as_ref(), Exact::negative, |v| {
                format!(
                    "[{}] {k} {v} is negative: delays can be 0 or less (S-0301 Edge cases)",
                    show(section)
                )
            });
        }
        for k in ["VeteranRatio", "VeteranCap"] {
            let state = self.global(GENERAL, k);
            // spec: S-0311 Edge cases (a negative veteran ratio or cap: reproduced, with an importer warning)
            self.each_dec(state.as_ref(), Exact::negative, |v| {
                format!("[General] {k} {v} is negative: experience gains or the cap are negative (S-0311 Edge cases)")
            });
        }
        for (_, k) in COEFFICIENTS {
            let state = self.global(GENERAL, k);
            // spec: S-0309 Edge cases (the importer warns for a coefficient magnitude above 10^6)
            self.each_dec(
                state.as_ref(),
                |x| x.gt(COEFFICIENT_WARN) || x.lt(-COEFFICIENT_WARN),
                |v| format!("[General] {k} {v} is above 10^6 in magnitude: threat scores can wrap (S-0309 Edge cases)"),
            );
        }
        let stages = self.global(GENERAL, "CloakingStages");
        // spec: S-0516 claim 43 and Import mapping (a stage count of 0 or below is invalid: our engine uses 1; the importer warns)
        self.each_int(
            stages.as_ref(),
            |n| n <= 0,
            |v| format!("[General] CloakingStages {v} is 0 or below: our engine uses 1 (S-0516 claim 43, a departure)"),
        );
        let disguise = self.global(GENERAL, "DisabledDisguiseDetectionPercent");
        // spec: S-0309 Import mapping (a list without a second item: our engine lets no disguised target through by the draw, a departure; the importer warns)
        self.each(
            disguise.as_ref(),
            |v| value::list(v, value::COMMON_TEXT_LIMIT).len() < 2,
            |v| format!("[General] DisabledDisguiseDetectionPercent {v:?} has no second item: no disguised target is let through by the draw (S-0309 claim 25, a departure)"),
        );
        let fires = self.global(GENERAL, "DamageFireTypes");
        if list_of(fires.as_ref(), self.i.ini_quirks.list_items_untrimmed)
            .unwrap_or_default()
            .len()
            < FIRE_TYPES
        {
            // spec: S-0302 Edge cases (the importer warns when the damage-fire list has fewer than three entries)
            self.warn_at(fires.as_ref(), "[General] DamageFireTypes has fewer than three animations: a building fire of a missing kind is not started (S-0302 claim 13, a departure)".to_owned());
        }
        if self.global(COMBAT_DAMAGE, "DeathWeapon").is_none() {
            // spec: S-0607 claim 18 (no death weapon, design: the importer warns, and each impact detonation becomes a plain area detonation of its warhead)
            self.warn(0, 0, "[CombatDamage] DeathWeapon is given by no rules layer: a magnetic-lift impact detonates its two warheads as plain area detonations, without projectile effects; the original stops with a fatal error (S-0607 claim 18, a design departure)".to_owned());
        }
        let splash = self.global(COMBAT_DAMAGE, "SplashList");
        if list_of(splash.as_ref(), self.i.ini_quirks.list_items_untrimmed)
            .unwrap_or_default()
            .is_empty()
        {
            // spec: S-0312 Edge cases, S-0317 claim 11 and S-0319 claim 8 (an empty splash list: no splash, a departure; the importer warns)
            self.warn_at(splash.as_ref(), "[CombatDamage] SplashList is empty: a vehicle dropped into water shows no splash (S-0312 claim 11, a departure), and neither does a debris piece or a bouncing animation that ends in water (S-0317 claim 11, S-0319 claim 8, departures)".to_owned());
        }
        let gravity = self.global(AUDIO_VISUAL, "Gravity");
        // spec: S-0306 Edge cases (the importer warns about a gravity of 0)
        self.each_int(gravity.as_ref(), |g| g == 0, |_| {
            "[AudioVisual] Gravity is 0: shot speeds and leads of zero-turn-rate projectiles are 0 (S-0306 claim 1)".to_owned()
        });
        let bob = self.global(GENERAL, "HoverBob");
        // spec: S-0503 Edge cases (a bob period trunc(m × q × 900) of 0 is taken as 1 tick; the importer warns; m = 1 gives the shorter period)
        self.each(
            bob.as_ref(),
            |v| {
                decimal_of(v).is_some_and(|q| {
                    // spec: S-0503 F6 (900 ticks per game minute)
                    let p = q.mul_rz(&Binary::from_i64(Format::Double, 900), Format::Double);
                    Exact::of(&p).int == 0
                })
            },
            |v| format!("[General] HoverBob {v} gives a bob period of 0 ticks: our engine takes it as 1 tick, with no bob (S-0503 Edge cases, a departure)"),
        );
        let step = self.global(GENERAL, "RepairStep");
        // spec: S-0314 Edge cases (a building repair step below 1 is raised to 1; the importer warns)
        self.each_int(step.as_ref(), |s| s < 1, |v| {
            format!("[General] RepairStep {v} is below 1: our engine uses 1 (S-0314 Edge cases, a departure)")
        });
        for k in ["SelfHealInfantryFrames", "SelfHealUnitFrames"] {
            let state = self.global(GENERAL, k);
            // spec: S-0314 Edge cases (an interval of 0 is treated as 1 tick; a negative one is reproduced; the importer warns), S-0111 claim 16 (the marker blinks with |T|, never at T = 0; the importer warns for both)
            self.each_int(state.as_ref(), |n| n <= 0, |v| {
                format!("[General] {k} {v} is not above 0: 0 is taken as 1 tick, a negative interval acts as its magnitude (S-0314 Edge cases); the self-heal marker blinks with that magnitude, and never at 0 (S-0111 claim 16)")
            });
        }
        for k in ["SelfHealInfantryAmount", "SelfHealUnitAmount"] {
            let state = self.global(GENERAL, k);
            // spec: S-0314 Edge cases (a negative self-heal amount is treated as 0; the importer warns)
            self.each_int(state.as_ref(), |n| n < 0, |v| {
                format!("[General] {k} {v} is negative: our engine reads it as 0 (S-0314 Edge cases, a departure)")
            });
        }
        let rate = self.global(GENERAL, "RepairRate");
        let ticks = Binary::from_i64(Format::Double, TICKS_PER_MINUTE);
        // spec: S-0314 Edge cases (a building repair period below one tick is taken as 1; a negative period is reproduced; the importer warns)
        self.each(
            rate.as_ref(),
            |v| {
                decimal_of(v).is_some_and(|r| {
                    let p = Exact::of(&r.mul_rz(&ticks, Format::Double));
                    p.int == 0 || p.negative()
                })
            },
            |v| format!("[General] RepairRate {v} gives a building repair period below one tick or negative: our engine uses at least 1 tick (S-0314 Edge cases)"),
        );
        for (crew, divisor, absent) in SIDE_CREWS {
            let d = self.global(GENERAL, divisor);
            let c = self.global(GENERAL, crew);
            let found =
                text_of(c.as_ref()).is_some_and(|n| self.find_rec(Kind::Infantry, &n).is_some());
            if int_of(d.as_ref()).unwrap_or(absent) > 0 && !found {
                // spec: S-0312 Edge cases (a missing crew type for a side with a survivor divisor above 0: no survivor, a departure)
                self.warn_at(c.as_ref().or(d.as_ref()), format!(
                    "[General] {crew} names no infantry type while {divisor} is above 0: our engine releases no survivor there (S-0312 F4, a departure)"
                ));
            }
        }
        // spec: S-0408 Edge cases (A crew choice that always gives an engineer) and Terms (the importer warns for a side crew type or a technician type that is an engineer type)
        for k in ["AlliedCrew", "SovietCrew", "ThirdCrew", "Technician"] {
            let state = self.global(GENERAL, k);
            let engineer = text_of(state.as_ref())
                .and_then(|n| self.find_rec(Kind::Infantry, &n))
                .is_some_and(|t| bool_of(key(t, "Engineer")) == Some(true));
            if engineer {
                self.warn_at(state.as_ref(), format!(
                    "[General] {k} names an engineer type: a sale or destruction survivor choice that gives it repeats, so a sale releases no survivor for that attempt (S-0408 Edge cases, a departure)"
                ));
            }
        }
        // spec: S-0601 Import mapping (the three overload lists: the importer warns when damage or frames is shorter than count, or when count is empty while the others are given)
        let untrimmed = self.i.ini_quirks.list_items_untrimmed;
        let lists: Vec<(&str, Option<KeyState>)> =
            ["OverloadCount", "OverloadDamage", "OverloadFrames"]
                .iter()
                .map(|k| (*k, self.global(COMBAT_DAMAGE, k)))
                .collect();
        let len = |st: &Option<KeyState>| {
            list_of(st.as_ref(), untrimmed)
                .map_or(0, |l| l.iter().filter(|x| !x.is_empty()).count())
        };
        let n = len(&lists[0].1);
        for (k, st) in &lists[1..] {
            if st.is_some() && n == 0 {
                self.warn_at(st.as_ref(), format!(
                    "[CombatDamage] {k} is given but OverloadCount is empty: our engine gives no overload pulses (S-0601 Import mapping, a design fallback)"
                ));
            } else if n > 0 && len(st) < n {
                self.warn_at(st.as_ref().or(lists[0].1.as_ref()), format!(
                    "[CombatDamage] {k} is shorter than OverloadCount: our engine uses 0 for a missing entry (S-0601 Import mapping, a design fallback)"
                ));
            }
        }
        for k in ["RepairPercent", "CrewEscape"] {
            let state = self.global(GENERAL, k);
            // spec: S-0003 claim 18, Import mapping row C9 (a percentage field without `%` above 1 in magnitude)
            self.each(
                state.as_ref(),
                |v| !v.contains(&b'%') && decimal_of(v).is_some_and(|b| { let x = Exact::of(&b); x.gt(1) || x.lt(-1) }),
                |v| format!("[General] {k} {v:?} has no `%`: it is taken as a fraction, not a percentage (S-0003 claim 18)"),
            );
        }
        // spec: S-0601 claim 8 and Import mapping (revision 9: the importer warns once for each absent fate-table mark when at least one of the four tables is non-empty; an absent mark never selects its table, our engine's design rule)
        let table = [
            "AICaptureNormal",
            "AICaptureWounded",
            "AICaptureLowPower",
            "AICaptureLowMoney",
        ]
        .iter()
        .map(|k| self.global(GENERAL, k))
        .find(|st| len(st) > 0);
        if let Some(table) = table {
            for (k, never) in [
                ("AICaptureLowMoneyMark", "the low-money table"),
                ("AICaptureWoundedMark", "the wounded table"),
            ] {
                if self.global(GENERAL, k).is_none() {
                    self.warn_at(table.as_ref(), format!(
                        "[General] {k} is absent while a capture fate table is given: our engine never selects {never} (S-0601 claim 8, a design rule)"
                    ));
                }
            }
        }
    }

    /// Warnings that look at several records' values in force.
    /// The gas-cloud checks (S-0613 Edge cases): a virus-death animation
    /// without a particle type, a wind direction outside the table while a
    /// particle type has a wind effect, a wind divisor of 0, a shared gas
    /// system with a positive lifetime (claim 17), and a cell animation that
    /// requests particles while loading, before the shared system exists.
    pub(super) fn gas_checks(&mut self) {
        let outcome = self.i.outcome;
        let empty = Vec::new();
        let virus = self.global(GENERAL, "InfantryVirus");
        if let Some(name) =
            text_of(virus.as_ref()).filter(|n| !n.is_empty() && !value::is_no_type(n))
        {
            let spawns = find_in(outcome, Kind::Animations, &name)
                .and_then(|t| self.art_section(&t.name))
                .and_then(|sec| sec.value(b"SpawnsParticle").map(<[u8]>::to_vec))
                .and_then(|v| self.particle_ref(&v, crate::refs::NEXT_PARTICLE_LIMIT));
            if spawns.is_none() {
                // spec: S-0613 Edge cases (a virus-death animation without a particle type: the original reads outside its type list; our engine creates no particle and the importer warns)
                self.warn_at(virus.as_ref(), format!(
                    "[General] InfantryVirus {}: the animation names no particle type; the original's result is undefined, our engine creates no gas particle (S-0613 Edge cases, a departure)",
                    show(&name)
                ));
            }
        }
        let particles = outcome.registries.get(&Kind::Particles).unwrap_or(&empty);
        let mut windy = false;
        for t in particles {
            let w = int_of(key(t, "WindEffect")).unwrap_or(0);
            let smoke =
                text_of(key(t, "BehavesLike")).is_some_and(|b| b.eq_ignore_ascii_case(b"Smoke"));
            // spec: S-0613 Edge cases (gas: a positive wind effect) and S-0616 claim 7 (smoke: any nonzero wind effect) with a direction outside 0 … 7: no push, the importer warns
            windy |= w > 0 || (smoke && w != 0);
            // spec: S-0613 F6 and S-0205 claim 3 (W above 10 makes the cadence divisor 0: the original fails; our engine takes the remainder as the tick number and the importer warns)
            if w > 10 {
                self.warn_at(key(t, "WindEffect"), format!(
                    "particle {} WindEffect {w}: the wind cadence divisor is 0, which divides by zero in the original; our engine pushes only at tick 0 (S-0613 F6, S-0205 claim 3)",
                    show(&t.name)
                ));
            }
        }
        let wind = self.global(GENERAL, "WindDirection");
        // spec: S-0613 Import mapping and Edge cases (WindDirection absent: −1)
        let direction = int_of(wind.as_ref()).unwrap_or(-1);
        if windy && !(0..8).contains(&direction) {
            // spec: S-0613 Edge cases (a wind direction outside 0–7 while a type has a wind effect: the original reads outside its table; our engine applies no push and the importer warns)
            self.warn_at(wind.as_ref(), format!(
                "[General] WindDirection {direction} is outside 0–7 while a particle type has a wind effect: the original's result is undefined, our engine applies no wind push (S-0613 Edge cases, a departure)"
            ));
        }
        let systems = outcome
            .registries
            .get(&Kind::ParticleSystems)
            .unwrap_or(&empty);
        // spec: S-0613 Import mapping (the shared gas system's type is the particle-system type named GasCloudSys)
        if let Some(t) = systems
            .iter()
            .find(|t| value::type_name_eq(&t.name, b"GasCloudSys"))
        {
            let lifetime = int_of(key(t, "Lifetime"));
            if let Some(l) = lifetime.filter(|&l| l > 0) {
                // spec: S-0613 claim 17 (the shared system leaves play after a positive lifetime and is not created again; the original then stops on the next particle request, our engine creates no particle; the importer warns when its lifetime is positive)
                self.warn_at(key(t, "Lifetime"), format!(
                    "particle system {} Lifetime {l}: the shared gas system leaves play after {l} turns and is not created again; the original then stops on the next virus death or particle request, our engine creates no gas particle (S-0613 claim 17, a departure)",
                    show(&t.name)
                ));
            }
        }
        let spark = self.global(COMBAT_DAMAGE, "DefaultSparkSystem");
        if spark.is_none() {
            // spec: S-0601 Import mapping (DefaultSparkSystem absent: the overload pulse makes no spark systems and takes no spark placement draws, and the importer warns), S-0616 Import mapping (no game-wide spark type) and S-0316 claim 6 (no electric-bolt sparks, and the importer warns)
            self.warn_at(None, "[CombatDamage] DefaultSparkSystem is absent: the original's result is not established; our engine makes no overload, drone, electric-bolt or other game-wide spark systems and takes no overload spark placement draws (S-0601 Import mapping, S-0316 claim 6, a design fallback)".to_owned());
        }
        self.loading_particle_checks();
    }

    /// A cell animation that requests particles at its creation while the
    /// map loads (μ = 0, S-0109 claim 14, S-0009 claim 12) meets no shared
    /// gas system (S-0613 claim 1 and Edge cases). μ is known only when the
    /// art lookup ran. Theater tile animations are not seen here (a seam).
    fn loading_particle_checks(&mut self) {
        let outcome = self.i.outcome;
        let Some(arts) = self.i.arts else {
            return;
        };
        let empty = Vec::new();
        let anims = outcome.registries.get(&Kind::Animations).unwrap_or(&empty);
        let overlays = outcome.registries.get(&Kind::Overlays).unwrap_or(&empty);
        let mut seen: Vec<usize> = Vec::new();
        for o in overlays {
            let Some(name) =
                text_of(key(o, "CellAnim")).filter(|n| !n.is_empty() && !value::is_no_type(n))
            else {
                continue;
            };
            let Some(i) = anims
                .iter()
                .position(|a| value::type_name_eq(&a.name, &name))
            else {
                continue;
            };
            if seen.contains(&i) {
                continue;
            }
            // spec: S-0109 claims 14 and 25 (μ = ⌊n / 2⌋ of the last sprite found; 0 when none was found)
            let mu = arts.get(&(Kind::Animations, i)).and_then(|a| {
                match (&a.middle_sprite, a.middle_sprite_frames) {
                    (None, _) => Some(0),
                    (Some(_), n) => n.map(|n| n / 2),
                }
            });
            if mu != Some(0) {
                continue;
            }
            let Some(sec) = self.art_section(&anims[i].name) else {
                continue;
            };
            let spawns = sec
                .value(b"SpawnsParticle")
                .map(<[u8]>::to_vec)
                .and_then(|v| self.particle_ref(&v, crate::refs::NEXT_PARTICLE_LIMIT));
            // spec: S-0202 Import mapping (NumParticles: an integer count; absent 0)
            let count = sec
                .value(b"NumParticles")
                .and_then(value::integer)
                .unwrap_or(0);
            if spawns.is_none() || count <= 0 {
                continue;
            }
            seen.push(i);
            // spec: S-0613 Edge cases (a particle requested before the shared system exists, at loading: the original stops; our engine takes the creation draws and creates no particle; the importer warns when such an animation exists)
            self.warn_at(key(o, "CellAnim"), format!(
                "overlay {} CellAnim {}: the animation requests particles when it is created while the map loads, before the shared gas system exists; the original stops there, our engine takes the draws and creates no particle (S-0613 Edge cases, a departure)",
                show(&o.name),
                show(&anims[i].name)
            ));
        }
    }

    /// The pip scale's import warnings (S-0111 claims 12 to 14, F2, Edge
    /// cases and Import mapping): bad data whose pip rows the original
    /// leaves unspecified or fails on, and our engine draws safely.
    fn pip_scale_checks(&mut self) {
        let outcome = self.i.outcome;
        let empty = Vec::new();
        // spec: S-0111 Import mapping (PipScale vocabulary; Pip and OccupyPip vocabulary)
        const KINDS: [&str; 6] = [
            "none",
            "ammo",
            "tiberium",
            "passengers",
            "power",
            "mindcontrol",
        ];
        const PIPS: [&str; 11] = [
            "green",
            "yellow",
            "white",
            "red",
            "blue",
            "persongreen",
            "personyellow",
            "personwhite",
            "personred",
            "personblue",
            "personpurple",
        ];
        // spec: S-0003 claim 16 (named values read as text cut to the field's limit, 31 or 127 bytes; every word here is shorter than 31)
        const LIMIT: usize = 31;
        let mut out: Vec<(Option<KeyState>, String)> = Vec::new();
        for kind in [
            Kind::Infantry,
            Kind::Vehicles,
            Kind::Aircraft,
            Kind::Buildings,
        ] {
            for t in outcome.registries.get(&kind).unwrap_or(&empty) {
                let name = show(&t.name);
                let word = text_of(key(t, "PipScale"));
                let pip_kind = word.as_deref().map(|w| value::named(w, LIMIT, &KINDS));
                if let Some(None) = pip_kind {
                    // spec: S-0111 Import mapping (an unknown PipScale word gives the kind none, with a warning)
                    out.push((
                        key(t, "PipScale").cloned(),
                        format!("{name} has a PipScale that matches no pip-scale kind: our engine reads it as none (S-0111 Import mapping)"),
                    ));
                }
                let pip_kind = pip_kind.flatten().unwrap_or(0);
                // spec: S-0111 claim 15 (a negative pip wrap is kept as read and shows no ammunition pip; the importer warns)
                if int_of(key(t, "PipWrap")).is_some_and(|w| w < 0) {
                    out.push((
                        key(t, "PipWrap").cloned(),
                        format!("{name} has a negative PipWrap: it shows no ammunition pips (S-0111 claim 15)"),
                    ));
                }
                // spec: S-0111 Import mapping (Passengers absent 0, S-0507 and S-0605)
                let capacity = int_of(key(t, "Passengers")).unwrap_or(0);
                // spec: S-0111 Edge cases (the importer warns about a type with a passenger capacity whose pip-scale kind is not passengers)
                if capacity > 0 && pip_kind != 3 {
                    out.push((
                        key(t, "Passengers").cloned(),
                        format!("{name} carries passengers but its pip-scale kind is not passengers: its passenger row takes that kind's row length (S-0111 claim 12 and Edge cases)"),
                    ));
                }
                let primary = text_of(key(t, "Primary"))
                    .filter(|n| !n.is_empty() && !value::is_no_type(n))
                    .and_then(|n| find_in(outcome, Kind::Weapons, &n));
                // spec: S-0111 claim 13 (the mind-control kind and no primary weapon: M = 0; the importer warns)
                if pip_kind == 5 && primary.is_none() {
                    out.push((
                        key(t, "PipScale").cloned(),
                        format!("{name} has the mind-control pip-scale kind and no primary weapon: the original fails drawing its pips; our engine draws a row of 0 pips (S-0111 claim 13, a departure)"),
                    ));
                }
                // spec: S-0111 F2 (a resource pip scale and no storage capacity, Storage absent 0: no resource pips; the importer warns)
                if pip_kind == 2 && int_of(key(t, "Storage")).unwrap_or(0) == 0 {
                    out.push((
                        key(t, "PipScale").cloned(),
                        format!("{name} has a resource pip scale and no storage capacity: our engine draws no resource pips (S-0111 F2, a departure)"),
                    ));
                }
                // spec: S-0111 T2 (row length by kind: none 0, ammo the ammunition capacity (absent −1, S-0301 Import mapping), resource 3 or 5, passengers the capacity, power 10, mind control the primary's damage or 0)
                let row = match pip_kind {
                    1 => int_of(key(t, "Ammo")).unwrap_or(-1),
                    2 => {
                        if kind == Kind::Infantry {
                            3
                        } else {
                            5
                        }
                    }
                    3 => capacity,
                    4 => 10,
                    5 => primary.map_or(0, |w| int_of(key(w, "Damage")).unwrap_or(0)),
                    _ => 0,
                };
                // spec: S-0111 claim 12 (a gunner transport whose row length is 0: the original's pip 0 is unspecified; our engine draws none; the importer warns)
                if capacity > 0 && bool_of(key(t, "Gunner")) == Some(true) && row <= 0 {
                    out.push((
                        key(t, "Gunner").cloned(),
                        format!("{name} is a gunner transport whose passenger row length is {row}: our engine draws no passenger pips (S-0111 claim 12, a departure)"),
                    ));
                }
                let absorbing = kind == Kind::Buildings
                    && (bool_of(key(t, "InfantryAbsorb")) == Some(true)
                        || bool_of(key(t, "UnitAbsorb")) == Some(true));
                // spec: S-0111 claim 12 (more passengers in an absorbing building than its row length: our engine shows the R most recent; checked at import as a capacity above the row length)
                if absorbing && capacity > row.max(0) {
                    out.push((
                        key(t, "Passengers").cloned(),
                        format!("{name} absorbs up to {capacity} passengers but its passenger row has {} pips: our engine shows only the most recent ones (S-0111 claim 12)", row.max(0)),
                    ));
                }
                if matches!(kind, Kind::Infantry | Kind::Vehicles) {
                    // spec: S-0111 claim 12 (the importer warns about a passenger-capable type with a size below 1)
                    if dec_of(key(t, "Size")).is_some_and(|z| z.lt(1)) {
                        out.push((
                            key(t, "Size").cloned(),
                            format!("{name} has a passenger size below 1: as a passenger its pips can reach below pip 0, which our engine does not draw (S-0111 claim 12)"),
                        ));
                    }
                }
                if kind == Kind::Infantry {
                    for k in ["Pip", "OccupyPip"] {
                        // spec: S-0111 Import mapping (a Pip or OccupyPip word that matches none gives image frame 1, with an importer warning)
                        if text_of(key(t, k))
                            .is_some_and(|w| value::named(&w, LIMIT, &PIPS).is_none())
                        {
                            out.push((
                                key(t, k).cloned(),
                                format!("{name} has a {k} that matches no pip colour: image frame 1 is shown (S-0111 Import mapping)"),
                            ));
                        }
                    }
                }
            }
        }
        for (at, text) in out {
            self.warn_at(at.as_ref(), text);
        }
    }

    pub(super) fn cross_checks(&mut self) {
        self.pip_scale_checks();
        let outcome = self.i.outcome;
        let empty = Vec::new();
        let weapons = outcome.registries.get(&Kind::Weapons).unwrap_or(&empty);
        let rad = |k: &str| int_of(self.global(RADIATION, k).as_ref()).unwrap_or(0);
        let (m, d_level, d_light) = (
            rad("RadDurationMultiple"),
            rad("RadLevelDelay"),
            rad("RadLightDelay"),
        );
        // spec: S-0519 Edge cases (a type with a positive strike size and no normal strike aircraft type stops the original at the first start; our engine treats it as no designator, a departure)
        for kind in [
            Kind::Infantry,
            Kind::Vehicles,
            Kind::Aircraft,
            Kind::Buildings,
        ] {
            for t in outcome.registries.get(&kind).unwrap_or(&empty) {
                let given = text_of(key(t, "AirstrikeTeamType"))
                    .is_some_and(|n| !n.is_empty() && !value::is_no_type(&n));
                if int_of(key(t, "AirstrikeTeam")).unwrap_or(0) > 0 && !given {
                    self.warn_at(key(t, "AirstrikeTeam"), format!(
                        "{} has a positive AirstrikeTeam but no AirstrikeTeamType: our engine treats it as no designator (S-0519 Edge cases, a departure)",
                        show(&t.name)
                    ));
                }
            }
        }
        // spec: S-0601 claim 4 and Import mapping (a takeover weapon whose firer gets no control record from its rookie primary; a takeover primary with damage 0 or below that is not unlimited: the importer warns)
        let takeover = |n: Option<Vec<u8>>| -> Option<(i32, bool)> {
            let w = n
                .filter(|n| !n.is_empty() && !value::is_no_type(n))
                .and_then(|n| find_in(outcome, Kind::Weapons, &n))?;
            let h =
                text_of(key(w, "Warhead")).and_then(|n| find_in(outcome, Kind::Warheads, &n))?;
            (bool_of(key(h, "MindControl")) == Some(true)).then(|| {
                (
                    int_of(key(w, "Damage")).unwrap_or(0),
                    bool_of(key(w, "InfiniteMindControl")) == Some(true),
                )
            })
        };
        let mut mind_warnings = Vec::new();
        for kind in [
            Kind::Infantry,
            Kind::Vehicles,
            Kind::Aircraft,
            Kind::Buildings,
        ] {
            for t in outcome.registries.get(&kind).unwrap_or(&empty) {
                let primary = takeover(text_of(key(t, "Primary")));
                let mut others: Vec<String> = ["Secondary", "ElitePrimary", "EliteSecondary"]
                    .iter()
                    .map(|k| (*k).to_owned())
                    .collect();
                others.extend((1..=SLOTS).map(|n| format!("Weapon{n}")));
                others.extend((1..=SLOTS).map(|n| format!("EliteWeapon{n}")));
                let other = others
                    .iter()
                    .find(|k| takeover(text_of(key(t, k))).is_some());
                match (primary, other) {
                    (None, Some(k)) => mind_warnings.push((
                        key(t, k).cloned(),
                        format!(
                            "{} carries a takeover weapon in {k} but its rookie Primary gives it no control record: our engine never fires that weapon (S-0601 claim 4, a departure)",
                            show(&t.name)
                        ),
                    )),
                    (Some((d, false)), _) if d <= 0 => mind_warnings.push((
                        key(t, "Primary").cloned(),
                        format!(
                            "{} has a takeover Primary with damage {d} and no unlimited control: it can take nothing (S-0601 Import mapping)",
                            show(&t.name)
                        ),
                    )),
                    _ => {}
                }
            }
        }
        for (at, text) in mind_warnings {
            self.warn_at(at.as_ref(), text);
        }
        for w in weapons {
            let name = format!("weapons {}", show(&w.name));
            let level = int_of(key(w, "RadLevel")).unwrap_or(0);
            if level > 0 && m > 0 {
                // spec: S-0525 Edge cases and claim 16 (a field duration with zero light steps, or zero level steps: the importer warns)
                let n = level.wrapping_mul(m);
                if d_light > 0 && n / d_light == 0 {
                    self.warn_at(key(w, "RadLevel"), format!(
                        "{name} has radiation level {level}: its field lasts {n} ticks, shorter than the light delay {d_light}, so the original stops when the field is made; our engine keeps the light as made (S-0525 claim 16, a departure)"
                    ));
                }
                if d_level > 0 && n / d_level == 0 {
                    self.warn_at(key(w, "RadLevel"), format!(
                        "{name} has radiation level {level}: its field has no level step, so its radiation never decays and adding to it removes nothing (S-0525 claims 5 and 6)"
                    ));
                }
            }
            let warhead =
                text_of(key(w, "Warhead")).filter(|n| !n.is_empty() && !value::is_no_type(n));
            let Some(h) = warhead
                .as_ref()
                .and_then(|n| find_in(outcome, Kind::Warheads, n))
            else {
                if !w.reads.is_empty() {
                    // spec: S-0303 Edge cases (a projectile without a warhead does nothing; the importer warns about a weapon whose warhead is absent or names nothing)
                    self.warn_at(key(w, "Warhead"), format!(
                        "{name} has no warhead or names none that exists: its detonations do nothing (S-0303 Edge cases, a departure)"
                    ));
                }
                continue;
            };
            let damage = int_of(key(w, "Damage")).unwrap_or(0);
            if bool_of(key(h, "Psychedelic")) == Some(true) && damage <= 0 {
                // spec: S-0302 Edge cases (the importer warns for a berserk warhead whose damage field is 0 or negative)
                self.warn_at(key(w, "Damage"), format!(
                    "{name} has a berserk warhead and damage {damage}: the berserk time is 0 or less (S-0302 claim 10)"
                ));
            }
            if damage < 0
                && list_of(key(h, "AnimList"), self.i.ini_quirks.list_items_untrimmed)
                    .is_some_and(|l| !l.is_empty())
            {
                // spec: S-0303 Edge cases (negative damage with an animation list: no animation for a negative entry; the importer warns)
                self.warn_at(key(w, "Damage"), format!(
                    "{name} has negative damage and its warhead an animation list: negative entries show no animation (S-0303 F5, a departure)"
                ));
            }
            let projectile =
                text_of(key(w, "Projectile")).and_then(|n| find_in(outcome, Kind::Projectiles, &n));
            if let Some(p) = projectile {
                if int_of(key(p, "ROT")).unwrap_or(0) != 0
                    && int_of(key(w, "Speed")).unwrap_or(0) == 0
                {
                    // spec: S-0306 Edge cases (the importer warns about a weapon with a turn rate other than 0 and speed 0)
                    self.warn_at(key(w, "Speed"), format!(
                        "{name} has speed 0 and a projectile with a turn rate: its shots are not led and a missile stalls (S-0306 Edge cases)"
                    ));
                }
                let flak = ["FlakScatter", "Inaccurate", "Arcing"]
                    .iter()
                    .all(|k| bool_of(key(p, k)) == Some(true));
                let range = fold(key(w, "Range"), None, |v, _| decimal_of(v)).map(|x| leptons(&x));
                if flak && range.is_none_or(Exact::is_zero) {
                    // spec: S-0306 Edge cases (flak range 0: a scatter radius of 0, a departure; the importer warns)
                    self.warn_at(key(w, "Range"), format!(
                        "{name} is a flak-scattering inaccurate arcing weapon with range 0: our engine uses a scatter radius of 0 (S-0306 Edge cases, a departure)"
                    ));
                }
            }
        }
        let projectiles = outcome.registries.get(&Kind::Projectiles).unwrap_or(&empty);
        let elevated = projectiles
            .iter()
            .find(|p| bool_of(key(p, "SubjectToElevation")) == Some(true));
        if let Some(p) = elevated {
            let which = show(&p.name);
            let inc = self.global(ELEVATION_MODEL, "ElevationIncrement");
            if int_of(inc.as_ref()).unwrap_or(0) <= 0 {
                // spec: S-0307 Edge cases (elevation increment 0 or negative with a subject-to-elevation projectile; the importer warns)
                self.warn_at(inc.as_ref(), format!(
                    "[ElevationModel] ElevationIncrement is not above 0 while projectile {which} is subject to elevation: 0 gives no elevation bonus (a departure), a negative value is reproduced (S-0307 Edge cases)"
                ));
            }
            let cap = self.global(ELEVATION_MODEL, "ElevationBonusCap");
            if let Some(c) = dec_of(cap.as_ref()) {
                if c.negative() {
                    // spec: S-0307 Edge cases (a negative elevation cap: reproduced, the importer warns)
                    self.warn_at(cap.as_ref(), "[ElevationModel] ElevationBonusCap is negative: every floor-bound pair gets a negative bonus (S-0307 Edge cases)".to_owned());
                }
                if c.int >= CAP_WARN {
                    // spec: S-0307 Edge cases (|trunc(c)| ≥ 148 with a subject-to-elevation projectile: the limits can wrap)
                    self.warn_at(cap.as_ref(), "[ElevationModel] ElevationBonusCap is 148 or more in magnitude: range limits can wrap (S-0307 Edge cases)".to_owned());
                }
            }
        }
        let overlays = outcome.registries.get(&Kind::Overlays).unwrap_or(&empty);
        let explodes = overlays
            .iter()
            .find(|o| bool_of(key(o, "Explodes")) == Some(true));
        if let Some(o) = explodes
            && find_in(outcome, Kind::Animations, BARREL_FIRE).is_none()
        {
            // spec: S-0312 Import mapping (no animation type named FIRE3: our engine creates no barrel fire; the importer warns)
            self.warn_at(key(o, "Explodes"), format!(
                "overlays {} explodes but no animation type is named FIRE3: our engine shows no barrel fire (S-0312 claim 23, a departure)",
                show(&o.name)
            ));
        }
        // spec: S-0511 Import mapping (the importer warns when a required bound low-bridge piece type is missing)
        let lowest = LOW_BRIDGE_BASES.iter().min().copied().unwrap_or(0);
        let highest = LOW_BRIDGE_BASES
            .iter()
            .map(|&b| b + LOW_BRIDGE_LAST)
            .max()
            .unwrap_or(0);
        if overlays.len() > lowest && overlays.len() <= highest {
            self.warn(0, 0, format!(
                "[OverlayTypes] has {} types: the low-bridge pieces bound from types {lowest} to {highest} are not all present, so low bridges past the list cannot change (S-0511 Import mapping)",
                overlays.len()
            ));
        }
    }
}

/// The base piece types t₀ of the four low-bridge palettes (S-0202
/// Formulas, table *Bridge piece type*).
// spec: S-0202 Formulas (bridge piece base types 74, 83, 205, 214)
const LOW_BRIDGE_BASES: [usize; 4] = [74, 83, 205, 214];

/// The largest displacement from t₀ that S-0511 binds (the destroyed piece
/// along x).
const LOW_BRIDGE_LAST: usize = 26; // spec: S-0511 Import mapping (X along x: displacement 26)

/// Three integers of a type key through its layers (S-0003 claim 9: the
/// components a value does not give keep the value in force, our engine;
/// a reset forgets it), or `None` when no layer gives the key.
fn int_triple(t: &TypeRec, k: &str) -> Option<[i32; 3]> {
    let mut held: Option<[i32; 3]> = None;
    for e in t
        .keys
        .get(k.as_bytes())
        .into_iter()
        .flat_map(|s| &s.history)
    {
        match &e.what {
            What::Set(v) => {
                // spec: S-0003 claim 9 (unread components keep the value in force, our engine)
                let (out, _) = value::int_tuple_or::<3>(v, held.unwrap_or([0; 3]));
                held = Some(out);
            }
            What::Reset => held = None,
            _ => {}
        }
    }
    held
}
