//! The economy fields of the typed `sim` section (M5): the fields of object
//! types (`econ`), countries (`econ`), tiberiums, superweapons and the
//! game-wide economy values (`general.economy`), each read as the Import
//! mapping of the spec named on its row says (S-0401 to S-0416), with the
//! importer warnings those specs ask for.

use super::*;

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
];

/// Fields of infantry types only.
const INFANTRY_ECON: &[Row] = &[
    ("harvest_rate", "HarvestRate", Int, "S-0402"),
    ("nominal", "Nominal", Bool, "S-0414"),
];

/// Fields of vehicle types only.
const VEHICLE_ECON: &[Row] = &[("powered_unit", "PoweredUnit", Bool, "S-0405")];

/// Fields of aircraft types only.
const AIRCRAFT_ECON: &[Row] = &[("airport_bound", "AirportBound", Bool, "S-0407")];

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
    // S-0308 table P row 6: a building that is both is no airstrike target.
    ("resource_gatherer", "ResourceGatherer", Bool, "S-0308"),
    (
        "resource_destination",
        "ResourceDestination",
        Bool,
        "S-0308",
    ),
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
];

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
    ("kind", "Type", Text, "S-0407"),
];

const GENERAL: &[u8] = b"General"; // spec: S-0004 Import mapping (the general settings section)
const AI: &[u8] = b"AI"; // spec: S-0407 Import mapping (`[AI]`)
const AUDIO_VISUAL: &[u8] = b"AudioVisual"; // spec: S-0409 Import mapping (`[AudioVisual]`)
const CRATE_RULES: &[u8] = b"CrateRules"; // spec: S-0412 Import mapping (`[CrateRules]`)
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
        Ints,
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
        "multiplayer_aicm",
        GENERAL,
        "MultiplayerAICM",
        Ints,
        "S-0412",
    ),
    ("ai_hate_delays", GENERAL, "AIHateDelays", Ints, "S-0412"),
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
    (
        "c4_warhead",
        COMBAT_DAMAGE,
        "C4Warhead",
        Ref(Kind::Warheads),
        "S-0414",
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
    for k in NO_EFFECT_FACTORS {
        out.push(("countries", k.to_owned(), "S-0413"));
    }
    for (_, section, k, _, spec) in GENERAL_ECON {
        let cat = match *section {
            s if s == AI => "global:AI",
            s if s == AUDIO_VISUAL => "global:AudioVisual",
            s if s == COMBAT_DAMAGE => "global:CombatDamage",
            s if s == CRATE_RULES => "global:CrateRules",
            s if s == DIALOG => "global:MultiplayerDialogSettings",
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
        out.push(("art", (*k).to_owned(), "S-0402"));
    }
    out.push(("art", "QueueingCell".to_owned(), "S-0402"));
    out.push(("art", "NumberOfDocks".to_owned(), "S-0402"));
    out.push(("art", "Buildup".to_owned(), "S-0108, S-0408"));
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
            }
            Kind::Aircraft => {
                self.efields(&mut econ, t, kind, UNIT_ECON);
                self.efields(&mut econ, t, kind, AIRCRAFT_ECON);
            }
            Kind::Buildings => {
                self.efields(&mut econ, t, kind, BUILDING_ECON);
                self.building_art_econ(&mut econ, index, t);
                self.factory_warning(t);
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

    /// The art fields of a building type's image section (S-0402 Import
    /// mapping).
    fn building_art_econ(&mut self, econ: &mut Rec, index: usize, t: &TypeRec) {
        let kind = Kind::Buildings;
        // spec: S-0402 Import mapping (art NumberOfDocks: an integer)
        if let Some(v) = self.art_int(kind, index, t, "NumberOfDocks") {
            econ.put("art_number_of_docks", int_node(v));
        }
        // spec: S-0402 Import mapping (art QueueingCell: two integers, cells)
        if let Some(v) = self.art_tuple::<2>(kind, index, t, "QueueingCell") {
            econ.put(
                "queueing_cell",
                jsonc::array(v.iter().map(|&x| int_node(x)).collect()),
            );
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
        if buildup == Some(true) {
            econ.put("buildup", jsonc::boolean(true));
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
    }

    /// `Factory` is not converted: no approved spec gives the words its value
    /// is compared with (placeholder, Q-0100).
    fn factory_warning(&mut self, t: &TypeRec) {
        let Some(state) = t.keys.get(&b"Factory"[..]) else {
            return;
        };
        let Some(e) = state.history.last() else {
            return;
        };
        if matches!(e.what, What::Set(_)) {
            self.warn(e.layer, e.line, format!(
                "buildings {} Factory: the words of the factory field are not specified yet; the building is not a factory building (placeholder, Q-0100)",
                show(&t.name)
            ));
        }
    }

    /// The `econ` record of a country.
    pub(super) fn country_econ(&mut self, rec: &mut Rec, t: &TypeRec) {
        let mut econ = Rec::default();
        self.efields(&mut econ, t, Kind::Countries, COUNTRY_ECON);
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
        self.efields(rec, t, Kind::Superweapons, SUPERWEAPON_FIELDS);
    }

    /// The game-wide economy values (`general.economy`).
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
