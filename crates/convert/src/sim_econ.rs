//! The economy fields of the typed `sim` section (M5): the fields of object
//! types (`econ`), countries (`econ`), tiberiums, superweapons and the
//! game-wide economy values (`general.economy`), each read as the Import
//! mapping of the spec named on its row says (S-0401 to S-0416), with the
//! importer warnings those specs ask for.

use super::*;
use glasnost_rules::sim_econ::{superweapon_action_known, superweapon_kind_number};

/// A table row: our field name, the INI key, the value form and the spec
/// whose Import mapping maps the key.
type Row = (&'static str, &'static str, Form, &'static str);

/// Fields that every object type (infantry, vehicle, aircraft, building)
/// carries in `econ`.
const ECON_FIELDS: &[Row] = &[
    (
        "requires_stolen_allied_tech",
        "RequiresStolenAlliedTech",
        Bool,
        "S-0407",
    ),
    (
        "requires_stolen_soviet_tech",
        "RequiresStolenSovietTech",
        Bool,
        "S-0407",
    ),
    (
        "requires_stolen_third_tech",
        "RequiresStolenThirdTech",
        Bool,
        "S-0407",
    ),
    ("build_limit", "BuildLimit", Int, "S-0407"),
    (
        "build_time_multiplier",
        "BuildTimeMultiplier",
        Dec,
        "S-0403",
    ),
    ("soylent", "Soylent", Int, "S-0408"),
    ("radar_invisible", "RadarInvisible", Bool, "S-0410"),
    ("radar_visible", "RadarVisible", Bool, "S-0410"),
    ("invisible", "Invisible", Bool, "S-0410"),
    ("reveal_to_all", "RevealToAll", Bool, "S-0409"),
    ("gap_radius_in_cells", "GapRadiusInCells", Int, "S-0409"),
    (
        "super_gap_radius_in_cells",
        "SuperGapRadiusInCells",
        Int,
        "S-0405, S-0409",
    ),
];

/// Lists of names kept as written (upper case), not resolved: the tech-tree
/// rules compare them themselves (S-0407 claims 3, 5 and 8).
const NAME_LISTS: &[(&str, &str, &str)] = &[
    ("prerequisite", "Prerequisite", "S-0407"),
    ("prerequisite_override", "PrerequisiteOverride", "S-0407"),
    ("required_houses", "RequiredHouses", "S-0407"),
    ("forbidden_houses", "ForbiddenHouses", "S-0407"),
];

/// Fields of infantry, vehicle and aircraft types.
const UNIT_ECON: &[Row] = &[
    ("dock", "Dock", Refs(Kind::Buildings), "S-0402, S-0404"),
    (
        "resource_gatherer",
        "ResourceGatherer",
        Bool,
        "S-0402, S-0414, S-0415",
    ),
    (
        "resource_destination",
        "ResourceDestination",
        Bool,
        "S-0402",
    ),
    ("enslaves", "Enslaves", Ref(Kind::Infantry), "S-0402"),
    ("slaves_number", "SlavesNumber", Int, "S-0402"),
    ("slave_regen_rate", "SlaveRegenRate", Int, "S-0402"),
    ("slave_reload_rate", "SlaveReloadRate", Int, "S-0402"),
    (
        "considered_aircraft",
        "ConsideredAircraft",
        Bool,
        "S-0004, S-0501",
    ),
];

/// Fields of infantry types only.
const INFANTRY_ECON: &[Row] = &[
    ("harvest_rate", "HarvestRate", Int, "S-0402"),
    ("nominal", "Nominal", Bool, "S-0414"),
];

/// Fields of vehicle types only.
const VEHICLE_ECON: &[Row] = &[("powered_unit", "PoweredUnit", Bool, "S-0405")];

/// Fields of aircraft types only.
const AIRCRAFT_ECON: &[Row] = &[
    ("airport_bound", "AirportBound", Bool, "S-0407, S-0501"),
    ("landable", "Landable", Bool, "S-0501"),
    ("fighter", "Fighter", Bool, "S-0501"),
    ("fly_by", "FlyBy", Bool, "S-0501"),
    ("fly_back", "FlyBack", Bool, "S-0501"),
    ("flight_level", "FlightLevel", Int, "S-0501"),
];

/// Fields of building types (rules).
const BUILDING_ECON: &[Row] = &[
    ("power", "Power", Int, "S-0405"),
    ("extra_power", "ExtraPower", Int, "S-0405"),
    ("powered_special", "PoweredSpecial", Bool, "S-0405"),
    ("powers_unit", "PowersUnit", Ref(Kind::Vehicles), "S-0405"),
    ("sight", "Sight", Int, "S-0409"),
    ("radar", "Radar", Bool, "S-0409, S-0410"),
    ("spy_sat", "SpySat", Bool, "S-0409"),
    ("refinery", "Refinery", Bool, "S-0402"),
    ("ore_purifier", "OrePurifier", Bool, "S-0402"),
    ("dock_unload", "DockUnload", Bool, "S-0402"),
    ("weeder", "Weeder", Bool, "S-0402"),
    ("door_first", "GDIBarracks", Bool, "S-0404"),
    ("door_second", "NODBarracks", Bool, "S-0404"),
    ("door_third", "YuriBarracks", Bool, "S-0404"),
    ("cloning", "Cloning", Bool, "S-0404"),
    // S-0308 table P row 6: a building that is both is no airstrike target.
    ("resource_gatherer", "ResourceGatherer", Bool, "S-0308"),
    (
        "resource_destination",
        "ResourceDestination",
        Bool,
        "S-0308, S-0608",
    ),
    // S-0402 claim 19: a building type that names a slave type creates a slave group.
    ("enslaves", "Enslaves", Ref(Kind::Infantry), "S-0402"),
    ("slaves_number", "SlavesNumber", Int, "S-0402"),
    ("slave_regen_rate", "SlaveRegenRate", Int, "S-0402"),
    ("slave_reload_rate", "SlaveReloadRate", Int, "S-0402"),
    ("factory_plant", "FactoryPlant", Bool, "S-0404"),
    ("infantry_cost_bonus", "InfantryCostBonus", Dec, "S-0404"),
    ("units_cost_bonus", "UnitsCostBonus", Dec, "S-0404"),
    ("aircraft_cost_bonus", "AircraftCostBonus", Dec, "S-0404"),
    ("buildings_cost_bonus", "BuildingsCostBonus", Dec, "S-0404"),
    ("defenses_cost_bonus", "DefensesCostBonus", Dec, "S-0404"),
    (
        "free_unit",
        "FreeUnit",
        Ref(Kind::Vehicles),
        "S-0404, S-0408",
    ),
    ("unsellable", "Unsellable", Bool, "S-0408"),
    ("click_repairable", "ClickRepairable", Bool, "S-0408"),
    ("adjacent", "Adjacent", Int, "S-0406"),
    ("base_normal", "BaseNormal", Bool, "S-0406"),
    (
        "eligible_for_ally_building",
        "EligibileForAllyBuilding",
        Bool,
        "S-0406",
    ),
    ("place_anywhere", "PlaceAnywhere", Bool, "S-0406"),
    ("water_bound", "WaterBound", Bool, "S-0406"),
    ("wall", "Wall", Bool, "S-0403, S-0406"),
    ("to_overlay", "ToOverlay", Ref(Kind::Overlays), "S-0406"),
    ("nuke_silo", "NukeSilo", Bool, "S-0513"),
    (
        "psychic_detection_radius",
        "PsychicDetectionRadius",
        Int,
        "S-0513, S-0610",
    ),
    ("has_radial_indicator", "HasRadialIndicator", Bool, "S-0610"),
    ("laser_fence_post", "LaserFencePost", Bool, "S-0406"),
    ("guard_range", "GuardRange", Dec, "S-0406"),
    (
        "powers_up_building",
        "PowersUpBuilding",
        Ref(Kind::Buildings),
        "S-0406",
    ),
    ("powers_up_to_level", "PowersUpToLevel", Int, "S-0406"),
    ("upgrades", "Upgrades", Int, "S-0406"),
    ("to_tile", "ToTile", Text, "S-0406"),
    ("ai_base_planning_side", "AIBasePlanningSide", Int, "S-0407"),
    (
        "super_weapon",
        "SuperWeapon",
        Ref(Kind::Superweapons),
        "S-0407, S-0412",
    ),
    (
        "super_weapon2",
        "SuperWeapon2",
        Ref(Kind::Superweapons),
        "S-0407, S-0412",
    ),
    ("secret_lab", "SecretLab", Bool, "S-0407"),
    (
        "secret_infantry",
        "SecretInfantry",
        Ref(Kind::Infantry),
        "S-0407",
    ),
    ("secret_unit", "SecretUnit", Ref(Kind::Vehicles), "S-0407"),
    (
        "secret_building",
        "SecretBuilding",
        Ref(Kind::Buildings),
        "S-0407",
    ),
    ("number_of_docks", "NumberOfDocks", Int, "S-0407"),
    ("unit_reload", "UnitReload", Bool, "S-0501"),
    ("deploy_facing", "DeployFacing", Int, "S-0415"),
];

/// Art keys of a building type's image section that are animation names.
const DOCK_ANIMS: &[(&str, &str)] = &[
    ("pre_production_anim", "PreProductionAnim"),
    ("pre_production_anim_damaged", "PreProductionAnimDamaged"),
    ("special_anim", "SpecialAnim"),
    ("special_anim_damaged", "SpecialAnimDamaged"),
    ("production_anim", "ProductionAnim"),
    ("production_anim_damaged", "ProductionAnimDamaged"),
    // spec: S-0606 Import mapping (special-2 to special-4 and their damaged forms, as S-0108)
    ("special_anim_two", "SpecialAnimTwo"),
    ("special_anim_two_damaged", "SpecialAnimTwoDamaged"),
    ("special_anim_three", "SpecialAnimThree"),
    ("special_anim_three_damaged", "SpecialAnimThreeDamaged"),
    ("special_anim_four", "SpecialAnimFour"),
    ("special_anim_four_damaged", "SpecialAnimFourDamaged"),
];

/// The slots whose names the simulation's slot model reads (S-0108 claim
/// 20; S-0402 claim 11), by art base key.
const SLOT_MODEL_BASES: &[&str] = &[
    "PreProductionAnim",
    "ProductionAnim",
    "SpecialAnim",
    "SpecialAnimTwo",
    "SpecialAnimThree",
];

/// A slot name's text limit (S-0108 Import mapping).
const SLOT_NAME_LIMIT: usize = 15; // spec: S-0108 Import mapping (names are text cut to 15 bytes)

/// The build-up sprite name's text limit (S-0108 Import mapping).
const BUILDUP_MAX: usize = 15; // spec: S-0108 Import mapping (Buildup, text cut to 15 characters)

/// Country fields.
const COUNTRY_ECON: &[Row] = &[
    ("cost_infantry_mult", "CostInfantryMult", Dec, "S-0404"),
    ("cost_units_mult", "CostUnitsMult", Dec, "S-0404"),
    ("cost_aircraft_mult", "CostAircraftMult", Dec, "S-0404"),
    ("cost_buildings_mult", "CostBuildingsMult", Dec, "S-0404"),
    ("cost_defenses_mult", "CostDefensesMult", Dec, "S-0404"),
    (
        "build_time_infantry_mult",
        "BuildTimeInfantryMult",
        Dec,
        "S-0403",
    ),
    ("build_time_units_mult", "BuildTimeUnitsMult", Dec, "S-0403"),
    (
        "build_time_aircraft_mult",
        "BuildTimeAircraftMult",
        Dec,
        "S-0403",
    ),
    (
        "build_time_buildings_mult",
        "BuildTimeBuildingsMult",
        Dec,
        "S-0403",
    ),
    (
        "build_time_defenses_mult",
        "BuildTimeDefensesMult",
        Dec,
        "S-0403",
    ),
    ("income_mult", "IncomeMult", Dec, "S-0402"),
    ("speed_aircraft_mult", "SpeedAircraftMult", Dec, "S-0413"),
    ("side", "Side", Text, "S-0413"),
    ("multiplay", "Multiplay", Bool, "S-0413"),
    ("color", "Color", Text, "S-0413"),
    ("parent_country", "ParentCountry", Text, "S-0413"),
    ("prefix", "Prefix", Text, "S-0413"),
    ("suffix", "Suffix", Text, "S-0413"),
    ("smart_ai", "SmartAI", Bool, "S-0413"),
];

/// The country's general factors with no effect (S-0413 claim 2): the
/// importer warns for a value other than 1.0. `Armor` is S-0302's.
const NO_EFFECT_FACTORS: [&str; 4] = ["Cost", "BuildTime", "Groundspeed", "Airspeed"]; // spec: S-0413 claim 2 (general factors with no effect)

/// Tiberium fields.
const TIBERIUM_FIELDS: &[Row] = &[
    ("value", "Value", Int, "S-0401"),
    ("growth", "Growth", Int, "S-0401"),
    ("growth_percentage", "GrowthPercentage", Dec, "S-0401"),
    ("spread", "Spread", Int, "S-0401"),
    ("spread_percentage", "SpreadPercentage", Dec, "S-0401"),
    ("image", "Image", Int, "S-0112"),
];

/// Superweapon fields.
const SUPERWEAPON_FIELDS: &[Row] = &[
    (
        "disableable_from_shell",
        "DisableableFromShell",
        Bool,
        "S-0407, S-0412",
    ),
    ("kind", "Type", Text, "S-0407, S-0512"),
    ("recharge_time", "RechargeTime", Dec, "S-0512"),
    ("is_powered", "IsPowered", Bool, "S-0512"),
    ("show_timer", "ShowTimer", Bool, "S-0512"),
    (
        "flash_sidebar_tab_frames",
        "FlashSidebarTabFrames",
        Int,
        "S-0512",
    ),
    ("pre_click", "PreClick", Bool, "S-0512"),
    ("post_click", "PostClick", Bool, "S-0512"),
    ("pre_dependent", "PreDependent", Text, "S-0512"),
    ("action", "Action", Text, "S-0512"),
    ("use_charge_drain", "UseChargeDrain", Bool, "S-0512"),
    ("manual_control", "ManualControl", Bool, "S-0512"),
    (
        "aux_building",
        "AuxBuilding",
        Ref(Kind::Buildings),
        "S-0512",
    ),
    ("ui_name", "UIName", Text, "S-0512"),
    ("sidebar_image", "SidebarImage", Text, "S-0512"),
    ("weapon_type", "WeaponType", Ref(Kind::Weapons), "S-0513"),
    ("start_sound", "StartSound", Text, "S-0512, S-0513"),
    ("special_sound", "SpecialSound", Text, "S-0512, S-0513"),
];

const GENERAL: &[u8] = b"General"; // spec: S-0004 Import mapping (the general settings section)
const AI: &[u8] = b"AI"; // spec: S-0407 Import mapping (`[AI]`)
const AUDIO_VISUAL: &[u8] = b"AudioVisual"; // spec: S-0409 Import mapping (`[AudioVisual]`)
const SPECIAL_WEAPONS: &[u8] = b"SpecialWeapons"; // spec: S-0004 Import mapping (`[SpecialWeapons]`)
const CRATE_RULES: &[u8] = b"CrateRules"; // spec: S-0412 Import mapping (`[CrateRules]`)
const POWERUPS: &[u8] = b"Powerups"; // spec: S-0520 Import mapping (`[Powerups]`)

/// The built-in power-up weights at a fresh start, in table order
/// (S-0520 claim 28, Import mapping).
// spec: S-0520 Import mapping (weights 50, 20, 1, 3, 5, 5, 20, 1, 1, 10, 10, 10, 1, 3, 1, 1, 1, 1, 1; no animations, water flags off, arguments 0.0)
const POWERUP_STARTUP_WEIGHTS: [i32; 19] = [
    50, 20, 1, 3, 5, 5, 20, 1, 1, 10, 10, 10, 1, 3, 1, 1, 1, 1, 1,
];

/// The power-up keys in table order (S-0520 claim 8, Import mapping).
pub(crate) const POWERUP_KEYS: [&str; 19] = [
    "Money",
    "Unit",
    "HealBase",
    "Cloak",
    "Explosion",
    "Napalm",
    "Squad",
    "Darkness",
    "Reveal",
    "Armor",
    "Speed",
    "Firepower",
    "ICBM",
    "Invulnerability",
    "Veteran",
    "IonStorm",
    "Gas",
    "Tiberium",
    "Pod",
];
const DIALOG: &[u8] = b"MultiplayerDialogSettings"; // spec: S-0412 Import mapping (`[MultiplayerDialogSettings]`)

/// Game-wide economy values: our field, the section, the key, the form and
/// the spec.
const GENERAL_ECON: &[(&str, &[u8], &str, Form, &str)] = &[
    (
        "harvester_load_rate",
        GENERAL,
        "HarvesterLoadRate",
        Int,
        "S-0402",
    ),
    (
        "harvester_dump_rate",
        GENERAL,
        "HarvesterDumpRate",
        Dec,
        "S-0402",
    ),
    (
        "tiberium_long_scan",
        GENERAL,
        "TiberiumLongScan",
        Dec,
        "S-0402",
    ),
    (
        "tiberium_short_scan",
        GENERAL,
        "TiberiumShortScan",
        Dec,
        "S-0402",
    ),
    (
        "harvester_too_far_distance",
        GENERAL,
        "HarvesterTooFarDistance",
        Int,
        "S-0402",
    ),
    (
        "chrono_harv_too_far_distance",
        GENERAL,
        "ChronoHarvTooFarDistance",
        Int,
        "S-0402",
    ),
    (
        "slave_miner_short_scan",
        GENERAL,
        "SlaveMinerShortScan",
        Dec,
        "S-0402",
    ),
    (
        "slave_miner_slave_scan",
        GENERAL,
        "SlaveMinerSlaveScan",
        Dec,
        "S-0402",
    ),
    (
        "slave_miner_long_scan",
        GENERAL,
        "SlaveMinerLongScan",
        Dec,
        "S-0402",
    ),
    (
        "slave_miner_scan_correction",
        GENERAL,
        "SlaveMinerScanCorrection",
        Dec,
        "S-0402",
    ),
    (
        "slave_miner_kick_frame_delay",
        GENERAL,
        "SlaveMinerKickFrameDelay",
        Int,
        "S-0402",
    ),
    ("purifier_bonus", GENERAL, "PurifierBonus", Dec, "S-0402"),
    (
        "ai_virtual_purifiers",
        GENERAL,
        "AIVirtualPurifiers",
        Maxima,
        "S-0402",
    ),
    ("build_speed", GENERAL, "BuildSpeed", Dec, "S-0403"),
    (
        "low_power_penalty_modifier",
        GENERAL,
        "LowPowerPenaltyModifier",
        Dec,
        "S-0403",
    ),
    (
        "min_low_power_production_speed",
        GENERAL,
        "MinLowPowerProductionSpeed",
        Dec,
        "S-0403",
    ),
    (
        "max_low_power_production_speed",
        GENERAL,
        "MaxLowPowerProductionSpeed",
        Dec,
        "S-0403",
    ),
    (
        "multiple_factory",
        GENERAL,
        "MultipleFactory",
        Dec,
        "S-0403",
    ),
    (
        "wall_build_speed_coefficient",
        GENERAL,
        "WallBuildSpeedCoefficient",
        Dec,
        "S-0403",
    ),
    (
        "maximum_queued_objects",
        GENERAL,
        "MaximumQueuedObjects",
        Int,
        "S-0404",
    ),
    (
        "pad_aircraft",
        GENERAL,
        "PadAircraft",
        Refs(Kind::Aircraft),
        "S-0404, S-0407, S-0408",
    ),
    (
        "separate_aircraft",
        GENERAL,
        "SeparateAircraft",
        Bool,
        "S-0404, S-0408",
    ),
    (
        "spy_power_blackout",
        GENERAL,
        "SpyPowerBlackout",
        Int,
        "S-0405",
    ),
    (
        "spy_money_steal_percent",
        GENERAL,
        "SpyMoneyStealPercent",
        Dec,
        "S-0510",
    ),
    ("refund_percent", GENERAL, "RefundPercent", Dec, "S-0408"),
    ("buildup_time", GENERAL, "BuildupTime", Dec, "S-0108"),
    (
        "prerequisite_power",
        GENERAL,
        "PrerequisitePower",
        Refs(Kind::Buildings),
        "S-0407",
    ),
    (
        "prerequisite_factory",
        GENERAL,
        "PrerequisiteFactory",
        Refs(Kind::Buildings),
        "S-0407",
    ),
    (
        "prerequisite_barracks",
        GENERAL,
        "PrerequisiteBarracks",
        Refs(Kind::Buildings),
        "S-0407",
    ),
    (
        "prerequisite_radar",
        GENERAL,
        "PrerequisiteRadar",
        Refs(Kind::Buildings),
        "S-0407",
    ),
    (
        "prerequisite_tech",
        GENERAL,
        "PrerequisiteTech",
        Refs(Kind::Buildings),
        "S-0407",
    ),
    (
        "prerequisite_proc",
        GENERAL,
        "PrerequisiteProc",
        Refs(Kind::Buildings),
        "S-0407",
    ),
    (
        "prerequisite_proc_alternate",
        GENERAL,
        "PrerequisiteProcAlternate",
        Ref(Kind::Vehicles),
        "S-0407",
    ),
    (
        "secret_infantry",
        GENERAL,
        "SecretInfantry",
        Refs(Kind::Infantry),
        "S-0407",
    ),
    (
        "secret_units",
        GENERAL,
        "SecretUnits",
        Refs(Kind::Vehicles),
        "S-0407",
    ),
    (
        "secret_buildings",
        GENERAL,
        "SecretBuildings",
        Refs(Kind::Buildings),
        "S-0407",
    ),
    (
        "leptons_per_sight_increase",
        GENERAL,
        "LeptonsPerSightIncrease",
        Int,
        "S-0409",
    ),
    (
        "reveal_by_height",
        GENERAL,
        "RevealByHeight",
        Bool,
        "S-0409",
    ),
    ("flash_frame_time", GENERAL, "FlashFrameTime", Int, "S-0410"),
    (
        "radar_combat_flash_time",
        GENERAL,
        "RadarCombatFlashTime",
        Int,
        "S-0410",
    ),
    (
        "radar_event_speed",
        GENERAL,
        "RadarEventSpeed",
        Dec,
        "S-0410",
    ),
    (
        "radar_event_min_radius",
        GENERAL,
        "RadarEventMinRadius",
        Int,
        "S-0410",
    ),
    (
        "radar_event_rotation_speed",
        GENERAL,
        "RadarEventRotationSpeed",
        Dec,
        "S-0410",
    ),
    (
        "multiplayer_aicm",
        GENERAL,
        "MultiplayerAICM",
        Maxima,
        "S-0412",
    ),
    ("ai_hate_delays", GENERAL, "AIHateDelays", Maxima, "S-0412"),
    (
        "build_tech",
        AI,
        "BuildTech",
        Refs(Kind::Buildings),
        "S-0407",
    ),
    (
        "build_refinery",
        AI,
        "BuildRefinery",
        Refs(Kind::Buildings),
        "S-0414",
    ),
    ("comp_easy_bonus", AI, "CompEasyBonus", Bool, "S-0412"),
    ("ally_reveal", AUDIO_VISUAL, "AllyReveal", Bool, "S-0409"),
    ("shroud_grow", AUDIO_VISUAL, "ShroudGrow", Bool, "S-0409"),
    ("shroud_rate", AUDIO_VISUAL, "ShroudRate", Dec, "S-0409"),
    ("fog_rate", AUDIO_VISUAL, "FogRate", Dec, "S-0409"),
    ("savour_delay", AUDIO_VISUAL, "SavourDelay", Dec, "S-0414"),
    ("pose_dir", AUDIO_VISUAL, "PoseDir", Int, "S-0404"),
    ("deploy_dir", AUDIO_VISUAL, "DeployDir", Int, "S-0502"),
    ("flight_level", GENERAL, "FlightLevel", Int, "S-0501"),
    ("reload_rate", GENERAL, "ReloadRate", Dec, "S-0501"),
    ("curley_shuffle", GENERAL, "CurleyShuffle", Bool, "S-0501"),
    (
        "attacking_aircraft_sight_range",
        GENERAL,
        "AttackingAircraftSightRange",
        Int,
        "S-0501",
    ),
    (
        "spy_plane_camera_frames",
        AUDIO_VISUAL,
        "SpyPlaneCameraFrames",
        Int,
        "S-0501",
    ),
    (
        "c4_warhead",
        COMBAT_DAMAGE,
        "C4Warhead",
        Ref(Kind::Warheads),
        "S-0414",
    ),
    (
        "crate_regen",
        CRATE_RULES,
        "CrateRegen",
        Dec,
        "S-0520, S-0524",
    ),
    (
        "unit_crate_type",
        CRATE_RULES,
        "UnitCrateType",
        Ref(Kind::Vehicles),
        "S-0520",
    ),
    (
        "wood_crate_img",
        CRATE_RULES,
        "WoodCrateImg",
        Ref(Kind::Overlays),
        "S-0520",
    ),
    (
        "water_crate_img",
        CRATE_RULES,
        "WaterCrateImg",
        Ref(Kind::Overlays),
        "S-0520",
    ),
    (
        "flame_damage",
        COMBAT_DAMAGE,
        "FlameDamage",
        Ref(Kind::Warheads),
        "S-0520",
    ),
    ("crate_minimum", CRATE_RULES, "CrateMinimum", Int, "S-0412"),
    ("crate_maximum", CRATE_RULES, "CrateMaximum", Int, "S-0412"),
    ("money", DIALOG, "Money", Int, "S-0412"),
    ("min_money", DIALOG, "MinMoney", Int, "S-0412"),
    ("max_money", DIALOG, "MaxMoney", Int, "S-0412"),
    ("money_increment", DIALOG, "MoneyIncrement", Int, "S-0412"),
    ("unit_count", DIALOG, "UnitCount", Int, "S-0412"),
    ("min_unit_count", DIALOG, "MinUnitCount", Int, "S-0412"),
    ("max_unit_count", DIALOG, "MaxUnitCount", Int, "S-0412"),
    ("short_game", DIALOG, "ShortGame", Bool, "S-0412"),
    (
        "bridge_destruction",
        DIALOG,
        "BridgeDestruction",
        Bool,
        "S-0511",
    ),
    ("crates", DIALOG, "Crates", Bool, "S-0412"),
    ("mcv_redeploys", DIALOG, "MCVRedeploys", Bool, "S-0412"),
    (
        "super_weapons_allowed",
        DIALOG,
        "SuperWeaponsAllowed",
        Bool,
        "S-0412",
    ),
    (
        "build_off_ally",
        DIALOG,
        "BuildOffAlly",
        Bool,
        "S-0406, S-0412",
    ),
    ("shroud", DIALOG, "Shroud", Bool, "S-0409"),
    ("allies_allowed", DIALOG, "AlliesAllowed", Bool, "S-0414"),
    (
        "ally_change_allowed",
        DIALOG,
        "AllyChangeAllowed",
        Bool,
        "S-0414",
    ),
    (
        "reveal_trigger_radius",
        GENERAL,
        "RevealTriggerRadius",
        Int,
        "S-0011",
    ),
    ("ai_difficulty", DIALOG, "AIDifficulty", Int, "S-0011"),
];

/// Game-wide superweapon values (`general.superweapons`): our field, the
/// section, the key, the form and the spec.
const GENERAL_SW: &[(&str, &[u8], &str, Form, &str)] = &[
    (
        "charge_to_drain_ratio",
        GENERAL,
        "ChargeToDrainRatio",
        Dec,
        "S-0512",
    ),
    (
        "nuke_take_off",
        GENERAL,
        "NukeTakeOff",
        Ref(Kind::Animations),
        "S-0513",
    ),
    ("dig_sound", AUDIO_VISUAL, "DigSound", Text, "S-0513"),
    (
        "lightning_deferment",
        GENERAL,
        "LightningDeferment",
        Int,
        "S-0513",
    ),
    (
        "lightning_storm_duration",
        GENERAL,
        "LightningStormDuration",
        Int,
        "S-0513",
    ),
    (
        "lightning_hit_delay",
        GENERAL,
        "LightningHitDelay",
        Int,
        "S-0513",
    ),
    (
        "lightning_scatter_delay",
        GENERAL,
        "LightningScatterDelay",
        Int,
        "S-0513",
    ),
    (
        "lightning_cell_spread",
        GENERAL,
        "LightningCellSpread",
        Int,
        "S-0513",
    ),
    (
        "lightning_separation",
        GENERAL,
        "LightningSeparation",
        Int,
        "S-0513",
    ),
    (
        "lightning_damage",
        GENERAL,
        "LightningDamage",
        Int,
        "S-0513",
    ),
    (
        "lightning_warhead",
        GENERAL,
        "LightningWarhead",
        Ref(Kind::Warheads),
        "S-0513",
    ),
    (
        "lightning_print_text",
        GENERAL,
        "LightningPrintText",
        Bool,
        "S-0513",
    ),
    (
        "weather_con_clouds",
        GENERAL,
        "WeatherConClouds",
        Refs(Kind::Animations),
        "S-0513",
    ),
    (
        "weather_con_bolts",
        GENERAL,
        "WeatherConBolts",
        Refs(Kind::Animations),
        "S-0513",
    ),
    (
        "weather_con_bolt_explosion",
        GENERAL,
        "WeatherConBoltExplosion",
        Ref(Kind::Animations),
        "S-0513",
    ),
    ("storm_sound", AUDIO_VISUAL, "StormSound", Text, "S-0513"),
    (
        "lightning_sounds",
        AUDIO_VISUAL,
        "LightningSounds",
        Texts,
        "S-0513",
    ),
    (
        "iron_curtain_duration",
        COMBAT_DAMAGE,
        "IronCurtainDuration",
        Int,
        "S-0513",
    ),
    (
        "iron_curtain_invoke_anim",
        GENERAL,
        "IronCurtainInvokeAnim",
        Ref(Kind::Animations),
        "S-0513",
    ),
    (
        "force_shield_invoke_anim",
        GENERAL,
        "ForceShieldInvokeAnim",
        Ref(Kind::Animations),
        "S-0513",
    ),
    (
        "force_shield_radius",
        GENERAL,
        "ForceShieldRadius",
        Int,
        "S-0513",
    ),
    (
        "force_shield_duration",
        GENERAL,
        "ForceShieldDuration",
        Int,
        "S-0513",
    ),
    (
        "force_shield_blackout_duration",
        GENERAL,
        "ForceShieldBlackoutDuration",
        Int,
        "S-0513",
    ),
    (
        "force_shield_play_fade_sound_time",
        GENERAL,
        "ForceShieldPlayFadeSoundTime",
        Int,
        "S-0513",
    ),
    (
        "iron_curtain_color",
        AUDIO_VISUAL,
        "IronCurtainColor",
        Int,
        "S-0513",
    ),
    (
        "force_shield_color",
        AUDIO_VISUAL,
        "ForceShieldColor",
        Int,
        "S-0513",
    ),
    (
        "chrono_placement",
        GENERAL,
        "ChronoPlacement",
        Ref(Kind::Animations),
        "S-0513",
    ),
    (
        "chrono_blast",
        GENERAL,
        "ChronoBlast",
        Ref(Kind::Animations),
        "S-0513",
    ),
    (
        "chrono_blast_dest",
        GENERAL,
        "ChronoBlastDest",
        Ref(Kind::Animations),
        "S-0513",
    ),
    ("chrono_delay", GENERAL, "ChronoDelay", Int, "S-0513"),
    (
        "chrono_reinf_delay",
        GENERAL,
        "ChronoReinfDelay",
        Int,
        "S-0513",
    ),
    (
        "ally_para_drop_inf",
        GENERAL,
        "AllyParaDropInf",
        Refs(Kind::Infantry),
        "S-0522, S-0513",
    ),
    (
        "ally_para_drop_num",
        GENERAL,
        "AllyParaDropNum",
        Maxima,
        "S-0522, S-0513",
    ),
    (
        "sov_para_drop_inf",
        GENERAL,
        "SovParaDropInf",
        Refs(Kind::Infantry),
        "S-0522",
    ),
    (
        "sov_para_drop_num",
        GENERAL,
        "SovParaDropNum",
        Maxima,
        "S-0522",
    ),
    (
        "yuri_para_drop_inf",
        GENERAL,
        "YuriParaDropInf",
        Refs(Kind::Infantry),
        "S-0522",
    ),
    (
        "yuri_para_drop_num",
        GENERAL,
        "YuriParaDropNum",
        Maxima,
        "S-0522",
    ),
    (
        "amer_para_drop_inf",
        GENERAL,
        "AmerParaDropInf",
        Refs(Kind::Infantry),
        "S-0522",
    ),
    (
        "amer_para_drop_num",
        GENERAL,
        "AmerParaDropNum",
        Maxima,
        "S-0522",
    ),
    (
        "spy_plane_camera",
        AUDIO_VISUAL,
        "SpyPlaneCamera",
        Text,
        "S-0513",
    ),
    ("paradrop_radius", GENERAL, "ParadropRadius", Int, "S-0522"),
    (
        "parachute",
        GENERAL,
        "Parachute",
        Ref(Kind::Animations),
        "S-0522",
    ),
    ("chute_sound", AUDIO_VISUAL, "ChuteSound", Text, "S-0522"),
    (
        "psychic_reveal_radius",
        COMBAT_DAMAGE,
        "PsychicRevealRadius",
        Int,
        "S-0610",
    ),
    (
        "psychic_reveal_activate_sound",
        AUDIO_VISUAL,
        "PsychicRevealActivateSound",
        Text,
        "S-0610",
    ),
    ("berserk_color", AUDIO_VISUAL, "BerserkColor", Int, "S-0610"),
    // spec: S-0610 claim 16 (BerzerkAllowed is read but has no effect)
    (
        "berzerk_allowed",
        COMBAT_DAMAGE,
        "BerzerkAllowed",
        Bool,
        "S-0610",
    ),
    // spec: S-0611 Import mapping (MutateExplosion a boolean, absent no: cell mode; the two warheads of [SpecialWeapons] and the conversion and launch animations are references, absent none; AnimToInfantry an infantry type list, absent empty; GeneticMutatorActivateSound a sound name, absent none)
    (
        "mutate_explosion",
        GENERAL,
        "MutateExplosion",
        Bool,
        "S-0611",
    ),
    (
        "mutate_explosion_warhead",
        SPECIAL_WEAPONS,
        "MutateExplosionWarhead",
        Ref(Kind::Warheads),
        "S-0611",
    ),
    (
        "mutate_warhead",
        SPECIAL_WEAPONS,
        "MutateWarhead",
        Ref(Kind::Warheads),
        "S-0611",
    ),
    (
        "infantry_mutate",
        GENERAL,
        "InfantryMutate",
        Ref(Kind::Animations),
        "S-0611",
    ),
    (
        "anim_to_infantry",
        GENERAL,
        "AnimToInfantry",
        Refs(Kind::Infantry),
        "S-0611",
    ),
    (
        "ion_blast",
        GENERAL,
        "IonBlast",
        Ref(Kind::Animations),
        "S-0611",
    ),
    (
        "genetic_mutator_activate_sound",
        AUDIO_VISUAL,
        "GeneticMutatorActivateSound",
        Text,
        "S-0611",
    ),
    // spec: S-0612 Import mapping (the two animation types and the warhead are references, absent none; the percentage, range and damage integers, absent 50, 2 and 50; PsychicDominatorActivateSound a sound name, absent none)
    (
        "dominator_first_anim",
        GENERAL,
        "DominatorFirstAnim",
        Ref(Kind::Animations),
        "S-0612",
    ),
    (
        "dominator_second_anim",
        GENERAL,
        "DominatorSecondAnim",
        Ref(Kind::Animations),
        "S-0612",
    ),
    (
        "dominator_fire_at_percentage",
        GENERAL,
        "DominatorFireAtPercentage",
        Int,
        "S-0612",
    ),
    (
        "dominator_capture_range",
        GENERAL,
        "DominatorCaptureRange",
        Int,
        "S-0612",
    ),
    (
        "dominator_damage",
        GENERAL,
        "DominatorDamage",
        Int,
        "S-0612",
    ),
    (
        "dominator_warhead",
        GENERAL,
        "DominatorWarhead",
        Ref(Kind::Warheads),
        "S-0612",
    ),
    (
        "psychic_dominator_activate_sound",
        AUDIO_VISUAL,
        "PsychicDominatorActivateSound",
        Text,
        "S-0612",
    ),
];

/// The coverage rows of the keys this module converts: (category, key,
/// spec).
pub fn converted_keys() -> Vec<(&'static str, String, &'static str)> {
    let mut out = Vec::new();
    const TYPES: [&str; 4] = ["infantry", "vehicles", "aircraft", "buildings"];
    const UNITS: [&str; 3] = ["infantry", "vehicles", "aircraft"];
    let mut add = |cats: &[&'static str], rows: &[Row]| {
        for c in cats {
            for r in rows {
                out.push((*c, r.1.to_owned(), r.3));
            }
        }
    };
    add(&TYPES, ECON_FIELDS);
    add(&UNITS, UNIT_ECON);
    add(&["infantry"], INFANTRY_ECON);
    add(&["vehicles"], VEHICLE_ECON);
    add(&["aircraft"], AIRCRAFT_ECON);
    add(&["buildings"], BUILDING_ECON);
    add(&["countries"], COUNTRY_ECON);
    add(&["tiberiums"], TIBERIUM_FIELDS);
    add(&["superweapons"], SUPERWEAPON_FIELDS);
    for c in TYPES {
        for (_, k, spec) in NAME_LISTS {
            out.push((c, (*k).to_owned(), *spec));
        }
    }
    out.push(("buildings", "Owner".to_owned(), "S-0407"));
    out.push(("buildings", "ExitCoord".to_owned(), "S-0404"));
    for k in NO_EFFECT_FACTORS {
        out.push(("countries", k.to_owned(), "S-0413"));
    }
    for (_, section, k, _, spec) in GENERAL_ECON.iter().chain(GENERAL_SW) {
        let cat = match *section {
            s if s == AI => "global:AI",
            s if s == AUDIO_VISUAL => "global:AudioVisual",
            s if s == COMBAT_DAMAGE => "global:CombatDamage",
            s if s == CRATE_RULES => "global:CrateRules",
            s if s == DIALOG => "global:MultiplayerDialogSettings",
            s if s == SPECIAL_WEAPONS => "global:SpecialWeapons",
            _ => "global:General",
        };
        out.push((cat, (*k).to_owned(), *spec));
    }
    out.push((
        "global:MultiplayerDialogSettings",
        "Bases".to_owned(),
        "S-0412",
    ));
    for (_, k) in DOCK_ANIMS {
        // spec: S-0606 Import mapping (special-2 to special-4 read for the bunker walls)
        let spec = if k.starts_with("SpecialAnimTwo")
            || k.starts_with("SpecialAnimThree")
            || k.starts_with("SpecialAnimFour")
        {
            "S-0606"
        } else {
            "S-0402"
        };
        out.push(("art", (*k).to_owned(), spec));
    }
    out.push(("art", "QueueingCell".to_owned(), "S-0402"));
    out.push(("art", "NumberOfDocks".to_owned(), "S-0402"));
    out.push(("art", "Buildup".to_owned(), "S-0108, S-0408"));
    out.push(("art", "GateStages".to_owned(), "S-0108, S-0406"));
    out
}

impl Builder<'_> {
    /// The fields of an economy table that the type's layered keys give.
    fn efields(&mut self, rec: &mut Rec, t: &TypeRec, kind: Kind, table: &[Row]) {
        for &(field, k, form, _) in table {
            let what = format!("{} {} {k}", kind.key(), show(&t.name));
            if let Some(n) = self.field(t.keys.get(k.as_bytes()), form, &what) {
                rec.put(field, n);
            }
        }
    }

    /// A list of names kept as written in upper case (S-0003 claim 12: split
    /// at commas, empty items skipped; a given list replaces the held one).
    fn upper_list(state: Option<&KeyState>, untrimmed: bool) -> Option<Node> {
        fold(state, None, |v, _| {
            Some(
                // spec: S-0003 claim 12 (quirks.list_items_untrimmed off: name items trimmed)
                value::name_items(v, value::COMMON_TEXT_LIMIT, untrimmed)
                    .into_iter()
                    .map(|item| jsonc::string(encode(&item.to_ascii_uppercase())))
                    .collect::<Vec<_>>(),
            )
        })
        .map(jsonc::array)
    }

    /// The `econ` record of an infantry, vehicle, aircraft or building type.
    pub(super) fn type_econ(&mut self, rec: &mut Rec, kind: Kind, index: usize, t: &TypeRec) {
        let mut econ = Rec::default();
        self.efields(&mut econ, t, kind, ECON_FIELDS);
        for &(field, k, _) in NAME_LISTS {
            if let Some(n) = Self::upper_list(
                t.keys.get(k.as_bytes()),
                self.i.ini_quirks.list_items_untrimmed,
            ) {
                econ.put(field, n);
            }
        }
        match kind {
            Kind::Infantry => {
                self.efields(&mut econ, t, kind, UNIT_ECON);
                self.efields(&mut econ, t, kind, INFANTRY_ECON);
            }
            Kind::Vehicles => {
                self.efields(&mut econ, t, kind, UNIT_ECON);
                self.efields(&mut econ, t, kind, VEHICLE_ECON);
                self.dock_unload_warning(t);
                self.zero_capacity_warning(t);
            }
            Kind::Aircraft => {
                self.efields(&mut econ, t, kind, UNIT_ECON);
                self.efields(&mut econ, t, kind, AIRCRAFT_ECON);
            }
            Kind::Buildings => {
                self.efields(&mut econ, t, kind, BUILDING_ECON);
                if let Some(v) = self.rules_triple(t, "ExitCoord") {
                    econ.put(
                        "exit_coord",
                        jsonc::array(v.iter().map(|&x| int_node(x)).collect()),
                    );
                }
                self.building_art_econ(&mut econ, index, t);
                self.upgrade_sale_warnings(t);
                self.depot_docks_warning(t);
                self.deploy_facing_warning(t);
                if let Some(word) = self.factory_kind(t) {
                    rec.put("factory", jsonc::string(word.to_string()));
                }
                // spec: S-0407 Import mapping (Owner: the owner countries, compared with the country registry)
                if let Some(list) =
                    self.name_list(t.keys.get(&b"Owner"[..]), Kind::Countries, "Owner")
                {
                    rec.put("owner", list);
                }
            }
            _ => {}
        }
        if !econ.0.is_empty() {
            rec.put("econ", econ.node());
        }
    }

    /// An integer triple of a type's rules keys (S-0003 claim 9): unread
    /// components keep the value in force's (0 before any), with a warning;
    /// a reset gives none.
    fn rules_triple(&mut self, t: &TypeRec, k: &str) -> Option<[i32; 3]> {
        let mut held: Option<[i32; 3]> = None;
        let mut unread_at = None;
        for e in t
            .keys
            .get(k.as_bytes())
            .into_iter()
            .flat_map(|s| &s.history)
        {
            match &e.what {
                What::Set(v) => {
                    // spec: S-0003 claim 9 (unread components take the value in force's, our engine; the importer warns)
                    let (out, unread) = value::int_tuple_or::<3>(v, held.unwrap_or([0; 3]));
                    if unread {
                        unread_at = Some((e.layer, e.line, v.clone()));
                    }
                    held = Some(out);
                }
                What::Reset => held = None,
                _ => {}
            }
        }
        if let Some((layer, line, v)) = unread_at {
            self.warn(layer, line, format!(
                "buildings {} {k} {:?} has fewer than 3 readable integers; the others keep the value in force (S-0003 claim 9)",
                show(&t.name),
                show(&v)
            ));
        }
        held
    }

    /// The importer warning of S-0314 Edge cases (several contact places):
    /// a repair depot with a rules `NumberOfDocks` above 1.
    fn depot_docks_warning(&mut self, t: &TypeRec) {
        let repair = fold(t.keys.get(&b"UnitRepair"[..]), None, |v, _| {
            value::boolean(v)
        });
        let docks = fold(t.keys.get(&b"NumberOfDocks"[..]), None, |v, e| {
            value::integer(v).map(|n| (n, e.layer, e.line))
        });
        if repair == Some(true)
            && let Some((n, layer, line)) = docks
            && n > 1
        {
            // spec: S-0314 Edge cases (several contact places: the importer warns for a count above 1 on a repair depot)
            self.warn(layer, line, format!(
                "buildings {} NumberOfDocks={n} on a repair depot: the original services only place 0; our engine services the lowest occupied place, one at a time (S-0314 Edge cases)",
                show(&t.name)
            ));
        }
    }

    /// The importer warnings of S-0408 Edge cases: a non-turret upgrade
    /// with a level field of 2 or 3, and a target upgrade limit above 3,
    /// can leave an upgrade sale at an empty position.
    fn upgrade_sale_warnings(&mut self, t: &TypeRec) {
        let int = |k: &[u8]| {
            fold(t.keys.get(k), None, |v, e| {
                value::integer(v).map(|n| (n, e.layer, e.line))
            })
        };
        let turret = fold(t.keys.get(&b"Turret"[..]), None, |v, _| value::boolean(v));
        if let Some((n, layer, line)) = int(b"PowersUpToLevel")
            && (2..=3).contains(&n)
            && turret != Some(true)
        {
            // spec: S-0408 Edge cases (the importer warns about a non-turret level-field upgrade above 1)
            self.warn(layer, line, format!(
                "buildings {} PowersUpToLevel={n} without Turret: selling it leaves a positive upgrade count at an empty position (S-0408 claim 5 and Edge cases)",
                show(&t.name)
            ));
        }
        if let Some((n, layer, line)) = int(b"Upgrades")
            && n > 3
        {
            // spec: S-0408 Edge cases (the importer warns about target limits above 3)
            self.warn(layer, line, format!(
                "buildings {} Upgrades={n} is above the 3 upgrade positions: a sale above 3 finds no position (S-0406 claim 12, S-0408 Edge cases)",
                show(&t.name)
            ));
        }
    }

    /// The importer warning of S-0402 Edge cases: a building type in a
    /// harvester's dock list that lacks the dock-unload flag never directs
    /// the harvester to a docking cell.
    fn dock_unload_warning(&mut self, t: &TypeRec) {
        let flag = |state: Option<&KeyState>| fold(state, None, |v, _| value::boolean(v));
        if flag(t.keys.get(&b"Harvester"[..])) != Some(true) {
            return;
        }
        let untrimmed = self.i.ini_quirks.list_items_untrimmed;
        let Some((names, layer, line)) = fold(t.keys.get(&b"Dock"[..]), None, |v, e| {
            Some((
                value::name_items(v, value::COMMON_TEXT_LIMIT, untrimmed)
                    .into_iter()
                    .map(<[u8]>::to_vec)
                    .collect::<Vec<_>>(),
                e.layer,
                e.line,
            ))
        }) else {
            return;
        };
        let outcome = self.i.outcome;
        let reg = &outcome.registries[&Kind::Buildings];
        for n in names {
            // spec: S-0003 claim 14 (names compared with ASCII case folding)
            let Some(b) = reg.iter().find(|b| b.name.eq_ignore_ascii_case(&n)) else {
                continue;
            };
            if flag(b.keys.get(&b"DockUnload"[..])) != Some(true) {
                // spec: S-0402 Edge cases (the importer warns when a building type in a harvester's dock list lacks the dock-unload flag)
                self.warn(layer, line, format!(
                    "vehicles {} Dock {}: no DockUnload, so a harvester never unloads there (S-0402 claim 10)",
                    show(&t.name),
                    show(&n)
                ));
            }
        }
    }

    /// The importer warning of S-0402 claim 1: a vehicle harvester whose
    /// capacity (`Storage`, absent 0) is 0 is bad data, in either setting of
    /// `quirks.zero_capacity_harvester_cycles`.
    fn zero_capacity_warning(&mut self, t: &TypeRec) {
        let flag = |state: Option<&KeyState>| fold(state, None, |v, _| value::boolean(v));
        if flag(t.keys.get(&b"Harvester"[..])) != Some(true) {
            return;
        }
        let storage = fold(t.keys.get(&b"Storage"[..]), None, |v, e| {
            Some((value::integer(v), e.layer, e.line))
        });
        // spec: S-0402 Import mapping (Storage absent: 0)
        let (c, layer, line) = storage.unwrap_or((Some(0), 0, 0));
        if c == Some(0) {
            // spec: S-0402 claim 1 and Import mapping (zero vehicle capacity warns in either setting)
            self.warn(layer, line, format!(
                "vehicles {} Storage 0: a harvester that never carries a bale, or 1 bale with quirks.zero_capacity_harvester_cycles off (S-0402 claim 1)",
                show(&t.name)
            ));
        }
    }

    /// The importer warning of S-0415 Edge cases and S-0406 Import mapping:
    /// a deploy facing outside 0 … 255 never lets a vehicle deploy into the
    /// type.
    fn deploy_facing_warning(&mut self, t: &TypeRec) {
        let d = fold(t.keys.get(&b"DeployFacing"[..]), None, |v, _| {
            value::integer(v)
        });
        // spec: S-0415 Edge cases and S-0406 claim 16 (a deploy facing outside 0 … 255 prevents deployment; the importer warns)
        if let Some(d) = d.filter(|d| !(0..=255).contains(d)) {
            self.warn(0, 0, format!(
                "buildings {} DeployFacing {d}: outside 0 … 255, so no vehicle ever deploys into it (S-0415 Edge cases; S-0406 claim 16)",
                show(&t.name)
            ));
        }
    }

    /// The importer warning of S-0406 claim 18 and Import mapping: a type
    /// whose construction range (S-0108 claim 5) has a positive delay and 1
    /// stage or fewer never completes its construction.
    fn single_stage_buildup_warning(
        &mut self,
        t: &TypeRec,
        frames: Option<i32>,
        gate_stages: Option<i32>,
    ) {
        let gate = fold(t.keys.get(&b"Gate"[..]), None, |v, _| value::boolean(v)) == Some(true);
        let n = if gate {
            // spec: S-0108 claim 5 and Import mapping (a gate: n = G + 1, GateStages absent 9)
            gate_stages.unwrap_or(9).saturating_add(1)
        } else {
            // spec: S-0108 claim 5 (n = ⌊N_b / 2⌋)
            frames.unwrap_or(0).div_euclid(2)
        };
        if n >= 2 || !self.buildup_delay_positive(n) {
            return;
        }
        // spec: S-0406 claim 18 and Import mapping (the importer warns about a positive-delay range of 1 stage or fewer, in both states of the switch)
        self.warn(0, 0, format!(
            "buildings {}: its build-up has {n} stage(s) with a positive delay, so its construction never completes (S-0406 claim 18; quirks.single_stage_buildup_never_completes)",
            show(&t.name)
        ));
    }

    /// Whether the build-up delay d_b = trunc(B × 900 / n) is positive
    /// (S-0108 claim 5, F1), with *B* the general build-up time in force.
    fn buildup_delay_positive(&self, n: i32) -> bool {
        if n <= 0 {
            // spec: S-0108 claim 5 (n ≤ 0: d_b = 1)
            return true;
        }
        let state = self
            .i
            .outcome
            .globals
            .get(GENERAL)
            .and_then(|s| s.get(&b"BuildupTime"[..]));
        // spec: S-0108 Import mapping (BuildupTime absent: 0.05)
        let text = fold(state, None, |v, _| decimal_text(v)).unwrap_or_else(|| "5e-2".into());
        decimal_times_at_least(&text, 900, n)
    }

    /// The docking displacements of S-0404 claim 15: `DockingOffset`*i*
    /// for each *i* below the rules `NumberOfDocks` count (absent 1) that an
    /// image section gives, as a list up to the highest such *i*; a place
    /// without a key starts at (0, 0, 0). None when no such key is given.
    fn docking_offsets(&mut self, index: usize, t: &TypeRec) -> Option<Vec<[i32; 3]>> {
        let kind = Kind::Buildings;
        // spec: S-0404 Import mapping (NumberOfDocks, rules: an integer, absent 1)
        let n = fold(t.keys.get(&b"NumberOfDocks"[..]), None, |v, _| {
            value::integer(v)
        })
        .unwrap_or(1);
        const PREFIX: &[u8] = b"DockingOffset";
        let mut found: std::collections::BTreeSet<i32> = std::collections::BTreeSet::new();
        for image in self.images(kind, index, t) {
            let Some(s) = self.art_section(&image) else {
                continue;
            };
            for e in s.entries() {
                let k = &e.key;
                if k.len() <= PREFIX.len() || !k[..PREFIX.len()].eq_ignore_ascii_case(PREFIX) {
                    continue;
                }
                let digits = &k[PREFIX.len()..];
                if let Some(i) = std::str::from_utf8(digits)
                    .ok()
                    .and_then(|d| d.parse::<i32>().ok())
                    && i.to_string().as_bytes() == digits
                    && i < n
                {
                    found.insert(i);
                }
            }
        }
        let last = *found.last()?;
        let mut out = vec![[0; 3]; last as usize + 1];
        for i in found {
            // spec: S-0404 Import mapping (DockingOffset i: an integer triple; keep the value in force; new places start at (0, 0, 0))
            if let Some(v) = self.art_tuple::<3>(kind, index, t, &format!("DockingOffset{i}")) {
                out[i as usize] = v;
            }
        }
        Some(out)
    }

    /// The art fields of a building type's image section (S-0402 Import
    /// mapping).
    fn building_art_econ(&mut self, econ: &mut Rec, index: usize, t: &TypeRec) {
        let kind = Kind::Buildings;
        // spec: S-0402 Import mapping (art NumberOfDocks: an integer)
        if let Some(v) = self.art_int(kind, index, t, "NumberOfDocks") {
            econ.put("art_number_of_docks", int_node(v));
        }
        // spec: S-0404 claim 15 and Import mapping (DockingOffset0, 1, …: integer triples, only indices below the rules NumberOfDocks count, absent 1, are read)
        if let Some(v) = self.docking_offsets(index, t) {
            econ.put(
                "docking_offsets",
                jsonc::array(
                    v.iter()
                        .map(|d| jsonc::array(d.iter().map(|&x| int_node(x)).collect()))
                        .collect(),
                ),
            );
        }
        // spec: S-0402 Import mapping (art QueueingCell: two integers, cells)
        if let Some(v) = self.art_tuple::<2>(kind, index, t, "QueueingCell") {
            econ.put(
                "queueing_cell",
                jsonc::array(v.iter().map(|&x| int_node(x)).collect()),
            );
        }
        // spec: S-0108 Import mapping (AnimIdle: three integers start, count, delay, absent 0, 1, 0; unread components keep the held values, with a warning)
        if let Some(v) = self.art_tuple_from::<3>(kind, index, t, "AnimIdle", [0, 1, 0]) {
            econ.put(
                "anim_idle",
                jsonc::array(v.iter().map(|&x| int_node(x)).collect()),
            );
        }
        // spec: S-0108 Import mapping (Normalized: a boolean, absent false)
        if let Some(v) = self.art_bool(kind, index, t, "Normalized") {
            econ.put("normalized", jsonc::boolean(v));
        }
        // spec: S-0108 Import mapping (Buildup: a name; none gives no build-up sprite, an empty value keeps the earlier name)
        let buildup =
            self.art_values(kind, index, t, "Buildup")
                .iter()
                .fold(None, |held, (v, _)| {
                    let name = value::text(v, BUILDUP_MAX);
                    if name.is_empty() {
                        held
                    } else {
                        Some(!value::is_no_type(name))
                    }
                });
        // spec: S-0108 claim 5 and S-0007 claim 11 (with the art looked up, a build-up name whose file does not resolve gives no build-up sprite)
        let art = self.i.arts.and_then(|a| a.get(&(kind, index)));
        let buildup = match art {
            Some(a) => buildup == Some(true) && a.building_sprites.contains_key("buildup"),
            None => buildup == Some(true),
        };
        let frames = art.and_then(|a| a.buildup_frames);
        if buildup {
            econ.put("buildup", jsonc::boolean(true));
            // spec: S-0108 claim 5 (N_b, the build-up sprite's image-frame count)
            if let Some(n) = frames {
                econ.put("buildup_frames", int_node(n));
            }
        }
        // spec: S-0108 Import mapping (GateStages: an integer, absent 9; S-0406 claim 15)
        let gate_stages = self.art_int(kind, index, t, "GateStages");
        if let Some(v) = gate_stages {
            econ.put("gate_stages", int_node(v));
        }
        if buildup && art.is_some() {
            self.single_stage_buildup_warning(t, frames, gate_stages);
        }
        // spec: S-0402 Import mapping (the docking, payment and unloading animations: animation type names)
        for &(field, k) in DOCK_ANIMS {
            let name = self
                .art_values(kind, index, t, k)
                .iter()
                .fold(None, |held, (v, _)| {
                    let name = value::text(v, value::COMMON_TEXT_LIMIT);
                    if name.is_empty() {
                        held
                    } else if value::is_no_type(name) {
                        None
                    } else {
                        Some(encode(name))
                    }
                });
            if let Some(n) = name {
                econ.put(field, jsonc::string(n));
            }
        }
        self.slot_names(kind, index, t, econ);
    }

    /// The *pre-production*, *production* and *special-1* to *special-3*
    /// slot names of S-0108's Import mapping, for the simulation's slot
    /// model (S-0108 claim 20, S-0402 claim 11): at each read of the image sections, a non-empty value
    /// replaces the held name and an empty damaged name then takes the
    /// held normal name.
    fn slot_names(&mut self, kind: Kind, index: usize, t: &TypeRec, econ: &mut Rec) {
        let mut entries = Vec::new();
        for base in SLOT_MODEL_BASES {
            let normal_key = (*base).to_owned();
            let damaged_key = format!("{base}Damaged");
            let mut held: [Vec<u8>; 2] = [Vec::new(), Vec::new()];
            for image in self.images(kind, index, t) {
                let Some(section) = self.art_section(&image) else {
                    continue;
                };
                for (i, k) in [&normal_key, &damaged_key].into_iter().enumerate() {
                    // spec: S-0108 Import mapping (names are text cut to 15 bytes; an empty value keeps the earlier name)
                    if let Some(e) = section.get(k.as_bytes()) {
                        let v = value::text(&e.value, SLOT_NAME_LIMIT);
                        if !v.is_empty() {
                            held[i] = v.to_vec();
                        }
                    }
                }
                // spec: S-0108 Import mapping (at each read, a damaged name that is empty once that read's names are taken takes the normal name held at that point)
                if held[1].is_empty() {
                    held[1] = held[0].clone();
                }
            }
            if held[0].is_empty() && held[1].is_empty() {
                continue;
            }
            entries.push((
                normal_key,
                jsonc::array(vec![
                    jsonc::string(encode(&held[0])),
                    jsonc::string(encode(&held[1])),
                ]),
            ));
        }
        if !entries.is_empty() {
            econ.put("slot_names", jsonc::object(entries));
        }
    }

    /// The factory kind of a building type as our format's word (S-0003
    /// claim 21, Import mapping row C19), or `None` for no factory.
    fn factory_kind(&mut self, t: &TypeRec) -> Option<&'static str> {
        let mut held: Option<&'static str> = None;
        for e in t
            .keys
            .get(&b"Factory"[..])
            .into_iter()
            .flat_map(|s| &s.history)
        {
            match &e.what {
                What::Set(v) => {
                    // spec: S-0003 claim 21 (claim 1's text reading, 31-byte limit, trailing whitespace trimmed)
                    let word = value::text(v, FACTORY_TEXT_LIMIT);
                    if word.is_empty() {
                        // spec: S-0003 claim 21 (missing or empty keeps the value in force)
                        continue;
                    }
                    match factory_word(word) {
                        Some(w) => held = w,
                        None => {
                            // spec: S-0003 claim 21 (a non-empty unmatched word gives no factory, with claim 16's warning)
                            held = None;
                            self.warn(e.layer, e.line, format!(
                                "buildings {} Factory {:?}: not a factory kind word; the building is not a factory building (S-0003 claim 21)",
                                show(&t.name),
                                show(word)
                            ));
                        }
                    }
                }
                What::Reset => held = None,
                _ => {}
            }
        }
        held
    }

    /// The `econ` record of a country.
    pub(super) fn country_econ(&mut self, rec: &mut Rec, t: &TypeRec) {
        let mut econ = Rec::default();
        let rows: Vec<Row> = COUNTRY_ECON
            .iter()
            .filter(|r| !matches!(r.1, "Prefix" | "Suffix"))
            .copied()
            .collect();
        self.efields(&mut econ, t, Kind::Countries, &rows);
        self.name_affixes(&mut econ, t);
        if !econ.0.is_empty() {
            rec.put("econ", econ.node());
        }
        let name = format!("countries {}", show(&t.name));
        for k in NO_EFFECT_FACTORS {
            for (v, layer, line) in Self::set_values(t.keys.get(k.as_bytes())) {
                let one = decimal_text(&v).is_none_or(|d| is_one_text(&d));
                if !one {
                    // spec: S-0413 claim 2 and Import mapping (a general factor with no effect: the importer warns when it is not 1.0)
                    self.warn(layer, line, format!(
                        "{name} {k} {:?}: the original ignores this general factor (S-0413 claim 2)",
                        show(&v)
                    ));
                }
            }
        }
    }

    /// A country's name prefix and suffix (S-0413 claim 4): a longer value
    /// keeps its first one or three characters, trimmed at the ends, with
    /// an importer warning. An empty value for either affix creates no
    /// entry and keeps the value in force, exactly as omission (S-0413
    /// claims 1 and 4, S-0001 claim 11 and Import mapping row I5; Q-0478):
    /// the lexer never hands an empty value to this reader.
    fn name_affixes(&mut self, econ: &mut Rec, t: &TypeRec) {
        const PREFIX_LEN: usize = 1; // spec: S-0413 Constants (prefix characters kept: 1)
        const SUFFIX_LEN: usize = 3; // spec: S-0413 Constants (suffix characters kept: 3)
        let name = format!("countries {}", show(&t.name));
        for (field, k, n) in [
            ("prefix", "Prefix", PREFIX_LEN),
            ("suffix", "Suffix", SUFFIX_LEN),
        ] {
            let Some(state) = t.keys.get(k.as_bytes()) else {
                continue;
            };
            let mut held: Option<Vec<u8>> = None;
            for e in &state.history {
                let What::Set(v) = &e.what else { continue };
                if v.len() > n {
                    // spec: S-0413 claim 4 (our importer warns for a longer value)
                    self.warn(e.layer, e.line, format!(
                        "{name} {k} {:?}: only the first {n} character(s) are kept (S-0413 claim 4)",
                        show(v)
                    ));
                }
                // spec: S-0413 claim 4 (the first one or three characters, whitespace at their ends removed)
                held = Some(v[..v.len().min(n)].trim_ascii().to_vec());
            }
            if let Some(v) = held {
                econ.put(field, jsonc::string(encode(&v)));
            }
        }
    }

    fn set_values(state: Option<&KeyState>) -> Vec<(Vec<u8>, usize, u32)> {
        state
            .into_iter()
            .flat_map(|s| &s.history)
            .filter_map(|e| match &e.what {
                What::Set(v) => Some((v.clone(), e.layer, e.line)),
                _ => None,
            })
            .collect()
    }

    /// A tiberium record.
    pub(super) fn tiberium(&mut self, rec: &mut Rec, t: &TypeRec) {
        self.efields(rec, t, Kind::Tiberiums, TIBERIUM_FIELDS);
        let images: Vec<i32> = Self::set_values(t.keys.get(&b"Image"[..]))
            .iter()
            .filter_map(|(v, _, _)| value::integer(v))
            .collect();
        if let Some([p, f, s]) = image_run(&images) {
            rec.put(
                "image_run",
                jsonc::array(vec![int_node(p), int_node(f), int_node(s)]),
            );
        }
    }

    /// A superweapon record.
    pub(super) fn superweapon(&mut self, rec: &mut Rec, t: &TypeRec) {
        // The kind, link and action words follow their own layering (below).
        let plain: Vec<Row> = SUPERWEAPON_FIELDS
            .iter()
            .copied()
            .filter(|r| !matches!(r.0, "kind" | "pre_dependent" | "action"))
            .collect();
        self.efields(rec, t, Kind::Superweapons, &plain);
        // spec: S-0512 claim 24 (Type and PreDependent: only one of the twelve words, matched whole and without regard to case, replaces the kind or link; an unknown word keeps the previous layer's; no hyphen, underscore or space is ignored)
        for (field, key) in [
            ("kind", &b"Type"[..]),
            ("pre_dependent", &b"PreDependent"[..]),
        ] {
            let mut unknown = Vec::new();
            let word = fold(t.keys.get(key), None, |v, e| {
                let w = value::text(v, value::COMMON_TEXT_LIMIT);
                let known = std::str::from_utf8(w)
                    .ok()
                    .filter(|w| superweapon_kind_number(w).is_some())
                    .map(str::to_ascii_lowercase);
                if known.is_none() {
                    unknown.push((e.layer, e.line, show(w)));
                }
                known
            });
            for (layer, line, w) in unknown {
                self.warn(
                    layer,
                    line,
                    format!(
                        "superweapon {} {field}: {w:?} is no kind word; the previous layer's value is kept (S-0512 claim 24)",
                        show(&t.name)
                    ),
                );
            }
            if let Some(w) = word {
                rec.put(field, jsonc::string(w));
            }
        }
        // spec: S-0512 claim 23 (Action: a nonempty word replaces the action, a word outside the vocabulary setting no action; an empty or omitted value keeps the previous one, S-0001 claim 11)
        let action = fold(t.keys.get(&b"Action"[..]), None, |v, _| {
            let w = value::text(v, value::COMMON_TEXT_LIMIT).trim_ascii();
            (!w.is_empty()).then(|| {
                let w = String::from_utf8_lossy(w).to_ascii_lowercase();
                // An unrecognised word sets no action: the explicit word `none`.
                if superweapon_action_known(&w) {
                    w
                } else {
                    "none".to_owned()
                }
            })
        });
        if let Some(a) = action {
            rec.put("action", jsonc::string(a));
        }
    }

    /// The game-wide superweapon values (`general.superweapons`).
    pub(super) fn general_sw(&mut self, rec: &mut Rec) {
        let mut sw = Rec::default();
        let globals = &self.i.outcome.globals;
        let rows: Vec<_> = GENERAL_SW
            .iter()
            .map(|&(field, section, k, form, _)| {
                (
                    field,
                    globals
                        .get(section)
                        .and_then(|s| s.get(k.as_bytes()))
                        .cloned(),
                    form,
                    section,
                    k,
                )
            })
            .collect();
        for (field, state, form, section, k) in rows {
            let what = format!("[{}] {k}", show(section));
            if let Some(n) = self.field(state.as_ref(), form, &what) {
                sw.put(field, n);
            }
        }
        if !sw.0.is_empty() {
            rec.put("superweapons", sw.node());
        }
    }

    /// The game-wide economy values (`general.economy`).
    /// The crate radius and the power-up table (S-0520 Import mapping, F3).
    fn crate_import(&mut self, econ: &mut Rec) {
        let globals = &self.i.outcome.globals;
        let radius = globals
            .get(CRATE_RULES)
            .and_then(|s| s.get(&b"CrateRadius"[..]))
            .cloned();
        // spec: S-0520 F3 (a read value of exactly -1 keeps the value in force: it takes no part)
        let text = fold(radius.as_ref(), None, |v, _| {
            let t = decimal_text(v)?;
            let one = glasnost_num::decimal::parse(
                &t,
                glasnost_num::decimal::Field::Decimal,
                glasnost_num::Rounding::Nearest,
            )
            .ok()?;
            (one != glasnost_num::F64::from_i64(-1, glasnost_num::Rounding::Nearest)).then_some(t)
        });
        if let Some(t) = text {
            econ.put("crate_radius", jsonc::string(t));
        }
        let Some(section) = globals.get(POWERUPS).cloned() else {
            return;
        };
        if section.is_empty() {
            return;
        }
        let mut entries = Vec::new();
        for (index, key) in POWERUP_KEYS.into_iter().enumerate() {
            let state = section.get(key.as_bytes());
            // spec: S-0520 claim 28 and Import mapping, S-0006 claim 5 revision 20 (an entry starts at its built-in startup values; a layer holding the section and omitting the key resets weight and animation only with quirks.override_resets_keys on, and the layering records no reset with it off, so an entry no layer mentions keeps its startup weight and no animation whatever the section's presence; the later-game carry-over of claim 10 is not modelled by a single import)
            let mut weight = POWERUP_STARTUP_WEIGHTS[index];
            let mut anim: Option<String> = None;
            let mut water = false;
            let mut arg: Option<String> = None;
            if let Some(st) = state {
                for e in &st.history {
                    if matches!(e.what, What::Reset) {
                        // spec: S-0520 claim 28 and S-0006 claim 5 revision 20 (a layer that contains the power-up section and omits the key resets its weight to 0 and its animation to no animation: the reserved name NONE cannot be registered as an animation, so the reset never finds a type; the water flag and the argument keep the held value; the layering records the reset only with quirks.override_resets_keys on)
                        weight = 0;
                        anim = None;
                        continue;
                    }
                    let What::Set(v) = &e.what else { continue };
                    let items: Vec<&[u8]> =
                        v.split(|&b| b == b',').map(|s| s.trim_ascii()).collect();
                    // spec: S-0520 Import mapping (the list: weight, animation, water flag, argument; a missing or unreadable token keeps the value in force)
                    if let Some(n) = items.first().and_then(|t| value::integer(t)) {
                        weight = n;
                    }
                    if let Some(t) = items.get(1) {
                        anim = self.ref_name(Kind::Animations, t);
                    }
                    if let Some(b) = items.get(2).and_then(|t| value::boolean(t)) {
                        water = b;
                    }
                    if let Some(t) = items.get(3).and_then(|t| decimal_text(t)) {
                        arg = Some(t);
                    }
                }
            }
            let mut obj = vec![
                ("weight".to_owned(), jsonc::integer(i128::from(weight))),
                ("water".to_owned(), jsonc::boolean(water)),
            ];
            if let Some(a) = anim {
                obj.push(("animation".to_owned(), jsonc::string(a)));
            }
            if let Some(a) = arg {
                obj.push(("argument".to_owned(), jsonc::string(a)));
            }
            entries.push(jsonc::object(obj));
        }
        econ.put("powerups", jsonc::array(entries));
    }

    pub(super) fn general_econ(&mut self, rec: &mut Rec) {
        let mut econ = Rec::default();
        let globals = &self.i.outcome.globals;
        let mut rows = Vec::new();
        for &(field, section, k, form, _) in GENERAL_ECON {
            rows.push((
                field,
                globals
                    .get(section)
                    .and_then(|s| s.get(k.as_bytes()))
                    .cloned(),
                form,
                section,
                k,
            ));
        }
        for (field, state, form, section, k) in rows {
            let what = format!("[{}] {k}", show(section));
            if let Some(n) = self.field(state.as_ref(), form, &what) {
                econ.put(field, n);
            }
        }
        self.crate_import(&mut econ);
        // spec: S-0510 Edge cases (a steal fraction outside 0 to 1 is warned about)
        let steal = self
            .i
            .outcome
            .globals
            .get(GENERAL)
            .and_then(|s| s.get(&b"SpyMoneyStealPercent"[..]))
            .cloned();
        if let Some(text) = fold(steal.as_ref(), None, |v, _| decimal_text(v)) {
            let negative = text.starts_with('-');
            let above = !is_one_text(&text) && decimal_times_at_least(&text, 1, 1);
            if negative || above {
                self.warn_at(
                    steal.as_ref(),
                    format!(
                        "[General] SpyMoneyStealPercent {text} is outside 0 to 1 (S-0510; quirks.spy_theft_unbacked_gain)"
                    ),
                );
            }
        }
        // spec: S-0412 claim 1 and Import mapping (Bases: the base rules' value only)
        let bases = self
            .i
            .outcome
            .globals
            .get(DIALOG)
            .and_then(|s| s.get(&b"Bases"[..]));
        if let Some(b) = fold(bases, Some(0), |v, _| value::boolean(v)) {
            econ.put("bases", jsonc::boolean(b));
        }
        if !econ.0.is_empty() {
            rec.put("economy", econ.node());
        }
    }
}

/// Whether decimal text `d` in our grammar, times `k`, is at least `n`
/// (`n` positive): the test d_b ≥ 1 of S-0108 F1 without a double.
pub(super) fn decimal_times_at_least(d: &str, k: u128, n: i32) -> bool {
    let (body, pct) = match d.strip_suffix('%') {
        Some(b) => (b, true),
        None => (d, false),
    };
    if body.starts_with('-') {
        return false;
    }
    let (mant, exp) = match body.split_once('e') {
        Some((m, e)) => (m, e.parse::<i32>().unwrap_or(0)),
        None => (body, 0),
    };
    // spec: S-0003 claim 7 (a percent value is a hundredth)
    let exp = if pct { exp - 2 } else { exp };
    let digits = mant.trim_start_matches('0');
    if digits.is_empty() {
        return false;
    }
    // Long mantissas only make the value larger than any count here.
    let m: u128 = digits[..digits.len().min(30)].parse().unwrap_or(u128::MAX);
    let exp = exp + i32::try_from(digits.len().saturating_sub(30)).unwrap_or(0);
    let lhs = m.saturating_mul(k);
    let n = u128::from(n.unsigned_abs());
    if exp >= 0 {
        lhs.saturating_mul(10u128.saturating_pow(exp.unsigned_abs())) >= n
    } else {
        lhs >= n.saturating_mul(10u128.saturating_pow(exp.unsigned_abs()))
    }
}

/// Whether decimal text in our grammar reads as exactly 1.
pub(super) fn is_one_text(d: &str) -> bool {
    let (body, pct) = match d.strip_suffix('%') {
        Some(b) => (b, true),
        None => (d, false),
    };
    let (mant, exp) = match body.split_once('e') {
        Some((m, e)) => (m, e.parse::<i64>().unwrap_or(0)),
        None => (body, 0),
    };
    if mant.starts_with('-') {
        return false;
    }
    let digits: String = mant.chars().filter(char::is_ascii_digit).collect();
    let frac = mant.split_once('.').map_or(0, |(_, f)| f.len() as i64);
    let digits = digits.trim_start_matches('0');
    let trimmed = digits.trim_end_matches('0');
    let zeros = (digits.len() - trimmed.len()) as i64;
    // value = trimmed × 10^(zeros − frac + exp − (2 if pct))
    let e = zeros - frac + exp - if pct { 2 } else { 0 };
    trimmed == "1" && e == 0
}

/// The image run (*P*, *F*, *S*) that a tiberium's image numbers leave, in
/// the order the rules layers gave them (S-0112 claim 10 and Constants): a
/// value other than 2 and −1 sets all three, 2 sets *P* and *F* and keeps
/// *S* (0 for a tiberium that never had another value), −1 changes
/// nothing. `None`: no run (S-0112 claim 12).
pub(crate) fn image_run(values: &[i32]) -> Option<[i32; 3]> {
    // spec: S-0112 Constants (F = 12 for every image number; S = 8 for every value other than 2 and −1)
    const FLAT: i32 = 12;
    const SLOPED: i32 = 8; // spec: S-0112 Constants (S = 8)
    let mut run: Option<[i32; 3]> = None;
    for &v in values {
        // spec: S-0112 Constants (image runs: 2 → P 27, keeps S; 3 → 127; 4 → 147; any other value except −1 → 102)
        run = match v {
            -1 => run,
            2 => Some([27, FLAT, run.map_or(0, |r| r[2])]),
            3 => Some([127, FLAT, SLOPED]),
            4 => Some([147, FLAT, SLOPED]),
            _ => Some([102, FLAT, SLOPED]),
        };
    }
    run
}

/// The text limit of the factory field (S-0003 Import mapping row C19).
const FACTORY_TEXT_LIMIT: usize = 31; // spec: S-0003 Import mapping row C19 (31-byte text limit)

/// A factory field's word, compared whole ignoring ASCII case (S-0003
/// claim 21, row C19): `Some(Some(category))` for a producing word or a
/// recognised word that names no production kind, in our format's words,
/// `Some(None)` for the no-factory word, `None` when unmatched.
fn factory_word(word: &[u8]) -> Option<Option<&'static str>> {
    let lower = word.to_ascii_lowercase();
    // spec: S-0003 Import mapping row C19 (the four producing words, the no-factory word and the other recognised words)
    // spec: S-0214 Import mapping (Factory: the imported category, including recognised non-producing categories) and claim 17
    Some(match lower.as_slice() {
        b"infantrytype" => Some("infantry"),
        b"unittype" => Some("vehicle"),
        b"aircrafttype" => Some("aircraft"),
        b"buildingtype" => Some("building"),
        b"terraintype" => Some("terrain"),
        b"infantry" => Some("infantry_object"),
        b"unit" => Some("vehicle_object"),
        b"aircraft" => Some("aircraft_object"),
        b"building" => Some("building_object"),
        b"<none>" => None,
        _ => return None,
    })
}
