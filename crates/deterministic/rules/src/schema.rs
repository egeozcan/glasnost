//! Schema 1 of our rules format (PLAN.md 4.6): the typed form of a resolved
//! ruleset.
//!
//! Two parts share one document:
//!
//! - **Our own typed data** (`owners`, `armor_classes`, `objects`,
//!   `weapons`, `warheads`, `globals`), written by hand or by mods. Its
//!   fields carry no absent-key defaults yet: each default arrives with the
//!   spec that defines the field, and cites it.
//! - **Imported data** (`import`, `registries`, `types`, `art_sections`,
//!   `ini_globals`, `sides`, `colours`, `colour_add`), written by the
//!   converter from the user's own install. Keys that no spec maps to a
//!   typed field yet stay as the stored INI text (`ini` maps), so nothing is
//!   lost and mods can patch them.
//!
//! The shape is our design; only the imported *values* follow the specs.
//! Which parts count towards `sim_rules_hash` is decided by path in
//! [`crate::hash`].

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::Deserialize;

/// The schema version this crate reads.
pub const SCHEMA_VERSION: u32 = 1; // nocite: our own format's version number

/// A resolved ruleset.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Ruleset {
    /// Format version; must be [`SCHEMA_VERSION`].
    pub schema: u32,
    /// Where imported data came from (informational, not hashed).
    #[serde(default)]
    pub import: Option<Import>,
    /// Imported type names per kind, in registration order (S-0004): the
    /// position is the registration index.
    #[serde(default)]
    pub registries: BTreeMap<String, Vec<String>>,
    /// Imported types per kind, by type name.
    #[serde(default)]
    pub types: BTreeMap<String, BTreeMap<String, ImportedType>>,
    /// The art database's sections as stored (S-0007), by section name.
    #[serde(default)]
    pub art_sections: BTreeMap<String, BTreeMap<String, String>>,
    /// Imported game-wide sections, layered (S-0006), by section name.
    #[serde(default)]
    pub ini_globals: BTreeMap<String, BTreeMap<String, String>>,
    /// Game-wide values the layering computed rather than copied (S-0006
    /// claim 5: extra-light values truncated on re-read, in thousandths),
    /// by section name; such keys are absent from `ini_globals`.
    #[serde(default)]
    pub ini_globals_derived: BTreeMap<String, BTreeMap<String, i64>>,
    /// Imported sides in index order (S-0004 claim 9).
    #[serde(default)]
    pub sides: Vec<Side>,
    /// Imported colour definitions in definition-position order (S-0103).
    #[serde(default)]
    pub colours: Vec<ColourDefinition>,
    /// The colour-adjustment table (S-0006 claim 2): 16 red, green, blue
    /// tints.
    // spec: S-0006 claim 2 (each entry is red, green, blue)
    #[serde(default)]
    pub colour_add: Vec<[u8; 3]>,
    /// Owners (our own format).
    #[serde(default)]
    pub owners: BTreeMap<String, Owner>,
    /// Armour classes (our own format).
    #[serde(default)]
    pub armor_classes: BTreeMap<String, Empty>,
    /// Objects (our own format).
    #[serde(default)]
    pub objects: BTreeMap<String, Object>,
    /// Weapons (our own format).
    #[serde(default)]
    pub weapons: BTreeMap<String, Weapon>,
    /// Warheads (our own format).
    #[serde(default)]
    pub warheads: BTreeMap<String, Warhead>,
    /// Game-wide settings (our own format).
    #[serde(default)]
    pub globals: Globals,
    /// Simulation data: typed fields of the user's data and our own
    /// (converted by the importer, patchable by mods).
    #[serde(default)]
    pub sim: crate::sim::SimData,
}

/// An empty object.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Empty {}

/// Provenance of imported data.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Import {
    /// Converter name and version.
    pub converter: String,
    /// Kind of game the layers were applied for (`skirmish` or `campaign`).
    pub game: String,
    /// Theater the art was looked up for.
    #[serde(default)]
    pub theater: Option<String>,
    /// Input files in layer order.
    #[serde(default)]
    pub inputs: Vec<Input>,
    /// Quirk switches in force, by name.
    #[serde(default)]
    pub quirks: BTreeMap<String, bool>,
    /// Kinds whose registries may lack types created by reference that no
    /// approved spec describes yet (Q-0005).
    #[serde(default)]
    pub incomplete_registries: Vec<String>,
}

/// One input file.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// Its role (`base rules`, `art database`, ...).
    pub role: String,
    /// File name.
    pub file: String,
    /// Where it was found: an archive path or `loose`.
    pub origin: String,
    /// SHA-256 of its bytes, hexadecimal.
    pub sha256: String,
}

/// An imported type.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ImportedType {
    /// Registration index (S-0004).
    pub index: u32,
    /// What created it: `list`, or the reference that did.
    pub created: String,
    /// The layers whose section it read (S-0006 claim 7).
    #[serde(default)]
    pub reads: Vec<String>,
    /// Layered keys as stored INI text (S-0006 claims 3 and 4).
    #[serde(default)]
    pub ini: BTreeMap<String, String>,
    /// Keys that hold their reset value (S-0006 claim 5); the value is the
    /// engine default the owning spec states.
    #[serde(default)]
    pub reset: Vec<String>,
    /// Values the layering computed rather than copied (S-0006 claim 5:
    /// light values in thousandths, truncated on re-read); such keys are
    /// absent from `ini`.
    #[serde(default)]
    pub derived: BTreeMap<String, i64>,
    /// For a country: its side index (S-0004 claim 9).
    #[serde(default)]
    pub side: Option<u32>,
    /// Art looked up for the import's theater (render data).
    #[serde(default)]
    pub art: Option<Art>,
}

/// Art looked up for a type (S-0007), for one theater. Render data.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Art {
    /// Image name (claims 1, 9, 10).
    pub image: String,
    /// The art section's flags in force.
    #[serde(default)]
    pub flags: Vec<String>,
    /// Sprite file seen in play, if any (claims 5 to 15).
    #[serde(default)]
    pub sprite: Option<String>,
    /// Voxel parts found, by role (claims 16 to 20).
    #[serde(default)]
    pub voxels: BTreeMap<String, String>,
    /// Other sprites of a building, by role (claim 11).
    #[serde(default)]
    pub building_sprites: BTreeMap<String, String>,
    /// Main sidebar icon (claim 22).
    #[serde(default)]
    pub icon: Option<String>,
    /// Alternate sidebar icon (claim 25).
    #[serde(default)]
    pub alt_icon: Option<String>,
    /// Translucent sprite (claim 23).
    #[serde(default)]
    pub translucent: Option<String>,
    /// Notes on cases the specs leave open.
    #[serde(default)]
    pub notes: Vec<String>,
}

/// An imported side.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Side {
    /// Side name.
    pub name: String,
    /// Member countries in list order.
    #[serde(default)]
    pub members: Vec<String>,
}

/// An imported colour definition (S-0103 claim 4).
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ColourDefinition {
    /// Name.
    pub name: String,
    /// Hue, saturation, value.
    // spec: S-0103 Terms (a colour definition is three bytes H, S, V)
    pub hsv: [u8; 3],
}

/// An owner (our own format).
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Owner {}

/// A kind of object (our own format).
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ObjectKind {
    /// Infantry.
    Infantry,
    /// Vehicle.
    Vehicle,
    /// Aircraft.
    Aircraft,
    /// Building.
    Building,
}

/// An object (our own format). Fields without a spec carry no default.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Object {
    /// Kind.
    #[serde(default)]
    pub kind: Option<ObjectKind>,
    /// Armour class.
    #[serde(default)]
    pub armor: Option<String>,
    /// Cost in credits.
    #[serde(default)]
    pub cost: Option<i64>,
    /// Health in hit points.
    #[serde(default)]
    pub health: Option<i64>,
    /// Speed in leptons per tick.
    #[serde(default)]
    pub speed: Option<i64>,
    /// Sight, a decimal with a unit (`"4c"`).
    #[serde(default)]
    pub sight: Option<String>,
    /// Owners that may build it (a set).
    #[serde(default)]
    pub owners: Vec<String>,
    /// Prerequisites (a set).
    #[serde(default)]
    pub prerequisites: Vec<String>,
    /// Weapons by slot.
    #[serde(default)]
    pub armament: BTreeMap<String, Armament>,
}

/// One armament slot.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Armament {
    /// Weapon.
    pub weapon: String,
}

/// A weapon (our own format).
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Weapon {
    /// Damage in hit points.
    #[serde(default)]
    pub damage: Option<i64>,
    /// Reload in ticks.
    #[serde(default)]
    pub reload: Option<i64>,
    /// Range, a decimal with a unit.
    #[serde(default)]
    pub range: Option<String>,
    /// Warhead.
    #[serde(default)]
    pub warhead: Option<String>,
}

/// A warhead (our own format).
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Warhead {
    /// Modifier per armour class, per mille.
    #[serde(default)]
    pub versus: BTreeMap<String, i64>,
}

/// Game-wide settings (our own format).
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Globals {
    /// Economy.
    #[serde(default)]
    pub economy: BTreeMap<String, i64>,
    /// Production.
    #[serde(default)]
    pub production: BTreeMap<String, i64>,
    /// Quirk switches of the simulation; absent ones take the original's
    /// behaviour.
    #[serde(default)]
    pub quirks: crate::quirks::Quirks,
}

/// A mod or ruleset manifest.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// Schema version the layers are written against.
    pub schema: u32,
    /// Name.
    pub name: String,
    /// Layer files in application order.
    pub layers: Vec<String>,
}

/// The JSON Schema of [`Ruleset`], pretty-printed.
pub fn json_schema() -> String {
    let schema = schemars::schema_for!(Ruleset);
    let mut s = serde_json::to_string_pretty(&schema).unwrap_or_default();
    s.push('\n');
    s
}
