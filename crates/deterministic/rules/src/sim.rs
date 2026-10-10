//! Typed simulation data: the fields of the user's rules and art data that
//! the simulation reads, converted by the importer (`convert::sim`) or written
//! by hand in our format.
//!
//! Every field is optional: an absent field takes the engine default that the
//! owning spec states, which the simulation applies (and cites) when it
//! builds its indexed tables. Rules layering (S-0006) has already happened:
//! a field holds the value in force after every layer, and the "not given"
//! values that keep a held value never appear here.
//!
//! Decimals are quoted strings in our format (PLAN.md 4.6), read exactly as
//! S-0003 reads the same text ([`Decimal`]). Type records are keyed by type
//! name; their registration order comes from `registries` (S-0004).

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::Deserialize;

/// A decimal written as text, `-?digits[.digits][e±digits][%]`, whose value
/// is the one S-0003 gives that text (claims 6 and 7).
#[derive(Clone, Debug, Default, JsonSchema, PartialEq, Eq)]
#[serde(transparent)]
pub struct Decimal(pub String);

impl<'de> Deserialize<'de> for Decimal {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        glasnost_num::decimal::parse_text(&s).map_err(serde::de::Error::custom)?;
        Ok(Decimal(s))
    }
}

impl Decimal {
    /// The value, read as S-0003 reads the text (a decimal field).
    pub fn value(&self, mode: glasnost_num::Rounding) -> glasnost_num::F64 {
        glasnost_num::decimal::parse(&self.0, glasnost_num::decimal::Field::Decimal, mode)
            .unwrap_or_default()
    }

    /// The value of a single-precision field (S-0003 claim 7).
    pub fn single_value(&self, mode: glasnost_num::Rounding) -> glasnost_num::F64 {
        glasnost_num::decimal::parse(&self.0, glasnost_num::decimal::Field::Single, mode)
            .unwrap_or_default()
    }
}

/// The twelve land types (S-0211 Terms).
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum LandType {
    /// Clear.
    Clear,
    /// Road.
    Road,
    /// Water.
    Water,
    /// Rock.
    Rock,
    /// Wall.
    Wall,
    /// Tiberium (resource ground).
    Tiberium,
    /// Beach.
    Beach,
    /// Rough.
    Rough,
    /// Ice.
    Ice,
    /// Railroad.
    Railroad,
    /// Tunnel.
    Tunnel,
    /// Weeds.
    Weeds,
}

/// The eight locomotion classes (S-0211 Terms).
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum LocomotionClass {
    /// Foot.
    Foot,
    /// Track.
    Track,
    /// Wheel.
    Wheel,
    /// Hover.
    Hover,
    /// Winged.
    Winged,
    /// Float.
    Float,
    /// Amphibious.
    Amphibious,
    /// Float-on-beach.
    FloatBeach,
}

/// The thirteen movement zones (S-0211 claim 12).
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MovementZone {
    /// Normal.
    Normal,
    /// Crusher.
    Crusher,
    /// Destroyer.
    Destroyer,
    /// Amphibious destroyer.
    AmphibiousDestroyer,
    /// Amphibious crusher.
    AmphibiousCrusher,
    /// Amphibious.
    Amphibious,
    /// Subterranean.
    Subterranean,
    /// Infantry.
    Infantry,
    /// Infantry destroyer.
    InfantryDestroyer,
    /// Fly.
    Fly,
    /// Water.
    Water,
    /// Water and beach.
    WaterBeach,
    /// Crusher-all.
    CrusherAll,
}

/// The eleven locomotors a type's locomotor field can select (S-0207 claim
/// 14, table T3). Absent from every rules layer: the teleporting locomotor.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Locomotor {
    /// Driving (S-0207).
    Drive,
    /// Hovering.
    Hover,
    /// Tunnelling.
    Tunnel,
    /// Walking (infantry walking, S-0208).
    Walk,
    /// Drop pod.
    DropPod,
    /// Flying.
    Fly,
    /// Teleporting.
    Teleport,
    /// Walking vehicle (mechanical walking).
    Mech,
    /// Ship.
    Ship,
    /// Jumpjet.
    Jumpjet,
    /// Rocket.
    Rocket,
}

/// Every simulation field of the ruleset.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SimData {
    /// Game-wide values.
    #[serde(default)]
    pub general: General,
    /// Land-type rows by land type (S-0211 claim 2). A missing row has the
    /// start-up row.
    #[serde(default)]
    pub land_types: BTreeMap<LandType, LandRow>,
    /// Behaviour properties by behaviour name (S-0215 Import mapping: our
    /// names, snake case).
    #[serde(default)]
    pub behaviours: BTreeMap<String, BehaviourProps>,
    /// Countries by type name.
    #[serde(default)]
    pub countries: BTreeMap<String, Country>,
    /// Infantry types by type name.
    #[serde(default)]
    pub infantry: BTreeMap<String, UnitType>,
    /// Vehicle types by type name.
    #[serde(default)]
    pub vehicles: BTreeMap<String, UnitType>,
    /// Aircraft types by type name.
    #[serde(default)]
    pub aircraft: BTreeMap<String, UnitType>,
    /// Building types by type name.
    #[serde(default)]
    pub buildings: BTreeMap<String, BuildingType>,
    /// Overlay types by type name.
    #[serde(default)]
    pub overlays: BTreeMap<String, OverlayType>,
    /// Terrain-object types by type name.
    #[serde(default)]
    pub terrain_objects: BTreeMap<String, TerrainObjectType>,
    /// Weapons by name.
    #[serde(default)]
    pub weapons: BTreeMap<String, WeaponType>,
    /// Projectiles by name.
    #[serde(default)]
    pub projectiles: BTreeMap<String, ProjectileType>,
    /// Warheads by name.
    #[serde(default)]
    pub warheads: BTreeMap<String, WarheadType>,
}

/// Game-wide values (keys of `[General]`, `[AI]`, `[IQ]`, `[AudioVisual]`,
/// the combat-damage section and `[MultiplayerDialogSettings]`).
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct General {
    /// `CloseEnough`, cells (S-0209 F2).
    #[serde(default)]
    pub close_enough: Option<Decimal>,
    /// `[AI] BlockagePathDelay`, ticks (S-0209).
    #[serde(default)]
    pub blockage_path_delay: Option<i32>,
    /// `[AI] PathDelay`, game minutes (S-0212 F3).
    #[serde(default)]
    pub path_delay: Option<Decimal>,
    /// `PlayerScatter` (S-0208 claim 11).
    #[serde(default)]
    pub player_scatter: Option<bool>,
    /// `PlayerReturnFire` (S-0215 claim 18).
    #[serde(default)]
    pub player_return_fire: Option<bool>,
    /// `PlayerAutoCrush` (S-0210 claim 11).
    #[serde(default)]
    pub player_auto_crush: Option<bool>,
    /// `Crush`, cells (S-0210 F2).
    #[serde(default)]
    pub crush_range: Option<Decimal>,
    /// `GuardAreaTargetingDelay`, ticks (S-0206 claim 9, S-0215 F6).
    #[serde(default)]
    pub guard_area_targeting_delay: Option<i32>,
    /// `NormalTargetingDelay`, ticks (S-0215 F6).
    #[serde(default)]
    pub normal_targeting_delay: Option<i32>,
    /// `GuardModeStray`, cells (S-0215 F5).
    #[serde(default)]
    pub guard_mode_stray: Option<Decimal>,
    /// `VeteranSpeed` (S-0207 claim 2, S-0208 F5).
    #[serde(default)]
    pub veteran_speed: Option<Decimal>,
    /// `TrackedUphill` (S-0207 claim 9).
    #[serde(default)]
    pub tracked_uphill: Option<Decimal>,
    /// `TrackedDownhill` (S-0207 claim 9).
    #[serde(default)]
    pub tracked_downhill: Option<Decimal>,
    /// `WheeledUphill` (S-0207 claim 9).
    #[serde(default)]
    pub wheeled_uphill: Option<Decimal>,
    /// `WheeledDownhill` (S-0207 claim 9).
    #[serde(default)]
    pub wheeled_downhill: Option<Decimal>,
    /// `CliffBackImpassability` (S-0211 claim 8).
    #[serde(default)]
    pub cliff_back_impassability: Option<i32>,
    /// `[AudioVisual] ConditionYellow` (S-0108 F3, S-0207 claim 9).
    #[serde(default)]
    pub condition_yellow: Option<Decimal>,
    /// `[AudioVisual] ConditionRed` (S-0107 F5).
    #[serde(default)]
    pub condition_red: Option<Decimal>,
    /// `[AudioVisual] IdleActionFrequency`, game minutes (S-0107 F4).
    #[serde(default)]
    pub idle_action_frequency: Option<Decimal>,
    /// `MaximumCheerRate`, ticks (S-0213 claim 19).
    #[serde(default)]
    pub maximum_cheer_rate: Option<i32>,
    /// `[IQ] Scatter` (S-0209 claim 11).
    #[serde(default)]
    pub iq_scatter: Option<i32>,
    /// `[IQ] GuardArea` (S-0215 claims 14 and 15).
    #[serde(default)]
    pub iq_guard_area: Option<i32>,
    /// `[IQ] AutoCrush` (S-0210 claim 11; the original's computer houses only).
    #[serde(default)]
    pub iq_auto_crush: Option<i32>,
    /// `[IQ] MaxIQLevels` (S-0209 claim 11).
    #[serde(default)]
    pub iq_max: Option<i32>,
    /// `[MultiplayerDialogSettings] GameSpeed` (S-0201 claim 7).
    #[serde(default)]
    pub lobby_game_speed: Option<i32>,
    /// `EngineerCaptureLevel` (S-0214 F2).
    #[serde(default)]
    pub engineer_capture_level: Option<Decimal>,
    /// `BaseUnit`: the base-unit list, vehicle type keys (S-0009 claim 45).
    #[serde(default)]
    pub base_unit: Option<Vec<String>>,
    /// `[MultiplayerDialogSettings] TechLevel` as the base rules alone give
    /// it: the game's tech level (S-0009 Terms).
    #[serde(default)]
    pub multiplayer_tech_level: Option<i32>,
    /// `GDIGateOne`: the first Allied gate type, a building key (S-0112).
    #[serde(default)]
    pub allied_gate_one: Option<String>,
    /// `GDIGateTwo`: the second Allied gate type (S-0112).
    #[serde(default)]
    pub allied_gate_two: Option<String>,
    /// `NodGateOne`: the first Soviet gate type (S-0112).
    #[serde(default)]
    pub soviet_gate_one: Option<String>,
    /// `NodGateTwo`: the second Soviet gate type (S-0112).
    #[serde(default)]
    pub soviet_gate_two: Option<String>,
    /// `WallTower`: the wall tower type (S-0112).
    #[serde(default)]
    pub wall_tower: Option<String>,
}

/// One land type's row (S-0211 claim 2): multipliers per locomotion class and
/// the buildable flag. A present row was given by some rules layer; its
/// missing keys are 1.0 and *no*.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LandRow {
    /// `Foot`.
    #[serde(default)]
    pub foot: Option<Decimal>,
    /// `Track`.
    #[serde(default)]
    pub track: Option<Decimal>,
    /// `Wheel`.
    #[serde(default)]
    pub wheel: Option<Decimal>,
    /// `Hover`.
    #[serde(default)]
    pub hover: Option<Decimal>,
    /// `Float`.
    #[serde(default)]
    pub float: Option<Decimal>,
    /// `Amphibious`.
    #[serde(default)]
    pub amphibious: Option<Decimal>,
    /// `FloatBeach`.
    #[serde(default)]
    pub float_beach: Option<Decimal>,
    /// `Buildable`.
    #[serde(default)]
    pub buildable: Option<bool>,
}

/// A behaviour's properties (S-0215 claims 6 and 16 to 21).
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BehaviourProps {
    /// `NoThreat`.
    #[serde(default)]
    pub no_threat: Option<bool>,
    /// `Paralyzed`.
    #[serde(default)]
    pub paralysed: Option<bool>,
    /// `Recruitable`.
    #[serde(default)]
    pub recruitable: Option<bool>,
    /// `Retaliate`.
    #[serde(default)]
    pub retaliate: Option<bool>,
    /// `Scatter`.
    #[serde(default)]
    pub scatter: Option<bool>,
    /// `Zombie`.
    #[serde(default)]
    pub zombie: Option<bool>,
    /// `Rate`, game minutes.
    #[serde(default)]
    pub rate: Option<Decimal>,
    /// `AARate`, game minutes (already reset to the rate where S-0215 claim
    /// 21 says).
    #[serde(default)]
    pub aa_rate: Option<Decimal>,
}

/// A country's simulation fields.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Country {
    /// `SpeedUnitsMult` (S-0207 claim 2), a single.
    #[serde(default)]
    pub speed_units_mult: Option<Decimal>,
    /// `SpeedInfantryMult` (S-0208 F5), a single.
    #[serde(default)]
    pub speed_infantry_mult: Option<Decimal>,
    /// `MultiplayPassive` (S-0009, S-0214).
    #[serde(default)]
    pub multiplay_passive: Option<bool>,
    /// The wall-owner flag (S-0210 claim 14, Import mapping).
    #[serde(default)]
    pub wall_owner: Option<bool>,
    /// `MultiplayPassive` as the base rules alone give it: the passive field
    /// the early phase of start positions sees (S-0009 claim 33).
    #[serde(default)]
    pub multiplay_passive_base: Option<bool>,
}

/// One sequence entry of an infantry type (S-0107 claim 1).
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SequenceEntry {
    /// Start image frame S.
    #[serde(default)]
    pub start: i32,
    /// Frame count C.
    #[serde(default)]
    pub count: i32,
    /// Facing multiplier M.
    #[serde(default)]
    pub multiplier: i32,
    /// Declared direction, 0 = N … 7 = NW.
    #[serde(default)]
    pub direction: Option<u8>,
}

/// An infantry, vehicle or aircraft type's simulation fields.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UnitType {
    /// `Strength`.
    #[serde(default)]
    pub strength: Option<i32>,
    /// `Speed`, the speed field v (S-0207 claim 1, S-0208 F4).
    #[serde(default)]
    pub speed: Option<i32>,
    /// `ROT`, the turn-rate field (S-0206 claim 6).
    #[serde(default)]
    pub rot: Option<i32>,
    /// `JumpjetTurnRate` (S-0206 claim 14).
    #[serde(default)]
    pub jumpjet_turn_rate: Option<i32>,
    /// `Turret`.
    #[serde(default)]
    pub turret: Option<bool>,
    /// `TurretSpins`.
    #[serde(default)]
    pub turret_spins: Option<bool>,
    /// The locomotion class as S-0211 claim 3 leaves it; absent: no class.
    #[serde(default)]
    pub locomotion_class: Option<LocomotionClass>,
    /// `MovementZone` (S-0211 claim 4).
    #[serde(default)]
    pub movement_zone: Option<MovementZone>,
    /// `Locomotor`, the locomotor its class identifier selects (S-0207
    /// claim 14); absent: the teleporting locomotor.
    #[serde(default)]
    pub locomotor: Option<Locomotor>,
    /// `MovementRestrictedTo` (S-0211 claim 14); absent: no restriction.
    #[serde(default)]
    pub movement_restricted_to: Option<LandType>,
    /// `Crusher`.
    #[serde(default)]
    pub crusher: Option<bool>,
    /// `OmniCrusher`.
    #[serde(default)]
    pub omni_crusher: Option<bool>,
    /// `OmniCrushResistant`.
    #[serde(default)]
    pub omni_crush_resistant: Option<bool>,
    /// `Crushable`.
    #[serde(default)]
    pub crushable: Option<bool>,
    /// `DeployedCrushable`.
    #[serde(default)]
    pub deployed_crushable: Option<bool>,
    /// `Accelerates`.
    #[serde(default)]
    pub accelerates: Option<bool>,
    /// `AccelerationFactor`.
    #[serde(default)]
    pub acceleration_factor: Option<Decimal>,
    /// `DeaccelerationFactor`.
    #[serde(default)]
    pub deacceleration_factor: Option<Decimal>,
    /// `SlowdownDistance`, leptons.
    #[serde(default)]
    pub slowdown_distance: Option<i32>,
    /// `Passive`.
    #[serde(default)]
    pub passive: Option<bool>,
    /// `IsTrain`.
    #[serde(default)]
    pub is_train: Option<bool>,
    /// `IsSimpleDeployer`.
    #[serde(default)]
    pub is_simple_deployer: Option<bool>,
    /// `Harvester`.
    #[serde(default)]
    pub harvester: Option<bool>,
    /// `Weeder`.
    #[serde(default)]
    pub weeder: Option<bool>,
    /// `JumpJet`.
    #[serde(default)]
    pub jumpjet: Option<bool>,
    /// `Fraidycat`.
    #[serde(default)]
    pub fraidycat: Option<bool>,
    /// `Cyborg`.
    #[serde(default)]
    pub cyborg: Option<bool>,
    /// `Fearless`.
    #[serde(default)]
    pub fearless: Option<bool>,
    /// `NotHuman`.
    #[serde(default)]
    pub not_human: Option<bool>,
    /// `VehicleThief`.
    #[serde(default)]
    pub vehicle_thief: Option<bool>,
    /// `Engineer`.
    #[serde(default)]
    pub engineer: Option<bool>,
    /// `Occupier`.
    #[serde(default)]
    pub occupier: Option<bool>,
    /// `Assaulter`.
    #[serde(default)]
    pub assaulter: Option<bool>,
    /// `Infiltrate`.
    #[serde(default)]
    pub infiltrate: Option<bool>,
    /// `Agent`.
    #[serde(default)]
    pub agent: Option<bool>,
    /// `C4`.
    #[serde(default)]
    pub c4: Option<bool>,
    /// `Ivan`.
    #[serde(default)]
    pub ivan: Option<bool>,
    /// `Deployer`.
    #[serde(default)]
    pub deployer: Option<bool>,
    /// `DeployFire`.
    #[serde(default)]
    pub deploy_fire: Option<bool>,
    /// `UndeployDelay`, ticks.
    #[serde(default)]
    pub undeploy_delay: Option<i32>,
    /// `DefaultToGuardArea`.
    #[serde(default)]
    pub default_to_guard_area: Option<bool>,
    /// `StupidHunt`.
    #[serde(default)]
    pub stupid_hunt: Option<bool>,
    /// `HoverAttack`.
    #[serde(default)]
    pub hover_attack: Option<bool>,
    /// `CloseRange`.
    #[serde(default)]
    pub close_range: Option<bool>,
    /// `DistributedFire`.
    #[serde(default)]
    pub distributed_fire: Option<bool>,
    /// `OpportunityFire`.
    #[serde(default)]
    pub opportunity_fire: Option<bool>,
    /// `OpenTopped`.
    #[serde(default)]
    pub open_topped: Option<bool>,
    /// `NonVehicle`.
    #[serde(default)]
    pub non_vehicle: Option<bool>,
    /// `PreventAttackMove`.
    #[serde(default)]
    pub prevent_attack_move: Option<bool>,
    /// `MoveToShroud`.
    #[serde(default)]
    pub move_to_shroud: Option<bool>,
    /// `Pushy`.
    #[serde(default)]
    pub pushy: Option<bool>,
    /// `PermaDisguise`.
    #[serde(default)]
    pub perma_disguise: Option<bool>,
    /// `AttackFriendlies`.
    #[serde(default)]
    pub attack_friendlies: Option<bool>,
    /// `AttackCursorOnFriendlies`.
    #[serde(default)]
    pub attack_cursor_on_friendlies: Option<bool>,
    /// `SmallVisceroid`.
    #[serde(default)]
    pub small_visceroid: Option<bool>,
    /// `LargeVisceroid` (S-0009 claim 46, S-0202 Import mapping).
    #[serde(default)]
    pub large_visceroid: Option<bool>,
    /// `Cost`, credits (S-0009 claim 45).
    #[serde(default)]
    pub cost: Option<i32>,
    /// `TechLevel` (S-0009 claim 45).
    #[serde(default)]
    pub tech_level: Option<i32>,
    /// `AllowedToStartInMultiplayer` (S-0009 claim 45).
    #[serde(default)]
    pub allowed_to_start_in_multiplayer: Option<bool>,
    /// `Owner`: the countries that own the type, as country keys (S-0009
    /// claim 45; unknown names dropped by the importer, S-0004 claim 3).
    #[serde(default)]
    pub owner: Option<Vec<String>>,
    /// `Immune`.
    #[serde(default)]
    pub immune: Option<bool>,
    /// `LegalTarget`.
    #[serde(default)]
    pub legal_target: Option<bool>,
    /// `Bombable`.
    #[serde(default)]
    pub bombable: Option<bool>,
    /// `ImmuneToRadiation`.
    #[serde(default)]
    pub immune_to_radiation: Option<bool>,
    /// `BalloonHover`.
    #[serde(default)]
    pub balloon_hover: Option<bool>,
    /// `SprayAttack`.
    #[serde(default)]
    pub spray_attack: Option<bool>,
    /// `TiltsWhenCrushes`.
    #[serde(default)]
    pub tilts_when_crushes: Option<bool>,
    /// `AutoCrush`.
    #[serde(default)]
    pub auto_crush: Option<bool>,
    /// `Carryall`.
    #[serde(default)]
    pub carryall: Option<bool>,
    /// `Spawned`.
    #[serde(default)]
    pub spawned: Option<bool>,
    /// `GuardRange`, cells.
    #[serde(default)]
    pub guard_range: Option<Decimal>,
    /// `Sight`, cells.
    #[serde(default)]
    pub sight: Option<i32>,
    /// `Passengers`.
    #[serde(default)]
    pub passengers: Option<i32>,
    /// `Storage`.
    #[serde(default)]
    pub storage: Option<i32>,
    /// `ThreatAvoidanceCoefficient`, a single.
    #[serde(default)]
    pub threat_avoidance_coefficient: Option<Decimal>,
    /// `DeploysInto`.
    #[serde(default)]
    pub deploys_into: Option<String>,
    /// `Primary`.
    #[serde(default)]
    pub primary: Option<String>,
    /// `Secondary`.
    #[serde(default)]
    pub secondary: Option<String>,
    /// `ElitePrimary`.
    #[serde(default)]
    pub elite_primary: Option<String>,
    /// `EliteSecondary`.
    #[serde(default)]
    pub elite_secondary: Option<String>,
    /// `TurretCount`.
    #[serde(default)]
    pub turret_count: Option<i32>,
    /// `WeaponCount`.
    #[serde(default)]
    pub weapon_count: Option<i32>,
    /// `VeteranAbilities`, upper-case ability names.
    #[serde(default)]
    pub veteran_abilities: Vec<String>,
    /// `EliteAbilities`, upper-case ability names.
    #[serde(default)]
    pub elite_abilities: Vec<String>,
    /// Art `Crawls` (S-0107 Import mapping).
    #[serde(default)]
    pub crawls: Option<bool>,
    /// Art sequence entries by our kind name (S-0107 claim 1).
    #[serde(default)]
    pub sequences: BTreeMap<String, SequenceEntry>,
    /// Art fire frames: standing, prone, secondary, secondary-prone (S-0107
    /// claim 11).
    // spec: S-0107 claim 11 (four fire-frame fields)
    #[serde(default)]
    pub fire_frames: Option<[i32; 4]>,
    /// `Teleporter`: a self-teleporting type (S-0214 claim 11).
    #[serde(default)]
    pub teleporter: Option<bool>,
}

/// A building type's simulation fields.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BuildingType {
    /// `Strength`.
    #[serde(default)]
    pub strength: Option<i32>,
    /// Art `Foundation`, width and height in cells.
    // nocite: a pair of numbers (width, height), our format's shape
    #[serde(default)]
    pub foundation: Option<[i32; 2]>,
    /// `Bib` (S-0204 Terms).
    #[serde(default)]
    pub bib: Option<bool>,
    /// `Gate`.
    #[serde(default)]
    pub gate: Option<bool>,
    /// `InvisibleInGame`.
    #[serde(default)]
    pub invisible_in_game: Option<bool>,
    /// `LaserFence`.
    #[serde(default)]
    pub laser_fence: Option<bool>,
    /// `FirestormWall`.
    #[serde(default)]
    pub firestorm_wall: Option<bool>,
    /// `Naval`.
    #[serde(default)]
    pub naval: Option<bool>,
    /// `Bunker`.
    #[serde(default)]
    pub bunker: Option<bool>,
    /// `UnitRepair`.
    #[serde(default)]
    pub unit_repair: Option<bool>,
    /// `Grinding`.
    #[serde(default)]
    pub grinding: Option<bool>,
    /// `UnitAbsorb`.
    #[serde(default)]
    pub unit_absorb: Option<bool>,
    /// `InfantryAbsorb`.
    #[serde(default)]
    pub infantry_absorb: Option<bool>,
    /// `BridgeRepairHut`.
    #[serde(default)]
    pub bridge_repair_hut: Option<bool>,
    /// `NumberImpassableRows` (S-0209 Terms); absent: none.
    #[serde(default)]
    pub number_impassable_rows: Option<i32>,
    /// `Factory`, the kind it builds (`infantry`, `vehicle`, `aircraft`,
    /// `building`).
    #[serde(default)]
    pub factory: Option<String>,
    /// `WeaponsFactory`.
    #[serde(default)]
    pub weapons_factory: Option<bool>,
    /// `ConstructionYard`.
    #[serde(default)]
    pub construction_yard: Option<bool>,
    /// `UndeploysInto`.
    #[serde(default)]
    pub undeploys_into: Option<String>,
    /// `Cloning`.
    #[serde(default)]
    pub cloning: Option<bool>,
    /// `Powered`.
    #[serde(default)]
    pub powered: Option<bool>,
    /// `TogglePower`.
    #[serde(default)]
    pub toggle_power: Option<bool>,
    /// `CanBeOccupied`.
    #[serde(default)]
    pub can_be_occupied: Option<bool>,
    /// `CanOccupyFire`.
    #[serde(default)]
    pub can_occupy_fire: Option<bool>,
    /// `Capturable`.
    #[serde(default)]
    pub capturable: Option<bool>,
    /// `Spyable`.
    #[serde(default)]
    pub spyable: Option<bool>,
    /// `CanC4`.
    #[serde(default)]
    pub can_c4: Option<bool>,
    /// `Repairable`.
    #[serde(default)]
    pub repairable: Option<bool>,
    /// `Hospital`.
    #[serde(default)]
    pub hospital: Option<bool>,
    /// `Armory`.
    #[serde(default)]
    pub armory: Option<bool>,
    /// `SAM`.
    #[serde(default)]
    pub sam: Option<bool>,
    /// `EMPulseCannon`.
    #[serde(default)]
    pub emp_cannon: Option<bool>,
    /// `HasStupidGuardMode`.
    #[serde(default)]
    pub stupid_guard_mode: Option<bool>,
    /// `Turret`.
    #[serde(default)]
    pub turret: Option<bool>,
    /// `ROT`.
    #[serde(default)]
    pub rot: Option<i32>,
    /// `Primary`.
    #[serde(default)]
    pub primary: Option<String>,
    /// `Secondary`.
    #[serde(default)]
    pub secondary: Option<String>,
    /// `TechLevel`.
    #[serde(default)]
    pub tech_level: Option<i32>,
    /// `GapGenerator`.
    #[serde(default)]
    pub gap_generator: Option<bool>,
    /// `Immune`.
    #[serde(default)]
    pub immune: Option<bool>,
    /// `LegalTarget`.
    #[serde(default)]
    pub legal_target: Option<bool>,
    /// `Crushable`.
    #[serde(default)]
    pub crushable: Option<bool>,
    /// `DeployTime`, game minutes: a gate's door time (S-0215 F9).
    #[serde(default)]
    pub deploy_time: Option<Decimal>,
    /// `GateCloseDelay`, game minutes: a gate's close delay (S-0215 F9).
    #[serde(default)]
    pub gate_close_delay: Option<Decimal>,
}

/// An overlay type's simulation fields.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OverlayType {
    /// `Land`; absent: clear (S-0211 claim 4; `<none>` is read as clear by
    /// the importer, with a warning).
    #[serde(default)]
    pub land: Option<LandType>,
    /// `NoUseTileLandType`.
    #[serde(default)]
    pub no_use_tile_land_type: Option<bool>,
    /// `Wall`.
    #[serde(default)]
    pub wall: Option<bool>,
    /// `Tiberium`.
    #[serde(default)]
    pub tiberium: Option<bool>,
    /// `Crushable`.
    #[serde(default)]
    pub crushable: Option<bool>,
    /// `IsARock`.
    #[serde(default)]
    pub rock: Option<bool>,
    /// `IsRubble`.
    #[serde(default)]
    pub rubble: Option<bool>,
    /// `Crate`.
    #[serde(default, rename = "crate")]
    pub crate_: Option<bool>,
    /// `DamageLevels` of the overlay type's image section in the art data
    /// (S-0210 claim 13).
    #[serde(default)]
    pub damage_levels: Option<i32>,
    /// `Strength`.
    #[serde(default)]
    pub strength: Option<i32>,
    /// `Armor`, the armour name as written.
    #[serde(default)]
    pub armor: Option<String>,
    /// Whether the type has artwork or a cell animation (S-0009 claim 12),
    /// as the importer's art lookup found.
    #[serde(default)]
    pub has_art: Option<bool>,
    /// `IsVeins`, the veins flag (S-0202 Terms).
    #[serde(default)]
    pub is_veins: Option<bool>,
}

/// A terrain-object type's simulation fields.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TerrainObjectType {
    /// `TemperateOccupationBits`.
    #[serde(default)]
    pub temperate_occupation_bits: Option<i32>,
    /// `SnowOccupationBits`.
    #[serde(default)]
    pub snow_occupation_bits: Option<i32>,
    /// `SpawnsTiberium`, the spawns-resources flag (S-0203 claim 2).
    #[serde(default)]
    pub spawns_tiberium: Option<bool>,
}

/// A weapon's simulation fields read by M3 rules.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WeaponType {
    /// `Damage`.
    #[serde(default)]
    pub damage: Option<i32>,
    /// `AmbientDamage`.
    #[serde(default)]
    pub ambient_damage: Option<i32>,
    /// `Projectile`.
    #[serde(default)]
    pub projectile: Option<String>,
    /// `Warhead`.
    #[serde(default)]
    pub warhead: Option<String>,
    /// `OmniFire`.
    #[serde(default)]
    pub omni_fire: Option<bool>,
    /// `AreaFire`.
    #[serde(default)]
    pub area_fire: Option<bool>,
    /// `RadLevel`.
    #[serde(default)]
    pub rad_level: Option<i32>,
}

/// A projectile's simulation fields read by M3 rules.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProjectileType {
    /// `ROT`.
    #[serde(default)]
    pub rot: Option<i32>,
    /// `AG`.
    #[serde(default)]
    pub ag: Option<bool>,
}

/// A warhead's simulation fields read by M3 rules.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WarheadType {
    /// `Wall`.
    #[serde(default)]
    pub wall: Option<bool>,
    /// `Wood`.
    #[serde(default)]
    pub wood: Option<bool>,
    /// `IvanBomb`.
    #[serde(default)]
    pub ivan_bomb: Option<bool>,
    /// `Sonic`.
    #[serde(default)]
    pub sonic: Option<bool>,
    /// `Fire`.
    #[serde(default)]
    pub fire: Option<bool>,
    /// `ElectricAssault`.
    #[serde(default)]
    pub electric_assault: Option<bool>,
    /// `InfDeath`.
    #[serde(default)]
    pub inf_death: Option<i32>,
}
