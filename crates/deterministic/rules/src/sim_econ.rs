//! Typed simulation data of the economy, production, structures, visibility
//! and game-setup specs (M5: S-0401 to S-0416). Like the rest of
//! [`crate::sim`], every field is optional and holds the value in force after
//! every rules layer; the simulation applies the absent-key default the spec
//! named in the field's doc comment.
//!
//! Lists of type names hold type keys (resolved by the importer with S-0003
//! claim 14's comparison, unknown names dropped with a warning) unless the
//! doc comment says the names are kept as written.

use schemars::JsonSchema;
use serde::Deserialize;

use crate::sim::Decimal;

/// Economy fields that infantry, vehicle, aircraft and building types carry
/// (written into `econ`). Fields that only one kind reads say so.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EconFields {
    /// `Prerequisite`, the prerequisite list (S-0407 claim 8): building type
    /// names and the group words, as written (upper case).
    #[serde(default)]
    pub prerequisite: Option<Vec<String>>,
    /// `PrerequisiteOverride`, the override list (S-0407 claim 3), names as
    /// written (upper case).
    #[serde(default)]
    pub prerequisite_override: Option<Vec<String>>,
    /// `RequiredHouses`, country names as written (S-0407 claim 5); absent:
    /// no restriction.
    #[serde(default)]
    pub required_houses: Option<Vec<String>>,
    /// `ForbiddenHouses`, country names as written (S-0407 claim 5); absent:
    /// no restriction.
    #[serde(default)]
    pub forbidden_houses: Option<Vec<String>>,
    /// `RequiresStolenAlliedTech` (S-0407 claim 6); absent: no.
    #[serde(default)]
    pub requires_stolen_allied_tech: Option<bool>,
    /// `RequiresStolenSovietTech` (S-0407 claim 6); absent: no.
    #[serde(default)]
    pub requires_stolen_soviet_tech: Option<bool>,
    /// `RequiresStolenThirdTech` (S-0407 claim 6); absent: no.
    #[serde(default)]
    pub requires_stolen_third_tech: Option<bool>,
    /// `BuildLimit` (S-0407 claims 9, 10, 16); absent: 2³¹ − 1.
    #[serde(default)]
    pub build_limit: Option<i32>,
    /// `BuildTimeMultiplier`, the type multiplier *m* (S-0403 claim 4), a
    /// single; absent: 1.0.
    #[serde(default)]
    pub build_time_multiplier: Option<Decimal>,
    /// `Soylent`, the refund override *v* (S-0408 claim 1); absent: 0.
    #[serde(default)]
    pub soylent: Option<i32>,
    /// `RadarInvisible` (S-0410 claim 7); absent: no.
    #[serde(default)]
    pub radar_invisible: Option<bool>,
    /// `RadarVisible` (S-0410 claim 7); absent: no.
    #[serde(default)]
    pub radar_visible: Option<bool>,
    /// `Invisible` (S-0410 claim 7); absent: no.
    #[serde(default)]
    pub invisible: Option<bool>,
    /// `RevealToAll` (S-0409 claim 8); absent: no.
    #[serde(default)]
    pub reveal_to_all: Option<bool>,
    /// `GapRadiusInCells`, the gap radius *R* (S-0409 claim 11); absent: 0.
    #[serde(default)]
    pub gap_radius_in_cells: Option<i32>,
    /// `SuperGapRadiusInCells` (S-0405 claim 16, S-0409); absent: 0.
    #[serde(default)]
    pub super_gap_radius_in_cells: Option<i32>,
    /// Units: `Dock`, the dock types (S-0402 Terms and claim 8; S-0404
    /// Terms); absent: empty.
    #[serde(default)]
    pub dock: Option<Vec<String>>,
    /// Units and buildings: `ResourceGatherer` (S-0402 claim 14; S-0414
    /// Terms; S-0415 claim 4; S-0308 table P row 6); absent: no.
    #[serde(default)]
    pub resource_gatherer: Option<bool>,
    /// Units and buildings: `ResourceDestination` (S-0402 claim 14; S-0308
    /// table P row 6); absent: no.
    #[serde(default)]
    pub resource_destination: Option<bool>,
    /// Units: `Enslaves`, the slave type (S-0402 claim 14); absent: none.
    #[serde(default)]
    pub enslaves: Option<String>,
    /// Units: `SlavesNumber` (S-0402 claim 14); absent: 0.
    #[serde(default)]
    pub slaves_number: Option<i32>,
    /// Units: `SlaveRegenRate`, ticks (S-0402 claim 14); absent: 0.
    #[serde(default)]
    pub slave_regen_rate: Option<i32>,
    /// Units: `SlaveReloadRate`, ticks (S-0402 claim 14); absent: 0.
    #[serde(default)]
    pub slave_reload_rate: Option<i32>,
    /// Infantry: `HarvestRate`, a slave's interval between bales *h*, ticks
    /// (S-0402 claim 14); absent: 1.
    #[serde(default)]
    pub harvest_rate: Option<i32>,
    /// Vehicles: `PoweredUnit` (S-0405 claim 13); absent: no.
    #[serde(default)]
    pub powered_unit: Option<bool>,
    /// Units: `ConsideredAircraft`, the counts-as-aircraft flag of the
    /// sidebar order (S-0004 claim 11 rule 3); absent: no.
    #[serde(default)]
    pub considered_aircraft: Option<bool>,
    /// Aircraft: `AirportBound` (S-0407 claim 16); absent: no.
    #[serde(default)]
    pub airport_bound: Option<bool>,
    /// Infantry: `Nominal` (S-0414 Terms, *nominal survivor*); absent: no.
    #[serde(default)]
    pub nominal: Option<bool>,
    /// Buildings: `Power`, base output (≥ 0) or base drain (−value)
    /// (S-0405 claims 1 and 2); absent: 0.
    #[serde(default)]
    pub power: Option<i32>,
    /// Buildings: `ExtraPower` (S-0405 claims 1, 2 and 16); absent: 0.
    #[serde(default)]
    pub extra_power: Option<i32>,
    /// Buildings: `PoweredSpecial`, the special-power flag (S-0405 claim 8);
    /// absent: no.
    #[serde(default)]
    pub powered_special: Option<bool>,
    /// Buildings: `PowersUnit`, the control centre's powered unit type
    /// (S-0405 claim 13); absent: none.
    #[serde(default)]
    pub powers_unit: Option<String>,
    /// Buildings: `Sight`, the sight field *S* in cells (S-0409 claims 4
    /// and 8; infantry, vehicles and aircraft carry it in their own
    /// section); absent: 0.
    #[serde(default)]
    pub sight: Option<i32>,
    /// Buildings: `Radar` (S-0410 claim 1, S-0409 claim 15); absent: no.
    #[serde(default)]
    pub radar: Option<bool>,
    /// Buildings: `SpySat` (S-0409 claim 15); absent: no.
    #[serde(default)]
    pub spy_sat: Option<bool>,
    /// Buildings: `Refinery` (S-0402 Import mapping); absent: no.
    #[serde(default)]
    pub refinery: Option<bool>,
    /// Buildings: `OrePurifier` (S-0402 Terms, claim 12); absent: no.
    #[serde(default)]
    pub ore_purifier: Option<bool>,
    /// Buildings: `DockUnload`, a dock-unload refinery (S-0402 Terms, claim 10); absent: no.
    #[serde(default)]
    pub dock_unload: Option<bool>,
    /// Buildings: `Weeder`, the weed-flag building of S-0402 claim 13's
    /// idle step-off; absent: no.
    #[serde(default)]
    pub weeder: Option<bool>,
    /// Buildings: `ExitCoord`, the exit displacement in leptons (S-0404
    /// Terms, claim 13); absent: (0, 0, 0).
    // spec: S-0404 Import mapping (ExitCoord: an integer triple)
    #[serde(default)]
    pub exit_coord: Option<[i32; 3]>,
    /// Buildings: `GDIBarracks`, the first barracks door kind (S-0404 claim
    /// 13); absent: no.
    #[serde(default)]
    pub door_first: Option<bool>,
    /// Buildings: `NODBarracks`, the second barracks door kind (S-0404
    /// claim 13); absent: no.
    #[serde(default)]
    pub door_second: Option<bool>,
    /// Buildings: `YuriBarracks`, the third barracks door kind (S-0404
    /// claim 13); absent: no.
    #[serde(default)]
    pub door_third: Option<bool>,
    /// Buildings: `Cloning`, a cloning building (S-0404 Terms, claim 14);
    /// absent: no.
    #[serde(default)]
    pub cloning: Option<bool>,
    /// Buildings: `FactoryPlant`, grants cost bonuses (S-0404 claim 2);
    /// absent: no.
    #[serde(default)]
    pub factory_plant: Option<bool>,
    /// Buildings: `InfantryCostBonus`, the cost bonus *β* for infantry, a single (S-0404 claim 2); absent: 1.0.
    #[serde(default)]
    pub infantry_cost_bonus: Option<Decimal>,
    /// Buildings: `UnitsCostBonus`, the cost bonus *β* for vehicles, a single (S-0404 claim 2); absent: 1.0.
    #[serde(default)]
    pub units_cost_bonus: Option<Decimal>,
    /// Buildings: `AircraftCostBonus`, the cost bonus *β* for aircraft, a single (S-0404 claim 2); absent: 1.0.
    #[serde(default)]
    pub aircraft_cost_bonus: Option<Decimal>,
    /// Buildings: `BuildingsCostBonus`, the cost bonus *β* for ordinary buildings, a single (S-0404 claim 2); absent: 1.0.
    #[serde(default)]
    pub buildings_cost_bonus: Option<Decimal>,
    /// Buildings: `DefensesCostBonus`, the cost bonus *β* for defences, a single (S-0404 claim 2); absent: 1.0.
    #[serde(default)]
    pub defenses_cost_bonus: Option<Decimal>,
    /// Buildings: `FreeUnit` (S-0404 claim 2, S-0408 claim 2); absent: none.
    #[serde(default)]
    pub free_unit: Option<String>,
    /// Buildings: `Unsellable` (S-0408 claims 3 and 12); absent: no.
    #[serde(default)]
    pub unsellable: Option<bool>,
    /// Buildings: `ClickRepairable` (S-0408 claim 14); absent: yes.
    #[serde(default)]
    pub click_repairable: Option<bool>,
    /// Buildings: `Adjacent`, adjacency *A* in cells (S-0406 claim 4);
    /// absent: 3.
    #[serde(default)]
    pub adjacent: Option<i32>,
    /// Buildings: `BaseNormal` (S-0406 claim 4); absent: yes.
    #[serde(default)]
    pub base_normal: Option<bool>,
    /// Buildings: `EligibileForAllyBuilding` (S-0406 claim 4); absent: no.
    #[serde(default)]
    pub eligible_for_ally_building: Option<bool>,
    /// Buildings: `PlaceAnywhere` (S-0406 claim 3); absent: no.
    #[serde(default)]
    pub place_anywhere: Option<bool>,
    /// Buildings: `WaterBound` (S-0406 claim 3 d); absent: no, unless the
    /// locomotion-class key names *float*.
    #[serde(default)]
    pub water_bound: Option<bool>,
    /// Buildings: `Wall`, wall type (S-0406 claim 10, S-0403 claim 7);
    /// absent: no.
    #[serde(default)]
    pub wall: Option<bool>,
    /// Buildings: `ToOverlay`, the wall overlay type (S-0406 claims 8 to
    /// 11); absent: none.
    #[serde(default)]
    pub to_overlay: Option<String>,
    /// Buildings: `LaserFencePost` (S-0406 claims 3 b and 9); absent: no.
    #[serde(default)]
    pub laser_fence_post: Option<bool>,
    /// Buildings: `GuardRange`, the wall fill length *G* as the decimal text
    /// of cells (S-0406 claim 11; leptons as S-0215 Import mapping states);
    /// absent: 0.
    #[serde(default)]
    pub guard_range: Option<Decimal>,
    /// Buildings: `PowersUpBuilding`, the target type of an upgrade (S-0406
    /// claim 12); absent: none.
    #[serde(default)]
    pub powers_up_building: Option<String>,
    /// Buildings: `PowersUpToLevel` (S-0406 claim 12); absent: −1.
    #[serde(default)]
    pub powers_up_to_level: Option<i32>,
    /// Buildings: `Upgrades`, the upgrade count (S-0406 claim 12); absent: 0.
    #[serde(default)]
    pub upgrades: Option<i32>,
    /// Buildings: `ToTile`, a tile name as written (S-0406 claims 3 and 5);
    /// absent: none.
    #[serde(default)]
    pub to_tile: Option<String>,
    /// Buildings: `AIBasePlanningSide` (S-0407 claim 6); absent: −1.
    #[serde(default)]
    pub ai_base_planning_side: Option<i32>,
    /// Buildings: `SuperWeapon`, the first superweapon (S-0407 claim 7,
    /// S-0412 claim 9); absent: none.
    #[serde(default)]
    pub super_weapon: Option<String>,
    /// Buildings: `SuperWeapon2`, the second superweapon; absent: none.
    #[serde(default)]
    pub super_weapon2: Option<String>,
    /// Buildings: `SecretLab` (S-0407 claims 4 and 14); absent: no.
    #[serde(default)]
    pub secret_lab: Option<bool>,
    /// Buildings: `SecretInfantry`, a lab's fixed secret type (S-0407 claim
    /// 14); absent: none.
    #[serde(default)]
    pub secret_infantry: Option<String>,
    /// Buildings: `SecretUnit`; absent: none.
    #[serde(default)]
    pub secret_unit: Option<String>,
    /// Buildings: `SecretBuilding`; absent: none.
    #[serde(default)]
    pub secret_building: Option<String>,
    /// Buildings: the rules `NumberOfDocks`, docks the building adds to the
    /// house total (S-0407 claim 16); absent: 0.
    #[serde(default)]
    pub number_of_docks: Option<i32>,
    /// Buildings: the art `NumberOfDocks`, dock slots (S-0402 Terms, claim
    /// 10; values below 1 give 1); absent: 1.
    #[serde(default)]
    pub art_number_of_docks: Option<i32>,
    /// Buildings: the art `QueueingCell`, the queueing shift in cells
    /// (S-0402 Terms, claim 9); absent: 0, 0.
    // nocite: a cell pair (x, y), our format's shape
    #[serde(default)]
    pub queueing_cell: Option<[i32; 2]>,
    /// Buildings: the art `PreProductionAnim`, an animation type name as written (S-0402 Terms, claim 11); absent: none.
    #[serde(default)]
    pub pre_production_anim: Option<String>,
    /// Buildings: the art `PreProductionAnimDamaged`, an animation type name as written (S-0402 Terms, claim 11); absent: none.
    #[serde(default)]
    pub pre_production_anim_damaged: Option<String>,
    /// Buildings: the art `SpecialAnim`, an animation type name as written (S-0402 Terms, claim 11); absent: none.
    #[serde(default)]
    pub special_anim: Option<String>,
    /// Buildings: the art `SpecialAnimDamaged`, an animation type name as written (S-0402 Terms, claim 11); absent: none.
    #[serde(default)]
    pub special_anim_damaged: Option<String>,
    /// Buildings: the art `ProductionAnim`, an animation type name as written (S-0402 Terms, claim 11); absent: none.
    #[serde(default)]
    pub production_anim: Option<String>,
    /// Buildings: the art `ProductionAnimDamaged`, an animation type name as written (S-0402 Terms, claim 11); absent: none.
    #[serde(default)]
    pub production_anim_damaged: Option<String>,
    /// Buildings: whether the type has a build-up sprite, from the art
    /// `Buildup` name (S-0108 claims 1 and 5, Import mapping; S-0408 claims
    /// 3 and 4); absent: no build-up sprite.
    #[serde(default)]
    pub buildup: Option<bool>,
    /// Buildings: the image-frame count *N_b* of the build-up sprite (S-0108
    /// claim 5: *n* = ⌊*N_b* / 2⌋ build-up frames), counted by the importer
    /// from the sprite file; absent: unknown, which the simulation reads as
    /// *N_b* = 0 (S-0406 claim 18 then applies).
    #[serde(default)]
    pub buildup_frames: Option<i32>,
    /// Buildings: the art `GateStages`, the gate-stage count *G* (S-0108
    /// claim 5 and Import mapping: a gate's build-up has *n* = *G* + 1
    /// frames; S-0406 claim 15); absent: 9.
    #[serde(default)]
    pub gate_stages: Option<i32>,
    /// Buildings: `DeployFacing`, the deploy facing *D* in 1/256 turn
    /// (S-0415 claims 5 and 10); absent: 128.
    #[serde(default)]
    pub deploy_facing: Option<i32>,
    /// Buildings: the art `AnimIdle`, the *idle* body range `start, count,
    /// delay` (S-0108 claims 2 and 6, Import mapping; S-0215 claim 11's
    /// body-range ready mark); absent: 0, 1, 0.
    #[serde(default)]
    pub anim_idle: Option<[i32; 3]>,
    /// Buildings: the art `Normalized`, the game-speed normalisation mark
    /// of the body ranges (S-0108 claim 6, Import mapping); absent: false.
    #[serde(default)]
    pub normalized: Option<bool>,
}

/// Economy fields of a country (written into `econ`).
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CountryEcon {
    /// `CostInfantryMult`, the cost factor *a* for infantry, a single (S-0404 claim 2); absent: 1.0.
    #[serde(default)]
    pub cost_infantry_mult: Option<Decimal>,
    /// `CostUnitsMult`, the cost factor *a* for vehicles, a single (S-0404 claim 2); absent: 1.0.
    #[serde(default)]
    pub cost_units_mult: Option<Decimal>,
    /// `CostAircraftMult`, the cost factor *a* for aircraft, a single (S-0404 claim 2); absent: 1.0.
    #[serde(default)]
    pub cost_aircraft_mult: Option<Decimal>,
    /// `CostBuildingsMult`, the cost factor *a* for ordinary buildings, a single (S-0404 claim 2); absent: 1.0.
    #[serde(default)]
    pub cost_buildings_mult: Option<Decimal>,
    /// `CostDefensesMult`, the cost factor *a* for defences, a single (S-0404 claim 2); absent: 1.0.
    #[serde(default)]
    pub cost_defenses_mult: Option<Decimal>,
    /// `BuildTimeInfantryMult`, the build-time factor *c* for infantry, a single (S-0403 claim 3); absent: 1.0.
    #[serde(default)]
    pub build_time_infantry_mult: Option<Decimal>,
    /// `BuildTimeUnitsMult`, the build-time factor *c* for vehicles, a single (S-0403 claim 3); absent: 1.0.
    #[serde(default)]
    pub build_time_units_mult: Option<Decimal>,
    /// `BuildTimeAircraftMult`, the build-time factor *c* for aircraft, a single (S-0403 claim 3); absent: 1.0.
    #[serde(default)]
    pub build_time_aircraft_mult: Option<Decimal>,
    /// `BuildTimeBuildingsMult`, the build-time factor *c* for ordinary buildings, a single (S-0403 claim 3); absent: 1.0.
    #[serde(default)]
    pub build_time_buildings_mult: Option<Decimal>,
    /// `BuildTimeDefensesMult`, the build-time factor *c* for defences, a single (S-0403 claim 3); absent: 1.0.
    #[serde(default)]
    pub build_time_defenses_mult: Option<Decimal>,
    /// `IncomeMult`, the income factor *M*, a single (S-0402 claim 12);
    /// absent: 1.0.
    #[serde(default)]
    pub income_mult: Option<Decimal>,
    /// `SpeedAircraftMult`, a single (S-0413 claim 3); absent: 1.0.
    #[serde(default)]
    pub speed_aircraft_mult: Option<Decimal>,
    /// `Side`, the side name as written (S-0413, S-0004 claim 9); absent: no
    /// side.
    #[serde(default)]
    pub side: Option<String>,
    /// `Multiplay`, the multiplayer listing flag (S-0413 claim 4); absent:
    /// off.
    #[serde(default)]
    pub multiplay: Option<bool>,
    /// `Color`, the colour definition name as written (S-0413 claim 5);
    /// absent: the definition at position 0.
    #[serde(default)]
    pub color: Option<String>,
    /// `ParentCountry`, no effect on play (S-0413 claim 4); absent: the
    /// country's own name.
    #[serde(default)]
    pub parent_country: Option<String>,
    /// `Prefix`, no effect on play (S-0413 claim 4); absent: the character A.
    #[serde(default)]
    pub prefix: Option<String>,
    /// `Suffix`, no effect on play (S-0413 claim 4); absent: empty.
    #[serde(default)]
    pub suffix: Option<String>,
    /// `SmartAI`, the smart-computer flag, no effect on play (S-0413 claim
    /// 4); absent: off.
    #[serde(default)]
    pub smart_ai: Option<bool>,
}

/// A tiberium (S-0401 Import mapping), keyed by its section name; the
/// registry `tiberiums` gives the order.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TiberiumType {
    /// `Value`, the value per bale *V* (S-0401 claims 2, 3); absent: 0.
    #[serde(default)]
    pub value: Option<i32>,
    /// `Growth`, the growth field *G* (S-0401 F3); absent: 0.
    #[serde(default)]
    pub growth: Option<i32>,
    /// `GrowthPercentage`, *g* (S-0401 F5), a single widened to double;
    /// absent: the double nearest 0.1.
    #[serde(default)]
    pub growth_percentage: Option<Decimal>,
    /// `Spread`, the spread field *S* (S-0401 F4); absent: 0.
    #[serde(default)]
    pub spread: Option<i32>,
    /// `SpreadPercentage`, *s* (S-0401 F6), a single widened to double;
    /// absent: the double nearest 0.1.
    #[serde(default)]
    pub spread_percentage: Option<Decimal>,
    /// `Image`, the image number (S-0112 claim 10).
    #[serde(default)]
    pub image: Option<i32>,
    /// The image run (*P*, *F*, *S*) that the image numbers given by every
    /// rules layer leave (S-0112 claim 10, Constants), worked out by the
    /// importer from the key's history (the value 2 keeps an earlier *S*);
    /// absent: worked out from `image` alone, and no run when that is
    /// absent too (S-0112 claim 12).
    // nocite: a triple (P, F, S), our format's shape
    #[serde(default)]
    pub image_run: Option<[i32; 3]>,
}

/// A superweapon type (S-0407 and S-0412 Import mapping).
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SuperweaponType {
    /// `DisableableFromShell` (S-0412 claim 9 b, S-0407 claim 7); absent: no.
    #[serde(default)]
    pub disableable_from_shell: Option<bool>,
    /// `Type`, the kind word as written (S-0407 claim 12 j); absent: none.
    #[serde(default)]
    pub kind: Option<String>,
}

/// Game-wide economy values (written into `general.economy`), from the
/// sections `[General]`, `[AI]`, `[AudioVisual]`, `[CombatDamage]`,
/// `[CrateRules]` and `[MultiplayerDialogSettings]` of the rules.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EconGeneral {
    /// `[General] HarvesterLoadRate`, *L* in ticks (S-0402 claim 5); absent: 2.
    #[serde(default)]
    pub harvester_load_rate: Option<i32>,
    /// `[General] HarvesterDumpRate`, *D* in game minutes (S-0402 claim 11);
    /// absent: the double nearest 0.016.
    #[serde(default)]
    pub harvester_dump_rate: Option<Decimal>,
    /// `[General] TiberiumLongScan`, cells as decimal text (lepton form,
    /// S-0402 Terms); absent: 8192 leptons.
    #[serde(default)]
    pub tiberium_long_scan: Option<Decimal>,
    /// `[General] TiberiumShortScan`, cells as decimal text (lepton form);
    /// absent: 1536 leptons.
    #[serde(default)]
    pub tiberium_short_scan: Option<Decimal>,
    /// `[General] HarvesterTooFarDistance`, cells (S-0402 claim 9); absent: 5.
    #[serde(default)]
    pub harvester_too_far_distance: Option<i32>,
    /// `[General] ChronoHarvTooFarDistance`, cells (S-0402 claims 9, 17);
    /// absent: 50.
    #[serde(default)]
    pub chrono_harv_too_far_distance: Option<i32>,
    /// `[General] SlaveMinerShortScan`, lepton form (S-0402 claim 14);
    /// absent: 1280 leptons.
    #[serde(default)]
    pub slave_miner_short_scan: Option<Decimal>,
    /// `[General] SlaveMinerSlaveScan`, lepton form; absent: 4096 leptons.
    #[serde(default)]
    pub slave_miner_slave_scan: Option<Decimal>,
    /// `[General] SlaveMinerLongScan`, lepton form; absent: 20480 leptons.
    #[serde(default)]
    pub slave_miner_long_scan: Option<Decimal>,
    /// `[General] SlaveMinerScanCorrection`, lepton form; absent: 768 leptons.
    #[serde(default)]
    pub slave_miner_scan_correction: Option<Decimal>,
    /// `[General] SlaveMinerKickFrameDelay`, *k* in ticks (S-0402 claim 14,
    /// F5); absent: 2147483647.
    #[serde(default)]
    pub slave_miner_kick_frame_delay: Option<i32>,
    /// `[General] PurifierBonus`, *b*, a single (S-0402 claim 12); absent:
    /// the single nearest 0.25.
    #[serde(default)]
    pub purifier_bonus: Option<Decimal>,
    /// `[General] AIVirtualPurifiers`, per difficulty (S-0402 claim 12, the
    /// original only); absent: empty.
    #[serde(default)]
    pub ai_virtual_purifiers: Option<Vec<i32>>,
    /// `[General] BuildSpeed`, *S* in game minutes per 1000 credits, a double
    /// (S-0403 claim 2); absent: 1.0.
    #[serde(default)]
    pub build_speed: Option<Decimal>,
    /// `[General] LowPowerPenaltyModifier`, *q*, a single (S-0403 claim 5);
    /// absent: 1.0.
    #[serde(default)]
    pub low_power_penalty_modifier: Option<Decimal>,
    /// `[General] MinLowPowerProductionSpeed`, a single (S-0403 claim 5);
    /// absent: 0.5.
    #[serde(default)]
    pub min_low_power_production_speed: Option<Decimal>,
    /// `[General] MaxLowPowerProductionSpeed`, a single (S-0403 claim 5);
    /// absent: the single nearest 0.9.
    #[serde(default)]
    pub max_low_power_production_speed: Option<Decimal>,
    /// `[General] MultipleFactory`, *g*, a single (S-0403 claim 6); absent: 1.0.
    #[serde(default)]
    pub multiple_factory: Option<Decimal>,
    /// `[General] WallBuildSpeedCoefficient`, *w*, a double (S-0403 claim 7);
    /// absent: 0.5.
    #[serde(default)]
    pub wall_build_speed_coefficient: Option<Decimal>,
    /// `[General] MaximumQueuedObjects` (S-0404 claim 3); absent: 5.
    #[serde(default)]
    pub maximum_queued_objects: Option<i32>,
    /// `[General] PadAircraft`, aircraft type keys (S-0404 claim 2, S-0408
    /// claim 2, S-0407 claim 16); absent: empty.
    #[serde(default)]
    pub pad_aircraft: Option<Vec<String>>,
    /// `[General] SeparateAircraft` (S-0404 claim 2, S-0408 claim 2); absent:
    /// no.
    #[serde(default)]
    pub separate_aircraft: Option<bool>,
    /// `[General] SpyPowerBlackout`, *d* in ticks (S-0405 claim 6); absent: 0.
    #[serde(default)]
    pub spy_power_blackout: Option<i32>,
    /// `[General] BuildupTime`, the build-up time *B* in game minutes, a
    /// double (S-0108 claim 5, F1); absent: the double nearest 0.05.
    #[serde(default)]
    pub buildup_time: Option<Decimal>,
    /// `[General] RefundPercent`, *r*, a double (S-0408 claim 1); absent: 0.5.
    #[serde(default)]
    pub refund_percent: Option<Decimal>,
    /// `[General] PrerequisitePower`, the members of the generic group POWER, building type keys (S-0407 claim 8 b); absent: empty.
    #[serde(default)]
    pub prerequisite_power: Option<Vec<String>>,
    /// `[General] PrerequisiteFactory`, the members of the generic group FACTORY, building type keys (S-0407 claim 8 b); absent: empty.
    #[serde(default)]
    pub prerequisite_factory: Option<Vec<String>>,
    /// `[General] PrerequisiteBarracks`, the members of the generic group BARRACKS, building type keys (S-0407 claim 8 b); absent: empty.
    #[serde(default)]
    pub prerequisite_barracks: Option<Vec<String>>,
    /// `[General] PrerequisiteRadar`, the members of the generic group RADAR, building type keys (S-0407 claim 8 b); absent: empty.
    #[serde(default)]
    pub prerequisite_radar: Option<Vec<String>>,
    /// `[General] PrerequisiteTech`, the members of the generic group TECH, building type keys (S-0407 claim 8 b); absent: empty.
    #[serde(default)]
    pub prerequisite_tech: Option<Vec<String>>,
    /// `[General] PrerequisiteProc`, the members of the generic group PROC, building type keys (S-0407 claim 8 b); absent: empty.
    #[serde(default)]
    pub prerequisite_proc: Option<Vec<String>>,
    /// `[General] PrerequisiteProcAlternate`, a vehicle type key (S-0407
    /// claim 8 b); absent: none.
    #[serde(default)]
    pub prerequisite_proc_alternate: Option<String>,
    /// `[General] SecretInfantry`, a secret list, type keys (S-0407 claim 14); absent: empty.
    #[serde(default)]
    pub secret_infantry: Option<Vec<String>>,
    /// `[General] SecretUnits`, a secret list, type keys (S-0407 claim 14); absent: empty.
    #[serde(default)]
    pub secret_units: Option<Vec<String>>,
    /// `[General] SecretBuildings`, a secret list, type keys (S-0407 claim 14); absent: empty.
    #[serde(default)]
    pub secret_buildings: Option<Vec<String>>,
    /// `[General] LeptonsPerSightIncrease`, *P* (S-0409 F1); absent: 50.
    #[serde(default)]
    pub leptons_per_sight_increase: Option<i32>,
    /// `[General] RevealByHeight` (S-0409 claim 7); absent: yes.
    #[serde(default)]
    pub reveal_by_height: Option<bool>,
    /// `[General] FlashFrameTime`, *F* in ticks (S-0410 claim 10); absent: 7.
    #[serde(default)]
    pub flash_frame_time: Option<i32>,
    /// `[General] RadarCombatFlashTime`, ticks (S-0410 claim 10); absent: 21.
    #[serde(default)]
    pub radar_combat_flash_time: Option<i32>,
    /// `[General] MultiplayerAICM`, the original's extra-credit percentages
    /// by difficulty index (S-0412 claim 3); absent: no entries.
    #[serde(default)]
    pub multiplayer_aicm: Option<Vec<i32>>,
    /// `[General] AIHateDelays`, the original's computer-player timer list
    /// by difficulty index (S-0412 claim 3, the original only; no effect in
    /// our engine); absent: no entries.
    #[serde(default)]
    pub ai_hate_delays: Option<Vec<i32>>,
    /// `[AI] BuildTech`, the tech-lab list, building type keys (S-0407
    /// claims 6 and 7); absent: empty.
    #[serde(default)]
    pub build_tech: Option<Vec<String>>,
    /// `[AI] BuildRefinery`, the refinery list, building type keys (S-0414
    /// claim 3); absent: empty.
    #[serde(default)]
    pub build_refinery: Option<Vec<String>>,
    /// `[AI] CompEasyBonus` (S-0412 claim 4, the original only); absent: yes.
    #[serde(default)]
    pub comp_easy_bonus: Option<bool>,
    /// `[AudioVisual] AllyReveal` (S-0409 claim 2); absent: yes.
    #[serde(default)]
    pub ally_reveal: Option<bool>,
    /// `[AudioVisual] ShroudGrow` (S-0409 claim 18); absent: no.
    #[serde(default)]
    pub shroud_grow: Option<bool>,
    /// `[AudioVisual] ShroudRate`, *g* in game minutes (S-0409 F5); absent: 4.0.
    #[serde(default)]
    pub shroud_rate: Option<Decimal>,
    /// `[AudioVisual] FogRate`, *f* in game minutes (S-0409 F5); absent: 0.05.
    #[serde(default)]
    pub fog_rate: Option<Decimal>,
    /// `[AudioVisual] SavourDelay`, *d* in game minutes, a double (S-0414
    /// claim 8); absent: 0.03.
    #[serde(default)]
    pub savour_delay: Option<Decimal>,
    /// `[AudioVisual] PoseDir`, the pose direction of an aircraft released
    /// onto a dock in 1/256 turns (S-0404 claim 13); absent: 0.
    #[serde(default)]
    pub pose_dir: Option<i32>,
    /// `[CombatDamage] C4Warhead`, a warhead key (S-0414 claim 5); absent:
    /// none.
    #[serde(default)]
    pub c4_warhead: Option<String>,
    /// `[CrateRules] CrateMinimum` (S-0412 claim 10); absent: 1.
    #[serde(default)]
    pub crate_minimum: Option<i32>,
    /// `[CrateRules] CrateMaximum` (S-0412 claim 10); absent: 255.
    #[serde(default)]
    pub crate_maximum: Option<i32>,
    /// `[MultiplayerDialogSettings] Money`, the default credits setting
    /// offered (S-0412 Import mapping); absent: 10000.
    #[serde(default)]
    pub money: Option<i32>,
    /// `[MultiplayerDialogSettings] MinMoney` (interface, S-0412 Import mapping); absent: 2500.
    #[serde(default)]
    pub min_money: Option<i32>,
    /// `[MultiplayerDialogSettings] MaxMoney` (interface, S-0412 Import mapping); absent: 10000.
    #[serde(default)]
    pub max_money: Option<i32>,
    /// `[MultiplayerDialogSettings] MoneyIncrement` (interface, S-0412 Import mapping); absent: 100.
    #[serde(default)]
    pub money_increment: Option<i32>,
    /// `[MultiplayerDialogSettings] UnitCount` (S-0412 Import mapping); absent: 10.
    #[serde(default)]
    pub unit_count: Option<i32>,
    /// `[MultiplayerDialogSettings] MinUnitCount` (S-0412 Import mapping); absent: 1.
    #[serde(default)]
    pub min_unit_count: Option<i32>,
    /// `[MultiplayerDialogSettings] MaxUnitCount` (S-0412 Import mapping); absent: 20.
    #[serde(default)]
    pub max_unit_count: Option<i32>,
    /// `[MultiplayerDialogSettings] Bases`, the base rules' value only
    /// (S-0412 claim 1); absent: yes.
    #[serde(default)]
    pub bases: Option<bool>,
    /// `[MultiplayerDialogSettings] ShortGame`, a default offered (S-0412 Import mapping); absent: no.
    #[serde(default)]
    pub short_game: Option<bool>,
    /// `[MultiplayerDialogSettings] Crates`, a default offered (S-0412 Import mapping); absent: yes.
    #[serde(default)]
    pub crates: Option<bool>,
    /// `[MultiplayerDialogSettings] MCVRedeploys`, a default offered (S-0412 Import mapping); absent: yes.
    #[serde(default)]
    pub mcv_redeploys: Option<bool>,
    /// `[MultiplayerDialogSettings] SuperWeaponsAllowed`, a default offered (S-0412 Import mapping); absent: yes.
    #[serde(default)]
    pub super_weapons_allowed: Option<bool>,
    /// `[MultiplayerDialogSettings] BuildOffAlly`, a default offered (S-0412 Import mapping); absent: yes.
    #[serde(default)]
    pub build_off_ally: Option<bool>,
    /// `[MultiplayerDialogSettings] Shroud`, the lobby shroud setting (S-0409
    /// claim 3); absent: yes.
    #[serde(default)]
    pub shroud: Option<bool>,
    /// `[MultiplayerDialogSettings] AlliesAllowed` (S-0414 claim 14); absent:
    /// no.
    #[serde(default)]
    pub allies_allowed: Option<bool>,
    /// `[MultiplayerDialogSettings] AllyChangeAllowed` (S-0414 claim 14);
    /// absent: yes.
    #[serde(default)]
    pub ally_change_allowed: Option<bool>,
}
