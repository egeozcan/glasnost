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
    /// Animation types by type name (their art sections, S-0109).
    #[serde(default)]
    pub animations: BTreeMap<String, AnimationType>,
    /// The difficulty rows by our name (`easy`, `normal`, `difficult`;
    /// S-0302 Terms, glossary *difficulty skill block*).
    #[serde(default)]
    pub difficulty: BTreeMap<String, DifficultyRow>,
    /// Tiberiums by section name (S-0401 Import mapping); the registry
    /// `tiberiums` gives their order (S-0004 claim 10).
    #[serde(default)]
    pub tiberiums: BTreeMap<String, crate::sim_econ::TiberiumType>,
    /// Superweapon types by type name (S-0407 and S-0412 Import mapping).
    #[serde(default)]
    pub superweapons: BTreeMap<String, crate::sim_econ::SuperweaponType>,
}

/// The eleven armour classes in class-index order (S-0305 claim 1).
#[derive(
    Clone, Copy, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[serde(rename_all = "snake_case")]
pub enum ArmourClass {
    /// *none* (0), also the class of a type no layer has given one.
    #[default]
    None,
    /// *flak* (1).
    Flak,
    /// *plate* (2).
    Plate,
    /// *light* (3).
    Light,
    /// *medium* (4).
    Medium,
    /// *heavy* (5).
    Heavy,
    /// *wood* (6).
    Wood,
    /// *steel* (7).
    Steel,
    /// *concrete* (8).
    Concrete,
    /// *special 1* (9).
    #[serde(rename = "special_1")]
    Special1,
    /// *special 2* (10).
    #[serde(rename = "special_2")]
    Special2,
}

impl ArmourClass {
    /// The classes in class-index order (S-0305 claim 1).
    // spec: S-0305 claim 1 (eleven classes, fixed order)
    pub const ALL: [ArmourClass; 11] = [
        ArmourClass::None,
        ArmourClass::Flak,
        ArmourClass::Plate,
        ArmourClass::Light,
        ArmourClass::Medium,
        ArmourClass::Heavy,
        ArmourClass::Wood,
        ArmourClass::Steel,
        ArmourClass::Concrete,
        ArmourClass::Special1,
        ArmourClass::Special2,
    ];

    /// The class index, 0 to 10 (S-0305 Terms).
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|c| *c == self).unwrap_or(0)
    }

    /// The armour words in class-index order (S-0305 Import mapping).
    // spec: S-0305 Import mapping (the eleven armour words)
    pub const WORDS: [&'static str; 11] = [
        "none",
        "flak",
        "plate",
        "light",
        "medium",
        "heavy",
        "wood",
        "steel",
        "concrete",
        "special_1",
        "special_2",
    ];

    /// The class an armour word names, comparing the whole text and folding
    /// A to Z only (S-0305 claims 3 and 4: an unknown word gives *none*).
    pub fn from_word(v: &[u8]) -> (ArmourClass, bool) {
        for (i, w) in Self::WORDS.iter().enumerate() {
            if v.eq_ignore_ascii_case(w.as_bytes()) {
                return (Self::ALL[i], true);
            }
        }
        // spec: S-0305 claim 4 (a word that matches nothing gives class none)
        (ArmourClass::None, false)
    }
}

/// One difficulty row's fields (S-0301 claim 3, S-0302 claim 3, S-0309
/// claim 25 part i).
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DifficultyRow {
    /// `ROF`, the row's rate-of-fire factor ℓ (a double).
    #[serde(default)]
    pub rof: Option<Decimal>,
    /// `FirePower`, the row's firepower factor φᵣ (a double).
    #[serde(default)]
    pub firepower: Option<Decimal>,
    /// `DestroyWalls`, the destroy-walls flag.
    #[serde(default)]
    pub destroy_walls: Option<bool>,
}

/// A muzzle position (F, L, H) in leptons (S-0313 Terms).
// spec: S-0313 Terms (three integers: forward, lateral, up)
pub type Flh = [i32; 3];

/// An animation type's fields that its creation draws read (S-0109; S-0202
/// claim 10).
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AnimationType {
    /// `RandomRate`: the random rate pair (ρ₁, ρ₂), a −1 already replaced by
    /// the end held (S-0109 claim 3).
    // spec: S-0109 Import mapping (RandomRate: two integers)
    #[serde(default)]
    pub random_rate: Option<[i32; 2]>,
    /// `Bouncer`.
    #[serde(default)]
    pub bouncer: Option<bool>,
    /// `IsMeteor`.
    #[serde(default)]
    pub is_meteor: Option<bool>,
    /// `TiberiumChainReaction`: the start event's resource chain reaction.
    #[serde(default)]
    pub tiberium_chain_reaction: Option<bool>,
    /// `Scorch`: a middle-event effect.
    #[serde(default)]
    pub scorch: Option<bool>,
    /// `Crater`: a middle-event effect.
    #[serde(default)]
    pub crater: Option<bool>,
    /// `SpawnsParticle` names a particle type: a middle-event effect.
    #[serde(default)]
    pub spawns_particle: Option<bool>,
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
    /// `[General] AIAutoDeployFrameDelay`, ticks: the auto-deploy delay list
    /// of S-0215 claim 31, entries for difficulty indexes 0, 1 and 2.
    #[serde(default)]
    pub ai_auto_deploy_frame_delay: Option<Vec<i32>>,
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
    // --- Combat (M4): the combat-damage section unless stated.
    /// `MaxDamage` (S-0302 claim 6).
    #[serde(default)]
    pub max_damage: Option<i32>,
    /// `OccupyROFMultiplier` m (S-0301 claim 7), a single.
    #[serde(default)]
    pub occupy_rof_multiplier: Option<Decimal>,
    /// `BunkerROFMultiplier` b (S-0301 claim 7), a single.
    #[serde(default)]
    pub bunker_rof_multiplier: Option<Decimal>,
    /// `OccupyDamageMultiplier` M_g (S-0302 F1), a single.
    #[serde(default)]
    pub occupy_damage_multiplier: Option<Decimal>,
    /// `BunkerDamageMultiplier` M_b (S-0302 F1), a single.
    #[serde(default)]
    pub bunker_damage_multiplier: Option<Decimal>,
    /// `OpenToppedDamageMultiplier` M_t (S-0302 F1), a single.
    #[serde(default)]
    pub open_topped_damage_multiplier: Option<Decimal>,
    /// `C4Warhead`, the demolition warhead (S-0302 claims 1 and 20).
    #[serde(default)]
    pub c4_warhead: Option<String>,
    /// `CrushWarhead` (S-0303 claim 9).
    #[serde(default)]
    pub crush_warhead: Option<String>,
    /// `IonCannonWarhead` (S-0303 claim 18).
    #[serde(default)]
    pub ion_cannon_warhead: Option<String>,
    /// `BridgeStrength` (S-0303 claim 18).
    #[serde(default)]
    pub bridge_strength: Option<i32>,
    /// `AmmoCrateDamage` (S-0303 claim 19).
    #[serde(default)]
    pub ammo_crate_damage: Option<i32>,
    /// `SplashList`, animation names (S-0303 F5).
    #[serde(default)]
    pub splash_list: Option<Vec<String>>,
    /// `BallisticScatter`, cells (S-0306 claim 4; lepton form of S-0307 claim 1).
    #[serde(default)]
    pub ballistic_scatter: Option<Decimal>,
    /// `OccupyWeaponRange`, cells (S-0307 claim 4).
    #[serde(default)]
    pub occupy_weapon_range: Option<i32>,
    /// `BunkerWeaponRangeBonus`, cells (S-0307 claim 5).
    #[serde(default)]
    pub bunker_weapon_range_bonus: Option<i32>,
    /// `OpenToppedRangeBonus`, cells (S-0307 claim 5).
    #[serde(default)]
    pub open_topped_range_bonus: Option<i32>,
    /// `FireSupress`, cells (S-0309 F6; lepton form).
    #[serde(default)]
    pub fire_supress: Option<Decimal>,
    /// `DeathWeapon`, the game-wide death weapon (S-0312 claim 6).
    #[serde(default)]
    pub death_weapon: Option<String>,
    /// `[General] VeteranROF` v (S-0301 claim 6), a double.
    #[serde(default)]
    pub veteran_rof: Option<Decimal>,
    /// `[General] VeteranCombat` (S-0302 F1), a double.
    #[serde(default)]
    pub veteran_combat: Option<Decimal>,
    /// `[General] VeteranArmor` (S-0302 F2), a double.
    #[serde(default)]
    pub veteran_armor: Option<Decimal>,
    /// `[General] VeteranSight` (S-0311 F4), a double.
    #[serde(default)]
    pub veteran_sight: Option<Decimal>,
    /// `[General] VeteranRatio` (S-0311 F3), a double.
    #[serde(default)]
    pub veteran_ratio: Option<Decimal>,
    /// `[General] VeteranCap` (S-0311 F3), a double.
    #[serde(default)]
    pub veteran_cap: Option<Decimal>,
    /// `[General] DamageFireTypes`, animation names (S-0302 claim 13).
    #[serde(default)]
    pub damage_fire_types: Option<Vec<String>>,
    /// `[General] InfantryExplode`, an animation name (S-0302 claim 15).
    #[serde(default)]
    pub infantry_explode: Option<String>,
    /// `[General] MissileROTVar` m (S-0306 claim 13), a double.
    #[serde(default)]
    pub missile_rot_var: Option<Decimal>,
    /// `[General] MissileSafetyAltitude`, leptons (S-0306 claim 17).
    #[serde(default)]
    pub missile_safety_altitude: Option<i32>,
    /// `[General] HarvesterUnit`: vehicle type keys (S-0303 claim 9).
    #[serde(default)]
    pub harvester_unit: Option<Vec<String>>,
    /// `[General] BarrelExplode`, an animation name (S-0303 claim 19).
    #[serde(default)]
    pub barrel_explode: Option<String>,
    /// `[General] BarrelDebris`, animation names (S-0303 claim 19).
    #[serde(default)]
    pub barrel_debris: Option<Vec<String>>,
    /// `[General] BarrelParticle`, a particle-system name (S-0303 claim 19).
    #[serde(default)]
    pub barrel_particle: Option<String>,
    /// `[General] MetallicDebris`, animation names (S-0303 F6, S-0312 F1).
    #[serde(default)]
    pub metallic_debris: Option<Vec<String>>,
    /// `[General] WeaponNullifyAnim` (S-0303 claim 15).
    #[serde(default)]
    pub weapon_nullify_anim: Option<String>,
    /// `[General] LightningWarhead` (S-0303 F5).
    #[serde(default)]
    pub lightning_warhead: Option<String>,
    /// `[General] WeatherConBoltExplosion` (S-0303 F5).
    #[serde(default)]
    pub weather_con_bolt_explosion: Option<String>,
    /// `[General] CrewEscape` (S-0312 claim 13), a double.
    #[serde(default)]
    pub crew_escape: Option<Decimal>,
    /// `[General] AlliedCrew`, an infantry type key (S-0312 F4).
    #[serde(default)]
    pub allied_crew: Option<String>,
    /// `[General] SovietCrew` (S-0312 F4).
    #[serde(default)]
    pub soviet_crew: Option<String>,
    /// `[General] ThirdCrew` (S-0312 F4).
    #[serde(default)]
    pub third_crew: Option<String>,
    /// `[General] Technician` (S-0312 F4).
    #[serde(default)]
    pub technician: Option<String>,
    /// `[General] Engineer` (S-0312 claim 18).
    #[serde(default)]
    pub engineer: Option<String>,
    /// `[General] AlliedSurvivorDivisor` (S-0312 F5).
    #[serde(default)]
    pub allied_survivor_divisor: Option<i32>,
    /// `[General] SovietSurvivorDivisor` (S-0312 F5).
    #[serde(default)]
    pub soviet_survivor_divisor: Option<i32>,
    /// `[General] ThirdSurvivorDivisor` (S-0312 F5).
    #[serde(default)]
    pub third_survivor_divisor: Option<i32>,
    /// `[General] ShipSinkingWeight` (S-0312 claim 10).
    #[serde(default)]
    pub ship_sinking_weight: Option<Decimal>,
    /// `[General] RepairRate` r_b (S-0314 F1), a double.
    #[serde(default)]
    pub repair_rate: Option<Decimal>,
    /// `[General] URepairRate` r_u (S-0314 F1).
    #[serde(default)]
    pub u_repair_rate: Option<Decimal>,
    /// `[General] IRepairRate` r_i (S-0314 F1).
    #[serde(default)]
    pub i_repair_rate: Option<Decimal>,
    /// `[General] RepairStep` (S-0314 claim 4).
    #[serde(default)]
    pub repair_step: Option<i32>,
    /// `[General] IRepairStep` (S-0314 claim 4).
    #[serde(default)]
    pub i_repair_step: Option<i32>,
    /// `[General] RepairPercent` (S-0314 F2), a double.
    #[serde(default)]
    pub repair_percent: Option<Decimal>,
    /// `[General] SelfHealInfantryFrames`, ticks (S-0314 claim 10).
    #[serde(default)]
    pub self_heal_infantry_frames: Option<i32>,
    /// `[General] SelfHealUnitFrames`, ticks (S-0314 claim 10).
    #[serde(default)]
    pub self_heal_unit_frames: Option<i32>,
    /// `[General] SelfHealInfantryAmount` (S-0314 claim 10).
    #[serde(default)]
    pub self_heal_infantry_amount: Option<i32>,
    /// `[General] SelfHealUnitAmount` (S-0314 claim 10).
    #[serde(default)]
    pub self_heal_unit_amount: Option<i32>,
    /// `[General] ThreatPerOccupant` (S-0309 Terms, *threat value*).
    #[serde(default)]
    pub threat_per_occupant: Option<i32>,
    /// `[General] EnemyHouseThreatBonus` B (S-0309 F4), a double.
    #[serde(default)]
    pub enemy_house_threat_bonus: Option<Decimal>,
    /// The five `[General] …CoefficientDefault` values, in the order
    /// own-effectiveness, target-effectiveness, special-threat, strength,
    /// distance (S-0309 Import mapping), doubles.
    // spec: S-0309 Import mapping (five game-wide default coefficients)
    #[serde(default)]
    pub coefficient_defaults: Option<[Decimal; 5]>,
    /// `[General] DisabledDisguiseDetectionPercent` (S-0309 claim 25 part g).
    #[serde(default)]
    pub disguise_detection_percent: Option<Vec<i32>>,
    /// `[AudioVisual] Gravity` (S-0306 Terms).
    #[serde(default)]
    pub gravity: Option<i32>,
    /// `[WallModel] AlliedWallTransparency` (S-0306 claim 22).
    #[serde(default)]
    pub allied_wall_transparency: Option<bool>,
    /// `[ElevationModel] ElevationIncrement` I (S-0307 F2).
    #[serde(default)]
    pub elevation_increment: Option<i32>,
    /// `[ElevationModel] ElevationIncrementBonus` b (S-0307 F2), a double.
    #[serde(default)]
    pub elevation_increment_bonus: Option<Decimal>,
    /// `[ElevationModel] ElevationBonusCap` c (S-0307 F2), a double.
    #[serde(default)]
    pub elevation_bonus_cap: Option<Decimal>,
    /// `[IQ] RepairSell` (S-0314 claim 8).
    #[serde(default)]
    pub iq_repair_sell: Option<i32>,
    /// Game-wide economy values (M5, S-0401 to S-0416).
    #[serde(default)]
    pub economy: crate::sim_econ::EconGeneral,
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
    /// Economy fields (M5).
    #[serde(default)]
    pub econ: crate::sim_econ::CountryEcon,
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
    /// `Firepower` φ_c (S-0302 claim 3), a double.
    #[serde(default)]
    pub firepower: Option<Decimal>,
    /// `ROF` κ (S-0301 claim 3), a double.
    #[serde(default)]
    pub rof: Option<Decimal>,
    /// `ArmorInfantryMult` (S-0302 F2), a single.
    #[serde(default)]
    pub armor_infantry_mult: Option<Decimal>,
    /// `ArmorUnitsMult` (S-0302 F2), a single.
    #[serde(default)]
    pub armor_units_mult: Option<Decimal>,
    /// `ArmorAircraftMult` (S-0302 F2), a single.
    #[serde(default)]
    pub armor_aircraft_mult: Option<Decimal>,
    /// `ArmorBuildingsMult` (S-0302 F2), a single.
    #[serde(default)]
    pub armor_buildings_mult: Option<Decimal>,
    /// `ArmorDefensesMult` (S-0302 F2), a single.
    #[serde(default)]
    pub armor_defenses_mult: Option<Decimal>,
    /// `VeteranInfantry`: infantry type keys (S-0311 claim 11).
    #[serde(default)]
    pub veteran_infantry: Option<Vec<String>>,
    /// `VeteranUnits`: vehicle type keys (S-0311 claim 11).
    #[serde(default)]
    pub veteran_units: Option<Vec<String>>,
    /// `VeteranAircraft`: aircraft type keys (S-0311 claim 11).
    #[serde(default)]
    pub veteran_aircraft: Option<Vec<String>>,
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
    /// `Thief`, the money-thief flag (S-0309 claim 5, Import mapping).
    #[serde(default)]
    pub thief: Option<bool>,
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
    /// `VoiceSelect`: the type's selection responses, in the listed order,
    /// every entry kept (S-0214 Terms, F5).
    #[serde(default)]
    pub voice_select: Option<Vec<String>>,
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
    /// The combat fields shared with buildings (M4).
    #[serde(default)]
    pub combat: CombatFields,
    /// Economy fields (M5).
    #[serde(default)]
    pub econ: crate::sim_econ::EconFields,
    /// `BurstDelay0` … `BurstDelay3` (S-0301 claim 4), ticks; −1: none.
    // spec: S-0301 Constants (four burst-delay fields)
    #[serde(default)]
    pub burst_delay: Option<[i32; 4]>,
    /// Art `FiringFrames` F (S-0301 claim 13).
    #[serde(default)]
    pub firing_frames: Option<i32>,
    /// Art `FiringSyncFrame0`, `FiringSyncFrame1` (S-0301 claim 13).
    // spec: S-0301 Constants (two firing sync frames)
    #[serde(default)]
    pub firing_sync_frames: Option<[i32; 2]>,
    /// `HunterSeeker` (S-0301 claim 9).
    #[serde(default)]
    pub hunter_seeker: Option<bool>,
    /// `MobileFire` (S-0301 claim 11, S-0310 claim 7).
    #[serde(default)]
    pub mobile_fire: Option<bool>,
    /// `Gunner` (S-0308 claim 6).
    #[serde(default)]
    pub gunner: Option<bool>,
    /// `IFVMode` (S-0308 claim 6).
    #[serde(default)]
    pub ifv_mode: Option<i32>,
    /// `OpenTransportWeapon` (S-0308 claims 8, 10).
    #[serde(default)]
    pub open_transport_weapon: Option<i32>,
    /// `DeployFireWeapon` (S-0308 claims 8, 9).
    #[serde(default)]
    pub deploy_fire_weapon: Option<i32>,
    /// `OccupyWeapon` (S-0308 claim 13).
    #[serde(default)]
    pub occupy_weapon: Option<String>,
    /// `EliteOccupyWeapon` (S-0308 claim 13).
    #[serde(default)]
    pub elite_occupy_weapon: Option<String>,
    /// `DeployToFire` (S-0310 claim 9).
    #[serde(default)]
    pub deploy_to_fire: Option<bool>,
    /// `JumpJetTurn` (S-0310 claim 8).
    #[serde(default)]
    pub jump_jet_turn: Option<bool>,
    /// `Crewed` (S-0312 claim 13, F4).
    #[serde(default)]
    pub crewed: Option<bool>,
    /// `CarriesCrate` (S-0312 claim 14).
    #[serde(default)]
    pub carries_crate: Option<bool>,
    /// `Crashable` (S-0312 claims 12, 15).
    #[serde(default)]
    pub crashable: Option<bool>,
    /// `Weight` (S-0312 claim 10).
    #[serde(default)]
    pub weight: Option<Decimal>,
    /// `SuppressionThreshold` (S-0315 claim 6).
    #[serde(default)]
    pub suppression_threshold: Option<i32>,
    /// `MissileSpawn` (S-0311 claim 8).
    #[serde(default)]
    pub missile_spawn: Option<bool>,
}

/// Combat fields that infantry, vehicle, aircraft and building types share
/// (M4). Each field's spec is in its doc comment.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CombatFields {
    /// `Armor`, the armour class (S-0305; the importer resolves the word).
    #[serde(default)]
    pub armor: Option<ArmourClass>,
    /// The weapon slots 0 … 17 of a multi-weapon type as `Weapon1` …
    /// `Weapon18` fill them (S-0308 claim 1), the empty string for a slot
    /// with no weapon; absent: slots 0 and 1 come from `primary` and
    /// `secondary`.
    #[serde(default)]
    pub weapon_slots: Option<Vec<String>>,
    /// The elite slots of a multi-weapon type (S-0308 claim 1).
    #[serde(default)]
    pub elite_weapon_slots: Option<Vec<String>>,
    /// `IsGattling` (S-0308 claim 7).
    #[serde(default)]
    pub is_gattling: Option<bool>,
    /// `IsChargeTurret` (S-0301 claim 16, S-0308 claim 5).
    #[serde(default)]
    pub is_charge_turret: Option<bool>,
    /// `LandTargeting` (S-0308 rows 13, 18).
    #[serde(default)]
    pub land_targeting: Option<i32>,
    /// `NavalTargeting` (S-0308 table N).
    #[serde(default)]
    pub naval_targeting: Option<i32>,
    /// `Underwater` (S-0308 table N, S-0303 claim 21).
    #[serde(default)]
    pub underwater: Option<bool>,
    /// `Organic` (S-0308 table N, S-0314 claim 10).
    #[serde(default)]
    pub organic: Option<bool>,
    /// `Unnatural` (S-0308 table N).
    #[serde(default)]
    pub unnatural: Option<bool>,
    /// `Naval` (S-0308 row 13, S-0303 claim 21, S-0312 claim 10).
    #[serde(default)]
    pub naval: Option<bool>,
    /// `Ammo` A (S-0301 claim 10).
    #[serde(default)]
    pub ammo: Option<i32>,
    /// `InitialAmmo` (S-0301 claim 10).
    #[serde(default)]
    pub initial_ammo: Option<i32>,
    /// `Reload` L (S-0301 F3).
    #[serde(default)]
    pub reload: Option<i32>,
    /// `EmptyReload` E (S-0301 F3).
    #[serde(default)]
    pub empty_reload: Option<i32>,
    /// `ReloadIncrement` I (S-0301 F3).
    #[serde(default)]
    pub reload_increment: Option<i32>,
    /// `PipWrap` W (S-0301 F3).
    #[serde(default)]
    pub pip_wrap: Option<i32>,
    /// `ManualReload` (S-0301 claim 11).
    #[serde(default)]
    pub manual_reload: Option<bool>,
    /// `DamageReducesReadiness` (S-0301 claim 17, S-0302 claim 10).
    #[serde(default)]
    pub damage_reduces_readiness: Option<bool>,
    /// `ReadinessReductionMultiplier` κ (S-0302 F6), a single.
    #[serde(default)]
    pub readiness_reduction_multiplier: Option<Decimal>,
    /// `AirRangeBonus`, cells (S-0307 claim 3; lepton form).
    #[serde(default)]
    pub air_range_bonus: Option<Decimal>,
    /// `RadialFireSegments` (S-0306 claims 2, 7).
    #[serde(default)]
    pub radial_fire_segments: Option<i32>,
    /// `TypeImmune` (S-0302 claim 8).
    #[serde(default)]
    pub type_immune: Option<bool>,
    /// `ImmuneToPsionicWeapons` (S-0302 claim 8).
    #[serde(default)]
    pub immune_to_psionic_weapons: Option<bool>,
    /// `ImmuneToPoison` (S-0302 claim 8).
    #[serde(default)]
    pub immune_to_poison: Option<bool>,
    /// `ImmuneToPsionics` (S-0302 claim 10).
    #[serde(default)]
    pub immune_to_psionics: Option<bool>,
    /// `DamageSelf` (S-0302 claim 8, S-0303 claim 9).
    #[serde(default)]
    pub damage_self: Option<bool>,
    /// `DamageParticleSystems`: the particle-system names that are
    /// eligible (behaviour *smoke*, S-0302 claim 13), in list order.
    #[serde(default)]
    pub damage_smoke_systems: Option<Vec<String>>,
    /// `MaxDebris` X (S-0312 F1).
    #[serde(default)]
    pub max_debris: Option<i32>,
    /// `MinDebris` Y (S-0312 F1).
    #[serde(default)]
    pub min_debris: Option<i32>,
    /// `DebrisTypes` (S-0312 F1).
    #[serde(default)]
    pub debris_types: Option<Vec<String>>,
    /// `DebrisMaximums` (S-0312 F1).
    #[serde(default)]
    pub debris_maximums: Option<Vec<i32>>,
    /// `DebrisAnims` (S-0312 F1).
    #[serde(default)]
    pub debris_anims: Option<Vec<String>>,
    /// `Explodes` (S-0312 Terms).
    #[serde(default)]
    pub explodes: Option<bool>,
    /// `DeathWeapon` (S-0312 claim 6).
    #[serde(default)]
    pub death_weapon: Option<String>,
    /// `DeathWeaponDamageModifier` φ (S-0312 F3), a single.
    #[serde(default)]
    pub death_weapon_damage_modifier: Option<Decimal>,
    /// `Explosion`, animation names (S-0312 claims 9, 16, 19).
    #[serde(default)]
    pub explosion: Option<Vec<String>>,
    /// `DestroyAnim`, animation names (S-0312 claims 9, 19).
    #[serde(default)]
    pub destroy_anim: Option<Vec<String>>,
    /// `Trainable` (S-0311 claim 8).
    #[serde(default)]
    pub trainable: Option<bool>,
    /// `DontScore` (S-0311 claim 7).
    #[serde(default)]
    pub dont_score: Option<bool>,
    /// `SelfHealing` (S-0314 claim 9).
    #[serde(default)]
    pub self_healing: Option<bool>,
    /// `CanPassiveAquire` (S-0309 claim 2).
    #[serde(default)]
    pub can_passive_aquire: Option<bool>,
    /// `CanRetaliate` (S-0309 claim 18).
    #[serde(default)]
    pub can_retaliate: Option<bool>,
    /// `DetectDisguise` (S-0309 table C).
    #[serde(default)]
    pub detect_disguise: Option<bool>,
    /// `NoAutoFire` (S-0309 claim 5).
    #[serde(default)]
    pub no_auto_fire: Option<bool>,
    /// `VHPScan`: `none`, `normal` or `strong` (S-0309 claim 14).
    #[serde(default)]
    pub vhp_scan: Option<String>,
    /// `SpecialThreatValue` (S-0309 F4), a double.
    #[serde(default)]
    pub special_threat_value: Option<Decimal>,
    /// The five threat coefficients, in the order own-effectiveness,
    /// target-effectiveness, special-threat, strength, distance (S-0309
    /// F4), doubles; an entry given as 0 already replaced by the importer
    /// where S-0309 Edge cases say.
    // spec: S-0309 Import mapping (five threat coefficients)
    #[serde(default)]
    pub coefficients: Option<[Decimal; 5]>,
    /// `Insignificant` (S-0309 table C).
    #[serde(default)]
    pub insignificant: Option<bool>,
    /// `ThreatPosed` (S-0309 Terms).
    #[serde(default)]
    pub threat_posed: Option<i32>,
    /// `AirstrikeTeam` (S-0309 claim 24).
    #[serde(default)]
    pub airstrike_team: Option<i32>,
    /// `Drainable` (S-0308 row 8).
    #[serde(default)]
    pub drainable: Option<bool>,
    /// Art `PrimaryFireFLH` (S-0313).
    #[serde(default)]
    pub primary_fire_flh: Option<Flh>,
    /// Art `SecondaryFireFLH` (S-0313).
    #[serde(default)]
    pub secondary_fire_flh: Option<Flh>,
    /// Art `ElitePrimaryFireFLH` (S-0313).
    #[serde(default)]
    pub elite_primary_fire_flh: Option<Flh>,
    /// Art `EliteSecondaryFireFLH` (S-0313).
    #[serde(default)]
    pub elite_secondary_fire_flh: Option<Flh>,
    /// Art `Weapon`n`FLH`, by slot (S-0313).
    #[serde(default)]
    pub weapon_flh: Option<Vec<Flh>>,
    /// Art `EliteWeapon`n`FLH`, by slot (S-0313).
    #[serde(default)]
    pub elite_weapon_flh: Option<Vec<Flh>>,
    /// Art `TurretOffset` δ (S-0313).
    #[serde(default)]
    pub turret_offset: Option<i32>,
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
    /// `VoiceSelect`: the type's selection responses, in the listed order,
    /// every entry kept (S-0214 Terms, F5).
    #[serde(default)]
    pub voice_select: Option<Vec<String>>,
    /// `CanBeOccupied`.
    #[serde(default)]
    pub can_be_occupied: Option<bool>,
    /// `CanOccupyFire`.
    #[serde(default)]
    pub can_occupy_fire: Option<bool>,
    /// `Capturable`.
    #[serde(default)]
    pub capturable: Option<bool>,
    /// `ImmuneToRadiation` (S-0302 claim 8).
    #[serde(default)]
    pub immune_to_radiation: Option<bool>,
    /// `NeedsEngineer` (S-0309 claim 25 part e, Import mapping).
    #[serde(default)]
    pub needs_engineer: Option<bool>,
    /// `Storage`, storage capacity (S-0309 table C, row "storage").
    #[serde(default)]
    pub storage: Option<i32>,
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
    /// The combat fields shared with mobile types (M4).
    #[serde(default)]
    pub combat: CombatFields,
    /// Economy fields (M5).
    #[serde(default)]
    pub econ: crate::sim_econ::EconFields,
    /// `Owner`, the owner countries (S-0407 claims 1 and 11), country keys.
    #[serde(default)]
    pub owner: Option<Vec<String>>,
    /// `Cost`, credits (S-0311 F1, S-0314 claim 5).
    #[serde(default)]
    pub cost: Option<i32>,
    /// `BuildCat` is `Combat`: a defence building (S-0302 claim 5).
    #[serde(default)]
    pub defense: Option<bool>,
    /// `EligibleForDelayKill` (S-0302 claim 16).
    #[serde(default)]
    pub eligible_for_delay_kill: Option<bool>,
    /// `LeaveRubble` (S-0312 claim 20).
    #[serde(default)]
    pub leave_rubble: Option<bool>,
    /// `Helipad` (S-0312 claim 17).
    #[serde(default)]
    pub helipad: Option<bool>,
    /// `Artillary` (S-0309 claim 18).
    #[serde(default)]
    pub artillary: Option<bool>,
    /// `MaxNumberOccupants` (S-0313 claim 6).
    #[serde(default)]
    pub max_number_occupants: Option<i32>,
    /// `InfantryGainSelfHeal` (S-0314 claim 10).
    #[serde(default)]
    pub infantry_gain_self_heal: Option<i32>,
    /// `UnitsGainSelfHeal` (S-0314 claim 10).
    #[serde(default)]
    pub units_gain_self_heal: Option<i32>,
    /// Art `TargetCoordOffset`, leptons (S-0306 Terms, aim shift).
    // spec: S-0306 Import mapping (an integer triple)
    #[serde(default)]
    pub target_coord_offset: Option<[i32; 3]>,
    /// Art `Height` H_b (S-0306 claim 6 c).
    #[serde(default)]
    pub art_height: Option<i32>,
    /// Art `TurretAnimIsVoxel` (S-0310 claim 4, S-0313 claim 8).
    #[serde(default)]
    pub turret_anim_is_voxel: Option<bool>,
    /// Art `BarrelAnimIsVoxel` (S-0313 claim 8).
    #[serde(default)]
    pub barrel_anim_is_voxel: Option<bool>,
    /// Art `PrimaryFirePixelOffset`, pixels (S-0313 claim 7).
    #[serde(default)]
    pub primary_fire_pixel_offset: Option<[i32; 2]>, // nocite: a pixel pair, our shape
    /// Art `PrimaryFireDualOffset` (S-0313 claim 7).
    #[serde(default)]
    pub primary_fire_dual_offset: Option<bool>,
    /// Art `TurretAnimX`, `TurretAnimY`, pixels (S-0313 claim 8).
    #[serde(default)]
    pub turret_anim_xy: Option<[i32; 2]>, // nocite: a pixel pair, our shape
    /// Art `IsAnimDelayedFire` (S-0301 claim 14).
    #[serde(default)]
    pub is_anim_delayed_fire: Option<bool>,
    /// Art `DelayedFireDelay`, ticks (S-0301 claim 14).
    #[serde(default)]
    pub delayed_fire_delay: Option<i32>,
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
    /// `ChainReaction` (S-0303 claim 17).
    #[serde(default)]
    pub chain_reaction: Option<bool>,
    /// `Explodes` (S-0303 claim 19).
    #[serde(default)]
    pub explodes: Option<bool>,
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
    /// `IsAnimated`, the animated flag of an ore mine (S-0401 claim 15).
    #[serde(default)]
    pub is_animated: Option<bool>,
    /// `AnimationRate`, *R* in ticks (S-0401 claim 15); absent: 0.
    #[serde(default)]
    pub animation_rate: Option<i32>,
    /// `AnimationProbability`, *p*, a single (S-0401 F8); absent: 0.
    #[serde(default)]
    pub animation_probability: Option<Decimal>,
    /// `Strength` (S-0302 claim 20).
    #[serde(default)]
    pub strength: Option<i32>,
    /// `Armor`, the armour class (S-0305).
    #[serde(default)]
    pub armor: Option<ArmourClass>,
    /// `Immune` (S-0302 claim 20).
    #[serde(default)]
    pub immune: Option<bool>,
}

/// A weapon's simulation fields.
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
    /// `ROF` R, ticks (S-0301).
    #[serde(default)]
    pub rof: Option<i32>,
    /// `Burst` B (S-0301 claim 4).
    #[serde(default)]
    pub burst: Option<i32>,
    /// `Speed`, the imported speed field v (S-0306 Import mapping).
    #[serde(default)]
    pub speed: Option<i32>,
    /// `Range`, cells (S-0307 claim 1; lepton form).
    #[serde(default)]
    pub range: Option<Decimal>,
    /// `MinimumRange`, cells (S-0307 claim 1).
    #[serde(default)]
    pub minimum_range: Option<Decimal>,
    /// `CellRangefinding` (S-0307 claim 8).
    #[serde(default)]
    pub cell_rangefinding: Option<bool>,
    /// `IsSonic` (S-0301 claim 5, S-0302 claim 2).
    #[serde(default)]
    pub is_sonic: Option<bool>,
    /// `UseSparkParticles` (S-0301 claim 5).
    #[serde(default)]
    pub use_spark_particles: Option<bool>,
    /// `UseFireParticles` (S-0301 claim 5, S-0302 claim 2).
    #[serde(default)]
    pub use_fire_particles: Option<bool>,
    /// `IsRailgun` (S-0301 claim 5).
    #[serde(default)]
    pub is_railgun: Option<bool>,
    /// `IsMagBeam` (S-0301 claim 5).
    #[serde(default)]
    pub is_mag_beam: Option<bool>,
    /// `Temporal` (S-0301 Terms).
    #[serde(default)]
    pub temporal: Option<bool>,
    /// `Spawner` (S-0301 claim 9).
    #[serde(default)]
    pub spawner: Option<bool>,
    /// `DiskLaser` (S-0301 claim 8).
    #[serde(default)]
    pub disk_laser: Option<bool>,
    /// `DecloakToFire` (S-0301 claim 12).
    #[serde(default)]
    pub decloak_to_fire: Option<bool>,
    /// `Lobber` (S-0306 claim 6 a).
    #[serde(default)]
    pub lobber: Option<bool>,
    /// `NeverUse` (S-0308 claim 4).
    #[serde(default)]
    pub never_use: Option<bool>,
    /// `DrainWeapon` (S-0308 row 8).
    #[serde(default)]
    pub drain_weapon: Option<bool>,
    /// `FireWhileMoving` (S-0310 claim 7).
    #[serde(default)]
    pub fire_while_moving: Option<bool>,
    /// `Suicide` (S-0312 Terms).
    #[serde(default)]
    pub suicide: Option<bool>,
    /// `Supress` (S-0309 claim 14).
    #[serde(default)]
    pub supress: Option<bool>,
    /// `FireOnce` (S-0309 claim 21).
    #[serde(default)]
    pub fire_once: Option<bool>,
}

/// A projectile's simulation fields.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProjectileType {
    /// `ROT`.
    #[serde(default)]
    pub rot: Option<i32>,
    /// `AG`.
    #[serde(default)]
    pub ag: Option<bool>,
    /// `AA` (S-0306 claim 23).
    #[serde(default)]
    pub aa: Option<bool>,
    /// `Arcing` (S-0306 Terms).
    #[serde(default)]
    pub arcing: Option<bool>,
    /// `Vertical` (S-0306 Terms).
    #[serde(default)]
    pub vertical: Option<bool>,
    /// `Inviso` (S-0306 Terms).
    #[serde(default)]
    pub inviso: Option<bool>,
    /// `Inaccurate` (S-0306 claim 4).
    #[serde(default)]
    pub inaccurate: Option<bool>,
    /// `FlakScatter` (S-0306 claim 4).
    #[serde(default)]
    pub flak_scatter: Option<bool>,
    /// `Dropping` (S-0306 claim 5).
    #[serde(default)]
    pub dropping: Option<bool>,
    /// `Airburst` (S-0306 claim 21).
    #[serde(default)]
    pub airburst: Option<bool>,
    /// `VeryHigh` (S-0306 claim 15).
    #[serde(default)]
    pub very_high: Option<bool>,
    /// `Level` (S-0306 T1).
    #[serde(default)]
    pub level: Option<bool>,
    /// `Floater` (S-0306 Terms).
    #[serde(default)]
    pub floater: Option<bool>,
    /// `SubjectToCliffs` (S-0306 claim 22).
    #[serde(default)]
    pub subject_to_cliffs: Option<bool>,
    /// `SubjectToWalls` (S-0306 claim 22).
    #[serde(default)]
    pub subject_to_walls: Option<bool>,
    /// `SubjectToElevation` (S-0307 claim 6).
    #[serde(default)]
    pub subject_to_elevation: Option<bool>,
    /// `Ranged` (S-0306 Terms).
    #[serde(default)]
    pub ranged: Option<bool>,
    /// `Degenerates` (S-0306 claim 18).
    #[serde(default)]
    pub degenerates: Option<bool>,
    /// `Scalable` (S-0306 claim 16).
    #[serde(default)]
    pub scalable: Option<bool>,
    /// `Acceleration` α (S-0306 claims 11, 12).
    #[serde(default)]
    pub acceleration: Option<i32>,
    /// `CourseLockDuration` κ (S-0306 claim 12).
    #[serde(default)]
    pub course_lock_duration: Option<i32>,
    /// `Arm`, ticks (S-0306 claim 9).
    #[serde(default)]
    pub arm: Option<i32>,
    /// `DetonationAltitude`, leptons (S-0306 claim 11).
    #[serde(default)]
    pub detonation_altitude: Option<i32>,
    /// `Cluster` n_c (S-0306 claim 21).
    #[serde(default)]
    pub cluster: Option<i32>,
    /// `AirburstWeapon` (S-0306 claim 21).
    #[serde(default)]
    pub airburst_weapon: Option<String>,
    /// `Trailer`, an animation name (S-0306 claim 16).
    #[serde(default)]
    pub trailer: Option<String>,
    /// `SpawnDelay`, ticks (S-0306 claim 16).
    #[serde(default)]
    pub spawn_delay: Option<i32>,
    /// Art `Voxel`: drawn as a voxel model (S-0306 claim 6 b).
    #[serde(default)]
    pub voxel: Option<bool>,
}

/// A warhead's simulation fields.
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
    /// `Verses`: the eleven versus values in class-index order (S-0003
    /// claim 13, S-0305), each written as the exact decimal of its double.
    #[serde(default)]
    pub verses: Option<Vec<Decimal>>,
    /// `CellSpread` S, cells (S-0303), a single.
    #[serde(default)]
    pub cell_spread: Option<Decimal>,
    /// `PercentAtMax` p (S-0303), a single.
    #[serde(default)]
    pub percent_at_max: Option<Decimal>,
    /// `ProneDamage` P (S-0302 F2), a double.
    #[serde(default)]
    pub prone_damage: Option<Decimal>,
    /// `PenetratesBunker` (S-0302 claim 8).
    #[serde(default)]
    pub penetrates_bunker: Option<bool>,
    /// `Radiation` (S-0302 claim 8).
    #[serde(default)]
    pub radiation: Option<bool>,
    /// `PsychicDamage` (S-0302 claim 8).
    #[serde(default)]
    pub psychic_damage: Option<bool>,
    /// `Poison` (S-0302 claim 8).
    #[serde(default)]
    pub poison: Option<bool>,
    /// `AffectsAllies` (S-0302 claim 8).
    #[serde(default)]
    pub affects_allies: Option<bool>,
    /// `Psychedelic`: the berserk warhead (S-0302 claim 10).
    #[serde(default)]
    pub psychedelic: Option<bool>,
    /// `Sparky` (S-0302 claim 13).
    #[serde(default)]
    pub sparky: Option<bool>,
    /// `CausesDelayKill` (S-0302 claim 16).
    #[serde(default)]
    pub causes_delay_kill: Option<bool>,
    /// `DelayKillFrames` F (S-0302 F7).
    #[serde(default)]
    pub delay_kill_frames: Option<i32>,
    /// `DelayKillAtMax` α (S-0302 F7), a single.
    #[serde(default)]
    pub delay_kill_at_max: Option<Decimal>,
    /// `WallAbsoluteDestroyer` (S-0303 claim 16).
    #[serde(default)]
    pub wall_absolute_destroyer: Option<bool>,
    /// `Tiberium`, the resource flag (S-0303 claim 17).
    #[serde(default)]
    pub tiberium: Option<bool>,
    /// `Rocker` (S-0303 claim 20).
    #[serde(default)]
    pub rocker: Option<bool>,
    /// `Particle`, a particle-system name (S-0303 claim 20).
    #[serde(default)]
    pub particle: Option<String>,
    /// `Conventional` (S-0303 F5).
    #[serde(default)]
    pub conventional: Option<bool>,
    /// `EMEffect` (S-0303 F5, S-0306 claim 20 c).
    #[serde(default)]
    pub em_effect: Option<bool>,
    /// `AnimList`, animation names (S-0303 F5).
    #[serde(default)]
    pub anim_list: Option<Vec<String>>,
    /// `MaxDebris` X (S-0303 F6).
    #[serde(default)]
    pub max_debris: Option<i32>,
    /// `MinDebris` Y (S-0303 F6).
    #[serde(default)]
    pub min_debris: Option<i32>,
    /// `DebrisTypes` (S-0303 F6).
    #[serde(default)]
    pub debris_types: Option<Vec<String>>,
    /// `DebrisMaximums` (S-0303 F6).
    #[serde(default)]
    pub debris_maximums: Option<Vec<i32>>,
    /// `Airstrike` (S-0308 row 6).
    #[serde(default)]
    pub airstrike: Option<bool>,
    /// `IsLocomotor` (S-0308 row 7).
    #[serde(default)]
    pub is_locomotor: Option<bool>,
}
