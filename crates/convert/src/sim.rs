//! The typed `sim` section of the converted ruleset (`glasnost_rules::sim`):
//! the fields of the user's rules and art data that the simulation reads.
//!
//! Every field is written only when the layered data gives it; an absent
//! field takes the engine default that the simulation applies. Values are
//! the ones in force after every rules layer (S-0006): each key's history is
//! folded in layer order, so a stored value that its value form cannot read
//! (S-0003 claims 2, 3 and 8) or a "not given" value (S-0006 claim 3) keeps
//! the value an earlier layer gave.
//!
//! Decimals are written as text in our grammar
//! `-?digits[.digits][e±digits][%]` that reads as S-0003 reads the stored
//! text ([`decimal_text`]).

use std::collections::BTreeMap;

use glasnost_ini::value::{self, Decimal as IniDecimal, DecimalField};
use glasnost_ini::{Document, Quirks as IniQuirks, Section};
use glasnost_rules::jsonc::{self, Node};

use crate::art::{ArtQuirks, IMAGE_MAX, TypeArt, bool_as_of, given_at};
use crate::emit::type_keys;
use crate::kinds::Kind;
use crate::layers::{
    COMBAT_DAMAGE, Event, KeyState, LayerQuirks, Outcome, TypeRec, Warning, What, is_zero,
};
use crate::names::{encode, show};

/// Bytes of a named value read (S-0003 claim 16: 31 or 127 per field; the
/// specs of these fields do not say which, and only text longer than 31
/// bytes that matches after the cut could tell them apart).
const NAME_LIMIT: usize = value::COMMON_TEXT_LIMIT; // spec: S-0003 claim 16, Constants (common text limit)
/// Bytes of the `Sequence=` value and of each kind value (S-0107 claim 22).
const SEQUENCE_VALUE_MAX: usize = 31; // spec: S-0107 claim 22 (31 characters)
/// Bytes of a building's `Foundation` value (S-0108 Import mapping).
const FOUNDATION_MAX: usize = 31; // spec: S-0108 Import mapping (Foundation, text cut to 31)

/// Decimal text in our grammar that reads as S-0003 reads `stored`, or
/// `None` when the stored value has no number (S-0003 claim 8). The digits
/// and power of ten are those of the decimal prefix (claim 6); a `%`
/// anywhere in the stored value multiplies by the percent factor (claim 7),
/// so it is appended.
pub fn decimal_text(stored: &[u8]) -> Option<String> {
    // spec: S-0003 claim 6 (the longest decimal prefix)
    let t = value::decimal_prefix(stored)?;
    let mut s = String::new();
    if t.neg {
        s.push('-');
    }
    // Leading zeros change no value.
    let first = t
        .digits
        .iter()
        .position(|&d| d != b'0')
        .unwrap_or(t.digits.len() - 1);
    s.push_str(&String::from_utf8_lossy(&t.digits[first..]));
    if t.exp10 != 0 {
        s.push_str(&format!("e{}", t.exp10));
    }
    // spec: S-0003 claim 7 (a `%` anywhere in the value)
    if stored.contains(&b'%') {
        s.push('%');
    }
    Some(s)
}

/// The land types in S-0211's order, with our names and their sections.
// spec: S-0211 Terms (land types 0 to 11), Import mapping (the twelve section names)
pub(crate) const LAND_TYPES: [(&str, &str); 12] = [
    ("clear", "Clear"),
    ("road", "Road"),
    ("water", "Water"),
    ("rock", "Rock"),
    ("wall", "Wall"),
    ("tiberium", "Tiberium"),
    ("beach", "Beach"),
    ("rough", "Rough"),
    ("ice", "Ice"),
    ("railroad", "Railroad"),
    ("tunnel", "Tunnel"),
    ("weeds", "Weeds"),
];

/// Land-type row keys (S-0211 Import mapping) and our field names.
// spec: S-0211 claim 2, Import mapping (the seven multiplier keys)
pub(crate) const LAND_MULTIPLIERS: [(&str, &str); 7] = [
    ("foot", "Foot"),
    ("track", "Track"),
    ("wheel", "Wheel"),
    ("hover", "Hover"),
    ("float", "Float"),
    ("amphibious", "Amphibious"),
    ("float_beach", "FloatBeach"),
];

/// The locomotion classes and their names (S-0211 Terms, Import mapping).
// spec: S-0211 Import mapping (SpeedType names, case ignored)
const LOCOMOTION_CLASSES: [(&str, &str); 8] = [
    ("foot", "Foot"),
    ("track", "Track"),
    ("wheel", "Wheel"),
    ("hover", "Hover"),
    ("winged", "Winged"),
    ("float", "Float"),
    ("amphibious", "Amphibious"),
    ("float_beach", "FloatBeach"),
];

/// The movement zones: our names and the spelling the data uses (S-0211
/// claim 12, Import mapping: hyphens removed, `subterannean` with one r).
// spec: S-0211 claim 4, Import mapping (movement zone names)
const MOVEMENT_ZONES: [(&str, &str); 13] = [
    ("normal", "Normal"),
    ("crusher", "Crusher"),
    ("destroyer", "Destroyer"),
    ("amphibious_destroyer", "AmphibiousDestroyer"),
    ("amphibious_crusher", "AmphibiousCrusher"),
    ("amphibious", "Amphibious"),
    ("subterranean", "Subterannean"),
    ("infantry", "Infantry"),
    ("infantry_destroyer", "InfantryDestroyer"),
    ("fly", "Fly"),
    ("water", "Water"),
    ("water_beach", "WaterBeach"),
    ("crusher_all", "CrusherAll"),
];

/// Behaviour sections (S-0215 Import mapping) and our snake-case names.
// spec: S-0215 Import mapping (behaviour property sections)
const BEHAVIOURS: [(&str, &str); 32] = [
    ("ambush", "Ambush"),
    ("area_guard", "Area Guard"),
    ("attack", "Attack"),
    ("attack_move", "Attack Move"),
    ("capture", "Capture"),
    ("construction", "Construction"),
    ("eaten", "Eaten"),
    ("enter", "Enter"),
    ("guard", "Guard"),
    ("harmless", "Harmless"),
    ("harvest", "Harvest"),
    ("hunt", "Hunt"),
    ("missile", "Missile"),
    ("move", "Move"),
    ("open", "Open"),
    ("paradrop_approach", "Paradrop Approach"),
    ("paradrop_overfly", "Paradrop Overfly"),
    ("patrol", "Patrol"),
    ("queued_move", "QMove"),
    ("repair", "Repair"),
    ("rescue", "Rescue"),
    ("retreat", "Retreat"),
    ("return", "Return"),
    ("sabotage", "Sabotage"),
    ("selling", "Selling"),
    ("sleep", "Sleep"),
    ("spy_plane_approach", "Spyplane Approach"),
    ("spy_plane_overfly", "Spyplane Overfly"),
    ("sticky", "Sticky"),
    ("stop", "Stop"),
    ("unload", "Unload"),
    ("wait", "Wait"),
];

/// Boolean behaviour properties (S-0215 Import mapping).
// spec: S-0215 Import mapping (keys in each behaviour section)
const BEHAVIOUR_FLAGS: [(&str, &str); 6] = [
    ("no_threat", "NoThreat"),
    ("paralysed", "Paralyzed"),
    ("recruitable", "Recruitable"),
    ("retaliate", "Retaliate"),
    ("scatter", "Scatter"),
    ("zombie", "Zombie"),
];

/// Sequence kinds: our snake-case names and their keys (S-0107 Terms and
/// Import mapping).
// spec: S-0107 Import mapping (kind names to keys)
const SEQUENCE_KINDS: [(&str, &str); 42] = [
    ("stand", "Ready"),
    ("guard", "Guard"),
    ("prone", "Prone"),
    ("walk", "Walk"),
    ("fire_standing", "FireUp"),
    ("go_prone", "Down"),
    ("crawl", "Crawl"),
    ("get_up", "Up"),
    ("fire_prone", "FireProne"),
    ("idle_a", "Idle1"),
    ("idle_b", "Idle2"),
    ("die_1", "Die1"),
    ("die_2", "Die2"),
    ("die_3", "Die3"),
    ("die_4", "Die4"),
    ("die_5", "Die5"),
    ("tread", "Tread"),
    ("swim", "Swim"),
    ("wet_idle_a", "WetIdle1"),
    ("wet_idle_b", "WetIdle2"),
    ("wet_die_1", "WetDie1"),
    ("wet_die_2", "WetDie2"),
    ("wet_fire", "WetAttack"),
    ("hover", "Hover"),
    ("fly", "Fly"),
    ("tumble", "Tumble"),
    ("fire_airborne", "FireFly"),
    ("deploy", "Deploy"),
    ("deployed", "Deployed"),
    ("deployed_fire", "DeployedFire"),
    ("deployed_idle", "DeployedIdle"),
    ("undeploy", "Undeploy"),
    ("cheer", "Cheer"),
    ("parachute", "Paradrop"),
    ("air_death_start", "AirDeathStart"),
    ("air_death_fall", "AirDeathFalling"),
    ("air_death_end", "AirDeathFinish"),
    ("panic", "Panic"),
    ("shovel", "Shovel"),
    ("carry", "Carry"),
    ("secondary_fire", "SecondaryFire"),
    ("secondary_fire_prone", "SecondaryProne"),
];

/// Declared directions, 0 = N … 7 = NW (S-0107 Import mapping).
// spec: S-0107 Import mapping (directions N, NE, E, SE, S, SW, W, NW)
const DIRECTIONS: [&str; 8] = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];

/// Fire-frame keys of the type's art section: standing, prone, secondary,
/// secondary-prone (S-0107 Import mapping).
// spec: S-0107 Import mapping (fire frames)
const FIRE_FRAME_KEYS: [&str; 4] = ["FireUp", "FireProne", "SecondaryFire", "SecondaryProne"];

/// The foundation vocabulary and its sizes (S-0108 Import mapping).
// spec: S-0108 Import mapping (Foundation vocabulary)
const FOUNDATIONS: [(&str, [i32; 2]); 22] = [
    ("1x1", [1, 1]),
    ("2x1", [2, 1]),
    ("1x2", [1, 2]),
    ("2x2", [2, 2]),
    ("2x3", [2, 3]),
    ("3x2", [3, 2]),
    ("3x3", [3, 3]),
    ("3x5", [3, 5]),
    ("4x2", [4, 2]),
    ("3x3Refinery", [3, 3]),
    ("1x3", [1, 3]),
    ("3x1", [3, 1]),
    ("4x3", [4, 3]),
    ("1x4", [1, 4]),
    ("1x5", [1, 5]),
    ("2x6", [2, 6]),
    ("2x5", [2, 5]),
    ("5x3", [5, 3]),
    ("4x4", [4, 4]),
    ("3x4", [3, 4]),
    ("6x4", [6, 4]),
    ("0x0", [0, 0]),
];
/// The 8-cell 3 × 3 foundation word (S-0406 claim 2, Import mapping).
const REFINERY_WORD: &str = "3x3Refinery"; // spec: S-0406 Import mapping (the word 3x3Refinery)
/// The foundation a word that matches nothing gives (S-0108 Import mapping).
// spec: S-0108 Import mapping (a word that matches none gives 1x1), S-0003 claim 16
const FOUNDATION_FALLBACK: [i32; 2] = [1, 1];

/// How a field's stored value is read.
#[derive(Clone, Copy)]
enum Form {
    /// S-0003 claim 3.
    Int,
    /// S-0003 claim 2.
    Bool,
    /// S-0003 claims 6 and 7, written as decimal text.
    Dec,
    /// S-0003 claim 1, a name as written (common text limit).
    Text,
    /// A type name (S-0003 claims 1 and 14): the key of the type of that
    /// kind it names, else the name as written; `none` and `<none>` give
    /// no value.
    Ref(Kind),
    /// A list of type names (S-0003 claims 12 and 14), each item as
    /// [`Form::Ref`] gives it; `none` and `<none>` items are left out.
    Refs(Kind),
    /// A list of integers (S-0003 claims 3 and 12).
    Ints,
    /// A debris maxima list (S-0003 claim 20): decimal integer prefixes.
    Maxima,
    /// A list of names that are not type names (S-0003 claims 1 and 12),
    /// each item as [`Form::Text`] gives it, kept in order with none left
    /// out.
    Texts,
}
use Form::{Bool, Dec, Int, Ints, Maxima, Ref, Refs, Text, Texts};

/// Rules fields shared by infantry, vehicle and aircraft types: our field
/// name, the INI key (the field docs of `glasnost_rules::sim::UnitType`).
const UNIT_FIELDS: &[(&str, &str, Form)] = &[
    ("strength", "Strength", Int),
    ("rot", "ROT", Int),
    ("jumpjet_turn_rate", "JumpjetTurnRate", Int),
    ("turret", "Turret", Bool),
    ("turret_spins", "TurretSpins", Bool),
    ("crusher", "Crusher", Bool),
    ("omni_crusher", "OmniCrusher", Bool),
    ("omni_crush_resistant", "OmniCrushResistant", Bool),
    ("crushable", "Crushable", Bool),
    ("deployed_crushable", "DeployedCrushable", Bool),
    ("accelerates", "Accelerates", Bool),
    ("acceleration_factor", "AccelerationFactor", Dec),
    ("deacceleration_factor", "DeaccelerationFactor", Dec),
    ("slowdown_distance", "SlowdownDistance", Int),
    ("passive", "Passive", Bool),
    ("is_train", "IsTrain", Bool),
    ("is_simple_deployer", "IsSimpleDeployer", Bool),
    ("harvester", "Harvester", Bool),
    ("weeder", "Weeder", Bool),
    ("jumpjet", "JumpJet", Bool),
    ("fraidycat", "Fraidycat", Bool),
    ("cyborg", "Cyborg", Bool),
    ("fearless", "Fearless", Bool),
    ("not_human", "NotHuman", Bool),
    ("vehicle_thief", "VehicleThief", Bool),
    ("engineer", "Engineer", Bool),
    ("occupier", "Occupier", Bool),
    ("assaulter", "Assaulter", Bool),
    ("infiltrate", "Infiltrate", Bool),
    ("agent", "Agent", Bool),
    ("thief", "Thief", Bool),
    ("c4", "C4", Bool),
    ("ivan", "Ivan", Bool),
    ("deployer", "Deployer", Bool),
    ("deploy_fire", "DeployFire", Bool),
    ("undeploy_delay", "UndeployDelay", Int),
    ("default_to_guard_area", "DefaultToGuardArea", Bool),
    ("stupid_hunt", "StupidHunt", Bool),
    ("hover_attack", "HoverAttack", Bool),
    ("close_range", "CloseRange", Bool),
    // spec: S-0215 Import mapping (CanApproachTarget: the can-approach property of claim 37 d)
    ("can_approach_target", "CanApproachTarget", Bool),
    ("distributed_fire", "DistributedFire", Bool),
    ("opportunity_fire", "OpportunityFire", Bool),
    ("open_topped", "OpenTopped", Bool),
    ("non_vehicle", "NonVehicle", Bool),
    ("prevent_attack_move", "PreventAttackMove", Bool),
    // spec: S-0214 Import mapping (VoiceSelect: the selection responses, a list of sound names kept in the listed order)
    ("voice_select", "VoiceSelect", Texts),
    ("move_to_shroud", "MoveToShroud", Bool),
    ("pushy", "Pushy", Bool),
    ("perma_disguise", "PermaDisguise", Bool),
    ("attack_friendlies", "AttackFriendlies", Bool),
    (
        "attack_cursor_on_friendlies",
        "AttackCursorOnFriendlies",
        Bool,
    ),
    ("small_visceroid", "SmallVisceroid", Bool),
    // spec: S-0009 Import mapping (LargeVisceroid, Cost, TechLevel, AllowedToStartInMultiplayer)
    ("large_visceroid", "LargeVisceroid", Bool),
    ("cost", "Cost", Int),
    ("tech_level", "TechLevel", Int),
    (
        "allowed_to_start_in_multiplayer",
        "AllowedToStartInMultiplayer",
        Bool,
    ),
    ("immune", "Immune", Bool),
    ("legal_target", "LegalTarget", Bool),
    ("bombable", "Bombable", Bool),
    ("immune_to_radiation", "ImmuneToRadiation", Bool),
    ("balloon_hover", "BalloonHover", Bool),
    ("spray_attack", "SprayAttack", Bool),
    ("tilts_when_crushes", "TiltsWhenCrushes", Bool),
    ("auto_crush", "AutoCrush", Bool),
    ("carryall", "Carryall", Bool),
    ("spawned", "Spawned", Bool),
    ("guard_range", "GuardRange", Dec),
    ("sight", "Sight", Int),
    ("passengers", "Passengers", Int),
    ("storage", "Storage", Int),
    (
        "threat_avoidance_coefficient",
        "ThreatAvoidanceCoefficient",
        Dec,
    ),
    ("deploys_into", "DeploysInto", Text),
    // `Primary`, `Secondary` and the elite keys: the weapon slots (S-0308 claim 1).
    ("turret_count", "TurretCount", Int),
    ("weapon_count", "WeaponCount", Int),
    // spec: S-0214 Import mapping (Teleporter)
    ("teleporter", "Teleporter", Bool),
];

/// Building rules fields (`glasnost_rules::sim::BuildingType`). `Factory`
/// is not converted: no approved spec gives the vocabulary of its value.
const BUILDING_FIELDS: &[(&str, &str, Form)] = &[
    ("strength", "Strength", Int),
    // spec: S-0311 F1 and Import mapping, S-0314 claim 5 and Import mapping (the cost field, an integer)
    ("cost", "Cost", Int),
    ("bib", "Bib", Bool),
    ("gate", "Gate", Bool),
    ("invisible_in_game", "InvisibleInGame", Bool),
    ("laser_fence", "LaserFence", Bool),
    ("firestorm_wall", "FirestormWall", Bool),
    ("naval", "Naval", Bool),
    ("bunker", "Bunker", Bool),
    ("unit_repair", "UnitRepair", Bool),
    ("grinding", "Grinding", Bool),
    ("unit_absorb", "UnitAbsorb", Bool),
    ("infantry_absorb", "InfantryAbsorb", Bool),
    ("bridge_repair_hut", "BridgeRepairHut", Bool),
    ("number_impassable_rows", "NumberImpassableRows", Int),
    ("weapons_factory", "WeaponsFactory", Bool),
    ("construction_yard", "ConstructionYard", Bool),
    ("undeploys_into", "UndeploysInto", Text),
    ("cloning", "Cloning", Bool),
    ("powered", "Powered", Bool),
    ("toggle_power", "TogglePower", Bool),
    // spec: S-0214 Import mapping (VoiceSelect: the selection responses, a list of sound names kept in the listed order)
    ("voice_select", "VoiceSelect", Texts),
    ("can_be_occupied", "CanBeOccupied", Bool),
    ("can_occupy_fire", "CanOccupyFire", Bool),
    ("capturable", "Capturable", Bool),
    ("needs_engineer", "NeedsEngineer", Bool),
    ("immune_to_radiation", "ImmuneToRadiation", Bool),
    ("storage", "Storage", Int),
    ("spyable", "Spyable", Bool),
    ("can_c4", "CanC4", Bool),
    ("repairable", "Repairable", Bool),
    ("hospital", "Hospital", Bool),
    ("armory", "Armory", Bool),
    ("sam", "SAM", Bool),
    ("emp_cannon", "EMPulseCannon", Bool),
    ("stupid_guard_mode", "HasStupidGuardMode", Bool),
    ("turret", "Turret", Bool),
    ("rot", "ROT", Int),
    // `Primary` and `Secondary`: the weapon slots (S-0308 claim 1).
    ("tech_level", "TechLevel", Int),
    ("gap_generator", "GapGenerator", Bool),
    ("immune", "Immune", Bool),
    ("legal_target", "LegalTarget", Bool),
    ("crushable", "Crushable", Bool),
    // spec: S-0215 Import mapping (DeployTime and GateCloseDelay, game minutes, S-0003 decimal form)
    ("deploy_time", "DeployTime", Dec),
    ("gate_close_delay", "GateCloseDelay", Dec),
];

/// Overlay rules fields other than `Land` (`glasnost_rules::sim::OverlayType`).
// spec: S-0211 Import mapping (overlay flags)
const OVERLAY_FIELDS: &[(&str, &str, Form)] = &[
    ("no_use_tile_land_type", "NoUseTileLandType", Bool),
    ("wall", "Wall", Bool),
    ("tiberium", "Tiberium", Bool),
    ("crushable", "Crushable", Bool),
    ("rock", "IsARock", Bool),
    ("rubble", "IsRubble", Bool),
    ("crate", "Crate", Bool),
    ("strength", "Strength", Int),
    ("armor", "Armor", Text),
    // spec: S-0202 Import mapping (IsVeins, the veins flag)
    ("is_veins", "IsVeins", Bool),
    // spec: S-0009 Import mapping (CellAnim: an animation type name, compared without regard to letter case; none and <none> clear it; an empty value keeps it)
    ("cell_anim", "CellAnim", Ref(Kind::Animations)),
];

/// Terrain-object fields (S-0211 Import mapping).
// spec: S-0211 Import mapping (occupation patterns)
const TERRAIN_OBJECT_FIELDS: &[(&str, &str, Form)] = &[
    ("temperate_occupation_bits", "TemperateOccupationBits", Int),
    ("snow_occupation_bits", "SnowOccupationBits", Int),
    // spec: S-0203 Import mapping (SpawnsTiberium)
    ("spawns_tiberium", "SpawnsTiberium", Bool),
    // spec: S-0401 Import mapping (IsAnimated, AnimationRate, AnimationProbability)
    ("is_animated", "IsAnimated", Bool),
    ("animation_rate", "AnimationRate", Int),
    ("animation_probability", "AnimationProbability", Dec),
];

/// Weapon fields.
const WEAPON_FIELDS: &[(&str, &str, Form)] = &[
    ("damage", "Damage", Int),
    ("ambient_damage", "AmbientDamage", Int),
    ("projectile", "Projectile", Text),
    ("warhead", "Warhead", Text),
    ("omni_fire", "OmniFire", Bool),
    ("area_fire", "AreaFire", Bool),
    ("rad_level", "RadLevel", Int),
];

/// Projectile fields.
const PROJECTILE_FIELDS: &[(&str, &str, Form)] = &[("rot", "ROT", Int), ("ag", "AG", Bool)];

/// Warhead fields.
const WARHEAD_FIELDS: &[(&str, &str, Form)] = &[
    ("wall", "Wall", Bool),
    ("wood", "Wood", Bool),
    ("ivan_bomb", "IvanBomb", Bool),
    ("sonic", "Sonic", Bool),
    ("fire", "Fire", Bool),
    ("electric_assault", "ElectricAssault", Bool),
    ("inf_death", "InfDeath", Int),
];

/// Country fields (S-0207, S-0208 Import mapping).
const COUNTRY_FIELDS: &[(&str, &str, Form)] = &[
    ("speed_units_mult", "SpeedUnitsMult", Dec),
    ("speed_infantry_mult", "SpeedInfantryMult", Dec),
    ("multiplay_passive", "MultiplayPassive", Bool),
    // spec: S-0210 Import mapping (the wall-owner key of a country)
    ("wall_owner", "WallOwner", Bool),
];

/// Game-wide fields: our name, section, key, form (the field docs of
/// `glasnost_rules::sim::General` and the Import mappings they cite).
const GENERAL_FIELDS: &[(&str, &[u8], &str, Form)] = &[
    // spec: S-0209 Import mapping
    ("close_enough", b"General", "CloseEnough", Dec),
    ("blockage_path_delay", b"AI", "BlockagePathDelay", Int),
    // spec: S-0212 Import mapping
    ("path_delay", b"AI", "PathDelay", Dec),
    // spec: S-0208, S-0209 Import mapping (the combat-damage section)
    ("player_scatter", COMBAT_DAMAGE, "PlayerScatter", Bool),
    // spec: S-0215 Import mapping (the combat-damage settings section)
    (
        "player_return_fire",
        COMBAT_DAMAGE,
        "PlayerReturnFire",
        Bool,
    ),
    // spec: S-0210 Import mapping
    ("player_auto_crush", COMBAT_DAMAGE, "PlayerAutoCrush", Bool),
    ("crush_range", COMBAT_DAMAGE, "Crush", Dec),
    // spec: S-0206, S-0215 Import mapping
    (
        "guard_area_targeting_delay",
        b"General",
        "GuardAreaTargetingDelay",
        Int,
    ),
    (
        "normal_targeting_delay",
        b"General",
        "NormalTargetingDelay",
        Int,
    ),
    ("guard_mode_stray", b"General", "GuardModeStray", Dec),
    // spec: S-0215 Import mapping (ApproachTargetResetMultiplier: m of F10, an integer)
    (
        "approach_target_reset_multiplier",
        b"General",
        "ApproachTargetResetMultiplier",
        Int,
    ),
    // spec: S-0207 Import mapping
    ("veteran_speed", b"General", "VeteranSpeed", Dec),
    ("tracked_uphill", b"General", "TrackedUphill", Dec),
    ("tracked_downhill", b"General", "TrackedDownhill", Dec),
    ("wheeled_uphill", b"General", "WheeledUphill", Dec),
    ("wheeled_downhill", b"General", "WheeledDownhill", Dec),
    // spec: S-0211 Import mapping
    (
        "cliff_back_impassability",
        b"General",
        "CliffBackImpassability",
        Int,
    ),
    // spec: S-0108, S-0107 Import mapping
    ("condition_yellow", b"AudioVisual", "ConditionYellow", Dec),
    ("condition_red", b"AudioVisual", "ConditionRed", Dec),
    (
        "idle_action_frequency",
        b"AudioVisual",
        "IdleActionFrequency",
        Dec,
    ),
    // spec: S-0213 Import mapping
    ("maximum_cheer_rate", b"General", "MaximumCheerRate", Int),
    // spec: S-0209, S-0215, S-0210 Import mapping ([IQ])
    ("iq_scatter", b"IQ", "Scatter", Int),
    ("iq_guard_area", b"IQ", "GuardArea", Int),
    ("iq_auto_crush", b"IQ", "AutoCrush", Int),
    ("iq_max", b"IQ", "MaxIQLevels", Int),
    // spec: S-0215 Import mapping (AIAutoDeployFrameDelay: a list of integers in ticks, for difficulty indexes 0, 1 and 2)
    (
        "ai_auto_deploy_frame_delay",
        b"General",
        "AIAutoDeployFrameDelay",
        Ints,
    ),
    // spec: S-0201 Import mapping
    (
        "lobby_game_speed",
        b"MultiplayerDialogSettings",
        "GameSpeed",
        Int,
    ),
    // spec: S-0214 Import mapping
    (
        "engineer_capture_level",
        b"General",
        "EngineerCaptureLevel",
        Dec,
    ),
];

/// Percentage fields among the game-wide fields (S-0003 Import mapping row
/// C9).
// spec: S-0003 Import mapping row C9 (ConditionYellow, ConditionRed; RepairPercent and CrewEscape in [General])
const PERCENTAGE_KEYS: [&str; 4] = [
    "ConditionYellow",
    "ConditionRed",
    "RepairPercent",
    "CrewEscape",
];

/// Entries of the auto-deploy delay list, one per difficulty index (S-0215
/// claim 31).
// spec: S-0215 claim 31 and Edge cases (one entry for each of the difficulty indexes 0, 1 and 2)
const AUTO_DEPLOY_ENTRIES: usize = 3;

/// Turn-rate fields of a turreted building for which the snap bound and the
/// turn rate agree (S-0215 Edge cases).
// spec: S-0215 Edge cases (fields from 0 to 127 are not affected)
const TURRET_SNAP_QUIET: std::ops::RangeInclusive<i32> = 0..=127;

/// Cliff-back settings the importer accepts without a warning (S-0211
/// Edge cases).
// spec: S-0211 Edge cases (the importer warns for any value other than 0, 1 and 2)
const CLIFF_BACK_QUIET: [i32; 3] = [0, 1, 2];

/// The speed field's range without a warning (S-0207 claim 1).
// spec: S-0207 claim 1 (warning above 100 or below −1)
const SPEED_QUIET: std::ops::RangeInclusive<i32> = -1..=100;

/// The class identifiers of the locomotors (S-0207 claim 14, table T3), as
/// 32 hexadecimal digits in upper case without braces or hyphens, with our
/// names (`glasnost_rules::sim::Locomotor`).
// spec: S-0207 table T3 (the eleven class identifiers)
const LOCOMOTORS: [(&str, &str); 11] = [
    ("4A582741983911D1B70900A024DDAFD1", "drive"),
    ("4A582742983911D1B70900A024DDAFD1", "hover"),
    ("4A582743983911D1B70900A024DDAFD1", "tunnel"),
    ("4A582744983911D1B70900A024DDAFD1", "walk"),
    ("4A582745983911D1B70900A024DDAFD1", "drop_pod"),
    ("4A582746983911D1B70900A024DDAFD1", "fly"),
    ("4A582747983911D1B70900A024DDAFD1", "teleport"),
    ("55D141B8DB9411D1AC98006008055BB5", "mech"),
    ("2BEA74E17CCA11D3BE1400104B62A16C", "ship"),
    ("92612C46F71F11D1AC9F006008055BB5", "jumpjet"),
    ("B7B49766E57611D39BD900104B972FE8", "rocket"),
];

/// The digits of a class identifier (S-0207 Terms): the value, with
/// leading and trailing blanks removed, is an opening brace, groups of 8,
/// 4, 4, 4 and 12 hexadecimal digits joined by hyphens, and a closing
/// brace. Returns the 32 digits in upper case, or `None` for any other
/// form.
pub fn class_identifier(v: &[u8]) -> Option<String> {
    // spec: S-0207 claim 14 (the value is read whole, after leading and trailing blanks are removed)
    let v = v.trim_ascii();
    let inner = v.strip_prefix(b"{")?.strip_suffix(b"}")?;
    let groups: Vec<&[u8]> = inner.split(|&b| b == b'-').collect();
    // spec: S-0207 Terms (five groups of 8, 4, 4, 4 and 12 hexadecimal digits)
    const LENGTHS: [usize; 5] = [8, 4, 4, 4, 12];
    if groups.len() != LENGTHS.len() {
        return None;
    }
    let mut out = String::new();
    for (g, n) in groups.iter().zip(LENGTHS) {
        // spec: S-0207 claim 14 (letters among the hexadecimal digits in either case)
        if g.len() != n || !g.iter().all(u8::is_ascii_hexdigit) {
            return None;
        }
        out.push_str(&String::from_utf8_lossy(g).to_ascii_uppercase());
    }
    Some(out)
}

/// What the builder reads.
pub struct SimInputs<'a> {
    /// The layered data.
    pub outcome: &'a Outcome,
    /// The art database.
    pub art: &'a Document,
    /// Each type's looked-up art, when a theater was given.
    pub arts: Option<&'a BTreeMap<(Kind, usize), TypeArt>>,
    /// Layering switches.
    pub quirks: &'a LayerQuirks,
    /// Art switches.
    pub art_quirks: &'a ArtQuirks,
    /// INI value-form switches (S-0003).
    pub ini_quirks: &'a IniQuirks,
    /// Switches of the typed simulation fields.
    pub sim_quirks: &'a SimQuirks,
}

/// Quirk switches of the importer's typed simulation fields. Each defaults
/// to the original's behaviour.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SimQuirks {
    /// `quirks.clear_weapons_any_turret_count` (S-0308 claim 1, Edge
    /// cases): the clear-weapons flag empties slots 0 and 1 and elite slots
    /// 0 and 1 whatever the turret count; off, only on types whose turret
    /// count is 0 or less.
    pub clear_weapons_any_turret_count: bool,
    /// `quirks.coefficient_zero_replaced` (S-0309 Edge cases): a held threat
    /// coefficient of 0 is replaced by the matching game-wide default when a
    /// later reading of the section lacks the key; off, an absent key always
    /// keeps the held value.
    pub coefficient_zero_replaced: bool,
    /// `quirks.tree_strength_fixed_at_first_section` (S-0102 claim 23,
    /// Edge cases): a terrain type that no layer gives a strength takes the
    /// game-wide tree strength in force when the first layer holding its
    /// section is read, and keeps it; off, the tree strength in force after
    /// the last layer.
    pub tree_strength_fixed_at_first_section: bool,
}

impl Default for SimQuirks {
    fn default() -> Self {
        // spec: S-0308 Edge cases, S-0309 Edge cases (both on by default: the original)
        Self {
            clear_weapons_any_turret_count: true,
            coefficient_zero_replaced: true,
            // spec: S-0102 Edge cases (quirks.tree_strength_fixed_at_first_section on by default: the original)
            tree_strength_fixed_at_first_section: true,
        }
    }
}

impl SimQuirks {
    /// The switches by name, for the import record.
    pub fn named(&self) -> [(&'static str, bool); 3] {
        [
            (
                "clear_weapons_any_turret_count",
                self.clear_weapons_any_turret_count,
            ),
            ("coefficient_zero_replaced", self.coefficient_zero_replaced),
            (
                "tree_strength_fixed_at_first_section",
                self.tree_strength_fixed_at_first_section,
            ),
        ]
    }
}

struct Builder<'a> {
    i: SimInputs<'a>,
    warnings: Vec<Warning>,
}

/// One record being built.
#[derive(Default)]
struct Rec(Vec<(String, Node)>);

impl Rec {
    fn put(&mut self, field: &str, node: Node) {
        self.0.push((field.to_owned(), node));
    }

    fn node(self) -> Node {
        jsonc::object(self.0)
    }
}

/// A key's value in force after its history (S-0006 claim 3): each set
/// value that `read` can read replaces the held value, one it cannot read
/// keeps it (S-0003 claims 2, 3 and 8), a "not given" value keeps it, and a
/// reset gives the engine default (`None`).
fn fold<T>(
    state: Option<&KeyState>,
    upto: Option<usize>,
    mut read: impl FnMut(&[u8], &Event) -> Option<T>,
) -> Option<T> {
    let mut held = None;
    for e in state.into_iter().flat_map(|s| &s.history) {
        if upto.is_some_and(|l| e.layer > l) {
            break;
        }
        match &e.what {
            What::Set(v) => {
                if let Some(x) = read(v, e) {
                    held = Some(x);
                }
            }
            What::Reset => held = None,
            _ => {}
        }
    }
    held
}

fn int_node(v: i32) -> Node {
    jsonc::integer(i128::from(v))
}

impl Builder<'_> {
    fn file(&self, layer: usize) -> String {
        self.i
            .outcome
            .files
            .get(layer)
            .map(|f| f.1.clone())
            .unwrap_or_default()
    }

    fn warn(&mut self, layer: usize, line: u32, message: String) {
        let w = Warning {
            file: self.file(layer),
            line,
            message,
        };
        // The same art section is read again at each type read: warn once.
        if !self.warnings.contains(&w) {
            self.warnings.push(w);
        }
    }

    fn art_warn(&mut self, line: u32, message: String) {
        let layer = self.i.outcome.files.len().saturating_sub(1);
        self.warn(layer, line, message);
    }

    /// A field's value in force, as a node.
    fn field(&mut self, state: Option<&KeyState>, form: Form, what: &str) -> Option<Node> {
        // spec: S-0003 claim 12 (quirks.list_items_untrimmed off: name items trimmed)
        let untrimmed = self.i.ini_quirks.list_items_untrimmed;
        match form {
            // spec: S-0003 claims 3 and 4 (integer forms; a hexadecimal form without digits keeps the value)
            Int => fold(state, None, |v, _| value::integer(v)).map(int_node),
            // spec: S-0003 claim 2 (an unrecognised first byte keeps the value)
            Bool => fold(state, None, |v, _| value::boolean(v)).map(jsonc::boolean),
            Dec => self.decimal(state, None, what).map(jsonc::string),
            // spec: S-0003 claim 1 (text cut to the common limit)
            Text => fold(state, None, |v, _| {
                Some(encode(value::text(v, value::COMMON_TEXT_LIMIT)))
            })
            .map(jsonc::string),
            // spec: S-0003 claims 1 and 14 (a type name; none and <none> name no type)
            Ref(kind) => fold(state, None, |v, _| Some(self.ref_name(kind, v)))
                .flatten()
                .map(jsonc::string),
            // spec: S-0003 claims 12, 14 and 19 (a list of type names; a given list replaces the held one; none and <none> items take no place, so a list of only those is empty)
            Refs(kind) => {
                let mut spaced = None;
                let list = fold(state, None, |v, e| {
                    let items = value::name_items(v, value::COMMON_TEXT_LIMIT, untrimmed);
                    if untrimmed && items.iter().any(|i| value::edge_whitespace(i)) {
                        spaced = Some((e.layer, e.line));
                    }
                    Some(
                        items
                            .into_iter()
                            .filter_map(|item| self.ref_name(kind, item))
                            .map(jsonc::string)
                            .collect::<Vec<_>>(),
                    )
                });
                // spec: S-0003 claim 12 (quirks.list_items_untrimmed on: the importer warns for a name item that begins or ends with whitespace)
                if let Some((layer, line)) = spaced {
                    self.warn(
                        layer,
                        line,
                        format!("{what}: a list item with whitespace at its start or end names a different type (S-0003 claim 12, quirks.list_items_untrimmed)"),
                    );
                }
                list.map(jsonc::array)
            }
            // spec: S-0003 claims 3 and 12 (a list of integers; an item the integer form cannot read is 0 and keeps its place: interim, request Q-0171)
            Ints => fold(state, None, |v, _| {
                Some(
                    value::list(v, value::COMMON_TEXT_LIMIT)
                        .into_iter()
                        .map(|item| int_node(value::integer(item).unwrap_or(0)))
                        .collect::<Vec<_>>(),
                )
            })
            .map(jsonc::array),
            // spec: S-0003 claim 20 (debris maxima: decimal prefixes, 511-byte limit; a given list replaces the held one)
            Maxima => fold(state, None, |v, _| {
                Some(
                    value::decimal_prefix_list(v)
                        .into_iter()
                        .map(int_node)
                        .collect::<Vec<_>>(),
                )
            })
            .map(jsonc::array),
            // spec: S-0003 claim 12 and S-0214 Import mapping (a list of names: every item kept in the listed order, none dropped or reordered)
            Texts => fold(state, None, |v, _| {
                Some(
                    value::name_items(v, value::COMMON_TEXT_LIMIT, untrimmed)
                        .into_iter()
                        .map(|item| {
                            jsonc::string(encode(value::text(item, value::COMMON_TEXT_LIMIT)))
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .map(jsonc::array),
        }
    }

    /// A type name (S-0003 claims 1 and 14): the key of the type of `kind`
    /// it names, compared by claim 14; a name that matches no type is kept
    /// as written (a field that S-0004 claim 13 does not list as creating,
    /// a long name that claim 8 duplicates, or the terrain objects and
    /// animations created after the rules layers, which are not imported).
    /// `none`, `<none>` and an empty name give `None`.
    fn ref_name(&self, kind: Kind, v: &[u8]) -> Option<String> {
        let name = value::text(v, value::COMMON_TEXT_LIMIT);
        if name.is_empty() || value::is_no_type(name) {
            return None;
        }
        Some(self.type_key(kind, name).unwrap_or_else(|| encode(name)))
    }

    /// A decimal key's text in force after layer `upto` (all layers when
    /// `None`), with S-0003's warnings.
    fn decimal(
        &mut self,
        state: Option<&KeyState>,
        upto: Option<usize>,
        what: &str,
    ) -> Option<String> {
        let mut notes = Vec::new();
        let out = fold(state, upto, |v, e| {
            match value::decimal(v, DecimalField::Double) {
                IniDecimal::Value { uncovered, .. } => {
                    if uncovered {
                        // spec: S-0003 Formulas (importer warning test for claim 6's uncovered cases)
                        notes.push((e.layer, e.line, format!(
                        "{what} {:?}: the original's result for this decimal is not characterised exactly (S-0003 claim 6)",
                        show(v)
                    )));
                    }
                    decimal_text(v)
                }
                IniDecimal::NoNumber => {
                    // spec: S-0003 claim 8 (no number: our engine keeps the value in force, with a warning)
                    notes.push((e.layer, e.line, format!(
                    "{what} {:?} has no number; the value in force is kept (S-0003 claim 8, a departure)",
                    show(v)
                )));
                    None
                }
            }
        });
        if upto.is_none() {
            for (l, line, m) in notes {
                self.warn(l, line, m);
            }
        }
        out
    }

    fn fields(&mut self, rec: &mut Rec, t: &TypeRec, kind: Kind, table: &[(&str, &str, Form)]) {
        for &(field, key, form) in table {
            let what = format!("{} {} {key}", kind.key(), show(&t.name));
            if let Some(n) = self.field(t.keys.get(key.as_bytes()), form, &what) {
                rec.put(field, n);
            }
        }
    }

    fn build(mut self) -> (Node, Vec<Warning>) {
        let mut root = Vec::new();
        let general = self.general();
        root.push(("general".to_owned(), general));
        let land = self.land_types();
        root.push(("land_types".to_owned(), land));
        let behaviours = self.behaviours();
        root.push(("behaviours".to_owned(), behaviours));
        for kind in [
            Kind::Countries,
            Kind::Infantry,
            Kind::Vehicles,
            Kind::Aircraft,
            Kind::Buildings,
            Kind::Overlays,
            Kind::TerrainObjects,
            Kind::Weapons,
            Kind::Projectiles,
            Kind::Warheads,
            Kind::Animations,
            Kind::Tiberiums,
            Kind::Superweapons,
        ] {
            let node = self.kind(kind);
            root.push((kind.key().to_owned(), node));
        }
        let difficulty = self.difficulty();
        root.push(("difficulty".to_owned(), difficulty));
        self.cross_checks();
        (jsonc::object(root), self.warnings)
    }

    fn general(&mut self) -> Node {
        let outcome = self.i.outcome;
        let globals = &outcome.globals;
        let mut rec = Rec::default();
        for &(field, section, key, form) in GENERAL_FIELDS {
            let state = globals.get(section).and_then(|s| s.get(key.as_bytes()));
            let what = format!("[{}] {key}", show(section));
            if let Some(n) = self.field(state, form, &what) {
                rec.put(field, n);
            } else if key == "GuardModeStray" {
                // spec: S-0215 claim 36 (b) and Questions for the project lead (decided: no layer gives the guard stray distance: the area-guard leash, and the importer warns)
                self.warn(0, 0, format!(
                    "[{}] {key} is given by no rules layer: the original's value is unspecified; our engine uses the area-guard leash instead (S-0215 claim 36, a departure)",
                    show(section)
                ));
            }
            let Some(state) = state else { continue };
            if key == "AIAutoDeployFrameDelay"
                && let Some(v) = &state.value
                && value::list(v, value::COMMON_TEXT_LIMIT).len() < AUTO_DEPLOY_ENTRIES
            {
                // spec: S-0215 claim 31 and Edge cases (the importer warns when the auto-deploy delay list has fewer than 3 entries)
                self.warn(0, 0, format!(
                    "[{}] {key} has fewer than {AUTO_DEPLOY_ENTRIES} entries: the original's delay for a missing difficulty index is unspecified; without a second entry the neutral and special houses' infantry never auto-deploys (S-0215 claim 31, a departure)",
                    show(section)
                ));
            }
            for e in &state.history {
                let What::Set(v) = &e.what else { continue };
                if key == "CliffBackImpassability"
                    && let Some(r) = value::integer(v)
                    && !CLIFF_BACK_QUIET.contains(&r)
                {
                    // spec: S-0211 Edge cases (cliff-back setting wraps; the importer warns)
                    self.warn(e.layer, e.line, format!(
                        "[General] CliffBackImpassability {r}: only its remainder modulo 256 matters (S-0211 claim 8)"
                    ));
                }
                if PERCENTAGE_KEYS.contains(&key)
                    && !v.contains(&b'%')
                    && let IniDecimal::Value { value, .. } = value::decimal(v, DecimalField::Double)
                    && magnitude_above_one(&value.to_exact_decimal())
                {
                    // spec: S-0003 claim 18 (a percentage field without `%` above 1 in magnitude)
                    self.warn(e.layer, e.line, format!(
                        "[{}] {key} {:?} has no `%`: it is taken as a fraction, not a percentage (S-0003 claim 18)",
                        show(section),
                        show(v)
                    ));
                }
            }
        }
        // spec: S-0009 Import mapping ([General] BaseUnit: a list of vehicle type names)
        let base_unit = globals
            .get(&b"General"[..])
            .and_then(|s| s.get(&b"BaseUnit"[..]))
            .cloned();
        if let Some(list) = self.name_list(base_unit.as_ref(), Kind::Vehicles, "[General] BaseUnit")
        {
            rec.put("base_unit", list);
        }
        // spec: S-0009 Terms and Import mapping (the game's tech level: the base rules' value only)
        let tech = globals
            .get(&b"MultiplayerDialogSettings"[..])
            .and_then(|s| s.get(&b"TechLevel"[..]));
        if let Some(v) = fold(tech, Some(0), |v, _| value::integer(v)) {
            rec.put("multiplayer_tech_level", int_node(v));
        }
        // spec: S-0009 claim 28 and Import mapping ([Maximums] Players: the maximum player count, from the base rules only, S-0006 claim 2)
        let players = globals
            .get(&b"Maximums"[..])
            .and_then(|s| s.get(&b"Players"[..]));
        if let Some(v) = fold(players, Some(0), |v, _| value::integer(v)) {
            rec.put("max_players", int_node(v));
        }
        // spec: S-0112 Import mapping (the gate and wall tower types of [General])
        for (field, key) in [
            ("allied_gate_one", "GDIGateOne"),
            ("allied_gate_two", "GDIGateTwo"),
            ("soviet_gate_one", "NodGateOne"),
            ("soviet_gate_two", "NodGateTwo"),
            ("wall_tower", "WallTower"),
        ] {
            let state = globals
                .get(&b"General"[..])
                .and_then(|s| s.get(key.as_bytes()));
            let name = fold(state, None, |v, _| {
                Some(value::text(v, value::COMMON_TEXT_LIMIT).to_vec())
            });
            if let Some(k) = name.and_then(|n| self.type_key(Kind::Buildings, &n)) {
                rec.put(field, jsonc::string(k));
            }
        }
        self.general_combat(&mut rec);
        self.general_econ(&mut rec);
        rec.node()
    }

    /// Land-type rows (S-0211 claim 2): a rules layer that contains a land
    /// type's section sets the whole row; the layering resets the keys it
    /// omits (S-0006 claim 5), so the folded values are the row.
    fn land_types(&mut self) -> Node {
        let mut rows = Vec::new();
        for (name, section) in LAND_TYPES {
            let keys = self.i.outcome.globals.get(section.as_bytes()).cloned();
            let layers: Vec<usize> = keys
                .iter()
                .flat_map(|k| k.values())
                .flat_map(|s| s.history.iter().map(|e| e.layer))
                .collect();
            if !layers.contains(&0) {
                // spec: S-0211 claim 2 (an importer warning for each land-type section the base rules lack)
                self.warn(0, 0, format!(
                    "the base rules lack the land-type section [{section}]; until a layer gives it, the land type has the start-up row (multipliers 0, not buildable; S-0211 claim 2)"
                ));
            }
            let (Some(keys), false) = (keys, layers.is_empty()) else {
                continue;
            };
            let mut rec = Rec::default();
            // spec: S-0211 claim 2 (a) and (c), S-0006 claim 5 (a missing key resets: the layering records the reset, `quirks.override_resets_keys`)
            for (field, key) in LAND_MULTIPLIERS {
                let state = keys.get(key.as_bytes());
                let what = format!("[{section}] {key}");
                // spec: S-0211 claim 2 (a) (the text; the cap at 1.0 is the simulation's)
                if let Some(text) = self.decimal(state, None, &what) {
                    rec.put(field, jsonc::string(text.clone()));
                    if text.starts_with('-') && !is_zero(text.as_bytes()) {
                        // spec: S-0211 Edge cases (the importer warns for any negative land multiplier)
                        let (layer, line) = state
                            .and_then(|s| {
                                s.history
                                    .iter()
                                    .rev()
                                    .find(|e| matches!(e.what, What::Set(_)))
                            })
                            .map_or((0, 0), |e| (e.layer, e.line));
                        self.warn(layer, line, format!(
                            "[{section}] {key} is negative ({text}): it allows entry and can leave vehicles stuck (S-0211 Edge cases)"
                        ));
                    }
                }
            }
            if let Some(b) = fold(keys.get(&b"Buildable"[..]), None, |v, _| value::boolean(v)) {
                // spec: S-0211 claim 2 (c) (the buildable key; missing gives no)
                rec.put("buildable", jsonc::boolean(b));
            }
            rows.push((name.to_owned(), rec.node()));
        }
        jsonc::object(rows)
    }

    /// Behaviour properties (S-0215 Import mapping, claim 21).
    fn behaviours(&mut self) -> Node {
        let mut out = Vec::new();
        for (name, section) in BEHAVIOURS {
            let Some(keys) = self.i.outcome.globals.get(section.as_bytes()).cloned() else {
                continue;
            };
            let mut rec = Rec::default();
            for (field, key) in BEHAVIOUR_FLAGS {
                if let Some(b) = fold(keys.get(key.as_bytes()), None, |v, _| value::boolean(v)) {
                    rec.put(field, jsonc::boolean(b));
                }
            }
            let rate_state = keys.get(&b"Rate"[..]);
            let what = format!("[{section}] Rate");
            if let Some(r) = self.decimal(rate_state, None, &what) {
                rec.put("rate", jsonc::string(r));
            }
            if let Some(aa) = self.aa_rate(section, &keys) {
                rec.put("aa_rate", jsonc::string(aa));
            }
            out.push((name.to_owned(), rec.node()));
        }
        jsonc::object(out)
    }

    /// The anti-air rate after every layer (S-0215 claim 21). `None` means
    /// the rate (the simulation's reading of an absent anti-air rate).
    fn aa_rate(&mut self, section: &str, keys: &BTreeMap<Vec<u8>, KeyState>) -> Option<String> {
        let rate_state = keys.get(&b"Rate"[..]);
        let aa_state = keys.get(&b"AARate"[..]);
        let mut layers: Vec<usize> = keys
            .values()
            .flat_map(|s| s.history.iter().map(|e| e.layer))
            .collect();
        layers.sort_unstable();
        layers.dedup();
        let what = format!("[{section}] AARate");
        let mut held: Option<String> = None;
        for l in layers {
            let given = aa_state.and_then(|s| {
                s.history.iter().rev().find_map(|e| match &e.what {
                    What::Set(v) if e.layer == l => Some((v.clone(), e.line)),
                    _ => None,
                })
            });
            match given {
                Some((v, line)) if !is_zero(&v) => match decimal_text(&v) {
                    Some(t) => held = Some(t),
                    None => {
                        // spec: S-0003 claim 8 (no number: our engine keeps the value in force, with a warning)
                        self.warn(l, line, format!(
                            "{what} {:?} has no number; the value in force is kept (S-0003 claim 8, a departure)",
                            show(&v)
                        ));
                    }
                },
                // spec: S-0215 claim 21 (an explicit 0 becomes the rate after that layer)
                Some(_) if self.i.quirks.anti_air_rate_layer_reset => {
                    held = self.decimal(rate_state, Some(l), &what);
                }
                // spec: S-0215 Edge cases (switch off: an anti-air rate given as 0 uses the rate)
                Some(_) => held = None,
                // spec: S-0215 claim 21 (a layer with the section but no anti-air rate: the rate after that layer)
                None if self.i.quirks.anti_air_rate_layer_reset => {
                    held = self.decimal(rate_state, Some(l), &what);
                }
                // spec: S-0215 Edge cases (switch off: an absent anti-air rate keeps the held value)
                None => {}
            }
        }
        held
    }

    fn kind(&mut self, kind: Kind) -> Node {
        let outcome = self.i.outcome;
        let reg = &outcome.registries[&kind];
        let keys = type_keys(reg);
        let mut out = Vec::new();
        for (i, (t, key)) in reg.iter().zip(&keys).enumerate() {
            let mut rec = Rec::default();
            match kind {
                Kind::Countries => {
                    self.fields(&mut rec, t, kind, COUNTRY_FIELDS);
                    self.country_combat(&mut rec, t);
                    self.country_econ(&mut rec, t);
                    // spec: S-0009 claim 33 (the early phase sees the base rules' passive field only)
                    let base = fold(t.keys.get(&b"MultiplayPassive"[..]), Some(0), |v, _| {
                        value::boolean(v)
                    });
                    if let Some(b) = base {
                        rec.put("multiplay_passive_base", jsonc::boolean(b));
                    }
                }
                Kind::Infantry | Kind::Vehicles | Kind::Aircraft => self.unit(&mut rec, kind, i, t),
                Kind::Buildings => {
                    self.fields(&mut rec, t, kind, BUILDING_FIELDS);
                    self.turret_snap_warning(t);
                    self.foundation(&mut rec, kind, i, t);
                    self.building_combat(&mut rec, i, t);
                    self.type_econ(&mut rec, kind, i, t);
                }
                Kind::Overlays => {
                    self.overlay(&mut rec, i, t);
                    self.overlay_combat(&mut rec, t);
                }
                Kind::TerrainObjects => {
                    self.fields(&mut rec, t, kind, TERRAIN_OBJECT_FIELDS);
                    self.terrain_combat(&mut rec, t);
                }
                Kind::Weapons => {
                    self.fields(&mut rec, t, kind, WEAPON_FIELDS);
                    self.weapon_combat(&mut rec, t);
                }
                Kind::Projectiles => {
                    self.fields(&mut rec, t, kind, PROJECTILE_FIELDS);
                    self.projectile_combat(&mut rec, i, t);
                }
                Kind::Warheads => {
                    self.fields(&mut rec, t, kind, WARHEAD_FIELDS);
                    self.warhead_combat(&mut rec, t);
                }
                Kind::Animations => self.animation(&mut rec, t),
                Kind::Tiberiums => self.tiberium(&mut rec, t),
                Kind::Superweapons => self.superweapon(&mut rec, t),
                _ => {}
            }
            // A weapon, projectile, warhead or animation that read no section still exists with the
            // engine defaults of its kind (S-0004 claims 6 and 13), and other records name it.
            let always = matches!(
                kind,
                Kind::Weapons | Kind::Projectiles | Kind::Warheads | Kind::Animations
            );
            if always || !rec.0.is_empty() {
                out.push((key.clone(), rec.node()));
            }
        }
        jsonc::object(out)
    }

    fn unit(&mut self, rec: &mut Rec, kind: Kind, index: usize, t: &TypeRec) {
        self.speed(rec, kind, t);
        self.fields(rec, t, kind, UNIT_FIELDS);
        self.locomotor(rec, kind, t);
        self.locomotion_class(rec, kind, t);
        self.movement_zone(rec, kind, t);
        if kind == Kind::Vehicles {
            self.restricted(rec, t);
        }
        for (field, key) in [
            ("veteran_abilities", "VeteranAbilities"),
            ("elite_abilities", "EliteAbilities"),
        ] {
            // spec: S-0003 claim 12 (list: split at commas, empty items skipped; a given list replaces the old one)
            let untrimmed = self.i.ini_quirks.list_items_untrimmed;
            // spec: S-0311 Import mapping and S-0003 claim 12 (ability items compared without trimming while quirks.list_items_untrimmed is on)
            let list = fold(t.keys.get(key.as_bytes()), None, |v, _| {
                Some(
                    value::name_items(v, value::COMMON_TEXT_LIMIT, untrimmed)
                        .into_iter()
                        .map(|item| jsonc::string(encode(&item.to_ascii_uppercase())))
                        .collect::<Vec<_>>(),
                )
            });
            if let Some(list) = list.filter(|l| !l.is_empty()) {
                rec.put(field, jsonc::array(list));
            }
        }
        // spec: S-0009 Import mapping (Owner: the owner list of country names; S-0004 claim 3: unknown names are ignored)
        if let Some(list) = self.name_list(t.keys.get(&b"Owner"[..]), Kind::Countries, "Owner") {
            rec.put("owner", list);
        }
        if kind == Kind::Infantry {
            self.infantry_art(rec, kind, index, t);
        }
        self.unit_combat(rec, kind, index, t);
        self.type_econ(rec, kind, index, t);
    }

    /// A list of type names after every layer (S-0003 claim 12: split at
    /// commas, empty items skipped, items not trimmed; a given list replaces
    /// the held one), each item resolved to a type of `kind` by S-0003 claim
    /// 14's comparison and written as that type's key. Items that name no
    /// type are dropped with a warning (S-0004 claim 3; S-0003 claim 12).
    fn name_list(&mut self, state: Option<&KeyState>, kind: Kind, what: &str) -> Option<Node> {
        let mut dropped = Vec::new();
        let untrimmed = self.i.ini_quirks.list_items_untrimmed;
        let list = fold(state, None, |v, e| {
            Some(
                // spec: S-0003 claim 12 (quirks.list_items_untrimmed off: name items trimmed)
                value::name_items(v, value::COMMON_TEXT_LIMIT, untrimmed)
                    .into_iter()
                    .filter_map(|item| {
                        let key = self.type_key(kind, item);
                        if key.is_none() && !value::is_no_type(item) {
                            dropped.push((e.layer, e.line, show(item)));
                        }
                        key
                    })
                    .map(jsonc::string)
                    .collect::<Vec<_>>(),
            )
        });
        for (l, line, item) in dropped {
            // spec: S-0003 claim 12 (the importer warns for every dropped item other than none and <none>)
            self.warn(
                l,
                line,
                format!("{what}: {item:?} names no {} type; dropped", kind.key()),
            );
        }
        list.map(jsonc::array)
    }

    /// The key of the type of `kind` that `name` names (S-0003 claim 14:
    /// ASCII case folding), if any.
    fn type_key(&self, kind: Kind, name: &[u8]) -> Option<String> {
        let reg = self.i.outcome.registries.get(&kind)?;
        let i = reg
            .iter()
            .position(|t| value::type_name_eq(&t.name, name))?;
        type_keys(reg).get(i).cloned()
    }

    /// The locomotor field (S-0207 claim 14): a class identifier of table
    /// T3 replaces the locomotor in force; a value of another form, and (our
    /// departure) an identifier not in T3, keep it, with a warning.
    fn locomotor(&mut self, rec: &mut Rec, kind: Kind, t: &TypeRec) {
        let mut notes = Vec::new();
        let loco = fold(t.keys.get(&b"Locomotor"[..]), None, |v, e| {
            let Some(id) = class_identifier(v) else {
                // spec: S-0207 claim 14 (not a class identifier: ignored, the locomotor in force is kept; our importer warns)
                notes.push((e.layer, e.line, format!(
                    "{} {} Locomotor {:?} is not a class identifier; the locomotor in force is kept (S-0207 claim 14)",
                    kind.key(),
                    show(&t.name),
                    show(v)
                )));
                return None;
            };
            let known = LOCOMOTORS.iter().find(|l| l.0 == id).map(|l| l.1);
            if known.is_none() {
                // spec: S-0207 claim 14 (an unknown class identifier: treated as not given, with a warning; a departure)
                notes.push((e.layer, e.line, format!(
                    "{} {} Locomotor {:?} names no known locomotor; the original stops with a program error when such an object is created, our engine keeps the locomotor in force (S-0207 claim 14, a departure)",
                    kind.key(),
                    show(&t.name),
                    show(v)
                )));
            }
            known
        });
        for (l, line, m) in notes {
            self.warn(l, line, m);
        }
        if let Some(name) = loco {
            rec.put("locomotor", jsonc::string(name));
        }
    }

    /// The turret snap bound of a turreted building reads its turn-rate field
    /// modulo 256 while the turn rate clamps it (S-0215 Edge cases, F8): the
    /// importer warns for a turreted building type whose field lies outside
    /// 0 to 127, where the two rules differ.
    fn turret_snap_warning(&mut self, t: &TypeRec) {
        let turret = fold(t.keys.get(&b"Turret"[..]), None, |v, _| value::boolean(v));
        let rot = fold(t.keys.get(&b"ROT"[..]), None, |v, _| value::integer(v));
        if turret == Some(true)
            && let Some(r) = rot
            && !TURRET_SNAP_QUIET.contains(&r)
        {
            // spec: S-0215 Edge cases (the importer warns for a turreted building type whose turn-rate field is outside 0 to 127)
            self.warn(0, 0, format!(
                "buildings {} ROT {r} is outside 0 to 127: its turret snap bound reads the field modulo 256 while its turn rate clamps it (S-0215 F8, quirks.turret_snap_bound_wraps)",
                show(&t.name)
            ));
        }
    }

    /// The speed field (S-0207 claim 1, S-0208 Import mapping): written as
    /// read; the simulation clamps it.
    fn speed(&mut self, rec: &mut Rec, kind: Kind, t: &TypeRec) {
        let state = t.keys.get(&b"Speed"[..]);
        if let Some(v) = fold(state, None, |v, _| value::integer(v)) {
            rec.put("speed", int_node(v));
        }
        if !matches!(kind, Kind::Vehicles | Kind::Infantry) {
            return;
        }
        for e in state.into_iter().flat_map(|s| &s.history) {
            if let What::Set(v) = &e.what
                && let Some(n) = value::integer(v)
                && !SPEED_QUIET.contains(&n)
            {
                // spec: S-0207 claim 1 (warning for a speed above 100 or below −1)
                self.warn(e.layer, e.line, format!(
                    "{} {} Speed {n} is outside −1 to 100; the original clamps it silently (S-0207 claim 1)",
                    kind.key(),
                    show(&t.name)
                ));
            }
        }
    }

    /// The locomotion class as S-0211 claim 3 leaves it, with our engine's
    /// departure for a name that names no class.
    fn locomotion_class(&mut self, rec: &mut Rec, kind: Kind, t: &TypeRec) {
        // spec: S-0211 claim 3 (before the first reading: infantry foot, aircraft winged, vehicles no class)
        let mut class: Option<usize> = match kind {
            Kind::Infantry => Some(0),
            Kind::Aircraft => Some(4),
            _ => None,
        };
        let state = t.keys.get(&b"SpeedType"[..]);
        for &r in &t.reads {
            let given = state.and_then(|s| {
                s.history.iter().rev().find_map(|e| match &e.what {
                    What::Set(v) if e.layer == r => Some((v.clone(), e.line)),
                    _ => None,
                })
            });
            let names: Vec<&str> = LOCOMOTION_CLASSES.iter().map(|c| c.1).collect();
            let named = given.as_ref().and_then(|(v, line)| {
                let n = value::named(v, NAME_LIMIT, &names);
                if n.is_none() {
                    // spec: S-0211 claim 3 (departure: a value that names no class is treated as absent, with a warning)
                    self.warn(r, *line, format!(
                        "{} {} SpeedType {:?} names no locomotion class; treated as absent (S-0211 claim 3, a departure)",
                        kind.key(),
                        show(&t.name),
                        show(v)
                    ));
                }
                n
            });
            match named {
                // spec: S-0211 claim 3 (a key that names a class gives that class)
                Some(c) => class = Some(c),
                // spec: S-0211 claim 3 (absent and no class: a vehicle gets track with the crusher flag, wheel otherwise)
                None if class.is_none() && kind == Kind::Vehicles => {
                    class = Some(if bool_as_of(t, "Crusher", r, false) {
                        1
                    } else {
                        2
                    });
                }
                None => {}
            }
        }
        if let Some(c) = class {
            rec.put("locomotion_class", jsonc::string(LOCOMOTION_CLASSES[c].0));
        }
    }

    fn movement_zone(&mut self, rec: &mut Rec, kind: Kind, t: &TypeRec) {
        let names: Vec<&str> = MOVEMENT_ZONES.iter().map(|z| z.1).collect();
        let mut notes = Vec::new();
        let zone = fold(t.keys.get(&b"MovementZone"[..]), None, |v, e| {
            let z = value::named(v, NAME_LIMIT, &names);
            if z.is_none() {
                // spec: S-0211 claim 4 (departure: an unrecognised zone keeps the held zone, with a warning)
                notes.push((e.layer, e.line, format!(
                    "{} {} MovementZone {:?} is not a movement zone; the zone in force is kept (S-0211 claim 4, a departure)",
                    kind.key(),
                    show(&t.name),
                    show(v)
                )));
            }
            z
        });
        for (l, line, m) in notes {
            self.warn(l, line, m);
        }
        if let Some(z) = zone {
            rec.put("movement_zone", jsonc::string(MOVEMENT_ZONES[z].0));
        }
    }

    /// A land-type name (S-0211 claim 4): `Some(index)` or `None` for none.
    fn land_name(v: &[u8]) -> Option<usize> {
        let names: Vec<&str> = LAND_TYPES.iter().map(|l| l.1).collect();
        value::named(v, NAME_LIMIT, &names)
    }

    fn restricted(&mut self, rec: &mut Rec, t: &TypeRec) {
        let state = t.keys.get(&b"MovementRestrictedTo"[..]);
        let mut held: Option<usize> = None;
        for e in state.into_iter().flat_map(|s| &s.history) {
            let What::Set(v) = &e.what else { continue };
            // spec: S-0211 claim 4 (`<none>` or any name not in the list gives none: no restriction)
            held = Self::land_name(v);
            if held.is_none() && !value::text(v, NAME_LIMIT).eq_ignore_ascii_case(b"<none>") {
                self.warn(e.layer, e.line, format!(
                    "vehicles {} MovementRestrictedTo {:?} is not a land type: the vehicle is unrestricted (S-0211 claim 4)",
                    show(&t.name),
                    show(v)
                ));
            }
        }
        if let Some(l) = held {
            rec.put("movement_restricted_to", jsonc::string(LAND_TYPES[l].0));
        }
    }

    fn overlay(&mut self, rec: &mut Rec, index: usize, t: &TypeRec) {
        self.fields(rec, t, Kind::Overlays, OVERLAY_FIELDS);
        // spec: S-0210 claim 13 (the damage-stage count from the image section of the art data, at each type read; the rules section's key is not read)
        let mut levels = None;
        for image in self.images(Kind::Overlays, index, t) {
            if let Some(v) = self
                .art_section(&image)
                .and_then(|sec| sec.value(b"DamageLevels").and_then(value::integer))
            {
                levels = Some(v);
            }
        }
        if let Some(v) = levels {
            rec.put("damage_levels", int_node(v));
        }
        // spec: S-0211 Import mapping (overlay land type absent: clear)
        let clear = 0;
        let tiberium = 5; // spec: S-0211 Terms (land type 5 tiberium)
        let mut land: Option<usize> = None;
        let state = t.keys.get(&b"Land"[..]);
        for &r in &t.reads {
            for e in state.into_iter().flat_map(|s| &s.history) {
                let What::Set(v) = &e.what else { continue };
                if e.layer != r {
                    continue;
                }
                land = Some(Self::land_name(v).unwrap_or_else(|| {
                    // spec: S-0211 claim 4 (departure: an overlay's land type none is read as clear, with a warning)
                    self.warn(e.layer, e.line, format!(
                        "overlays {} Land {:?} gives no land type; read as clear (S-0211 claim 4, a departure)",
                        show(&t.name),
                        show(v)
                    ));
                    clear
                }));
            }
            // spec: S-0211 claim 5 (an ore or gem overlay type whose land type is clear is given tiberium when its section is read)
            if land.unwrap_or(clear) == clear && bool_as_of(t, "Tiberium", r, false) {
                land = Some(tiberium);
            }
        }
        if let Some(l) = land {
            rec.put("land", jsonc::string(LAND_TYPES[l].0));
        }
        if let Some(arts) = self.i.arts {
            // spec: S-0009 claim 12 and Import mapping (the overlay type has artwork or a cell animation; the test reads the same CellAnim value)
            let cell_anim = self
                .field(
                    t.keys.get(&b"CellAnim"[..]),
                    Ref(Kind::Animations),
                    "CellAnim",
                )
                .is_some();
            let has = cell_anim
                || arts
                    .get(&(Kind::Overlays, index))
                    .is_some_and(|a| a.sprite.is_some());
            rec.put("has_art", jsonc::boolean(has));
        }
    }

    /// An animation type's fields that its creation draws read (S-0109
    /// Import mapping; S-0202 claim 10), from its section of the art
    /// database: the random rate pair, each end keeping the value held when
    /// it is −1 (S-0109 claim 3), the bouncing flags, the resource chain
    /// reaction and the middle-event effects.
    fn animation(&mut self, rec: &mut Rec, t: &TypeRec) {
        let Some(sec) = self.art_section(&t.name) else {
            return;
        };
        if let Some(v) = sec.value(b"RandomRate") {
            // spec: S-0109 claim 3 and Import mapping (two integers; −1 in either place keeps that end)
            let read = value::int_tuple::<2>(v);
            let pair = read.map(|e| e.filter(|&x| x != -1).unwrap_or(0));
            rec.put(
                "random_rate",
                jsonc::array(pair.iter().map(|&x| int_node(x)).collect()),
            );
        }
        for (field, key) in [
            ("bouncer", "Bouncer"),
            ("is_meteor", "IsMeteor"),
            ("tiberium_chain_reaction", "TiberiumChainReaction"),
            ("scorch", "Scorch"),
            ("crater", "Crater"),
        ] {
            // spec: S-0109 Import mapping (booleans; absent: no)
            if let Some(b) = sec.value(key.as_bytes()).and_then(value::boolean) {
                rec.put(field, jsonc::boolean(b));
            }
        }
        if let Some(v) = sec.value(b"SpawnsParticle") {
            // spec: S-0109 Import mapping (SpawnsParticle: a particle type name; none: no particle)
            let name = value::text(v, value::COMMON_TEXT_LIMIT);
            if !name.is_empty() && !value::is_no_type(name) {
                rec.put("spawns_particle", jsonc::boolean(true));
            }
        }
    }

    /// The image name after each type read (S-0007 claim 1: the image key
    /// replaces it, cut to 24 bytes). With the art lookup's result, the last
    /// read uses its image (which carries the arctic suffix, S-0007 claims 9
    /// and 10).
    fn images(&self, kind: Kind, index: usize, t: &TypeRec) -> Vec<Vec<u8>> {
        let mut image = t.name.clone();
        let mut out: Vec<Vec<u8>> = t
            .reads
            .iter()
            .map(|&r| {
                if let Some(v) = given_at(t, "Image", r) {
                    // spec: S-0007 claim 1 (the image key replaces the image name, cut to 24 bytes)
                    image = value::text(v, IMAGE_MAX).to_vec();
                }
                image.clone()
            })
            .collect();
        if let (Some(last), Some(a)) = (
            out.last_mut(),
            self.i.arts.and_then(|a| a.get(&(kind, index))),
        ) {
            last.clone_from(&a.image);
        }
        out
    }

    fn art_section(&self, name: &[u8]) -> Option<Section> {
        self.i.art.section(name).cloned()
    }

    /// Art fields of an infantry type (S-0107 Import mapping), from its art
    /// section at each type read: a key the section gives replaces the held
    /// value, an absent one keeps it.
    fn infantry_art(&mut self, rec: &mut Rec, kind: Kind, index: usize, t: &TypeRec) {
        let mut crawls = None;
        let mut fire: [Option<i32>; 4] = [None; 4];
        let mut sequences: Option<Vec<(String, Node)>> = None;
        // The die-1 and die-2 frame counts of the table in force (claim 1: all zero without one).
        let mut die_counts = [0i32; 2];
        let mut sequence_line = 0u32;
        for image in self.images(kind, index, t) {
            let Some(sec) = self.art_section(&image) else {
                continue;
            };
            // spec: S-0107 Import mapping (Crawls, a boolean of the type's art section)
            if let Some(b) = sec.value(b"Crawls").and_then(value::boolean) {
                crawls = Some(b);
            }
            for (slot, key) in fire.iter_mut().zip(FIRE_FRAME_KEYS) {
                // spec: S-0107 Import mapping (fire frames: integers of the type's art section)
                if let Some(v) = sec.value(key.as_bytes()).and_then(value::integer) {
                    *slot = Some(v);
                }
            }
            if let Some(entry) = sec.get(b"Sequence") {
                let (v, line) = (entry.value.clone(), entry.line);
                let name = self.cut(&v, line, "Sequence");
                let (table, die) = self.sequence_table(&name, t);
                sequences = Some(table);
                die_counts = die;
                sequence_line = line;
            }
        }
        if die_counts.contains(&0) {
            // spec: S-0107 claim 17 and Edge cases (the importer warns about an infantry type whose die-1 or die-2 entry has no image frames, whatever quirks.refused_death_keeps_unit says)
            self.art_warn(sequence_line, format!(
                "infantry {}: its die-1 or die-2 sequence entry has no image frames, so a death of that kind is turned down (S-0107 claim 17, quirks.refused_death_keeps_unit)",
                show(&t.name)
            ));
        }
        if let Some(b) = crawls {
            rec.put("crawls", jsonc::boolean(b));
        }
        if let Some(seq) = sequences.filter(|s| !s.is_empty()) {
            rec.put("sequences", jsonc::object(seq));
        }
        if fire.iter().any(Option::is_some) {
            // spec: S-0107 Import mapping (an absent fire frame is 0)
            rec.put(
                "fire_frames",
                jsonc::array(fire.iter().map(|f| int_node(f.unwrap_or(0))).collect()),
            );
        }
    }

    /// A sequence value cut as S-0107 claim 22 says, with the importer
    /// warning for a longer value.
    fn cut(&mut self, v: &[u8], line: u32, key: &str) -> Vec<u8> {
        if !self.i.art_quirks.sequence_value_length_limit {
            return value::text(v, usize::MAX).to_vec();
        }
        if v.len() > SEQUENCE_VALUE_MAX {
            // spec: S-0107 Import mapping (warning for a value longer than 31 characters)
            self.art_warn(line, format!(
                "{key} value {:?} is longer than 31 characters; the rest is not read (S-0107 claim 22)",
                show(v)
            ));
        }
        // spec: S-0107 claim 22 (each value is read as at most 31 characters)
        value::text(v, SEQUENCE_VALUE_MAX).to_vec()
    }

    /// The entries of a sequence section (S-0107 Import mapping), with the
    /// frame counts of the die-1 and die-2 entries (0 when absent, claim 1).
    fn sequence_table(&mut self, name: &[u8], t: &TypeRec) -> (Vec<(String, Node)>, [i32; 2]) {
        let mut die = [0i32; 2];
        let Some(sec) = self.art_section(name) else {
            return (Vec::new(), die);
        };
        let mut out = Vec::new();
        for (kind, key) in SEQUENCE_KINDS {
            let Some(entry) = sec.get(key.as_bytes()) else {
                continue;
            };
            let (v, line) = (entry.value.clone(), entry.line);
            let text = self.cut(&v, line, key);
            let (e, problem) =
                parse_sequence_entry(&text, self.i.art_quirks.case_sensitive_direction);
            if let Some(p) = problem {
                // spec: S-0107 Import mapping (warning for a missing or unparseable field, or an inexact direction)
                self.art_warn(
                    line,
                    format!(
                        "infantry {} sequence {} {key}={:?}: {p} (S-0107 Import mapping)",
                        show(&t.name),
                        show(name),
                        show(&v)
                    ),
                );
            }
            match kind {
                "die_1" => die[0] = e.0[1],
                "die_2" => die[1] = e.0[1],
                _ => {}
            }
            let mut members = vec![
                ("start".to_owned(), int_node(e.0[0])),
                ("count".to_owned(), int_node(e.0[1])),
                ("multiplier".to_owned(), int_node(e.0[2])),
            ];
            if let Some(d) = e.1 {
                members.push(("direction".to_owned(), jsonc::integer(i128::from(d))));
            }
            out.push((kind.to_owned(), jsonc::object(members)));
        }
        (out, die)
    }

    /// A building's foundation (S-0108 Import mapping): at each type read
    /// the image name's art section gives the size; the type id's section
    /// replaces it only with a size other than 1 × 1.
    fn foundation(&mut self, rec: &mut Rec, kind: Kind, index: usize, t: &TypeRec) {
        let mut held: Option<([i32; 2], bool)> = None;
        for image in self.images(kind, index, t) {
            if let Some(entry) = self
                .art_section(&image)
                .and_then(|s| s.get(b"Foundation").cloned())
            {
                let size = self.foundation_word(&entry.value, entry.line);
                held = Some((size, Self::refinery_word(&entry.value)));
            }
            if let Some(entry) = self
                .art_section(&t.name)
                .and_then(|s| s.get(b"Foundation").cloned())
            {
                let size = self.foundation_word(&entry.value, entry.line);
                // spec: S-0108 Import mapping (the id's section decides only with a size other than 1x1)
                if size != FOUNDATION_FALLBACK {
                    held = Some((size, Self::refinery_word(&entry.value)));
                }
            }
        }
        if let Some(([w, h], refinery)) = held {
            rec.put("foundation", jsonc::array(vec![int_node(w), int_node(h)]));
            if refinery {
                // spec: S-0406 claim 2 and Import mapping (the word 3x3Refinery is the 8-cell 3 × 3 word)
                rec.put("foundation_refinery", jsonc::boolean(true));
            }
        }
    }

    /// Whether a foundation value names the 8-cell 3 × 3 word (S-0406
    /// Import mapping), compared as S-0108 compares the words.
    fn refinery_word(v: &[u8]) -> bool {
        let words: Vec<&str> = FOUNDATIONS.iter().map(|f| f.0).collect();
        value::named(v, FOUNDATION_MAX, &words).is_some_and(|i| FOUNDATIONS[i].0 == REFINERY_WORD)
    }

    fn foundation_word(&mut self, v: &[u8], line: u32) -> [i32; 2] {
        let words: Vec<&str> = FOUNDATIONS.iter().map(|f| f.0).collect();
        // spec: S-0108 Import mapping (named value, text cut to 31, compared ignoring case)
        match value::named(v, FOUNDATION_MAX, &words) {
            Some(i) => FOUNDATIONS[i].1,
            None => {
                // spec: S-0003 claim 16 (the importer warns for a word that matches nothing, naming the fallback)
                self.art_warn(line, format!(
                    "Foundation {:?} is not a foundation word; it gives 1x1 (S-0108 Import mapping)",
                    show(v)
                ));
                FOUNDATION_FALLBACK
            }
        }
    }
}

/// Whether an exact decimal expansion is above 1 in magnitude.
fn magnitude_above_one(exact: &str) -> bool {
    let m = exact.trim_start_matches('-');
    let (int, frac) = m.split_once('.').unwrap_or((m, ""));
    let int = int.trim_start_matches('0');
    !int.is_empty() && (int != "1" || frac.bytes().any(|b| b != b'0'))
}

/// A parsed sequence entry: S, C, M and the declared direction.
type SequenceValues = ([i32; 3], Option<u8>);

/// One sequence entry `start,count,multiplier,direction` (S-0107 Import
/// mapping): when a field is missing or does not parse, it and every later
/// field keep their defaults (0, 0, 0, none). An integer field parses when
/// it starts, after optional whitespace and sign, with a digit (S-0003
/// claim 3's decimal form). Returns the values and the problem to warn
/// about, if any.
pub fn parse_sequence_entry(text: &[u8], case_sensitive: bool) -> (SequenceValues, Option<String>) {
    let mut out = [0i32; 3];
    let mut fields = text.split(|&b| b == b',');
    let names = ["start", "count", "multiplier"];
    for (slot, name) in out.iter_mut().zip(names) {
        let Some(f) = fields.next() else {
            return ((out, None), Some(format!("the {name} field is missing")));
        };
        let digits = f
            .iter()
            .skip_while(|b| glasnost_ini::lex::is_whitespace(**b))
            .skip_while(|b| matches!(b, b'+' | b'-'))
            .take_while(|b| b.is_ascii_digit())
            .count();
        if digits == 0 {
            return (
                (out, None),
                Some(format!("the {name} field does not parse")),
            );
        }
        *slot = value::decimal_integer(f);
    }
    // spec: S-0107 Import mapping (a left-out direction gives none, without a warning)
    let Some(d) = fields.next() else {
        return ((out, None), None);
    };
    // spec: S-0107 claim 22 (exact, case-sensitive: `quirks.case_sensitive_direction`)
    let dir = DIRECTIONS.iter().position(|n| {
        if case_sensitive {
            d == n.as_bytes()
        } else {
            d.eq_ignore_ascii_case(n.as_bytes())
        }
    });
    match dir {
        Some(i) => ((out, Some(i as u8)), None),
        None => (
            (out, None),
            Some("the direction is not an exact upper-case compass abbreviation; none".to_owned()),
        ),
    }
}

/// Builds the `sim` object of the converted tree and the importer warnings
/// its reading raises.
pub fn build(inputs: SimInputs<'_>) -> (Node, Vec<Warning>) {
    Builder {
        i: inputs,
        warnings: Vec::new(),
    }
    .build()
}

/// Puts `sim` into the converted tree, replacing an earlier one.
pub fn attach(tree: &mut Node, sim: Node) {
    let jsonc::Value::Object(root) = &mut tree.value else {
        return;
    };
    root.retain(|m| m.key != "sim");
    root.push(jsonc::Member {
        key: "sim".into(),
        key_span: Default::default(),
        value: sim,
    });
}

#[path = "sim_combat.rs"]
mod combat;
pub use combat::converted_keys;

#[path = "sim_econ.rs"]
mod econ;
pub use econ::converted_keys as econ_converted_keys;

#[cfg(test)]
#[path = "sim_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "sim_combat_tests.rs"]
mod combat_tests;

#[cfg(test)]
#[path = "sim_econ_tests.rs"]
mod econ_tests;
