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
    /// Units and buildings: `Enslaves`, the slave type (S-0402 claims 14
    /// and 19); absent: none.
    #[serde(default)]
    pub enslaves: Option<String>,
    /// Units and buildings: `SlavesNumber` (S-0402 claims 14 and 19); absent: 0.
    #[serde(default)]
    pub slaves_number: Option<i32>,
    /// Units and buildings: `SlaveRegenRate`, ticks (S-0402 claim 14); absent: 0.
    #[serde(default)]
    pub slave_regen_rate: Option<i32>,
    /// Units and buildings: `SlaveReloadRate`, ticks (S-0402 claim 14); absent: 0.
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
    /// sidebar order (S-0004 claim 11 rule 3) and of tracking from a
    /// placement in the air (S-0502 claim 29, S-0501 claim 47); absent: no,
    /// yes for an aircraft type (S-0501 Import mapping).
    #[serde(default)]
    pub considered_aircraft: Option<bool>,
    /// Aircraft: `AirportBound` (S-0407 claim 16); absent: no.
    #[serde(default)]
    pub airport_bound: Option<bool>,
    /// Aircraft: `Landable`, the landable flag (S-0501 Terms and claim 6);
    /// absent: no.
    #[serde(default)]
    pub landable: Option<bool>,
    /// Aircraft: `Fighter`, the fighter flag (S-0501 claims 11 and 19);
    /// absent: no.
    #[serde(default)]
    pub fighter: Option<bool>,
    /// Aircraft: `FlyBy` (S-0501 claims 4, 5 and 10); absent: no.
    #[serde(default)]
    pub fly_by: Option<bool>,
    /// Aircraft: `FlyBack` (S-0501 claim 10); absent: no.
    #[serde(default)]
    pub fly_back: Option<bool>,
    /// Aircraft: `FlightLevel`, the cruise height in leptons (S-0501
    /// Terms); absent: unset, the game-wide value.
    #[serde(default)]
    pub flight_level: Option<i32>,
    /// Buildings: `UnitReload`, the reload-pad flag (S-0501 Terms and
    /// claim 26); absent: no.
    #[serde(default)]
    pub unit_reload: Option<bool>,
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
    /// Buildings: `NukeSilo`, the missile-silo flag (S-0513 claim 7);
    /// absent: no.
    #[serde(default)]
    pub nuke_silo: Option<bool>,
    /// Buildings: `PsychicDetectionRadius`, cells (S-0513 claim 11); absent:
    /// 0.
    #[serde(default)]
    pub psychic_detection_radius: Option<i32>,
    /// Buildings: `HasRadialIndicator`, shows a range ring when selected
    /// (S-0610 claim 12); absent: no.
    #[serde(default)]
    pub has_radial_indicator: Option<bool>,
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
    /// Buildings: the art `DockingOffset0`, `DockingOffset1`, … read below
    /// the rules `NumberOfDocks` count, displacements in leptons (x, y,
    /// height) from the foundation centre (S-0404 claim 15); a place
    /// without a key, or past the list, is (0, 0, 0).
    // nocite: a list of integer triples, our format's shape
    #[serde(default)]
    pub docking_offsets: Option<Vec<[i32; 3]>>,
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
    /// Buildings: the art `SpecialAnimTwo`, an animation type name as written (S-0606 Terms, *special-2*, S-0108); absent: none.
    #[serde(default)]
    pub special_anim_two: Option<String>,
    /// Buildings: the art `SpecialAnimTwoDamaged`, an animation type name as written (S-0606 Terms, the damaged variant); absent: none.
    #[serde(default)]
    pub special_anim_two_damaged: Option<String>,
    /// Buildings: the art `SpecialAnimThree`, an animation type name as written (S-0606 Terms, *special-3*, S-0108); absent: none.
    #[serde(default)]
    pub special_anim_three: Option<String>,
    /// Buildings: the art `SpecialAnimThreeDamaged`, an animation type name as written (S-0606 Terms, the damaged variant); absent: none.
    #[serde(default)]
    pub special_anim_three_damaged: Option<String>,
    /// Buildings: the art `SpecialAnimFour`, an animation type name as written (S-0606 Terms, *special-4*, S-0108); absent: none.
    #[serde(default)]
    pub special_anim_four: Option<String>,
    /// Buildings: the art `SpecialAnimFourDamaged`, an animation type name as written (S-0606 Terms, the damaged variant); absent: none.
    #[serde(default)]
    pub special_anim_four_damaged: Option<String>,
    /// Buildings: the art `ProductionAnim`, an animation type name as written (S-0402 Terms, claim 11); absent: none.
    #[serde(default)]
    pub production_anim: Option<String>,
    /// Buildings: the art `ProductionAnimDamaged`, an animation type name as written (S-0402 Terms, claim 11); absent: none.
    #[serde(default)]
    pub production_anim_damaged: Option<String>,
    /// Buildings: the slot names of the *pre-production*, *production*,
    /// *special-1*, *special-2* and *special-3* slots as S-0108's Import
    /// mapping reads them (each cut to 15 bytes, an empty value keeping the
    /// earlier name, an empty damaged name taking the normal name), keyed
    /// by the slot's art base key, each `[normal, damaged]`; a slot with
    /// both names empty is left out. The simulation's slot model reads them
    /// (S-0108 claim 20, S-0402 claim 11); absent: none.
    // nocite: a map of name pairs, our format's shape
    #[serde(default)]
    pub slot_names: Option<std::collections::BTreeMap<String, [String; 2]>>,
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
    pub anim_idle: Option<[i32; 3]>, // spec: S-0108 Import mapping (AnimIdle: three values, start, count, delay)
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

/// The superweapon kind words in kind-number order 0 to 11 (S-0512 Import
/// mapping, claim 24): literal lower-case words, matched as a complete word
/// without regard to case. Hyphens, underscores and internal spaces are not
/// ignored.
// spec: S-0512 claim 24 (the twelve kind words of the Import mapping table, in kind-number order)
pub const SUPERWEAPON_KIND_WORDS: [&str; 12] = [
    "multimissile",
    "ironcurtain",
    "lightningstorm",
    "chronosphere",
    "chronowarp",
    "paradrop",
    "amerparadrop",
    "psychicdominator",
    "spyplane",
    "geneticconverter",
    "forceshield",
    "psychicreveal",
];

/// The accepted aim-action words (S-0512 Import mapping, claim 23), in the
/// table's reading order; `none` is no action. Matched as a complete word
/// without regard to case.
// spec: S-0512 claim 23 and Import mapping (the complete action-word vocabulary)
pub const SUPERWEAPON_ACTION_WORDS: &[&str] = &[
    "none",
    "move",
    "nomove",
    "enter",
    "self",
    "attack",
    "harvest",
    "select",
    "toggleselect",
    "capture",
    "eaten",
    "repair",
    "sell",
    "sellunit",
    "nosell",
    "norepair",
    "sabotage",
    "tote",
    "dontuse2",
    "dontuse3",
    "nuke",
    "dontuse4",
    "dontuse5",
    "dontuse6",
    "dontuse7",
    "dontuse8",
    "guardarea",
    "heal",
    "damage",
    "grepair",
    "nodeploy",
    "noenter",
    "nogrepair",
    "togglepower",
    "notogglepower",
    "entertunnel",
    "noentertunnel",
    "ironcurtain",
    "lightningstorm",
    "chronosphere",
    "chronowarp",
    "paradrop",
    "placewaypoint",
    "tibsunbug",
    "enterwaypointmode",
    "followwaypoint",
    "selectwaypoint",
    "loopwaypointpath",
    "dragwaypoint",
    "attackwaypoint",
    "enterwaypoint",
    "patrolwaypoint",
    "areaattack",
    "ivanbomb",
    "noivanbomb",
    "detonate",
    "detonateall",
    "disarmbomb",
    "selectnode",
    "attacksupport",
    "placebeacon",
    "selectbeacon",
    "attackmovenav",
    "attackmovetar",
    "demolish",
    "amerparadrop",
    "psychicdominator",
    "spyplane",
    "geneticconverter",
    "forceshield",
    "noforceshield",
    "airstrike",
    "psychicreveal",
];

/// The kind number of a kind word (S-0512 claim 24), or `None` for a word
/// outside the vocabulary.
pub fn superweapon_kind_number(word: &str) -> Option<usize> {
    SUPERWEAPON_KIND_WORDS
        .iter()
        .position(|k| k.eq_ignore_ascii_case(word))
}

/// Whether `word` is in the aim-action vocabulary (S-0512 claim 23); `none`
/// included.
pub fn superweapon_action_known(word: &str) -> bool {
    !word.is_empty()
        && SUPERWEAPON_ACTION_WORDS
            .iter()
            .any(|k| k.eq_ignore_ascii_case(word))
}

/// A superweapon type (S-0407 and S-0412 Import mapping).
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SuperweaponType {
    /// `DisableableFromShell` (S-0412 claim 9 b, S-0407 claim 7); absent: no.
    #[serde(default)]
    pub disableable_from_shell: Option<bool>,
    /// `Type`, the kind word, one of the twelve words of S-0512 claim 24
    /// (S-0407 claim 12 j); absent: none. The importer keeps the previous
    /// layer's word for an unknown one.
    #[serde(default)]
    pub kind: Option<String>,
    /// `RechargeTime`, the charge time in game minutes, single precision
    /// (S-0512 claim 21, F1); absent or 0: the previous value, engine default
    /// 4500 ticks.
    #[serde(default)]
    pub recharge_time: Option<Decimal>,
    /// `IsPowered`, power-dependent (S-0512 Import mapping); absent: yes.
    #[serde(default)]
    pub is_powered: Option<bool>,
    /// `ShowTimer`, the countdown flag (S-0512 claim 18); absent: no.
    #[serde(default)]
    pub show_timer: Option<bool>,
    /// `FlashSidebarTabFrames`, the flash window in ticks, −1 unlimited
    /// (S-0512 claim 17); absent: −1.
    #[serde(default)]
    pub flash_sidebar_tab_frames: Option<i32>,
    /// `PreClick`, the first-stage flag (S-0512 Terms); absent: no.
    #[serde(default)]
    pub pre_click: Option<bool>,
    /// `PostClick`, the second-stage flag (S-0512 Terms); absent: no.
    #[serde(default)]
    pub post_click: Option<bool>,
    /// `PreDependent`, the second stage's link, a kind word of S-0512 claim
    /// 24 (claim 11); absent: none.
    #[serde(default)]
    pub pre_dependent: Option<String>,
    /// `Action`, the aim action word as written (S-0512 claim 23); a word
    /// outside the vocabulary sets no action; absent: none.
    #[serde(default)]
    pub action: Option<String>,
    /// `UseChargeDrain`, the drain-mode flag (S-0512 claim 19); absent: no.
    #[serde(default)]
    pub use_charge_drain: Option<bool>,
    /// `ManualControl`, the manual-charge flag (S-0512 claim 20); absent: no.
    #[serde(default)]
    pub manual_control: Option<bool>,
    /// `AuxBuilding`, a building type key (S-0512 Terms, giving building);
    /// absent: none.
    #[serde(default)]
    pub aux_building: Option<String>,
    /// `UIName`, the display-name label (S-0512 claim 18); absent: none.
    #[serde(default)]
    pub ui_name: Option<String>,
    /// `SidebarImage`, the icon art name (S-0512 Import mapping, S-0007 claim
    /// 31); absent: none.
    #[serde(default)]
    pub sidebar_image: Option<String>,
    /// `WeaponType`, the silo's upward weapon, a weapon key (S-0513 claim 8);
    /// absent: none.
    #[serde(default)]
    pub weapon_type: Option<String>,
    /// `StartSound`, the effect start sound (S-0513 claim 32); absent: none.
    #[serde(default)]
    pub start_sound: Option<String>,
    /// `SpecialSound`, the effect fading sound (S-0513 claim 32); absent:
    /// none.
    #[serde(default)]
    pub special_sound: Option<String>,
}

/// Game-wide superweapon values (written into `general.superweapons`) from
/// `[General]`, `[AudioVisual]`, `[CombatDamage]` and the fixed type names
/// of S-0512 and S-0513 (Import mapping).
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SwGeneral {
    /// `[General] ChargeToDrainRatio` ρ, a double (S-0512 F5); absent: 3.
    #[serde(default)]
    pub charge_to_drain_ratio: Option<Decimal>,
    /// `[General] NukeTakeOff`, the silo take-off animation (S-0513 claim 8);
    /// absent: none.
    #[serde(default)]
    pub nuke_take_off: Option<String>,
    /// `[AudioVisual] DigSound`, the nuclear siren (S-0513 claim 5); absent:
    /// none.
    #[serde(default)]
    pub dig_sound: Option<String>,
    /// `[General] LightningDeferment`, the storm countdown *L* in ticks
    /// (S-0513 claim 14); absent: 250.
    #[serde(default)]
    pub lightning_deferment: Option<i32>,
    /// `[General] LightningStormDuration`, the storm duration *D* in ticks,
    /// −1 endless (S-0513 claim 15); absent: 900.
    #[serde(default)]
    pub lightning_storm_duration: Option<i32>,
    /// `[General] LightningHitDelay`, the hit period in ticks (S-0513 claim
    /// 16); absent: 90.
    #[serde(default)]
    pub lightning_hit_delay: Option<i32>,
    /// `[General] LightningScatterDelay`, the scatter period in ticks
    /// (S-0513 claim 16); absent: 10.
    #[serde(default)]
    pub lightning_scatter_delay: Option<i32>,
    /// `[General] LightningCellSpread` *s*, cells (S-0513 F6); absent: 10.
    #[serde(default)]
    pub lightning_cell_spread: Option<i32>,
    /// `[General] LightningSeparation`, cells (S-0513 F6); absent: 3.
    #[serde(default)]
    pub lightning_separation: Option<i32>,
    /// `[General] LightningDamage`, health points (S-0513 claim 18); absent:
    /// 200.
    #[serde(default)]
    pub lightning_damage: Option<i32>,
    /// `[General] LightningWarhead`, a warhead key (S-0513 claim 18);
    /// absent: none.
    #[serde(default)]
    pub lightning_warhead: Option<String>,
    /// `[General] LightningPrintText`, the storm-message flag (S-0513 claim
    /// 14); absent: yes.
    #[serde(default)]
    pub lightning_print_text: Option<bool>,
    /// `[General] WeatherConClouds`, the cloud animation list (S-0513 claim
    /// 17); absent: empty.
    #[serde(default)]
    pub weather_con_clouds: Option<Vec<String>>,
    /// `[General] WeatherConBolts`, the bolt animation list (S-0513 claims
    /// 17 and 18, F4); absent: empty.
    #[serde(default)]
    pub weather_con_bolts: Option<Vec<String>>,
    /// `[General] WeatherConBoltExplosion`, the storm explosion animation
    /// (S-0513 claim 18); absent: none.
    #[serde(default)]
    pub weather_con_bolt_explosion: Option<String>,
    /// `[AudioVisual] StormSound` (S-0513 claim 15); absent: none.
    #[serde(default)]
    pub storm_sound: Option<String>,
    /// `[AudioVisual] LightningSounds` (S-0513 claim 18); absent: empty.
    #[serde(default)]
    pub lightning_sounds: Option<Vec<String>>,
    /// `[CombatDamage] IronCurtainDuration`, ticks (S-0513 claim 26);
    /// absent: 0.
    #[serde(default)]
    pub iron_curtain_duration: Option<i32>,
    /// `[General] IronCurtainInvokeAnim` (S-0513 claim 22); absent: none.
    #[serde(default)]
    pub iron_curtain_invoke_anim: Option<String>,
    /// `[General] ForceShieldInvokeAnim` (S-0513 claim 31); absent: none.
    #[serde(default)]
    pub force_shield_invoke_anim: Option<String>,
    /// `[General] ForceShieldRadius` *R*, cells (S-0513 F3); absent: 10.
    #[serde(default)]
    pub force_shield_radius: Option<i32>,
    /// `[General] ForceShieldDuration` *D*, ticks (S-0513 claim 31);
    /// absent: 400.
    #[serde(default)]
    pub force_shield_duration: Option<i32>,
    /// `[General] ForceShieldBlackoutDuration`, ticks (S-0513 claim 30);
    /// absent: 800.
    #[serde(default)]
    pub force_shield_blackout_duration: Option<i32>,
    /// `[General] ForceShieldPlayFadeSoundTime` *P*, ticks (S-0513 claim
    /// 32); absent: 50.
    #[serde(default)]
    pub force_shield_play_fade_sound_time: Option<i32>,
    /// `[AudioVisual] IronCurtainColor`, a tint selector (S-0513 Import
    /// mapping); absent: 0.
    #[serde(default)]
    pub iron_curtain_color: Option<i32>,
    /// `[AudioVisual] ForceShieldColor`, a tint selector (S-0513 Import
    /// mapping); absent: 0.
    #[serde(default)]
    pub force_shield_color: Option<i32>,
    /// `[General] ChronoPlacement`, the source marker (S-0513 claim 33);
    /// absent: none.
    #[serde(default)]
    pub chrono_placement: Option<String>,
    /// `[General] ChronoBlast`, the blast at the source (S-0513 claim 34);
    /// absent: none.
    #[serde(default)]
    pub chrono_blast: Option<String>,
    /// `[General] ChronoBlastDest`, the blast at the destination (S-0513
    /// claim 34); absent: none.
    #[serde(default)]
    pub chrono_blast_dest: Option<String>,
    /// `[General] ChronoDelay`, the teleport delay in ticks (S-0513 claim
    /// 40); absent: 60.
    #[serde(default)]
    pub chrono_delay: Option<i32>,
    /// `[General] ChronoReinfDelay`, the reinforcement delay in ticks
    /// (S-0513 claim 40); absent: 60.
    #[serde(default)]
    pub chrono_reinf_delay: Option<i32>,
    /// `[General] AllyParaDropInf`, infantry keys (S-0522 claim 2, S-0513
    /// claim 43); absent: empty.
    #[serde(default)]
    pub ally_para_drop_inf: Option<Vec<String>>,
    /// `[General] AllyParaDropNum`, counts (S-0522 claim 2); absent: empty.
    #[serde(default)]
    pub ally_para_drop_num: Option<Vec<i32>>,
    /// `[General] SovParaDropInf` (S-0522 claim 2); absent: empty.
    #[serde(default)]
    pub sov_para_drop_inf: Option<Vec<String>>,
    /// `[General] SovParaDropNum` (S-0522 claim 2); absent: empty.
    #[serde(default)]
    pub sov_para_drop_num: Option<Vec<i32>>,
    /// `[General] YuriParaDropInf` (S-0522 claim 2); absent: empty.
    #[serde(default)]
    pub yuri_para_drop_inf: Option<Vec<String>>,
    /// `[General] YuriParaDropNum` (S-0522 claim 2); absent: empty.
    #[serde(default)]
    pub yuri_para_drop_num: Option<Vec<i32>>,
    /// `[General] AmerParaDropInf` (S-0522 claim 2); absent: empty.
    #[serde(default)]
    pub amer_para_drop_inf: Option<Vec<String>>,
    /// `[General] AmerParaDropNum` (S-0522 claim 2); absent: empty.
    #[serde(default)]
    pub amer_para_drop_num: Option<Vec<i32>>,
    /// `[AudioVisual] SpyPlaneCamera`, the camera sound (S-0513 claim 44);
    /// absent: none.
    #[serde(default)]
    pub spy_plane_camera: Option<String>,
    /// `[General] ParadropRadius`, leptons (S-0522 claims 6 and 7); absent:
    /// 1024.
    #[serde(default)]
    pub paradrop_radius: Option<i32>,
    /// `[General] Parachute`, the parachute animation type (S-0522 claim
    /// 13); absent: none.
    #[serde(default)]
    pub parachute: Option<String>,
    /// `[AudioVisual] ChuteSound`, the parachute sound (S-0522 claim 9);
    /// absent: none.
    #[serde(default)]
    pub chute_sound: Option<String>,
    /// `[CombatDamage] PsychicRevealRadius`, cells (S-0610 claim 14);
    /// absent: 3.
    #[serde(default)]
    pub psychic_reveal_radius: Option<i32>,
    /// `[AudioVisual] PsychicRevealActivateSound`, the reveal's activation
    /// sound (S-0610 claim 13); absent: none.
    #[serde(default)]
    pub psychic_reveal_activate_sound: Option<String>,
    /// `[AudioVisual] BerserkColor`, the berserk tint's colour-adjustment
    /// table position (S-0610 claim 7); absent: 0.
    #[serde(default)]
    pub berserk_color: Option<i32>,
    /// `[CombatDamage] BerzerkAllowed`: read, no effect (S-0610 claim 16);
    /// absent: no.
    #[serde(default)]
    pub berzerk_allowed: Option<bool>,
    /// `[General] MutateExplosion`: explosion mode when on, cell mode when
    /// off (S-0611 claims 3 and 4); absent: no (cell mode).
    #[serde(default)]
    pub mutate_explosion: Option<bool>,
    /// `[SpecialWeapons] MutateExplosionWarhead`, the explosion warhead
    /// (S-0611 claim 3); absent: none.
    #[serde(default)]
    pub mutate_explosion_warhead: Option<String>,
    /// `[SpecialWeapons] MutateWarhead`, the cell warhead (S-0611 claim 4);
    /// absent: none.
    #[serde(default)]
    pub mutate_warhead: Option<String>,
    /// `[General] InfantryMutate`, the conversion animation type (S-0611
    /// claims 8 to 12); absent: none.
    #[serde(default)]
    pub infantry_mutate: Option<String>,
    /// `[General] AnimToInfantry`, the animation-to-infantry list (S-0611
    /// F4); absent: empty.
    #[serde(default)]
    pub anim_to_infantry: Option<Vec<String>>,
    /// `[General] IonBlast`, the genetic mutation's launch animation
    /// (S-0611 claim 2); absent: none.
    #[serde(default)]
    pub ion_blast: Option<String>,
    /// `[AudioVisual] GeneticMutatorActivateSound`, the activation sound
    /// (S-0611 claim 2); absent: none.
    #[serde(default)]
    pub genetic_mutator_activate_sound: Option<String>,
    /// `[General] DominatorFirstAnim`, the domination's lead-in animation
    /// type (S-0612 claims 1 and 4); absent: none.
    #[serde(default)]
    pub dominator_first_anim: Option<String>,
    /// `[General] DominatorSecondAnim`, the domination's sweep animation
    /// type (S-0612 claims 5 and 8); absent: none.
    #[serde(default)]
    pub dominator_second_anim: Option<String>,
    /// `[General] DominatorFireAtPercentage`, *p* of S-0612 F1; absent: 50.
    #[serde(default)]
    pub dominator_fire_at_percentage: Option<i32>,
    /// `[General] DominatorCaptureRange`, *r* of S-0612 claim 11 (ring
    /// metric); absent: 2.
    #[serde(default)]
    pub dominator_capture_range: Option<i32>,
    /// `[General] DominatorDamage`, the sweep's area damage amount (S-0612
    /// claim 9); absent: 50.
    #[serde(default)]
    pub dominator_damage: Option<i32>,
    /// `[General] DominatorWarhead`, the sweep's area damage warhead
    /// (S-0612 claim 9); absent: none.
    #[serde(default)]
    pub dominator_warhead: Option<String>,
    /// `[AudioVisual] PsychicDominatorActivateSound`, the activation sound
    /// (S-0612 claim 1); absent: none.
    #[serde(default)]
    pub psychic_dominator_activate_sound: Option<String>,
}

/// One entry of the power-up table (S-0520 Terms): weight, animation, water
/// flag and argument.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Powerup {
    /// The weight of the random choice.
    #[serde(default)]
    pub weight: i32,
    /// The animation type key shown at the pickup.
    #[serde(default)]
    pub animation: Option<String>,
    /// Whether the effect works on a water cell.
    #[serde(default)]
    pub water: bool,
    /// The argument *A*, a double.
    #[serde(default)]
    pub argument: Option<Decimal>,
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
    /// original only; decimal-prefix items, S-0003 claim 23); absent: empty.
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
    /// `[General] SpyMoneyStealPercent`, the steal fraction *f*, a
    /// percentage field kept in single precision (S-0510 claim 6, Import
    /// mapping); absent: 0.5.
    #[serde(default)]
    pub spy_money_steal_percent: Option<Decimal>,
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
    /// `[General] RadarEventSpeed`, the radar-event zoom contraction speed
    /// *S* in pixels per drawn frame, a single (S-0410 F9); absent: 1.
    #[serde(default)]
    pub radar_event_speed: Option<Decimal>,
    /// `[General] RadarEventMinRadius`, the minimum zoom radius *M* in
    /// pixels (S-0410 F9); absent: 5.
    #[serde(default)]
    pub radar_event_min_radius: Option<i32>,
    /// `[General] RadarEventRotationSpeed`, the initial angular speed *Ω*
    /// in radians per drawn frame, a single (S-0410 F9); absent:
    /// 0.10000000149011612.
    #[serde(default)]
    pub radar_event_rotation_speed: Option<Decimal>,
    /// `[General] MultiplayerAICM`, the original's extra-credit percentages
    /// by difficulty index (S-0412 claim 3; decimal-prefix items, S-0003
    /// claim 23); absent: no entries.
    #[serde(default)]
    pub multiplayer_aicm: Option<Vec<i32>>,
    /// `[General] AIHateDelays`, the original's computer-player timer list
    /// by difficulty index (S-0412 claim 3, the original only; no effect in
    /// our engine; decimal-prefix items, S-0003 claim 23); absent: no
    /// entries.
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
    /// `[AudioVisual] DeployDir`, the game-wide deploy direction, an 8-step
    /// direction (S-0502 claim 15 and Import mapping; absent 0).
    #[serde(default)]
    pub deploy_dir: Option<i32>,
    /// `[General] FlightLevel`, the game-wide cruise height in leptons
    /// (S-0501 Terms); absent: 500.
    #[serde(default)]
    pub flight_level: Option<i32>,
    /// `[General] ReloadRate`, the pad reload rate *r* in game minutes, a
    /// double (S-0501 F10); absent: 0.05.
    #[serde(default)]
    pub reload_rate: Option<Decimal>,
    /// `[General] CurleyShuffle`, the re-approach flag (S-0501 claim 14);
    /// absent: no.
    #[serde(default)]
    pub curley_shuffle: Option<bool>,
    /// `[General] AttackingAircraftSightRange`, the reveal radius on firing
    /// in cells (S-0501 claim 20); absent: 5.
    #[serde(default)]
    pub attacking_aircraft_sight_range: Option<i32>,
    /// `[AudioVisual] SpyPlaneCameraFrames`, the spy-plane approach delay in
    /// ticks (S-0501 claim 34); absent: 16.
    #[serde(default)]
    pub spy_plane_camera_frames: Option<i32>,
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
    /// `[CrateRules] CrateRadius`, cells as decimal text (S-0520 Import
    /// mapping, F3); absent: 640 leptons. The importer drops a value of
    /// exactly -1 (it keeps the value in force).
    #[serde(default)]
    pub crate_radius: Option<Decimal>,
    /// `[CrateRules] CrateRegen`, game minutes, a double (S-0520 Import
    /// mapping); absent: 10.
    #[serde(default)]
    pub crate_regen: Option<Decimal>,
    /// `[CrateRules] UnitCrateType`, a vehicle type key (S-0520 claim 13).
    #[serde(default)]
    pub unit_crate_type: Option<String>,
    /// `[CrateRules] WoodCrateImg`, an overlay type key (S-0520 claim 3).
    #[serde(default)]
    pub wood_crate_img: Option<String>,
    /// `[CrateRules] WaterCrateImg`, an overlay type key (S-0520 claim 3).
    #[serde(default)]
    pub water_crate_img: Option<String>,
    /// `[CombatDamage] FlameDamage`, a warhead key (S-0520 claim 17).
    #[serde(default)]
    pub flame_damage: Option<String>,
    /// `[Powerups]`, the nineteen power-ups in table order (S-0520 Import
    /// mapping); absent: the built-in table.
    #[serde(default)]
    pub powerups: Option<Vec<Powerup>>,
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
    /// `[MultiplayerDialogSettings] BridgeDestruction`, the default of the
    /// bridge-destruction game option (S-0511 Import mapping); absent: yes.
    #[serde(default)]
    pub bridge_destruction: Option<bool>,
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
    /// `[General] RevealTriggerRadius`, *R* of the reveal action in cells
    /// (S-0011 claim 26); absent: 5.
    #[serde(default)]
    pub reveal_trigger_radius: Option<i32>,
    /// `[MultiplayerDialogSettings] AIDifficulty`, the initial session
    /// computer difficulty that gates map triggers (S-0011 claim 4); absent: 1.
    #[serde(default)]
    pub ai_difficulty: Option<i32>,
}
