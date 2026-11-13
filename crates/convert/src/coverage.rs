//! `coverage.csv` (PLAN.md 4.6): one row per (category, key) of the user's
//! rules and art files, with the outcome of each key in this converter.
//!
//! Outcomes: `field` (interpreted per an approved spec and part of the sim
//! data), `render` (interpreted per a rendering spec), `folded` (consumed
//! by the importer itself, such as type lists), `ignored`, and
//! `unsupported` (kept as INI text: no approved spec maps the key yet, or
//! one names it but this converter does not interpret it yet). The file
//! lists names only, never values.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use glasnost_ini::Document;

use crate::kinds::Kind;
use crate::layers::Outcome as Layered;
use crate::names::show;

/// A coverage outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Outcome {
    /// Interpreted, sim data.
    Field,
    /// Consumed by the importer (type lists, side lists, colour tables).
    Folded,
    /// Interpreted, render data.
    Render,
    /// Hint for our own AI.
    AiHint,
    /// Deliberately ignored.
    Ignored,
    /// Kept as INI text.
    Unsupported,
}

impl Outcome {
    /// The CSV word.
    pub fn word(self) -> &'static str {
        match self {
            Outcome::Field => "field",
            Outcome::Folded => "folded",
            Outcome::Render => "render",
            Outcome::AiHint => "ai-hint",
            Outcome::Ignored => "ignored",
            Outcome::Unsupported => "unsupported",
        }
    }
}

/// Keys this converter interprets: (category, key, outcome, spec).
/// Category `*` matches every category.
// spec: S-0004 Import mapping, S-0006 Import mapping, S-0007 Import mapping, S-0103 Import mapping
const CONVERTED: &[(&str, &str, Outcome, &str)] = &[
    ("countries", "Side", Outcome::Field, "S-0004"),
    ("countries", "Name", Outcome::Field, "S-0004"),
    ("countries", "ParentCountry", Outcome::Field, "S-0006"),
    ("vehicles", "DeploysInto", Outcome::Field, "S-0004"),
    ("projectiles", "AirburstWeapon", Outcome::Field, "S-0004"),
    (
        "projectiles",
        "ShrapnelWeapon",
        Outcome::Field,
        "S-0004, S-0318",
    ),
    (
        "particle_systems",
        "HoldsWhat",
        Outcome::Field,
        "S-0004, S-0006",
    ),
    ("particle_systems", "BehavesLike", Outcome::Field, "S-0006"),
    ("particles", "BehavesLike", Outcome::Field, "S-0006"),
    ("buildings", "SuperWeapon", Outcome::Field, "S-0004"),
    // spec: S-0605 Import mapping (absorbing buildings)
    ("buildings", "InfantryAbsorb", Outcome::Field, "S-0605"),
    ("buildings", "UnitAbsorb", Outcome::Field, "S-0605"),
    ("buildings", "Passengers", Outcome::Field, "S-0605"),
    ("buildings", "SizeLimit", Outcome::Field, "S-0605"),
    (
        "global:AudioVisual",
        "EnterBioReactorSound",
        Outcome::Field,
        "S-0605",
    ),
    (
        "global:AudioVisual",
        "LeaveBioReactorSound",
        Outcome::Field,
        "S-0605",
    ),
    // spec: S-0606 Import mapping (vehicle bunkers)
    ("buildings", "Bunker", Outcome::Field, "S-0606"),
    ("vehicles", "Bunkerable", Outcome::Field, "S-0606"),
    ("infantry", "Bunkerable", Outcome::Field, "S-0606"),
    ("aircraft", "Bunkerable", Outcome::Field, "S-0606"),
    (
        "global:AudioVisual",
        "BunkerWallsUpSound",
        Outcome::Field,
        "S-0606",
    ),
    (
        "global:AudioVisual",
        "BunkerWallsDownSound",
        Outcome::Field,
        "S-0606",
    ),
    // spec: S-0603 Import mapping (recycling buildings)
    ("buildings", "Grinding", Outcome::Field, "S-0603"),
    (
        "global:AudioVisual",
        "EnterGrinderSound",
        Outcome::Field,
        "S-0603",
    ),
    (
        "global:AudioVisual",
        "LeaveGrinderSound",
        Outcome::Field,
        "S-0603",
    ),
    (
        "warheads",
        "Verses",
        Outcome::Field,
        "S-0003, S-0006, S-0305",
    ),
    ("superweapons", "RechargeTime", Outcome::Field, "S-0006"),
    ("global:CrateRules", "CrateImg", Outcome::Field, "S-0004"),
    // spec: S-0520 Import mapping (the radius and the power-up table are read apart; the sounds and the campaign-only and no-effect keys are read and ignored)
    ("global:CrateRules", "CrateRadius", Outcome::Field, "S-0520"),
    ("global:Powerups", "Money", Outcome::Field, "S-0520"),
    ("global:Powerups", "Unit", Outcome::Field, "S-0520"),
    ("global:Powerups", "HealBase", Outcome::Field, "S-0520"),
    ("global:Powerups", "Cloak", Outcome::Field, "S-0520"),
    ("global:Powerups", "Explosion", Outcome::Field, "S-0520"),
    ("global:Powerups", "Napalm", Outcome::Field, "S-0520"),
    ("global:Powerups", "Squad", Outcome::Field, "S-0520"),
    ("global:Powerups", "Darkness", Outcome::Field, "S-0520"),
    ("global:Powerups", "Reveal", Outcome::Field, "S-0520"),
    ("global:Powerups", "Armor", Outcome::Field, "S-0520"),
    ("global:Powerups", "Speed", Outcome::Field, "S-0520"),
    ("global:Powerups", "Firepower", Outcome::Field, "S-0520"),
    ("global:Powerups", "ICBM", Outcome::Field, "S-0520"),
    (
        "global:Powerups",
        "Invulnerability",
        Outcome::Field,
        "S-0520",
    ),
    ("global:Powerups", "Veteran", Outcome::Field, "S-0520"),
    ("global:Powerups", "IonStorm", Outcome::Field, "S-0520"),
    ("global:Powerups", "Gas", Outcome::Field, "S-0520"),
    ("global:Powerups", "Tiberium", Outcome::Field, "S-0520"),
    ("global:Powerups", "Pod", Outcome::Field, "S-0520"),
    (
        "global:CrateRules",
        "HealCrateSound",
        Outcome::Ignored,
        "S-0520",
    ),
    ("global:CrateRules", "FreeMCV", Outcome::Ignored, "S-0520"),
    (
        "global:CrateRules",
        "SoloCrateMoney",
        Outcome::Ignored,
        "S-0520",
    ),
    (
        "global:CrateRules",
        "SilverCrate",
        Outcome::Ignored,
        "S-0520",
    ),
    ("global:CrateRules", "WoodCrate", Outcome::Ignored, "S-0520"),
    (
        "global:CrateRules",
        "WaterCrate",
        Outcome::Ignored,
        "S-0520",
    ),
    (
        "global:AudioVisual",
        "CrateMoneySound",
        Outcome::Ignored,
        "S-0520",
    ),
    (
        "global:AudioVisual",
        "CrateRevealSound",
        Outcome::Ignored,
        "S-0520",
    ),
    (
        "global:AudioVisual",
        "CrateFireSound",
        Outcome::Ignored,
        "S-0520",
    ),
    (
        "global:AudioVisual",
        "CrateArmourSound",
        Outcome::Ignored,
        "S-0520",
    ),
    (
        "global:AudioVisual",
        "CrateSpeedSound",
        Outcome::Ignored,
        "S-0520",
    ),
    (
        "global:AudioVisual",
        "CrateUnitSound",
        Outcome::Ignored,
        "S-0520",
    ),
    (
        "global:AudioVisual",
        "CratePromoteSound",
        Outcome::Ignored,
        "S-0520",
    ),
    (
        "global:CrateRules",
        "WoodCrateImg",
        Outcome::Field,
        "S-0004",
    ),
    (
        "global:CrateRules",
        "WaterCrateImg",
        Outcome::Field,
        "S-0004",
    ),
    (
        "global:Maximums",
        "Players",
        Outcome::Field,
        "S-0006, S-0009",
    ),
    ("overlays", "CellAnim", Outcome::Field, "S-0009"),
    // spec: S-0502 claim 2 and Import mapping ([JumpjetControls] TurnRate, Speed, Climb, Acceleration, WobblesPerSecond and WobbleDeviation are read and ignored: no effect on play; CruiseHeight is S-0102's layer threshold)
    (
        "global:JumpjetControls",
        "TurnRate",
        Outcome::Ignored,
        "S-0502",
    ),
    (
        "global:JumpjetControls",
        "Speed",
        Outcome::Ignored,
        "S-0502",
    ),
    (
        "global:JumpjetControls",
        "Climb",
        Outcome::Ignored,
        "S-0502",
    ),
    (
        "global:JumpjetControls",
        "Acceleration",
        Outcome::Ignored,
        "S-0502",
    ),
    (
        "global:JumpjetControls",
        "WobblesPerSecond",
        Outcome::Ignored,
        "S-0502",
    ),
    (
        "global:JumpjetControls",
        "WobbleDeviation",
        Outcome::Ignored,
        "S-0502",
    ),
    ("global:Easy", "*", Outcome::Field, "S-0006"),
    ("global:Normal", "*", Outcome::Field, "S-0006"),
    ("global:Difficult", "*", Outcome::Field, "S-0006"),
    (
        "global:AudioVisual",
        "ExtraUnitLight",
        Outcome::Render,
        "S-0104, S-0006",
    ),
    (
        "global:AudioVisual",
        "ExtraInfantryLight",
        Outcome::Render,
        "S-0104, S-0006",
    ),
    (
        "global:AudioVisual",
        "ExtraAircraftLight",
        Outcome::Render,
        "S-0104, S-0006",
    ),
    (
        "buildings",
        "LightIntensity",
        Outcome::Render,
        "S-0104, S-0006",
    ),
    (
        "buildings",
        "LightRedTint",
        Outcome::Render,
        "S-0104, S-0006",
    ),
    (
        "buildings",
        "LightGreenTint",
        Outcome::Render,
        "S-0104, S-0006",
    ),
    (
        "buildings",
        "LightBlueTint",
        Outcome::Render,
        "S-0104, S-0006",
    ),
    (
        "buildings",
        "BarrelTravel",
        Outcome::Render,
        "S-0105, S-0006",
    ),
    (
        "buildings",
        "BarrelCompressFrames",
        Outcome::Render,
        "S-0105, S-0006",
    ),
    (
        "buildings",
        "BarrelHoldFrames",
        Outcome::Render,
        "S-0105, S-0006",
    ),
    (
        "buildings",
        "BarrelRecoverFrames",
        Outcome::Render,
        "S-0105, S-0006",
    ),
    ("*", "Image", Outcome::Field, "S-0007"),
    ("global:Colors", "*", Outcome::Render, "S-0103"),
    ("global:ColorAdd", "*", Outcome::Render, "S-0006"),
    ("list", "*", Outcome::Folded, "S-0002, S-0004"),
    ("global:Sides", "*", Outcome::Folded, "S-0004"),
    // Keys the combat specs say have no effect (read and ignored, with the warnings they ask for).
    // spec: S-0302 Edge cases (MinDamage, a country's Armor), S-0309 Edge cases (the Dumb coefficients), S-0314 claim 11 (TiberiumHeal, HealScanRadius)
    (
        "global:CombatDamage",
        "MinDamage",
        Outcome::Ignored,
        "S-0302",
    ),
    ("countries", "Armor", Outcome::Ignored, "S-0302"),
    (
        "global:General",
        "DumbMyEffectivenessCoefficient",
        Outcome::Ignored,
        "S-0309",
    ),
    (
        "global:General",
        "DumbTargetEffectivenessCoefficient",
        Outcome::Ignored,
        "S-0309",
    ),
    (
        "global:General",
        "DumbTargetSpecialThreatCoefficient",
        Outcome::Ignored,
        "S-0309",
    ),
    (
        "global:General",
        "DumbTargetStrengthCoefficient",
        Outcome::Ignored,
        "S-0309",
    ),
    (
        "global:General",
        "DumbTargetDistanceCoefficient",
        Outcome::Ignored,
        "S-0309",
    ),
    ("global:General", "TiberiumHeal", Outcome::Ignored, "S-0314"),
    (
        "global:General",
        "HealScanRadius",
        Outcome::Ignored,
        "S-0314",
    ),
    ("infantry", "TiberiumHeal", Outcome::Ignored, "S-0314"),
    ("vehicles", "TiberiumHeal", Outcome::Ignored, "S-0314"),
    ("aircraft", "TiberiumHeal", Outcome::Ignored, "S-0314"),
    ("buildings", "TiberiumHeal", Outcome::Ignored, "S-0314"),
];

/// Keys each approved spec's import mapping names, for the `spec` column of
/// keys this converter does not interpret yet.
// spec: S-0003, S-0004, S-0006, S-0007, S-0101 to S-0108, S-0301 to S-0315, S-0405 (Import mapping key names)
const NAMED: &[(&str, &[&str])] = &[
    ("S-0301", &["CompEasyBonus"]),
    ("S-0405", &["ActivateSound", "DeactivateSound"]),
    (
        "S-0303",
        &[
            "DestroyableBridges",
            "ShakeXhi",
            "ShakeXlo",
            "ShakeYhi",
            "ShakeYlo",
        ],
    ),
    ("S-0311", &["EliteFlashTimer"]),
    (
        "S-0312",
        &["Bright", "CrateBeneath", "CrateBeneathIsMoney", "Wake"],
    ),
    (
        "S-0313",
        &[
            "AlternateFLH0",
            "AlternateFLH1",
            "AlternateFLH2",
            "AlternateFLH3",
            "AlternateFLH4",
        ],
    ),
    (
        "S-0003",
        &[
            "Armor",
            "ConditionRed",
            "ConditionYellow",
            "Cost",
            "ForbiddenHouses",
            "Foundation",
            "LaserColor",
            "MovementZone",
            "Owner",
            "Pip",
            "Prerequisite",
            "PrimaryFireFLH",
            "RequiredHouses",
            "SecondaryFireFLH",
            "SecretHouses",
            "Selectable",
            "Speed",
            "SpeedType",
            "Strength",
        ],
    ),
    (
        "S-0004",
        &[
            "AIBasePlanningSide",
            "BuildCat",
            "ConsideredAircraft",
            "Naval",
            "Owner",
            "TechLevel",
            "UIName",
            "Value",
        ],
    ),
    (
        "S-0007",
        &[
            "AlphaImage",
            "AltCameo",
            "AlternateArcticArt",
            "BarrelAnimIsVoxel",
            "BibShape",
            "Buildup",
            "Cameo",
            "DeployingAnim",
            "DoorAnim",
            "IsGattling",
            "NewTheater",
            "NoSpawnAlt",
            "RoofDeployingAnim",
            "Rubble",
            "SpecialZOverlay",
            "Theater",
            "Trainable",
            "Turret",
            "TurretAnim",
            "TurretAnimIsVoxel",
            "TurretCount",
            "UndeploysInto",
            "UnderDoorAnim",
            "UnderRoofDoorAnim",
            "Voxel",
            "VoxelBarrelFile",
        ],
    ),
    (
        "S-0102",
        &[
            "CruiseHeight",
            "DrawFlat",
            "Flat",
            "Gate",
            "IsARock",
            "JumpJet",
            "JumpjetHeight",
            "Layer",
            "Locomotor",
            "NormalZAdjust",
            "Shadow",
            "SpecialZOverlayZAdjust",
            "Tiberium",
            "Tiled",
            "TurretAnimX",
            "TurretAnimY",
            "TurretAnimYSort",
            "TurretAnimZAdjust",
            "Wall",
            "YDrawOffset",
            "YSortAdjust",
            "ZAdjust",
            "ZFudgeBridge",
            "ZFudgeCliff",
            "ZFudgeColumn",
            "ZFudgeTunnel",
            "ZShapePointMove",
            "BalloonHover",
        ],
    ),
    (
        "S-0103",
        &[
            "AltPalette",
            "BombParachute",
            "IsVeins",
            "Palette",
            "Parachute",
            "ShouldUseCellDrawer",
            "TerrainPalette",
            "UseNormalLight",
        ],
    ),
    (
        "S-0104",
        &["LightVisibility", "NukeAmbientChangeRate", "SpawnsTiberium"],
    ),
    (
        "S-0105",
        &[
            "DisableShadowCache",
            "DisableVoxelCache",
            "FireAngle",
            "NoShadow",
            "PitchAngle",
            "PitchSpeed",
            "RollAngle",
            "ShadowIndex",
            "TurretCompressFrames",
            "TurretHoldFrames",
            "TurretOffset",
            "TurretRecoil",
            "TurretRecoverFrames",
            "TurretTravel",
            "UseBuffer",
            "UseTurretShadow",
        ],
    ),
    (
        "S-0107",
        &[
            "Crashable",
            "Crawls",
            "Cyborg",
            "DeadBodies",
            "DeathAnims",
            "DeploySound",
            "EnterWaterSound",
            "Fearless",
            "Fraidycat",
            "IdleActionFrequency",
            "InfDeath",
            "LeaveWaterSound",
            "NotHuman",
            "UndeploySound",
            "VoiceComment",
        ],
    ),
    (
        "S-0108",
        &[
            "ActiveAnim",
            "ActiveAnimFour",
            "ActiveAnimThree",
            "ActiveAnimTwo",
            "BuildupTime",
            "CanBeOccupied",
            "ChargedAnimTime",
            "DamagedDoor",
            "DoorStages",
            "ExtraDamageStage",
            "Gate",
            "GateStages",
            "IdleAnim",
            "InfantryAbsorb",
            "InvisibleInGame",
            "IsAnimDelayedFire",
            "Normalized",
            "Power",
            "PreProductionAnim",
            "ProductionAnim",
            "Refinery",
            "SpecialAnim",
            "SpecialAnimFour",
            "SpecialAnimThree",
            "SpecialAnimTwo",
            "Storage",
            "SuperAnim",
            "SuperAnimFour",
            "SuperAnimThree",
            "SuperAnimTwo",
            "TurretAnimDamaged",
            "TurretAnimGarrisoned",
            "UnitRepair",
            "Upgrades",
        ],
    ),
];

/// One row.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Row {
    /// Category: a type kind, `global:Section`, `list`, `art` or `other`.
    pub category: String,
    /// The key.
    pub key: String,
    /// Outcome.
    pub outcome: Outcome,
    /// Specs that name the key.
    pub spec: String,
    /// Note.
    pub note: String,
}

/// The simulation keys `convert::sim` converts: (category, key) to spec.
type SimKeys = BTreeMap<(String, String), &'static str>;

fn sim_keys() -> SimKeys {
    let mut m = SimKeys::new();
    for (cat, key, spec) in crate::sim::converted_keys()
        .into_iter()
        .chain(crate::sim::econ_converted_keys())
    {
        m.entry((cat.to_owned(), key)).or_insert(spec);
    }
    m
}

fn classify(category: &str, key: &str, sim: &SimKeys) -> (Outcome, String, String) {
    let hit = CONVERTED.iter().find(|(c, k, _, _)| {
        (*c == "*" || *c == category) && (*k == "*" || k.as_bytes() == key.as_bytes())
    });
    if let Some((_, _, outcome, spec)) = hit {
        return (*outcome, (*spec).to_owned(), String::new());
    }
    // spec: S-0105 Import mapping (the turret table pairs of a vehicle type, read for the fixed type name only, quirks.turret_table_fixed_type_only)
    if category == "vehicles" && crate::turrets::PAIR_KEYS.contains(&key) {
        return (Outcome::Render, "S-0105".to_owned(), String::new());
    }
    if let Some(spec) = sim.get(&(category.to_owned(), key.to_owned())) {
        return (Outcome::Field, (*spec).to_owned(), String::new());
    }
    let named: Vec<&str> = NAMED
        .iter()
        .filter(|(_, keys)| keys.contains(&key))
        .map(|(s, _)| *s)
        .collect();
    if named.is_empty() {
        (
            Outcome::Unsupported,
            String::new(),
            "no approved spec maps this key yet; kept as INI text".into(),
        )
    } else {
        (
            Outcome::Unsupported,
            named.join(", "),
            "named by a spec, not converted yet; kept as INI text".into(),
        )
    }
}

/// Every (category, key) of the base rules document and the art database,
/// classified. Categories follow the layered result: a section is a type
/// section of each kind that has a type of that stored name.
pub fn rows(base: &Document, art: &Document, layered: &Layered) -> Vec<Row> {
    let mut kinds_of: BTreeMap<Vec<u8>, Vec<Kind>> = BTreeMap::new();
    for (kind, reg) in &layered.registries {
        for t in reg {
            kinds_of.entry(t.name.clone()).or_default().push(*kind);
        }
    }
    let lists: BTreeSet<&[u8]> = Kind::all().filter_map(Kind::list_section).collect();
    let mut pairs: BTreeSet<(String, String)> = BTreeSet::new();
    for s in base.sections() {
        let keys: BTreeSet<String> = s.entries().iter().map(|e| show(&e.key)).collect();
        let cats: Vec<String> = if lists.contains(s.name()) {
            vec!["list".into()]
        } else if s.name() == b"Sides" || s.name() == b"Colors" || s.name() == b"ColorAdd" {
            vec![format!("global:{}", show(s.name()))]
        } else if let Some(kinds) = kinds_of
            .get(s.name())
            .filter(|k| k.iter().any(|k| *k != Kind::Animations))
        {
            kinds
                .iter()
                .filter(|k| **k != Kind::Animations)
                .map(|k| k.key().to_owned())
                .collect()
        } else if layered.globals.contains_key(s.name()) {
            vec![format!("global:{}", show(s.name()))]
        } else {
            vec!["other".into()]
        };
        for c in cats {
            if c == "list" {
                pairs.insert((c.clone(), "*".into()));
                continue;
            }
            for k in &keys {
                pairs.insert((c.clone(), k.clone()));
            }
        }
    }
    for s in art.sections() {
        let cat = if kinds_of
            .get(s.name())
            .is_some_and(|k| k.contains(&Kind::Animations))
        {
            "art:animations"
        } else {
            "art"
        };
        for e in s.entries() {
            pairs.insert((cat.to_owned(), show(&e.key)));
        }
    }
    let sim = sim_keys();
    pairs
        .into_iter()
        .map(|(category, key)| {
            let lookup_cat = if category.starts_with("art") {
                "*"
            } else {
                category.as_str()
            };
            let (mut outcome, mut spec, mut note) = classify(lookup_cat, &key, &sim);
            if category.starts_with("art") {
                // Art-file keys that S-0007 and S-0103 interpret.
                const ART_RENDER: [&str; 19] = [
                    // spec: S-0007 Import mapping, S-0103 Import mapping (art database keys)
                    "Voxel",
                    "Theater",
                    "NewTheater",
                    "Buildup",
                    "DeployingAnim",
                    "RoofDeployingAnim",
                    "DoorAnim",
                    "UnderDoorAnim",
                    "UnderRoofDoorAnim",
                    "Rubble",
                    "SpecialZOverlay",
                    "BibShape",
                    "Cameo",
                    "AltCameo",
                    "AlphaImage",
                    "AlternateArcticArt",
                    "TerrainPalette",
                    "AltPalette",
                    "Palette",
                ];
                if ART_RENDER.contains(&key.as_str()) || key == "Image" {
                    outcome = Outcome::Render;
                    spec = if matches!(key.as_str(), "TerrainPalette" | "AltPalette" | "Palette") {
                        "S-0103".into()
                    } else {
                        "S-0007".into()
                    };
                    note = String::new();
                } else if let Some(s) = sim.get(&("art".to_owned(), key.clone())) {
                    // Art keys the simulation's combat fields read (S-0301, S-0306, S-0310, S-0313).
                    outcome = Outcome::Field;
                    spec = (*s).to_owned();
                    note = String::new();
                }
            }
            Row {
                category,
                key,
                outcome,
                spec,
                note,
            }
        })
        .collect()
}

/// Spellings the original reads where stock data has a different letter
/// case (S-0001 Import mapping, row I2).
// spec: S-0001 Import mapping row I2
const CASE_SENSITIVE_SPELLINGS: [&str; 3] = ["JumpjetAccel", "JumpjetTurnRate", "MaxDebris"];

/// Every key spelling this importer knows: the keys it interprets, the
/// keys the specs' import mappings name, and S-0001's row I2.
pub fn known_keys() -> std::collections::BTreeSet<&'static str> {
    CONVERTED
        .iter()
        .map(|(_, k, _, _)| *k)
        .filter(|k| *k != "*")
        .chain(NAMED.iter().flat_map(|(_, keys)| keys.iter().copied()))
        .chain(crate::turrets::PAIR_KEYS)
        .chain(CASE_SENSITIVE_SPELLINGS)
        .collect()
}

/// Importer warnings for keys whose bytes differ from a key the importer
/// reads only in the letter case of A to Z (S-0001 claim 13, Edge cases):
/// such an entry has no effect.
pub fn case_warnings(doc: &Document, file: &str) -> Vec<crate::layers::Warning> {
    let known = known_keys();
    let mut out = Vec::new();
    for section in doc.sections() {
        for e in section.entries() {
            let Some(k) = known.iter().find(|k| {
                k.as_bytes() != e.key.as_slice() && k.as_bytes().eq_ignore_ascii_case(&e.key)
            }) else {
                continue;
            };
            out.push(crate::layers::Warning {
                file: file.to_owned(),
                line: e.line,
                message: format!(
                    "key {:?} in [{}] differs from {k:?} only in letter case; keys are case-sensitive, so it \
                     has no effect (S-0001 claim 13)",
                    show(&e.key),
                    show(section.name())
                ),
            });
        }
    }
    out
}

fn csv_field(s: &str) -> String {
    if s.contains([',', '"', '\n']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_owned()
    }
}

/// The rows as CSV text with a header line.
pub fn csv(rows: &[Row]) -> String {
    let mut out = String::from("category,key,outcome,spec,note\n");
    for r in rows {
        let _ = writeln!(
            out,
            "{},{},{},{},{}",
            csv_field(&r.category),
            csv_field(&r.key),
            r.outcome.word(),
            csv_field(&r.spec),
            csv_field(&r.note)
        );
    }
    out
}

/// Number of rows per outcome.
pub fn summary(rows: &[Row]) -> BTreeMap<Outcome, usize> {
    let mut m = BTreeMap::new();
    for r in rows {
        *m.entry(r.outcome).or_default() += 1;
    }
    m
}
