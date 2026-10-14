//! Builds the [`Scene`] of a decoded map: every tile, overlay, terrain
//! object, building and unit placed in world pixels in draw order, with
//! its depth (S-0102), palette row (S-0103, S-0104) and image frame
//! (S-0107, S-0108, S-0105).
//!
//! The map's initial state is drawn: no tick has run, so buildings are in
//! their idle state at stage 0 and infantry show the *stand* entry at stage
//! 0. Choices that no approved spec makes are marked provisional with their
//! request, and listed in [`Built::notes`].

use std::collections::{BTreeMap, BTreeSet};

use glasnost_convert::Import;
use glasnost_convert::kinds::Kind;
use glasnost_convert::theater::{
    Catalogue, TheaterQuirks, alternate_image_mark, variant_image, variant_pattern,
};
use glasnost_fmt::hva::Hva;
use glasnost_fmt::image::IndexedImage;
use glasnost_fmt::shp::Shp;
use glasnost_fmt::tmp::Tmp;
use glasnost_fmt::vpl::Vpl;
use glasnost_fmt::vxl::Vxl;
use glasnost_ini::Document;
use glasnost_ini::value;
use glasnost_map::objects::{HealthKind, ListKind, facing_from_value, starting_health};
use glasnost_map::{CellGrid, CellXY, Map, NO_OVERLAY};
use glasnost_num::F64;
use glasnost_num::geom::n_step;
use glasnost_render::compose::{DIAMOND_WIDTH, DepthTest, Paint, Profile, Shade, row565};

use glasnost_render::light::{self, DetailLevel, LightQuirks, LightSource, TileTable};
use glasnost_render::overlay::{
    DECK_BRIDGES, Flags, OverlayQuirks, Row as OverlayRow, Run, apply_image, deck_frame,
    draw_shift, table_row as overlay_table_row, tiberium_body, tiberium_of,
};
use glasnost_render::palette::{PaletteState, TINT_EXEMPT, house_palette};
use glasnost_render::projection::{
    LEVEL_LEPTONS, TERRAIN_LIFT, cell_centre, cell_top, height_lift, project_lifted,
    screen_to_ground,
};
use glasnost_render::scene::{Group, Item, ItemDepth, Scene};
use glasnost_render::voxel::{self, VoxelQuirks};
use glasnost_sim::rules::{Kind as SimKind, SimRules};

use super::quirks::{
    ViewQuirks, cliff_factor, drawn_body_frame, occupiable_hurt, overlay_depth_lift,
    sprite_vehicle_facing_part,
};
use super::slots;

/// File bytes read for a scene, by upper-case name.
#[derive(Clone, Debug, Default)]
pub struct Assets {
    files: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl Assets {
    /// Adds a file.
    pub fn insert(&mut self, name: &[u8], bytes: Vec<u8>) {
        self.files.insert(name.to_ascii_uppercase(), bytes);
    }

    /// A file by name, ignoring case.
    pub fn get(&self, name: &[u8]) -> Option<&[u8]> {
        self.files
            .get(&name.to_ascii_uppercase())
            .map(Vec::as_slice)
    }

    /// Number of files.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Whether no file is held.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

/// Viewer choices for a scene (none of them is game data).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    /// Detail level (S-0104): building lights need 2.
    pub detail: DetailLevel,
    /// Skirmish view: drop objects whose owner no skirmish house carries
    /// (S-0009 claim 27).
    pub skirmish: bool,
    /// Lobby colour slot of the human placeholder house (S-0103 claim 10).
    pub human_slot: u8,
    /// Lobby colour slot of the computer placeholder house.
    pub computer_slot: u8,
    /// Lobby colour slots by house name, ahead of the placeholder slots
    /// (a skirmish of the game client: S-0103 claim 10, S-0412 claim 7).
    pub slots: Vec<(Vec<u8>, u8)>,
    /// Colour definitions (H, S, V) of the map's own owners outside a
    /// skirmish, by house name ([`map_owner_colours`], S-0103 claim 14).
    pub map_colours: Vec<(Vec<u8>, [u8; 3])>,
    /// The map's owners outside a skirmish that match no house the map
    /// creates (S-0009 claim 48): drawn in our grey and noted.
    pub unresolved_owners: Vec<Vec<u8>>,
    /// The game a loaded map starts.
    pub game: GameMode,
}

/// The game the viewer starts on a loaded map (our own viewer choice).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GameMode {
    /// One house per object owner, every owner's objects placed
    /// (`glasnost_sim::load::sandbox_setup`).
    #[default]
    Sandbox,
    /// A two-house skirmish (`econ::setup::skirmish`): the user's house
    /// and a computer house played by `glasnost_ai`'s script, each
    /// starting with its base unit.
    Skirmish,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            detail: DetailLevel::High,
            skirmish: true,
            human_slot: 0,
            computer_slot: 1,
            slots: Vec::new(),
            map_colours: Vec::new(),
            unresolved_owners: Vec::new(),
            game: GameMode::Sandbox,
        }
    }
}

/// What the viewer shows about one object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectInfo {
    /// Kind label.
    pub kind: &'static str,
    /// Type name.
    pub type_name: String,
    /// Owner as written.
    pub owner: String,
    /// Cell.
    pub cell: CellXY,
    /// The map entry as written, or the overlay's stream values.
    pub entry: String,
    /// Notes (provisional choices that apply to it).
    pub notes: Vec<String>,
}

/// A built scene.
#[derive(Clone, Debug, Default)]
pub struct Built {
    /// The scene.
    pub scene: Scene,
    /// Objects, indexed by [`Item::object`].
    pub objects: Vec<ObjectInfo>,
    /// Provisional choices and gaps that this map met.
    pub notes: BTreeSet<String>,
    /// Objects dropped (unknown type, no art, or an owner no skirmish house
    /// carries), and cells drawn without a tile image.
    pub dropped: BTreeMap<String, usize>,
    /// With a live simulation: the static objects' ground-layer entries
    /// (building voxel parts), not merged into the items.
    pub ground: Vec<GroundEntry>,
    /// With a live simulation: the static objects' entries of the other
    /// layers (S-0102 claim 7: never sorted, in the order they entered).
    pub layered: Vec<(slots::Layer, GroundEntry)>,
    /// Each drawn cell's shown tile image: its file and the tile cell's
    /// index there (S-0010 claims 13 and 24), for the tile radar colour.
    pub shown_tiles: BTreeMap<CellXY, (Vec<u8>, usize)>,
}

/// The fixed house names of S-0009 Import mapping (claim 27).
const NEUTRAL: &[u8] = b"Neutral"; // spec: S-0009 Import mapping (house names)
const SPECIAL: &[u8] = b"Special"; // spec: S-0009 Import mapping (house names)
const HUMAN: &[u8] = b"<human player>"; // spec: S-0009 Import mapping (house names)
const COMPUTER: &[u8] = b"Computer"; // spec: S-0009 Import mapping (house names)
/// The colour name both non-player houses show (S-0103 claim 10).
const LIGHT_GREY: &[u8] = b"LightGrey"; // spec: S-0103 Import mapping
/// Rows of a house-colour palette's shade table (S-0103 claim 5).
const HOUSE_ROWS: usize = 53; // spec: S-0103 Constants (shade counts)
/// White tint in per-mille (S-0104 claim 3).
const WHITE: [i32; 3] = [1000, 1000, 1000]; // spec: S-0104 claim 3 (house-colour palettes are shaded with white light)
/// Ore and gem overlays are drawn at brightness 1000 (S-0104 claim 13).
const NEUTRAL_BRIGHTNESS: i32 = 1000; // spec: S-0104 claim 13 (ore and gem overlays)
/// The veinhole marker overlay index (S-0009 claim 14).
/// Rows an ore-spawning terrain object is drawn higher (S-0102 claim 23).
const ORE_SPAWNER_RAISE: i32 = 16; // spec: S-0102 Constants (ore-spawning terrain object raise)
/// Height levels a bridge deck adds to an overlay's depth (S-0102 claim 15).
const DECK_LEVELS: i32 = 4; // spec: S-0102 Constants (overlay depth bias: level + 4 for bridge decks)
/// Bridge deck height in levels (S-0102 claim 15).
const SITUATIONAL: i32 = -1; // spec: S-0102 claim 14 (s = −1 in the ordinary case)
/// A cliff cell is more than this many levels higher (S-0102 claim 14).
const CLIFF_LEVELS: i32 = 3; // spec: S-0102 Constants (cliff fudge test)
/// Absent-key default of the cliff fudge (S-0102 claim 14).
const DEFAULT_CLIFF_FUDGE: i32 = 10; // spec: S-0102 claim 14 (absent fudge values: cliff 10)
/// Default light radius of building lights (S-0104 claim 12).
const DEFAULT_LIGHT_RADIUS: i32 = 5000; // spec: S-0104 Constants (building light radius default)
/// Voxel lighting table file (S-0105 Import mapping).
pub const VPL_FILE: &[u8] = b"voxels.vpl"; // spec: S-0105 Import mapping

/// The notes a scene can carry: gaps in the specs (with their request) and
/// parts our viewer does not draw yet.
mod notes {
    pub const LIVE_SPRITE_VEHICLE: &str = "vehicles drawn as sprites keep no walk counter in the live game: a moving one is drawn with its standing frame, not the moving row (S-0102 claim 22)";
    pub const TILES_UNLOADED: &str = "no game was set up: tiles are drawn as the map stores them, before the load's terrain recalculation and smoothing (S-0010 claim 12)";
    pub const RUBBLE: &str = "rubble overlays outside a rubble cell draw nothing (S-0112 claim 5); rubble cells arise only in play";
    pub const SMUDGE_CLIP: &str = "a smudge is painted whole once per covered cell, not clipped to that cell (S-0102 claim 25): it differs only where covered cells differ in level or light";
    pub const TERRAIN_STRENGTH: &str = "the rules did not resolve: a terrain object type without a strength key is drawn with image frame 0 (its strength from the game-wide tree strength, S-0102 claim 23, needs the typed rules)";
    pub const ORE_MINE_STAGE: &str = "an ore mine's animation (S-0401 claim 15) is not played: it shows image frame 0, its stage at placement (S-0102 claim 23)";
    pub const VOXEL_SHADOW_POINT: &str = "voxel shadows are placed at the object's ground point (the locomotor's shadow point has no spec)";
    pub const SLOT_ANIMS_STILL: &str = "building slot animations (S-0108 claim 11) are drawn at their first image frame: animation playback (S-0109) is not implemented, so they do not advance and power (S-0108 claim 13) does not change them";
    pub const SLOT_ANIM_DRAW_POINT: &str = "a slot animation's draw point is taken as the screen pixel of its position moved down by its type's vertical draw shift (no spec gives an animation's draw point, S-0102 claim 19)";
    pub const SLOT_ANIM_TRANSLUCENT: &str = "translucent slot animations are drawn opaque: the blend of a translucency code (S-0109 claim 20) has no spec";
    pub const SLOT_ANIM_TILED: &str = "tiled (stacked) slot animations are drawn as one copy: the number of stacked copies (S-0102 claim 13) has no spec";
    pub const SLOT_ANIM_SHADOW_ORDER: &str = "a slot animation's shadow image frame is drawn before its image: their order (S-0109 claim 10) has no spec";
    pub const NO_COMPANION: &str =
        "building depth images (S-0102 F5): the shared file is not named by a spec: not used";
    pub const NO_RULES_HEALTH: &str = "the rules did not resolve: buildings are drawn undamaged (S-0009 claim 37 needs their strengths)";
    pub const UPGRADES: &str = "upgrades a map entry places (S-0009 claim 40) are not drawn: the building upgrade slots of S-0108 need animation playback";
    pub const AIRCRAFT_AIR: &str = "an aircraft on a cell outside the usable area is drawn landed: its flight height has no spec (S-0009 claim 38)";
    pub const NO_VPL: &str = "no voxel lighting table, or one shorter than its header and palette (S-0013 claim 4): every row reads as zero, so every voxel writes a transparent pixel (S-0105 claims 19 and 38)";
    pub const VPL_REJECTED: &str = "the voxel lighting table is missing or short and quirks.vpl_unloaded_rows_blank is off: the importer rejects it (S-0105 claim 38)";
    pub const SHORT_VPL: &str = "the voxel lighting table ends before its last row, or has more than 128 rows: missing rows read as zero (S-0105 claim 38, S-0013 claims 4 and 6)";
    pub const FEW_VPL_ROWS: &str = "the voxel lighting table declares fewer than 32 rows: the undeclared rows read as zero (S-0013 Import mapping)";
    pub const TRAILING_VPL: &str = "the voxel lighting table has bytes after its declared rows: they are ignored (S-0013 claim 5)";
    pub const OWNER_COLOUR_OURS: &str = "objects of an owner that matches no house the map creates are drawn in our grey: the original drops them, so they have no original colour (S-0009 claim 48, S-0103 claim 14)";
    pub const WATER_FILL: &str = "water-fill variants are drawn by the game load (S-0009 claim 6, S-0202): without a game the base water tile is drawn";
}

fn upper(name: &[u8]) -> Vec<u8> {
    name.to_ascii_uppercase()
}

/// The registration index of `name` in `kind` (names compared as S-0003
/// compares type names).
pub fn find_type(import: &Import, kind: Kind, name: &[u8]) -> Option<usize> {
    import
        .outcome
        .registries
        .get(&kind)?
        .iter()
        .position(|t| value::type_name_eq(&t.name, name))
}

/// A rules key of a type, as layered.
fn rule<'a>(import: &'a Import, kind: Kind, idx: usize, key: &[u8]) -> Option<&'a [u8]> {
    import
        .outcome
        .registries
        .get(&kind)?
        .get(idx)?
        .keys
        .get(key)?
        .value
        .as_deref()
}

fn rule_bool(import: &Import, kind: Kind, idx: usize, key: &[u8], default: bool) -> bool {
    rule(import, kind, idx, key)
        .and_then(value::boolean)
        .unwrap_or(default)
}

pub(crate) fn rule_int(import: &Import, kind: Kind, idx: usize, key: &[u8], default: i32) -> i32 {
    rule(import, kind, idx, key)
        .and_then(value::integer)
        .unwrap_or(default)
}

fn derived(import: &Import, kind: Kind, idx: usize, key: &[u8]) -> Option<i64> {
    import
        .outcome
        .registries
        .get(&kind)?
        .get(idx)?
        .keys
        .get(key)?
        .derived
}

/// The colour definition an owner shows (S-0103 claim 10), and whether
/// the choice is ours (an owner no skirmish house carries).
pub fn owner_definition(import: &Import, options: &Options, owner: &[u8]) -> ([u8; 3], bool) {
    let colours = &import.outcome.colours;
    let by_name = |n: &[u8]| {
        colours
            .iter()
            .find(|c| c.name.eq_ignore_ascii_case(n))
            .map(|c| c.hsv)
    };
    let by_slot = |s: u8| {
        glasnost_render::palette::slot_position(s)
            .and_then(|p| colours.get(p))
            .map(|c| c.hsv)
    };
    let grey = by_name(LIGHT_GREY).unwrap_or([0, 0, 200]);
    let named = options
        .slots
        .iter()
        .find(|(n, _)| n.as_slice() == owner)
        .map(|(_, s)| *s);
    let imported = options
        .map_colours
        .iter()
        .find(|(n, _)| n.as_slice() == owner)
        .map(|(_, hsv)| *hsv);
    if options
        .unresolved_owners
        .iter()
        .any(|n| n.as_slice() == owner)
    {
        // spec: S-0009 claim 48, S-0103 claim 14 (an unresolved owner has no original colour: ours, marked)
        (grey, true)
    } else if let Some(s) = named {
        // spec: S-0103 claim 10 (a house's lobby colour slot picks its colour definition)
        (by_slot(s).unwrap_or(grey), false)
    } else if let Some(hsv) = imported {
        // spec: S-0103 claim 14 (a map-defined owner shows its imported colour definition)
        (hsv, false)
    } else if owner == NEUTRAL || owner == SPECIAL {
        (grey, false)
    } else if owner == HUMAN {
        (by_slot(options.human_slot).unwrap_or(grey), false)
    } else if owner == COMPUTER {
        (by_slot(options.computer_slot).unwrap_or(grey), false)
    } else {
        (grey, true)
    }
}

/// An owner's display RGB for the radar (S-0103 claim 15, S-0410 claim 8):
/// palette index 16, the brightest ramp entry of its colour definition,
/// at shade row 26 under ordinary white tint, each component *u* reduced
/// to ⌊max(0, *u* − 1) / 2^(8 − *k*)⌋ for a display channel of `bits[c]`
/// = *k* bits and widened again with zero low bits (S-0103 Formulas).
pub fn owner_radar_colour(
    import: &Import,
    options: &Options,
    owner: &[u8],
    bits: [u8; 3],
) -> [u8; 3] {
    let (hsv, _) = owner_definition(import, options, owner);
    // spec: S-0103 claim 15 (ramp index 16, the brightest entry, is ramp entry 0 of claim 6)
    let c = glasnost_render::palette::ramp(hsv)[0];
    let mut out = [c.r, c.g, c.b];
    for (u, k) in out.iter_mut().zip(bits) {
        // spec: S-0103 Formulas (house display RGB: subtract 1, keep k bits, widen with zeros)
        let shift = 8 - k;
        *u = (u.saturating_sub(1) >> shift) << shift;
    }
    out
}

/// The colour definition a colour selector names (S-0103 claim 14): only
/// its first 31 characters count, names compare without regard to letter
/// case, and a number is a name, not a position.
fn colour_by_selector(import: &Import, selector: &[u8]) -> Option<[u8; 3]> {
    const SELECTOR_LEN: usize = 31; // spec: S-0103 claim 14 (only the first 31 characters of a selector are used)
    let sel = &selector[..selector.len().min(SELECTOR_LEN)];
    if sel.is_empty() {
        return None;
    }
    // spec: S-0103 claim 5 (the first definition of a name is the one kept)
    import
        .outcome
        .colours
        .iter()
        .find(|c| c.name.eq_ignore_ascii_case(sel))
        .map(|c| c.hsv)
}

/// A country's imported colour definition (S-0103 claim 14): each rules
/// layer's selector that names a definition replaces the choice; an absent
/// or unknown selector keeps the earlier one; with none, the first
/// definition (position 0).
fn country_colour(import: &Import, country: &[u8]) -> Option<[u8; 3]> {
    use glasnost_convert::layers::What;
    let rec = import
        .outcome
        .registries
        .get(&Kind::Countries)?
        .iter()
        .find(|t| t.name == country)?;
    // spec: S-0103 claim 14 (with no earlier value, the first definition, position 0)
    let mut hsv = import.outcome.colours.first()?.hsv;
    if let Some(k) = rec.keys.get(COLOR_KEY) {
        for e in &k.history {
            if let What::Set(v) = &e.what
                && let Some(c) = colour_by_selector(import, v)
            {
                hsv = c;
            }
        }
    }
    Some(hsv)
}

/// The colour key of a country's or a map house's section (S-0103 Import mapping).
const COLOR_KEY: &[u8] = b"Color"; // spec: S-0103 Import mapping (rules country section and map house section: Color)
/// The map's house list section.
const HOUSES_SECTION: &[u8] = b"Houses"; // spec: S-0009 Import mapping ([Houses] (any key): map-inspection house names)
/// The country key of a map house's own section.
const COUNTRY_KEY: &[u8] = b"Country"; // spec: S-0009 Import mapping (map house's own section: Country)
/// A country's name field.
const NAME_KEY: &[u8] = b"Name"; // spec: S-0009 Import mapping (rules country: Name)

/// The map's own owners outside a skirmish ([`map_owner_colours`]).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MapOwners {
    /// Each resolved owner's colour definition (H, S, V), by owner name.
    pub colours: Vec<(Vec<u8>, [u8; 3])>,
    /// Owners matching no house the map creates (S-0009 claim 48).
    pub unresolved: Vec<Vec<u8>>,
    /// Importer warnings (S-0009 claim 47's departures).
    pub warnings: Vec<String>,
}

/// The first `n` bytes of `v`, then trimmed of leading and trailing
/// whitespace (S-0009 claim 47).
fn cut_trim(v: &[u8], n: usize) -> &[u8] {
    v[..v.len().min(n)].trim_ascii()
}

/// The colour definition the country selector `selector` of a map house
/// gives it (S-0009 claim 47, S-0103 claim 14); `None` when it selects
/// no country (an empty registry, with a warning).
fn house_country_colour(
    import: &Import,
    house: &[u8],
    selector: Option<&[u8]>,
    warnings: &mut Vec<String>,
) -> Option<[u8; 3]> {
    const SELECTOR_LEN: usize = 127; // spec: S-0009 claim 47 (a nonempty selector uses its first 127 bytes, trimmed after the cut)
    let countries = import
        .outcome
        .registries
        .get(&Kind::Countries)
        .map_or(&[][..], |r| r.as_slice());
    let mut sel = selector.map_or(&[][..], |s| cut_trim(s, SELECTOR_LEN));
    // spec: S-0009 Import mapping (Country = <random> ignores ASCII case; <Player @ A> … <Player @ H> match exact case)
    let reserved = sel.eq_ignore_ascii_case(b"<random>")
        || (b'A'..=b'H').any(|c| sel == [&b"<Player @ "[..], &[c], b">"].concat());
    if reserved {
        // spec: S-0009 claim 47 (our importer treats a reserved selector as an absent one and warns)
        warnings.push(format!(
            "map house {}: reserved country selector {} treated as absent (S-0009 claim 47)",
            String::from_utf8_lossy(house),
            String::from_utf8_lossy(sel)
        ));
        sel = &[];
    }
    let position_0 = import.outcome.colours.first().map(|c| c.hsv);
    if sel.is_empty() {
        // spec: S-0009 claim 47 (an absent or empty selector uses the country at registration index 0)
        let Some(first) = countries.first() else {
            // spec: S-0009 claim 47 (no country at index 0: the house is left unresolved, with a warning)
            warnings.push(format!(
                "map house {}: no country at registration index 0, house unresolved (S-0009 claim 47)",
                String::from_utf8_lossy(house)
            ));
            return None;
        };
        return country_colour(import, &first.name);
    }
    // spec: S-0009 claim 47 (type name or name field, ASCII case ignored; the earliest country matching either wins)
    let found = countries.iter().find(|c| {
        let name = c
            .keys
            .get(NAME_KEY)
            .and_then(|k| k.value.as_deref())
            .unwrap_or(&c.name);
        c.name.eq_ignore_ascii_case(sel) || name.eq_ignore_ascii_case(sel)
    });
    if let Some(c) = found {
        return country_colour(import, &c.name);
    }
    // A later selector naming the created country matches it, with the same colour.
    // spec: S-0009 claim 47, S-0103 claim 14 (an unknown name creates a default country, initially colour definition position 0)
    position_0
}

/// The map's own owners outside a skirmish (S-0009 claims 47 and 48,
/// S-0103 claim 14), from the map file read as `map_ini`. A nonempty
/// house list creates only the listed houses, each inheriting its
/// country selector's colour; an empty one creates a house per country,
/// named by the first 20 bytes of its type name. A house's own section
/// can replace the inherited colour by naming a definition. An object's
/// owner must equal a created house's name exactly; any other is
/// unresolved (the original drops its objects). The two player
/// placeholders are the viewer's and get no entry.
pub fn map_owner_colours(map: &Map, import: &Import, map_ini: &Document) -> MapOwners {
    const LIST_NAME_LEN: usize = 19; // spec: S-0009 claim 47 (only the first 19 bytes of a list value are used, then trimmed)
    const COUNTRY_HOUSE_LEN: usize = 20; // spec: S-0009 claim 48 (the first 20 bytes of the country's type name)
    let mut out = MapOwners::default();
    // spec: S-0009 claim 48 (an empty-valued list line is no entry)
    let listed: Vec<&[u8]> = map_ini
        .section(HOUSES_SECTION)
        .map(|s| {
            s.entries()
                .iter()
                .map(|e| cut_trim(&e.value, LIST_NAME_LEN))
                .filter(|v| !v.is_empty())
                .collect()
        })
        .unwrap_or_default();
    let own = |house: &[u8]| {
        // spec: S-0103 claim 14 (a valid selector in the house's own section replaces the inherited colour)
        map_ini
            .value(house, COLOR_KEY)
            .and_then(|v| colour_by_selector(import, v))
    };
    let mut houses: Vec<(Vec<u8>, Option<[u8; 3]>)> = Vec::new();
    if listed.is_empty() {
        // spec: S-0009 claim 48 (no list entries: one house per registered country, in country order)
        for c in import
            .outcome
            .registries
            .get(&Kind::Countries)
            .map_or(&[][..], |r| r.as_slice())
        {
            let name = &c.name[..c.name.len().min(COUNTRY_HOUSE_LEN)];
            let hsv = own(name).or(country_colour(import, &c.name));
            houses.push((name.to_vec(), hsv));
        }
    } else {
        for house in listed {
            // spec: S-0009 claim 47 (the house's own section's country selector, not its name, picks the country)
            let inherited = house_country_colour(
                import,
                house,
                map_ini.value(house, COUNTRY_KEY),
                &mut out.warnings,
            );
            let hsv = inherited.map(|i| own(house).unwrap_or(i));
            houses.push((house.to_vec(), hsv));
        }
    }
    let o = &map.objects;
    let owners = o
        .units
        .iter()
        .map(|x| &x.placed)
        .chain(o.infantry.iter().map(|x| &x.placed))
        .chain(o.aircraft.iter().map(|x| &x.placed))
        .chain(o.structures.iter().map(|x| &x.placed))
        .map(|p| p.owner.as_slice());
    for owner in owners {
        let seen = out.colours.iter().any(|(n, _)| n.as_slice() == owner)
            || out.unresolved.iter().any(|n| n.as_slice() == owner);
        if [HUMAN, COMPUTER].contains(&owner) || seen {
            continue;
        }
        // spec: S-0009 claim 48 (an owner must match a created house's name exactly, including letter case)
        match houses.iter().find(|(n, _)| n.as_slice() == owner) {
            Some((_, Some(hsv))) => out.colours.push((owner.to_vec(), *hsv)),
            _ => out.unresolved.push(owner.to_vec()),
        }
    }
    out
}

/// Every file the scene of `map` may read: tile files (found with the
/// suffix fallback of S-0010 claim 3 through `exists`), sprites, voxel
/// models with their motion files, and the voxel lighting table.
pub fn wanted_files(
    map: &Map,
    cells: &CellGrid,
    import: &Import,
    catalogue: &Catalogue,
    exists: &dyn Fn(&[u8]) -> bool,
) -> BTreeSet<Vec<u8>> {
    let mut out = BTreeSet::new();
    let mut tiles = BTreeSet::new();
    for (_, cell) in cells.iter() {
        if let Some(t) = catalogue.art_tile(cell.tile) {
            tiles.insert(t);
        }
    }
    // spec: S-0010 claim 24 (a cell without a usable tile shows the clear tile)
    if (0..catalogue.tile_count).contains(&catalogue.clear_base) {
        tiles.insert(catalogue.clear_base);
    }
    for t in tiles {
        // spec: S-0010 claim 3 (the plain image and its variants, each with the suffix fallback)
        let images = catalogue.tile_images(t, exists);
        out.extend(
            images
                .plain
                .iter()
                .chain(&images.variants)
                .map(|f| upper(f)),
        );
    }

    let art = art_document(import);
    let mut add_art = |kind: Kind, idx: usize| {
        art_files(import, kind, idx, &mut out);
        if kind == Kind::Buildings {
            slot_files(import, &art, idx, &mut out);
        }
    };
    let used_overlays: BTreeSet<u8> = map.cells.iter().map(|(_, c)| c.overlay).collect();
    for o in used_overlays {
        if o != NO_OVERLAY {
            add_art(Kind::Overlays, usize::from(o));
        }
    }
    let objects = &map.objects;
    let names = objects
        .units
        .iter()
        .map(|u| (Kind::Vehicles, &u.placed.type_name))
        .chain(
            objects
                .infantry
                .iter()
                .map(|u| (Kind::Infantry, &u.placed.type_name)),
        )
        .chain(
            objects
                .aircraft
                .iter()
                .map(|u| (Kind::Aircraft, &u.placed.type_name)),
        )
        .chain(
            objects
                .structures
                .iter()
                .map(|u| (Kind::Buildings, &u.placed.type_name)),
        )
        .chain(
            objects
                .terrain
                .iter()
                .map(|t| (Kind::TerrainObjects, &t.type_name)),
        )
        .chain(
            objects
                .smudges
                .iter()
                .map(|t| (Kind::Smudges, &t.type_name)),
        );
    for (kind, name) in names {
        if let Some(i) = find_type(import, kind, name) {
            add_art(kind, i);
        }
    }
    out.insert(upper(VPL_FILE));
    // spec: S-0111 Import mapping (the overlay sprites and palette of the status extras)
    for f in [
        super::combat::PIP_SPRITE,
        super::combat::BRACKET_SPRITE,
        super::combat::OVERLAY_PALETTE,
    ] {
        out.insert(upper(f));
    }
    // spec: S-0103 claim 13 (the sidebar icon palette)
    out.insert(upper(glasnost_render::palette::ICON_PALETTE_FILE));
    out
}

/// The files of one type's art: sprite, voxel models with their motion
/// files, and building part sprites.
fn art_files(import: &Import, kind: Kind, idx: usize, out: &mut BTreeSet<Vec<u8>>) {
    if let Some(a) = import.arts.get(&(kind, idx)) {
        out.extend(a.sprite.iter().map(|s| upper(s)));
        for v in a.voxels.values() {
            out.insert(upper(v));
            out.insert(upper(&hva_name(v)));
        }
        out.extend(a.building_sprites.values().map(|s| upper(s)));
    }
}

/// The art files of every building, vehicle and infantry type: a live
/// game can create types the map does not hold (deploys, production).
pub fn game_files(import: &Import) -> BTreeSet<Vec<u8>> {
    let mut out = BTreeSet::new();
    let art = art_document(import);
    for &(kind, idx) in import.arts.keys() {
        if matches!(kind, Kind::Buildings | Kind::Vehicles | Kind::Infantry) {
            art_files(import, kind, idx, &mut out);
        }
        if kind == Kind::Buildings {
            slot_files(import, &art, idx, &mut out);
        }
    }
    out
}

/// The import's art file as a document (empty without one).
fn art_document(import: &Import) -> Document {
    match &import.art_file {
        Some(f) => Document::parse(&f.bytes, Default::default(), &mut Vec::new()),
        None => Document::empty(Default::default()),
    }
}

/// The sprites of the animations a building of type `idx` starts when it
/// is placed (S-0108 claim 11), normal and damaged variants.
fn slot_files(import: &Import, art: &Document, idx: usize, out: &mut BTreeSet<Vec<u8>>) {
    let Some(image) = import.arts.get(&(Kind::Buildings, idx)).map(|a| &a.image) else {
        return;
    };
    for base in slots::SETUP_SLOTS {
        let def = slots::read_slot(art, image, base);
        for name in [&def.normal, &def.damaged] {
            if let Some(a) = find_type(import, Kind::Animations, name)
                && let Some(sprite) = import
                    .arts
                    .get(&(Kind::Animations, a))
                    .and_then(|a| a.sprite.as_ref())
            {
                out.insert(upper(sprite));
            }
        }
    }
}

/// The motion file of a model file (S-0105 glossary *voxel model*: same
/// base name).
pub fn hva_name(vxl: &[u8]) -> Vec<u8> {
    let base = vxl
        .iter()
        .rposition(|&b| b == b'.')
        .map_or(vxl, |i| &vxl[..i]);
    let mut n = base.to_vec();
    n.extend_from_slice(b".hva");
    n
}

/// Per-cell light (S-0104 claim 9).
struct Lights {
    side: i32,
    cells: Vec<Option<light::CellLight>>,
}

impl Lights {
    fn get(&self, c: CellXY) -> Option<&light::CellLight> {
        if c.x < 0 || c.y < 0 || c.x >= self.side || c.y >= self.side {
            return None;
        }
        self.cells[(c.y * self.side + c.x) as usize].as_ref()
    }
}

/// Palette rows added to the scene, by table.
#[derive(Default)]
struct Rows {
    tiles: BTreeMap<TileTable, (usize, usize)>,
    houses: BTreeMap<[u8; 3], (usize, usize)>,
    /// The start-up tables: `false` the resource table, `true` the mine
    /// table (S-0112 claim 2).
    startup: BTreeMap<bool, (usize, usize)>,
    /// The anim palette's untinted table (S-0103 claim 8 row 5).
    anim: Option<(usize, usize)>,
    /// The single-shade table of the first colour definition (S-0103
    /// claim 8 row 4).
    alt: Option<(usize, usize)>,
}

/// What a scene's builder keeps for drawing more items later: the art file,
/// cell lights, palette rows and the images already added to the scene.
pub struct Painter {
    art: Document,
    lights: Lights,
    rows: Rows,
    images: BTreeMap<(Vec<u8>, usize, u8), Option<u32>>,
    /// Each type's image cache of voxel part and shadow images (S-0105
    /// claims 26, 31), as scene images with their offsets.
    voxels: BTreeMap<(Kind, usize), voxel::ImageCache<Option<PlacedPart>>>,
    /// Vehicle shadows with the current part image excluded, by unmasked
    /// shadow image and part images (S-0105 claim 39): a memo of the
    /// per-appearance exclusion, bounded by the cached images it combines.
    cutouts: BTreeMap<(u32, Vec<u32>), Option<PlacedPart>>,
    /// Where fresh voxel draws are made (S-0105 claims 32, 34): each starts
    /// transparent, and the depth-buffered mode is on only during a fresh
    /// depth-option part draw.
    canvas: voxel::VoxelCanvas,
    /// Parsed voxel models by file.
    models: BTreeMap<Vec<u8>, Option<std::rc::Rc<voxel::VoxelModel>>>,
    /// The voxel lighting table, once read.
    vpl: Option<Option<Vpl>>,
    /// Tile files decoded once, by upper-case name.
    tile_files: BTreeMap<Vec<u8>, Option<TileFile>>,
}

/// A voxel part image added to the scene: its index and offset from the
/// draw point (S-0105 claim 25, F6).
type PlacedPart = (u32, [i32; 2]);

/// A decoded tile file: its size in cells and its tile cells, row by row.
#[derive(Clone, Debug)]
struct TileFile {
    width: i32,
    height: i32,
    cells: Vec<Option<TileImages>>,
    /// Each tile cell's flag value (0 for an empty cell), row by row
    /// (S-0010 Import mapping).
    flags: Vec<u32>,
}

struct Builder<'a> {
    map: &'a Map,
    rules: Option<&'a SimRules>,
    cells_loaded: Option<&'a CellGrid>,
    import: &'a Import,
    catalogue: &'a Catalogue,
    palettes: &'a PaletteState,
    assets: &'a Assets,
    options: &'a Options,
    p: Painter,
    built: Built,
    /// Whether the builder draws the live game's objects ([`Painter::mobiles`]).
    live: bool,
}

/// A decoded tile cell: diamond image, its depth data and template
/// position, and the extra image with its depth and offset from the
/// diamond.
type TileImages = (
    u32,
    Option<u32>,
    i32,
    i32,
    Option<(u32, Option<u32>, i32, i32)>,
);

/// A sprite frame: image index, the frame's offset on the canvas, and the
/// canvas size.
type SpriteFrame = (u32, (i32, i32), (i32, i32));

/// One entry of the object pass's ground layer (S-0102 claims 5 and 6).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GroundEntry {
    /// Sort key (S-0102 claim 5).
    pub key: i64,
    /// Items, in drawing order.
    pub items: Vec<Item>,
    /// The object the entry draws ([`Item::object`]).
    pub object: Option<u32>,
}

/// Builds the scene of `map`.
pub fn build(
    map: &Map,
    import: &Import,
    catalogue: &Catalogue,
    palettes: &PaletteState,
    assets: &Assets,
    options: &Options,
) -> Built {
    let inputs = Inputs {
        map,
        import,
        catalogue,
        palettes,
        assets,
        options,
        rules: None,
        cells: None,
    };
    build_parts(&inputs, false).0
}

/// Builds the scene of a map. With `live`, the map-placed vehicles,
/// infantry and aircraft are left out (a simulation draws them) and the
/// ground-layer entries of the static objects are kept in
/// [`Built::ground`] instead of being merged into the items, which then
/// hold the cell pass only. Returns the painter that draws more items into
/// the scene later.
pub fn build_parts(inputs: &Inputs, live: bool) -> (Built, Painter) {
    build_parts_with(inputs, live, voxel::VoxelCanvas::new())
}

/// [`build_parts`] with the voxel canvas `canvas` carried over from earlier
/// games. Its depth-buffered mode is off between games and no earlier draw
/// affects a fresh one (S-0105 claims 32, 34, S2). The image caches start
/// empty, as S-0105 claim 26 has our engine empty them when a game starts.
pub fn build_parts_with(
    inputs: &Inputs,
    live: bool,
    canvas: voxel::VoxelCanvas,
) -> (Built, Painter) {
    let import = inputs.import;
    let art = art_document(import);
    let mut b = Builder {
        map: inputs.map,
        rules: inputs.rules,
        cells_loaded: inputs.cells,
        import,
        catalogue: inputs.catalogue,
        palettes: inputs.palettes,
        assets: inputs.assets,
        options: inputs.options,
        p: Painter::new(art, canvas),
        built: Built::default(),
        live: false,
    };
    b.light_cells();
    b.tiles();
    // spec: S-0102 claim 25 (a smudge is painted right after the terrain tile; overlays come later)
    b.smudges();
    b.overlays();
    // spec: S-0102 claims 10, 16 and 23 (animated terrain objects leave the cell pass for the
    // ground layer of the object pass)
    let terrain = b.terrain_objects();
    let mut ground = Vec::new();
    b.structures(&mut ground);
    b.built.scene.cell_pass_len = b.built.scene.items.len();
    if live {
        b.built.ground = terrain;
        b.built.ground.extend(ground);
        return (b.built, b.p);
    }
    // spec: S-0009 claim 43 (objects enter the drawing order terrain objects, vehicles, aircraft,
    // infantry, then buildings)
    let mut entries = terrain;
    b.mobiles(&mut entries);
    entries.extend(ground);
    // spec: S-0102 claim 6 (an entering object goes before the first strictly greater key: a stable sort)
    entries.sort_by_key(|g| g.key);
    // spec: S-0102 claims 1 and 7 (the layers in the order underground, surface, ground, air, top;
    // the other layers unsorted, in entering order)
    let layered = std::mem::take(&mut b.built.layered);
    let (below, above): (Vec<_>, Vec<_>) = layered
        .into_iter()
        .partition(|(l, _)| *l < slots::Layer::Ground);
    let mut below = below;
    below.sort_by_key(|(l, _)| *l);
    let mut above = above;
    above.sort_by_key(|(l, _)| *l);
    for (_, g) in below {
        b.built.scene.items.extend(g.items);
    }
    for g in entries {
        b.built.scene.items.extend(g.items);
    }
    for (_, g) in above {
        b.built.scene.items.extend(g.items);
    }
    (b.built, b.p)
}

impl Builder<'_> {
    fn note(&mut self, n: &str) {
        self.built.notes.insert(n.to_owned());
    }

    fn drop_object(&mut self, why: &str) {
        *self.built.dropped.entry(why.to_owned()).or_default() += 1;
    }

    /// Full recompute of every cell's light (S-0104 claim 9) with the
    /// building light sources (claim 12).
    fn light_cells(&mut self) {
        let l = self.map.lighting.normal;
        let mut sources = Vec::new();
        for s in &self.map.objects.structures {
            let Some(i) = find_type(self.import, Kind::Buildings, &s.placed.type_name) else {
                continue;
            };
            if !self.owner_kept(&s.placed.owner) || self.map.cells.get(s.placed.cell).is_none() {
                continue;
            }
            let intensity =
                derived(self.import, Kind::Buildings, i, b"LightIntensity").unwrap_or(0);
            if intensity == 0 {
                continue;
            }
            let (w, h) = self.foundation(i);
            // spec: S-0104 claim 12 (an absent tint's default, quirks.building_light_tint_default_1000)
            let absent = light::building_light_tint_default(&LightQuirks::default());
            let tint =
                |k: &[u8]| derived(self.import, Kind::Buildings, i, k).map_or(absent, |v| v as i32);
            let c = s.placed.cell;
            // spec: S-0104 claim 12 (one light at the building's centre, counting at detail 2)
            sources.push(LightSource::building(
                (c.x, c.y),
                (w, h),
                rule_int(
                    self.import,
                    Kind::Buildings,
                    i,
                    b"LightVisibility",
                    DEFAULT_LIGHT_RADIUS,
                ),
                intensity as i32,
                [
                    tint(b"LightRedTint"),
                    tint(b"LightGreenTint"),
                    tint(b"LightBlueTint"),
                ],
            ));
        }
        let side = i32::from(self.map.cells.footprint().coordinate_side());
        let mut cells = vec![None; (side * side) as usize];
        let q = LightQuirks::default();
        // spec: S-0104 claim 1 (the map's normal set; current, base and target ambient start equal; tint in hundredths)
        let tint = l.tint;
        for (c, cell) in self.map.cells.iter() {
            cells[(c.y * side + c.x) as usize] = Some(light::full_recompute(
                l.ambient,
                l.level,
                l.ground,
                tint,
                (c.x, c.y),
                i32::from(cell.level),
                &sources,
                self.options.detail,
                &q,
            ));
        }
        self.p.lights = Lights { side, cells };
    }

    fn tile_row(&mut self, table: TileTable, brightness: i32) -> Shade {
        let (base, n) = match self.p.rows.tiles.get(&table) {
            Some(r) => *r,
            None => {
                // spec: S-0103 claim 8 (terrain tiles use the tile palette, shaded by their cell's table)
                let t = light::shade_table(&self.palettes.tile, table.tint(), table.rows(), None);
                let base = self.built.scene.rows.len();
                for row in &t {
                    self.built.scene.add_row(row565(row));
                }
                self.p.rows.tiles.insert(table, (base, t.len()));
                (base, t.len())
            }
        };
        // spec: S-0104 claim 15, S-0110 claim 5 (each pixel takes the row of its light-alpha value)
        Shade::new(base, n, brightness)
    }

    fn house_row(&mut self, hsv: [u8; 3], brightness: i32) -> Shade {
        let (base, n) = match self.p.rows.houses.get(&hsv) {
            Some(r) => *r,
            None => {
                let pal = house_palette(&self.palettes.unit, hsv);
                // spec: S-0103 claim 7 (indices 240 … 254 are tint-exempt in house-colour palettes)
                let t = light::shade_table(&pal, WHITE, HOUSE_ROWS, Some(TINT_EXEMPT));
                let base = self.built.scene.rows.len();
                for row in &t {
                    self.built.scene.add_row(row565(row));
                }
                self.p.rows.houses.insert(hsv, (base, t.len()));
                (base, t.len())
            }
        };
        // spec: S-0104 claim 15, S-0110 claim 5 (each pixel takes the row of its light-alpha value)
        Shade::new(base, n, brightness)
    }

    /// The colour definition an owner shows (S-0103 claim 10), and whether
    /// the choice is ours (an owner no skirmish house carries).
    fn owner_colour(&mut self, owner: &[u8]) -> ([u8; 3], bool) {
        let (hsv, ours) = owner_definition(self.import, self.options, owner);
        if ours {
            self.note(notes::OWNER_COLOUR_OURS);
        }
        (hsv, ours)
    }

    /// Whether an owner's objects appear (S-0009 claim 27, in the skirmish
    /// view).
    fn owner_kept(&self, owner: &[u8]) -> bool {
        // spec: S-0009 claim 27 (only the neutral and special houses and the player placeholders match)
        !self.options.skirmish || [NEUTRAL, SPECIAL, HUMAN, COMPUTER].contains(&owner)
    }

    /// An image of a sprite file's frame, decoded once, with its frame
    /// rectangle's offset on the canvas and the canvas size.
    fn sprite_frame(&mut self, file: &[u8], frame: usize) -> Option<SpriteFrame> {
        let bytes = self.assets.get(file)?;
        let shp = Shp::parse(bytes).ok()?;
        let f = *shp.frames.get(frame)?;
        let canvas = (i32::from(shp.width), i32::from(shp.height));
        let key = (upper(file), frame, 0u8);
        let id = match self.p.images.get(&key) {
            Some(id) => *id,
            None => {
                let id = shp
                    .decode_frame(frame)
                    .ok()
                    .filter(|i| i.width() > 0 && i.height() > 0)
                    .map(|i| self.built.scene.add_image(i));
                self.p.images.insert(key, id);
                id
            }
        };
        Some((id?, (i32::from(f.x), i32::from(f.y)), canvas))
    }

    fn sprite_frames(&self, file: &[u8]) -> Option<usize> {
        Shp::parse(self.assets.get(file)?)
            .ok()
            .map(|s| s.frames.len())
    }

    /// Places a sprite frame with its canvas centred on world pixel `p`
    /// (S-0102 claim 19, F6: the frame at (x − ⌊W/2⌋ + c, y − ⌊H/2⌋ + r); S-0108 claim 8 for buildings).
    #[allow(clippy::too_many_arguments)]
    fn place_sprite(
        &mut self,
        file: &[u8],
        frame: usize,
        p: (i32, i32),
        test: DepthTest,
        depth: ItemDepth,
        paint: Paint,
        group: Group,
        provisional: bool,
        object: Option<u32>,
    ) -> Option<Item> {
        let (image, (fx, fy), (cw, ch)) = self.sprite_frame(file, frame)?;
        Some(Item {
            image,
            // spec: S-0102 F6 (⌊W/2⌋ and ⌊H/2⌋: W and H are positive, so truncation is the floor)
            x: p.0 - cw / 2 + fx,
            y: p.1 - ch / 2 + fy,
            test,
            depth,
            paint,
            group,
            provisional,
            object,
        })
    }

    fn add_object(&mut self, info: ObjectInfo) -> u32 {
        self.built.objects.push(info);
        (self.built.objects.len() - 1) as u32
    }

    /// A tile file decoded once: its size in cells and its tile cells.
    fn tile_file(&mut self, file: &[u8]) -> Option<TileFile> {
        let key = upper(file);
        if let Some(t) = self.p.tile_files.get(&key) {
            return t.clone();
        }
        let decoded = self.assets.get(&key).and_then(|bytes| {
            let tmp = Tmp::parse(bytes).ok()?;
            let mut cells = Vec::new();
            for i in 0..tmp.tiles.len() {
                cells.push(tmp.decode_tile(i).ok().map(|d| {
                    let dia = &d.diamond;
                    let img = self.built.scene.add_image(dia.image.clone());
                    let dep = dia.depth.clone().map(|i| self.built.scene.add_image(i));
                    let extra = d.extra.as_ref().map(|e| {
                        let ei = self.built.scene.add_image(e.image.clone());
                        let ed = e.depth.clone().map(|i| self.built.scene.add_image(i));
                        // spec: S-0010 claim 23 (only the difference of the two stored positions matters)
                        (ei, ed, e.x - dia.x, e.y - dia.y)
                    });
                    (img, dep, dia.x, dia.y, extra)
                }));
            }
            let flags = tmp
                .tiles
                .iter()
                .map(|t| t.as_ref().map_or(0, |t| t.flags))
                .collect();
            Some(TileFile {
                width: tmp.cells_x as i32,
                height: tmp.cells_y as i32,
                cells,
                flags,
            })
        });
        self.p.tile_files.insert(key, decoded.clone());
        decoded
    }

    /// Terrain tiles (S-0010 claims 13, 22 to 24; S-0101 claim 4; S-0102
    /// claim 9 and F3; S-0104 claim 13).
    fn tiles(&mut self) {
        if self.cells_loaded.is_none() {
            self.note(notes::TILES_UNLOADED);
        }
        if self.map.fill_draws > 0 && self.cells_loaded.is_none() {
            self.note(notes::WATER_FILL);
        }
        let q = TheaterQuirks::default();
        // spec: S-0010 claim 14, S-0202 claim 11 (the 8 × 8 pattern from the unsynchronised stream, built afresh at map load)
        let mut stream = glasnost_sim::rng::Rng::new(super::live::SANDBOX_SEED);
        let (p8, _) = variant_pattern(|| stream.raw());
        let grid = self.cells_loaded.unwrap_or(&self.map.cells);
        let loaded = self.cells_loaded.is_some();
        let cells: Vec<(CellXY, glasnost_map::Cell)> = grid
            .visiting_order(true)
            .into_iter()
            .filter_map(|c| Some((c, *grid.get(c)?)))
            .collect();
        // Cells the visiting order misses (W = 1) keep their blank state; draw them too.
        let mut seen: BTreeSet<CellXY> = cells.iter().map(|(c, _)| *c).collect();
        let mut all = cells;
        for (c, cell) in grid.iter() {
            if seen.insert(c) {
                all.push((c, *cell));
            }
        }
        let count = self.catalogue.tile_count;
        let assets = self.assets;
        let exists = |n: &[u8]| assets.get(n).is_some();
        let mut images: BTreeMap<i32, glasnost_convert::theater::TileImages> = BTreeMap::new();
        for (c, cell) in all {
            let (t, s) = (cell.tile, i32::from(cell.sub_tile));
            let mut files = |b: &mut Self, t: i32| {
                images
                    .entry(t)
                    .or_insert_with(|| b.catalogue.tile_images(t, &exists))
                    .clone()
            };
            // spec: S-0010 claims 7 and 24 (the tile case paints the cell's tile; the no-tile case, a stray
            // or a negative number paint position 0 of the clear tile, at the cell's own coordinates)
            let mut shown: Option<(i32, i32, glasnost_convert::theater::TileImages)> = None;
            if (0..count).contains(&t) {
                let f = files(self, t);
                let plain = f.plain.as_ref().and_then(|p| self.tile_file(p));
                let usable = plain.as_ref().is_some_and(|p| {
                    s < p.width * p.height && p.cells.get(s as usize).is_some_and(Option::is_some)
                });
                if usable || loaded {
                    // In loaded cells an unusable tile number is the overlay case (claim 7).
                    shown = Some((t, s, f));
                }
            }
            // spec: S-0010 claim 13 (a no-tile cell shows the clear tile by the pattern; no mark is read for it)
            let mut read_mark = shown.is_some();
            let (_, s, f) = match shown {
                Some(x) => x,
                None => {
                    read_mark = false;
                    let clear = self.catalogue.clear_base;
                    if !(0..count).contains(&clear) {
                        self.drop_object(
                            "cell drawn blank: its tile and the clear tile are outside the catalogue",
                        );
                        continue;
                    }
                    let f = files(self, clear);
                    (clear, 0, f)
                }
            };
            let Some(plain) = f.plain.as_ref().and_then(|p| self.tile_file(p)) else {
                // spec: S-0010 claim 24 (a tile without an image paints nothing)
                self.drop_object("cell drawn blank: no file holds its tile image");
                continue;
            };
            // spec: S-0010 claim 13 (the mark of the plain image's tile cell at index t mod (w × h), from the plain image's file only; none where it has no tile cell there)
            let marked = read_mark
                && plain
                    .flags
                    .get(s.rem_euclid((plain.width * plain.height).max(1)) as usize)
                    .is_some_and(|&fl| alternate_image_mark(fl));
            // spec: S-0010 claim 13 (the image is chosen with the plain image's size)
            let v = variant_image(
                (c.x, c.y),
                s,
                (plain.width, plain.height),
                f.variants.len(),
                &p8,
                q.variant_origin_mismatch,
                marked,
            );
            let file = if v == 0 {
                f.plain.clone()
            } else {
                f.variants.get(v - 1).cloned()
            };
            let Some((image, name)) = file.and_then(|n| Some((self.tile_file(&n)?, n))) else {
                self.drop_object("cell drawn blank: its tile file is missing or unreadable");
                continue;
            };
            // spec: S-0010 claim 24 (index t mod N_s of the shown image, counted with its empty positions)
            let n_s = (image.width * image.height).max(1);
            let index = s.rem_euclid(n_s) as usize;
            // spec: S-0010 claim 20 (the radar's tile colour comes from the image variant the cell shows)
            self.built.shown_tiles.insert(c, (upper(&name), index));
            let cell_images = match image.cells.get(index).copied().flatten() {
                Some(i) => i,
                // spec: S-0010 claim 24 (quirks.variant_cell_missing: an empty position paints nothing)
                None if !q.variant_cell_missing => {
                    match plain.cells.get(s.max(0) as usize).copied().flatten() {
                        Some(i) => i,
                        None => continue,
                    }
                }
                None => {
                    self.drop_object("cell paints nothing: its shown image has no tile cell there");
                    continue;
                }
            };
            let (img, dep, _, _, extra) = cell_images;
            let level = i32::from(cell.level);
            // spec: S-0010 claim 22 ((u, v) the view pixel of the top corner at height 0)
            let (u, v0) = cell_top(c.x, c.y);
            // spec: S-0010 claim 22, S-0101 claim 4 (diamond pixel (c, r) at (u − 30 + c, v − 15ℓ + r))
            let left = u - DIAMOND_WIDTH / 2;
            let top = v0 - TERRAIN_LIFT * level;
            // spec: S-0102 F3 (t = the tile's top row as drawn plus 15 × its height level)
            let reference_row = top + TERRAIN_LIFT * level;
            let (table, b) = match self.p.lights.get(c) {
                // spec: S-0104 claim 13 (a terrain tile, ground and extra image: the cell's scaled ground level)
                Some(l) => (l.table, l.scaled_ground),
                None => (TileTable::Neutral, NEUTRAL_BRIGHTNESS),
            };
            let row = self.tile_row(table, b);
            let mut push = |image: u32, depth: Option<u32>, x: i32, y: i32, paint: Paint| {
                self.built.scene.items.push(Item {
                    image,
                    x,
                    y,
                    test: DepthTest::Tile,
                    depth: ItemDepth::Tile {
                        depth,
                        reference_row,
                    },
                    paint,
                    group: Group::Tiles,
                    provisional: false,
                    object: None,
                });
            };
            // spec: S-0010 claim 22 (every pixel of the diamond is painted, index 0 included)
            push(img, dep, left, top, Paint::LitDiamond(row));
            if let Some((ei, ed, dx, dy)) = extra {
                // spec: S-0010 claim 23 (the extra image at the diamond box's corner plus the position difference;
                // index 0 transparent; drawn right after its own cell's diamond)
                push(ei, ed, left + dx, top + dy, Paint::Lit(row));
            }
        }
    }

    /// The shade row of an untinted start-up table (S-0112 Terms, claim 2;
    /// S-0104 claim 6): the resource table, or the mine table.
    fn startup_row(&mut self, mine: bool, brightness: i32) -> Shade {
        let q = OverlayQuirks::default();
        let (base, n) = match self.p.rows.startup.get(&mine) {
            Some(r) => *r,
            None => {
                // spec: S-0112 claim 2 (quirks.resource_palettes_from_start: the start-up palettes; off,
                // the loaded theater's theater palette and unit palette)
                let pal = match (mine, q.resource_palettes_from_start) {
                    (false, true) => &self.palettes.resource,
                    (true, true) => &self.palettes.mine,
                    (false, false) => &self.palettes.theater,
                    (true, false) => &self.palettes.unit,
                };
                let t = light::untinted_table(pal);
                let base = self.built.scene.rows.len();
                for row in &t {
                    self.built.scene.add_row(row565(row));
                }
                self.p.rows.startup.insert(mine, (base, t.len()));
                (base, t.len())
            }
        };
        // spec: S-0104 claim 15, S-0110 claim 5 (each pixel takes the row of its light-alpha value)
        Shade::new(base, n, brightness)
    }

    /// Each tiberium's image run, in tiberium list order, from the image
    /// numbers its rules layers gave (S-0112 claim 10).
    fn tiberium_runs(&self) -> Vec<Option<Run>> {
        let q = OverlayQuirks::default();
        let Some(list) = self.import.outcome.registries.get(&Kind::Tiberiums) else {
            return Vec::new();
        };
        list.iter()
            .map(|t| {
                let mut run = None;
                if let Some(k) = t.keys.get(&b"Image"[..]) {
                    for e in &k.history {
                        if let glasnost_convert::layers::What::Set(v) = &e.what
                            && let Some(v) = value::integer(v)
                        {
                            run = apply_image(run, v, &q);
                        }
                    }
                }
                run
            })
            .collect()
    }

    /// Overlays and their shadows (S-0112 claims 1 to 9; S-0102 claims 12,
    /// 13 and 24; S-0009 claim 12).
    fn overlays(&mut self) {
        let q = OverlayQuirks::default();
        let k = Kind::Overlays;
        let list_len = self.import.outcome.registries.get(&k).map_or(0, Vec::len);
        let grid = self.cells_loaded.unwrap_or(&self.map.cells);
        let cells: Vec<(CellXY, glasnost_map::Cell)> =
            grid.iter().map(|(c, cell)| (c, *cell)).collect();
        let runs = self.tiberium_runs();
        let (local_hsv, _) = self.owner_colour(HUMAN);
        let mut beyond = 0usize;
        let mut shadows = Vec::new();
        for (c, cell) in cells {
            let o = cell.overlay;
            if o == NO_OVERLAY {
                continue;
            }
            let idx = usize::from(o);
            if idx >= list_len {
                // spec: S-0009 Edge cases (an index beyond the list: no overlay, and the importer warns)
                beyond += 1;
                continue;
            }
            let flag = |key: &[u8]| rule_bool(self.import, k, idx, key, false);
            let flags = Flags {
                tiberium: flag(b"Tiberium"),
                wall: flag(b"Wall"),
                crate_: flag(b"Crate"),
                rubble: flag(b"IsRubble"),
                // spec: S-0102 Import mapping (overlay Land: a railroad land type)
                railroad: rule(self.import, k, idx, b"Land")
                    .is_some_and(|v| value::text(v, usize::MAX).eq_ignore_ascii_case(b"Railroad")),
            };
            // spec: S-0009 claim 12 (crate overlays never appear in skirmish or multiplayer, quirks.skirmish_drops_map_crates)
            let drops_crates = self
                .rules
                .is_none_or(|r| r.quirks.skirmish_drops_map_crates);
            if self.cells_loaded.is_none() && self.options.skirmish && flags.crate_ && drops_crates
            {
                continue;
            }
            // spec: S-0112 claim 1 (a deck cell: one where an overlay at a bridge position was
            // placed). The viewer draws the cells after loading, where such a cell holds its
            // bridge type or a veinhole marker that replaced it; row 1 wins for the marker either
            // way, so testing the overlay held gives the same rows.
            let deck = DECK_BRIDGES.contains(&o);
            let table_row = overlay_table_row(o, flags, deck);
            let d = cell.overlay_data;
            let level = i32::from(cell.level);
            let light = self.p.lights.get(c).copied();
            let scaled = light.map_or(NEUTRAL_BRIGHTNESS, |l| l.scaled_ground);
            let table = light.map_or(TileTable::Neutral, |l| l.table);
            // spec: S-0112 claim 1 (palette, shade table and brightness by row)
            let paint_row = match table_row {
                OverlayRow::Marker | OverlayRow::Rubble => None,
                OverlayRow::Deck => Some(
                    self.tile_row(table, light.map_or(NEUTRAL_BRIGHTNESS, |l| l.scaled_raised)),
                ),
                // spec: S-0112 claim 1 row 3 (ore and gems: the start-up resource table at 1000)
                OverlayRow::Tiberium => Some(self.startup_row(false, NEUTRAL_BRIGHTNESS)),
                // spec: S-0112 claim 1 row 4 (walls: the local player's house colour, quirks.wall_local_player_colour; U)
                OverlayRow::Wall => {
                    let _ = q.wall_local_player_colour;
                    Some(self.house_row(
                        local_hsv,
                        light.map_or(NEUTRAL_BRIGHTNESS, |l| l.unscaled_ground),
                    ))
                }
                OverlayRow::Vein => Some(self.startup_row(false, scaled)),
                OverlayRow::Crate | OverlayRow::Other => Some(self.tile_row(table, scaled)),
            };
            let own_sprite = self.overlay_sprite(idx);
            // spec: S-0112 claims 1, 4 and 6 (the body's sprite and image frame)
            let body: Option<(Vec<u8>, usize)> = match table_row {
                OverlayRow::Marker | OverlayRow::Rubble => None,
                OverlayRow::Deck => own_sprite.clone().map(|s| (s, deck_frame(d, (c.x, c.y)))),
                OverlayRow::Tiberium => {
                    let run = tiberium_of(i32::from(o), &runs).and_then(|t| runs[t]);
                    match run {
                        // spec: S-0112 claim 4 (the body comes from the tiberium's run, by cell position)
                        Some(run) => {
                            let p = tiberium_body(run, (c.x, c.y), cell.slope);
                            usize::try_from(p)
                                .ok()
                                .and_then(|p| self.overlay_sprite(p))
                                .map(|s| (s, usize::from(d)))
                        }
                        // spec: S-0112 claim 12 (no run: the stored type, with the data value as the frame)
                        None => own_sprite.clone().map(|s| (s, usize::from(d))),
                    }
                }
                OverlayRow::Crate => own_sprite.clone().map(|s| (s, 0)),
                OverlayRow::Wall | OverlayRow::Vein | OverlayRow::Other => {
                    own_sprite.clone().map(|s| (s, usize::from(d)))
                }
            };
            if table_row == OverlayRow::Rubble {
                self.note(notes::RUBBLE);
            }
            // spec: S-0102 claim 12 (drawn flat unless the type says otherwise; walls always upright)
            let rock = flag(b"IsARock");
            let flat = !flags.wall
                && (flags.tiberium || rule_bool(self.import, k, idx, b"DrawFlat", true));
            // spec: S-0102 Constants (overlay depth bias: level + 4 for bridge decks)
            let depth_level = if deck { level + DECK_LEVELS } else { level };
            // spec: S-0102 claim 13 (overlay biases: −2 − 15ℓ, or −17 − 15ℓ for other upright types;
            // quirks.overlay_lift_exact_levels)
            let lift = overlay_depth_lift(depth_level, &ViewQuirks::default());
            let bias = if flat || flags.wall || rock || flags.tiberium {
                -2 - lift
            } else {
                -17 - lift
            };
            // spec: S-0102 claim 24 (the cell's centre at level 0, lifted 15 rows per level, plus the shift s)
            let (shift, shadow_offset) = draw_shift(o, flags, deck, d);
            let (qx, qy) = cell_top(c.x, c.y);
            let p = (qx, qy + 15 - TERRAIN_LIFT * level + shift);
            let name = self.import.outcome.registries[&k][idx].name.clone();
            let object = self.add_object(ObjectInfo {
                kind: "overlay",
                type_name: String::from_utf8_lossy(&name).into_owned(),
                owner: String::new(),
                cell: c,
                entry: format!("type index {o}, data {d}"),
                notes: Vec::new(),
            });
            // spec: S-0112 claim 9 (an image frame beyond the sprite draws nothing)
            if let (Some((sprite, frame)), Some(row)) = (&body, paint_row)
                && self.sprite_frames(sprite).is_some_and(|n| *frame < n)
                && let Some(item) = self.place_sprite(
                    sprite,
                    *frame,
                    p,
                    DepthTest::Writer,
                    ItemDepth::Profile {
                        profile: if flat {
                            Profile::Flat
                        } else {
                            Profile::Upright
                        },
                        bias,
                        companion: None,
                    },
                    Paint::Lit(row),
                    Group::Overlays,
                    false,
                    Some(object),
                )
            {
                self.built.scene.items.push(item);
            }
            // spec: S-0112 claim 8 (the shadow: crate N/2; rubble none; otherwise the stored type, d + N/2)
            let shadow: Option<(Vec<u8>, usize)> = if flags.crate_ {
                own_sprite.map(|s| {
                    let n = self.sprite_frames(&s).unwrap_or(0);
                    (s, n / 2)
                })
            } else if flags.rubble {
                None
            } else if q.overlay_shadow_from_stored_type {
                own_sprite.map(|s| {
                    let n = self.sprite_frames(&s).unwrap_or(0);
                    (s, usize::from(d) + n / 2)
                })
            } else {
                // spec: S-0112 Edge cases (switch off: the body's sprite, body frame + N/2)
                body.map(|(s, f)| {
                    let n = self.sprite_frames(&s).unwrap_or(0);
                    (s, f + n / 2)
                })
            };
            // spec: S-0102 claim 13 (an overlay's shadow: −2 − 15 × ℓ_s with ℓ_s the cell's level alone,
            // never the deck's 4 levels nor the upright −15; flat; writes), S-0112 claim 8
            // (every overlay kind); S-0102 Edge cases (quirks.overlay_lift_exact_levels covers overlay shadows)
            let shadow_bias = -2 - overlay_depth_lift(level, &ViewQuirks::default());
            if let Some((sprite, frame)) = shadow
                && self.sprite_frames(&sprite).is_some_and(|n| frame < n)
                && let Some(item) = self.place_sprite(
                    &sprite,
                    frame,
                    (p.0 + shadow_offset.0, p.1 + shadow_offset.1),
                    DepthTest::Writer,
                    ItemDepth::Profile {
                        profile: Profile::Flat,
                        bias: shadow_bias,
                        companion: None,
                    },
                    Paint::Shadow,
                    Group::Shadows,
                    false,
                    Some(object),
                )
            {
                shadows.push(item);
            }
        }
        // spec: S-0102 claim 16 rank 3 (every overlay of the repainted area ranks before every overlay
        // shadow of that area)
        self.built.scene.items.extend(shadows);
        if beyond > 0 {
            *self
                .built
                .dropped
                .entry("overlay index beyond the overlay list".into())
                .or_default() += beyond;
        }
    }

    /// The sprite of the overlay type at list position `idx`.
    fn overlay_sprite(&self, idx: usize) -> Option<Vec<u8>> {
        self.import
            .arts
            .get(&(Kind::Overlays, idx))
            .and_then(|a| a.sprite.clone())
    }

    /// Smudges (S-0102 claim 25, S-0112 claim 3, S-0009 claim 42): painted
    /// once for every covered cell, image frame 0, from the top corner of
    /// the smudge's first cell lifted by the painted cell's level, with that
    /// cell's tile shade table and scaled ground level, without depth.
    fn smudges(&mut self) {
        let smudges = self.map.objects.smudges.clone();
        for s in smudges {
            let Some(i) = find_type(self.import, Kind::Smudges, &s.type_name) else {
                // spec: S-0009 claim 42 (a type field that names no smudge type places nothing)
                self.drop_object("unknown smudge type");
                continue;
            };
            let Some(sprite) = self
                .import
                .arts
                .get(&(Kind::Smudges, i))
                .and_then(|a| a.sprite.clone())
            else {
                self.drop_object("smudge without art");
                continue;
            };
            // spec: S-0009 Import mapping (smudge Width and Height, absent 1 and 1)
            let w = rule_int(self.import, Kind::Smudges, i, b"Width", 1);
            let h = rule_int(self.import, Kind::Smudges, i, b"Height", 1);
            let object = self.add_object(ObjectInfo {
                kind: "smudge",
                type_name: String::from_utf8_lossy(&s.type_name).into_owned(),
                owner: String::new(),
                cell: s.cell,
                entry: String::new(),
                notes: vec![notes::SMUDGE_CLIP.into()],
            });
            // spec: S-0102 claim 25 (the top corner of the smudge's first cell)
            let (qx, qy) = cell_top(s.cell.x, s.cell.y);
            // spec: S-0009 claim 42 (the w × h cells (x + i, y + j)), S-0102 claim 25 (each covered cell paints the image again)
            for j in 0..h.max(0) {
                for i in 0..w.max(0) {
                    let c = CellXY::new(s.cell.x + i, s.cell.y + j);
                    let Some(cell) = self.map.cells.get(c).copied() else {
                        continue;
                    };
                    let level = i32::from(cell.level);
                    let light = self.p.lights.get(c);
                    // spec: S-0112 claim 3 (the painted cell's tile shade table and scaled ground level)
                    let row = self.tile_row(
                        light.map_or(TileTable::Neutral, |l| l.table),
                        light.map_or(NEUTRAL_BRIGHTNESS, |l| l.scaled_ground),
                    );
                    if let Some(item) = self.place_sprite(
                        &sprite,
                        0,
                        (qx, qy - TERRAIN_LIFT * level),
                        // spec: S-0102 claim 25 (no depth test and no depth write)
                        DepthTest::Ignore,
                        ItemDepth::Profile {
                            profile: Profile::Flat,
                            bias: 0,
                            companion: None,
                        },
                        Paint::Lit(row),
                        Group::Smudges,
                        false,
                        Some(object),
                    ) {
                        self.built.scene.items.push(item);
                    }
                }
            }
        }
    }

    /// Terrain objects (S-0102 claims 10, 13, 16 and 23, S-0112 claim 3,
    /// S-0104 claim 13): those that are not animated in the cell pass, the
    /// animated ones returned as ground-layer entries of the object pass.
    fn terrain_objects(&mut self) -> Vec<GroundEntry> {
        let mut animated_entries = Vec::new();
        let list = self.map.objects.terrain.clone();
        for t in list {
            let Some(i) = find_type(self.import, Kind::TerrainObjects, &t.type_name) else {
                self.drop_object("unknown terrain object type");
                continue;
            };
            let Some(sprite) = self
                .import
                .arts
                .get(&(Kind::TerrainObjects, i))
                .and_then(|a| a.sprite.clone())
            else {
                self.drop_object("terrain object without art");
                continue;
            };
            let Some(cell) = self.map.cells.get(t.cell).copied() else {
                continue;
            };
            let z = LEVEL_LEPTONS * i32::from(cell.level);
            let light = self.p.lights.get(t.cell).copied();
            let spawns = rule_bool(
                self.import,
                Kind::TerrainObjects,
                i,
                b"SpawnsTiberium",
                false,
            );
            // spec: S-0112 claim 3 (plain: its cell's tile shade table and scaled ground; ore-spawning:
            // the start-up mine table and U)
            let row = if spawns {
                self.startup_row(
                    true,
                    light.map_or(NEUTRAL_BRIGHTNESS, |l| l.unscaled_ground),
                )
            } else {
                self.tile_row(
                    light.map_or(TileTable::Neutral, |l| l.table),
                    light.map_or(NEUTRAL_BRIGHTNESS, |l| l.scaled_ground),
                )
            };
            // spec: S-0102 claim 23 (the centre of its cell at the terrain height; an ore spawner 16 rows higher)
            let (x, y) = cell_centre(t.cell.x, t.cell.y);
            let point = project_lifted(x, y, z);
            let p = if spawns {
                (point.0, point.1 - ORE_SPAWNER_RAISE)
            } else {
                point
            };
            let frames = self.sprite_frames(&sprite).unwrap_or(0);
            // spec: S-0102 claim 23 (an animated terrain object: the type's animated flag; no
            // destruction sequence has begun at placement)
            let animated = rule_bool(self.import, Kind::TerrainObjects, i, b"IsAnimated", false);
            let frame = if animated {
                // spec: S-0102 claim 23, Constants (an animated terrain object's stage is 0 at
                // placement, whatever its health)
                if spawns {
                    self.note(notes::ORE_MINE_STAGE);
                }
                0
            } else {
                // spec: S-0102 claim 23 (frame 0 while its health, its type's strength at the start, is 2 or more; 1 below)
                // spec: S-0009 claim 42 (the starting health is the type's strength with the game-wide tree strength applied; unset: −1)
                // (The typed rules list the terrain types in the import's registration order.)
                let typed = self
                    .rules
                    .and_then(|r| r.terrain.get(i))
                    .map(|tr| tr.strength);
                let strength = typed.or_else(|| {
                    rule(self.import, Kind::TerrainObjects, i, b"Strength").and_then(value::integer)
                });
                if typed.is_none() && strength.is_none() {
                    self.note(notes::TERRAIN_STRENGTH);
                }
                usize::from(strength.is_some_and(|s| s < 2))
            };
            // spec: S-0102 claim 10 (cell pass: writers; object pass: readers)
            let test = if animated {
                DepthTest::Reader
            } else {
                DepthTest::Writer
            };
            let object = self.add_object(ObjectInfo {
                kind: "terrain object",
                type_name: String::from_utf8_lossy(&t.type_name).into_owned(),
                owner: String::new(),
                cell: t.cell,
                entry: String::new(),
                notes: Vec::new(),
            });
            let lz = height_lift(z);
            let mut items = Vec::new();
            // spec: S-0102 claim 13 (terrain object: −L(z) − 12, upright; writes in the cell pass,
            // reads in the object pass)
            items.extend(self.place_sprite(
                &sprite,
                frame,
                p,
                test,
                ItemDepth::Profile {
                    profile: Profile::Upright,
                    bias: -lz - 12,
                    companion: None,
                },
                Paint::Lit(row),
                Group::TerrainObjects,
                false,
                Some(object),
            ));
            // spec: S-0102 claims 13 and 23 (its shadow: body frame + ⌊N/2⌋, −L(z) − 3, flat);
            // claim 30 (an animated object's body ranks before its own shadow, in one group with
            // nothing between them; the same for an ore mine)
            if frame + frames / 2 < frames {
                items.extend(self.place_sprite(
                    &sprite,
                    frame + frames / 2,
                    p,
                    test,
                    ItemDepth::Profile {
                        profile: Profile::Flat,
                        bias: -lz - 3,
                        companion: None,
                    },
                    Paint::Shadow,
                    Group::Shadows,
                    false,
                    Some(object),
                ));
            }
            if animated {
                animated_entries.push(GroundEntry {
                    // spec: S-0102 claim 5 (sort key X + Y of its position, the centre of its cell)
                    key: i64::from(x) + i64::from(y),
                    items,
                    object: Some(object),
                });
            } else {
                self.built.scene.items.extend(items);
            }
        }
        animated_entries
    }

    /// Whether a pre-placed building with health value `value` starts
    /// damaged and badly damaged (S-0009 claim 37, S-0108 F3). Undamaged
    /// without the typed rules.
    fn building_condition(&mut self, idx: usize, value: i32) -> (bool, bool) {
        let Some(r) = self.rules else {
            self.note(notes::NO_RULES_HEALTH);
            return (false, false);
        };
        let name = &self.import.outcome.registries[&Kind::Buildings][idx].name;
        let Some(strength) = glasnost_sim::objects::type_index(r, SimKind::Building, name)
            .and_then(|t| r.buildings.get(t as usize))
            .map(|b| b.strength)
        else {
            return (false, false);
        };
        if strength == 0 {
            return (false, false);
        }
        // spec: S-0009 claim 37 (raised to full strength above S − 3, quirks.near_full_health_rounds_up)
        let (h, _) = starting_health(
            HealthKind::Building,
            value,
            strength,
            r.quirks.near_full_health_rounds_up,
            r.quirks.chop_rounding,
        );
        condition_of(r, h, strength)
    }

    /// The foundation W × H of a building (S-0108 Import mapping).
    fn foundation(&self, idx: usize) -> (i32, i32) {
        const WORDS: [(&str, (i32, i32)); 22] = [
            ("1x1", (1, 1)),
            ("2x1", (2, 1)),
            ("1x2", (1, 2)),
            ("2x2", (2, 2)),
            ("2x3", (2, 3)),
            ("3x2", (3, 2)),
            ("3x3", (3, 3)),
            ("3x5", (3, 5)),
            ("4x2", (4, 2)),
            ("3x3Refinery", (3, 3)),
            ("1x3", (1, 3)),
            ("3x1", (3, 1)),
            ("4x3", (4, 3)),
            ("1x4", (1, 4)),
            ("1x5", (1, 5)),
            ("2x6", (2, 6)),
            ("2x5", (2, 5)),
            ("5x3", (5, 3)),
            ("4x4", (4, 4)),
            ("3x4", (3, 4)),
            ("6x4", (6, 4)),
            ("0x0", (0, 0)),
        ]; // spec: S-0108 Import mapping (foundation vocabulary)
        let read = |section: &[u8]| {
            let v = self.p.art.value(section, b"Foundation")?;
            // spec: S-0108 Import mapping (text cut to 31 bytes, compared ignoring case)
            let v = value::text(v, 31);
            Some(
                WORDS
                    .iter()
                    .find(|(w, _)| w.as_bytes().eq_ignore_ascii_case(v))
                    .map_or((1, 1), |(_, s)| *s),
            )
        };
        let name = &self.import.outcome.registries[&Kind::Buildings][idx].name;
        let image = self
            .import
            .arts
            .get(&(Kind::Buildings, idx))
            .map_or(name.clone(), |a| a.image.clone());
        // spec: S-0108 Import mapping (the id's section decides when it gives a size other than 1 × 1)
        match read(name) {
            Some(s) if s != (1, 1) => s,
            _ => read(&image).unwrap_or((1, 1)),
        }
    }

    fn art_value(&self, kind: Kind, idx: usize, key: &[u8]) -> Option<&[u8]> {
        let image = self.import.arts.get(&(kind, idx))?.image.clone();
        let doc: &Document = &self.p.art;
        doc.value(&image, key)
    }

    /// Buildings in the cell pass (S-0108, S-0102 claims 13 and 16).
    fn structures(&mut self, ground: &mut Vec<GroundEntry>) {
        let list = self.map.objects.structures.clone();
        let mut any = false;
        for (n, s) in list.iter().enumerate() {
            let p = &s.placed;
            let fields = self.map.objects.fields(ListKind::Buildings, n).clone();
            if fields.short {
                // spec: S-0009 claim 36 (an entry that lacks a required field is dropped, a departure)
                self.drop_object("entry lacks a required field");
                continue;
            }
            if !self.owner_kept(&p.owner) {
                self.drop_object("owner no skirmish house carries");
                continue;
            }
            let Some(i) = find_type(self.import, Kind::Buildings, &p.type_name) else {
                self.drop_object("unknown building type");
                continue;
            };
            let Some(art) = self.import.arts.get(&(Kind::Buildings, i)).cloned() else {
                self.drop_object("building without art");
                continue;
            };
            let Some(cell) = self.map.cells.get(p.cell).copied() else {
                self.drop_object("building on a non-legal cell");
                continue;
            };
            any = true;
            let object = self.add_object(ObjectInfo {
                kind: "building",
                type_name: String::from_utf8_lossy(&p.type_name).into_owned(),
                owner: String::from_utf8_lossy(&p.owner).into_owned(),
                cell: p.cell,
                entry: p
                    .raw
                    .iter()
                    .map(|f| String::from_utf8_lossy(f).into_owned())
                    .collect::<Vec<_>>()
                    .join(","),
                notes: Vec::new(),
            });
            if fields.upgrade_count.unwrap_or(0) > 0 {
                self.note(notes::UPGRADES);
            }
            let condition = self.building_condition(i, p.health);
            // spec: S-0009 claim 39 (a building's turret starts at 256 × (v mod 256))
            let facing = facing_from_value(p.facing);
            let items = self.building_items(
                i, &art, p.cell, cell.level, &p.owner, condition, facing, object, ground,
            );
            self.built.scene.items.extend(items);
        }
        if any {
            self.note(notes::NO_COMPANION);
        }
    }

    /// The items of one building of type `idx` with its foundation's
    /// top-left cell at `cell` (height level `level`): body, shadow and bib
    /// (returned, for the cell pass) and its voxel turret or barrel (pushed
    /// to `ground` as a ground-layer entry). `condition` is (damaged, badly
    /// damaged); `facing` the turret's facing.
    #[allow(clippy::too_many_arguments)]
    fn building_items(
        &mut self,
        i: usize,
        art: &glasnost_convert::art::TypeArt,
        cell: CellXY,
        level: i8,
        owner: &[u8],
        condition: (bool, bool),
        facing: u16,
        object: u32,
        ground: &mut Vec<GroundEntry>,
    ) -> Vec<Item> {
        let mut items = Vec::new();
        let (hsv, _) = self.owner_colour(owner);
        let z = LEVEL_LEPTONS * i32::from(level);
        let lz = height_lift(z);
        // spec: S-0108 claim 8, S-0009 claim 38 (centred on the top corner of the foundation's top-left cell)
        let anchor = project_lifted(256 * cell.x, 256 * cell.y, z);
        let u = self
            .p
            .lights
            .get(cell)
            .map_or(NEUTRAL_BRIGHTNESS, |l| l.unscaled_ground);
        let terrain_palette = self
            .art_value(Kind::Buildings, i, b"TerrainPalette")
            .and_then(value::boolean)
            .unwrap_or(false);
        let row = if terrain_palette {
            // spec: S-0103 claim 8, S-0104 claim 13 (terrain-palette building: the centre cell's table and scaled ground)
            let (w, h) = self.foundation(i);
            let (cx, cy) = light::building_centre_cell((cell.x, cell.y), (w, h));
            let centre = CellXY::new(cx, cy);
            let l = self.p.lights.get(centre);
            self.tile_row(
                l.map_or(TileTable::Neutral, |l| l.table),
                l.map_or(NEUTRAL_BRIGHTNESS, |l| l.scaled_ground),
            )
        } else {
            // spec: S-0104 claim 13 (an ordinary building: U of its top-left foundation cell)
            self.house_row(hsv, u)
        };
        let Some(sprite) = art.sprite.clone() else {
            return items;
        };
        let n = self.sprite_frames(&sprite).unwrap_or(0);
        // spec: S-0009 claim 40 (a map-placed building is complete and idle at once);
        // S-0108 claim 6 (a change during the game's set-up is immediate: stage = a of the range)
        let idle = self
            .art_value(Kind::Buildings, i, b"AnimIdle")
            .map(|v| value::int_tuple_or::<3>(v, [0, 1, 0]).0)
            .unwrap_or([0, 1, 0]);
        // spec: S-0108 claim 3 (a negative start is kept: a negative index draws nothing)
        let k = idle[0];
        let occupiable = rule_bool(self.import, Kind::Buildings, i, b"CanBeOccupied", false);
        let gate = rule_bool(self.import, Kind::Buildings, i, b"Gate", false);
        let (damaged, badly) = condition;
        // spec: S-0108 claim 2 (outside construction the occupiable rule comes before the stationary
        // gate rule: occupiable 2·[occupied] + [hurt], hurt badly damaged at tech level 0 or below,
        // damaged above; a gate 0, damaged G + 1; others k, damaged k + 1 in the idle state)
        let f: i32 = if occupiable {
            // spec: S-0108 Import mapping (TechLevel absent: 255)
            let tech = rule_int(self.import, Kind::Buildings, i, b"TechLevel", 255);
            // spec: S-0108 claim 2 (quirks.occupiable_hurt_rule)
            i32::from(occupiable_hurt(
                tech,
                damaged,
                badly,
                &ViewQuirks::default(),
            ))
        } else if gate {
            if damaged {
                // spec: S-0108 Import mapping (GateStages, an art key: integer, absent 9)
                let g = self
                    .art_value(Kind::Buildings, i, b"GateStages")
                    .and_then(value::integer)
                    .unwrap_or(9);
                g.saturating_add(1)
            } else {
                0
            }
        } else {
            k.saturating_add(i32::from(damaged))
        };
        let n_i = i32::try_from(n).unwrap_or(i32::MAX);
        // spec: S-0108 claim 3 (drawn frame min(f, ⌊N/2⌋ − 1); with no body frames nothing is drawn)
        let body_frame = drawn_body_frame(f, n_i);
        // spec: S-0108 claim 3, F2 (an index below 0 or at or past N draws nothing)
        let frame_of = |x: i32| usize::try_from(x).ok().filter(|&x| x < n);
        let normal_z = self
            .art_value(Kind::Buildings, i, b"NormalZAdjust")
            .and_then(value::integer)
            .unwrap_or(0);
        // spec: S-0102 claim 16 (each building's group: body, then shadow, then bib)
        // spec: S-0102 claim 13 (body: −L(z) − 2 + normal depth adjustment, upright, writes)
        if let Some(frame) = frame_of(body_frame)
            && let Some(item) = self.place_sprite(
                &sprite,
                frame,
                anchor,
                DepthTest::Writer,
                ItemDepth::Profile {
                    profile: Profile::Upright,
                    bias: -lz - 2 + normal_z,
                    companion: None,
                },
                Paint::Lit(row),
                Group::Buildings,
                true,
                Some(object),
            )
        {
            items.push(item);
        }
        // spec: S-0108 claim 9 (shadow frame F + ⌊N/2⌋ unless the type suppresses shadows; none past the sprite)
        let shadow = frame_of(body_frame.saturating_add(n_i / 2));
        if !rule_bool(self.import, Kind::Buildings, i, b"NoShadow", false)
            && let Some(shadow) = shadow
            && let Some(item) = self.place_sprite(
                &sprite,
                shadow,
                anchor,
                DepthTest::Writer,
                // spec: S-0102 claim 13 (building shadow: −L(z) − 4, flat, writes)
                ItemDepth::Profile {
                    profile: Profile::Flat,
                    bias: -lz - 4,
                    companion: None,
                },
                Paint::Shadow,
                Group::Shadows,
                true,
                Some(object),
            )
        {
            items.push(item);
        }
        // spec: S-0108 claim 7 (bib: same frame f, no clamp, outside the construction state)
        // spec: S-0108 claim 3 (a negative bib index draws nothing; one past the bib sprite neither)
        if let Some(bib) = art.building_sprites.get("bib").cloned()
            && let Ok(bib_frame) = usize::try_from(f)
            && let Some(item) = self.place_sprite(
                &bib,
                bib_frame,
                anchor,
                DepthTest::Writer,
                // spec: S-0102 claim 13 (bib: −L(z) − 3, flat, writes)
                ItemDepth::Profile {
                    profile: Profile::Flat,
                    bias: -lz - 3,
                    companion: None,
                },
                Paint::Lit(row),
                Group::Bibs,
                true,
                Some(object),
            )
        {
            items.push(item);
        }
        self.slot_animations(i, cell, z, u, hsv, damaged, object, ground);
        // Voxel turret or barrel in the object pass (S-0105 claims 5, 9, 37; S-0102 claims 5, 13).
        let turret_voxel = rule_bool(self.import, Kind::Buildings, i, b"TurretAnimIsVoxel", false);
        if turret_voxel || art.voxels.contains_key("barrel") {
            let items = self.building_voxels(i, art, facing, anchor, lz, u, hsv, object);
            // spec: S-0102 F4 (key = 256·(c_x + c_y) + 32 for a voxel turret − 16 for a gate)
            let key = i64::from(256 * (cell.x + cell.y)) + if turret_voxel { 32 } else { 0 }
                - if gate { 16 } else { 0 };
            if !items.is_empty() {
                ground.push(GroundEntry {
                    key,
                    items,
                    object: Some(object),
                });
            }
        }
        items
    }

    /// The slot animations a building starts when it is placed (S-0108
    /// claims 10 and 11), each drawn at its first image frame as an
    /// animation of its own in its type's layer (S-0102 claim 17): ground
    /// layer entries go to `ground`, the others to [`Built::layered`].
    #[allow(clippy::too_many_arguments)]
    fn slot_animations(
        &mut self,
        i: usize,
        cell: CellXY,
        z: i32,
        u: i32,
        hsv: [u8; 3],
        damaged: bool,
        object: u32,
        ground: &mut Vec<GroundEntry>,
    ) {
        let Some(image) = self
            .import
            .arts
            .get(&(Kind::Buildings, i))
            .map(|a| a.image.clone())
        else {
            return;
        };
        let refinery = rule_bool(self.import, Kind::Buildings, i, b"Refinery", false);
        // spec: S-0108 claim 11 (placed during the set-up: idle and active-1 … active-4, occupied
        // variants allowed; grand opening of a building completed in play: a refinery only the active
        // slot of its fill level, which is active-1 with no stored ore by claim 15, others the same
        // five slots)
        let starts: &[&[u8]] = if self.live && refinery {
            &slots::SETUP_SLOTS[1..2]
        } else {
            &slots::SETUP_SLOTS
        };
        let defs: Vec<slots::SlotDef> = starts
            .iter()
            .map(|base| slots::read_slot(&self.p.art, &image, base))
            .collect();
        let mut any = false;
        for def in defs {
            // spec: S-0108 claim 11 (no occupants at placement: the damaged or the normal variant)
            let name = slots::variant(&def, damaged, false, true).to_vec();
            if name.is_empty() {
                continue;
            }
            // Unknown names start nothing here (S-0108 claim 11 leaves them unmeasured).
            let Some(a) = find_type(self.import, Kind::Animations, &name) else {
                continue;
            };
            let Some(art) = self.import.arts.get(&(Kind::Animations, a)).cloned() else {
                continue;
            };
            let Some(sprite) = art.sprite.clone() else {
                continue;
            };
            any = true;
            let t = slots::read_anim(&self.p.art, &art.image);
            // spec: S-0109 claim 19 (detail field above the player's level: not drawn)
            if !slots::shown_at(&t, self.options.detail.number()) {
                continue;
            }
            let n = i32::try_from(self.sprite_frames(&sprite).unwrap_or(0)).unwrap_or(i32::MAX);
            // spec: S-0108 claim 10, S-0101 F6 (the anchor moved by the displacement, converted to
            // world units)
            let (dx, dy) = screen_to_ground(def.x, def.y);
            let pos = (256 * cell.x + dx, 256 * cell.y + dy, z);
            let lz = height_lift(z);
            let p = project_lifted(pos.0, pos.1, pos.2);
            let point = (p.0, p.1 + t.y_draw_offset);
            let row = self.anim_row(&t, hsv, u, CellXY::new(pos.0 >> 8, pos.1 >> 8));
            if slots::first_translucency(&t, self.options.detail.number(), n) > 0 {
                self.note(notes::SLOT_ANIM_TRANSLUCENT);
            }
            if t.tiled {
                self.note(notes::SLOT_ANIM_TILED);
            }
            let frame = slots::first_frame(&t, n);
            let frame_of = |x: i32| {
                usize::try_from(x)
                    .ok()
                    .filter(|&x| (x as i64) < i64::from(n))
            };
            let mut items = Vec::new();
            // spec: S-0109 claim 10 (a shadow type also draws f + ⌊N/2⌋ as its shadow, unless flat or tiled)
            if t.shadow
                && !t.flat
                && !t.tiled
                && let Some(sf) = frame_of(frame.saturating_add(n / 2))
            {
                self.note(notes::SLOT_ANIM_SHADOW_ORDER);
                if let Some(item) = self.place_sprite(
                    &sprite,
                    sf,
                    point,
                    DepthTest::Reader,
                    // spec: S-0102 claim 13 (animation shadow: −2 − L(z), flat)
                    ItemDepth::Profile {
                        profile: Profile::Flat,
                        bias: -2 - lz,
                        companion: None,
                    },
                    Paint::Shadow,
                    Group::Buildings,
                    true,
                    Some(object),
                ) {
                    items.push(item);
                }
            }
            // spec: S-0102 claims 13 and 17 (animation rows; the slot's depth adjustment replaces the type's)
            let (profile, bias) = if t.tiled {
                (Profile::Upright, t.y_draw_offset + def.z_adjust - lz - 50)
            } else if t.flat {
                (Profile::Flat, t.y_draw_offset + def.z_adjust - lz - 3)
            } else {
                (Profile::Upright, t.y_draw_offset + def.z_adjust - lz - 2)
            };
            if let Some(f) = frame_of(frame)
                && let Some(item) = self.place_sprite(
                    &sprite,
                    f,
                    point,
                    // spec: S-0102 claim 17 (slot animations are depth readers)
                    DepthTest::Reader,
                    ItemDepth::Profile {
                        profile,
                        bias,
                        companion: None,
                    },
                    Paint::Lit(row),
                    Group::Buildings,
                    true,
                    Some(object),
                )
            {
                items.push(item);
            }
            if items.is_empty() {
                continue;
            }
            let entry = GroundEntry {
                // spec: S-0102 claims 5 and 17 (key X + Y of the position plus the slot's y-sort)
                key: i64::from(pos.0) + i64::from(pos.1) + i64::from(def.y_sort),
                items,
                object: Some(object),
            };
            // spec: S-0102 claims 3 and 17 (an unattached animation is in its type's layer)
            if t.layer == slots::Layer::Ground {
                ground.push(entry);
            } else {
                self.built.layered.push((t.layer, entry));
            }
        }
        if any {
            self.note(notes::SLOT_ANIMS_STILL);
            self.note(notes::SLOT_ANIM_DRAW_POINT);
        }
    }

    /// The shade row of a building's slot animation of type `t` (S-0103
    /// claim 8 rows 3 to 5): `hsv` and `u` are its building's colour
    /// definition and brightness, `cell` the animation's cell.
    fn anim_row(&mut self, t: &slots::AnimType, hsv: [u8; 3], u: i32, cell: CellXY) -> Shade {
        if t.cell_drawer {
            // spec: S-0103 claim 8 row 3 (the owner's 53-shade palette at the building's U, or 1000
            // with the normal-light flag)
            return self.house_row(
                hsv,
                if t.normal_light {
                    NEUTRAL_BRIGHTNESS
                } else {
                    u
                },
            );
        }
        if t.alt_palette {
            let (base, n) = match self.p.rows.alt {
                Some(r) => r,
                None => {
                    // spec: S-0103 claim 8 row 4, claim 5 (the single-shade palette of the first colour
                    // definition, never tinted, S-0104 claim 3)
                    let first = self
                        .import
                        .outcome
                        .colours
                        .first()
                        .map_or([0, 0, 0], |c| c.hsv);
                    let pal = house_palette(&self.palettes.unit, first);
                    let t = light::shade_table(&pal, WHITE, 1, None);
                    let base = self.built.scene.rows.len();
                    for row in &t {
                        self.built.scene.add_row(row565(row));
                    }
                    self.p.rows.alt = Some((base, t.len()));
                    (base, t.len())
                }
            };
            return Shade::new(base, n, NEUTRAL_BRIGHTNESS);
        }
        let (base, n) = match self.p.rows.anim {
            Some(r) => r,
            None => {
                // spec: S-0103 claim 8 row 5, S-0104 claim 6 (the anim palette's untinted table)
                let t = light::untinted_table(&self.palettes.anim);
                let base = self.built.scene.rows.len();
                for row in &t {
                    self.built.scene.add_row(row565(row));
                }
                self.p.rows.anim = Some((base, t.len()));
                (base, t.len())
            }
        };
        // spec: S-0103 claim 8 row 5, S-0104 claim 13 (its cell's unscaled ground level; 1000 with
        // the normal-light flag)
        let b = if t.normal_light {
            NEUTRAL_BRIGHTNESS
        } else {
            self.p
                .lights
                .get(cell)
                .map_or(NEUTRAL_BRIGHTNESS, |l| l.unscaled_ground)
        };
        Shade::new(base, n, b)
    }

    /// A building's voxel turret and barrel, or its barrel alone (S-0105
    /// claims 5, 9 to 11, 37; S-0102 claim 13).
    #[allow(clippy::too_many_arguments)]
    fn building_voxels(
        &mut self,
        idx: usize,
        art: &glasnost_convert::art::TypeArt,
        facing: u16,
        anchor: (i32, i32),
        lz: i32,
        brightness: i32,
        hsv: [u8; 3],
        object: u32,
    ) -> Vec<Item> {
        let q = VoxelQuirks::default();
        let vpl = self.vpl();
        let k = Kind::Buildings;
        // spec: S-0105 Terms (turret anchor: the draw point moved by TurretAnimX and TurretAnimY, rules keys);
        // S-0009 claim 38 (the draw point is the S-0108 claim 8 image anchor)
        let at = (
            anchor.0 + rule_int(self.import, k, idx, b"TurretAnimX", 0),
            anchor.1 + rule_int(self.import, k, idx, b"TurretAnimY", 0),
        );
        let offset = self
            .art_value(k, idx, b"TurretOffset")
            .and_then(value::integer)
            .unwrap_or(0);
        // spec: S-0105 claim 6 (a building's elevation is level until it re-aims to fire)
        let elevation = voxel::ELEVATION_LEVEL;
        let name = self.import.outcome.registries[&k][idx].name.clone();
        let depth_buffered = self
            .p
            .art
            .value(&name, b"UseBuffer")
            .and_then(value::boolean)
            .unwrap_or(false);
        // spec: S-0102 claim 13 (a building's voxel parts: −L(z) + the turret slot depth adjustment)
        let bias = -lz + rule_int(self.import, k, idx, b"TurretAnimZAdjust", 0);
        // spec: S-0104 claim 13 (every part of an ordinary building: U of its top-left cell)
        let row = self.house_row(hsv, brightness);
        let turret = art.voxels.get("turret").and_then(|f| self.voxel_model(f));
        let barrel = art.voxels.get("barrel").and_then(|f| self.voxel_model(f));
        let mut items = Vec::new();
        // spec: S-0105 claim 16 (an animation-frame counter of 0 before any attack: frame 0)
        let counter = 0;
        let parts = match turret {
            Some(t) => {
                let pose = voxel::building_turret_parts(facing, offset, elevation, &q);
                let turret_frame = voxel::building_turret_frame(counter, t.frame_count() as u32);
                // spec: S-0105 claim 16 (quirks.barrel_frame_zero)
                let barrel_frame = voxel::barrel_frame_with_turret(turret_frame, &q);
                // spec: S-0105 claim 26 (building turret key; the barrel's is the turret's
                // plus the current 32-step elevation)
                let turret_key = voxel::building_turret_key(facing, turret_frame);
                let mut v = vec![
                    (
                        Some(t),
                        pose.turret,
                        turret_frame,
                        voxel::ImageKey::Turret(turret_key),
                    ),
                    (
                        barrel,
                        pose.barrel,
                        barrel_frame,
                        voxel::barrel_key(turret_key, elevation),
                    ),
                ];
                // spec: S-0105 claim 11 (turret first when its 4-step facing is 1 or 2)
                if !pose.turret_first {
                    v.reverse();
                }
                v
            }
            // spec: S-0105 claim 37 (a barrel without a turret: V · Rz(θ) · Ry(φ), frame from the counter;
            // kept with the barrel images under its facing, frame and elevation)
            None => {
                let frame = barrel.as_ref().map_or(0, |b| {
                    voxel::barrel_only_frame(counter, b.frame_count() as u32)
                });
                vec![(
                    barrel,
                    voxel::building_barrel_only(facing, elevation, &q),
                    frame,
                    voxel::barrel_only_key(facing, frame, elevation),
                )]
            }
        };
        for (model, draw, frame, key) in parts {
            let Some(m) = model else {
                continue;
            };
            if let Some(part) = self.cached_part(k, idx, key, |b| {
                b.p.canvas
                    .draw_part(&m, frame, &draw, vpl.as_ref(), depth_buffered, &q)
            }) {
                items.push(Self::part_item(
                    part,
                    at,
                    bias,
                    Paint::Lit(row),
                    Group::Buildings,
                    object,
                ));
            }
        }
        items
    }

    /// Map-placed vehicles, infantry and aircraft in the ground layer
    /// (S-0102 claims 3 and 5) in their initial state: on the ground, every
    /// one of them is in the ground layer.
    fn mobiles(&mut self, ground: &mut Vec<GroundEntry>) {
        // spec: S-0009 claim 43 (vehicles, then aircraft, then infantry, each in entry order)
        for lk in [ListKind::Vehicles, ListKind::Aircraft, ListKind::Infantry] {
            let kind = match lk {
                ListKind::Vehicles => Kind::Vehicles,
                ListKind::Aircraft => Kind::Aircraft,
                _ => Kind::Infantry,
            };
            let entries: Vec<_> = self.map.objects.list(lk).into_iter().cloned().collect();
            for (n, p) in entries.iter().enumerate() {
                if self.map.objects.fields(lk, n).short {
                    // spec: S-0009 claim 36 (an entry that lacks a required field is dropped, a departure)
                    self.drop_object("entry lacks a required field");
                    continue;
                }
                let sub = (lk == ListKind::Infantry)
                    .then(|| self.map.objects.infantry.get(n).map_or(0, |i| i.sub_cell));
                if let Some(g) = self.placed_mobile(kind, p, sub) {
                    ground.push(g);
                }
            }
        }
    }

    /// Common checks for a placed mobile: owner, type, art and cell.
    fn mobile_start(
        &mut self,
        kind: Kind,
        p: &glasnost_map::objects::Placed,
    ) -> Option<(usize, glasnost_map::Cell)> {
        if !self.owner_kept(&p.owner) {
            self.drop_object("owner no skirmish house carries");
            return None;
        }
        let Some(i) = find_type(self.import, kind, &p.type_name) else {
            self.drop_object("unknown unit type");
            return None;
        };
        if !self.import.arts.contains_key(&(kind, i)) {
            self.drop_object("unit without art");
            return None;
        }
        let Some(cell) = self.map.cells.get(p.cell).copied() else {
            self.drop_object("unit on a non-legal cell");
            return None;
        };
        Some((i, cell))
    }

    /// A map-placed mobile in its initial state, before any tick.
    fn placed_mobile(
        &mut self,
        kind: Kind,
        p: &glasnost_map::objects::Placed,
        sub_cell: Option<i32>,
    ) -> Option<GroundEntry> {
        let (i, cell) = self.mobile_start(kind, p)?;
        let (x, y) = match sub_cell {
            Some(sub) => {
                // spec: S-0009 claim 38 (an infantry unit stands at standing position field 6, S-0208 table P;
                // outside 0 … 4 the centre, S-0208 claim 5's departure)
                let sp = u8::try_from(sub).ok().filter(|s| *s <= 4).unwrap_or(0);
                let (u, v) = glasnost_sim::walk::standing_point(sp);
                (256 * p.cell.x + u, 256 * p.cell.y + v)
            }
            // spec: S-0009 claim 38 (a vehicle or aircraft stands on the centre of its cell)
            None => cell_centre(p.cell.x, p.cell.y),
        };
        let z = LEVEL_LEPTONS * i32::from(cell.level);
        if kind == Kind::Aircraft
            && !self
                .map
                .footprint
                .is_usable(p.cell, i32::from(cell.level), cell.slope != 0)
        {
            self.note(notes::AIRCRAFT_AIR);
        }
        // spec: S-0009 claim 39 (body facing 256 × (v mod 256); infantry: field 8)
        let facing = facing_from_value(p.facing);
        let fire_angle = rule_int(
            self.import,
            kind,
            i,
            b"FireAngle",
            voxel::DEFAULT_FIRE_ANGLE,
        );
        let infantry = (kind == Kind::Infantry).then(|| {
            // spec: S-0107 claim 2 (no current sequence: tread on a water cell, else stand; the land type
            // needs the loaded cells, so without a game every unit shows stand)
            let water = self
                .cells_loaded
                .and_then(|g| g.get(p.cell))
                .is_some_and(|c| c.land == glasnost_map::terrain::land::WATER);
            let key: &[u8] = if water { b"Tread" } else { b"Ready" };
            let (start, count, multiplier) = self.sequence_entry(i, key);
            InfantryFrame {
                start,
                count,
                multiplier,
                stage: 0,
            }
        });
        let object = self.add_object(ObjectInfo {
            kind: match kind {
                Kind::Infantry => "infantry",
                Kind::Aircraft => "aircraft",
                _ => "vehicle",
            },
            type_name: String::from_utf8_lossy(&p.type_name).into_owned(),
            owner: String::from_utf8_lossy(&p.owner).into_owned(),
            cell: p.cell,
            entry: p
                .raw
                .iter()
                .map(|f| String::from_utf8_lossy(f).into_owned())
                .collect::<Vec<_>>()
                .join(","),
            notes: Vec::new(),
        });
        let m = Mobile {
            kind,
            idx: i,
            owner: p.owner.clone(),
            pos: (x, y, z),
            body: facing,
            // spec: S-0009 claim 39 (a vehicle's or aircraft's turret starts at its body facing)
            turret: facing,
            // spec: S-0105 claim 6 (placed level, then turned toward the fire angle; no tick has run)
            elevation: voxel::barrel_elevation_after(fire_angle, 0),
            infantry,
            object,
        };
        Some(self.mobile(&m))
    }

    /// The unit depth adjustment *U* = −L(z) + s + F (S-0102 claim 14) for a
    /// unit of `kind`/`idx` standing on cell `c` at world height `z`. Only
    /// the cliff fudge can be judged: bridges, columns and tunnels need
    /// specs not written yet, so their factors are 0.
    fn unit_depth(&self, kind: Kind, idx: usize, c: CellXY, z: i32) -> i32 {
        let level = |dx: i32| {
            self.map
                .cells
                .get(CellXY::new(c.x + dx, c.y + dx))
                .map(|cell| i32::from(cell.level))
        };
        let own = level(0).unwrap_or(0);
        // spec: S-0102 claim 14 (a cell is high when more than 3 levels above the unit's cell)
        let high = |dx: i32| level(dx).is_some_and(|l| l - own > CLIFF_LEVELS);
        // spec: S-0102 claim 14 (cliff factor, quirks.cliff_fudge_far_cliff_weaker)
        let factor = cliff_factor(high(1), high(2), &ViewQuirks::default());
        let fudge = rule_int(self.import, kind, idx, b"ZFudgeCliff", DEFAULT_CLIFF_FUDGE);
        // spec: S-0102 claim 14 (U = −L(z) + s + F, F the largest fudge)
        -height_lift(z) + SITUATIONAL + (factor * fudge).max(0)
    }

    /// The rules' extra-light value in ‰ (S-0104 claim 13), 0 when absent.
    fn extra_light(&self, key: &[u8]) -> i32 {
        self.import
            .outcome
            .globals
            .get(&b"AudioVisual"[..])
            .and_then(|s| s.get(key))
            .and_then(|k| k.derived)
            .map_or(0, |v| v as i32)
    }

    /// The sequence entry (start, count, multiplier) of `key` for infantry
    /// type `idx` (S-0107 Import mapping; claim 1: absent fields are 0).
    fn sequence_entry(&self, idx: usize, key: &[u8]) -> (i32, i32, i32) {
        // spec: S-0107 Import mapping (values read as at most 31 bytes of text)
        const VALUE_LIMIT: usize = 31; // spec: S-0107 Constants (value length limit)
        let Some(seq) = self.art_value(Kind::Infantry, idx, b"Sequence") else {
            // spec: S-0107 claim 1 (no sequence table: every entry is zero)
            return (0, 0, 0);
        };
        let seq = value::text(seq, VALUE_LIMIT).to_vec();
        let Some(v) = self.p.art.value(&seq, key) else {
            return (0, 0, 0);
        };
        let v = &v[..v.len().min(VALUE_LIMIT)];
        // spec: S-0107 Import mapping (a missing or unparseable field keeps it and every later field at 0)
        let [s, c, m] = value::int_tuple::<3>(v);
        match (s, c, m) {
            (Some(s), Some(c), Some(m)) => (s, c, m),
            (Some(s), Some(c), None) => (s, c, 0),
            (Some(s), None, _) => (s, 0, 0),
            _ => (0, 0, 0),
        }
    }

    /// A mobile object's ground-layer entry: its items and sort key.
    fn mobile(&mut self, m: &Mobile) -> GroundEntry {
        let (x, y, z) = m.pos;
        let point = project_lifted(x, y, z);
        let cell = CellXY::new(x / 256, y / 256); // spec: S-0204 claim 2 (the cell holding a position)
        let u = self.unit_depth(m.kind, m.idx, cell, z);
        let (hsv, _) = self.owner_colour(&m.owner);
        let ground = self
            .p
            .lights
            .get(cell)
            .map_or(NEUTRAL_BRIGHTNESS, |l| l.unscaled_ground);
        let items = if m.kind == Kind::Infantry {
            // spec: S-0104 claim 13 (infantry: U + extra infantry light; shadowed cells need the theater spec)
            let row = self.house_row(hsv, ground + self.extra_light(b"ExtraInfantryLight"));
            self.infantry_items(m, point, u, row)
        } else {
            // spec: S-0104 claim 13 (vehicles: U + extra unit light; aircraft on the ground: U + extra aircraft light)
            let extra = if m.kind == Kind::Aircraft {
                self.extra_light(b"ExtraAircraftLight")
            } else {
                self.extra_light(b"ExtraUnitLight")
            };
            let row = self.house_row(hsv, ground + extra);
            self.vehicle_items(m, point, u, row)
        };
        GroundEntry {
            // spec: S-0102 claim 5 (sort key X + Y of the position; height plays no part)
            key: i64::from(x) + i64::from(y),
            items,
            object: Some(m.object),
        }
    }

    /// An infantry unit's body and own shadow (S-0107 claims 2, 3, 19;
    /// S-0102 claims 13, 20 and 28): the shadow right after the body, both
    /// centred on the draw point. Units in the viewer stand on the ground
    /// (height above the surface 0).
    fn infantry_items(&mut self, m: &Mobile, point: (i32, i32), u: i32, row: Shade) -> Vec<Item> {
        let Some(sprite) = self
            .import
            .arts
            .get(&(m.kind, m.idx))
            .and_then(|a| a.sprite.clone())
        else {
            return Vec::new();
        };
        let n = self.sprite_frames(&sprite).unwrap_or(0);
        let f = m.infantry.unwrap_or_default();
        let Ok(frame) = usize::try_from(infantry_frame(&f, m.body)) else {
            return Vec::new();
        };
        let lz = height_lift(m.pos.2);
        let mut items = Vec::new();
        // spec: S-0102 claim 13 (infantry body, h = 0: U − 12, upright, reader)
        if let Some(item) = self.place_sprite(
            &sprite,
            frame,
            point,
            DepthTest::Reader,
            ItemDepth::Profile {
                profile: Profile::Upright,
                bias: u - 12,
                companion: None,
            },
            Paint::Lit(row),
            Group::Infantry,
            false,
            Some(m.object),
        ) {
            items.push(item);
        }
        // spec: S-0107 claim 19 (on the ground: shadow frame + ⌊L/2⌋ at the same position, unless suppressed);
        // S-0102 claims 20 and 28 (drawn right after the body; it darkens the body's own pixels)
        if !rule_bool(self.import, m.kind, m.idx, b"NoShadow", false)
            && let Some(item) = self.place_sprite(
                &sprite,
                frame + n / 2,
                point,
                DepthTest::Reader,
                // spec: S-0102 claim 13 (own shadow of an infantry unit, h = 0: −L(z) − 4, flat)
                ItemDepth::Profile {
                    profile: Profile::Flat,
                    bias: -lz - 4,
                    companion: None,
                },
                Paint::Shadow,
                Group::Shadows,
                false,
                Some(m.object),
            )
        {
            items.push(item);
        }
        items
    }

    /// A vehicle's or aircraft's items: voxel parts (S-0105), or for a
    /// vehicle a sprite (S-0102 claims 21 and 22). An aircraft whose art is
    /// not a voxel model shows no body (claim 21).
    fn vehicle_items(&mut self, m: &Mobile, point: (i32, i32), u: i32, row: Shade) -> Vec<Item> {
        let Some(art) = self.import.arts.get(&(m.kind, m.idx)).cloned() else {
            return Vec::new();
        };
        let group = if m.kind == Kind::Aircraft {
            Group::Aircraft
        } else {
            Group::Vehicles
        };
        if art.voxels.contains_key("body") {
            self.vehicle_voxels(m, &art, point, u, row, group)
        } else if m.kind == Kind::Vehicles
            && let Some(sprite) = art.sprite.clone()
        {
            self.sprite_vehicle(m, &sprite, point, u, row)
        } else {
            // spec: S-0102 claim 21 (an aircraft is never drawn as a sprite)
            Vec::new()
        }
    }

    /// A vehicle drawn as a sprite (S-0102 claims 13, 21, 22 and 28).
    fn sprite_vehicle(
        &mut self,
        m: &Mobile,
        sprite: &[u8],
        point: (i32, i32),
        u: i32,
        row: Shade,
    ) -> Vec<Item> {
        let (k, i) = (m.kind, m.idx);
        let n = self.sprite_frames(sprite).unwrap_or(0);
        let art_int = |b: &Self, key: &[u8]| b.art_value(k, i, key).and_then(value::integer);
        let turret = rule_bool(self.import, k, i, b"Turret", false);
        // spec: S-0102 Import mapping (WalkFrames 12, FiringFrames 0)
        let walk_frames = art_int(self, b"WalkFrames").unwrap_or(12);
        let firing_frames = art_int(self, b"FiringFrames").unwrap_or(0);
        // spec: S-0102 Import mapping (Facings 8, but 1 without firing frames and turret)
        let facings =
            art_int(self, b"Facings").unwrap_or(if firing_frames == 0 && !turret { 1 } else { 8 });
        // spec: S-0102 Import mapping (standing frames 0, or 1 with firing frames; walking start 0;
        // standing start F_w·n with standing frames, else the walking start)
        let standing_frames = i32::from(firing_frames > 0);
        let walk_start = 0;
        let stand_start = if standing_frames > 0 {
            walk_frames * facings
        } else {
            walk_start
        };
        // spec: S-0102 claim 22 (facing part; quirks.sprite_vehicle_eight_facings_only)
        let d = sprite_vehicle_facing_part(facings, m.body, &ViewQuirks::default());
        // spec: S-0102 Import mapping (IdleRate absent: 0)
        let idle_rate = rule_int(self.import, k, i, b"IdleRate", 0);
        // spec: S-0102 claim 22 (frame table: the walk counter w is 0 and nothing fires or dies; the
        // standing-pose state is on from creation until a movement step starts and again once it
        // ends, so the row d never applies to a vehicle that is not part-way through a step)
        let frame = if idle_rate != 0 {
            walk_start + d * walk_frames
        } else if standing_frames > 0 {
            stand_start + d * standing_frames
        } else {
            walk_start + d * walk_frames
        };
        if self.live {
            self.note(notes::LIVE_SPRITE_VEHICLE);
        }
        let Ok(frame) = usize::try_from(frame) else {
            return Vec::new();
        };
        let lz = height_lift(m.pos.2);
        // spec: S-0102 claim 21 (no own shadow with the no-shadow flag or either visceroid flag)
        let shadowed = !rule_bool(self.import, k, i, b"NoShadow", false)
            && !rule_bool(self.import, k, i, b"SmallVisceroid", false)
            && !rule_bool(self.import, k, i, b"LargeVisceroid", false);
        let shadow_item = |b: &mut Self, bias: i32| {
            if !shadowed {
                return None;
            }
            // spec: S-0102 claim 21 (the shadow is the body frame + ⌊N/2⌋, centred on the same point)
            b.place_sprite(
                sprite,
                frame + n / 2,
                point,
                DepthTest::Reader,
                ItemDepth::Profile {
                    profile: Profile::Flat,
                    bias,
                    companion: None,
                },
                Paint::Shadow,
                Group::Shadows,
                false,
                Some(m.object),
            )
        };
        let body = |b: &mut Self, f: usize| {
            // spec: S-0102 claim 13 (a vehicle drawn as a sprite, h = 0: U − 2, upright, reader)
            b.place_sprite(
                sprite,
                f,
                point,
                DepthTest::Reader,
                ItemDepth::Profile {
                    profile: Profile::Upright,
                    bias: u - 2,
                    companion: None,
                },
                Paint::Lit(row),
                Group::Vehicles,
                false,
                Some(m.object),
            )
        };
        let mut items = Vec::new();
        if turret {
            // spec: S-0102 claim 22 (a turret vehicle: body and turret frame ((t₃₂ + 4) mod 32) + 8·F_w
            // combined into one image; its shadow, U − 2, never darkens them, so it goes first)
            items.extend(shadow_item(self, u - 2));
            items.extend(body(self, frame));
            let t = (n_step(m.turret, 32) as i32 + 4) % 32 + 8 * walk_frames;
            if let Ok(t) = usize::try_from(t) {
                items.extend(body(self, t));
            }
        } else {
            // spec: S-0102 claims 21 and 28 (the own shadow right after the body: −L(z) − 4, flat)
            items.extend(body(self, frame));
            items.extend(shadow_item(self, -lz - 4));
        }
        items
    }

    /// A parsed voxel model and its motion file from the assets, parsed
    /// once.
    fn voxel_model(&mut self, file: &[u8]) -> Option<std::rc::Rc<voxel::VoxelModel>> {
        let key = upper(file);
        if let Some(m) = self.p.models.get(&key) {
            return m.clone();
        }
        let m = Vxl::parse(self.assets.get(file)?).ok().map(|vxl| {
            let hva = self
                .assets
                .get(&hva_name(file))
                .and_then(|b| Hva::parse(b).ok());
            std::rc::Rc::new(voxel::VoxelModel::new(
                &vxl,
                hva.as_ref(),
                &VoxelQuirks::default(),
            ))
        });
        self.p.models.insert(key, m.clone());
        m
    }

    /// The voxel lighting table (S-0105 claims 19 and 38; its byte layout
    /// is S-0013's). A missing or too short file gives the
    /// empty table, whose rows all read as zero (claim 38,
    /// `quirks.vpl_unloaded_rows_blank`).
    fn vpl(&mut self) -> Option<Vpl> {
        let assets = self.assets;
        let v = self
            .p
            .vpl
            .get_or_insert_with(|| assets.get(VPL_FILE).and_then(|b| Vpl::parse(b).ok()))
            .clone();
        let blank = VoxelQuirks::default().vpl_unloaded_rows_blank;
        let v = match v {
            Some(v) => {
                if v.stated_rows < glasnost_fmt::vpl::USUAL_ROWS {
                    // spec: S-0013 Import mapping (R below 32: undeclared rows zero, warned)
                    self.note(notes::FEW_VPL_ROWS);
                }
                if v.trailing {
                    // spec: S-0013 claim 5 (trailing bytes warned, ignored)
                    self.note(notes::TRAILING_VPL);
                }
                if v.short {
                    self.note(if blank {
                        notes::SHORT_VPL
                    } else {
                        notes::VPL_REJECTED
                    });
                } else if v.stated_rows > glasnost_fmt::vpl::MAX_ROWS {
                    // spec: S-0105 claim 38 (more than 128 rows: a departure, the first 128 kept)
                    self.note(notes::SHORT_VPL);
                }
                v
            }
            None => {
                self.note(if blank {
                    notes::NO_VPL
                } else {
                    notes::VPL_REJECTED
                });
                Vpl::empty()
            }
        };
        Some(v)
    }

    /// Adds a voxel part image to the scene: its index and offset from the
    /// draw point (S-0105 claim 25, F6).
    fn add_part(&mut self, img: &voxel::PartImage) -> Option<(u32, [i32; 2])> {
        add_part_image(&mut self.built.scene, img)
    }

    /// A voxel part image placed at `point` (S-0105 claim 25, F6; S-0102 F2:
    /// the drawing rectangle is the copy rectangle).
    fn part_item(
        part: (u32, [i32; 2]),
        point: (i32, i32),
        bias: i32,
        paint: Paint,
        group: Group,
        object: u32,
    ) -> Item {
        Item {
            image: part.0,
            x: point.0 + part.1[0],
            y: point.1 + part.1[1],
            test: DepthTest::Reader,
            depth: ItemDepth::Profile {
                profile: Profile::Upright,
                bias,
                companion: None,
            },
            paint,
            group,
            provisional: false,
            object: Some(object),
        }
    }

    /// A voxel part or shadow image of type `(kind, idx)` from the type's
    /// image cache, or drawn fresh by `draw` and kept (S-0105 claims 26,
    /// 31). The cache-disable options are not honoured: the scene keeps
    /// every image added, so a type drawn fresh in every drawn frame would
    /// add images without end.
    fn cached_part(
        &mut self,
        kind: Kind,
        idx: usize,
        key: voxel::ImageKey,
        draw: impl FnOnce(&mut Self) -> Option<voxel::PartImage>,
    ) -> Option<(u32, [i32; 2])> {
        if let Some(p) = self.p.voxels.get(&(kind, idx)).and_then(|c| c.get(&key)) {
            return *p;
        }
        let part = draw(self).and_then(|img| self.add_part(&img));
        self.p
            .voxels
            .entry((kind, idx))
            .or_default()
            .insert(key, part);
        part
    }

    /// A vehicle's or aircraft's voxel body, turret, barrel and shadow
    /// (S-0105 claims 3 to 11, 16, 29 to 31; S-0102 claims 13, 18).
    fn vehicle_voxels(
        &mut self,
        m: &Mobile,
        art: &glasnost_convert::art::TypeArt,
        point: (i32, i32),
        u: i32,
        row: Shade,
        group: Group,
    ) -> Vec<Item> {
        let q = VoxelQuirks::default();
        let (kind, idx) = (m.kind, m.idx);
        let Some(body_file) = art.voxels.get("body").cloned() else {
            return Vec::new();
        };
        let art_int = |b: &Self, key: &[u8], d: i32| {
            b.art_value(kind, idx, key)
                .and_then(value::integer)
                .unwrap_or(d)
        };
        // spec: S-0105 Import mapping (TurretOffset, Art [image], default 0)
        let offset = art_int(self, b"TurretOffset", 0);
        let name = self.import.outcome.registries[&kind][idx].name.clone();
        // spec: S-0105 Import mapping (UseBuffer is read from the art section of the type's own name)
        let depth_buffered = self
            .p
            .art
            .value(&name, b"UseBuffer")
            .and_then(value::boolean)
            .unwrap_or(false);
        let (body_raw, turret_raw, elevation) = (m.body, m.turret, m.elevation);
        let pose = move || voxel::vehicle_parts(body_raw, turret_raw, offset, elevation, &q);
        let object = m.object;
        let mut items = Vec::new();
        // spec: S-0105 claims 29 to 31 (the main body's shadow section in frame 0; skipped with the no-shadow option)
        let shadow = (!rule_bool(self.import, kind, idx, b"NoShadow", false)).then(|| {
            let section = art_int(self, b"ShadowIndex", 0).max(0) as usize;
            let aircraft_like = rule_bool(self.import, kind, idx, b"ConsideredAircraft", false);
            (section, aircraft_like)
        });
        // spec: S-0105 claim 31 (shadow key: ramp, 32-step body facing; bodies stand on flat ground)
        let shadow_key = voxel::shadow_key(voxel::RAMP_FLAT, body_raw);
        let shadow_file = body_file.clone();
        let draw_shadow = move |b: &mut Self, (section, aircraft_like): (usize, bool)| {
            let body = b.voxel_model(&shadow_file)?;
            // spec: S-0105 claim 32 (a shadow never enables depth mode, whatever the type's option)
            b.p.canvas
                .draw_shadow(&body, section, &pose().shadow, 0, aircraft_like, &q)
        };
        // An aircraft's shadow comes before the body, which covers it.
        if kind != Kind::Vehicles
            && let Some(s) = shadow
            && let Some(part) = self.cached_part(kind, idx, shadow_key, |b| draw_shadow(b, s))
        {
            // spec: S-0102 claim 18 (the voxel shadow: bias U, upright, a reader, in the object's
            // own drawing; it never paints over its own aircraft)
            items.push(Self::part_item(
                part,
                point,
                u,
                Paint::Shadow,
                Group::Shadows,
                object,
            ));
            self.note(notes::VOXEL_SHADOW_POINT);
        }
        let mut placed = Vec::new();
        // spec: S-0105 claim 16 (walk counter 0: body frame 0; the simulation keeps no walk counter yet)
        let body_frame = 0;
        // spec: S-0105 claim 26 (body key; flat ground, the alternative-model state off: claim 12's
        // alternative body is not drawn, the main body always is)
        let key = voxel::body_key(voxel::RAMP_FLAT, body_raw, false, body_frame);
        if let Some(part) = self.cached_part(kind, idx, key, |b| {
            let body = b.voxel_model(&body_file)?;
            let vpl = b.vpl();
            b.p.canvas.draw_part(
                &body,
                body_frame,
                &pose().body,
                vpl.as_ref(),
                depth_buffered,
                &q,
            )
        }) {
            placed.push(part);
            // spec: S-0102 claim 13 (a voxel vehicle: bias U, upright, a reader)
            items.push(Self::part_item(
                part,
                point,
                u,
                Paint::Lit(row),
                group,
                object,
            ));
        }
        // spec: S-0105 claims 4, 8 (a turret part only with the turret flag)
        if rule_bool(self.import, kind, idx, b"Turret", false) {
            // spec: S-0105 claim 16 (body frame 0 and an animation-frame counter of 0 before any attack: turret frame 0)
            let turret_frame = voxel::vehicle_turret_frame(body_frame, 0, None);
            // spec: S-0105 claim 16 (quirks.barrel_frame_zero)
            let barrel_frame = voxel::barrel_frame_with_turret(turret_frame, &q);
            // spec: S-0105 claim 26 (turret key; flat ground, no multi-turret models, the alternative-model state off)
            let turret_key = voxel::vehicle_turret_key(
                voxel::RAMP_FLAT,
                body_raw,
                turret_raw,
                offset,
                None,
                false,
                turret_frame,
            );
            let mut parts = vec![
                (false, "turret", voxel::ImageKey::Turret(turret_key)),
                // spec: S-0105 claim 26 (barrel key: the turret's key plus the current 32-step elevation)
                (true, "barrel", voxel::barrel_key(turret_key, elevation)),
            ];
            // spec: S-0105 claim 11 (turret first when its 4-step facing is 1 or 2)
            if !voxel::turret_drawn_first(m.turret) {
                parts.reverse();
            }
            for (is_barrel, part_name, key) in parts {
                let Some(file) = art.voxels.get(part_name).cloned() else {
                    continue;
                };
                if let Some(part) = self.cached_part(kind, idx, key, |b| {
                    let model = b.voxel_model(&file)?;
                    let vpl = b.vpl();
                    let p = pose();
                    let (draw, frame) = if is_barrel {
                        (p.barrel, barrel_frame)
                    } else {
                        (p.turret, turret_frame)
                    };
                    b.p.canvas
                        .draw_part(&model, frame, &draw, vpl.as_ref(), depth_buffered, &q)
                }) {
                    placed.push(part);
                    items.push(Self::part_item(
                        part,
                        point,
                        u,
                        Paint::Lit(row),
                        group,
                        object,
                    ));
                }
            }
        }
        // spec: S-0105 claim 39 (a vehicle's shadow is made after its parts and excludes the
        // completed part image; it darkens the scene after the parts have contributed)
        if kind == Kind::Vehicles
            && let Some(s) = shadow
        {
            let cached = self
                .p
                .voxels
                .get(&(kind, idx))
                .is_some_and(|c| c.get(&shadow_key).is_some());
            let fresh = (!cached).then(|| draw_shadow(self, s));
            let part = vehicle_shadow(
                &mut self.built.scene,
                self.p.voxels.entry((kind, idx)).or_default(),
                &mut self.p.cutouts,
                shadow_key,
                &placed,
                fresh,
            );
            if let Some(part) = part {
                // spec: S-0102 claim 18 (the voxel shadow: bias U, upright, a reader, in the object's own drawing)
                items.push(Self::part_item(
                    part,
                    point,
                    u,
                    Paint::Shadow,
                    Group::Shadows,
                    object,
                ));
                self.note(notes::VOXEL_SHADOW_POINT);
            }
        }
        items
    }
}

/// Adds a voxel part or shadow image to `scene`: its index and offset from
/// the draw point (S-0105 claim 25, F6). An empty image adds nothing.
fn add_part_image(scene: &mut Scene, img: &voxel::PartImage) -> Option<PlacedPart> {
    let image = IndexedImage::new(img.width, img.height, img.pixels.clone()).ok()?;
    if image.width() == 0 || image.height() == 0 {
        return None;
    }
    let id = scene.add_image(image);
    Some((id, [img.offset[0] as i32, img.offset[1] as i32]))
}

/// A scene image placed as a voxel part, back as a part image.
fn placed_image(scene: &Scene, (id, offset): PlacedPart) -> voxel::PartImage {
    let image = &scene.images[id as usize];
    voxel::PartImage {
        width: image.width(),
        height: image.height(),
        pixels: image.pixels().to_vec(),
        offset: [i64::from(offset[0]), i64::from(offset[1])],
        copy_size: [i64::from(image.width()), i64::from(image.height())],
    }
}

/// A vehicle shadow's appearance (S-0105 claims 31, 39). `fresh` is the
/// newly drawn unmasked footprint on a cache miss (`None` when the type's
/// cache holds `key`); `parts` are this draw's completed body, turret and
/// barrel images.
///
/// The cache keeps the unmasked footprint, and every appearance, fresh or
/// reused, excludes the current parts (claim 39).
fn vehicle_shadow(
    scene: &mut Scene,
    cache: &mut voxel::ImageCache<Option<PlacedPart>>,
    cutouts: &mut BTreeMap<(u32, Vec<u32>), Option<PlacedPart>>,
    key: voxel::ImageKey,
    parts: &[PlacedPart],
    fresh: Option<Option<voxel::PartImage>>,
) -> Option<PlacedPart> {
    let unmasked = match fresh {
        Some(img) => {
            // spec: S-0105 claims 31, 39 (cache only the unmasked footprint)
            let part = img.and_then(|i| add_part_image(scene, &i));
            cache.insert(key, part);
            part
        }
        // spec: S-0105 claims 31, 39 (a reused footprint is not redrawn)
        None => cache.get(&key).copied().flatten(),
    }?;
    // spec: S-0105 claim 39 (exclude this object's current completed part image on every
    // appearance, fresh or reused; no old-pose cutouts persist)
    let ids = (unmasked.0, parts.iter().map(|p| p.0).collect());
    if let Some(p) = cutouts.get(&ids) {
        return *p;
    }
    let mut img = placed_image(scene, unmasked);
    let images: Vec<voxel::PartImage> = parts.iter().map(|&p| placed_image(scene, p)).collect();
    let refs: Vec<&voxel::PartImage> = images.iter().collect();
    voxel::exclude_own_image(&mut img, &refs);
    let part = add_part_image(scene, &img);
    cutouts.insert(ids, part);
    part
}

/// A type's fire angle (S-0105 claim 6, Import mapping: `FireAngle`,
/// default 8).
pub fn fire_angle(inputs: &Inputs, kind: Kind, idx: usize) -> i32 {
    rule_int(
        inputs.import,
        kind,
        idx,
        b"FireAngle",
        voxel::DEFAULT_FIRE_ANGLE,
    )
}

/// The sequence entry and stage an infantry unit is drawn with (S-0107
/// claims 1 to 4).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InfantryFrame {
    /// Start image frame *S*.
    pub start: i32,
    /// Frame count *C*.
    pub count: i32,
    /// Facing multiplier *M*.
    pub multiplier: i32,
    /// Stage *k*.
    pub stage: i32,
}

/// The displayed body image frame of an infantry unit facing `facing`
/// (S-0107 claim 2, F1, F2).
pub fn infantry_frame(f: &InfantryFrame, facing: u16) -> i32 {
    // spec: S-0107 F2 (g = (7 − ⌊(φ + 5120) / 8192⌋) mod 8)
    let g = (7 - (i32::from(facing) + 5120) / 8192).rem_euclid(8);
    // spec: S-0107 F1 (frame = S + (k mod max(C, 1)) + [M > 0]·M·g)
    f.start
        + f.stage.max(0).rem_euclid(f.count.max(1))
        + if f.multiplier > 0 {
            f.multiplier * g
        } else {
            0
        }
}

/// A mobile object to draw: its type, owner, position and pose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mobile {
    /// Kind: vehicles, infantry or aircraft.
    pub kind: Kind,
    /// Registration index in the import.
    pub idx: usize,
    /// Owner's house name (for the house colour, S-0103 claim 10).
    pub owner: Vec<u8>,
    /// Position in leptons (x, y, z; S-0204 claim 1).
    pub pos: (i32, i32, i32),
    /// Body facing (S-0206).
    pub body: u16,
    /// Turret facing (S-0206).
    pub turret: u16,
    /// Raw barrel elevation (S-0105 claim 6).
    pub elevation: u16,
    /// For infantry: the entry and stage to show.
    pub infantry: Option<InfantryFrame>,
    /// The object code its items carry ([`Item::object`]).
    pub object: u32,
}

/// The inputs a scene is drawn from.
#[derive(Clone, Copy)]
pub struct Inputs<'a> {
    /// The decoded map.
    pub map: &'a Map,
    /// The rules import.
    pub import: &'a Import,
    /// The theater's catalogue.
    pub catalogue: &'a Catalogue,
    /// The palettes in force.
    pub palettes: &'a PaletteState,
    /// Files read for the scene.
    pub assets: &'a Assets,
    /// Viewer options.
    pub options: &'a Options,
    /// The typed rules of the game, when the ruleset resolved: the strengths
    /// and condition ratios a pre-placed building's starting health and
    /// damaged state need (S-0009 claim 37, S-0108 F3).
    pub rules: Option<&'a SimRules>,
    /// The map's cells after loading (S-0009, S-0010 claim 12: the
    /// recalculation passes and smoothing), when a game was set up; the
    /// cells as the map stores them otherwise.
    pub cells: Option<&'a CellGrid>,
}

impl Painter {
    fn new(art: Document, canvas: voxel::VoxelCanvas) -> Self {
        Self {
            art,
            lights: Lights {
                side: 0,
                cells: Vec::new(),
            },
            rows: Rows::default(),
            images: BTreeMap::new(),
            voxels: BTreeMap::new(),
            cutouts: BTreeMap::new(),
            canvas,
            models: BTreeMap::new(),
            vpl: None,
            tile_files: BTreeMap::new(),
        }
    }

    /// The voxel buffer, for the next game of the program (S-0105 claim
    /// 34, S2).
    pub fn canvas(&self) -> &voxel::VoxelCanvas {
        &self.canvas
    }

    /// An integer of a type's art section (the section of its image name),
    /// read as the importer's integer form.
    pub fn art_int(&self, import: &Import, kind: Kind, idx: usize, key: &[u8]) -> Option<i32> {
        let image = match import.arts.get(&(kind, idx)) {
            Some(a) => a.image.clone(),
            None => import.outcome.registries.get(&kind)?.get(idx)?.name.clone(),
        };
        self.art.value(&image, key).and_then(value::integer)
    }

    /// Draws `mobiles` into `scene` (adding the images and palette rows they
    /// need, reusing those added before) and returns their ground-layer
    /// entries in the same order, with the notes they met.
    pub fn mobiles(
        &mut self,
        inputs: &Inputs,
        scene: &mut Scene,
        mobiles: &[Mobile],
    ) -> (Vec<GroundEntry>, BTreeSet<String>) {
        let mut b = Builder {
            map: inputs.map,
            rules: inputs.rules,
            cells_loaded: inputs.cells,
            import: inputs.import,
            catalogue: inputs.catalogue,
            palettes: inputs.palettes,
            assets: inputs.assets,
            options: inputs.options,
            p: std::mem::replace(
                self,
                Painter::new(
                    Document::empty(Default::default()),
                    voxel::VoxelCanvas::new(),
                ),
            ),
            built: Built {
                scene: std::mem::take(scene),
                ..Default::default()
            },
            live: true,
        };
        let entries = mobiles.iter().map(|m| b.mobile(m)).collect();
        *self = b.p;
        *scene = b.built.scene;
        (entries, b.built.notes)
    }
    /// Draws the live game's own buildings into `scene`: for each, its
    /// items (body, shadow, bib, then any voxel turret or barrel).
    pub fn buildings(
        &mut self,
        inputs: &Inputs,
        scene: &mut Scene,
        list: &[LiveBuilding],
    ) -> (Vec<Vec<Item>>, BTreeSet<String>) {
        let mut b = Builder {
            map: inputs.map,
            rules: inputs.rules,
            cells_loaded: inputs.cells,
            import: inputs.import,
            catalogue: inputs.catalogue,
            palettes: inputs.palettes,
            assets: inputs.assets,
            options: inputs.options,
            p: std::mem::replace(
                self,
                Painter::new(
                    Document::empty(Default::default()),
                    voxel::VoxelCanvas::new(),
                ),
            ),
            built: Built {
                scene: std::mem::take(scene),
                ..Default::default()
            },
            live: true,
        };
        let mut out = Vec::new();
        for d in list {
            let Some(art) = b.import.arts.get(&(Kind::Buildings, d.idx)).cloned() else {
                out.push(Vec::new());
                continue;
            };
            let level = inputs
                .cells
                .and_then(|g| g.get(d.cell))
                .or_else(|| inputs.map.cells.get(d.cell))
                .map_or(0, |c| c.level);
            let condition = match inputs.rules {
                Some(r) => condition_of(r, d.health.0, d.health.1),
                None => (false, false),
            };
            let mut ground = Vec::new();
            let mut items = b.building_items(
                d.idx,
                &art,
                d.cell,
                level,
                &d.owner,
                condition,
                d.facing,
                d.object,
                &mut ground,
            );
            for g in ground {
                items.extend(g.items);
            }
            // Our placeholder order for a building the game placed: its slot
            // animations of every layer follow its other items.
            for (_, g) in std::mem::take(&mut b.built.layered) {
                items.extend(g.items);
            }
            out.push(items);
        }
        *self = b.p;
        *scene = b.built.scene;
        (out, b.built.notes)
    }
}

/// Whether a building with health `h` of strength `strength` is damaged
/// and badly damaged (S-0108 F3).
fn condition_of(r: &SimRules, h: i32, strength: i32) -> (bool, bool) {
    if strength == 0 {
        return (false, false);
    }
    // spec: S-0108 F3 (h = health / strength in binary64, compared with the ratios)
    let ratio =
        F64::from_i64(i64::from(h), r.mode).div(F64::from_i64(i64::from(strength), r.mode), r.mode);
    (
        ratio.le(r.general.condition_yellow),
        ratio.le(r.general.condition_red),
    )
}

/// A building the live game put on the map (deployed or placed), not one
/// of the map's own.
#[derive(Clone, Debug)]
pub struct LiveBuilding {
    /// Import registration index of its type.
    pub idx: usize,
    /// Top-left foundation cell.
    pub cell: CellXY,
    /// Owner house name.
    pub owner: Vec<u8>,
    /// Health and strength.
    pub health: (i32, i32),
    /// Turret facing.
    pub facing: u16,
    /// Item object code.
    pub object: u32,
}

/// A greyscale-free minimap: each cell as two pixels with its tile radar
/// colour (`colours`, from [`tile_radar_colours`]), cells outside the
/// usable area of S-0009 claim 4 dimmed. Size 2W × 2H. Our own viewer
/// design.
pub fn minimap(map: &Map, colours: &BTreeMap<CellXY, [u8; 3]>) -> (u32, u32, Vec<u8>) {
    let f = map.footprint;
    // Cells exist only at coordinates 0 … 511 (S-0009 claim 2), so a legal
    // cell's d and s fit a minimap of at most 1024 × 1024 pixels.
    let (w, h) = (
        f.width.clamp(0, 512) as u32 * 2,
        f.height.clamp(0, 512) as u32 * 2,
    );
    let mut px = vec![0u8; (w * h * 4) as usize];
    for (c, cell) in map.cells.iter() {
        let (colour, usable) = (
            colours.get(&c).copied().unwrap_or([0, 0, 0]),
            // spec: S-0009 claim 5a (the height-aware test limits the minimap; slopes need the theater spec)
            f.is_usable(c, i32::from(cell.level), false),
        );
        let x = c.d() + i64::from(f.width.min(512)) - 1;
        let y = c.s() - i64::from(f.width) - 1;
        // spec: S-0010 claim 20 (the two stored colours do not give the cell's two points different colours)
        for (dx, col) in [(0, colour), (1, colour)] {
            let (px_x, px_y) = (x + dx, y);
            if px_x < 0 || px_y < 0 || px_x >= i64::from(w) || px_y >= i64::from(h) {
                continue;
            }
            let at = ((px_y as u32 * w + px_x as u32) * 4) as usize;
            let dim = |v: u8| if usable { v } else { v / 3 };
            px[at..at + 4].copy_from_slice(&[dim(col[0]), dim(col[1]), dim(col[2]), 255]);
        }
    }
    (w, h, px)
}

/// Each drawn cell's tile radar colour: the first tile radar colour of the
/// tile cell its shown image holds (S-0010 claim 20; the second colour has
/// no effect on the radar), for the cells whose tile file was found.
pub fn tile_radar_colours(
    shown: &BTreeMap<CellXY, (Vec<u8>, usize)>,
    assets: &Assets,
) -> BTreeMap<CellXY, [u8; 3]> {
    let mut files: BTreeMap<&[u8], Option<Tmp>> = BTreeMap::new();
    let mut out = BTreeMap::new();
    for (&c, (file, index)) in shown {
        let tmp = files
            .entry(file.as_slice())
            .or_insert_with(|| assets.get(file).and_then(|b| Tmp::parse(b).ok()));
        if let Some(tile) = tmp
            .as_ref()
            .and_then(|t| t.tiles.get(*index).copied().flatten())
        {
            out.insert(c, tile.radar_left);
        }
    }
    out
}

#[cfg(test)]
mod shadow_tests {
    use super::*;

    /// A synthetic part image: `w × h` at `offset`, pixels `px`.
    fn img(w: u32, h: u32, offset: [i64; 2], px: &[u8]) -> voxel::PartImage {
        voxel::PartImage {
            width: w,
            height: h,
            pixels: px.to_vec(),
            offset,
            copy_size: [i64::from(w), i64::from(h)],
        }
    }

    /// A 4 × 1 footprint at (0, 0), every pixel marked.
    fn footprint() -> voxel::PartImage {
        img(4, 1, [0, 0], &[voxel::SHADOW_MARK; 4])
    }

    fn pixels(scene: &Scene, p: Option<PlacedPart>) -> Vec<u8> {
        scene.images[p.unwrap().0 as usize].pixels().to_vec()
    }

    /// spec: S-0105 claims 31, 39 (the cache keeps the unmasked footprint; the current
    /// completed part image is excluded on every appearance, fresh or reused, so no old-pose
    /// cutout persists; lead-confirmed departure), S-0106 claim 11 (every vehicle shadow excludes
    /// the current own-part image)
    #[test]
    fn cutouts_follow_the_current_parts() {
        let mut scene = Scene::default();
        let mut cache = voxel::ImageCache::default();
        let mut cutouts = BTreeMap::new();
        let key = voxel::shadow_key(voxel::RAMP_FLAT, 0);
        let a = add_part_image(&mut scene, &img(1, 1, [1, 0], &[9])).unwrap();
        let b = add_part_image(&mut scene, &img(2, 1, [2, 0], &[0, 9])).unwrap();
        let m = voxel::SHADOW_MARK;
        let first = vehicle_shadow(
            &mut scene,
            &mut cache,
            &mut cutouts,
            key,
            &[a],
            Some(Some(footprint())),
        );
        assert_eq!(pixels(&scene, first), [m, 0, m, m]);
        // The cache holds the unmasked footprint.
        assert_eq!(pixels(&scene, *cache.get(&key).unwrap()), [m; 4]);
        let second = vehicle_shadow(&mut scene, &mut cache, &mut cutouts, key, &[b], None);
        // B's transparent pixel at x = 2 excludes nothing; its pixel at x = 3 does.
        assert_eq!(pixels(&scene, second), [m, m, m, 0]);
        // The same parts again reuse the masked image: the scene does not grow.
        let n = scene.images.len();
        let third = vehicle_shadow(&mut scene, &mut cache, &mut cutouts, key, &[b], None);
        // Back to A's pose: A's cutout again, not B's.
        let fourth = vehicle_shadow(&mut scene, &mut cache, &mut cutouts, key, &[a], None);
        assert_eq!(fourth, first);
        assert_eq!((third, scene.images.len()), (second, n));
    }
}
