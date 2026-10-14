//! Registration and layering: the rules layers of one game applied in
//! order (S-0004, S-0006, S-0103 claim 5).
//!
//! Each layer registers its type lists (S-0004 claim 2), sets up sides
//! (claim 9), runs the reading passes in rank order (claim 6) in which every
//! existing type reads its own section from that layer, reads the
//! game-wide sections, and finally its tiberium list (claim 10). Every key
//! a type reads keeps its full history, so the converter can say which
//! layer, file and line set each value.

use std::collections::{BTreeMap, BTreeSet};

use glasnost_ini::binfloat::{self, Binary, Format};
use glasnost_ini::value::{self, Decimal, DecimalField};
use glasnost_ini::{Document, Section};

use crate::kinds::Kind;
use crate::names::show;
use crate::refs;

/// Longest stored type name (S-0004 claim 5).
pub const STORED_NAME_MAX: usize = 24; // spec: S-0004 Constants (stored name length)
/// Bytes of a type-list value read before lookup (S-0004 claim 5).
pub const LIST_VALUE_MAX: usize = 31; // spec: S-0004 Constants (type-list value read length)
/// Bytes of a tiberium-list value (S-0004 claim 10).
pub const TIBERIUM_VALUE_MAX: usize = 23; // spec: S-0004 Constants (tiberium list value length)
/// Entries of the colour-adjustment table (S-0006 claim 2).
pub const COLOUR_ADD_LEN: usize = 16; // spec: S-0006 Constants (colour-adjustment table size)
/// Thousandths per unit of the light values (S-0104 claim 12).
const PER_MILLE: i64 = 1000; // spec: S-0104 claim 12 (stored in thousandths)

/// The role of a rules layer (S-0006 claim 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    /// The base rules.
    Base,
    /// The language rules file.
    Language,
    /// The game-mode layer.
    Mode,
    /// The map layer.
    Map,
}

impl Role {
    /// Our label for the layer.
    pub fn label(self) -> &'static str {
        match self {
            Role::Base => "base rules",
            Role::Language => "language rules",
            Role::Mode => "game mode",
            Role::Map => "map",
        }
    }
}

/// Which kind of game the layers are applied for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GameKind {
    /// A skirmish or multiplayer game: the game-mode layer applies and
    /// side indices follow the countries' side fields (S-0004 claim 9).
    #[default]
    Skirmish,
    /// A campaign mission: no game-mode layer, side indices follow the
    /// side list.
    Campaign,
}

/// One rules layer.
#[derive(Clone, Debug)]
pub struct Layer {
    /// Its role.
    pub role: Role,
    /// File name and where it was found, for histories.
    pub file: String,
    /// The document (empty for an empty layer, which still counts).
    pub doc: Document,
}

/// Quirk switches of S-0004, S-0006, S-0103 claim 5, S-0104 claims 12 and
/// 13, S-0105 claim 27 and S-0215 claim 21 that this pass implements. Each defaults to the
/// original's behaviour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayerQuirks {
    /// `quirks.unlisted_reference_unread` (S-0004 claim 6).
    pub unlisted_reference_unread: bool,
    /// `quirks.long_name_duplicates` (S-0004 claim 8).
    pub long_name_duplicates: bool,
    /// `quirks.tiberium_index_by_key` (S-0004 claim 10).
    pub tiberium_index_by_key: bool,
    /// `quirks.side_change_removes_by_position` (S-0004 claim 9).
    pub side_change_removes_by_position: bool,
    /// `quirks.particle_none_is_a_name` (S-0004 claim 2).
    pub particle_none_is_a_name: bool,
    /// `quirks.override_resets_keys` (S-0006 claim 5).
    pub override_resets_keys: bool,
    /// `quirks.barrel_recoil_layer_reset` (S-0105 claim 27).
    pub barrel_recoil_layer_reset: bool,
    /// `quirks.reread_truncates_light_values` (S-0104 claim 12).
    pub reread_truncates_light_values: bool,
    /// `quirks.colour_redefinition_ignored` (S-0103 claim 5).
    pub colour_redefinition_ignored: bool,
    /// `quirks.bad_language_rules_skips_mode` (S-0006 claim 12): a
    /// language rules file with sections that fails its check at game
    /// start makes the game skip its game-mode layer and leaves the
    /// colour-adjustment table to the map alone.
    pub bad_language_rules_skips_mode: bool,
    /// `quirks.anti_air_rate_layer_reset` (S-0215 claim 21): a layer that
    /// contains a behaviour's section but gives no anti-air rate, or gives
    /// 0, sets the anti-air rate to the rate after that layer. Off: an
    /// absent anti-air rate keeps the held value.
    pub anti_air_rate_layer_reset: bool,
    /// `quirks.persist_global_overrides` (S-0006 claim 10): game-wide
    /// values carry over into later games of the session. Not consulted:
    /// the importer imports one game, the first of its session.
    pub persist_global_overrides: bool,
    /// `quirks.persist_tiberium_overrides` (S-0006 claim 10, Formulas,
    /// claim 14): tiberiums are not rebuilt at a game start. Not consulted,
    /// as above.
    pub persist_tiberium_overrides: bool,
    /// `quirks.mission_rules_before_reset` (S-0006 claim 14): a campaign
    /// mission's mission-named file acts as a layer of the previous game.
    /// Not consulted: campaigns are out of scope.
    pub mission_rules_before_reset: bool,
    /// `quirks.mode_number_first_wins` (S-0006 claim 13): a mode chosen
    /// from the list plays the first listed mode with its number
    /// (`modes::select`).
    pub mode_number_first_wins: bool,
    /// `quirks.extra_light_truncation` (S-0104 claim 13): the extra-light
    /// values are stored as ⌊v × 1000⌋, so a value such as 0.57 loses a
    /// unit. Off: v × 1000 rounded to nearest.
    pub extra_light_truncation: bool,
}

impl Default for LayerQuirks {
    fn default() -> Self {
        // spec: S-0004 Edge cases, S-0006 Edge cases, S-0103 Edge cases (all default on)
        Self {
            unlisted_reference_unread: true,
            long_name_duplicates: true,
            tiberium_index_by_key: true,
            side_change_removes_by_position: true,
            particle_none_is_a_name: true,
            override_resets_keys: true,
            barrel_recoil_layer_reset: true,
            reread_truncates_light_values: true,
            colour_redefinition_ignored: true,
            bad_language_rules_skips_mode: true,
            // spec: S-0215 Edge cases (anti_air_rate_layer_reset defaults on)
            anti_air_rate_layer_reset: true,
            // spec: S-0006 Edge cases (persist_global_overrides, persist_tiberium_overrides, mission_rules_before_reset and mode_number_first_wins default on)
            persist_global_overrides: true,
            persist_tiberium_overrides: true,
            mission_rules_before_reset: true,
            mode_number_first_wins: true,
            // spec: S-0104 Edge cases (extra_light_truncation defaults on)
            extra_light_truncation: true,
        }
    }
}

impl LayerQuirks {
    /// The switches by name, for the import record.
    pub fn named(&self) -> [(&'static str, bool); 16] {
        [
            ("unlisted_reference_unread", self.unlisted_reference_unread),
            ("long_name_duplicates", self.long_name_duplicates),
            ("tiberium_index_by_key", self.tiberium_index_by_key),
            (
                "side_change_removes_by_position",
                self.side_change_removes_by_position,
            ),
            ("particle_none_is_a_name", self.particle_none_is_a_name),
            ("override_resets_keys", self.override_resets_keys),
            ("barrel_recoil_layer_reset", self.barrel_recoil_layer_reset),
            (
                "reread_truncates_light_values",
                self.reread_truncates_light_values,
            ),
            (
                "colour_redefinition_ignored",
                self.colour_redefinition_ignored,
            ),
            (
                "bad_language_rules_skips_mode",
                self.bad_language_rules_skips_mode,
            ),
            ("anti_air_rate_layer_reset", self.anti_air_rate_layer_reset),
            ("persist_global_overrides", self.persist_global_overrides),
            (
                "persist_tiberium_overrides",
                self.persist_tiberium_overrides,
            ),
            (
                "mission_rules_before_reset",
                self.mission_rules_before_reset,
            ),
            ("mode_number_first_wins", self.mode_number_first_wins),
            ("extra_light_truncation", self.extra_light_truncation),
        ]
    }
}

/// What happened to a key in one layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum What {
    /// The layer gave the key this stored value.
    Set(Vec<u8>),
    /// The layer gave a "not given" value, which keeps the held value
    /// (S-0006 claim 3).
    NotGiven(Vec<u8>),
    /// The layer has the section but not the key, and the key resets to
    /// its engine default (S-0006 claim 5).
    Reset,
    /// The barrel value took the turret's value (S-0105 claim 27).
    FromTurret(Option<Vec<u8>>),
    /// The value, in thousandths, was truncated on re-read (S-0104 claim
    /// 12).
    Truncated(i64),
    /// The key is read from the base rules only (S-0006 claim 2).
    Ignored(Vec<u8>),
}

/// One entry of a key's history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    /// Index of the layer in [`Outcome::layers`] (or the art database for
    /// animations).
    pub layer: usize,
    /// Line in that layer's file (0 when the event has no line).
    pub line: u32,
    /// What happened.
    pub what: What,
}

/// A key's layered state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyState {
    /// The stored value in force, if any layer gave one.
    pub value: Option<Vec<u8>>,
    /// The key holds its reset value (S-0006 claim 5).
    pub reset: bool,
    /// A value in thousandths computed by the layering (S-0104 claim 12).
    pub derived: Option<i64>,
    /// Every layer's effect, in order.
    pub history: Vec<Event>,
}

/// How a type came into existence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum How {
    /// Its kind's type list (S-0004 claim 2).
    List,
    /// A key of another type's section (S-0004 claim 6).
    Reference {
        /// The referring type's kind.
        kind: Kind,
        /// The referring type.
        name: Vec<u8>,
        /// The key.
        key: String,
    },
    /// A game-wide section (S-0004 claim 4, rows 2 and 4).
    GameWide {
        /// The section.
        section: &'static str,
        /// The key.
        key: &'static str,
    },
    /// A particle system that omits its emitted particle (S-0006 claim 5).
    EmptyParticle,
    /// The side field of a country.
    SideField,
    /// The particle system the original creates at game start (S-0004
    /// claim 3).
    GameStart,
}

/// A registered type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeRec {
    /// Stored name (at most 24 bytes, S-0004 claim 5).
    pub name: Vec<u8>,
    /// Layer that created it.
    pub created_in: usize,
    /// How.
    pub how: How,
    /// Layers whose section it read (S-0006 claim 7), in order.
    pub reads: Vec<usize>,
    /// Keys read, by key bytes.
    pub keys: BTreeMap<Vec<u8>, KeyState>,
    /// For a country: its side's index (S-0004 claim 9).
    pub side: Option<usize>,
}

/// A side (S-0004 claim 9).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SideRec {
    /// Name.
    pub name: Vec<u8>,
    /// Member countries (registration indices) in list order.
    pub members: Vec<usize>,
}

/// A colour definition (S-0103 claims 4 and 5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColourDef {
    /// Name.
    pub name: Vec<u8>,
    /// Hue, saturation, value.
    pub hsv: [u8; 3],
    /// Layer and line that created it (and, with the switch off, the last
    /// one that changed it).
    pub layer: usize,
    /// Line.
    pub line: u32,
}

/// One position of the colour-adjustment table: red, green, blue, and the
/// layer and line that set it (none: still zero).
pub type ColourAddEntry = ([u8; 3], Option<(usize, u32)>);

/// An importer warning (glossary, *importer warning*).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Warning {
    /// File (with its archive), as in [`Layer::file`].
    pub file: String,
    /// 1-based line, 0 when the warning concerns the whole file.
    pub line: u32,
    /// What and why, citing the spec.
    pub message: String,
}

/// The result of applying the layers of one game.
#[derive(Clone, Debug, Default)]
pub struct Outcome {
    /// File labels: the layers, then the art database.
    pub files: Vec<(String, String)>,
    /// Registries per kind, in registration order.
    pub registries: BTreeMap<Kind, Vec<TypeRec>>,
    /// Sides in index order.
    pub sides: Vec<SideRec>,
    /// Game-wide sections, layered.
    pub globals: BTreeMap<Vec<u8>, BTreeMap<Vec<u8>, KeyState>>,
    /// The colour-adjustment table and where each entry came from.
    pub colour_add: [ColourAddEntry; COLOUR_ADD_LEN],
    /// Colour definitions in definition-position order.
    pub colours: Vec<ColourDef>,
    /// Importer warnings.
    pub warnings: Vec<Warning>,
}

/// Reset-on-override keys per kind (S-0006 claim 5, Import mapping).
fn reset_keys(kind: Kind) -> &'static [&'static str] {
    // spec: S-0006 claim 5 (keys that reset on override)
    match kind {
        Kind::Countries => &["ParentCountry"],
        Kind::Warheads => &["Verses"],
        Kind::ParticleSystems => &["HoldsWhat", "BehavesLike"],
        Kind::Particles => &["BehavesLike"],
        _ => &[],
    }
}

/// The buildable key of a land-type section, which resets to no (S-0006
/// claim 5, Import mapping).
// spec: S-0006 Import mapping (land-type keys and reset values)
const LAND_BUILDABLE: &str = "Buildable";

/// The difficulty sections and their reset keys (S-0006 claim 5).
// spec: S-0006 Import mapping (difficulty sections and keys)
const DIFFICULTY_SECTIONS: [&str; 3] = ["Easy", "Normal", "Difficult"];
// spec: S-0006 Import mapping (difficulty keys that reset)
const DIFFICULTY_KEYS: [&str; 12] = [
    "FirePower",
    "Groundspeed",
    "Airspeed",
    "Armor",
    "ROF",
    "Cost",
    "BuildTime",
    "RepairDelay",
    "BuildDelay",
    "BuildSlowdown",
    "DestroyWalls",
    "ContentScan",
];

/// Barrel keys and the turret keys they take (S-0105 claim 27).
// spec: S-0105 claim 27, S-0006 Import mapping (barrel recoil values)
const BARREL_KEYS: [(&str, &str); 4] = [
    ("BarrelTravel", "TurretTravel"),
    ("BarrelCompressFrames", "TurretCompressFrames"),
    ("BarrelHoldFrames", "TurretHoldFrames"),
    ("BarrelRecoverFrames", "TurretRecoverFrames"),
];

/// Building light keys stored as trunc(v × 1000 + 0.1) (S-0104 claim 12).
// spec: S-0104 claim 12, S-0006 Import mapping (building light values)
const BUILDING_LIGHT_KEYS: [&str; 4] = [
    "LightIntensity",
    "LightRedTint",
    "LightGreenTint",
    "LightBlueTint",
];

/// Extra-light keys of the audio-visual settings, ⌊v × 1000⌋ (S-0104
/// claims 12 and 13).
// spec: S-0104 Import mapping, S-0006 Import mapping (extra-light values)
const EXTRA_LIGHT_KEYS: [&str; 3] = ["ExtraUnitLight", "ExtraInfantryLight", "ExtraAircraftLight"];
/// The audio-visual settings section.
const AUDIO_VISUAL: &[u8] = b"AudioVisual"; // spec: S-0004 Import mapping
/// The special-weapon settings section (S-0004 claim 6).
const SPECIAL_WEAPONS: &[u8] = b"SpecialWeapons"; // spec: S-0004 Import mapping
/// The side list (S-0004 Import mapping).
const SIDES: &[u8] = b"Sides"; // spec: S-0004 Import mapping
/// The colour definitions (S-0103 Import mapping).
const COLORS: &[u8] = b"Colors"; // spec: S-0103 Import mapping
/// The colour-adjustment section (S-0006 Import mapping).
const COLOR_ADD: &[u8] = b"ColorAdd"; // spec: S-0006 Import mapping
/// The maximum number of players, read from the base rules only (S-0006
/// claim 2).
const MAXIMUMS_PLAYERS: (&[u8], &[u8]) = (b"Maximums", b"Players"); // spec: S-0006 Import mapping
/// The combat-damage section (S-0004 Import mapping).
pub const COMBAT_DAMAGE: &[u8] = b"CombatDamage"; // spec: S-0004 Import mapping (the combat-damage section)

/// How a "not given" value is compared (S-0006 claim 3: on the number the
/// key's value form gives, exactly).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NotGivenForm {
    /// An integer key whose value −1 is "not given".
    IntegerMinusOne,
    /// A decimal key whose value −1 is "not given".
    DecimalMinusOne,
    /// A decimal key whose value 0 is "not given".
    DecimalZero,
}

/// Type keys with a "not given" value, by kind (S-0006 claim 3 and the
/// topic specs that name further keys).
fn type_not_given(kind: Kind, key: &[u8]) -> Option<NotGivenForm> {
    use NotGivenForm::*;
    match (kind, key) {
        // spec: S-0006 claim 3 (a superweapon's recharge time: 0)
        (Kind::Superweapons, b"RechargeTime") => Some(DecimalZero),
        // spec: S-0006 claim 3, Import mapping (pitch and roll angles: −1)
        (
            Kind::Infantry | Kind::Vehicles | Kind::Aircraft | Kind::Buildings,
            b"PitchAngle" | b"RollAngle",
        ) => Some(DecimalMinusOne),
        // spec: S-0006 claim 3, Import mapping (a voxel animation's angular velocities: 0)
        (Kind::VoxelAnimations, b"MinAngularVelocity" | b"MaxAngularVelocity") => Some(DecimalZero),
        // spec: S-0207 claim 1 (vehicle speed field: −1 keeps the held value), S-0208 Import mapping (infantry)
        (Kind::Vehicles | Kind::Infantry, b"Speed") => Some(IntegerMinusOne),
        // spec: S-0215 Import mapping (GuardRange: absent or −1 keeps the held value; a type section)
        (Kind::Infantry | Kind::Vehicles | Kind::Aircraft | Kind::Buildings, b"GuardRange") => {
            Some(DecimalMinusOne)
        }
        // spec: S-0306 Import mapping (weapon Speed: −1 keeps the value in force)
        (Kind::Weapons, b"Speed") => Some(IntegerMinusOne),
        // spec: S-0307 claim 1 (the lepton form: a value read as exactly −1 keeps the value in force)
        (Kind::Weapons, b"Range" | b"MinimumRange") => Some(DecimalMinusOne),
        // spec: S-0307 claim 1, S-0309 Import mapping (AirRangeBonus: the lepton form, as GuardRange)
        (Kind::Infantry | Kind::Vehicles | Kind::Aircraft | Kind::Buildings, b"AirRangeBonus") => {
            Some(DecimalMinusOne)
        }
        _ => None,
    }
}

/// Game-wide keys with a "not given" value. S-0006 claim 3 says no
/// game-wide key has one; the topic specs below name these.
fn global_not_given(section: &[u8], key: &[u8]) -> Option<NotGivenForm> {
    match (section, key) {
        // spec: S-0209 Import mapping (`[General]` CloseEnough: −1 is the "not given" value)
        (b"General", b"CloseEnough") => Some(NotGivenForm::DecimalMinusOne),
        // spec: S-0215 Import mapping (`[General]` GuardModeStray: absent or −1 keeps the held value)
        (b"General", b"GuardModeStray") => Some(NotGivenForm::DecimalMinusOne),
        // spec: S-0210 Import mapping, F2 (combat-damage `Crush`: exactly −1 counts as absent)
        (COMBAT_DAMAGE, b"Crush") => Some(NotGivenForm::DecimalMinusOne),
        // spec: S-0306 Import mapping, S-0309 Import mapping (BallisticScatter, FireSupress: the lepton form of S-0307 claim 1, exactly −1 keeps the held value)
        (COMBAT_DAMAGE, b"BallisticScatter" | b"FireSupress") => {
            Some(NotGivenForm::DecimalMinusOne)
        }
        _ => None,
    }
}

/// Whether a type key's stored value is its "not given" value (S-0006
/// claim 3 and the topic specs named in [`type_not_given`]).
pub(crate) fn type_value_not_given(kind: Kind, key: &[u8], stored: &[u8]) -> bool {
    type_not_given(kind, key).is_some_and(|f| is_not_given(f, stored))
}

/// Whether a game-wide key's stored value is its "not given" value.
pub(crate) fn global_value_not_given(section: &[u8], key: &[u8], stored: &[u8]) -> bool {
    global_not_given(section, key).is_some_and(|f| is_not_given(f, stored))
}

/// Whether `stored` is the "not given" value of its form (S-0006 claim 3:
/// every spelling that reads as that number counts).
fn is_not_given(form: NotGivenForm, stored: &[u8]) -> bool {
    match form {
        NotGivenForm::IntegerMinusOne => value::integer(stored) == Some(-1),
        NotGivenForm::DecimalMinusOne => match value::decimal(stored, DecimalField::Double) {
            Decimal::Value { value, .. } => value.to_exact_decimal() == "-1",
            Decimal::NoNumber => false,
        },
        NotGivenForm::DecimalZero => is_zero(stored),
    }
}

/// trunc(v × 1000 + 0.1) toward zero, v the single-precision decimal
/// widened to double, each step in double precision rounded toward zero (S-0104 claim 12, Formulas "Precision of decimal
/// conversions"). `None` when the value has no number (S-0003 claim 8: our
/// engine keeps the value in force).
pub fn building_light_thousandths(stored: &[u8]) -> Option<i64> {
    // spec: S-0104 claim 12 (r18: v is the single-precision decimal of S-0003 claim 6, widened to double; a `%` value stays the double of S-0003 claim 7)
    let Decimal::Value { value, .. } = value::decimal(stored, DecimalField::Double) else {
        return None;
    };
    let thousand = Binary::from_i64(Format::Double, PER_MILLE);
    // spec: S-0104 claim 12 (the constant 0.1, a double)
    let tenth = tenth_double();
    value
        .mul_rz(&thousand, Format::Double)
        .add_rz(&tenth, Format::Double)
        .trunc_to_i64()
}

/// ⌊v × 1000⌋ truncated toward zero (S-0104 claim 13, Import mapping,
/// extra-light values). With `truncate` false
/// (`quirks.extra_light_truncation` off) v × 1000 is rounded to the nearest
/// integer instead, halves away from zero.
pub fn extra_light_thousandths(stored: &[u8], truncate: bool) -> Option<i64> {
    let Decimal::Value { value, .. } = value::decimal(stored, DecimalField::Double) else {
        return None;
    };
    let p = value.mul_rz(&Binary::from_i64(Format::Double, PER_MILLE), Format::Double);
    if truncate {
        // spec: S-0104 claim 13 (stored as ⌊v × 1000⌋ with no rounding allowance)
        return p.trunc_to_i64();
    }
    // spec: S-0104 Edge cases (quirks.extra_light_truncation off: round to nearest)
    // 2p is exact, and trunc(2p) decides the nearest integer: sgn(q)·⌊(|q| + 1)/2⌋.
    let q = p
        .mul_rz(&Binary::from_i64(Format::Double, 2), Format::Double)
        .trunc_to_i64()?;
    Some(q.signum() * ((q.abs() + 1) / 2))
}

fn tenth_double() -> Binary {
    let text = binfloat::DecimalText {
        neg: false,
        digits: b"1".to_vec(),
        exp10: -1,
    };
    // 0.1 is not near a tie, so claim 6's rule gives the nearest double.
    binfloat::decimal_to_binary(&text, Format::Double).value
}

/// The building light value after a re-read that omits the key: the whole
/// part *k*, then trunc(1000·*k* + 0.1) (S-0006 Formulas).
fn truncate_building_light(t: i64) -> i64 {
    let k = t / PER_MILLE;
    // spec: S-0006 Formulas (t' = 1000k when k >= 0, 1000k + 1 when k < 0)
    if k >= 0 {
        PER_MILLE * k
    } else {
        PER_MILLE * k + 1
    }
}

/// The extra-light value after such a re-read: 1000·*k* (S-0006 Formulas).
fn truncate_extra_light(t: i64) -> i64 {
    // spec: S-0006 Formulas (t' = 1000k for the extra-light values)
    PER_MILLE * (t / PER_MILLE)
}

/// Applies `layers` in order (S-0006 claim 1). `art` is the art database,
/// read by animations in every layer (S-0006 claim 8).
pub fn apply(
    layers: &[Layer],
    art: &Document,
    art_file: &str,
    game: GameKind,
    quirks: &LayerQuirks,
) -> Outcome {
    apply_with(layers, art, art_file, game, quirks, false)
}

/// [`apply`], for a game whose language rules file has sections and fails
/// its check (`bad_language_rules`, S-0006 claim 12). With
/// `quirks.bad_language_rules_skips_mode` on, the caller leaves the
/// game-mode layer out and the colour-adjustment table takes only the
/// map's entries.
pub fn apply_with(
    layers: &[Layer],
    art: &Document,
    art_file: &str,
    game: GameKind,
    quirks: &LayerQuirks,
    bad_language_rules: bool,
) -> Outcome {
    apply_in(
        layers,
        art,
        art_file,
        game,
        quirks,
        bad_language_rules,
        &ArtSelect::default(),
    )
}

/// What selects the art section of a building's or aircraft's art-file
/// references (S-0004 claim 14): whether the game's theater is snow and
/// `quirks.arctic_suffix_on_flag_turning_on` (S-0007 claims 9 and 10).
#[derive(Clone, Copy, Debug, Default)]
pub struct ArtSelect {
    /// The game's theater is snow.
    pub snow: bool,
    /// The art lookup's switches.
    pub quirks: crate::art::ArtQuirks,
}

/// [`apply_with`], in a theater: the snow suffix selects the art section of
/// building and aircraft references (S-0004 claim 14).
pub fn apply_in(
    layers: &[Layer],
    art: &Document,
    art_file: &str,
    game: GameKind,
    quirks: &LayerQuirks,
    bad_language_rules: bool,
    select: &ArtSelect,
) -> Outcome {
    let mut out = Outcome::default();
    for l in layers {
        out.files.push((l.role.label().to_owned(), l.file.clone()));
    }
    out.files
        .push(("art database".to_owned(), art_file.to_owned()));
    let art_layer = layers.len();
    for k in Kind::all() {
        out.registries.insert(k, Vec::new());
    }
    let mut run = Run {
        out,
        quirks: *quirks,
        art,
        select: *select,
        art_layer,
        pending: Vec::new(),
        layer_now: 0,
        phase: Phase::Early,
        origin: BTreeMap::new(),
        // spec: S-0006 claim 12 (the table is left with zero in all 16 positions; only the map's entries count)
        map_only_colour_add: bad_language_rules && quirks.bad_language_rules_skips_mode,
        untrimmed: layers
            .first()
            .is_none_or(|l| l.doc.quirks().list_items_untrimmed),
    };
    for (li, layer) in layers.iter().enumerate() {
        if li == 0 && game == GameKind::Skirmish {
            run.preseed_sides(layer);
        }
        run.layer(li, layer);
    }
    run.finish();
    run.out
}

/// The image name that selects a projectile's trailer section of the art
/// database at its type read of layer `li` (S-0004 claim 14, S-0007 claim
/// 33): the image key of that read's rules section, cut to 24 bytes, when
/// it is given and not empty. Nothing carries over from an earlier read.
pub(crate) fn projectile_trailer_image(t: &TypeRec, li: usize) -> Option<&[u8]> {
    // spec: S-0004 claim 14, S-0007 claims 26 and 33 (afresh at each read; absent or empty: no trailer read)
    crate::art::given_at(t, "Image", li)
        .map(|v| value::text(v, crate::art::IMAGE_MAX))
        .filter(|n| !n.is_empty())
}

/// A type by kind and registration index.
type TypeAt = (Kind, usize);

/// Where in a layer types are being created (S-0004 claim 4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    /// Row 2: the early game-wide sections, before the passes.
    Early,
    /// Row 3: the reading pass of a kind.
    Pass(Kind),
    /// Rows 4 to 6: the late game-wide sections, the second warhead
    /// reading and the tiberium list, after every pass.
    Late,
}

struct Run<'a> {
    out: Outcome,
    quirks: LayerQuirks,
    art: &'a Document,
    /// The theater and switches the art-file references' section depends on.
    select: ArtSelect,
    art_layer: usize,
    /// Types created by reference that read the creating layer anyway
    /// (switch `quirks.unlisted_reference_unread` off).
    pending: Vec<(Kind, usize)>,
    /// The layer being processed (the art database's reads create types
    /// in this layer).
    layer_now: usize,
    /// Where in the layer the run is (S-0004 claim 4).
    phase: Phase,
    /// For each type created by reference: the type whose field created
    /// it and the name it was looked up by (the guard of
    /// [`Run::repeats`]).
    origin: BTreeMap<TypeAt, (Option<TypeAt>, Vec<u8>)>,
    /// Only the map layer fills the colour-adjustment table (S-0006 claim
    /// 12).
    map_only_colour_add: bool,
    /// `quirks.list_items_untrimmed` (S-0003 claim 12), from the layers'
    /// ini switches.
    untrimmed: bool,
}

impl Run<'_> {
    fn warn(&mut self, layer: usize, line: u32, message: String) {
        let file = self
            .out
            .files
            .get(layer)
            .map(|f| f.1.clone())
            .unwrap_or_default();
        self.out.warnings.push(Warning {
            file,
            line,
            message,
        });
    }

    fn reg(&self, kind: Kind) -> &[TypeRec] {
        self.out.registries.get(&kind).map_or(&[], Vec::as_slice)
    }

    fn reg_mut(&mut self, kind: Kind) -> &mut Vec<TypeRec> {
        self.out.registries.entry(kind).or_default()
    }

    /// Finds a type by name, ignoring ASCII case (S-0003 claim 14) and
    /// comparing against stored names (S-0004 claim 8).
    fn find(&self, kind: Kind, name: &[u8]) -> Option<usize> {
        let probe = if self.quirks.long_name_duplicates {
            name
        } else {
            // spec: S-0004 claim 8 (switch off: names truncated before every comparison)
            &name[..name.len().min(STORED_NAME_MAX)]
        };
        self.reg(kind)
            .iter()
            .position(|t| value::type_name_eq(&t.name, probe))
    }

    fn create(&mut self, kind: Kind, name: &[u8], layer: usize, how: How) -> usize {
        // spec: S-0004 claim 5 (a type stores at most the first 24 bytes of its name)
        let stored = name[..name.len().min(STORED_NAME_MAX)].to_vec();
        let reg = self.reg_mut(kind);
        reg.push(TypeRec {
            name: stored,
            created_in: layer,
            how,
            reads: Vec::new(),
            keys: BTreeMap::new(),
            side: None,
        });
        reg.len() - 1
    }

    fn find_or_create(&mut self, kind: Kind, name: &[u8], layer: usize, how: How) -> (usize, bool) {
        match self.find(kind, name) {
            Some(i) => (i, false),
            None => (self.create(kind, name, layer, how), true),
        }
    }

    /// S-0004 claims 1 and 9: in a skirmish, the side registry holds the
    /// sides that the base rules' countries' side fields name, in
    /// country-list order, before the base rules are processed.
    fn preseed_sides(&mut self, base: &Layer) {
        let Some(list) = base
            .doc
            .section(Kind::Countries.list_section().unwrap_or_default())
        else {
            return;
        };
        for (_, winner) in list.list_positions() {
            let name = value::text(&winner.value, LIST_VALUE_MAX);
            if name.is_empty() || value::is_no_type(name) {
                continue;
            }
            let stored = &name[..name.len().min(STORED_NAME_MAX)];
            let side = base.doc.section(stored).and_then(|s| s.value(b"Side"));
            if let Some(side) = side {
                let side = value::text(side, value::COMMON_TEXT_LIMIT).to_vec();
                if !self
                    .out
                    .sides
                    .iter()
                    .any(|s| value::type_name_eq(&s.name, &side))
                {
                    self.out.sides.push(SideRec {
                        name: side,
                        members: Vec::new(),
                    });
                }
            }
        }
    }

    /// With `quirks.unlisted_reference_unread` off, the types created by
    /// reference that the original leaves unread read the creating layer's
    /// section (S-0004 claim 6), in creation order.
    fn drain_pending(&mut self, li: usize, doc: &Document, type_sections: &mut BTreeSet<usize>) {
        // spec: S-0004 claim 6 (switch off: a type created by reference reads the creating layer)
        let mut next = 0;
        while next < self.pending.len() {
            let (kind, i) = self.pending[next];
            next += 1;
            let name = self.reg(kind)[i].name.clone();
            if kind == Kind::Animations {
                if let Some(section) = self.art.section(&name) {
                    let section = section.clone();
                    self.read(kind, i, &section, self.art_layer);
                    self.reg_mut(kind)[i].reads.push(li);
                }
            } else if let Some(si) = doc.section_index(&name) {
                type_sections.insert(si);
                let section = doc.sections()[si].clone();
                self.read(kind, i, &section, li);
            }
        }
        self.pending.clear();
    }

    fn layer(&mut self, li: usize, layer: &Layer) {
        let doc = &layer.doc;
        self.layer_now = li;
        // S-0004 claim 2 and Import mapping: every list but the tiberiums', countries first.
        for kind in Kind::PASSES {
            if let Some(list) = kind.list_section().and_then(|s| doc.section(s)) {
                self.register_list(kind, list, li);
            }
        }
        // S-0004 claim 9: the side list takes effect before the countries' side fields.
        if let Some(sides) = doc.section(SIDES) {
            self.side_list(sides, li);
        }
        // S-0004 claim 4, row 2: the early game-wide sections, the AI settings first.
        self.phase = Phase::Early;
        for g in refs::EARLY {
            self.game_wide_refs(g, li, doc);
        }
        // S-0004 claim 6: reading passes in rank order (row 3).
        let mut type_sections: BTreeSet<usize> = BTreeSet::new();
        for kind in Kind::PASSES {
            self.phase = Phase::Pass(kind);
            self.pass(kind, li, doc, &mut type_sections);
        }
        // S-0004 claim 4, row 4: the late game-wide sections, in the claim's order.
        self.phase = Phase::Late;
        for g in refs::LATE {
            self.game_wide_refs(g, li, doc);
        }
        self.drain_pending(li, doc, &mut type_sections);
        // S-0004 claim 6: a layer with the special-weapon settings gives every warhead a second reading (row 5).
        if doc.section(SPECIAL_WEAPONS).is_some() {
            for i in 0..self.reg(Kind::Warheads).len() {
                let name = self.reg(Kind::Warheads)[i].name.clone();
                if let Some(si) = doc.section_index(&name) {
                    type_sections.insert(si);
                    let section = doc.sections()[si].clone();
                    self.read(Kind::Warheads, i, &section, li);
                }
            }
            self.drain_pending(li, doc, &mut type_sections);
        }
        // S-0004 claim 10: the tiberium list at the end of the layer (row 6).
        self.tiberiums(li, doc, &mut type_sections);
        self.drain_pending(li, doc, &mut type_sections);
        self.game_wide(li, layer, &type_sections);
        self.colours(li, doc);
        self.colour_add(li, layer);
    }

    fn register_list(&mut self, kind: Kind, list: &Section, li: usize) {
        for (_, winner) in list.list_positions() {
            // spec: S-0004 claim 5 (the value is cut to 31 bytes, trailing spaces removed)
            let name = value::text(&winner.value, LIST_VALUE_MAX);
            // spec: S-0004 claim 2 (empty, `none` and `<none>` create nothing)
            if name.is_empty() || value::is_no_type(name) {
                continue;
            }
            // spec: S-0004 claim 2 (an existing name keeps its index; any other is created)
            self.find_or_create(kind, name, li, How::List);
        }
    }

    fn country_by_name(&self, name: &[u8]) -> Option<usize> {
        if let Some(i) = self.find(Kind::Countries, name) {
            return Some(i);
        }
        // spec: S-0004 claim 9 (a side list may name a country by its name field)
        self.reg(Kind::Countries).iter().position(|c| {
            c.keys
                .get(&b"Name"[..])
                .and_then(|k| k.value.as_deref())
                .is_some_and(|n| value::type_name_eq(n, name))
        })
    }

    fn side_index(&mut self, name: &[u8]) -> usize {
        // spec: S-0004 claim 9 (side names are compared without regard to case; unknown names create a side)
        match self
            .out
            .sides
            .iter()
            .position(|s| value::type_name_eq(&s.name, name))
        {
            Some(i) => i,
            None => {
                self.out.sides.push(SideRec {
                    name: name.to_vec(),
                    members: Vec::new(),
                });
                self.out.sides.len() - 1
            }
        }
    }

    /// A country's side field (S-0004 claim 9): an unknown side is
    /// created; a change of side moves the country to the new side's
    /// member list.
    fn country_side(&mut self, country: usize, side_name: &[u8]) {
        let new = self.side_index(side_name);
        let old = self.reg(Kind::Countries)[country].side;
        if old == Some(new) {
            return;
        }
        if let Some(o) = old {
            let members = &mut self.out.sides[o].members;
            if self.quirks.side_change_removes_by_position {
                // spec: S-0004 claim 9 (the member at the position equal to the country's index is removed)
                if country < members.len() {
                    members.remove(country);
                }
            } else {
                members.retain(|&m| m != country);
            }
        }
        self.out.sides[new].members.push(country);
        self.reg_mut(Kind::Countries)[country].side = Some(new);
    }

    fn side_list(&mut self, list: &Section, li: usize) {
        for (_, winner) in list.list_positions() {
            let side = self.side_index(&winner.key);
            let mut members = Vec::new();
            for item in value::name_items(&winner.value, value::COMMON_TEXT_LIMIT, self.untrimmed) {
                if let Some(c) = self.country_by_name(item) {
                    members.push(c);
                } else if let Some(s) = self
                    .out
                    .sides
                    .iter()
                    .position(|s| value::type_name_eq(&s.name, item))
                {
                    // spec: S-0004 claim 9 (a side's name stands for its current members)
                    members.extend(self.out.sides[s].members.clone());
                } else {
                    self.warn(
                        li,
                        winner.line,
                        format!(
                            "side list names {:?}, which is neither a country nor a side; skipped (S-0004 claim 9)",
                            show(item)
                        ),
                    );
                }
            }
            let mut unique = Vec::new();
            for m in members {
                if !unique.contains(&m) {
                    unique.push(m);
                }
            }
            for &c in &unique {
                self.reg_mut(Kind::Countries)[c].side = Some(side);
            }
            // spec: S-0004 claim 9 (the member list is replaced by the listed countries)
            self.out.sides[side].members = unique;
        }
    }

    fn pass(&mut self, kind: Kind, li: usize, doc: &Document, type_sections: &mut BTreeSet<usize>) {
        let mut i = 0;
        // Types created during their own kind's pass are read in that pass (S-0004 claim 6).
        while i < self.reg(kind).len() {
            let name = self.reg(kind)[i].name.clone();
            if kind == Kind::Animations {
                // spec: S-0006 claim 8 (animations read the art database in every layer)
                if let Some(section) = self.art.section(&name) {
                    let section = section.clone();
                    self.read(kind, i, &section, self.art_layer);
                    self.reg_mut(kind)[i].reads.push(li);
                }
            } else if let Some(si) = doc.section_index(&name) {
                // spec: S-0004 claim 7 (the section named exactly as the stored name)
                type_sections.insert(si);
                let section = doc.sections()[si].clone();
                self.read(kind, i, &section, li);
            }
            i += 1;
        }
    }

    /// One type reads one section (S-0006 claims 3 to 5).
    fn read(&mut self, kind: Kind, index: usize, section: &Section, li: usize) {
        let quirks = self.quirks;
        let earlier_reads = self.reg(kind)[index].reads.len();
        if kind != Kind::Animations && self.reg(kind)[index].reads.last() != Some(&li) {
            self.reg_mut(kind)[index].reads.push(li);
        }
        let mut seen: BTreeSet<&[u8]> = BTreeSet::new();
        let mut given: Vec<(Vec<u8>, Vec<u8>, u32)> = Vec::new();
        for e in section.entries() {
            if !seen.insert(e.key.as_slice()) {
                continue;
            }
            // The value a lookup of the key returns (S-0002 claim 1).
            let winner = section.get(&e.key).unwrap_or(e);
            given.push((e.key.clone(), winner.value.clone(), winner.line));
        }
        for (key, stored, line) in &given {
            let what = if type_value_not_given(kind, key, stored) {
                // spec: S-0006 claim 3 (a "not given" value keeps the held value)
                What::NotGiven(stored.clone())
            } else {
                What::Set(stored.clone())
            };
            let state = self.reg_mut(kind)[index]
                .keys
                .entry(key.clone())
                .or_default();
            if let What::Set(v) = &what {
                state.value = Some(v.clone());
                state.reset = false;
            }
            state.history.push(Event {
                layer: li,
                line: *line,
                what,
            });
        }
        let has = |k: &str| {
            given
                .iter()
                .any(|(key, _, _)| key.as_slice() == k.as_bytes())
        };

        // spec: S-0006 claim 5 (keys that reset on override)
        if quirks.override_resets_keys {
            for &k in reset_keys(kind) {
                if !has(k) {
                    let state = self.reg_mut(kind)[index]
                        .keys
                        .entry(k.as_bytes().to_vec())
                        .or_default();
                    state.value = None;
                    state.reset = true;
                    state.history.push(Event {
                        layer: li,
                        line: section.line(),
                        what: What::Reset,
                    });
                }
            }
        }

        // The reference fields of the section (S-0004 claim 13, Import mapping).
        self.references(kind, index, section, li);

        if kind == Kind::Countries
            && let Some((_, v, _)) = given.iter().find(|(k, _, _)| k.as_slice() == b"Side")
        {
            self.country_side(index, value::text(v, value::COMMON_TEXT_LIMIT));
        }

        if kind == Kind::Buildings {
            self.barrels(index, li, section, &has);
            self.building_lights(index, li, section, earlier_reads, &given);
        }
    }

    /// The reference fields a type takes from the section it is reading
    /// (S-0004 claim 13, Import mapping). `hist` is the history layer of
    /// that section (the art database for animations); the types it
    /// creates belong to the layer being processed.
    fn references(&mut self, kind: Kind, index: usize, section: &Section, hist: usize) {
        let from = Some((kind, index));
        let referrer = self.reg(kind)[index].name.clone();
        let art_section = self.art_reference_section(kind, index, hist);
        for f in refs::type_fields(kind) {
            let (src, src_layer) = if f.art {
                match &art_section {
                    Some(a) => (a, self.art_layer),
                    None => continue,
                }
            } else {
                (section, hist)
            };
            let Some(entry) = src.get(f.key.as_bytes()) else {
                continue;
            };
            let (stored, line) = (entry.value.clone(), entry.line);
            let how = |key: &str| How::Reference {
                kind,
                name: referrer.clone(),
                key: key.to_owned(),
            };
            self.field(f, &stored, src_layer, line, &how(f.key), from);
        }
        if refs::has_weapon_slots(kind) {
            self.weapon_fields(kind, index, section, hist, &referrer);
        }
        if kind == Kind::Buildings {
            for key in refs::SUPERWEAPON_FIELDS {
                let Some(entry) = section.get(key.as_bytes()) else {
                    continue;
                };
                // spec: S-0004 claims 3 and 13 (the superweapon fields find only; an unknown name means none)
                let name = value::text(&entry.value, refs::REFERENCE_LIMIT);
                if !value::is_no_type(name) && self.find(Kind::Superweapons, name).is_none() {
                    self.warn(
                        hist,
                        entry.line,
                        format!(
                            "buildings {} {key} names the unknown superweapon {:?}, which means none (S-0004 claim 3)",
                            show(&referrer),
                            show(name)
                        ),
                    );
                }
            }
        }
        // spec: S-0006 claim 5, S-0004 claim 2 (an omitted emitted particle is the particle with the empty name)
        if kind == Kind::ParticleSystems
            && section.get(b"HoldsWhat").is_none()
            && self.quirks.override_resets_keys
        {
            self.reference(Kind::Particles, b"", hist, 0, How::EmptyParticle, from);
        }
    }

    /// The art section a building's or aircraft's or projectile's art-file
    /// references come from at its type read of layer `li` (S-0004 claim
    /// 14, S-0007 claim 33). `None` for other kinds, and for a projectile
    /// whose section of this layer gives no image or an empty one.
    fn art_reference_section(&self, kind: Kind, index: usize, li: usize) -> Option<Section> {
        let t = &self.reg(kind)[index];
        let image = match kind {
            Kind::Buildings | Kind::Aircraft => {
                // spec: S-0004 claim 14 (the image in force at this read, with the snow suffix of S-0007 claims 9 and 10; none for voxel aircraft)
                let mut walk =
                    crate::art::ImageWalk::new(t, kind, self.select.snow, &self.select.quirks);
                for &r in t.reads.iter().filter(|&&r| r <= li) {
                    walk.step(self.art, t, r);
                }
                walk.image
            }
            // spec: S-0004 claim 14 (a projectile: this read's non-empty rules image, cut to 24 bytes, no suffix and no fallback to the type name)
            Kind::Projectiles => projectile_trailer_image(t, li)?.to_vec(),
            _ => return None,
        };
        self.art.section(&image).cloned()
    }

    /// One creating field's value (S-0004 claim 13): cut to the field's
    /// limit, split into items for a list, each item found or created.
    fn field(
        &mut self,
        f: &refs::Field,
        stored: &[u8],
        hist: usize,
        line: u32,
        how: &How,
        from: Option<(Kind, usize)>,
    ) {
        let limit = if f.limit == refs::ATTACHED_SYSTEM_LIMIT && !self.quirks.long_name_duplicates {
            // spec: S-0004 claim 8 (switch off: the attached-particle-system field reads up to 24 bytes)
            STORED_NAME_MAX
        } else {
            f.limit
        };
        let items: Vec<&[u8]> = match f.form {
            // spec: S-0004 claim 13 (the value is cut to the field's limit, trailing whitespace removed)
            refs::Form::Single => vec![value::text(stored, limit)],
            // spec: S-0004 claim 13, S-0003 claim 12 (list items in item order, empty items skipped)
            // spec: S-0003 claim 12 (quirks.list_items_untrimmed off: name items trimmed; on: the importer warns for an item with whitespace at either end)
            refs::Form::List => {
                let items = value::name_items(stored, limit, self.untrimmed);
                if self.untrimmed && items.iter().any(|i| value::edge_whitespace(i)) {
                    self.warn(
                        hist,
                        line,
                        format!(
                            "{}: a list item with whitespace at its start or end names a different type (S-0003 claim 12, quirks.list_items_untrimmed)",
                            f.key
                        ),
                    );
                }
                items
            }
        };
        for name in items {
            if name.is_empty() {
                continue;
            }
            match f.names {
                refs::Names::Standard => {
                    // spec: S-0004 claims 2 and 13 (`none` and `<none>` mean no type)
                    if value::is_no_type(name) {
                        continue;
                    }
                }
                refs::Names::Particle => {
                    // spec: S-0004 claim 2 (in the four particle fields `<none>` names no type)
                    if name.eq_ignore_ascii_case(b"<none>") {
                        if f.limit == refs::ATTACHED_SYSTEM_LIMIT {
                            // spec: S-0004 Edge cases (quirks.attached_system_none_keeps: the importer warns)
                            self.warn(
                                hist,
                                line,
                                format!(
                                    "{} {:?} keeps the attached particle system in force (S-0004 claim 2)",
                                    f.key,
                                    show(name)
                                ),
                            );
                        }
                        continue;
                    }
                    // spec: S-0004 Edge cases (quirks.particle_none_is_a_name off: `none` means no type)
                    if !self.quirks.particle_none_is_a_name && value::is_no_type(name) {
                        continue;
                    }
                }
            }
            if f.form == refs::Form::List
                && (name
                    .first()
                    .is_some_and(|&b| glasnost_ini::lex::is_whitespace(b))
                    || name
                        .last()
                        .is_some_and(|&b| glasnost_ini::lex::is_whitespace(b)))
            {
                // spec: S-0003 claim 12 (the importer warns for a name item that begins or ends with whitespace)
                self.warn(
                    hist,
                    line,
                    format!(
                        "{} item {:?} begins or ends with whitespace, so it names a different type (S-0003 claim 12)",
                        f.key,
                        show(name)
                    ),
                );
            }
            self.reference(f.kind, name, hist, line, how.clone(), from);
        }
    }

    /// The weapon fields of a building, vehicle, aircraft or infantry type
    /// (S-0004 claim 13, weapon-slot table). The turret count, the weapon
    /// count and the clear-all-weapons flag act with the values the type
    /// holds once this section has given them.
    fn weapon_fields(
        &mut self,
        kind: Kind,
        index: usize,
        section: &Section,
        hist: usize,
        referrer: &[u8],
    ) {
        let (tc_key, wc_key, clear_key) = refs::SLOT_KEYS;
        let t = &self.reg(kind)[index];
        // spec: S-0308 Import mapping (turret count, weapon count: engine default 0; clear flag: no)
        let tc = crate::art::int_as_of(t, tc_key, self.layer_now, 0);
        let wc = crate::art::int_as_of(t, wc_key, self.layer_now, 0);
        let clear = crate::art::bool_as_of(t, clear_key, self.layer_now, false);
        let mut keys: Vec<String> = Vec::new();
        if tc < 1 {
            // spec: S-0004 claim 13 (turret count below 1: the four standard fields, unless the flag is set)
            if !clear {
                keys.extend(refs::STANDARD_WEAPONS.iter().map(|k| (*k).to_owned()));
            }
        } else {
            // spec: S-0004 claim 13 (turret count 1 or more: Weapon1, EliteWeapon1, …, Weapon c, EliteWeapon c)
            let mut numbered: Vec<(u32, usize, String)> = Vec::new();
            for e in section.entries() {
                for (j, prefix) in refs::NUMBERED_WEAPONS.iter().enumerate() {
                    let Some(digits) = e.key.strip_prefix(prefix.as_bytes()) else {
                        continue;
                    };
                    let Some(n) = numbered_suffix(digits) else {
                        continue;
                    };
                    if i64::from(n) <= i64::from(wc) {
                        numbered.push((n, j, show(&e.key)));
                    }
                }
            }
            numbered.sort();
            numbered.dedup();
            keys.extend(numbered.into_iter().map(|(_, _, k)| k));
        }
        // spec: S-0004 Import mapping (DeathWeapon and the slot fields name weapons)
        let field = refs::Field {
            key: "",
            kind: Kind::Weapons,
            form: refs::Form::Single,
            limit: refs::REFERENCE_LIMIT,
            names: refs::Names::Standard,
            art: false,
        };
        for key in keys {
            let Some(entry) = section.get(key.as_bytes()) else {
                continue;
            };
            let (stored, line) = (entry.value.clone(), entry.line);
            let how = How::Reference {
                kind,
                name: referrer.to_vec(),
                key: key.clone(),
            };
            self.field(&field, &stored, hist, line, &how, Some((kind, index)));
        }
    }

    /// Finds or creates a type by reference (S-0004 claims 6 and 13).
    fn reference(
        &mut self,
        target: Kind,
        name: &[u8],
        hist: usize,
        line: u32,
        how: How,
        from: Option<(Kind, usize)>,
    ) {
        if self.find(target, name).is_some() {
            return;
        }
        // spec: S-0004 claims 4 and 6 (row 2 is read in the layer; row 3 only if created no later than its kind's pass; rows 4 to 6 not)
        let read_here = match self.phase {
            Phase::Early => true,
            Phase::Pass(k) => target.rank() >= k.rank(),
            Phase::Late => false,
        };
        let read_now = read_here || !self.quirks.unlisted_reference_unread;
        if read_now && self.repeats(target, name, from) {
            // spec: S-0004 claim 15 (departure: a repeated same-layer creation in a chain of creating references is suppressed, with a warning; no switch)
            self.warn(
                hist,
                line,
                format!(
                    "{} {:?} would create another {} of that stored name, which would read the same section and name it again without end; not created (S-0004 claim 15, a departure)",
                    how_key(&how),
                    show(name),
                    target.key()
                ),
            );
            return;
        }
        let li = self.layer_now;
        let i = self.create(target, name, li, how);
        self.origin.insert((target, i), (from, name.to_vec()));
        if !read_here && !self.quirks.unlisted_reference_unread {
            self.pending.push((target, i));
        }
    }

    /// Whether creating a type of `target` by `name` from `from` repeats a
    /// creation of the same chain in this layer: some type in the chain of
    /// creators of `from` (itself included), created in this layer, is of
    /// `target` and was created by the same name. Such a new type would
    /// get the same stored name, read the same section and create the next
    /// one in turn (S-0004 claims 8 and 15 with names longer than their
    /// stored form). Names are compared after their field's limit and
    /// trimming, without regard to letter case (claim 15).
    fn repeats(&self, target: Kind, name: &[u8], from: Option<(Kind, usize)>) -> bool {
        let mut cur = from;
        while let Some(c) = cur {
            if self.reg(c.0)[c.1].created_in != self.layer_now {
                return false;
            }
            let Some((parent, probe)) = self.origin.get(&c) else {
                return false;
            };
            if c.0 == target && value::type_name_eq(probe, name) {
                return true;
            }
            cur = *parent;
        }
        false
    }

    /// The creating fields of a game-wide section of this layer (S-0004
    /// claims 4 and 13).
    fn game_wide_refs(&mut self, g: &refs::GameWide, li: usize, doc: &Document) {
        let Some(section) = doc.section(g.section.as_bytes()) else {
            return;
        };
        let section = section.clone();
        for f in g.fields {
            let Some(entry) = section.get(f.key.as_bytes()) else {
                continue;
            };
            let how = How::GameWide {
                section: g.section,
                key: f.key,
            };
            self.field(f, &entry.value, li, entry.line, &how, None);
        }
    }
    fn barrels(&mut self, index: usize, li: usize, section: &Section, has: &dyn Fn(&str) -> bool) {
        if !self.quirks.barrel_recoil_layer_reset {
            return;
        }
        for (barrel, turret) in BARREL_KEYS {
            if has(barrel) {
                continue;
            }
            // spec: S-0105 claim 27 (an absent barrel value takes the turret's value after the layer)
            let turret_value = self.reg(Kind::Buildings)[index]
                .keys
                .get(turret.as_bytes())
                .and_then(|k| k.value.clone());
            let state = self.reg_mut(Kind::Buildings)[index]
                .keys
                .entry(barrel.as_bytes().to_vec())
                .or_default();
            state.value.clone_from(&turret_value);
            state.history.push(Event {
                layer: li,
                line: section.line(),
                what: What::FromTurret(turret_value),
            });
        }
    }

    fn building_lights(
        &mut self,
        index: usize,
        li: usize,
        section: &Section,
        earlier_reads: usize,
        given: &[(Vec<u8>, Vec<u8>, u32)],
    ) {
        for key in BUILDING_LIGHT_KEYS {
            let stored = given
                .iter()
                .find(|(k, _, _)| k.as_slice() == key.as_bytes());
            let held = self.reg(Kind::Buildings)[index]
                .keys
                .get(key.as_bytes())
                .and_then(|k| k.derived);
            let (derived, what, line) = match stored {
                Some((_, v, line)) => match building_light_thousandths(v) {
                    Some(t) => (Some(t), None, *line),
                    None => {
                        self.warn(
                            li,
                            *line,
                            format!(
                                "{key} {:?} has no number; the value in force is kept (S-0003 claim 8, a departure)",
                                show(v)
                            ),
                        );
                        (held, None, *line)
                    }
                },
                // spec: S-0104 claim 12 (re-reading a section truncates an absent light value)
                None if earlier_reads > 0 && self.quirks.reread_truncates_light_values => {
                    match held {
                        Some(t) => {
                            let t2 = truncate_building_light(t);
                            (Some(t2), Some(What::Truncated(t2)), section.line())
                        }
                        None => (None, None, 0),
                    }
                }
                None => (held, None, 0),
            };
            if derived.is_none() && what.is_none() {
                continue;
            }
            let state = self.reg_mut(Kind::Buildings)[index]
                .keys
                .entry(key.as_bytes().to_vec())
                .or_default();
            state.derived = derived;
            if let Some(what) = what {
                state.history.push(Event {
                    layer: li,
                    line,
                    what,
                });
            }
        }
    }

    fn tiberiums(&mut self, li: usize, doc: &Document, type_sections: &mut BTreeSet<usize>) {
        let Some(list) = Kind::Tiberiums.list_section().and_then(|s| doc.section(s)) else {
            return;
        };
        let entries: Vec<(Vec<u8>, Vec<u8>, u32)> = list
            .list_positions()
            .map(|(_, w)| (w.key.clone(), w.value.clone(), w.line))
            .collect();
        for (key, stored, line) in entries {
            // spec: S-0004 claim 10 (the value is cut to 23 bytes)
            let name = value::text(&stored, TIBERIUM_VALUE_MAX).to_vec();
            if name.is_empty() {
                continue;
            }
            let index = if self.quirks.tiberium_index_by_key {
                let k = tiberium_key(&key);
                let n = self.reg(Kind::Tiberiums).len();
                if k < 0 {
                    // spec: S-0004 claim 10 (a negative key: our engine ignores the entry; a departure)
                    self.warn(
                        li,
                        line,
                        format!(
                            "tiberium list key {:?} is negative; the entry is ignored (S-0004 claim 10, a departure)",
                            show(&key)
                        ),
                    );
                    continue;
                }
                if (k as usize) < n {
                    // spec: S-0004 claim 10 (an existing index: the name in the value is ignored)
                    k as usize
                } else {
                    // spec: S-0004 claim 10 (k >= n: a new tiberium at the next free index)
                    self.create(Kind::Tiberiums, &name, li, How::List)
                }
            } else {
                self.find_or_create(Kind::Tiberiums, &name, li, How::List).0
            };
            let tname = self.reg(Kind::Tiberiums)[index].name.clone();
            // spec: S-0004 claim 10 (each named tiberium reads its section at once)
            if let Some(si) = doc.section_index(&tname) {
                type_sections.insert(si);
                let section = doc.sections()[si].clone();
                self.read(Kind::Tiberiums, index, &section, li);
            }
        }
    }

    fn game_wide(&mut self, li: usize, layer: &Layer, type_sections: &BTreeSet<usize>) {
        let doc = &layer.doc;
        let lists: BTreeSet<&[u8]> = Kind::all()
            .filter_map(Kind::list_section)
            .chain([SIDES, COLORS, COLOR_ADD])
            .collect();
        // A section named after any registered type is that type's section,
        // read or not, never a game-wide one.
        let type_names: BTreeSet<Vec<u8>> = self
            .out
            .registries
            .iter()
            .filter(|(k, _)| **k != Kind::Animations)
            .flat_map(|(_, r)| r.iter().map(|t| t.name.clone()))
            .collect();
        for (si, section) in doc.sections().iter().enumerate() {
            if type_sections.contains(&si)
                || lists.contains(section.name())
                || type_names.contains(section.name())
                || doc.section_index(section.name()) != Some(si)
            {
                continue;
            }
            let name = section.name().to_vec();
            let mut seen: BTreeSet<&[u8]> = BTreeSet::new();
            let mut given: Vec<(Vec<u8>, Vec<u8>, u32)> = Vec::new();
            for e in section.entries() {
                if seen.insert(e.key.as_slice()) {
                    let w = section.get(&e.key).unwrap_or(e);
                    given.push((e.key.clone(), w.value.clone(), w.line));
                }
            }
            let quirks = self.quirks;
            let keys = self.out.globals.entry(name.clone()).or_default();
            for (key, stored, line) in &given {
                let base_only = (name.as_slice(), key.as_slice()) == MAXIMUMS_PLAYERS;
                let state = keys.entry(key.clone()).or_default();
                if base_only && layer.role != Role::Base {
                    // spec: S-0006 claim 2 (the maximum number of players comes from the base rules only)
                    state.history.push(Event {
                        layer: li,
                        line: *line,
                        what: What::Ignored(stored.clone()),
                    });
                    continue;
                }
                if global_value_not_given(&name, key, stored) {
                    // spec: S-0209, S-0210 and S-0215 Import mapping (a "not given" value keeps the held value)
                    state.history.push(Event {
                        layer: li,
                        line: *line,
                        what: What::NotGiven(stored.clone()),
                    });
                    continue;
                }
                state.value = Some(stored.clone());
                state.reset = false;
                state.history.push(Event {
                    layer: li,
                    line: *line,
                    what: What::Set(stored.clone()),
                });
                if name.as_slice() == AUDIO_VISUAL
                    && EXTRA_LIGHT_KEYS
                        .iter()
                        .any(|k| k.as_bytes() == key.as_slice())
                    && let Some(t) = extra_light_thousandths(stored, quirks.extra_light_truncation)
                {
                    state.derived = Some(t);
                }
            }
            let has = |k: &str| {
                given
                    .iter()
                    .any(|(key, _, _)| key.as_slice() == k.as_bytes())
            };
            // spec: S-0006 claim 5 (difficulty sections reset omitted keys)
            if quirks.override_resets_keys
                && DIFFICULTY_SECTIONS
                    .iter()
                    .any(|d| d.as_bytes() == name.as_slice())
            {
                for k in DIFFICULTY_KEYS {
                    if !has(k) {
                        let state = keys.entry(k.as_bytes().to_vec()).or_default();
                        state.value = None;
                        state.reset = true;
                        state.history.push(Event {
                            layer: li,
                            line: section.line(),
                            what: What::Reset,
                        });
                    }
                }
            }
            // spec: S-0006 claim 5 (the twelve land-type sections reset omitted keys: multipliers 100 %, buildable no), S-0211 claim 2
            if quirks.override_resets_keys
                && crate::sim::LAND_TYPES
                    .iter()
                    .any(|(_, l)| l.as_bytes() == name.as_slice())
            {
                let land_keys = crate::sim::LAND_MULTIPLIERS
                    .iter()
                    .map(|(_, k)| *k)
                    .chain([LAND_BUILDABLE]);
                for k in land_keys {
                    if !has(k) {
                        let state = keys.entry(k.as_bytes().to_vec()).or_default();
                        state.value = None;
                        state.reset = true;
                        state.history.push(Event {
                            layer: li,
                            line: section.line(),
                            what: What::Reset,
                        });
                    }
                }
            }
            // spec: S-0104 claims 12 and 13 (extra-light values truncated on re-read)
            if quirks.reread_truncates_light_values && name.as_slice() == AUDIO_VISUAL {
                for k in EXTRA_LIGHT_KEYS {
                    if has(k) {
                        continue;
                    }
                    if let Some(state) = keys.get_mut(k.as_bytes())
                        && let Some(t) = state.derived
                    {
                        let t2 = truncate_extra_light(t);
                        state.derived = Some(t2);
                        state.history.push(Event {
                            layer: li,
                            line: section.line(),
                            what: What::Truncated(t2),
                        });
                    }
                }
            }
        }
    }

    fn colours(&mut self, li: usize, doc: &Document) {
        let Some(section) = doc.section(COLORS) else {
            return;
        };
        for (entry, winner) in section.list_positions() {
            // spec: S-0103 claim 4 (colour-definition form)
            let hsv = value::colour_definition(&winner.value);
            let existing = self
                .out
                .colours
                .iter()
                .position(|c| c.name.eq_ignore_ascii_case(&entry.key));
            match existing {
                // spec: S-0103 claim 5 (the first definition of a name wins)
                Some(_) if self.quirks.colour_redefinition_ignored => {}
                Some(i) => {
                    let c = &mut self.out.colours[i];
                    c.hsv = hsv;
                    c.layer = li;
                    c.line = winner.line;
                }
                None => self.out.colours.push(ColourDef {
                    name: entry.key.clone(),
                    hsv,
                    layer: li,
                    line: winner.line,
                }),
            }
        }
    }

    fn colour_add(&mut self, li: usize, layer: &Layer) {
        if self.map_only_colour_add && layer.role != Role::Map {
            return;
        }
        let Some(section) = layer.doc.section(COLOR_ADD) else {
            return;
        };
        for (n, (_, winner)) in section.list_positions().enumerate() {
            if n >= COLOUR_ADD_LEN {
                // spec: S-0006 claim 2 (entries after the 16th: our engine ignores them; a departure)
                self.warn(
                    li,
                    winner.line,
                    "colour-adjustment entries after the 16th are ignored (S-0006 claim 2, a departure)".into(),
                );
                break;
            }
            // spec: S-0006 claim 2 (the language rules file cannot change a position the base rules give)
            if layer.role == Role::Language && self.out.colour_add[n].1.is_some_and(|(l, _)| l == 0)
            {
                continue;
            }
            // spec: S-0003 Import mapping row C7 (generic byte triple)
            let (rgb, unread) = value::byte_triple(&winner.value, self.out.colour_add[n].0);
            if unread {
                self.warn(
                    li,
                    winner.line,
                    format!(
                        "colour-adjustment entry {:?} has unreadable components; they keep the value in force (S-0003 claim 11, a departure)",
                        show(&winner.value)
                    ),
                );
            }
            self.out.colour_add[n] = (rgb, Some((li, winner.line)));
        }
    }

    fn finish(&mut self) {
        // spec: S-0004 claim 3 (at game start the fixed particle system is created if missing; it reads no section)
        if self
            .find(Kind::ParticleSystems, refs::GAS_CLOUD_SYSTEM)
            .is_none()
        {
            let li = self.layer_now;
            self.create(
                Kind::ParticleSystems,
                refs::GAS_CLOUD_SYSTEM,
                li,
                How::GameStart,
            );
        }
        // With the switch off, a barrel value that no layer sets is the turret's after the last layer.
        if !self.quirks.barrel_recoil_layer_reset {
            for t in self.reg_mut(Kind::Buildings) {
                for (barrel, turret) in BARREL_KEYS {
                    if t.keys
                        .get(barrel.as_bytes())
                        .is_none_or(|k| k.value.is_none())
                        && let Some(v) = t.keys.get(turret.as_bytes()).and_then(|k| k.value.clone())
                    {
                        let state = t.keys.entry(barrel.as_bytes().to_vec()).or_default();
                        state.value = Some(v.clone());
                        state.history.push(Event {
                            layer: 0,
                            line: 0,
                            what: What::FromTurret(Some(v)),
                        });
                    }
                }
            }
        }
    }
}

/// The key that created a type, for warnings.
fn how_key(how: &How) -> String {
    match how {
        How::Reference { kind, name, key } => format!("{} {} {key}", kind.key(), show(name)),
        How::GameWide { section, key } => format!("[{section}] {key}"),
        How::EmptyParticle | How::SideField | How::GameStart | How::List => String::new(),
    }
}

/// The number of a numbered weapon field's key (`Weapon`*n*): the key's
/// remainder written as a decimal number from 1, without a sign or leading
/// zeros (S-0004 Import mapping).
fn numbered_suffix(digits: &[u8]) -> Option<u32> {
    if digits.is_empty() || digits[0] == b'0' || !digits.iter().all(u8::is_ascii_digit) {
        return None;
    }
    std::str::from_utf8(digits).ok()?.parse().ok()
}

/// Whether a decimal stored value is zero (either sign).
pub fn is_zero(stored: &[u8]) -> bool {
    match value::decimal(stored, DecimalField::Double) {
        Decimal::Value { value, .. } => {
            let sign = match value.format {
                Format::Single => 1u64 << 31,
                Format::Double => 1u64 << 63,
            };
            value.bits & !sign == 0
        }
        Decimal::NoNumber => false,
    }
}

/// The key of a tiberium-list entry as an integer (S-0004 Formulas).
pub fn tiberium_key(key: &[u8]) -> i32 {
    let start = key
        .iter()
        .take_while(|&&b| glasnost_ini::lex::is_whitespace(b))
        .count();
    let s = &key[start..];
    let (neg, s) = match s.first() {
        Some(b'-') => (true, &s[1..]),
        Some(b'+') => (false, &s[1..]),
        _ => (false, s),
    };
    let mut d = 0u32;
    for &b in s.iter().take_while(|b| b.is_ascii_digit()) {
        // spec: S-0004 Formulas (d = digits read in base 10; wrap32 of σ × d)
        d = d.wrapping_mul(10).wrapping_add(u32::from(b - b'0'));
    }
    (if neg { d.wrapping_neg() } else { d }) as i32
}
