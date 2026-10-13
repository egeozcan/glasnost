//! The combat fields of the typed `sim` section (M4): weapons, projectiles,
//! warheads, the combat fields of object types, countries, the difficulty
//! rows and the game-wide combat values, each read as the Import mapping of
//! the combat spec named on its row says (S-0301 to S-0315), with the
//! importer warnings those specs ask for.

use glasnost_ini::binfloat::{Binary, Format};
use glasnost_rules::sim::ArmourClass;

use super::*;
use crate::art::int_as_of;

/// A table row: our field name, the INI key, the value form and the spec
/// whose Import mapping maps the key.
type Row = (&'static str, &'static str, Form, &'static str);

/// Combat fields shared by infantry, vehicle, aircraft and building types
/// (`glasnost_rules::sim::CombatFields`), written into `combat`.
pub(super) const COMBAT_FIELDS: &[Row] = &[
    ("is_gattling", "IsGattling", Bool, "S-0308"),
    ("is_charge_turret", "IsChargeTurret", Bool, "S-0301, S-0308"),
    ("land_targeting", "LandTargeting", Int, "S-0308"),
    ("naval_targeting", "NavalTargeting", Int, "S-0308"),
    ("underwater", "Underwater", Bool, "S-0308"),
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
        "S-0302",
    ),
    ("immune_to_poison", "ImmuneToPoison", Bool, "S-0302"),
    ("immune_to_psionics", "ImmuneToPsionics", Bool, "S-0302"),
    ("damage_self", "DamageSelf", Bool, "S-0302, S-0303"),
    ("max_debris", "MaxDebris", Int, "S-0312"),
    ("min_debris", "MinDebris", Int, "S-0312"),
    (
        "debris_types",
        "DebrisTypes",
        Refs(Kind::VoxelAnimations),
        "S-0312",
    ),
    ("debris_maximums", "DebrisMaximums", Ints, "S-0312"),
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
    ("airstrike_team", "AirstrikeTeam", Int, "S-0309"),
    ("drainable", "Drainable", Bool, "S-0308"),
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
    ("crashable", "Crashable", Bool, "S-0312"),
    ("weight", "Weight", Dec, "S-0312"),
    (
        "suppression_threshold",
        "SuppressionThreshold",
        Int,
        "S-0315",
    ),
    ("missile_spawn", "MissileSpawn", Bool, "S-0311"),
];

/// Combat fields the Import mappings give for vehicle types only.
const VEHICLE_COMBAT: &[Row] = &[
    ("hunter_seeker", "HunterSeeker", Bool, "S-0301"),
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
    (
        "eligible_for_delay_kill",
        "EligibleForDelayKill",
        Bool,
        "S-0302",
    ),
    ("leave_rubble", "LeaveRubble", Bool, "S-0312"),
    ("helipad", "Helipad", Bool, "S-0312"),
    ("artillary", "Artillary", Bool, "S-0309"),
    ("max_number_occupants", "MaxNumberOccupants", Int, "S-0313"),
    (
        "infantry_gain_self_heal",
        "InfantryGainSelfHeal",
        Int,
        "S-0314",
    ),
    ("units_gain_self_heal", "UnitsGainSelfHeal", Int, "S-0314"),
];

/// Combat fields of overlay types.
const OVERLAY_COMBAT: &[Row] = &[
    ("chain_reaction", "ChainReaction", Bool, "S-0303"),
    ("explodes", "Explodes", Bool, "S-0303"),
];

/// Combat fields of terrain-object types.
const TERRAIN_COMBAT: &[Row] = &[("immune", "Immune", Bool, "S-0302")];

/// The game-wide tree strength's absent-key default (S-0102 claim 23 and
/// Import mapping).
const TREE_STRENGTH_DEFAULT: i32 = 25; // spec: S-0102 Constants (game-wide tree strength default 25)

/// The strength value that unsets a terrain type's strength (S-0102 claim
/// 23).
const UNSET_STRENGTH: i32 = -1; // spec: S-0102 claim 23 (writing −1 unsets the strength)

/// Combat fields of weapons.
const WEAPON_COMBAT: &[Row] = &[
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
    ("is_mag_beam", "IsMagBeam", Bool, "S-0301"),
    ("temporal", "Temporal", Bool, "S-0301"),
    ("spawner", "Spawner", Bool, "S-0301"),
    ("disk_laser", "DiskLaser", Bool, "S-0301"),
    ("decloak_to_fire", "DecloakToFire", Bool, "S-0301"),
    ("lobber", "Lobber", Bool, "S-0306"),
    ("never_use", "NeverUse", Bool, "S-0308"),
    ("drain_weapon", "DrainWeapon", Bool, "S-0308"),
    ("fire_while_moving", "FireWhileMoving", Bool, "S-0310"),
    ("suicide", "Suicide", Bool, "S-0312"),
    ("supress", "Supress", Bool, "S-0309"),
    ("fire_once", "FireOnce", Bool, "S-0309"),
    ("reveal_on_fire", "RevealOnFire", Bool, "S-0409"),
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
    ("trailer", "Trailer", Ref(Kind::Animations), "S-0306"),
    ("spawn_delay", "SpawnDelay", Int, "S-0306"),
];

/// Combat fields of warheads (`Verses` has its own reading).
const WARHEAD_COMBAT: &[Row] = &[
    ("cell_spread", "CellSpread", Dec, "S-0302, S-0303"),
    ("percent_at_max", "PercentAtMax", Dec, "S-0302, S-0303"),
    ("prone_damage", "ProneDamage", Dec, "S-0302"),
    ("penetrates_bunker", "PenetratesBunker", Bool, "S-0302"),
    ("radiation", "Radiation", Bool, "S-0302"),
    ("psychic_damage", "PsychicDamage", Bool, "S-0302"),
    ("poison", "Poison", Bool, "S-0302"),
    ("affects_allies", "AffectsAllies", Bool, "S-0302, S-0303"),
    ("psychedelic", "Psychedelic", Bool, "S-0302"),
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
    ("debris_maximums", "DebrisMaximums", Ints, "S-0303"),
    ("airstrike", "Airstrike", Bool, "S-0308"),
    ("is_locomotor", "IsLocomotor", Bool, "S-0308"),
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

/// Game-wide combat fields: our name, section, key, form, spec.
const GENERAL_COMBAT: &[(&str, &[u8], &str, Form, &str)] = &[
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
    (
        "c4_warhead",
        COMBAT_DAMAGE,
        "C4Warhead",
        Ref(Kind::Warheads),
        "S-0302, S-0303",
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
        "S-0302",
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
    (
        "metallic_debris",
        GENERAL,
        "MetallicDebris",
        Refs(Kind::Animations),
        "S-0303, S-0312",
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
    ("gravity", AUDIO_VISUAL, "Gravity", Int, "S-0306"),
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
const FLH_ABSENT: [i32; 3] = [0, 0, 0]; // spec: S-0313 Import mapping (muzzle positions absent: 0, 0, 0)

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
fn last_set(state: Option<&KeyState>) -> (usize, u32) {
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

fn int_of(state: Option<&KeyState>) -> Option<i32> {
    fold(state, None, |v, _| value::integer(v))
}

fn bool_of(state: Option<&KeyState>) -> Option<bool> {
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

fn list_of(state: Option<&KeyState>) -> Option<Vec<Vec<u8>>> {
    fold(state, None, |v, _| {
        Some(
            value::list(v, value::COMMON_TEXT_LIMIT)
                .into_iter()
                .filter(|i| !value::is_no_type(i))
                .map(<[u8]>::to_vec)
                .collect(),
        )
    })
}

fn key<'a>(t: &'a TypeRec, k: &str) -> Option<&'a KeyState> {
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
            _ => "global:General",
        };
        out.push((cat, (*k).to_owned(), *spec));
    }
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
        "TurretAnimX",
        "TurretAnimY",
    ]);
    for n in 1..=SLOTS {
        flh.push(format!("Weapon{n}FLH"));
        flh.push(format!("EliteWeapon{n}FLH"));
    }
    keyed(&["art"], &flh, "S-0313");
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
    keyed(&["art"], &s(&["TargetCoordOffset", "Height"]), "S-0306");
    keyed(
        &["art"],
        &s(&["TurretAnimIsVoxel", "BarrelAnimIsVoxel"]),
        "S-0310, S-0313",
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

    fn warn_at(&mut self, state: Option<&KeyState>, message: String) {
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
    pub(super) fn art_values(&self, kind: Kind, index: usize, t: &TypeRec, k: &str) -> Vec<(Vec<u8>, u32)> {
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
    fn art_bool(&self, kind: Kind, index: usize, t: &TypeRec, k: &str) -> Option<bool> {
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
        let mut held: Option<[i32; N]> = None;
        for (v, line) in self.art_values(kind, index, t, k) {
            // spec: S-0003 claim 9 (unread components take the value in force's, our engine; the importer warns)
            let (out, unread) = value::int_tuple_or::<N>(&v, held.unwrap_or([0; N]));
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

    /// The combat fields every object type with weapons shares (S-0301 to
    /// S-0315), written into `combat`.
    fn shared_combat(&mut self, combat: &mut Rec, kind: Kind, index: usize, t: &TypeRec) {
        self.cfields(combat, t, kind, COMBAT_FIELDS);
        if let Some(word) = self.armour(kind, t) {
            combat.put("armor", jsonc::string(word));
        }
        self.coefficients(combat, kind, t);
        // spec: S-0313 Import mapping (muzzle positions: three integers of the type's image section)
        for (field, k) in FLH_KEYS {
            if let Some(v) = self.art_tuple::<3>(kind, index, t, k) {
                Self::put_tuple(combat, field, v);
            }
        }
        for (field, prefix) in [
            ("weapon_flh", "Weapon"),
            ("elite_weapon_flh", "EliteWeapon"),
        ] {
            let mut by_slot = Vec::new();
            for n in 1..=SLOTS {
                by_slot.push(self.art_tuple::<3>(kind, index, t, &format!("{prefix}{n}FLH")));
            }
            if let Some(last) = by_slot.iter().rposition(Option::is_some) {
                combat.put(
                    field,
                    jsonc::array(
                        by_slot[..=last]
                            .iter()
                            .map(|f| {
                                let f = f.unwrap_or(FLH_ABSENT);
                                jsonc::array(f.iter().map(|&x| int_node(x)).collect())
                            })
                            .collect(),
                    ),
                );
            }
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
        let types = list_of(key(t, "DebrisTypes")).unwrap_or_default();
        let maxima = fold(key(t, "DebrisMaximums"), None, |v, _| {
            Some(
                value::list(v, value::COMMON_TEXT_LIMIT)
                    .into_iter()
                    .map(|i| value::integer(i).unwrap_or(0))
                    .collect::<Vec<_>>(),
            )
        })
        .unwrap_or_default();
        let metallic = self.global(GENERAL, "MetallicDebris");
        let metallic_empty = list_of(metallic.as_ref()).unwrap_or_default().is_empty();
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
        if !types.is_empty() && maxima.iter().take(types.len()).all(|&m| m == 0) {
            // spec: S-0303 Edge cases, S-0312 Edge cases (every maximum 0 with a positive count: one pass that places nothing)
            self.warn_at(key(t, "DebrisMaximums"), format!(
                "{name} has MaxDebris {max} but every debris maximum is 0: our engine stops after one pass that places nothing (S-0303 F6, a departure)"
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
            // spec: Q-0061 (placeholder: the held coefficient before any layer gives the key is 0)
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
        let name = format!("{} {}", kind.key(), show(&t.name));
        // spec: S-0308 Edge cases (a weapon count above 18: our engine reads the first 18 numbered keys; the importer warns)
        self.each_int(key(t, "WeaponCount"), |n| n > SLOTS as i32, |v| {
            format!("{name} WeaponCount {v} is above 18; only Weapon1 … Weapon18 are read (S-0308 claim 1, a departure)")
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

    /// `DamageParticleSystems` (S-0302 Import mapping): the listed particle
    /// systems whose `BehavesLike` is `Smoke` (letter case ignored), in list
    /// order, as type keys.
    fn smoke_systems(&mut self, combat: &mut Rec, t: &TypeRec) {
        let Some(list) = list_of(key(t, "DamageParticleSystems")) else {
            return;
        };
        let Some(reg) = self.i.outcome.registries.get(&Kind::ParticleSystems) else {
            combat.put("damage_smoke_systems", jsonc::array(Vec::new()));
            return;
        };
        let keys = type_keys(reg);
        let mut out = Vec::new();
        for item in list {
            let Some(i) = reg.iter().position(|p| value::type_name_eq(&p.name, &item)) else {
                continue;
            };
            // spec: S-0302 Import mapping (eligible when BehavesLike is Smoke, letter case ignored; other and unknown words are not)
            let smoke = text_of(key(&reg[i], "BehavesLike"))
                .is_some_and(|b| b.eq_ignore_ascii_case(b"Smoke"));
            if smoke {
                out.push(jsonc::string(keys[i].clone()));
            }
        }
        combat.put("damage_smoke_systems", jsonc::array(out));
    }

    /// The combat fields of an infantry, vehicle or aircraft type.
    pub(super) fn unit_combat(&mut self, rec: &mut Rec, kind: Kind, index: usize, t: &TypeRec) {
        let mut combat = Rec::default();
        self.shared_combat(&mut combat, kind, index, t);
        self.weapon_slots(rec, &mut combat, kind, t);
        Self::numbered_flh_only_with_slots(&mut combat);
        self.smoke_systems(&mut combat, t);
        if !combat.0.is_empty() {
            rec.put("combat", combat.node());
        }
        self.cfields(rec, t, kind, UNIT_COMBAT);
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
        for k in ["VeteranAbilities", "EliteAbilities"] {
            for (v, layer, line) in sets(key(t, k)) {
                for item in value::list(&v, value::COMMON_TEXT_LIMIT) {
                    if !ABILITIES
                        .iter()
                        .any(|a| item.eq_ignore_ascii_case(a.as_bytes()))
                    {
                        // spec: S-0311 Edge cases (items are not trimmed; the importer warns for every item that names no ability)
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
        // spec: S-0302 Import mapping, S-0004 Import mapping (BuildCat: a defence building when the word is Combat; a named value, S-0003 claim 16; placeholder, Q-0061)
        if let Some(d) = fold(key(t, "BuildCat"), None, |v, _| {
            Some(value::named(v, NAME_LIMIT, &["Combat"]).is_some())
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
            ("turret_anim_is_voxel", "TurretAnimIsVoxel"),
            ("barrel_anim_is_voxel", "BarrelAnimIsVoxel"),
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
        // spec: S-0313 Import mapping (TurretAnimX, TurretAnimY: integers, pixels; absent 0)
        let xy = ["TurretAnimX", "TurretAnimY"].map(|k| self.art_int(kind, index, t, k));
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
                // spec: S-0102 claim 23 (after a layer holding the section, an unset strength takes the tree strength as it stands then)
                held = Some(tree_at(Some(r)));
            }
        }
        held
    }

    /// Weapon combat fields and their warnings.
    pub(super) fn weapon_combat(&mut self, rec: &mut Rec, t: &TypeRec) {
        self.cfields(rec, t, Kind::Weapons, WEAPON_COMBAT);
        let name = format!("weapons {}", show(&t.name));
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
        // spec: S-0306 Import mapping (art Voxel of the projectile image: a boolean)
        if let Some(b) = self.art_bool(kind, index, t, "Voxel") {
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
        let trailer =
            text_of(key(t, "Trailer")).is_some_and(|n| !n.is_empty() && !value::is_no_type(&n));
        if trailer && int_of(key(t, "SpawnDelay")) == Some(0) {
            // spec: S-0306 Edge cases (a spawn delay of 0 with a trailer: no trailer animation, a departure; the importer warns)
            self.warn_at(key(t, "SpawnDelay"), format!(
                "{name} has a trailer and SpawnDelay 0: our engine creates no trailer animation (S-0306 Edge cases, a departure)"
            ));
        }
    }

    /// Warhead combat fields, the versus list and their warnings.
    pub(super) fn warhead_combat(&mut self, rec: &mut Rec, t: &TypeRec) {
        self.cfields(rec, t, Kind::Warheads, WARHEAD_COMBAT);
        if let Some(v) = self.verses(t) {
            rec.put("verses", v);
        }
        let name = format!("warheads {}", show(&t.name));
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
        let mut notes = Vec::new();
        let list = fold(key(t, "Verses"), None, |v, e| {
            let (vals, n) = value::percent_list(v, VERSES, integer_prefix);
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
            if let Some(n) = self.field(state.as_ref(), form, &what) {
                rec.put(field, n);
            }
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

    fn general_warnings(&mut self) {
        let cd = |k: &str| format!("[CombatDamage] {k}");
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
        let disguise = self.global(GENERAL, "DisabledDisguiseDetectionPercent");
        // spec: S-0309 Import mapping (a list without a second item: our engine lets no disguised target through by the draw, a departure; the importer warns)
        self.each(
            disguise.as_ref(),
            |v| value::list(v, value::COMMON_TEXT_LIMIT).len() < 2,
            |v| format!("[General] DisabledDisguiseDetectionPercent {v:?} has no second item: no disguised target is let through by the draw (S-0309 claim 25, a departure)"),
        );
        let fires = self.global(GENERAL, "DamageFireTypes");
        if list_of(fires.as_ref()).unwrap_or_default().len() < FIRE_TYPES {
            // spec: S-0302 Edge cases (the importer warns when the damage-fire list has fewer than three entries)
            self.warn_at(fires.as_ref(), "[General] DamageFireTypes has fewer than three animations: a building fire of a missing kind is not started (S-0302 claim 13, a departure)".to_owned());
        }
        let splash = self.global(COMBAT_DAMAGE, "SplashList");
        if list_of(splash.as_ref()).unwrap_or_default().is_empty() {
            // spec: S-0312 Edge cases (an empty splash list: no splash, a departure; the importer warns)
            self.warn_at(splash.as_ref(), "[CombatDamage] SplashList is empty: a vehicle dropped into water shows no splash (S-0312 claim 11, a departure)".to_owned());
        }
        let gravity = self.global(AUDIO_VISUAL, "Gravity");
        // spec: S-0306 Edge cases (the importer warns about a gravity of 0)
        self.each_int(gravity.as_ref(), |g| g == 0, |_| {
            "[AudioVisual] Gravity is 0: shot speeds and leads of zero-turn-rate projectiles are 0 (S-0306 claim 1)".to_owned()
        });
        let step = self.global(GENERAL, "RepairStep");
        // spec: S-0314 Edge cases (a building repair step below 1 is raised to 1; the importer warns)
        self.each_int(step.as_ref(), |s| s < 1, |v| {
            format!("[General] RepairStep {v} is below 1: our engine uses 1 (S-0314 Edge cases, a departure)")
        });
        for k in ["SelfHealInfantryFrames", "SelfHealUnitFrames"] {
            let state = self.global(GENERAL, k);
            // spec: S-0314 Edge cases (an interval of 0 is treated as 1 tick; a negative one is reproduced; the importer warns)
            self.each_int(state.as_ref(), |n| n <= 0, |v| {
                format!("[General] {k} {v} is not above 0: 0 is taken as 1 tick, a negative interval acts as its magnitude (S-0314 Edge cases)")
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
        for k in ["RepairPercent", "CrewEscape"] {
            let state = self.global(GENERAL, k);
            // spec: S-0003 claim 18, Import mapping row C9 (a percentage field without `%` above 1 in magnitude)
            self.each(
                state.as_ref(),
                |v| !v.contains(&b'%') && decimal_of(v).is_some_and(|b| { let x = Exact::of(&b); x.gt(1) || x.lt(-1) }),
                |v| format!("[General] {k} {v:?} has no `%`: it is taken as a fraction, not a percentage (S-0003 claim 18)"),
            );
        }
    }

    /// Warnings that look at several records' values in force.
    pub(super) fn cross_checks(&mut self) {
        let outcome = self.i.outcome;
        let empty = Vec::new();
        let weapons = outcome.registries.get(&Kind::Weapons).unwrap_or(&empty);
        for w in weapons {
            let name = format!("weapons {}", show(&w.name));
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
            if damage < 0 && list_of(key(h, "AnimList")).is_some_and(|l| !l.is_empty()) {
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
    }
}
