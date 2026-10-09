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

/// Quirk switches of S-0004, S-0006, S-0103 claim 5, S-0104 claim 12 and
/// S-0105 claim 27 that this pass implements. Each defaults to the
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
        }
    }
}

impl LayerQuirks {
    /// The switches by name, for the import record.
    pub fn named(&self) -> [(&'static str, bool); 10] {
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
        key: &'static str,
    },
    /// A game-wide section (S-0004 claim 4, row 4).
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

/// Building light keys stored as ⌊v × 1000 + 0.1⌋ (S-0104 claim 12).
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
/// The crate rules and the overlays they name (S-0004 Import mapping).
const CRATE_RULES: &str = "CrateRules"; // spec: S-0004 Import mapping
// spec: S-0004 Import mapping (crate overlays, created by reference in the late phase)
const CRATE_KEYS: [&str; 3] = ["CrateImg", "WoodCrateImg", "WaterCrateImg"];
/// The side list (S-0004 Import mapping).
const SIDES: &[u8] = b"Sides"; // spec: S-0004 Import mapping
/// The colour definitions (S-0103 Import mapping).
const COLORS: &[u8] = b"Colors"; // spec: S-0103 Import mapping
/// The colour-adjustment section (S-0006 Import mapping).
const COLOR_ADD: &[u8] = b"ColorAdd"; // spec: S-0006 Import mapping
/// The maximum number of players, read from the base rules only (S-0006
/// claim 2).
const MAXIMUMS_PLAYERS: (&[u8], &[u8]) = (b"Maximums", b"Players"); // spec: S-0006 Import mapping

/// ⌊v × 1000 + 0.1⌋ truncated toward zero, in double precision rounded
/// toward zero (S-0104 claim 12, Formulas "Precision of decimal
/// conversions"). `None` when the value has no number (S-0003 claim 8: our
/// engine keeps the value in force).
pub fn building_light_thousandths(stored: &[u8]) -> Option<i64> {
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

/// ⌊v × 1000⌋ truncated toward zero (S-0104 Import mapping, extra-light
/// values).
pub fn extra_light_thousandths(stored: &[u8]) -> Option<i64> {
    let Decimal::Value { value, .. } = value::decimal(stored, DecimalField::Double) else {
        return None;
    };
    value
        .mul_rz(&Binary::from_i64(Format::Double, PER_MILLE), Format::Double)
        .trunc_to_i64()
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
        art_layer,
        pending: Vec::new(),
        // spec: S-0006 claim 12 (the table is left with zero in all 16 positions; only the map's entries count)
        map_only_colour_add: bad_language_rules && quirks.bad_language_rules_skips_mode,
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

struct Run<'a> {
    out: Outcome,
    quirks: LayerQuirks,
    art: &'a Document,
    art_layer: usize,
    /// Types created by reference that read the creating layer anyway
    /// (switch `quirks.unlisted_reference_unread` off).
    pending: Vec<(Kind, usize)>,
    /// Only the map layer fills the colour-adjustment table (S-0006 claim
    /// 12).
    map_only_colour_add: bool,
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

    fn layer(&mut self, li: usize, layer: &Layer) {
        let doc = &layer.doc;
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
        // S-0004 claim 6: reading passes in rank order.
        let mut type_sections: BTreeSet<usize> = BTreeSet::new();
        for kind in Kind::PASSES {
            self.pass(kind, li, doc, &mut type_sections);
        }
        // S-0004 claim 4, row 4: the late game-wide sections; only the crate rules create types here.
        self.crate_rules(li, doc);
        // spec: S-0004 claim 6 (switch off: a type created by reference reads the creating layer)
        while let Some((kind, i)) = self.pending.pop() {
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
        // S-0004 claim 6: a layer with the special-weapon settings gives every warhead a second reading.
        if doc.section(SPECIAL_WEAPONS).is_some() {
            for i in 0..self.reg(Kind::Warheads).len() {
                let name = self.reg(Kind::Warheads)[i].name.clone();
                if let Some(si) = doc.section_index(&name) {
                    type_sections.insert(si);
                    let section = &doc.sections()[si];
                    self.read(Kind::Warheads, i, section, li);
                }
            }
        }
        // S-0004 claim 10: the tiberium list at the end of the layer.
        self.tiberiums(li, doc, &mut type_sections);
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
            for item in value::list(&winner.value, value::COMMON_TEXT_LIMIT) {
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
            let what = if kind == Kind::Superweapons
                && key.as_slice() == b"RechargeTime"
                && is_zero(stored)
            {
                // spec: S-0006 claim 3 (a recharge time of 0 is "not given")
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

        // References the specs name (S-0004 claims 2, 3 and 6, Import mapping).
        self.references(kind, index, li, &given, has("HoldsWhat"));

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

    fn references(
        &mut self,
        kind: Kind,
        index: usize,
        li: usize,
        given: &[(Vec<u8>, Vec<u8>, u32)],
        holds_what_given: bool,
    ) {
        let referrer = self.reg(kind)[index].name.clone();
        // spec: S-0004 Import mapping (reference fields used in the scenarios)
        let refs: &[(&'static str, Kind, bool)] = match kind {
            Kind::Vehicles => &[("DeploysInto", Kind::Buildings, true)],
            Kind::Projectiles => &[
                ("AirburstWeapon", Kind::Weapons, true),
                ("ShrapnelWeapon", Kind::Weapons, true),
            ],
            Kind::ParticleSystems => &[("HoldsWhat", Kind::Particles, true)],
            // spec: S-0004 claim 3 (an unknown superweapon means "none"; find only)
            Kind::Buildings => &[("SuperWeapon", Kind::Superweapons, false)],
            _ => &[],
        };
        for &(key, target, creates) in refs {
            let Some((_, stored, line)) = given
                .iter()
                .find(|(k, _, _)| k.as_slice() == key.as_bytes())
            else {
                continue;
            };
            let name = value::text(stored, value::COMMON_TEXT_LIMIT);
            if target == Kind::Particles {
                // spec: S-0004 claim 2 (for the emitted particle only `<none>` means no type)
                if name.eq_ignore_ascii_case(b"<none>") {
                    continue;
                }
                if !self.quirks.particle_none_is_a_name && value::is_no_type(name) {
                    continue;
                }
            } else if value::is_no_type(name) {
                continue;
            }
            if !creates {
                if self.find(target, name).is_none() {
                    self.warn(
                        li,
                        *line,
                        format!(
                            "{} {} names the unknown {} {:?}, which means none (S-0004 claim 3)",
                            kind.key(),
                            show(&referrer),
                            target.key(),
                            show(name)
                        ),
                    );
                }
                continue;
            }
            let how = How::Reference {
                kind,
                name: referrer.clone(),
                key,
            };
            self.reference(target, name, li, how, kind);
        }
        // spec: S-0006 claim 5 (an omitted emitted particle is the particle with the empty name)
        if kind == Kind::ParticleSystems && !holds_what_given && self.quirks.override_resets_keys {
            self.reference(Kind::Particles, b"", li, How::EmptyParticle, kind);
        }
    }

    /// Creates or finds a type by reference during `during`'s pass.
    fn reference(&mut self, target: Kind, name: &[u8], li: usize, how: How, during: Kind) {
        let (i, created) = self.find_or_create(target, name, li, how);
        // spec: S-0004 claim 6 (a type created during a later-ranked pass is not read in this layer)
        if created && !self.quirks.unlisted_reference_unread && target.rank() < during.rank() {
            self.read_now(target, i, li);
        }
    }

    /// Queues a type created by reference for a reading of the creating
    /// layer's section after that layer's passes (the switch-off form of
    /// S-0004 claim 6).
    fn read_now(&mut self, kind: Kind, index: usize, _li: usize) {
        self.pending.push((kind, index));
    }

    fn crate_rules(&mut self, li: usize, doc: &Document) {
        let Some(section) = doc.section(CRATE_RULES.as_bytes()) else {
            return;
        };
        for key in CRATE_KEYS {
            let Some(stored) = section.value(key.as_bytes()) else {
                continue;
            };
            let name = value::text(stored, value::COMMON_TEXT_LIMIT);
            if name.is_empty() || value::is_no_type(name) {
                continue;
            }
            let how = How::GameWide {
                section: CRATE_RULES,
                key,
            };
            let (i, created) = self.find_or_create(Kind::Overlays, name, li, how);
            // spec: S-0004 claim 6 (types first named by the late sections are not read in that layer)
            if created && !self.quirks.unlisted_reference_unread {
                self.pending.push((Kind::Overlays, i));
            }
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
                    && let Some(t) = extra_light_thousandths(stored)
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

/// Whether a decimal stored value is zero (either sign).
fn is_zero(stored: &[u8]) -> bool {
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
