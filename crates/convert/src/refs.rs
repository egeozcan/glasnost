//! The reference fields that create types (S-0004 claim 13, Import
//! mapping): for each section kind and each game-wide section, the keys
//! that name a type, the kind they name, their form and text limit.
//!
//! The tables are sets in the spec; the order here is our fixed order for
//! keys of one section that create types of one kind (S-0004 claim 4,
//! "within one section": any fixed order), except for the crate rules,
//! whose order claim 4 fixes. Find-only fields create nothing and need no
//! entry; the building superweapon fields are listed in
//! [`SUPERWEAPON_FIELDS`] for the importer's warning only.

use crate::kinds::Kind;

/// Text limit of most reference fields (S-0004 claim 13).
pub const REFERENCE_LIMIT: usize = 127; // spec: S-0004 Constants (reference field text limit)
/// Text limit of the emitted particle type (S-0004 claim 13).
pub const EMITTED_PARTICLE_LIMIT: usize = 63; // spec: S-0004 Constants (emitted particle type limit)
/// Text limit of the next and spawned particle types (S-0004 claim 13).
pub const NEXT_PARTICLE_LIMIT: usize = 31; // spec: S-0004 Constants (next and spawned particle type limit)
/// Text limit of a weapon's attached particle system (S-0004 claims 8
/// and 13).
pub const ATTACHED_SYSTEM_LIMIT: usize = 19; // spec: S-0004 Constants (attached particle system limit)

/// A single reference or a comma-separated list (S-0004 Terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    /// Names one type.
    Single,
    /// Names a list of types; items take effect in item order (claim 13).
    List,
}

/// How a field reads the reserved names (S-0004 claim 2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Names {
    /// `none` and `<none>` mean "no type".
    Standard,
    /// One of the particle and particle-system fields of claim 2: `none`
    /// names a type called none (`quirks.particle_none_is_a_name`),
    /// `<none>` names no type.
    Particle,
}

/// A creating reference field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Field {
    /// The key, as the rules data spells it.
    pub key: &'static str,
    /// The kind it names.
    pub kind: Kind,
    /// Single or list.
    pub form: Form,
    /// Its text limit in bytes.
    pub limit: usize,
    /// How it reads the reserved names.
    pub names: Names,
    /// Read from the art file (claim 13, "fields read from the art file").
    pub art: bool,
}

const fn s(key: &'static str, kind: Kind) -> Field {
    Field {
        key,
        kind,
        form: Form::Single,
        limit: REFERENCE_LIMIT,
        names: Names::Standard,
        art: false,
    }
}

const fn l(key: &'static str, kind: Kind) -> Field {
    Field {
        key,
        kind,
        form: Form::List,
        limit: REFERENCE_LIMIT,
        names: Names::Standard,
        art: false,
    }
}

const fn art(f: Field) -> Field {
    Field { art: true, ..f }
}

const fn particle(f: Field, limit: usize) -> Field {
    Field {
        limit,
        names: Names::Particle,
        ..f
    }
}

use Kind::{
    Aircraft as A, Animations as Anim, Buildings as B, Infantry as I, Overlays as O,
    ParticleSystems as Ps, Particles as P, Projectiles as Pr, Vehicles as V, VoxelAnimations as Vx,
    Warheads as Wh, Weapons as W,
};

// spec: S-0004 Import mapping (fields shared by buildings, vehicles, aircraft and infantry; the weapon fields of claim 13's weapon-slot table are handled apart)
const UNIT_SHARED: &[Field] = &[
    s("DeployingAnim", Anim),
    l("DebrisAnims", Anim),
    l("DestroyAnim", Anim),
    l("Explosion", Anim),
    s("DeploysInto", B),
    l("Dock", B),
    s("AirstrikeTeamType", A),
    s("EliteAirstrikeTeamType", A),
    s("Spawns", A),
    s("PowersUnit", V),
    s("UndeploysInto", V),
    s("UnloadingClass", V),
    s("Enslaves", I),
    s("DeathWeapon", W),
    s("NaturalParticleSystem", Ps),
    s("RefinerySmokeParticleSystem", Ps),
    l("DamageParticleSystems", Ps),
    l("DestroyParticleSystems", Ps),
    l("DebrisTypes", Vx),
];

// spec: S-0004 Import mapping (fields of one kind's own section)
const COUNTRY: &[Field] = &[
    l("VeteranAircraft", A),
    l("VeteranUnits", V),
    l("VeteranInfantry", I),
];
// spec: S-0004 Import mapping (a superweapon)
const SUPERWEAPON: &[Field] = &[s("AuxBuilding", B), s("WeaponType", W)];
// spec: S-0004 Import mapping (an animation, art file; SpawnsParticle 31 bytes, claim 2)
const ANIMATION: &[Field] = &[
    s("BounceAnim", Anim),
    s("ExpireAnim", Anim),
    s("Next", Anim),
    s("Spawns", Anim),
    s("TrailerAnim", Anim),
    s("Warhead", Wh),
    s("TiberiumSpawnType", O),
    particle(s("SpawnsParticle", P), NEXT_PARTICLE_LIMIT),
];
// spec: S-0004 Import mapping (a building; ToOverlay from the art file)
const BUILDING: &[Field] = &[
    s("SecretBuilding", B),
    s("FreeUnit", V),
    s("SecretUnit", V),
    s("SecretInfantry", I),
    art(s("ToOverlay", O)),
];
// spec: S-0004 Import mapping (an aircraft; Trailer from the art file)
const AIRCRAFT: &[Field] = &[art(s("Trailer", Anim))];
// spec: S-0004 Import mapping (an infantry type)
const INFANTRY: &[Field] = &[
    l("DeadBodies", Anim),
    l("DeathAnims", Anim),
    s("EliteOccupyWeapon", W),
    s("OccupyWeapon", W),
];
// spec: S-0004 Import mapping (a weapon; AttachedParticleSystem 19 bytes, claim 2)
const WEAPON: &[Field] = &[
    l("Anim", Anim),
    s("AssaultAnim", Anim),
    s("OccupantAnim", Anim),
    s("OpenToppedAnim", Anim),
    s("Projectile", Pr),
    s("Warhead", Wh),
    particle(s("AttachedParticleSystem", Ps), ATTACHED_SYSTEM_LIMIT),
];
// spec: S-0004 Import mapping (a projectile; Trailer from the art file)
const PROJECTILE: &[Field] = &[
    art(s("Trailer", Anim)),
    s("AirburstWeapon", W),
    s("ShrapnelWeapon", W),
];
// spec: S-0004 Import mapping (a warhead)
const WARHEAD: &[Field] = &[l("AnimList", Anim), s("Particle", Ps), l("DebrisTypes", Vx)];
// spec: S-0004 Import mapping (an overlay)
const OVERLAY: &[Field] = &[s("CellAnim", Anim)];
// spec: S-0004 Import mapping (a particle type; NextParticle 31 bytes, claim 2)
const PARTICLE: &[Field] = &[
    s("Warhead", Wh),
    particle(s("NextParticle", P), NEXT_PARTICLE_LIMIT),
];
// spec: S-0004 Import mapping (a particle system; HoldsWhat 63 bytes, claim 2)
const PARTICLE_SYSTEM: &[Field] = &[particle(s("HoldsWhat", P), EMITTED_PARTICLE_LIMIT)];
// spec: S-0004 Import mapping (a voxel animation; ShareSource is find only)
const VOXEL_ANIMATION: &[Field] = &[
    s("BounceAnim", Anim),
    s("ExpireAnim", Anim),
    s("TrailerAnim", Anim),
    s("Warhead", Wh),
    s("AttachedSystem", Ps),
    s("Spawns", Vx),
];
// spec: S-0004 Import mapping (a tiberium)
const TIBERIUM: &[Field] = &[l("Debris", Anim)];

/// The creating fields a type of `kind` reads from its own section (and,
/// for the fields marked `art`, from its art section), the shared fields
/// of buildings, vehicles, aircraft and infantry first. Terrain objects
/// and smudges have none.
pub fn type_fields(kind: Kind) -> impl Iterator<Item = &'static Field> {
    let (shared, own): (&[Field], &[Field]) = match kind {
        Kind::Countries => (&[], COUNTRY),
        Kind::Superweapons => (&[], SUPERWEAPON),
        Kind::Animations => (&[], ANIMATION),
        Kind::Buildings => (UNIT_SHARED, BUILDING),
        Kind::Aircraft => (UNIT_SHARED, AIRCRAFT),
        Kind::Vehicles => (UNIT_SHARED, &[]),
        Kind::Infantry => (UNIT_SHARED, INFANTRY),
        Kind::Weapons => (&[], WEAPON),
        Kind::Projectiles => (&[], PROJECTILE),
        Kind::Warheads => (&[], WARHEAD),
        Kind::Overlays => (&[], OVERLAY),
        Kind::Particles => (&[], PARTICLE),
        Kind::ParticleSystems => (&[], PARTICLE_SYSTEM),
        Kind::VoxelAnimations => (&[], VOXEL_ANIMATION),
        Kind::Tiberiums => (&[], TIBERIUM),
        Kind::TerrainObjects | Kind::Smudges => (&[], &[]),
    };
    shared.iter().chain(own)
}

/// Whether a type of `kind` has the weapon fields of claim 13's
/// weapon-slot table.
pub fn has_weapon_slots(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::Buildings | Kind::Vehicles | Kind::Aircraft | Kind::Infantry
    )
}

/// The building's find-only superweapon fields (S-0004 claims 3 and 13).
// spec: S-0004 Import mapping (a building: SuperWeapon, SuperWeapon2, find only)
pub const SUPERWEAPON_FIELDS: [&str; 2] = ["SuperWeapon", "SuperWeapon2"];

/// The four standard weapon fields, in the index order of the weapons
/// they create (S-0004 claim 13, weapon-slot table).
// spec: S-0004 claim 13 (weapon-slot table: primary, secondary, elite primary, elite secondary)
pub const STANDARD_WEAPONS: [&str; 4] = ["Primary", "Secondary", "ElitePrimary", "EliteSecondary"];

/// The turret-count field, the weapon-count field and the
/// clear-all-weapons flag (S-0004 Import mapping).
// spec: S-0004 Import mapping (TurretCount, WeaponCount, ClearAllWeapons)
pub const SLOT_KEYS: (&str, &str, &str) = ("TurretCount", "WeaponCount", "ClearAllWeapons");

/// Prefixes of the numbered weapon fields `Weapon`*n* and
/// `EliteWeapon`*n*, in the index order of claim 13 for one *n*.
// spec: S-0004 Import mapping (Weapon n and EliteWeapon n)
pub const NUMBERED_WEAPONS: [&str; 2] = ["Weapon", "EliteWeapon"];

/// A game-wide section that names types, with its creating fields.
pub struct GameWide {
    /// The section name.
    pub section: &'static str,
    /// Its creating fields, in our fixed order (the crate overlays in the
    /// order claim 4 fixes).
    pub fields: &'static [Field],
}

/// The early game-wide sections that name types, in the order claim 4
/// gives them: the AI settings, then the general settings (row 2).
// spec: S-0004 claim 4 (row 2: AI settings before general settings), Import mapping
pub const EARLY: &[GameWide] = &[
    GameWide {
        section: "AI",
        fields: AI,
    },
    GameWide {
        section: "General",
        fields: GENERAL,
    },
];

/// The late game-wide sections that name types, in the order claim 4
/// gives them (row 4). The difficulty sections and the elevation and wall
/// models name no types.
// spec: S-0004 claim 4 (row 4: crate rules, combat damage, radiation, audio-visual settings, special weapons), Import mapping
pub const LATE: &[GameWide] = &[
    GameWide {
        section: "CrateRules",
        fields: CRATE_RULES,
    },
    GameWide {
        section: "CombatDamage",
        fields: COMBAT_DAMAGE,
    },
    GameWide {
        section: "Radiation",
        fields: RADIATION,
    },
    GameWide {
        section: "AudioVisual",
        fields: AUDIO_VISUAL,
    },
    GameWide {
        section: "SpecialWeapons",
        fields: SPECIAL_WEAPONS,
    },
];

// spec: S-0004 Import mapping ([AI], all lists of buildings)
const AI: &[Field] = &[
    l("AlliedBaseDefenses", B),
    l("BuildAA", B),
    l("BuildBarracks", B),
    l("BuildConst", B),
    l("BuildDefense", B),
    l("BuildDummy", B),
    l("BuildHelipad", B),
    l("BuildNavalYard", B),
    l("BuildPDefense", B),
    l("BuildPower", B),
    l("BuildRadar", B),
    l("BuildRefinery", B),
    l("BuildTech", B),
    l("BuildWeapons", B),
    l("ConcreteWalls", B),
    l("EWGates", B),
    l("NeutralTechBuildings", B),
    l("NSGates", B),
    l("SovietBaseDefenses", B),
    l("ThirdBaseDefenses", B),
];

// spec: S-0004 Import mapping ([General]; the prerequisite groups are find only)
const GENERAL: &[Field] = &[
    s("BarrelExplode", Anim),
    s("Behind", Anim),
    s("BombParachute", Anim),
    l("BridgeExplosions", Anim),
    s("ChronoBeam", Anim),
    s("ChronoBlast", Anim),
    s("ChronoBlastDest", Anim),
    s("ChronoPlacement", Anim),
    s("ChronoSparkle1", Anim),
    l("DamageFireTypes", Anim),
    l("DeadBodies", Anim),
    s("DominatorFirstAnim", Anim),
    s("DominatorSecondAnim", Anim),
    l("DropPod", Anim),
    s("DropZoneAnim", Anim),
    s("EMPulseSparkles", Anim),
    s("FlamingInfantry", Anim),
    s("ForceShieldInvokeAnim", Anim),
    s("InfantryBrute", Anim),
    s("InfantryExplode", Anim),
    s("InfantryHeadPop", Anim),
    s("InfantryMutate", Anim),
    s("InfantryNuked", Anim),
    s("InfantryVirus", Anim),
    s("IonBeam", Anim),
    s("IonBlast", Anim),
    s("IronCurtainInvokeAnim", Anim),
    l("MetallicDebris", Anim),
    s("MoveFlash", Anim),
    s("NukeTakeOff", Anim),
    s("OreTwinkle", Anim),
    s("Parachute", Anim),
    s("Wake", Anim),
    s("WarpAway", Anim),
    s("WarpIn", Anim),
    s("WarpOut", Anim),
    s("WeaponNullifyAnim", Anim),
    s("WeatherConBoltExplosion", Anim),
    l("WeatherConBolts", Anim),
    l("WeatherConClouds", Anim),
    s("GDIGateOne", B),
    s("GDIGateTwo", B),
    s("GDIPowerPlant", B),
    s("NodAdvancedPower", B),
    s("NodGateOne", B),
    s("NodGateTwo", B),
    s("NodRegularPower", B),
    s("PrismType", B),
    l("RepairBay", B),
    l("SecretBuildings", B),
    l("Shipyard", B),
    s("ThirdPowerPlant", B),
    s("WallTower", B),
    s("CMislType", A),
    s("DMislType", A),
    l("PadAircraft", A),
    s("V3RocketType", A),
    l("BaseUnit", V),
    l("HarvesterUnit", V),
    s("LargeVisceroid", V),
    s("PrerequisiteProcAlternate", V),
    l("SecretUnits", V),
    s("SmallVisceroid", V),
    s("AlliedCrew", I),
    s("AlliedDisguise", I),
    l("AllyParaDropInf", I),
    l("AmerParaDropInf", I),
    l("AnimToInfantry", I),
    s("Engineer", I),
    s("Paratrooper", I),
    s("Pilot", I),
    l("SecretInfantry", I),
    s("SovietCrew", I),
    s("SovietDisguise", I),
    l("SovParaDropInf", I),
    s("Technician", I),
    s("ThirdCrew", I),
    s("ThirdDisguise", I),
    l("YuriParaDropInf", I),
    s("DropPodWeapon", W),
    s("DominatorWarhead", Wh),
    s("LightningWarhead", Wh),
    l("DefaultMirageDisguises", Kind::TerrainObjects),
    s("VeinholeTypeClass", Kind::TerrainObjects),
    s("BarrelParticle", Ps),
    l("BarrelDebris", Vx),
    l("ExplosiveVoxelDebris", Vx),
    s("ScrapVoxelDebris", Vx),
    s("TireVoxelDebris", Vx),
];

// spec: S-0004 claim 4 (the wood-crate overlay first, then the crate overlay, then the water-crate overlay), Import mapping ([CrateRules])
const CRATE_RULES: &[Field] = &[
    s("UnitCrateType", V),
    s("WoodCrateImg", O),
    s("CrateImg", O),
    s("WaterCrateImg", O),
];

// spec: S-0004 Import mapping (the combat-damage section)
const COMBAT_DAMAGE: &[Field] = &[
    s("ControlledAnimationType", Anim),
    s("DrainAnimationType", Anim),
    s("PermaControlledAnimationType", Anim),
    l("SplashList", Anim),
    s("DeathWeapon", W),
    s("C4Warhead", Wh),
    s("CMislEliteWarhead", Wh),
    s("CMislWarhead", Wh),
    s("CrushWarhead", Wh),
    s("DMislEliteWarhead", Wh),
    s("DMislWarhead", Wh),
    s("FlameDamage", Wh),
    s("FlameDamage2", Wh),
    s("IonCannonWarhead", Wh),
    s("IvanWarhead", Wh),
    s("V3EliteWarhead", Wh),
    s("V3Warhead", Wh),
    l("Scorches", Kind::Smudges),
    l("Scorches1", Kind::Smudges),
    l("Scorches2", Kind::Smudges),
    l("Scorches3", Kind::Smudges),
    l("Scorches4", Kind::Smudges),
    s("DefaultDebrisSmokeSystem", Ps),
    s("DefaultFireStreamSystem", Ps),
    s("DefaultLargeGreySmokeSystem", Ps),
    s("DefaultLargeRedSmokeSystem", Ps),
    s("DefaultRepairParticleSystem", Ps),
    s("DefaultSmallGreySmokeSystem", Ps),
    s("DefaultSmallRedSmokeSystem", Ps),
    s("DefaultSparkSystem", Ps),
    s("DefaultTestParticleSystem", Ps),
];

// spec: S-0004 Import mapping ([Radiation])
const RADIATION: &[Field] = &[s("RadSiteWarhead", Wh)];

// spec: S-0004 Import mapping ([AudioVisual]; Smoke is read twice, for two fields), Edge cases
const AUDIO_VISUAL: &[Field] = &[
    s("AtmosphereEntry", Anim),
    s("Dig", Anim),
    s("DropPodPuff", Anim),
    s("LargeFire", Anim),
    l("OnFire", Anim),
    s("SmallFire", Anim),
    s("Smoke", Anim),
    s("Smoke", Anim),
    l("TreeFire", Anim),
    s("VeinAttack", Anim),
];

// spec: S-0004 Import mapping ([SpecialWeapons])
const SPECIAL_WEAPONS: &[Field] = &[
    s("EMPulseProjectile", Pr),
    s("NukeDown", Pr),
    s("NukeProjectile", Pr),
    s("EMPulseWarhead", Wh),
    s("MutateExplosionWarhead", Wh),
    s("MutateWarhead", Wh),
    s("NukeWarhead", Wh),
];

/// The particle-system name the original fixes, created at game start
/// when no particle system of that name exists (S-0004 claim 3).
pub const GAS_CLOUD_SYSTEM: &[u8] = b"GasCloudSys"; // spec: S-0004 Import mapping (fixed particle-system name)
